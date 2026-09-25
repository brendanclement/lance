// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Read-path masking for cell flags registered with `mask_when_false`: every
//! consumer of the masked field sees NULL where the flag is false.

use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Int32Type, Int64Type, UInt64Type};
use arrow_array::{
    ArrayRef, Int32Array, RecordBatch, RecordBatchIterator, StringArray, UInt64Array, record_batch,
};
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use arrow_select::concat::concat_batches;
use datafusion::prelude::{DataFrame, SessionContext};
use futures::{StreamExt, TryStreamExt, stream};
use lance_core::Error;
use lance_core::datatypes::Schema as LanceSchema;
use lance_core::utils::address::RowAddress;
use lance_core::utils::tempfile::TempStrDir;
use lance_file::version::LanceFileVersion;
use lance_index::scalar::expression::IndexInformationProvider;
use lance_index::scalar::inverted::InvertedIndexParams;
use lance_index::scalar::{BuiltinIndexType, FullTextSearchQuery, ScalarIndexParams};
use lance_index::{IndexParams, IndexType};
use lance_select::{RowAddrSelection, RowSetOps};
use lance_table::format::{CellFlagRegistry, pb};
use lance_table::utils::stream::ReadBatchFutStream;
use roaring::RoaringBitmap;
use rstest::rstest;

use super::dataset_cell_flags::{commit_replacement, full, register_ready, set_true};
use crate::Dataset;
use crate::dataset::builder::DatasetBuilder;
use crate::dataset::cell_flag::CellFlagOptions;
use crate::dataset::fragment::FragReadConfig;
use crate::dataset::mem_wal::scanner::LsmScanner;
use crate::dataset::scanner::{AggregateExpr, ColumnOrdering, MaterializationStyle};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{DataReplacementGroup, Operation};
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::{
    MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder, WriteDestination, WriteParams,
};
use crate::index::{DatasetIndexExt, DatasetIndexInternalExt};
use crate::session::Session;

/// Every live article's `(id, summary)` as reads must see it; see
/// [`masked_articles`].
const VISIBLE: [(i32, Option<&str>); 9] = [
    (0, Some("s0")),
    (1, None),
    (2, Some("s2")),
    (4, Some("s4")),
    (5, None),
    (7, None),
    (8, None),
    (10, None),
    (11, None),
];

const LIVE_IDS: [i32; 9] = [0, 1, 2, 4, 5, 7, 8, 10, 11];

/// Twelve articles in three fragments of four, with `summary` masked by
/// `summary.ready` (which watches `title` and `body`) and published so that
/// each fragment holds a different flag state:
///
/// ```text
/// fragment  ids   flag true on     stored summary     reads as
/// 0         0-3   every row        s0 NULL s2 s3      s0 NULL s2 -
/// 1         4-7   offsets 0, 2     s4 s5 s6 s7        s4 NULL -  NULL
/// 2         8-11  no row           s8 s9 s10 s11      NULL -  NULL NULL
/// ```
///
/// Ids 3, 6 and 9 are deleted. Id 1 is assigned NULL (flag true); ids 5 and 7
/// are pending (flag false over a stored value). Returns the flag id.
async fn masked_articles(stable_row_ids: bool) -> (Dataset, u32) {
    masked_articles_at("memory://", stable_row_ids).await
}

async fn masked_articles_at(uri: &str, stable_row_ids: bool) -> (Dataset, u32) {
    let text = |prefix: &str| -> ArrayRef {
        Arc::new(StringArray::from_iter_values(
            (0..12).map(|id| format!("{prefix}{id}")),
        ))
    };
    let batch = RecordBatch::try_from_iter([
        (
            "id",
            Arc::new(Int32Array::from_iter_values(0..12)) as ArrayRef,
        ),
        ("title", text("t")),
        ("body", text("b")),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        uri,
        Some(WriteParams {
            max_rows_per_file: 4,
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
    let flag_id = register_ready(&mut dataset).await;

    let stored: Vec<Option<String>> = (0..12)
        .map(|id| (id != 1).then(|| format!("s{id}")))
        .collect();
    let mut groups = Vec::new();
    for (fragment_id, values) in stored.chunks(4).enumerate() {
        groups.push(stage_summaries(&dataset, fragment_id, values).await);
    }
    let unpublished = groups.pop().unwrap();
    let mut published = full(&[0]);
    published.insert_bitmap(1, RoaringBitmap::from_iter([0, 2]));
    let dataset = commit_replacement(&dataset, groups, set_true(flag_id, published))
        .await
        .unwrap();
    // Writing the output without publishing it leaves the flag false.
    let mut dataset = commit_replacement(&dataset, vec![unpublished], vec![])
        .await
        .unwrap();
    dataset.delete("id IN (3, 6, 9)").await.unwrap();
    (dataset, flag_id)
}

/// Stage a full-fragment `summary` file holding `values`, one per physical row.
async fn stage_summaries(
    dataset: &Dataset,
    fragment_id: usize,
    values: &[Option<String>],
) -> DataReplacementGroup {
    let schema = LanceSchema {
        fields: vec![dataset.schema().field("summary").unwrap().clone()],
        metadata: Default::default(),
    };
    let batch = RecordBatch::try_new(
        Arc::new(ArrowSchema::from(&schema)),
        vec![Arc::new(StringArray::from(values.to_vec()))],
    )
    .unwrap();
    dataset
        .get_fragment(fragment_id)
        .unwrap()
        .write_columns(stream::iter([Ok(batch)]), &schema)
        .await
        .unwrap()
}

/// The [`VISIBLE`] entries for `ids`, sorted by id.
fn expected(ids: &[i32]) -> Vec<(i32, Option<String>)> {
    VISIBLE
        .iter()
        .filter(|(id, _)| ids.contains(id))
        .map(|(id, summary)| (*id, summary.map(str::to_string)))
        .collect()
}

/// The `(id, summary)` pairs of `batch`, sorted by id.
fn id_summaries(batch: &RecordBatch) -> Vec<(i32, Option<String>)> {
    let ids = batch["id"].as_primitive::<Int32Type>();
    let summaries = batch["summary"].as_string::<i32>();
    let mut pairs: Vec<_> = ids
        .values()
        .iter()
        .zip(summaries.iter())
        .map(|(id, summary)| (*id, summary.map(str::to_string)))
        .collect();
    pairs.sort();
    pairs
}

async fn scan_id_summaries(dataset: &Dataset) -> Vec<(i32, Option<String>)> {
    let batch = dataset
        .scan()
        .project(&["id", "summary"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    id_summaries(&batch)
}

fn id_summary_projection(dataset: &Dataset) -> LanceSchema {
    dataset.schema().project(&["id", "summary"]).unwrap()
}

async fn collect_reads(batches: ReadBatchFutStream) -> RecordBatch {
    let batches: Vec<RecordBatch> = batches.buffered(1).try_collect().await.unwrap();
    concat_batches(&batches[0].schema(), &batches).unwrap()
}

async fn collect_frame(frame: DataFrame) -> RecordBatch {
    let schema = Arc::new(frame.schema().as_arrow().clone());
    let batches = frame.collect().await.unwrap();
    concat_batches(&schema, &batches).unwrap()
}

/// `sql` over `dataset` through [`Dataset::sql`], which reads through
/// `LanceTableProvider` and the scanner.
async fn sql_batch(dataset: &Dataset, sql: &str) -> RecordBatch {
    collect_frame(dataset.sql(sql).build().await.unwrap().into_dataframe()).await
}

/// `sql` over `dataset` registered directly as a DataFusion table provider,
/// which reads through `LanceScanExec` and filters above it.
async fn provider_batch(dataset: &Dataset, sql: &str) -> RecordBatch {
    let ctx = SessionContext::new();
    ctx.register_table("dataset", Arc::new(dataset.clone()))
        .unwrap();
    collect_frame(ctx.sql(sql).await.unwrap()).await
}

#[rstest]
#[tokio::test]
async fn test_masked_field_reads_null_where_flag_is_false(
    #[values(false, true)] stable_row_ids: bool,
) {
    let test_uri = TempStrDir::default();
    let (dataset, flag_id) = masked_articles_at(&test_uri, stable_row_ids).await;
    assert_eq!(scan_id_summaries(&dataset).await, expected(&LIVE_IDS));
    let reopened = DatasetBuilder::from_uri(&test_uri)
        .with_session(Arc::new(Session::default()))
        .load()
        .await
        .unwrap();
    assert_eq!(
        scan_id_summaries(&reopened).await,
        expected(&LIVE_IDS),
        "a cold reopen must mask from the stored flag state"
    );

    // Id 1 (assigned NULL) and id 5 (pending) read alike; only the flag state
    // tells them apart.
    let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
    assert_eq!(true_rows.get(&0), Some(&RowAddrSelection::Full));
    assert!(true_rows.contains(RowAddress::new_from_parts(0, 1).into()));
    assert!(!true_rows.contains(RowAddress::new_from_parts(1, 1).into()));
    assert!(true_rows.get(&2).is_none());

    // The masked values stay stored: dropping the flag exposes them.
    let mut unmasked = dataset.clone();
    unmasked.drop_cell_flag("summary", "ready").await.unwrap();
    let stored: Vec<(i32, Option<String>)> = LIVE_IDS
        .iter()
        .map(|id| (*id, (*id != 1).then(|| format!("s{id}"))))
        .collect();
    assert_eq!(scan_id_summaries(&unmasked).await, stored);
}

/// Fragment 0 needs no mask, 1 a partial one and 2 masks every row. One row
/// per batch checks that each batch is masked by its own physical offsets.
#[rstest]
#[case::all_true(0)]
#[case::partial(1)]
#[case::all_false(2)]
#[tokio::test]
async fn test_every_reader_funnel_masks_each_batch(#[case] fragment_id: usize) {
    let (dataset, _) = masked_articles(false).await;
    let fragment = dataset.get_fragment(fragment_id).unwrap();
    let reader = fragment
        .open(&id_summary_projection(&dataset), FragReadConfig::default())
        .await
        .unwrap();
    let ids = |offsets: &[i32]| -> Vec<i32> {
        offsets
            .iter()
            .map(|offset| fragment_id as i32 * 4 + offset)
            .collect()
    };

    let all = collect_reads(reader.read_all(1).await.unwrap()).await;
    assert_eq!(id_summaries(&all), expected(&ids(&[0, 1, 2, 3])));
    let ranges = collect_reads(
        reader
            .read_ranges(Arc::from(vec![0..1, 2..4]), 1)
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(id_summaries(&ranges), expected(&ids(&[0, 2, 3])));
    let taken = reader.take_as_batch(&[1, 3], None).await.unwrap();
    assert_eq!(id_summaries(&taken), expected(&ids(&[1, 3])));
    let range = reader.read_range_as_batch(1..4).await.unwrap();
    assert_eq!(id_summaries(&range), expected(&ids(&[1, 2, 3])));
}

#[rstest]
#[case::is_null("summary IS NULL", &[1, 5, 7, 8, 10, 11], false)]
#[case::is_not_null("summary IS NOT NULL", &[0, 2, 4], false)]
#[case::stale_value("summary = 's5'", &[], false)]
#[case::published_value("summary = 's4'", &[4], false)]
#[case::not_stale_value("NOT (summary = 's7')", &[0, 2, 4], false)]
#[case::indexed_and_is_null("id >= 4 AND summary IS NULL", &[5, 7, 8, 10, 11], true)]
#[case::indexed_and_stale_value("id < 8 AND summary = 's5'", &[], true)]
#[case::indexed_or_stale_value("id = 0 OR summary = 's8'", &[0], false)]
#[tokio::test]
async fn test_filters_see_masked_values(
    #[case] filter: &str,
    #[case] expected_ids: &[i32],
    #[case] uses_id_index: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (mut dataset, _) = masked_articles(stable_row_ids).await;
    dataset
        .create_index(
            &["id"],
            IndexType::BTree,
            None,
            &ScalarIndexParams::default(),
            false,
        )
        .await
        .unwrap();
    for use_scalar_index in [false, true] {
        let mut scan = dataset.scan();
        scan.project(&["id", "summary"])
            .unwrap()
            .filter(filter)
            .unwrap()
            .use_scalar_index(use_scalar_index);
        let plan = scan.explain_plan(false).await.unwrap();
        assert_eq!(
            plan.contains("ScalarIndexQuery"),
            use_scalar_index && uses_id_index,
            "{plan}"
        );
        let batch = scan.try_into_batch().await.unwrap();
        assert_eq!(
            id_summaries(&batch),
            expected(expected_ids),
            "{filter} with use_scalar_index={use_scalar_index}"
        );
    }
    let sql = format!("SELECT id, summary FROM dataset WHERE {filter}");
    for batch in [
        sql_batch(&dataset, &sql).await,
        provider_batch(&dataset, &sql).await,
    ] {
        assert_eq!(id_summaries(&batch), expected(expected_ids), "{sql}");
    }
    assert_eq!(
        dataset.count_rows(Some(filter.to_string())).await.unwrap(),
        expected_ids.len()
    );
}

/// Without an inverted index, full-text search tokenizes the values the
/// reader returns, so a stale token under a false flag never matches.
#[rstest]
#[case::published("s4", &[4])]
#[case::pending("s5", &[])]
#[case::fragment_all_false("s8", &[])]
#[tokio::test]
async fn test_full_text_search_sees_masked_values(
    #[case] token: &str,
    #[case] expected_ids: &[i32],
) {
    let (dataset, _) = masked_articles(false).await;
    let mut scan = dataset.scan();
    scan.project(&["id", "summary"])
        .unwrap()
        .full_text_search(
            FullTextSearchQuery::new(token.to_string())
                .with_column("summary".to_string())
                .unwrap(),
        )
        .unwrap();
    let plan = scan.explain_plan(false).await.unwrap();
    assert!(plan.contains("FlatMatchQuery"), "{plan}");
    let batch = scan.try_into_batch().await.unwrap();
    assert_eq!(id_summaries(&batch), expected(expected_ids), "{token}");
}

#[tokio::test]
async fn test_lsm_scan_of_a_masked_dataset_is_refused() {
    let (dataset, flag_id) = masked_articles(false).await;
    let summary_id = dataset.schema().field("summary").unwrap().id;
    let error = LsmScanner::new(Arc::new(dataset), vec![], vec!["id".to_string()])
        .project(&["id", "summary"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(&format!(
            "LSM scan: field 'summary' (field id {summary_id}) is masked by cell flag 'ready' \
             (flag id {flag_id}), and MemWAL rows carry no cell flag state"
        )),
        "{error}"
    );
}

#[tokio::test]
async fn test_aggregates_count_masked_cells_as_null() {
    let (dataset, _) = masked_articles(false).await;
    let mut scan = dataset.scan();
    scan.aggregate(
        AggregateExpr::builder()
            .count_star()
            .alias("rows")
            .count("summary")
            .alias("summaries")
            .max("summary")
            .alias("latest")
            .build(),
    )
    .unwrap();
    let scanned = scan.try_into_batch().await.unwrap();
    let sql = sql_batch(
        &dataset,
        "SELECT COUNT(*) AS rows, COUNT(summary) AS summaries, MAX(summary) AS latest \
         FROM dataset",
    )
    .await;
    for batch in [scanned, sql] {
        assert_eq!(batch["rows"].as_primitive::<Int64Type>().value(0), 9);
        assert_eq!(batch["summaries"].as_primitive::<Int64Type>().value(0), 3);
        // "s8" is stored but masked.
        assert_eq!(batch["latest"].as_string::<i32>().value(0), "s4");
    }
    assert_eq!(dataset.count_rows(None).await.unwrap(), 9);
    assert_eq!(
        dataset
            .count_rows(Some("summary IS NULL".to_string()))
            .await
            .unwrap(),
        6
    );
}

#[tokio::test]
async fn test_sort_and_group_by_see_masked_values() {
    let (dataset, _) = masked_articles(false).await;
    let mut scan = dataset.scan();
    scan.project(&["summary"])
        .unwrap()
        .order_by(Some(vec![ColumnOrdering::desc_nulls_last(
            "summary".to_string(),
        )]))
        .unwrap();
    let sorted = scan.try_into_batch().await.unwrap();
    let summaries: Vec<Option<&str>> = sorted["summary"].as_string::<i32>().iter().collect();
    let mut expected_order = vec![Some("s4"), Some("s2"), Some("s0")];
    expected_order.extend([None; 6]);
    assert_eq!(summaries, expected_order);

    let mut scan = dataset.scan();
    scan.aggregate(
        AggregateExpr::builder()
            .group_by("summary")
            .count_star()
            .alias("n")
            .build(),
    )
    .unwrap();
    let scanned = scan.try_into_batch().await.unwrap();
    let sql = sql_batch(
        &dataset,
        "SELECT summary, COUNT(*) AS n FROM dataset GROUP BY summary",
    )
    .await;
    for batch in [scanned, sql] {
        let mut groups: Vec<(Option<String>, i64)> = batch["summary"]
            .as_string::<i32>()
            .iter()
            .zip(batch["n"].as_primitive::<Int64Type>().values())
            .map(|(summary, n)| (summary.map(str::to_string), *n))
            .collect();
        groups.sort();
        assert_eq!(
            groups,
            vec![
                (None, 6),
                (Some("s0".to_string()), 1),
                (Some("s2".to_string()), 1),
                (Some("s4".to_string()), 1),
            ]
        );
    }
}

#[rstest]
#[tokio::test]
async fn test_takes_and_late_materialization_mask(#[values(false, true)] stable_row_ids: bool) {
    let (dataset, _) = masked_articles(stable_row_ids).await;

    // Logical offsets: contiguous in one fragment, sorted across fragments,
    // then unsorted.
    for (offsets, ids) in [
        ([0, 1, 2], [0, 1, 2]),
        ([0, 3, 4], [0, 4, 5]),
        ([3, 7, 0], [4, 10, 0]),
    ] {
        let batch = dataset
            .take(&offsets, id_summary_projection(&dataset))
            .await
            .unwrap();
        assert_eq!(id_summaries(&batch), expected(&ids), "offsets {offsets:?}");
    }

    let mut scan = dataset.scan();
    scan.with_row_id().project(&["id"]).unwrap();
    let batch = scan.try_into_batch().await.unwrap();
    let row_ids: HashMap<i32, u64> = batch["id"]
        .as_primitive::<Int32Type>()
        .values()
        .iter()
        .copied()
        .zip(
            batch["_rowid"]
                .as_primitive::<UInt64Type>()
                .values()
                .iter()
                .copied(),
        )
        .collect();
    for ids in [vec![4, 5], vec![7, 0, 10]] {
        let requested: Vec<u64> = ids.iter().map(|id| row_ids[id]).collect();
        let batch = dataset
            .take_rows(&requested, id_summary_projection(&dataset))
            .await
            .unwrap();
        assert_eq!(id_summaries(&batch), expected(&ids), "ids {ids:?}");
    }

    let mut scan = dataset.scan();
    scan.project(&["id", "summary"])
        .unwrap()
        .filter("id >= 4")
        .unwrap()
        .materialization_style(MaterializationStyle::AllLate);
    let plan = scan.explain_plan(false).await.unwrap();
    // `summary` is read by row id after the filter on `id`.
    assert!(
        plan.contains("projection=[summary], source=stream(_rowid)"),
        "{plan}"
    );
    let batch = scan.try_into_batch().await.unwrap();
    assert_eq!(id_summaries(&batch), expected(&[4, 5, 7, 8, 10, 11]));
}

#[rstest]
#[tokio::test]
async fn test_write_predicates_see_masked_values(#[values(false, true)] stable_row_ids: bool) {
    let (dataset, _) = masked_articles(stable_row_ids).await;
    let update = |predicate: &str| {
        UpdateBuilder::new(Arc::new(dataset.clone()))
            .update_where(predicate)
            .unwrap()
            .set("title", "'edited'")
            .unwrap()
            .build()
            .unwrap()
            .execute()
    };
    assert_eq!(update("summary = 's5'").await.unwrap().rows_updated, 0);
    let updated = update("summary IS NOT NULL").await.unwrap();
    assert_eq!(updated.rows_updated, 3);
    let batch = updated
        .new_dataset
        .scan()
        .project(&["id"])
        .unwrap()
        .filter("title = 'edited'")
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    let mut edited = batch["id"].as_primitive::<Int32Type>().values().to_vec();
    edited.sort();
    assert_eq!(edited, [0, 2, 4]);

    // Deletes commit to the same store as the update above, so they start
    // from a fresh copy of the fixture.
    let (mut deleting, _) = masked_articles(stable_row_ids).await;
    assert_eq!(
        deleting
            .delete("summary = 's8'")
            .await
            .unwrap()
            .num_deleted_rows,
        0
    );
    assert_eq!(
        deleting
            .delete("summary IS NULL")
            .await
            .unwrap()
            .num_deleted_rows,
        6
    );
    assert_eq!(scan_id_summaries(&deleting).await, expected(&[0, 2, 4]));
}

/// The version that writes a source is the version that masks the output.
#[rstest]
#[case::merge_insert_in_place(true)]
#[case::update_moving_rows(false)]
#[tokio::test]
async fn test_source_write_masks_output_in_its_own_version(#[case] is_in_place: bool) {
    let (dataset, _) = masked_articles(false).await;
    let previous = dataset.version().version;
    let written = if is_in_place {
        let source = record_batch!(("id", Int32, [4]), ("body", Utf8, ["rewritten"])).unwrap();
        MergeInsertBuilder::try_new(Arc::new(dataset), vec!["id".to_string()])
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
    } else {
        UpdateBuilder::new(Arc::new(dataset))
            .update_where("id = 4")
            .unwrap()
            .set("body", "'rewritten'")
            .unwrap()
            .build()
            .unwrap()
            .execute()
            .await
            .unwrap()
            .new_dataset
    };
    let version = written.version().version;
    assert_eq!(version, previous + 1);

    let read = |dataset: Dataset| async move {
        let batch = dataset
            .scan()
            .project(&["id", "body", "summary"])
            .unwrap()
            .filter("id IN (0, 4)")
            .unwrap()
            .try_into_batch()
            .await
            .unwrap();
        let bodies: HashMap<i32, String> = batch["id"]
            .as_primitive::<Int32Type>()
            .values()
            .iter()
            .copied()
            .zip(
                batch["body"]
                    .as_string::<i32>()
                    .iter()
                    .map(|body| body.unwrap().to_string()),
            )
            .collect();
        (bodies, id_summaries(&batch))
    };
    let (bodies, summaries) = read(written.checkout_version(version).await.unwrap()).await;
    assert_eq!(bodies[&4], "rewritten");
    assert_eq!(
        summaries,
        vec![(0, Some("s0".to_string())), (4, None)],
        "the new body must never be visible with the old summary"
    );
    let (bodies, summaries) = read(written.checkout_version(previous).await.unwrap()).await;
    assert_eq!(bodies[&4], "b4");
    assert_eq!(summaries, expected(&[0, 4]));
}

#[tokio::test]
async fn test_overlay_on_another_field_keeps_masking() {
    let (dataset, _) = masked_articles(false).await;
    let read_version = dataset.version().version;
    let fragment = dataset.get_fragment(1).unwrap();
    let mut overlay = fragment
        .write_overlay(&dataset.schema().project(&["id"]).unwrap())
        .await
        .unwrap();
    let addresses: Vec<u64> = [0, 1]
        .into_iter()
        .map(|offset| RowAddress::new_from_parts(1, offset).into())
        .collect();
    let values = RecordBatch::try_from_iter([
        (
            "_rowaddr",
            Arc::new(UInt64Array::from(addresses)) as ArrayRef,
        ),
        ("id", Arc::new(Int32Array::from(vec![104, 105])) as ArrayRef),
    ])
    .unwrap();
    overlay.write_batch(&values).await.unwrap();
    let group = overlay.finish().await.unwrap().unwrap();
    let dataset = Dataset::commit(
        WriteDestination::Dataset(Arc::new(dataset)),
        Operation::DataOverlay {
            groups: vec![group],
        },
        Some(read_version),
        None,
        None,
        Arc::new(Default::default()),
        false,
    )
    .await
    .unwrap();

    // Fragment 1 is partially masked: offset 0 (id 104) is true, offset 1
    // (id 105) pending, offset 3 (id 7) pending and not overlaid.
    let overlaid = vec![(7, None), (104, Some("s4".to_string())), (105, None)];
    let batch = dataset
        .scan()
        .project(&["id", "summary"])
        .unwrap()
        .filter("id >= 100 OR id = 7")
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    assert_eq!(id_summaries(&batch), overlaid);
    let taken = dataset
        .take(&[3, 4, 5], id_summary_projection(&dataset))
        .await
        .unwrap();
    assert_eq!(id_summaries(&taken), overlaid);
}

#[rstest]
#[case::btree(IndexType::BTree)]
#[case::bitmap(IndexType::Bitmap)]
#[case::zone_map(IndexType::ZoneMap)]
#[case::inverted(IndexType::Inverted)]
#[tokio::test]
async fn test_indexing_a_masked_field_fails_before_building(#[case] index_type: IndexType) {
    let (mut dataset, flag_id) = masked_articles(false).await;
    let version = dataset.version().version;
    let inverted = InvertedIndexParams::default();
    let scalar = match index_type {
        IndexType::Bitmap => ScalarIndexParams::for_builtin(BuiltinIndexType::Bitmap),
        IndexType::ZoneMap => ScalarIndexParams::for_builtin(BuiltinIndexType::ZoneMap),
        _ => ScalarIndexParams::default(),
    };
    let params: &dyn IndexParams = if index_type == IndexType::Inverted {
        &inverted
    } else {
        &scalar
    };
    let summary_id = dataset.schema().field("summary").unwrap().id;
    let expected = format!(
        "CreateIndex: column 'summary' (field id {summary_id}) is masked by cell flag 'ready' \
         (flag id {flag_id})"
    );

    let uncommitted = dataset
        .create_index_builder(&["summary"], index_type, params)
        .execute_uncommitted()
        .await
        .unwrap_err();
    let committed = dataset
        .create_index(&["summary"], index_type, None, params, false)
        .await
        .unwrap_err();
    for error in [uncommitted, committed] {
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(error.to_string().contains(&expected), "{error}");
    }
    assert_eq!(dataset.version().version, version);
    assert!(dataset.load_indices().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_masking_an_indexed_field_is_refused() {
    let mut dataset = summaries_at("memory://", LanceFileVersion::Stable).await;
    dataset
        .create_index(
            &["summary"],
            IndexType::BTree,
            None,
            &ScalarIndexParams::default(),
            false,
        )
        .await
        .unwrap();
    let version = dataset.version().version;
    let error = dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["id"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(
            "cannot register masking cell flag 'ready': index 'summary_idx' on 'summary'"
        ),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("would serve the values the flag masks"),
        "{error}"
    );
    assert_eq!(dataset.version().version, version);
    assert!(dataset.cell_flags().is_empty());
}

/// `dataset` with a masking flag on `field` that is false on every row, set
/// only in memory: commits refuse the states the read-path defenses guard
/// against, so there is no other way to reach them.
fn with_masking_flag_in_memory(dataset: &Dataset, field: &str) -> Dataset {
    let registry = CellFlagRegistry::try_from(pb::CellFlagRegistry {
        definitions: vec![pb::CellFlagDefinition {
            flag_id: 1,
            field_id: dataset.schema().field(field).unwrap().id,
            name: "ready".to_string(),
            clear_on_write: vec![dataset.schema().field("id").unwrap().id],
            mask_when_false: true,
        }],
        next_flag_id: 2,
        states: vec![],
    })
    .unwrap();
    let mut manifest = dataset.manifest.as_ref().clone();
    manifest.cell_flags = Some(Arc::new(registry));
    let mut dataset = dataset.clone();
    dataset.manifest = Arc::new(manifest);
    dataset
}

async fn summaries_at(uri: &str, storage: LanceFileVersion) -> Dataset {
    let batch = record_batch!(
        ("id", Int32, [1, 2, 3, 4]),
        ("summary", Utf8, ["s1", "s2", "s3", "s4"])
    )
    .unwrap();
    let schema = batch.schema();
    Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        uri,
        Some(WriteParams {
            max_rows_per_file: 2,
            data_storage_version: Some(storage),
            ..Default::default()
        }),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn test_scalar_index_on_a_masked_field_is_not_used() {
    let mut dataset = summaries_at("memory://", LanceFileVersion::Stable).await;
    dataset
        .create_index(
            &["summary"],
            IndexType::BTree,
            None,
            &ScalarIndexParams::default(),
            false,
        )
        .await
        .unwrap();
    assert!(
        dataset
            .scalar_index_info()
            .await
            .unwrap()
            .get_index("summary")
            .is_some()
    );

    let masked = with_masking_flag_in_memory(&dataset, "summary");
    // The index still maps 's1' to a row, which reads as NULL.
    for (filter, expected) in [("summary = 's1'", 0), ("summary IS NULL", 4)] {
        assert_eq!(
            masked.count_rows(Some(filter.to_string())).await.unwrap(),
            expected,
            "{filter}"
        );
    }
    assert!(
        masked
            .scalar_index_info()
            .await
            .unwrap()
            .get_index("summary")
            .is_none()
    );
}

#[rstest]
#[case::scan(None)]
#[case::pushdown_scan(Some("summary IS NULL"))]
#[tokio::test]
async fn test_v1_read_of_a_masked_field_is_refused(#[case] filter: Option<&str>) {
    let dataset = summaries_at("memory://", LanceFileVersion::Legacy).await;
    let masked = with_masking_flag_in_memory(&dataset, "summary");
    let mut scan = masked.scan();
    scan.project(&["id", "summary"]).unwrap();
    if let Some(filter) = filter {
        scan.filter(filter).unwrap();
        let plan = scan.explain_plan(false).await.unwrap();
        assert!(plan.contains("LancePushdownScan"), "{plan}");
    }
    let error = scan.try_into_batch().await.unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(
            "cannot read field 'summary' of fragment 0: cell flag 'ready' (flag id 1) masks \
             it, and the legacy (v1) storage format cannot mask cells"
        ),
        "{error}"
    );
}
