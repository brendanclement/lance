// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! What dependent cell flags cost across the operations they touch, against
//! tables without flags, on shared tables.
//!
//! Create mode writes every table under one root. All have the same rows
//! (`cell_flags_common`'s articles: `id`, `title`, `body`, `language`,
//! `views`) and three nullable text outputs: `summary = f(title, body)`,
//! `translation = g(body, language)` and `keywords = h(summary, language)`.
//! Each is built the same way: the source columns are written, the outputs
//! added as all-NULL columns, the table's flags registered, and every output
//! stored for every fragment by one `DataReplacement` that publishes every
//! flag on every row. The topologies differ only in which outputs have a
//! dependent masking flag `ready`, watching the output's inputs:
//!
//! | Topology | Flagged outputs |
//! |---|---|
//! | `plain` | none |
//! | `one` | `summary` |
//! | `shared` | `summary`, `translation` (sharing `body`) |
//! | `chain` | `summary`, `keywords` (`body` → `summary` → `keywords`) |
//!
//! Tables in other states start from those: `partial_1pct`, `partial_50pct`
//! and `masked` (`one`) get an in-place `body` write on 1%, 50% or all rows,
//! which clears their flag, and `null_1pct`, `null_50pct` and `null_all`
//! (`plain`) store `summary` NULL there and get the same write, so each pair
//! reads the same visible data. `ready_moved` (`one`) and `plain_moved` get a
//! row-moving `views` update of 1% of rows: every flag stays true, but on
//! fragments that lost rows, so reads take the per-row mask path where
//! `one`'s whole-fragment states skip it. `pending_<topology>` and
//! `plain_empty` never store their outputs. `<topology>_h16` and `_h64` take
//! 16 or 64 row-moving `body` updates of 100 rows, and `<topology>_dense` one
//! of every tenth row, left pending; `one_h16r`, `one_h64r` and their `plain_`
//! pairs refresh `summary` after every update instead.
//!
//! What the refresh workloads write depends on where the pending rows are. A
//! row-moving write puts them in a new fragment, and the refresh stages that
//! fragment only. An in-place write leaves them scattered over the table's
//! fragments, and both the stager and a plain `merge_insert` in
//! `RewriteColumns` mode rewrite the refreshed columns of every fragment
//! touched (all ten, for 100 scattered rows); `_rewrite_rows` measures
//! `merge_insert` rewriting whole rows instead. Pending rows are found the way
//! the prototype documents, with a scan of live row addresses less each
//! flag's true rows: caller-side code whose cost grows with the table's rows.
//! A plain table's refresh finds the rows it wrote with a filter instead, so
//! its `locate` phase is the flagged `find` and `read_inputs` together, and
//! its `merge` phase is `merge_insert`'s write, join included.
//!
//! Measure mode runs the workloads `BENCH_WORKLOADS` names exactly (`list`
//! mode prints them all). Reads use the root tables. Every other sample works on a
//! copy of its table under `BENCH_SCRATCH_DIR` (default `<root>-scratch`),
//! cloned with `cp -Rc` (APFS clonefile; `cp --reflink=auto` elsewhere) so
//! every sample starts from the same bytes and history, then read once so
//! the OS page cache holds it, and opened in a new `Session`. The root is
//! never written.
//!
//! Each sample is one or more timed phases. A sample's record has, per phase
//! and summed over its phases: wall and CPU time, retired instructions and
//! cycles where available (`cell_flags_common/counters.rs`), the object
//! store's bytes and requests (lance-io `test-util` counters), and with
//! `BENCH_COUNT_ALLOCATIONS=1` allocation volume and peak live growth
//! (`cell_flags_common/alloc.rs`); plus the metadata sizes of the version it
//! ends at (reads and reopens: the root table's). A phase's peak is above the
//! bytes live at its start; a sample's is the most live during any of its
//! phases above those live when it started. Both count requested capacity,
//! such as the 5 MiB buffer every manifest write reserves, not resident
//! memory. Lance inlines a transaction of up to 20 MiB into the manifest
//! file, so `manifest_bytes` holds that copy too; `manifest_struct_bytes` is
//! the manifest proper (fragments, schema, flag state) that every open reads
//! and every commit rewrites. Output values are computed outside every phase,
//! so no phase includes the function's own computation. Workloads run in a
//! rotated order, reversed on every other pass, so each runs after both of
//! its neighbours.
//!
//! ```bash
//! BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/root BENCH_SCALE_ROWS=1000000 <binary>
//! BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=/abs/root BENCH_READ_SAMPLES=5 \
//!   BENCH_WORKLOADS=cycle_inplace_sparse_one BENCH_BUILD=final BENCH_OUT=/abs/samples.jsonl <binary>
//! ```
//!
//! `BENCH_TABLES` limits create mode to the named tables. Records follow
//! `cell_flags_scan_counters` (schema 2), and
//! `prototypes/dependent-cell-flags/bench/run_counters_rotation.py --bench
//! cell_flags_costs` drives it. Uses the prototype's cell flag API, so it
//! does not build on `main`. Sets `LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1`
//! itself, since release builds refuse flagged datasets without it.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/alloc.rs"]
mod allocations;
#[path = "cell_flags_common/mod.rs"]
mod common;
#[path = "cell_flags_common/counters.rs"]
mod counters;

use std::collections::HashMap;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use arrow_array::builder::StringBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::{Int64Type, UInt64Type};
use arrow_array::{Array, ArrayRef, Int64Array, RecordBatch, RecordBatchIterator, new_null_array};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use futures::{TryStreamExt, stream};
use lance::dataset::cell_flag::{
    CellFlagOptions, DeferralReason, PublicationReport, PublicationStager,
};
use lance::dataset::transaction::{
    CellFlagChanges, CellFlagUpdate, Operation, Transaction, TransactionBuilder,
};
use lance::dataset::write::merge_insert::UncommittedMergeInsert;
use lance::dataset::{
    CommitBuilder, Dataset, DependencyConflictPolicy, InsertBuilder, MergeInsertBuilder,
    MergeInsertWriteMode, NewColumnTransform, ProjectionRequest, PublicationResult, WhenMatched,
    WhenNotMatched, WriteMode, WriteParams,
};
use lance_core::ROW_ADDR;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::feature_flags::ENABLE_UNSTABLE_CELL_FLAGS_ENV;
use roaring::RoaringTreemap;
use serde_json::{Map, Value, json};

use allocations::{AllocationDelta, Allocations};
use common::{
    BenchConfig, IoDelta, IoMeter, SimulatedUdf, base_batch, base_schema, body_source_batch,
    dir_bytes, id_in_predicate, merge_in_place, open_fresh_session, round_robin_groups,
    scattered_ids, update_rows, version_file_sizes, views_for,
};
use counters::{
    CPU_NS_SOURCE, Counters, HARDWARE_SOURCE, dataset_identity, delta, hardware_counters_advance,
    required_env,
};

/// Version of the record layout; readers reject records of other versions.
const SCHEMA: u32 = 2;

const FLAG: &str = "ready";
const GENERATE_BATCH_ROWS: u64 = 8192;
/// Rows of every sparse write.
const SPARSE_ROWS: u64 = 100;
/// Rows a conflict workload's refresh publishes.
const CONFLICT_PENDING_ROWS: u64 = 1000;
/// Rows each concurrent write of a conflict workload changes.
const CONFLICT_ROWS_PER_COMMIT: u64 = 10;
const APPEND_ROWS: u64 = 100;
const TAKE_ROWS: u64 = 1000;
/// Rows of each update a history table takes.
const HISTORY_ROWS: u64 = 100;
/// Output values need not be expensive: no phase computes them.
const UDF: SimulatedUdf = SimulatedUdf { iterations: 1 };

const SEED_ONE_PERCENT: u64 = 21;
const SEED_HALF: u64 = 22;
/// `cell_flags_scan_counters`'s take seed.
const SEED_TAKE: u64 = 14;
const SEED_SPARSE: u64 = 31;
const SEED_CONFLICT: u64 = 32;
const SEED_AFTER_HISTORY: u64 = 33;
const SEED_HISTORY: u64 = 0x100;

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Output {
    Summary,
    Translation,
    Keywords,
}

impl Output {
    const ALL: [Self; 3] = [Self::Summary, Self::Translation, Self::Keywords];

    fn name(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Translation => "translation",
            Self::Keywords => "keywords",
        }
    }

    /// The fields whose writes clear the output's flag.
    fn inputs(self) -> [&'static str; 2] {
        match self {
            Self::Summary => ["title", "body"],
            Self::Translation => ["body", "language"],
            Self::Keywords => ["summary", "language"],
        }
    }
}

fn names(outputs: &[Output]) -> Vec<&'static str> {
    outputs.iter().map(|output| output.name()).collect()
}

fn outputs_schema() -> Arc<ArrowSchema> {
    Arc::new(ArrowSchema::new(
        Output::ALL
            .map(|output| Field::new(output.name(), DataType::Utf8, true))
            .to_vec(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Topology {
    Plain,
    One,
    Shared,
    Chain,
}

impl Topology {
    const ALL: [Self; 4] = [Self::Plain, Self::One, Self::Shared, Self::Chain];
    const FLAGGED: [Self; 3] = [Self::One, Self::Shared, Self::Chain];

    fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::One => "one",
            Self::Shared => "shared",
            Self::Chain => "chain",
        }
    }

    /// The outputs with a dependent masking flag, upstream first.
    fn flagged(self) -> &'static [Output] {
        match self {
            Self::Plain => &[],
            Self::One => &[Output::Summary],
            Self::Shared => &[Output::Summary, Output::Translation],
            Self::Chain => &[Output::Summary, Output::Keywords],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Published,
    /// Flags registered, outputs never stored.
    Pending,
    /// Published, then an in-place `body` write of this many rows per
    /// thousand; a plain table stores `summary` NULL on them instead.
    Cleared {
        per_mille: u64,
    },
    /// Published, then this many row-moving `body` updates of
    /// [`HISTORY_ROWS`] scattered rows each, left pending.
    SparseHistory(u64),
    /// Published, then one row-moving `body` update of every tenth row, left
    /// pending.
    DenseHistory,
    /// Published, then this many row-moving `body` updates of
    /// [`HISTORY_ROWS`] scattered rows each, each followed by a refresh of
    /// `summary`: a publication, or on a plain table a `merge_insert`.
    RefreshedHistory(u64),
    /// Published, then a row-moving `views` update of this many rows per
    /// thousand, which clears no flag.
    UnrelatedMoved {
        per_mille: u64,
    },
}

struct TableSpec {
    name: &'static str,
    topology: Topology,
    state: State,
}

const fn spec(name: &'static str, topology: Topology, state: State) -> TableSpec {
    TableSpec {
        name,
        topology,
        state,
    }
}

const TABLES: [TableSpec; 28] = [
    spec("plain", Topology::Plain, State::Published),
    spec("one", Topology::One, State::Published),
    spec("shared", Topology::Shared, State::Published),
    spec("chain", Topology::Chain, State::Published),
    spec(
        "partial_1pct",
        Topology::One,
        State::Cleared { per_mille: 10 },
    ),
    spec(
        "null_1pct",
        Topology::Plain,
        State::Cleared { per_mille: 10 },
    ),
    spec(
        "partial_50pct",
        Topology::One,
        State::Cleared { per_mille: 500 },
    ),
    spec(
        "null_50pct",
        Topology::Plain,
        State::Cleared { per_mille: 500 },
    ),
    spec("masked", Topology::One, State::Cleared { per_mille: 1000 }),
    spec(
        "null_all",
        Topology::Plain,
        State::Cleared { per_mille: 1000 },
    ),
    spec("pending_one", Topology::One, State::Pending),
    spec("pending_shared", Topology::Shared, State::Pending),
    spec("pending_chain", Topology::Chain, State::Pending),
    spec("plain_empty", Topology::Plain, State::Pending),
    spec("plain_h16", Topology::Plain, State::SparseHistory(16)),
    spec("one_h16", Topology::One, State::SparseHistory(16)),
    spec("plain_h64", Topology::Plain, State::SparseHistory(64)),
    spec("one_h64", Topology::One, State::SparseHistory(64)),
    spec("shared_h64", Topology::Shared, State::SparseHistory(64)),
    spec("chain_h64", Topology::Chain, State::SparseHistory(64)),
    spec("plain_dense", Topology::Plain, State::DenseHistory),
    spec("one_dense", Topology::One, State::DenseHistory),
    spec("plain_h16r", Topology::Plain, State::RefreshedHistory(16)),
    spec("one_h16r", Topology::One, State::RefreshedHistory(16)),
    spec("plain_h64r", Topology::Plain, State::RefreshedHistory(64)),
    spec("one_h64r", Topology::One, State::RefreshedHistory(64)),
    spec(
        "ready_moved",
        Topology::One,
        State::UnrelatedMoved { per_mille: 10 },
    ),
    spec(
        "plain_moved",
        Topology::Plain,
        State::UnrelatedMoved { per_mille: 10 },
    ),
];

/// Flagged tables and the plain tables that read the same visible data.
const READ_PAIRS: [(&str, &str); 5] = [
    ("one", "plain"),
    ("ready_moved", "plain_moved"),
    ("partial_1pct", "null_1pct"),
    ("partial_50pct", "null_50pct"),
    ("masked", "null_all"),
];

const HISTORY_TABLES: [&str; 16] = [
    "plain",
    "one",
    "shared",
    "chain",
    "plain_h16",
    "one_h16",
    "plain_h64",
    "one_h64",
    "shared_h64",
    "chain_h64",
    "plain_dense",
    "one_dense",
    "plain_h16r",
    "one_h16r",
    "plain_h64r",
    "one_h64r",
];

fn table_spec(name: &str) -> &'static TableSpec {
    TABLES
        .iter()
        .find(|spec| spec.name == name)
        .unwrap_or_else(|| panic!("no table {name}"))
}

fn cleared_ids(state: State, rows: u64) -> RoaringTreemap {
    match state {
        State::Cleared { per_mille: 10 } | State::UnrelatedMoved { per_mille: 10 } => {
            scattered_ids(rows, rows / 100, SEED_ONE_PERCENT)
                .into_iter()
                .collect()
        }
        State::Cleared { per_mille: 500 } => scattered_ids(rows, rows / 2, SEED_HALF)
            .into_iter()
            .collect(),
        State::Cleared { per_mille: 1000 } => (0..rows).collect(),
        State::Cleared { per_mille } | State::UnrelatedMoved { per_mille } => {
            panic!("no set of {per_mille} rows per mille")
        }
        _ => RoaringTreemap::new(),
    }
}

fn keywords_of(summary: &str, language: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in summary.bytes().chain([0xff]).chain(language.bytes()) {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    format!("kw:{hash:016x}")
}

/// `outputs` of each row of `inputs` (`id`, `title`, `body`, `language`),
/// with `summary` NULL on `null_summaries`. `keywords` is computed from the
/// `summary` computed here.
fn compute(
    inputs: &RecordBatch,
    outputs: &[Output],
    null_summaries: Option<&RoaringTreemap>,
) -> Vec<ArrayRef> {
    let rows = inputs.num_rows();
    let ids = inputs["id"].as_primitive::<Int64Type>();
    let title = inputs["title"].as_string::<i32>();
    let body = inputs["body"].as_string::<i32>();
    let language = inputs["language"].as_string::<i32>();
    let mut builders: Vec<StringBuilder> = outputs
        .iter()
        .map(|_| StringBuilder::with_capacity(rows, rows * 24))
        .collect();
    for row in 0..rows {
        let summary = UDF.summary(title.value(row), body.value(row));
        for (builder, output) in builders.iter_mut().zip(outputs) {
            match output {
                Output::Summary
                    if null_summaries
                        .is_some_and(|nulls| nulls.contains(ids.value(row) as u64)) =>
                {
                    builder.append_null()
                }
                Output::Summary => builder.append_value(&summary),
                Output::Translation => {
                    builder.append_value(UDF.translation(body.value(row), language.value(row)))
                }
                Output::Keywords => {
                    builder.append_value(keywords_of(&summary, language.value(row)))
                }
            }
        }
    }
    builders
        .into_iter()
        .map(|mut builder| Arc::new(builder.finish()) as ArrayRef)
        .collect()
}

async fn create_table(root: &Path, spec: &TableSpec, config: &BenchConfig) -> Dataset {
    let uri = root.join(spec.name);
    let total = config.scale_rows;
    let batches = (0..total)
        .step_by(GENERATE_BATCH_ROWS as usize)
        .map(move |start| {
            Ok(base_batch(
                start,
                GENERATE_BATCH_ROWS.min(total - start) as usize,
            ))
        });
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
    .expect("write the source columns");
    dataset
        .add_columns(NewColumnTransform::AllNulls(outputs_schema()), None, None)
        .await
        .expect("declare the outputs");
    let mut flag_ids = Vec::new();
    for output in spec.topology.flagged() {
        let definition = dataset
            .register_cell_flag(
                output.name(),
                FLAG,
                CellFlagOptions::default()
                    .with_clear_on_write(output.inputs())
                    .with_mask_when_false(true),
            )
            .await
            .expect("register a dependent masking flag");
        flag_ids.push(definition.flag_id);
    }
    if spec.state == State::Pending {
        return dataset;
    }

    let cleared = cleared_ids(spec.state, total);
    let stored_nulls = (spec.topology == Topology::Plain
        && matches!(spec.state, State::Cleared { .. }))
    .then_some(&cleared);
    let mut dataset = populate(dataset, &flag_ids, stored_nulls).await;
    match spec.state {
        State::Cleared { .. } => {
            let ids: Vec<u64> = cleared.iter().collect();
            let source = body_source_batch(&dataset, &ids, 1);
            dataset = merge_in_place(Arc::new(dataset), source).await.dataset;
        }
        State::SparseHistory(updates) => {
            for update in 0..updates {
                let ids = scattered_ids(total, HISTORY_ROWS, SEED_HISTORY + update);
                dataset = update_body(dataset, &id_in_predicate(&ids)).await.0;
            }
        }
        State::DenseHistory => dataset = update_body(dataset, "id % 10 = 3").await.0,
        State::RefreshedHistory(updates) => {
            for update in 0..updates {
                let ids = scattered_ids(total, HISTORY_ROWS, SEED_HISTORY + update);
                dataset = update_body(dataset, &id_in_predicate(&ids)).await.0;
                dataset = Box::pin(refresh_summaries(dataset, spec.topology, &ids)).await;
            }
        }
        State::UnrelatedMoved { .. } => {
            let ids: Vec<u64> = cleared.iter().collect();
            let predicate = id_in_predicate(&ids);
            let (moved, _) = update_rows(Arc::new(dataset), &predicate, "views", "views + 1").await;
            dataset = Arc::unwrap_or_clone(moved);
        }
        State::Published | State::Pending => {}
    }
    dataset
}

/// Store every output for every fragment with one `DataReplacement` that
/// sets each of `flag_ids` true on every row.
async fn populate(
    dataset: Dataset,
    flag_ids: &[u32],
    stored_nulls: Option<&RoaringTreemap>,
) -> Dataset {
    let target = dataset
        .schema()
        .project(&Output::ALL.map(Output::name))
        .expect("the outputs are declared");
    let mut replacements = Vec::new();
    let mut every_row = RowAddrTreeMap::new();
    for fragment in dataset.get_fragments() {
        let mut scanner = fragment.scan();
        scanner
            .project(&["id", "title", "body", "language"])
            .expect("project the inputs")
            .scan_in_order(true);
        let outputs = scanner
            .try_into_stream()
            .await
            .expect("scan the inputs")
            .map_ok(|inputs| {
                RecordBatch::try_new(
                    outputs_schema(),
                    compute(&inputs, &Output::ALL, stored_nulls),
                )
                .expect("outputs match their schema")
            });
        replacements.push(
            fragment
                .write_columns(outputs, &target)
                .await
                .expect("stage the outputs"),
        );
        every_row.insert_fragment(fragment.id() as u32);
    }
    let mut transaction = TransactionBuilder::new(
        dataset.version().version,
        Operation::DataReplacement { replacements },
    );
    if !flag_ids.is_empty() {
        transaction = transaction.cell_flag_changes(CellFlagChanges {
            updates: flag_ids
                .iter()
                .map(|flag_id| CellFlagUpdate {
                    flag_id: *flag_id,
                    value: true,
                    rows: every_row.clone(),
                })
                .collect(),
            ..Default::default()
        });
    }
    CommitBuilder::new(Arc::new(dataset))
        .execute(transaction.build())
        .await
        .expect("store the outputs")
}

/// A row-moving `body` update where `predicate` holds.
async fn update_body(dataset: Dataset, predicate: &str) -> (Dataset, u64) {
    let (dataset, rows) =
        update_rows(Arc::new(dataset), predicate, "body", "concat(body, '!')").await;
    (Arc::unwrap_or_clone(dataset), rows)
}

/// Bring `summary` of the rows just written to `ids` up to date: publish it,
/// or on a plain table rewrite it with `merge_insert`.
async fn refresh_summaries(dataset: Dataset, topology: Topology, ids: &[u64]) -> Dataset {
    let outputs = [Output::Summary];
    if topology == Topology::Plain {
        let inputs = filter_inputs(&dataset, &id_in_predicate(ids)).await;
        let source = merge_source(
            &dataset,
            &inputs,
            &outputs,
            compute(&inputs, &outputs, None),
        );
        let dataset = Arc::new(dataset);
        let uncommitted = merge_uncommitted(
            dataset.clone(),
            source,
            MergeInsertWriteMode::RewriteColumns,
        )
        .await;
        return commit_merge(dataset, uncommitted).await;
    }
    assert_eq!(
        topology.flagged(),
        outputs,
        "histories refresh `one` tables"
    );
    let pending = pending_rows(&dataset, &outputs).await;
    let inputs = read_inputs(&dataset, &pending).await;
    let computed = computed_batch(&inputs, &outputs, compute(&inputs, &outputs, None));
    let snapshot = Arc::new(dataset);
    let publication = stage(&snapshot, &outputs, vec![computed]).await;
    let result = publish(snapshot, publication, DependencyConflictPolicy::Reject)
        .await
        .expect("publish the history's refresh");
    assert_published(&result.report, &result.dataset, &outputs, pending.len());
    result.dataset
}

fn flag_id(dataset: &Dataset, output: Output) -> u32 {
    dataset
        .cell_flag(output.name(), FLAG)
        .unwrap_or_else(|| panic!("no {FLAG} flag on {}", output.name()))
        .flag_id
}

fn true_row_count(dataset: &Dataset, flag_id: u32) -> u64 {
    let true_rows = dataset
        .cell_flag_true_rows(flag_id)
        .expect("the flag is registered");
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

/// What a scan of `id` and `summary` shows: the visible summaries and a digest
/// of each row's id and summary, or its NULL.
async fn summary_digest(dataset: &Dataset) -> (u64, String) {
    let mut scanner = dataset.scan();
    scanner
        .project(&["id", "summary"])
        .expect("project")
        .scan_in_order(true);
    let mut stream = scanner.try_into_stream().await.expect("scan");
    let mut hasher = blake3::Hasher::new();
    let mut visible = 0u64;
    while let Some(batch) = stream.try_next().await.expect("batch") {
        let ids = batch["id"].as_primitive::<Int64Type>();
        let summaries = batch["summary"].as_string::<i32>();
        for row in 0..batch.num_rows() {
            hasher.update(&ids.value(row).to_le_bytes());
            if summaries.is_valid(row) {
                visible += 1;
                hasher.update(&[1]);
                hasher.update(summaries.value(row).as_bytes());
            } else {
                hasher.update(&[0]);
            }
        }
    }
    (visible, hasher.finalize().to_hex().to_string())
}

async fn create(config: &BenchConfig, root: &Path) {
    let selected = std::env::var("BENCH_TABLES").ok().map(|names| {
        names
            .split(',')
            .map(|name| table_spec(name.trim()).name)
            .collect::<Vec<_>>()
    });
    std::fs::create_dir_all(root).expect("create the root directory");
    for spec in &TABLES {
        if selected
            .as_ref()
            .is_some_and(|selected| !selected.contains(&spec.name))
        {
            continue;
        }
        let uri = root.join(spec.name);
        assert!(
            !uri.exists(),
            "{} already exists; create writes new tables only",
            uri.display()
        );
        let start = Instant::now();
        let dataset = Box::pin(create_table(root, spec, config)).await;
        let flags: Vec<String> = spec
            .topology
            .flagged()
            .iter()
            .map(|output| {
                format!(
                    "{}.{FLAG} true on {}",
                    output.name(),
                    true_row_count(&dataset, flag_id(&dataset, *output))
                )
            })
            .collect();
        println!(
            "created {} in {:.1} s: {} fragments, {} bytes, manifest {:?} B; {}",
            spec.name,
            start.elapsed().as_secs_f64(),
            dataset.get_fragments().len(),
            dir_bytes(&uri),
            version_file_sizes(&dataset).await.0,
            flags.join(", ")
        );
    }
    for (flagged, plain) in READ_PAIRS {
        let (Some(flagged_table), Some(plain_table)) = (
            open_table(root, flagged).await,
            open_table(root, plain).await,
        ) else {
            continue;
        };
        let digest = summary_digest(&flagged_table).await;
        assert_eq!(
            digest,
            summary_digest(&plain_table).await,
            "{flagged} and {plain} read different summaries"
        );
        println!(
            "{flagged} reads the same {} visible summaries as {plain} (digest {})",
            digest.0, digest.1
        );
    }
    println!("{}", tables_identity(root).await);
}

async fn open_table(root: &Path, name: &str) -> Option<Dataset> {
    let uri = root.join(name);
    if !uri.exists() {
        return None;
    }
    Some(open_fresh_session(uri.to_str().expect("utf-8 path")).await)
}

/// Every table's identity, and one `manifest_blake3` over their manifests.
async fn tables_identity(root: &Path) -> Value {
    let mut combined = blake3::Hasher::new();
    let mut identities = Map::new();
    for spec in &TABLES {
        let Some(dataset) = open_table(root, spec.name).await else {
            continue;
        };
        let mut identity = dataset_identity(&dataset).await;
        combined.update(spec.name.as_bytes());
        combined.update(b"\0");
        combined.update(
            identity["manifest_blake3"]
                .as_str()
                .expect("manifest hash")
                .as_bytes(),
        );
        combined.update(b"\n");
        let (manifest_bytes, transaction_bytes) = version_file_sizes(&dataset).await;
        let flags: Map<String, Value> = spec
            .topology
            .flagged()
            .iter()
            .map(|output| {
                let flag_id = flag_id(&dataset, *output);
                (
                    output.name().to_string(),
                    json!({"flag_id": flag_id, "true_rows": true_row_count(&dataset, flag_id)}),
                )
            })
            .collect();
        let fields = identity.as_object_mut().expect("identity is an object");
        fields.insert("fragments".into(), json!(dataset.get_fragments().len()));
        fields.insert("manifest_bytes".into(), json!(manifest_bytes));
        fields.insert("transaction_bytes".into(), json!(transaction_bytes));
        fields.insert("masking_flags".into(), Value::Object(flags));
        identities.insert(spec.name.into(), identity);
    }
    json!({
        "uri": root,
        "manifest_blake3": combined.finalize().to_hex().to_string(),
        "tables": identities,
    })
}

// ---------------------------------------------------------------------------
// Workloads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Read {
    /// Every `summary`, counting the visible ones.
    Scan,
    /// `count_rows` of `summary IS NULL`.
    FilterNull,
    /// `summary` of [`TAKE_ROWS`] scattered rows.
    Take,
}

impl Read {
    const ALL: [Self; 3] = [Self::Scan, Self::FilterNull, Self::Take];

    fn name(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::FilterNull => "filter_null",
            Self::Take => "take",
        }
    }
}

/// The source write a cycle starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// `UpdateBuilder` of `body` on [`SPARSE_ROWS`] scattered rows, which
    /// moves them to a new fragment.
    MovingSparse,
    /// `UpdateBuilder` of `body` on every tenth row.
    MovingDense,
    /// `merge_insert` of `body` on [`SPARSE_ROWS`] scattered rows, in place.
    InPlaceSparse,
}

impl Shape {
    const ALL: [Self; 3] = [Self::MovingSparse, Self::MovingDense, Self::InPlaceSparse];

    fn name(self) -> &'static str {
        match self {
            Self::MovingSparse => "moving_sparse",
            Self::MovingDense => "moving_dense",
            Self::InPlaceSparse => "inplace_sparse",
        }
    }

    fn predicate(self, ids: &[u64]) -> String {
        match self {
            Self::MovingDense => "id % 10 = 3".to_string(),
            Self::MovingSparse | Self::InPlaceSparse => id_in_predicate(ids),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Workload {
    Read {
        read: Read,
        table: &'static str,
    },
    /// A sparse write of `views`, which no flag watches.
    Unrelated {
        in_place: bool,
        table: Topology,
    },
    /// Source write, read while pending, find and read what to recompute,
    /// stage, commit, read again. On a plain table, `refresh` names the
    /// outputs rewritten with `merge_insert`.
    Cycle {
        shape: Shape,
        table: Topology,
        refresh: Topology,
        /// On a plain table, `merge_insert` rewrites whole rows
        /// (`RewriteRows`) instead of the refreshed columns.
        rewrite_rows: bool,
    },
    /// Publish every row of never-stored outputs; on `plain_empty`, write
    /// them with one `DataReplacement`.
    Backfill {
        table: Topology,
        refresh: Topology,
    },
    /// A refresh of [`CONFLICT_PENDING_ROWS`] rows raced by `commits` source
    /// writes of [`CONFLICT_ROWS_PER_COMMIT`] of them, then its retry or
    /// follow-up.
    Conflict {
        in_place: bool,
        commits: usize,
        policy: DependencyConflictPolicy,
        refresh: Topology,
    },
    /// A sparse refresh committed after `commits` unrelated appends.
    AfterAppends {
        commits: usize,
        table: Topology,
    },
    /// Open in a new `Session`.
    Reopen(&'static str),
    /// Commit an append of [`APPEND_ROWS`] rows.
    Append(&'static str),
    /// A row-moving sparse `body` update.
    Update(&'static str),
}

impl Workload {
    fn all() -> Vec<Self> {
        let mut all = Vec::new();
        for (flagged, plain) in READ_PAIRS {
            for table in [flagged, plain] {
                all.extend(Read::ALL.map(|read| Self::Read { read, table }));
            }
        }
        for in_place in [false, true] {
            all.extend(Topology::ALL.map(|table| Self::Unrelated { in_place, table }));
        }
        for shape in Shape::ALL {
            for refresh in Topology::FLAGGED {
                for table in [refresh, Topology::Plain] {
                    all.push(Self::Cycle {
                        shape,
                        table,
                        refresh,
                        rewrite_rows: false,
                    });
                }
            }
        }
        all.push(Self::Cycle {
            shape: Shape::InPlaceSparse,
            table: Topology::Plain,
            refresh: Topology::One,
            rewrite_rows: true,
        });
        for refresh in Topology::FLAGGED {
            for table in [refresh, Topology::Plain] {
                all.push(Self::Backfill { table, refresh });
            }
        }
        for (in_place, commits, refresh) in [
            (false, 1, Topology::One),
            (true, 1, Topology::One),
            (true, 8, Topology::One),
            (true, 1, Topology::Chain),
        ] {
            for policy in [
                DependencyConflictPolicy::Reject,
                DependencyConflictPolicy::Skip,
            ] {
                all.push(Self::Conflict {
                    in_place,
                    commits,
                    policy,
                    refresh,
                });
            }
        }
        for commits in [0, 8, 32] {
            for table in [Topology::One, Topology::Plain] {
                all.push(Self::AfterAppends { commits, table });
            }
        }
        for table in HISTORY_TABLES {
            all.extend([
                Self::Reopen(table),
                Self::Append(table),
                Self::Update(table),
            ]);
        }
        all
    }

    fn name(self) -> String {
        let on_plain = |table: Topology| {
            if table == Topology::Plain {
                "plain_"
            } else {
                ""
            }
        };
        match self {
            Self::Read { read, table } => format!("{}_{table}", read.name()),
            Self::Unrelated { in_place, table } => format!(
                "unrelated_{}_{}",
                if in_place { "inplace" } else { "moving" },
                table.name()
            ),
            Self::Cycle {
                shape,
                table,
                refresh,
                rewrite_rows,
            } => format!(
                "cycle_{}_{}{}{}",
                shape.name(),
                on_plain(table),
                refresh.name(),
                if rewrite_rows { "_rewrite_rows" } else { "" }
            ),
            Self::Backfill { table, refresh } => {
                format!("backfill_{}{}", on_plain(table), refresh.name())
            }
            Self::Conflict {
                in_place,
                commits,
                policy,
                refresh,
            } => format!(
                "conflict_{}_k{commits}_{}{}",
                if in_place { "inplace" } else { "moving" },
                match policy {
                    DependencyConflictPolicy::Reject => "reject",
                    DependencyConflictPolicy::Skip => "skip",
                },
                if refresh == Topology::Chain {
                    "_chain"
                } else {
                    ""
                }
            ),
            Self::AfterAppends { commits, table } => {
                format!("after_appends_k{commits}_{}one", on_plain(table))
            }
            Self::Reopen(table) => format!("reopen_{table}"),
            Self::Append(table) => format!("append_{table}"),
            Self::Update(table) => format!("update_{table}"),
        }
    }

    /// The root table the workload reads or copies.
    fn table(self) -> &'static str {
        match self {
            Self::Read { table, .. }
            | Self::Reopen(table)
            | Self::Append(table)
            | Self::Update(table) => table,
            Self::Unrelated { table, .. }
            | Self::Cycle { table, .. }
            | Self::AfterAppends { table, .. } => table.name(),
            Self::Backfill { table, refresh } => match (table, refresh) {
                (Topology::Plain, _) => "plain_empty",
                (_, Topology::One) => "pending_one",
                (_, Topology::Shared) => "pending_shared",
                (_, _) => "pending_chain",
            },
            Self::Conflict { refresh, .. } => refresh.name(),
        }
    }
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

struct Phase {
    name: &'static str,
    wall_ns: u64,
    cpu_ns: Option<u64>,
    instructions: Option<u64>,
    cycles: Option<u64>,
    allocated: Option<AllocationDelta>,
    /// Bytes live at the phase's start above those live at the sample's.
    live_offset: i64,
    /// `None` where no object store is metered (a new `Session`'s open).
    io: Option<IoDelta>,
}

/// Times the phases of one sample.
struct Recorder {
    hardware: bool,
    io: Option<IoMeter>,
    phases: Vec<Phase>,
    /// Bytes live when the sample started.
    live_bytes: i64,
}

impl Recorder {
    fn new(hardware: bool) -> Self {
        Self {
            hardware,
            io: None,
            phases: Vec::new(),
            live_bytes: allocations::live_bytes(),
        }
    }

    /// Meter the object store `dataset` uses, which the datasets it commits
    /// share.
    async fn meter(mut self, dataset: &Dataset) -> Self {
        self.io = Some(IoMeter::new(dataset).await);
        self
    }

    /// Time `future` as the phase `name`. It is boxed here, before its
    /// phase starts, since Lance's write futures are large.
    fn phase<'a, F: Future + 'a>(
        &'a mut self,
        name: &'static str,
        future: F,
    ) -> impl Future<Output = F::Output> + 'a {
        let future = Box::pin(future);
        async move {
            if let Some(io) = &self.io {
                io.reset();
            }
            let before = Counters::read(self.hardware);
            let allocations = Allocations::start();
            let start = Instant::now();
            let output = future.await;
            let wall_ns = start.elapsed().as_nanos() as u64;
            let allocated = allocations.since();
            let after = Counters::read(self.hardware);
            self.phases.push(Phase {
                name,
                wall_ns,
                cpu_ns: delta(after.cpu_ns, before.cpu_ns),
                instructions: delta(after.instructions, before.instructions),
                cycles: delta(after.cycles, before.cycles),
                allocated,
                live_offset: allocations.live_bytes() - self.live_bytes,
                io: self.io.as_ref().map(IoMeter::take),
            });
            output
        }
    }

    async fn finish(self, rows: u64, end: &Dataset, extra: Map<String, Value>) -> Sample {
        Sample {
            phases: self.phases,
            rows,
            sizes: Sizes::of(end).await,
            extra,
        }
    }
}

struct Sample {
    phases: Vec<Phase>,
    rows: u64,
    sizes: Sizes,
    extra: Map<String, Value>,
}

/// Metadata file sizes of a version.
#[derive(Debug, Clone, Copy)]
struct Sizes {
    /// The manifest file, including the inline transaction.
    manifest_bytes: u64,
    /// The manifest proper, and the file's footer.
    manifest_struct_bytes: u64,
    inline_transaction_bytes: Option<u64>,
    /// The separate transaction file.
    transaction_bytes: Option<u64>,
}

impl Sizes {
    /// Reads the manifest's footer from the local file system.
    async fn of(dataset: &Dataset) -> Self {
        let location = dataset.manifest_location();
        let path = Path::new("/").join(location.path.as_ref());
        let bytes = std::fs::read(&path).expect("read the manifest file");
        let len = bytes.len();
        // A Lance footer: the manifest struct's position, the version, "LANC".
        assert!(
            len >= 16 && bytes.ends_with(b"LANC"),
            "{} is not a manifest",
            path.display()
        );
        let position = u64::from_le_bytes(bytes[len - 16..len - 8].try_into().expect("8 bytes"));
        let manifest = dataset.manifest();
        Self {
            manifest_bytes: len as u64,
            manifest_struct_bytes: len as u64 - position,
            inline_transaction_bytes: manifest
                .transaction_section
                .map(|start| position - start as u64),
            transaction_bytes: version_file_sizes(dataset).await.1,
        }
    }

    fn insert_into(self, record: &mut Map<String, Value>) {
        record.insert("manifest_bytes".into(), json!(self.manifest_bytes));
        record.insert(
            "manifest_struct_bytes".into(),
            json!(self.manifest_struct_bytes),
        );
        record.insert(
            "inline_transaction_bytes".into(),
            json!(self.inline_transaction_bytes),
        );
        record.insert("transaction_bytes".into(), json!(self.transaction_bytes));
    }
}

fn sum(values: impl Iterator<Item = Option<u64>>) -> Option<u64> {
    values.sum()
}

/// One phase's counters, or with `is_sample` the sum of a sample's, whose peak
/// is the most bytes live during any phase above those live at its start.
fn counters_json(phases: &[&Phase], is_sample: bool) -> Map<String, Value> {
    let allocated = |f: fn(&AllocationDelta) -> u64| -> Option<u64> {
        sum(phases.iter().map(|phase| phase.allocated.as_ref().map(f)))
    };
    let io = |f: fn(&IoDelta) -> u64| -> Option<u64> {
        sum(phases.iter().map(|phase| phase.io.as_ref().map(f)))
    };
    let fields = json!({
        "wall_ns": phases.iter().map(|phase| phase.wall_ns).sum::<u64>(),
        "cpu_ns": sum(phases.iter().map(|phase| phase.cpu_ns)),
        "instructions": sum(phases.iter().map(|phase| phase.instructions)),
        "cycles": sum(phases.iter().map(|phase| phase.cycles)),
        "allocations": allocated(|delta| delta.allocations),
        "allocated_bytes": allocated(|delta| delta.allocated_bytes),
        "peak_live_growth_bytes": phases
            .iter()
            .map(|phase| {
                let offset = if is_sample { phase.live_offset } else { 0 };
                phase
                    .allocated
                    .map(|delta| (offset + delta.peak_live_growth_bytes as i64).max(0) as u64)
            })
            .collect::<Option<Vec<_>>>()
            .map(|peaks| peaks.into_iter().max().unwrap_or(0)),
        "read_iops": io(|delta| delta.read_iops),
        "read_bytes": io(|delta| delta.read_bytes),
        "write_iops": io(|delta| delta.write_iops),
        "written_bytes": io(|delta| delta.written_bytes),
    });
    match fields {
        Value::Object(fields) => fields,
        _ => unreachable!("a JSON object"),
    }
}

struct Ctx {
    rows: u64,
    root: PathBuf,
    scratch: PathBuf,
    hardware: bool,
    /// Root tables the read workloads use, each opened once.
    tables: HashMap<&'static str, Dataset>,
    sizes: HashMap<&'static str, Sizes>,
    /// Rows `Read::Take` fetches.
    take_indices: Vec<u64>,
}

/// A clone of a root table under the scratch directory, removed on drop.
struct WorkingCopy {
    dir: PathBuf,
}

impl WorkingCopy {
    fn uri(&self) -> &str {
        self.dir.to_str().expect("utf-8 path")
    }
}

impl Drop for WorkingCopy {
    fn drop(&mut self) {
        let removed = std::fs::remove_dir_all(&self.dir);
        // A second panic while a failed sample unwinds would abort before its
        // message prints.
        if let Err(error) = removed
            && !std::thread::panicking()
        {
            panic!("remove {}: {error}", self.dir.display());
        }
    }
}

impl Ctx {
    /// Clone `table` and read every file of the clone once.
    fn working_copy(&self, table: &str) -> WorkingCopy {
        let dir = self.scratch.join(table);
        assert!(
            !dir.exists(),
            "{} is left from an earlier sample",
            dir.display()
        );
        let mut command = Command::new("cp");
        if cfg!(target_os = "macos") {
            command.arg("-Rc");
        } else {
            command.args(["-R", "--reflink=auto"]);
        }
        let status = command
            .arg(self.root.join(table))
            .arg(&dir)
            .status()
            .expect("run cp");
        assert!(status.success(), "clone {table}: cp exited with {status}");
        read_every_file(&dir);
        WorkingCopy { dir }
    }
}

fn read_every_file(path: &Path) {
    for entry in std::fs::read_dir(path).expect("list the working copy") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            read_every_file(&path);
        } else {
            let mut file = std::fs::File::open(&path).expect("open a working copy file");
            std::io::copy(&mut file, &mut std::io::sink()).expect("read a working copy file");
        }
    }
}

// ---------------------------------------------------------------------------
// Refresh steps
// ---------------------------------------------------------------------------

/// The addresses of `rows`, which name no whole fragment.
fn addresses(rows: &RowAddrTreeMap) -> RoaringTreemap {
    let mut out = RoaringTreemap::new();
    for (fragment, selection) in rows.iter() {
        let RowAddrSelection::Partial(offsets) = selection else {
            panic!("fragment {fragment} is listed whole");
        };
        let base = u64::from(*fragment) << 32;
        out.extend(offsets.iter().map(|offset| base | u64::from(offset)));
    }
    out
}

/// Live rows where the flag of any of `outputs` is false: an ordered scan of
/// `_rowaddr` alone, without the flags' true rows.
async fn pending_rows(dataset: &Dataset, outputs: &[Output]) -> RoaringTreemap {
    let true_rows: Vec<RowAddrTreeMap> = outputs
        .iter()
        .map(|output| {
            dataset
                .cell_flag_true_rows(flag_id(dataset, *output))
                .expect("the flag is registered")
        })
        .collect();
    let mut scanner = dataset.scan();
    scanner
        .project::<&str>(&[])
        .expect("project nothing")
        .with_row_address()
        .scan_in_order(true);
    let mut stream = scanner.try_into_stream().await.expect("scan row addresses");
    let mut pending = RoaringTreemap::new();
    while let Some(batch) = stream.try_next().await.expect("row address batch") {
        for addr in batch[ROW_ADDR].as_primitive::<UInt64Type>().values() {
            if true_rows.iter().any(|rows| !rows.contains(*addr)) {
                pending.insert(*addr);
            }
        }
    }
    pending
}

const INPUTS: [&str; 4] = ["id", "title", "body", "language"];

/// The inputs of `rows`, with `_rowaddr`, in address order.
async fn read_inputs(dataset: &Dataset, rows: &RoaringTreemap) -> RecordBatch {
    let addrs: Vec<u64> = rows.iter().collect();
    let batch = Arc::new(dataset.clone())
        .take_builder(
            &addrs,
            ProjectionRequest::from_columns(INPUTS, dataset.schema()),
        )
        .expect("take builder")
        .with_row_address(true)
        .execute()
        .await
        .expect("take the inputs");
    assert_eq!(
        batch[ROW_ADDR]
            .as_primitive::<UInt64Type>()
            .values()
            .as_ref(),
        addrs.as_slice(),
        "take returns rows in address order"
    );
    batch
}

/// The inputs of every row, with `_rowaddr`, from one ordered scan.
async fn scan_inputs(dataset: &Dataset) -> Vec<RecordBatch> {
    let mut scanner = dataset.scan();
    scanner
        .project(&INPUTS)
        .expect("project the inputs")
        .with_row_address()
        .scan_in_order(true);
    scanner
        .try_into_stream()
        .await
        .expect("scan the inputs")
        .try_collect()
        .await
        .expect("read the inputs")
}

/// The inputs where `predicate` holds, as a caller without flags finds the
/// rows it wrote.
async fn filter_inputs(dataset: &Dataset, predicate: &str) -> RecordBatch {
    let mut scanner = dataset.scan();
    scanner
        .project(&INPUTS)
        .expect("project the inputs")
        .filter(predicate)
        .expect("filter");
    scanner.try_into_batch().await.expect("read the inputs")
}

/// `_rowaddr` and the computed `outputs` of `inputs`.
fn computed_batch(inputs: &RecordBatch, outputs: &[Output], values: Vec<ArrayRef>) -> RecordBatch {
    let columns = std::iter::once((ROW_ADDR, inputs[ROW_ADDR].clone()))
        .chain(names(outputs).into_iter().zip(values));
    RecordBatch::try_from_iter(columns).expect("computed batch")
}

/// An `id` and `outputs` source for `merge_insert`.
fn merge_source(
    dataset: &Dataset,
    inputs: &RecordBatch,
    outputs: &[Output],
    values: Vec<ArrayRef>,
) -> RecordBatch {
    let full = ArrowSchema::from(dataset.schema());
    let positions: Vec<usize> = std::iter::once("id")
        .chain(names(outputs))
        .map(|name| full.index_of(name).expect("source column exists"))
        .collect();
    let schema = Arc::new(full.project(&positions).expect("project the merge source"));
    let columns = std::iter::once(inputs["id"].clone())
        .chain(values)
        .collect();
    RecordBatch::try_new(schema, columns).expect("merge source")
}

async fn stage(
    snapshot: &Arc<Dataset>,
    outputs: &[Output],
    computed: Vec<RecordBatch>,
) -> Transaction {
    PublicationStager::try_new(snapshot.clone(), &names(outputs))
        .expect("stager")
        .stage(stream::iter(
            computed.into_iter().map(Ok::<_, lance::Error>),
        ))
        .await
        .expect("stage the publication")
        .expect("the publication assigns rows")
}

async fn publish(
    snapshot: Arc<Dataset>,
    publication: Transaction,
    policy: DependencyConflictPolicy,
) -> lance::Result<PublicationResult> {
    CommitBuilder::new(snapshot)
        .with_dependency_conflict_policy(policy)
        .execute_with_report(publication)
        .await
}

/// Assert that `report` published `rows` rows of each of `outputs` and
/// deferred nothing.
fn assert_published(report: &PublicationReport, dataset: &Dataset, outputs: &[Output], rows: u64) {
    assert!(
        report.deferred_rows.is_empty() && report.deferred_groups.is_empty(),
        "an unraced publication defers nothing: {report:?}"
    );
    for output in outputs {
        let published = report.published_rows(flag_id(dataset, *output));
        let count: u64 = published
            .iter()
            .map(|(_, selection)| match selection {
                RowAddrSelection::Partial(rows) => rows.len(),
                RowAddrSelection::Full => panic!("a publication assigns rows"),
            })
            .sum();
        assert_eq!(count, rows, "{} rows published", output.name());
    }
}

async fn merge_uncommitted(
    dataset: Arc<Dataset>,
    source: RecordBatch,
    mode: MergeInsertWriteMode,
) -> UncommittedMergeInsert {
    MergeInsertBuilder::try_new(dataset, vec!["id".to_string()])
        .expect("merge insert builder")
        .when_matched(WhenMatched::UpdateAll)
        .when_not_matched(WhenNotMatched::DoNothing)
        .write_mode(mode)
        .try_build()
        .expect("build merge insert")
        .execute_uncommitted_batches(vec![source])
        .await
        .expect("execute merge insert")
}

/// Commit a merge as `MergeInsertJob::execute` does.
async fn commit_merge(dataset: Arc<Dataset>, uncommitted: UncommittedMergeInsert) -> Dataset {
    let mut builder = CommitBuilder::new(dataset);
    if let Some(affected_rows) = uncommitted.affected_rows {
        builder = builder.with_affected_rows(affected_rows);
    }
    builder
        .execute(uncommitted.transaction)
        .await
        .expect("commit merge insert")
}

/// Visible values of each of `outputs`.
async fn count_visible(dataset: &Dataset, outputs: &[Output]) -> Vec<u64> {
    let mut scanner = dataset.scan();
    scanner
        .project(&names(outputs))
        .expect("project the outputs");
    let mut stream = scanner.try_into_stream().await.expect("scan the outputs");
    let mut visible = vec![0; outputs.len()];
    while let Some(batch) = stream.try_next().await.expect("output batch") {
        for (count, column) in visible.iter_mut().zip(batch.columns()) {
            *count += (column.len() - column.null_count()) as u64;
        }
    }
    visible
}

/// Assert that the `outputs` of `ids` hold what their current inputs compute.
async fn check_values(dataset: &Dataset, ids: &[u64], outputs: &[Output]) {
    let mut scanner = dataset.scan();
    let columns: Vec<&str> = INPUTS.into_iter().chain(names(outputs)).collect();
    scanner
        .project(&columns)
        .expect("project")
        .filter(&id_in_predicate(ids))
        .expect("filter");
    let batch = scanner
        .try_into_batch()
        .await
        .expect("read the checked rows");
    assert_eq!(batch.num_rows(), ids.len(), "checked rows");
    for (output, expected) in outputs.iter().zip(compute(&batch, outputs, None)) {
        assert_eq!(
            batch[output.name()].as_ref(),
            expected.as_ref(),
            "{} holds values computed from other inputs",
            output.name()
        );
    }
}

// ---------------------------------------------------------------------------
// Samples
// ---------------------------------------------------------------------------

async fn read_sample(ctx: &Ctx, read: Read, table: &'static str) -> Sample {
    let dataset = &ctx.tables[table];
    let mut recorder = Recorder::new(ctx.hardware).meter(dataset).await;
    let rows = recorder
        .phase("read", async {
            match read {
                Read::Scan => count_visible(dataset, &[Output::Summary]).await[0],
                Read::FilterNull => {
                    let mut scanner = dataset.scan();
                    scanner.filter("summary IS NULL").expect("filter");
                    scanner.count_rows().await.expect("count")
                }
                Read::Take => {
                    let batch = dataset
                        .take(
                            &ctx.take_indices,
                            ProjectionRequest::from_columns(["summary"], dataset.schema()),
                        )
                        .await
                        .expect("take");
                    let summary = &batch["summary"];
                    (summary.len() - summary.null_count()) as u64
                }
            }
        })
        .await;
    Sample {
        phases: recorder.phases,
        rows,
        sizes: ctx.sizes[table],
        extra: Map::new(),
    }
}

async fn unrelated_sample(ctx: &Ctx, in_place: bool, table: Topology) -> Sample {
    let copy = ctx.working_copy(table.name());
    let dataset = open_fresh_session(copy.uri()).await;
    let mut recorder = Recorder::new(ctx.hardware).meter(&dataset).await;
    let ids = scattered_ids(ctx.rows, SPARSE_ROWS, SEED_SPARSE);
    let (dataset, rows) = if in_place {
        let schema = Arc::new(
            ArrowSchema::from(dataset.schema())
                .project(&[0, 4])
                .expect("project id and views"),
        );
        assert_eq!(schema.field(1).name(), "views");
        let source = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from_iter_values(
                    ids.iter().map(|id| *id as i64),
                )),
                Arc::new(Int64Array::from_iter_values(
                    ids.iter().map(|id| views_for(*id) + 1),
                )),
            ],
        )
        .expect("views source");
        let outcome = recorder
            .phase("update", merge_in_place(Arc::new(dataset), source))
            .await;
        (outcome.dataset, outcome.stats.num_updated_rows)
    } else {
        let predicate = id_in_predicate(&ids);
        let (dataset, rows) = recorder
            .phase(
                "update",
                update_rows(Arc::new(dataset), &predicate, "views", "views + 1"),
            )
            .await;
        (Arc::unwrap_or_clone(dataset), rows)
    };
    assert_eq!(rows, SPARSE_ROWS);
    assert!(
        pending_rows(&dataset, table.flagged()).await.is_empty(),
        "a write of views clears no flag"
    );
    recorder.finish(rows, &dataset, Map::new()).await
}

/// The cycle's source write; returns the rows written.
async fn source_write(dataset: Dataset, shape: Shape, ids: &[u64]) -> (Dataset, u64) {
    match shape {
        Shape::MovingSparse | Shape::MovingDense => {
            update_body(dataset, &shape.predicate(ids)).await
        }
        Shape::InPlaceSparse => {
            let source = body_source_batch(&dataset, ids, 1);
            let outcome = merge_in_place(Arc::new(dataset), source).await;
            (outcome.dataset, outcome.stats.num_updated_rows)
        }
    }
}

async fn cycle_sample(
    ctx: &Ctx,
    shape: Shape,
    table: Topology,
    refresh: Topology,
    rewrite_rows: bool,
) -> Sample {
    let copy = ctx.working_copy(table.name());
    let dataset = open_fresh_session(copy.uri()).await;
    let mut recorder = Recorder::new(ctx.hardware).meter(&dataset).await;
    let outputs = refresh.flagged();
    let is_flagged = table != Topology::Plain;
    let ids = scattered_ids(ctx.rows, SPARSE_ROWS, SEED_SPARSE);

    let (dataset, written) = recorder
        .phase("update", source_write(dataset, shape, &ids))
        .await;
    let visible = recorder
        .phase("read_pending", count_visible(&dataset, outputs))
        .await;
    let expected = if is_flagged {
        ctx.rows - written
    } else {
        ctx.rows
    };
    assert!(
        visible.iter().all(|count| *count == expected),
        "{visible:?}"
    );

    let dataset = if is_flagged {
        let pending = recorder
            .phase("find", pending_rows(&dataset, outputs))
            .await;
        assert_eq!(pending.len(), written, "rows pending after the write");
        let inputs = recorder
            .phase("read_inputs", read_inputs(&dataset, &pending))
            .await;
        let computed = computed_batch(&inputs, outputs, compute(&inputs, outputs, None));
        let snapshot = Arc::new(dataset);
        let publication = recorder
            .phase("stage", stage(&snapshot, outputs, vec![computed]))
            .await;
        let result = recorder
            .phase(
                "commit",
                publish(snapshot, publication, DependencyConflictPolicy::Reject),
            )
            .await
            .expect("publish");
        assert_published(&result.report, &result.dataset, outputs, written);
        result.dataset
    } else {
        let inputs = recorder
            .phase("locate", filter_inputs(&dataset, &shape.predicate(&ids)))
            .await;
        assert_eq!(inputs.num_rows() as u64, written, "rows the write changed");
        let source = merge_source(&dataset, &inputs, outputs, compute(&inputs, outputs, None));
        let mode = if rewrite_rows {
            MergeInsertWriteMode::RewriteRows
        } else {
            MergeInsertWriteMode::RewriteColumns
        };
        let dataset = Arc::new(dataset);
        let uncommitted = recorder
            .phase("merge", merge_uncommitted(dataset.clone(), source, mode))
            .await;
        recorder
            .phase("commit", commit_merge(dataset, uncommitted))
            .await
    };
    let visible = recorder
        .phase("read_published", count_visible(&dataset, outputs))
        .await;
    assert!(
        visible.iter().all(|count| *count == ctx.rows),
        "{visible:?}"
    );
    check_values(&dataset, &ids[..5], outputs).await;
    recorder.finish(written, &dataset, Map::new()).await
}

async fn backfill_sample(ctx: &Ctx, table: Topology, refresh: Topology) -> Sample {
    let copy = ctx.working_copy(Workload::Backfill { table, refresh }.table());
    let dataset = open_fresh_session(copy.uri()).await;
    let outputs = refresh.flagged();
    let is_flagged = table != Topology::Plain;
    // Every row is pending, so a backfill needs no scan to find them.
    if is_flagged {
        assert_eq!(pending_rows(&dataset, outputs).await.len(), ctx.rows);
    }
    let mut recorder = Recorder::new(ctx.hardware).meter(&dataset).await;

    let inputs = recorder.phase("read_inputs", scan_inputs(&dataset)).await;
    let dataset = if is_flagged {
        let computed = inputs
            .iter()
            .map(|batch| computed_batch(batch, outputs, compute(batch, outputs, None)))
            .collect();
        let snapshot = Arc::new(dataset);
        let publication = recorder
            .phase("stage", stage(&snapshot, outputs, computed))
            .await;
        let result = recorder
            .phase(
                "commit",
                publish(snapshot, publication, DependencyConflictPolicy::Reject),
            )
            .await
            .expect("publish");
        result.dataset
    } else {
        let target_arrow = Arc::new(ArrowSchema::new(
            names(outputs)
                .into_iter()
                .map(|name| Field::new(name, DataType::Utf8, true))
                .collect::<Vec<_>>(),
        ));
        let mut by_fragment: Vec<(u32, Vec<RecordBatch>)> = Vec::new();
        for batch in &inputs {
            let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
            let fragment = (addrs.value(0) >> 32) as u32;
            assert!(
                addrs
                    .values()
                    .iter()
                    .all(|addr| (addr >> 32) as u32 == fragment),
                "a scan batch holds one fragment's rows"
            );
            let values = RecordBatch::try_new(target_arrow.clone(), compute(batch, outputs, None))
                .expect("outputs match their schema");
            match by_fragment.last_mut() {
                Some((last, batches)) if *last == fragment => batches.push(values),
                _ => by_fragment.push((fragment, vec![values])),
            }
        }
        let target = dataset
            .schema()
            .project(&names(outputs))
            .expect("the outputs are declared");
        let replacements = recorder
            .phase("stage", async {
                let mut replacements = Vec::with_capacity(by_fragment.len());
                for (fragment, batches) in by_fragment {
                    let fragment = dataset
                        .get_fragment(fragment as usize)
                        .expect("a scanned fragment");
                    replacements.push(
                        fragment
                            .write_columns(stream::iter(batches.into_iter().map(Ok)), &target)
                            .await
                            .expect("stage the outputs"),
                    );
                }
                replacements
            })
            .await;
        let transaction = TransactionBuilder::new(
            dataset.version().version,
            Operation::DataReplacement { replacements },
        )
        .build();
        recorder
            .phase(
                "commit",
                CommitBuilder::new(Arc::new(dataset)).execute(transaction),
            )
            .await
            .expect("commit the outputs")
    };
    let visible = recorder
        .phase("read_published", count_visible(&dataset, outputs))
        .await;
    assert!(
        visible.iter().all(|count| *count == ctx.rows),
        "{visible:?}"
    );
    recorder.finish(ctx.rows, &dataset, Map::new()).await
}

/// A sparse source write of `ids`; `generation` makes each write's values new.
async fn write_ids(dataset: Dataset, in_place: bool, ids: &[u64], generation: u64) -> Dataset {
    if in_place {
        let source = body_source_batch(&dataset, ids, generation);
        merge_in_place(Arc::new(dataset), source).await.dataset
    } else {
        update_body(dataset, &id_in_predicate(ids)).await.0
    }
}

async fn conflict_sample(
    ctx: &Ctx,
    in_place: bool,
    commits: usize,
    policy: DependencyConflictPolicy,
    refresh: Topology,
) -> Sample {
    let copy = ctx.working_copy(refresh.name());
    let dataset = open_fresh_session(copy.uri()).await;
    let outputs = refresh.flagged();
    let pending_ids = scattered_ids(ctx.rows, CONFLICT_PENDING_ROWS, SEED_CONFLICT);
    let snapshot = Arc::new(write_ids(dataset, in_place, &pending_ids, 1).await);
    let mut recorder = Recorder::new(ctx.hardware).meter(&snapshot).await;

    let pending = recorder
        .phase("find", pending_rows(&snapshot, outputs))
        .await;
    assert_eq!(pending.len(), CONFLICT_PENDING_ROWS);
    let inputs = recorder
        .phase("read_inputs", read_inputs(&snapshot, &pending))
        .await;
    let computed = computed_batch(&inputs, outputs, compute(&inputs, outputs, None));
    let publication = recorder
        .phase("stage", stage(&snapshot, outputs, vec![computed]))
        .await;

    let groups = round_robin_groups(
        &pending_ids,
        (CONFLICT_PENDING_ROWS / CONFLICT_ROWS_PER_COMMIT) as usize,
    );
    let mut head = (*snapshot).clone();
    for (commit, ids) in groups.iter().take(commits).enumerate() {
        head = write_ids(head, in_place, ids, 2 + commit as u64).await;
    }
    let head_version = head.version().version;
    drop(head);
    let written: Vec<u64> = groups.iter().take(commits).flatten().copied().collect();

    let published = recorder
        .phase("publish", publish(snapshot.clone(), publication, policy))
        .await;
    let mut recomputed = pending.len();
    let mut extra = Map::new();
    let end = match policy {
        DependencyConflictPolicy::Reject => {
            assert!(
                matches!(published, Err(lance::Error::RetryableCommitConflict { .. })),
                "a raced publication under Reject fails as retryable: {published:?}"
            );
            // A retry refreshes everything pending at the head it loads.
            let head = recorder
                .phase("retry_checkout", async {
                    let mut latest = (*snapshot).clone();
                    latest.checkout_latest().await.expect("load the head");
                    latest
                })
                .await;
            assert_eq!(head.version().version, head_version);
            let head = Arc::new(head);
            let pending = recorder
                .phase("retry_find", pending_rows(&head, outputs))
                .await;
            let inputs = recorder
                .phase("retry_read_inputs", read_inputs(&head, &pending))
                .await;
            recomputed += pending.len();
            let computed = computed_batch(&inputs, outputs, compute(&inputs, outputs, None));
            let publication = recorder
                .phase("retry_stage", stage(&head, outputs, vec![computed]))
                .await;
            let result = recorder
                .phase(
                    "retry_commit",
                    publish(head, publication, DependencyConflictPolicy::Reject),
                )
                .await
                .expect("the retry publishes");
            assert_published(&result.report, &result.dataset, outputs, pending.len());
            result.dataset
        }
        DependencyConflictPolicy::Skip => {
            let result = published.expect("Skip publishes the safe part");
            let report = &result.report;
            let mut deferred: HashMap<String, RoaringTreemap> = HashMap::new();
            for rows in &report.deferred_rows {
                *deferred.entry(format!("{:?}", rows.reason)).or_default() |= addresses(&rows.rows);
            }
            let is_vacated = deferred.contains_key(&format!("{:?}", DeferralReason::RowVacated));
            let deferred: Map<String, Value> = deferred
                .into_iter()
                .map(|(reason, rows)| (reason, json!(rows.len())))
                .collect();
            extra.insert("rows_deferred".into(), Value::Object(deferred));
            extra.insert(
                "groups_deferred".into(),
                json!(report.deferred_groups.len()),
            );
            let follow = Arc::new(result.dataset.clone());
            assert_eq!(
                follow.version().version,
                report.committed_version.unwrap_or(report.checked_version)
            );
            let plan = recorder
                .phase("follow_up_plan", async {
                    PublicationStager::try_new(follow.clone(), &names(outputs))
                        .expect("stager")
                        .follow_up(report)
                        .await
                        .expect("plan the follow-up")
                })
                .await;
            // These races defer rows, never whole groups, and the published
            // rows are true, so nothing staged is left to reuse.
            let mut planned = RoaringTreemap::new();
            for output in outputs {
                let rows = plan.rows(output.name()).expect("a staged output");
                assert!(rows.reuse.is_empty(), "{} reuses rows", output.name());
                planned |= addresses(&rows.recompute);
            }
            // The plan lists what to recompute in place; rows a race moved
            // are pending at new addresses, which only a scan finds.
            let pending = if is_vacated {
                recorder
                    .phase("follow_up_find", pending_rows(&follow, outputs))
                    .await
            } else {
                planned.clone()
            };
            assert_eq!(pending, pending_rows(&follow, outputs).await);
            assert!(planned.is_subset(&pending));
            extra.insert("rows_reused".into(), json!(0));
            recomputed += pending.len();
            let inputs = recorder
                .phase("follow_up_read_inputs", read_inputs(&follow, &pending))
                .await;
            let computed = computed_batch(&inputs, outputs, compute(&inputs, outputs, None));
            let publication = recorder
                .phase("follow_up_stage", stage(&follow, outputs, vec![computed]))
                .await;
            let result = recorder
                .phase(
                    "follow_up_commit",
                    publish(follow, publication, DependencyConflictPolicy::Reject),
                )
                .await
                .expect("the follow-up publishes");
            assert_published(&result.report, &result.dataset, outputs, pending.len());
            result.dataset
        }
    };
    assert!(
        pending_rows(&end, outputs).await.is_empty(),
        "the refresh completes every row"
    );
    check_values(&end, &written, outputs).await;
    extra.insert("commits".into(), json!(commits));
    recorder.finish(recomputed, &end, extra).await
}

/// `len` new rows from id `start`, outputs NULL.
fn appended_batch(dataset: &Dataset, start: u64, len: u64) -> RecordBatch {
    let base = base_batch(start, len as usize);
    let columns = base
        .columns()
        .iter()
        .cloned()
        .chain(Output::ALL.map(|_| new_null_array(&DataType::Utf8, len as usize)))
        .collect();
    RecordBatch::try_new(Arc::new(ArrowSchema::from(dataset.schema())), columns)
        .expect("appended rows match the table")
}

async fn append(dataset: Dataset, start: u64) -> Dataset {
    let params = WriteParams {
        mode: WriteMode::Append,
        skip_auto_cleanup: true,
        ..Default::default()
    };
    let batch = appended_batch(&dataset, start, APPEND_ROWS);
    let dataset = Arc::new(dataset);
    let transaction = InsertBuilder::new(dataset.clone())
        .with_params(&params)
        .execute_uncommitted(vec![batch])
        .await
        .expect("write the appended fragment");
    CommitBuilder::new(dataset)
        .execute(transaction)
        .await
        .expect("commit the append")
}

async fn after_appends_sample(ctx: &Ctx, commits: usize, table: Topology) -> Sample {
    let copy = ctx.working_copy(table.name());
    let dataset = open_fresh_session(copy.uri()).await;
    let outputs = Topology::One.flagged();
    let ids = scattered_ids(ctx.rows, SPARSE_ROWS, SEED_SPARSE);
    let snapshot = Arc::new(write_ids(dataset, true, &ids, 1).await);
    let mut recorder = Recorder::new(ctx.hardware).meter(&snapshot).await;

    enum Staged {
        Publication(Transaction),
        Merge(UncommittedMergeInsert),
    }
    let staged = if table == Topology::Plain {
        let inputs = recorder
            .phase("locate", filter_inputs(&snapshot, &id_in_predicate(&ids)))
            .await;
        let source = merge_source(&snapshot, &inputs, outputs, compute(&inputs, outputs, None));
        Staged::Merge(
            recorder
                .phase(
                    "merge",
                    merge_uncommitted(
                        snapshot.clone(),
                        source,
                        MergeInsertWriteMode::RewriteColumns,
                    ),
                )
                .await,
        )
    } else {
        let pending = recorder
            .phase("find", pending_rows(&snapshot, outputs))
            .await;
        assert_eq!(pending.len(), SPARSE_ROWS);
        let inputs = recorder
            .phase("read_inputs", read_inputs(&snapshot, &pending))
            .await;
        let computed = computed_batch(&inputs, outputs, compute(&inputs, outputs, None));
        Staged::Publication(
            recorder
                .phase("stage", stage(&snapshot, outputs, vec![computed]))
                .await,
        )
    };
    let mut head = (*snapshot).clone();
    for commit in 0..commits as u64 {
        head = append(head, ctx.rows + commit * APPEND_ROWS).await;
    }
    let end = match staged {
        Staged::Publication(publication) => {
            let result = recorder
                .phase(
                    "commit",
                    publish(snapshot, publication, DependencyConflictPolicy::Reject),
                )
                .await
                .expect("publish over unrelated appends");
            assert_published(&result.report, &result.dataset, outputs, SPARSE_ROWS);
            result.dataset
        }
        Staged::Merge(uncommitted) => {
            recorder
                .phase("commit", commit_merge(snapshot, uncommitted))
                .await
        }
    };
    assert_eq!(
        end.version().version,
        head.version().version + 1,
        "the commit lands on the appends"
    );
    check_values(&end, &ids[..5], outputs).await;
    let mut extra = Map::new();
    extra.insert("commits".into(), json!(commits));
    recorder.finish(SPARSE_ROWS, &end, extra).await
}

async fn reopen_sample(ctx: &Ctx, table: &'static str) -> Sample {
    let uri = ctx.root.join(table);
    let mut recorder = Recorder::new(ctx.hardware);
    let dataset = recorder
        .phase(
            "open",
            open_fresh_session(uri.to_str().expect("utf-8 path")),
        )
        .await;
    let rows = dataset.count_rows(None).await.expect("count rows") as u64;
    Sample {
        phases: recorder.phases,
        rows,
        sizes: ctx.sizes[table],
        extra: Map::new(),
    }
}

async fn append_sample(ctx: &Ctx, table: &'static str) -> Sample {
    let copy = ctx.working_copy(table);
    let dataset = Arc::new(open_fresh_session(copy.uri()).await);
    let mut recorder = Recorder::new(ctx.hardware).meter(&dataset).await;
    let params = WriteParams {
        mode: WriteMode::Append,
        skip_auto_cleanup: true,
        ..Default::default()
    };
    let batch = appended_batch(&dataset, ctx.rows, APPEND_ROWS);
    let transaction = recorder
        .phase(
            "stage",
            InsertBuilder::new(dataset.clone())
                .with_params(&params)
                .execute_uncommitted(vec![batch]),
        )
        .await
        .expect("write the appended fragment");
    let end = recorder
        .phase("commit", CommitBuilder::new(dataset).execute(transaction))
        .await
        .expect("commit the append");
    recorder.finish(APPEND_ROWS, &end, Map::new()).await
}

async fn update_sample(ctx: &Ctx, table: &'static str) -> Sample {
    let copy = ctx.working_copy(table);
    let dataset = open_fresh_session(copy.uri()).await;
    let mut recorder = Recorder::new(ctx.hardware).meter(&dataset).await;
    let ids = scattered_ids(ctx.rows, SPARSE_ROWS, SEED_AFTER_HISTORY);
    let (end, rows) = recorder
        .phase("update", update_body(dataset, &id_in_predicate(&ids)))
        .await;
    assert_eq!(rows, SPARSE_ROWS);
    recorder.finish(rows, &end, Map::new()).await
}

async fn run_workload(ctx: &Ctx, workload: Workload) -> Sample {
    match workload {
        Workload::Read { read, table } => read_sample(ctx, read, table).await,
        Workload::Unrelated { in_place, table } => unrelated_sample(ctx, in_place, table).await,
        Workload::Cycle {
            shape,
            table,
            refresh,
            rewrite_rows,
        } => Box::pin(cycle_sample(ctx, shape, table, refresh, rewrite_rows)).await,
        Workload::Backfill { table, refresh } => {
            Box::pin(backfill_sample(ctx, table, refresh)).await
        }
        Workload::Conflict {
            in_place,
            commits,
            policy,
            refresh,
        } => Box::pin(conflict_sample(ctx, in_place, commits, policy, refresh)).await,
        Workload::AfterAppends { commits, table } => {
            Box::pin(after_appends_sample(ctx, commits, table)).await
        }
        Workload::Reopen(table) => reopen_sample(ctx, table).await,
        Workload::Append(table) => append_sample(ctx, table).await,
        Workload::Update(table) => Box::pin(update_sample(ctx, table)).await,
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    allocations::count_from_env();
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
    match std::env::var("BENCH_COUNTERS_MODE").as_deref() {
        Ok("list") => {
            for workload in Workload::all() {
                println!("{}", workload.name());
            }
        }
        Ok("create") => {
            Box::pin(create(
                &config,
                Path::new(&required_env("BENCH_COUNTERS_URI")),
            ))
            .await
        }
        Ok("measure") => {
            Box::pin(measure(
                &config,
                Path::new(&required_env("BENCH_COUNTERS_URI")),
            ))
            .await
        }
        other => panic!("BENCH_COUNTERS_MODE must be list, create or measure, got {other:?}"),
    }
}

/// The workloads `BENCH_WORKLOADS` names exactly, in `Workload::all` order.
fn selected_workloads(config: &BenchConfig) -> Vec<(String, Workload)> {
    let all: Vec<(String, Workload)> = Workload::all()
        .into_iter()
        .map(|workload| (workload.name(), workload))
        .collect();
    for token in &config.workloads {
        assert!(
            token == "all" || all.iter().any(|(name, _)| name == token),
            "BENCH_WORKLOADS names {token}, which is no workload; list mode prints them"
        );
    }
    let selected: Vec<(String, Workload)> = all
        .into_iter()
        .filter(|(name, _)| {
            config.workloads.is_empty()
                || config
                    .workloads
                    .iter()
                    .any(|token| token == "all" || token == name)
        })
        .collect();
    assert!(!selected.is_empty(), "BENCH_WORKLOADS selects no workload");
    selected
}

async fn measure(config: &BenchConfig, root: &Path) {
    let build = required_env("BENCH_BUILD");
    let out_path = required_env("BENCH_OUT");
    let workloads = selected_workloads(config);

    let identity = tables_identity(root).await;
    if let Ok(expected) = std::env::var("BENCH_COUNTERS_EXPECT_MANIFEST_BLAKE3") {
        assert_eq!(
            identity["manifest_blake3"].as_str(),
            Some(expected.as_str()),
            "the tables under {} are not the expected ones",
            root.display()
        );
    }
    let mut tables = HashMap::new();
    let mut sizes = HashMap::new();
    for (_, workload) in &workloads {
        let table = workload.table();
        if sizes.contains_key(table) {
            continue;
        }
        let dataset = open_table(root, table)
            .await
            .unwrap_or_else(|| panic!("{} has no table {table}", root.display()));
        assert_eq!(
            dataset.count_rows(None).await.expect("count rows") as u64,
            config.scale_rows,
            "{table} holds BENCH_SCALE_ROWS rows"
        );
        sizes.insert(table, Sizes::of(&dataset).await);
        if matches!(workload, Workload::Read { .. }) {
            tables.insert(table, dataset);
        }
    }
    let base = match std::env::var("BENCH_SCRATCH_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            let name = root.file_name().expect("the root has a name");
            root.with_file_name(format!("{}-scratch", name.to_string_lossy()))
        }
    };
    let scratch = base.join(std::process::id().to_string());
    // Only a killed process that had this pid can have left it.
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).expect("remove a stale scratch directory");
    }
    std::fs::create_dir_all(&scratch).expect("create the scratch directory");
    let hardware = hardware_counters_advance();
    let ctx = Ctx {
        rows: config.scale_rows,
        root: root.to_path_buf(),
        scratch: scratch.clone(),
        hardware,
        tables,
        sizes,
        take_indices: scattered_ids(
            config.scale_rows,
            TAKE_ROWS.min(config.scale_rows),
            SEED_TAKE,
        ),
    };

    if let Some(parent) = Path::new(&out_path).parent() {
        std::fs::create_dir_all(parent).expect("create the BENCH_OUT directory");
    }
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&out_path)
        .expect("open BENCH_OUT");
    let allocation_source = allocations::is_counting().then_some(allocations::SOURCE);
    let io_source = "ObjectStore::io_stats_incremental (lance-io test-util)";
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
        "workloads": workloads.iter().map(|(name, _)| name).collect::<Vec<_>>(),
        "order": "rotated by one position per pass, reversed on odd passes",
        "working_copies": {
            "scratch": scratch,
            "method": if cfg!(target_os = "macos") { "cp -Rc" } else { "cp -R --reflink=auto" },
            "then": "every file read once; opened in a new Session",
        },
        "constants": {
            "sparse_rows": SPARSE_ROWS,
            "conflict_pending_rows": CONFLICT_PENDING_ROWS,
            "conflict_rows_per_commit": CONFLICT_ROWS_PER_COMMIT,
            "append_rows": APPEND_ROWS,
            "take_rows": TAKE_ROWS,
            "history_rows": HISTORY_ROWS,
        },
        "counter_sources": {
            "wall_ns": "std::time::Instant",
            "cpu_ns": CPU_NS_SOURCE,
            "instructions": if hardware { HARDWARE_SOURCE } else { None },
            "cycles": if hardware { HARDWARE_SOURCE } else { None },
            "allocations": allocation_source,
            "allocated_bytes": allocation_source,
            "peak_live_growth_bytes": allocation_source,
            "read_iops": io_source,
            "read_bytes": io_source,
            "write_iops": io_source,
            "written_bytes": io_source,
            "manifest_bytes": "the manifest file",
            "manifest_struct_bytes": "the manifest file from the footer's struct position",
            "inline_transaction_bytes": "the manifest file's transaction section",
            "transaction_bytes": "the separate transaction file",
        },
    });
    writeln!(out, "{run}").expect("write run record");

    let count = workloads.len();
    for pass in 0..config.warmup + config.read_samples {
        for position in 0..count {
            let index = if pass % 2 == 0 {
                (pass + position) % count
            } else {
                (pass + count - 1 - position) % count
            };
            let (name, workload) = &workloads[index];
            let sample = run_workload(&ctx, *workload).await;
            if pass < config.warmup {
                continue;
            }
            let phases: Vec<&Phase> = sample.phases.iter().collect();
            let mut record = counters_json(&phases, true);
            let by_phase: Map<String, Value> = sample
                .phases
                .iter()
                .map(|phase| {
                    (
                        phase.name.to_string(),
                        Value::Object(counters_json(&[phase], false)),
                    )
                })
                .collect();
            record.insert("record".into(), json!("sample"));
            record.insert("schema".into(), json!(SCHEMA));
            record.insert("build".into(), json!(build));
            record.insert("workload".into(), json!(name));
            record.insert("table".into(), json!(workload.table()));
            record.insert("sample".into(), json!(pass - config.warmup));
            record.insert("position".into(), json!(position));
            record.insert("rows".into(), json!(sample.rows));
            sample.sizes.insert_into(&mut record);
            record.insert(
                "phase_order".into(),
                json!(
                    sample
                        .phases
                        .iter()
                        .map(|phase| phase.name)
                        .collect::<Vec<_>>()
                ),
            );
            record.insert("phases".into(), Value::Object(by_phase));
            record.insert("extra".into(), Value::Object(sample.extra));
            writeln!(out, "{}", Value::Object(record)).expect("write sample");
        }
    }
    std::fs::remove_dir(&scratch).expect("every working copy is removed");
}
