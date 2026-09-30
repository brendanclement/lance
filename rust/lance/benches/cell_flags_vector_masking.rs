// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Read cost of masking an unindexed embedding with a dependent cell flag,
//! against tables without flags that read the same visible data.
//!
//! Create mode writes eight tables under one root directory. Every table has
//! the same logical rows (`id`, `body`, and `embedding`, a
//! `FixedSizeList<Float32, 128>` computed from `id`) and is built the same
//! way: `id` and `body` are written, `embedding` is added as an all-NULL
//! column and then stored for every fragment by one `DataReplacement`, and the
//! table's cleared rows get an in-place `body` write (`merge_insert`). On a
//! flagged table, `embedding.ready` watches `body` and masks `embedding`, the
//! replacement publishes every row, and the body write clears the flag. On a
//! plain table, the cleared rows are stored as NULL vectors instead.
//!
//! | Flagged | Plain, same visible data | Cleared rows |
//! |---|---|---|
//! | `ready` | `plain` | none |
//! | `partial_1pct` | `null_1pct` | 1%, scattered |
//! | `partial_50pct` | `null_50pct` | 50%, scattered |
//! | `masked` | `null_all` | all |
//!
//! Create checks that each pair reads the same visible data. Measure runs, for
//! every table, a full scan of `embedding`, an unfiltered flat nearest-neighbor
//! search (k = 10) and a take of 1,000 scattered rows, named `<read>_<table>`
//! (`scan_partial_1pct`). Each sample runs every selected workload once, the
//! order rotated by one position per sample.
//!
//! ```bash
//! BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/root BENCH_SCALE_ROWS=1000000 <binary>
//! BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=/abs/root BENCH_READ_SAMPLES=20 \
//!   BENCH_BUILD=zip BENCH_OUT=/abs/samples.jsonl <binary>
//! ```
//!
//! Records follow `cell_flags_scan_counters` (schema 2), and
//! `prototypes/dependent-cell-flags/bench/run_counters_rotation.py --bench
//! cell_flags_vector_masking` drives it. The run record's dataset identity
//! lists every table and hashes their manifests into one `manifest_blake3`.
//! Samples also carry what a counting global allocator saw: `allocations`
//! (calls to `alloc`, `alloc_zeroed` and `realloc`), `allocated_bytes` (the
//! sizes they requested, a `realloc` counting its new size) and
//! `peak_live_growth_bytes` (the most live bytes above those live at the
//! sample's start). They count every thread of the process.
//!
//! Uses the prototype's cell flag API, so it does not build on `main`. Sets
//! `LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1` itself, since release builds refuse
//! flagged datasets without it.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/mod.rs"]
mod common;
#[path = "cell_flags_common/counters.rs"]
mod counters;

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use arrow_array::cast::AsArray;
use arrow_array::types::{Float32Type, Int64Type};
use arrow_array::{
    Array, ArrayRef, FixedSizeListArray, Float32Array, Int64Array, RecordBatch,
    RecordBatchIterator, StringArray,
};
use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use futures::{StreamExt, TryStreamExt};
use lance::dataset::builder::DatasetBuilder;
use lance::dataset::cell_flag::CellFlagOptions;
use lance::dataset::scanner::Scanner;
use lance::dataset::transaction::{CellFlagChanges, CellFlagUpdate, Operation, TransactionBuilder};
use lance::dataset::{
    CommitBuilder, Dataset, NewColumnTransform, ProjectionRequest, WriteMode, WriteParams,
};
use lance::session::Session;
use lance_select::{RowAddrSelection, RowAddrTreeMap};
use lance_table::feature_flags::ENABLE_UNSTABLE_CELL_FLAGS_ENV;
use roaring::RoaringTreemap;
use serde_json::{Map, Value, json};

use common::{BenchConfig, dir_bytes, merge_in_place, scattered_ids, splitmix64};
use counters::{
    CPU_NS_SOURCE, Counters, HARDWARE_SOURCE, dataset_identity, delta, hardware_counters_advance,
    required_env,
};

/// Version of the record layout; readers reject records of other versions.
const SCHEMA: u32 = 2;

const DIM: i32 = 128;
const FLAG: &str = "ready";
const NEAREST_K: usize = 10;
const TAKE_ROWS: u64 = 1000;
const GENERATE_BATCH_ROWS: u64 = 8192;
const SEED_EMBEDDING: u64 = 0xe3be_dd16_0000_0080;
const SEED_ONE_PERCENT: u64 = 21;
const SEED_HALF: u64 = 22;
/// `cell_flags_scan_counters`'s take seed.
const SEED_TAKE: u64 = 14;
/// Not a row id, so no row embeds exactly as the query.
const QUERY_ID: u64 = u64::MAX;

/// Rows a table's input write clears: the flag goes false on a flagged
/// table, and a plain table stores them as NULL vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cleared {
    None,
    OnePercent,
    Half,
    All,
}

struct Table {
    name: &'static str,
    is_flagged: bool,
    cleared: Cleared,
}

const TABLES: [Table; 8] = [
    Table {
        name: "plain",
        is_flagged: false,
        cleared: Cleared::None,
    },
    Table {
        name: "ready",
        is_flagged: true,
        cleared: Cleared::None,
    },
    Table {
        name: "partial_1pct",
        is_flagged: true,
        cleared: Cleared::OnePercent,
    },
    Table {
        name: "partial_50pct",
        is_flagged: true,
        cleared: Cleared::Half,
    },
    Table {
        name: "masked",
        is_flagged: true,
        cleared: Cleared::All,
    },
    Table {
        name: "null_1pct",
        is_flagged: false,
        cleared: Cleared::OnePercent,
    },
    Table {
        name: "null_50pct",
        is_flagged: false,
        cleared: Cleared::Half,
    },
    Table {
        name: "null_all",
        is_flagged: false,
        cleared: Cleared::All,
    },
];

#[derive(Debug, Clone, Copy)]
enum Read {
    Scan,
    Nearest,
    Take,
}

impl Read {
    const ALL: [Self; 3] = [Self::Scan, Self::Nearest, Self::Take];

    fn name(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Nearest => "nearest",
            Self::Take => "take",
        }
    }
}

// ---------------------------------------------------------------------------
// Allocation counting
// ---------------------------------------------------------------------------

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

/// The system allocator, counting what passes through it.
struct CountingAllocator;

impl CountingAllocator {
    fn allocated(size: usize) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
        let live = LIVE_BYTES.fetch_add(size, Ordering::Relaxed) + size;
        PEAK_LIVE_BYTES.fetch_max(live, Ordering::Relaxed);
    }
}

// SAFETY: every method forwards its arguments unchanged to `System` and
// returns its result; the counters only record sizes.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            Self::allocated(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            Self::allocated(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = System.realloc(ptr, layout, new_size);
        if !moved.is_null() {
            LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
            Self::allocated(new_size);
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[derive(Clone, Copy)]
struct Allocations {
    count: u64,
    bytes: u64,
    live_bytes: usize,
}

impl Allocations {
    /// Start a sample: the peak restarts from the bytes live now.
    fn start() -> Self {
        let live_bytes = LIVE_BYTES.load(Ordering::Relaxed);
        PEAK_LIVE_BYTES.store(live_bytes, Ordering::Relaxed);
        Self {
            count: ALLOCATIONS.load(Ordering::Relaxed),
            bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
            live_bytes,
        }
    }

    /// `(allocations, allocated_bytes, peak_live_growth_bytes)` since `self`.
    fn since(self) -> (u64, u64, u64) {
        let peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
        (
            ALLOCATIONS.load(Ordering::Relaxed) - self.count,
            ALLOCATED_BYTES.load(Ordering::Relaxed) - self.bytes,
            peak.saturating_sub(self.live_bytes) as u64,
        )
    }
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

fn item_field() -> Arc<Field> {
    Arc::new(Field::new("item", DataType::Float32, true))
}

fn embedding_schema() -> Arc<ArrowSchema> {
    Arc::new(ArrowSchema::new(vec![Field::new(
        "embedding",
        DataType::FixedSizeList(item_field(), DIM),
        true,
    )]))
}

fn base_schema() -> Arc<ArrowSchema> {
    Arc::new(ArrowSchema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("body", DataType::Utf8, false),
    ]))
}

fn base_batch(ids: std::ops::Range<u64>) -> RecordBatch {
    RecordBatch::try_new(
        base_schema(),
        vec![
            Arc::new(Int64Array::from_iter_values(
                ids.clone().map(|id| id as i64),
            )),
            Arc::new(StringArray::from_iter_values(
                ids.map(|id| format!("b{id}")),
            )),
        ],
    )
    .expect("base batch matches its schema")
}

/// Appends the embedding of `id`: values uniform in [-1, 1).
fn push_embedding(id: u64, values: &mut Vec<f32>) {
    let mut state = SEED_EMBEDDING ^ id.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for _ in 0..DIM / 2 {
        let bits = splitmix64(&mut state);
        for half in [bits >> 40, (bits >> 16) & 0xff_ffff] {
            values.push(half as f32 / (1 << 23) as f32 - 1.0);
        }
    }
}

/// The embeddings of `ids`, NULL on `stored_nulls` with zeros under the slot.
fn embedding_batch(ids: &Int64Array, stored_nulls: Option<&RoaringTreemap>) -> RecordBatch {
    let mut values = Vec::with_capacity(ids.len() * DIM as usize);
    for id in ids.values() {
        let id = *id as u64;
        if stored_nulls.is_some_and(|nulls| nulls.contains(id)) {
            values.resize(values.len() + DIM as usize, 0.0);
        } else {
            push_embedding(id, &mut values);
        }
    }
    let nulls = stored_nulls.map(|nulls| {
        NullBuffer::from_iter(ids.values().iter().map(|id| !nulls.contains(*id as u64)))
    });
    let embedding = FixedSizeListArray::new(
        item_field(),
        DIM,
        Arc::new(Float32Array::from(values)),
        nulls,
    );
    RecordBatch::try_new(embedding_schema(), vec![Arc::new(embedding)])
        .expect("embedding batch matches its schema")
}

fn query() -> Float32Array {
    let mut values = Vec::with_capacity(DIM as usize);
    push_embedding(QUERY_ID, &mut values);
    Float32Array::from(values)
}

fn cleared_rows(cleared: Cleared, rows: u64) -> RoaringTreemap {
    match cleared {
        Cleared::None => RoaringTreemap::new(),
        Cleared::OnePercent => scattered_ids(rows, rows / 100, SEED_ONE_PERCENT)
            .into_iter()
            .collect(),
        Cleared::Half => scattered_ids(rows, rows / 2, SEED_HALF)
            .into_iter()
            .collect(),
        Cleared::All => (0..rows).collect(),
    }
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

fn main() {
    if std::env::var_os(ENABLE_UNSTABLE_CELL_FLAGS_ENV).is_none() {
        // SAFETY: no other thread exists yet; the runtime starts below.
        unsafe { std::env::set_var(ENABLE_UNSTABLE_CELL_FLAGS_ENV, "1") };
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(Box::pin(run()));
}

async fn run() {
    let config = BenchConfig::from_env();
    let root = required_env("BENCH_COUNTERS_URI");
    match std::env::var("BENCH_COUNTERS_MODE").as_deref() {
        Ok("create") => Box::pin(create(&config, Path::new(&root))).await,
        Ok("measure") => measure(&config, Path::new(&root)).await,
        other => panic!("BENCH_COUNTERS_MODE must be create or measure, got {other:?}"),
    }
}

async fn create(config: &BenchConfig, root: &Path) {
    assert!(
        !root.exists(),
        "BENCH_COUNTERS_URI {} already exists; create writes new tables only",
        root.display()
    );
    std::fs::create_dir_all(root).expect("create the root directory");
    let mut tables = Vec::with_capacity(TABLES.len());
    for table in &TABLES {
        let start = Instant::now();
        let cleared = cleared_rows(table.cleared, config.scale_rows);
        let dataset = Box::pin(create_table(
            &root.join(table.name),
            table,
            config,
            &cleared,
        ))
        .await;
        println!(
            "created {} in {:.1} s: {} bytes",
            table.name,
            start.elapsed().as_secs_f64(),
            dir_bytes(&root.join(table.name))
        );
        tables.push(dataset);
    }
    check_equal_visible_data(config, &tables).await;
    println!("{}", tables_identity(root, &tables).await);
}

async fn create_table(
    uri: &Path,
    table: &Table,
    config: &BenchConfig,
    cleared: &RoaringTreemap,
) -> Dataset {
    let total = config.scale_rows;
    let batches = (0..total)
        .step_by(GENERATE_BATCH_ROWS as usize)
        .map(move |start| Ok(base_batch(start..(start + GENERATE_BATCH_ROWS).min(total))));
    let params = WriteParams {
        mode: WriteMode::Create,
        max_rows_per_file: config.rows_per_fragment as usize,
        auto_cleanup: None,
        skip_auto_cleanup: true,
        ..Default::default()
    };
    let mut dataset = Dataset::write(
        RecordBatchIterator::new(batches, base_schema()),
        uri.to_str().expect("utf-8 path"),
        Some(params),
    )
    .await
    .expect("write id and body");
    dataset
        .add_columns(NewColumnTransform::AllNulls(embedding_schema()), None, None)
        .await
        .expect("declare embedding");
    let flag_id = if table.is_flagged {
        let options = CellFlagOptions::default()
            .with_clear_on_write(["body"])
            .with_mask_when_false(true);
        let definition = dataset
            .register_cell_flag("embedding", FLAG, options)
            .await
            .expect("register the masking flag");
        Some(definition.flag_id)
    } else {
        None
    };

    let stored_nulls = (!table.is_flagged).then_some(cleared);
    let target = dataset
        .schema()
        .project(&["embedding"])
        .expect("embedding is declared");
    let mut replacements = Vec::new();
    let mut every_row = RowAddrTreeMap::new();
    for fragment in dataset.get_fragments() {
        let mut scanner = fragment.scan();
        scanner.project(&["id"]).expect("project id");
        scanner.scan_in_order(true);
        let embeddings = scanner
            .try_into_stream()
            .await
            .expect("scan ids")
            .map_ok(|batch| {
                let ids = batch["id"].as_primitive::<Int64Type>();
                embedding_batch(ids, stored_nulls)
            });
        replacements.push(
            fragment
                .write_columns(embeddings, &target)
                .await
                .expect("stage embeddings"),
        );
        every_row.insert_fragment(fragment.id() as u32);
    }
    let mut transaction = TransactionBuilder::new(
        dataset.version().version,
        Operation::DataReplacement { replacements },
    );
    if let Some(flag_id) = flag_id {
        transaction = transaction.cell_flag_changes(CellFlagChanges {
            updates: vec![CellFlagUpdate {
                flag_id,
                value: true,
                rows: every_row,
            }],
            ..Default::default()
        });
    }
    let mut dataset = CommitBuilder::new(Arc::new(dataset))
        .execute(transaction.build())
        .await
        .expect("store embeddings");

    if !cleared.is_empty() {
        let ids = Int64Array::from_iter_values(cleared.iter().map(|id| id as i64));
        let bodies = StringArray::from_iter_values(cleared.iter().map(|id| format!("c{id}")));
        let source = RecordBatch::try_new(base_schema(), vec![Arc::new(ids), Arc::new(bodies)])
            .expect("body source matches the base schema");
        dataset = merge_in_place(Arc::new(dataset), source).await.dataset;
    }
    if let Some(flag_id) = flag_id {
        assert_eq!(
            true_row_count(&dataset, flag_id),
            total - cleared.len(),
            "{}: rows whose flag is true",
            table.name
        );
    }
    dataset
}

fn true_row_count(dataset: &Dataset, flag_id: u32) -> u64 {
    let true_rows = dataset
        .cell_flag_true_rows(flag_id)
        .expect("flag is registered");
    dataset
        .get_fragments()
        .iter()
        .map(|fragment| match true_rows.get(&(fragment.id() as u32)) {
            Some(RowAddrSelection::Full) => fragment
                .metadata()
                .physical_rows
                .expect("physical row count") as u64,
            Some(RowAddrSelection::Partial(rows)) => rows.len(),
            None => 0,
        })
        .sum()
}

/// A digest of what a scan of `id` and `embedding` shows: each row's id, and
/// its vector where the slot is valid. The values under a NULL slot are left
/// out, since masking and a stored NULL leave different ones.
async fn visible_digest(dataset: &Dataset) -> (u64, String) {
    let mut scanner = dataset.scan();
    scanner.project(&["id", "embedding"]).expect("project");
    let mut stream = scanner.try_into_stream().await.expect("scan");
    let mut hasher = blake3::Hasher::new();
    let mut visible = 0u64;
    let width = DIM as usize * size_of::<f32>();
    while let Some(batch) = stream.try_next().await.expect("batch") {
        let ids = batch["id"].as_primitive::<Int64Type>();
        let embeddings = batch["embedding"].as_fixed_size_list();
        let values = embeddings.values().as_primitive::<Float32Type>();
        let bytes = values.values().inner().as_slice();
        for row in 0..batch.num_rows() {
            hasher.update(&ids.value(row).to_le_bytes());
            if embeddings.is_valid(row) {
                visible += 1;
                hasher.update(&[1]);
                hasher.update(&bytes[row * width..(row + 1) * width]);
            } else {
                hasher.update(&[0]);
            }
        }
    }
    (visible, hasher.finalize().to_hex().to_string())
}

/// `(id, distance bits)` of the nearest hits.
async fn nearest_hits(dataset: &Dataset, query: &Float32Array) -> Vec<(i64, u32)> {
    let batch = nearest_scanner(dataset, query)
        .try_into_batch()
        .await
        .expect("nearest");
    let ids = batch["id"].as_primitive::<Int64Type>();
    let distances = batch["_distance"].as_primitive::<Float32Type>();
    (0..batch.num_rows())
        .map(|row| (ids.value(row), distances.value(row).to_bits()))
        .collect()
}

/// Each flagged table must read what its plain pair reads.
async fn check_equal_visible_data(config: &BenchConfig, tables: &[Dataset]) {
    let query = query();
    for (flagged, table) in TABLES.iter().enumerate().filter(|(_, t)| t.is_flagged) {
        let plain = TABLES
            .iter()
            .position(|other| !other.is_flagged && other.cleared == table.cleared)
            .expect("every flagged table has a plain pair");
        let expected_visible =
            config.scale_rows - cleared_rows(table.cleared, config.scale_rows).len();
        let digest = visible_digest(&tables[flagged]).await;
        assert_eq!(digest.0, expected_visible, "{}: visible rows", table.name);
        assert_eq!(
            digest,
            visible_digest(&tables[plain]).await,
            "{} and {} read different data",
            table.name,
            TABLES[plain].name
        );
        let hits = nearest_hits(&tables[flagged], &query).await;
        assert_eq!(
            hits.len(),
            NEAREST_K.min(expected_visible as usize),
            "{}: nearest hits",
            table.name
        );
        assert_eq!(
            hits,
            nearest_hits(&tables[plain], &query).await,
            "{} and {} find different neighbors",
            table.name,
            TABLES[plain].name
        );
        println!(
            "{} reads the same {expected_visible} visible rows as {} (digest {})",
            table.name, TABLES[plain].name, digest.1
        );
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

/// Every table's identity, and one `manifest_blake3` over their manifests.
async fn tables_identity(root: &Path, tables: &[Dataset]) -> Value {
    let mut combined = blake3::Hasher::new();
    let mut identities = Map::new();
    for (table, dataset) in TABLES.iter().zip(tables) {
        let mut identity = dataset_identity(dataset).await;
        combined.update(table.name.as_bytes());
        combined.update(b"\0");
        combined.update(
            identity["manifest_blake3"]
                .as_str()
                .expect("manifest hash")
                .as_bytes(),
        );
        combined.update(b"\n");
        let flag = dataset.cell_flag("embedding", FLAG).map(|definition| {
            json!({
                "flag_id": definition.flag_id,
                "true_rows": true_row_count(dataset, definition.flag_id),
            })
        });
        let fields = identity.as_object_mut().expect("identity is an object");
        fields.insert(
            "data_storage_version".into(),
            json!(dataset.manifest().data_storage_format.version.to_string()),
        );
        fields.insert("masking_flag".into(), json!(flag));
        identities.insert(table.name.into(), identity);
    }
    json!({
        "uri": root,
        "manifest_blake3": combined.finalize().to_hex().to_string(),
        "tables": identities,
    })
}

fn nearest_scanner(dataset: &Dataset, query: &Float32Array) -> Scanner {
    let mut scanner = dataset.scan();
    scanner
        .nearest("embedding", query, NEAREST_K)
        .expect("nearest")
        .project(&["id"])
        .expect("project id");
    scanner
}

async fn measure(config: &BenchConfig, root: &Path) {
    let build = required_env("BENCH_BUILD");
    let out_path = required_env("BENCH_OUT");
    let mut tables = Vec::with_capacity(TABLES.len());
    for table in &TABLES {
        let uri = root.join(table.name);
        let dataset = DatasetBuilder::from_uri(uri.to_str().expect("utf-8 path"))
            .with_session(Arc::new(Session::default()))
            .load()
            .await
            .unwrap_or_else(|error| panic!("open table {}: {error}", uri.display()));
        tables.push(dataset);
    }
    let identity = tables_identity(root, &tables).await;
    if let Ok(expected) = std::env::var("BENCH_COUNTERS_EXPECT_MANIFEST_BLAKE3") {
        assert_eq!(
            identity["manifest_blake3"].as_str(),
            Some(expected.as_str()),
            "the tables under {} are not the expected ones",
            root.display()
        );
    }
    let query = query();
    let mut workloads = Vec::new();
    for (index, table) in TABLES.iter().enumerate() {
        let rows = tables[index].count_rows(None).await.expect("count rows") as u64;
        let take = scattered_ids(rows, TAKE_ROWS.min(rows), SEED_TAKE);
        for read in Read::ALL {
            let name = format!("{}_{}", read.name(), table.name);
            if !config.selects(&name) {
                continue;
            }
            if matches!(read, Read::Nearest) {
                let plan = nearest_scanner(&tables[index], &query)
                    .explain_plan(false)
                    .await
                    .expect("explain nearest");
                assert!(
                    plan.contains("KNNVectorDistance") && !plan.contains("ANN"),
                    "{name} is not a flat search: {plan}"
                );
            }
            workloads.push((name, index, read, take.clone()));
        }
    }
    assert!(!workloads.is_empty(), "BENCH_WORKLOADS selects no workload");
    let hardware = hardware_counters_advance();

    if let Some(parent) = Path::new(&out_path).parent() {
        std::fs::create_dir_all(parent).expect("create the BENCH_OUT directory");
    }
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&out_path)
        .expect("open BENCH_OUT");
    let run = json!({
        "record": "run",
        "schema": SCHEMA,
        "build": build,
        "git_sha": config.git_sha,
        "binary_path": std::env::current_exe().expect("path of this binary"),
        "binary_sha256": std::env::var("BENCH_BINARY_SHA256").ok(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "debug_assertions": cfg!(debug_assertions),
        "pid": std::process::id(),
        "dataset": identity,
        "warmup": config.warmup,
        "samples": config.read_samples,
        "workloads": workloads.iter().map(|(name, ..)| name).collect::<Vec<_>>(),
        "vector": {"item": "Float32", "dim": DIM, "nearest_k": NEAREST_K, "take_rows": TAKE_ROWS},
        "counter_sources": {
            "wall_ns": "std::time::Instant",
            "cpu_ns": CPU_NS_SOURCE,
            "instructions": if hardware { HARDWARE_SOURCE } else { None },
            "cycles": if hardware { HARDWARE_SOURCE } else { None },
            "allocations": "counting #[global_allocator] over std::alloc::System",
            "allocated_bytes": "counting #[global_allocator] over std::alloc::System",
            "peak_live_growth_bytes": "counting #[global_allocator] over std::alloc::System",
        },
    });
    writeln!(out, "{run}").expect("write run record");

    for cycle in 0..config.warmup + config.read_samples {
        for position in 0..workloads.len() {
            let (name, index, read, take) = &workloads[(cycle + position) % workloads.len()];
            let before = Counters::read(hardware);
            let allocations = Allocations::start();
            let start = Instant::now();
            let observed = read_once(&tables[*index], *read, &query, take).await;
            let wall_ns = start.elapsed().as_nanos() as u64;
            let (allocation_count, allocated_bytes, peak_live_growth_bytes) = allocations.since();
            let after = Counters::read(hardware);
            if cycle < config.warmup {
                continue;
            }
            let record = json!({
                "record": "sample",
                "schema": SCHEMA,
                "build": build,
                "workload": name,
                "table": TABLES[*index].name,
                "read": read.name(),
                "sample": cycle - config.warmup,
                "wall_ns": wall_ns,
                "cpu_ns": delta(after.cpu_ns, before.cpu_ns),
                "instructions": delta(after.instructions, before.instructions),
                "cycles": delta(after.cycles, before.cycles),
                "allocations": allocation_count,
                "allocated_bytes": allocated_bytes,
                "peak_live_growth_bytes": peak_live_growth_bytes,
                "rows": observed,
            });
            writeln!(out, "{record}").expect("write sample");
        }
    }
}

/// Visible embeddings the read returned, or the hits of a search.
async fn read_once(dataset: &Dataset, read: Read, query: &Float32Array, take: &[u64]) -> u64 {
    match read {
        Read::Scan => {
            let mut scanner = dataset.scan();
            scanner.project(&["embedding"]).expect("project embedding");
            let mut stream = scanner.try_into_stream().await.expect("scan");
            let mut visible = 0u64;
            while let Some(batch) = stream.next().await {
                let embedding = &batch.expect("batch")["embedding"];
                visible += (embedding.len() - embedding.null_count()) as u64;
            }
            visible
        }
        Read::Nearest => nearest_scanner(dataset, query)
            .try_into_batch()
            .await
            .expect("nearest")
            .num_rows() as u64,
        Read::Take => {
            let batch = dataset
                .take(
                    take,
                    ProjectionRequest::from_columns(["embedding"], dataset.schema()),
                )
                .await
                .expect("take");
            let embedding: &ArrayRef = &batch["embedding"];
            (embedding.len() - embedding.null_count()) as u64
        }
    }
}
