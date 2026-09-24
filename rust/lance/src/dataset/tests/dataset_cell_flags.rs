// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Dependent cell flags end to end: registration, publication through a staged
//! `DataReplacement`, and the clears that writes to the watched fields record.

use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, RecordBatchIterator, StringArray, record_batch};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use futures::stream;
use lance_core::Error;
use lance_core::datatypes::Schema as LanceSchema;
use lance_select::{RowAddrTreeMap, RowSetOps};
use lance_table::feature_flags::FLAG_UNSTABLE_CELL_FLAGS;
use rstest::rstest;

use crate::dataset::cell_flag::CellFlagOptions;
use crate::dataset::optimize::{CompactionOptions, compact_files};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{
    CellFlagChanges, CellFlagUpdate, DataReplacementGroup, Operation, TransactionBuilder,
};
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::write::{CommitBuilder, WriteParams};
use crate::dataset::{MergeInsertBuilder, MergeInsertWriteMode};
use crate::{Dataset, Result};

/// Two fragments of two articles each, with a `summary` column declared but
/// never written, the way a computed column starts.
async fn articles(stable_row_ids: bool) -> Dataset {
    let batch = record_batch!(
        ("id", Int32, [1, 2, 3, 4]),
        ("title", Utf8, ["t1", "t2", "t3", "t4"]),
        ("body", Utf8, ["b1", "b2", "b3", "b4"])
    )
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            max_rows_per_file: 2,
            enable_stable_row_ids: stable_row_ids,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let summary = ArrowSchema::new(vec![ArrowField::new("summary", DataType::Utf8, true)]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(summary)), None, None)
        .await
        .unwrap();
    assert_eq!(dataset.get_fragments().len(), 2);
    dataset
}

async fn register_ready(dataset: &mut Dataset) -> u32 {
    let ready = dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["title", "body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    ready.flag_id
}

/// Stage a full-fragment file for `column` holding `{prefix}-{fragment}-{offset}`.
async fn stage(
    dataset: &Dataset,
    fragment_id: u64,
    column: &str,
    prefix: &str,
) -> DataReplacementGroup {
    let fragment = dataset.get_fragment(fragment_id as usize).unwrap();
    let physical_rows = fragment.physical_rows().await.unwrap();
    let values: Vec<String> = (0..physical_rows)
        .map(|offset| format!("{prefix}-{fragment_id}-{offset}"))
        .collect();
    let batch = RecordBatch::try_new(
        Arc::new(ArrowSchema::new(vec![ArrowField::new(
            column,
            DataType::Utf8,
            true,
        )])),
        vec![Arc::new(StringArray::from(values))],
    )
    .unwrap();
    let schema = LanceSchema {
        fields: vec![dataset.schema().field(column).unwrap().clone()],
        metadata: Default::default(),
    };
    fragment
        .write_columns(stream::iter([Ok(batch)]), &schema)
        .await
        .unwrap()
}

async fn stage_all(dataset: &Dataset, column: &str, prefix: &str) -> Vec<DataReplacementGroup> {
    let mut groups = Vec::new();
    for fragment in dataset.get_fragments() {
        groups.push(stage(dataset, fragment.id() as u64, column, prefix).await);
    }
    groups
}

fn full(fragment_ids: &[u32]) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for fragment_id in fragment_ids {
        rows.insert_fragment(*fragment_id);
    }
    rows
}

fn set_true(flag_id: u32, rows: RowAddrTreeMap) -> Vec<CellFlagUpdate> {
    vec![CellFlagUpdate {
        flag_id,
        value: true,
        rows,
    }]
}

/// Commit `replacements` staged against `dataset`'s version.
async fn commit_replacement(
    dataset: &Dataset,
    replacements: Vec<DataReplacementGroup>,
    updates: Vec<CellFlagUpdate>,
) -> Result<Dataset> {
    let transaction = TransactionBuilder::new(
        dataset.manifest.version,
        Operation::DataReplacement { replacements },
    )
    .cell_flag_changes(CellFlagChanges {
        updates,
        ..Default::default()
    })
    .build();
    CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(transaction)
        .await
}

/// The derived invalidations the transaction file of `dataset`'s version holds.
async fn recorded_invalidations(dataset: &Dataset) -> Vec<CellFlagUpdate> {
    let transaction = dataset
        .read_transaction_from_storage(&dataset.manifest, &dataset.manifest_location)
        .await
        .unwrap()
        .unwrap();
    let cached = dataset.read_transaction().await.unwrap().unwrap();
    assert_eq!(cached.cell_flag_changes, transaction.cell_flag_changes);
    transaction
        .cell_flag_changes
        .map(|changes| changes.derived_invalidations.clone())
        .unwrap_or_default()
}

fn cleared(flag_id: u32, rows: RowAddrTreeMap) -> Vec<CellFlagUpdate> {
    vec![CellFlagUpdate {
        flag_id,
        value: false,
        rows,
    }]
}

#[tokio::test]
async fn test_dependent_flag_publication_and_invalidation() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let ready = dataset.cell_flag("summary", "ready").unwrap();
    let schema = dataset.schema();
    assert_eq!(
        ready.clear_on_write,
        vec![
            schema.field("title").unwrap().id,
            schema.field("body").unwrap().id
        ]
    );
    assert!(ready.mask_when_false);
    assert_eq!(dataset.cell_flags().len(), 1);
    assert!(dataset.cell_flag_true_rows(flag_id).unwrap().is_empty());
    assert_ne!(
        dataset.manifest.reader_feature_flags & FLAG_UNSTABLE_CELL_FLAGS,
        0
    );
    assert_ne!(
        dataset.manifest.writer_feature_flags & FLAG_UNSTABLE_CELL_FLAGS,
        0
    );

    let groups = stage_all(&dataset, "summary", "s").await;
    let dataset = commit_replacement(&dataset, groups, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap();
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[0, 1]));
    assert!(
        recorded_invalidations(&dataset).await.is_empty(),
        "a publication does not invalidate itself"
    );
    let summaries = dataset
        .scan()
        .project(&["summary"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    let expected: ArrayRef = Arc::new(StringArray::from(vec!["s-0-0", "s-0-1", "s-1-0", "s-1-1"]));
    assert_eq!(&summaries["summary"], &expected);

    // A source write clears the fragment it rewrote and records the clear.
    let body = stage(&dataset, 0, "body", "b").await;
    let dataset = commit_replacement(&dataset, vec![body], vec![])
        .await
        .unwrap();
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[1]));
    assert_eq!(
        recorded_invalidations(&dataset).await,
        cleared(flag_id, full(&[0]))
    );

    // Recorded again while the flag is already false there.
    let body = stage(&dataset, 0, "body", "b2").await;
    let dataset = commit_replacement(&dataset, vec![body], vec![])
        .await
        .unwrap();
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[1]));
    assert_eq!(
        recorded_invalidations(&dataset).await,
        cleared(flag_id, full(&[0]))
    );

    // A write to a field the flag does not watch keeps it and records nothing.
    let id_file = {
        let fragment = dataset.get_fragment(1).unwrap();
        let batch = record_batch!(("id", Int32, [3, 4])).unwrap();
        let schema = LanceSchema {
            fields: vec![dataset.schema().field("id").unwrap().clone()],
            metadata: Default::default(),
        };
        fragment
            .write_columns(stream::iter([Ok(batch)]), &schema)
            .await
            .unwrap()
    };
    let dataset = commit_replacement(&dataset, vec![id_file], vec![])
        .await
        .unwrap();
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[1]));
    let transaction = dataset.read_transaction().await.unwrap().unwrap();
    assert!(transaction.cell_flag_changes.is_none());

    // Compaction cannot remap flag state yet, so it is refused.
    let mut compacted = dataset.clone();
    let error = compact_files(&mut compacted, CompactionOptions::default(), None)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains("compaction would move rows"),
        "{error}"
    );
    assert_eq!(compacted.version().version, dataset.version().version);
}

#[tokio::test]
async fn test_dropped_flag_fences_staged_publication() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let staged = stage_all(&dataset, "summary", "stale").await;

    let mut dropper = dataset.clone();
    dropper.drop_cell_flag("summary", "ready").await.unwrap();
    assert!(dropper.cell_flag("summary", "ready").is_none());
    let dropped_at = dropper.version().version;

    let error = commit_replacement(&dataset, staged, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains(&format!("cell flag {flag_id} is not registered")),
        "{error}"
    );
    dropper.checkout_latest().await.unwrap();
    assert_eq!(
        dropper.version().version,
        dropped_at,
        "nothing was committed"
    );
    let summaries = dropper
        .scan()
        .project(&["summary"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    assert_eq!(summaries["summary"].null_count(), 4);

    // Replacing fences in the same way: the new registration takes a new id.
    let mut dataset = articles(false).await;
    let old_id = register_ready(&mut dataset).await;
    let staged = stage_all(&dataset, "summary", "stale").await;
    let mut replacer = dataset.clone();
    let replaced = replacer
        .replace_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap();
    assert_ne!(replaced.flag_id, old_id);
    let error = commit_replacement(&dataset, staged, set_true(old_id, full(&[0, 1])))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
}

#[tokio::test]
async fn test_restore_keeps_flag_ids_monotonic() {
    let mut dataset = articles(false).await;
    let first = register_ready(&mut dataset).await;
    let registered_at = dataset.version().version;
    dataset.drop_cell_flag("summary", "ready").await.unwrap();
    let second = register_ready(&mut dataset).await;
    assert!(second > first);

    let mut restored = dataset.checkout_version(registered_at).await.unwrap();
    restored.restore().await.unwrap();
    // The restored registry is the old one, except that its ids stay unused.
    assert_eq!(
        restored.cell_flag("summary", "ready").unwrap().flag_id,
        first
    );
    let reviewed = restored
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap();
    assert!(
        reviewed.flag_id > second,
        "{} reused an id",
        reviewed.flag_id
    );

    // Restoring past the first registration still keeps the allocator.
    let mut before_flags = restored.checkout_version(registered_at - 1).await.unwrap();
    before_flags.restore().await.unwrap();
    assert!(before_flags.cell_flags().is_empty());
    let again = register_ready(&mut before_flags).await;
    assert!(again > reviewed.flag_id, "{again} reused an id");
}

#[rstest]
#[tokio::test]
async fn test_partial_merge_insert_clears_written_rows(
    #[values(false, true)] stable_row_ids: bool,
) {
    let mut dataset = articles(stable_row_ids).await;
    let flag_id = register_ready(&mut dataset).await;
    let groups = stage_all(&dataset, "summary", "s").await;
    let dataset = commit_replacement(&dataset, groups, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap();

    // Rewrites `body` in place for id 2, the second row of fragment 0.
    let source = record_batch!(("id", Int32, [2]), ("body", Utf8, ["new body"])).unwrap();
    let (dataset, _) = MergeInsertBuilder::try_new(Arc::new(dataset), vec!["id".to_string()])
        .unwrap()
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::DoNothing)
        .write_mode(MergeInsertWriteMode::RewriteColumns)
        .try_build()
        .unwrap()
        .execute_batches(vec![source])
        .await
        .unwrap();

    // Only stable row ids record which rows matched; otherwise the whole
    // rewritten fragment is cleared.
    let mut expected = full(&[1]);
    let written = if stable_row_ids {
        expected.insert(0);
        let mut written = RowAddrTreeMap::new();
        written.insert(1);
        written
    } else {
        full(&[0])
    };
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), expected);
    assert_eq!(
        recorded_invalidations(&dataset).await,
        cleared(flag_id, written)
    );
}
