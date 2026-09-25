# 10M full-scan sampling profile (supplementary)

macOS `sample` profiles of the regression harness's `scan_summary_full` at 10M rows / 100
fragments, one per build, taken 2026-09-25 between 09:33 and 09:40 UTC: after `10m-reads-reversed`
finished (09:05) and before the `10m-reads-variant-fragment-reverted` worktree was created (09:41),
with no benchmark running. This is not a paired run and yields no regression ratio.

| File | Content |
|---|---|
| `proto.top-of-stack.txt`, `base.top-of-stack.txt` | The heading and first 60 entries of `Sort by top of stack` from each `sample` report, with the Rust symbols demangled by `c++filt` (counts and libraries unchanged) |
| `proto.harness.log`, `base.harness.log` | The harness's stdout for each profiled process |

The full `sample` reports (about 6 MB each) and the JSONL records are not committed.

## Binaries

| Label | Binary | Build |
|---|---|---|
| `proto` | `cell_flags_regression-cc60bdbcaa7919a6` in the prototype worktree's `target/release-with-debug/deps/` | Nested build (rustflags twice). Linked 2026-09-25T08:50:43Z by the `1m-reads-maskfix` run at `e48573011`, whose code equals `26388225a`; the same binary ran `1m-reads-maskfix`, `10m-reads-maskfix` and `10m-reads-reversed`. |
| `base` | `cell_flags_regression-bff5b3f24f8697e9` in the baseline worktree | Baseline `e3671b2f5`, the baseline binary of every recorded run. |

## Command

As run, with `$S` the session's scratch directory, `P` the `proto` binary and `B` the `base`
binary:

```sh
D=$S/profile; rm -f $D/*.jsonl; for which in proto base; do bin=$P; [ $which = base ] && bin=$B; \
BENCH_BUILD=$which BENCH_SCALE_ROWS=10000000 BENCH_ROWS_PER_FRAGMENT=100000 BENCH_READ_SAMPLES=1500 \
BENCH_WARMUP=1 BENCH_WORKLOADS=scan_summary_full BENCH_DATA_DIR=$S/bench-data/profile-$which \
BENCH_OUT=$D/$which.jsonl $bin > $D/$which.log 2>&1 & pid=$!; \
until [ -f $D/$which.jsonl ] && [ $(wc -l < $D/$which.jsonl) -gt 100 ]; do sleep 1; done; \
sample $pid 15 -mayDie -f $D/$which.sample.txt > /dev/null 2>&1; wait $pid; echo "$which done"; done
```

`sample` attached once 100 records were written, 18.6 s after launch in both reports (their
`Launch Time` and `Date/Time`), and sampled every 1 ms for 15 s. The harness writes one record per
scan, `populated` `warm` first (1,500 scans of about 24 ms), so both profiles cover only
`scan_summary_full` `populated` `warm`.

## Frames compared

Counts from `Sort by top of stack`. Shares are of the non-wait samples: all listed entries except
`__psynch_cvwait`, `kevent`, `__psynch_mutexwait`, `__ulock_wait` and `__ulock_wait2` (idle
worker threads).

| Frame | `proto` samples | share | `base` samples | share |
|---|---|---|---|---|
| `fsst::decompress` | 20,152 | 32.1% | 18,870 | 32.5% |
| `_xzm_free` | 5,169 | 8.2% | 4,433 | 7.6% |
| `_platform_memmove` | 4,983 | 7.9% | 4,160 | 7.2% |
| `_platform_memset` | 4,368 | 7.0% | 4,317 | 7.4% |
| `pread` | 3,056 | 4.9% | 3,163 | 5.4% |
| `_xzm_xzone_malloc_tiny` | 2,885 | 4.6% | 2,221 | 3.8% |
| `VariableWidthDataBlockBuilder::append_validated` | 1,484 | 2.4% | 1,345 | 2.3% |
| `VariableWidthBlock::into_arrow` | 1,450 | 2.3% | 1,374 | 2.4% |
| `fsst::validate_offsets` | 1,415 | 2.3% | 1,338 | 2.3% |
| `DecodeMiniBlockTask::decode` | 1,372 | 2.2% | 1,135 | 2.0% |
| `core::str::from_utf8` | 1,340 | 2.1% | 1,273 | 2.2% |
| `BinaryMiniBlockDecompressor::decompress` | 994 | 1.6% | 970 | 1.7% |
| Non-wait samples | 62,830 | | 58,090 | |
| Wait samples | 280,143 | | 268,687 | |

Cell flag code in the call graphs (`grep -E 'CellFlag|cell_flag_mask|resolve_cells|merge_overlays'`;
a plain `cell_flag` search matches the binary name on every line): `proto` has two entries of one
sample each, `CellFlagMasks::resolve` and `FragmentReader::resolve_cells` (the prototype's wrapper
around `merge_overlays`); `base` has three one-sample entries in `FragmentReader::merge_overlays`.

## Harness medians while profiling

From `*.harness.log`, 1,500 scans per variant, one process per build run one after the other. The
sampler was attached during part of `populated` `warm`.

| variant | cache | `proto` med ms | `base` med ms |
|---|---|---|---|
| `populated` | warm | 24.33 | 24.53 |
| `populated` | fresh-session | 24.28 | 24.69 |
| `null_1pct` | warm | 26.33 | 26.53 |
| `null_1pct` | fresh-session | 26.96 | 27.09 |

## Caveats

- `proto` is the nested build, not the clean build of `10m-reads-clean-*` that shows the open
  regression; no clean-build binary was profiled.
- One 15 s profile per build, taken in sequence. The differences in share above are not tested
  for significance, and a 1 ms statistical profile cannot locate a few percent of wall time spread
  over many frames.
- The medians above come from one unpaired process per build and are not comparable with the
  paired rounds under `results/`.
