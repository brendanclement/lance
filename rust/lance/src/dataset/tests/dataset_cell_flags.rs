// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Dependent cell flags end to end: registration, publication through a staged
//! `DataReplacement`, and the clears that writes to the watched fields record.

use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, RecordBatchIterator, StringArray, record_batch};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use futures::stream;
use lance_core::Error;
use lance_core::datatypes::Schema as LanceSchema;
use lance_core::utils::tempfile::TempStdDir;
use lance_select::{RowAddrTreeMap, RowSetOps};
use lance_table::feature_flags::FLAG_UNSTABLE_CELL_FLAGS;
use rstest::rstest;

use crate::dataset::cell_flag::CellFlagOptions;
use crate::dataset::optimize::{CompactionOptions, compact_files};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{
    CellFlagChanges, CellFlagRegistration, CellFlagUpdate, DataReplacementGroup, Operation,
    Transaction, TransactionBuilder, translate_config_updates,
};
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::write::{CommitBuilder, InsertBuilder, WriteMode, WriteParams};
use crate::dataset::{ColumnAlteration, MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder};
use crate::{Dataset, Result};

/// Two fragments of two articles each, with a `summary` column declared but
/// never written, the way a computed column starts.
async fn articles(stable_row_ids: bool) -> Dataset {
    articles_at("memory://", stable_row_ids).await
}

async fn articles_at(uri: &str, stable_row_ids: bool) -> Dataset {
    let batch = record_batch!(
        ("id", Int32, [1, 2, 3, 4]),
        ("title", Utf8, ["t1", "t2", "t3", "t4"]),
        ("body", Utf8, ["b1", "b2", "b3", "b4"])
    )
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        uri,
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
pub(super) async fn stage(
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

pub(super) async fn stage_all(
    dataset: &Dataset,
    column: &str,
    prefix: &str,
) -> Vec<DataReplacementGroup> {
    let mut groups = Vec::new();
    for fragment in dataset.get_fragments() {
        groups.push(stage(dataset, fragment.id() as u64, column, prefix).await);
    }
    groups
}

pub(super) fn full(fragment_ids: &[u32]) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for fragment_id in fragment_ids {
        rows.insert_fragment(*fragment_id);
    }
    rows
}

pub(super) fn set_true(flag_id: u32, rows: RowAddrTreeMap) -> Vec<CellFlagUpdate> {
    vec![CellFlagUpdate {
        flag_id,
        value: true,
        rows,
    }]
}

fn replacement_txn(
    read_version: u64,
    replacements: Vec<DataReplacementGroup>,
    updates: Vec<CellFlagUpdate>,
) -> Transaction {
    TransactionBuilder::new(read_version, Operation::DataReplacement { replacements })
        .cell_flag_changes(CellFlagChanges {
            updates,
            ..Default::default()
        })
        .build()
}

/// Commit `replacements` staged against `dataset`'s version.
pub(super) async fn commit_replacement(
    dataset: &Dataset,
    replacements: Vec<DataReplacementGroup>,
    updates: Vec<CellFlagUpdate>,
) -> Result<Dataset> {
    CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(replacement_txn(
            dataset.manifest.version,
            replacements,
            updates,
        ))
        .await
}

pub(super) fn update_config() -> Operation {
    Operation::UpdateConfig {
        config_updates: None,
        table_metadata_updates: None,
        schema_metadata_updates: None,
        field_metadata_updates: HashMap::new(),
    }
}

/// The derived invalidations the transaction file of `dataset`'s version holds.
async fn recorded_invalidations(dataset: &Dataset) -> Vec<CellFlagUpdate> {
    let transaction = dataset
        .read_transaction_from_storage(&dataset.manifest, &dataset.manifest_location)
        .await
        .unwrap()
        .unwrap();
    // The commit outcome check compares the two, so they must not diverge.
    let cached = dataset.read_transaction().await.unwrap().unwrap();
    assert_eq!(cached, transaction);
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

    // Only the matched row is cleared, with or without stable row ids.
    let mut expected = full(&[1]);
    expected.insert(0);
    let mut written = RowAddrTreeMap::new();
    written.insert(1);
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), expected);
    assert_eq!(
        recorded_invalidations(&dataset).await,
        cleared(flag_id, written)
    );
}

#[tokio::test]
async fn test_clears_propagate_down_dependency_chains() {
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
    let groups = stage_all(&dataset, "summary", "s").await;
    let dataset = commit_replacement(&dataset, groups, set_true(ready, full(&[0, 1])))
        .await
        .unwrap();
    let groups = stage_all(&dataset, "translation", "t").await;
    let dataset = commit_replacement(&dataset, groups, set_true(translated, full(&[0, 1])))
        .await
        .unwrap();

    // Writing body masks summary on fragment 0, so the translation computed
    // from it is stale there too.
    let body = stage(&dataset, 0, "body", "b").await;
    let dataset = commit_replacement(&dataset, vec![body], vec![])
        .await
        .unwrap();
    for flag_id in [ready, translated] {
        assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[1]));
    }
    let mut expected = cleared(ready, full(&[0]));
    expected.extend(cleared(translated, full(&[0])));
    expected.sort_by_key(|update| update.flag_id);
    assert_eq!(recorded_invalidations(&dataset).await, expected);
}

#[tokio::test]
async fn test_write_staged_before_registration_records_clear() {
    let mut dataset = articles(false).await;
    let read_version = dataset.manifest.version;
    let staged_body = stage(&dataset, 0, "body", "b").await;

    let flag_id = register_ready(&mut dataset).await;
    let groups = stage_all(&dataset, "summary", "s").await;
    let dataset = commit_replacement(&dataset, groups, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap();

    // Staged before the flag existed, so only the head's registry knows it
    // must clear.
    let dataset = CommitBuilder::new(Arc::new(dataset))
        .execute(replacement_txn(read_version, vec![staged_body], vec![]))
        .await
        .unwrap();
    assert_eq!(dataset.cell_flag_true_rows(flag_id).unwrap(), full(&[1]));
    assert_eq!(
        recorded_invalidations(&dataset).await,
        cleared(flag_id, full(&[0]))
    );
}

#[tokio::test]
async fn test_publication_needs_flag_registered_at_read_version() {
    let mut dataset = articles(false).await;
    let read = dataset.clone();
    let staged = stage_all(&read, "summary", "stale").await;
    let flag_id = register_ready(&mut dataset).await;
    let registered_at = dataset.version().version;

    let error = commit_replacement(&read, staged, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains(&format!(
            "cell flag {flag_id} is set true, but it was not registered at version {}",
            read.version().version
        )),
        "{error}"
    );

    // A replacement registered after the read is new to it in the same way.
    let read = dataset.clone();
    let staged = stage_all(&read, "summary", "stale").await;
    let replaced = dataset
        .replace_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default().with_clear_on_write(["body"]),
        )
        .await
        .unwrap();
    let error = commit_replacement(&read, staged, set_true(replaced.flag_id, full(&[0, 1])))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("was not registered"), "{error}");

    dataset.checkout_latest().await.unwrap();
    assert_eq!(dataset.version().version, registered_at + 1);
    assert!(
        dataset
            .cell_flag_true_rows(replaced.flag_id)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_publication_needs_every_version_since_read() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let read = dataset.clone();
    let staged = stage_all(&read, "summary", "s").await;
    dataset.update_config([("a", "1")]).await.unwrap();
    let missing = dataset.version().version;
    dataset.update_config([("b", "2")]).await.unwrap();

    // Cleanup could remove a version whose transaction invalidated the rows.
    let missing_location = dataset
        .checkout_version(missing)
        .await
        .unwrap()
        .manifest_location
        .path
        .clone();
    dataset
        .object_store
        .delete(&missing_location)
        .await
        .unwrap();

    let error = commit_replacement(&read, staged, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains(&format!("could not all be loaded (missing: [{missing}])")),
        "{error}"
    );
}

#[tokio::test]
async fn test_publication_needs_a_read_version() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let staged = stage_all(&dataset, "summary", "s").await;
    let error = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(replacement_txn(0, staged, set_true(flag_id, full(&[0, 1]))))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
    assert!(error.to_string().contains("read_version is 0"), "{error}");
}

#[tokio::test]
async fn test_detached_and_batch_commits_are_refused() {
    let mut dataset = articles(false).await;
    register_ready(&mut dataset).await;
    let config = |dataset: &Dataset| {
        TransactionBuilder::new(
            dataset.manifest.version,
            Operation::UpdateConfig {
                config_updates: Some(translate_config_updates(
                    &HashMap::from([("k".to_string(), "v".to_string())]),
                    &[],
                )),
                table_metadata_updates: None,
                schema_metadata_updates: None,
                field_metadata_updates: HashMap::new(),
            },
        )
        .build()
    };
    let error = CommitBuilder::new(Arc::new(dataset.clone()))
        .with_detached(true)
        .execute(config(&dataset))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains("detached commits are not supported on datasets with cell flags"),
        "{error}"
    );

    let append = TransactionBuilder::new(
        dataset.manifest.version,
        Operation::Append { fragments: vec![] },
    )
    .cell_flag_changes(CellFlagChanges {
        drops: vec![1],
        ..Default::default()
    })
    .build();
    let Err(error) = CommitBuilder::new(Arc::new(dataset.clone()))
        .execute_batch(vec![append])
        .await
    else {
        panic!("a batch commit with cell flag changes must fail");
    };
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains("batch commits cannot carry cell flag changes"),
        "{error}"
    );

    // With every flag dropped nothing is left to track.
    dataset.drop_cell_flag("summary", "ready").await.unwrap();
    CommitBuilder::new(Arc::new(dataset.clone()))
        .with_detached(true)
        .execute(config(&dataset))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_clones_keep_cell_flags() {
    let test_dir = TempStdDir::default();
    let uri = |name: &str| test_dir.join(name).to_str().unwrap().to_string();
    let mut dataset = articles_at(&uri("source"), false).await;
    let flag_id = register_ready(&mut dataset).await;
    let groups = stage_all(&dataset, "summary", "s").await;
    let mut dataset = commit_replacement(&dataset, groups, set_true(flag_id, full(&[0])))
        .await
        .unwrap();
    let version = dataset.version().version;
    dataset.tags().create("v", version).await.unwrap();

    let shallow = dataset
        .shallow_clone(&uri("shallow"), "v", None)
        .await
        .unwrap();
    let deep = dataset.deep_clone(&uri("deep"), "v", None).await.unwrap();
    for clone in [shallow, deep] {
        assert_eq!(clone.manifest.cell_flags, dataset.manifest.cell_flags);
        assert_eq!(clone.cell_flag_true_rows(flag_id).unwrap(), full(&[0]));
        assert_ne!(
            clone.manifest.reader_feature_flags & FLAG_UNSTABLE_CELL_FLAGS,
            0
        );
        assert_ne!(
            clone.manifest.writer_feature_flags & FLAG_UNSTABLE_CELL_FLAGS,
            0
        );
    }
}

/// `summary.ready` staged against `dataset`, with field ids resolved there.
fn ready_registration(dataset: &Dataset) -> Transaction {
    let schema = dataset.schema();
    let field_id = |name: &str| schema.field(name).unwrap().id;
    TransactionBuilder::new(dataset.manifest.version, update_config())
        .cell_flag_changes(CellFlagChanges {
            registrations: vec![CellFlagRegistration {
                field_id: field_id("summary"),
                name: "ready".to_string(),
                clear_on_write: vec![field_id("title"), field_id("body")],
                mask_when_false: true,
            }],
            ..Default::default()
        })
        .build()
}

#[tokio::test]
async fn test_registration_conflicts_with_concurrent_overwrite() {
    let dataset = articles(false).await;
    let schema = dataset.schema();
    let field_id = |name: &str| schema.field(name).unwrap().id;
    let registration = ready_registration(&dataset);

    // The same field ids now name different columns.
    let replacement = record_batch!(
        ("sku", Utf8, ["a"]),
        ("qty", Utf8, ["1"]),
        ("name", Utf8, ["n"]),
        ("note", Utf8, [None::<&str>])
    )
    .unwrap();
    let overwritten = InsertBuilder::new(Arc::new(dataset.clone()))
        .with_params(&WriteParams {
            mode: WriteMode::Overwrite,
            ..Default::default()
        })
        .execute(vec![replacement])
        .await
        .unwrap();
    assert_eq!(
        overwritten.schema().field("note").unwrap().id,
        field_id("summary")
    );

    let error = CommitBuilder::new(Arc::new(dataset))
        .execute(registration)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::IncompatibleTransaction { .. }),
        "{error}"
    );
    let mut latest = overwritten;
    latest.checkout_latest().await.unwrap();
    assert!(latest.cell_flags().is_empty());
}

#[rstest]
#[case::dropped_id_reused(false)]
#[case::source_cast_to_new_id(true)]
#[tokio::test]
async fn test_registration_conflicts_with_concurrent_field_id_change(#[case] is_cast: bool) {
    let dataset = articles(false).await;
    let registration = ready_registration(&dataset);

    let mut changed = dataset.clone();
    if is_cast {
        let title_id = dataset.schema().field("title").unwrap().id;
        changed
            .alter_columns(&[ColumnAlteration::new("title".into()).cast_to(DataType::LargeUtf8)])
            .await
            .unwrap();
        assert_ne!(changed.schema().field("title").unwrap().id, title_id);
    } else {
        // No data file holds the dropped column, so its id goes to the next one.
        let summary_id = dataset.schema().field("summary").unwrap().id;
        changed.drop_columns(&["summary"]).await.unwrap();
        let note = ArrowSchema::new(vec![ArrowField::new("note", DataType::Utf8, true)]);
        changed
            .add_columns(NewColumnTransform::AllNulls(Arc::new(note)), None, None)
            .await
            .unwrap();
        assert_eq!(changed.schema().field("note").unwrap().id, summary_id);
    }

    let error = CommitBuilder::new(Arc::new(dataset))
        .execute(registration)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::RetryableCommitConflict { .. }),
        "{error}"
    );
    changed.checkout_latest().await.unwrap();
    assert!(changed.cell_flags().is_empty());
}

#[tokio::test]
async fn test_registration_rebases_over_concurrent_add_columns() {
    let dataset = articles(false).await;
    let registration = ready_registration(&dataset);
    let mut added = dataset.clone();
    let extra = ArrowSchema::new(vec![ArrowField::new("extra", DataType::Utf8, true)]);
    added
        .add_columns(NewColumnTransform::AllNulls(Arc::new(extra)), None, None)
        .await
        .unwrap();

    let registered = CommitBuilder::new(Arc::new(dataset))
        .execute(registration)
        .await
        .unwrap();
    let ready = registered.cell_flag("summary", "ready").unwrap();
    assert!(ready.mask_when_false);
    assert!(registered.schema().field("extra").is_some());
}

#[tokio::test]
async fn test_creating_a_dataset_with_cell_flag_changes_is_refused() {
    let schema = ArrowSchema::new(vec![ArrowField::new("id", DataType::Int32, false)]);
    let transaction = TransactionBuilder::new(
        0,
        Operation::Overwrite {
            fragments: vec![],
            schema: LanceSchema::try_from(&schema).unwrap(),
            config_upsert_values: None,
            initial_bases: None,
        },
    )
    .cell_flag_changes(CellFlagChanges {
        drops: vec![1],
        ..Default::default()
    })
    .build();
    let error = CommitBuilder::new("memory://")
        .execute(transaction)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains("cannot be committed with Overwrite, which creates the dataset"),
        "{error}"
    );
}

#[tokio::test]
async fn test_flag_update_conflicts_with_concurrent_row_move() {
    let mut dataset = articles(false).await;
    let reviewed = dataset
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap();
    // Sets the flag on id 2, fragment 0 offset 1, as addressed at this version.
    let mut row = RowAddrTreeMap::new();
    row.insert(1);
    let staged = TransactionBuilder::new(dataset.manifest.version, update_config())
        .cell_flag_changes(CellFlagChanges {
            updates: set_true(reviewed.flag_id, row),
            ..Default::default()
        })
        .build();

    // Moves id 2 into a new fragment.
    UpdateBuilder::new(Arc::new(dataset.clone()))
        .update_where("id = 2")
        .unwrap()
        .set("body", "'moved'")
        .unwrap()
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap();

    let error = CommitBuilder::new(Arc::new(dataset))
        .execute(staged)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::RetryableCommitConflict { .. }),
        "{error}"
    );
}

#[tokio::test]
async fn test_stale_merge_conflicts_instead_of_being_refused() {
    let mut dataset = articles(false).await;
    let flag_id = register_ready(&mut dataset).await;
    let mut stale = dataset.clone();
    let groups = stage_all(&dataset, "summary", "s").await;
    commit_replacement(&dataset, groups, set_true(flag_id, full(&[0, 1])))
        .await
        .unwrap();

    // Built from the fragments before the publication, so it would appear to
    // rewrite summary if checked against the head before the rebase.
    let extra = ArrowSchema::new(vec![ArrowField::new("extra", DataType::Utf8, true)]);
    let error = stale
        .add_columns(NewColumnTransform::AllNulls(Arc::new(extra)), None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::RetryableCommitConflict { .. }),
        "{error}"
    );
}
