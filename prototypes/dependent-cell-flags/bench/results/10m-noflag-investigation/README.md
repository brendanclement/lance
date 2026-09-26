# No-flag read regression at 10M rows: investigation

**Result.** On tables without cell flags, the prototype's read path does no measurable extra
work. Every comparison of builds retired the same instructions to 0.3%. The wall-time differences
between builds are code-generation effects, and their size and sign depend on the build profile.

- **Benchmark profile** (`release-with-debug`: thin LTO, 16 codegen units).
  - On one shared table, the prototype's full-column reads were 3.2–4.4% slower than `main`, with
    fewer cores busy.
  - Narrowly scoped variants tied this to the prototype's `async fn resolve_cells`, which wraps the
    `merge_overlays` future in both `FragmentReader` read funnels.
  - `2fd300ac2` restores `main`'s call and applies masks afterwards through a plain function. In
    the run that measured the prototype at 1.032–1.038×, it measured 1.000–1.001× `main`. Later
    rotations put it at 1.000–1.032×.
  - Edits that do no work produce the same spread: `main` plus one unused field on
    `FragmentReader` measures 1.014–1.021×, and the fix with `fragment.rs` taken from `main`
    1.056–1.064×.
- **Shipping profile** (`release`: fat LTO, 1 codegen unit), one rotation on the same table.
  - The prototype measures 0.997–1.025× `main`.
  - The fix measures 1.032–1.044×, which is 1.018–1.043× the prototype.
  - `main` plus the unused field measures 1.024–1.032×.

`2fd300ac2` is therefore not a performance fix: it removes the gap in the benchmark profile and
costs about as much in the shipping profile. It stays on the branch because it keeps the read path
for unmasked fields structurally identical to `main`. A per-row or per-batch cost would show up in
the retired instructions, and none does.

## Setup

- Checkouts outside any other Lance checkout (see `../../README.md`), built with
  `--profile release-with-debug` (thin LTO, 16 codegen units). Run 09 instead used
  `--profile release` (fat LTO, 1 codegen unit), the profile of shipped builds. The cargo
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
- `rust/lance/benches/cell_flags_scan_counters.rs` records, per sample, the wall time and the
  whole process's retired instructions, cycles and CPU time (`proc_pid_rusage`,
  `RUSAGE_INFO_V4`). "Busy cores" is CPU time divided by wall time.
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

- **The prototype does the same work.** Instructions match to 0.1% and cycles are within noise.
  In the benchmark profile the lost time is parallelism: fewer cores are busy.
- **Isolation, in the benchmark profile.** The per-open mask lookup is not the cause, since
  `noresolve` behaves like the prototype. A pure layout change of similar size moves the reads by
  about 1% (`layout`). Restoring `main`'s funnel structure removes the gap: `funnel` measures
  1.007–1.013× and `fix` 1.000–1.001×. How the nested future cost overlap between fragment reads
  was not identified.
- **Behavior-free edits move the reads as much as the prototype did.**
  - The fix measured 1.000–1.032× `main` across runs 05–08. On any one table it varies by up to 2
    percentage points between runs.
  - Adding one unused field to `main`'s `FragmentReader` (`base-field`) costs 1.4–2.1%.
  - Inside the prototype crate, `main`'s own `fragment.rs` (`fragbase`) is 3.9–5.5% slower than
    the fix's. Adding the same field back (`fragbase-field`) brings it level with the fix. So the
    same field hurt in `main` and helped in the prototype crate.
  - None of these changes alters the instructions retired by more than 0.2%.
- **The shipping profile reorders the builds** (09). There the prototype is within the range of
  `base-field` and the fix is the slowest build, with equal busy cores and 3.7–4.5% more cycles.
  Which read-path shape is fastest depends on how the compiler partitions and inlines the crate,
  not on what the code does, so no source change here can be relied on to hold wall-time parity.
  The reliable check for a real cost is the instruction count, measured with this rotation on a
  shared table.
- **The regression harness overstated the regression.** `run_paired.sh` writes a new table in
  each process and compares 3 rounds. Each round therefore compares two physically different
  tables. The fix-written table costs both binaries 0.3% more instructions, and the fix reads it
  2.0–2.7% slower than the baseline does. The harness's first rounds also ran soon after the builds.
  - Its clean runs put the prototype at 6–9% (`10m-reads-clean-*`).
  - Its runs of the fix (`10m-reads-fix-*`) put the fix at 1.00–1.07× on full-column reads. Round 1
    favored the fix and rounds 2–3 the baseline, in both orders.
  - On shared tables with rotation, the prototype's regression measured 3.2–4.4%.

## Reproduce

```bash
# checkouts outside any other Lance checkout, e.g. /abs/lance-base and /abs/lance-fix
cargo bench -p lance --bench cell_flags_scan_counters --profile release-with-debug --no-run
BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/abs/articles_10m BENCH_SCALE_ROWS=10000000 \
  BENCH_ROWS_PER_FRAGMENT=100000 /abs/lance-base/target/release-with-debug/deps/cell_flags_scan_counters-<hash>
DATASET=/abs/articles_10m OUT=/abs/rotation ROUNDS=20 \
  BUILDS="baseline=/abs/lance-base fix=/abs/lance-fix fragbase=/abs/lance-fragbase layout=/abs/lance-layout" \
  prototypes/dependent-cell-flags/bench/run_counters_rotation.sh
python3 prototypes/dependent-cell-flags/bench/analyze_counters.py /abs/rotation \
  --reference baseline --builds fix,fragbase,layout
```

The baseline checkout needs these files copied from this branch, plus their `[[bench]]` entries:
- `rust/lance/benches/cell_flags_common/mod.rs`
- `cell_flags_regression.rs` (the counters bench includes the common module)
- `cell_flags_scan_counters.rs`

For the shipping profile, build with `--profile release` and run the rotation with
`PROFILE=release`, as `09-rotation20-release/build.sh` did.

A variant is its base checkout with one file of `patches/` applied. `run_counters_rotation.sh`
takes a directory per build and uses the first `cell_flags_scan_counters-*` binary under
`target/$PROFILE/deps`. Run 08 pointed it at directories holding only the copied binaries.
`run_counters_rotation.sh` replaces the one-off drivers of runs 01–05, which followed the same
design, and `06-rotation20-data-source/driver.sh` is the driver of run 06. The measured copies of
`cell_flags_scan_counters.rs` differ from the committed one only in comments and formatting.
