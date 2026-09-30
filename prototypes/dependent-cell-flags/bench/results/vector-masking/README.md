# Vector masking read cost

**Status: measured once, on one laptop.** The timed rotation `rotation12-zip-nullif/` ran on
2026-09-30. It had 12 rounds and 20 samples per workload, with the builds `zip`, `nullif` and
`zip-copy`, an identical-binary control. See Results.

The measurements inform one decision: whether masked vector slots keep their NULL child values.
**Outcome:** they do not. `b56278dc8` removed child-nulling after this rotation, so the `nullif`
build is the implementation now and the guarantee covers only readers that respect list validity.
They are not a performance acceptance test and do not represent production. The rotation used
one Apple M5 Pro laptop running macOS, a `release-with-debug` build (thin LTO, 16 codegen units),
a local SSD with a warm page cache, and flat search only.

## Questions

1. What does nulling the child values of partly masked vector slots cost? Since `4f191f852`,
   `mask_batch` (`rust/lance/src/dataset/fragment/cell_flag_mask.rs`) masks a partly masked
   `FixedSizeList` batch with `arrow_select::zip` against a NULL scalar, which builds new child
   values. The alternative, `nullif`, only replaces the list's validity and keeps the stored vector
   under the NULL slot. Every other type uses `nullif`.
2. What do reads of masked embeddings cost compared with a table without flags that stores the
   same visible data as NULL vectors?

## Tables

`rust/lance/benches/cell_flags_vector_masking.rs` in create mode wrote eight tables under one root.
They were created once with the `zip` build (binary sha256 `1716f294c457…`), with
`BENCH_SCALE_ROWS=1000000` and the default `BENCH_ROWS_PER_FRAGMENT=100000`. The root is
`/private/tmp/claude-501/-Users-brendan-code-lance--claude-worktrees-dependency-aware-cell-flags-43fc32/74020bc7-5e3b-4fa2-a700-40b87904a7b1/scratchpad/vbench/tables`.
It is a temporary directory; recreate it with the create command below if it is gone.

Every table has the same 1,000,000 logical rows in 10 fragments: `id` (Int64), `body` (Utf8), and
`embedding`, a nullable `FixedSizeList<Float32, 128>` computed from `id`. Storage format 2.2.
Every table is built the same way:

1. Write `id` and `body`.
2. Add `embedding` as an all-NULL column.
3. Store `embedding` for every fragment with one `DataReplacement` of `write_columns` files.
4. Give the table's cleared rows an in-place `body` write (`merge_insert`, `RewriteColumns`).

On a flagged table, `embedding.ready` is registered before step 3. It watches `body` and masks
`embedding` (`with_clear_on_write(["body"]).with_mask_when_false(true)`). The replacement
publishes every row, and the body write clears the flag on the cleared rows. On a plain table, the
cleared rows are stored as NULL vectors in step 3 (NULL slot, zeros under it), and step 4 writes
the same bodies.

| Table | Flag | Cleared rows | Visible embeddings | Flag true rows | Manifest bytes | On disk (du) |
|---|---|---|---|---|---|---|
| `plain` | none | none | 1,000,000 | — | 2,673 | 498.2 MiB |
| `ready` | `embedding.ready` | none | 1,000,000 | 1,000,000 | 2,886 | 498.2 MiB |
| `partial_1pct` | `embedding.ready` | 10,000, scattered | 990,000 | 990,000 | 85,056 | 508.1 MiB |
| `partial_50pct` | `embedding.ready` | 500,000, scattered | 500,000 | 500,000 | 496,991 | 508.8 MiB |
| `masked` | `embedding.ready` | all | 0 | 0 | 332,814 | 508.6 MiB |
| `null_1pct` | none | 10,000 stored NULL | 990,000 | — | 4,429 | 508.9 MiB |
| `null_50pct` | none | 500,000 stored NULL | 500,000 | — | 4,428 | 508.9 MiB |
| `null_all` | none | all stored NULL | 0 | — | 4,388 | 19.7 MiB |

In total the root holds 3.48 GiB. Its combined manifest hash (`manifest_blake3` in every run
record) is `3c9e9a1c1577bcf8521622ffc68306bd86a21761784be2d74bbcaa1565f05f2f`.

- **Equal visible data.** Create checked that `ready`, `partial_1pct`, `partial_50pct` and `masked`
  each read the same visible data as `plain`, `null_1pct`, `null_50pct` and `null_all`. It compared
  a digest of every row's id, validity and valid vector, and the same nearest hits (ids and
  distance bits).
- **Stored bytes.** The 1% and 50% plain tables store NULL vectors at full size, while `null_all`
  is encoded as all-NULL pages.
- **Manifests.** The flagged tables' last manifests are larger because they carry the body write's
  transaction, including its clear of the flag. Tables are opened once per run, outside the
  samples.

## Builds

| Label | Checkout | Source |
|---|---|---|
| `zip` | `…/scratchpad/vbench/zip` | this commit as is |
| `nullif` | `…/scratchpad/vbench/nullif` | this commit plus `patches/nullif-fixed-size-list.patch` (uncommitted; the driver records it) |
| `zip-copy` | none | `zip`'s executable run again under another label (`--control zip`), the identical-binary control |

Both checkouts are git worktrees outside any other checkout. Each has its own `target/` and was
built with `cargo bench -p lance --bench cell_flags_vector_masking --profile release-with-debug
--no-run`. The patch changes one condition so that `FixedSizeList` falls through to `nullif`, and
the compiler drops the `zip` branch. `nullif` was a measurement variant when this ran, and
`b56278dc8` later made it the implementation. It exposes stored vectors under masked slots to
readers of the child values.

The patch reaches only partly masked batches. Those occur in the `partial_1pct` and
`partial_50pct` workloads (scan, nearest, take). The other 18 workloads never run the changed
branch: `ready` batches are unmasked, and `masked` batches are built with `new_null_array`. Their
`nullif`/`zip` ratios are therefore a negative control for code-layout effects. Behavior-free edits
moved reads by 1–5% in `../10m-noflag-investigation/`.

## Workloads and records

The bench runs three reads on each table, 24 workloads named `<read>_<table>`:
- `scan`: a full scan of `embedding`, counting visible vectors.
- `nearest`: an unfiltered flat nearest-neighbor search of one fixed query, k = 10, projecting
  `id`. The bench checks that each plan is a flat `KNNVectorDistance`.
- `take`: a take of 1,000 scattered rows of `embedding`.

Each sample runs every selected workload once, and the order rotates by one position per sample.
Records follow the counters bench's schema 2. Each run record lists the binary, every table's
identity and the combined hash. Each sample record has `wall_ns`, `cpu_ns`, `instructions` and
`cycles` (macOS `proc_pid_rusage`), and `rows`. It also has what a counting `#[global_allocator]`
over the system allocator saw, across all threads:
- `allocations`: calls to `alloc`, `alloc_zeroed` and `realloc`.
- `allocated_bytes`: the sizes they requested, with a `realloc` counting its new size, so growing
  buffers count more than their final size.
- `peak_live_growth_bytes`: the most live bytes above those live at the sample's start.

`run_counters_rotation.py --bench cell_flags_vector_masking` drives the builds. With three labels
it runs cyclic rotations. `analyze_counters.py` reports the allocation fields. Its `--pair A:B`
compares two workloads within each build (for example `scan_partial_1pct:scan_null_1pct`).

## Commands

Set `V=/private/tmp/claude-501/-Users-brendan-code-lance--claude-worktrees-dependency-aware-cell-flags-43fc32/74020bc7-5e3b-4fa2-a700-40b87904a7b1/scratchpad/vbench`
and run from a checkout of this branch.

Recreate the tables, only if `$V/tables` is gone (any build of the bench; create refuses an
existing root):

```bash
BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI="$V/tables" BENCH_SCALE_ROWS=1000000 \
  "$V"/zip/target/release-with-debug/deps/cell_flags_vector_masking-<hash>
```

The timed rotation, on an idle machine, is the step still to do. Before it:
- stop other builds and benchmarks;
- keep the machine on power;
- check that `git -C "$V/zip" status` is clean at this commit, and that `$V/nullif` differs only
  by the patch.

`--out` must not exist yet.

```bash
python3 prototypes/dependent-cell-flags/bench/run_counters_rotation.py \
  --bench cell_flags_vector_masking \
  --dataset "$V/tables" --out "$V/rotation" --profile release-with-debug \
  --build zip="$V/zip" --build nullif="$V/nullif" --control zip \
  --rounds 12 --samples 20 --warmup 2
python3 prototypes/dependent-cell-flags/bench/analyze_counters.py "$V/rotation" \
  --reference zip --builds zip-copy,nullif \
  --pair scan_ready:scan_plain --pair nearest_ready:nearest_plain --pair take_ready:take_plain \
  --pair scan_partial_1pct:scan_null_1pct --pair nearest_partial_1pct:nearest_null_1pct \
  --pair take_partial_1pct:take_null_1pct \
  --pair scan_partial_50pct:scan_null_50pct --pair nearest_partial_50pct:nearest_null_50pct \
  --pair take_partial_50pct:take_null_50pct \
  --pair scan_masked:scan_null_all --pair nearest_masked:nearest_null_all \
  --pair take_masked:take_null_all \
  > "$V/rotation/analysis.md"
```

**Expected duration.** One cycle of the 24 workloads took about 1.55 s of sample time in the
calibration run. That run used the full tables, the `zip` build, and a machine busy with builds.
With 2 warm-up and 20 recorded cycles, each run takes about 35 s. The 36 runs (12 rounds × 3
labels) take about 20–25 minutes, plus the builds' freshness check, one warm-up run per label, and
hashing the 3.5 GB root before and after.

Copy `run.json`, `order.tsv`, `runs.tsv`, `source/`, the `round*.jsonl` files and `analysis.md`
into a directory here, but not `bins/`.

## What each comparison answers

| Question | Comparison |
|---|---|
| Child-nulling cost | `nullif` against `zip` on the `partial_*` workloads |
| Noise floor | the same ratio on the 18 unaffected workloads, and `zip-copy` against `zip` |
| Copy cost | `allocated_bytes` and `allocations` of those workloads |
| Masking against equal visible data | the `--pair` ratios within `zip` (and within `nullif`, for what masking would cost without child-nulling) |
| Cost of a registered, fully published flag | `ready` against `plain` |

## Results

Source: `rotation12-zip-nullif/`, with `analysis.md` produced by the analyzer command above.
- **Builds:** `zip` (sha256 `1716f294c457…`) and `nullif` (`335bc13c18fd…`), both at `f8cc8a0a8`;
  `nullif` also has `patches/nullif-fixed-size-list.patch`. `zip-copy` runs `zip`'s executable.
  `run.json` records the rest.
- **Load:** the 1-minute load average was 3.6–16.9 before each run. The first rounds ran while it
  fell from an earlier build.
- **Table:** unchanged across the rotation (`dataset_before == dataset_after`).
- **Figures:** medians over every sample. Ratios are geometric means of per-round medians, with
  bootstrap 95% intervals.

| Workload | `zip` ms | `nullif` ms | `nullif` / `zip` | `zip` MiB allocated | `nullif` MiB |
|---|---|---|---|---|---|
| `scan_partial_1pct` | 235.94 | 13.03 | 0.055 [0.055, 0.056] | 2429.8 | 986.1 |
| `scan_partial_50pct` | 136.87 | 13.54 | 0.099 [0.098, 0.100] | 2100.4 | 986.1 |
| `nearest_partial_1pct` | 256.00 | 226.49 | 0.887 [0.875, 0.899] | 2955.4 | 1511.6 |
| `nearest_partial_50pct` | 163.41 | 121.44 | 0.747 [0.738, 0.757] | 2376.7 | 1262.4 |
| `take_partial_1pct` | 7.76 | 7.80 | 1.008 [0.971, 1.045] | 5.8 | 5.0 |
| `take_partial_50pct` | 7.75 | 7.86 | 1.013 [0.973, 1.054] | 6.4 | 5.0 |

**Noise floor.**
- `zip-copy` against `zip` is 0.990–1.002 on every workload.
- The 18 workloads the patch cannot reach are 0.993–1.014, except `scan_null_1pct` at 0.960.

**Masking against equal visible data**, medians in ms with `zip`:

| Reads | Flagged table | Table without flags |
|---|---|---|
| all visible | `ready`: scan 12.97, nearest 15.99, take 7.52 | `plain`: 13.05, 16.05, 7.44 |
| 1% hidden | `partial_1pct`: scan 235.94 (13.03 under `nullif`), nearest 256.00 | `null_1pct`: 30.16, 239.36 |
| 50% hidden | `partial_50pct`: scan 136.87 (13.54), nearest 163.41 | `null_50pct`: 27.19, 133.10 |
| none visible | `masked`: scan 14.74, nearest 18.81 | `null_all`: 4.16, 7.84 |

What the numbers show:
- **Child-nulling is the whole cost of a partly masked scan.**
  - `arrow_select::zip` against a NULL scalar takes 10–18× the time of `nullif` and allocates
    about 2.5× the bytes.
  - With `nullif`, a partly masked scan (13.0 ms) costs no more than an all-visible one (13.0 ms).
    It is also cheaper than the table storing the same rows as NULL vectors (30.2 ms), whose
    decoder builds validity for the stored NULLs.
- **A registered, fully published flag costs nothing measurable.** `ready` matches `plain` in
  every read.
- **Flat search with any NULL vector in the column is slow in this build, flags or not.**
  - `nearest_null_1pct` (no flags) takes 239 ms against 16 ms for `nearest_plain`, and keeps
    about 1.6 cores busy against about 10.
  - Masking inherits this, which is why `nullif` saves less on search than on scans.
  - It does not come from masking. It is a separate issue in the flat KNN path on `main`, worth
    its own investigation.
- **A fully masked fragment still decodes the stored vectors before replacing them.** `scan_masked`
  takes 14.7 ms, while all-NULL pages take 4.2 ms. Skipping the decode of a fully masked column
  would recover that.
- **Takes are unaffected.** 1,000 rows cost the same everywhere.

**Decision input.**
- The guarantee that no raw reader of the child values sees a stored vector is worth keeping:
  `.values` reshapes and Arrow C Data consumers can read the values buffer directly.
- But `zip` is the wrong way to provide it. A replacement that overwrites only the masked slots'
  values (in place when the buffer is uniquely owned, otherwise one copy of the values buffer) and
  sets their child validity would cost about one copy of the batch at most. That is an estimate,
  not measured here.
- After this rotation the guarantee was narrowed to validity-aware readers instead, and
  `b56278dc8` removed child-nulling.

## Smoke run

The pipeline check ran the driver with both builds and the control on a separate 50,000-row root
(5 fragments of 10,000), then ran the analyzer with `--pair`:
- 3 rounds, 2 samples and 1 warm-up per run;
- the machine was busy with other builds (load average 11–16).

The run exercised every step:
- the builds (both fresh);
- the sha256 of each copied binary;
- the source patches;
- the per-run checks of the run record (binary, workloads, combined table hash, sample count);
- the root's hash before and after (unchanged);
- the analyzer's consistency checks.

Every workload returned the same rows under all three labels.

It ran binaries with sha256 `1716f294c457…` (`zip`) and `335bc13c18fd…` (`nullif`). They were built
from a pre-amend revision of this commit with the same Rust sources. The smoke numbers are not
measurements and are not recorded here. They suggested differences large enough to check first:
`nullif` took about 0.11× `zip`'s wall time on `scan_partial_1pct` and 0.18× on
`scan_partial_50pct`, with 0.43–0.47× the allocated bytes. The unaffected workloads stayed within
0.92–1.02×. The timed rotation has to confirm or refute this at full scale on an idle machine.

## Limitations

- **Machine.** One laptop (Apple M5 Pro, 18 cores, 48 GB, macOS), with the tables on local SSD and
  warm in the page cache. There is no object store, no cold read, and no other machine.
- **Profile.** `release-with-debug` (thin LTO, 16 codegen units), not `release` (fat LTO). In
  `../10m-noflag-investigation/` the local `release` profile reordered builds.
- **Search.** Flat search only. Indexes on masked fields are refused, so no index workload exists.
  `fast_search` would return nothing.
- **Vectors.** Only `Float32` with 128 dimensions, and one query. `Float16` and `Float64` are
  supported but not measured.
- **Consumer cost.** The scan counts visible vectors and does not read child values. What a
  consumer pays, such as a tensor conversion, is not measured.
- **Allocation counting.** The counting allocator adds atomic updates to every allocation in every
  build, and the timings include them. `allocated_bytes` counts requested sizes, not bytes copied.
- **Rotation balance.** Cyclic rotation of three labels balances positions within a round, but each
  label always follows the same other label.
- **Workload order.** Workloads within a run are interleaved and rotated, not randomized.
