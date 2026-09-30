// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Publishing dependent cell flags while inputs and outputs change
//! concurrently: the conflict rules, the `Skip` policy and the report.
//!
//! Every test stages a refresh at one version and commits the competing
//! transactions in a chosen order. `published_values` shows the rows whose
//! flag is true; `column_values` without a flag is the masked read, where the
//! stale values `Skip` installs under a false flag read as NULL.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use arrow_array::cast::AsArray;
use arrow_array::types::{Int32Type, UInt64Type};
use arrow_array::{
    Array, ArrayRef, BinaryArray, Int32Array, Int64Array, LargeBinaryArray, LargeStringArray,
    RecordBatch, RecordBatchIterator, StringArray, UInt64Array, record_batch,
};
use arrow_buffer::BooleanBuffer;
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use futures::{StreamExt, TryStreamExt, stream};
use lance_arrow::json::ARROW_JSON_EXT_NAME;
use lance_arrow::{ARROW_EXT_NAME_KEY, BLOB_META_KEY};
use lance_core::datatypes::Schema as LanceSchema;
use lance_core::utils::address::RowAddress;
use lance_core::utils::tempfile::TempStrDir;
use lance_core::{Error, ROW_ADDR, ROW_ID};
use lance_file::version::LanceFileVersion;
use lance_io::object_store::ObjectStore;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::format::{IndexMetadata, Manifest, Transaction as TableTransaction};
use lance_table::io::commit::{
    CommitError, CommitHandler, ManifestLocation, ManifestNamingScheme, ManifestWriter,
};
use object_store::path::Path;
use roaring::RoaringBitmap;
use rstest::rstest;

use super::dataset_cell_flags::{
    articles, cleared, commit_replacement, full, recorded_invalidations, register_ready,
    replacement_txn, set_true, stage_all, update_config,
};
use crate::dataset::cell_flag::{
    CellFlagOptions, ComputedBatch, DeferralReason, DeferredGroup, DeferredRows,
    DependencyConflictPolicy, FollowUpRows, PublicationReport, PublicationResult,
    PublicationStager,
};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{
    CellFlagChanges, CellFlagRegistration, CellFlagUpdate, DataReplacementGroup, Operation,
    Transaction, TransactionBuilder,
};
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::write::{CommitBuilder, WriteParams};
use crate::dataset::{DeleteBuilder, MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder};
use crate::{Dataset, Result};

use DeferralReason::{
    FragmentRemoved, InputChanged, NewerResult, RowVacated, UpstreamNotPublished,
};
use DependencyConflictPolicy::{Reject, Skip};

fn rows(entries: &[(u32, &[u32])]) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for (fragment, offsets) in entries {
        rows.insert_bitmap(*fragment, RoaringBitmap::from_iter(offsets.iter().copied()));
    }
    rows
}

fn with_full(mut rows: RowAddrTreeMap, fragment: u32) -> RowAddrTreeMap {
    rows.insert_fragment(fragment);
    rows
}

fn values(entries: &[(i32, Option<&str>)]) -> Vec<(i32, Option<String>)> {
    entries
        .iter()
        .map(|(id, value)| (*id, value.map(str::to_string)))
        .collect()
}

fn deferred(
    flag_id: u32,
    rows: RowAddrTreeMap,
    reason: DeferralReason,
    conflicting_version: u64,
) -> DeferredRows {
    DeferredRows {
        flag_id,
        rows,
        reason,
        conflicting_version,
    }
}

fn published(flag_id: u32, rows: RowAddrTreeMap) -> CellFlagUpdate {
    CellFlagUpdate {
        flag_id,
        value: true,
        rows,
    }
}

fn flag_label(dataset: &Dataset, flag_id: u32) -> String {
    let summary = dataset.schema().field("summary").unwrap().id;
    format!("cell flag 'ready' (flag id {flag_id}) on 'summary' (field id {summary})")
}

/// Stage `columns` of `fragment_id` with `value(column, offset)` per physical row.
async fn stage_rows(
    dataset: &Dataset,
    fragment_id: u64,
    columns: &[&str],
    value: impl Fn(&str, u64) -> Option<String>,
) -> DataReplacementGroup {
    let fragment = dataset.get_fragment(fragment_id as usize).unwrap();
    let physical_rows = fragment.physical_rows().await.unwrap() as u64;
    let arrow_fields: Vec<ArrowField> = columns
        .iter()
        .map(|column| ArrowField::new(*column, DataType::Utf8, true))
        .collect();
    let arrays: Vec<ArrayRef> = columns
        .iter()
        .map(|column| {
            let column_values: Vec<Option<String>> = (0..physical_rows)
                .map(|offset| value(column, offset))
                .collect();
            Arc::new(StringArray::from(column_values)) as ArrayRef
        })
        .collect();
    let batch = RecordBatch::try_new(Arc::new(ArrowSchema::new(arrow_fields)), arrays).unwrap();
    let schema = LanceSchema {
        fields: columns
            .iter()
            .map(|column| dataset.schema().field(column).unwrap().clone())
            .collect(),
        metadata: Default::default(),
    };
    fragment
        .write_columns(stream::iter([Ok(batch)]), &schema)
        .await
        .unwrap()
}

/// Commit the refresh `groups`/`updates` staged against `read` under `policy`.
async fn publish(
    read: &Dataset,
    groups: Vec<DataReplacementGroup>,
    updates: Vec<CellFlagUpdate>,
    policy: DependencyConflictPolicy,
) -> Result<PublicationResult> {
    CommitBuilder::new(Arc::new(read.clone()))
        .with_dependency_conflict_policy(policy)
        .execute_with_report(replacement_txn(read.manifest.version, groups, updates))
        .await
}

async fn latest(dataset: &Dataset) -> Dataset {
    let mut latest = dataset.clone();
    latest.checkout_latest().await.unwrap();
    latest
}

/// `(id, value)` of `column` sorted by id, for the rows where `only_true` is
/// true when given.
async fn column_values(
    dataset: &Dataset,
    column: &str,
    only_true: Option<u32>,
) -> Vec<(i32, Option<String>)> {
    let true_rows = only_true.map(|flag_id| dataset.cell_flag_true_rows(flag_id).unwrap());
    let batch = dataset
        .scan()
        .project(&["id", column])
        .unwrap()
        .with_row_address()
        .try_into_batch()
        .await
        .unwrap();
    let ids = batch["id"].as_primitive::<Int32Type>();
    let column_values = batch[column].as_string::<i32>();
    let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
    let mut result: Vec<(i32, Option<String>)> = (0..batch.num_rows())
        .filter(|row| {
            true_rows
                .as_ref()
                .is_none_or(|true_rows| true_rows.contains(addrs.value(*row)))
        })
        .map(|row| {
            let value = column_values
                .is_valid(row)
                .then(|| column_values.value(row).to_string());
            (ids.value(row), value)
        })
        .collect();
    result.sort();
    result
}

/// What a masked read of `column` shows where `flag_id` is true.
async fn published_values(
    dataset: &Dataset,
    column: &str,
    flag_id: u32,
) -> Vec<(i32, Option<String>)> {
    column_values(dataset, column, Some(flag_id)).await
}

/// Path of the data file holding `column` in fragment `fragment_id`.
fn file_of(dataset: &Dataset, fragment_id: u64, column: &str) -> String {
    let field_id = dataset.schema().field(column).unwrap().id;
    dataset
        .get_fragment(fragment_id as usize)
        .unwrap()
        .metadata()
        .files
        .iter()
        .find(|file| file.fields.contains(&field_id))
        .unwrap()
        .path
        .clone()
}

async fn merge_insert_body(dataset: &Dataset, id: i32, body: &str) -> Dataset {
    merge_insert_column(dataset, id, "body", body).await
}

/// Rewrite `column` of `id` in place with a partial-schema merge_insert.
async fn merge_insert_column(dataset: &Dataset, id: i32, column: &str, value: &str) -> Dataset {
    let source = RecordBatch::try_from_iter([
        ("id", Arc::new(Int32Array::from(vec![id])) as ArrayRef),
        (column, Arc::new(StringArray::from(vec![value])) as ArrayRef),
    ])
    .unwrap();
    let (dataset, _) = MergeInsertBuilder::try_new(Arc::new(dataset.clone()), vec!["id".into()])
        .unwrap()
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::DoNothing)
        .write_mode(MergeInsertWriteMode::RewriteColumns)
        .try_build()
        .unwrap()
        .execute_batches(vec![source])
        .await
        .unwrap();
    dataset.as_ref().clone()
}

async fn update_where(dataset: &Dataset, predicate: &str, column: &str, value: &str) -> Dataset {
    UpdateBuilder::new(Arc::new(dataset.clone()))
        .update_where(predicate)
        .unwrap()
        .set(column, &format!("'{value}'"))
        .unwrap()
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap()
        .new_dataset
        .as_ref()
        .clone()
}

/// A write of `body` on fragment 0, which holds ids 1 and 2 at offsets 0, 1.
#[derive(Debug, Clone, Copy)]
enum BodyWrite {
    /// A `DataReplacement` of body: every row of fragment 0, in place.
    Replace,
    /// A partial-schema merge_insert of id 2, in place.
    MergeInsert,
    /// An `UpdateBuilder` update of id 2, which moves it to a new fragment.
    Update,
}

impl BodyWrite {
    async fn apply(self, dataset: &Dataset) -> Dataset {
        match self {
            Self::Replace => {
                let body = stage_rows(dataset, 0, &["body"], |_, offset| {
                    Some(format!("new-{offset}"))
                })
                .await;
                commit_replacement(dataset, vec![body], vec![])
                    .await
                    .unwrap()
            }
            Self::MergeInsert => merge_insert_body(dataset, 2, "new").await,
            Self::Update => update_where(dataset, "id = 2", "body", "new").await,
        }
    }

    /// Write `body` as id 2's body; a replacement also rewrites id 1's body
    /// with its original value.
    async fn write_body_of_id_2(self, dataset: &Dataset, body: &str) -> Dataset {
        match self {
            Self::Replace => {
                let file = stage_rows(dataset, 0, &["body"], |_, offset| {
                    Some(if offset == 1 {
                        body.to_string()
                    } else {
                        "b1".into()
                    })
                })
                .await;
                commit_replacement(dataset, vec![file], vec![])
                    .await
                    .unwrap()
            }
            Self::MergeInsert => merge_insert_body(dataset, 2, body).await,
            Self::Update => update_where(dataset, "id = 2", "body", body).await,
        }
    }

    /// The clears the write records, as its transaction file holds them.
    fn clears(self) -> RowAddrTreeMap {
        match self {
            Self::Replace => full(&[0]),
            Self::MergeInsert => rows(&[(0, &[1])]),
            Self::Update => RowAddrTreeMap::new(),
        }
    }

    /// The rows of fragment 0 a refresh staged before the write loses.
    fn stale(self) -> RowAddrTreeMap {
        match self {
            Self::Replace => rows(&[(0, &[0, 1])]),
            Self::MergeInsert | Self::Update => rows(&[(0, &[1])]),
        }
    }

    fn reason(self) -> DeferralReason {
        match self {
            Self::Update => RowVacated,
            Self::Replace | Self::MergeInsert => InputChanged,
        }
    }

    /// A full refresh of both fragments less the stale rows.
    fn still_valid(self) -> RowAddrTreeMap {
        match self {
            Self::Replace => full(&[1]),
            Self::MergeInsert | Self::Update => with_full(rows(&[(0, &[0])]), 1),
        }
    }

    fn body_of_id_2(self) -> &'static str {
        match self {
            Self::Replace => "new-1",
            Self::MergeInsert | Self::Update => "new",
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_input_write_masks_published_output(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert, BodyWrite::Update)] write: BodyWrite,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read_version = dataset.version().version;
    let staged = stage_all(&dataset, "summary", "s").await;
    let result = publish(&dataset, staged, set_true(flag_id, full(&[0, 1])), Reject)
        .await
        .unwrap();
    assert_eq!(
        result.report,
        PublicationReport {
            read_version,
            checked_version: read_version,
            committed_version: Some(read_version + 1),
            published: vec![published(flag_id, full(&[0, 1]))],
            deferred_rows: vec![],
            deferred_groups: vec![],
        }
    );

    let written = write.apply(&result.dataset).await;
    assert_eq!(
        recorded_invalidations(&written).await,
        if write.clears().is_empty() {
            vec![]
        } else {
            cleared(flag_id, write.clears())
        }
    );
    // A row-moving update clears the moved row's old address, and its new
    // row starts unassigned because the update wrote a source.
    assert_eq!(
        written.cell_flag_true_rows(flag_id).unwrap(),
        write.still_valid()
    );
    let id_2_published = !matches!(write, BodyWrite::Replace);
    let mut expected = vec![(3, Some("s-1-0")), (4, Some("s-1-1"))];
    if id_2_published {
        expected.insert(0, (1, Some("s-0-0")));
    }
    assert_eq!(
        published_values(&written, "summary", flag_id).await,
        values(&expected),
        "id 2 no longer reads its summary"
    );
    assert_eq!(
        column_values(&written, "body", None).await[1],
        (2, Some(write.body_of_id_2().to_string()))
    );
}

#[rstest]
#[tokio::test]
async fn test_refresh_staged_before_input_write(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert, BodyWrite::Update)] write: BodyWrite,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let staged = stage_all(&dataset, "summary", "old").await;
    let dataset = publish(&dataset, staged, set_true(flag_id, full(&[0, 1])), Reject)
        .await
        .unwrap()
        .dataset;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "new").await;
    let written = write.apply(&dataset).await;
    let flags_after_write = written.cell_flag_true_rows(flag_id).unwrap();

    let result = publish(
        &read,
        refresh.clone(),
        set_true(flag_id, full(&[0, 1])),
        policy,
    )
    .await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        let expected = match write {
            BodyWrite::Update => "preempted by concurrent transaction Update".to_string(),
            BodyWrite::Replace | BodyWrite::MergeInsert => format!(
                "{} cannot be published on {} row(s) of fragment 0: concurrent {} at version {} \
                 changed their inputs",
                flag_label(&read, flag_id),
                write.stale().len().unwrap(),
                if matches!(write, BodyWrite::Replace) {
                    "DataReplacement"
                } else {
                    "Update"
                },
                written.version().version
            ),
        };
        assert!(error.to_string().contains(&expected), "{error}");
        let head = latest(&written).await;
        assert_eq!(head.version().version, written.version().version);
        assert_eq!(
            head.cell_flag_true_rows(flag_id).unwrap(),
            flags_after_write
        );
        assert_eq!(file_of(&head, 1, "summary"), file_of(&read, 1, "summary"));
        return;
    }

    let result = result.unwrap();
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: written.version().version,
            committed_version: Some(written.version().version + 1),
            published: vec![published(flag_id, write.still_valid())],
            deferred_rows: vec![deferred(
                flag_id,
                write.stale(),
                write.reason(),
                written.version().version
            )],
            deferred_groups: vec![],
        }
    );
    let committed = result.dataset;
    assert_eq!(
        committed.cell_flag_true_rows(flag_id).unwrap(),
        write.still_valid()
    );
    let mut expected = vec![(3, Some("new-1-0")), (4, Some("new-1-1"))];
    if !matches!(write, BodyWrite::Replace) {
        expected.insert(0, (1, Some("new-0-0")));
    }
    assert_eq!(
        published_values(&committed, "summary", flag_id).await,
        values(&expected)
    );
    // The stale values are installed with the group, under a false flag.
    assert_eq!(file_of(&committed, 0, "summary"), refresh[0].1.path);
    let mut masked = expected;
    masked.push((2, None));
    if matches!(write, BodyWrite::Replace) {
        masked.push((1, None));
    }
    masked.sort();
    assert_eq!(
        column_values(&committed, "summary", None).await,
        values(&masked),
        "the stale values read as NULL"
    );
    assert_eq!(
        column_values(&committed, "body", None).await[1],
        (2, Some(write.body_of_id_2().to_string()))
    );
}

/// A row-moving update records no clear: it moves the row, and the refresh's
/// assignment at the old address is vacated instead.
#[rstest]
#[tokio::test]
async fn test_refresh_skips_rows_invalidated_while_flag_false(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert, BodyWrite::Update)] write: BodyWrite,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let written = write.apply(&dataset).await;
    assert!(written.cell_flag_true_rows(flag_id).unwrap().is_empty());
    let changes = written
        .read_transaction()
        .await
        .unwrap()
        .unwrap()
        .cell_flag_changes
        .unwrap();
    if matches!(write, BodyWrite::Update) {
        let body = written.schema().field("body").unwrap().id;
        assert_eq!(changes.moved_rows_written_fields, Some(vec![body]));
        assert!(changes.derived_invalidations.is_empty());
    } else {
        assert_eq!(
            recorded_invalidations(&written).await,
            cleared(flag_id, write.clears()),
            "recorded although the flag was never true"
        );
    }

    let result = publish(&read, refresh, set_true(flag_id, full(&[0, 1])), policy).await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        let expected = match write {
            BodyWrite::Update => format!(
                "preempted by concurrent transaction Update at version {}",
                written.version().version
            ),
            BodyWrite::Replace | BodyWrite::MergeInsert => format!(
                "cannot be published on {} row(s) of fragment 0",
                write.stale().len().unwrap()
            ),
        };
        assert!(error.to_string().contains(&expected), "{error}");
        let head = latest(&written).await;
        assert_eq!(head.version().version, written.version().version);
        assert!(head.cell_flag_true_rows(flag_id).unwrap().is_empty());
        assert!(
            column_values(&head, "summary", None)
                .await
                .iter()
                .all(|(_, value)| value.is_none())
        );
        return;
    }
    let result = result.unwrap();
    assert_eq!(
        result.report.deferred_rows,
        vec![deferred(
            flag_id,
            write.stale(),
            write.reason(),
            written.version().version
        )]
    );
    assert_eq!(
        result.report.published,
        vec![published(flag_id, write.still_valid())]
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        write.still_valid()
    );
    let id_1 = (!matches!(write, BodyWrite::Replace)).then_some("s-0-0");
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[(1, id_1), (2, None), (3, Some("s-1-0")), (4, Some("s-1-1"))]),
        "id 2 is not published and its stale value reads as NULL"
    );
}

#[rstest]
#[tokio::test]
async fn test_input_restored_before_refresh_publishes(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert, BodyWrite::Update)] write: BodyWrite,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let changed = write.write_body_of_id_2(&dataset, "changed").await;
    let restored = write.write_body_of_id_2(&changed, "b2").await;
    assert_eq!(
        column_values(&restored, "body", None).await,
        column_values(&read, "body", None).await,
        "the input reads as it did when the refresh read it"
    );

    let result = publish(&read, refresh, set_true(flag_id, full(&[0, 1])), policy).await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains(&format!("at version {}", changed.version().version)),
            "{error}"
        );
        assert!(
            latest(&restored)
                .await
                .cell_flag_true_rows(flag_id)
                .unwrap()
                .is_empty()
        );
        return;
    }
    let result = result.unwrap();
    // Each write invalidated id 2; the first one is reported.
    assert_eq!(
        result.report.deferred_rows,
        vec![deferred(
            flag_id,
            write.stale(),
            write.reason(),
            changed.version().version
        )]
    );
    assert_eq!(result.report.checked_version, restored.version().version);
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        write.still_valid()
    );
    let id_1 = (!matches!(write, BodyWrite::Replace)).then_some("s-0-0");
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[(1, id_1), (2, None), (3, Some("s-1-0")), (4, Some("s-1-1"))])
    );
}

#[rstest]
#[tokio::test]
async fn test_newer_result_survives_older_refresh(
    #[values(false, true)] is_input_changed: bool,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let older_read = dataset.clone();
    let older = stage_all(&older_read, "summary", "older").await;
    if is_input_changed {
        dataset = merge_insert_body(&dataset, 2, "new").await;
    }
    let input_changed_at = dataset.version().version;
    let newer = stage_rows(&dataset, 0, &["summary"], |_, offset| {
        Some(format!("newer-0-{offset}"))
    })
    .await;
    let repaired = publish(&dataset, vec![newer], set_true(flag_id, full(&[0])), Reject)
        .await
        .unwrap()
        .dataset;

    let result = publish(
        &older_read,
        older.clone(),
        set_true(flag_id, full(&[0, 1])),
        policy,
    )
    .await;
    let newer_values = [(1, Some("newer-0-0")), (2, Some("newer-0-1"))];
    let head = match policy {
        Reject => {
            let error = result.unwrap_err();
            assert!(
                matches!(error, Error::RetryableCommitConflict { .. }),
                "{error}"
            );
            // The input change is checked first, being the earlier version.
            let expected = if is_input_changed {
                format!(
                    "{} cannot be published on 1 row(s) of fragment 0: concurrent Update at \
                     version {input_changed_at} changed their inputs",
                    flag_label(&older_read, flag_id)
                )
            } else {
                format!(
                    "preempted by concurrent transaction DataReplacement at version {}",
                    repaired.version().version
                )
            };
            assert!(error.to_string().contains(&expected), "{error}");
            let head = latest(&repaired).await;
            assert_eq!(head.version().version, repaired.version().version);
            assert_eq!(head.cell_flag_true_rows(flag_id).unwrap(), full(&[0]));
            assert_eq!(
                published_values(&head, "summary", flag_id).await,
                values(&newer_values)
            );
            return;
        }
        Skip => result.unwrap(),
    };
    // A stale row is reported even though its group was not installed, so
    // only id 1's older value stays reusable.
    let (deferred_rows, valid) = if is_input_changed {
        (
            vec![deferred(
                flag_id,
                rows(&[(0, &[1])]),
                InputChanged,
                input_changed_at,
            )],
            rows(&[(0, &[0])]),
        )
    } else {
        (vec![], rows(&[(0, &[0, 1])]))
    };
    assert_eq!(
        head.report,
        PublicationReport {
            read_version: older_read.version().version,
            checked_version: repaired.version().version,
            committed_version: Some(repaired.version().version + 1),
            published: vec![published(flag_id, full(&[1]))],
            deferred_rows,
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: older[0].1.clone(),
                reason: NewerResult,
                conflicting_version: repaired.version().version,
                valid_rows: vec![published(flag_id, valid.clone())],
            }],
        }
    );
    assert_eq!(
        head.report.reusable_rows(flag_id),
        with_full(valid, 1),
        "the published rows and the deferred group's valid rows"
    );
    assert_eq!(
        head.dataset.cell_flag_true_rows(flag_id).unwrap(),
        full(&[0, 1])
    );
    let mut expected = newer_values.to_vec();
    expected.extend([(3, Some("older-1-0")), (4, Some("older-1-1"))]);
    assert_eq!(
        published_values(&head.dataset, "summary", flag_id).await,
        values(&expected)
    );
}

#[rstest]
#[tokio::test]
async fn test_input_write_staged_before_publication_clears_it(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert)] write: BodyWrite,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let stale = dataset.clone();
    let staged_body = stage_rows(&stale, 0, &["body"], |_, offset| {
        Some(format!("new-{offset}"))
    })
    .await;
    let staged = stage_all(&dataset, "summary", "s").await;
    let publication = publish(&dataset, staged, set_true(flag_id, full(&[0, 1])), Reject)
        .await
        .unwrap();
    assert_eq!(
        publication.report.published,
        set_true(flag_id, full(&[0, 1]))
    );
    assert!(publication.report.deferred_rows.is_empty());
    assert!(publication.report.deferred_groups.is_empty());
    let published_at = publication.dataset;

    let written = match write {
        BodyWrite::Replace => CommitBuilder::new(Arc::new(published_at.clone()))
            .execute(replacement_txn(
                stale.version().version,
                vec![staged_body],
                vec![],
            ))
            .await
            .unwrap(),
        // Conflicts with the publication, then retries from the head.
        _ => merge_insert_body(&stale, 2, "new").await,
    };
    assert_eq!(
        written.version().version,
        published_at.version().version + 1
    );
    assert_eq!(
        recorded_invalidations(&written).await,
        cleared(flag_id, write.clears())
    );
    assert_eq!(
        written.cell_flag_true_rows(flag_id).unwrap(),
        write.still_valid()
    );
    let mut expected = vec![(3, Some("s-1-0")), (4, Some("s-1-1"))];
    if matches!(write, BodyWrite::MergeInsert) {
        expected.insert(0, (1, Some("s-0-0")));
    }
    assert_eq!(
        published_values(&written, "summary", flag_id).await,
        values(&expected)
    );
    assert_eq!(
        column_values(&written, "body", None).await[1],
        (2, Some(write.body_of_id_2().to_string()))
    );
}

/// A concurrent change that makes fragment 0's group unsafe to install.
#[derive(Debug, Clone, Copy)]
enum UnsafeWrite {
    DeleteFragment,
    MoveFragment,
    ReplaceOutput,
    MergeInsertOutput,
    Republish,
}

/// `stale` lists the offsets of fragment 0 whose flag the write clears: a
/// write to the output clears it like a write to a source.
#[rstest]
#[case::delete_fragment(UnsafeWrite::DeleteFragment, DeferralReason::FragmentRemoved, &[])]
#[case::move_fragment(UnsafeWrite::MoveFragment, DeferralReason::FragmentRemoved, &[])]
#[case::replace_output(UnsafeWrite::ReplaceOutput, DeferralReason::OutputWritten, &[0, 1])]
#[case::merge_insert_output(
    UnsafeWrite::MergeInsertOutput,
    DeferralReason::OutputWritten,
    &[1]
)]
#[case::republish(UnsafeWrite::Republish, DeferralReason::NewerResult, &[])]
#[tokio::test]
async fn test_unsafe_fragment_does_not_block_safe_ones(
    #[case] unsafe_write: UnsafeWrite,
    #[case] reason: DeferralReason,
    #[case] stale: &[u32],
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let other = match unsafe_write {
        UnsafeWrite::DeleteFragment => {
            dataset.delete("id <= 2").await.unwrap();
            dataset
        }
        UnsafeWrite::MoveFragment => update_where(&dataset, "id <= 2", "body", "moved").await,
        UnsafeWrite::ReplaceOutput => {
            let output = stage_rows(&dataset, 0, &["summary"], |_, _| Some("typed".into())).await;
            commit_replacement(&dataset, vec![output], vec![])
                .await
                .unwrap()
        }
        UnsafeWrite::MergeInsertOutput => {
            let source = record_batch!(("id", Int32, [2]), ("summary", Utf8, ["typed"])).unwrap();
            MergeInsertBuilder::try_new(Arc::new(dataset.clone()), vec!["id".into()])
                .unwrap()
                .when_matched(WhenMatched::UpdateAll)
                .when_not_matched(WhenNotMatched::DoNothing)
                .write_mode(MergeInsertWriteMode::RewriteColumns)
                .try_build()
                .unwrap()
                .execute_batches(vec![source])
                .await
                .unwrap()
                .0
                .as_ref()
                .clone()
        }
        UnsafeWrite::Republish => {
            let newer = stage_rows(&dataset, 0, &["summary"], |_, offset| {
                Some(format!("newer-0-{offset}"))
            })
            .await;
            publish(&dataset, vec![newer], set_true(flag_id, full(&[0])), Reject)
                .await
                .unwrap()
                .dataset
        }
    };
    let flags_after_write = other.cell_flag_true_rows(flag_id).unwrap();
    let is_republished = matches!(unsafe_write, UnsafeWrite::Republish);
    let newer_values = values(&[(1, Some("newer-0-0")), (2, Some("newer-0-1"))]);

    let result = publish(
        &read,
        refresh.clone(),
        set_true(flag_id, full(&[0, 1])),
        policy,
    )
    .await;
    if policy == Reject {
        let error = result.unwrap_err();
        if reason == DeferralReason::FragmentRemoved {
            assert!(
                matches!(error, Error::IncompatibleTransaction { .. }),
                "{error}"
            );
            assert!(
                error
                    .to_string()
                    .contains("target fragment 0 was removed by concurrent"),
                "{error}"
            );
        } else {
            assert!(
                matches!(error, Error::RetryableCommitConflict { .. }),
                "{error}"
            );
            assert!(error.to_string().contains("preempted"), "{error}");
        }
        let head = latest(&other).await;
        assert_eq!(head.version().version, other.version().version);
        assert_eq!(
            head.cell_flag_true_rows(flag_id).unwrap(),
            flags_after_write
        );
        return;
    }

    let result = result.unwrap();
    let conflicting_version = other.version().version;
    let deferred_rows = if stale.is_empty() {
        vec![]
    } else {
        vec![deferred(
            flag_id,
            rows(&[(0, stale)]),
            InputChanged,
            conflicting_version,
        )]
    };
    let valid: Vec<u32> = (0..2).filter(|offset| !stale.contains(offset)).collect();
    let valid_rows = if reason == DeferralReason::FragmentRemoved || valid.is_empty() {
        vec![]
    } else {
        vec![published(flag_id, rows(&[(0, &valid)]))]
    };
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: conflicting_version,
            committed_version: Some(conflicting_version + 1),
            published: vec![published(flag_id, full(&[1]))],
            deferred_rows,
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh[0].1.clone(),
                reason,
                conflicting_version,
                valid_rows,
            }],
        }
    );
    let expected_flags = if is_republished {
        full(&[0, 1])
    } else {
        full(&[1])
    };
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        expected_flags
    );
    let mut expected = if is_republished { newer_values } else { vec![] };
    expected.extend(values(&[(3, Some("s-1-0")), (4, Some("s-1-1"))]));
    assert_eq!(
        published_values(&result.dataset, "summary", flag_id).await,
        expected
    );
    if result.dataset.get_fragment(0).is_some() {
        assert_ne!(file_of(&result.dataset, 0, "summary"), refresh[0].1.path);
    }
    // Ids 1 and 2 still read NULL wherever they live, including the values
    // the output writes typed in without publishing them.
    if !is_republished && !matches!(unsafe_write, UnsafeWrite::DeleteFragment) {
        expected.extend(values(&[(1, None), (2, None)]));
        expected.sort();
    }
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        expected
    );
}

#[rstest]
#[tokio::test]
async fn test_copied_rows_cannot_overwrite_newer_result(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    // Both refreshes read id 1 and id 2 as NULL. The older computes id 1 and
    // copies id 2; the newer computes id 2 and copies id 1.
    let older = stage_rows(&dataset, 0, &["summary"], |_, offset| {
        (offset == 0).then(|| "older-0-0".to_string())
    })
    .await;
    let newer = stage_rows(&dataset, 0, &["summary"], |_, offset| {
        (offset == 1).then(|| "newer-0-1".to_string())
    })
    .await;
    let repaired = publish(
        &dataset,
        vec![newer],
        set_true(flag_id, rows(&[(0, &[1])])),
        Reject,
    )
    .await
    .unwrap()
    .dataset;
    assert!(recorded_invalidations(&repaired).await.is_empty());

    let result = publish(
        &dataset,
        vec![older.clone()],
        set_true(flag_id, rows(&[(0, &[0])])),
        policy,
    )
    .await;
    let report = match policy {
        Reject => {
            let error = result.unwrap_err();
            assert!(
                matches!(error, Error::RetryableCommitConflict { .. }),
                "{error}"
            );
            let expected = format!(
                "preempted by concurrent transaction DataReplacement at version {}",
                repaired.version().version
            );
            assert!(error.to_string().contains(&expected), "{error}");
            None
        }
        Skip => {
            let result = result.unwrap();
            assert_eq!(
                result.report,
                PublicationReport {
                    read_version: dataset.version().version,
                    checked_version: repaired.version().version,
                    committed_version: None,
                    published: vec![],
                    deferred_rows: vec![],
                    deferred_groups: vec![DeferredGroup {
                        fragment_id: 0,
                        data_file: older.1.clone(),
                        reason: NewerResult,
                        conflicting_version: repaired.version().version,
                        valid_rows: vec![published(flag_id, rows(&[(0, &[0])]))],
                    }],
                }
            );
            assert_eq!(result.dataset.version().version, repaired.version().version);
            Some(result.report)
        }
    };
    let head = latest(&repaired).await;
    assert_eq!(head.version().version, repaired.version().version);
    assert_eq!(
        head.cell_flag_true_rows(flag_id).unwrap(),
        rows(&[(0, &[1])])
    );
    assert_eq!(
        column_values(&head, "summary", None).await,
        values(&[(1, None), (2, Some("newer-0-1")), (3, None), (4, None)])
    );
    let Some(report) = report else {
        return;
    };

    // Restage the deferred group against the head it was checked against:
    // reuse the older value where the report says it is still valid, and copy
    // the newer result.
    let reusable = report.reusable_rows(flag_id);
    assert_eq!(reusable, rows(&[(0, &[0])]));
    let restaged = stage_rows(&head, 0, &["summary"], |_, offset| match offset {
        0 => Some("older-0-0".to_string()),
        _ => Some("newer-0-1".to_string()),
    })
    .await;
    let reused = publish(
        &head,
        vec![restaged],
        set_true(flag_id, reusable.clone()),
        Skip,
    )
    .await
    .unwrap();
    assert_eq!(
        reused.report,
        PublicationReport {
            read_version: report.checked_version,
            checked_version: report.checked_version,
            committed_version: Some(report.checked_version + 1),
            published: vec![published(flag_id, reusable)],
            deferred_rows: vec![],
            deferred_groups: vec![],
        }
    );
    assert_eq!(
        reused.dataset.cell_flag_true_rows(flag_id).unwrap(),
        full(&[0])
    );
    assert_eq!(
        published_values(&reused.dataset, "summary", flag_id).await,
        values(&[(1, Some("older-0-0")), (2, Some("newer-0-1"))])
    );
}

#[rstest]
#[tokio::test]
async fn test_registration_change_fences_old_publisher(
    #[values(false, true)] is_replace: bool,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let staged = stage_all(&read, "summary", "stale").await;
    if is_replace {
        dataset
            .replace_cell_flag(
                "summary",
                "ready",
                CellFlagOptions::default()
                    .with_clear_on_write(["body"])
                    .with_mask_when_false(true),
            )
            .await
            .unwrap();
    } else {
        dataset.drop_cell_flag("summary", "ready").await.unwrap();
    }
    let fenced_at = dataset.version().version;

    let error = publish(&read, staged, set_true(flag_id, full(&[0, 1])), policy)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains(&format!(
            "cell flag {flag_id} is not registered since version {fenced_at}: concurrent \
             UpdateConfig dropped or replaced {}",
            flag_label(&read, flag_id)
        )),
        "{error}"
    );
    let head = latest(&dataset).await;
    assert_eq!(head.version().version, fenced_at);
    if let Some(replaced) = head.cell_flag("summary", "ready") {
        assert!(
            head.cell_flag_true_rows(replaced.flag_id)
                .unwrap()
                .is_empty()
        );
    }
    assert!(
        column_values(&head, "summary", None)
            .await
            .iter()
            .all(|(_, value)| value.is_none())
    );
}

#[rstest]
#[tokio::test]
async fn test_write_staged_before_registration_defers_later_refresh(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let before = dataset.clone();
    let staged_body = stage_rows(&before, 0, &["body"], |_, offset| {
        Some(format!("new-{offset}"))
    })
    .await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let written = CommitBuilder::new(Arc::new(dataset))
        .execute(replacement_txn(
            before.version().version,
            vec![staged_body],
            vec![],
        ))
        .await
        .unwrap();
    assert_eq!(
        recorded_invalidations(&written).await,
        cleared(flag_id, full(&[0]))
    );

    let result = publish(&read, refresh, set_true(flag_id, full(&[0, 1])), policy).await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("cannot be published on 2 row(s) of fragment 0"),
            "{error}"
        );
        assert!(
            latest(&written)
                .await
                .cell_flag_true_rows(flag_id)
                .unwrap()
                .is_empty()
        );
        return;
    }
    let result = result.unwrap();
    assert_eq!(
        result.report.deferred_rows,
        vec![deferred(
            flag_id,
            rows(&[(0, &[0, 1])]),
            InputChanged,
            written.version().version
        )]
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        full(&[1])
    );
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[(1, None), (2, None), (3, Some("s-1-0")), (4, Some("s-1-1"))])
    );
}

/// Commits `competitor` just before the first manifest write it is asked for,
/// so that attempt loses the version and the commit retries against it.
#[derive(Debug)]
struct CommitsCompetitorFirst {
    inner: Arc<dyn CommitHandler>,
    competitor: Mutex<Option<(Arc<Dataset>, Transaction)>>,
}

#[async_trait::async_trait]
impl CommitHandler for CommitsCompetitorFirst {
    async fn commit(
        &self,
        manifest: &mut Manifest,
        indices: Option<Vec<IndexMetadata>>,
        base_path: &Path,
        object_store: &ObjectStore,
        manifest_writer: ManifestWriter,
        naming_scheme: ManifestNamingScheme,
        transaction: Option<TableTransaction>,
    ) -> std::result::Result<ManifestLocation, CommitError> {
        let competitor = self.competitor.lock().unwrap().take();
        if let Some((dataset, competitor)) = competitor {
            CommitBuilder::new(dataset)
                .with_commit_handler(self.inner.clone())
                .execute(competitor)
                .await
                .map_err(CommitError::OtherError)?;
        }
        self.inner
            .commit(
                manifest,
                indices,
                base_path,
                object_store,
                manifest_writer,
                naming_scheme,
                transaction,
            )
            .await
    }
}

/// A transaction the handler commits while a body write's first manifest
/// write is in flight.
#[derive(Debug, Clone, Copy)]
enum Competitor {
    /// Registers `translation.ready`, which watches body.
    Registration,
    /// Publishes `summary.ready` on every row.
    Publication,
}

/// The body write loses its first commit slot to the competitor and retries:
/// through the commit loop's rebase, or, where the competitor rewrote a
/// fragment it updates, by rerunning the whole write. The transaction file it
/// finally commits must record the clears the head at that point implies.
#[rstest]
#[tokio::test]
async fn test_commit_retry_records_every_invalidation(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert, BodyWrite::Update)] write: BodyWrite,
    #[values(Competitor::Registration, Competitor::Publication)] competitor: Competitor,
) {
    let mut dataset = articles(false).await;
    let translation = ArrowSchema::new(vec![ArrowField::new("translation", DataType::Utf8, true)]);
    dataset
        .add_columns(
            NewColumnTransform::AllNulls(Arc::new(translation)),
            None,
            None,
        )
        .await
        .unwrap();
    let ready = register_ready(&mut dataset).await;
    let schema = dataset.schema();
    let competing = match competitor {
        Competitor::Registration => {
            TransactionBuilder::new(dataset.version().version, update_config())
                .cell_flag_changes(CellFlagChanges {
                    registrations: vec![CellFlagRegistration {
                        field_id: schema.field("translation").unwrap().id,
                        name: "ready".to_string(),
                        clear_on_write: vec![schema.field("body").unwrap().id],
                        mask_when_false: true,
                    }],
                    ..Default::default()
                })
                .build()
        }
        Competitor::Publication => replacement_txn(
            dataset.version().version,
            stage_all(&dataset, "summary", "s").await,
            set_true(ready, full(&[0, 1])),
        ),
    };
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: dataset.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(dataset.clone()), competing))),
    });
    let mut writer = dataset.clone();
    writer.commit_handler = handler.clone();

    let written = write.apply(&writer).await;
    assert!(handler.competitor.lock().unwrap().is_none());
    assert_eq!(
        written.version().version,
        dataset.version().version + 2,
        "the competitor took the first attempt's version"
    );
    let changes = written
        .read_transaction()
        .await
        .unwrap()
        .unwrap()
        .cell_flag_changes
        .unwrap();
    let translated = written
        .cell_flag("translation", "ready")
        .map(|flag| flag.flag_id);
    if matches!(write, BodyWrite::Update) {
        // The moved row's new address starts unassigned for both flags.
        let body = schema.field("body").unwrap().id;
        assert_eq!(changes.moved_rows_written_fields, Some(vec![body]));
        assert!(recorded_invalidations(&written).await.is_empty());
    } else {
        let mut expected = cleared(ready, write.clears());
        if let Some(translated) = translated {
            expected.extend(cleared(translated, write.clears()));
        }
        assert_eq!(recorded_invalidations(&written).await, expected);
    }
    match competitor {
        Competitor::Registration => {
            assert!(written.cell_flag_true_rows(ready).unwrap().is_empty());
            let translated = translated.unwrap();
            assert!(written.cell_flag_true_rows(translated).unwrap().is_empty());
        }
        Competitor::Publication => {
            assert_eq!(
                written.cell_flag_true_rows(ready).unwrap(),
                write.still_valid()
            );
            let id_1 = (!matches!(write, BodyWrite::Replace)).then_some("s-0-0");
            assert_eq!(
                column_values(&written, "summary", None).await,
                values(&[(1, id_1), (2, None), (3, Some("s-1-0")), (4, Some("s-1-1"))])
            );
        }
    }
    assert_eq!(
        column_values(&written, "body", None).await[1],
        (2, Some(write.body_of_id_2().to_string()))
    );
}

#[tokio::test]
async fn test_publication_retry_accumulates_deferrals() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let newer = stage_rows(&dataset, 0, &["summary"], |_, offset| {
        Some(format!("newer-0-{offset}"))
    })
    .await;
    let repaired = publish(&dataset, vec![newer], set_true(flag_id, full(&[0])), Reject)
        .await
        .unwrap()
        .dataset;
    // Changes the inputs of ids 1 and 3, the first row of each fragment, while
    // the first attempt writes its manifest.
    let source = record_batch!(("id", Int32, [1, 3]), ("body", Utf8, ["new", "new"])).unwrap();
    let competitor = MergeInsertBuilder::try_new(Arc::new(repaired.clone()), vec!["id".into()])
        .unwrap()
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::DoNothing)
        .write_mode(MergeInsertWriteMode::RewriteColumns)
        .try_build()
        .unwrap()
        .execute_uncommitted_batches(vec![source])
        .await
        .unwrap()
        .transaction;
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: repaired.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(repaired.clone()), competitor))),
    });

    let result = CommitBuilder::new(Arc::new(read.clone()))
        .with_commit_handler(handler.clone())
        .with_dependency_conflict_policy(Skip)
        .execute_with_report(replacement_txn(
            read.version().version,
            refresh.clone(),
            set_true(flag_id, full(&[0, 1])),
        ))
        .await
        .unwrap();
    assert!(handler.competitor.lock().unwrap().is_none());
    let deferred_at = repaired.version().version;
    let changed_at = deferred_at + 1;
    // The first attempt deferred fragment 0's group; the second finds the
    // competitor's change in the installed group and in the deferred group's
    // valid rows alike.
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: changed_at,
            committed_version: Some(changed_at + 1),
            published: vec![published(flag_id, rows(&[(1, &[1])]))],
            deferred_rows: vec![deferred(
                flag_id,
                rows(&[(0, &[0]), (1, &[0])]),
                InputChanged,
                changed_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh[0].1.clone(),
                reason: NewerResult,
                conflicting_version: deferred_at,
                valid_rows: vec![published(flag_id, rows(&[(0, &[1])]))],
            }],
        }
    );
    let committed = result.dataset.read_transaction().await.unwrap().unwrap();
    assert_eq!(
        committed.read_version,
        read.version().version,
        "the retried publication keeps the version its values were computed at"
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        rows(&[(0, &[1]), (1, &[1])])
    );
    // Id 1's newer value and id 3's installed stale value are masked alike.
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[
            (1, None),
            (2, Some("newer-0-1")),
            (3, None),
            (4, Some("s-1-1"))
        ])
    );
}

/// The first attempt installs fragment 0 with id 2's stale, unmasked value and
/// clears `ready` there; the retry defers that group to a newer result, which
/// must keep its flag on id 2.
#[tokio::test]
async fn test_retry_that_defers_a_group_keeps_the_newer_flag() {
    let mut dataset = articles(false).await;
    let ready = dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap()
        .flag_id;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let written = merge_insert_body(&dataset, 2, "new").await;
    let newer = stage_rows(&written, 0, &["summary"], |_, offset| {
        Some(format!("newer-0-{offset}"))
    })
    .await;
    let competitor = replacement_txn(
        written.version().version,
        vec![newer],
        set_true(ready, full(&[0])),
    );
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: written.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(written.clone()), competitor))),
    });

    let result = CommitBuilder::new(Arc::new(read.clone()))
        .with_commit_handler(handler.clone())
        .with_dependency_conflict_policy(Skip)
        .execute_with_report(replacement_txn(
            read.version().version,
            refresh.clone(),
            set_true(ready, full(&[0, 1])),
        ))
        .await
        .unwrap();
    assert!(handler.competitor.lock().unwrap().is_none());
    let changed_at = written.version().version;
    let republished_at = changed_at + 1;
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: republished_at,
            committed_version: Some(republished_at + 1),
            published: vec![published(ready, full(&[1]))],
            deferred_rows: vec![deferred(
                ready,
                rows(&[(0, &[1])]),
                InputChanged,
                changed_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh[0].1.clone(),
                reason: NewerResult,
                conflicting_version: republished_at,
                valid_rows: vec![published(ready, rows(&[(0, &[0])]))],
            }],
        }
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(ready).unwrap(),
        full(&[0, 1])
    );
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[
            (1, Some("newer-0-0")),
            (2, Some("newer-0-1")),
            (3, Some("s-1-0")),
            (4, Some("s-1-1")),
        ])
    );
}

/// How fragment 0 is removed after a refresh staged it.
#[derive(Debug, Clone, Copy)]
enum Removal {
    Delete,
    MoveRows,
}

/// Fragment 0's group is deferred to a newer result and id 2's input changes
/// before the fragment is removed, while the publication rebases or while its
/// first attempt writes its manifest. The removal is what the caller must act
/// on, and no row of fragment 0 is left to reuse or recompute.
#[rstest]
#[case::delete(Removal::Delete, false)]
#[case::move_rows(Removal::MoveRows, false)]
#[case::delete_during_retry(Removal::Delete, true)]
#[tokio::test]
async fn test_fragment_removal_supersedes_earlier_deferral(
    #[case] removal: Removal,
    #[case] is_during_retry: bool,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let newer = stage_rows(&dataset, 0, &["summary"], |_, offset| {
        Some(format!("newer-0-{offset}"))
    })
    .await;
    let repaired = publish(&dataset, vec![newer], set_true(flag_id, full(&[0])), Reject)
        .await
        .unwrap()
        .dataset;
    let written = merge_insert_body(&repaired, 2, "new").await;
    let publication = replacement_txn(
        read.version().version,
        refresh.clone(),
        set_true(flag_id, full(&[0, 1])),
    );
    let mut commit =
        CommitBuilder::new(Arc::new(read.clone())).with_dependency_conflict_policy(Skip);
    let handler = if is_during_retry {
        let delete = DeleteBuilder::new(Arc::new(written.clone()), "id <= 2")
            .execute_uncommitted()
            .await
            .unwrap()
            .transaction;
        let handler = Arc::new(CommitsCompetitorFirst {
            inner: written.commit_handler.clone(),
            competitor: Mutex::new(Some((Arc::new(written.clone()), delete))),
        });
        commit = commit.with_commit_handler(handler.clone());
        Some(handler)
    } else {
        match removal {
            Removal::Delete => {
                written.clone().delete("id <= 2").await.unwrap();
            }
            Removal::MoveRows => {
                update_where(&written, "id <= 2", "body", "moved").await;
            }
        }
        None
    };

    let result = commit.execute_with_report(publication).await.unwrap();
    if let Some(handler) = handler {
        assert!(handler.competitor.lock().unwrap().is_none());
    }
    let removed_at = written.version().version + 1;
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: removed_at,
            committed_version: Some(removed_at + 1),
            published: vec![published(flag_id, full(&[1]))],
            deferred_rows: vec![],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh[0].1.clone(),
                reason: FragmentRemoved,
                conflicting_version: removed_at,
                valid_rows: vec![],
            }],
        }
    );
    assert_eq!(result.report.reusable_rows(flag_id), full(&[1]));
    assert!(result.dataset.get_fragment(0).is_none());
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        full(&[1])
    );
    let mut expected = values(&[(3, Some("s-1-0")), (4, Some("s-1-1"))]);
    if matches!(removal, Removal::MoveRows) && !is_during_retry {
        expected.splice(0..0, values(&[(1, None), (2, None)]));
    }
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        expected
    );
}

/// `ready` publishes an unmasked summary; `fresh` is computed from it.
#[tokio::test]
async fn test_unmasked_stale_output_invalidates_downstream_flags() {
    let mut dataset = articles(false).await;
    let keywords = ArrowSchema::new(vec![ArrowField::new("keywords", DataType::Utf8, true)]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(keywords)), None, None)
        .await
        .unwrap();
    let ready = dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap()
        .flag_id;
    let fresh = dataset
        .register_cell_flag(
            "keywords",
            "fresh",
            CellFlagOptions::default()
                .with_clear_on_write(["summary"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap()
        .flag_id;
    let summary_read = dataset.clone();
    let summaries = stage_all(&summary_read, "summary", "s").await;
    let written = merge_insert_body(&dataset, 2, "new").await;
    // Computes id 2's keywords from its summary, which reads as NULL here.
    let keywords_read = written.clone();
    let keywords = stage_rows(&keywords_read, 0, &["keywords"], |_, offset| {
        (offset == 1).then(|| "kw-of-null".to_string())
    })
    .await;

    let summarized = publish(
        &summary_read,
        summaries,
        set_true(ready, full(&[0, 1])),
        Skip,
    )
    .await
    .unwrap();
    let summarized_at = written.version().version + 1;
    assert_eq!(
        summarized.report.deferred_rows,
        vec![deferred(
            ready,
            rows(&[(0, &[1])]),
            InputChanged,
            written.version().version
        )]
    );
    // Id 2's stale summary is installed unmasked, which changes what it reads
    // as, so fresh is cleared there as well as on the published rows.
    assert_eq!(
        column_values(&summarized.dataset, "summary", None).await[1],
        (2, Some("s-0-1".to_string()))
    );
    let mut fresh_cleared = rows(&[(0, &[0, 1])]);
    fresh_cleared.insert_fragment(1);
    assert_eq!(
        recorded_invalidations(&summarized.dataset).await,
        cleared(fresh, fresh_cleared)
    );

    let keywords_path = keywords.1.path.clone();
    let result = publish(
        &keywords_read,
        vec![keywords],
        set_true(fresh, rows(&[(0, &[1])])),
        Skip,
    )
    .await
    .unwrap();
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: keywords_read.version().version,
            checked_version: summarized_at,
            committed_version: Some(summarized_at + 1),
            published: vec![],
            deferred_rows: vec![deferred(
                fresh,
                rows(&[(0, &[1])]),
                InputChanged,
                summarized_at
            )],
            deferred_groups: vec![],
        }
    );
    assert!(
        result
            .dataset
            .cell_flag_true_rows(fresh)
            .unwrap()
            .is_empty()
    );
    // The stale keywords are installed under a false flag and read as NULL.
    assert_eq!(file_of(&result.dataset, 0, "keywords"), keywords_path);
    assert_eq!(
        column_values(&result.dataset, "keywords", None).await,
        values(&[(1, None), (2, None), (3, None), (4, None)])
    );
}

/// A flag published together with a dependent flag upstream of it keeps the
/// rows both assign: Lance takes it to be computed from the upstream values
/// the same transaction publishes, not from the read snapshot. Published on
/// its own, computed from the snapshot where the upstream output reads NULL,
/// it is cleared once the upstream is published.
#[rstest]
#[tokio::test]
async fn test_chained_outputs_published_together(#[values(false, true)] is_together: bool) {
    let mut dataset = articles(false).await;
    let translation = ArrowSchema::new(vec![ArrowField::new("translation", DataType::Utf8, true)]);
    dataset
        .add_columns(
            NewColumnTransform::AllNulls(Arc::new(translation)),
            None,
            None,
        )
        .await
        .unwrap();
    let masked_on = |sources: &[&str]| {
        CellFlagOptions::default()
            .with_clear_on_write(sources.iter().copied())
            .with_mask_when_false(true)
    };
    let ready = dataset
        .register_cell_flag("summary", "ready", masked_on(&["body"]))
        .await
        .unwrap()
        .flag_id;
    let translated = dataset
        .register_cell_flag("translation", "ready", masked_on(&["summary"]))
        .await
        .unwrap()
        .flag_id;
    let summary_of = |fragment_id: u64, offset: u64| format!("s-{fragment_id}-{offset}");

    let dataset = if is_together {
        let mut staged = Vec::new();
        for fragment_id in [0, 1] {
            staged.push(
                stage_rows(
                    &dataset,
                    fragment_id,
                    &["summary", "translation"],
                    |column, offset| {
                        let summary = summary_of(fragment_id, offset);
                        Some(match column {
                            "summary" => summary,
                            _ => format!("tr-{summary}"),
                        })
                    },
                )
                .await,
            );
        }
        let updates = vec![
            published(ready, full(&[0, 1])),
            published(translated, full(&[0, 1])),
        ];
        let dataset = publish(&dataset, staged, updates, Reject)
            .await
            .unwrap()
            .dataset;
        assert!(recorded_invalidations(&dataset).await.is_empty());
        dataset
    } else {
        let mut staged = Vec::new();
        for fragment_id in [0, 1] {
            staged.push(
                stage_rows(&dataset, fragment_id, &["translation"], |_, _| {
                    Some("tr-NULL".to_string())
                })
                .await,
            );
        }
        let dataset = publish(
            &dataset,
            staged,
            set_true(translated, full(&[0, 1])),
            Reject,
        )
        .await
        .unwrap()
        .dataset;
        let mut staged = Vec::new();
        for fragment_id in [0, 1] {
            staged.push(
                stage_rows(&dataset, fragment_id, &["summary"], |_, offset| {
                    Some(summary_of(fragment_id, offset))
                })
                .await,
            );
        }
        let dataset = publish(&dataset, staged, set_true(ready, full(&[0, 1])), Reject)
            .await
            .unwrap()
            .dataset;
        assert_eq!(
            recorded_invalidations(&dataset).await,
            cleared(translated, full(&[0, 1]))
        );
        dataset
    };

    assert_eq!(dataset.cell_flag_true_rows(ready).unwrap(), full(&[0, 1]));
    let translated_rows = if is_together {
        full(&[0, 1])
    } else {
        RowAddrTreeMap::new()
    };
    assert_eq!(
        dataset.cell_flag_true_rows(translated).unwrap(),
        translated_rows
    );
    let expected: Vec<(i32, Option<String>)> = column_values(&dataset, "summary", None)
        .await
        .into_iter()
        .map(|(id, summary)| {
            let translation = summary
                .filter(|_| is_together)
                .map(|summary| format!("tr-{summary}"));
            (id, translation)
        })
        .collect();
    assert_eq!(column_values(&dataset, "translation", None).await, expected);
}

/// `summary` computed from body and `translation` from summary.
const CHAIN: [(&str, &str); 2] = [("summary", "body"), ("translation", "summary")];

/// Two fragments of `fragment_rows` rows holding `id`, `body` (`b{id}`) and a
/// NULL masked output for each `(output, input)` of `outputs`, computed from
/// that input with [`computed`].
async fn computed_outputs(fragment_rows: i32, outputs: &[(&str, &str)]) -> Dataset {
    computed_outputs_with(2, fragment_rows, outputs, true, false).await
}

/// [`computed_outputs`] over `fragments` fragments, with masking flags when
/// `is_masked` and stable row ids when `stable_row_ids`.
async fn computed_outputs_with(
    fragments: i32,
    fragment_rows: i32,
    outputs: &[(&str, &str)],
    is_masked: bool,
    stable_row_ids: bool,
) -> Dataset {
    let ids: Vec<i32> = (1..=fragments * fragment_rows).collect();
    let bodies = StringArray::from_iter_values(ids.iter().map(|id| format!("b{id}")));
    let row_count = ids.len();
    let mut columns = vec![
        ("id", Arc::new(Int32Array::from(ids)) as ArrayRef),
        ("body", Arc::new(bodies) as ArrayRef),
    ];
    // Stored rather than metadata-only: a replacement file must write only
    // fields its fragment's files cover, or only fields they do not, so a
    // follow-up could not stage summary with a translation a competitor
    // already replaced.
    for (output, _) in outputs {
        columns.push((
            *output,
            Arc::new(StringArray::new_null(row_count)) as ArrayRef,
        ));
    }
    let batch = RecordBatch::try_from_iter(columns).unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            max_rows_per_file: fragment_rows as usize,
            enable_stable_row_ids: stable_row_ids,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    for (output, input) in outputs {
        dataset
            .register_cell_flag(
                output,
                "ready",
                CellFlagOptions::default()
                    .with_clear_on_write([*input])
                    .with_mask_when_false(is_masked),
            )
            .await
            .unwrap();
    }
    dataset
}

/// What `column` computes from its input as a masked read shows it.
fn computed(column: &str, input: Option<&str>) -> String {
    format!("{column}({})", input.unwrap_or("NULL"))
}

fn flag_of(dataset: &Dataset, column: &str) -> u32 {
    dataset.cell_flag(column, "ready").unwrap().flag_id
}

fn input_of(dataset: &Dataset, column: &str) -> String {
    let flag = dataset.cell_flag(column, "ready").unwrap();
    let input = dataset
        .schema()
        .field_by_id(flag.clear_on_write[0])
        .unwrap();
    input.name.clone()
}

/// `column` of every live row of `dataset`, by row address.
async fn values_by_addr(dataset: &Dataset, column: &str) -> BTreeMap<u64, Option<String>> {
    let batch = dataset
        .scan()
        .project(&[column])
        .unwrap()
        .with_row_address()
        .try_into_batch()
        .await
        .unwrap();
    let column_values = batch[column].as_string::<i32>();
    let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
    (0..batch.num_rows())
        .map(|row| {
            let value = column_values
                .is_valid(row)
                .then(|| column_values.value(row).to_string());
            (addrs.value(row), value)
        })
        .collect()
}

/// Every value a masked read of `column` shows is what it computes from its
/// input as the same read shows it.
async fn assert_visible_values_follow_inputs(dataset: &Dataset, column: &str) {
    let inputs = values_by_addr(dataset, &input_of(dataset, column)).await;
    for (addr, value) in values_by_addr(dataset, column).await {
        if let Some(value) = value {
            let expected = computed(column, inputs[&addr].as_deref());
            assert_eq!(value, expected, "visible {column} at row {addr:#x}");
        }
    }
}

/// Values of assigned rows, by output and row address.
type StagedValues = BTreeMap<(String, u64), Option<String>>;

/// A refresh of computed outputs staged against one snapshot.
#[derive(Default)]
struct Refresh {
    groups: Vec<DataReplacementGroup>,
    assigned: BTreeMap<u32, RowAddrTreeMap>,
    staged: StagedValues,
}

impl Refresh {
    /// Stage one file on fragment `fragment_id` of `read` for `outputs`, in
    /// dependency order with the offsets each assigns. An assigned offset is
    /// computed from its input as this file holds it, or as `read` shows it
    /// when the file does not write the input, unless `reused` holds its
    /// value and the file does not also assign the input there. Other
    /// offsets are copied, so an output given none is written unassigned.
    async fn stage(
        &mut self,
        read: &Dataset,
        fragment_id: u32,
        outputs: &[(&str, &[u32])],
        reused: &StagedValues,
    ) {
        let fragment = read.get_fragment(fragment_id as usize).unwrap();
        let physical_rows = fragment.physical_rows().await.unwrap() as u32;
        let addr = |offset: u32| u64::from(RowAddress::new_from_parts(fragment_id, offset));
        let mut file: BTreeMap<String, Vec<Option<String>>> = BTreeMap::new();
        let file_offsets: BTreeMap<&str, &[u32]> = outputs.iter().copied().collect();
        for (column, offsets) in outputs {
            let input = input_of(read, column);
            let input_offsets = file_offsets.get(input.as_str()).copied().unwrap_or(&[]);
            let inputs = match file.get(&input) {
                Some(inputs) => inputs.clone(),
                None => {
                    let shown = values_by_addr(read, &input).await;
                    (0..physical_rows)
                        .map(|offset| shown[&addr(offset)].clone())
                        .collect()
                }
            };
            let shown = values_by_addr(read, column).await;
            let mut column_values = Vec::new();
            for offset in 0..physical_rows {
                let key = (column.to_string(), addr(offset));
                let value = if offsets.contains(&offset) {
                    let value = match reused.get(&key) {
                        Some(value) if !input_offsets.contains(&offset) => value.clone(),
                        _ => Some(computed(column, inputs[offset as usize].as_deref())),
                    };
                    self.staged.insert(key, value.clone());
                    value
                } else {
                    shown[&addr(offset)].clone()
                };
                column_values.push(value);
            }
            file.insert(column.to_string(), column_values);

            if offsets.is_empty() {
                continue;
            }
            let mut rows = RowAddrTreeMap::new();
            if offsets.len() == physical_rows as usize {
                rows.insert_fragment(fragment_id);
            } else {
                rows.insert_bitmap(fragment_id, offsets.iter().copied().collect());
            }
            *self.assigned.entry(flag_of(read, column)).or_default() |= &rows;
        }
        let columns: Vec<&str> = outputs.iter().map(|(column, _)| *column).collect();
        let group = stage_rows(read, u64::from(fragment_id), &columns, |column, offset| {
            file[column][offset as usize].clone()
        })
        .await;
        self.groups.push(group);
    }

    fn updates(&self) -> Vec<CellFlagUpdate> {
        self.assigned
            .iter()
            .map(|(flag_id, rows)| published(*flag_id, rows.clone()))
            .collect()
    }

    fn transaction(&self, read: &Dataset) -> Transaction {
        replacement_txn(read.version().version, self.groups.clone(), self.updates())
    }

    async fn publish(
        &self,
        read: &Dataset,
        policy: DependencyConflictPolicy,
    ) -> Result<PublicationResult> {
        publish(read, self.groups.clone(), self.updates(), policy).await
    }

    /// The staged values `report` lists as reusable.
    fn reusable(&self, report: &PublicationReport, dataset: &Dataset) -> StagedValues {
        self.staged
            .iter()
            .filter(|((column, addr), _)| {
                report
                    .reusable_rows(flag_of(dataset, column))
                    .contains(*addr)
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    /// Every row this refresh assigned is reported once: published, deferred
    /// or valid in a deferred group.
    fn assert_report_accounts_for_every_row(&self, read: &Dataset, report: &PublicationReport) {
        let addrs = |rows: &RowAddrTreeMap| -> Vec<u64> {
            let mut addrs = Vec::new();
            for (fragment, selection) in rows.iter() {
                let offsets: Vec<u32> = match selection {
                    RowAddrSelection::Full => {
                        let fragment = read.get_fragment(*fragment as usize).unwrap();
                        (0..fragment.metadata().physical_rows.unwrap() as u32).collect()
                    }
                    RowAddrSelection::Partial(offsets) => offsets.iter().collect(),
                };
                addrs.extend(
                    offsets
                        .into_iter()
                        .map(|offset| u64::from(RowAddress::new_from_parts(*fragment, offset))),
                );
            }
            addrs
        };
        for (flag_id, assigned) in &self.assigned {
            let published = report.published_rows(*flag_id);
            let deferred = report
                .deferred_rows
                .iter()
                .filter(|deferred| deferred.flag_id == *flag_id)
                .map(|deferred| &deferred.rows);
            let valid = report
                .deferred_groups
                .iter()
                .flat_map(|group| &group.valid_rows)
                .filter(|valid| valid.flag_id == *flag_id)
                .map(|valid| &valid.rows);
            let mut reported = BTreeSet::new();
            for addr in std::iter::once(&published)
                .chain(deferred)
                .chain(valid)
                .flat_map(addrs)
            {
                assert!(
                    reported.insert(addr),
                    "flag {flag_id} reports row {addr:#x} twice"
                );
            }
            let assigned: BTreeSet<u64> = addrs(assigned).into_iter().collect();
            assert_eq!(reported, assigned, "flag {flag_id}");
        }
    }

    /// Every staged value `report` lists as reusable is what its output
    /// computes from its input as `head` shows it.
    async fn assert_reusable_values_follow_inputs(
        &self,
        head: &Dataset,
        report: &PublicationReport,
    ) {
        let mut inputs: BTreeMap<String, BTreeMap<u64, Option<String>>> = BTreeMap::new();
        for ((column, addr), value) in self.reusable(report, head) {
            let input = input_of(head, &column);
            if !inputs.contains_key(&input) {
                let shown = values_by_addr(head, &input).await;
                inputs.insert(input.clone(), shown);
            }
            let expected = computed(&column, inputs[&input][&addr].as_deref());
            assert_eq!(
                value.as_deref(),
                Some(expected.as_str()),
                "reusable {column} at row {addr:#x}"
            );
        }
    }

    /// Finish this refresh's deferred groups as `report` says, reading
    /// `head`: one file per deferred group assigns `outputs`, in dependency
    /// order, on the rows reported reusable or to recompute, reuses the staged
    /// values on the reusable rows, computes the others from `head`, and
    /// copies every other row. Returns the follow-up's version.
    async fn follow_report(
        &self,
        report: &PublicationReport,
        head: &Dataset,
        outputs: &[&str],
    ) -> Dataset {
        let reused = self.reusable(report, head);
        let mut follow_up = Self::default();
        for group in &report.deferred_groups {
            let fragment_id = u32::try_from(group.fragment_id).unwrap();
            let offsets: Vec<(&str, Vec<u32>)> = outputs
                .iter()
                .map(|column| {
                    let flag_id = flag_of(head, column);
                    let rows = report.reusable_rows(flag_id)
                        | report.deferred_rows_of(flag_id, InputChanged)
                        | report.deferred_rows_of(flag_id, UpstreamNotPublished);
                    let offsets: Vec<u32> = match rows.get(&fragment_id) {
                        None => Vec::new(),
                        Some(RowAddrSelection::Full) => {
                            let fragment = head.get_fragment(fragment_id as usize).unwrap();
                            (0..fragment.metadata().physical_rows.unwrap() as u32).collect()
                        }
                        Some(RowAddrSelection::Partial(offsets)) => offsets.iter().collect(),
                    };
                    (*column, offsets)
                })
                .filter(|(_, offsets)| !offsets.is_empty())
                .collect();
            let outputs: Vec<(&str, &[u32])> = offsets
                .iter()
                .map(|(column, offsets)| (*column, offsets.as_slice()))
                .collect();
            follow_up.stage(head, fragment_id, &outputs, &reused).await;
        }
        follow_up.publish(head, Reject).await.unwrap().dataset
    }
}

/// What the follow-up of
/// [`test_deferred_group_recomputes_outputs_of_its_staged_upstream`] does with
/// the reusable summaries of fragment 0.
#[derive(Debug, Clone, Copy)]
enum SummaryFollowUp {
    /// Leaves them unpublished and translates alone.
    Unpublished,
    /// Publishes them, then translates alone.
    First,
    /// Publishes them in the translations' file.
    Together,
}

impl SummaryFollowUp {
    fn publishes_summary(self) -> bool {
        !matches!(self, Self::Unpublished)
    }
}

/// Summary and the translation computed from it are staged in one file, and a
/// concurrent translation of id 1 defers that file. Its translations were
/// computed from summaries that never committed, so they are deferred for
/// recomputation, while its summaries stay reusable. A follow-up that follows
/// the report, translating alone, after republishing the reusable summaries
/// or together with them, leaves every translation computed from the
/// committed summary.
#[rstest]
#[case::unpublished(SummaryFollowUp::Unpublished)]
#[case::first(SummaryFollowUp::First)]
#[case::together(SummaryFollowUp::Together)]
#[tokio::test]
async fn test_deferred_group_recomputes_outputs_of_its_staged_upstream(
    #[case] follow_up: SummaryFollowUp,
) {
    let read = computed_outputs(2, &CHAIN).await;
    let (summary, translation) = (flag_of(&read, "summary"), flag_of(&read, "translation"));
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        let outputs: [(&str, &[u32]); 2] = [("summary", &[0, 1]), ("translation", &[0, 1])];
        refresh
            .stage(&read, fragment_id, &outputs, &StagedValues::new())
            .await;
    }
    let mut competitor = Refresh::default();
    competitor
        .stage(&read, 0, &[("translation", &[0])], &StagedValues::new())
        .await;
    let competed_at = competitor
        .publish(&read, Reject)
        .await
        .unwrap()
        .dataset
        .version()
        .version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: vec![
                published(summary, full(&[1])),
                published(translation, full(&[1]))
            ],
            deferred_rows: vec![deferred(
                translation,
                rows(&[(0, &[0, 1])]),
                UpstreamNotPublished,
                competed_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: vec![published(summary, rows(&[(0, &[0, 1])]))],
            }],
        }
    );
    assert_eq!(
        report.reusable_rows(summary),
        with_full(rows(&[(0, &[0, 1])]), 1)
    );
    assert_eq!(report.reusable_rows(translation), full(&[1]));
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    let mut head = result.dataset.clone();
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[1]));
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(
        column_values(&head, "translation", None).await,
        values(&[
            (1, Some("translation(NULL)")),
            (2, None),
            (3, Some("translation(summary(b3))")),
            (4, Some("translation(summary(b4))")),
        ])
    );

    // Fragment 0's work: reuse what is reusable, recompute what is deferred
    // from the snapshot each step reads.
    let offsets = |rows: RowAddrTreeMap| -> Vec<u32> {
        rows.get_fragment_bitmap(0).unwrap().iter().collect()
    };
    let summaries = offsets(report.reusable_rows(summary));
    let translations = offsets(report.deferred_rows_of(translation, UpstreamNotPublished));
    let reused = refresh.reusable(report, &head);
    let mut outputs: Vec<(&str, &[u32])> = Vec::new();
    match follow_up {
        SummaryFollowUp::Unpublished => {}
        SummaryFollowUp::First => {
            let mut summarize = Refresh::default();
            summarize
                .stage(&head, 0, &[("summary", &summaries)], &reused)
                .await;
            head = summarize.publish(&head, Reject).await.unwrap().dataset;
            // Published alone, summary clears the translation computed from it.
            assert_eq!(head.cell_flag_true_rows(translation).unwrap(), full(&[1]));
        }
        SummaryFollowUp::Together => outputs.push(("summary", &summaries)),
    }
    outputs.push(("translation", &translations));
    let mut translate = Refresh::default();
    translate.stage(&head, 0, &outputs, &reused).await;
    let done = translate.publish(&head, Reject).await.unwrap().dataset;

    let summary_rows = if follow_up.publishes_summary() {
        full(&[0, 1])
    } else {
        full(&[1])
    };
    assert_eq!(done.cell_flag_true_rows(summary).unwrap(), summary_rows);
    assert_eq!(
        done.cell_flag_true_rows(translation).unwrap(),
        full(&[0, 1])
    );
    let summaries = column_values(&done, "summary", None).await;
    let fragment_0 = if follow_up.publishes_summary() {
        [Some("summary(b1)"), Some("summary(b2)")]
    } else {
        [None, None]
    };
    assert_eq!(
        summaries[..2],
        values(&[(1, fragment_0[0]), (2, fragment_0[1])])
    );
    let expected: Vec<(i32, Option<String>)> = summaries
        .into_iter()
        .map(|(id, summary)| (id, Some(computed("translation", summary.as_deref()))))
        .collect();
    assert_eq!(column_values(&done, "translation", None).await, expected);
}

/// Summary is assigned on ids 1 and 2, translation on ids 2 and 3: only id 2's
/// translation was computed from a staged summary. Id 3's was computed from
/// the published summary the file copies, and stays reusable.
#[tokio::test]
async fn test_deferred_group_defers_only_rows_its_upstream_assigns() {
    let dataset = computed_outputs(3, &CHAIN).await;
    let (summary, translation) = (
        flag_of(&dataset, "summary"),
        flag_of(&dataset, "translation"),
    );
    let mut first = Refresh::default();
    for fragment_id in [0, 1] {
        first
            .stage(
                &dataset,
                fragment_id,
                &[("summary", &[0, 1, 2])],
                &StagedValues::new(),
            )
            .await;
    }
    let summarized = first.publish(&dataset, Reject).await.unwrap().dataset;
    let written = merge_insert_body(&summarized, 1, "new1").await;
    let read = merge_insert_body(&written, 2, "new2").await;
    assert_eq!(
        read.cell_flag_true_rows(summary).unwrap(),
        with_full(rows(&[(0, &[2])]), 1)
    );

    let mut refresh = Refresh::default();
    let outputs: [(&str, &[u32]); 2] = [("summary", &[0, 1]), ("translation", &[1, 2])];
    refresh
        .stage(&read, 0, &outputs, &StagedValues::new())
        .await;
    refresh
        .stage(
            &read,
            1,
            &[("translation", &[0, 1, 2])],
            &StagedValues::new(),
        )
        .await;
    let mut competitor = Refresh::default();
    competitor
        .stage(&read, 0, &[("translation", &[0])], &StagedValues::new())
        .await;
    let competed_at = competitor
        .publish(&read, Reject)
        .await
        .unwrap()
        .dataset
        .version()
        .version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: vec![published(translation, full(&[1]))],
            deferred_rows: vec![deferred(
                translation,
                rows(&[(0, &[1])]),
                UpstreamNotPublished,
                competed_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: vec![
                    published(summary, rows(&[(0, &[0, 1])])),
                    published(translation, rows(&[(0, &[2])])),
                ],
            }],
        }
    );
    assert_eq!(report.reusable_rows(summary), rows(&[(0, &[0, 1])]));
    assert_eq!(
        report.reusable_rows(translation),
        with_full(rows(&[(0, &[2])]), 1)
    );
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    assert_eq!(
        result.dataset.cell_flag_true_rows(summary).unwrap(),
        with_full(rows(&[(0, &[2])]), 1)
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[
            (1, None),
            (2, None),
            (3, Some("summary(b3)")),
            (4, Some("summary(b4)")),
            (5, Some("summary(b5)")),
            (6, Some("summary(b6)")),
        ])
    );
    assert_eq!(
        column_values(&result.dataset, "translation", None).await,
        values(&[
            (1, Some("translation(NULL)")),
            (2, None),
            (3, None),
            (4, Some("translation(summary(b4))")),
            (5, Some("translation(summary(b5))")),
            (6, Some("translation(summary(b6))")),
        ])
    );
}

/// Summary, translation (from summary) and keywords (from translation) staged
/// in one file: each output's rows where its direct upstream is assigned are
/// deferred. Id 1's keywords was computed from the translation the file
/// copies, though its summary is staged, and stays reusable. A follow-up
/// restaging every pending output in one file must still recompute a
/// reusable value where it also publishes the value's input.
#[tokio::test]
async fn test_deferred_group_defers_each_link_of_a_chain() {
    let read = computed_outputs(
        3,
        &[
            ("summary", "body"),
            ("translation", "summary"),
            ("keywords", "translation"),
        ],
    )
    .await;
    let [summary, translation, keywords] =
        ["summary", "translation", "keywords"].map(|column| flag_of(&read, column));
    let mut refresh = Refresh::default();
    let outputs: [(&str, &[u32]); 3] = [
        ("summary", &[0, 1]),
        ("translation", &[1, 2]),
        ("keywords", &[0, 2]),
    ];
    refresh
        .stage(&read, 0, &outputs, &StagedValues::new())
        .await;
    let outputs: [(&str, &[u32]); 3] = [
        ("summary", &[0, 1, 2]),
        ("translation", &[0, 1, 2]),
        ("keywords", &[0, 1, 2]),
    ];
    refresh
        .stage(&read, 1, &outputs, &StagedValues::new())
        .await;
    let mut competitor = Refresh::default();
    competitor
        .stage(&read, 0, &[("keywords", &[1])], &StagedValues::new())
        .await;
    let competed_at = competitor
        .publish(&read, Reject)
        .await
        .unwrap()
        .dataset
        .version()
        .version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: [summary, translation, keywords]
                .map(|flag_id| published(flag_id, full(&[1])))
                .to_vec(),
            deferred_rows: vec![
                deferred(
                    translation,
                    rows(&[(0, &[1])]),
                    UpstreamNotPublished,
                    competed_at
                ),
                deferred(
                    keywords,
                    rows(&[(0, &[2])]),
                    UpstreamNotPublished,
                    competed_at
                ),
            ],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: vec![
                    published(summary, rows(&[(0, &[0, 1])])),
                    published(translation, rows(&[(0, &[2])])),
                    published(keywords, rows(&[(0, &[0])])),
                ],
            }],
        }
    );
    for (flag_id, valid) in [
        (summary, &[0, 1][..]),
        (translation, &[2]),
        (keywords, &[0]),
    ] {
        assert_eq!(
            report.reusable_rows(flag_id),
            with_full(rows(&[(0, valid)]), 1)
        );
    }
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    for flag_id in [summary, translation] {
        assert_eq!(
            result.dataset.cell_flag_true_rows(flag_id).unwrap(),
            full(&[1])
        );
    }
    assert_eq!(
        result.dataset.cell_flag_true_rows(keywords).unwrap(),
        with_full(rows(&[(0, &[1])]), 1)
    );
    assert_eq!(
        column_values(&result.dataset, "translation", None).await,
        values(&[
            (1, None),
            (2, None),
            (3, None),
            (4, Some("translation(summary(b4))")),
            (5, Some("translation(summary(b5))")),
            (6, Some("translation(summary(b6))")),
        ])
    );
    assert_eq!(
        column_values(&result.dataset, "keywords", None).await,
        values(&[
            (1, None),
            (2, Some("keywords(NULL)")),
            (3, None),
            (4, Some("keywords(translation(summary(b4)))")),
            (5, Some("keywords(translation(summary(b5)))")),
            (6, Some("keywords(translation(summary(b6)))")),
        ])
    );

    // The reusable translation of id 3 and keywords of id 1 were computed
    // from inputs this follow-up republishes, so it recomputes them.
    let outputs: [(&str, &[u32]); 3] = [
        ("summary", &[0, 1, 2]),
        ("translation", &[0, 1, 2]),
        ("keywords", &[0, 2]),
    ];
    let mut follow_up = Refresh::default();
    follow_up
        .stage(
            &result.dataset,
            0,
            &outputs,
            &refresh.reusable(report, &result.dataset),
        )
        .await;
    let done = follow_up
        .publish(&result.dataset, Reject)
        .await
        .unwrap()
        .dataset;
    for flag_id in [summary, translation] {
        assert_eq!(done.cell_flag_true_rows(flag_id).unwrap(), full(&[0, 1]));
    }
    // Id 2's keywords was computed from the translation this follow-up
    // replaces, so the translation's publication clears it.
    assert_eq!(
        done.cell_flag_true_rows(keywords).unwrap(),
        with_full(rows(&[(0, &[0, 2])]), 1)
    );
    let summarized = |id: i32| computed("summary", Some(&format!("b{id}")));
    let translated = |id: i32| computed("translation", Some(&summarized(id)));
    let by_id = |value: &dyn Fn(i32) -> Option<String>| -> Vec<(i32, Option<String>)> {
        (1..=6).map(|id| (id, value(id))).collect()
    };
    assert_eq!(
        column_values(&done, "summary", None).await,
        by_id(&|id| Some(summarized(id)))
    );
    assert_eq!(
        column_values(&done, "translation", None).await,
        by_id(&|id| Some(translated(id)))
    );
    assert_eq!(
        column_values(&done, "keywords", None).await,
        by_id(&|id| (id != 2).then(|| computed("keywords", Some(&translated(id)))))
    );
}

/// What reaches fragment 0 while the publication's first attempt writes its
/// manifest, so that only its retry sees it.
#[derive(Debug, Clone, Copy)]
enum DuringRetry {
    /// A translation of id 1 that defers the group, which the first attempt
    /// installed without id 2, whose body had changed.
    Deferral,
    /// A new body for id 2, after the first attempt deferred the group to a
    /// translation of id 1.
    BodyWrite,
}

/// However the attempts split the conflicts, the final report defers the
/// translations computed from staged summaries, reports each row once and
/// leaves nothing chained reusable.
#[rstest]
#[case::deferral(DuringRetry::Deferral)]
#[case::body_write(DuringRetry::BodyWrite)]
#[tokio::test]
async fn test_retry_keeps_chained_outputs_of_a_deferred_group_unreusable(
    #[case] during_retry: DuringRetry,
) {
    let read = computed_outputs(2, &CHAIN).await;
    let (summary, translation) = (flag_of(&read, "summary"), flag_of(&read, "translation"));
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        let outputs: [(&str, &[u32]); 2] = [("summary", &[0, 1]), ("translation", &[0, 1])];
        refresh
            .stage(&read, fragment_id, &outputs, &StagedValues::new())
            .await;
    }
    let translate_id_1 = async |dataset: &Dataset| {
        let mut competitor = Refresh::default();
        competitor
            .stage(dataset, 0, &[("translation", &[0])], &StagedValues::new())
            .await;
        competitor.transaction(dataset)
    };
    let (competitor_read, competitor, written_at, translated_at) = match during_retry {
        DuringRetry::Deferral => {
            let written = merge_insert_body(&read, 2, "new").await;
            let written_at = written.version().version;
            let publication = translate_id_1(&written).await;
            (written, publication, written_at, written_at + 1)
        }
        DuringRetry::BodyWrite => {
            let translated = CommitBuilder::new(Arc::new(read.clone()))
                .execute(translate_id_1(&read).await)
                .await
                .unwrap();
            let translated_at = translated.version().version;
            let source = record_batch!(("id", Int32, [2]), ("body", Utf8, ["new"])).unwrap();
            let write =
                MergeInsertBuilder::try_new(Arc::new(translated.clone()), vec!["id".into()])
                    .unwrap()
                    .when_matched(WhenMatched::UpdateAll)
                    .when_not_matched(WhenNotMatched::DoNothing)
                    .write_mode(MergeInsertWriteMode::RewriteColumns)
                    .try_build()
                    .unwrap()
                    .execute_uncommitted_batches(vec![source])
                    .await
                    .unwrap()
                    .transaction;
            (translated, write, translated_at + 1, translated_at)
        }
    };
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: read.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(competitor_read), competitor))),
    });

    let result = CommitBuilder::new(Arc::new(read.clone()))
        .with_commit_handler(handler.clone())
        .with_dependency_conflict_policy(Skip)
        .execute_with_report(refresh.transaction(&read))
        .await
        .unwrap();
    assert!(handler.competitor.lock().unwrap().is_none());
    let report = &result.report;
    let checked_version = written_at.max(translated_at);
    // Either the first attempt drops id 2, whose body changed, and the retry
    // defers the group, or the first attempt defers the group and the retry
    // finds id 2's summary stale among its valid rows.
    let mut deferred_rows = vec![deferred(
        summary,
        rows(&[(0, &[1])]),
        InputChanged,
        written_at,
    )];
    deferred_rows.extend(match during_retry {
        DuringRetry::Deferral => vec![
            deferred(translation, rows(&[(0, &[1])]), InputChanged, written_at),
            deferred(
                translation,
                rows(&[(0, &[0])]),
                UpstreamNotPublished,
                translated_at,
            ),
        ],
        DuringRetry::BodyWrite => vec![deferred(
            translation,
            rows(&[(0, &[0, 1])]),
            UpstreamNotPublished,
            translated_at,
        )],
    });
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version,
            committed_version: Some(checked_version + 1),
            published: vec![
                published(summary, full(&[1])),
                published(translation, full(&[1]))
            ],
            deferred_rows,
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: translated_at,
                valid_rows: vec![published(summary, rows(&[(0, &[0])]))],
            }],
        }
    );
    assert_eq!(
        report.reusable_rows(summary),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(report.reusable_rows(translation), full(&[1]));
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    assert_eq!(
        result.dataset.cell_flag_true_rows(summary).unwrap(),
        full(&[1])
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(
        column_values(&result.dataset, "translation", None).await,
        values(&[
            (1, Some("translation(NULL)")),
            (2, None),
            (3, Some("translation(summary(b3))")),
            (4, Some("translation(summary(b4))")),
        ])
    );
    assert_eq!(
        column_values(&result.dataset, "body", None).await[1],
        (2, Some("new".to_string()))
    );
}

/// Summary and keywords, both computed from body and neither from the other,
/// share a file that a concurrent result for id 1 defers: both stay reusable,
/// also when that result publishes both outputs, since neither is an input of
/// the other.
#[rstest]
#[case::keywords(&["keywords"])]
#[case::summary_and_keywords(&["summary", "keywords"])]
#[tokio::test]
async fn test_deferred_group_keeps_independent_outputs_reusable(#[case] competing: &[&str]) {
    let read = computed_outputs(2, &[("summary", "body"), ("keywords", "body")]).await;
    let (summary, keywords) = (flag_of(&read, "summary"), flag_of(&read, "keywords"));
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        let outputs: [(&str, &[u32]); 2] = [("summary", &[0, 1]), ("keywords", &[0, 1])];
        refresh
            .stage(&read, fragment_id, &outputs, &StagedValues::new())
            .await;
    }
    let mut competitor = Refresh::default();
    let outputs: Vec<(&str, &[u32])> = competing.iter().map(|column| (*column, &[0][..])).collect();
    competitor
        .stage(&read, 0, &outputs, &StagedValues::new())
        .await;
    let competed_at = competitor
        .publish(&read, Reject)
        .await
        .unwrap()
        .dataset
        .version()
        .version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: both(summary, keywords, full(&[1])),
            deferred_rows: vec![],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: both(summary, keywords, rows(&[(0, &[0, 1])])),
            }],
        }
    );
    for flag_id in [summary, keywords] {
        assert_eq!(
            report.reusable_rows(flag_id),
            with_full(rows(&[(0, &[0, 1])]), 1)
        );
    }
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    let summarized = if competing.contains(&"summary") {
        with_full(rows(&[(0, &[0])]), 1)
    } else {
        full(&[1])
    };
    assert_eq!(
        result.dataset.cell_flag_true_rows(summary).unwrap(),
        summarized
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(keywords).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(
        column_values(&result.dataset, "keywords", None).await,
        values(&[
            (1, Some("keywords(b1)")),
            (2, None),
            (3, Some("keywords(b3)")),
            (4, Some("keywords(b4)")),
        ])
    );

    let done = refresh
        .follow_report(report, &result.dataset, &["summary", "keywords"])
        .await;
    for (flag_id, column) in [(summary, "summary"), (keywords, "keywords")] {
        assert_eq!(done.cell_flag_true_rows(flag_id).unwrap(), full(&[0, 1]));
        assert_visible_values_follow_inputs(&done, column).await;
    }
}

/// Fragment `fragment_id` of `after` shows what it showed at `before`: the
/// same values of `columns` and the same true rows of their flags.
async fn assert_fragment_unchanged(
    before: &Dataset,
    after: &Dataset,
    fragment_id: u32,
    columns: &[&str],
) {
    let on_fragment = |values: BTreeMap<u64, Option<String>>| -> BTreeMap<u64, Option<String>> {
        values
            .into_iter()
            .filter(|(addr, _)| RowAddress::from(*addr).fragment_id() == fragment_id)
            .collect()
    };
    for column in columns {
        assert_eq!(
            on_fragment(values_by_addr(after, column).await),
            on_fragment(values_by_addr(before, column).await),
            "{column} of fragment {fragment_id}"
        );
        let flag_id = flag_of(before, column);
        assert_eq!(
            after
                .cell_flag_true_rows(flag_id)
                .unwrap()
                .get(&fragment_id)
                .cloned(),
            before
                .cell_flag_true_rows(flag_id)
                .unwrap()
                .get(&fragment_id)
                .cloned(),
            "{column} flag of fragment {fragment_id}"
        );
    }
}

/// When the competitor of
/// [`test_deferred_group_defers_rows_whose_upstream_was_republished`] commits.
#[derive(Debug, Clone, Copy)]
enum Republication {
    /// Before the publication's first attempt, whose rebase defers the group.
    Intervening,
    /// While the first attempt writes its manifest: the retry defers the group.
    DuringRetry,
    /// While the first attempt, which deferred the group to a newer keywords
    /// result, writes its manifest: only the check of the deferred group sees
    /// it.
    AfterDeferral,
}

/// A competitor summarizes ids 1 and 2 and translates id 1 from its new
/// summary, in one file. It records no clear of translation on id 1, whose
/// translation it republishes, yet id 1's input changed as much as id 2's:
/// the translations staged for both, from the NULL summary, are stale, and so
/// are their sibling keywords. Id 3 stays reusable. A follow-up that restages
/// the deferred file as the report says keeps the competitor's translation.
#[rstest]
#[case::intervening(Republication::Intervening)]
#[case::during_retry(Republication::DuringRetry)]
#[case::after_deferral(Republication::AfterDeferral)]
#[tokio::test]
async fn test_deferred_group_defers_rows_whose_upstream_was_republished(
    #[case] republication: Republication,
) {
    let read = computed_outputs(
        3,
        &[
            ("summary", "body"),
            ("translation", "summary"),
            ("keywords", "body"),
        ],
    )
    .await;
    let [summary, translation, keywords] =
        ["summary", "translation", "keywords"].map(|column| flag_of(&read, column));
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        let outputs: [(&str, &[u32]); 2] = [("translation", &[0, 1, 2]), ("keywords", &[0, 1, 2])];
        refresh
            .stage(&read, fragment_id, &outputs, &StagedValues::new())
            .await;
    }
    let republish = async |dataset: &Dataset| {
        let mut competitor = Refresh::default();
        let outputs: [(&str, &[u32]); 2] = [("summary", &[0, 1]), ("translation", &[0])];
        competitor
            .stage(dataset, 0, &outputs, &StagedValues::new())
            .await;
        competitor.transaction(dataset)
    };
    let (competitor, deferred_at) = match republication {
        Republication::Intervening => {
            let republished = CommitBuilder::new(Arc::new(read.clone()))
                .execute(republish(&read).await)
                .await
                .unwrap();
            assert!(
                recorded_invalidations(&republished)
                    .await
                    .contains(&CellFlagUpdate {
                        flag_id: translation,
                        value: false,
                        rows: rows(&[(0, &[1])]),
                    })
            );
            (None, republished.version().version)
        }
        Republication::DuringRetry => (
            Some((read.clone(), republish(&read).await)),
            read.version().version + 1,
        ),
        Republication::AfterDeferral => {
            let mut newer = Refresh::default();
            newer
                .stage(&read, 0, &[("keywords", &[2])], &StagedValues::new())
                .await;
            let deferred = newer.publish(&read, Reject).await.unwrap().dataset;
            let competitor = republish(&deferred).await;
            let deferred_at = deferred.version().version;
            (Some((deferred, competitor)), deferred_at)
        }
    };
    let republished_at = match republication {
        Republication::AfterDeferral => deferred_at + 1,
        Republication::Intervening | Republication::DuringRetry => deferred_at,
    };
    let mut commit =
        CommitBuilder::new(Arc::new(read.clone())).with_dependency_conflict_policy(Skip);
    let handler = competitor.map(|(dataset, competitor)| {
        Arc::new(CommitsCompetitorFirst {
            inner: read.commit_handler.clone(),
            competitor: Mutex::new(Some((Arc::new(dataset), competitor))),
        })
    });
    if let Some(handler) = &handler {
        commit = commit.with_commit_handler(handler.clone());
    }

    let result = commit
        .execute_with_report(refresh.transaction(&read))
        .await
        .unwrap();
    if let Some(handler) = handler {
        assert!(handler.competitor.lock().unwrap().is_none());
    }
    let report = &result.report;
    let stale = rows(&[(0, &[0, 1])]);
    let valid = rows(&[(0, &[2])]);
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: republished_at,
            committed_version: Some(republished_at + 1),
            published: vec![
                published(translation, full(&[1])),
                published(keywords, full(&[1]))
            ],
            deferred_rows: vec![
                deferred(translation, stale.clone(), InputChanged, republished_at),
                deferred(keywords, stale, InputChanged, republished_at),
            ],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: deferred_at,
                valid_rows: vec![
                    published(translation, valid.clone()),
                    published(keywords, valid.clone()),
                ],
            }],
        }
    );
    for flag_id in [translation, keywords] {
        assert_eq!(report.reusable_rows(flag_id), with_full(valid.clone(), 1));
    }
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    let head = &result.dataset;
    assert_eq!(
        head.cell_flag_true_rows(summary).unwrap(),
        rows(&[(0, &[0, 1])])
    );
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    let newer_keywords = match republication {
        Republication::AfterDeferral => with_full(rows(&[(0, &[2])]), 1),
        Republication::Intervening | Republication::DuringRetry => full(&[1]),
    };
    assert_eq!(head.cell_flag_true_rows(keywords).unwrap(), newer_keywords);
    assert_eq!(
        column_values(head, "translation", None).await,
        values(&[
            (1, Some("translation(summary(b1))")),
            (2, None),
            (3, None),
            (4, Some("translation(NULL)")),
            (5, Some("translation(NULL)")),
            (6, Some("translation(NULL)")),
        ])
    );
    let competed = head.checkout_version(republished_at).await.unwrap();
    assert_fragment_unchanged(&competed, head, 0, &["summary", "translation", "keywords"]).await;

    let done = refresh
        .follow_report(report, head, &["translation", "keywords"])
        .await;
    for (flag_id, column) in [(translation, "translation"), (keywords, "keywords")] {
        assert_eq!(done.cell_flag_true_rows(flag_id).unwrap(), full(&[0, 1]));
        assert_visible_values_follow_inputs(&done, column).await;
    }
    assert_eq!(
        column_values(&done, "translation", None).await[..3],
        values(&[
            (1, Some("translation(summary(b1))")),
            (2, Some("translation(summary(b2))")),
            (3, Some("translation(NULL)")),
        ])
    );
}

/// Summary was published before the refresh read, and a competitor recomputes
/// id 1's summary, as a newer model would, together with the translation of
/// it. The competitor records no clear, yet the translation staged for id 1
/// from the older summary is stale: a publication gives the output it
/// publishes a new value, whatever its flag was.
#[tokio::test]
async fn test_deferred_group_defers_rows_whose_published_upstream_was_recomputed() {
    let dataset = computed_outputs(2, &CHAIN).await;
    let (summary, translation) = (
        flag_of(&dataset, "summary"),
        flag_of(&dataset, "translation"),
    );
    let mut summaries = Refresh::default();
    summaries
        .stage(&dataset, 0, &[("summary", &[0, 1])], &StagedValues::new())
        .await;
    let read = summaries.publish(&dataset, Reject).await.unwrap().dataset;
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        refresh
            .stage(
                &read,
                fragment_id,
                &[("translation", &[0, 1])],
                &StagedValues::new(),
            )
            .await;
    }
    let recomputed = StagedValues::from([(
        (
            "summary".to_string(),
            u64::from(RowAddress::new_from_parts(0, 0)),
        ),
        Some("summary(b1, v2)".to_string()),
    )]);
    let mut competitor = Refresh::default();
    competitor
        .stage(
            &read,
            0,
            &[("summary", &[0]), ("translation", &[0])],
            &recomputed,
        )
        .await;
    let competed = competitor.publish(&read, Reject).await.unwrap().dataset;
    assert!(recorded_invalidations(&competed).await.is_empty());
    let competed_at = competed.version().version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: vec![published(translation, full(&[1]))],
            deferred_rows: vec![deferred(
                translation,
                rows(&[(0, &[0])]),
                InputChanged,
                competed_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: vec![published(translation, rows(&[(0, &[1])]))],
            }],
        }
    );
    assert_eq!(
        report.reusable_rows(translation),
        with_full(rows(&[(0, &[1])]), 1)
    );
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    let head = &result.dataset;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0]));
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    assert_eq!(
        column_values(head, "translation", None).await,
        values(&[
            (1, Some("translation(summary(b1, v2))")),
            (2, None),
            (3, Some("translation(NULL)")),
            (4, Some("translation(NULL)")),
        ])
    );
    assert_fragment_unchanged(&competed, head, 0, &["summary", "translation"]).await;

    let done = refresh.follow_report(report, head, &["translation"]).await;
    assert_eq!(
        done.cell_flag_true_rows(translation).unwrap(),
        full(&[0, 1])
    );
    assert_visible_values_follow_inputs(&done, "translation").await;
    assert_eq!(
        column_values(&done, "translation", None).await[..2],
        values(&[
            (1, Some("translation(summary(b1, v2))")),
            (2, Some("translation(summary(b2))")),
        ])
    );
}

/// What the competitor of
/// [`test_deferred_group_defers_each_link_a_competitor_republishes`] publishes
/// on fragment 0.
#[derive(Debug, Clone, Copy)]
enum ChainRepublication {
    /// Summary and translation of id 1.
    SummaryAndTranslation,
    /// Translation and keywords of id 3.
    TranslationAndKeywords,
    /// Summary and translation of ids 1 and 3, keywords of id 3.
    WholeChain,
    /// Keywords of id 3, from the translation it already reads.
    KeywordsAlone,
}

impl ChainRepublication {
    fn outputs(self) -> &'static [(&'static str, &'static [u32])] {
        match self {
            Self::SummaryAndTranslation => &[("summary", &[0]), ("translation", &[0])],
            Self::TranslationAndKeywords => &[("translation", &[2]), ("keywords", &[2])],
            Self::WholeChain => &[
                ("summary", &[0, 2]),
                ("translation", &[0, 2]),
                ("keywords", &[2]),
            ],
            Self::KeywordsAlone => &[("keywords", &[2])],
        }
    }

    /// Offsets of fragment 0 whose staged translations and keywords go stale.
    fn stale(self) -> (&'static [u32], &'static [u32]) {
        match self {
            Self::SummaryAndTranslation => (&[0], &[]),
            Self::TranslationAndKeywords => (&[], &[2]),
            Self::WholeChain => (&[0], &[2]),
            Self::KeywordsAlone => (&[], &[]),
        }
    }
}

/// Summary, translation (from summary) and keywords (from translation): the
/// refresh stages translations of ids 1 and 2 and keywords of ids 2 and 3 in
/// one file, and a competitor republishes part of the chain. A staged value
/// is stale where the competitor gave its direct input a new value, whether
/// or not it also republished the value's own output there; a republished
/// output whose input did not change stays reusable. Id 2's keywords was
/// computed from the translation the file stages
/// ([`DeferralReason::UpstreamNotPublished`]).
#[rstest]
#[case::summary_and_translation(ChainRepublication::SummaryAndTranslation)]
#[case::translation_and_keywords(ChainRepublication::TranslationAndKeywords)]
#[case::whole_chain(ChainRepublication::WholeChain)]
#[case::keywords_alone(ChainRepublication::KeywordsAlone)]
#[tokio::test]
async fn test_deferred_group_defers_each_link_a_competitor_republishes(
    #[case] republication: ChainRepublication,
) {
    let (stale_translations, stale_keywords) = republication.stale();
    let read = computed_outputs(
        3,
        &[
            ("summary", "body"),
            ("translation", "summary"),
            ("keywords", "translation"),
        ],
    )
    .await;
    let [translation, keywords] = ["translation", "keywords"].map(|column| flag_of(&read, column));
    let mut refresh = Refresh::default();
    let outputs: [(&str, &[u32]); 2] = [("translation", &[0, 1]), ("keywords", &[1, 2])];
    refresh
        .stage(&read, 0, &outputs, &StagedValues::new())
        .await;
    let outputs: [(&str, &[u32]); 2] = [("translation", &[0, 1, 2]), ("keywords", &[0, 1, 2])];
    refresh
        .stage(&read, 1, &outputs, &StagedValues::new())
        .await;
    let mut competitor = Refresh::default();
    competitor
        .stage(&read, 0, republication.outputs(), &StagedValues::new())
        .await;
    let competed = competitor.publish(&read, Reject).await.unwrap().dataset;
    let competed_at = competed.version().version;

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    let without = |offsets: &[u32], stale: &[u32]| -> Vec<u32> {
        offsets
            .iter()
            .copied()
            .filter(|offset| !stale.contains(offset))
            .collect()
    };
    let mut deferred_rows = Vec::new();
    if !stale_translations.is_empty() {
        deferred_rows.push(deferred(
            translation,
            rows(&[(0, stale_translations)]),
            InputChanged,
            competed_at,
        ));
    }
    if !stale_keywords.is_empty() {
        deferred_rows.push(deferred(
            keywords,
            rows(&[(0, stale_keywords)]),
            InputChanged,
            competed_at,
        ));
    }
    deferred_rows.push(deferred(
        keywords,
        rows(&[(0, &[1])]),
        UpstreamNotPublished,
        competed_at,
    ));
    let valid_translations = without(&[0, 1], stale_translations);
    let valid_keywords = without(&[2], stale_keywords);
    let mut valid_rows = vec![published(translation, rows(&[(0, &valid_translations)]))];
    if !valid_keywords.is_empty() {
        valid_rows.push(published(keywords, rows(&[(0, &valid_keywords)])));
    }
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: competed_at,
            committed_version: Some(competed_at + 1),
            published: vec![
                published(translation, full(&[1])),
                published(keywords, full(&[1]))
            ],
            deferred_rows,
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows,
            }],
        }
    );
    for (flag_id, valid) in [
        (translation, &valid_translations),
        (keywords, &valid_keywords),
    ] {
        let mut reusable = full(&[1]);
        if !valid.is_empty() {
            reusable |= &rows(&[(0, valid)]);
        }
        assert_eq!(report.reusable_rows(flag_id), reusable);
    }
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    assert_fragment_unchanged(
        &competed,
        &result.dataset,
        0,
        &["summary", "translation", "keywords"],
    )
    .await;
    for (flag_id, column) in [(translation, "translation"), (keywords, "keywords")] {
        assert_eq!(
            result.dataset.cell_flag_true_rows(flag_id).unwrap().get(&1),
            Some(&RowAddrSelection::Full),
            "{column} flag of fragment 1"
        );
        assert_visible_values_follow_inputs(&result.dataset, column).await;
    }

    let done = refresh
        .follow_report(report, &result.dataset, &["translation", "keywords"])
        .await;
    for (flag_id, column, assigned) in [
        (translation, "translation", &[0, 1][..]),
        (keywords, "keywords", &[1, 2]),
    ] {
        let true_rows = done.cell_flag_true_rows(flag_id).unwrap();
        for offset in assigned {
            let addr = u64::from(RowAddress::new_from_parts(0, *offset));
            assert!(true_rows.contains(addr), "{column} at row {addr:#x}");
        }
        assert_visible_values_follow_inputs(&done, column).await;
    }
}

/// A dependent flag registered after the refresh read, on a column that was
/// plain there, in [`test_flag_registered_after_the_read_is_resolved`]. A
/// dropped one is dropped again before the publication commits, so that the
/// head does not know it either.
#[derive(Debug, Clone, Copy)]
enum LateRegistration {
    /// On summary, which translation is computed from.
    Upstream,
    DroppedUpstream,
    /// On keywords, which translation is not computed from.
    Unrelated,
    DroppedUnrelated,
}

impl LateRegistration {
    fn column(self) -> &'static str {
        match self {
            Self::Upstream | Self::DroppedUpstream => "summary",
            Self::Unrelated | Self::DroppedUnrelated => "keywords",
        }
    }

    fn is_dropped(self) -> bool {
        matches!(self, Self::DroppedUpstream | Self::DroppedUnrelated)
    }
}

/// Translation is computed from summary. After the refresh reads, a column
/// that was plain there becomes a dependent output, unmasked, so its
/// registration clears nothing, and a competitor publishes id 1's value of it
/// together with id 1's translation. Where that column is summary, id 1's
/// staged translation is stale, although the read version's registry does
/// not know the flag, dropped again or not. Where it is keywords, id 1's
/// translation keeps its input and stays reusable.
#[rstest]
#[case::upstream(LateRegistration::Upstream)]
#[case::dropped_upstream(LateRegistration::DroppedUpstream)]
#[case::unrelated(LateRegistration::Unrelated)]
#[case::dropped_unrelated(LateRegistration::DroppedUnrelated)]
#[tokio::test]
async fn test_flag_registered_after_the_read_is_resolved(#[case] registration: LateRegistration) {
    let mut read = computed_outputs(
        2,
        &[
            ("summary", "body"),
            ("translation", "summary"),
            ("keywords", "body"),
        ],
    )
    .await;
    for column in ["summary", "keywords"] {
        read.drop_cell_flag(column, "ready").await.unwrap();
    }
    let translation = flag_of(&read, "translation");
    let mut refresh = Refresh::default();
    for fragment_id in [0, 1] {
        refresh
            .stage(
                &read,
                fragment_id,
                &[("translation", &[0, 1])],
                &StagedValues::new(),
            )
            .await;
    }
    let late = registration.column();
    let mut registered = read.clone();
    registered
        .register_cell_flag(
            late,
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap();
    assert!(recorded_invalidations(&registered).await.is_empty());
    let mut competitor = Refresh::default();
    competitor
        .stage(
            &registered,
            0,
            &[(late, &[0]), ("translation", &[0])],
            &StagedValues::new(),
        )
        .await;
    let mut competed = competitor
        .publish(&registered, Reject)
        .await
        .unwrap()
        .dataset;
    assert!(recorded_invalidations(&competed).await.is_empty());
    let competed_at = competed.version().version;
    let checked_version = if registration.is_dropped() {
        competed.drop_cell_flag(late, "ready").await.unwrap();
        assert!(recorded_invalidations(&competed).await.is_empty());
        competed_at + 1
    } else {
        competed_at
    };

    let result = refresh.publish(&read, Skip).await.unwrap();
    let report = &result.report;
    let (deferred_rows, valid) = match registration {
        LateRegistration::Upstream | LateRegistration::DroppedUpstream => (
            vec![deferred(
                translation,
                rows(&[(0, &[0])]),
                InputChanged,
                competed_at,
            )],
            rows(&[(0, &[1])]),
        ),
        LateRegistration::Unrelated | LateRegistration::DroppedUnrelated => {
            (vec![], rows(&[(0, &[0, 1])]))
        }
    };
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version,
            committed_version: Some(checked_version + 1),
            published: vec![published(translation, full(&[1]))],
            deferred_rows,
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: competed_at,
                valid_rows: vec![published(translation, valid.clone())],
            }],
        }
    );
    assert_eq!(report.reusable_rows(translation), with_full(valid, 1));
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(&result.dataset, report)
        .await;
    let head = &result.dataset;
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        with_full(rows(&[(0, &[0])]), 1)
    );
    let translated = match registration {
        LateRegistration::Upstream | LateRegistration::DroppedUpstream => {
            "translation(summary(b1))"
        }
        LateRegistration::Unrelated | LateRegistration::DroppedUnrelated => "translation(NULL)",
    };
    assert_eq!(
        column_values(head, "translation", None).await,
        values(&[
            (1, Some(translated)),
            (2, None),
            (3, Some("translation(NULL)")),
            (4, Some("translation(NULL)")),
        ])
    );
    assert_eq!(
        column_values(head, late, None).await[0],
        (1, Some(format!("{late}(b1)")))
    );
    let competed = head.checkout_version(competed_at).await.unwrap();
    assert_fragment_unchanged(&competed, head, 0, &["translation"]).await;

    let done = refresh.follow_report(report, head, &["translation"]).await;
    assert_eq!(
        done.cell_flag_true_rows(translation).unwrap(),
        full(&[0, 1])
    );
    assert_visible_values_follow_inputs(&done, "translation").await;
    assert_eq!(
        column_values(&done, "translation", None).await[0],
        (1, Some(translated.to_string()))
    );
}

/// The flags registered after the refresh of
/// [`test_late_flag_on_an_unrelated_output_changes_no_input`] reads.
#[derive(Debug, Clone, Copy)]
enum LateFlags {
    /// Keywords', which the head knows.
    Keywords,
    /// Keywords', dropped again before the publication commits, so that no
    /// registry the check has knows it.
    KeywordsDropped,
    /// Summary's and then keywords', which the head knows.
    SummaryAndKeywords,
}

impl LateFlags {
    fn columns(self) -> &'static [&'static str] {
        match self {
            Self::Keywords | Self::KeywordsDropped => &["keywords"],
            Self::SummaryAndKeywords => &["summary", "keywords"],
        }
    }
}

/// Where the competitor of
/// [`test_late_flag_on_an_unrelated_output_changes_no_input`] publishes
/// summary. Its file on fragment 0 writes summary either way.
#[derive(Debug, Clone, Copy)]
enum SummaryPublished {
    /// On id 3, of fragment 0.
    SameFragment,
    /// On id 4, of fragment 1, so that fragment 0's summaries are only copied.
    OtherFragment,
}

/// How the refresh of [`test_late_flag_on_an_unrelated_output_changes_no_input`]
/// commits, after the competitor.
#[derive(Debug, Clone, Copy)]
enum LateCommit {
    Reject,
    Skip,
    /// Under `Skip`, with the competitor committed while the first attempt
    /// writes its manifest. That attempt deferred fragment 0's group to a
    /// translation of id 3 published before the late registrations, so only
    /// the check of the deferred group sees the competitor.
    SkipAfterDeferral,
}

impl LateCommit {
    fn policy(self) -> DependencyConflictPolicy {
        match self {
            Self::Reject => Reject,
            Self::Skip | Self::SkipAfterDeferral => Skip,
        }
    }
}

/// Translation is computed from summary. After the refresh reads, keywords,
/// and possibly summary, which were plain there, get dependent flags,
/// unmasked, so that registering them clears nothing. A competitor's
/// file on fragment 0 writes summary and keywords: it publishes id 1's
/// keywords, and summary on id 3 or only on fragment 1. The read version's
/// registry does not know the keywords flag, but the head's resolves it
/// unless it was dropped again, and then no flag's output but summary's can
/// be summary: its flag, which the competitor also publishes, is the field's
/// only dependent flag. Either way the staged translations of ids 1 and 2
/// keep their input: they are published, or stay reusable in their deferred
/// group, alongside the competitor's values and flags.
#[rstest]
#[case::keywords_reject(
    LateFlags::Keywords,
    SummaryPublished::SameFragment,
    LateCommit::Reject
)]
#[case::keywords_skip(LateFlags::Keywords, SummaryPublished::SameFragment, LateCommit::Skip)]
#[case::keywords_dropped_reject(
    LateFlags::KeywordsDropped,
    SummaryPublished::SameFragment,
    LateCommit::Reject
)]
#[case::keywords_dropped_copied_skip(
    LateFlags::KeywordsDropped,
    SummaryPublished::OtherFragment,
    LateCommit::Skip
)]
#[case::summary_and_keywords_skip(
    LateFlags::SummaryAndKeywords,
    SummaryPublished::SameFragment,
    LateCommit::Skip
)]
#[case::summary_and_keywords_copied_reject(
    LateFlags::SummaryAndKeywords,
    SummaryPublished::OtherFragment,
    LateCommit::Reject
)]
#[case::keywords_after_deferral(
    LateFlags::Keywords,
    SummaryPublished::SameFragment,
    LateCommit::SkipAfterDeferral
)]
#[case::summary_and_keywords_after_deferral(
    LateFlags::SummaryAndKeywords,
    SummaryPublished::SameFragment,
    LateCommit::SkipAfterDeferral
)]
#[tokio::test]
async fn test_late_flag_on_an_unrelated_output_changes_no_input(
    #[case] late: LateFlags,
    #[case] summary_published: SummaryPublished,
    #[case] commit: LateCommit,
) {
    let after_deferral = matches!(commit, LateCommit::SkipAfterDeferral);
    let mut read = computed_outputs(
        3,
        &[
            ("summary", "body"),
            ("translation", "summary"),
            ("keywords", "body"),
        ],
    )
    .await;
    for column in late.columns() {
        read.drop_cell_flag(column, "ready").await.unwrap();
    }
    let translation = flag_of(&read, "translation");
    let mut refresh = Refresh::default();
    refresh
        .stage(&read, 0, &[("translation", &[0, 1])], &StagedValues::new())
        .await;
    if after_deferral {
        // Installed, so that the first attempt writes a manifest.
        refresh
            .stage(
                &read,
                1,
                &[("translation", &[0, 1, 2])],
                &StagedValues::new(),
            )
            .await;
    }
    let mut registered = read.clone();
    let deferred_at = if after_deferral {
        let mut newer = Refresh::default();
        newer
            .stage(&read, 0, &[("translation", &[2])], &StagedValues::new())
            .await;
        registered = newer.publish(&read, Reject).await.unwrap().dataset;
        Some(registered.version().version)
    } else {
        None
    };
    for column in late.columns() {
        registered
            .register_cell_flag(
                column,
                "ready",
                CellFlagOptions::default().with_clear_on_write(["body"]),
            )
            .await
            .unwrap();
        assert!(recorded_invalidations(&registered).await.is_empty());
    }
    let (summaries_of_fragment_0, summaries_of_fragment_1): (&[u32], &[u32]) =
        match summary_published {
            SummaryPublished::SameFragment => (&[2], &[]),
            SummaryPublished::OtherFragment => (&[], &[0]),
        };
    let mut competitor = Refresh::default();
    competitor
        .stage(
            &registered,
            0,
            &[("summary", summaries_of_fragment_0), ("keywords", &[0])],
            &StagedValues::new(),
        )
        .await;
    if !summaries_of_fragment_1.is_empty() {
        competitor
            .stage(
                &registered,
                1,
                &[("summary", summaries_of_fragment_1), ("keywords", &[])],
                &StagedValues::new(),
            )
            .await;
    }
    let competed_at = registered.version().version + 1;
    let is_dropped = matches!(late, LateFlags::KeywordsDropped);
    let mut commit_builder =
        CommitBuilder::new(Arc::new(read.clone())).with_dependency_conflict_policy(commit.policy());
    let handler = if after_deferral {
        let handler = Arc::new(CommitsCompetitorFirst {
            inner: read.commit_handler.clone(),
            competitor: Mutex::new(Some((
                Arc::new(registered.clone()),
                competitor.transaction(&registered),
            ))),
        });
        commit_builder = commit_builder.with_commit_handler(handler.clone());
        Some(handler)
    } else {
        let mut competed = competitor
            .publish(&registered, Reject)
            .await
            .unwrap()
            .dataset;
        if is_dropped {
            competed.drop_cell_flag("keywords", "ready").await.unwrap();
            assert!(recorded_invalidations(&competed).await.is_empty());
        }
        None
    };
    let checked_version = if is_dropped {
        competed_at + 1
    } else {
        competed_at
    };

    let result = commit_builder
        .execute_with_report(refresh.transaction(&read))
        .await
        .unwrap();
    if let Some(handler) = handler {
        assert!(handler.competitor.lock().unwrap().is_none());
    }
    let head = &result.dataset;
    let competed = head.checkout_version(competed_at).await.unwrap();
    let summary_cleared = match summary_published {
        SummaryPublished::SameFragment => rows(&[(0, &[2])]),
        SummaryPublished::OtherFragment => rows(&[(1, &[0])]),
    };
    assert_eq!(
        recorded_invalidations(&competed).await,
        cleared(translation, summary_cleared)
    );
    let report = &result.report;
    let staged = rows(&[(0, &[0, 1])]);
    let (published_rows, deferred_groups) = match deferred_at {
        None => (staged.clone(), vec![]),
        Some(deferred_at) => (
            full(&[1]),
            vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh.groups[0].1.clone(),
                reason: NewerResult,
                conflicting_version: deferred_at,
                valid_rows: vec![published(translation, staged.clone())],
            }],
        ),
    };
    assert_eq!(
        *report,
        PublicationReport {
            read_version: read.version().version,
            checked_version,
            committed_version: Some(checked_version + 1),
            published: vec![published(translation, published_rows.clone())],
            deferred_rows: vec![],
            deferred_groups,
        }
    );
    let reusable = staged | published_rows.clone();
    assert_eq!(report.reusable_rows(translation), reusable);
    refresh.assert_report_accounts_for_every_row(&read, report);
    refresh
        .assert_reusable_values_follow_inputs(head, report)
        .await;
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        published_rows
    );
    assert_visible_values_follow_inputs(head, "translation").await;
    let kept: &[&str] = if is_dropped {
        &["summary"]
    } else {
        &["summary", "keywords"]
    };
    for fragment_id in [0, 1] {
        assert_fragment_unchanged(&competed, head, fragment_id, kept).await;
    }
    assert_eq!(
        column_values(head, "keywords", None).await[0],
        (1, Some("keywords(b1)".to_string()))
    );

    let done = if after_deferral {
        assert_fragment_unchanged(&competed, head, 0, &["translation"]).await;
        refresh.follow_report(report, head, &["translation"]).await
    } else {
        head.clone()
    };
    assert_eq!(done.cell_flag_true_rows(translation).unwrap(), reusable);
    assert_visible_values_follow_inputs(&done, "translation").await;
    let translated_fragment_1 = after_deferral.then_some("translation(NULL)");
    assert_eq!(
        column_values(&done, "translation", None).await,
        values(&[
            (1, Some("translation(NULL)")),
            (2, Some("translation(NULL)")),
            (3, None),
            (4, translated_fragment_1),
            (5, translated_fragment_1),
            (6, translated_fragment_1),
        ])
    );
}

/// `summary` (from title and body) and `keywords` (from body) computed
/// together: two dependent flags whose outputs share one file per fragment.
async fn sibling_outputs() -> (Dataset, u32, u32, Vec<DataReplacementGroup>) {
    let mut dataset = articles(false).await;
    let keywords = ArrowSchema::new(vec![ArrowField::new("keywords", DataType::Utf8, true)]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(keywords)), None, None)
        .await
        .unwrap();
    let summary = register_ready(&mut dataset).await;
    let keywords = dataset
        .register_cell_flag(
            "keywords",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap()
        .flag_id;
    let staged = stage_siblings(&dataset, "").await;
    (dataset, summary, keywords, staged)
}

/// One file per fragment holding `{prefix}{column}-{fragment}-{offset}`.
async fn stage_siblings(dataset: &Dataset, prefix: &str) -> Vec<DataReplacementGroup> {
    let mut staged = Vec::new();
    for fragment_id in [0, 1] {
        staged.push(
            stage_rows(
                dataset,
                fragment_id,
                &["summary", "keywords"],
                |column, offset| Some(format!("{prefix}{column}-{fragment_id}-{offset}")),
            )
            .await,
        );
    }
    staged
}

fn both(summary: u32, keywords: u32, rows: RowAddrTreeMap) -> Vec<CellFlagUpdate> {
    vec![published(summary, rows.clone()), published(keywords, rows)]
}

/// A newer refresh of both sibling outputs on `fragment_ids`, committed
/// before the one under test.
async fn publish_newer(
    dataset: &Dataset,
    summary: u32,
    keywords: u32,
    fragment_ids: &[u32],
) -> Dataset {
    let mut staged = Vec::new();
    for fragment_id in fragment_ids {
        staged.push(
            stage_rows(
                dataset,
                u64::from(*fragment_id),
                &["summary", "keywords"],
                |column, offset| Some(format!("newer-{column}-{fragment_id}-{offset}")),
            )
            .await,
        );
    }
    publish(
        dataset,
        staged,
        both(summary, keywords, full(fragment_ids)),
        Reject,
    )
    .await
    .unwrap()
    .dataset
}

/// Why a publication of both sibling outputs fails.
#[derive(Debug, Clone, Copy)]
enum PublicationFailure {
    /// A concurrent write changed id 3's body, and `Reject` refuses it all.
    InputChanged,
    /// The assignment names an offset fragment 1 does not have.
    RowOutOfRange,
    /// As `RowOutOfRange`, under `Skip`, with a newer result on fragment 1
    /// deferring the group that holds the bad offset.
    DeferredRowOutOfRange,
    /// Under `Skip`, summary is assigned on fragment 1, whose file writes only
    /// keywords, and a newer result on fragment 1 defers that group.
    DeferredMissingOutput,
    /// As `RowOutOfRange`, under `Skip`, with newer results deferring every
    /// group.
    EveryGroupDeferred,
}

/// Invalid assignments fail the same way whether or not a concurrent commit
/// would defer the group they are in.
#[rstest]
#[case::input_changed(PublicationFailure::InputChanged)]
#[case::row_out_of_range(PublicationFailure::RowOutOfRange)]
#[case::deferred_row_out_of_range(PublicationFailure::DeferredRowOutOfRange)]
#[case::deferred_missing_output(PublicationFailure::DeferredMissingOutput)]
#[case::every_group_deferred(PublicationFailure::EveryGroupDeferred)]
#[tokio::test]
async fn test_rejected_publication_exposes_nothing(#[case] failure: PublicationFailure) {
    let (dataset, summary, keywords, mut staged) = sibling_outputs().await;
    let out_of_range = with_full(rows(&[(1, &[0, 1, 5])]), 0);
    let (head, newer_fragments, policy, assigned): (_, &[u32], _, _) = match failure {
        PublicationFailure::InputChanged => (
            merge_insert_body(&dataset, 3, "new").await,
            &[],
            Reject,
            full(&[0, 1]),
        ),
        PublicationFailure::RowOutOfRange => (dataset.clone(), &[], Reject, out_of_range),
        PublicationFailure::DeferredRowOutOfRange => (
            publish_newer(&dataset, summary, keywords, &[1]).await,
            &[1],
            Skip,
            out_of_range,
        ),
        PublicationFailure::DeferredMissingOutput => {
            staged[1] = stage_rows(&dataset, 1, &["keywords"], |column, offset| {
                Some(format!("{column}-1-{offset}"))
            })
            .await;
            (
                publish_newer(&dataset, summary, keywords, &[1]).await,
                &[1],
                Skip,
                full(&[0, 1]),
            )
        }
        PublicationFailure::EveryGroupDeferred => (
            publish_newer(&dataset, summary, keywords, &[0, 1]).await,
            &[0, 1],
            Skip,
            out_of_range,
        ),
    };

    let error = publish(&dataset, staged, both(summary, keywords, assigned), policy)
        .await
        .unwrap_err();
    match failure {
        PublicationFailure::InputChanged => {
            assert!(
                matches!(error, Error::RetryableCommitConflict { .. }),
                "{error}"
            );
            assert!(
                error
                    .to_string()
                    .contains("cannot be published on 1 row(s) of fragment 1"),
                "{error}"
            );
        }
        PublicationFailure::RowOutOfRange
        | PublicationFailure::DeferredRowOutOfRange
        | PublicationFailure::EveryGroupDeferred => {
            assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
            assert!(
                error.to_string().contains(&format!(
                    "cell flag {summary} update names offset 5 of fragment 1, which has 2 \
                     physical rows"
                )),
                "{error}"
            );
        }
        PublicationFailure::DeferredMissingOutput => {
            let summary_id = dataset.schema().field("summary").unwrap().id;
            assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
            assert!(
                error.to_string().contains(&format!(
                    "{} is set true on fragment 1, but the DataReplacement has no group for \
                     that fragment whose file writes 'summary' (field id {summary_id})",
                    flag_label(&dataset, summary)
                )),
                "{error}"
            );
        }
    }
    let latest = latest(&head).await;
    assert_eq!(latest.version().version, head.version().version);
    assert_eq!(
        latest.manifest.fragments, head.manifest.fragments,
        "no staged file was installed"
    );
    for (flag_id, column) in [(summary, "summary"), (keywords, "keywords")] {
        assert_eq!(
            latest.cell_flag_true_rows(flag_id).unwrap(),
            full(newer_fragments)
        );
        let expected: Vec<(i32, Option<String>)> = (1..=4)
            .map(|id| {
                let (fragment, offset) = ((id - 1) / 2, (id - 1) % 2);
                let is_newer = newer_fragments.contains(&(fragment as u32));
                (
                    id,
                    is_newer.then(|| format!("newer-{column}-{fragment}-{offset}")),
                )
            })
            .collect();
        assert_eq!(
            column_values(&latest, column, None).await,
            expected,
            "{column} shows only the newer results"
        );
    }
}

/// With `is_recompute`, both outputs are already published and the refresh
/// recomputes them. Id 3's recomputed keywords is then installed over a
/// published value it no longer assigns, so keywords must be cleared there.
#[rstest]
#[tokio::test]
async fn test_stale_row_defers_every_sibling_output(#[values(false, true)] is_recompute: bool) {
    let (mut dataset, summary, keywords, mut staged) = sibling_outputs().await;
    let prefix = if is_recompute { "re-" } else { "" };
    if is_recompute {
        dataset = publish(
            &dataset,
            staged,
            both(summary, keywords, full(&[0, 1])),
            Reject,
        )
        .await
        .unwrap()
        .dataset;
        staged = stage_siblings(&dataset, prefix).await;
    }
    // Clears only summary on id 3, the first row of fragment 1.
    let clear = TransactionBuilder::new(dataset.version().version, update_config())
        .cell_flag_changes(CellFlagChanges {
            updates: vec![CellFlagUpdate {
                flag_id: summary,
                value: false,
                rows: rows(&[(1, &[0])]),
            }],
            ..Default::default()
        })
        .build();
    let cleared_at = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(clear)
        .await
        .unwrap();

    let result = publish(
        &dataset,
        staged,
        both(summary, keywords, full(&[0, 1])),
        Skip,
    )
    .await
    .unwrap();
    let version = cleared_at.version().version;
    let valid = with_full(rows(&[(1, &[1])]), 0);
    assert_eq!(
        result.report.published,
        both(summary, keywords, valid.clone())
    );
    assert_eq!(
        result.report.deferred_rows,
        vec![
            deferred(summary, rows(&[(1, &[0])]), InputChanged, version),
            deferred(keywords, rows(&[(1, &[0])]), InputChanged, version),
        ]
    );
    for (flag_id, column) in [(summary, "summary"), (keywords, "keywords")] {
        assert_eq!(result.dataset.cell_flag_true_rows(flag_id).unwrap(), valid);
        assert_eq!(
            column_values(&result.dataset, column, None).await,
            values(&[
                (1, Some(&format!("{prefix}{column}-0-0"))),
                (2, Some(&format!("{prefix}{column}-0-1"))),
                (3, None),
                (4, Some(&format!("{prefix}{column}-1-1"))),
            ]),
            "id 3's installed {column} reads as NULL"
        );
    }
}

/// Fragment 0's group is deferred to a newer result of both outputs, and only
/// summary is then cleared on id 1, while the publication rebases or while its
/// first attempt writes its manifest. Either way id 1's staged values are
/// deferred for both outputs.
#[rstest]
#[tokio::test]
async fn test_deferred_group_defers_every_sibling_output(
    #[values(false, true)] is_during_retry: bool,
) {
    let (dataset, summary, keywords, staged) = sibling_outputs().await;
    let newer = stage_rows(&dataset, 0, &["summary", "keywords"], |column, offset| {
        Some(format!("newer-{column}-0-{offset}"))
    })
    .await;
    let repaired = publish(
        &dataset,
        vec![newer],
        both(summary, keywords, full(&[0])),
        Reject,
    )
    .await
    .unwrap()
    .dataset;
    let clear = TransactionBuilder::new(repaired.version().version, update_config())
        .cell_flag_changes(CellFlagChanges {
            updates: vec![CellFlagUpdate {
                flag_id: summary,
                value: false,
                rows: rows(&[(0, &[0])]),
            }],
            ..Default::default()
        })
        .build();
    let publication = replacement_txn(
        dataset.version().version,
        staged.clone(),
        both(summary, keywords, full(&[0, 1])),
    );
    let mut commit =
        CommitBuilder::new(Arc::new(dataset.clone())).with_dependency_conflict_policy(Skip);
    let handler = if is_during_retry {
        let handler = Arc::new(CommitsCompetitorFirst {
            inner: repaired.commit_handler.clone(),
            competitor: Mutex::new(Some((Arc::new(repaired.clone()), clear))),
        });
        commit = commit.with_commit_handler(handler.clone());
        Some(handler)
    } else {
        CommitBuilder::new(Arc::new(repaired.clone()))
            .execute(clear)
            .await
            .unwrap();
        None
    };

    let result = commit.execute_with_report(publication).await.unwrap();
    if let Some(handler) = handler {
        assert!(handler.competitor.lock().unwrap().is_none());
    }
    let republished_at = repaired.version().version;
    let cleared_at = republished_at + 1;
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: dataset.version().version,
            checked_version: cleared_at,
            committed_version: Some(cleared_at + 1),
            published: both(summary, keywords, full(&[1])),
            deferred_rows: vec![
                deferred(summary, rows(&[(0, &[0])]), InputChanged, cleared_at),
                deferred(keywords, rows(&[(0, &[0])]), InputChanged, cleared_at),
            ],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: staged[0].1.clone(),
                reason: NewerResult,
                conflicting_version: republished_at,
                valid_rows: both(summary, keywords, rows(&[(0, &[1])])),
            }],
        }
    );
    assert_eq!(
        result.report.reusable_rows(keywords),
        with_full(rows(&[(0, &[1])]), 1)
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(summary).unwrap(),
        with_full(rows(&[(0, &[1])]), 1)
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(keywords).unwrap(),
        full(&[0, 1])
    );
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[
            (1, None),
            (2, Some("newer-summary-0-1")),
            (3, Some("summary-1-0")),
            (4, Some("summary-1-1")),
        ])
    );
    assert_eq!(
        published_values(&result.dataset, "keywords", keywords).await,
        values(&[
            (1, Some("newer-keywords-0-0")),
            (2, Some("newer-keywords-0-1")),
            (3, Some("keywords-1-0")),
            (4, Some("keywords-1-1")),
        ])
    );
}

#[rstest]
#[tokio::test]
async fn test_rows_deleted_before_publication_are_reported_vacated(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    dataset.delete("id = 2").await.unwrap();

    let result = publish(&read, refresh, set_true(flag_id, full(&[0, 1])), policy)
        .await
        .unwrap();
    let valid = with_full(rows(&[(0, &[0])]), 1);
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: dataset.version().version,
            committed_version: Some(dataset.version().version + 1),
            published: vec![published(flag_id, valid.clone())],
            deferred_rows: vec![deferred(
                flag_id,
                rows(&[(0, &[1])]),
                RowVacated,
                dataset.version().version
            )],
            deferred_groups: vec![],
        }
    );
    assert_eq!(result.dataset.cell_flag_true_rows(flag_id).unwrap(), valid);
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[(1, Some("s-0-0")), (3, Some("s-1-0")), (4, Some("s-1-1"))])
    );
}

/// How id 1 leaves offset 0 of a deferred group's fragment in
/// [`test_deferred_group_reports_rows_moved_or_deleted_as_vacated`].
#[derive(Debug, Clone, Copy)]
enum Vacate {
    /// An `UpdateBuilder` update of its body, which moves it to a new address
    /// and records no clear at the old one.
    Update,
    Delete,
    /// A full-row `merge_insert` upsert of its body, which moves it without
    /// listing it as moved, while the publication's first attempt writes its
    /// manifest.
    UpsertDuringRetry,
    /// A delete while the publication's first attempt writes its manifest.
    DeleteDuringRetry,
}

/// Fragment 0's group is deferred to a newer summary of id 2, and then id 1
/// leaves offset 0: moved by a write of its body, or deleted. The staged
/// value there is reported vacated, not reusable, whether the rebase that
/// defers the group sees it or, during the retry, only the check of the
/// deferred group does.
#[rstest]
#[case::update(Vacate::Update)]
#[case::delete(Vacate::Delete)]
#[case::upsert_during_retry(Vacate::UpsertDuringRetry)]
#[case::delete_during_retry(Vacate::DeleteDuringRetry)]
#[tokio::test]
async fn test_deferred_group_reports_rows_moved_or_deleted_as_vacated(#[case] vacate: Vacate) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let newer = stage_rows(&read, 0, &["summary"], |_, offset| {
        (offset == 1).then(|| "newer-0-1".to_string())
    })
    .await;
    let newer = publish(
        &read,
        vec![newer],
        set_true(flag_id, rows(&[(0, &[1])])),
        Reject,
    )
    .await
    .unwrap()
    .dataset;
    let deferred_at = newer.version().version;
    let competitor = match vacate {
        Vacate::Update => {
            update_where(&newer, "id = 1", "body", "new").await;
            None
        }
        Vacate::Delete => {
            newer.clone().delete("id = 1").await.unwrap();
            None
        }
        Vacate::UpsertDuringRetry => {
            let source = RecordBatch::try_from_iter([
                ("id", Arc::new(Int32Array::from(vec![1])) as ArrayRef),
                ("title", Arc::new(StringArray::from(vec!["t1"])) as ArrayRef),
                ("body", Arc::new(StringArray::from(vec!["new"])) as ArrayRef),
                (
                    "summary",
                    Arc::new(StringArray::from(vec![None::<&str>])) as ArrayRef,
                ),
            ])
            .unwrap();
            let upsert = MergeInsertBuilder::try_new(Arc::new(newer.clone()), vec!["id".into()])
                .unwrap()
                .when_matched(WhenMatched::UpdateAll)
                .when_not_matched(WhenNotMatched::DoNothing)
                .try_build()
                .unwrap()
                .execute_uncommitted_batches(vec![source])
                .await
                .unwrap()
                .transaction;
            Some(upsert)
        }
        Vacate::DeleteDuringRetry => Some(
            DeleteBuilder::new(Arc::new(newer.clone()), "id = 1")
                .execute_uncommitted()
                .await
                .unwrap()
                .transaction,
        ),
    };
    let mut commit =
        CommitBuilder::new(Arc::new(read.clone())).with_dependency_conflict_policy(Skip);
    let handler = competitor.map(|competitor| {
        Arc::new(CommitsCompetitorFirst {
            inner: read.commit_handler.clone(),
            competitor: Mutex::new(Some((Arc::new(newer.clone()), competitor))),
        })
    });
    if let Some(handler) = &handler {
        commit = commit.with_commit_handler(handler.clone());
    }

    let result = commit
        .execute_with_report(replacement_txn(
            read.version().version,
            refresh.clone(),
            set_true(flag_id, full(&[0, 1])),
        ))
        .await
        .unwrap();
    if let Some(handler) = handler {
        assert!(handler.competitor.lock().unwrap().is_none());
    }
    let vacated_at = deferred_at + 1;
    assert_eq!(
        result.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: vacated_at,
            committed_version: Some(vacated_at + 1),
            published: vec![published(flag_id, full(&[1]))],
            deferred_rows: vec![deferred(
                flag_id,
                rows(&[(0, &[0])]),
                RowVacated,
                vacated_at
            )],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 0,
                data_file: refresh[0].1.clone(),
                reason: NewerResult,
                conflicting_version: deferred_at,
                valid_rows: vec![published(flag_id, rows(&[(0, &[1])]))],
            }],
        }
    );
    assert_eq!(
        result.report.reusable_rows(flag_id),
        with_full(rows(&[(0, &[1])]), 1)
    );
    assert_eq!(
        result.dataset.cell_flag_true_rows(flag_id).unwrap(),
        with_full(rows(&[(0, &[1])]), 1)
    );
    let mut expected = values(&[
        (2, Some("newer-0-1")),
        (3, Some("s-1-0")),
        (4, Some("s-1-1")),
    ]);
    if matches!(vacate, Vacate::Update | Vacate::UpsertDuringRetry) {
        expected.insert(0, (1, None));
    }
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        expected
    );
}

#[tokio::test]
async fn test_follow_up_refresh_reuses_published_rows() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let first = stage_all(&read, "summary", "first").await;
    merge_insert_body(&dataset, 2, "new").await;
    let result = publish(&read, first, set_true(flag_id, full(&[0, 1])), Skip)
        .await
        .unwrap();
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        values(&[
            (1, Some("first-0-0")),
            (2, None),
            (3, Some("first-1-0")),
            (4, Some("first-1-1"))
        ])
    );
    let report = &result.report;
    let reusable = report.published_rows(flag_id);
    let to_recompute = report.deferred_rows_of(flag_id, InputChanged);
    assert_eq!(to_recompute, rows(&[(0, &[1])]));
    assert!((reusable.clone() & &to_recompute).is_empty());

    // Restage fragment 0 from the committed version: recompute id 2 and reuse
    // id 1's first value. Fragment 1 is complete and is left alone.
    let follow_up_read = latest(&result.dataset).await;
    let follow_up_version = follow_up_read.version().version;
    assert_eq!(Some(follow_up_version), report.committed_version);
    let recomputed = to_recompute.get_fragment_bitmap(0).unwrap().clone();
    let reused = reusable.get_fragment_bitmap(0).unwrap().clone();
    let group = stage_rows(&follow_up_read, 0, &["summary"], |_, offset| {
        let offset = offset as u32;
        if recomputed.contains(offset) {
            Some(format!("second-0-{offset}"))
        } else {
            reused.contains(offset).then(|| format!("first-0-{offset}"))
        }
    })
    .await;
    let mut assigned = RowAddrTreeMap::new();
    assigned.insert_bitmap(0, &recomputed | &reused);
    let follow_up = publish(
        &follow_up_read,
        vec![group],
        set_true(flag_id, assigned.clone()),
        Skip,
    )
    .await
    .unwrap();
    assert_eq!(
        follow_up.report,
        PublicationReport {
            read_version: follow_up_version,
            checked_version: follow_up_version,
            committed_version: Some(follow_up_version + 1),
            published: vec![published(flag_id, assigned)],
            deferred_rows: vec![],
            deferred_groups: vec![],
        }
    );
    assert_eq!(
        follow_up.dataset.cell_flag_true_rows(flag_id).unwrap(),
        full(&[0, 1])
    );
    assert_eq!(
        published_values(&follow_up.dataset, "summary", flag_id).await,
        values(&[
            (1, Some("first-0-0")),
            (2, Some("second-0-1")),
            (3, Some("first-1-0")),
            (4, Some("first-1-1")),
        ])
    );
}

/// One live row as the masked scan reads it.
struct Article {
    id: i32,
    title: String,
    body: String,
    language: String,
    summary: Option<String>,
    translation: Option<String>,
}

impl Article {
    fn output(&self, column: &str) -> Option<&str> {
        match column {
            "summary" => self.summary.as_deref(),
            _ => self.translation.as_deref(),
        }
    }
}

type Compute = fn(&Article) -> String;

fn summary_of(article: &Article) -> String {
    format!("sum({},{})", article.title, article.body)
}

fn translation_of(article: &Article) -> String {
    format!("tr({},{})", article.body, article.language)
}

/// The computed columns of the refresh loop, with the functions, unknown to
/// Lance, that compute them.
const OUTPUTS: [(&str, Compute); 2] = [("summary", summary_of), ("translation", translation_of)];

/// Every live row of `dataset`, by row address.
async fn articles_by_addr(dataset: &Dataset) -> BTreeMap<u64, Article> {
    let batch = dataset
        .scan()
        .project(&["id", "title", "body", "language", "summary", "translation"])
        .unwrap()
        .with_row_address()
        .try_into_batch()
        .await
        .unwrap();
    let text = |column: &str, row: usize| {
        let array = batch[column].as_string::<i32>();
        array.is_valid(row).then(|| array.value(row).to_string())
    };
    let ids = batch["id"].as_primitive::<Int32Type>();
    let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
    (0..batch.num_rows())
        .map(|row| {
            let article = Article {
                id: ids.value(row),
                title: text("title", row).unwrap(),
                body: text("body", row).unwrap(),
                language: text("language", row).unwrap(),
                summary: text("summary", row),
                translation: text("translation", row),
            };
            (addrs.value(row), article)
        })
        .collect()
}

/// The live rows of `read` whose `column` flag is false: a refresh's work.
async fn pending_articles(read: &Dataset, column: &str) -> BTreeMap<u64, Article> {
    let flag_id = read.cell_flag(column, "ready").unwrap().flag_id;
    let true_rows = read.cell_flag_true_rows(flag_id).unwrap();
    let mut articles = articles_by_addr(read).await;
    articles.retain(|addr, _| !true_rows.contains(*addr));
    articles
}

/// Stage, against `read`, one full-fragment `column` file for every fragment
/// `outputs` names: `outputs` on the rows it keys, every other row copied
/// through as `read` shows it. Returns the groups and the rows they assign.
async fn stage_outputs(
    read: &Dataset,
    column: &str,
    outputs: &BTreeMap<u64, String>,
) -> (Vec<DataReplacementGroup>, RowAddrTreeMap) {
    let shown: BTreeMap<u64, Option<String>> = articles_by_addr(read)
        .await
        .into_iter()
        .map(|(addr, article)| (addr, article.output(column).map(str::to_string)))
        .collect();
    let fragments: BTreeSet<u32> = outputs
        .keys()
        .map(|addr| RowAddress::from(*addr).fragment_id())
        .collect();
    let mut groups = Vec::new();
    for fragment_id in fragments {
        let value = |_: &str, offset: u64| {
            let addr = u64::from(RowAddress::new_from_parts(fragment_id, offset as u32));
            outputs
                .get(&addr)
                .cloned()
                .or_else(|| shown.get(&addr).cloned().flatten())
        };
        groups.push(stage_rows(read, u64::from(fragment_id), &[column], value).await);
    }
    let mut assigned = RowAddrTreeMap::new();
    for addr in outputs.keys() {
        assigned.insert(*addr);
    }
    (groups, assigned)
}

/// Every output of `dataset` reads the function of its row's inputs at that
/// version where its flag is true, and NULL where it is false.
async fn assert_outputs_follow_inputs(dataset: &Dataset) {
    let version = dataset.version().version;
    let articles = articles_by_addr(dataset).await;
    for (column, compute) in OUTPUTS {
        let true_rows = dataset
            .cell_flag(column, "ready")
            .map(|flag| dataset.cell_flag_true_rows(flag.flag_id).unwrap())
            .unwrap_or_default();
        for (addr, article) in &articles {
            let expected = true_rows.contains(*addr).then(|| compute(article));
            assert_eq!(
                article.output(column),
                expected.as_deref(),
                "{column} of id {} at version {version}",
                article.id
            );
        }
    }
}

/// The whole refresh loop over three fragments, with `summary` computed from
/// title and body and `translation` from body and language. Both refreshes
/// read one snapshot. While they run, sparse source writes land through
/// `UpdateBuilder` and partial merge_insert, and a priority refresh of one
/// row publishes. Both publish under `Skip`; follow-up refreshes compute only
/// what the reports leave pending and reuse the staged values they report as
/// still valid. No version ever shows an output that is not the function of
/// the inputs shown with it.
#[rstest]
#[tokio::test]
async fn test_refresh_loop_never_shows_a_stale_output(#[values(false, true)] stable_row_ids: bool) {
    let text = |prefix: &str| -> ArrayRef {
        Arc::new(StringArray::from_iter_values(
            (1..=9).map(|id| format!("{prefix}{id}")),
        ))
    };
    let languages: ArrayRef = Arc::new(StringArray::from_iter_values(
        ["en", "fr", "de"].into_iter().cycle().take(9),
    ));
    let batch = RecordBatch::try_from_iter([
        (
            "id",
            Arc::new(Int32Array::from_iter_values(1..=9)) as ArrayRef,
        ),
        ("title", text("t")),
        ("body", text("b")),
        ("language", languages),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            max_rows_per_file: 3,
            enable_stable_row_ids: stable_row_ids,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let outputs = ArrowSchema::new(vec![
        ArrowField::new("summary", DataType::Utf8, true),
        ArrowField::new("translation", DataType::Utf8, true),
    ]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(outputs)), None, None)
        .await
        .unwrap();
    let first_version = dataset.version().version;
    let masked_on = |sources: &[&str]| {
        CellFlagOptions::default()
            .with_clear_on_write(sources.iter().copied())
            .with_mask_when_false(true)
    };
    let summary = dataset
        .register_cell_flag("summary", "ready", masked_on(&["title", "body"]))
        .await
        .unwrap()
        .flag_id;
    let translation = dataset
        .register_cell_flag("translation", "ready", masked_on(&["body", "language"]))
        .await
        .unwrap()
        .flag_id;

    let read = dataset.clone();
    let computed = |articles: BTreeMap<u64, Article>, compute: Compute| {
        articles
            .into_iter()
            .map(|(addr, article)| (addr, compute(&article)))
            .collect::<BTreeMap<u64, String>>()
    };
    let summaries = computed(pending_articles(&read, "summary").await, summary_of);
    let translations = computed(pending_articles(&read, "translation").await, translation_of);
    assert_eq!((summaries.len(), translations.len()), (9, 9));
    let (summary_groups, summary_rows) = stage_outputs(&read, "summary", &summaries).await;
    let (translation_groups, translation_rows) =
        stage_outputs(&read, "translation", &translations).await;

    // Id 2 (fragment 0) gets a new body, which moves it; id 5 (fragment 1) a
    // new language and id 8 (fragment 2) a new title, in place.
    let moved_at = update_where(&dataset, "id = 2", "body", "b2-edited").await;
    let language_at = merge_insert_column(&moved_at, 5, "language", "es").await;
    let title_at = merge_insert_column(&language_at, 8, "title", "t8-edited").await;
    let id_4 = rows(&[(1, &[0])]);
    let mut priority = pending_articles(&title_at, "summary").await;
    priority.retain(|addr, _| id_4.contains(*addr));
    let (groups, assigned) =
        stage_outputs(&title_at, "summary", &computed(priority, summary_of)).await;
    let prioritized = publish(&title_at, groups, set_true(summary, assigned), Skip)
        .await
        .unwrap();
    let prioritized_at = title_at.version().version + 1;
    assert_eq!(prioritized.report.committed_version, Some(prioritized_at));
    assert_eq!(prioritized.report.published, vec![published(summary, id_4)]);

    let summarized = publish(
        &read,
        summary_groups.clone(),
        set_true(summary, summary_rows),
        Skip,
    )
    .await
    .unwrap();
    let summarized_at = prioritized_at + 1;
    assert_eq!(
        summarized.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: prioritized_at,
            committed_version: Some(summarized_at),
            published: vec![published(summary, rows(&[(0, &[0, 2]), (2, &[0, 2])]))],
            deferred_rows: vec![
                deferred(
                    summary,
                    rows(&[(0, &[1])]),
                    RowVacated,
                    moved_at.version().version
                ),
                deferred(
                    summary,
                    rows(&[(2, &[1])]),
                    InputChanged,
                    title_at.version().version
                ),
            ],
            deferred_groups: vec![DeferredGroup {
                fragment_id: 1,
                data_file: summary_groups[1].1.clone(),
                reason: NewerResult,
                conflicting_version: prioritized_at,
                valid_rows: vec![published(summary, rows(&[(1, &[0, 1, 2])]))],
            }],
        }
    );
    let translated = publish(
        &read,
        translation_groups,
        set_true(translation, translation_rows),
        Skip,
    )
    .await
    .unwrap();
    assert_eq!(
        translated.report,
        PublicationReport {
            read_version: read.version().version,
            checked_version: summarized_at,
            committed_version: Some(summarized_at + 1),
            published: vec![published(
                translation,
                rows(&[(0, &[0, 2]), (1, &[0, 2]), (2, &[0, 1, 2])])
            )],
            deferred_rows: vec![
                deferred(
                    translation,
                    rows(&[(0, &[1])]),
                    RowVacated,
                    moved_at.version().version
                ),
                deferred(
                    translation,
                    rows(&[(1, &[1])]),
                    InputChanged,
                    language_at.version().version
                ),
            ],
            deferred_groups: vec![],
        }
    );

    // Both follow-ups read the latest version. Each recomputes only the
    // pending rows its report does not list as reusable: the rows whose inputs
    // changed and id 2, which moved to a fresh address.
    let head = latest(&translated.dataset).await;
    let mut follow_ups = Vec::new();
    for ((column, compute), (flag_id, staged, report), (reused_ids, computed_ids)) in [
        (
            OUTPUTS[0],
            (summary, &summaries, &summarized.report),
            ([5, 6].as_slice(), [2, 8].as_slice()),
        ),
        (
            OUTPUTS[1],
            (translation, &translations, &translated.report),
            ([].as_slice(), [2, 5].as_slice()),
        ),
    ] {
        let reusable = report.reusable_rows(flag_id);
        let recompute = report.deferred_rows_of(flag_id, InputChanged);
        let mut outputs = BTreeMap::new();
        let (mut reused, mut recomputed) = (BTreeSet::new(), BTreeSet::new());
        for (addr, article) in pending_articles(&head, column).await {
            if reusable.contains(addr) {
                outputs.insert(addr, staged[&addr].clone());
                reused.insert(article.id);
            } else {
                assert!(
                    recompute.contains(addr) || RowAddress::from(addr).fragment_id() == 3,
                    "{column} of id {} is neither reported nor moved",
                    article.id
                );
                outputs.insert(addr, compute(&article));
                recomputed.insert(article.id);
            }
        }
        assert_eq!(reused, reused_ids.iter().copied().collect(), "{column}");
        assert_eq!(
            recomputed,
            computed_ids.iter().copied().collect(),
            "{column}"
        );
        let (groups, assigned) = stage_outputs(&head, column, &outputs).await;
        follow_ups.push((flag_id, groups, assigned));
    }
    let mut dataset = head.clone();
    for (flag_id, groups, assigned) in follow_ups {
        let result = publish(&head, groups, set_true(flag_id, assigned.clone()), Skip)
            .await
            .unwrap();
        assert_eq!(result.report.published, vec![published(flag_id, assigned)]);
        assert!(result.report.deferred_rows.is_empty());
        assert!(result.report.deferred_groups.is_empty());
        dataset = result.dataset;
    }

    let articles = articles_by_addr(&dataset).await;
    assert_eq!(articles.len(), 9);
    for flag_id in [summary, translation] {
        let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
        assert!(
            articles.keys().all(|addr| true_rows.contains(*addr)),
            "flag {flag_id} is true on every row"
        );
    }
    for version in first_version..=dataset.version().version {
        assert_outputs_follow_inputs(&dataset.checkout_version(version).await.unwrap()).await;
    }
}

#[derive(Debug, Clone, Copy)]
enum IneligibleSkip {
    Execute,
    SourceWrite,
    OrdinaryFlag,
    ExtraField,
}

#[rstest]
#[case::execute(IneligibleSkip::Execute, "use it instead of CommitBuilder::execute")]
#[case::source_write(IneligibleSkip::SourceWrite, "this DataReplacement sets no cell flag")]
#[case::ordinary_flag(IneligibleSkip::OrdinaryFlag, "which is ordinary, to true")]
#[case::extra_field(
    IneligibleSkip::ExtraField,
    "writes 'title' (field id 1), which is not the output of a flag it publishes"
)]
#[tokio::test]
async fn test_skip_needs_a_publication_and_a_report(
    #[case] transaction: IneligibleSkip,
    #[case] expected: &str,
) {
    let mut dataset = articles(false).await;
    let ready = register_ready(&mut dataset).await;
    let reviewed = dataset
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap()
        .flag_id;
    let version = dataset.version().version;
    let builder =
        CommitBuilder::new(Arc::new(dataset.clone())).with_dependency_conflict_policy(Skip);
    let error = match transaction {
        IneligibleSkip::Execute => {
            let staged = stage_all(&dataset, "summary", "s").await;
            builder
                .execute(replacement_txn(
                    version,
                    staged,
                    set_true(ready, full(&[0, 1])),
                ))
                .await
                .unwrap_err()
        }
        IneligibleSkip::SourceWrite => {
            let body = stage_all(&dataset, "body", "b").await;
            builder
                .execute_with_report(replacement_txn(version, body, vec![]))
                .await
                .unwrap_err()
        }
        IneligibleSkip::OrdinaryFlag => {
            let staged = stage_all(&dataset, "summary", "s").await;
            let mut updates = set_true(ready, full(&[0, 1]));
            updates.extend(set_true(reviewed, full(&[0])));
            builder
                .execute_with_report(replacement_txn(version, staged, updates))
                .await
                .unwrap_err()
        }
        IneligibleSkip::ExtraField => {
            let group = stage_rows(&dataset, 0, &["summary", "title"], |column, offset| {
                Some(format!("{column}-{offset}"))
            })
            .await;
            builder
                .execute_with_report(replacement_txn(
                    version,
                    vec![group],
                    set_true(ready, full(&[0])),
                ))
                .await
                .unwrap_err()
        }
    };
    assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    assert!(error.to_string().contains(expected), "{error}");
    assert_eq!(latest(&dataset).await.version().version, version);
}

#[rstest]
#[case::same_row(&[1], true)]
#[case::other_row(&[0], false)]
#[tokio::test]
async fn test_explicit_updates_of_the_same_rows_conflict(
    #[case] other_offsets: &[u32],
    #[case] is_conflict: bool,
) {
    let mut dataset = articles(false).await;
    let reviewed = dataset
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap()
        .flag_id;
    let explicit = |dataset: &Dataset, value: bool, offsets: &[u32]| {
        TransactionBuilder::new(dataset.version().version, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: reviewed,
                    value,
                    rows: rows(&[(0, offsets)]),
                }],
                ..Default::default()
            })
            .build()
    };
    let staged = explicit(&dataset, true, &[1]);
    let other = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(explicit(&dataset, false, other_offsets))
        .await
        .unwrap();

    let result = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(staged)
        .await;
    if is_conflict {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        let expected = format!(
            "preempted by concurrent transaction UpdateConfig at version {}, which also updated \
             cell flag {reviewed} on rows of fragment(s) [0] that this transaction sets to true",
            other.version().version
        );
        assert!(error.to_string().contains(&expected), "{error}");
        let head = latest(&other).await;
        assert_eq!(head.version().version, other.version().version);
        assert!(head.cell_flag_true_rows(reviewed).unwrap().is_empty());
    } else {
        assert_eq!(
            result.unwrap().cell_flag_true_rows(reviewed).unwrap(),
            rows(&[(0, &[1])])
        );
    }
}

fn addr(fragment_id: u32, offset: u32) -> u64 {
    RowAddress::new_from_parts(fragment_id, offset).into()
}

fn text<'a>(values: impl IntoIterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from_iter(values))
}

/// `[_rowaddr, columns...]` on `addrs`.
fn addressed(addrs: &[u64], columns: Vec<(&str, ArrayRef)>) -> RecordBatch {
    let mut all = vec![(
        ROW_ADDR,
        Arc::new(UInt64Array::from(addrs.to_vec())) as ArrayRef,
    )];
    all.extend(columns);
    RecordBatch::try_from_iter(all).unwrap()
}

/// A computed batch on `addrs` carrying `outputs`, in dependency order with
/// the rows each assigns. An assigned value is computed from its input as the
/// batch assigns it on the row, or as `read` shows it; every other cell holds
/// `WRONG`, which a stager must never publish.
async fn computed_batch(
    read: &Dataset,
    addrs: &[u64],
    outputs: &[(&str, &[u64])],
) -> ComputedBatch {
    let mut columns: Vec<(&str, Vec<Option<String>>)> = Vec::new();
    let mut masks = Vec::new();
    for (column, assigned) in outputs {
        let input = input_of(read, column);
        let shown = values_by_addr(read, &input).await;
        let upstream = columns
            .iter()
            .zip(outputs)
            .find(|((output, _), _)| *output == input);
        let mut column_values = Vec::with_capacity(addrs.len());
        for (row, addr) in addrs.iter().enumerate() {
            if !assigned.contains(addr) {
                column_values.push(Some("WRONG".to_string()));
                continue;
            }
            let input_value = match upstream {
                Some(((_, upstream_values), (_, upstream_rows)))
                    if upstream_rows.contains(addr) =>
                {
                    upstream_values[row].clone()
                }
                _ => shown[addr].clone(),
            };
            column_values.push(Some(computed(column, input_value.as_deref())));
        }
        masks.push(BooleanBuffer::collect_bool(addrs.len(), |row| {
            assigned.contains(&addrs[row])
        }));
        columns.push((column, column_values));
    }
    let arrays = columns
        .iter()
        .map(|(column, column_values)| {
            (
                *column,
                Arc::new(StringArray::from(column_values.clone())) as ArrayRef,
            )
        })
        .collect();
    let mut batch = ComputedBatch::new(addressed(addrs, arrays));
    for ((column, _), mask) in outputs.iter().zip(masks) {
        if mask.count_set_bits() < addrs.len() {
            batch = batch.with_assigned(*column, mask);
        }
    }
    batch
}

/// The addresses of every live row of `dataset`, in scan order.
async fn live_addrs(dataset: &Dataset) -> Vec<u64> {
    values_by_addr(dataset, "body").await.into_keys().collect()
}

fn stager_of(dataset: &Dataset, outputs: &[&str]) -> PublicationStager {
    PublicationStager::try_new(Arc::new(dataset.clone()), outputs).unwrap()
}

async fn stage_batches(
    stager: &PublicationStager,
    batches: Vec<ComputedBatch>,
) -> Result<Option<Transaction>> {
    stager
        .stage(stream::iter(batches.into_iter().map(Ok)))
        .await
}

async fn commit_staged(
    read: &Dataset,
    transaction: Transaction,
    policy: DependencyConflictPolicy,
) -> Result<PublicationResult> {
    CommitBuilder::new(Arc::new(read.clone()))
        .with_dependency_conflict_policy(policy)
        .execute_with_report(transaction)
        .await
}

/// Stage `outputs` of `read` on `addrs`, each assigned on every one, and
/// commit them under `Reject`.
async fn publish_computed(read: &Dataset, outputs: &[&str], addrs: &[u64]) -> Dataset {
    let assigned: Vec<(&str, &[u64])> = outputs.iter().map(|output| (*output, addrs)).collect();
    let batch = computed_batch(read, addrs, &assigned).await;
    let staged = stage_batches(&stager_of(read, outputs), vec![batch])
        .await
        .unwrap()
        .unwrap();
    commit_staged(read, staged, Reject).await.unwrap().dataset
}

fn groups_of(transaction: &Transaction) -> &[DataReplacementGroup] {
    let Operation::DataReplacement { replacements } = &transaction.operation else {
        panic!("not a DataReplacement: {}", transaction.operation);
    };
    replacements
}

fn updates_of(transaction: &Transaction) -> Vec<CellFlagUpdate> {
    transaction
        .cell_flag_changes
        .as_ref()
        .unwrap()
        .updates
        .clone()
}

/// Every path under `dataset`'s data directory.
async fn data_files(dataset: &Dataset) -> BTreeSet<String> {
    dataset
        .object_store
        .read_dir_all(&dataset.data_dir(), None)
        .map_ok(|meta| meta.location.to_string())
        .try_collect()
        .await
        .unwrap()
}

/// How a refresh batches the rows it scanned.
#[derive(Debug, Clone, Copy)]
enum Batching {
    OneBatch,
    RowPerBatch,
    /// Two rows per batch, so a fragment's first batch can assign nothing.
    TwoRowBatches,
    /// One batch per fragment, the last fragment first.
    ReverseFragments,
}

impl Batching {
    fn split(self, addrs: &[u64]) -> Vec<Vec<u64>> {
        match self {
            Self::OneBatch => vec![addrs.to_vec()],
            Self::RowPerBatch => addrs.chunks(1).map(<[u64]>::to_vec).collect(),
            Self::TwoRowBatches => addrs.chunks(2).map(<[u64]>::to_vec).collect(),
            Self::ReverseFragments => {
                let mut by_fragment: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
                for addr in addrs {
                    by_fragment
                        .entry(RowAddress::from(*addr).fragment_id())
                        .or_default()
                        .push(*addr);
                }
                by_fragment.into_values().rev().collect()
            }
        }
    }
}

/// A refresh reads a version where some rows are complete and one is stale,
/// scans every live row, and assigns only some of the pending ones. Every
/// other cell is copied: complete values survive, the stale value stays as
/// stored under its false flag, and no carried placeholder is published.
/// However the scan is batched, the same publication is staged.
#[rstest]
#[tokio::test]
async fn test_stager_incremental_refresh_copies_completed_rows(
    #[values(false, true)] stable_row_ids: bool,
    #[values(false, true)] is_masked: bool,
    #[values(
        Batching::OneBatch,
        Batching::RowPerBatch,
        Batching::TwoRowBatches,
        Batching::ReverseFragments
    )]
    batching: Batching,
) {
    let dataset =
        computed_outputs_with(3, 3, &[("summary", "body")], is_masked, stable_row_ids).await;
    let summary = flag_of(&dataset, "summary");
    let completed = [(0, 0), (0, 1), (1, 0), (1, 1), (1, 2), (2, 0), (2, 1)]
        .map(|(fragment_id, offset)| addr(fragment_id, offset));
    let completed = publish_computed(&dataset, &["summary"], &completed).await;
    // Id 7's body changes in place, so its published summary goes stale.
    let read = merge_insert_body(&completed, 7, "new").await;
    assert_eq!(
        read.cell_flag_true_rows(summary).unwrap(),
        with_full(rows(&[(0, &[0, 1]), (2, &[1])]), 1)
    );

    // Ids 3 and 9 are assigned; id 7 stays pending, and fragment 1 is complete.
    let assigned = [addr(0, 2), addr(2, 2)];
    let mut batches = Vec::new();
    for scanned in batching.split(&live_addrs(&read).await) {
        batches.push(computed_batch(&read, &scanned, &[("summary", &assigned)]).await);
    }
    let staged = stage_batches(&stager_of(&read, &["summary"]), batches)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(staged.read_version, read.version().version);
    let groups = groups_of(&staged);
    assert_eq!(
        groups
            .iter()
            .map(|DataReplacementGroup(fragment_id, _)| *fragment_id)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(groups[0].1.fields, groups[1].1.fields);
    let updates = updates_of(&staged);
    assert_eq!(
        updates,
        vec![published(summary, rows(&[(0, &[2]), (2, &[2])]))]
    );

    let result = commit_staged(&read, staged, Reject).await.unwrap();
    assert_eq!(result.report.published, updates);
    assert_eq!(
        result.dataset.cell_flag_true_rows(summary).unwrap(),
        with_full(rows(&[(2, &[1, 2])]), 0) | full(&[1])
    );
    let stale = (!is_masked).then(|| computed("summary", Some("b7")));
    let expected: Vec<(i32, Option<String>)> = (1..=9)
        .map(|id| match id {
            7 => (id, stale.clone()),
            _ => (id, Some(computed("summary", Some(&format!("b{id}"))))),
        })
        .collect();
    assert_eq!(
        column_values(&result.dataset, "summary", None).await,
        expected
    );
}

/// A NULL on an assigned row is published under a true flag; a cell the
/// batch carries unassigned keeps its value and its flag, true or false.
#[tokio::test]
async fn test_stager_distinguishes_computed_null_from_unassigned() {
    let dataset = computed_outputs(2, &[("summary", "body")]).await;
    let summary = flag_of(&dataset, "summary");
    let read = publish_computed(&dataset, &["summary"], &[addr(0, 0)]).await;

    let batch = addressed(
        &[addr(0, 0), addr(0, 1), addr(1, 0)],
        vec![("summary", text([Some("WRONG"), None, Some("WRONG")]))],
    );
    let assigned = BooleanBuffer::from(vec![false, true, false]);
    let staged = stage_batches(
        &stager_of(&read, &["summary"]),
        vec![ComputedBatch::new(batch).with_assigned("summary", assigned)],
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(groups_of(&staged).len(), 1);
    assert_eq!(
        updates_of(&staged),
        vec![published(summary, rows(&[(0, &[1])]))]
    );
    let head = commit_staged(&read, staged, Reject).await.unwrap().dataset;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0]));
    let summary_1 = computed("summary", Some("b1"));
    assert_eq!(
        published_values(&head, "summary", summary).await,
        values(&[(1, Some(&summary_1)), (2, None)])
    );
    assert_eq!(
        column_values(&head, "summary", None).await,
        values(&[(1, Some(&summary_1)), (2, None), (3, None), (4, None)])
    );
}

/// A dependent output declared non-nullable takes no computed NULL, and
/// ignores one on a row it does not assign.
#[rstest]
#[case::assigned(true)]
#[case::unassigned(false)]
#[tokio::test]
async fn test_stager_refuses_null_for_a_non_nullable_output(#[case] is_assigned: bool) {
    let schema = Arc::new(ArrowSchema::new(vec![
        ArrowField::new("id", DataType::Int32, false),
        ArrowField::new("body", DataType::Utf8, true),
        ArrowField::new("code", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int32Array::from(vec![1, 2])),
            text([Some("b1"), Some("b2")]),
            text([Some("c1"), Some("c2")]),
        ],
    )
    .unwrap();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        None,
    )
    .await
    .unwrap();
    let code = dataset
        .register_cell_flag(
            "code",
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap()
        .flag_id;
    let before = data_files(&dataset).await;
    let computed_rows = ComputedBatch::new(addressed(
        &[addr(0, 0), addr(0, 1)],
        vec![("code", text([Some("code(b1)"), None]))],
    ))
    .with_assigned("code", BooleanBuffer::from(vec![true, is_assigned]));

    let staged = stage_batches(&stager_of(&dataset, &["code"]), vec![computed_rows]).await;
    if is_assigned {
        let error = staged.unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(
            error.to_string().contains(&format!(
                "output 'code' is not nullable, but the computed batch assigns NULL at row {:#x}",
                addr(0, 1)
            )),
            "{error}"
        );
        assert_eq!(data_files(&dataset).await, before);
        return;
    }
    let head = commit_staged(&dataset, staged.unwrap().unwrap(), Reject)
        .await
        .unwrap()
        .dataset;
    assert_eq!(head.cell_flag_true_rows(code).unwrap(), rows(&[(0, &[0])]));
    assert_eq!(
        column_values(&head, "code", None).await,
        values(&[(1, Some("code(b1)")), (2, Some("c2"))])
    );
}

#[derive(Debug, Clone, Copy)]
enum DeletedRow {
    /// A computed row deleted at the snapshot.
    AssignedAtSnapshot,
    /// A stale row, deleted at the snapshot, of a fragment the publication
    /// restages.
    Copied,
    /// A computed row a concurrent delete removes before the commit.
    DeletedAfterStaging(DependencyConflictPolicy),
}

#[rstest]
#[case::assigned_at_snapshot(DeletedRow::AssignedAtSnapshot, true)]
#[case::copied_masked(DeletedRow::Copied, true)]
#[case::copied_unmasked(DeletedRow::Copied, false)]
#[case::deleted_after_staging_reject(DeletedRow::DeletedAfterStaging(Reject), true)]
#[case::deleted_after_staging_skip(DeletedRow::DeletedAfterStaging(Skip), true)]
#[tokio::test]
async fn test_stager_deleted_rows(#[case] deleted: DeletedRow, #[case] is_masked: bool) {
    let dataset = computed_outputs_with(2, 2, &[("summary", "body")], is_masked, false).await;
    let summary = flag_of(&dataset, "summary");
    match deleted {
        DeletedRow::AssignedAtSnapshot => {
            let mut read = dataset.clone();
            read.delete("id = 2").await.unwrap();
            let before = data_files(&read).await;
            let batch = addressed(
                &[addr(0, 0), addr(0, 1)],
                vec![("summary", text([Some("s1"), Some("s2")]))],
            );
            let error = stage_batches(&stager_of(&read, &["summary"]), vec![batch.into()])
                .await
                .unwrap_err();
            assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
            assert!(
                error.to_string().contains(&format!(
                    "computed row {:#x} (fragment 0, offset 1) is deleted at version {}; a \
                     deleted row cannot be published",
                    addr(0, 1),
                    read.version().version
                )),
                "{error}"
            );
            assert_eq!(data_files(&read).await, before);
        }
        DeletedRow::Copied => {
            // Id 2's published value goes stale in place before it is deleted.
            let published = publish_computed(&dataset, &["summary"], &[addr(0, 1)]).await;
            let mut read = merge_insert_body(&published, 2, "new").await;
            read.delete("id = 2").await.unwrap();
            let physical = |dataset: Dataset| async move {
                let fragment = dataset.get_fragment(0).unwrap();
                let schema = dataset.schema().project(&["summary"]).unwrap();
                let batches: Vec<RecordBatch> = fragment
                    .read_physical_slice(0..2, &schema, 2)
                    .await
                    .unwrap()
                    .buffered(1)
                    .try_collect()
                    .await
                    .unwrap();
                batches
                    .iter()
                    .flat_map(|batch| {
                        let column = batch["summary"].as_string::<i32>();
                        (0..column.len())
                            .map(|row| column.is_valid(row).then(|| column.value(row).to_string()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            let stored = (!is_masked).then(|| computed("summary", Some("b2")));
            assert_eq!(physical(read.clone()).await, vec![None, stored.clone()]);

            let head = publish_computed(&read, &["summary"], &[addr(0, 0)]).await;
            let summary_1 = Some(computed("summary", Some("b1")));
            assert_eq!(
                physical(head.clone()).await,
                vec![summary_1.clone(), stored]
            );
            assert_eq!(
                head.cell_flag_true_rows(summary).unwrap(),
                rows(&[(0, &[0])])
            );
        }
        DeletedRow::DeletedAfterStaging(policy) => {
            let assigned = [addr(0, 0), addr(0, 1), addr(1, 0)];
            let batch = computed_batch(&dataset, &assigned, &[("summary", &assigned)]).await;
            let staged = stage_batches(&stager_of(&dataset, &["summary"]), vec![batch])
                .await
                .unwrap()
                .unwrap();
            let mut deleted_at = dataset.clone();
            deleted_at.delete("id = 2").await.unwrap();
            let result = commit_staged(&dataset, staged, policy).await.unwrap();
            let valid = rows(&[(0, &[0]), (1, &[0])]);
            assert_eq!(
                result.report,
                PublicationReport {
                    read_version: dataset.version().version,
                    checked_version: deleted_at.version().version,
                    committed_version: Some(deleted_at.version().version + 1),
                    published: vec![published(summary, valid.clone())],
                    deferred_rows: vec![deferred(
                        summary,
                        rows(&[(0, &[1])]),
                        RowVacated,
                        deleted_at.version().version
                    )],
                    deferred_groups: vec![],
                }
            );
            assert_eq!(result.dataset.cell_flag_true_rows(summary).unwrap(), valid);
        }
    }
}

/// Summary and the translation computed from it, over three fragments: both
/// on fragment 0, summary alone on one row of fragment 1, where both were
/// published, and translation alone on fragment 2. Per-output masks over one
/// scan and batches carrying only the outputs they assign stage the same
/// publication.
#[tokio::test]
async fn test_stager_multiple_fragments_and_outputs() {
    let dataset = computed_outputs_with(3, 2, &CHAIN, true, false).await;
    let (summary, translation) = (
        flag_of(&dataset, "summary"),
        flag_of(&dataset, "translation"),
    );
    let outputs = ["summary", "translation"];
    let read = publish_computed(&dataset, &outputs, &[addr(1, 0), addr(1, 1)]).await;
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let fragment_2 = [addr(2, 0), addr(2, 1)];
    let summarized = [addr(0, 0), addr(0, 1), addr(1, 0)];
    let translated = [addr(0, 0), addr(0, 1), addr(2, 0), addr(2, 1)];

    let masked = computed_batch(
        &read,
        &live_addrs(&read).await,
        &[("summary", &summarized), ("translation", &translated)],
    )
    .await;
    let split = vec![
        computed_batch(
            &read,
            &fragment_0,
            &[("summary", &fragment_0), ("translation", &fragment_0)],
        )
        .await,
        computed_batch(&read, &[addr(1, 0)], &[("summary", &[addr(1, 0)])]).await,
        computed_batch(&read, &fragment_2, &[("translation", &fragment_2)]).await,
    ];
    let stager = stager_of(&read, &["translation", "summary"]);
    let staged = stage_batches(&stager, vec![masked]).await.unwrap().unwrap();
    let staged_split = stage_batches(&stager, split).await.unwrap().unwrap();

    let shape = |transaction: &Transaction| {
        let groups: Vec<(u64, Vec<i32>)> = groups_of(transaction)
            .iter()
            .map(|DataReplacementGroup(fragment_id, file)| (*fragment_id, file.fields.to_vec()))
            .collect();
        (groups, updates_of(transaction), transaction.read_version)
    };
    let schema = read.schema();
    let output_ids = vec![
        schema.field("summary").unwrap().id,
        schema.field("translation").unwrap().id,
    ];
    let expected_updates = vec![
        published(summary, rows(&[(0, &[0, 1]), (1, &[0])])),
        published(translation, rows(&[(0, &[0, 1]), (2, &[0, 1])])),
    ];
    assert_eq!(
        shape(&staged),
        (
            [0, 1, 2]
                .map(|fragment_id| (fragment_id, output_ids.clone()))
                .to_vec(),
            expected_updates.clone(),
            read.version().version
        )
    );
    assert_eq!(shape(&staged_split), shape(&staged));

    let head = commit_staged(&read, staged, Reject).await.unwrap().dataset;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0, 1]));
    // Translation is cleared where summary alone was recomputed.
    assert_eq!(
        head.cell_flag_true_rows(translation).unwrap(),
        full(&[0, 2]) | rows(&[(1, &[1])])
    );
    // Fragment 2's translations are computed from the summary it copies.
    let translation_of_null = computed("translation", None);
    let fragment_2_translations: Vec<(i32, Option<String>)> =
        published_values(&head, "translation", translation)
            .await
            .into_iter()
            .filter(|(id, _)| *id > 4)
            .collect();
    assert_eq!(
        fragment_2_translations,
        values(&[
            (5, Some(&translation_of_null)),
            (6, Some(&translation_of_null))
        ])
    );
    for column in outputs {
        assert_visible_values_follow_inputs(&head, column).await;
    }
}

/// `sibling_outputs` with `tags`, computed from keywords: a flag downstream
/// of keywords. Returns the dataset and the summary, keywords and tags flags.
async fn siblings_with_tags() -> (Dataset, u32, u32, u32) {
    let (mut dataset, summary, keywords, _) = sibling_outputs().await;
    let tags = ArrowSchema::new(vec![ArrowField::new("tags", DataType::Utf8, true)]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(tags)), None, None)
        .await
        .unwrap();
    let tags = dataset
        .register_cell_flag(
            "tags",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["keywords"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap()
        .flag_id;
    (dataset, summary, keywords, tags)
}

/// A publication of summary and keywords that assigns only summary writes
/// keywords as copies. Its empty keywords update tells the commit so:
/// keywords and tags, downstream of it, stay true, a concurrent tags
/// publisher is not deferred, and the update survives the transaction file.
#[rstest]
#[tokio::test]
async fn test_stager_copy_through_output_with_empty_assignment(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let (dataset, summary, keywords, tags) = siblings_with_tags().await;
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let siblings = ["summary", "keywords"];
    let with_keywords = stage_batches(
        &stager_of(&dataset, &siblings),
        vec![computed_batch(&dataset, &fragment_0, &[("keywords", &fragment_0)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        updates_of(&with_keywords),
        vec![
            published(summary, RowAddrTreeMap::new()),
            published(keywords, rows(&[(0, &[0, 1])]))
        ]
    );
    let with_keywords = commit_staged(&dataset, with_keywords, Reject)
        .await
        .unwrap()
        .dataset;
    let read = publish_computed(&with_keywords, &["tags"], &[addr(0, 0)]).await;
    let tags_publisher = stage_batches(
        &stager_of(&read, &["tags"]),
        vec![computed_batch(&read, &[addr(0, 1)], &[("tags", &[addr(0, 1)])]).await],
    )
    .await
    .unwrap()
    .unwrap();

    let staged = stage_batches(
        &stager_of(&read, &siblings),
        vec![computed_batch(&read, &fragment_0, &[("summary", &fragment_0)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    let updates = updates_of(&staged);
    assert_eq!(
        updates,
        vec![
            published(summary, rows(&[(0, &[0, 1])])),
            published(keywords, RowAddrTreeMap::new())
        ]
    );
    let result = commit_staged(&read, staged, policy).await.unwrap();
    assert_eq!(result.report.published, updates[..1].to_vec());
    assert!(result.report.deferred_rows.is_empty());
    assert!(result.report.deferred_groups.is_empty());
    let head = result.dataset;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0]));
    assert_eq!(head.cell_flag_true_rows(keywords).unwrap(), full(&[0]));
    assert_eq!(head.cell_flag_true_rows(tags).unwrap(), rows(&[(0, &[0])]));
    assert!(recorded_invalidations(&head).await.is_empty());
    let stored = head
        .read_transaction_from_storage(&head.manifest, &head.manifest_location)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updates_of(&stored), updates);

    let tags_result = commit_staged(&read, tags_publisher, Skip).await.unwrap();
    assert!(tags_result.report.deferred_rows.is_empty());
    assert_eq!(
        tags_result.report.published,
        vec![published(tags, rows(&[(0, &[1])]))]
    );
    assert_eq!(
        tags_result.dataset.cell_flag_true_rows(tags).unwrap(),
        full(&[0])
    );
}

/// The empty update names the flag, so dropping it concurrently fences the
/// publication like any other flag it publishes.
#[tokio::test]
async fn test_stager_empty_assignment_is_fenced_by_a_dropped_flag() {
    let (dataset, _, keywords, _) = siblings_with_tags().await;
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let staged = stage_batches(
        &stager_of(&dataset, &["summary", "keywords"]),
        vec![computed_batch(&dataset, &fragment_0, &[("summary", &fragment_0)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    let mut dropped = dataset.clone();
    dropped.drop_cell_flag("tags", "ready").await.unwrap();
    dropped.drop_cell_flag("keywords", "ready").await.unwrap();

    let error = commit_staged(&dataset, staged, Skip).await.unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains(&format!(
            "cell flag {keywords} is not registered since version {}",
            dropped.version().version
        )),
        "{error}"
    );
    assert_eq!(
        latest(&dropped).await.version().version,
        dropped.version().version
    );
}

/// The copy-through reads the outputs through Lance. An overlay committed on
/// summary before its flag was registered is copied as the value of its row
/// and no longer applies once the publication replaces the fragment. A stale
/// value under a false flag is copied as a masked read shows it: kept when
/// the flag does not mask, written as NULL when it does, so dropping the
/// flag reveals no stale value either way.
#[rstest]
#[tokio::test]
async fn test_stager_copy_through_reads_through_lance(#[values(false, true)] is_masked: bool) {
    let batch = record_batch!(
        ("id", Int32, [1, 2, 3, 4]),
        ("body", Utf8, ["b1", "b2", "b3", "b4"]),
        ("summary", Utf8, ["s1", "s2", "s3", "s4"])
    )
    .unwrap();
    let schema = batch.schema();
    let dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            max_rows_per_file: 2,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let summary_field = dataset.schema().field("summary").unwrap().id;
    let mut overlay = dataset
        .get_fragment(0)
        .unwrap()
        .write_overlay(&dataset.schema().project(&["summary"]).unwrap())
        .await
        .unwrap();
    overlay
        .write_batch(&addressed(
            &[addr(0, 0)],
            vec![("summary", text([Some("o")]))],
        ))
        .await
        .unwrap();
    let groups = vec![overlay.finish().await.unwrap().unwrap()];
    let mut dataset = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(
            TransactionBuilder::new(dataset.version().version, Operation::DataOverlay { groups })
                .build(),
        )
        .await
        .unwrap();
    // Registered after the overlay, which it would refuse on its output.
    dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(is_masked),
        )
        .await
        .unwrap();
    let summary = flag_of(&dataset, "summary");
    // Id 3's published summary goes stale in place.
    let published = publish_computed(&dataset, &["summary"], &[addr(1, 0)]).await;
    let read = merge_insert_body(&published, 3, "new").await;
    let overlays_summary = |dataset: &Dataset| {
        dataset
            .get_fragment(0)
            .unwrap()
            .metadata()
            .overlays
            .iter()
            .any(|overlay| overlay.data_file.fields.contains(&summary_field))
    };
    assert!(overlays_summary(&read));
    let stale = computed("summary", Some("b3"));
    let shown = |value: &str| (!is_masked).then(|| value.to_string());
    assert_eq!(
        column_values(&read, "summary", None).await,
        vec![
            (1, shown("o")),
            (2, shown("s2")),
            (3, shown(&stale)),
            (4, shown("s4"))
        ]
    );

    // Ids 2 and 4 are assigned; ids 1 and 3 are copied.
    let head = publish_computed(&read, &["summary"], &[addr(0, 1), addr(1, 1)]).await;
    assert!(!overlays_summary(&head));
    assert_eq!(
        head.cell_flag_true_rows(summary).unwrap(),
        rows(&[(0, &[1]), (1, &[1])])
    );
    let expected = vec![
        (1, shown("o")),
        (2, Some(computed("summary", Some("b2")))),
        (3, shown(&stale)),
        (4, Some(computed("summary", Some("b4")))),
    ];
    assert_eq!(column_values(&head, "summary", None).await, expected);
    let mut dropped = head;
    dropped.drop_cell_flag("summary", "ready").await.unwrap();
    assert_eq!(column_values(&dropped, "summary", None).await, expected);
}

#[derive(Debug, Clone, Copy)]
enum InvalidOutputs {
    NoOutputs,
    UnknownField,
    NestedField,
    ReservedColumn,
    DuplicateOutput,
    OrdinaryFlagOnly,
    BlobOutput,
    JsonOutput,
    LegacyDataset,
}

#[rstest]
#[case::no_outputs(InvalidOutputs::NoOutputs, "a publication stages at least one output")]
#[case::unknown_field(
    InvalidOutputs::UnknownField,
    "output 'abstract' is not a top-level field of version"
)]
#[case::nested_field(
    InvalidOutputs::NestedField,
    "output 'meta.text' is not a top-level field of version"
)]
#[case::reserved_column(
    InvalidOutputs::ReservedColumn,
    "output '_rowaddr' is a reserved column"
)]
#[case::duplicate_output(
    InvalidOutputs::DuplicateOutput,
    "output 'summary' is named more than once"
)]
#[case::ordinary_flag_only(
    InvalidOutputs::OrdinaryFlagOnly,
    "output 'title' has no dependent cell flag registered at version 4; register one with \
     clear_on_write sources before publishing it"
)]
#[case::blob_output(
    InvalidOutputs::BlobOutput,
    "output 'doc' is a blob column, which publication staging does not support"
)]
#[case::json_output(
    InvalidOutputs::JsonOutput,
    "output 'meta' is or holds a JSON field, which publication staging does not support"
)]
#[case::legacy_dataset(
    InvalidOutputs::LegacyDataset,
    "publication staging needs a V2 data storage format, but version 1 uses the legacy (v1) \
     format"
)]
#[tokio::test]
async fn test_stager_rejects_invalid_outputs(
    #[case] invalid: InvalidOutputs,
    #[case] expected: &str,
) {
    let mut dataset = articles(false).await;
    register_ready(&mut dataset).await;
    let (dataset, outputs): (Dataset, &[&str]) = match invalid {
        InvalidOutputs::NoOutputs => (dataset, &[]),
        InvalidOutputs::UnknownField => (dataset, &["abstract"]),
        InvalidOutputs::NestedField => {
            let meta = ArrowSchema::new(vec![ArrowField::new(
                "meta",
                DataType::Struct(vec![ArrowField::new("text", DataType::Utf8, true)].into()),
                true,
            )]);
            dataset
                .add_columns(NewColumnTransform::AllNulls(Arc::new(meta)), None, None)
                .await
                .unwrap();
            (dataset, &["meta.text"])
        }
        InvalidOutputs::ReservedColumn => (dataset, &["_rowaddr"]),
        InvalidOutputs::DuplicateOutput => (dataset, &["summary", "summary"]),
        InvalidOutputs::OrdinaryFlagOnly => {
            dataset
                .register_cell_flag("title", "reviewed", CellFlagOptions::default())
                .await
                .unwrap();
            (dataset, &["title"])
        }
        InvalidOutputs::JsonOutput => {
            let meta = ArrowField::new("meta", DataType::Utf8, true).with_metadata(
                [(
                    ARROW_EXT_NAME_KEY.to_string(),
                    ARROW_JSON_EXT_NAME.to_string(),
                )]
                .into(),
            );
            let schema = Arc::new(ArrowSchema::new(vec![
                ArrowField::new("body", DataType::Utf8, true),
                meta,
            ]));
            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![text([Some("b1")]), text([Some(r#"{"k": 1}"#)])],
            )
            .unwrap();
            let mut dataset = Dataset::write(
                RecordBatchIterator::new([Ok(batch)], schema),
                "memory://",
                None,
            )
            .await
            .unwrap();
            dataset
                .register_cell_flag(
                    "meta",
                    "ready",
                    CellFlagOptions::default().with_clear_on_write(["body"]),
                )
                .await
                .unwrap();
            (dataset, &["meta"])
        }
        InvalidOutputs::BlobOutput | InvalidOutputs::LegacyDataset => {
            let field = if matches!(invalid, InvalidOutputs::BlobOutput) {
                ArrowField::new("doc", DataType::LargeBinary, true)
                    .with_metadata([(BLOB_META_KEY.to_string(), "true".to_string())].into())
            } else {
                ArrowField::new("summary", DataType::LargeBinary, true)
            };
            let schema = Arc::new(ArrowSchema::new(vec![field]));
            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(LargeBinaryArray::from_iter_values([b"x"]))],
            )
            .unwrap();
            let storage = match invalid {
                InvalidOutputs::LegacyDataset => LanceFileVersion::Legacy,
                _ => LanceFileVersion::Stable,
            };
            let dataset = Dataset::write(
                RecordBatchIterator::new([Ok(batch)], schema),
                "memory://",
                Some(WriteParams {
                    data_storage_version: Some(storage),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
            let outputs: &[&str] = match invalid {
                InvalidOutputs::BlobOutput => &["doc"],
                _ => &["summary"],
            };
            (dataset, outputs)
        }
    };
    let error = PublicationStager::try_new(Arc::new(dataset), outputs).unwrap_err();
    let is_not_supported = matches!(
        invalid,
        InvalidOutputs::BlobOutput | InvalidOutputs::JsonOutput | InvalidOutputs::LegacyDataset
    );
    if is_not_supported {
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    } else {
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    }
    assert!(error.to_string().contains(expected), "{error}");
}

/// Digest is computed from translation, and translation from summary.
const DIGEST_CHAIN: [(&str, &str); 3] = [
    ("summary", "body"),
    ("translation", "summary"),
    ("digest", "translation"),
];

/// Publishing summary clears translation where it is not assigned, and the
/// commit carries that clear on to digest, so a stager of summary and digest
/// without translation would stage rows the commit refuses. It is refused up
/// front; with translation declared, or without digest, the rows stage and
/// commit.
#[rstest]
#[case::without_translation(&["summary", "digest"], false)]
#[case::whole_chain(&["summary", "translation", "digest"], true)]
#[case::without_digest(&["summary", "translation"], true)]
#[tokio::test]
async fn test_stager_refuses_an_undeclared_output_between_outputs(
    #[case] outputs: &[&str],
    #[case] is_accepted: bool,
) {
    let read = computed_outputs(2, &DIGEST_CHAIN).await;
    if !is_accepted {
        let before = data_files(&read).await;
        let error = PublicationStager::try_new(Arc::new(read.clone()), outputs).unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(
            error.to_string().contains(
                "output 'digest' depends on 'summary' through 'translation', which this stager \
                 does not stage; declare 'translation' too, or stage 'digest' in a separate \
                 publication"
            ),
            "{error}"
        );
        assert_eq!(data_files(&read).await, before);
        return;
    }
    let head = publish_computed(&read, outputs, &[addr(0, 0)]).await;
    for output in outputs {
        assert_eq!(
            head.cell_flag_true_rows(flag_of(&head, output)).unwrap(),
            rows(&[(0, &[0])]),
            "{output}"
        );
        assert_visible_values_follow_inputs(&head, output).await;
    }
}

/// With the whole chain declared, a row assigning summary and digest must
/// assign translation too: publishing summary clears translation where it is
/// not assigned, and the commit carries that clear on to digest, so the
/// commit would refuse the row. Leaving digest unassigned, or assigning
/// translation, stages and commits.
#[rstest]
#[case::skipping_translation(&[("summary", 0), ("digest", 0)], false)]
#[case::whole_chain(&[("summary", 0), ("translation", 0), ("digest", 0)], true)]
#[case::without_digest(&[("summary", 0), ("translation", 0)], true)]
#[case::summary_alone(&[("summary", 0)], true)]
#[case::on_different_rows(&[("summary", 0), ("digest", 1)], true)]
#[tokio::test]
async fn test_stager_refuses_a_row_skipping_an_output_between_outputs(
    #[case] assignments: &[(&str, u32)],
    #[case] is_accepted: bool,
) {
    let read = computed_outputs(2, &DIGEST_CHAIN).await;
    let outputs = ["summary", "translation", "digest"];
    let computed_rows = [addr(0, 0), addr(0, 1)];
    let assigned: Vec<(&str, Vec<u64>)> = outputs
        .iter()
        .map(|output| {
            let offsets = assignments
                .iter()
                .filter(|(assigned, _)| assigned == output)
                .map(|(_, offset)| addr(0, *offset))
                .collect();
            (*output, offsets)
        })
        .collect();
    let assigned: Vec<(&str, &[u64])> = assigned
        .iter()
        .filter(|(_, offsets)| !offsets.is_empty())
        .map(|(output, offsets)| (*output, offsets.as_slice()))
        .collect();
    let before = data_files(&read).await;
    let staged = stage_batches(
        &stager_of(&read, &outputs),
        vec![computed_batch(&read, &computed_rows, &assigned).await],
    )
    .await;
    if !is_accepted {
        let error = staged.unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(
            error.to_string().contains(
                "assigns 'summary' and 'digest' but not 'translation' between them: publishing \
                 'summary' clears 'translation' there, and with it 'digest'"
            ),
            "{error}"
        );
        assert_eq!(data_files(&read).await, before);
        return;
    }
    let head = commit_staged(&read, staged.unwrap().unwrap(), Reject)
        .await
        .unwrap()
        .dataset;
    for (output, offsets) in &assigned {
        let true_rows = head.cell_flag_true_rows(flag_of(&head, output)).unwrap();
        for offset in offsets.iter() {
            assert!(true_rows.contains(*offset), "{output} at {offset:#x}");
        }
    }
    for output in outputs {
        assert_visible_values_follow_inputs(&head, output).await;
    }
}

/// Summary is published on fragment 0, in a file of its own, and keywords is
/// then added as an all-NULL, metadata-only column with its own flag. One
/// file cannot replace a stored summary and a metadata-only keywords, which
/// the build would refuse, so a refresh of both is refused on fragment 0,
/// after fragment 1, which stores neither, was staged, and leaves nothing
/// behind. Publishing keywords on its own first, in a separate commit, lets
/// both refresh together: on fragment 0 from separate files (tombstoned) and
/// then from one (swapped in place), and on fragment 1 appended.
#[tokio::test]
async fn test_stager_rejects_mixed_output_layout() {
    let mut dataset = articles(false).await;
    let summary = register_ready(&mut dataset).await;
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let fragment_1 = [addr(1, 0), addr(1, 1)];
    let mut read = publish_computed(&dataset, &["summary"], &fragment_0).await;
    let keywords = ArrowSchema::new(vec![ArrowField::new("keywords", DataType::Utf8, true)]);
    read.add_columns(NewColumnTransform::AllNulls(Arc::new(keywords)), None, None)
        .await
        .unwrap();
    let keywords = read
        .register_cell_flag(
            "keywords",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap()
        .flag_id;
    let siblings = ["summary", "keywords"];
    let before = data_files(&read).await;
    let mut batches = Vec::new();
    for fragment in [&fragment_1, &fragment_0] {
        let assigned = [("summary", &fragment[..]), ("keywords", &fragment[..])];
        batches.push(computed_batch(&read, fragment, &assigned).await);
    }
    let error = stage_batches(&stager_of(&read, &siblings), batches)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    assert!(
        error.to_string().contains(
            "outputs [summary, keywords] cannot be replaced together on fragment 0: [summary] \
             are stored in its data files but [keywords] are not, so one commit cannot publish \
             them together there; publish [keywords] on its own first, then stage them together"
        ),
        "{error}"
    );
    assert_eq!(data_files(&read).await, before);
    assert_eq!(
        latest(&read).await.version().version,
        read.version().version
    );
    let by_hand = stage_rows(&read, 0, &siblings, |column, offset| {
        Some(format!("{column}-{offset}"))
    })
    .await;
    let error = publish(
        &read,
        vec![by_hand],
        both(summary, keywords, full(&[0])),
        Reject,
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Expected to modify the fragment but no changes were made"),
        "{error}"
    );

    let mut head = publish_computed(&read, &["keywords"], &fragment_0).await;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0]));
    assert_eq!(head.cell_flag_true_rows(keywords).unwrap(), full(&[0]));
    for _ in 0..2 {
        head = publish_computed(&head, &siblings, &live_addrs(&head).await).await;
    }
    for flag_id in [summary, keywords] {
        assert_eq!(head.cell_flag_true_rows(flag_id).unwrap(), full(&[0, 1]));
    }
    for (column, input) in [("summary", "t"), ("keywords", "b")] {
        let expected: Vec<(i32, Option<String>)> = (1..=4)
            .map(|id| (id, Some(computed(column, Some(&format!("{input}{id}"))))))
            .collect();
        assert_eq!(
            column_values(&head, column, None).await,
            expected,
            "{column}"
        );
    }
}

/// A computed batch, or stream item, that breaks the stager's contract.
#[derive(Debug, Clone, Copy)]
enum BadRows {
    MissingRowAddr,
    RowIdInsteadOfRowAddr,
    RowAddrNotUInt64,
    NullRowAddr,
    UndeclaredColumn,
    DuplicateColumn,
    LargeUtf8ForUtf8,
    BatchWithoutOutput,
    MaskLengthMismatch,
    MaskForAbsentColumn,
    MaskTwice,
    UnsortedOffsets,
    DuplicateAcrossBatches,
    FragmentRevisited,
    OffsetPastPhysicalRows,
    FragmentNotInSnapshot,
    /// Offset 1 of fragment 1 is deleted at the snapshot.
    DeletedRow,
    CallerStreamError,
}

impl BadRows {
    /// The stream item after fragment 0's rows and the first row of fragment
    /// 1, which has 3 physical rows.
    fn item(self) -> Result<ComputedBatch> {
        let summary = |len: usize| ("summary", text(vec![Some("x"); len]));
        let row = |fragment_id: u32, offset: u32| {
            addressed(&[addr(fragment_id, offset)], vec![summary(1)])
        };
        let with_addrs =
            |addrs: ArrayRef| RecordBatch::try_from_iter([(ROW_ADDR, addrs), summary(1)]).unwrap();
        let batch = match self {
            Self::MissingRowAddr => RecordBatch::try_from_iter([summary(1)]).unwrap(),
            Self::RowIdInsteadOfRowAddr => RecordBatch::try_from_iter([
                (
                    ROW_ID,
                    Arc::new(UInt64Array::from(vec![addr(1, 1)])) as ArrayRef,
                ),
                summary(1),
            ])
            .unwrap(),
            Self::RowAddrNotUInt64 => with_addrs(Arc::new(Int64Array::from(vec![1]))),
            Self::NullRowAddr => with_addrs(Arc::new(UInt64Array::from(vec![None]))),
            Self::UndeclaredColumn => addressed(
                &[addr(1, 1)],
                vec![summary(1), ("body", text([Some("b5")]))],
            ),
            Self::DuplicateColumn => addressed(&[addr(1, 1)], vec![summary(1), summary(1)]),
            Self::LargeUtf8ForUtf8 => addressed(
                &[addr(1, 1)],
                vec![(
                    "summary",
                    Arc::new(LargeStringArray::from(vec!["x"])) as ArrayRef,
                )],
            ),
            Self::BatchWithoutOutput => addressed(&[addr(1, 1)], vec![]),
            Self::MaskLengthMismatch => {
                return Ok(ComputedBatch::new(row(1, 1))
                    .with_assigned("summary", BooleanBuffer::from(vec![true, true])));
            }
            Self::MaskForAbsentColumn => {
                return Ok(ComputedBatch::new(row(1, 1))
                    .with_assigned("translation", BooleanBuffer::from(vec![true])));
            }
            Self::MaskTwice => {
                return Ok(ComputedBatch::new(row(1, 1))
                    .with_assigned("summary", BooleanBuffer::from(vec![true]))
                    .with_assigned("summary", BooleanBuffer::from(vec![true])));
            }
            Self::UnsortedOffsets => addressed(&[addr(1, 2), addr(1, 1)], vec![summary(2)]),
            Self::DuplicateAcrossBatches => row(1, 0),
            Self::FragmentRevisited => row(0, 1),
            Self::OffsetPastPhysicalRows => row(1, 3),
            Self::FragmentNotInSnapshot => row(7, 0),
            Self::DeletedRow => row(1, 1),
            Self::CallerStreamError => return Err(Error::io("the computation failed")),
        };
        Ok(batch.into())
    }

    fn expected(self) -> String {
        match self {
            Self::MissingRowAddr => "computed batch has no '_rowaddr' column".to_string(),
            Self::RowIdInsteadOfRowAddr => "computed batch has no '_rowaddr' column; row ids are \
                                            not physical addresses: scan with with_row_address()"
                .to_string(),
            Self::RowAddrNotUInt64 => {
                "computed batch has a '_rowaddr' of type Int64, expected UInt64".to_string()
            }
            Self::NullRowAddr => "computed batch has a null '_rowaddr' at batch row 0".to_string(),
            Self::UndeclaredColumn => "computed batch has column 'body', which is not an output \
                                       of this publication (outputs: [summary, translation])"
                .to_string(),
            Self::DuplicateColumn => "computed batch has column 'summary' twice".to_string(),
            Self::LargeUtf8ForUtf8 => {
                "computed values for output 'summary' do not match its field at version".to_string()
            }
            Self::BatchWithoutOutput => {
                "computed batch of 1 rows carries no output column".to_string()
            }
            Self::MaskLengthMismatch => {
                "assignment mask for 'summary' has 2 bits for a batch of 1 rows".to_string()
            }
            Self::MaskForAbsentColumn => {
                "assignment mask for 'translation' names no column of the batch".to_string()
            }
            Self::MaskTwice => "assignment mask for 'summary' is given twice".to_string(),
            Self::UnsortedOffsets => format!(
                "computed row {:#x} (fragment 1, offset 1) follows offset 2; offsets must \
                 strictly ascend within a fragment",
                addr(1, 1)
            ),
            Self::DuplicateAcrossBatches => format!(
                "computed row {:#x} (fragment 1, offset 0) follows offset 0",
                addr(1, 0)
            ),
            Self::FragmentRevisited => "computed rows for fragment 0 arrived after its run \
                                        ended; send each fragment's rows together, once"
                .to_string(),
            Self::OffsetPastPhysicalRows => format!(
                "computed row {:#x} names offset 3 of fragment 1, which has 3 physical rows at \
                 version",
                addr(1, 3)
            ),
            Self::FragmentNotInSnapshot => format!(
                "computed row {:#x} names fragment 7, which is not in version",
                addr(7, 0)
            ),
            Self::DeletedRow => format!(
                "computed row {:#x} (fragment 1, offset 1) is deleted at version",
                addr(1, 1)
            ),
            Self::CallerStreamError => "the computation failed".to_string(),
        }
    }
}

/// Each bad item arrives while fragment 1 stages, after fragment 0's file is
/// complete: the call fails with the item's error and leaves no staged file.
#[rstest]
#[case::missing_rowaddr(BadRows::MissingRowAddr)]
#[case::rowid_instead_of_rowaddr(BadRows::RowIdInsteadOfRowAddr)]
#[case::rowaddr_not_uint64(BadRows::RowAddrNotUInt64)]
#[case::null_rowaddr(BadRows::NullRowAddr)]
#[case::undeclared_column(BadRows::UndeclaredColumn)]
#[case::duplicate_column(BadRows::DuplicateColumn)]
#[case::large_utf8_for_utf8(BadRows::LargeUtf8ForUtf8)]
#[case::batch_without_output(BadRows::BatchWithoutOutput)]
#[case::mask_length_mismatch(BadRows::MaskLengthMismatch)]
#[case::mask_for_absent_column(BadRows::MaskForAbsentColumn)]
#[case::mask_twice(BadRows::MaskTwice)]
#[case::unsorted_offsets(BadRows::UnsortedOffsets)]
#[case::duplicate_across_batches(BadRows::DuplicateAcrossBatches)]
#[case::fragment_revisited(BadRows::FragmentRevisited)]
#[case::offset_past_physical_rows(BadRows::OffsetPastPhysicalRows)]
#[case::fragment_not_in_snapshot(BadRows::FragmentNotInSnapshot)]
#[case::deleted_row(BadRows::DeletedRow)]
#[case::caller_stream_error(BadRows::CallerStreamError)]
#[tokio::test]
async fn test_stager_validates_computed_rows(#[case] bad: BadRows) {
    let mut read = computed_outputs(3, &CHAIN).await;
    if matches!(bad, BadRows::DeletedRow) {
        read.delete("id = 5").await.unwrap();
    }
    let before = data_files(&read).await;
    let fragment_0 = [addr(0, 0), addr(0, 1), addr(0, 2)];
    let items = vec![
        Ok(computed_batch(&read, &fragment_0, &[("summary", &fragment_0)]).await),
        Ok(computed_batch(&read, &[addr(1, 0)], &[("summary", &[addr(1, 0)])]).await),
        bad.item(),
    ];
    let error = stager_of(&read, &["summary", "translation"])
        .stage(stream::iter(items))
        .await
        .unwrap_err();
    if matches!(bad, BadRows::CallerStreamError) {
        assert!(matches!(error, Error::IO { .. }), "{error}");
    } else {
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    }
    assert!(error.to_string().contains(&bad.expected()), "{error}");
    assert_eq!(data_files(&read).await, before);
    assert_eq!(
        latest(&read).await.version().version,
        read.version().version
    );
}

#[rstest]
#[case::empty_stream(vec![])]
#[case::zero_row_batches(vec![
    addressed(&[], vec![("summary", text(Vec::<Option<&str>>::new()))]).into(),
    addressed(&[], vec![]).into(),
])]
#[case::all_false_masks(vec![
    ComputedBatch::new(addressed(
        &[addr(0, 0), addr(1, 0)],
        vec![("summary", text([Some("x"), Some("y")]))],
    ))
    .with_assigned("summary", BooleanBuffer::from(vec![false, false])),
])]
#[tokio::test]
async fn test_stager_nothing_to_publish(#[case] batches: Vec<ComputedBatch>) {
    let read = computed_outputs(2, &[("summary", "body")]).await;
    let before = data_files(&read).await;
    let staged = stage_batches(&stager_of(&read, &["summary"]), batches)
        .await
        .unwrap();
    assert!(staged.is_none());
    assert_eq!(data_files(&read).await, before);
    assert_eq!(
        latest(&read).await.version().version,
        read.version().version
    );
}

/// Two stagers read one snapshot. The newer publishes id 2 first, so the
/// older, which copied id 2, cannot install fragment 0 over it.
#[rstest]
#[tokio::test]
async fn test_stager_cannot_overwrite_newer_result(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let dataset = computed_outputs(2, &[("summary", "body")]).await;
    let summary = flag_of(&dataset, "summary");
    let older_rows = [addr(0, 0), addr(1, 0), addr(1, 1)];
    let older = stage_batches(
        &stager_of(&dataset, &["summary"]),
        vec![computed_batch(&dataset, &older_rows, &[("summary", &older_rows)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    let newer = stage_batches(
        &stager_of(&dataset, &["summary"]),
        vec![addressed(&[addr(0, 1)], vec![("summary", text([Some("newer")]))]).into()],
    )
    .await
    .unwrap()
    .unwrap();
    let repaired = commit_staged(&dataset, newer, Reject)
        .await
        .unwrap()
        .dataset;

    let result = commit_staged(&dataset, older.clone(), policy).await;
    let head = match policy {
        Reject => {
            let error = result.unwrap_err();
            assert!(
                matches!(error, Error::RetryableCommitConflict { .. }),
                "{error}"
            );
            let expected = format!(
                "preempted by concurrent transaction DataReplacement at version {}",
                repaired.version().version
            );
            assert!(error.to_string().contains(&expected), "{error}");
            let head = latest(&repaired).await;
            assert_eq!(head.version().version, repaired.version().version);
            head
        }
        Skip => {
            let result = result.unwrap();
            let fragment_1 = rows(&[(1, &[0, 1])]);
            assert_eq!(
                result.report,
                PublicationReport {
                    read_version: dataset.version().version,
                    checked_version: repaired.version().version,
                    committed_version: Some(repaired.version().version + 1),
                    published: vec![published(summary, fragment_1)],
                    deferred_rows: vec![],
                    deferred_groups: vec![DeferredGroup {
                        fragment_id: 0,
                        data_file: groups_of(&older)[0].1.clone(),
                        reason: NewerResult,
                        conflicting_version: repaired.version().version,
                        valid_rows: vec![published(summary, rows(&[(0, &[0])]))],
                    }],
                }
            );
            result.dataset
        }
    };
    assert_eq!(
        published_values(&head, "summary", summary).await[0],
        (2, Some("newer".to_string()))
    );
}

/// The whole loop through the stager and the report. A refresh stages
/// summary and its translation on every row. Before it commits, id 3's body
/// changes, a newer result publishes id 2's summary, and, while the refresh
/// writes its first manifest, id 3 is deleted. The follow-up plan at the
/// committed version reuses id 1's staged summary, recomputes the
/// translations of fragment 0, and leaves out id 2's newer summary and the
/// deleted id 3. Staged and committed, it completes every live row.
#[tokio::test]
async fn test_stager_follow_up_restages_deferred_work() {
    let read = computed_outputs(2, &CHAIN).await;
    let (summary, translation) = (flag_of(&read, "summary"), flag_of(&read, "translation"));
    let outputs = ["summary", "translation"];
    let every_row = live_addrs(&read).await;
    let staged = stage_batches(
        &stager_of(&read, &outputs),
        vec![
            computed_batch(
                &read,
                &every_row,
                &[("summary", &every_row), ("translation", &every_row)],
            )
            .await,
        ],
    )
    .await
    .unwrap()
    .unwrap();
    let body_written = merge_insert_body(&read, 3, "new").await;
    let newer = publish_computed(&body_written, &["summary"], &[addr(0, 1)]).await;
    let delete = DeleteBuilder::new(Arc::new(newer.clone()), "id = 3")
        .execute_uncommitted()
        .await
        .unwrap()
        .transaction;
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: newer.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(newer.clone()), delete))),
    });
    let result = CommitBuilder::new(Arc::new(read.clone()))
        .with_commit_handler(handler.clone())
        .with_dependency_conflict_policy(Skip)
        .execute_with_report(staged)
        .await
        .unwrap();
    assert!(handler.competitor.lock().unwrap().is_none());
    let report = &result.report;
    let deleted_at = newer.version().version + 1;
    assert_eq!(report.committed_version, Some(deleted_at + 1));
    // Id 2 is deferred with its translation, whose input the newer result
    // changed, and id 3 for its body; id 3 is deleted after that.
    assert_eq!(
        report.deferred_rows_of(summary, InputChanged),
        rows(&[(0, &[1]), (1, &[0])])
    );
    assert_eq!(report.deferred_groups.len(), 1);
    assert_eq!(report.deferred_groups[0].reason, NewerResult);
    assert_eq!(report.reusable_rows(summary), rows(&[(0, &[0]), (1, &[1])]));

    let head = result.dataset.clone();
    let plan = stager_of(&head, &outputs).follow_up(report).await.unwrap();
    assert_eq!(
        plan.rows("summary"),
        Some(&FollowUpRows {
            reuse: rows(&[(0, &[0])]),
            recompute: RowAddrTreeMap::new(),
        })
    );
    assert_eq!(
        plan.rows("translation"),
        Some(&FollowUpRows {
            reuse: RowAddrTreeMap::new(),
            recompute: rows(&[(0, &[0, 1])]),
        })
    );
    assert!(!plan.is_empty());

    // The report only defers translation's rows, so a stager without it plans
    // summary alone.
    let summary_plan = stager_of(&head, &["summary"])
        .follow_up(report)
        .await
        .unwrap();
    assert_eq!(summary_plan.rows("summary"), plan.rows("summary"));
    assert_eq!(summary_plan.rows("translation"), None);

    // Stage the plan: id 1's summary is the reused value and its translation
    // follows it; id 2's translation follows the newer summary it copies.
    let planned = |output: &str| {
        let rows = plan.rows(output).unwrap();
        let mut addrs: Vec<u64> = (rows.reuse.clone() | &rows.recompute)
            .row_addrs()
            .unwrap()
            .map(u64::from)
            .collect();
        addrs.sort();
        addrs
    };
    let (summarized, translated) = (planned("summary"), planned("translation"));
    let follow_up = stage_batches(
        &stager_of(&head, &outputs),
        vec![
            computed_batch(
                &head,
                &translated,
                &[("summary", &summarized), ("translation", &translated)],
            )
            .await,
        ],
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        updates_of(&follow_up),
        vec![
            published(summary, rows(&[(0, &[0])])),
            published(translation, rows(&[(0, &[0, 1])])),
        ]
    );
    let completed = commit_staged(&head, follow_up, Reject)
        .await
        .unwrap()
        .dataset;
    for flag_id in [summary, translation] {
        assert_eq!(
            completed.cell_flag_true_rows(flag_id).unwrap(),
            with_full(rows(&[(1, &[1])]), 0)
        );
    }
    assert_eq!(
        published_values(&completed, "summary", summary).await[..2],
        values(&[
            (1, Some(&computed("summary", Some("b1")))),
            (2, Some(&computed("summary", Some("b2"))))
        ])
    );
    for column in outputs {
        assert_visible_values_follow_inputs(&completed, column).await;
    }

    // Nor can a stager at an earlier version, or at a later one, where the
    // rows the report certifies may have changed.
    for other in [read, newer, completed] {
        let error = stager_of(&other, &outputs)
            .follow_up(report)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(
            error.to_string().contains(&format!(
                "must read version {}, where it committed (or was last checked when nothing \
                 committed), but this stager reads version {}",
                head.version().version,
                other.version().version
            )),
            "{error}"
        );
    }
}

/// Translation, computed from summary, precedes it in the schema. An older
/// refresh assigns summary alone on fragment 0, where translation is true on
/// id 1, and a newer summary of id 2 defers its group whole. Reusing id 1's
/// summary would clear its translation, so the plan recomputes translation
/// there, which only the upstream rule, planned upstream first, can tell. A
/// stager without summary cannot plan its reusable rows.
#[tokio::test]
async fn test_stager_follow_up_recomputes_downstream_of_a_planned_upstream() {
    let dataset = computed_outputs(2, &[("translation", "summary"), ("summary", "body")]).await;
    let (summary, translation) = (
        flag_of(&dataset, "summary"),
        flag_of(&dataset, "translation"),
    );
    let outputs = ["summary", "translation"];
    let read = publish_computed(&dataset, &["translation"], &[addr(0, 0)]).await;
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let older = stage_batches(
        &stager_of(&read, &outputs),
        vec![computed_batch(&read, &fragment_0, &[("summary", &fragment_0)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    let newer = publish_computed(&read, &["summary"], &[addr(0, 1)]).await;
    let result = commit_staged(&read, older.clone(), Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(report.committed_version, None);
    assert!(report.deferred_rows.is_empty());
    assert_eq!(report.deferred_groups.len(), 1);
    assert_eq!(report.deferred_groups[0].reason, NewerResult);
    assert_eq!(report.reusable_rows(summary), rows(&[(0, &[0, 1])]));

    let plan = stager_of(&newer, &outputs).follow_up(report).await.unwrap();
    assert_eq!(
        plan.rows("summary"),
        Some(&FollowUpRows {
            reuse: rows(&[(0, &[0])]),
            recompute: RowAddrTreeMap::new(),
        })
    );
    assert_eq!(
        plan.rows("translation"),
        Some(&FollowUpRows {
            reuse: RowAddrTreeMap::new(),
            recompute: rows(&[(0, &[0])]),
        })
    );
    let error = stager_of(&newer, &["translation"])
        .follow_up(report)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    assert!(
        error.to_string().contains(&format!(
            "the report certifies rows of cell flag 'ready' (flag id {summary}) on 'summary'"
        )) && error
            .to_string()
            .contains("but this stager does not stage its output 'summary'"),
        "{error}"
    );

    let rows_of = |output: &str| {
        let rows = plan.rows(output).unwrap();
        let mut addrs: Vec<u64> = (rows.reuse.clone() | &rows.recompute)
            .row_addrs()
            .unwrap()
            .map(u64::from)
            .collect();
        addrs.sort();
        addrs
    };
    let (summarized, translated) = (rows_of("summary"), rows_of("translation"));
    let follow_up = stage_batches(
        &stager_of(&newer, &outputs),
        vec![
            computed_batch(
                &newer,
                &summarized,
                &[("summary", &summarized), ("translation", &translated)],
            )
            .await,
        ],
    )
    .await
    .unwrap()
    .unwrap();
    let completed = commit_staged(&newer, follow_up, Reject)
        .await
        .unwrap()
        .dataset;
    assert_eq!(completed.cell_flag_true_rows(summary).unwrap(), full(&[0]));
    assert_eq!(
        completed.cell_flag_true_rows(translation).unwrap(),
        rows(&[(0, &[0])])
    );
    for column in outputs {
        assert_visible_values_follow_inputs(&completed, column).await;
    }
}

/// A refresh recomputes summary alone on id 1 while a body write clears both
/// outputs there, so the file commits with summary's row deferred
/// InputChanged, and the report names no translation row. The plan still
/// recomputes translation on id 1, because the follow-up recomputes summary
/// there.
#[tokio::test]
async fn test_stager_follow_up_recomputes_downstream_of_a_recomputed_upstream() {
    let dataset = computed_outputs(2, &CHAIN).await;
    let outputs = ["summary", "translation"];
    let every_row = live_addrs(&dataset).await;
    let read = publish_computed(&dataset, &outputs, &every_row).await;
    let (summary, translation) = (flag_of(&read, "summary"), flag_of(&read, "translation"));
    let staged = stage_batches(
        &stager_of(&read, &outputs),
        vec![computed_batch(&read, &[addr(0, 0)], &[("summary", &[addr(0, 0)])]).await],
    )
    .await
    .unwrap()
    .unwrap();
    let written = merge_insert_body(&read, 1, "new").await;
    let result = commit_staged(&read, staged, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        report.committed_version,
        Some(result.dataset.version().version)
    );
    assert_eq!(
        report.deferred_rows,
        vec![deferred(
            summary,
            rows(&[(0, &[0])]),
            InputChanged,
            written.version().version
        )]
    );

    let head = result.dataset;
    let plan = stager_of(&head, &outputs).follow_up(report).await.unwrap();
    let recompute_only = |recompute| {
        Some(FollowUpRows {
            reuse: RowAddrTreeMap::new(),
            recompute,
        })
    };
    assert_eq!(
        plan.rows("summary"),
        recompute_only(rows(&[(0, &[0])])).as_ref()
    );
    assert_eq!(
        plan.rows("translation"),
        recompute_only(rows(&[(0, &[0])])).as_ref()
    );

    let follow_up = stage_batches(
        &stager_of(&head, &outputs),
        vec![
            computed_batch(
                &head,
                &[addr(0, 0)],
                &[("summary", &[addr(0, 0)]), ("translation", &[addr(0, 0)])],
            )
            .await,
        ],
    )
    .await
    .unwrap()
    .unwrap();
    let completed = commit_staged(&head, follow_up, Reject)
        .await
        .unwrap()
        .dataset;
    assert_eq!(
        completed.cell_flag_true_rows(summary).unwrap(),
        full(&[0, 1])
    );
    assert_eq!(
        completed.cell_flag_true_rows(translation).unwrap(),
        full(&[0, 1])
    );
    for column in outputs {
        assert_visible_values_follow_inputs(&completed, column).await;
    }
}

/// Summary and translation are added as all-NULL, metadata-only columns. A
/// refresh stages both on fragment 0 while a newer summary-only result of
/// id 2 stores summary alone there, which defers the refresh whole. No one
/// file can now replace both outputs on fragment 0, so the plan of both
/// cannot stage. A stager of summary alone plans its reusable row, since the
/// report only defers translation's rows, and a refresh of pending
/// translations completes the fragment.
#[tokio::test]
async fn test_stager_follow_up_plans_stored_outputs_over_a_mixed_layout() {
    let mut read = computed_outputs(2, &[]).await;
    let columns = ArrowSchema::new(vec![
        ArrowField::new("summary", DataType::Utf8, true),
        ArrowField::new("translation", DataType::Utf8, true),
    ]);
    read.add_columns(NewColumnTransform::AllNulls(Arc::new(columns)), None, None)
        .await
        .unwrap();
    for (output, input) in CHAIN {
        read.register_cell_flag(
            output,
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write([input])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    }
    let (summary, translation) = (flag_of(&read, "summary"), flag_of(&read, "translation"));
    let outputs = ["summary", "translation"];
    let fragment_0 = [addr(0, 0), addr(0, 1)];
    let older = stage_batches(
        &stager_of(&read, &outputs),
        vec![
            computed_batch(
                &read,
                &fragment_0,
                &[("summary", &fragment_0), ("translation", &fragment_0)],
            )
            .await,
        ],
    )
    .await
    .unwrap()
    .unwrap();
    let newer = publish_computed(&read, &["summary"], &[addr(0, 1)]).await;
    let result = commit_staged(&read, older, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(report.committed_version, None);
    assert_eq!(report.checked_version, newer.version().version);
    assert_eq!(report.reusable_rows(summary), rows(&[(0, &[0])]));
    assert!(report.reusable_rows(translation).is_empty());

    let error = stage_batches(
        &stager_of(&newer, &outputs),
        vec![computed_batch(&newer, &[addr(0, 0)], &[("summary", &[addr(0, 0)])]).await],
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outputs [summary, translation] cannot be replaced together on fragment 0"),
        "{error}"
    );
    let plan = stager_of(&newer, &["summary"])
        .follow_up(report)
        .await
        .unwrap();
    assert_eq!(
        plan.rows("summary"),
        Some(&FollowUpRows {
            reuse: rows(&[(0, &[0])]),
            recompute: RowAddrTreeMap::new(),
        })
    );
    assert_eq!(plan.rows("translation"), None);

    let reused = publish_computed(&newer, &["summary"], &[addr(0, 0)]).await;
    let mut pending = pending_addrs(&reused, "translation").await;
    pending.retain(|addr| RowAddress::from(*addr).fragment_id() == 0);
    assert_eq!(pending, fragment_0);
    let completed = publish_computed(&reused, &["translation"], &pending).await;
    for flag_id in [summary, translation] {
        assert_eq!(completed.cell_flag_true_rows(flag_id).unwrap(), full(&[0]));
    }
    for column in outputs {
        assert_visible_values_follow_inputs(&completed, column).await;
    }
}

/// A refresh's work: the live rows of `dataset` whose `output` flag is false,
/// in scan order.
async fn pending_addrs(dataset: &Dataset, output: &str) -> Vec<u64> {
    let true_rows = dataset
        .cell_flag_true_rows(flag_of(dataset, output))
        .unwrap();
    let mut pending = live_addrs(dataset).await;
    pending.retain(|addr| !true_rows.contains(*addr));
    pending
}

/// A refresh stages every pending row at one snapshot. Before it commits, a
/// row-moving update gives id 2 a new body, and so a new address, and an
/// in-place write changes id 4's body. Under `Skip` the publication reports
/// id 2 vacated and defers id 4, so the follow-up plan recomputes id 4 alone.
/// The moved row is pending at its new address, where a scan of pending rows
/// finds it. The two refreshes complete every live row, once, with values
/// computed from its current inputs.
#[rstest]
#[case::summary(&CHAIN[..1])]
#[case::chain(&CHAIN)]
#[tokio::test]
async fn test_stager_refresh_finds_rows_moved_during_publication(
    #[case] chain: &[(&str, &str)],
    #[values(false, true)] stable_row_ids: bool,
) {
    let read = computed_outputs_with(2, 3, chain, true, stable_row_ids).await;
    let outputs: Vec<&str> = chain.iter().map(|(output, _)| *output).collect();
    let stage_pending = |read: Dataset, pending: Vec<u64>| {
        let outputs = outputs.clone();
        async move {
            let assigned: Vec<(&str, &[u64])> = outputs
                .iter()
                .map(|output| (*output, pending.as_slice()))
                .collect();
            let batch = computed_batch(&read, &pending, &assigned).await;
            stage_batches(&stager_of(&read, &outputs), vec![batch])
                .await
                .unwrap()
                .unwrap()
        }
    };
    let staged = stage_pending(read.clone(), pending_addrs(&read, "summary").await).await;

    let moved_at = update_where(&read, "id = 2", "body", "b2-moved").await;
    let written_at = merge_insert_body(&moved_at, 4, "b4-edited").await;
    let result = commit_staged(&read, staged, Skip).await.unwrap();
    let report = &result.report;
    assert_eq!(
        report.committed_version,
        Some(written_at.version().version + 1)
    );
    assert!(report.deferred_groups.is_empty());
    assert_eq!(report.deferred_rows.len(), 2 * outputs.len());
    // The body write clears every output on id 4, downstream ones included.
    for output in &outputs {
        let flag_id = flag_of(&read, output);
        assert_eq!(
            report.published_rows(flag_id),
            rows(&[(0, &[0, 2]), (1, &[1, 2])]),
            "{output}"
        );
        assert_eq!(
            report.deferred_rows_of(flag_id, RowVacated),
            rows(&[(0, &[1])]),
            "{output}"
        );
        assert_eq!(
            report.deferred_rows_of(flag_id, InputChanged),
            rows(&[(1, &[0])]),
            "{output}"
        );
    }

    let head = result.dataset;
    let moved = values_by_addr(&head, "body")
        .await
        .into_iter()
        .find_map(|(addr, body)| (body.as_deref() == Some("b2-moved")).then_some(addr))
        .unwrap();
    assert_ne!(RowAddress::from(moved).fragment_id(), 0);
    // Neither id 2's vacated address nor its new one is planned.
    let plan = stager_of(&head, &outputs).follow_up(report).await.unwrap();
    for output in &outputs {
        assert_eq!(
            plan.rows(output),
            Some(&FollowUpRows {
                reuse: RowAddrTreeMap::new(),
                recompute: rows(&[(1, &[0])]),
            }),
            "{output}"
        );
    }
    let follow_up = stage_pending(head.clone(), vec![addr(1, 0)]).await;
    let followed = commit_staged(&head, follow_up, Reject)
        .await
        .unwrap()
        .dataset;

    for output in &outputs {
        assert_eq!(
            pending_addrs(&followed, output).await,
            vec![moved],
            "{output}"
        );
    }
    let refresh = stage_pending(followed.clone(), vec![moved]).await;
    let completed = commit_staged(&followed, refresh, Reject)
        .await
        .unwrap()
        .dataset;

    let bodies = column_values(&completed, "body", None).await;
    assert_eq!(
        bodies,
        (1..=6)
            .map(|id| {
                let body = match id {
                    2 => "b2-moved".to_string(),
                    4 => "b4-edited".to_string(),
                    _ => format!("b{id}"),
                };
                (id, Some(body))
            })
            .collect::<Vec<_>>()
    );
    for (index, output) in outputs.iter().enumerate() {
        assert!(
            pending_addrs(&completed, output).await.is_empty(),
            "{output}"
        );
        let expected: Vec<(i32, Option<String>)> = bodies
            .iter()
            .map(|(id, body)| {
                let value = chain[..=index]
                    .iter()
                    .fold(body.clone(), |input, (column, _)| {
                        Some(computed(column, input.as_deref()))
                    });
                (*id, value)
            })
            .collect();
        assert_eq!(
            column_values(&completed, output, None).await,
            expected,
            "{output}"
        );
    }
}

/// The publication keeps the version its values were computed at, so a
/// write committed after it is checked against the staged rows.
#[rstest]
#[tokio::test]
async fn test_stager_preserves_read_version(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let read = computed_outputs(2, &[("summary", "body")]).await;
    let summary = flag_of(&read, "summary");
    let every_row = live_addrs(&read).await;
    let staged = stage_batches(
        &stager_of(&read, &["summary"]),
        vec![computed_batch(&read, &every_row, &[("summary", &every_row)]).await],
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(staged.read_version, read.version().version);
    let written = merge_insert_body(&read, 1, "new").await;

    let result = commit_staged(&read, staged, policy).await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        return;
    }
    let result = result.unwrap();
    assert_eq!(
        result.report.deferred_rows,
        vec![deferred(
            summary,
            rows(&[(0, &[0])]),
            InputChanged,
            written.version().version
        )]
    );
    let committed = result.dataset.read_transaction().await.unwrap().unwrap();
    assert_eq!(committed.read_version, read.version().version);
}

/// With two-row copy windows and one-row batches, fragments are written one
/// after another as the stream arrives: fragment 0's file is complete once
/// the stream is asked for fragment 1's second row, and not before. Writing
/// within a fragment as its rows stream is pinned by
/// [`test_stager_writes_a_fragment_while_its_rows_stream`].
#[tokio::test]
async fn test_stager_streams_without_collecting() {
    let read = computed_outputs(4, &[("summary", "body")]).await;
    let summary = flag_of(&read, "summary");
    let every_row = live_addrs(&read).await;
    let mut batches = Vec::new();
    for row in &every_row {
        batches.push(computed_batch(&read, &[*row], &[("summary", &[*row])]).await);
    }
    let staged_files = Arc::new(Mutex::new(Vec::new()));
    let baseline = data_files(&read).await.len();
    let computed = stream::unfold(
        (batches.into_iter(), read.clone(), staged_files.clone()),
        |(mut batches, dataset, staged_files)| async move {
            let batch = batches.next()?;
            let files = data_files(&dataset).await.len() - baseline;
            staged_files.lock().unwrap().push(files);
            Some((Ok(batch), (batches, dataset, staged_files)))
        },
    );
    let staged = stager_of(&read, &["summary"])
        .with_copy_batch_size(2)
        .stage(computed)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(*staged_files.lock().unwrap(), vec![0, 0, 0, 0, 0, 1, 1, 1]);
    let head = commit_staged(&read, staged, Reject).await.unwrap().dataset;
    assert_eq!(head.cell_flag_true_rows(summary).unwrap(), full(&[0, 1]));
    assert_visible_values_follow_inputs(&head, "summary").await;
}

/// Within one fragment, `stage` writes the rows it has before the stream
/// yields the rest: the staged file has received bytes by the time the stream
/// is asked for the fragment's last batch. The file writer caches 8 MiB per
/// column before it encodes a page, so the fragment holds 12 MiB of digests,
/// each small enough (32 KiB) to be stored uncompressed.
#[tokio::test]
async fn test_stager_writes_a_fragment_while_its_rows_stream() {
    const ROWS: u32 = 384;
    const BATCH_ROWS: u32 = 16;
    const DIGEST_BYTES: usize = 32 * 1024;
    let test_uri = TempStrDir::default();
    let batch = RecordBatch::try_from_iter([
        (
            "body",
            Arc::new(StringArray::from_iter_values(
                (0..ROWS).map(|offset| format!("b{offset}")),
            )) as ArrayRef,
        ),
        (
            "digest",
            Arc::new(BinaryArray::new_null(ROWS as usize)) as ArrayRef,
        ),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        &test_uri,
        None,
    )
    .await
    .unwrap();
    let digest = dataset
        .register_cell_flag(
            "digest",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap()
        .flag_id;
    // The writer stages into a temporary file beside the data files.
    let data_dir = std::path::Path::new(test_uri.as_str()).join("data");
    let data_bytes = move || -> u64 {
        std::fs::read_dir(&data_dir)
            .unwrap()
            .map(|entry| entry.unwrap().metadata().unwrap().len())
            .sum()
    };
    let baseline = data_bytes();
    let digest_of = |offset: u32| vec![(offset % 251) as u8; DIGEST_BYTES];
    let staged_at_pull = Arc::new(Mutex::new(Vec::new()));
    let computed = stream::iter((0..ROWS).step_by(BATCH_ROWS as usize)).map({
        let staged_at_pull = staged_at_pull.clone();
        move |start| {
            staged_at_pull.lock().unwrap().push(data_bytes() - baseline);
            let offsets = start..start + BATCH_ROWS;
            let addrs: Vec<u64> = offsets.clone().map(|offset| addr(0, offset)).collect();
            let digests = BinaryArray::from_iter_values(offsets.map(digest_of));
            Ok::<_, Error>(addressed(
                &addrs,
                vec![("digest", Arc::new(digests) as ArrayRef)],
            ))
        }
    });

    let staged = stager_of(&dataset, &["digest"])
        .with_copy_batch_size(BATCH_ROWS)
        .stage(computed)
        .await
        .unwrap()
        .unwrap();
    let staged_at_pull = staged_at_pull.lock().unwrap().clone();
    assert_eq!(staged_at_pull.len(), (ROWS / BATCH_ROWS) as usize);
    assert_eq!(staged_at_pull[0], 0);
    assert!(
        *staged_at_pull.last().unwrap() > 0,
        "nothing was written before the last batch was pulled: {staged_at_pull:?}"
    );

    let head = commit_staged(&dataset, staged, Reject)
        .await
        .unwrap()
        .dataset;
    assert_eq!(head.cell_flag_true_rows(digest).unwrap(), full(&[0]));
    let digests = head
        .scan()
        .project(&["digest"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    let digests = digests["digest"].as_binary::<i32>();
    assert_eq!(digests.len(), ROWS as usize);
    for offset in 0..ROWS {
        assert_eq!(
            digests.value(offset as usize),
            digest_of(offset).as_slice(),
            "digest at offset {offset}"
        );
    }
}
