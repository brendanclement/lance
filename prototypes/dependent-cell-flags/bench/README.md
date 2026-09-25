# Dependent cell flag benchmarks

Benchmarks for the dependent cell flag prototype (option D). They answer three questions:

1. Do tables **without** flags get slower? The regression harness runs on the baseline and on the
   prototype, and the two are compared.
2. What do flags cost? On the prototype, the flag harness runs the same writes and reads with
   dependent masking flags registered, so flag invalidation and masking can be compared against
   the plain runs.
3. How much refresh work gets done while source writes race with it? This covers what gets
   published, what gets deferred, and how much the follow-up has to recompute versus reuse.

| File | Purpose |
|---|---|
| `rust/lance/benches/cell_flags_common/mod.rs` | Shared harness code: config, deterministic data, the simulated UDF, a permissive refresh, source writes, sample reset, IO/size accounting, the JSONL reporter. Uses `main` APIs only. |
| `rust/lance/benches/cell_flags_regression.rs` | Regression workloads on tables without flags, using `main` APIs only. |
| `rust/lance/benches/cell_flags_common/flags.rs` | Flag helpers, included only by the flag harness: flagged tables, the refresh executor, report accounting, correctness checks. |
| `rust/lance/benches/dependent_cell_flags.rs` | Flag workloads (`harness = false`). |
| `run_paired.sh` | Builds both worktrees and runs baseline, prototype and flag harnesses in alternating rounds. |
| `analyze.py` | Python standard library only. Turns the JSONL into markdown tables (`results/<scale>/analysis.md` and the `REPORT.md` sections). |

`mod.rs` and `cell_flags_regression.rs` must be **byte-identical** in the baseline and the
prototype worktree, and `run_paired.sh` refuses to run otherwise. Keep additions in `flags.rs`.

## Setup

The baseline is a worktree at the commit the prototype branched from, with the regression harness
files copied in. It is scratch and never committed:

```bash
git worktree add --detach ../lance-baseline e3671b2f5
cp -R rust/lance/benches/cell_flags_common ../lance-baseline/rust/lance/benches/
rm ../lance-baseline/rust/lance/benches/cell_flags_common/flags.rs
cp rust/lance/benches/cell_flags_regression.rs ../lance-baseline/rust/lance/benches/
# add to ../lance-baseline/rust/lance/Cargo.toml:
# [[bench]]
# name = "cell_flags_regression"
# harness = false
```

Each worktree must build into its **own** target directory, so leave `CARGO_TARGET_DIR` unset.
Cargo hashes workspace members by their path relative to the workspace root, so two worktrees
sharing a target directory silently overwrite each other's `lance` artifacts and bench binaries.
To avoid rebuilding every dependency, clone a built target with `cp -Rc` (APFS clonefile), then
`cargo clean --profile <p> -p <member>` for each workspace member in the copy.

## Running

Paired runs are the only runs whose baseline/prototype ratios mean anything. Scale presets:

| Scale | Rows / rows per fragment | Samples | Read samples | Rounds | Workloads |
|---|---|---|---|---|---|
| `smoke` | 100k / 10k | 3 | 3 | 1 | all |
| `1m` | 1M / 100k | 5 | 20 | 3 | all |
| `10m` | 10M / 100k | 3 | 20 | 1 | reads, clean refresh, sparse updates, publish_after_k |

```bash
cd <prototype worktree>
BASELINE_WORKTREE=/abs/path/lance-baseline BENCH_DATA_DIR=/abs/path/bench-data \
  prototypes/dependent-cell-flags/bench/run_paired.sh 1m
```

The script builds `cell_flags_regression` in the baseline worktree and `cell_flags_regression` +
`dependent_cell_flags` here, all with `--profile release-with-debug` and at `nice -n 15`. It checks
that each binary lies under its own worktree. Then, in each round, it runs baseline regression,
prototype regression and prototype flags, in that order. It writes:

- `results/<scale>/round<r>-<build>.jsonl` (raw samples) and `round<r>-<build>.log` (stdout
  median table)
- `results/<scale>/env.json`: `hw.model`, CPU, `hw.ncpu`, `hw.memsize`, the macOS version, `rustc -V`
  in both worktrees, both SHAs with their `git status`, the binaries and the config
- `results/<scale>/runs.tsv`: the start and end of each run, plus the load average before it
- `results/<scale>/analysis.md` and the `results:<scale>` section of `REPORT.md`

It refuses to write into a results directory that already holds rounds. Any preset value can be
overridden: `ROUNDS`, `BENCH_SAMPLES`, `BENCH_READ_SAMPLES`, `BENCH_WARMUP`, `BENCH_WORKLOADS`,
`BENCH_SCALE_ROWS`, `BENCH_ROWS_PER_FRAGMENT`, `BENCH_UDF_ITERS`, `BENCH_CONFLICT_KS` and
`BENCH_PUBLISH_KS`. The script also reads these:

| Variable | Default | Meaning |
|---|---|---|
| `BASELINE_WORKTREE` | required | Baseline worktree with the regression harness. |
| `BENCH_DATA_DIR` | `$TMPDIR/lance_cell_flags_bench` | Absolute path. Each harness uses and removes its own subdirectory. |
| `RESULTS_DIR` | `results/<scale>` | Where results go. |
| `BENCH_RUN_NICE` | `0` | Niceness of the benchmark runs. Builds always run at 15. |
| `FLAG_EVERY_ROUND` | `1` | `0` runs the flag harness in the last round only. |

To re-analyze existing results:

```bash
python3 prototypes/dependent-cell-flags/bench/analyze.py prototypes/dependent-cell-flags/bench/results/1m \
  --out prototypes/dependent-cell-flags/bench/results/1m/analysis.md \
  --report prototypes/dependent-cell-flags/bench/REPORT.md
```

To run one harness by hand, for example a quick dev-profile check:

```bash
nice -n 15 cargo build -p lance --bench dependent_cell_flags
BENCH_SCALE_ROWS=20000 BENCH_ROWS_PER_FRAGMENT=5000 BENCH_SAMPLES=2 \
BENCH_DATA_DIR=/abs/bench-data BENCH_OUT=/abs/bench-data/flags.jsonl \
  target/debug/deps/dependent_cell_flags-<hash>
```

Release builds refuse flagged datasets unless `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` is set. The flag
harness sets it itself before starting its runtime, and `run_paired.sh` sets it for the flag run
only. Records carry `build=prototype-flags` unless `BENCH_BUILD` is set.

Both harnesses read the same variables (`BenchConfig` in `cell_flags_common/mod.rs`):

| Variable | Default | Meaning |
|---|---|---|
| `BENCH_BUILD` | `unknown` (`prototype-flags` for the flag harness) | Label written to every record. |
| `BENCH_GIT_SHA` | null | Written to every record. |
| `BENCH_SCALE_ROWS` / `BENCH_ROWS_PER_FRAGMENT` | 1000000 / 100000 | Table size, and the `max_rows_per_file` that fixes the fragment count. |
| `BENCH_SAMPLES` / `BENCH_READ_SAMPLES` / `BENCH_WARMUP` | 5 / `BENCH_SAMPLES` / 1 | Recorded samples per mutation variant and per read variant, and unrecorded warmup samples. |
| `BENCH_WORKLOADS` | all | Comma list of names or `name_` prefixes. For example, `flagged_refresh_conflicts` selects every K. |
| `BENCH_DATA_DIR` / `BENCH_OUT` | under `$TMPDIR` | Absolute dataset directory (the harness wipes its subdirectory at start and removes it at the end) and JSONL output (appended). |
| `BENCH_UDF_ITERS` | 16 | Hash rounds per row of the simulated UDF. |
| `BENCH_CONFLICT_KS` / `BENCH_PUBLISH_KS` | `1,4,16` / `0,1,4,16,64` | K values of the conflict and publish-after workloads. |
| `BENCH_APPEND_ROWS` | 10000 | Rows of the regression `append` workload. |
| `BENCH_KEEP_DATA` / `BENCH_STABLE_ROW_IDS` | off | Keep the datasets / create tables with stable row ids. |

## Tables and flags

Both harnesses use the same "articles" table: `id`, `title` (16 B), `body` (64 B), `language`,
`views`, and the outputs `summary = f(title, body)` and `translation = g(body, language)`. Every
value is a SplitMix64 function of a fixed seed and `id`. The source columns are written with
`max_rows_per_file = BENCH_ROWS_PER_FRAGMENT`, so there are several fragments, and the outputs are
declared all-NULL through metadata only. The flag harness then registers **dependent masking**
flags, because masking is restricted to dependent flags:

- `groups=1`: `summary.ready`, with `clear_on_write = [title, body]` and `mask_when_false`
- `groups=2`: additionally `translation.ready`, with `clear_on_write = [body, language]` and
  `mask_when_false`. The two outputs share `body`.

This state is tagged `pending`. The harness then publishes each output with its own refresh and
tags the result `published`. Every sample restores one of the two tags (`restore` plus
`cleanup_old_versions`, metadata only and untimed). Each sample asserts that the fragments and
flag state equal the tagged state's, and the first sample of each workload also checks a content
fingerprint of all 7 columns, read masked.

The simulated UDF is `SimulatedUdf` (FNV-1a rounds, `BENCH_UDF_ITERS`). It is labeled in every
refresh record's notes, and its time is reported apart from the rest as `udf_ms`. Each run prints
its calibration in ns/row.

## Refresh executor

`flags::stage_pending` is written against the public API only:

1. Read the snapshot's flag state with `cell_flag_true_rows(flag_id)`. A fragment is a candidate
   when some physical row is not true.
2. Scan the fragment in offset order with `with_row_address()`, projecting `id` and the inputs,
   plus the output when some rows are already true. The live pending rows are the offsets that
   are not true. Offsets the scan skips are deleted rows.
3. Compute the pending rows with the UDF. With a `Reuse`, take the staged values the report marks
   reusable instead.
4. Copy each completed row through, as read via Lance, which means masked. That read also asserts
   that every pending row reads NULL, which checks masking on the fragment scan path.
5. Stage one full-fragment file with `FileFragment::write_columns`. Deleted offsets get NULL.
6. Commit `DataReplacement` + `CellFlagUpdate { value: true }` for exactly the live pending rows,
   as `Full` when that is every physical row. The commit uses
   `CommitBuilder::with_dependency_conflict_policy(..).execute_with_report(..)`.

With `keep_values`, the staged values are kept in memory per fragment, indexed by physical offset,
so a follow-up refresh can reuse what the report says is still valid.

## Workloads

Regression harness, on tables without flags. It runs on both builds, and every read result and
the clean refresh's values are asserted:

| Workload | Variants | What it measures |
|---|---|---|
| `append` | `default` | A 10k-row `InsertBuilder` append. `stage_ms` is the fragment write and `commit_ms` the commit. |
| `update_sparse`, `update_dense`, `update_unrelated_sparse` | `default` | `UpdateBuilder` (row-moving) of `body` on 100 scattered ids, of `body` where `id % 10 = 3`, and of `views` on the same 100 ids. |
| `merge_insert_partial_sparse` | `default` | In-place `merge_insert (id, body)` with `RewriteColumns` on 100 ids. |
| `refresh_permissive_clean` | `default` | Read + UDF + stage + plain `DataReplacement` commit of `summary`. |
| `refresh_permissive_conflicts_K` | `update_row_moving`, `merge_insert_in_place` | Stage at V, K source writes of 10 ids, then publish at V. Not correctness-equivalent: in-place writes publish stale summaries (`stale_rows`), and row-moving writes are rejected by main's rules. |
| `publish_after_k_commits` | `k=0,1,4,16,64` | The publication commit after K unrelated appends. |
| the five read workloads | `populated`, `null_1pct` × warm / fresh-session | The read queries below, on a populated summary and on one with 1% stored NULLs. |

Flag harness workloads:

| Workload | Variants | Start | What it measures |
|---|---|---|---|
| `flagged_update_{sparse,dense,unrelated_sparse,title_sparse}` | `groups=1,2` | published | `UpdateBuilder` (row-moving) on `body` for 100 scattered ids or `id % 10 = 3`, on `views`, or on `title`. Wall time, IO, manifest and txn bytes. `extra`: `flags_cleared` per output, `flags_carried` (true flags on the moved rows), `derived_invalidation_rows` and `moved_rows` from the transaction. `title_sparse` shows that shared inputs invalidate all and only the right outputs. |
| `flagged_merge_insert_partial_sparse` | `groups=1,2` | published | In-place `merge_insert (id, body)` with `RewriteColumns` on 100 ids. `stage_ms`, `commit_ms`, and the derived clears. |
| `flagged_refresh_clean` | `groups=1,2` | pending | One publication per output, run one after another. Each is read + UDF + assemble + stage + commit (`Reject`), with per-output times in `extra.per_output`. |
| `flagged_refresh_conflicts_K` | `{update_row_moving, merge_insert_in_place, merge_insert_output_in_place}:{reject,skip}` | pending | Stage a full `summary` refresh at V, commit K source writes of 10 scattered ids each, then publish at V. Records the outcome, `rows_published`, `rows_deferred_by_reason`, `fragments_deferred(_by_reason)`, `valid_rows_in_deferred_groups`, `rows_reusable`, and `commit_ms`, the conflict-checked commit. `merge_insert_output_in_place` writes `summary` itself, as an output override. It is the only source write that defers whole groups, so it is the case where reusing staged values matters. |
| `flagged_refresh_conflicts_K_followup` | `<conflict variant>:{recompute_all_pending, reuse_valid_staged}` | the conflicted publication's result | Refreshes to completion from the same post-publication state (tagged `followup`), once per strategy. Records `rows_recomputed`, `rows_reused`, `rows_copied_through`, `udf_ms`, `stage_ms` and `commit_ms`. `recompute_all_pending` ignores the report. `reuse_valid_staged` reuses `PublicationReport::reusable_rows`. `Reject` returns an error, so there is no report and nothing is known to be reusable. |
| `flagged_publish_after_k_commits` | `k=0,1,4,16,64` | pending | Stage at V, run K unrelated 100-row appends, then publish (`Reject`). `wall = commit_ms`, the conflict checks over K transactions. The appended rows stay pending. |
| `scan_summary_full`, `filter_summary_is_null_count`, `count_summary_vs_star` (`:aggregate`, `:sql`), `filter_id_range_project_summary`, `take_random_1k` | `all_true`, `partial_1pct`, `all_pending` × warm / fresh-session | separate `groups=1` tables | The regression harness's queries, on the masked `summary`. `partial_1pct` is published and then has the regression's `null_1pct` ids invalidated by one in-place `body` write, so it compares directly with `null_1pct`: stored NULLs against masked values. `all_pending` is registered but never published, so it has no summary data file. |
| `flag_state_size` | `groups=0,1,2` × `invalidated=0pct,0.1pct,1pct,10pct` | published | Manifest bytes, the invalidating write's txn bytes, the serialized true sets, and fresh-session open latency plus IO, after one in-place `body` write on that fraction of scattered rows. `groups=0` is an unflagged control with the same data files. |

The follow-up and conflict records share one sample index, so a conflict record and its two
follow-ups describe the same run.

## Correctness self-checks

Every sample asserts the following, so a broken prototype stops the run instead of reporting
plausible numbers:

- **Reads**: every result equals the expected masked answer, warmup included. Masked rows count
  as NULL in `IS NULL`, `COUNT(summary)`, scans, and `take`.
- **Source writes**: each flag that watches the written field loses exactly the written rows, and
  every other flag keeps them. Row-moving updates carry exactly the unwatched flags.
  `<output> IS NULL` counts exactly the false rows. `derived_invalidation_rows` equals the written
  rows for in-place writes. On the written rows plus 1000 scattered ids, a false flag reads NULL
  and a true flag reads the UDF of the row's current inputs.
- **Publications**: `check_report` verifies that the published, deferred, deferred-group-valid
  and removed-fragment rows together account for exactly the assigned rows. Published rows must
  be assigned and must not also be deferred, and published rows must be true after the commit.
- **Conflicts**: the outcome must match the README's conflict table. `Reject` must refuse with a
  retryable conflict and commit nothing. `Skip` must publish everything else, with the exact row
  or group deferrals expected. After the publication, no visible value on the changed ids or the
  1000 verification ids may differ from the UDF of the current inputs.
- **Follow-ups**: every flag of every row must be true, and every value must equal the UDF of the
  row's **current** inputs. This is a full scan, which catches a report that marked stale values
  reusable.
- **Clean refresh**: every row is published, and 1000 scattered ids hold current values.

## Output

Each record is one JSON line with the regression harness's shape: `build, git_sha, workload,
variant, scale_rows, rows_per_fragment, fragments, sample, wall_ms, udf_ms, stage_ms, commit_ms,
rows_written, rows_published, read_iops, read_bytes, write_iops, written_bytes, manifest_bytes,
txn_bytes, dataset_bytes, cache_state, stable_row_ids, udf_iterations, outcome,
stale_results_published, stale_rows, notes, extra`. For the flag harness, `rows_written` is the
number of rows written in the staged files.

`analyze.py` prints these sections, per (build, workload, variant, cache): n, median, min, max, and
p95 when n ≥ 20.

- **Environment**: the machine, the toolchain and the runs.
- **Regression**: baseline vs prototype. The ratio is paired per round, and the median of the
  per-round ratios is shown next to the individual rounds. Conflict rows are marked † as not
  correctness-equivalent.
- **Flag overhead**: prototype without flags vs `prototype-flags`, paired by round, covering
  updates, merge_insert, clean refresh, publish_after_k and reads. The source writes' flag effects
  are shown as well.
- **Refresh under concurrent source writes**, with the permissive baseline beside it, labeled not
  correctness-equivalent.
- **Follow-up**: rows computed, reused and recomputed, and `total UDF rows`. Anything above N rows
  is repeated computation.
- **Publication commit latency after K commits**.
- **Flag state size**.
- **Every group**: all groups, so no result is omitted.

## Limitations

- Warm and fresh-session only. The OS page cache is not controlled, so nothing here is a cold
  read. Local disk only.
- Use `release-with-debug` (thin LTO, 16 codegen units) and compare only builds with the same
  profile. Other processes on the machine add noise. `runs.tsv` records the load average before
  each run, and rounds alternate builds to spread out drift, but they cannot remove it.
- The refresh executor runs one fragment at a time on one task. `read_ms`, `udf_ms`,
  `assemble_ms` and `stage_ms` are sums, not overlapped. The flag executor's `udf_ms` covers only
  the UDF calls. The permissive `stage_refresh` also counts appending into the builder, a few ns
  per row.
- The follow-up copies completed rows through by reading the whole fragment, inputs included,
  because a publication stages full-fragment files. That full-fragment write footprint is part of
  the measurement, not overhead the harness adds.
- The flag harness uses stable row ids off, the default. `BENCH_STABLE_ROW_IDS` is untested here.
- `UpdateBuilder` has no uncommitted path, so the update workloads report only `wall_ms`.
- In the conflict workloads, `Skip` never defers whole groups for `update_row_moving` or
  `merge_insert_in_place`: rows are deferred and the group still installs. For those writes the
  two follow-up strategies recompute the same rows, and only `merge_insert_output_in_place` shows
  what reuse saves. Under `Reject` there is no report, so reuse has nothing to work with.
- `all_pending` has no summary data file, since it was never refreshed. It measures the
  fresh-column state, not a fully masked data file.
- Flag state is persisted as the bitmaps the commit ends up with. The executor builds assignment
  bitmaps one offset at a time, which gives roaring array or bitmap containers rather than runs,
  and assigns `Full` only when a fragment has no deletions. Manifest bytes after a publication on
  a fragment with deletions therefore reflect that encoding.
