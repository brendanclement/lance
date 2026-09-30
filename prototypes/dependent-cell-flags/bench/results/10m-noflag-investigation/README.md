# No-flag read regression at 10M rows: investigation

**Status: narrowed down, unresolved.** Performance acceptance for reads of tables without cell
flags is open.

What the measurements establish, on one laptop (see Setup):
- **Nearly equal instruction counts.** In every comparison of builds, the whole process retired the
  same number of instructions to within 0.3%.
- **Build-dependent wall time.** The prototype's full-column reads on one shared table measured:
  - `release-with-debug` (thin LTO, 16 codegen units): 3.2–4.4% slower than `main` (`4bcfb2585`
    against `e3671b2f5`), with fewer cores busy;
  - the repository's local `release` profile (fat LTO, 1 codegen unit): 0.997–1.025× `main`.
- **Edits that do nothing move the reads comparably.** `main` plus one unused field on
  `FragmentReader` measured 1.014–1.021× in `release-with-debug` and 1.024–1.032× in `release`.
  The fix with `fragment.rs` taken from `main` measured 1.056–1.064× in `release-with-debug`.

What they do not establish:
- **Zero overhead.** Equal instruction counts can still carry different memory, cache,
  synchronization and scheduling costs. None of those was measured: there are no cache-miss,
  branch-miss, context-switch or task-scheduling data. The lower busy-core counts in
  `release-with-debug` point at how fragment reads overlap, not only at code speed.
- **The cause.** In `release-with-debug`, narrowly scoped variants tied the gap to the
  prototype's `async fn resolve_cells`, which wraps the `merge_overlays` future in both
  `FragmentReader` read funnels. The mechanism was not identified, and the attribution did not
  carry over to `release`. There, `2fd300ac2`, which removes the wrapper, measured 1.032–1.044× `main`
  against the prototype's 0.997–1.025×.
- **Representativeness.** The runs use one Apple-silicon laptop running macOS, one table shape and
  three full-column read workloads. Neither local build profile matches a deployment build (see
  Limitations).

`2fd300ac2` is not a proven performance fix. It removed the gap in the build and run that also
measured the prototype, and measured slower in the other profile.

No representative comparison was run, and no performance threshold is agreed, so performance
acceptance stays open (see "Representative comparison: not run").

## Limitations

- **Hardware and OS.** One Apple M5 Pro laptop (18 cores, 48 GB) running macOS 26.7, with the system
  allocator, a local APFS SSD, a warm and uncontrolled page cache, and other applications and
  management daemons running. Deployed Lance runs mostly as Linux wheels on x86_64 (Haswell or newer)
  and aarch64 servers, usually reading object storage.
- **Build settings.** Neither local profile matches a shipped build:

  | Build | LTO | codegen units | target CPU | features |
  |---|---|---|---|---|
  | `release-with-debug` (runs 01–08) | thin | 16 | `apple-m1` +neon,fp16,fhm,dotprod | `lance` defaults; `lance-io` `test-util` from the dev-dependency |
  | local `release` (run 09) | fat | 1 | same | same |
  | Linux wheel (`python/`, `maturin build --release`) | thin | 1 | `haswell` +avx2,fma,f16c on x86_64; generic armv8-a on aarch64 | `lance` defaults plus `dynamodb`, `substrait`, `metrics`; `lance-io` `metrics`; no `test-util` |

  The wheel settings come from `python/.cargo/config.toml`, `python/Cargo.toml` and
  `.github/workflows/pypi-publish.yml`. The Java JNI build is fat LTO with 1 codegen unit.
- **IO path.** `cargo bench -p lance` enables `lance-io`'s `test-util` through a dev-dependency. That
  records every local `get_range` in a `Vec` behind a mutex (`rust/lance-io/src/local.rs`,
  `utils/tracking_store.rs`), a synchronization cost that wheels do not have, and it is present in
  every build measured here. Wheels instead wrap stores in the `metrics` meter, which these builds
  lack.
- **Counters.** Instructions, cycles and CPU time are whole-process totals from `proc_pid_rusage`,
  including runtime and IO threads. Cache misses, branch misses, context switches, lock contention
  and task scheduling were not measured.
- **Tables and workloads.** One 10M-row, 100-fragment table shape with one string output. Only the
  full scan, `IS NULL` count and `COUNT` aggregate were rotated. Selective reads (id range, take)
  have only the paired harness's 3 rounds. Two copies of the same table differ physically, and the
  runs identify their tables only by label (see Setup).
- **Statistics.** The intervals are bootstrap intervals over 20 rounds of one table on one machine.
  They do not cover variation between machines, builds of the same source, or table copies.

## Representative comparison: not run

The comparison needs:
- a Linux host of the deployed class (x86_64, Haswell or newer; ideally also aarch64);
- builds of `main` (`e3671b2f5`), the prototype before `2fd300ac2` (`4bcfb2585`) and with it, all
  with the wheel's settings;
- one shared table on the storage deployments use;
- rotated order, with an identical-binary control.

None is available here. The only Linux environment is a Docker VM on this laptop, which is the same
hardware as the runs above, and provisioning paid infrastructure is out of scope. The repository
also defines no wheel-matching Rust profile (thin LTO, 1 codegen unit) for Rust benches. No
performance threshold is documented: `release_process.md` and `CONTRIBUTING.md` mention regressions
without a number, and `rust-benchmark.yml` sets no alert threshold.

The tooling is ready for when such an environment exists.
- `rust/lance/benches/cell_flags_scan_counters.rs` builds and runs on Linux, verified in a Linux
  aarch64 container. There it records CPU time and writes instructions and cycles as null.
- `run_counters_rotation.py` builds each checkout and takes the single bench executable cargo
  reports. It records the executable's sha256, the source revision and patch, rustc, rustflags,
  features and profile. It runs the builds in a Latin square with `--control` as the
  identical-binary control, and hashes the table before and after.
- Linux instruction and cycle counts would need `perf_event_open`, which the libc crate exposes only
  as a syscall number. Until then, `perf stat` around whole runs is the substitute.

## Recommendation on `2fd300ac2`

Keep it, for maintainability rather than speed.
- **Performance evidence is inconclusive.**
  - `release-with-debug`: 1.000–1.001× `main` in the run that measured the prototype at
    1.032–1.038×, and 1.000–1.032× over later runs.
  - Local `release`: 1.032–1.044× `main`, against the prototype's 0.997–1.025×.
  - Neither profile is a deployment build, and behavior-free edits of `main` move the same reads by
    1.4–3.2%.
- **Maintainability favors it.**
  - `merge_overlays` is byte-identical to `main`, and both read funnels call it as `main` does.
  - Each funnel adds one `mask_cells` call, a plain stream stage that returns the stream unchanged
    when no projected field is masked.
  - Removing either call fails `test_every_reader_funnel_masks_each_batch`.
- **Reverting it** would put masking back inside an async wrapper around `merge_overlays`, so the
  function would differ from `main` again, for no measured benefit in any representative build.

## Setup

- Checkouts outside any other Lance checkout (see `../../README.md`), built with
  `--profile release-with-debug` (thin LTO, 16 codegen units). Run 09 instead used
  `--profile release` (fat LTO, 1 codegen unit) from the repository's `.cargo/config.toml`, which
  is not verified to match any deployment build. The cargo
  fingerprints of every bench binary show identical rustflags (`-C target-cpu=apple-m1 -C
  target-feature=+neon,+fp16,+fhm,+dotprod`, once) and features.
  - `baseline`: `e3671b2f5` plus the benchmark files.
  - `prototype`: `4bcfb2585` (code equal to `93ada9bca`).
  - `fix`: `2fd300ac2` (`patches/fix.patch`).
  - Variants, each one patch in `patches/` on the build named:

    | Variant | Base | Patch | What it changes |
    |---|---|---|---|
    | `noresolve` | prototype | `noresolve.patch` | never resolve masks when a fragment opens |
    | `funnel` | prototype | `funnel.patch` | both funnels call `main`'s `merge_overlays`, copied verbatim |
    | `layout` | prototype (04), fix (07) | `layout.patch` | one unused function in `dataset.rs`, a code-placement control |
    | `fragbase` | fix | `fragbase.patch` | `fragment.rs` exactly as on `main`; the rest of the prototype stays |
    | `base-field` | baseline | `probe-field.patch` | an unused `Option<Arc<()>>` field on `FragmentReader`, the size of the prototype's mask field |
    | `fragbase-field` | fragbase | `probe-field.patch` | the same field |

- Tables: 10M rows, 100 fragments, the articles table with `summary` populated, created with
  `BENCH_COUNTERS_MODE=create`.
  - Runs 01–05 and 09 share one table written by the baseline binary.
  - Runs 07 and 08 share a second table written by the fix binary.
  - Run 06 reads both tables.
  - Writes are not byte-identical, although every value is a deterministic function of the row id.
    The data files measure 854,216 KB (baseline-written), 854,184 KB (a second baseline-written
    copy, used only for this comparison) and 854,616 KB (fix-written).
- For runs 01–09, `rust/lance/benches/cell_flags_scan_counters.rs` recorded, per sample, the wall
  time and the whole process's retired instructions, cycles and CPU time (`proc_pid_rusage`,
  `RUSAGE_INFO_V4`, CPU time in Mach ticks). "Busy cores" is CPU time divided by wall time.
- Apple M5 Pro, 18 cores, 48 GB, macOS 26.7, local APFS SSD. The OS page cache was warm and not
  controlled. Other desktop applications and management daemons were running.

## Runs

| Directory | Design | Builds |
|---|---|---|
| `01-abba-prototype-vs-baseline` | 6 rounds, alternating order; started right after the builds (load average 20–35) | baseline, prototype |
| `02-aa-baseline-vs-itself` | same design, with the baseline binary under both labels | baseline, baseline |
| `03-rotation10-noresolve-funnel` | 10 rounds, rotating order. A zsh 1-based array made the driver skip the `layout` build and run an empty fifth slot each round. The other four builds' labels and binaries came from the same index and are valid | baseline, prototype, noresolve, funnel |
| `04-rotation20-funnel-layout` | 20 rounds, 4×4 Latin square (`order.tsv`), one warm-up per build | baseline, prototype, funnel, layout |
| `05-rotation20-fix` | same as 04 | baseline, prototype, fix, funnel |
| `06-rotation20-data-source` | same design, driven by `driver.sh`; each binary reads both tables | baseline and fix × baseline-written and fix-written table |
| `07-rotation20-fix-variants` | same as 04, fix-written table | baseline, fix, fragbase, layout (on the fix) |
| `08-rotation20-probe-field` | same as 04, fix-written table; binaries copied out of their checkouts (`checksums.txt`) | baseline, fix, base-field, fragbase-field |
| `09-rotation20-release` | same as 04, baseline-written table; `--profile release` binaries built by `build.sh` (`checksums.txt`), `PROFILE=release` | baseline, prototype, fix, base-field |

Each directory has the raw JSONL and an `analysis.md` from `analyze_counters.py`.

## Findings

The workloads are full scan, `IS NULL` count and `COUNT` aggregate. Each ratio is the geometric
mean over rounds of the per-round medians, and each range spans the three workloads. An interval
is a bootstrap 95% interval.

| Comparison (run) | Wall time | Instructions | Cycles |
|---|---|---|---|
| prototype / baseline (01) | 1.056–1.073 | 1.000–1.001 | 0.995–1.031 |
| baseline / baseline, A/A (02) | 0.977–0.985 (per round 0.93–1.03) | 0.999–1.000 | — |
| noresolve / baseline (03) | 1.014–1.027 (prototype 1.011–1.022) | 1.001 | — |
| prototype / baseline (04) | 1.033–1.044, intervals exclude 1 | 1.000–1.001 | 1.004–1.023 |
| funnel / baseline (04) | 1.007–1.013, intervals include 1 | 1.000 | 1.000–1.015 |
| layout / prototype (04) | 0.983–0.993, intervals include 1 | 1.000 | — |
| prototype / funnel (04) | 1.026–1.031, intervals exclude 1 | 1.000–1.001 | — |
| prototype / baseline (05) | 1.032–1.038, intervals exclude 1 | 1.000–1.001 | 1.005–1.012 |
| **fix / baseline (05)** | **1.000–1.001, intervals [0.98, 1.02]** | 1.000 | 1.009–1.019 |
| fix / funnel (05) | 1.002–1.008, intervals include 1 | 1.000 | — |
| fix / baseline, baseline-written table (06) | 1.009–1.017, intervals include 1 | 0.999–1.000 | 1.004–1.018 |
| baseline, fix-written / baseline-written table (06) | 1.005–1.009, intervals include 1 | 1.003 | 0.998–1.007 |
| fix / baseline, fix-written table (06) | 1.020–1.027, intervals exclude 1 | 1.000 | 1.016–1.033 |
| fix / baseline (07) | 1.009–1.016, intervals include 1 or touch it | 0.999–1.000 | 1.006–1.019 |
| layout / fix (07) | 0.999–1.008, intervals include 1 | 1.000–1.001 | 0.960–0.967 |
| fragbase / baseline (07) | 1.056–1.064, intervals exclude 1 | 1.001–1.002 | 1.035–1.044 |
| fragbase / fix (07) | 1.039–1.055, intervals exclude 1 | 1.002 | 1.016–1.037 |
| fix / baseline (08) | 1.022–1.032, intervals exclude 1 | 1.000 | 1.013–1.029 |
| **base-field / baseline (08)** | **1.014–1.021, intervals exclude 1** | 1.000 | 1.020–1.023 |
| fragbase-field / baseline (08) | 1.026–1.032, intervals exclude 1 | 1.000–1.001 | 0.991–1.001 |
| fragbase-field / fix (08) | 0.999–1.004, intervals include 1 | — | — |
| prototype / baseline, `release` (09) | 0.997–1.025; only the full scan's interval excludes 1 | 0.999–1.000 | 0.987–1.026 |
| fix / baseline, `release` (09) | 1.032–1.044, intervals exclude 1 | 1.001–1.002 | 1.037–1.045 |
| fix / prototype, `release` (09) | 1.018–1.043, intervals exclude 1 | — | — |
| base-field / baseline, `release` (09) | 1.024–1.032, intervals exclude 1 | 1.001 | 1.005–1.018 |

Busy cores on the full scan: baseline 9.81, prototype 9.45, fix 9.88 and funnel 9.63 (05);
baseline 9.73, fix 9.75, fragbase 9.49 and layout 9.25 (07); baseline 9.75, fix 9.66,
base-field 9.81 and fragbase-field 9.40 (08); baseline 9.53, prototype 9.54, fix 9.52 and
base-field 9.40 (09, `release`).

- **Nearly the same instructions, less overlap.** The prototype's instructions match `main` to
  0.1%, and its cycles are within noise. In `release-with-debug` it keeps fewer cores busy. What
  the lost overlap consists of (waiting on IO, lock or channel contention, task scheduling, cache
  behavior) was not measured.
- **Isolation, in `release-with-debug` only.** The per-open mask lookup is not responsible:
  `noresolve` behaves like the prototype. A pure layout change of similar size moves the reads by
  about 1% (`layout`). Restoring `main`'s funnel structure removes the gap: `funnel` measures
  1.007–1.013× and `fix` 1.000–1.001×. How the nested future reduces overlap between fragment reads
  was not identified.
- **Behavior-free edits move the reads by comparable amounts.**
  - The fix measured 1.000–1.032× `main` across runs 05–08. On any one table it varies by up to 2
    percentage points between runs.
  - Adding one unused field to `main`'s `FragmentReader` (`base-field`) costs 1.4–2.1%.
  - Inside the prototype crate, `main`'s own `fragment.rs` (`fragbase`) is 3.9–5.5% slower than
    the fix's. Adding the same field back (`fragbase-field`) brings it level with the fix. So the
    same field hurt in `main` and helped in the prototype crate.
  - None of these changes alters the instructions retired by more than 0.2%.
  - This sensitivity is consistent with code-generation effects (placement, inlining, future
    layout), but none was measured directly. It also means differences of this size cannot be
    attributed to a single source change from these runs alone.
- **The local `release` profile reorders the builds** (09). There the prototype is within the
  range of `base-field`, and the fix is the slowest build, with equal busy cores and 3.7–4.5% more
  cycles.
- **The regression harness overstated the regression.** `run_paired.sh` writes a new table in
  each process and compares 3 rounds. Each round therefore compares two physically different
  tables. The fix-written table costs both binaries 0.3% more instructions, and the fix reads it
  2.0–2.7% slower than the baseline does. The harness's first rounds also ran soon after the builds.
  - Its clean runs put the prototype at 6–9% (`10m-reads-clean-*`).
  - Its runs of the fix (`10m-reads-fix-*`) put the fix at 1.00–1.07× on full-column reads. Round 1
    favored the fix and rounds 2–3 the baseline, in both orders.
  - On shared tables with rotation, the prototype's regression measured 3.2–4.4%.

## Reproduce

New runs use `run_counters_rotation.py`. It builds each checkout itself, and each checkout must
sit outside any other Lance checkout:

```bash
# once: write the table with any build of the bench
BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/articles_10m BENCH_SCALE_ROWS=10000000 \
  BENCH_ROWS_PER_FRAGMENT=100000 /abs/lance-main/target/release-with-debug/deps/cell_flags_scan_counters-<hash>
python3 prototypes/dependent-cell-flags/bench/run_counters_rotation.py \
  --dataset /abs/articles_10m --out /abs/rotation --profile release-with-debug \
  --build main=/abs/lance-main --build prototype=/abs/lance-proto --build fix=/abs/lance-fix \
  --control main
python3 prototypes/dependent-cell-flags/bench/analyze_counters.py /abs/rotation \
  --reference main --builds main-copy,prototype,fix
```

Reproduce the recorded runs 01–09 as follows:
- Analyze their records, which predate the `schema` field and store CPU time in Mach ticks, with
  `--legacy-mach-timebase 125/3` (Apple silicon). The committed `analysis.md` files regenerate byte
  for byte.
- They were driven by one-off loops (01–06, `06-rotation20-data-source/driver.sh`) and
  `driver-runs-07-09.sh`, the former `bench/run_counters_rotation.sh`. That script takes the first
  matching binary and records no provenance.

The baseline checkout needs these files copied from this branch, plus their `[[bench]]` entries:
- `rust/lance/benches/cell_flags_common/mod.rs`
- `rust/lance/benches/cell_flags_common/counters.rs` (since the vector masking bench shares it)
- `cell_flags_regression.rs` (the counters bench includes the common module)
- `cell_flags_scan_counters.rs`

A variant is its base checkout with one file of `patches/` applied. Run 08 ran copied binaries
(`checksums.txt`). Run 09's `build.sh` built the local `release` profile. Runs 01–09 used the
bench as committed in `060e20459`, apart from comments and formatting; rerun them with that
revision of `cell_flags_scan_counters.rs`. The current bench writes schema-2 records: CPU time in
nanoseconds, unavailable counters as null, and a run record with the binary and table identity.
