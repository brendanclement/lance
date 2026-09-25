## Results: 10m reads after the mask fix (26388225a), prototype first

Source: `results/10m-reads-reversed`

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
| prototype_sha | `e48573011780928ea1fd6b823a2eb71783ec939c` |
| rounds | `3` |
| flag_every_round | `False` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M prototypes/dependent-cell-flags/bench/run_paired.sh; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-maskfix/; ?? prototypes/dependent-cell-flags/bench/results/1m-reads-maskfix/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	prototype	2026-09-25T08:59:53Z	2026-09-25T09:00:37Z	3.32 6.63 7.37
1	baseline	2026-09-25T09:00:37Z	2026-09-25T09:01:19Z	3.83 6.26 7.19
2	prototype	2026-09-25T09:01:19Z	2026-09-25T09:02:02Z	4.69 6.16 7.11
2	baseline	2026-09-25T09:02:02Z	2026-09-25T09:02:45Z	5.04 5.98 6.99
3	prototype	2026-09-25T09:02:45Z	2026-09-25T09:03:29Z	4.21 5.64 6.82
3	baseline	2026-09-25T09:03:29Z	2026-09-25T09:04:12Z	7.34 6.22 6.97
3	prototype-flags	2026-09-25T09:04:12Z	2026-09-25T09:05:04Z	4.80 5.69 6.73
```

Records: 3600. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.85 | 23.07 | 25.09 | 24.69 | 60 | 25.19 | 23.43 | 26.59 | 26.31 | 1.057 | 1.082 1.017 1.057 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 26.16 | 25.08 | 37.61 | 28.65 | 60 | 27.37 | 25.47 | 28.67 | 28.35 | 1.025 | 1.069 1.021 1.025 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.10 | 23.20 | 24.91 | 24.67 | 60 | 25.44 | 24.11 | 27.29 | 26.92 | 1.047 | 1.096 1.031 1.047 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.18 | 31.59 | 34.37 | 33.96 | 60 | 34.69 | 33.37 | 36.17 | 35.75 | 1.041 | 1.064 1.032 1.041 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.70 | 12.33 | 12.99 | 12.89 | 60 | 13.21 | 12.78 | 13.84 | 13.72 | 1.038 | 1.082 1.021 1.038 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.61 | 10.05 | 12.11 | 11.59 | 60 | 10.66 | 10.10 | 11.18 | 10.99 | 0.999 | 0.999 0.983 1.013 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 23.92 | 23.17 | 24.81 | 24.62 | 60 | 25.11 | 23.69 | 26.67 | 26.42 | 1.064 | 1.073 1.007 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.30 | 25.07 | 27.25 | 26.93 | 60 | 27.51 | 25.99 | 29.48 | 29.19 | 1.059 | 1.087 1.004 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.43 | 23.58 | 25.13 | 24.92 | 60 | 25.45 | 24.18 | 27.83 | 27.28 | 1.046 | 1.089 1.018 1.046 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.56 | 32.24 | 35.61 | 34.59 | 60 | 35.11 | 32.72 | 37.24 | 36.44 | 1.038 | 1.079 1.013 1.038 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 12.91 | 13.84 | 13.80 | 60 | 14.03 | 13.58 | 14.77 | 14.64 | 1.059 | 1.082 1.018 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.50 | 11.80 | 13.53 | 12.96 | 60 | 12.65 | 11.84 | 13.77 | 13.31 | 1.007 | 1.007 1.002 1.030 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.23 | 25.40 | 26.93 | 26.72 | 60 | 26.93 | 25.63 | 28.65 | 28.53 | 1.033 | 1.068 1.003 1.033 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.51 | 27.96 | 30.72 | 30.24 | 60 | 31.06 | 29.48 | 33.21 | 32.62 | 1.064 | 1.077 1.025 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.19 | 25.46 | 27.75 | 27.39 | 60 | 27.65 | 26.47 | 28.73 | 28.66 | 1.050 | 1.050 1.040 1.066 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 35.97 | 34.71 | 37.51 | 36.91 | 60 | 37.67 | 35.24 | 39.58 | 38.92 | 1.051 | 1.064 1.035 1.051 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.63 | 12.21 | 13.16 | 13.10 | 60 | 13.20 | 12.82 | 13.90 | 13.76 | 1.052 | 1.078 1.001 1.052 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.79 | 9.91 | 11.43 | 11.26 | 60 | 10.80 | 10.27 | 11.31 | 11.29 | 0.993 | 0.993 0.989 1.017 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.44 | 25.53 | 29.11 | 27.71 | 60 | 27.65 | 26.49 | 31.16 | 28.85 | 1.052 | 1.052 1.031 1.053 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.24 | 29.35 | 32.52 | 30.97 | 60 | 31.85 | 30.52 | 33.52 | 33.11 | 1.061 | 1.079 1.013 1.061 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.20 | 25.95 | 30.30 | 28.70 | 60 | 28.08 | 26.87 | 29.58 | 29.46 | 1.053 | 1.062 0.998 1.053 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.01 | 34.73 | 40.11 | 39.06 | 60 | 37.76 | 35.68 | 39.48 | 38.77 | 1.021 | 1.003 1.021 1.060 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.56 | 13.23 | 14.24 | 13.95 | 60 | 14.05 | 13.66 | 14.77 | 14.65 | 1.044 | 1.070 1.010 1.044 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.69 | 11.67 | 13.91 | 13.27 | 60 | 12.78 | 12.29 | 13.25 | 13.19 | 1.007 | 1.007 1.019 0.988 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.19 | 23.43 | 26.59 | 26.31 | 20 | 25.33 | 24.71 | 25.84 | 25.72 | 1.006 | 1.006 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.37 | 25.47 | 28.67 | 28.35 | 20 | 27.17 | 26.38 | 28.34 | 27.77 | 0.992 | 0.992 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.44 | 24.11 | 27.29 | 26.92 | 20 | 24.76 | 24.35 | 25.29 | 25.29 | 0.974 | 0.974 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 34.69 | 33.37 | 36.17 | 35.75 | 20 | 33.65 | 33.05 | 34.34 | 34.19 | 0.970 | 0.970 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 13.21 | 12.78 | 13.84 | 13.72 | 20 | 13.57 | 13.39 | 13.71 | 13.69 | 1.032 | 1.032 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.66 | 10.10 | 11.18 | 10.99 | 20 | 10.63 | 10.31 | 11.10 | 11.08 | 0.990 | 0.990 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.11 | 23.69 | 26.67 | 26.42 | 20 | 25.09 | 24.48 | 25.63 | 25.54 | 0.999 | 0.999 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 27.51 | 25.99 | 29.48 | 29.19 | 20 | 27.65 | 27.02 | 28.34 | 28.10 | 1.005 | 1.005 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 25.45 | 24.18 | 27.83 | 27.28 | 20 | 25.06 | 24.31 | 26.56 | 25.68 | 0.985 | 0.985 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 35.11 | 32.72 | 37.24 | 36.44 | 20 | 33.93 | 32.55 | 34.82 | 34.75 | 0.968 | 0.968 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 14.03 | 13.58 | 14.77 | 14.64 | 20 | 14.54 | 14.22 | 14.73 | 14.71 | 1.041 | 1.041 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.65 | 11.84 | 13.77 | 13.31 | 20 | 12.52 | 12.22 | 12.99 | 12.70 | 0.976 | 0.976 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 26.93 | 25.63 | 28.65 | 28.53 | 20 | 47.48 | 44.02 | 51.55 | 51.21 | 1.766 | 1.766 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.06 | 29.48 | 33.21 | 32.62 | 20 | 52.73 | 45.75 | 54.64 | 54.34 | 1.698 | 1.698 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 27.65 | 26.47 | 28.73 | 28.66 | 20 | 46.90 | 40.77 | 48.65 | 47.95 | 1.697 | 1.697 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 37.67 | 35.24 | 39.58 | 38.92 | 20 | 66.87 | 60.45 | 67.79 | 67.62 | 1.777 | 1.777 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 13.20 | 12.82 | 13.90 | 13.76 | 20 | 13.06 | 12.69 | 13.29 | 13.22 | 0.992 | 0.992 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.80 | 10.27 | 11.31 | 11.29 | 20 | 10.65 | 10.06 | 11.06 | 11.02 | 0.979 | 0.979 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 27.65 | 26.49 | 31.16 | 28.85 | 20 | 47.85 | 43.35 | 50.04 | 49.78 | 1.736 | 1.736 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 31.85 | 30.52 | 33.52 | 33.11 | 20 | 53.25 | 46.59 | 54.68 | 54.44 | 1.678 | 1.678 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.08 | 26.87 | 29.58 | 29.46 | 20 | 47.61 | 45.42 | 49.59 | 48.81 | 1.698 | 1.698 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 37.76 | 35.68 | 39.48 | 38.77 | 20 | 67.26 | 61.23 | 69.49 | 68.50 | 1.781 | 1.781 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 14.05 | 13.66 | 14.77 | 14.65 | 20 | 13.78 | 13.48 | 14.00 | 13.99 | 0.984 | 0.984 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.78 | 12.29 | 13.25 | 13.19 | 20 | 12.58 | 12.34 | 13.39 | 13.13 | 0.990 | 0.990 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.70 | 2.65 | 2.77 | 2.75 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.53 | 5.41 | 5.81 | 5.74 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.26 | 3.21 | 3.33 | 3.32 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 7.08 | 6.97 | 7.59 | 7.37 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 12.36 | 11.84 | 12.54 | 12.47 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.37 | 0.36 | 0.40 | 0.39 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.71 | 2.67 | 2.76 | 2.73 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.53 | 5.41 | 5.66 | 5.63 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.29 | 3.22 | 3.38 | 3.34 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 7.08 | 6.99 | 7.46 | 7.34 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 13.16 | 12.86 | 13.38 | 13.30 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.40 | 0.38 | 0.43 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.85 | 23.07 | 25.09 | 24.69 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 26.16 | 25.08 | 37.61 | 28.65 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.10 | 23.20 | 24.91 | 24.67 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.18 | 31.59 | 34.37 | 33.96 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.70 | 12.33 | 12.99 | 12.89 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.61 | 10.05 | 12.11 | 11.59 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 23.92 | 23.17 | 24.81 | 24.62 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.30 | 25.07 | 27.25 | 26.93 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.43 | 23.58 | 25.13 | 24.92 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.56 | 32.24 | 35.61 | 34.59 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 12.91 | 13.84 | 13.80 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.50 | 11.80 | 13.53 | 12.96 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.23 | 25.40 | 26.93 | 26.72 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.51 | 27.96 | 30.72 | 30.24 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.19 | 25.46 | 27.75 | 27.39 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 35.97 | 34.71 | 37.51 | 36.91 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.63 | 12.21 | 13.16 | 13.10 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.79 | 9.91 | 11.43 | 11.26 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.44 | 25.53 | 29.11 | 27.71 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.24 | 29.35 | 32.52 | 30.97 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.20 | 25.95 | 30.30 | 28.70 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.01 | 34.73 | 40.11 | 39.06 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.56 | 13.23 | 14.24 | 13.95 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.69 | 11.67 | 13.91 | 13.27 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.19 | 23.43 | 26.59 | 26.31 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.37 | 25.47 | 28.67 | 28.35 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.44 | 24.11 | 27.29 | 26.92 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 34.69 | 33.37 | 36.17 | 35.75 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 13.21 | 12.78 | 13.84 | 13.72 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.66 | 10.10 | 11.18 | 10.99 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.11 | 23.69 | 26.67 | 26.42 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 27.51 | 25.99 | 29.48 | 29.19 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 25.45 | 24.18 | 27.83 | 27.28 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 35.11 | 32.72 | 37.24 | 36.44 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 14.03 | 13.58 | 14.77 | 14.64 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.65 | 11.84 | 13.77 | 13.31 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 26.93 | 25.63 | 28.65 | 28.53 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.06 | 29.48 | 33.21 | 32.62 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.65 | 26.47 | 28.73 | 28.66 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 37.67 | 35.24 | 39.58 | 38.92 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 13.20 | 12.82 | 13.90 | 13.76 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.80 | 10.27 | 11.31 | 11.29 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.65 | 26.49 | 31.16 | 28.85 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.85 | 30.52 | 33.52 | 33.11 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.08 | 26.87 | 29.58 | 29.46 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.76 | 35.68 | 39.48 | 38.77 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 14.05 | 13.66 | 14.77 | 14.65 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.78 | 12.29 | 13.25 | 13.19 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 25.33 | 24.71 | 25.84 | 25.72 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 27.17 | 26.38 | 28.34 | 27.77 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 24.76 | 24.35 | 25.29 | 25.29 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 33.65 | 33.05 | 34.34 | 34.19 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 13.57 | 13.39 | 13.71 | 13.69 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.63 | 10.31 | 11.10 | 11.08 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 25.09 | 24.48 | 25.63 | 25.54 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 27.65 | 27.02 | 28.34 | 28.10 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 25.06 | 24.31 | 26.56 | 25.68 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 33.93 | 32.55 | 34.82 | 34.75 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 14.54 | 14.22 | 14.73 | 14.71 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.52 | 12.22 | 12.99 | 12.70 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 47.48 | 44.02 | 51.55 | 51.21 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 52.73 | 45.75 | 54.64 | 54.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 46.90 | 40.77 | 48.65 | 47.95 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 66.87 | 60.45 | 67.79 | 67.62 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 13.06 | 12.69 | 13.29 | 13.22 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.65 | 10.06 | 11.06 | 11.02 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 47.85 | 43.35 | 50.04 | 49.78 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 53.25 | 46.59 | 54.68 | 54.44 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 47.61 | 45.42 | 49.59 | 48.81 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 67.26 | 61.23 | 69.49 | 68.50 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 13.78 | 13.48 | 14.00 | 13.99 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.58 | 12.34 | 13.39 | 13.13 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.70 | 2.65 | 2.77 | 2.75 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.53 | 5.41 | 5.81 | 5.74 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.26 | 3.21 | 3.33 | 3.32 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 7.08 | 6.97 | 7.59 | 7.37 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 12.36 | 11.84 | 12.54 | 12.47 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.37 | 0.36 | 0.40 | 0.39 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.71 | 2.67 | 2.76 | 2.73 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.53 | 5.41 | 5.66 | 5.63 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.29 | 3.22 | 3.38 | 3.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 7.08 | 6.99 | 7.46 | 7.34 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 13.16 | 12.86 | 13.38 | 13.30 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.40 | 0.38 | 0.43 | 0.42 | - | - | - | - |
