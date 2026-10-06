# Partial-mask read costs and sparse-publication amplification

Handoff copy: original report at local revision `b5a59bd77`. The additional benchmark cases,
profiles and raw cost records remain local with Brendan; this published prototype includes
the report only. See [DESIGN_REVIEW.md](DESIGN_REVIEW.md) for qualifications of its conclusions.

2026-10-05. Coding task 3 of [CODING_PLAN.md](CODING_PLAN.md). **Unstable research prototype.**
Base `16413f22b`; local branch `brendan/cell-flag-cost-review`:

- `c3362349f`: benchmark cases in `cell_flags_costs.rs`:
  - scattered and clustered mask densities with stored-NULL pairs;
  - refreshes concentrated in k fragments;
  - a plain whole-column replacement control;
  - staged-file records and a verify mode.
- `17e60c827`: `BENCH_PPROF` stack sampling, used for diagnosis only.

Production code was unchanged during the experiment, and nothing was pushed then. Raw samples,
commands and provenance are in `bench/results/linux-cost-review` on the local experiment branch;
that directory is not included in this published prototype branch.
This is evidence for a review, not production acceptance, and it does not start an
implementation phase.

**Questions.**

1. Does partial masking need a processing optimization?
2. Is whole-fragment publication acceptable for sparse refresh?
3. Which costs need a representation decision rather than faster code?

**Answers.**

1. **Yes, for scattered masks.** A scan of 1M rows whose masked rows are scattered over
   every fragment takes 2.1–2.2× as long as the same values, stored the same way, read
   without masks.
   That is +7.6–8.4 ms at every density from 0.1% to 50%, yet only +6.3–7.7 ms of CPU
   (1.12–1.14×).
   - The extra work is per-batch mask construction run serially on the task that polls
     the scan stream.
   - Clustered masks touch one or two fragments and cost 0–3%.
2. **Yes, for a bounded first workload, and only there.** Cost scales linearly with the
   number of touched fragments, independent of the number of assigned rows.
   - Each touched 100k-row fragment of this string output, stored at about 14 bytes per
     row, costs 1.37 MiB and about 15 ms of staging.
   - Concentrating 100 refreshed rows into 1 fragment instead of 10 cuts staging 9.75×,
     staged bytes 10.1× and the refresh 6.3×.
   - It still writes about 14 KB per assigned row, against 14.5 bytes per row for a
     dense backfill.
3. **Three representation costs remain whatever the processing speed:**
   - whole-fragment publication bytes;
   - stale values that partial masks leave stored and decoded;
   - inline flag state that grows with scattered partial states.

   Ordinary whole-column replacement shares the byte amplification. The
   publication-specific processing is 6–12%.

## Method

- **Harness.** The existing saved-binary harness drives balanced rotations of one binary
  as `cost` against itself as `cost-copy`, the identical-binary control. The workloads
  are:
  - Experiment A: `scan_<table>`;
  - Experiment B: `cycle_inplace_<k>frag_{one,plain_one,plain_one_replace}`,
    `cycle_inplace_sparse_*` and `backfill_*one`.
- **Dataset.** One 1M-row dataset with 100k rows per fragment in file format 2.2. Each
  flagged table pairs with a plain table storing NULL where the flag is false, with
  identical visible values, types, rows, deletions (none) and projection. The masked
  rows are:
  - scattered over the table with fixed seeds; or
  - one run of ids starting at 45% of the table, which begins in the middle of
    fragment 4.
- **Statistics.** Ratios are geometric means over rounds of the per-round ratio of
  medians, with the analyzer's bootstrap 95% intervals. Each comparison's workloads
  ran in one process, interleaved in rotated order.
- **Sample counts.** Reads, mutations and allocation counting have 12, 8 and 4 rounds
  of 30, 6 and 3 samples per process. That is 12,960 read, 1,536 mutation and 816
  allocation-counting samples, plus a separate 12,960-sample quiet read replication.
  Warm-ups are excluded.
- **Controls.** Identical-binary controls stayed within 0.98–1.02 for reads and within
  0.96–1.04 for mutations (0.97–1.01 for stage, merge, commit and refresh phases).
- **Load.** Another session's build saturated the machine during the first reads (load
  up to 33, 0% idle). The read rotation was therefore replicated immediately afterwards
  at load 4.5–9. The read table below uses that replication; the main rotation agrees.
  Mutations ran at load 5–16.
- **Cache state.**
  - Reads use root tables opened once per process in a new `Session` and warmed twice,
    so Lance metadata caches are warm.
  - Each mutation sample copies its table, reads every file and opens a new `Session`,
    so Lance caches start cold.
  - tmpfs keeps all data in memory. This says nothing about cold disk, network or S3
    reads.
- **Counters.** Process CPU time is available; instructions and cycles are not. The
  harness reads hardware counters only on macOS, and this host has
  `perf_event_paranoid=2` and no `perf`.
- **Excluded from timed phases:** function computation and correctness checks.

## Experiment A: partial masking

Every scan reads `summary` of all 1M rows and counts the visible values. The table shows
quiet-replication medians. A "fragment state" is how many fragments the flag is true on
wholly, partly or not at all; each stored-NULL table stores NULL in the same rows.

| Masked rows | Fragment states | Flagged ms (busy cores) | Stored-NULL ms | Flagged ÷ `one`, wall | Flagged ÷ `one`, CPU | Stored NULL ÷ `plain` | Flagged ÷ stored NULL | Summary MiB read, flagged / NULL |
| --- | --- | ---: | ---: | --- | --- | --- | --- | --- |
| 0% (`one`/`plain`) | 10/0/0 | 6.93 (7.9) | 6.91 | — | — | — | 1.00 [0.99, 1.01] | 13.92 / 13.90 |
| 0.1% scattered | 0/10/0 | 15.10 (4.1) | 7.22 | 2.18 [2.16, 2.19] | 1.14 [1.13, 1.15] | 1.04 [1.03, 1.05] | 2.10 [2.08, 2.11] | 13.91 / 13.87 |
| 1% scattered | 0/10/0 | 15.35 (4.1) | 7.31 | 2.21 [2.18, 2.23] | 1.14 [1.13, 1.15] | 1.06 [1.05, 1.07] | 2.09 [2.06, 2.11] | 13.82 / 13.81 |
| 10% scattered | 0/10/0 | 15.28 (4.1) | 7.22 | 2.20 [2.18, 2.22] | 1.14 [1.13, 1.16] | 1.04 [1.03, 1.05] | 2.12 [2.10, 2.14] | 13.79 / 13.18 |
| 50% scattered | 0/10/0 | 14.48 (4.2) | 6.56 | 2.08 [2.06, 2.11] | 1.12 [1.10, 1.14] | 0.95 [0.94, 0.96] | 2.20 [2.18, 2.22] | 13.59 / 9.32 |
| 100% (`masked`/`null_all`) | 0/0/10 | 0.87 (1.4) | 1.49 | 0.13 [0.13, 0.13] | 0.02 [0.02, 0.02] | 0.22 [0.21, 0.22] | 0.58 [0.57, 0.60] | 0 / 0 |
| 1% clustered | 9/1/0 | 6.93 (7.7) | 6.93 | 1.00 [0.99, 1.01] | 0.98 [0.97, 0.99] | 1.00 [0.99, 1.01] | 1.01 [1.00, 1.02] | 13.61 / 13.73 |
| 10% clustered | 8/2/0 | 7.12 (7.6) | 6.61 | 1.02 [1.01, 1.03] | 1.00 [0.99, 1.01] | 0.96 [0.95, 0.97] | 1.07 [1.06, 1.08] | 13.71 / 12.68 |
| 50% clustered | 4/2/4 | 5.00 (6.4) | 4.87 | 0.72 [0.72, 0.73] | 0.59 [0.59, 0.59] | 0.70 [0.69, 0.71] | 1.03 [1.02, 1.04] | 8.17 / 7.36 |

**Read requests and replication.** Each scan made one read request per fragment that
read its page: 10, or 6 for 50% clustered, and 0 when every fragment is masked or NULL.
The main rotation, under contention, agrees:

- scattered flagged ÷ `one` was 1.82–1.94× in wall time (intervals 1.64–2.13) and
  1.18–1.21× in CPU;
- clustered was 1.01, 1.03 and 0.70×;
- `one` ÷ `plain` was 0.99×.

**Why scattered masks cost what they do.** Each partly masked fragment adds per-batch
work in `CellFlagMasks::mask_batch`, after decode and before deletions:

- linear passes over each batch's physical offsets, for the span minimum and maximum
  and a contiguity check;
- `masked_in_span`, which iterates the batch's true rows in the roaring bitmap.

The scan drives batch futures with one `try_buffered` stream (`scan.rs:380`). Decode is
spread over the runtime's worker threads. The mask code runs inline whenever that stream
is polled (here on the main thread, which drives it through `block_on`), so it
serializes.

**What the profiles show** (`profiles/profile.md`):

- **Where the samples fall:** 55–58% of `scan_partial_1pct` and `scan_partial_50pct`
  samples are under `CellFlagMasks`. The offset passes take 29–37% of all samples and
  `masked_in_span` 12–24%. `nullif` and offset materialization take 0.3–0.4% each.
- **Which thread runs them:** 59–63% of partial-scan samples are on the thread polling
  the scan, against 17% for `one`.
- **Clustered layouts:** they skip almost every batch through the run fast paths, and
  only 2.4% of samples fall in mask code.

Sampling was biased toward the polling thread, so these shares attribute work within a
profile; they are not CPU proportions. The measured costs are +6.3–7.7 ms CPU and
+7.6–8.4 ms wall: almost all of the extra CPU lands on the critical path.

**Stale values versus stored NULLs** (from file metadata in `dataset/create.log`).
Every fragment stores `summary` in one page: a 100k-item FSST mini-block page, or a
constant page when the fragment is wholly NULL.

- **Flagged tables:** they keep the stale values of masked rows and use the all-valid
  layout, with no definition levels. They read and decode 13.6–13.9 MiB at every density,
  and masking discards the stale values after decoding.
- **Stored-NULL tables:** they use the nullable layout, which stores only the valid
  strings plus definition levels:
  - RLE-compressed levels at 0.1–1%, and 1-bit levels at 10–50%;
  - 13.2 MiB at 10% and 9.3 MiB at 50%;
  - wholly NULL fragments become constant pages with no buffers.
- **Where stored NULLs pay:** sparse NULLs decode about 4–6% slower than an all-valid
  page (stored NULL ÷ `plain` is 1.04–1.06), despite slightly fewer bytes.
- **Where flags win:** a wholly masked fragment skips IO and decode entirely through the
  NULL reader, faster than reading constant NULL pages (0.58×).

The flagged-to-NULL comparison is therefore not all mask CPU: at 50% it includes 4.3 MiB
more stale data to decode. The flagged ÷ `one` column compares identical logical values
in identical layouts. FSST output varies between identical writes: by up to 2.3% across
these tables, and by up to 9% across repeated single-fragment staged files.

**Allocations** (separate counting rotation, medians per scan):

| Scan | Allocations | Allocated MiB | Peak live growth MiB |
| --- | ---: | ---: | ---: |
| `one` | 71,910 | 152.6 | 20.1 |
| partial, scattered 0.1–50% | 73,089–73,101 | 156.7–160.4 | 22.3–23.9 |
| partial, clustered 1% / 10% | 71,964 / 72,088 | 149.7 / 152.1 | 20.2 / 20.4 |
| stored NULL, scattered 0.1–10% | 89,050–89,051 | 153.7–158.0 | 19.3–20.5 |
| stored NULL, 50% | 88,880 | 107.5 | 14.1 |
| `masked` / `null_all` | 7,673 / 11,974 | 4.7 / 7.1 | 0.8 / 0.9 |

Scattered partial masks add about 1,190 allocations per scan, 4–8 MiB of allocations
and 2–4 MiB of peak live growth. By source inspection (`mask_cells` in `fragment.rs`),
this includes 800 KB of physical offsets per partly masked fragment. The nullable decode
of stored NULLs allocates more objects than the all-valid decode.

**Inline flag state.** The manifest proper (`manifest_struct_bytes`, excluding the
inline transaction) is:

| Flag state | Manifest proper |
| --- | --- |
| fully true | 2.2 KiB |
| scattered 0.1% | 7.2 KiB |
| scattered 1% | 42 KiB |
| scattered 10% and 50% | 163 KiB |
| clustered | 2.3–2.7 KiB |
| fully false | 3.0 KiB |

Every open and commit reads and writes it. The scans here use already-open datasets, so
this cost is recorded but not timed.

## Experiment B: sparse publication

Each cycle writes `body` in place for 100 rows, which clears their flag, and then refreshes
`summary` for those rows. The k-fragment cases put the rows in k evenly spaced
fragments, 100/k scattered within each. The refresh variants are:

- **Stager:** `find` scans for pending rows, `read_inputs` takes their inputs, `stage`
  runs `PublicationStager` with whole-fragment copy-through, and `commit` publishes.
- **Plain replacement:** a filtered scan locates the rows; `write_columns` copies each
  touched fragment's column through with the rows swapped in; one `DataReplacement` is
  committed. Its stored bytes match the stager's.
- **`merge_insert`:** `RewriteColumns` on `id`.

Neither plain control provides the freshness guarantees of dependent publication.

The table gives main-rotation medians. "Refresh" is every phase after the source write;
peak live growth is for the stage, or for the merge in the `merge_insert` column.

| Touched fragments | Staged files | Staged MiB (`merge_insert`) | Bytes per assigned row | Stager stage ms (CPU) | Plain replace stage ms | `merge_insert` ms (CPU) | Commit ms, stager / plain | Refresh ms: stager / replace / merge | Peak live growth MiB, stage / merge |
| ---: | ---: | --- | ---: | --- | ---: | --- | --- | --- | --- |
| 1 | 1 | 1.37 (1.56) | 14,321 | 14.7 (18.4) | 13.5 | 24.8 (48.3) | 0.80 / 0.73 | 25.8 / 26.5 / 37.9 | 12.9 / 13.7 |
| 2 | 2 | 2.74 (3.16) | 28,698 | 30.6 (37.6) | 27.0 | 26.7 (67.9) | 0.80 / 0.75 | 43.1 / 40.1 / 39.2 | 12.9 / 26.8 |
| 5 | 5 | 6.92 (8.04) | 72,537 | 74.6 (93.1) | 67.8 | 32.3 (155.1) | 0.86 / 0.78 | 89.8 / 81.2 / 45.3 | 12.9 / 65.5 |
| 10 | 10 | 13.83 (16.12) | 145,029 | 144.1 (181.9) | 134.7 | 54.3 (318.9) | 0.87 / 0.83 | 163.1 / 148.7 / 67.9 | 12.9 / 103.4 |
| 10, random ids | 10 | 13.83 (16.13) | 144,980 | 146.2 (182.3) | — | 53.5 (314.3) | 0.90 / 0.83 (merge) | 165.7 / — / 67.4 | 12.9 / 103.5 |
| dense backfill, 1M rows | 10 | 13.83 | 14.5 | 126.1 (125.2) | 115.7 | — | 0.87 / 0.82 | 157.0 / 147.5 / — | 10.3 / 10.3 |

**Within-process ratios** (main rotation, wall time; CPU in parentheses):

| Touched fragments | Stage ÷ 1-fragment stage | Stager ÷ plain replace, stage | Stager stage ÷ `merge_insert` | Stager refresh ÷ `merge_insert` refresh |
| ---: | --- | --- | --- | --- |
| 1 | 1 | 1.09 [1.03, 1.13] (1.08) | 0.60 [0.58, 0.61] (0.38) | 0.68 [0.67, 0.69] |
| 2 | 2.07 [2.03, 2.11] | 1.12 [1.10, 1.15] (1.10) | 1.15 [1.13, 1.17] (0.56) | 1.10 [1.08, 1.11] |
| 5 | 5.10 [4.94, 5.25] | 1.11 [1.09, 1.13] (1.08) | 2.34 [2.29, 2.39] (0.60) | 1.99 [1.95, 2.03] |
| 10 | 9.75 [9.53, 9.99] | 1.06 [1.04, 1.08] (1.06) | 2.66 [2.61, 2.72] (0.58) | 2.42 [2.37, 2.47] |

The ten-fragment refresh of 100 rows stages longer than a dense backfill of all
1M rows: 1.14× [1.13, 1.16] in wall time and 1.45× in CPU.

**Shared amplification.** The bytes staged depend only on the touched fragments: about
1.37 MiB per touched 100k-row fragment of this output. Every whole-column replacement
pays this:

- the plain replacement writes the same files;
- `merge_insert` writes 14–17% more, because it rewrites the `id` key too;
- a dense backfill writes the same 13.8 MiB but assigns every row.

So output bytes per assigned row are 14,321 bytes at one fragment and 145,029 at ten,
against 14.5 for the backfill: amplification of 1,000× and 10,000×.

The source write has the same shape: the in-place `body` write rewrites 5 MiB per
touched fragment. It takes 53–120 ms and is identical for flagged and plain tables (ratio
0.99–1.01).

**Publication-specific cost.** On identical bytes the stager costs 1.06–1.12× the plain
replacement in wall time and 1.06–1.10× in CPU. Profiles under the stage show the
difference:

- the copy-through read of each touched, partly masked fragment takes the same
  per-batch mask path as Experiment A;
- `FragmentMerge::check_alignment` checks every row address;
- the merge itself uses the same `interleave`.

Pending-row discovery (7–13 ms, growing with partly true fragments), input reads
(2.6–4.5 ms) and commit (0.8–0.9 ms) are per-refresh costs that do not depend on output
width.

**Processing tradeoff.** The stager stages one fragment at a time and keeps peak live
growth at 12.9 MiB for any number of fragments, so wall time grows linearly at about
14.4 ms per fragment. `merge_insert` updates fragments concurrently:

- it is 2.3–2.7× faster in wall time at 5–10 fragments;
- it uses 1.7–1.8× the CPU and grows peak memory to 103 MiB at 10 fragments;
- at one fragment it is slower (stager ÷ `merge_insert` is 0.60×): it joins on the key
  and reads 3.8 MiB, against the stager's 1.4 MiB.

## Correctness and focused checks

- **Visible results.** Create mode checked that every flagged table and its stored-NULL
  pair read the same visible summaries (blake3 digests over every id and value).
  Verify mode repeated this after all rotations, and the digests and root identity were
  unchanged.
- **Bench assertions** (outside timed phases):
  - visible counts before and after each refresh;
  - published rows equal to the assigned rows, with nothing deferred;
  - the values of all 100 rows recomputed from current inputs;
  - k touched fragments for every concentrated cycle;
  - a stable visible count per read workload in every sample.
- **Driver checks.** File-level hashes of the dataset root matched before and after each
  rotation, and executable hashes matched the snapshots.
- **Clippy** on the bench target at both commits, with `-D warnings`: clean.
- **Formatting:** `cargo fmt --all -- --check` passes.
- **Focused tests:**
  `cargo test -p lance --profile ci --lib -- cell_flag fragment_write_columns`
  ran 481 passed, 0 failed, and 1 ignored. The ignored test is the existing fixture
  writer for the Python tests. These include the masking unit tests and the publication,
  stager and replacement suites.

## Conclusions

**Measured processing bottleneck**

1. **Scattered partial-mask scans.** They take 2.1–2.2× the wall time of identical
   unmasked bytes at every density from 0.1% to 50%: +7.6–8.4 ms per 1M rows, with
   +6.3–7.7 ms of CPU.
   - The cause is per-batch mask construction serialized on the scan's polling task.
     IO is identical and the mask math is not the cost: `nullif` and offset
     materialization are negligible.
   - Clustered masks cost 0–3%.
   - A processing change could plausibly recover most of the 2.1×, but none was tried.

**Processing tradeoff, not a bottleneck**

2. **Fragment-serial staging.** It bounds peak memory at the cost of linear wall time
   across touched fragments. Concurrency would lower latency but not bytes or CPU; the
   `merge_insert` comparison shows the ceiling and the memory cost.
3. **Publication-specific processing** costs 6–12% over an equivalent plain replacement.
   It is measured but too small to drive a decision.

**Representation costs**

4. **Whole-fragment publication bytes:** touched fragments × output column bytes,
   independent of the rows assigned. Ordinary whole-column replacement and
   `merge_insert` share this. It does not depend on flag processing, and faster code
   cannot reduce it.
5. **Stale values under partial masks** remain stored, read and decoded. Stored-NULL
   data shrinks with density; the flagged data does not until a rewrite replaces it.
   Stored NULLs are not a free baseline either: sparse NULLs decode 4–6% slower than
   all-valid pages.
6. **Inline flag state** grows to 163 KiB per flag at 1M rows for scattered partial
   states, against about 2–3 KiB clustered. Its open and commit cost was not timed here.
7. **Source-write amplification** applies to flagged and plain tables alike.

**Inconclusive or negligible**

8. **Differences of 3% or less:** `one` against `plain`, and clustered 1–10% masks
   against `one`. Identical-binary controls spread by 0.98–1.02, and FSST output varies
   by 2–9% between identical writes.
9. **Commit time:** the stager's commit is 1.05–1.08× the plain commits, a difference
   of 0.04–0.08 ms. It is resolved, but negligible.
10. **Absolute timings under load.** The main read rotation, at load 12–33, gave
    scattered ratios of 1.82–1.94× against 2.08–2.21× when quiet. The direction holds,
    but the magnitude depends on the cores available.
11. **Unmeasured:**
    - the proposed commit-time replacement validator, which is not implemented;
    - cold, network and S3 reads;
    - other fragment sizes;
    - wide and vector outputs;
    - open cost against flag-state size;
    - raced publications.

## Recommendation, ranked

1. **Make the publication-granularity decision against a stated budget.** Do this before
   more storage work.
   - **Does concentration help?** Yes, materially. Going from 10 fragments to 1 cuts
     staging 9.75×, bytes 10.1× and the refresh 6.3×, because cost follows touched
     fragments, not rows.
   - **Is that enough for a bounded first workload?** It makes the current
     representation reasonable for narrow, row-local scalar outputs whose refreshes
     concentrate in a few fragments. Examples are appends or partition-local updates:
     about 1.4 MiB and 15 ms of staging per touched fragment here.
   - **Where it is not enough:** the amplification per assigned row remains at least
     1,000× and scales with output width. CODING_PLAN's payload model puts a
     768-dimensional float32 output at about 300 MB per touched 100k-row fragment; that
     is a model, not a measurement here.
   - **What to write next:** for scattered sparse refresh, wide outputs or frequent small
     refreshes, write the row-selective publication design first. State the acceptable
     bytes and latency per publication.
2. **Optimize scattered partial-mask reads.** This is the one measured processing
   bottleneck. Candidate changes:
   - build masks off the scan's polling task, for example inside the decode task;
   - skip per-batch offset passes for contiguous range reads;
   - derive validity from roaring containers in bulk.

   Target scattered partial scans near the stored-NULL cost (1.04–1.08× of `plain`).
   Keep parent-only vector masking and current semantics, and re-measure with this
   harness.
3. **Consider bounded concurrent fragment staging** only if multi-fragment publication
   latency matters more than the stager's bounded-memory contract.
4. **Measure open and commit cost against inline flag-state size** for scattered partial
   states before any external-state decision.
5. **Measure the replacement validator** when it exists; its cost is unknown.
