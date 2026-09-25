// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Cell flag state across updates of existing rows. `UpdateBuilder` moves the
//! flags of the rows it rewrites into new fragments. A row-moving
//! `merge_insert` or a hand-staged update cannot, so it is refused where an
//! ordinary flag is true and leaves the rows it moves unassigned for dependent
//! flags. A partial-schema `merge_insert` rewrites columns in place and clears
//! only the flags watching them.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Int32Type, UInt64Type};
use arrow_array::{
    Array, ArrayRef, Int32Array, RecordBatch, RecordBatchIterator, StringArray, record_batch,
};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use lance_core::utils::address::RowAddress;
use lance_core::{Error, ROW_ADDR};
use lance_select::{RowAddrTreeMap, RowSetOps};
use roaring::RoaringBitmap;
use rstest::rstest;

use super::dataset_cell_flags::{
    cleared, commit_replacement, full, recorded_invalidations, set_true, stage_all, update_config,
};
use crate::dataset::cell_flag::CellFlagOptions;
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{
    CellFlagChanges, Operation, Transaction, TransactionBuilder, UpdateMode,
};
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::write::{CommitBuilder, InsertBuilder, WriteMode, WriteParams};
use crate::dataset::{MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder};
use crate::{Dataset, Result};

struct Flags {
    /// `summary.ready`: dependent on title and body, masking.
    summary: u32,
    /// `translation.ready`: dependent on body and language, masking.
    translation: u32,
    /// `title.reviewed`: ordinary.
    reviewed: u32,
}

/// Three fragments of two articles each, ids 1 to 6 in order, with the
/// computed `summary` and `translation` columns declared but never written.
async fn articles(stable_row_ids: bool) -> Dataset {
    let batch = record_batch!(
        ("id", Int32, [1, 2, 3, 4, 5, 6]),
        ("title", Utf8, ["t1", "t2", "t3", "t4", "t5", "t6"]),
        ("body", Utf8, ["b1", "b2", "b3", "b4", "b5", "b6"]),
        ("language", Utf8, ["en", "fr", "en", "de", "en", "fr"]),
        ("views", Int32, [10, 20, 30, 40, 50, 60])
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
    let outputs = ArrowSchema::new(vec![
        ArrowField::new("summary", DataType::Utf8, true),
        ArrowField::new("translation", DataType::Utf8, true),
    ]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(outputs)), None, None)
        .await
        .unwrap();
    assert_eq!(dataset.get_fragments().len(), 3);
    dataset
}

async fn flagged_articles(stable_row_ids: bool) -> (Dataset, Flags) {
    let mut dataset = articles(stable_row_ids).await;
    let masked_on = |sources: &[&str]| {
        CellFlagOptions::default()
            .with_clear_on_write(sources.iter().copied())
            .with_mask_when_false(true)
    };
    let summary = dataset
        .register_cell_flag("summary", "ready", masked_on(&["title", "body"]))
        .await
        .unwrap();
    let translation = dataset
        .register_cell_flag("translation", "ready", masked_on(&["body", "language"]))
        .await
        .unwrap();
    let reviewed = dataset
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap();
    let flags = Flags {
        summary: summary.flag_id,
        translation: translation.flag_id,
        reviewed: reviewed.flag_id,
    };
    (dataset, flags)
}

/// `flagged_articles` with `summary` published on every row but id 3, which
/// stays pending, `translation` published on every row, and `reviewed` set on
/// `reviewed_ids`. Published values are `{s|t}-{fragment}-{offset}`.
async fn published_articles(stable_row_ids: bool, reviewed_ids: &[i32]) -> (Dataset, Flags) {
    let (dataset, flags) = Box::pin(flagged_articles(stable_row_ids)).await;
    let mut all_but_id_3 = full(&[0, 2]);
    all_but_id_3.insert_bitmap(1, RoaringBitmap::from_iter([1_u32]));
    let groups = stage_all(&dataset, "summary", "s").await;
    let dataset = commit_replacement(&dataset, groups, set_true(flags.summary, all_but_id_3))
        .await
        .unwrap();
    let groups = stage_all(&dataset, "translation", "t").await;
    let dataset = commit_replacement(
        &dataset,
        groups,
        set_true(flags.translation, full(&[0, 1, 2])),
    )
    .await
    .unwrap();
    let dataset = if reviewed_ids.is_empty() {
        dataset
    } else {
        Box::pin(set_reviewed(&dataset, flags.reviewed, reviewed_ids)).await
    };
    (dataset, flags)
}

/// Commit, against `dataset`'s version, setting the ordinary flag `reviewed`
/// on `ids`.
async fn set_reviewed(dataset: &Dataset, reviewed: u32, ids: &[i32]) -> Dataset {
    let rows = row_addrs_of(dataset, ids).await;
    let transaction = TransactionBuilder::new(dataset.manifest.version, update_config())
        .cell_flag_changes(CellFlagChanges {
            updates: set_true(reviewed, rows),
            ..Default::default()
        })
        .build();
    CommitBuilder::new(Arc::new(dataset.clone()))
        .execute(transaction)
        .await
        .unwrap()
}

/// The value published for `id` before anything moved it.
fn published(prefix: &str, id: i32) -> String {
    format!("{prefix}-{}-{}", (id - 1) / 2, (id - 1) % 2)
}

/// Where the fixture writes `id`.
fn original_addr(id: i32) -> u64 {
    RowAddress::new_from_parts(((id - 1) / 2) as u32, ((id - 1) % 2) as u32).into()
}

async fn scan_with_addrs(dataset: &Dataset, columns: &[&str]) -> RecordBatch {
    let mut scanner = dataset.scan();
    scanner.project(columns).unwrap().with_row_address();
    scanner.try_into_batch().await.unwrap()
}

async fn addrs_by_id(dataset: &Dataset) -> BTreeMap<i32, u64> {
    let batch = scan_with_addrs(dataset, &["id"]).await;
    let ids = batch["id"].as_primitive::<Int32Type>();
    let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
    ids.values()
        .iter()
        .copied()
        .zip(addrs.values().iter().copied())
        .collect()
}

async fn row_addrs_of(dataset: &Dataset, ids: &[i32]) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for (id, addr) in addrs_by_id(dataset).await {
        if ids.contains(&id) {
            rows.insert(addr);
        }
    }
    rows
}

/// The live rows, by id, where `flag_id` is true.
async fn true_ids(dataset: &Dataset, flag_id: u32) -> BTreeSet<i32> {
    let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
    addrs_by_id(dataset)
        .await
        .into_iter()
        .filter(|(_, addr)| true_rows.contains(*addr))
        .map(|(id, _)| id)
        .collect()
}

/// `column` by id as the scan reads it, which must be NULL wherever
/// `flag_id` is false, whatever is stored there.
async fn masked(dataset: &Dataset, column: &str, flag_id: u32) -> BTreeMap<i32, Option<String>> {
    let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
    let batch = scan_with_addrs(dataset, &["id", column]).await;
    let ids = batch["id"].as_primitive::<Int32Type>();
    let values = batch[column].as_string::<i32>();
    let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
    (0..batch.num_rows())
        .map(|row| {
            assert!(
                true_rows.contains(addrs.value(row)) || values.is_null(row),
                "{column} of id {} reads a value under a false flag",
                ids.value(row)
            );
            (
                ids.value(row),
                values.is_valid(row).then(|| values.value(row).to_string()),
            )
        })
        .collect()
}

fn update_builder(dataset: &Dataset, predicate: &str, column: &str, value: &str) -> UpdateBuilder {
    UpdateBuilder::new(Arc::new(dataset.clone()))
        .update_where(predicate)
        .unwrap()
        .set(column, value)
        .unwrap()
}

async fn update(dataset: &Dataset, predicate: &str, column: &str, value: &str) -> Dataset {
    let result = update_builder(dataset, predicate, column, value)
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap();
    result.new_dataset.as_ref().clone()
}

#[rstest]
#[case::unrelated_field("views", "views + 1", false, false)]
#[case::summary_source("title", "'retitled'", true, false)]
#[case::shared_source("body", "'rewritten'", true, true)]
#[case::translation_source("language", "'es'", false, true)]
#[case::summary_output("summary", "'edited'", true, false)]
#[tokio::test]
async fn test_update_moves_flags_whose_watched_fields_it_does_not_set(
    #[case] column: &str,
    #[case] value: &str,
    #[case] clears_summary: bool,
    #[case] clears_translation: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (dataset, flags) = published_articles(stable_row_ids, &[2, 4]).await;
    // One row of each fragment; id 3 is pending for summary.
    let updated = [2, 3, 5];
    let result = update_builder(&dataset, "id IN (2, 3, 5)", column, value)
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap();
    assert_eq!(result.rows_updated, 3);
    let dataset = result.new_dataset;

    let addrs = addrs_by_id(&dataset).await;
    for id in 1..=6 {
        let fragment_id = RowAddress::from(addrs[&id]).fragment_id();
        if updated.contains(&id) {
            assert_eq!(fragment_id, 3, "id {id} moved to the new fragment");
        } else {
            assert_eq!(addrs[&id], original_addr(id), "id {id} stayed in place");
        }
    }
    for (output, flag_id, prefix, clears) in [
        ("summary", flags.summary, "s", clears_summary),
        ("translation", flags.translation, "t", clears_translation),
    ] {
        let expected: BTreeMap<i32, Option<String>> = (1..=6)
            .map(|id| {
                let is_pending = output == "summary" && id == 3;
                let is_cleared = clears && updated.contains(&id);
                (
                    id,
                    (!is_pending && !is_cleared).then(|| published(prefix, id)),
                )
            })
            .collect();
        assert_eq!(
            masked(&dataset, output, flag_id).await,
            expected,
            "{output}"
        );
    }
    // Ordinary flags move whatever the update sets, their own field included.
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([2, 4])
    );

    let changes = dataset
        .read_transaction()
        .await
        .unwrap()
        .unwrap()
        .cell_flag_changes
        .unwrap();
    assert_eq!(
        changes.moved_rows_written_fields,
        Some(vec![dataset.schema().field(column).unwrap().id])
    );
    let sources: Vec<u64> = changes
        .moved_rows
        .iter()
        .flat_map(|moved| moved.source_row_addrs.iter())
        .collect();
    assert_eq!(sources, updated.map(original_addr));
}

#[rstest]
#[case::unrelated_field("views", false, false)]
#[case::summary_source("title", true, false)]
#[case::shared_source("body", true, true)]
#[case::translation_source("language", false, true)]
#[case::summary_output("summary", true, false)]
#[tokio::test]
async fn test_in_place_merge_insert_clears_only_flags_watching_it(
    #[case] column: &str,
    #[case] clears_summary: bool,
    #[case] clears_translation: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (dataset, flags) = published_articles(stable_row_ids, &[2, 4]).await;
    // One row of each fragment; id 3 is pending for summary.
    let written = [2, 3, 5];
    let values: ArrayRef = if column == "views" {
        Arc::new(Int32Array::from(vec![21, 31, 51]))
    } else {
        Arc::new(StringArray::from(vec!["x"; 3]))
    };
    let source = RecordBatch::try_from_iter_with_nullable([
        (
            "id",
            Arc::new(Int32Array::from(written.to_vec())) as ArrayRef,
            false,
        ),
        (column, values, true),
    ])
    .unwrap();
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

    let addrs = addrs_by_id(&dataset).await;
    assert!(
        (1..=6).all(|id| addrs[&id] == original_addr(id)),
        "rows keep their addresses"
    );
    let mut expected_clears = Vec::new();
    for (output, flag_id, prefix, clears) in [
        ("summary", flags.summary, "s", clears_summary),
        ("translation", flags.translation, "t", clears_translation),
    ] {
        let expected: BTreeMap<i32, Option<String>> = (1..=6)
            .map(|id| {
                let is_pending = output == "summary" && id == 3;
                let is_cleared = clears && written.contains(&id);
                (
                    id,
                    (!is_pending && !is_cleared).then(|| published(prefix, id)),
                )
            })
            .collect();
        assert_eq!(
            masked(&dataset, output, flag_id).await,
            expected,
            "{output}"
        );
        if clears {
            let mut rows = RowAddrTreeMap::new();
            for id in written {
                rows.insert(original_addr(id));
            }
            // Recorded on id 3 too, where summary was already false.
            expected_clears.extend(cleared(flag_id, rows));
        }
    }
    assert_eq!(recorded_invalidations(&dataset).await, expected_clears);
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([2, 4])
    );
}

#[rstest]
#[tokio::test]
async fn test_update_after_deletions_moves_flags(#[values(false, true)] stable_row_ids: bool) {
    let (mut dataset, flags) = published_articles(stable_row_ids, &[2, 4]).await;
    dataset.delete("id = 1").await.unwrap();
    // id 2 is the last live row of fragment 0, so the update removes it.
    let dataset = update(&dataset, "id IN (2, 4)", "title", "'retitled'").await;
    assert!(
        dataset
            .get_fragments()
            .iter()
            .all(|fragment| fragment.id() != 0),
        "fragment 0 is removed"
    );

    let summaries = masked(&dataset, "summary", flags.summary).await;
    let expected: BTreeMap<i32, Option<String>> = [
        (2, None),
        (3, None),
        (4, None),
        (5, Some(published("s", 5))),
        (6, Some(published("s", 6))),
    ]
    .into_iter()
    .collect();
    assert_eq!(summaries, expected);
    let translations = masked(&dataset, "translation", flags.translation).await;
    let expected: BTreeMap<i32, Option<String>> =
        (2..=6).map(|id| (id, Some(published("t", id)))).collect();
    assert_eq!(translations, expected);
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([2, 4])
    );
    let translation_rows = dataset.cell_flag_true_rows(flags.translation).unwrap();
    assert!(
        translation_rows.get(&0).is_none(),
        "state of the removed fragment is dropped"
    );
}

#[rstest]
#[tokio::test]
async fn test_update_records_moved_rows_only_when_flags_are_registered(
    #[values(false, true)] stable_row_ids: bool,
) {
    let plain = articles(stable_row_ids).await;
    let updated = update(&plain, "id = 2", "views", "1").await;
    assert!(
        updated
            .read_transaction()
            .await
            .unwrap()
            .unwrap()
            .cell_flag_changes
            .is_none()
    );

    // Nothing is published, but the commit copies the head's state, so the
    // moved rows are listed as soon as any flag is registered.
    let (dataset, _) = flagged_articles(stable_row_ids).await;
    let updated = update(&dataset, "id = 2", "views", "1").await;
    let changes = updated
        .read_transaction()
        .await
        .unwrap()
        .unwrap()
        .cell_flag_changes
        .unwrap();
    assert_eq!(changes.moved_rows.len(), 1);
    assert_eq!(
        changes.moved_rows[0]
            .source_row_addrs
            .iter()
            .collect::<Vec<_>>(),
        vec![original_addr(2)]
    );

    // An update that matches nothing moves nothing.
    let unchanged = update(&updated, "id = 99", "views", "1").await;
    assert_eq!(unchanged.version().version, updated.version().version + 1);
    assert_eq!(unchanged.manifest.cell_flags, updated.manifest.cell_flags);
    assert!(
        unchanged
            .read_transaction()
            .await
            .unwrap()
            .unwrap()
            .cell_flag_changes
            .is_none()
    );
}

#[rstest]
#[tokio::test]
async fn test_update_moves_flags_of_rows_moved_before(#[values(false, true)] stable_row_ids: bool) {
    let (dataset, flags) = published_articles(stable_row_ids, &[2]).await;
    let dataset = update(&dataset, "id = 2", "views", "views + 1").await;
    let moved_before = addrs_by_id(&dataset).await[&2];
    assert_eq!(RowAddress::from(moved_before).fragment_id(), 3);

    // The scan reads id 3 first although id 2 has the smaller stable row id,
    // so pairing by row id would swap their state.
    let dataset = update(&dataset, "id IN (2, 3)", "views", "views + 1").await;
    let changes = dataset
        .read_transaction()
        .await
        .unwrap()
        .unwrap()
        .cell_flag_changes
        .unwrap();
    let sources: Vec<u64> = changes
        .moved_rows
        .iter()
        .flat_map(|moved| moved.source_row_addrs.iter())
        .collect();
    assert_eq!(sources, vec![original_addr(3), moved_before]);
    let addrs = addrs_by_id(&dataset).await;
    assert_eq!(addrs[&3], u64::from(RowAddress::new_from_parts(4, 0)));
    assert_eq!(addrs[&2], u64::from(RowAddress::new_from_parts(4, 1)));

    let summaries: BTreeMap<i32, Option<String>> = (1..=6)
        .map(|id| (id, (id != 3).then(|| published("s", id))))
        .collect();
    assert_eq!(masked(&dataset, "summary", flags.summary).await, summaries);
    let translations: BTreeMap<i32, Option<String>> =
        (1..=6).map(|id| (id, Some(published("t", id)))).collect();
    assert_eq!(
        masked(&dataset, "translation", flags.translation).await,
        translations
    );
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([2])
    );
}

#[rstest]
#[tokio::test]
async fn test_update_moves_ordinary_flag_set_after_its_read(
    #[values(false, true)] stable_row_ids: bool,
) {
    let (stale, flags) = flagged_articles(stable_row_ids).await;
    let head = set_reviewed(&stale, flags.reviewed, &[2]).await;

    // Explicit flag updates do not conflict with the update, which copies the
    // state the head has, not the state it read.
    let dataset = update_builder(&stale, "id = 2", "title", "'retitled'")
        .conflict_retries(0)
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap()
        .new_dataset;
    assert_eq!(dataset.version().version, head.version().version + 1);
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([2])
    );
    assert_eq!(
        RowAddress::from(addrs_by_id(&dataset).await[&2]).fragment_id(),
        3
    );
}

#[rstest]
#[case::unrelated_field("views", "views + 1", false)]
#[case::summary_source("title", "'retitled'", true)]
#[tokio::test]
async fn test_update_retries_over_concurrent_publication(
    #[case] column: &str,
    #[case] value: &str,
    #[case] clears_summary: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (stale, flags) = flagged_articles(stable_row_ids).await;
    let groups = stage_all(&stale, "summary", "s").await;
    let publication = commit_replacement(&stale, groups, set_true(flags.summary, full(&[0, 1, 2])))
        .await
        .unwrap();

    // Read before the publication, which rewrote the fragments it moves rows
    // out of: committing would pair the head's true flags with the unpublished
    // values it read, so the commit conflicts.
    let error = update_builder(&stale, "id IN (2, 5)", column, value)
        .conflict_retries(0)
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::TooMuchWriteContention { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Attempted 0 retries"), "{error}");
    let mut latest = stale.clone();
    latest.checkout_latest().await.unwrap();
    assert_eq!(latest.version().version, publication.version().version);

    // The retry reads the published values and moves their flags with them.
    let dataset = update(&stale, "id IN (2, 5)", column, value).await;
    assert_eq!(dataset.version().version, publication.version().version + 1);
    let expected: BTreeMap<i32, Option<String>> = (1..=6)
        .map(|id| {
            let is_cleared = clears_summary && [2, 5].contains(&id);
            (id, (!is_cleared).then(|| published("s", id)))
        })
        .collect();
    assert_eq!(masked(&dataset, "summary", flags.summary).await, expected);
}

/// Every column of the published fixture for `id`, unchanged but for `views`,
/// so the write rewrites the whole row.
fn upsert_of(id: i32) -> RecordBatch {
    let language = ["en", "fr", "en", "de", "en", "fr"][(id - 1) as usize];
    let (title, body) = (format!("t{id}"), format!("b{id}"));
    let (summary, translation) = (published("s", id), published("t", id));
    record_batch!(
        ("id", Int32, [id]),
        ("title", Utf8, [title.as_str()]),
        ("body", Utf8, [body.as_str()]),
        ("language", Utf8, [language]),
        ("views", Int32, [id * 10 + 1]),
        ("summary", Utf8, [summary.as_str()]),
        ("translation", Utf8, [translation.as_str()])
    )
    .unwrap()
}

/// Row-moving writes that list no moved rows, so the commit cannot move the
/// flag state of the rows they rewrite.
#[derive(Clone, Copy, Debug)]
enum StatelessRowMove {
    /// An upsert of every column, which merge_insert commits as RewriteRows.
    MergeInsert,
    /// A RewriteRows update staged by hand, as the bindings can.
    StagedUpdate,
}

/// Move `id`, still at its original address, into a new fragment with
/// `write`.
async fn move_row(dataset: &Dataset, id: i32, write: StatelessRowMove) -> Result<Dataset> {
    match write {
        StatelessRowMove::MergeInsert => {
            let mut builder =
                MergeInsertBuilder::try_new(Arc::new(dataset.clone()), vec!["id".to_string()])?;
            builder
                .when_matched(WhenMatched::UpdateAll)
                .when_not_matched(WhenNotMatched::DoNothing);
            let (dataset, _) = builder
                .try_build()?
                .execute_batches(vec![upsert_of(id)])
                .await?;
            Ok(dataset.as_ref().clone())
        }
        StatelessRowMove::StagedUpdate => {
            let append = WriteParams {
                mode: WriteMode::Append,
                ..Default::default()
            };
            let appended = InsertBuilder::new(Arc::new(dataset.clone()))
                .with_params(&append)
                .execute_uncommitted(vec![upsert_of(id)])
                .await?;
            let Operation::Append { fragments } = appended.operation else {
                panic!("an insert stages an Append, not {:?}", appended.operation);
            };
            let source = RowAddress::from(original_addr(id));
            let (removed_fragment_ids, updated_fragments) = match dataset
                .get_fragment(source.fragment_id() as usize)
                .unwrap()
                .extend_deletions([source.row_offset()])
                .await?
            {
                Some(updated) => (vec![], vec![updated.metadata().clone()]),
                None => (vec![u64::from(source.fragment_id())], vec![]),
            };
            let operation = Operation::Update {
                removed_fragment_ids,
                updated_fragments,
                new_fragments: fragments,
                fields_modified: vec![],
                compacted_sstables: vec![],
                fields_for_preserving_frag_bitmap: vec![],
                update_mode: Some(UpdateMode::RewriteRows),
                inserted_rows_filter: None,
                updated_fragment_offsets: None,
            };
            CommitBuilder::new(Arc::new(dataset.clone()))
                .execute(Transaction::new(dataset.manifest.version, operation, None))
                .await
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_stateless_row_move_is_refused_where_an_ordinary_flag_is_true(
    #[values(StatelessRowMove::MergeInsert, StatelessRowMove::StagedUpdate)]
    write: StatelessRowMove,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (dataset, _) = published_articles(stable_row_ids, &[2]).await;
    let error = move_row(&dataset, 2, write).await.unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(
            "the update moves rows out of fragment 0, where the flag is true, without moving \
             the flag's state"
        ),
        "{error}"
    );
    let mut latest = dataset.clone();
    latest.checkout_latest().await.unwrap();
    assert_eq!(latest.version().version, dataset.version().version);
}

#[rstest]
#[tokio::test]
async fn test_stateless_row_move_unassigns_dependent_flags(
    #[values(StatelessRowMove::MergeInsert, StatelessRowMove::StagedUpdate)]
    write: StatelessRowMove,
    #[values(false, true)] stable_row_ids: bool,
) {
    // The ordinary flag is true only on a fragment the write does not touch.
    let (dataset, flags) = published_articles(stable_row_ids, &[4]).await;
    let dataset = move_row(&dataset, 2, write).await.unwrap();

    let transaction = dataset.read_transaction().await.unwrap().unwrap();
    assert!(
        matches!(
            transaction.operation,
            Operation::Update {
                update_mode: Some(UpdateMode::RewriteRows),
                ..
            }
        ),
        "{:?}",
        transaction.operation
    );
    assert!(transaction.cell_flag_changes.is_none());
    assert_ne!(
        RowAddress::from(addrs_by_id(&dataset).await[&2]).fragment_id(),
        0
    );
    // The row moved without its state, so both outputs are pending there
    // although neither input changed: safe, but the work is lost.
    for (output, flag_id, prefix, pending) in [
        ("summary", flags.summary, "s", [2, 3].as_slice()),
        ("translation", flags.translation, "t", [2].as_slice()),
    ] {
        let expected: BTreeMap<i32, Option<String>> = (1..=6)
            .map(|id| (id, (!pending.contains(&id)).then(|| published(prefix, id))))
            .collect();
        assert_eq!(
            masked(&dataset, output, flag_id).await,
            expected,
            "{output}"
        );
    }
    assert_eq!(
        true_ids(&dataset, flags.reviewed).await,
        BTreeSet::from([4])
    );
}

#[rstest]
#[tokio::test]
async fn test_update_leaves_no_flag_state_where_it_moved_rows_from(
    #[values(StatelessRowMove::MergeInsert, StatelessRowMove::StagedUpdate)]
    write: StatelessRowMove,
    #[values(false, true)] stable_row_ids: bool,
) {
    // id 2 holds the only true ordinary flag of fragment 0.
    let (dataset, flags) = published_articles(stable_row_ids, &[2]).await;
    let dataset = update(&dataset, "id = 2", "views", "views + 1").await;
    let moved_to = addrs_by_id(&dataset).await[&2];
    assert_eq!(RowAddress::from(moved_to).fragment_id(), 3);
    for flag_id in [flags.reviewed, flags.summary, flags.translation] {
        let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
        assert!(true_rows.contains(moved_to), "flag {flag_id}");
        assert!(!true_rows.contains(original_addr(2)), "flag {flag_id}");
    }

    // Nothing true is left in fragment 0, so a write that cannot move state
    // may move its other row.
    let dataset = move_row(&dataset, 1, write).await.unwrap();
    assert_ne!(
        RowAddress::from(addrs_by_id(&dataset).await[&1]).fragment_id(),
        0
    );
    // The moved row is all of its new fragment.
    assert_eq!(
        dataset.cell_flag_true_rows(flags.reviewed).unwrap(),
        full(&[3])
    );
}

#[tokio::test]
async fn test_update_keeping_dependent_flags_needs_every_version_since_read() {
    let (stale, flags) = flagged_articles(false).await;
    let groups = stage_all(&stale, "summary", "s").await;
    let publication = commit_replacement(&stale, groups, set_true(flags.summary, full(&[0, 1, 2])))
        .await
        .unwrap();
    let mut head = publication.clone();
    head.update_config([("a", "1")]).await.unwrap();

    // Cleanup could remove the publication's version. Without it the update
    // would copy the published flags onto the unpublished values it read.
    let missing = publication.version().version;
    head.object_store
        .delete(&publication.manifest_location.path)
        .await
        .unwrap();
    let error = update_builder(&stale, "id = 2", "views", "views + 1")
        .build()
        .unwrap()
        .execute()
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
    let mut latest = stale.clone();
    latest.checkout_latest().await.unwrap();
    assert_eq!(latest.version().version, head.version().version);
}

#[rstest]
#[tokio::test]
async fn test_row_move_retries_over_flag_registered_after_its_read(
    #[values(false, true)] stable_row_ids: bool,
) {
    let stale = articles(stable_row_ids).await;
    let mut dataset = stale.clone();
    let reviewed = dataset
        .register_cell_flag("title", "reviewed", CellFlagOptions::default())
        .await
        .unwrap()
        .flag_id;
    let head = set_reviewed(&dataset, reviewed, &[2]).await;

    // The update read no flag, so it lists no moved rows: committing would
    // leave the flag behind on the row it deletes.
    let error = update_builder(&stale, "id = 2", "views", "views + 1")
        .conflict_retries(0)
        .build()
        .unwrap()
        .execute()
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::TooMuchWriteContention { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Attempted 0 retries"), "{error}");
    let error = move_row(&stale, 2, StatelessRowMove::StagedUpdate)
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::RetryableCommitConflict { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains(&format!(
            "the Update moves rows out of fragment 0 without their cell flag state, and cell \
             flag 'reviewed' (flag id {reviewed}) on 'title' (field id 1) is true there but was \
             registered after version {}",
            stale.version().version
        )),
        "{error}"
    );
    // A merge_insert retries too, and from the latest version is refused,
    // since it never moves flag state.
    let error = move_row(&stale, 2, StatelessRowMove::MergeInsert)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains("moves rows out of fragment 0, where the flag is true"),
        "{error}"
    );
    let mut latest = stale.clone();
    latest.checkout_latest().await.unwrap();
    assert_eq!(latest.version().version, head.version().version);

    // The retry reads the registration and moves the flag with the row.
    let dataset = update(&stale, "id = 2", "views", "views + 1").await;
    assert_eq!(dataset.version().version, head.version().version + 1);
    let moved_to = addrs_by_id(&dataset).await[&2];
    assert_eq!(RowAddress::from(moved_to).fragment_id(), 3);
    assert_eq!(dataset.cell_flag_true_rows(reviewed).unwrap(), full(&[3]));
}
