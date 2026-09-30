// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Process-wide CPU time and hardware counters, and the identity of what a
//! run read, for the counter benches.
//!
//! Included with `#[path = "cell_flags_common/counters.rs"] mod counters;`.
//! Uses only APIs that exist on `main`, so it builds against the baseline and
//! the prototype.
//!
//! | Value | Linux | macOS | other |
//! |---|---|---|---|
//! | `cpu_ns` | `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` | same | null |
//! | `instructions`, `cycles` | null | `proc_pid_rusage(RUSAGE_INFO_V4)` | null |
//!
//! A value the platform or machine cannot provide is `None`, written as null,
//! never as zero. The instruction and cycle counters are also `None` where
//! [`hardware_counters_advance`] sees them not advance, as in virtual machines
//! without counter access.

use lance::dataset::Dataset;
use serde_json::json;

/// Process-wide running totals. `None` means this platform or machine cannot
/// provide the value.
#[derive(Clone, Copy)]
pub struct Counters {
    pub cpu_ns: Option<u64>,
    pub instructions: Option<u64>,
    pub cycles: Option<u64>,
}

impl Counters {
    pub fn read(hardware: bool) -> Self {
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

pub fn delta(after: Option<u64>, before: Option<u64>) -> Option<u64> {
    let (after, before) = (after?, before?);
    Some(
        after
            .checked_sub(before)
            .unwrap_or_else(|| panic!("process counter went backwards: {before} then {after}")),
    )
}

pub const CPU_NS_SOURCE: Option<&str> = if cfg!(any(target_os = "linux", target_os = "macos")) {
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

pub const HARDWARE_SOURCE: Option<&str> = if cfg!(target_os = "macos") {
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
pub fn hardware_counters_advance() -> bool {
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

pub fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

/// What identifies the table a run read: its version, the manifest's content
/// hash, and the shape it describes.
pub async fn dataset_identity(dataset: &Dataset) -> serde_json::Value {
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
