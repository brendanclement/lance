# Local optimization continuation — 2026-10-05

The prototype was restored from the verified transfer into the standalone checkout
`/home/brendan/work/lance-cell-flags`, on `codex/dependent-cell-flags-next`.
Source and Git history live on the CIFS network mount. Sophon's work and pinned
submodules were not modified. Nothing was pushed or opened as a PR.

## Preserved starting state

The transfer's 1,320 payload files passed checksum verification; the standalone
bundle and restored Git object connectivity passed verification. These refs remain:

- `brendan/dependency-aware-cell-flags`:
  `cd44c394c8599df366016a8830380b6b684aaf8a`.
- Last transferred production change: `b56278dc86ffb1dd8b86f85749e2aa4279caa9b3`.
- Historical experimental baseline: `e3671b2f5730eea927a088a42cbf30e273edc43c`.
- Independent correctness fix, `brendan/flat-search-null-offset`:
  `8d09fb6236b27a4be9acf41cc086fb9844815396`.

The continuation has not been rebased onto current main. The independent flat-search
fix was not conflated with the optimization work.

## Reviewable changes

| Commit | Change |
| --- | --- |
| `c822c90ba` | Protobuf adapter for network checkouts; immutable benchmark snapshots |
| `1f70175dd` | Reuse saved binary bytes across rotations with hard links |
| `064ee10e2` | Skip masking when every live physical row is true |
| `5e2fce629` | Supply wholly masked outputs through the existing NULL reader |
| `679ce90dd` | Slice fully assigned staging windows; validate uniform chain assignments once |
| `fabb89983` | Prune masked projections by stable field ID, with mixed masked/unmasked projection coverage |

The live-row check counts true offsets inside the fragment's physical bounds and
checks that every hole is deleted. Merely comparing the numbers of true and live
rows is unsafe: a true deleted row could hide a false live row. Normal reads skip
the mask only after this coverage check. Physical copy-through reads and readers
retaining deleted slots still mask false physical slots. Computed NULLs remain
NULL with flag=true, and moved rows retain their state at their new addresses.

A wholly masked top-level output is removed from the decoded projection and supplied
by `NullReader`, preserving output schema, row alignment, deletions and system columns.
Partially masked outputs still decode their true values. Legacy masked reads remain
refused. Regression tests exercise range reads, take, deleted slots, scalar and vector
outputs, and mixed projections across fragments.

The staging fast path requires one strictly ordered computed run to cover the entire
copy window and assign every row of the output. It returns the computed array slice
and inserts the assigned range instead of rebuilding per-row interleave indices.
Uniform chain assignment patterns are checked once; varying masks keep the per-row
validation. Ordering, bounds, deleted-row checks, copy-through alignment, bounded
streaming, and whole-fragment replacement remain in place.

No commit, invalidation, conflict or report code changed. False-to-false clears,
fixed read versions through retries, latest-registry resolution, registration fencing,
newer-result protection over full file footprints, upstream/downstream reuse rules,
and report-driven follow-ups retain their existing regression coverage.
Vector masking still uses parent validity only. The Torch and KMeans fixes are retained.

## Local method

These measurements compare the transferred production implementation with the
continuation on this machine; they do not compare this machine with the old laptop.
The saved `before` source tree differs from the transferred head only in the two
benchmark utility scripts. No Rust production or benchmark source changed before
that binary was built and saved. Its recorded tree is exactly the tree of
`1f70175dd`, so that commit is the reproducible local baseline. Both source trees
and executable SHA-256 hashes
are recorded in each rotation's `run.json`.

- Fedora Linux x86_64, AMD Ryzen 9 3900X, 24 logical CPUs, approximately 125 GiB RAM.
- Rust/Cargo 1.97.0; locked dependencies; identical default features.
- Repository `release-with-debug` profile: thin LTO, 16 codegen units, debug symbols.
- Identical effective flags: `target-cpu=haswell`, `+avx2,+fma,+f16c`.
- Source on CIFS; build caches, immutable executables, synthetic data and working
  copies in local `/tmp` tmpfs. This measures warm local data, not CIFS, SSD or S3 IO.
- Shared 1M-row tables generated once with the saved baseline; 100k rows per fragment.
  The moved pair has 11 fragments, 1,010,000 physical rows and 1M live rows.
- Every mutating sample begins from `cp -R --reflink=auto` of the same root bytes/history,
  reads every copied file once, and opens a fresh Lance session. On tmpfs this is an
  ordinary copy rather than an APFS clone. Root file hashes are checked before/after.
- Balanced rotations include an identical optimized-binary control (`after-copy`).
  Function computation is outside every timed phase. Allocations are measured in a
  separate rotation, as requested capacity and peak live growth rather than RSS.
- Linux process CPU time is available; retired instructions and cycles are unavailable
  and recorded as null. Background host activity and load are recorded; the machine
  is not an isolated benchmark host.

## Results

The read and mutation rotations each have six balanced rounds and an identical
optimized-binary control. Reads have 20 samples per workload per process; mutations
have six. The separate allocation rotation has three rounds and three samples. Every
process also warms up. All 6,228 recorded samples passed the driver checks, and all
root dataset hashes matched before and after every rotation. These results compare
the combined optimizations, rather than isolating each commit's timing contribution.

Absolute times below are pooled sample medians. Ratios are geometric means of the
six paired per-round median ratios, with the existing analyzer's bootstrap 95%
intervals; they need not equal the ratio of the displayed pooled medians.

| Operation | Before ms | After ms | After / before [95% interval] |
| --- | ---: | ---: | --- |
| All-valid scan after movement | 45.23 | 22.95 | 0.505 [0.489, 0.521] |
| Wholly masked scan | 8.44 | 1.06 | 0.126 [0.114, 0.134] |
| Wholly masked take, 1,000 rows | 8.81 | 0.29 | 0.033 [0.032, 0.034] |
| Backfill stage, one output | 246.71 | 127.23 | 0.482 [0.414, 0.539] |
| Backfill stage, shared inputs | 401.04 | 158.14 | 0.367 [0.322, 0.404] |
| Backfill stage, chained outputs | 403.51 | 151.92 | 0.361 [0.320, 0.393] |

The moved plain table scans in 23.39 ms after the change, near the flagged table's
22.95 ms. The ordinary stored-all-NULL table scans in 1.67 ms; the wholly masked
fast path takes 1.06 ms and reads **zero data bytes**, against 14.00 MiB before.
On the optimized build, plain `write_columns` backfill staging takes 119.25 ms for one output and
144.89 / 145.81 ms for the shared / chain pairs; the optimized stager is near those
timings, with residual differences below the precision of this noisy run.

The three backfill-stage identical-binary control ratios are 1.052, 1.063 and 1.048;
their 95% intervals reach 1.228, 1.230 and 1.173. Read control ratios range from
0.931 to 1.002. Other mutation phases vary more (up to 1.219 for a control ratio).
The large gains exceed these controls; small differences are not established effects.

Partially masked scalar scans show no demonstrated improvement: 16.50 → 17.20 ms,
ratio 1.043 [0.998, 1.084], with nearly identical allocation volume. The ordinary
1%-NULL scan is 8.78 ms afterward. Sparse in-place staging is also unresolved:
146.40 → 149.26 ms, paired ratio 0.939 [0.836, 1.033], while its identical-binary
control is 1.045 [0.947, 1.210]. No sparse-stage speedup is claimed.

Staging still writes every touched fragment's output column: about **13.8 MiB**
for one-output backfill or 100 scattered refreshed rows, and **27.6–27.8 MiB** for
two-output backfill. Encoded sizes vary slightly; publication granularity did not change.

Allocation medians from the separate counting rotation (MiB, requested capacity):

| Operation | Allocated before → after | Peak live growth before → after |
| --- | ---: | ---: |
| All-valid moved scan | 251.95 → 244.27 | 36.17 → 31.25 |
| Wholly masked scan | 157.56 → 4.72 | 19.37 → 0.75 |
| One-output backfill stage | 275.41 → 255.31 | 10.27 → 10.25 |
| Chain backfill stage | 530.42 → 490.55 | 18.52 → 18.49 |
| Sparse in-place stage | 436.75 → 436.11 | 12.94 → 12.94 |

Wholly masked scans remove most decoding allocations. Backfill allocation volume
falls about 7%, but peak growth stays similar; its large time gain is primarily a
reduction in per-row processing. Peak growth excludes buffers already live when a
phase starts, including precomputed function outputs, and is not RSS.

Raw records, source identities, hashes, control analysis, verification logs and replay
commands are in [`bench/results/linux-local-optimizations/`](bench/results/linux-local-optimizations/README.md).
The preliminary baseline-only smoke run reproduced the diagnostic costs before Rust
edits; final conclusions use the completed matched rotations above.

## Verification on this machine

Final production code and tests at `fabb89983996569ceb0ba480696528cdd34b34b5`:

- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy -p lance --tests --benches --profile release-with-debug --locked -j 12 -- -D warnings`: passed.
- `cargo test -p lance --lib --profile ci --locked -j 12`: **4,604 passed, 4 ignored**.
  This includes **442 passing cell-flag tests** and one ignored fixture generator.
- `cargo test -p lance-table --profile ci --locked -j 12`: **580 passed**, plus **2 passing doctests**.
- `cargo test -p lance --doc --profile ci --locked -j 12`: **74 passed, 22 existing ignored examples**.
- Benchmark Python utility compilation, replay shell syntax, and all six protobuf
  descriptor comparisons: passed.

Cargo checks used the local target directories and protoc adapter below; tests also
set `LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1`. The `ci` profile is only for correctness
checks; all performance measurements use `release-with-debug`. An initial added test
used a dotted top-level field name that Lance rejects; it was corrected to a supported
mixed projection before the final full-suite pass. No existing assertions were weakened.

These are current Linux x86_64 results. Historical macOS, aarch64, MSRV and Torch
results in the earlier reports are not current-machine test results.

## Running from the network checkout

CIFS restores Git symlinks as plain files when `core.symlinks=false`. The six Rust
protobuf crates' `protos` files therefore need the small protoc adapter, without
changing build scripts or Sophon's submodules. All six descriptor outputs matched
byte-for-byte against a temporary local checkout with real symlinks. That temporary
checkout was removed. The descriptor hashes are preserved with the results.

```sh
cd /home/brendan/work/lance-cell-flags
mkdir -p /tmp/cell-flags-tools
cp prototypes/dependent-cell-flags/bench/protoc_network.py /tmp/cell-flags-tools/protoc-network
chmod +x /tmp/cell-flags-tools/protoc-network
PROTOC=/tmp/cell-flags-tools/protoc-network CARGO_TARGET_DIR=/tmp/cell-flags-build \
  cargo bench -p lance --bench cell_flags_costs --profile release-with-debug --locked --no-run -j 12
```

The adapter resolves the recorded `../../protos` target to the real root protobuf
directory and invokes `/usr/bin/protoc`. It is specific to this checkout layout.
If protobuf sources change on a mount without symlinks, clean the affected protobuf
crates before rebuilding: existing `rerun-if-changed=protos` directives see the plain
placeholder file rather than the directory. No protobuf source changed in this phase.

Targets, generated data and multi-GiB debug executables are intentionally outside Git.
`/tmp` is ephemeral. Committed raw records, source identities, hashes and replay commands
are retained; saved binaries and datasets can be regenerated.

## Remaining decisions and limits

This is still an unstable research prototype. Partly masked scalar reads still build
per-row masks. Sparse in-place refresh still rewrites whole output columns of touched
fragments; this phase does not change its write amplification. The stager still reads
bounded copy-through windows even when assignments replace the entire window.

External flag-state storage, finer publication granularity, masked indexes, compaction,
bindings, staged-file leases, and function-version provenance remain separate decisions.
The broadened ordinary `DataReplacement` partial-coverage rule still needs maintainer
review. Caller-computed values and same-publication chain provenance remain trusted.
Late streaming errors may follow staged writes; lack of publication is not a guarantee
that no staged bytes were written.

Parent-only vector masking is ordinary Arrow NULL behavior, not secure erasure of child
values. Consumers must honor parent validity. Nullable integer Torch dtype/precision
policy remains unresolved; no Python/Torch validation was rerun in this phase.
NULL-vector flat-search slowness and slow take after movement have not been reproduced
on clean main here and remain unconfirmed upstream issues. Vector performance, cold
caches, S3, larger scales and production workloads were not measured in this phase.
