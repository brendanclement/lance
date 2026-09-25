## Control: 10m reads, prototype with `fragment.rs` reverted to the baseline (read path unchanged), baseline first

Source: `results/10m-reads-variant-fragment-reverted`

### Environment

| key | value |
|---|---|
| scale | `10m` |
| profile | `release-with-debug` |
| hw_model | `Mac17,8` |
| cpu_brand | `Apple M5 Pro` |
| ncpu | `18` |
| memsize_bytes | `51539607552` |
| macos | `26.6.2 (25G83)` |
| rustc_baseline | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| rustc_prototype | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| baseline_sha | `e3671b2f5730eea927a088a42cbf30e273edc43c` |
| prototype_sha | `26388225a273052b3a1ff0ec705c53da1b68c7cb` |
| rounds | `3` |
| flag_every_round | `True` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/run_paired.sh; M  rust/lance/src/dataset/fragment.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T09:51:36Z	2026-09-25T09:52:19Z	20.90 16.81 11.45
1	prototype	2026-09-25T09:52:19Z	2026-09-25T09:53:04Z	12.60 15.14 11.11
2	baseline	2026-09-25T09:53:04Z	2026-09-25T09:53:47Z	8.12 13.57 10.74
2	prototype	2026-09-25T09:53:47Z	2026-09-25T09:54:30Z	6.57 12.47 10.46
3	baseline	2026-09-25T09:54:30Z	2026-09-25T09:55:13Z	6.65 11.61 10.24
3	prototype	2026-09-25T09:55:13Z	2026-09-25T09:55:57Z	5.27 10.50 9.90
```

Records: 2880. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.51 | 22.77 | 25.47 | 25.34 | 60 | 24.82 | 23.62 | 26.46 | 26.30 | 1.060 | 1.060 1.000 1.113 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 25.77 | 25.07 | 27.16 | 27.03 | 60 | 26.77 | 25.73 | 28.46 | 28.16 | 1.028 | 1.028 1.002 1.089 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.25 | 23.35 | 26.31 | 25.31 | 60 | 25.16 | 24.07 | 26.50 | 26.30 | 1.025 | 1.025 1.006 1.090 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.23 | 31.79 | 35.58 | 34.99 | 60 | 33.92 | 32.37 | 35.84 | 35.15 | 1.025 | 1.025 0.984 1.049 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.93 | 12.32 | 13.77 | 13.32 | 60 | 12.91 | 11.92 | 13.56 | 13.51 | 0.975 | 0.966 0.975 1.041 | 24,799 | 24,799 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.60 | 9.92 | 11.31 | 10.98 | 60 | 10.69 | 10.09 | 11.09 | 11.00 | 1.009 | 1.009 1.019 1.009 | 24,799 | 24,799 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 24.01 | 22.68 | 25.36 | 25.05 | 60 | 24.67 | 23.87 | 26.35 | 25.85 | 1.057 | 1.057 0.980 1.060 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.25 | 24.95 | 28.16 | 27.81 | 60 | 27.19 | 25.82 | 28.84 | 28.39 | 1.037 | 1.037 0.985 1.069 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.76 | 23.33 | 26.07 | 25.82 | 60 | 24.98 | 23.89 | 28.51 | 26.79 | 1.041 | 1.041 0.960 1.059 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.18 | 32.00 | 35.34 | 35.17 | 60 | 34.08 | 32.72 | 36.44 | 35.51 | 1.035 | 1.035 0.966 1.063 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.88 | 13.26 | 14.34 | 14.27 | 60 | 13.92 | 12.69 | 14.45 | 14.40 | 0.986 | 0.967 0.986 1.027 | 24,799 | 24,799 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.53 | 12.00 | 13.22 | 12.97 | 60 | 12.58 | 12.02 | 13.24 | 12.91 | 1.007 | 1.014 1.007 1.000 | 24,799 | 24,799 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.12 | 24.75 | 27.28 | 26.98 | 60 | 26.71 | 25.97 | 27.81 | 27.63 | 1.045 | 1.059 0.989 1.045 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.50 | 28.31 | 31.19 | 30.57 | 60 | 30.19 | 28.85 | 31.82 | 31.41 | 1.034 | 1.034 0.989 1.049 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.51 | 25.50 | 27.99 | 27.64 | 60 | 27.12 | 26.35 | 28.04 | 27.85 | 1.039 | 1.041 0.979 1.039 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 36.41 | 34.97 | 38.75 | 37.82 | 60 | 36.97 | 35.17 | 38.62 | 37.99 | 1.032 | 1.039 0.972 1.032 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.32 | 17.03 | 14.30 | 60 | 12.49 | 11.87 | 13.28 | 13.22 | 0.960 | 0.960 0.908 1.009 | 24,799 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.75 | 10.05 | 12.15 | 11.25 | 60 | 10.92 | 10.35 | 12.47 | 11.32 | 1.026 | 1.026 1.034 1.002 | 24,799 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.58 | 25.29 | 29.19 | 27.63 | 60 | 27.13 | 26.00 | 30.41 | 28.74 | 1.047 | 1.047 0.988 1.053 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.46 | 29.10 | 31.55 | 31.41 | 60 | 31.26 | 29.76 | 32.62 | 32.41 | 1.047 | 1.047 0.989 1.048 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.37 | 26.01 | 28.89 | 28.33 | 60 | 27.78 | 26.62 | 31.48 | 29.55 | 1.021 | 1.055 0.992 1.021 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.15 | 34.49 | 37.85 | 37.57 | 60 | 37.04 | 35.37 | 40.98 | 38.37 | 1.032 | 1.053 0.985 1.032 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.32 | 16.15 | 14.38 | 60 | 13.24 | 12.82 | 14.34 | 14.33 | 0.960 | 0.960 0.925 1.029 | 24,799 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.79 | 12.20 | 13.36 | 13.22 | 60 | 12.82 | 12.25 | 13.75 | 13.43 | 1.005 | 1.005 1.000 1.011 | 24,799 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

No flag records.

### Refresh under concurrent source writes

A full `summary` refresh staged at V, K source commits of 10 scattered rows each, then the publication at V. `commit ms` is the conflict-checked publication commit. `merge_insert_output_in_place` writes `summary` itself (an output override); it is the only source write here that defers whole groups.

No flagged conflict records.

### Follow-up refresh to completion: saved and repeated computation

From the state the conflicted publication left, each strategy refreshes every pending row and publishes; afterwards every flag is asserted true and every value equal to the UDF of its current inputs. `recompute_all_pending` ignores the report; `reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject returns an error, so it has no report). `total UDF rows` = rows the conflicted publication computed + rows the follow-up recomputed; with N rows, anything above N is repeated computation.

No follow-up records.

### Publication commit latency after K unrelated commits

`wall = commit_ms` of the publication only. Conflict checks read every transaction since the read version.

No publish_after_k records.

### Flag state size as the true set fragments

Head after one in-place `body` write invalidating the given fraction of scattered rows. `groups=0` is the unflagged control with the same data files. `wall` is a fresh-session open (OS page cache not controlled).

No flag_state_size records.

### Every group (wall_ms)

One row per (build, workload, variant, cache) in the input.

| build | workload | variant | cache | n | med ms | min | max | p95 | udf med ms | stage med ms | commit med ms | outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | scan_summary_full | populated | warm | 60 | 23.51 | 22.77 | 25.47 | 25.34 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 25.77 | 25.07 | 27.16 | 27.03 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.25 | 23.35 | 26.31 | 25.31 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.23 | 31.79 | 35.58 | 34.99 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.93 | 12.32 | 13.77 | 13.32 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.60 | 9.92 | 11.31 | 10.98 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 24.01 | 22.68 | 25.36 | 25.05 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.25 | 24.95 | 28.16 | 27.81 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.76 | 23.33 | 26.07 | 25.82 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.18 | 32.00 | 35.34 | 35.17 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.88 | 13.26 | 14.34 | 14.27 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.53 | 12.00 | 13.22 | 12.97 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.12 | 24.75 | 27.28 | 26.98 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.50 | 28.31 | 31.19 | 30.57 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.51 | 25.50 | 27.99 | 27.64 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.41 | 34.97 | 38.75 | 37.82 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.32 | 17.03 | 14.30 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.75 | 10.05 | 12.15 | 11.25 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.58 | 25.29 | 29.19 | 27.63 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.46 | 29.10 | 31.55 | 31.41 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.37 | 26.01 | 28.89 | 28.33 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.15 | 34.49 | 37.85 | 37.57 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.32 | 16.15 | 14.38 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.79 | 12.20 | 13.36 | 13.22 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 24.82 | 23.62 | 26.46 | 26.30 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 26.77 | 25.73 | 28.46 | 28.16 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.16 | 24.07 | 26.50 | 26.30 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 33.92 | 32.37 | 35.84 | 35.15 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 12.91 | 11.92 | 13.56 | 13.51 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.69 | 10.09 | 11.09 | 11.00 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 24.67 | 23.87 | 26.35 | 25.85 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 27.19 | 25.82 | 28.84 | 28.39 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.98 | 23.89 | 28.51 | 26.79 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.08 | 32.72 | 36.44 | 35.51 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 13.92 | 12.69 | 14.45 | 14.40 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.58 | 12.02 | 13.24 | 12.91 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 26.71 | 25.97 | 27.81 | 27.63 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 30.19 | 28.85 | 31.82 | 31.41 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.12 | 26.35 | 28.04 | 27.85 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.97 | 35.17 | 38.62 | 37.99 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 12.49 | 11.87 | 13.28 | 13.22 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.92 | 10.35 | 12.47 | 11.32 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.13 | 26.00 | 30.41 | 28.74 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.26 | 29.76 | 32.62 | 32.41 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.78 | 26.62 | 31.48 | 29.55 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.04 | 35.37 | 40.98 | 38.37 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.24 | 12.82 | 14.34 | 14.33 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.82 | 12.25 | 13.75 | 13.43 | - | - | - | - |
