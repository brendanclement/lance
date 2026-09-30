// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Read-path masking for cell flags registered with `mask_when_false`: every
//! consumer of the masked field sees NULL where the flag is false.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Float32Type, Int32Type, Int64Type, UInt64Type};
use arrow_array::{
    Array, ArrayRef, FixedSizeListArray, Float32Array, Int32Array, RecordBatch,
    RecordBatchIterator, StringArray, UInt64Array, record_batch,
};
use arrow_buffer::BooleanBuffer;
use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
use arrow_select::concat::concat_batches;
use datafusion::physical_plan::collect;
use datafusion::prelude::{DataFrame, SessionContext, col};
use datafusion::scalar::ScalarValue;
use futures::{StreamExt, TryStreamExt, stream};
use lance_arrow::FixedSizeListArrayExt;
use lance_core::datatypes::{LANCE_UNENFORCED_PRIMARY_KEY_POSITION, Schema as LanceSchema};
use lance_core::utils::address::RowAddress;
use lance_core::utils::tempfile::TempStrDir;
use lance_core::{Error, ROW_ADDR, ROW_ID};
use lance_file::version::LanceFileVersion;
use lance_index::mem_wal::MEM_WAL_INDEX_NAME;
use lance_index::scalar::expression::IndexInformationProvider;
use lance_index::scalar::inverted::InvertedIndexParams;
use lance_index::scalar::{BuiltinIndexType, FullTextSearchQuery, ScalarIndexParams};
use lance_index::vector::DIST_COL;
use lance_index::vector::hnsw::builder::HnswBuildParams;
use lance_index::vector::ivf::IvfBuildParams;
use lance_index::vector::sq::builder::SQBuildParams;
use lance_index::{IndexParams, IndexType};
use lance_linalg::distance::DistanceType;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::format::{CellFlagRegistry, pb};
use lance_table::utils::stream::ReadBatchFutStream;
use roaring::RoaringBitmap;
use rstest::rstest;
use uuid::Uuid;

use super::dataset_cell_flags::{commit_replacement, full, register_ready, set_true};
use crate::Dataset;
use crate::dataset::builder::DatasetBuilder;
use crate::dataset::cell_flag::{
    CellFlagOptions, ComputedBatch, DeferralReason, DependencyConflictPolicy, FollowUpRows,
    PublicationResult, PublicationStager,
};
use crate::dataset::fragment::FragReadConfig;
use crate::dataset::mem_wal::scanner::{
    LsmDataSourceCollector, LsmFtsSearchPlanner, LsmPointLookupPlanner, LsmScanner,
    LsmVectorSearchPlanner,
};
use crate::dataset::mem_wal::{DatasetMemWalExt, ShardWriter, ShardWriterConfig};
use crate::dataset::scanner::{AggregateExpr, ColumnOrdering, MaterializationStyle};
use crate::dataset::schema_evolution::NewColumnTransform;
use crate::dataset::transaction::{DataReplacementGroup, Operation};
use crate::dataset::write::CommitBuilder;
use crate::dataset::write::merge_insert::{WhenMatched, WhenNotMatched};
use crate::dataset::{
    MergeInsertBuilder, MergeInsertWriteMode, UpdateBuilder, WriteDestination, WriteParams,
};
use crate::index::vector::VectorIndexParams;
use crate::index::{DatasetIndexExt, DatasetIndexInternalExt};
use crate::io::exec::QUERY_INDEX_COL;
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

/// The live ids whose rows `flag_id` is true on, sorted.
async fn flagged_ids(dataset: &Dataset, flag_id: u32) -> Vec<i32> {
    let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
    let mut scan = dataset.scan();
    scan.with_row_address().project(&["id"]).unwrap();
    let batch = scan.try_into_batch().await.unwrap();
    let mut ids: Vec<i32> = batch["id"]
        .as_primitive::<Int32Type>()
        .values()
        .iter()
        .zip(batch["_rowaddr"].as_primitive::<UInt64Type>().values())
        .filter(|(_, row_addr)| true_rows.contains(**row_addr))
        .map(|(id, _)| *id)
        .collect();
    ids.sort();
    ids
}

/// The latest version in storage. A failed call leaves the handle as it was
/// even if it committed first, so only storage shows whether it did.
async fn latest(dataset: &Dataset) -> Dataset {
    let mut latest = dataset.clone();
    latest.checkout_latest().await.unwrap();
    latest
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

/// [`masked_articles`] with a `vector` column and keyed on `id`.
async fn keyed_masked_articles() -> (Dataset, u32) {
    let (mut dataset, flag_id) = masked_articles(false).await;
    let item = Arc::new(ArrowField::new("item", DataType::Float32, true));
    let vector = ArrowSchema::new(vec![ArrowField::new(
        "vector",
        DataType::FixedSizeList(item, 2),
        true,
    )]);
    dataset
        .add_columns(NewColumnTransform::AllNulls(Arc::new(vector)), None, None)
        .await
        .unwrap();
    dataset
        .update_field_metadata()
        .update("id", [(LANCE_UNENFORCED_PRIMARY_KEY_POSITION, "1")])
        .unwrap()
        .await
        .unwrap();
    (dataset, flag_id)
}

/// [`keyed_masked_articles`], checked out where `summary.ready` masks it, and
/// a shard whose active memtable rewrites id 4's body next to the summary
/// published for its old body: the pair masking exists to hide. MemWAL cannot
/// be initialized while a flag masks a field, so the head drops the flag
/// first; the old version is read through time travel.
async fn masked_articles_with_fresh_row() -> (Arc<Dataset>, ShardWriter, u32) {
    let (mut dataset, flag_id) = keyed_masked_articles().await;
    let masked = dataset.clone();
    dataset.drop_cell_flag("summary", "ready").await.unwrap();
    dataset
        .initialize_mem_wal()
        .unsharded()
        .execute()
        .await
        .unwrap();
    let writer = dataset
        .mem_wal_writer(Uuid::new_v4(), ShardWriterConfig::default())
        .await
        .unwrap();
    let vectors =
        FixedSizeListArray::try_new_from_values(Float32Array::from(vec![1.0, 0.0]), 2).unwrap();
    let fresh = RecordBatch::try_new(
        Arc::new(ArrowSchema::from(dataset.schema())),
        vec![
            Arc::new(Int32Array::from(vec![4])),
            Arc::new(StringArray::from(vec!["t4"])),
            Arc::new(StringArray::from(vec!["rewritten"])),
            Arc::new(StringArray::from(vec!["s4"])),
            Arc::new(vectors),
        ],
    )
    .unwrap();
    writer.put(vec![fresh]).await.unwrap();
    (Arc::new(masked), writer, flag_id)
}

#[derive(Clone, Copy, Debug)]
enum LsmRead {
    Scan,
    PointLookup,
    VectorSearch,
    FullTextSearch,
    FreshTierMembership,
}

/// Run `read` over `base`, plus the memtables of `writer` when given, to the
/// end of its output.
async fn run_lsm_read(
    read: LsmRead,
    base: &Arc<Dataset>,
    writer: Option<&ShardWriter>,
) -> lance_core::Result<()> {
    let pk = vec!["id".to_string()];
    let schema = Arc::new(ArrowSchema::from(base.schema()));
    let projection = ["id".to_string(), "summary".to_string()];
    let mut collector = LsmDataSourceCollector::new(base.clone(), vec![]);
    let mut scanner = LsmScanner::new(base.clone(), vec![], pk.clone());
    if let Some(writer) = writer {
        let memtables = writer.in_memory_memtable_refs().await?;
        collector = collector.with_in_memory_memtables(writer.shard_id(), memtables.clone());
        scanner = scanner.with_in_memory_memtables(writer.shard_id(), memtables);
    }
    let task_ctx = SessionContext::new().task_ctx();
    match read {
        LsmRead::Scan => {
            scanner.project(&projection)?.try_into_batch().await?;
        }
        LsmRead::PointLookup => {
            LsmPointLookupPlanner::new(collector, pk, schema)?
                .lookup(&[ScalarValue::Int32(Some(4))], Some(projection.as_slice()))
                .await?;
        }
        LsmRead::VectorSearch => {
            let query =
                FixedSizeListArray::try_new_from_values(Float32Array::from(vec![1.0, 0.0]), 2)?;
            let plan = LsmVectorSearchPlanner::new(
                collector,
                pk,
                schema,
                "vector".to_string(),
                DistanceType::L2,
            )
            .with_filter(Some(col("summary").is_null()))
            .plan_search(&query, 1, 1, Some(projection.as_slice()), false, 1.0)
            .await?;
            collect(plan, task_ctx).await?;
        }
        LsmRead::FullTextSearch => {
            let query =
                FullTextSearchQuery::new("s4".to_string()).with_column("summary".to_string())?;
            let plan = LsmFtsSearchPlanner::new(collector, pk, schema)
                .plan_search(query, Some(1), Some(projection.as_slice()))
                .await?;
            collect(plan, task_ctx).await?;
        }
        LsmRead::FreshTierMembership => {
            scanner
                .contains_pks(&record_batch!(("id", Int32, [4, 5]))?)
                .await?;
        }
    }
    Ok(())
}

/// Memtable and SSTable rows carry no cell flag state, so every LSM reader
/// over a base table with a masking flag refuses, whether or not a shard or
/// the masked column is involved.
#[rstest]
#[case::scan_without_shards(LsmRead::Scan, false)]
#[case::scan(LsmRead::Scan, true)]
#[case::point_lookup(LsmRead::PointLookup, true)]
#[case::vector_search(LsmRead::VectorSearch, true)]
#[case::full_text_search(LsmRead::FullTextSearch, true)]
#[case::fresh_tier_membership(LsmRead::FreshTierMembership, true)]
#[tokio::test]
async fn test_lsm_reads_of_a_masked_dataset_are_refused(
    #[case] read: LsmRead,
    #[case] has_fresh_row: bool,
) {
    let (dataset, writer, flag_id) = Box::pin(masked_articles_with_fresh_row()).await;
    let summary_id = dataset.schema().field("summary").unwrap().id;
    let error = run_lsm_read(read, &dataset, has_fresh_row.then_some(&writer))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(&format!(
            "LSM read: field 'summary' (field id {summary_id}) of the base table at version {} \
             is masked by cell flag 'ready' (flag id {flag_id}), and MemWAL rows carry no cell \
             flag state",
            dataset.version().version
        )),
        "{error}"
    );
}

/// What the refusal prevents: without the flag, the LSM readers serve the
/// memtable's new body next to the summary published for the old one.
#[tokio::test]
async fn test_lsm_reads_serve_fresh_rows_unmasked() {
    let (dataset, writer, _) = Box::pin(masked_articles_with_fresh_row()).await;
    let mut unmasked = dataset.as_ref().clone();
    unmasked.checkout_latest().await.unwrap();
    assert!(unmasked.cell_flags().is_empty());
    let unmasked = Arc::new(unmasked);
    let memtables = writer.in_memory_memtable_refs().await.unwrap();
    let projection = ["id", "body", "summary"].map(str::to_string);
    // The `(body, summary)` of id 4.
    let fresh = |batch: &RecordBatch| -> Vec<(String, Option<String>)> {
        batch["id"]
            .as_primitive::<Int32Type>()
            .values()
            .iter()
            .zip(batch["body"].as_string::<i32>().iter())
            .zip(batch["summary"].as_string::<i32>().iter())
            .filter(|((id, _), _)| **id == 4)
            .map(|((_, body), summary)| (body.unwrap().to_string(), summary.map(str::to_string)))
            .collect()
    };
    let stale = vec![("rewritten".to_string(), Some("s4".to_string()))];

    let scanned = LsmScanner::new(unmasked.clone(), vec![], vec!["id".to_string()])
        .with_in_memory_memtables(writer.shard_id(), memtables.clone())
        .project(&projection)
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    assert_eq!(fresh(&scanned), stale);
    let looked_up = LsmPointLookupPlanner::new(
        LsmDataSourceCollector::new(unmasked.clone(), vec![])
            .with_in_memory_memtables(writer.shard_id(), memtables),
        vec!["id".to_string()],
        Arc::new(ArrowSchema::from(unmasked.schema())),
    )
    .unwrap()
    .lookup(&[ScalarValue::Int32(Some(4))], Some(projection.as_slice()))
    .await
    .unwrap()
    .unwrap();
    assert_eq!(fresh(&looked_up), stale);
}

async fn initialized(dataset: &Dataset) -> lance_core::Result<Dataset> {
    let mut dataset = dataset.clone();
    dataset.initialize_mem_wal().unsharded().execute().await?;
    Ok(dataset)
}

/// How a MemWAL and a masking flag would come to coexist.
#[derive(Debug, Clone, Copy)]
enum MemWalBesideMask {
    /// Initialize MemWAL where a flag masks a field.
    Initialize,
    /// Initialize MemWAL from a version read before the flag was registered.
    InitializeStagedBeforeMask,
    /// Open a MemWAL writer where a flag masks a field.
    OpenWriter,
    /// Register a masking flag where MemWAL is initialized.
    RegisterMask,
    /// Register a masking flag from a version read before MemWAL was
    /// initialized.
    RegisterMaskStagedBeforeMemWal,
}

/// MemWAL rows carry no cell flag state, so fresh-tier reads, some of which
/// never see the base table's manifest, could not mask them. MemWAL and a
/// masking flag are therefore never both part of one version.
#[rstest]
#[case::initialize(MemWalBesideMask::Initialize)]
#[case::initialize_staged_before_mask(MemWalBesideMask::InitializeStagedBeforeMask)]
#[case::open_writer(MemWalBesideMask::OpenWriter)]
#[case::register_mask(MemWalBesideMask::RegisterMask)]
#[case::register_mask_staged_before_mem_wal(MemWalBesideMask::RegisterMaskStagedBeforeMemWal)]
#[tokio::test]
async fn test_mem_wal_and_masking_are_exclusive(#[case] path: MemWalBesideMask) {
    let (mut dataset, flag_id) = keyed_masked_articles().await;
    let summary_id = dataset.schema().field("summary").unwrap().id;
    let flag =
        format!("cell flag 'ready' (flag id {flag_id}) on 'summary' (field id {summary_id})");
    let mask = || {
        CellFlagOptions::default()
            .with_clear_on_write(["title", "body"])
            .with_mask_when_false(true)
    };
    let (error, expected) = match path {
        MemWalBesideMask::Initialize | MemWalBesideMask::InitializeStagedBeforeMask => {
            let mut staged = dataset.clone();
            if matches!(path, MemWalBesideMask::InitializeStagedBeforeMask) {
                dataset.drop_cell_flag("summary", "ready").await.unwrap();
                staged = dataset.clone();
                dataset
                    .register_cell_flag("summary", "ready", mask())
                    .await
                    .unwrap();
            }
            let flag_id = dataset.cell_flag("summary", "ready").unwrap().flag_id;
            let error = initialized(&staged).await.unwrap_err();
            (
                error,
                format!(
                    "CreateIndex is not supported on a dataset with cell flag 'ready' (flag id \
                     {flag_id}) on 'summary' (field id {summary_id}): MemWAL rows carry no cell \
                     flag state, so fresh-tier reads would serve the masked field unmasked"
                ),
            )
        }
        MemWalBesideMask::OpenWriter => {
            let Err(error) = dataset
                .mem_wal_writer(Uuid::new_v4(), ShardWriterConfig::default())
                .await
            else {
                panic!("a MemWAL writer opened where a flag masks a field");
            };
            (
                error,
                format!(
                    "cannot open a MemWAL writer: {flag} masks its field at version {}, and \
                     MemWAL rows carry no cell flag state to mask it with",
                    dataset.version().version
                ),
            )
        }
        MemWalBesideMask::RegisterMask | MemWalBesideMask::RegisterMaskStagedBeforeMemWal => {
            dataset.drop_cell_flag("summary", "ready").await.unwrap();
            let mut staged = dataset.clone();
            dataset = initialized(&dataset).await.unwrap();
            if matches!(path, MemWalBesideMask::RegisterMask) {
                staged = dataset.clone();
            }
            let error = staged
                .register_cell_flag("summary", "ready", mask())
                .await
                .unwrap_err();
            (
                error,
                format!(
                    "cannot register masking cell flag 'ready' on 'summary' (field id \
                     {summary_id}): MemWAL is initialized on this dataset, and MemWAL rows \
                     carry no cell flag state to mask it with"
                ),
            )
        }
    };
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(error.to_string().contains(&expected), "{error}");
    let latest = latest(&dataset).await;
    assert_eq!(latest.version().version, dataset.version().version);
    assert_eq!(latest.cell_flags(), dataset.cell_flags());
    let has_mem_wal = latest
        .load_indices()
        .await
        .unwrap()
        .iter()
        .any(|index| index.name == MEM_WAL_INDEX_NAME);
    let has_mask = latest.cell_flags().iter().any(|flag| flag.mask_when_false);
    assert!(has_mem_wal != has_mask, "exactly one of them is present");
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

/// Update predicates read masked values, and the rows an update moves keep
/// the flags whose inputs it does not set: setting `id` keeps ids 0, 2 and 4
/// published at their new addresses, while setting `title` masks them.
#[rstest]
#[case::watched_source("title", "'edited'", "title = 'edited'", false)]
#[case::unwatched_field("id", "id + 100", "id >= 100", true)]
#[tokio::test]
async fn test_write_predicates_see_masked_values(
    #[case] column: &str,
    #[case] value: &str,
    #[case] updated_filter: &str,
    #[case] keeps_flag: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (dataset, flag_id) = masked_articles(stable_row_ids).await;
    let update = |predicate: &str| {
        UpdateBuilder::new(Arc::new(dataset.clone()))
            .update_where(predicate)
            .unwrap()
            .set(column, value)
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
        .filter(updated_filter)
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    let mut edited = batch["id"].as_primitive::<Int32Type>().values().to_vec();
    edited.sort();
    let moved: Vec<i32> = [0, 2, 4]
        .into_iter()
        .map(|id| if keeps_flag { id + 100 } else { id })
        .collect();
    assert_eq!(edited, moved);
    let mut summaries: Vec<(i32, Option<String>)> = expected(&LIVE_IDS)
        .into_iter()
        .map(|(id, summary)| {
            if ![0, 2, 4].contains(&id) {
                (id, summary)
            } else if keeps_flag {
                (id + 100, summary)
            } else {
                (id, None)
            }
        })
        .collect();
    summaries.sort();
    assert_eq!(scan_id_summaries(&updated.new_dataset).await, summaries);
    let mut flagged = vec![1];
    if keeps_flag {
        flagged.extend(&moved);
    }
    assert_eq!(flagged_ids(&updated.new_dataset, flag_id).await, flagged);
    assert_eq!(flagged_ids(&dataset, flag_id).await, [0, 1, 2, 4]);

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
    assert_eq!(flagged_ids(&deleting, flag_id).await, [0, 2, 4]);
}

/// The version that writes a source is the version that masks the output,
/// and it clears the flag on the written row only. Id 0 is in the fragment
/// whose flag is true on every row, id 4 in the one where it is partial.
#[rstest]
#[case::merge_insert_in_place(true)]
#[case::update_moving_rows(false)]
#[tokio::test]
async fn test_source_write_masks_output_in_its_own_version(
    #[case] is_in_place: bool,
    #[values(0, 4)] written_id: i32,
) {
    let (dataset, flag_id) = masked_articles(false).await;
    let previous = dataset.version().version;
    let written = if is_in_place {
        let source =
            record_batch!(("id", Int32, [written_id]), ("body", Utf8, ["rewritten"])).unwrap();
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
            .update_where(&format!("id = {written_id}"))
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
            .filter("id IN (0, 2, 4)")
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
    let current = written.checkout_version(version).await.unwrap();
    let (bodies, summaries) = read(current.clone()).await;
    assert_eq!(bodies[&written_id], "rewritten");
    let mut masked = expected(&[0, 2, 4]);
    for (id, summary) in &mut masked {
        if *id == written_id {
            *summary = None;
        }
    }
    assert_eq!(
        summaries, masked,
        "the new body must never be visible with the old summary"
    );
    let untouched: Vec<i32> = [0, 1, 2, 4]
        .into_iter()
        .filter(|id| *id != written_id)
        .collect();
    assert_eq!(flagged_ids(&current, flag_id).await, untouched);

    let before = written.checkout_version(previous).await.unwrap();
    let (bodies, summaries) = read(before.clone()).await;
    assert_eq!(bodies[&written_id], format!("b{written_id}"));
    assert_eq!(summaries, expected(&[0, 2, 4]));
    assert_eq!(flagged_ids(&before, flag_id).await, [0, 1, 2, 4]);
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

/// Every index build path refuses a masked column before reading it: the
/// builder, the multi-segment FM-index build that `create_index` takes for
/// `num_segments > 1`, and committing a segment built before the flag was
/// registered, as a distributed build that raced the registration would.
/// Vector indexes are refused alike, so a masked vector is only ever searched
/// flat.
#[rstest]
#[case::btree(IndexType::BTree)]
#[case::bitmap(IndexType::Bitmap)]
#[case::zone_map(IndexType::ZoneMap)]
#[case::inverted(IndexType::Inverted)]
#[case::fm_multi_segment(IndexType::Fm)]
#[case::ivf_flat(IndexType::IvfFlat)]
#[case::ivf_hnsw_sq(IndexType::IvfHnswSq)]
#[tokio::test]
async fn test_indexing_a_masked_field_fails_before_building(#[case] index_type: IndexType) {
    let is_vector = matches!(index_type, IndexType::IvfFlat | IndexType::IvfHnswSq);
    // `masked_articles` adds `summary` at version 2 and registers the flag at
    // version 3; `masked_embeddings` writes `embedding` at version 1 and
    // registers at version 2.
    let (column, (mut dataset, flag_id), unregistered_version) = if is_vector {
        ("embedding", masked_embeddings(false).await, 1)
    } else {
        ("summary", masked_articles(false).await, 2)
    };
    let version = dataset.version().version;
    let inverted = InvertedIndexParams::default();
    let scalar = match index_type {
        IndexType::Bitmap => ScalarIndexParams::for_builtin(BuiltinIndexType::Bitmap),
        IndexType::ZoneMap => ScalarIndexParams::for_builtin(BuiltinIndexType::ZoneMap),
        IndexType::Fm => ScalarIndexParams {
            index_type: "fm".to_string(),
            params: Some(r#"{"num_segments": 2}"#.to_string()),
        },
        _ => ScalarIndexParams::default(),
    };
    let vector = if index_type == IndexType::IvfHnswSq {
        VectorIndexParams::with_ivf_hnsw_sq_params(
            DistanceType::L2,
            IvfBuildParams::new(1),
            HnswBuildParams::default(),
            SQBuildParams::default(),
        )
    } else {
        VectorIndexParams::ivf_flat(1, DistanceType::L2)
    };
    let params: &dyn IndexParams = if index_type == IndexType::Inverted {
        &inverted
    } else if is_vector {
        &vector
    } else {
        &scalar
    };
    let field_id = dataset.schema().field(column).unwrap().id;
    let expected = format!(
        "CreateIndex: column '{column}' (field id {field_id}) is masked by cell flag 'ready' \
         (flag id {flag_id})"
    );

    // The segment is left untrained: the refusal comes before it is read.
    let mut unregistered = dataset
        .checkout_version(unregistered_version)
        .await
        .unwrap();
    assert!(unregistered.cell_flags().is_empty());
    let segment = unregistered
        .create_index_builder(&[column], index_type, params)
        .train(false)
        .execute_uncommitted()
        .await
        .unwrap();

    let uncommitted = dataset
        .create_index_builder(&[column], index_type, params)
        .execute_uncommitted()
        .await
        .unwrap_err();
    let committed = dataset
        .create_index(&[column], index_type, None, params, false)
        .await
        .unwrap_err();
    let segment_name = segment.name.clone();
    let prebuilt = dataset
        .commit_existing_index_segments(&segment_name, column, vec![segment])
        .await
        .unwrap_err();
    for error in [uncommitted, committed, prebuilt] {
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(error.to_string().contains(&expected), "{error}");
    }
    let latest = latest(&dataset).await;
    assert_eq!(latest.version().version, version);
    assert!(latest.load_indices().await.unwrap().is_empty());
}

#[rstest]
#[case::btree("summary", IndexType::BTree)]
#[case::ivf_flat("embedding", IndexType::IvfFlat)]
#[tokio::test]
async fn test_masking_an_indexed_field_is_refused(
    #[case] column: &str,
    #[case] index_type: IndexType,
) {
    let mut dataset = summaries_at("memory://", LanceFileVersion::Stable).await;
    let scalar = ScalarIndexParams::default();
    let vector = VectorIndexParams::ivf_flat(1, DistanceType::L2);
    let params: &dyn IndexParams = if index_type == IndexType::IvfFlat {
        &vector
    } else {
        &scalar
    };
    dataset
        .create_index(&[column], index_type, None, params, false)
        .await
        .unwrap();
    let version = dataset.version().version;
    let error = dataset
        .register_cell_flag(
            column,
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["id"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap_err();
    assert!(matches!(error, Error::NotSupported { .. }), "{error}");
    assert!(
        error.to_string().contains(&format!(
            "cannot register masking cell flag 'ready': index '{column}_idx' on '{column}'"
        )),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("would serve the values the flag masks"),
        "{error}"
    );
    let latest = latest(&dataset).await;
    assert_eq!(latest.version().version, version);
    assert!(latest.cell_flags().is_empty());
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
    let vectors: Vec<_> = (1..=4).map(|n| Some(embedding_of(n as f32))).collect();
    let batch = RecordBatch::try_from_iter_with_nullable([
        (
            "id",
            Arc::new(Int32Array::from(vec![1, 2, 3, 4])) as ArrayRef,
            true,
        ),
        (
            "summary",
            Arc::new(StringArray::from(vec!["s1", "s2", "s3", "s4"])) as ArrayRef,
            true,
        ),
        ("embedding", embeddings(&vectors), true),
    ])
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

const DIM: i32 = 4;

/// The query of the nearest-neighbor searches over [`masked_embeddings`].
/// Every distance to it is exact in f32 and distinct, and the stale, deleted
/// and unpublished values are nearer to it than id 0, so a leak of any of
/// them changes the hits of a search whose `k` exceeds the visible rows.
const QUERY: [f32; 4] = [5.25, 0.0, 0.0, 1.0];

/// The `(id, distance)` hits of [`QUERY`] over [`masked_embeddings`].
const NEAREST: [(i32, f32); 3] = [(4, 1.5625), (2, 10.5625), (0, 27.5625)];

fn embedding_of(n: f32) -> Vec<f32> {
    vec![n, 0.0, 0.0, 1.0]
}

/// What `embedding` computes from a body: `b{n}` embeds as
/// [`embedding_of`]`(n)`, anything else as NULL.
fn embed(body: &str) -> Option<Vec<f32>> {
    body.strip_prefix('b')
        .map(|n| embedding_of(n.parse().unwrap()))
}

fn embeddings(values: &[Option<Vec<f32>>]) -> ArrayRef {
    Arc::new(
        FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            values
                .iter()
                .map(|value| value.as_ref().map(|value| value.iter().copied().map(Some))),
            DIM,
        ),
    )
}

fn addr(fragment_id: u32, offset: u32) -> u64 {
    RowAddress::new_from_parts(fragment_id, offset).into()
}

/// Twelve rows in three fragments of four, with `embedding`, a vector, masked
/// by `embedding.ready`, which watches `body`. Every row is written with
/// [`QUERY`] as its embedding. After the flag is registered, a publication
/// through [`PublicationStager`] computes ids 0-6 from their bodies and
/// leaves id 7 unassigned, so the stager copies id 7's masked NULL:
///
/// ```text
/// fragment  ids   stored embedding    reads as
/// 0         0-3   e0 NULL e2 e3        e0 NULL e2 -
/// 1         4-7   e4 e5 e6 NULL        e4 NULL -  NULL
/// 2         8-11  Q  Q  Q  Q           NULL -  NULL NULL
/// ```
///
/// `en` is [`embedding_of`]`(n)` and `Q` is [`QUERY`]. Id 1's body computes
/// a NULL embedding, published with a true flag; ids 3, 6 and 9 are deleted;
/// id 5's body is rewritten after the publication, leaving e5 stale under a
/// false flag. Returns the flag id.
async fn masked_embeddings(stable_row_ids: bool) -> (Dataset, u32) {
    masked_embeddings_at("memory://", stable_row_ids).await
}

async fn masked_embeddings_at(uri: &str, stable_row_ids: bool) -> (Dataset, u32) {
    let bodies = StringArray::from_iter_values((0..12).map(|id| {
        if id == 1 {
            "none".to_string()
        } else {
            format!("b{id}")
        }
    }));
    let batch = RecordBatch::try_from_iter_with_nullable([
        (
            "id",
            Arc::new(Int32Array::from_iter_values(0..12)) as ArrayRef,
            false,
        ),
        ("body", Arc::new(bodies) as ArrayRef, false),
        (
            "embedding",
            embeddings(&vec![Some(QUERY.to_vec()); 12]),
            true,
        ),
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
    let ready = dataset
        .register_cell_flag(
            "embedding",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    let addrs: Vec<u64> = (0..8).map(|row| addr(row / 4, row % 4)).collect();
    let computed = computed_embeddings(&dataset, &addrs, &[addr(1, 3)]).await;
    let mut dataset = publish_embeddings(&dataset, computed, DependencyConflictPolicy::Reject)
        .await
        .unwrap()
        .dataset;
    dataset.delete("id IN (3, 6, 9)").await.unwrap();
    let dataset = merge_insert_body(&dataset, 5, "b50").await;
    (dataset, ready.flag_id)
}

/// The `(id, embedding)` pairs of `batch`, sorted by id, NULL where the list
/// slot is.
fn id_embeddings(batch: &RecordBatch) -> Vec<(i32, Option<Vec<f32>>)> {
    let ids = batch["id"].as_primitive::<Int32Type>();
    let embeddings = batch["embedding"].as_fixed_size_list();
    let mut pairs: Vec<_> = (0..batch.num_rows())
        .map(|row| {
            let embedding = embeddings.is_valid(row).then(|| {
                embeddings
                    .value(row)
                    .as_primitive::<Float32Type>()
                    .values()
                    .to_vec()
            });
            (ids.value(row), embedding)
        })
        .collect();
    pairs.sort_by_key(|(id, _)| *id);
    pairs
}

/// The child values under the list slot of each of `ids` in `batch`, which a
/// reader of the list's values, such as `lance.torch`, takes whatever the
/// slot's validity.
fn embedding_children(batch: &RecordBatch, ids: &[i32]) -> Vec<(i32, Vec<Option<f32>>)> {
    let rows: HashMap<i32, usize> = batch["id"]
        .as_primitive::<Int32Type>()
        .values()
        .iter()
        .enumerate()
        .map(|(row, id)| (*id, row))
        .collect();
    let embeddings = batch["embedding"].as_fixed_size_list();
    ids.iter()
        .map(|id| {
            let children = embeddings.value(rows[id]);
            (*id, children.as_primitive::<Float32Type>().iter().collect())
        })
        .collect()
}

/// What [`embedding_children`] shows for masked `ids`: no stored value.
fn masked_children(ids: &[i32]) -> Vec<(i32, Vec<Option<f32>>)> {
    ids.iter()
        .map(|id| (*id, vec![None; DIM as usize]))
        .collect()
}

async fn scan_embeddings(dataset: &Dataset, batch_size: Option<usize>) -> RecordBatch {
    let mut scan = dataset.scan();
    scan.project(&["id", "embedding"]).unwrap();
    if let Some(batch_size) = batch_size {
        scan.batch_size(batch_size);
    }
    scan.try_into_batch().await.unwrap()
}

async fn scan_id_embeddings(
    dataset: &Dataset,
    batch_size: Option<usize>,
) -> Vec<(i32, Option<Vec<f32>>)> {
    id_embeddings(&scan_embeddings(dataset, batch_size).await)
}

/// What reads of [`masked_embeddings`] show for `ids`: only ids 0, 2 and 4
/// have a visible embedding.
fn visible_embeddings(ids: &[i32]) -> Vec<(i32, Option<Vec<f32>>)> {
    ids.iter()
        .map(|id| {
            (
                *id,
                [0, 2, 4].contains(id).then(|| embedding_of(*id as f32)),
            )
        })
        .collect()
}

/// `body` of every live row of `dataset`, by row address.
async fn bodies_by_addr(dataset: &Dataset) -> BTreeMap<u64, String> {
    let batch = dataset
        .scan()
        .project(&["body"])
        .unwrap()
        .with_row_address()
        .try_into_batch()
        .await
        .unwrap();
    batch[ROW_ADDR]
        .as_primitive::<UInt64Type>()
        .values()
        .iter()
        .copied()
        .zip(
            batch["body"]
                .as_string::<i32>()
                .iter()
                .map(|body| body.unwrap().to_string()),
        )
        .collect()
}

/// The live rows whose embedding flag is false, in scan order: a refresh's
/// work.
async fn pending_embeddings(dataset: &Dataset) -> Vec<u64> {
    let flag_id = dataset.cell_flag("embedding", "ready").unwrap().flag_id;
    let true_rows = dataset.cell_flag_true_rows(flag_id).unwrap();
    bodies_by_addr(dataset)
        .await
        .into_keys()
        .filter(|addr| !true_rows.contains(*addr))
        .collect()
}

/// A computed batch assigning the embeddings of the bodies `read` shows on
/// `addrs`, in scan order, except on `unassigned`, which carry [`QUERY`]: a
/// value the stager must never publish.
async fn computed_embeddings(read: &Dataset, addrs: &[u64], unassigned: &[u64]) -> ComputedBatch {
    let bodies = bodies_by_addr(read).await;
    let values: Vec<Option<Vec<f32>>> = addrs
        .iter()
        .map(|addr| {
            if unassigned.contains(addr) {
                Some(QUERY.to_vec())
            } else {
                embed(&bodies[addr])
            }
        })
        .collect();
    let rows = RecordBatch::try_from_iter([
        (
            ROW_ADDR,
            Arc::new(UInt64Array::from(addrs.to_vec())) as ArrayRef,
        ),
        ("embedding", embeddings(&values)),
    ])
    .unwrap();
    let assigned =
        BooleanBuffer::collect_bool(addrs.len(), |row| !unassigned.contains(&addrs[row]));
    ComputedBatch::new(rows).with_assigned("embedding", assigned)
}

/// Stage `computed` against `read` and commit it under `policy`.
async fn publish_embeddings(
    read: &Dataset,
    computed: ComputedBatch,
    policy: DependencyConflictPolicy,
) -> lance_core::Result<PublicationResult> {
    let read = Arc::new(read.clone());
    let staged = PublicationStager::try_new(read.clone(), &["embedding"])?
        .stage(stream::iter([Ok(computed)]))
        .await?
        .expect("the batch assigns a row");
    CommitBuilder::new(read)
        .with_dependency_conflict_policy(policy)
        .execute_with_report(staged)
        .await
}

/// Rewrite the body of `id` in place, with a partial-schema merge_insert.
async fn merge_insert_body(dataset: &Dataset, id: i32, body: &str) -> Dataset {
    let source = RecordBatch::try_from_iter([
        ("id", Arc::new(Int32Array::from(vec![id])) as ArrayRef),
        ("body", Arc::new(StringArray::from(vec![body])) as ArrayRef),
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

/// Rewrite the body of `id` with an update, which moves the row to a new
/// fragment.
async fn update_body(dataset: &Dataset, id: i32, body: &str) -> Dataset {
    UpdateBuilder::new(Arc::new(dataset.clone()))
        .update_where(&format!("id = {id}"))
        .unwrap()
        .set("body", &format!("'{body}'"))
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

/// How a nearest-neighbor search of `embedding` is configured.
#[derive(Debug, Clone, Copy)]
enum Knn {
    Plain,
    Prefilter(&'static str),
    Postfilter(&'static str),
    /// `use_index(false)` and `refine(1)`, which a flat search ignores.
    FlatRefined,
    DistanceRange(f32, f32),
    /// Only the fragment of this id, which needs a prefilter.
    Fragment(usize),
    FastSearch,
    /// A plain search over a scan whose batches are rechunked to at most this
    /// many bytes, which slices them.
    BatchBytes(u64),
}

/// The `(id, distance)` hits of each of `queries`, in result order, of a
/// search of `embedding` configured by `knn` that reads `batch_size` rows at a
/// time. More than one query makes a batch search. Checks the plan is a flat
/// search.
async fn nearest_hits(
    dataset: &Dataset,
    queries: &[[f32; 4]],
    k: usize,
    knn: Knn,
    batch_size: Option<usize>,
) -> Vec<Vec<(i32, f32)>> {
    let mut scan = dataset.scan();
    if let Some(batch_size) = batch_size {
        scan.batch_size(batch_size);
    }
    match knn {
        Knn::Prefilter(filter) => {
            scan.prefilter(true).filter(filter).unwrap();
        }
        Knn::Postfilter(filter) => {
            scan.filter(filter).unwrap();
        }
        Knn::Fragment(fragment_id) => {
            let fragment = dataset.get_fragment(fragment_id).unwrap();
            scan.prefilter(true)
                .with_fragments(vec![fragment.metadata().clone()]);
        }
        Knn::BatchBytes(bytes) => {
            scan.batch_size_bytes(bytes);
        }
        _ => {}
    }
    let values = Float32Array::from_iter_values(queries.iter().flatten().copied());
    if queries.len() == 1 {
        scan.nearest("embedding", &values, k).unwrap();
    } else {
        let batch = FixedSizeListArray::try_new_from_values(values, DIM).unwrap();
        scan.nearest("embedding", &batch, k).unwrap();
    }
    match knn {
        Knn::FlatRefined => {
            scan.use_index(false).refine(1);
        }
        Knn::DistanceRange(lower, upper) => {
            scan.distance_range(Some(lower), Some(upper));
        }
        Knn::FastSearch => {
            scan.fast_search();
        }
        _ => {}
    }
    scan.project(&["id"]).unwrap();
    let plan = scan.explain_plan(false).await.unwrap();
    if matches!(knn, Knn::FastSearch) {
        assert!(plan.contains("EmptyExec"), "{plan}");
    } else {
        assert!(plan.contains("KNNVectorDistance"), "{plan}");
        assert!(!plan.contains("ANN"), "{plan}");
    }
    let batch = scan.try_into_batch().await.unwrap();
    let mut hits = vec![Vec::new(); queries.len()];
    if batch.num_rows() == 0 {
        return hits;
    }
    let ids = batch["id"].as_primitive::<Int32Type>();
    let distances = batch[DIST_COL].as_primitive::<Float32Type>();
    for row in 0..batch.num_rows() {
        let query = if queries.len() == 1 {
            0
        } else {
            batch[QUERY_INDEX_COL]
                .as_primitive::<Int32Type>()
                .value(row) as usize
        };
        hits[query].push((ids.value(row), distances.value(row)));
    }
    hits
}

/// The stager publishes vectors: an unassigned row gets the snapshot's
/// masked NULL, not the value its batch carries, and a computed batch must
/// match the field's item type and dimension, but not its item's name.
#[tokio::test]
async fn test_embedding_publication_through_the_stager() {
    let (mut unmasked, flag_id) = masked_embeddings(false).await;
    assert_eq!(flagged_ids(&unmasked, flag_id).await, [0, 1, 2, 4]);

    // Dropping the flag shows what the masks hide: stale e5 and the Q that
    // fragment 2 was written with.
    unmasked.drop_cell_flag("embedding", "ready").await.unwrap();
    let query = Some(QUERY.to_vec());
    let mut stored = visible_embeddings(&[0, 1, 2, 4]);
    stored.extend([
        (5, Some(embedding_of(5.0))),
        (7, None),
        (8, query.clone()),
        (10, query.clone()),
        (11, query),
    ]);
    assert_eq!(scan_id_embeddings(&unmasked, None).await, stored);

    let (dataset, _) = masked_embeddings(false).await;
    // Id 7's embedding, computed as `embedding`.
    let computed = |embedding: FixedSizeListArray| {
        let rows = RecordBatch::try_from_iter([
            (
                ROW_ADDR,
                Arc::new(UInt64Array::from(vec![addr(1, 3)])) as ArrayRef,
            ),
            ("embedding", Arc::new(embedding) as ArrayRef),
        ])
        .unwrap();
        ComputedBatch::new(rows)
    };
    let stager = PublicationStager::try_new(Arc::new(dataset.clone()), &["embedding"]).unwrap();
    for (item, dimension) in [(DataType::Float32, 3), (DataType::Float64, DIM)] {
        let field = Arc::new(ArrowField::new("item", item.clone(), true));
        let values = arrow_array::new_null_array(&item, dimension as usize);
        let embedding = FixedSizeListArray::new(field, dimension, values, None);
        let error = stager
            .stage(stream::iter([Ok(computed(embedding))]))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(
            error.to_string().contains(
                "computed values for output 'embedding' do not match its field at version"
            ),
            "{item} x {dimension}: {error}"
        );
    }
    let element = Arc::new(ArrowField::new("element", DataType::Float32, true));
    let values = Arc::new(Float32Array::from(embedding_of(7.0)));
    let embedding = FixedSizeListArray::new(element, DIM, values, None);
    let result = publish_embeddings(
        &dataset,
        computed(embedding),
        DependencyConflictPolicy::Reject,
    )
    .await
    .unwrap();
    let mut published = visible_embeddings(&LIVE_IDS);
    published[5] = (7, Some(embedding_of(7.0)));
    assert_eq!(scan_id_embeddings(&result.dataset, None).await, published);
}

/// Float16 and Float64 vectors mask as Float32 ones do. Fragments 0 and 1
/// are published and id 5's body is then rewritten, leaving e5 stale in a
/// partly masked fragment; fragment 2 is never published, so every row of it
/// is masked. A leak of either adds a row to a search that asks for all 12.
#[rstest]
#[case::float16(DataType::Float16)]
#[case::float64(DataType::Float64)]
#[tokio::test]
async fn test_vectors_of_every_float_width_mask(#[case] item: DataType) {
    let vector_type = DataType::FixedSizeList(Arc::new(ArrowField::new("item", item, true)), DIM);
    let vectors = |ids: &[i32]| {
        let values: Vec<_> = ids
            .iter()
            .map(|id| Some(embedding_of(*id as f32)))
            .collect();
        arrow_cast::cast(&embeddings(&values), &vector_type).unwrap()
    };
    let ids: Vec<i32> = (0..12).collect();
    let bodies = StringArray::from_iter_values(ids.iter().map(|id| format!("b{id}")));
    let batch = RecordBatch::try_from_iter_with_nullable([
        (
            "id",
            Arc::new(Int32Array::from(ids.clone())) as ArrayRef,
            false,
        ),
        ("body", Arc::new(bodies) as ArrayRef, false),
        ("embedding", vectors(&ids), true),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            max_rows_per_file: 4,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    dataset
        .register_cell_flag(
            "embedding",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    let addrs: Vec<u64> = (0..8).map(|row| addr(row / 4, row % 4)).collect();
    let rows = RecordBatch::try_from_iter([
        (ROW_ADDR, Arc::new(UInt64Array::from(addrs)) as ArrayRef),
        ("embedding", vectors(&ids[..8])),
    ])
    .unwrap();
    let published = publish_embeddings(
        &dataset,
        ComputedBatch::new(rows),
        DependencyConflictPolicy::Reject,
    )
    .await
    .unwrap()
    .dataset;
    let dataset = merge_insert_body(&published, 5, "b50").await;

    let batch = dataset
        .scan()
        .project(&["id", "embedding"])
        .unwrap()
        .try_into_batch()
        .await
        .unwrap();
    let float32 = DataType::FixedSizeList(
        Arc::new(ArrowField::new("item", DataType::Float32, true)),
        DIM,
    );
    let as_float32 = RecordBatch::try_from_iter([
        ("id", batch["id"].clone()),
        (
            "embedding",
            arrow_cast::cast(&batch["embedding"], &float32).unwrap(),
        ),
    ])
    .unwrap();
    let visible: Vec<_> = ids
        .iter()
        .map(|id| (*id, (*id < 8 && *id != 5).then(|| embedding_of(*id as f32))))
        .collect();
    assert_eq!(id_embeddings(&as_float32), visible);
    assert_eq!(
        nearest_hits(&dataset, &[QUERY], 12, Knn::Plain, None).await,
        [[
            (6, 0.5625),
            (4, 1.5625),
            (7, 3.0625),
            (3, 5.0625),
            (2, 10.5625),
            (1, 18.0625),
            (0, 27.5625),
        ]]
    );
}

/// Writing a row's body masks its embedding in the version of the write,
/// whether the write rewrites the row in place or moves it.
#[rstest]
#[tokio::test]
async fn test_input_write_masks_an_embedding_in_its_own_version(
    #[values(true, false)] is_in_place: bool,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (dataset, _) = masked_embeddings(stable_row_ids).await;
    assert_eq!(
        nearest_hits(&dataset, &[QUERY], 1, Knn::Plain, None).await,
        [[NEAREST[0]]]
    );
    let previous = dataset.version().version;
    let written = if is_in_place {
        merge_insert_body(&dataset, 4, "b40").await
    } else {
        update_body(&dataset, 4, "b40").await
    };
    assert_eq!(written.version().version, previous + 1);

    let mut masked = visible_embeddings(&LIVE_IDS);
    masked[3] = (4, None);
    assert_eq!(scan_id_embeddings(&written, None).await, masked);
    assert_eq!(
        nearest_hits(&written, &[QUERY], 12, Knn::Plain, None).await,
        [&NEAREST[1..]]
    );
    assert_eq!(
        written
            .count_rows(Some("embedding IS NOT NULL".to_string()))
            .await
            .unwrap(),
        2
    );
    let before = written.checkout_version(previous).await.unwrap();
    assert_eq!(
        scan_id_embeddings(&before, None).await,
        visible_embeddings(&LIVE_IDS)
    );

    // Only the mask hides e4: the write kept it stored.
    let mut unmasked = written.clone();
    unmasked.drop_cell_flag("embedding", "ready").await.unwrap();
    let stored = scan_id_embeddings(&unmasked, None).await;
    assert_eq!(stored[3], (4, Some(embedding_of(4.0))));
}

/// Scans, filters, takes and late materialization read masked and pending
/// embeddings as NULL, also after a cold reopen and one row per batch.
#[rstest]
#[tokio::test]
async fn test_masked_embeddings_on_scan_filter_and_take(
    #[values(false, true)] stable_row_ids: bool,
) {
    let test_uri = TempStrDir::default();
    let (mut dataset, _) = masked_embeddings_at(&test_uri, stable_row_ids).await;
    let reopened = DatasetBuilder::from_uri(&test_uri)
        .with_session(Arc::new(Session::default()))
        .load()
        .await
        .unwrap();
    // Stale e5 stays stored under id 5's masked slot, in the partly true
    // fragment 1.
    let masked = [5, 7, 8, 10, 11];
    for read in [&dataset, &reopened] {
        for batch_size in [None, Some(1), Some(3)] {
            let batch = scan_embeddings(read, batch_size).await;
            assert_eq!(
                id_embeddings(&batch),
                visible_embeddings(&LIVE_IDS),
                "batch size {batch_size:?}"
            );
            assert_eq!(
                embedding_children(&batch, &masked),
                masked_children(&masked),
                "batch size {batch_size:?}"
            );
        }
    }
    assert_eq!(
        nearest_hits(&reopened, &[QUERY], 12, Knn::Plain, None).await,
        [NEAREST]
    );

    let projection = dataset.schema().project(&["id", "embedding"]).unwrap();
    // Logical offsets 1, 3, 4 and 6 hold ids 1, 4, 5 and 8.
    let taken = dataset
        .take(&[6, 1, 4, 3], projection.clone())
        .await
        .unwrap();
    assert_eq!(id_embeddings(&taken), visible_embeddings(&[1, 4, 5, 8]));
    assert_eq!(
        embedding_children(&taken, &[5, 8]),
        masked_children(&[5, 8])
    );
    let mut scan = dataset.scan();
    scan.with_row_id().project(&["id"]).unwrap();
    let batch = scan.try_into_batch().await.unwrap();
    let row_ids: HashMap<i32, u64> = batch["id"]
        .as_primitive::<Int32Type>()
        .values()
        .iter()
        .copied()
        .zip(
            batch[ROW_ID]
                .as_primitive::<UInt64Type>()
                .values()
                .iter()
                .copied(),
        )
        .collect();
    let requested: Vec<u64> = [5, 4, 8, 1].iter().map(|id| row_ids[id]).collect();
    let taken = dataset.take_rows(&requested, projection).await.unwrap();
    assert_eq!(id_embeddings(&taken), visible_embeddings(&[1, 4, 5, 8]));
    assert_eq!(
        embedding_children(&taken, &[5, 8]),
        masked_children(&[5, 8])
    );

    let mut scan = dataset.scan();
    scan.project(&["id", "embedding"])
        .unwrap()
        .filter("id >= 4")
        .unwrap()
        .materialization_style(MaterializationStyle::AllLate);
    let plan = scan.explain_plan(false).await.unwrap();
    assert!(
        plan.contains("projection=[embedding], source=stream(_rowid)"),
        "{plan}"
    );
    let batch = scan.try_into_batch().await.unwrap();
    assert_eq!(
        id_embeddings(&batch),
        visible_embeddings(&[4, 5, 7, 8, 10, 11])
    );

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
    for (filter, ids) in [
        ("embedding IS NULL", &[1, 5, 7, 8, 10, 11][..]),
        ("embedding IS NOT NULL", &[0, 2, 4]),
        ("id >= 4 AND embedding IS NULL", &[5, 7, 8, 10, 11]),
    ] {
        for use_scalar_index in [false, true] {
            let mut scan = dataset.scan();
            scan.project(&["id", "embedding"])
                .unwrap()
                .filter(filter)
                .unwrap()
                .use_scalar_index(use_scalar_index);
            let batch = scan.try_into_batch().await.unwrap();
            assert_eq!(
                id_embeddings(&batch),
                visible_embeddings(ids),
                "{filter} with use_scalar_index={use_scalar_index}"
            );
        }
        let sql = format!("SELECT id, embedding FROM dataset WHERE {filter}");
        for batch in [
            sql_batch(&dataset, &sql).await,
            provider_batch(&dataset, &sql).await,
        ] {
            assert_eq!(id_embeddings(&batch), visible_embeddings(ids), "{sql}");
        }
        assert_eq!(
            dataset.count_rows(Some(filter.to_string())).await.unwrap(),
            ids.len(),
            "{filter}"
        );
    }
}

/// A flat search skips masked, pending, deleted and computed-NULL embeddings
/// on every batch size: it returns exactly the visible rows, in distance
/// order, however it is filtered or restricted, and fewer than `k` when fewer
/// are visible. A search that needs an index finds nothing.
#[rstest]
#[case::plain(Knn::Plain, &[QUERY], 12, vec![NEAREST.to_vec()])]
#[case::nearest_two(Knn::Plain, &[QUERY], 2, vec![NEAREST[..2].to_vec()])]
#[case::indexed_prefilter(Knn::Prefilter("id >= 4"), &[QUERY], 1, vec![vec![NEAREST[0]]])]
#[case::prefilter(Knn::Prefilter("id < 4"), &[QUERY], 1, vec![vec![NEAREST[1]]])]
#[case::postfilter(Knn::Postfilter("id < 4"), &[QUERY], 3, vec![NEAREST[1..].to_vec()])]
#[case::flat_refined(Knn::FlatRefined, &[QUERY], 12, vec![NEAREST.to_vec()])]
#[case::distance_range(Knn::DistanceRange(0.0, 2.0), &[QUERY], 12, vec![vec![NEAREST[0]]])]
#[case::partial_fragment(Knn::Fragment(1), &[QUERY], 12, vec![vec![NEAREST[0]]])]
#[case::unpublished_fragment(Knn::Fragment(2), &[QUERY], 12, vec![vec![]])]
#[case::fast_search(Knn::FastSearch, &[QUERY], 12, vec![vec![]])]
#[case::batch(
    Knn::Plain,
    &[QUERY, [0.0; 4]],
    12,
    vec![NEAREST.to_vec(), vec![(0, 1.0), (2, 5.0), (4, 17.0)]]
)]
#[tokio::test]
async fn test_unindexed_nearest_skips_masked_embeddings(
    #[case] knn: Knn,
    #[case] queries: &[[f32; 4]],
    #[case] k: usize,
    #[case] expected: Vec<Vec<(i32, f32)>>,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (mut dataset, _) = masked_embeddings(stable_row_ids).await;
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
    for batch_size in [None, Some(1), Some(3)] {
        assert_eq!(
            nearest_hits(&dataset, queries, k, knn, batch_size).await,
            expected,
            "batch size {batch_size:?}"
        );
    }
}

/// A byte budget slices the scan's batches, so the search gets slices of a
/// partly masked fragment whose validity starts mid-buffer, and without a
/// deletion no row-id nulls are merged into a fresh buffer. Every row of the
/// one fragment is published and then id 5's body is rewritten, leaving e5,
/// the nearest to [`QUERY`], stale under a false flag. The search must skip
/// only e5.
#[rstest]
#[tokio::test]
async fn test_nearest_over_sliced_batches_skips_masked_embeddings(
    #[values(false, true)] stable_row_ids: bool,
) {
    let batch = RecordBatch::try_from_iter_with_nullable([
        (
            "id",
            Arc::new(Int32Array::from_iter_values(0..8)) as ArrayRef,
            false,
        ),
        (
            "body",
            Arc::new(StringArray::from_iter_values(
                (0..8).map(|id| format!("b{id}")),
            )) as ArrayRef,
            false,
        ),
        (
            "embedding",
            embeddings(&vec![Some(QUERY.to_vec()); 8]),
            true,
        ),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        "memory://",
        Some(WriteParams {
            enable_stable_row_ids: stable_row_ids,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    dataset
        .register_cell_flag(
            "embedding",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    let addrs: Vec<u64> = (0..8).map(|row| addr(0, row)).collect();
    let computed = computed_embeddings(&dataset, &addrs, &[]).await;
    let published = publish_embeddings(&dataset, computed, DependencyConflictPolicy::Reject)
        .await
        .unwrap()
        .dataset;
    let dataset = merge_insert_body(&published, 5, "b50").await;
    assert!(
        dataset.get_fragments()[0]
            .metadata()
            .deletion_file
            .is_none()
    );

    let nearest = vec![
        (6, 0.5625),
        (4, 1.5625),
        (7, 3.0625),
        (3, 5.0625),
        (2, 10.5625),
        (1, 18.0625),
        (0, 27.5625),
    ];
    let nearest_to_origin = vec![
        (0, 1.0),
        (1, 2.0),
        (2, 5.0),
        (3, 10.0),
        (4, 17.0),
        (6, 37.0),
        (7, 50.0),
    ];
    for bytes in (128..=640).step_by(64) {
        let knn = Knn::BatchBytes(bytes);
        assert_eq!(
            nearest_hits(&dataset, &[QUERY], 8, knn, None).await,
            [nearest.clone()],
            "{bytes} bytes"
        );
        assert_eq!(
            nearest_hits(&dataset, &[QUERY, [0.0; 4]], 8, knn, None).await,
            [nearest.clone(), nearest_to_origin.clone()],
            "{bytes} bytes"
        );
    }
}

/// A refresh staged before its inputs change cannot make a stale embedding
/// visible, and a refresh of the rows the follow-up plans and a scan of
/// pending rows finds restores every embedding from the current bodies. Id
/// 5's body is rewritten in place, and id 8's by a row-moving update.
#[rstest]
#[tokio::test]
async fn test_stale_embedding_publication_stays_masked_until_refreshed(
    #[values(DependencyConflictPolicy::Reject, DependencyConflictPolicy::Skip)]
    policy: DependencyConflictPolicy,
    #[values(false, true)] stable_row_ids: bool,
) {
    let (read, flag_id) = masked_embeddings(stable_row_ids).await;
    let pending = pending_embeddings(&read).await;
    assert_eq!(
        pending,
        [addr(1, 1), addr(1, 3), addr(2, 0), addr(2, 2), addr(2, 3)]
    );
    let computed = computed_embeddings(&read, &pending, &[]).await;
    let edited = merge_insert_body(&read, 5, "b500").await;
    let written = update_body(&edited, 8, "b80").await;

    let result = publish_embeddings(&read, computed, policy).await;
    let (head, report) = if policy == DependencyConflictPolicy::Reject {
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::RetryableCommitConflict { .. }),
            "{error}"
        );
        let head = latest(&written).await;
        assert_eq!(head.version().version, written.version().version);
        (head, None)
    } else {
        let PublicationResult { dataset, report } = result.unwrap();
        assert_eq!(
            report.published_rows(flag_id),
            RowAddrTreeMap::from_iter([addr(1, 3), addr(2, 2), addr(2, 3)])
        );
        assert_eq!(
            report.deferred_rows_of(flag_id, DeferralReason::InputChanged),
            RowAddrTreeMap::from_iter([addr(1, 1)])
        );
        assert_eq!(
            report.deferred_rows_of(flag_id, DeferralReason::RowVacated),
            RowAddrTreeMap::from_iter([addr(2, 0)])
        );
        (dataset, Some(report))
    };

    // Id 5's staged e50 is installed under a false flag by `Skip`; it would be
    // the nearest to e50 if it leaked.
    let published: &[i32] = if report.is_some() {
        &[0, 2, 4, 7, 10, 11]
    } else {
        &[0, 2, 4]
    };
    let visible: Vec<(i32, Option<Vec<f32>>)> = LIVE_IDS
        .iter()
        .map(|id| {
            (
                *id,
                published.contains(id).then(|| embedding_of(*id as f32)),
            )
        })
        .collect();
    let batch = scan_embeddings(&head, None).await;
    assert_eq!(id_embeddings(&batch), visible);
    // Id 5 is masked in the partly true fragment 1, over stale e5 or e50.
    assert_eq!(embedding_children(&batch, &[5]), masked_children(&[5]));
    let (nearest, nearest_to_e50) = if report.is_some() {
        (
            vec![
                (4, 1.5625),
                (7, 3.0625),
                (2, 10.5625),
                (10, 22.5625),
                (0, 27.5625),
                (11, 33.0625),
            ],
            (11, 1521.0),
        )
    } else {
        (NEAREST.to_vec(), (4, 2116.0))
    };
    assert_eq!(
        nearest_hits(&head, &[QUERY], 12, Knn::Plain, None).await,
        [nearest]
    );
    assert_eq!(
        nearest_hits(&head, &[[50.0, 0.0, 0.0, 1.0]], 1, Knn::Plain, None).await,
        [[nearest_to_e50]]
    );

    let mut scan = head.scan();
    scan.with_row_address()
        .project(&["id"])
        .unwrap()
        .filter("id = 8")
        .unwrap();
    let moved = scan.try_into_batch().await.unwrap()[ROW_ADDR]
        .as_primitive::<UInt64Type>()
        .value(0);
    assert!(RowAddress::from(moved).fragment_id() > 2);
    let mut refresh = vec![addr(1, 1)];
    if let Some(report) = &report {
        let plan = PublicationStager::try_new(Arc::new(head.clone()), &["embedding"])
            .unwrap()
            .follow_up(report)
            .await
            .unwrap();
        assert_eq!(
            plan.rows("embedding"),
            Some(&FollowUpRows {
                reuse: RowAddrTreeMap::new(),
                recompute: RowAddrTreeMap::from_iter([addr(1, 1)]),
            })
        );
    } else {
        refresh.extend([addr(1, 3), addr(2, 2), addr(2, 3)]);
    }
    refresh.push(moved);
    assert_eq!(pending_embeddings(&head).await, refresh);
    let computed = computed_embeddings(&head, &refresh, &[]).await;
    let completed = publish_embeddings(&head, computed, DependencyConflictPolicy::Reject)
        .await
        .unwrap()
        .dataset;

    assert!(pending_embeddings(&completed).await.is_empty());
    assert_eq!(flagged_ids(&completed, flag_id).await, LIVE_IDS);
    let batch = completed
        .scan()
        .project(&["id", "body", "embedding"])
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
    let computed: Vec<_> = LIVE_IDS
        .iter()
        .map(|id| (*id, embed(&bodies[id])))
        .collect();
    assert_eq!(id_embeddings(&batch), computed);
    assert_eq!(
        nearest_hits(&completed, &[[500.0, 0.0, 0.0, 1.0]], 1, Knn::Plain, None).await,
        [[(5, 0.0)]]
    );
    assert_eq!(
        nearest_hits(&completed, &[QUERY], 12, Knn::Plain, None).await,
        [[
            (4, 1.5625),
            (7, 3.0625),
            (2, 10.5625),
            (10, 22.5625),
            (0, 27.5625),
            (11, 33.0625),
            (8, 5587.5625),
            (5, 244_777.56),
        ]]
    );
}
