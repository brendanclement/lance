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
    Array, ArrayRef, Int32Array, RecordBatch, RecordBatchIterator, StringArray, record_batch,
};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use futures::stream;
use lance_core::datatypes::Schema as LanceSchema;
use lance_core::utils::address::RowAddress;
use lance_core::{Error, ROW_ADDR};
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
    CellFlagOptions, DeferralReason, DeferredGroup, DeferredRows, DependencyConflictPolicy,
    PublicationReport, PublicationResult,
};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{
    CellFlagChanges, CellFlagRegistration, CellFlagUpdate, DataReplacementGroup, Transaction,
    TransactionBuilder,
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
    let ids: Vec<i32> = (1..=2 * fragment_rows).collect();
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
                    .with_mask_when_false(true),
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
