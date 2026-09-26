// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Hardware-counter comparison of reads on a table without cell flags.
//!
//! Wall time alone cannot tell whether a build does more work or does the same
//! work more slowly (code layout, branch alignment), so each sample also
//! records the whole process's retired instructions and cycles from
//! `proc_pid_rusage` (macOS). Both builds read the same dataset files:
//!
//! ```bash
//! # once, with either build: write the populated articles table
//! BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/table BENCH_SCALE_ROWS=10000000 <binary>
//! # per build, alternating builds between runs
//! BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=/abs/table BENCH_READ_SAMPLES=60 \
//!   BENCH_BUILD=baseline BENCH_OUT=/abs/counters.jsonl <binary>
//! ```
//!
//! Uses only APIs that exist on `main`, so the same file builds against the
//! baseline and the prototype.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/mod.rs"]
mod common;

use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use arrow_array::Array;
use futures::TryStreamExt;
use lance::dataset::scanner::AggregateExpr;
use lance::dataset::{Dataset, ProjectionRequest};
use lance::session::Session;

use common::{
    BenchConfig, OutputColumn, SimulatedUdf, commit, create_articles_dataset, scattered_ids,
    stage_refresh,
};

const WORKLOADS: [&str; 5] = [
    "scan_summary_full",
    "filter_summary_is_null_count",
    "count_summary_aggregate",
    "filter_id_range_project_summary",
    "take_random_1k",
];

#[derive(Clone, Copy)]
struct Counters {
    instructions: u64,
    cycles: u64,
    user_time: u64,
    system_time: u64,
}

fn counters() -> Counters {
    // SAFETY: `rusage_info_v4` is a plain C struct of integers, for which all
    // zero bytes are a valid value.
    let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    // SAFETY: the buffer is a live, correctly sized `rusage_info_v4` for the
    // `RUSAGE_INFO_V4` flavor, and the kernel only writes within it.
    let status = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            (&mut info as *mut libc::rusage_info_v4).cast::<libc::rusage_info_t>(),
        )
    };
    assert_eq!(status, 0, "proc_pid_rusage failed");
    Counters {
        instructions: info.ri_instructions,
        cycles: info.ri_cycles,
        user_time: info.ri_user_time,
        system_time: info.ri_system_time,
    }
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(Box::pin(run()));
}

async fn run() {
    let config = BenchConfig::from_env();
    let uri = std::env::var("BENCH_COUNTERS_URI").expect("set BENCH_COUNTERS_URI");
    match std::env::var("BENCH_COUNTERS_MODE").as_deref() {
        Ok("create") => create(&config, &uri).await,
        Ok("measure") => measure(&config, &uri).await,
        other => panic!("BENCH_COUNTERS_MODE must be create or measure, got {other:?}"),
    }
}

async fn create(config: &BenchConfig, uri: &str) {
    let dataset = create_articles_dataset(
        std::path::Path::new(uri),
        config,
        Arc::new(Session::default()),
    )
    .await;
    let udf = SimulatedUdf {
        iterations: config.udf_iterations,
    };
    let staged = stage_refresh(&dataset, OutputColumn::Summary, udf, None).await;
    let dataset = commit(Arc::new(dataset), staged.transaction())
        .await
        .expect("populate summary");
    println!(
        "created {uri}: version {} with {} fragments",
        dataset.version().version,
        dataset.get_fragments().len()
    );
}

async fn measure(config: &BenchConfig, uri: &str) {
    let dataset = lance::dataset::builder::DatasetBuilder::from_uri(uri)
        .with_session(Arc::new(Session::default()))
        .load()
        .await
        .expect("open table");
    let rows = dataset.count_rows(None).await.expect("count rows") as u64;
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.out_path)
        .expect("open BENCH_OUT");
    for workload in WORKLOADS {
        if !config.selects(workload) {
            continue;
        }
        for sample in 0..config.warmup + config.read_samples {
            let before = counters();
            let start = Instant::now();
            let observed = read(&dataset, workload, rows).await;
            let wall_ns = start.elapsed().as_nanos() as u64;
            let after = counters();
            if sample < config.warmup {
                continue;
            }
            writeln!(
                out,
                "{{\"build\":\"{}\",\"workload\":\"{workload}\",\"sample\":{},\"wall_ns\":{wall_ns},\
                 \"instructions\":{},\"cycles\":{},\"user_time\":{},\"system_time\":{},\"rows\":{}}}",
                config.build,
                sample - config.warmup,
                after.instructions - before.instructions,
                after.cycles - before.cycles,
                after.user_time - before.user_time,
                after.system_time - before.system_time,
                observed
            )
            .expect("write sample");
        }
    }
}

async fn read(dataset: &Dataset, workload: &str, rows: u64) -> u64 {
    match workload {
        "scan_summary_full" => {
            let mut scanner = dataset.scan();
            scanner.project(&["summary"]).expect("project summary");
            count_stream(&scanner).await
        }
        "filter_summary_is_null_count" => {
            let mut scanner = dataset.scan();
            scanner.filter("summary IS NULL").expect("filter");
            scanner.count_rows().await.expect("count")
        }
        "count_summary_aggregate" => {
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
                .expect("aggregate");
            scanner
                .try_into_batch()
                .await
                .expect("run aggregate")
                .num_rows() as u64
        }
        "filter_id_range_project_summary" => {
            let start = rows * 37 / 100;
            let end = start + (rows / 100).max(1);
            let mut scanner = dataset.scan();
            scanner
                .project(&["summary"])
                .expect("project")
                .filter(&format!("id >= {start} AND id < {end}"))
                .expect("filter");
            count_stream(&scanner).await
        }
        "take_random_1k" => {
            let indices = scattered_ids(rows, 1000.min(rows), 14);
            let batch = dataset
                .take(
                    &indices,
                    ProjectionRequest::from_columns(["summary"], dataset.schema()),
                )
                .await
                .expect("take");
            (batch["summary"].len() - batch["summary"].null_count()) as u64
        }
        other => panic!("unknown workload {other}"),
    }
}

async fn count_stream(scanner: &lance::dataset::scanner::Scanner) -> u64 {
    let mut stream = scanner.try_into_stream().await.expect("scan");
    let mut non_null = 0u64;
    while let Some(batch) = stream.try_next().await.expect("batch") {
        let summary = &batch["summary"];
        non_null += (summary.len() - summary.null_count()) as u64;
    }
    non_null
}
