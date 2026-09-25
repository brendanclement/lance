// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Regression benchmark for the dependency-aware cell-flag prototype.
//!
//! Runs ordinary writes, refresh publications and reads on an "articles" table
//! that has **no** cell flags, using only APIs that exist on `main`, so the same
//! file builds against the baseline and the prototype. Compare the two builds
//! on the same machine with the same profile:
//!
//! ```bash
//! BENCH_BUILD=baseline BENCH_GIT_SHA=$(git rev-parse HEAD) \
//! BENCH_OUT=/path/results.jsonl BENCH_DATA_DIR=/path/bench-data \
//!   cargo bench -p lance --bench cell_flags_regression --profile release-with-debug
//! ```
//!
//! Configuration (environment variables, see `cell_flags_common::BenchConfig`):
//! `BENCH_SCALE_ROWS`, `BENCH_ROWS_PER_FRAGMENT`, `BENCH_SAMPLES`, `BENCH_READ_SAMPLES`,
//! `BENCH_WARMUP`,
//! `BENCH_WORKLOADS` (comma list of names or `name_` prefixes), `BENCH_DATA_DIR`,
//! `BENCH_OUT`, `BENCH_BUILD`, `BENCH_GIT_SHA`, `BENCH_STABLE_ROW_IDS`,
//! `BENCH_UDF_ITERS`, `BENCH_KEEP_DATA`, `BENCH_APPEND_ROWS`,
//! `BENCH_CONFLICT_KS`, `BENCH_PUBLISH_KS`.
//!
//! Each sample is one JSON line in `BENCH_OUT`; a median table goes to stdout.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use futures::TryStreamExt;
use lance::dataset::builder::DatasetBuilder;
use lance::dataset::scanner::AggregateExpr;
use lance::dataset::{Dataset, ProjectionRequest};
use lance::session::Session;
use roaring::RoaringTreemap;

use common::{
    BenchConfig, CacheState, Fingerprint, IoMeter, OutputColumn, Record, Reporter, SimulatedUdf,
    append, articles_batch, body_source_batch, commit, count_stale_summaries,
    create_articles_dataset, dir_bytes, env_list, env_or, fingerprint, id_in_predicate,
    merge_in_place, ms_since, open_fresh_session, reset_to_initial, rewritten_body_for,
    round_robin_groups, scattered_ids, stage_refresh, timed, update_rows, version_file_sizes,
};

/// Rows touched by the sparse update and merge-insert workloads.
const SPARSE_ROWS: u64 = 100;
/// Rows whose `body` each source commit rewrites in the conflict workloads.
const CONFLICT_ROWS_PER_COMMIT: u64 = 10;
/// Rows per unrelated append in `publish_after_k_commits`.
const PUBLISH_APPEND_ROWS: u64 = 100;
/// Rows fetched by `take_random_1k`.
const TAKE_ROWS: u64 = 1000;
/// Rows re-checked against the UDF after a clean refresh.
const VERIFY_ROWS: u64 = 1000;

// Independent seeds for each deterministic row selection.
const SEED_SPARSE: u64 = 11;
const SEED_CONFLICTS: u64 = 12;
const SEED_NULLS: u64 = 13;
const SEED_TAKE: u64 = 14;
const SEED_VERIFY: u64 = 15;

const CONFLICT_NOTES: &str = "BASELINE IS NOT CORRECTNESS-EQUIVALENT TO THE PROTOTYPE: main has no \
    dependency tracking. An in-place source write (partial merge_insert RewriteColumns) does not \
    conflict with the staged DataReplacement, so main publishes summaries computed from the \
    pre-update bodies (stale_rows counts them). A row-moving source write (UpdateBuilder) moves \
    rows out of the replaced fragments, which main rejects as a retryable conflict.";

#[derive(Debug, Clone, Copy)]
enum SourceWrite {
    /// `UpdateBuilder`: deletes the old rows and writes them to new fragments.
    RowMoving,
    /// Partial-schema `merge_insert` with `RewriteColumns`: rows stay in place.
    InPlace,
}

impl SourceWrite {
    fn label(self) -> &'static str {
        match self {
            Self::RowMoving => "update_row_moving",
            Self::InPlace => "merge_insert_in_place",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mutation {
    Append,
    UpdateSparse,
    UpdateDense,
    UpdateUnrelatedSparse,
    MergeInsertPartialSparse,
    RefreshClean,
    RefreshConflicts {
        commits: usize,
        source_write: SourceWrite,
    },
    PublishAfter {
        commits: usize,
    },
}

impl Mutation {
    fn workload(self) -> String {
        match self {
            Self::Append => "append".into(),
            Self::UpdateSparse => "update_sparse".into(),
            Self::UpdateDense => "update_dense".into(),
            Self::UpdateUnrelatedSparse => "update_unrelated_sparse".into(),
            Self::MergeInsertPartialSparse => "merge_insert_partial_sparse".into(),
            Self::RefreshClean => "refresh_permissive_clean".into(),
            Self::RefreshConflicts { commits, .. } => {
                format!("refresh_permissive_conflicts_{commits}")
            }
            Self::PublishAfter { .. } => "publish_after_k_commits".into(),
        }
    }

    fn variant(self) -> String {
        match self {
            Self::RefreshConflicts { source_write, .. } => source_write.label().into(),
            Self::PublishAfter { commits } => format!("k={commits}"),
            _ => "default".into(),
        }
    }

    fn notes(self) -> &'static str {
        match self {
            Self::Append => "wall = write fragment (stage_ms) + commit (commit_ms)",
            Self::UpdateSparse => {
                "UpdateBuilder body on 100 scattered ids; wall = execute (write + commit)"
            }
            Self::UpdateDense => {
                "UpdateBuilder body where id % 10 = 3; wall = execute (write + commit)"
            }
            Self::UpdateUnrelatedSparse => {
                "UpdateBuilder views on 100 scattered ids; wall = execute (write + commit)"
            }
            Self::MergeInsertPartialSparse => {
                "merge_insert (id, body) RewriteColumns on 100 scattered ids; wall = stage_ms + commit_ms"
            }
            Self::RefreshClean => {
                "wall = read + udf + stage + commit of a summary DataReplacement; extra.read_ms is the snapshot read"
            }
            Self::RefreshConflicts { .. } => CONFLICT_NOTES,
            Self::PublishAfter { .. } => {
                "wall = commit_ms = the publication commit only, after k unrelated 100-row appends; \
                 staging and appends are in extra"
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

    fn variant(self, data: ReadData) -> String {
        match self {
            Self::CountSummaryVsStar(CountApi::ScannerAggregate) => {
                format!("{}:aggregate", data.label())
            }
            Self::CountSummaryVsStar(CountApi::Sql) => format!("{}:sql", data.label()),
            _ => data.label().to_string(),
        }
    }

    fn notes(self, cache: CacheState) -> String {
        let query = match self {
            Self::ScanSummaryFull => "scan projecting summary; non-NULL counted in Rust",
            Self::FilterSummaryIsNullCount => "Scanner filter `summary IS NULL` + count_rows",
            Self::CountSummaryVsStar(CountApi::ScannerAggregate) => {
                "Scanner::aggregate COUNT(summary), COUNT(*)"
            }
            Self::CountSummaryVsStar(CountApi::Sql) => {
                "Dataset::sql SELECT COUNT(summary), COUNT(*) FROM dataset"
            }
            Self::FilterIdRangeProjectSummary => {
                "filter `id >= a AND id < b` (1% of rows, no index) projecting summary"
            }
            Self::TakeRandom1k => "Dataset::take of 1000 scattered row offsets projecting summary",
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

#[derive(Debug, Clone, Copy)]
enum ReadData {
    /// Every row has a summary.
    Populated,
    /// 1% of rows (scattered, deterministic) have a NULL summary.
    NullOnePercent,
}

impl ReadData {
    fn label(self) -> &'static str {
        match self {
            Self::Populated => "populated",
            Self::NullOnePercent => "null_1pct",
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
    data: ReadData,
    uri: String,
    rows: u64,
    null_ids: RoaringTreemap,
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
                (rows + 1, non_null + u64::from(!self.null_ids.contains(id)))
            })
        };
        let (rows, non_null_summary) = match read {
            Read::ScanSummaryFull => non_null_in(&mut (0..self.rows)),
            Read::FilterSummaryIsNullCount => (self.null_ids.len(), 0),
            Read::CountSummaryVsStar(_) => (self.rows, self.rows - self.null_ids.len()),
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

struct Ctx {
    config: BenchConfig,
    udf: SimulatedUdf,
    append_rows: u64,
}

fn main() {
    let config = BenchConfig::from_env();
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
        append_rows: env_or("BENCH_APPEND_ROWS", 10_000),
        config,
    };
    let config = &ctx.config;
    let run_dir = config
        .data_dir
        .join(format!("cell_flags_regression-{}", config.build));
    if run_dir.exists() {
        std::fs::remove_dir_all(&run_dir).expect("remove previous run directory");
    }
    std::fs::create_dir_all(&run_dir).expect("create run directory");

    println!(
        "cell_flags_regression build={} git_sha={} scale_rows={} rows_per_fragment={} fragments={} \
         samples={} read_samples={} warmup={} stable_row_ids={} data={}",
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
    if !mutations.is_empty() {
        let dir = run_dir.join("mutations");
        let base = create_articles_dataset(&dir, config, Arc::new(Session::default())).await;
        let initial = fingerprint(&base).await;
        for mutation in mutations {
            Box::pin(run_mutation(
                &ctx,
                &base,
                &dir,
                &initial,
                mutation,
                &mut reporter,
            ))
            .await;
        }
    }

    let reads = Read::ALL
        .into_iter()
        .filter(|read| config.selects(read.workload()))
        .collect::<Vec<_>>();
    if !reads.is_empty() {
        for data in [ReadData::Populated, ReadData::NullOnePercent] {
            let table = Box::pin(build_read_table(&ctx, &run_dir, data)).await;
            for cache in [CacheState::Warm, CacheState::FreshSession] {
                Box::pin(run_reads(&ctx, &table, &reads, cache, &mut reporter)).await;
            }
        }
    }

    reporter.print_summary();
    if !config.keep_data {
        std::fs::remove_dir_all(&run_dir).expect("remove benchmark datasets");
    }
}

fn mutation_plan() -> Vec<Mutation> {
    let mut plan = vec![
        Mutation::Append,
        Mutation::UpdateSparse,
        Mutation::UpdateDense,
        Mutation::UpdateUnrelatedSparse,
        Mutation::MergeInsertPartialSparse,
        Mutation::RefreshClean,
    ];
    for commits in env_list("BENCH_CONFLICT_KS", &[1usize, 4, 16]) {
        for source_write in [SourceWrite::RowMoving, SourceWrite::InPlace] {
            plan.push(Mutation::RefreshConflicts {
                commits,
                source_write,
            });
        }
    }
    for commits in env_list("BENCH_PUBLISH_KS", &[0usize, 1, 4, 16, 64]) {
        plan.push(Mutation::PublishAfter { commits });
    }
    plan
}

async fn run_mutation(
    ctx: &Ctx,
    base: &Dataset,
    dir: &Path,
    initial: &Fingerprint,
    mutation: Mutation,
    reporter: &mut Reporter,
) {
    let warmup = ctx.config.warmup;
    for iteration in 0..warmup + ctx.config.samples {
        let start_state = reset_to_initial(base).await;
        assert_eq!(
            start_state.manifest().fragments,
            base.manifest().fragments,
            "restoring the initial tag must reproduce the initial fragments before {}",
            mutation.workload()
        );
        if iteration == 0 {
            assert_eq!(
                &fingerprint(&start_state).await,
                initial,
                "restoring the initial tag must reproduce the initial state before {}",
                mutation.workload()
            );
        }
        let mut record = Box::pin(mutation_sample(ctx, Arc::new(start_state), dir, mutation)).await;
        if iteration >= warmup {
            record.sample = iteration - warmup;
            reporter.emit(record);
        }
    }
}

/// Run one mutation sample from the initial state. IO counters cover exactly
/// the region timed by `wall_ms`.
async fn mutation_sample(
    ctx: &Ctx,
    dataset: Arc<Dataset>,
    dir: &Path,
    mutation: Mutation,
) -> Record {
    let rows = ctx.config.scale_rows;
    let mut record = Record::new(mutation.workload(), mutation.variant());
    record.fragments = dataset.get_fragments().len();
    record.notes = mutation.notes().to_string();
    record.extra("version_before", dataset.version().version);
    let meter = IoMeter::new(&dataset).await;

    let result: Option<Dataset> = match mutation {
        Mutation::Append => {
            let batch = articles_batch(rows, ctx.append_rows as usize);
            meter.reset();
            let (committed, stage_ms, commit_ms) = append(dataset, batch).await;
            record.io = meter.take();
            record.wall_ms = stage_ms + commit_ms;
            record.stage_ms = Some(stage_ms);
            record.commit_ms = Some(commit_ms);
            record.rows_written = ctx.append_rows;
            Some(committed)
        }
        Mutation::UpdateSparse | Mutation::UpdateDense | Mutation::UpdateUnrelatedSparse => {
            let sparse_ids = scattered_ids(rows, SPARSE_ROWS.min(rows), SEED_SPARSE);
            let replacement_body = format!("'{}'", rewritten_body_for(0, 1));
            let (predicate, column, value, expected_rows) = match mutation {
                Mutation::UpdateSparse => (
                    id_in_predicate(&sparse_ids),
                    "body",
                    replacement_body.as_str(),
                    sparse_ids.len() as u64,
                ),
                Mutation::UpdateDense => (
                    "id % 10 = 3".to_string(),
                    "body",
                    replacement_body.as_str(),
                    (0..rows).filter(|id| id % 10 == 3).count() as u64,
                ),
                _ => (
                    id_in_predicate(&sparse_ids),
                    "views",
                    "views + 1",
                    sparse_ids.len() as u64,
                ),
            };
            meter.reset();
            let ((updated, rows_updated), wall_ms) =
                timed(update_rows(dataset, &predicate, column, value)).await;
            record.io = meter.take();
            assert_eq!(rows_updated, expected_rows, "rows updated by {predicate}");
            record.wall_ms = wall_ms;
            record.rows_written = rows_updated;
            Some(updated.as_ref().clone())
        }
        Mutation::MergeInsertPartialSparse => {
            let ids = scattered_ids(rows, SPARSE_ROWS.min(rows), SEED_SPARSE);
            let source = body_source_batch(&dataset, &ids, 1);
            meter.reset();
            let outcome = merge_in_place(dataset, source).await;
            record.io = meter.take();
            assert_eq!(outcome.stats.num_updated_rows, ids.len() as u64);
            record.wall_ms = outcome.stage_ms + outcome.commit_ms;
            record.stage_ms = Some(outcome.stage_ms);
            record.commit_ms = Some(outcome.commit_ms);
            record.rows_written = outcome.stats.num_updated_rows;
            record.extra("data_bytes_written", outcome.stats.bytes_written);
            Some(outcome.dataset)
        }
        Mutation::RefreshClean => {
            meter.reset();
            let start = Instant::now();
            let staged = stage_refresh(&dataset, OutputColumn::Summary, ctx.udf, None).await;
            let (published, commit_ms) = timed(commit(dataset.clone(), staged.transaction())).await;
            record.wall_ms = ms_since(start);
            record.io = meter.take();
            let published = published.expect("publish clean refresh");
            record.udf_ms = Some(staged.udf_ms);
            record.stage_ms = Some(staged.stage_ms);
            record.commit_ms = Some(commit_ms);
            record.rows_written = staged.rows_staged;
            record.rows_published = Some(staged.rows_staged);
            record.extra("read_ms", staged.read_ms);
            record.outcome = Some("committed".into());
            let verify_ids = scattered_ids(rows, VERIFY_ROWS.min(rows), SEED_VERIFY);
            let stale = count_stale_summaries(&published, ctx.udf, &verify_ids).await;
            assert_eq!(stale, 0, "a clean refresh must publish current summaries");
            record.stale_results_published = Some(false);
            record.stale_rows = Some(stale);
            Some(published)
        }
        Mutation::RefreshConflicts {
            commits,
            source_write,
        } => {
            let conflict_ids = scattered_ids(
                rows,
                (CONFLICT_ROWS_PER_COMMIT * commits as u64).min(rows),
                SEED_CONFLICTS,
            );
            let groups = round_robin_groups(&conflict_ids, commits);
            meter.reset();
            let start = Instant::now();
            let staged = stage_refresh(&dataset, OutputColumn::Summary, ctx.udf, None).await;
            let source_start = Instant::now();
            let mut head = dataset.clone();
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
                };
            }
            let source_commits_ms = ms_since(source_start);
            // The refresh holds the snapshot it read; the commit rebases onto the head.
            let (published, commit_ms) = timed(commit(dataset.clone(), staged.transaction())).await;
            record.wall_ms = ms_since(start);
            record.io = meter.take();
            record.udf_ms = Some(staged.udf_ms);
            record.stage_ms = Some(staged.stage_ms);
            record.commit_ms = Some(commit_ms);
            record.rows_written = staged.rows_staged;
            record.extra("read_ms", staged.read_ms);
            record.extra("source_commits", commits);
            record.extra("source_commits_ms", source_commits_ms);
            match published {
                Ok(published) => {
                    let stale = count_stale_summaries(&published, ctx.udf, &conflict_ids).await;
                    record.outcome = Some("committed".into());
                    record.rows_published = Some(staged.rows_staged);
                    record.stale_results_published = Some(stale > 0);
                    record.stale_rows = Some(stale);
                    Some(published)
                }
                Err(error) => {
                    record.outcome = Some(classify_error(&error));
                    record.rows_published = Some(0);
                    record.stale_results_published = Some(false);
                    record.stale_rows = Some(0);
                    record.extra("error", error.to_string());
                    None
                }
            }
        }
        Mutation::PublishAfter { commits } => {
            let staged = stage_refresh(&dataset, OutputColumn::Summary, ctx.udf, None).await;
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
            meter.reset();
            let (published, commit_ms) = timed(commit(dataset.clone(), staged.transaction())).await;
            record.io = meter.take();
            let published = published.expect("publish after unrelated appends");
            record.wall_ms = commit_ms;
            record.commit_ms = Some(commit_ms);
            record.rows_written = staged.rows_staged;
            record.rows_published = Some(staged.rows_staged);
            record.outcome = Some("committed".into());
            record.extra("staged_udf_ms", staged.udf_ms);
            record.extra("staged_stage_ms", staged.stage_ms);
            record.extra("staged_read_ms", staged.read_ms);
            record.extra("source_commits", commits);
            Some(published)
        }
    };

    if let Some(result) = result {
        let (manifest_bytes, txn_bytes) = version_file_sizes(&result).await;
        record.manifest_bytes = manifest_bytes;
        record.txn_bytes = txn_bytes;
        record.extra("version_after", result.version().version);
    }
    record.dataset_bytes = dir_bytes(dir);
    record
}

fn classify_error(error: &lance::Error) -> String {
    match error {
        lance::Error::RetryableCommitConflict { .. } => "conflict:retryable".into(),
        lance::Error::CommitConflict { .. } => "conflict".into(),
        other => {
            let debug = format!("{other:?}");
            let kind = debug
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()
                .unwrap_or_default()
                .to_string();
            format!("error:{kind}")
        }
    }
}

/// Create a table and populate `summary` with the permissive refresh, leaving
/// 1% of rows NULL for [`ReadData::NullOnePercent`].
async fn build_read_table(ctx: &Ctx, run_dir: &Path, data: ReadData) -> ReadTable {
    let rows = ctx.config.scale_rows;
    let dir: PathBuf = run_dir.join(format!("reads_{}", data.label()));
    let null_ids = match data {
        ReadData::Populated => RoaringTreemap::new(),
        ReadData::NullOnePercent => scattered_ids(rows, rows / 100, SEED_NULLS)
            .into_iter()
            .collect(),
    };
    let dataset = create_articles_dataset(&dir, &ctx.config, Arc::new(Session::default())).await;
    let staged = stage_refresh(
        &dataset,
        OutputColumn::Summary,
        ctx.udf,
        Some(&null_ids).filter(|ids| !ids.is_empty()),
    )
    .await;
    let dataset = commit(Arc::new(dataset), staged.transaction())
        .await
        .expect("populate summary for reads");
    let (manifest_bytes, txn_bytes) = version_file_sizes(&dataset).await;
    ReadTable {
        data,
        uri: dir.to_str().expect("utf-8 path").to_string(),
        rows,
        null_ids,
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
            DatasetBuilder::from_uri(&table.uri)
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
                "{} on {} returned unexpected rows",
                read.workload(),
                read.variant(table.data)
            );
            if iteration < warmup {
                continue;
            }
            let mut record = Record::new(read.workload(), read.variant(table.data));
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
