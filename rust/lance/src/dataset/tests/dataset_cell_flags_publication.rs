// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Publishing dependent cell flags while inputs and outputs change
//! concurrently: the conflict rules, the `Skip` policy and the report.
//!
//! Every test stages a refresh at one version and commits the competing
//! transactions in a chosen order. Values are asserted where the flag is true,
//! so the assertions hold with or without masking; rows whose stale values are
//! stored under a false flag are asserted through the installed data file.

use std::sync::{Arc, Mutex};

use arrow_array::cast::AsArray;
use arrow_array::types::{Int32Type, UInt64Type};
use arrow_array::{Array, ArrayRef, RecordBatch, StringArray, record_batch};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use futures::stream;
use lance_core::datatypes::Schema as LanceSchema;
use lance_core::{Error, ROW_ADDR};
use lance_io::object_store::ObjectStore;
use lance_select::{RowAddrTreeMap, RowSetOps};
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
use crate::dataset::write::CommitBuilder;
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::{DeleteBuilder, MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder};
use crate::{Dataset, Result};

use DeferralReason::{FragmentRemoved, InputChanged, NewerResult, RowVacated};
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
    let source = record_batch!(("id", Int32, [id]), ("body", Utf8, [body])).unwrap();
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
    assert_eq!(
        column_values(&committed, "body", None).await[1],
        (2, Some(write.body_of_id_2().to_string()))
    );
}

#[rstest]
#[tokio::test]
async fn test_refresh_skips_rows_invalidated_while_flag_false(
    #[values(BodyWrite::Replace, BodyWrite::MergeInsert)] write: BodyWrite,
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let written = write.apply(&dataset).await;
    assert!(written.cell_flag_true_rows(flag_id).unwrap().is_empty());
    assert_eq!(
        recorded_invalidations(&written).await,
        cleared(flag_id, write.clears()),
        "recorded although the flag was never true"
    );

    let result = publish(&read, refresh, set_true(flag_id, full(&[0, 1])), policy).await;
    if policy == Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        assert!(
            error.to_string().contains(&format!(
                "cannot be published on {} row(s) of fragment 0",
                write.stale().len().unwrap()
            )),
            "{error}"
        );
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
            InputChanged,
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
    assert!(
        !published_values(&result.dataset, "summary", flag_id)
            .await
            .iter()
            .any(|(id, _)| *id == 2),
        "id 2 is not published"
    );
}

#[rstest]
#[tokio::test]
async fn test_input_restored_before_refresh_publishes(
    #[values(Reject, Skip)] policy: DependencyConflictPolicy,
) {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let refresh = stage_all(&read, "summary", "s").await;
    let changed = merge_insert_body(&dataset, 2, "changed").await;
    let restored = merge_insert_body(&changed, 2, "b2").await;
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
            rows(&[(0, &[1])]),
            InputChanged,
            changed.version().version
        )]
    );
    assert_eq!(result.report.checked_version, restored.version().version);
    let valid = with_full(rows(&[(0, &[0])]), 1);
    assert_eq!(result.dataset.cell_flag_true_rows(flag_id).unwrap(), valid);
    assert_eq!(
        published_values(&result.dataset, "summary", flag_id).await,
        values(&[(1, Some("s-0-0")), (3, Some("s-1-0")), (4, Some("s-1-1"))])
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
        published_values(&result.dataset, "summary", flag_id).await,
        values(&[(3, Some("s-1-0")), (4, Some("s-1-1"))])
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

#[tokio::test]
async fn test_commit_retry_records_every_invalidation() {
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
    let registration = TransactionBuilder::new(dataset.version().version, update_config())
        .cell_flag_changes(CellFlagChanges {
            registrations: vec![CellFlagRegistration {
                field_id: schema.field("translation").unwrap().id,
                name: "ready".to_string(),
                clear_on_write: vec![schema.field("body").unwrap().id],
                mask_when_false: true,
            }],
            ..Default::default()
        })
        .build();
    let handler = Arc::new(CommitsCompetitorFirst {
        inner: dataset.commit_handler.clone(),
        competitor: Mutex::new(Some((Arc::new(dataset.clone()), registration))),
    });
    let body = stage_rows(&dataset, 0, &["body"], |_, offset| {
        Some(format!("new-{offset}"))
    })
    .await;

    let written = CommitBuilder::new(Arc::new(dataset.clone()))
        .with_commit_handler(handler.clone())
        .execute(replacement_txn(
            dataset.version().version,
            vec![body],
            vec![],
        ))
        .await
        .unwrap();
    assert!(handler.competitor.lock().unwrap().is_none());
    assert_eq!(
        written.version().version,
        dataset.version().version + 2,
        "the competitor took the first attempt's version"
    );
    let translated = written.cell_flag("translation", "ready").unwrap().flag_id;
    let mut expected = cleared(ready, full(&[0]));
    expected.extend(cleared(translated, full(&[0])));
    assert_eq!(recorded_invalidations(&written).await, expected);
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
    assert_eq!(
        published_values(&result.dataset, "summary", flag_id).await,
        values(&[(2, Some("newer-0-1")), (4, Some("s-1-1"))])
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
    assert_eq!(
        published_values(&result.dataset, "summary", flag_id).await,
        values(&[(3, Some("s-1-0")), (4, Some("s-1-1"))])
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
    // The stale keywords are installed under a false flag.
    assert_eq!(file_of(&result.dataset, 0, "keywords"), keywords_path);
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

#[tokio::test]
async fn test_rejected_publication_exposes_nothing() {
    let (dataset, summary, keywords, staged) = sibling_outputs().await;
    let written = merge_insert_body(&dataset, 3, "new").await;

    let error = publish(
        &dataset,
        staged,
        both(summary, keywords, full(&[0, 1])),
        Reject,
    )
    .await
    .unwrap_err();
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
    let head = latest(&written).await;
    assert_eq!(head.version().version, written.version().version);
    for (flag_id, column) in [(summary, "summary"), (keywords, "keywords")] {
        assert!(head.cell_flag_true_rows(flag_id).unwrap().is_empty());
        assert!(
            column_values(&head, column, None)
                .await
                .iter()
                .all(|(_, value)| value.is_none()),
            "{column} was not published anywhere"
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
            published_values(&result.dataset, column, flag_id).await,
            values(&[
                (1, Some(&format!("{prefix}{column}-0-0"))),
                (2, Some(&format!("{prefix}{column}-0-1"))),
                (4, Some(&format!("{prefix}{column}-1-1"))),
            ])
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
        published_values(&result.dataset, "summary", summary).await,
        values(&[
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
