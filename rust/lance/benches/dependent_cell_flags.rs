// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Benchmark of the dependency-aware cell-flag prototype on the same
//! "articles" table as `cell_flags_regression`, with dependent masking flags
//! registered: `summary.ready` (cleared by `title`, `body`) and, for
//! `groups=2`, `translation.ready` (cleared by `body`, `language`).
//!
//! It measures what flags cost on ordinary writes and reads, and what a
//! refresh executor built on the public publication API gets done when
//! source writes race with it. Every workload asserts its answers (masked
//! reads, no stale visible output, report accounting), so a broken prototype
//! fails instead of producing numbers.
//!
//! ```bash
//! BENCH_OUT=/path/results.jsonl BENCH_DATA_DIR=/path/bench-data \
//!   cargo bench -p lance --bench dependent_cell_flags --profile release-with-debug
//! ```
//!
//! Records carry `build=prototype-flags` unless `BENCH_BUILD` is set. The
//! harness sets `LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1` itself, since release
//! builds refuse flagged datasets without it. Configuration is shared with
//! the regression harness (see `cell_flags_common::BenchConfig`), plus
//! `BENCH_CONFLICT_KS` and `BENCH_PUBLISH_KS`.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/mod.rs"]
mod common;
#[path = "cell_flags_common/flags.rs"]
mod flags;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::{Int64Type, UInt64Type};
use futures::TryStreamExt;
use lance::dataset::cell_flag::DeferralReason;
use lance::dataset::scanner::AggregateExpr;
use lance::dataset::{Dataset, DependencyConflictPolicy, ProjectionRequest};
use lance::session::Session;
use lance_core::ROW_ADDR;
use lance_select::RowAddrSelection;
use lance_table::feature_flags::ENABLE_UNSTABLE_CELL_FLAGS_ENV;
use roaring::RoaringTreemap;
use serde_json::{Value, json};

use common::{
    BenchConfig, CacheState, IoMeter, OutputColumn, Record, Reporter, SimulatedUdf, append,
    articles_batch, body_source_batch, commit, create_articles_dataset, dir_bytes, env_list,
    id_in_predicate, merge_in_place, ms_since, open_fresh_session, rewritten_body_for,
    round_robin_groups, scattered_ids, stage_refresh, timed, update_rows, version_file_sizes,
};
use flags::{
    FlaggedOutput, PENDING_TAG, PUBLISHED_TAG, ReportCounts, Reuse, StagedPublication, TagState,
    assert_complete, check_report, check_visible_outputs, column_source_batch, count_rows,
    create_flagged_dataset, derived_invalidations, flagged_outputs, materialize, moved_rows,
    physical_rows, reset_to_tag, stage_pending, true_rows_of,
};

/// Rows touched by the sparse update and merge-insert workloads.
const SPARSE_ROWS: u64 = 100;
/// Rows each source commit writes in the conflict workloads.
const CONFLICT_ROWS_PER_COMMIT: u64 = 10;
/// Rows per unrelated append in `flagged_publish_after_k_commits`.
const PUBLISH_APPEND_ROWS: u64 = 100;
/// Rows fetched by `take_random_1k`.
const TAKE_ROWS: u64 = 1000;
/// Rows re-checked against the UDF after refreshes and updates.
const VERIFY_ROWS: u64 = 1000;
/// Invalidated fractions for `flag_state_size`, in parts per thousand.
const FRAGMENTATION_PER_MILLE: [u64; 4] = [0, 1, 10, 100];

// The regression harness's seeds, so both harnesses touch the same rows.
const SEED_SPARSE: u64 = 11;
const SEED_CONFLICTS: u64 = 12;
const SEED_NULLS: u64 = 13;
const SEED_TAKE: u64 = 14;
const SEED_VERIFY: u64 = 15;
const SEED_FRAGMENTATION: u64 = 16;

/// Tag of the post-publication state both follow-up strategies start from.
const FOLLOWUP_TAG: &str = "followup";

const UDF_NOTE: &str = "udf_ms is the labeled simulated UDF (SimulatedUdf, FNV-1a rounds), \
    reported separately from read, assemble, stage and commit time";

/// Output groups registered on a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Groups {
    /// No flags (a control table for `flag_state_size`).
    Zero,
    /// `summary.ready`.
    One,
    /// `summary.ready` and `translation.ready`, sharing `body`.
    Two,
}

impl Groups {
    const FLAGGED: [Self; 2] = [Self::One, Self::Two];

    fn columns(self) -> &'static [OutputColumn] {
        match self {
            Self::Zero => &[],
            Self::One => &[OutputColumn::Summary],
            Self::Two => &[OutputColumn::Summary, OutputColumn::Translation],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Zero => "groups=0",
            Self::One => "groups=1",
            Self::Two => "groups=2",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UpdateShape {
    /// `body` on 100 scattered ids: clears every flag.
    Sparse,
    /// `body` where `id % 10 = 3`: clears every flag on 10% of rows.
    Dense,
    /// `views` on 100 scattered ids: clears nothing, flags move with the rows.
    UnrelatedSparse,
    /// `title` on 100 scattered ids: clears `summary.ready` only.
    TitleSparse,
}

impl UpdateShape {
    fn workload(self) -> &'static str {
        match self {
            Self::Sparse => "flagged_update_sparse",
            Self::Dense => "flagged_update_dense",
            Self::UnrelatedSparse => "flagged_update_unrelated_sparse",
            Self::TitleSparse => "flagged_update_title_sparse",
        }
    }

    fn column(self) -> &'static str {
        match self {
            Self::Sparse | Self::Dense => "body",
            Self::UnrelatedSparse => "views",
            Self::TitleSparse => "title",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SourceWrite {
    /// `UpdateBuilder` of `body`: moves the rows to new fragments.
    RowMoving,
    /// Partial-schema `merge_insert` of `body` with `RewriteColumns`: in place.
    InPlace,
    /// Partial-schema `merge_insert` of `summary` itself, in place: an output
    /// override. The only source write here that defers whole groups.
    OutputOverride,
}

impl SourceWrite {
    const ALL: [Self; 3] = [Self::RowMoving, Self::InPlace, Self::OutputOverride];

    fn label(self) -> &'static str {
        match self {
            Self::RowMoving => "update_row_moving",
            Self::InPlace => "merge_insert_in_place",
            Self::OutputOverride => "merge_insert_output_in_place",
        }
    }
}

fn policy_label(policy: DependencyConflictPolicy) -> &'static str {
    match policy {
        DependencyConflictPolicy::Reject => "reject",
        DependencyConflictPolicy::Skip => "skip",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    /// Recompute every pending row at the follow-up snapshot; ignore the report.
    RecomputeAllPending,
    /// Reuse staged values the report says are still valid.
    ReuseValidStaged,
}

impl Strategy {
    const ALL: [Self; 2] = [Self::RecomputeAllPending, Self::ReuseValidStaged];

    fn label(self) -> &'static str {
        match self {
            Self::RecomputeAllPending => "recompute_all_pending",
            Self::ReuseValidStaged => "reuse_valid_staged",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mutation {
    Update {
        shape: UpdateShape,
        groups: Groups,
    },
    MergeInsertPartialSparse {
        groups: Groups,
    },
    RefreshClean {
        groups: Groups,
    },
    RefreshConflicts {
        commits: usize,
        source_write: SourceWrite,
        policy: DependencyConflictPolicy,
    },
    PublishAfter {
        commits: usize,
    },
}

impl Mutation {
    fn workload(self) -> String {
        match self {
            Self::Update { shape, .. } => shape.workload().into(),
            Self::MergeInsertPartialSparse { .. } => "flagged_merge_insert_partial_sparse".into(),
            Self::RefreshClean { .. } => "flagged_refresh_clean".into(),
            Self::RefreshConflicts { commits, .. } => {
                format!("flagged_refresh_conflicts_{commits}")
            }
            Self::PublishAfter { .. } => "flagged_publish_after_k_commits".into(),
        }
    }

    fn variant(self) -> String {
        match self {
            Self::Update { groups, .. }
            | Self::MergeInsertPartialSparse { groups }
            | Self::RefreshClean { groups } => groups.label().into(),
            Self::RefreshConflicts {
                source_write,
                policy,
                ..
            } => format!("{}:{}", source_write.label(), policy_label(policy)),
            Self::PublishAfter { commits } => format!("k={commits}"),
        }
    }

    fn groups(self) -> Groups {
        match self {
            Self::Update { groups, .. }
            | Self::MergeInsertPartialSparse { groups }
            | Self::RefreshClean { groups } => groups,
            Self::RefreshConflicts { .. } | Self::PublishAfter { .. } => Groups::One,
        }
    }

    /// The tagged state every sample starts from.
    fn start_tag(self) -> &'static str {
        match self {
            Self::Update { .. } | Self::MergeInsertPartialSparse { .. } => PUBLISHED_TAG,
            _ => PENDING_TAG,
        }
    }

    fn notes(self) -> &'static str {
        match self {
            Self::Update { shape, .. } => match shape {
                UpdateShape::Sparse => {
                    "UpdateBuilder body on 100 scattered ids of a fully published table; \
                     wall = execute (write + commit, row-moving)"
                }
                UpdateShape::Dense => {
                    "UpdateBuilder body where id % 10 = 3 on a fully published table; wall = execute"
                }
                UpdateShape::UnrelatedSparse => {
                    "UpdateBuilder views on 100 scattered ids of a fully published table; flags \
                     move with the rows; wall = execute"
                }
                UpdateShape::TitleSparse => {
                    "UpdateBuilder title on 100 scattered ids of a fully published table; clears \
                     summary.ready only; wall = execute"
                }
            },
            Self::MergeInsertPartialSparse { .. } => {
                "merge_insert (id, body) RewriteColumns on 100 scattered ids of a fully published \
                 table; wall = stage_ms + commit_ms"
            }
            Self::RefreshClean { .. } => {
                "one publication per output, each read + udf + stage + commit (DataReplacement + \
                 CellFlagUpdate, Reject) of every pending row"
            }
            Self::RefreshConflicts { .. } => {
                "stage a full summary refresh at V, commit K source writes of 10 rows each, then \
                 publish at V with the policy; wall = staging to publication; commit_ms = the \
                 conflict-checked publication"
            }
            Self::PublishAfter { .. } => {
                "wall = commit_ms = the publication commit only (Reject), after k unrelated \
                 100-row appends; staging and appends are in extra"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountApi {
    ScannerAggregate,
    Sql,
}

#[derive(Debug, Clone, Copy)]
enum Read {
    ScanSummaryFull,
    FilterSummaryIsNullCount,
    CountSummaryVsStar(CountApi),
    FilterIdRangeProjectSummary,
    TakeRandom1k,
}

impl Read {
    const ALL: [Self; 6] = [
        Self::ScanSummaryFull,
        Self::FilterSummaryIsNullCount,
        Self::CountSummaryVsStar(CountApi::ScannerAggregate),
        Self::CountSummaryVsStar(CountApi::Sql),
        Self::FilterIdRangeProjectSummary,
        Self::TakeRandom1k,
    ];

    fn workload(self) -> &'static str {
        match self {
            Self::ScanSummaryFull => "scan_summary_full",
            Self::FilterSummaryIsNullCount => "filter_summary_is_null_count",
            Self::CountSummaryVsStar(_) => "count_summary_vs_star",
            Self::FilterIdRangeProjectSummary => "filter_id_range_project_summary",
            Self::TakeRandom1k => "take_random_1k",
        }
    }

    fn variant(self, state: FlagState) -> String {
        match self {
            Self::CountSummaryVsStar(CountApi::ScannerAggregate) => {
                format!("{}:aggregate", state.label())
            }
            Self::CountSummaryVsStar(CountApi::Sql) => format!("{}:sql", state.label()),
            _ => state.label().to_string(),
        }
    }

    fn notes(self, cache: CacheState) -> String {
        let query = match self {
            Self::ScanSummaryFull => "scan projecting masked summary; non-NULL counted in Rust",
            Self::FilterSummaryIsNullCount => {
                "Scanner filter `summary IS NULL` + count_rows (masked rows count)"
            }
            Self::CountSummaryVsStar(CountApi::ScannerAggregate) => {
                "Scanner::aggregate COUNT(summary), COUNT(*) over the masked column"
            }
            Self::CountSummaryVsStar(CountApi::Sql) => {
                "Dataset::sql SELECT COUNT(summary), COUNT(*) FROM dataset over the masked column"
            }
            Self::FilterIdRangeProjectSummary => {
                "filter `id >= a AND id < b` (1% of rows, no index) projecting masked summary"
            }
            Self::TakeRandom1k => {
                "Dataset::take of 1000 scattered row offsets projecting masked summary"
            }
        };
        let cache = match cache {
            CacheState::Warm => "one open Dataset/Session reused across samples",
            CacheState::FreshSession => {
                "new Session + open per sample (Lance caches empty, OS page cache not \
                 controlled); wall excludes the open (extra.open_ms)"
            }
        };
        format!("{query}; {cache}")
    }
}

/// Flag state of a read table.
#[derive(Debug, Clone, Copy)]
enum FlagState {
    /// Every row published.
    AllTrue,
    /// Published, then 1% of rows (the regression's `null_1pct` ids)
    /// invalidated by one in-place `body` write.
    PartialOnePercent,
    /// Registered and never published: `summary` has no data file yet and
    /// every row is masked.
    AllPending,
}

impl FlagState {
    fn label(self) -> &'static str {
        match self {
            Self::AllTrue => "all_true",
            Self::PartialOnePercent => "partial_1pct",
            Self::AllPending => "all_pending",
        }
    }
}

/// What a read returns: rows produced (or counted) and how many had a non-NULL summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReadObservation {
    rows: u64,
    non_null_summary: u64,
}

struct ReadTable {
    state: FlagState,
    uri: String,
    rows: u64,
    /// Ids whose summary reads NULL (masked).
    masked_ids: RoaringTreemap,
    fragments: usize,
    manifest_bytes: Option<u64>,
    txn_bytes: Option<u64>,
    dataset_bytes: u64,
}

impl ReadTable {
    fn id_range(&self) -> (u64, u64) {
        let start = self.rows * 37 / 100;
        (start, start + (self.rows / 100).max(1))
    }

    fn take_indices(&self) -> Vec<u64> {
        scattered_ids(self.rows, TAKE_ROWS.min(self.rows), SEED_TAKE)
    }

    fn expected(&self, read: Read) -> ReadObservation {
        let non_null_in = |ids: &mut dyn Iterator<Item = u64>| {
            ids.fold((0, 0), |(rows, non_null), id| {
                (
                    rows + 1,
                    non_null + u64::from(!self.masked_ids.contains(id)),
                )
            })
        };
        let (rows, non_null_summary) = match read {
            Read::ScanSummaryFull => non_null_in(&mut (0..self.rows)),
            Read::FilterSummaryIsNullCount => (self.masked_ids.len(), 0),
            Read::CountSummaryVsStar(_) => (self.rows, self.rows - self.masked_ids.len()),
            Read::FilterIdRangeProjectSummary => {
                let (start, end) = self.id_range();
                non_null_in(&mut (start..end))
            }
            Read::TakeRandom1k => non_null_in(&mut self.take_indices().into_iter()),
        };
        ReadObservation {
            rows,
            non_null_summary,
        }
    }
}

/// A base table for mutation workloads with its two tagged states.
struct FlagTable {
    dir: PathBuf,
    base: Dataset,
    groups: Groups,
    outputs: Vec<FlaggedOutput>,
    pending: TagState,
    published: TagState,
}

struct Ctx {
    config: BenchConfig,
    udf: SimulatedUdf,
}

impl Ctx {
    fn verify_ids(&self) -> Vec<u64> {
        let rows = self.config.scale_rows;
        scattered_ids(rows, VERIFY_ROWS.min(rows), SEED_VERIFY)
    }
}

fn main() {
    if std::env::var_os(ENABLE_UNSTABLE_CELL_FLAGS_ENV).is_none() {
        // SAFETY: no other thread exists yet; the runtime starts below.
        unsafe { std::env::set_var(ENABLE_UNSTABLE_CELL_FLAGS_ENV, "1") };
    }
    let mut config = BenchConfig::from_env();
    if std::env::var_os("BENCH_BUILD").is_none() {
        config.build = "prototype-flags".to_string();
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(Box::pin(run(config)));
}

async fn run(config: BenchConfig) {
    let ctx = Ctx {
        udf: SimulatedUdf {
            iterations: config.udf_iterations,
        },
        config,
    };
    let config = &ctx.config;
    let run_dir = config
        .data_dir
        .join(format!("dependent_cell_flags-{}", config.build));
    if run_dir.exists() {
        std::fs::remove_dir_all(&run_dir).expect("remove previous run directory");
    }
    std::fs::create_dir_all(&run_dir).expect("create run directory");

    println!(
        "dependent_cell_flags build={} git_sha={} scale_rows={} rows_per_fragment={} \
         fragments={} samples={} read_samples={} warmup={} stable_row_ids={} data={}",
        config.build,
        config.git_sha.as_deref().unwrap_or("-"),
        config.scale_rows,
        config.rows_per_fragment,
        config.expected_fragments(),
        config.samples,
        config.read_samples,
        config.warmup,
        config.stable_row_ids,
        run_dir.display(),
    );
    println!(
        "udf iterations={} summary={:.0} ns/row translation={:.0} ns/row",
        ctx.udf.iterations,
        ctx.udf.calibrate_ns_per_row(OutputColumn::Summary, 20_000),
        ctx.udf
            .calibrate_ns_per_row(OutputColumn::Translation, 20_000),
    );

    let mut reporter = Reporter::new(config);

    let mutations = mutation_plan()
        .into_iter()
        .filter(|mutation| config.selects(&mutation.workload()))
        .collect::<Vec<_>>();
    for groups in Groups::FLAGGED {
        let selected = mutations
            .iter()
            .filter(|mutation| mutation.groups() == groups)
            .copied()
            .collect::<Vec<_>>();
        if selected.is_empty() {
            continue;
        }
        let table = Box::pin(build_flag_table(&ctx, &run_dir, groups)).await;
        for mutation in selected {
            Box::pin(run_mutation(&ctx, &table, mutation, &mut reporter)).await;
        }
    }

    let reads = Read::ALL
        .into_iter()
        .filter(|read| config.selects(read.workload()))
        .collect::<Vec<_>>();
    if !reads.is_empty() {
        for state in [
            FlagState::AllTrue,
            FlagState::PartialOnePercent,
            FlagState::AllPending,
        ] {
            let table = Box::pin(build_read_table(&ctx, &run_dir, state)).await;
            for cache in [CacheState::Warm, CacheState::FreshSession] {
                Box::pin(run_reads(&ctx, &table, &reads, cache, &mut reporter)).await;
            }
        }
    }

    if config.selects("flag_state_size") {
        for groups in [Groups::Zero, Groups::One, Groups::Two] {
            Box::pin(run_flag_state_size(&ctx, &run_dir, groups, &mut reporter)).await;
        }
    }

    reporter.print_summary();
    if !config.keep_data {
        std::fs::remove_dir_all(&run_dir).expect("remove benchmark datasets");
    }
}

fn mutation_plan() -> Vec<Mutation> {
    let mut plan = Vec::new();
    for groups in Groups::FLAGGED {
        for shape in [
            UpdateShape::Sparse,
            UpdateShape::Dense,
            UpdateShape::UnrelatedSparse,
            UpdateShape::TitleSparse,
        ] {
            plan.push(Mutation::Update { shape, groups });
        }
        plan.push(Mutation::MergeInsertPartialSparse { groups });
        plan.push(Mutation::RefreshClean { groups });
    }
    for commits in env_list("BENCH_CONFLICT_KS", &[1usize, 4, 16]) {
        for source_write in SourceWrite::ALL {
            for policy in [
                DependencyConflictPolicy::Reject,
                DependencyConflictPolicy::Skip,
            ] {
                plan.push(Mutation::RefreshConflicts {
                    commits,
                    source_write,
                    policy,
                });
            }
        }
    }
    for commits in env_list("BENCH_PUBLISH_KS", &[0usize, 1, 4, 16, 64]) {
        plan.push(Mutation::PublishAfter { commits });
    }
    plan
}

async fn build_flag_table(ctx: &Ctx, run_dir: &Path, groups: Groups) -> FlagTable {
    let dir = run_dir.join(format!("mutations_g{}", groups.columns().len()));
    let (base, outputs) =
        create_flagged_dataset(&dir, &ctx.config, groups.columns(), ctx.udf, true).await;
    assert_complete(&base, &outputs, ctx.udf, ctx.config.scale_rows).await;
    let pending = TagState::of(&base, PENDING_TAG, &outputs).await;
    let published = TagState::of(&base, PUBLISHED_TAG, &outputs).await;
    FlagTable {
        dir,
        base,
        groups,
        outputs,
        pending,
        published,
    }
}

async fn run_mutation(ctx: &Ctx, table: &FlagTable, mutation: Mutation, reporter: &mut Reporter) {
    let warmup = ctx.config.warmup;
    let tag = mutation.start_tag();
    let tag_state = if tag == PENDING_TAG {
        &table.pending
    } else {
        &table.published
    };
    for iteration in 0..warmup + ctx.config.samples {
        let start_state = reset_to_tag(&table.base, tag).await;
        assert_eq!(
            flagged_outputs(&start_state, table.groups.columns()),
            table.outputs,
            "restoring keeps the flag ids"
        );
        tag_state
            .assert_matches(&start_state, &table.outputs, iteration == 0)
            .await;
        let records = Box::pin(mutation_sample(ctx, table, Arc::new(start_state), mutation)).await;
        if iteration >= warmup {
            for mut record in records {
                record.sample = iteration - warmup;
                reporter.emit(record);
            }
        }
    }
}

/// Run one mutation sample from the tagged start state. Returns the
/// workload's record and, for the conflict workloads, the follow-up records.
async fn mutation_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    mutation: Mutation,
) -> Vec<Record> {
    let mut record = Record::new(mutation.workload(), mutation.variant());
    record.fragments = dataset.get_fragments().len();
    record.notes = mutation.notes().to_string();
    record.extra("version_before", dataset.version().version);
    record.extra("groups", table.outputs.len());

    let (result, mut records) = match mutation {
        Mutation::Update { shape, .. } => (
            Some(update_sample(ctx, table, dataset, shape, &mut record).await),
            Vec::new(),
        ),
        Mutation::MergeInsertPartialSparse { .. } => (
            Some(merge_insert_sample(ctx, table, dataset, &mut record).await),
            Vec::new(),
        ),
        Mutation::RefreshClean { .. } => (
            Some(refresh_clean_sample(ctx, table, dataset, &mut record).await),
            Vec::new(),
        ),
        Mutation::RefreshConflicts {
            commits,
            source_write,
            policy,
        } => {
            let (result, followups) = Box::pin(refresh_conflicts_sample(
                ctx,
                table,
                dataset,
                commits,
                source_write,
                policy,
                &mut record,
            ))
            .await;
            (result, followups)
        }
        Mutation::PublishAfter { commits } => (
            Some(publish_after_sample(ctx, table, dataset, commits, &mut record).await),
            Vec::new(),
        ),
    };
    if let Some(result) = result {
        let (manifest_bytes, txn_bytes) = version_file_sizes(&result).await;
        record.manifest_bytes = manifest_bytes;
        record.txn_bytes = txn_bytes;
        record.extra("version_after", result.version().version);
    }
    record.dataset_bytes = dir_bytes(&table.dir);
    for followup in &mut records {
        followup.dataset_bytes = record.dataset_bytes;
    }
    records.insert(0, record);
    records
}

/// Per-output JSON object `{summary: .., translation: ..}`.
fn per_output(outputs: &[FlaggedOutput], values: &[u64]) -> Value {
    Value::Object(
        outputs
            .iter()
            .zip(values)
            .map(|(output, value)| (output.column.name().to_string(), json!(value)))
            .collect(),
    )
}

/// Rows with a true flag in fragments that are not in `before`.
fn true_rows_in_new_fragments(
    dataset: &Dataset,
    outputs: &[FlaggedOutput],
    before: &BTreeMap<u32, u32>,
) -> Vec<u64> {
    let physical = physical_rows(dataset);
    true_rows_of(dataset, outputs)
        .iter()
        .map(|rows| {
            rows.iter()
                .filter(|(fragment, _)| !before.contains_key(fragment))
                .map(|(fragment, selection)| match selection {
                    RowAddrSelection::Full => u64::from(physical[fragment]),
                    RowAddrSelection::Partial(offsets) => offsets.len(),
                })
                .sum()
        })
        .collect()
}

async fn masked_null_count(dataset: &Dataset, column: &str) -> u64 {
    let mut scanner = dataset.scan();
    scanner
        .filter(&format!("{column} IS NULL"))
        .expect("filter IS NULL");
    scanner.count_rows().await.expect("count masked rows")
}

/// Check the flag state and masked reads after a source write that set
/// `field` on `written` rows: every flag watching `field` lost exactly those
/// rows, every other flag kept them, and reads mask exactly the false rows.
async fn check_source_write(
    ctx: &Ctx,
    outputs: &[FlaggedOutput],
    written_dataset: &Dataset,
    field: &str,
    written: u64,
    written_ids: Option<&[u64]>,
) -> Vec<u64> {
    let rows = ctx.config.scale_rows;
    let physical = physical_rows(written_dataset);
    let true_after: Vec<u64> = true_rows_of(written_dataset, outputs)
        .iter()
        .map(|rows| count_rows(rows, &physical))
        .collect();
    for (output, true_after) in outputs.iter().zip(&true_after) {
        let expected = if output.watches(field) {
            rows - written
        } else {
            rows
        };
        assert_eq!(
            *true_after,
            expected,
            "{} true rows after writing {field} on {written} rows",
            output.column.name()
        );
        assert_eq!(
            masked_null_count(written_dataset, output.column.name()).await,
            rows - expected,
            "{} IS NULL must count exactly the invalidated rows",
            output.column.name()
        );
    }
    let mut ids = ctx.verify_ids();
    if let Some(written_ids) = written_ids {
        ids.extend_from_slice(written_ids);
        ids.sort_unstable();
        ids.dedup();
    }
    let check = check_visible_outputs(written_dataset, outputs, ctx.udf, Some(&ids)).await;
    assert_eq!(check.rows, ids.len() as u64, "checked rows");
    assert_eq!(
        check.total_stale(),
        0,
        "no visible output may disagree with its inputs"
    );
    true_after
}

async fn update_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    shape: UpdateShape,
    record: &mut Record,
) -> Dataset {
    let rows = ctx.config.scale_rows;
    let outputs = &table.outputs;
    let sparse_ids = scattered_ids(rows, SPARSE_ROWS.min(rows), SEED_SPARSE);
    let (predicate, value, expected_rows, written_ids) = match shape {
        UpdateShape::Dense => (
            "id % 10 = 3".to_string(),
            format!("'{}'", rewritten_body_for(0, 1)),
            (0..rows).filter(|id| id % 10 == 3).count() as u64,
            None,
        ),
        UpdateShape::Sparse | UpdateShape::UnrelatedSparse | UpdateShape::TitleSparse => {
            let value = match shape {
                UpdateShape::UnrelatedSparse => "views + 1".to_string(),
                UpdateShape::TitleSparse => "'retitled article'".to_string(),
                _ => format!("'{}'", rewritten_body_for(0, 1)),
            };
            (
                id_in_predicate(&sparse_ids),
                value,
                sparse_ids.len() as u64,
                Some(sparse_ids.as_slice()),
            )
        }
    };
    let before = physical_rows(&dataset);
    let meter = IoMeter::new(&dataset).await;
    meter.reset();
    let ((updated, rows_updated), wall_ms) =
        timed(update_rows(dataset, &predicate, shape.column(), &value)).await;
    record.io = meter.take();
    assert_eq!(rows_updated, expected_rows, "rows updated by {predicate}");
    record.wall_ms = wall_ms;
    record.rows_written = rows_updated;

    let true_after = check_source_write(
        ctx,
        outputs,
        &updated,
        shape.column(),
        rows_updated,
        written_ids,
    )
    .await;
    let carried = true_rows_in_new_fragments(&updated, outputs, &before);
    let moved = moved_rows(&updated).await;
    let mut physical = before.clone();
    physical.extend(physical_rows(&updated));
    assert_eq!(
        moved, rows_updated,
        "UpdateBuilder lists every row it moves"
    );
    for (output, carried) in outputs.iter().zip(&carried) {
        let expected = if output.watches(shape.column()) {
            0
        } else {
            rows_updated
        };
        assert_eq!(
            *carried,
            expected,
            "{} flags carried to the moved rows",
            output.column.name()
        );
    }
    let cleared: Vec<u64> = true_after.iter().map(|after| rows - after).collect();
    record.extra("flags_cleared", per_output(outputs, &cleared));
    record.extra("flags_carried", per_output(outputs, &carried));
    record.extra(
        "derived_invalidation_rows",
        per_output(
            outputs,
            &derived_invalidations(&updated, outputs, &physical).await,
        ),
    );
    record.extra("moved_rows", moved);
    updated.as_ref().clone()
}

async fn merge_insert_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    record: &mut Record,
) -> Dataset {
    let rows = ctx.config.scale_rows;
    let outputs = &table.outputs;
    let ids = scattered_ids(rows, SPARSE_ROWS.min(rows), SEED_SPARSE);
    let source = body_source_batch(&dataset, &ids, 1);
    let before = physical_rows(&dataset);
    let meter = IoMeter::new(&dataset).await;
    meter.reset();
    let outcome = merge_in_place(dataset, source).await;
    record.io = meter.take();
    assert_eq!(outcome.stats.num_updated_rows, ids.len() as u64);
    record.wall_ms = outcome.stage_ms + outcome.commit_ms;
    record.stage_ms = Some(outcome.stage_ms);
    record.commit_ms = Some(outcome.commit_ms);
    record.rows_written = outcome.stats.num_updated_rows;
    record.extra("data_bytes_written", outcome.stats.bytes_written);

    let true_after = check_source_write(
        ctx,
        outputs,
        &outcome.dataset,
        "body",
        ids.len() as u64,
        Some(&ids),
    )
    .await;
    let derived = derived_invalidations(&outcome.dataset, outputs, &before).await;
    for (output, derived) in outputs.iter().zip(&derived) {
        assert_eq!(
            *derived,
            ids.len() as u64,
            "{} derived invalidations of an in-place body write",
            output.column.name()
        );
    }
    let cleared: Vec<u64> = true_after.iter().map(|after| rows - after).collect();
    record.extra("flags_cleared", per_output(outputs, &cleared));
    record.extra("derived_invalidation_rows", per_output(outputs, &derived));
    outcome.dataset
}

fn refresh_extra(record: &mut Record, prefix: &str, staged: &StagedPublication) {
    let stats = &staged.stats;
    record.extra(&format!("{prefix}read_ms"), stats.read_ms);
    record.extra(&format!("{prefix}assemble_ms"), stats.assemble_ms);
    record.extra(&format!("{prefix}rows_computed"), stats.rows_computed);
    record.extra(&format!("{prefix}rows_reused"), stats.rows_reused);
    record.extra(&format!("{prefix}rows_copied_through"), stats.rows_copied);
    record.extra(&format!("{prefix}fragments_staged"), stats.fragments_staged);
}

async fn refresh_clean_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    record: &mut Record,
) -> Dataset {
    let rows = ctx.config.scale_rows;
    let meter = IoMeter::new(&dataset).await;
    meter.reset();
    let start = Instant::now();
    let mut head = dataset;
    let mut per_output_ms = serde_json::Map::new();
    let (mut udf_ms, mut stage_ms, mut commit_ms, mut read_ms) = (0.0, 0.0, 0.0, 0.0);
    let (mut staged_rows, mut published_rows) = (0, 0);
    for output in &table.outputs {
        let staged = stage_pending(head.clone(), *output, ctx.udf, None, false).await;
        let (result, output_commit_ms) = staged.publish(DependencyConflictPolicy::Reject).await;
        let result = result.expect("publish clean refresh");
        let counts = check_report(&staged, &result.report);
        assert_eq!(
            counts.published, rows,
            "a clean refresh publishes every row"
        );
        assert_eq!(staged.stats.rows_computed, rows);
        udf_ms += staged.stats.udf_ms;
        stage_ms += staged.stats.stage_ms;
        read_ms += staged.stats.read_ms;
        commit_ms += output_commit_ms;
        staged_rows += staged.stats.rows_staged;
        published_rows += counts.published;
        per_output_ms.insert(
            output.column.name().to_string(),
            json!({
                "udf_ms": staged.stats.udf_ms,
                "read_ms": staged.stats.read_ms,
                "assemble_ms": staged.stats.assemble_ms,
                "stage_ms": staged.stats.stage_ms,
                "commit_ms": output_commit_ms,
            }),
        );
        head = Arc::new(result.dataset);
    }
    record.wall_ms = ms_since(start);
    record.io = meter.take();
    record.udf_ms = Some(udf_ms);
    record.stage_ms = Some(stage_ms);
    record.commit_ms = Some(commit_ms);
    record.rows_written = staged_rows;
    record.rows_published = Some(published_rows);
    record.outcome = Some("committed".into());
    record.extra("read_ms", read_ms);
    record.extra("publications", table.outputs.len());
    record.extra("per_output", per_output_ms);
    record.notes = format!("{}; {UDF_NOTE}", record.notes);

    let check =
        check_visible_outputs(&head, &table.outputs, ctx.udf, Some(&ctx.verify_ids())).await;
    assert_eq!(
        check.total_stale(),
        0,
        "a clean refresh publishes current values"
    );
    for (output, rows_true) in table
        .outputs
        .iter()
        .zip(true_rows_of(&head, &table.outputs))
    {
        assert_eq!(
            count_rows(&rows_true, &physical_rows(&head)),
            rows,
            "{} fully published",
            output.column.name()
        );
    }
    record.stale_results_published = Some(false);
    record.stale_rows = Some(0);
    head.as_ref().clone()
}

/// Run the K source writes of a conflict workload from `head`.
async fn source_writes(
    head: Arc<Dataset>,
    groups: &[Vec<u64>],
    source_write: SourceWrite,
) -> Arc<Dataset> {
    let mut head = head;
    for (generation, ids) in groups.iter().enumerate() {
        let generation = generation as u64 + 1;
        head = match source_write {
            SourceWrite::RowMoving => {
                let value = format!("'{}'", rewritten_body_for(0, generation));
                update_rows(head, &id_in_predicate(ids), "body", &value)
                    .await
                    .0
            }
            SourceWrite::InPlace => {
                let source = body_source_batch(&head, ids, generation);
                Arc::new(merge_in_place(head, source).await.dataset)
            }
            SourceWrite::OutputOverride => {
                let source = column_source_batch(&head, "summary", ids, |id| {
                    format!("override:{generation}:{id}")
                });
                Arc::new(merge_in_place(head, source).await.dataset)
            }
        };
    }
    head
}

/// Fragments at `dataset`'s version that hold any of `ids`.
async fn fragments_of_ids(dataset: &Dataset, ids: &[u64]) -> BTreeSet<u64> {
    let mut scanner = dataset.scan();
    scanner
        .project(&["id"])
        .expect("project id")
        .filter(&id_in_predicate(ids))
        .expect("filter ids")
        .with_row_address();
    let mut stream = scanner.try_into_stream().await.expect("scan ids");
    let mut fragments = BTreeSet::new();
    while let Some(batch) = stream.try_next().await.expect("read ids batch") {
        let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
        fragments.extend(addrs.values().iter().map(|addr| addr >> 32));
    }
    fragments
}

/// Assert what the conflicted publication must report. The expectations
/// follow the prototype README's conflict table.
fn assert_conflict_outcome(
    source_write: SourceWrite,
    counts: &ReportCounts,
    rows: u64,
    changed: u64,
    group_rows: u64,
    groups_touched: u64,
) {
    match source_write {
        SourceWrite::InPlace => {
            assert_eq!(counts.deferred_of(DeferralReason::InputChanged), changed);
            assert_eq!(counts.deferred(), changed);
            assert_eq!(counts.groups_deferred(), 0);
            assert_eq!(counts.published, rows - changed);
        }
        SourceWrite::RowMoving => {
            assert_eq!(counts.deferred_of(DeferralReason::RowVacated), changed);
            assert_eq!(counts.deferred(), changed);
            assert_eq!(counts.groups_deferred(), 0);
            assert_eq!(counts.published, rows - changed);
        }
        SourceWrite::OutputOverride => {
            assert_eq!(
                counts.deferred_groups.get("OutputWritten").copied(),
                Some(groups_touched)
            );
            assert_eq!(counts.groups_deferred(), groups_touched);
            assert_eq!(counts.deferred_of(DeferralReason::InputChanged), changed);
            assert_eq!(counts.valid_rows_in_deferred_groups, group_rows - changed);
            assert_eq!(counts.published, rows - group_rows);
        }
    }
}

fn report_extra(record: &mut Record, counts: &ReportCounts) {
    record.extra("rows_assigned", counts.assigned);
    record.extra("rows_deferred", counts.deferred());
    record.extra("rows_deferred_by_reason", json!(counts.deferred_rows));
    record.extra("fragments_deferred", counts.groups_deferred());
    record.extra(
        "fragments_deferred_by_reason",
        json!(counts.deferred_groups),
    );
    record.extra(
        "valid_rows_in_deferred_groups",
        counts.valid_rows_in_deferred_groups,
    );
    record.extra("rows_reusable", counts.reusable);
}

#[allow(clippy::too_many_arguments)]
async fn refresh_conflicts_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    commits: usize,
    source_write: SourceWrite,
    policy: DependencyConflictPolicy,
    record: &mut Record,
) -> (Option<Dataset>, Vec<Record>) {
    let rows = ctx.config.scale_rows;
    let output = table.outputs[0];
    let conflict_ids = scattered_ids(
        rows,
        (CONFLICT_ROWS_PER_COMMIT * commits as u64).min(rows),
        SEED_CONFLICTS,
    );
    let changed = conflict_ids.len() as u64;
    let id_groups = round_robin_groups(&conflict_ids, commits);
    let read_physical = physical_rows(&dataset);
    let touched_fragments = fragments_of_ids(&dataset, &conflict_ids).await;
    let group_rows: u64 = touched_fragments
        .iter()
        .map(|fragment| u64::from(read_physical[&(*fragment as u32)]))
        .sum();

    let meter = IoMeter::new(&dataset).await;
    meter.reset();
    let start = Instant::now();
    let staged = stage_pending(dataset.clone(), output, ctx.udf, None, true).await;
    let source_start = Instant::now();
    let head = source_writes(dataset.clone(), &id_groups, source_write).await;
    let source_commits_ms = ms_since(source_start);
    let (published, commit_ms) = staged.publish(policy).await;
    record.wall_ms = ms_since(start);
    record.io = meter.take();
    record.udf_ms = Some(staged.stats.udf_ms);
    record.stage_ms = Some(staged.stats.stage_ms);
    record.commit_ms = Some(commit_ms);
    record.rows_written = staged.stats.rows_staged;
    record.extra("read_ms", staged.stats.read_ms);
    record.extra("source_commits", commits);
    record.extra("source_commits_ms", source_commits_ms);
    record.extra("rows_changed_by_sources", changed);
    record.extra("policy", policy_label(policy));
    record.notes = format!("{}; {UDF_NOTE}", record.notes);

    let (after, report) = match published {
        Ok(result) => {
            assert_eq!(
                policy,
                DependencyConflictPolicy::Skip,
                "Reject must refuse a publication whose rows {} made stale",
                source_write.label()
            );
            let counts = check_report(&staged, &result.report);
            assert_conflict_outcome(
                source_write,
                &counts,
                rows,
                changed,
                group_rows,
                touched_fragments.len() as u64,
            );
            record.outcome = Some(
                if result.report.committed_version.is_none() {
                    "deferred_all"
                } else if counts.deferred() + counts.groups_deferred() > 0 {
                    "committed_partial"
                } else {
                    "committed"
                }
                .into(),
            );
            record.rows_published = Some(counts.published);
            report_extra(record, &counts);
            record.extra("checked_version", result.report.checked_version);
            record.extra("committed_version", json!(result.report.committed_version));
            let published_rows = materialize(
                &result.report.published_rows(output.flag_id),
                &read_physical,
            );
            let true_rows = materialize(
                &result
                    .dataset
                    .cell_flag_true_rows(output.flag_id)
                    .expect("flag registered"),
                &physical_rows(&result.dataset),
            );
            assert!(
                published_rows.is_subset(&true_rows),
                "every published row has its flag true after the commit"
            );
            (result.dataset, Some(result.report))
        }
        Err(error) => {
            assert_eq!(
                policy,
                DependencyConflictPolicy::Reject,
                "Skip must publish the rows the source writes left valid: {error}"
            );
            assert!(
                matches!(error, lance::Error::RetryableCommitConflict { .. }),
                "Reject refuses with a retryable conflict: {error}"
            );
            record.outcome = Some("conflict:retryable".into());
            record.rows_published = Some(0);
            record.extra("rows_assigned", staged.stats.rows_pending);
            record.extra("rows_rejected", staged.stats.rows_pending);
            record.extra("error", error.to_string());
            let mut latest = head.as_ref().clone();
            latest.checkout_latest().await.expect("checkout latest");
            assert_eq!(
                latest.version().version,
                head.version().version,
                "a rejected publication commits nothing"
            );
            (latest, None)
        }
    };

    let mut ids = ctx.verify_ids();
    ids.extend_from_slice(&conflict_ids);
    ids.sort_unstable();
    ids.dedup();
    let check = check_visible_outputs(&after, &table.outputs, ctx.udf, Some(&ids)).await;
    assert_eq!(check.rows, ids.len() as u64, "checked rows");
    assert_eq!(
        check.total_stale(),
        0,
        "a publication must not make values computed from old inputs visible"
    );
    record.stale_results_published = Some(false);
    record.stale_rows = Some(0);

    let followups = Box::pin(run_followups(
        ctx,
        table,
        &after,
        &staged,
        report.as_ref(),
        record,
    ))
    .await;
    (Some(after), followups)
}

/// Complete the refresh from the state `after` the conflicted publication,
/// once per strategy, from the same state each time. Each run is checked to
/// leave every flag true and every value current.
async fn run_followups(
    ctx: &Ctx,
    table: &FlagTable,
    after: &Dataset,
    conflicted: &StagedPublication,
    report: Option<&lance::dataset::PublicationReport>,
    conflict_record: &Record,
) -> Vec<Record> {
    let rows = ctx.config.scale_rows;
    let output = conflicted.output;
    let read_physical = physical_rows(&conflicted.snapshot);
    after
        .tags()
        .create(FOLLOWUP_TAG, after.version().version)
        .await
        .expect("tag the follow-up start");
    let mut records = Vec::new();
    for (index, strategy) in Strategy::ALL.into_iter().enumerate() {
        let start_state = if index == 0 {
            after.clone()
        } else {
            reset_to_tag(after, FOLLOWUP_TAG).await
        };
        let reuse = match (strategy, report) {
            (Strategy::ReuseValidStaged, Some(report)) => Some(Reuse {
                values: &conflicted.values,
                rows: materialize(&report.reusable_rows(output.flag_id), &read_physical),
            }),
            _ => None,
        };
        let dataset = Arc::new(start_state);
        let meter = IoMeter::new(&dataset).await;
        meter.reset();
        let start = Instant::now();
        let staged = stage_pending(dataset, output, ctx.udf, reuse.as_ref(), false).await;
        let (result, commit_ms) = staged.publish(DependencyConflictPolicy::Reject).await;
        let wall_ms = ms_since(start);
        let io = meter.take();
        let result = result.expect("an uncontended follow-up publishes");
        let counts = check_report(&staged, &result.report);
        assert_eq!(counts.published, counts.assigned);
        assert_eq!(counts.published, staged.stats.rows_pending);
        assert_complete(&result.dataset, &table.outputs, ctx.udf, rows).await;
        if strategy == Strategy::RecomputeAllPending {
            assert_eq!(staged.stats.rows_reused, 0);
        }

        let mut record = Record::new(
            format!("{}_followup", conflict_record.workload),
            format!("{}:{}", conflict_record.variant, strategy.label()),
        );
        record.fragments = result.dataset.get_fragments().len();
        record.wall_ms = wall_ms;
        record.io = io;
        record.udf_ms = Some(staged.stats.udf_ms);
        record.stage_ms = Some(staged.stats.stage_ms);
        record.commit_ms = Some(commit_ms);
        record.rows_written = staged.stats.rows_staged;
        record.rows_published = Some(counts.published);
        record.outcome = Some("committed".into());
        record.stale_results_published = Some(false);
        record.stale_rows = Some(0);
        record.notes = format!(
            "follow-up refresh to completion after the conflicted publication; {} {}; verified \
             every flag true and every value = UDF(current inputs); {UDF_NOTE}",
            strategy.label(),
            match (strategy, report) {
                (Strategy::RecomputeAllPending, _) => "ignores the report",
                (Strategy::ReuseValidStaged, Some(_)) =>
                    "reuses staged values of PublicationReport::reusable_rows",
                (Strategy::ReuseValidStaged, None) =>
                    "has no report (Reject returned an error), so nothing is known reusable",
            }
        );
        refresh_extra(&mut record, "", &staged);
        record.extra("rows_recomputed", staged.stats.rows_computed);
        record.extra("report_available", report.is_some());
        record.extra("strategy", strategy.label());
        let (manifest_bytes, txn_bytes) = version_file_sizes(&result.dataset).await;
        record.manifest_bytes = manifest_bytes;
        record.txn_bytes = txn_bytes;
        records.push(record);
    }
    after
        .tags()
        .delete(FOLLOWUP_TAG)
        .await
        .expect("delete the follow-up tag");
    records
}

async fn publish_after_sample(
    ctx: &Ctx,
    table: &FlagTable,
    dataset: Arc<Dataset>,
    commits: usize,
    record: &mut Record,
) -> Dataset {
    let rows = ctx.config.scale_rows;
    let output = table.outputs[0];
    let staged = stage_pending(dataset.clone(), output, ctx.udf, None, false).await;
    let appends_start = Instant::now();
    let mut head = dataset.clone();
    for commit_index in 0..commits as u64 {
        let batch = articles_batch(
            rows + commit_index * PUBLISH_APPEND_ROWS,
            PUBLISH_APPEND_ROWS as usize,
        );
        head = Arc::new(append(head, batch).await.0);
    }
    record.extra("appends_ms", ms_since(appends_start));
    let meter = IoMeter::new(&dataset).await;
    meter.reset();
    let (published, commit_ms) = staged.publish(DependencyConflictPolicy::Reject).await;
    record.io = meter.take();
    let published = published.expect("publish after unrelated appends");
    let counts = check_report(&staged, &published.report);
    assert_eq!(counts.published, rows);
    assert_eq!(
        published.report.checked_version,
        head.version().version,
        "the publication is checked against every append"
    );
    let physical = physical_rows(&published.dataset);
    assert_eq!(
        count_rows(
            &published
                .dataset
                .cell_flag_true_rows(output.flag_id)
                .expect("flag registered"),
            &physical
        ),
        rows,
        "appended rows stay pending"
    );
    record.wall_ms = commit_ms;
    record.commit_ms = Some(commit_ms);
    record.rows_written = staged.stats.rows_staged;
    record.rows_published = Some(counts.published);
    record.outcome = Some("committed".into());
    record.extra("staged_udf_ms", staged.stats.udf_ms);
    record.extra("staged_stage_ms", staged.stats.stage_ms);
    record.extra("staged_read_ms", staged.stats.read_ms);
    record.extra("source_commits", commits);
    published.dataset
}

/// A groups=1 table in `state`.
async fn build_read_table(ctx: &Ctx, run_dir: &Path, state: FlagState) -> ReadTable {
    let rows = ctx.config.scale_rows;
    let dir: PathBuf = run_dir.join(format!("reads_{}", state.label()));
    let columns = Groups::One.columns();
    let (dataset, outputs, masked_ids) = match state {
        FlagState::AllTrue => {
            let (dataset, outputs) =
                create_flagged_dataset(&dir, &ctx.config, columns, ctx.udf, true).await;
            (dataset, outputs, RoaringTreemap::new())
        }
        FlagState::PartialOnePercent => {
            let (dataset, outputs) =
                create_flagged_dataset(&dir, &ctx.config, columns, ctx.udf, true).await;
            let ids = scattered_ids(rows, rows / 100, SEED_NULLS);
            let source = body_source_batch(&dataset, &ids, 1);
            let written = merge_in_place(Arc::new(dataset), source).await.dataset;
            (written, outputs, ids.into_iter().collect())
        }
        FlagState::AllPending => {
            let (dataset, outputs) =
                create_flagged_dataset(&dir, &ctx.config, columns, ctx.udf, false).await;
            let mut all = RoaringTreemap::new();
            all.insert_range(0..rows);
            (dataset, outputs, all)
        }
    };
    let physical = physical_rows(&dataset);
    assert_eq!(
        count_rows(
            &dataset
                .cell_flag_true_rows(outputs[0].flag_id)
                .expect("flag registered"),
            &physical
        ),
        rows - masked_ids.len(),
        "true rows of the {} read table",
        state.label()
    );
    let (manifest_bytes, txn_bytes) = version_file_sizes(&dataset).await;
    ReadTable {
        state,
        uri: dir.to_str().expect("utf-8 path").to_string(),
        rows,
        masked_ids,
        fragments: dataset.get_fragments().len(),
        manifest_bytes,
        txn_bytes,
        dataset_bytes: dir_bytes(&dir),
    }
}

async fn run_reads(
    ctx: &Ctx,
    table: &ReadTable,
    reads: &[Read],
    cache: CacheState,
    reporter: &mut Reporter,
) {
    let warm = match cache {
        CacheState::Warm => Some(
            lance::dataset::builder::DatasetBuilder::from_uri(&table.uri)
                .with_session(Arc::new(Session::default()))
                .load()
                .await
                .expect("open read table"),
        ),
        CacheState::FreshSession => None,
    };
    let warmup = ctx.config.warmup;
    for read in reads {
        let expected = table.expected(*read);
        for iteration in 0..warmup + ctx.config.read_samples {
            let (dataset, open_ms) = match &warm {
                Some(dataset) => (dataset.clone(), None),
                None => {
                    let (dataset, open_ms) = timed(open_fresh_session(&table.uri)).await;
                    (dataset, Some(open_ms))
                }
            };
            let meter = IoMeter::new(&dataset).await;
            let (observed, wall_ms) = timed(execute_read(&dataset, *read, table)).await;
            let io = meter.take();
            assert_eq!(
                observed,
                expected,
                "{} on {} returned an answer other than the masked one",
                read.workload(),
                read.variant(table.state)
            );
            if iteration < warmup {
                continue;
            }
            let mut record = Record::new(read.workload(), read.variant(table.state));
            record.sample = iteration - warmup;
            record.fragments = table.fragments;
            record.wall_ms = wall_ms;
            record.io = io;
            record.cache_state = cache;
            record.manifest_bytes = table.manifest_bytes;
            record.txn_bytes = table.txn_bytes;
            record.dataset_bytes = table.dataset_bytes;
            record.notes = read.notes(cache);
            record.extra("rows_returned", observed.rows);
            record.extra("non_null_summary", observed.non_null_summary);
            record.extra("masked_rows", table.masked_ids.len());
            if let Some(open_ms) = open_ms {
                record.extra("open_ms", open_ms);
            }
            reporter.emit(record);
        }
    }
}

async fn execute_read(dataset: &Dataset, read: Read, table: &ReadTable) -> ReadObservation {
    match read {
        Read::ScanSummaryFull => {
            let mut scanner = dataset.scan();
            scanner.project(&["summary"]).expect("project summary");
            count_summary_stream(&scanner).await
        }
        Read::FilterSummaryIsNullCount => {
            let mut scanner = dataset.scan();
            scanner
                .filter("summary IS NULL")
                .expect("filter summary IS NULL");
            ReadObservation {
                rows: scanner.count_rows().await.expect("count NULL summaries"),
                non_null_summary: 0,
            }
        }
        Read::CountSummaryVsStar(api) => {
            let batch = match api {
                CountApi::ScannerAggregate => {
                    let mut scanner = dataset.scan();
                    scanner
                        .aggregate(
                            AggregateExpr::builder()
                                .count("summary")
                                .alias("n_summary")
                                .count_star()
                                .alias("n_star")
                                .build(),
                        )
                        .expect("aggregate counts");
                    scanner.try_into_batch().await.expect("run aggregate")
                }
                CountApi::Sql => {
                    let batches = dataset
                        .sql("SELECT COUNT(summary) AS n_summary, COUNT(*) AS n_star FROM dataset")
                        .build()
                        .await
                        .expect("plan SQL counts")
                        .into_batch_records()
                        .await
                        .expect("run SQL counts");
                    assert_eq!(batches.iter().map(|b| b.num_rows()).sum::<usize>(), 1);
                    batches
                        .into_iter()
                        .find(|batch| batch.num_rows() == 1)
                        .expect("one SQL result row")
                }
            };
            let count = |name: &str| batch[name].as_primitive::<Int64Type>().value(0) as u64;
            ReadObservation {
                rows: count("n_star"),
                non_null_summary: count("n_summary"),
            }
        }
        Read::FilterIdRangeProjectSummary => {
            let (start, end) = table.id_range();
            let mut scanner = dataset.scan();
            scanner
                .project(&["summary"])
                .expect("project summary")
                .filter(&format!("id >= {start} AND id < {end}"))
                .expect("filter id range");
            count_summary_stream(&scanner).await
        }
        Read::TakeRandom1k => {
            let batch = dataset
                .take(
                    &table.take_indices(),
                    ProjectionRequest::from_columns(["summary"], dataset.schema()),
                )
                .await
                .expect("take rows");
            let summary = &batch["summary"];
            ReadObservation {
                rows: batch.num_rows() as u64,
                non_null_summary: (summary.len() - summary.null_count()) as u64,
            }
        }
    }
}

async fn count_summary_stream(scanner: &lance::dataset::scanner::Scanner) -> ReadObservation {
    let mut stream = scanner.try_into_stream().await.expect("scan summary");
    let mut observation = ReadObservation {
        rows: 0,
        non_null_summary: 0,
    };
    while let Some(batch) = stream.try_next().await.expect("read summary batch") {
        let summary = &batch["summary"];
        observation.rows += batch.num_rows() as u64;
        observation.non_null_summary += (summary.len() - summary.null_count()) as u64;
    }
    observation
}

/// Manifest and transaction bytes and fresh-session open latency as the true
/// set fragments. Every table has both outputs populated, so the three
/// variants differ only in their flags: `groups=0` is the unflagged control,
/// `groups=1` flags summary (translation is written with a plain
/// DataReplacement), `groups=2` flags both.
async fn run_flag_state_size(ctx: &Ctx, run_dir: &Path, groups: Groups, reporter: &mut Reporter) {
    let rows = ctx.config.scale_rows;
    let dir = run_dir.join(format!("flag_state_g{}", groups.columns().len()));
    let uri = dir.to_str().expect("utf-8 path").to_string();
    let (base, outputs) = match groups {
        Groups::Zero => {
            let mut dataset =
                create_articles_dataset(&dir, &ctx.config, Arc::new(Session::default())).await;
            for column in [OutputColumn::Summary, OutputColumn::Translation] {
                let staged = stage_refresh(&dataset, column, ctx.udf, None).await;
                dataset = commit(Arc::new(dataset), staged.transaction())
                    .await
                    .expect("populate outputs of the control table");
            }
            (dataset, Vec::new())
        }
        Groups::One | Groups::Two => {
            let (mut dataset, outputs) =
                create_flagged_dataset(&dir, &ctx.config, groups.columns(), ctx.udf, true).await;
            if groups == Groups::One {
                dataset
                    .tags()
                    .delete(PUBLISHED_TAG)
                    .await
                    .expect("move the published tag");
                let staged =
                    stage_refresh(&dataset, OutputColumn::Translation, ctx.udf, None).await;
                dataset = commit(Arc::new(dataset), staged.transaction())
                    .await
                    .expect("populate the unflagged translation");
            }
            (dataset, outputs)
        }
    };
    if groups != Groups::Two {
        base.tags()
            .create(PUBLISHED_TAG, base.version().version)
            .await
            .expect("tag published state");
    }
    let warmup = ctx.config.warmup;
    for per_mille in FRAGMENTATION_PER_MILLE {
        let start_state = reset_to_tag(&base, PUBLISHED_TAG).await;
        let invalidated = rows * per_mille / 1000;
        let (head, txn_bytes) = if invalidated == 0 {
            (start_state, None)
        } else {
            let ids = scattered_ids(rows, invalidated, SEED_FRAGMENTATION);
            let source = body_source_batch(&start_state, &ids, 1);
            let written = merge_in_place(Arc::new(start_state), source).await.dataset;
            (written.clone(), version_file_sizes(&written).await.1)
        };
        let physical = physical_rows(&head);
        let true_rows: Vec<u64> = true_rows_of(&head, &outputs)
            .iter()
            .map(|true_rows| count_rows(true_rows, &physical))
            .collect();
        for true_rows in &true_rows {
            assert_eq!(
                *true_rows,
                rows - invalidated,
                "flag state after invalidation"
            );
        }
        let flag_state_bytes: usize = true_rows_of(&head, &outputs)
            .iter()
            .map(|true_rows| true_rows.serialized_size())
            .sum();
        let (manifest_bytes, _) = version_file_sizes(&head).await;
        let variant = format!(
            "{}:invalidated={}",
            groups.label(),
            match per_mille {
                0 => "0pct".to_string(),
                1 => "0.1pct".to_string(),
                other => format!("{}pct", other / 10),
            }
        );
        for iteration in 0..warmup + ctx.config.read_samples {
            let (dataset, open_ms) = timed(open_fresh_session(&uri)).await;
            let open_io = dataset
                .object_store(None)
                .await
                .expect("object store")
                .io_stats_incremental();
            assert_eq!(dataset.version().version, head.version().version);
            if iteration < warmup {
                continue;
            }
            let mut record = Record::new("flag_state_size", variant.clone());
            record.sample = iteration - warmup;
            record.fragments = dataset.get_fragments().len();
            record.wall_ms = open_ms;
            record.cache_state = CacheState::FreshSession;
            record.io.read_iops = open_io.read_iops;
            record.io.read_bytes = open_io.read_bytes;
            record.manifest_bytes = manifest_bytes;
            record.txn_bytes = txn_bytes;
            record.dataset_bytes = dir_bytes(&dir);
            record.rows_written = invalidated;
            record.notes = "wall = fresh-session open (new Session, OS page cache not \
                controlled) of the head after one in-place body write invalidating the given \
                fraction of scattered rows; txn_bytes is that write's transaction; groups=0 is \
                the unflagged control with the same data files"
                .to_string();
            record.extra("invalidated_rows", invalidated);
            record.extra("flag_true_rows", per_output(&outputs, &true_rows));
            record.extra("flag_state_serialized_bytes", flag_state_bytes);
            reporter.emit(record);
        }
    }
}
