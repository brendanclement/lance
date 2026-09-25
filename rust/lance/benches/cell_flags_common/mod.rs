// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Shared utilities for the cell-flag benchmarks.
//!
//! Included from bench targets with
//! `#[path = "cell_flags_common/mod.rs"] mod common;`. This directory has no
//! `main.rs`, so Cargo's bench auto-discovery does not treat it as a target.
//!
//! Everything here uses only APIs that exist on `main`, so the same file
//! compiles against the baseline and the cell-flag prototype.

// Each bench binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use arrow_array::builder::StringBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{
    Array, ArrayRef, Int64Array, RecordBatch, RecordBatchIterator, StringArray, new_null_array,
};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use futures::{TryStreamExt, stream};
use lance::dataset::builder::DatasetBuilder;
use lance::dataset::transaction::{DataReplacementGroup, Operation, Transaction};
use lance::dataset::{
    CommitBuilder, Dataset, InsertBuilder, MergeInsertBuilder, MergeInsertWriteMode, MergeStats,
    NewColumnTransform, UpdateBuilder, WhenMatched, WhenNotMatched, WriteMode, WriteParams,
};
use lance::session::Session;
use lance_io::object_store::ObjectStore;
use roaring::RoaringTreemap;
use serde_json::{Map, Value, json};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Parse an environment variable, or return `default` when it is unset.
///
/// A set but unparsable value aborts the run rather than silently falling back.
pub fn env_or<T: FromStr>(name: &str, default: T) -> T {
    match std::env::var(name) {
        Ok(raw) => raw
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("cannot parse env var {name}={raw:?}")),
        Err(_) => default,
    }
}

/// Parse a comma-separated environment variable, or return `default` when unset.
pub fn env_list<T: FromStr + Clone>(name: &str, default: &[T]) -> Vec<T> {
    match std::env::var(name) {
        Ok(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(|token| {
                token
                    .parse()
                    .unwrap_or_else(|_| panic!("cannot parse {token:?} in env var {name}={raw:?}"))
            })
            .collect(),
        Err(_) => default.to_vec(),
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|raw| matches!(raw.trim(), "1" | "true" | "yes" | "on"))
}

#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// `BENCH_BUILD`, e.g. "baseline" or "prototype".
    pub build: String,
    /// `BENCH_GIT_SHA`.
    pub git_sha: Option<String>,
    /// `BENCH_SCALE_ROWS`: rows in the initial table.
    pub scale_rows: u64,
    /// `BENCH_ROWS_PER_FRAGMENT`: `max_rows_per_file` of the initial write.
    pub rows_per_fragment: u64,
    /// `BENCH_SAMPLES`: recorded samples per workload variant.
    pub samples: usize,
    /// `BENCH_READ_SAMPLES`: recorded samples per read workload variant
    /// (default `BENCH_SAMPLES`); reads are cheap and noisy, so use more.
    pub read_samples: usize,
    /// `BENCH_WARMUP`: unrecorded samples run first per workload variant.
    pub warmup: usize,
    /// `BENCH_WORKLOADS`: selected workload names or `name_` prefixes; empty selects all.
    pub workloads: Vec<String>,
    /// `BENCH_DATA_DIR`: parent directory for the benchmark datasets.
    pub data_dir: PathBuf,
    /// `BENCH_OUT`: JSON Lines output, appended to.
    pub out_path: PathBuf,
    /// `BENCH_STABLE_ROW_IDS`: create tables with stable row ids (default off).
    pub stable_row_ids: bool,
    /// `BENCH_UDF_ITERS`: hash rounds per row in the simulated UDF.
    pub udf_iterations: u32,
    /// `BENCH_KEEP_DATA`: keep the datasets after the run.
    pub keep_data: bool,
}

impl BenchConfig {
    pub fn from_env() -> Self {
        let default_root = std::env::temp_dir().join("lance_cell_flags_bench");
        let data_dir = std::env::var("BENCH_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_root.join("data"));
        let out_path = std::env::var("BENCH_OUT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_root.join("results.jsonl"));
        let samples = env_or("BENCH_SAMPLES", 5);
        let config = Self {
            build: std::env::var("BENCH_BUILD").unwrap_or_else(|_| "unknown".to_string()),
            git_sha: std::env::var("BENCH_GIT_SHA").ok(),
            scale_rows: env_or("BENCH_SCALE_ROWS", 1_000_000),
            rows_per_fragment: env_or("BENCH_ROWS_PER_FRAGMENT", 100_000),
            samples,
            read_samples: env_or("BENCH_READ_SAMPLES", samples),
            warmup: env_or("BENCH_WARMUP", 1),
            workloads: env_list::<String>("BENCH_WORKLOADS", &[]),
            data_dir,
            out_path,
            stable_row_ids: env_flag("BENCH_STABLE_ROW_IDS"),
            udf_iterations: env_or("BENCH_UDF_ITERS", DEFAULT_UDF_ITERATIONS),
            keep_data: env_flag("BENCH_KEEP_DATA"),
        };
        assert!(
            config.scale_rows > 0
                && config.rows_per_fragment > 0
                && config.samples > 0
                && config.read_samples > 0,
            "BENCH_SCALE_ROWS ({}), BENCH_ROWS_PER_FRAGMENT ({}), BENCH_SAMPLES ({}) and \
             BENCH_READ_SAMPLES ({}) must be positive",
            config.scale_rows,
            config.rows_per_fragment,
            config.samples,
            config.read_samples
        );
        config
    }

    /// Whether `name` was selected by `BENCH_WORKLOADS` (exact name, `name_`
    /// prefix such as `refresh_permissive_conflicts`, or `all`).
    pub fn selects(&self, name: &str) -> bool {
        self.workloads.is_empty()
            || self.workloads.iter().any(|token| {
                token == "all" || token == name || name.starts_with(&format!("{token}_"))
            })
    }

    pub fn expected_fragments(&self) -> u64 {
        self.scale_rows.div_ceil(self.rows_per_fragment)
    }
}

// ---------------------------------------------------------------------------
// Deterministic data
// ---------------------------------------------------------------------------

/// Root seed; every generated value is a pure function of it and the row id.
pub const SEED: u64 = 0x5eed_ce11_f1a9_2026;
pub const TITLE_LEN: usize = 16;
pub const BODY_LEN: usize = 64;
pub const LANGUAGES: [&str; 8] = ["en", "fr", "de", "es", "it", "pt", "ja", "zh"];
/// Tag naming the initial logical state that mutation samples restore.
pub const INITIAL_TAG: &str = "initial";
pub const ALL_COLUMNS: [&str; 7] = [
    "id",
    "title",
    "body",
    "language",
    "views",
    "summary",
    "translation",
];

/// SplitMix64 step: advances `state` and returns the next output.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn row_hash(id: u64, salt: u64) -> u64 {
    let mut state = SEED ^ salt ^ id.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    splitmix64(&mut state)
}

const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz eaiot";

fn text_for(id: u64, salt: u64, len: usize) -> String {
    let mut state = SEED ^ salt ^ id.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    let mut out = String::with_capacity(len);
    let mut bits = 0u64;
    let mut available = 0u32;
    for _ in 0..len {
        if available < 5 {
            bits = splitmix64(&mut state);
            available = 64;
        }
        out.push(ALPHABET[(bits & 31) as usize] as char);
        bits >>= 5;
        available -= 5;
    }
    out
}

pub fn title_for(id: u64) -> String {
    text_for(id, 1, TITLE_LEN)
}

pub fn body_for(id: u64) -> String {
    text_for(id, 2, BODY_LEN)
}

/// A replacement body for source writes; `generation` distinguishes rewrites.
pub fn rewritten_body_for(id: u64, generation: u64) -> String {
    text_for(id, 0x100 + generation, BODY_LEN)
}

pub fn language_for(id: u64) -> &'static str {
    LANGUAGES[(row_hash(id, 3) % LANGUAGES.len() as u64) as usize]
}

pub fn views_for(id: u64) -> i64 {
    (row_hash(id, 4) % 1_000_000) as i64
}

pub fn base_schema() -> Arc<ArrowSchema> {
    Arc::new(ArrowSchema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("body", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("views", DataType::Int64, false),
    ]))
}

pub fn output_schema() -> Arc<ArrowSchema> {
    Arc::new(ArrowSchema::new(vec![
        Field::new("summary", DataType::Utf8, true),
        Field::new("translation", DataType::Utf8, true),
    ]))
}

pub fn articles_schema() -> Arc<ArrowSchema> {
    let fields = base_schema()
        .fields()
        .iter()
        .chain(output_schema().fields().iter())
        .cloned()
        .collect::<Vec<_>>();
    Arc::new(ArrowSchema::new(fields))
}

/// Source columns for ids `start..start + len`.
pub fn base_batch(start: u64, len: usize) -> RecordBatch {
    let ids = start..start + len as u64;
    RecordBatch::try_new(
        base_schema(),
        vec![
            Arc::new(Int64Array::from_iter_values(
                ids.clone().map(|id| id as i64),
            )),
            Arc::new(StringArray::from_iter_values(ids.clone().map(title_for))),
            Arc::new(StringArray::from_iter_values(ids.clone().map(body_for))),
            Arc::new(StringArray::from_iter_values(ids.clone().map(language_for))),
            Arc::new(Int64Array::from_iter_values(ids.map(views_for))),
        ],
    )
    .expect("base batch matches base schema")
}

/// Full-schema rows for ids `start..start + len`, with NULL outputs (for appends).
pub fn articles_batch(start: u64, len: usize) -> RecordBatch {
    let base = base_batch(start, len);
    let mut columns = base.columns().to_vec();
    columns.push(new_null_array(&DataType::Utf8, len));
    columns.push(new_null_array(&DataType::Utf8, len));
    RecordBatch::try_new(articles_schema(), columns).expect("articles batch matches schema")
}

/// `k` distinct ids in `0..n`, chosen by Floyd's algorithm from `seed`, sorted.
pub fn scattered_ids(n: u64, k: u64, seed: u64) -> Vec<u64> {
    assert!(k <= n, "cannot choose {k} distinct ids out of {n}");
    let mut state = SEED ^ seed;
    let mut chosen = RoaringTreemap::new();
    for upper in (n - k)..n {
        let candidate = splitmix64(&mut state) % (upper + 1);
        if !chosen.insert(candidate) {
            chosen.insert(upper);
        }
    }
    chosen.into_iter().collect()
}

/// Split sorted `ids` into `groups` round-robin, so each group spans the id range.
pub fn round_robin_groups(ids: &[u64], groups: usize) -> Vec<Vec<u64>> {
    let mut out = vec![Vec::with_capacity(ids.len().div_ceil(groups)); groups];
    for (position, id) in ids.iter().enumerate() {
        out[position % groups].push(*id);
    }
    out
}

pub fn id_in_predicate(ids: &[u64]) -> String {
    let list = ids
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("id IN ({list})")
}

/// Create the articles table at `uri`: the source columns written with
/// `max_rows_per_file = rows_per_fragment`, then `summary` and `translation`
/// declared all-NULL with a metadata-only `add_columns`. The resulting version
/// is tagged [`INITIAL_TAG`]. Any existing directory at `uri` is removed.
pub async fn create_articles_dataset(
    uri: &Path,
    config: &BenchConfig,
    session: Arc<Session>,
) -> Dataset {
    if uri.exists() {
        std::fs::remove_dir_all(uri).expect("remove stale benchmark dataset");
    }
    const GENERATE_BATCH_ROWS: u64 = 8192;
    let total = config.scale_rows;
    let batches = (0..total)
        .step_by(GENERATE_BATCH_ROWS as usize)
        .map(move |start| {
            Ok(base_batch(
                start,
                GENERATE_BATCH_ROWS.min(total - start) as usize,
            ))
        });
    let reader = RecordBatchIterator::new(batches, base_schema());
    let params = WriteParams {
        mode: WriteMode::Create,
        max_rows_per_file: config.rows_per_fragment as usize,
        enable_stable_row_ids: config.stable_row_ids,
        auto_cleanup: None,
        skip_auto_cleanup: true,
        session: Some(session),
        ..Default::default()
    };
    let mut dataset = Dataset::write(reader, uri.to_str().expect("utf-8 path"), Some(params))
        .await
        .expect("write articles dataset");
    dataset
        .add_columns(NewColumnTransform::AllNulls(output_schema()), None, None)
        .await
        .expect("declare output columns");
    dataset
        .tags()
        .create(INITIAL_TAG, dataset.version().version)
        .await
        .expect("tag initial version");
    assert_eq!(
        dataset.get_fragments().len() as u64,
        config.expected_fragments(),
        "fragment count of the initial write"
    );
    dataset
}

// ---------------------------------------------------------------------------
// Simulated UDF
// ---------------------------------------------------------------------------

/// Hash rounds per row: about 1.3 µs/row for `summary` and 1.1 µs/row for
/// `translation` on an Apple M5 Pro in `release-with-debug`. Each run prints
/// its own calibration.
pub const DEFAULT_UDF_ITERATIONS: u32 = 16;

/// Output columns the simulated UDFs compute and the source columns each reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColumn {
    /// `summary = f(title, body)`
    Summary,
    /// `translation = g(body, language)`
    Translation,
}

impl OutputColumn {
    pub fn name(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Translation => "translation",
        }
    }

    pub fn inputs(self) -> [&'static str; 2] {
        match self {
            Self::Summary => ["title", "body"],
            Self::Translation => ["body", "language"],
        }
    }
}

/// A deterministic stand-in for an expensive model call: `iterations` rounds of
/// FNV-1a over the inputs, formatted as a short string.
#[derive(Debug, Clone, Copy)]
pub struct SimulatedUdf {
    pub iterations: u32,
}

impl SimulatedUdf {
    fn digest(&self, salt: u64, first: &str, second: &str) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325 ^ salt;
        for round in 0..self.iterations {
            hash ^= u64::from(round);
            for byte in first.bytes().chain([0xff]).chain(second.bytes()) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
            }
        }
        hash
    }

    pub fn summary(&self, title: &str, body: &str) -> String {
        format!("summary:{:016x}", self.digest(0x5u64, title, body))
    }

    pub fn translation(&self, body: &str, language: &str) -> String {
        format!("{language}:{:016x}", self.digest(0x7u64, body, language))
    }

    pub fn compute(&self, column: OutputColumn, first: &str, second: &str) -> String {
        match column {
            OutputColumn::Summary => self.summary(first, second),
            OutputColumn::Translation => self.translation(first, second),
        }
    }

    /// Compute `column` for a batch holding `id` and the column's inputs. Rows
    /// whose id is in `null_ids` get NULL instead.
    pub fn apply(
        &self,
        column: OutputColumn,
        batch: &RecordBatch,
        null_ids: Option<&RoaringTreemap>,
    ) -> ArrayRef {
        let [first_name, second_name] = column.inputs();
        let ids = batch["id"].as_primitive::<Int64Type>();
        let first = batch[first_name].as_string::<i32>();
        let second = batch[second_name].as_string::<i32>();
        let mut builder = StringBuilder::with_capacity(batch.num_rows(), batch.num_rows() * 32);
        for row in 0..batch.num_rows() {
            if null_ids.is_some_and(|nulls| nulls.contains(ids.value(row) as u64)) {
                builder.append_null();
            } else {
                builder.append_value(self.compute(column, first.value(row), second.value(row)));
            }
        }
        Arc::new(builder.finish())
    }

    /// Mean nanoseconds per row for `column` over freshly generated rows.
    pub fn calibrate_ns_per_row(&self, column: OutputColumn, rows: usize) -> f64 {
        let batch = base_batch(0, rows);
        let start = Instant::now();
        let output = self.apply(column, &batch, None);
        let elapsed = start.elapsed();
        std::hint::black_box(output);
        elapsed.as_nanos() as f64 / rows as f64
    }
}

// ---------------------------------------------------------------------------
// Refresh (compute + stage + publish)
// ---------------------------------------------------------------------------

/// Column files staged by a refresh, not yet committed.
pub struct StagedRefresh {
    pub column: OutputColumn,
    /// Version the refresh read; the publication must commit with this read version.
    pub read_version: u64,
    pub replacements: Vec<DataReplacementGroup>,
    pub rows_staged: u64,
    /// Time spent pulling input batches from the snapshot.
    pub read_ms: f64,
    /// Time spent inside the simulated UDF.
    pub udf_ms: f64,
    /// Time spent in `FileFragment::write_columns` (staging the column files).
    pub stage_ms: f64,
}

impl StagedRefresh {
    /// The permissive publication: a plain `DataReplacement` at `read_version`.
    pub fn transaction(&self) -> Transaction {
        Transaction::new(
            self.read_version,
            Operation::DataReplacement {
                replacements: self.replacements.clone(),
            },
            None,
        )
    }
}

/// Read `snapshot` fragment by fragment, compute `column` for every row, and
/// stage one full-fragment column file per fragment with
/// `FileFragment::write_columns`. Fragments are processed sequentially; each
/// fragment's output is held in memory only until it is staged.
pub async fn stage_refresh(
    snapshot: &Dataset,
    column: OutputColumn,
    udf: SimulatedUdf,
    null_ids: Option<&RoaringTreemap>,
) -> StagedRefresh {
    let target_schema = snapshot
        .schema()
        .project(&[column.name()])
        .expect("output column is declared");
    let target_arrow = Arc::new(ArrowSchema::from(&target_schema));
    let [first, second] = column.inputs();
    let mut staged = StagedRefresh {
        column,
        read_version: snapshot.version().version,
        replacements: Vec::new(),
        rows_staged: 0,
        read_ms: 0.0,
        udf_ms: 0.0,
        stage_ms: 0.0,
    };
    for fragment in snapshot.get_fragments() {
        let metadata = fragment.metadata();
        assert!(
            metadata.deletion_file.is_none(),
            "refresh staging assumes fragment {} has no deletions",
            metadata.id
        );
        let mut scanner = fragment.scan();
        scanner
            .project(&["id", first, second])
            .expect("project refresh inputs");
        scanner.scan_in_order(true);

        let read_start = Instant::now();
        let mut inputs = scanner.try_into_stream().await.expect("scan fragment");
        staged.read_ms += ms_since(read_start);
        let mut outputs = Vec::new();
        loop {
            let read_start = Instant::now();
            let Some(batch) = inputs.try_next().await.expect("read fragment batch") else {
                staged.read_ms += ms_since(read_start);
                break;
            };
            staged.read_ms += ms_since(read_start);

            let udf_start = Instant::now();
            let values = udf.apply(column, &batch, null_ids);
            staged.udf_ms += ms_since(udf_start);
            staged.rows_staged += values.len() as u64;
            outputs.push(
                RecordBatch::try_new(target_arrow.clone(), vec![values])
                    .expect("output batch matches the declared column"),
            );
        }

        let stage_start = Instant::now();
        let replacement = fragment
            .write_columns(stream::iter(outputs.into_iter().map(Ok)), &target_schema)
            .await
            .expect("stage column file");
        staged.stage_ms += ms_since(stage_start);
        staged.replacements.push(replacement);
    }
    staged
}

/// Commit `transaction` against `dataset` through `CommitBuilder`.
pub async fn commit(dataset: Arc<Dataset>, transaction: Transaction) -> lance::Result<Dataset> {
    CommitBuilder::new(dataset).execute(transaction).await
}

/// Rows among `ids` whose stored `summary` is non-NULL and differs from the UDF
/// applied to the row's current `title` and `body`.
pub async fn count_stale_summaries(dataset: &Dataset, udf: SimulatedUdf, ids: &[u64]) -> u64 {
    let mut scanner = dataset.scan();
    scanner
        .project(&["id", "title", "body", "summary"])
        .expect("project stale check")
        .filter(&id_in_predicate(ids))
        .expect("filter stale check");
    let mut stream = scanner.try_into_stream().await.expect("scan stale check");
    let mut stale = 0;
    while let Some(batch) = stream.try_next().await.expect("read stale check batch") {
        let title = batch["title"].as_string::<i32>();
        let body = batch["body"].as_string::<i32>();
        let summary = batch["summary"].as_string::<i32>();
        for row in 0..batch.num_rows() {
            if summary.is_valid(row)
                && summary.value(row) != udf.summary(title.value(row), body.value(row))
            {
                stale += 1;
            }
        }
    }
    stale
}

// ---------------------------------------------------------------------------
// Source writes
// ---------------------------------------------------------------------------

/// A row-moving write: `UpdateBuilder` setting `column = value_sql` where `predicate`.
/// Returns the new dataset and the number of rows updated.
pub async fn update_rows(
    dataset: Arc<Dataset>,
    predicate: &str,
    column: &str,
    value_sql: &str,
) -> (Arc<Dataset>, u64) {
    let result = UpdateBuilder::new(dataset)
        .update_where(predicate)
        .expect("update predicate")
        .set(column, value_sql)
        .expect("update assignment")
        .build()
        .expect("build update")
        .execute()
        .await
        .expect("execute update");
    (result.new_dataset, result.rows_updated)
}

/// An `(id, body)` source batch that rewrites `body` for `ids`.
pub fn body_source_batch(dataset: &Dataset, ids: &[u64], generation: u64) -> RecordBatch {
    let full = ArrowSchema::from(dataset.schema());
    let positions = ["id", "body"].map(|name| full.index_of(name).expect("source column exists"));
    let schema = Arc::new(
        full.project(&positions)
            .expect("project merge source schema"),
    );
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from_iter_values(
                ids.iter().map(|id| *id as i64),
            )),
            Arc::new(StringArray::from_iter_values(
                ids.iter().map(|id| rewritten_body_for(*id, generation)),
            )),
        ],
    )
    .expect("merge source batch")
}

/// Timings of a merge insert split into its uncommitted write and its commit.
pub struct MergeOutcome {
    pub dataset: Dataset,
    pub stats: MergeStats,
    pub stage_ms: f64,
    pub commit_ms: f64,
}

/// An in-place partial-schema write: `merge_insert` on `id` with
/// `MergeInsertWriteMode::RewriteColumns`, `WhenMatched::UpdateAll`,
/// `WhenNotMatched::DoNothing`, committed through `CommitBuilder` exactly as
/// `MergeInsertJob::execute` does.
pub async fn merge_in_place(dataset: Arc<Dataset>, source: RecordBatch) -> MergeOutcome {
    let job = MergeInsertBuilder::try_new(dataset.clone(), vec!["id".to_string()])
        .expect("merge insert builder")
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::DoNothing)
        .write_mode(MergeInsertWriteMode::RewriteColumns)
        .try_build()
        .expect("build merge insert");
    let stage_start = Instant::now();
    let uncommitted = job
        .execute_uncommitted_batches(vec![source])
        .await
        .expect("execute merge insert");
    let stage_ms = ms_since(stage_start);
    let mut builder = CommitBuilder::new(dataset);
    if let Some(affected_rows) = uncommitted.affected_rows {
        builder = builder.with_affected_rows(affected_rows);
    }
    let commit_start = Instant::now();
    let committed = builder
        .execute(uncommitted.transaction)
        .await
        .expect("commit merge insert");
    MergeOutcome {
        dataset: committed,
        stats: uncommitted.stats,
        stage_ms,
        commit_ms: ms_since(commit_start),
    }
}

/// Append `batch` with `InsertBuilder`, split into write and commit.
/// Returns `(dataset, stage_ms, commit_ms)`.
pub async fn append(dataset: Arc<Dataset>, batch: RecordBatch) -> (Dataset, f64, f64) {
    let params = WriteParams {
        mode: WriteMode::Append,
        skip_auto_cleanup: true,
        ..Default::default()
    };
    let stage_start = Instant::now();
    let transaction = InsertBuilder::new(dataset.clone())
        .with_params(&params)
        .execute_uncommitted(vec![batch])
        .await
        .expect("write appended fragment");
    let stage_ms = ms_since(stage_start);
    let commit_start = Instant::now();
    let committed = commit(dataset, transaction).await.expect("commit append");
    (committed, stage_ms, ms_since(commit_start))
}

// ---------------------------------------------------------------------------
// Sample state control
// ---------------------------------------------------------------------------

/// Start a mutation sample from the initial logical state.
///
/// Checks out the [`INITIAL_TAG`] version and commits `Operation::Restore` of
/// it (metadata only: one manifest and one transaction file), so every sample
/// starts from identical fragments and data files. Then removes every other
/// untagged version and its unreferenced files, so `_versions` and the data
/// directory do not grow across samples. Version *numbers* still grow: each
/// sample adds one restore version plus the versions its workload commits.
/// Restore keeps `max_fragment_id` (and `next_row_id`) as high-water marks, so
/// fragments created after a restore get fresh ids.
pub async fn reset_to_initial(dataset: &Dataset) -> Dataset {
    let mut restored = dataset
        .checkout_version(INITIAL_TAG)
        .await
        .expect("checkout initial tag");
    restored.restore().await.expect("restore initial version");
    restored
        .cleanup_old_versions(chrono::Duration::zero(), Some(true), Some(false))
        .await
        .expect("clean up previous samples");
    restored
}

/// Logical content of a table: row and fragment counts plus a hash of every
/// column in scan order. Equal fingerprints mean identical logical state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    pub rows: u64,
    pub fragments: usize,
    pub content_hash: u64,
}

pub async fn fingerprint(dataset: &Dataset) -> Fingerprint {
    let mut scanner = dataset.scan();
    scanner
        .project(&ALL_COLUMNS)
        .expect("project all columns")
        .scan_in_order(true);
    let mut stream = scanner.try_into_stream().await.expect("scan fingerprint");
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
        }
    };
    let mut rows = 0;
    while let Some(batch) = stream.try_next().await.expect("read fingerprint batch") {
        rows += batch.num_rows() as u64;
        for column in batch.columns() {
            for row in 0..column.len() {
                if column.is_null(row) {
                    mix(&[0xfe]);
                } else if let Some(strings) = column.as_string_opt::<i32>() {
                    mix(strings.value(row).as_bytes());
                    mix(&[0xff]);
                } else {
                    mix(&column.as_primitive::<Int64Type>().value(row).to_le_bytes());
                }
            }
        }
    }
    Fingerprint {
        rows,
        fragments: dataset.get_fragments().len(),
        content_hash: hash,
    }
}

// ---------------------------------------------------------------------------
// Timing, IO and size accounting
// ---------------------------------------------------------------------------

pub fn ms_since(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1e3
}

/// Await `future` and return its output with the elapsed milliseconds.
pub async fn timed<F: Future>(future: F) -> (F::Output, f64) {
    let start = Instant::now();
    let output = future.await;
    (output, ms_since(start))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IoDelta {
    pub read_iops: u64,
    pub read_bytes: u64,
    pub write_iops: u64,
    pub written_bytes: u64,
}

/// Object-store IO counters of one dataset's store, via
/// `ObjectStore::io_stats_incremental`. Datasets sharing a `Session` share the
/// store (and its counters) for the same base, so run measured regions serially.
pub struct IoMeter {
    store: Arc<ObjectStore>,
}

impl IoMeter {
    pub async fn new(dataset: &Dataset) -> Self {
        let meter = Self {
            store: dataset
                .object_store(None)
                .await
                .expect("dataset object store"),
        };
        meter.reset();
        meter
    }

    pub fn reset(&self) {
        self.store.io_stats_incremental();
    }

    /// IO since the last `reset`/`take`; resets the counters.
    pub fn take(&self) -> IoDelta {
        let stats = self.store.io_stats_incremental();
        IoDelta {
            read_iops: stats.read_iops,
            read_bytes: stats.read_bytes,
            write_iops: stats.write_iops,
            written_bytes: stats.written_bytes,
        }
    }
}

/// Sizes of the checked-out version's manifest file and of its transaction file
/// (`_transactions/<manifest.transaction_file>`). Issues HEAD requests; call it
/// outside measured regions.
pub async fn version_file_sizes(dataset: &Dataset) -> (Option<u64>, Option<u64>) {
    let store = dataset
        .object_store(None)
        .await
        .expect("dataset object store");
    let location = dataset.manifest_location();
    let manifest_bytes = match location.size {
        Some(size) => Some(size),
        None => store.size(&location.path).await.ok(),
    };
    let txn_bytes = match dataset.manifest().transaction_file.as_deref() {
        Some(file) if !file.is_empty() => store
            .size(&dataset.transactions_dir().join(file))
            .await
            .ok(),
        _ => None,
    };
    (manifest_bytes, txn_bytes)
}

/// Total bytes of all files under `path`.
pub fn dir_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| match entry.metadata() {
            Ok(metadata) if metadata.is_dir() => dir_bytes(&entry.path()),
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Open `uri` in a brand-new `Session` (empty Lance caches, new object store).
/// The OS page cache is not controlled.
pub async fn open_fresh_session(uri: &str) -> Dataset {
    DatasetBuilder::from_uri(uri)
        .with_session(Arc::new(Session::default()))
        .load()
        .await
        .expect("open dataset in a fresh session")
}

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// The open `Dataset`/`Session` is reused across samples.
    Warm,
    /// A new `Session` and `Dataset` open per sample. Lance caches start empty;
    /// the OS page cache is not controlled, so this is not a cold read.
    FreshSession,
}

impl CacheState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Warm => "warm",
            Self::FreshSession => "fresh-session",
        }
    }
}

/// One measured sample. `None` fields serialize as JSON `null`.
#[derive(Debug, Clone)]
pub struct Record {
    pub workload: String,
    pub variant: String,
    pub sample: usize,
    pub fragments: usize,
    pub wall_ms: f64,
    pub udf_ms: Option<f64>,
    pub stage_ms: Option<f64>,
    pub commit_ms: Option<f64>,
    pub rows_written: u64,
    pub rows_published: Option<u64>,
    pub io: IoDelta,
    pub manifest_bytes: Option<u64>,
    pub txn_bytes: Option<u64>,
    pub dataset_bytes: u64,
    pub cache_state: CacheState,
    /// "committed", "conflict", or "error:<kind>" for workloads that may fail.
    pub outcome: Option<String>,
    pub stale_results_published: Option<bool>,
    pub stale_rows: Option<u64>,
    pub notes: String,
    /// Workload-specific values (e.g. `read_ms`, `open_ms`, `version`).
    pub extra: Map<String, Value>,
}

impl Record {
    pub fn new(workload: impl Into<String>, variant: impl Into<String>) -> Self {
        Self {
            workload: workload.into(),
            variant: variant.into(),
            sample: 0,
            fragments: 0,
            wall_ms: 0.0,
            udf_ms: None,
            stage_ms: None,
            commit_ms: None,
            rows_written: 0,
            rows_published: None,
            io: IoDelta::default(),
            manifest_bytes: None,
            txn_bytes: None,
            dataset_bytes: 0,
            cache_state: CacheState::Warm,
            outcome: None,
            stale_results_published: None,
            stale_rows: None,
            notes: String::new(),
            extra: Map::new(),
        }
    }

    pub fn extra(&mut self, key: &str, value: impl Into<Value>) -> &mut Self {
        self.extra.insert(key.to_string(), value.into());
        self
    }
}

/// Writes records as JSON Lines to `BENCH_OUT` and keeps them for the summary.
pub struct Reporter {
    config: BenchConfig,
    out: std::fs::File,
    records: Vec<Record>,
}

impl Reporter {
    pub fn new(config: &BenchConfig) -> Self {
        if let Some(parent) = config.out_path.parent() {
            std::fs::create_dir_all(parent).expect("create output directory");
        }
        let out = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.out_path)
            .expect("open BENCH_OUT");
        Self {
            config: config.clone(),
            out,
            records: Vec::new(),
        }
    }

    pub fn emit(&mut self, record: Record) {
        let value = json!({
            "build": self.config.build,
            "git_sha": self.config.git_sha,
            "workload": record.workload,
            "variant": record.variant,
            "scale_rows": self.config.scale_rows,
            "rows_per_fragment": self.config.rows_per_fragment,
            "fragments": record.fragments,
            "sample": record.sample,
            "wall_ms": record.wall_ms,
            "udf_ms": record.udf_ms,
            "stage_ms": record.stage_ms,
            "commit_ms": record.commit_ms,
            "rows_written": record.rows_written,
            "rows_published": record.rows_published,
            "read_iops": record.io.read_iops,
            "read_bytes": record.io.read_bytes,
            "write_iops": record.io.write_iops,
            "written_bytes": record.io.written_bytes,
            "manifest_bytes": record.manifest_bytes,
            "txn_bytes": record.txn_bytes,
            "dataset_bytes": record.dataset_bytes,
            "cache_state": record.cache_state.label(),
            "stable_row_ids": self.config.stable_row_ids,
            "udf_iterations": self.config.udf_iterations,
            "outcome": record.outcome,
            "stale_results_published": record.stale_results_published,
            "stale_rows": record.stale_rows,
            "notes": record.notes,
            "extra": record.extra,
        });
        writeln!(self.out, "{value}").expect("write JSON line");
        self.records.push(record);
    }

    /// Print medians per (workload, variant, cache_state), in run order.
    pub fn print_summary(&self) {
        let mut order: Vec<(String, String, &'static str)> = Vec::new();
        let mut groups: HashMap<(String, String, &'static str), Vec<&Record>> = HashMap::new();
        for record in &self.records {
            let key = (
                record.workload.clone(),
                record.variant.clone(),
                record.cache_state.label(),
            );
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(record);
        }
        println!(
            "\n{:<34} {:<24} {:<13} {:>2} {:>10} {:>9} {:>9} {:>9} {:>7} {:>9} {:>6} {:>10} {:>8} {:>7}  outcome",
            "workload",
            "variant",
            "cache",
            "n",
            "wall_ms",
            "udf_ms",
            "stage_ms",
            "commit_ms",
            "r_iops",
            "r_KiB",
            "w_iops",
            "w_KiB",
            "manif_B",
            "txn_B",
        );
        for key in order {
            let records = &groups[&key];
            let median_f = |f: &dyn Fn(&Record) -> Option<f64>| {
                median(records.iter().filter_map(|record| f(record)).collect())
            };
            let fmt_ms = |value: Option<f64>| value.map_or("-".to_string(), |v| format!("{v:.2}"));
            let fmt_n = |value: Option<f64>| value.map_or("-".to_string(), |v| format!("{v:.0}"));
            let mut outcomes: Vec<String> = Vec::new();
            for record in records.iter() {
                let label = match (&record.outcome, record.stale_rows) {
                    (Some(outcome), Some(stale)) if stale > 0 => {
                        format!("{outcome}(stale={stale})")
                    }
                    (Some(outcome), _) => outcome.clone(),
                    (None, _) => continue,
                };
                if !outcomes.contains(&label) {
                    outcomes.push(label);
                }
            }
            println!(
                "{:<34} {:<24} {:<13} {:>2} {:>10} {:>9} {:>9} {:>9} {:>7} {:>9} {:>6} {:>10} {:>8} {:>7}  {}",
                key.0,
                key.1,
                key.2,
                records.len(),
                fmt_ms(median_f(&|r| Some(r.wall_ms))),
                fmt_ms(median_f(&|r| r.udf_ms)),
                fmt_ms(median_f(&|r| r.stage_ms)),
                fmt_ms(median_f(&|r| r.commit_ms)),
                fmt_n(median_f(&|r| Some(r.io.read_iops as f64))),
                fmt_n(median_f(&|r| Some(r.io.read_bytes as f64 / 1024.0))),
                fmt_n(median_f(&|r| Some(r.io.write_iops as f64))),
                fmt_n(median_f(&|r| Some(r.io.written_bytes as f64 / 1024.0))),
                fmt_n(median_f(&|r| r.manifest_bytes.map(|v| v as f64))),
                fmt_n(median_f(&|r| r.txn_bytes.map(|v| v as f64))),
                outcomes.join(","),
            );
        }
        println!("\nrecords appended to {}", self.config.out_path.display());
    }
}

pub fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Some(if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}
