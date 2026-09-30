// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! A counting global allocator for the counter benches.
//!
//! Included with `#[path = "cell_flags_common/alloc.rs"] mod allocations;`, which
//! also installs it as the bench's `#[global_allocator]`. Standard library
//! only, so it builds against the baseline and the prototype.
//!
//! Counting is off unless `BENCH_COUNT_ALLOCATIONS=1`: every thread updating
//! the same counters on every allocation slows allocation-heavy parallel reads
//! down, so timed runs leave it off and a separate run records allocations.
//! Off, [`Allocations::since`] returns `None`, written as null.
//!
//! On, a sample records:
//! - `allocations`: calls to `alloc`, `alloc_zeroed` and `realloc`;
//! - `allocated_bytes`: the sizes they requested, a `realloc` counting its new
//!   size (allocation volume);
//! - `peak_live_growth_bytes`: the most bytes live at once above those live
//!   at the region's start (peak memory, which volume does not bound). Both
//!   count requested capacity, not resident memory.
//!
//! All count every thread of the process.

// Each bench binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

pub const SOURCE: &str = "counting #[global_allocator] over std::alloc::System";

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
// Signed: memory allocated before counting started is freed while counting.
static LIVE_BYTES: AtomicI64 = AtomicI64::new(0);
static PEAK_LIVE_BYTES: AtomicI64 = AtomicI64::new(0);

/// Turn counting on when `BENCH_COUNT_ALLOCATIONS=1`, and return whether it
/// is on. Call it once, before the runtime starts.
pub fn count_from_env() -> bool {
    let counting = match std::env::var("BENCH_COUNT_ALLOCATIONS").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        Ok(other) => panic!("BENCH_COUNT_ALLOCATIONS must be 0 or 1, got {other:?}"),
    };
    COUNTING.store(counting, Ordering::Relaxed);
    counting
}

pub fn is_counting() -> bool {
    COUNTING.load(Ordering::Relaxed)
}

/// Bytes live now, as counted; only differences between readings mean
/// anything.
pub fn live_bytes() -> i64 {
    LIVE_BYTES.load(Ordering::Relaxed)
}

/// The system allocator, counting what passes through it while counting is on.
struct CountingAllocator;

impl CountingAllocator {
    fn allocated(size: usize) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
        let live = LIVE_BYTES.fetch_add(size as i64, Ordering::Relaxed) + size as i64;
        PEAK_LIVE_BYTES.fetch_max(live, Ordering::Relaxed);
    }
}

// SAFETY: every method forwards its arguments unchanged to `System` and
// returns its result; the counters only record sizes.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && is_counting() {
            Self::allocated(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && is_counting() {
            Self::allocated(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        if is_counting() {
            LIVE_BYTES.fetch_sub(layout.size() as i64, Ordering::Relaxed);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = unsafe { System.realloc(ptr, layout, new_size) };
        if !moved.is_null() && is_counting() {
            LIVE_BYTES.fetch_sub(layout.size() as i64, Ordering::Relaxed);
            Self::allocated(new_size);
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Counter values at the start of a measured region.
#[derive(Clone, Copy)]
pub struct Allocations {
    count: u64,
    bytes: u64,
    live_bytes: i64,
}

/// What a measured region allocated.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllocationDelta {
    pub allocations: u64,
    pub allocated_bytes: u64,
    pub peak_live_growth_bytes: u64,
}

impl Allocations {
    /// Start a region: the peak restarts from the bytes live now. Regions must
    /// not overlap.
    pub fn start() -> Self {
        let live_bytes = LIVE_BYTES.load(Ordering::Relaxed);
        PEAK_LIVE_BYTES.store(live_bytes, Ordering::Relaxed);
        Self {
            count: ALLOCATIONS.load(Ordering::Relaxed),
            bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
            live_bytes,
        }
    }

    /// Bytes live when the region started.
    pub fn live_bytes(&self) -> i64 {
        self.live_bytes
    }

    /// What was allocated since `self`, or `None` when counting is off.
    pub fn since(self) -> Option<AllocationDelta> {
        is_counting().then(|| AllocationDelta {
            allocations: ALLOCATIONS.load(Ordering::Relaxed) - self.count,
            allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed) - self.bytes,
            peak_live_growth_bytes: (PEAK_LIVE_BYTES.load(Ordering::Relaxed) - self.live_bytes)
                .max(0) as u64,
        })
    }
}
