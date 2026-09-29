// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Counter comparison of reads on a table without cell flags.
//!
//! Wall time alone cannot tell whether a build does more work or does the same
//! work differently, so each sample also records the whole process's CPU time
//! and, where the platform exposes them, retired instructions and cycles.
//! Every build reads the same dataset files:
//!
//! ```bash
//! # once, with any build: write the populated articles table
//! BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/table BENCH_SCALE_ROWS=10000000 <binary>
//! # per build; prototypes/dependent-cell-flags/bench/run_counters_rotation.py
//! # rotates builds and records their provenance
//! BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=/abs/table BENCH_READ_SAMPLES=60 \
//!   BENCH_BUILD=baseline BENCH_OUT=/abs/counters.jsonl <binary>
//! ```
//!
//! | Value | Linux | macOS | other |
//! |---|---|---|---|
//! | `cpu_ns` | `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` | same | null |
//! | `instructions`, `cycles` | null | `proc_pid_rusage(RUSAGE_INFO_V4)` | null |
//!
//! A value the platform or machine cannot provide is written as null, never as
//! zero. The instruction and cycle counters are also null where a startup probe
//! sees them not advance, as in virtual machines without counter access.
//!
//! Each measure run writes one `run` record (binary, dataset identity, counter
//! sources) before its `sample` records. Uses only APIs that exist on `main`,
//! so the same file builds against the baseline and the prototype.

#![allow(clippy::print_stdout)]

#[path = "cell_flags_common/mod.rs"]
mod common;

use std::io::Write;
use std::path::Path;
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
use serde_json::json;

use common::{
    BenchConfig, OutputColumn, SimulatedUdf, commit, create_articles_dataset, scattered_ids,
    stage_refresh,
};

/// Version of the record layout; readers reject records of other versions.
const SCHEMA: u32 = 2;

const WORKLOADS: [&str; 5] = [
    "scan_summary_full",
    "filter_summary_is_null_count",
    "count_summary_aggregate",
    "filter_id_range_project_summary",
    "take_random_1k",
];

/// Process-wide running totals. `None` means this platform or machine cannot
/// provide the value.
#[derive(Clone, Copy)]
struct Counters {
    cpu_ns: Option<u64>,
    instructions: Option<u64>,
    cycles: Option<u64>,
}

impl Counters {
    fn read(hardware: bool) -> Self {
        let (instructions, cycles) = match hardware.then(hardware_counters).flatten() {
            Some((instructions, cycles)) => (Some(instructions), Some(cycles)),
            None => (None, None),
        };
        Self {
            cpu_ns: process_cpu_ns(),
            instructions,
            cycles,
        }
    }
}

fn delta(after: Option<u64>, before: Option<u64>) -> Option<u64> {
    let (after, before) = (after?, before?);
    Some(
        after
            .checked_sub(before)
            .unwrap_or_else(|| panic!("process counter went backwards: {before} then {after}")),
    )
}

const CPU_NS_SOURCE: Option<&str> = if cfg!(any(target_os = "linux", target_os = "macos")) {
    Some("clock_gettime(CLOCK_PROCESS_CPUTIME_ID)")
} else {
    None
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn process_cpu_ns() -> Option<u64> {
    // SAFETY: an all-zero `timespec` is a valid value, including on targets
    // where the struct has private padding.
    let mut now: libc::timespec = unsafe { std::mem::zeroed() };
    // SAFETY: `now` is a live, writable `timespec`.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut now) };
    assert_eq!(
        status,
        0,
        "clock_gettime(CLOCK_PROCESS_CPUTIME_ID): {}",
        std::io::Error::last_os_error()
    );
    let seconds = u64::try_from(now.tv_sec).expect("process CPU seconds are non-negative");
    let nanos = u64::try_from(now.tv_nsec).expect("process CPU nanoseconds are non-negative");
    Some(seconds * 1_000_000_000 + nanos)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn process_cpu_ns() -> Option<u64> {
    None
}

const HARDWARE_SOURCE: Option<&str> = if cfg!(target_os = "macos") {
    Some("proc_pid_rusage(RUSAGE_INFO_V4)")
} else {
    None
};

/// Retired instructions and cycles of the whole process.
#[cfg(target_os = "macos")]
fn hardware_counters() -> Option<(u64, u64)> {
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
    assert_eq!(
        status,
        0,
        "proc_pid_rusage: {}",
        std::io::Error::last_os_error()
    );
    Some((info.ri_instructions, info.ri_cycles))
}

#[cfg(not(target_os = "macos"))]
fn hardware_counters() -> Option<(u64, u64)> {
    None
}

/// Whether the instruction counter advances over a busy loop. Machines without
/// counter access report a constant.
fn hardware_counters_advance() -> bool {
    let Some((before, _)) = hardware_counters() else {
        return false;
    };
    let mut sum = 0u64;
    for value in 0..1_000_000u64 {
        sum = std::hint::black_box(sum.wrapping_add(value));
    }
    std::hint::black_box(sum);
    let Some((after, _)) = hardware_counters() else {
        return false;
    };
    after > before
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(Box::pin(run()));
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

async fn run() {
    let config = BenchConfig::from_env();
    let uri = required_env("BENCH_COUNTERS_URI");
    match std::env::var("BENCH_COUNTERS_MODE").as_deref() {
        Ok("create") => create(&config, &uri).await,
        Ok("measure") => measure(&config, &uri).await,
        other => panic!("BENCH_COUNTERS_MODE must be create or measure, got {other:?}"),
    }
}

async fn create(config: &BenchConfig, uri: &str) {
    // `create_articles_dataset` replaces whatever is at the path.
    assert!(
        !Path::new(uri).exists(),
        "BENCH_COUNTERS_URI {uri} already exists; create writes a new table only"
    );
    let dataset =
        create_articles_dataset(Path::new(uri), config, Arc::new(Session::default())).await;
    let udf = SimulatedUdf {
        iterations: config.udf_iterations,
    };
    let staged = stage_refresh(&dataset, OutputColumn::Summary, udf, None).await;
    let dataset = commit(Arc::new(dataset), staged.transaction())
        .await
        .expect("populate summary");
    let identity = dataset_identity(&dataset).await;
    println!("created {uri}: {identity}");
}

async fn measure(config: &BenchConfig, uri: &str) {
    let build = required_env("BENCH_BUILD");
    let out_path = required_env("BENCH_OUT");
    let mut builder = DatasetBuilder::from_uri(uri).with_session(Arc::new(Session::default()));
    if let Ok(version) = std::env::var("BENCH_COUNTERS_VERSION") {
        let version = version
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("BENCH_COUNTERS_VERSION must be a version, got {version}"));
        builder = builder.with_version(version);
    }
    let dataset = builder.load().await.expect("open table");
    let identity = dataset_identity(&dataset).await;
    if let Ok(expected) = std::env::var("BENCH_COUNTERS_EXPECT_MANIFEST_BLAKE3") {
        assert_eq!(
            identity["manifest_blake3"].as_str(),
            Some(expected.as_str()),
            "the table at {uri} is not the expected one"
        );
    }
    let rows = dataset.count_rows(None).await.expect("count rows") as u64;
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
        "workloads": WORKLOADS.iter().filter(|workload| config.selects(workload)).collect::<Vec<_>>(),
        "counter_sources": {
            "wall_ns": "std::time::Instant",
            "cpu_ns": CPU_NS_SOURCE,
            "instructions": if hardware { HARDWARE_SOURCE } else { None },
            "cycles": if hardware { HARDWARE_SOURCE } else { None },
        },
    });
    writeln!(out, "{run}").expect("write run record");

    for workload in WORKLOADS {
        if !config.selects(workload) {
            continue;
        }
        for sample in 0..config.warmup + config.read_samples {
            let before = Counters::read(hardware);
            let start = Instant::now();
            let observed = read(&dataset, workload, rows).await;
            let wall_ns = start.elapsed().as_nanos() as u64;
            let after = Counters::read(hardware);
            if sample < config.warmup {
                continue;
            }
            let record = json!({
                "record": "sample",
                "schema": SCHEMA,
                "build": build,
                "workload": workload,
                "sample": sample - config.warmup,
                "wall_ns": wall_ns,
                "cpu_ns": delta(after.cpu_ns, before.cpu_ns),
                "instructions": delta(after.instructions, before.instructions),
                "cycles": delta(after.cycles, before.cycles),
                "rows": observed,
            });
            writeln!(out, "{record}").expect("write sample");
        }
    }
}

/// What identifies the table a run read: its version, the manifest's content
/// hash, and the shape it describes.
async fn dataset_identity(dataset: &Dataset) -> serde_json::Value {
    let location = dataset.manifest_location();
    let store = dataset.object_store(None).await.expect("object store");
    let manifest = store
        .read_one_all(&location.path)
        .await
        .expect("read the manifest");
    let fragments = dataset.get_fragments();
    let mut physical_rows = 0u64;
    for fragment in &fragments {
        physical_rows += fragment
            .metadata()
            .physical_rows
            .expect("physical row count") as u64;
    }
    json!({
        "uri": dataset.uri(),
        "version": dataset.version().version,
        "manifest_path": location.path.to_string(),
        "manifest_bytes": manifest.len(),
        "manifest_blake3": blake3::hash(&manifest).to_hex().to_string(),
        "fragments": fragments.len(),
        "physical_rows": physical_rows,
        "rows": dataset.count_rows(None).await.expect("count rows"),
    })
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
            let batch = scanner.try_into_batch().await.expect("run aggregate");
            let n_summary = batch["n_summary"].as_primitive::<Int64Type>().value(0);
            u64::try_from(n_summary).expect("a count is non-negative")
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
