# Feature costs of the current prototype

**Status: measured once, on one laptop, on 2026-09-30.** Every number here is a measurement of the
current prototype (code `b56278dc8`, benchmark `dc98f0c06`) on one Apple M5 Pro running macOS,
with a local SSD and a warm page cache, in the `release-with-debug` profile. None of it describes
production builds, object storage such as S3, or cold caches, and none of it is a performance
acceptance test.

Output values were computed outside every timed phase, so no number includes the function's own
computation.

## Cost table

Rows are at 1M rows in 10 fragments unless they say otherwise. "Flagged" and "comparison" are
median milliseconds, with the range of per-process medians in parentheses (the run-to-run
variation). A ratio is the geometric mean over processes of the per-process ratio of medians,
with a bootstrap 95% interval. There are 16 processes per comparison: 8 in the 40-fragment and 4M
scaling runs, and 6 per build for the no-flag reads. Identical-binary controls agree within
±3.7% per workload and ±4.3% per phase at 1M rows, and within 0.94–1.08 per phase in the scaling
runs. Sizes are KiB and MiB.

The cause column says where a cost comes from:
- **semantics**: what dependent flags require under any representation;
- **manifest**: flag state and flag changes stored inline in the manifest and transaction;
- **whole-fragment**: publishing through whole-fragment `DataReplacement` files;
- **implementation**: the prototype's code;
- **caller**: code the refresher runs.

Evidence labels:
- **strong**: the effect repeats in every process with a tight interval, and the recorded bytes,
  allocations or CPU pin down the mechanism;
- **strong (timing)**: the timing is that solid but the mechanism is inferred;
- **weak**: not consistent between rotations.

| Operation | Flagged | Comparison | Ratio | Likely cause | Evidence | Grows with |
|---|---|---|---|---|---|---|
| Read without flags, scan, 10M rows, against `main` | 25.91 (24.57–26.52) | 25.33 (24.60–25.97) | 1.01 [0.98, 1.04]; filter and count 1.02; 0.99–1.00 in the counting rotation | none found: +0.1% instructions, identical allocation count, volume and peak, and the sign flips between rotations | weak (unresolved) | not measured |
| Read, all flags true, whole fragments, scan | 2.96 (2.88–3.08) | 3.02 (2.92–3.12) no flags | 0.98 [0.98, 0.99]; filter and take 0.99 | whole-fragment states skip masking | no effect beyond noise | — |
| Read, all flags true, fragments lost rows (1% moved), scan | 15.96 (15.60–16.13) | 9.21 (9.01–9.37) same moves, no flags | 1.73 [1.73, 1.74]; filter 1.72; take 1.00 (both 1.6 s, see Not measured) | implementation: a per-row mask is built from the row-set state and applied though nothing is masked; manifest 41 KiB against 2.3 KiB | strong (timing) | not measured at other scales |
| Read, 1% masked, scan | 5.50 (5.41–5.62) | 3.25 (3.17–3.36) 1% NULL | 1.69 [1.67, 1.70]; filter 1.65; take 0.97. Against no NULLs (3.02 ms): 1.83, 1.85, 0.99 | implementation: the mask runs after decoding, on the critical path (5.0 busy cores against 9.4, with less CPU in total); same bytes read and allocation volume | strong (timing) | rows: +2.4 ms at 1M, +9.0 ms at 4M (40 fragments); not fragments (1.64× in 40) |
| Read, 50% masked, scan | 5.94 (5.71–6.08) | 3.19 (3.09–3.42) 50% NULL | 1.85 [1.83, 1.87]; filter 1.83; take 0.99. Against no NULLs: 1.97, 2.76, 0.99 | as above, plus decoding the stale values that NULL storage omits (13.6 against 9.3 MiB read, 157 against 107 MiB allocated) | strong (timing) | not measured |
| Read, all masked, scan | 3.05 (2.90–3.23) | 1.01 (0.96–1.04) all NULL | 3.02 [2.97, 3.06]; filter 2.50; take 16.0. Against no NULLs: 1.01, 1.00, 0.98 | implementation: the stored column is decoded, then replaced (13.7 against 0 MiB read, 154 against 7 MiB allocated) | strong | rows (inferred) |
| Read vectors, 1% masked, scan (1M × 128 Float32) | 12.92 (12.87–12.97) | 12.77 (12.73–12.83) all valid | 1.01 [1.01, 1.01]; take 1.05; flat search 14.5 (219 against 15.1 ms). Against 1% NULL vectors: 0.46, 0.99, 0.95. At 50%: scan 1.06 and 0.51, flat search 7.7 and 0.92 | masking nulls list slots only. Any NULL vector, masked or stored, sends flat search to a slow path (1.5 busy cores against 13.7). Stored NULL vectors decode slower (2.4 GiB allocated against 0.96 GiB). Both are in code the prototype shares with `main` (`main` not run) | strong | not measured |
| Read vectors, all masked, scan | 14.12 (14.02–14.26) | 3.65 (3.60–3.69) all NULL | 3.87 [3.86, 3.89]; flat search 2.55; take 17.1 | implementation: stored vectors are decoded, then replaced | strong | rows (inferred) |
| Unrelated write, 100 rows, row-moving | 16.54 (15.58–17.32) | 16.55 (15.56–17.13) no flags | 1.00 [1.00, 1.01] for one output; 1.00–1.01 for two; in place 0.99–1.00 | none measurable. Flags move with moved rows: +0.8–1.5 KiB of manifest, and moving 1% of rows makes a 30 KiB transaction against 2 KiB | no effect beyond noise | manifest: moved rows × flags |
| Source write, sparse, 100 rows, row-moving | 16.56 (15.70–17.19) | 16.54 (15.54–17.10) | 1.00 [1.00, 1.01]; in place 1.00 | none measurable | no effect beyond noise | — |
| Source write, dense, 100k rows moved | 75.29 (74.58–75.99) | 60.43 (59.33–62.10) | 1.24 [1.24, 1.25]; two outputs 1.25 | semantics and implementation, not separated: clearing and recording each moved row, +14 ms of serial CPU at equal allocation volume and bytes. Manifest: 163 KiB of state per flag, and a 166 KiB transaction | strong (timing) | rows invalidated (one count measured); manifest × flags |
| Find pending rows (the documented live scan) | 4.1–18.3 at 1M, one flag; 8.0–14.7, two flags; 16–37 at 4M | — | — | caller: enumerates every live row address (no data read) and checks each flag per row, whatever the number pending | strong | rows, deletions, flags |
| Stage pending rows that sit in a new fragment, 100 rows | 0.56 (0.54–0.59) | 11.11 (10.93–11.42) `merge_insert` | 0.05 [0.05, 0.05] | the stager writes the 100-row fragment only; `merge_insert` joins over the table | strong | — |
| Same, 100k rows (dense) | 25.23 (25.02–25.48) | 24.34 (24.14–24.76) | 1.04 [1.03, 1.04]; two outputs 1.52; 1.52 at 4M (132 against 87 ms); 0.88 at 1M in 40 fragments | implementation: the stager's per-row path, while allocating a third as much (80 against 232 MiB) | strong (timing) | pending rows × outputs |
| Stage 100 rows scattered in place | 65.55 (64.51–66.39) | 23.94 (23.60–24.26) `merge_insert` `RewriteColumns` | 2.74 [2.72, 2.75]; two outputs 2.58–2.63; 4.08 at 1M in 40 fragments; 3.50 at 4M | whole-fragment: both rewrite every touched fragment (13.8 and 16.1 MiB written). Implementation: the stager takes 41.6 ms longer while writing and allocating less (peak 12.9 against 102.6 MiB) | strong (timing) | rows rewritten and fragments touched: 65.6, 119 and 234 ms at 1M in 10, 1M in 40 and 4M in 40 |
| Copy unchanged rows (in-place stage less the new-fragment stage; not timed alone) | ≈ 65 at 1M in 10 fragments; 118 in 40; 233 at 4M | 0.56 (the 100 rows alone) | ≈ 117× | whole-fragment and implementation: 13.7 MiB read and 13.8 MiB written per output for 100 new values | moderate (derived) | rows × fragments touched |
| Backfill stage, every row computed | 186 (185–187) | 53.76 (52.83–54.38) `write_columns` | 3.46 [3.44, 3.47]; two outputs 5.24–5.37 | implementation: the stager's computed-row path. Same bytes written; 275 against 240 MiB allocated. Copy-through costs 0.24–0.35× of it per fragment | strong (timing) | rows (inferred) × outputs |
| Commit a publication (conflict checks and publishing; the stager's own validation runs in stage) | 0.93 (0.92–0.97) | 0.92 (0.90–0.95) plain commit | 1.01 [1.00, 1.02] sparse; dense 1.06 (one output), 1.14–1.15 (two) | semantics (checks) plus manifest (writing 163–323 KiB of state) | strong (timing) | manifest size |
| Complete cycle, in place, one output | 142 (140–143) | 101 (99.9–103) plain cycle | 1.40 [1.40, 1.40]; two outputs 1.47–1.49 | the in-place stage (+41.6 ms), and reads while pending (1.5×) | strong | as staging in place |
| Complete cycle, row-moving dense, one output | 133 (132–134) | 114 (113–117) | 1.16 [1.16, 1.17]; two outputs 1.30 | the source write (+14.9 of +19 ms); with two outputs, staging the second (+13.9 ms) | strong | rows invalidated × outputs |
| Complete cycle, row-moving sparse, one output | 51.12 (49.43–52.17) | 61.18 (59.26–62.60) | 0.84 [0.83, 0.84]; two outputs 0.96 | staging a new fragment beats `merge_insert`'s join; reads while pending and after publication cost 1.19–1.34× | strong | — |
| Race, `Reject`: the retry after one in-place source write | 90.94 (89.66–92.16) | 90.78 (89.73–92.22) the first attempt | 1.00 [1.00, 1.00]; 1.00–1.06 for other races; 0.97–0.98 at 40 fragments | semantics: restage the rejected publication and recompute the raced rows (10–80). Caller: this retry also re-finds and recomputes all 1,000 pending rows, 2,000 in all. Whole-fragment: restaging rewrites every touched fragment | strong | the whole refresh |
| Race, `Skip`: the follow-up after one in-place source write | 57.85 (56.78–58.70) | 90.85 (89.92–92.72) the first attempt | 0.64 [0.63, 0.64]; eight raced commits 0.72; chain 0.59; row-moving race 1.01; 0.20–0.22 at 40 fragments | semantics: recompute the raced rows (1,010–1,080 recomputed in all). Whole-fragment: restage each fragment holding a raced row. Caller: after a row-moving race, the scan for moved rows (17.9 ms) dominates | strong | fragments the race touches; rows (scan) |
| Publish after 32 unrelated commits | 1.49 (1.46–1.56) | 1.50 (1.44–1.54) `merge_insert` commit | 1.00 [0.99, 1.02]; 1.01 after 0 and 8 | conflict checking: 1.01, 1.13 and 1.49 ms after 0, 8 and 32 commits, with or without flags | no effect beyond noise | intervening commits (not flag-specific) |
| Reopen after 64 sparse updates | 0.279 (0.269–0.297) | 0.248 (0.237–0.261) same history, no flags | 1.13 [1.12, 1.15]; after 16: 1.31; after a dense one: 1.27; after 64 refreshed: 1.15 | manifest: every open parses the flag state (1.3 MiB allocated at 163 KiB). The +0.02–0.05 ms does not scale with its size | strong for size; moderate for time | size: moved rows × flags, over commits; time: not resolved |
| Commit after that history (append) | 1.43 (1.39–1.45) | 1.43 (1.40–1.46) | 1.00 [0.99, 1.01]; other histories and sparse updates 0.96–1.05 | flag state up to 163 KiB is not measurable in a local commit | no consistent effect (both signs, within ±5%) | not measurable at ≤ 163 KiB locally |

## Bytes and memory

Flagged / comparison, medians. Bytes and manifest sizes come from the timed rotations. Allocation
volume and peak live growth come from the counting rotation, and count requested capacity. Peak is
the most bytes live at once above those live when the sample or phase started; volume is the
total requested, which peak does not bound. The manifest column is the manifest proper, without the
inline transaction, of the version the sample ends at.

| Operation | Read MiB | Written MiB | Allocated MiB | Peak MiB | Manifest KiB |
|---|---|---|---|---|---|
| Scan, 1% masked / 1% NULL | 13.6 / 13.8 | 0 / 0 | 156 / 157 | 22.4 / 19.1 | 42.0 / 3.0 |
| Scan, 50% masked / 50% NULL | 13.6 / 9.3 | 0 / 0 | 157 / 107 | 22.3 / 13.2 | 163.3 / 3.0 |
| Scan, all masked / all NULL | 13.7 / 0.0 | 0 / 0 | 154 / 7 | 18.7 / 0.7 | 3.0 / 3.0 |
| Scan, all true after moves / no flags | 13.8 / 14.1 | 0 / 0 | 251 / 249 | 32.2 / 28.7 | 41.4 / 2.3 |
| Dense source write, 100k rows | 110.2 / 110.4 | 11.8 / 11.3 | 4971 / 4979 | 1205 / 1209 | — |
| Stage in place / `merge_insert` | 13.7 / 18.5 | 13.8 / 16.1 | 433 / 552 | 12.9 / 102.6 | — |
| Stage dense / `merge_insert` | 1.4 / 2.5 | 1.4 / 1.6 | 80 / 232 | 10.3 / 27.9 | — |
| Backfill stage / `write_columns` | 0 / 0 | 13.8 / 13.8 | 275 / 240 | 10.3 / 10.3 | — |
| Complete cycle, in place | 95.7 / 102.7 | 64.7 / 66.9 | 2096 / 2229 | 224 / 224 | 3.9 / 3.9 |
| Complete cycle, dense, one output | 148.7 / 152.7 | 13.3 / 12.9 | 7101 / 7335 | 1205 / 1209 | 162.8 / 2.4 |
| Complete cycle, dense, two outputs | 181.1 / 183.4 | 15.1 / 14.3 | 9031 / 9213 | 1206 / 1206 | 323.1 / 2.4 |
| `Reject` retry / first attempt | 23.1 / 23.2 | 13.8 / 13.8 | 553 / 548 | 15.1 / 15.2 | — |
| `Skip` follow-up / first attempt | 12.7 / 23.2 | 12.5 / 13.8 | 379 / 548 | 13.0 / 15.2 | — |

## The complete cycle, by phase

One output, 1M rows in 10 fragments; median milliseconds, flagged / plain. The plain cycle finds
its rows with a filter (in the read-inputs row) and writes with `merge_insert` (in the stage row).
Function time is excluded, so the phases do not sum to the total, whose median is taken separately.
`summary.md` has each phase's range and interval.

| Phase | In place, 100 rows | Row-moving dense, 100k rows | Row-moving sparse, 100 rows |
|---|---|---|---|
| Update source rows | 59.34 / 59.45 | 75.29 / 60.43 | 16.56 / 16.54 |
| Read while pending | 4.65 / 3.09 | 10.43 / 8.85 | 11.36 / 9.29 |
| Find pending rows | 4.10 / — | 8.12 / — | 10.72 / — |
| Read their inputs | 4.08 / 10.97 | 2.90 / 11.41 | 0.32 / 14.25 |
| Stage precomputed results | 65.55 / 23.94 | 25.23 / 24.34 | 0.56 / 11.11 |
| Publish | 1.02 / 1.03 | 1.03 / 0.97 | 0.93 / 0.92 |
| Read again | 2.95 / 2.94 | 9.90 / 8.30 | 10.79 / 9.01 |
| Total | 142 / 101 | 133 / 114 | 51.1 / 61.2 |

## Costs by source

- **Required by the semantics:** invalidating dependent flags on source writes, whose cost grows
  with the rows written (the prototype's +15 ms per 100k moved rows is not split from its own
  overhead); recomputing raced rows; checking a publication against intervening commits, which
  costs the same without flags.
- **Inline manifest storage:** the state kept after moves and clears. 1% cleared in place
  leaves 42 KiB against 3 KiB; 50%, 163 KiB; a dense update, 163 KiB per flag; 64 sparse updates,
  34 KiB for one flag and 59 KiB for two. Holes at moved rows are permanent without compaction.
  Transactions that carry flag changes are large too: 42 KiB for 1% cleared in place, 323 KiB for
  50% or all, 166 KiB for a dense update. Lance writes each twice, as the transaction file and
  inline in the manifest. Locally the state adds 0.02–0.15 ms to opens and commits. Every open
  reads it and every commit rewrites it; its cost on object storage and at larger sizes was not
  measured.
- **Whole-fragment replacement:** publishing 100 scattered values rewrites the whole column of
  every touched fragment, 13.8 MiB per output at 1M rows. Plain Lance writes 16.1 MiB for the
  same in-place `merge_insert` in `RewriteColumns` mode. In `RewriteRows` mode it writes 0.03 MiB,
  but reads 96 MiB and moves the rows, which leaves later scans 3.2× slower. `Skip` follow-ups pay
  the rewrite again for each fragment a race touches.
- **Prototype implementation:** the stager's copy-through and computed-row paths (2.6–2.7× and
  3.5–5.4× the plain writes); a per-row mask applied after decoding, also over row-set states
  where nothing is masked; decoding fully masked columns.
- **Caller code:** the documented pending-row scan, which enumerates every live row, and the
  `Reject` retry's choice to recompute every pending row rather than only the raced ones.

### What grows with what

| Driver | Measured growth |
|---|---|
| Rows | masked-read overhead (+2.4 ms at 1M and +9.0 ms at 4M, both in 40 fragments); the pending-row scan (9.6 → 36.6 ms at 40 fragments); in-place staging (119 → 234 ms at 40 fragments); dense staging's ratio (0.88× → 1.52× at 40 fragments) |
| Fragments at fixed rows (10 → 40) | in-place staging (65.6 → 119 ms, against 23.9 → 29 ms for `merge_insert`); `Skip`'s follow-up share falls (0.64× → 0.22×) because a race touches fewer of them; the masked-read ratio does not grow (1.69× → 1.64×) |
| Dependencies (one output, two sharing an input, a two-level chain) | manifest state doubles with the second flag; staging two outputs takes 1.6× as long as one in dense refreshes and 1.21–1.25× in place (the chain 3% below shared); the backfill ratio rises from 3.5× to 5.2–5.4×; the dense update's added time barely changes (+14.9 → +15.3 ms); two sharing an input and a chain otherwise cost the same |
| Intervening commits | publication commit 1.01 → 1.49 ms after 0 → 32 appends, the same as plain Lance; flag state after 16 and 64 sparse updates 10 and 34 KiB (one flag); opens 0.22 and 0.28 ms, against 0.16 and 0.25 ms for plain tables with the same history |

## Not measured

- Production-like builds: the Linux wheels use thin LTO with 1 codegen unit, `haswell` on
  x86_64 and the `metrics` features. Linux and x86_64 were not run.
- Object storage (S3 and others): request counts and bytes are recorded; latency is not.
- Cold caches, memory pressure, and resident memory. Allocation figures count requested bytes,
  and every manifest write reserves a 5 MiB buffer, which dominates small commit phases.
- Validation on its own: the stager's layout and chain checks run inside `stage`, and the
  conflict checks inside `commit`; neither was timed separately.
- Dense in-place source writes and their refresh (dense writes were only row-moving), and more
  than one dense invalidation count.
- Tables above 4M rows, wider rows, more than two flags, chains deeper than two, histories
  longer than 64 commits, and repeated dense invalidations.
- Parallel writers: the races are sequential commits between staging and publication, from one
  process.
- Vector refreshes (only vector reads), masked reads through filters on other columns, late
  materialization and SQL, and the real function's cost.
- Index maintenance and compaction, which the prototype refuses.
- `main` for the vector reads. Two costs seen along the way are in code shared with `main` and
  were not investigated: flat search over NULL vectors, masked or stored (15 ms all valid;
  219 ms 1% masked, 230 ms 1% NULL), and a `take` of 1,000 rows from a table with 1% of rows moved
  (1.6 s, with or without flags).

## Setup

**Machine.** Apple M5 Pro (18 cores, 48 GiB), macOS 26.7, local SSD. Background system daemons
(`duetexpertd`, `knowledgeconstructiond`, an endpoint security agent) ran throughout, and the
1-minute load average at process start ranged 1.9–12.2 (median 5.1; `runs.tsv` in each
directory). Every flagged/plain comparison interleaves its two sides within each process. The
no-flag comparison against `main` and the identical-binary control compare separate processes run
back to back in rotated order. The control bounds how much that noise can bias a ratio.

**Builds.** `release-with-debug` (thin LTO, 16 codegen units) with the checkout's
`target-cpu=apple-m1` rustflags. `final` is `dc98f0c06` (code `b56278dc8` plus the benchmark);
`final-copy` runs the same executable again. For the no-flag reads, `main` is the prototype's
baseline `e3671b2f5` with `cell_flags_scan_counters.rs` and `cell_flags_common/{mod,counters,alloc}.rs`
copied in, byte-identical to `final`'s (`run_costs.sh` checks). `run.json` in each directory
records each executable's sha256, source revision, patch and build settings.

**Tables.** `cell_flags_costs` create mode, as its module documentation describes: articles rows
with three text outputs, `summary = f(title, body)`, `translation = g(body, language)` and
`keywords = h(summary, language)`, stored by one `DataReplacement` that publishes every flag. The
topologies differ only in which outputs carry a dependent masking flag: `plain` none, `one`
`summary`, `shared` `summary` and `translation` (sharing `body`), `chain` `summary` and
`keywords`. Create mode checks that each flagged read table reads the same visible summaries as its
plain pair. Roots: `costs-1m` (28 tables, 1M rows in 10 fragments), `costs-1m-40frag` and
`costs-4m` (six tables, 1M rows or 4M rows in 40 fragments), and `articles-10m` (10M rows in 100
fragments, no flags). The vector tables are those of `../vector-masking/` (1M × 128 Float32).

**Samples.** Each mutating sample clones its table with APFS `clonefile`, reads every file once
so the page cache holds it, and opens it in a new `Session`. Each sample is one or more timed
phases. Every phase records wall and CPU time, retired instructions and cycles, and object-store
bytes and requests. With allocation counting on (a separate rotation) it also records allocation
volume and peak live growth. Every sample records the metadata sizes of the version it ends at:
the manifest file, the manifest proper, and the inline and separate transaction copies. The
module documentation of `rust/lance/benches/cell_flags_costs.rs` defines each workload and phase.
A plain refresh finds its rows with a filter (`locate`), which the flagged refresh does with
`find` and `read_inputs`, and writes with `merge_insert` (`merge`) where the flagged refresh
stages.

| Directory | Workloads | Labels | Rounds | Samples (warm-up) | Counting |
|---|---|---|---|---|---|
| `reads/` | 30 scalar reads | `final`, `final-copy` | 8 | 30 (2) | off |
| `mutations/` | 95 writes, refreshes, races, histories | `final`, `final-copy` | 8 | 10 (2) | off |
| `allocations/` | all 125 | `final`, `final-copy` | 2 | 4 (2) | on |
| `scaling-4m/`, `scaling-1m-40frag/` | 32 of them | `final`, `final-copy` | 4 | 6 (2) | off |
| `vector/`, `vector-allocations/` | 24 vector reads | `final`, `final-copy` | 8 and 2 | 20 (2), 4 (2) | off, on |
| `noflag/`, `noflag-allocations/` | 3 reads of `articles-10m` | `main`, `final`, `final-copy` | 6 and 3 | 30 (2), 4 (2) | off, on |

Allocation figures come from the counting rotations and everything else from the timed ones.
Workloads run in a rotated order, reversed on every other pass, so each flagged workload and its
plain pair run in both orders.

## Commands

From the repository root, with `S` a scratch directory outside any checkout:

```bash
git worktree add --detach "$S/final" dc98f0c06
git worktree add --detach "$S/main" e3671b2f5
for f in cell_flags_scan_counters.rs cell_flags_common/mod.rs cell_flags_common/counters.rs cell_flags_common/alloc.rs; do
  mkdir -p "$(dirname "$S/main/rust/lance/benches/$f")"
  cp "rust/lance/benches/$f" "$S/main/rust/lance/benches/$f"
done
# add to $S/main/rust/lance/Cargo.toml:
# [[bench]]
# name = "cell_flags_scan_counters"
# harness = false
export FINAL="$S/final" MAIN="$S/main" ROOTS="$S/roots" OUT="$S/results" VECTOR_TABLES=/abs/vector-masking-tables
for step in create reads mutations allocations scaling vector noflag; do
  prototypes/dependent-cell-flags/bench/run_costs.sh "$step"
done
```

Each directory's `analysis.md` comes from `analyze_counters.py` and `summary.md` from
`summarize_costs.py`:

```bash
B=prototypes/dependent-cell-flags/bench
D=$B/results/feature-costs
python3 $B/analyze_counters.py $D/reads --reference final --builds final-copy --spread > $D/reads/analysis.md
python3 $B/summarize_costs.py $D/reads --allocations $D/allocations --control > $D/reads/summary.md
python3 $B/analyze_counters.py $D/mutations --reference final --builds final-copy --spread --only '^[^.]*$' > $D/mutations/analysis.md
python3 $B/summarize_costs.py $D/mutations --allocations $D/allocations --control > $D/mutations/summary.md
python3 $B/analyze_counters.py $D/noflag --reference main --builds final,final-copy --spread > $D/noflag/analysis.md
```

The scaling, allocation and vector directories follow the same pattern. The vector analyses add
`--pair <read>_<flagged>:<read>_<plain>` for each read and each pair: `ready:plain`,
`partial_1pct:null_1pct`, `partial_1pct:plain`, `partial_50pct:null_50pct` and `masked:null_all`.

## Files

Each directory holds the driver's `run.json`, `order.tsv` and `runs.tsv`, the raw
`round<r>-<label>.jsonl` records, the warm-up records, logs, and the source patches of each
build, but not the executables.
