## Results: 10m-reads-fix-prototype-first

Source: `results/10m-reads-fix-prototype-first`

### Environment

| key | value |
|---|---|
| scale | `10m` |
| profile | `release-with-debug` |
| hw_model | `Mac17,8` |
| cpu_brand | `Apple M5 Pro` |
| ncpu | `18` |
| memsize_bytes | `51539607552` |
| macos | `26.7 (25G229)` |
| rustc_baseline | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| rustc_prototype | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| baseline_sha | `e3671b2f5730eea927a088a42cbf30e273edc43c` |
| prototype_sha | `2fd300ac2885f07d1357a0a438ceb874d7d6a519` |
| rounds | `3` |
| flag_every_round | `False` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs; ?? rust/lance/benches/cell_flags_scan_counters.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M rust/lance/Cargo.toml; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-fix-baseline-first/; ?? rust/lance/benches/cell_flags_scan_counters.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	prototype	2026-09-26T05:29:05Z	2026-09-26T05:29:47Z	5.09 10.16 12.19
1	baseline	2026-09-26T05:29:47Z	2026-09-26T05:30:31Z	5.26 9.53 11.87
2	prototype	2026-09-26T05:30:31Z	2026-09-26T05:31:15Z	4.06 8.58 11.39
2	baseline	2026-09-26T05:31:15Z	2026-09-26T05:31:58Z	4.67 8.03 11.04
3	prototype	2026-09-26T05:31:58Z	2026-09-26T05:32:42Z	4.31 7.48 10.70
3	baseline	2026-09-26T05:32:42Z	2026-09-26T05:33:25Z	4.50 7.05 10.37
3	prototype-flags	2026-09-26T05:33:25Z	2026-09-26T05:34:17Z	4.85 6.75 10.09
```

Records: 3600. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 24.04 | 23.29 | 26.79 | 26.19 | 60 | 25.82 | 23.74 | 26.56 | 26.41 | 1.070 | 0.928 1.070 1.096 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 26.48 | 25.30 | 28.88 | 28.55 | 60 | 27.78 | 25.88 | 28.67 | 28.38 | 1.054 | 0.926 1.054 1.088 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.61 | 23.67 | 35.79 | 27.06 | 60 | 26.08 | 23.96 | 27.02 | 26.69 | 1.063 | 0.921 1.063 1.089 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 34.06 | 31.79 | 39.61 | 37.07 | 60 | 34.79 | 32.45 | 36.06 | 35.68 | 1.020 | 0.917 1.020 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 13.10 | 12.18 | 18.17 | 15.76 | 60 | 12.50 | 12.18 | 13.12 | 13.05 | 0.974 | 0.944 0.974 0.997 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.64 | 10.19 | 11.09 | 11.02 | 60 | 10.57 | 9.91 | 11.36 | 10.90 | 0.988 | 0.988 0.983 0.998 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 24.45 | 23.02 | 26.58 | 26.31 | 60 | 25.78 | 23.81 | 27.25 | 26.70 | 1.060 | 0.926 1.060 1.097 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 27.15 | 25.77 | 29.42 | 29.28 | 60 | 28.20 | 25.99 | 29.06 | 28.98 | 1.043 | 0.911 1.043 1.085 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.95 | 23.99 | 33.89 | 27.28 | 60 | 26.08 | 24.01 | 29.01 | 27.15 | 1.046 | 0.925 1.046 1.102 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 34.39 | 32.46 | 36.10 | 35.81 | 60 | 34.89 | 32.68 | 36.47 | 35.85 | 1.017 | 0.950 1.017 1.067 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.68 | 13.08 | 14.40 | 14.32 | 60 | 13.41 | 13.09 | 14.07 | 13.87 | 0.976 | 0.976 0.970 0.998 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.51 | 11.59 | 13.39 | 13.35 | 60 | 12.44 | 11.89 | 12.98 | 12.78 | 1.002 | 1.012 0.981 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.44 | 25.11 | 28.03 | 27.86 | 60 | 27.62 | 25.90 | 28.74 | 28.45 | 1.063 | 0.952 1.063 1.076 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 30.17 | 29.18 | 32.48 | 32.32 | 60 | 31.31 | 28.90 | 32.87 | 32.47 | 1.055 | 0.939 1.055 1.077 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.52 | 25.73 | 28.94 | 28.85 | 60 | 28.22 | 25.84 | 29.67 | 28.90 | 1.069 | 0.937 1.069 1.092 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 37.09 | 34.76 | 39.00 | 38.93 | 60 | 38.02 | 35.71 | 44.29 | 40.55 | 1.016 | 0.971 1.016 1.068 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.92 | 12.48 | 13.42 | 13.34 | 60 | 12.80 | 12.33 | 13.68 | 13.32 | 0.986 | 1.016 0.972 0.986 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.75 | 10.03 | 13.27 | 11.18 | 60 | 10.70 | 10.13 | 13.53 | 11.24 | 0.997 | 0.989 0.997 1.016 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.88 | 25.86 | 28.97 | 28.67 | 60 | 28.51 | 26.36 | 29.52 | 29.29 | 1.063 | 0.954 1.063 1.092 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.12 | 29.23 | 33.07 | 32.79 | 60 | 32.49 | 29.81 | 33.49 | 33.22 | 1.044 | 0.945 1.044 1.078 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.51 | 26.56 | 29.85 | 29.65 | 60 | 28.74 | 26.91 | 30.39 | 29.88 | 1.056 | 0.925 1.056 1.084 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.87 | 35.23 | 39.03 | 38.99 | 60 | 37.51 | 35.81 | 40.09 | 39.09 | 1.018 | 0.942 1.018 1.081 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.38 | 14.48 | 14.36 | 60 | 13.59 | 13.15 | 14.15 | 13.89 | 0.982 | 0.982 0.967 0.992 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.67 | 12.16 | 13.24 | 13.00 | 60 | 12.72 | 12.10 | 13.89 | 13.58 | 1.003 | 1.003 0.991 1.043 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.82 | 23.74 | 26.56 | 26.41 | 20 | 24.47 | 24.04 | 25.26 | 25.24 | 0.938 | 0.938 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.78 | 25.88 | 28.67 | 28.38 | 20 | 26.62 | 26.13 | 27.28 | 27.24 | 0.948 | 0.948 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 26.08 | 23.96 | 27.02 | 26.69 | 20 | 24.74 | 24.32 | 25.30 | 25.19 | 0.937 | 0.937 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 34.79 | 32.45 | 36.06 | 35.68 | 20 | 33.04 | 32.36 | 33.94 | 33.70 | 0.939 | 0.939 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 12.50 | 12.18 | 13.12 | 13.05 | 20 | 12.86 | 12.68 | 13.27 | 13.19 | 1.035 | 1.035 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.57 | 9.91 | 11.36 | 10.90 | 20 | 10.61 | 10.16 | 11.31 | 11.28 | 0.997 | 0.997 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.78 | 23.81 | 27.25 | 26.70 | 20 | 24.70 | 23.90 | 29.01 | 27.83 | 0.941 | 0.941 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 28.20 | 25.99 | 29.06 | 28.98 | 20 | 27.18 | 26.35 | 27.69 | 27.53 | 0.949 | 0.949 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 26.08 | 24.01 | 29.01 | 27.15 | 20 | 25.19 | 24.57 | 25.52 | 25.48 | 0.943 | 0.943 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 34.89 | 32.68 | 36.47 | 35.85 | 20 | 33.62 | 32.66 | 34.25 | 34.23 | 0.949 | 0.949 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 13.41 | 13.09 | 14.07 | 13.87 | 20 | 13.82 | 13.60 | 14.17 | 13.97 | 1.035 | 1.035 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.44 | 11.89 | 12.98 | 12.78 | 20 | 12.57 | 12.07 | 13.00 | 12.90 | 1.004 | 1.004 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 27.62 | 25.90 | 28.74 | 28.45 | 20 | 47.52 | 42.69 | 48.87 | 48.21 | 1.690 | 1.690 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.31 | 28.90 | 32.87 | 32.47 | 20 | 52.08 | 49.38 | 60.80 | 53.43 | 1.632 | 1.632 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 28.22 | 25.84 | 29.67 | 28.90 | 20 | 48.17 | 46.29 | 51.00 | 49.28 | 1.686 | 1.686 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 38.02 | 35.71 | 44.29 | 40.55 | 20 | 65.38 | 59.89 | 66.40 | 66.02 | 1.697 | 1.697 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 12.80 | 12.33 | 13.68 | 13.32 | 20 | 12.68 | 12.40 | 12.93 | 12.93 | 1.014 | 1.014 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.70 | 10.13 | 13.53 | 11.24 | 20 | 10.68 | 10.07 | 11.30 | 11.21 | 0.987 | 0.987 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 28.51 | 26.36 | 29.52 | 29.29 | 20 | 47.99 | 42.38 | 49.00 | 48.97 | 1.656 | 1.656 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 32.49 | 29.81 | 33.49 | 33.22 | 20 | 52.62 | 45.18 | 54.54 | 53.95 | 1.602 | 1.602 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.74 | 26.91 | 30.39 | 29.88 | 20 | 48.81 | 43.60 | 49.66 | 49.43 | 1.654 | 1.654 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 37.51 | 35.81 | 40.09 | 39.09 | 20 | 66.18 | 60.29 | 66.58 | 66.57 | 1.705 | 1.705 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 13.59 | 13.15 | 14.15 | 13.89 | 20 | 13.39 | 13.09 | 13.57 | 13.55 | 0.994 | 0.994 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.72 | 12.10 | 13.89 | 13.58 | 20 | 12.55 | 12.20 | 13.18 | 12.85 | 0.952 | 0.952 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.78 | 2.74 | 2.83 | 2.83 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.48 | 5.41 | 5.60 | 5.53 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.20 | 3.15 | 3.25 | 3.23 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 6.83 | 6.79 | 6.99 | 6.94 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 11.97 | 11.84 | 12.12 | 12.12 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.38 | 0.37 | 0.41 | 0.40 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.77 | 2.73 | 2.83 | 2.81 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.49 | 5.42 | 5.54 | 5.53 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.22 | 3.15 | 3.28 | 3.27 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 6.89 | 6.77 | 7.25 | 7.19 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 12.68 | 12.43 | 12.89 | 12.85 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.39 | 0.37 | 0.42 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 24.04 | 23.29 | 26.79 | 26.19 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 26.48 | 25.30 | 28.88 | 28.55 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.61 | 23.67 | 35.79 | 27.06 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 34.06 | 31.79 | 39.61 | 37.07 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 13.10 | 12.18 | 18.17 | 15.76 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.64 | 10.19 | 11.09 | 11.02 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 24.45 | 23.02 | 26.58 | 26.31 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 27.15 | 25.77 | 29.42 | 29.28 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.95 | 23.99 | 33.89 | 27.28 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.39 | 32.46 | 36.10 | 35.81 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.68 | 13.08 | 14.40 | 14.32 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.51 | 11.59 | 13.39 | 13.35 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.44 | 25.11 | 28.03 | 27.86 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 30.17 | 29.18 | 32.48 | 32.32 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.52 | 25.73 | 28.94 | 28.85 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 37.09 | 34.76 | 39.00 | 38.93 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.92 | 12.48 | 13.42 | 13.34 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.75 | 10.03 | 13.27 | 11.18 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.88 | 25.86 | 28.97 | 28.67 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.12 | 29.23 | 33.07 | 32.79 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.51 | 26.56 | 29.85 | 29.65 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.87 | 35.23 | 39.03 | 38.99 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.38 | 14.48 | 14.36 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.67 | 12.16 | 13.24 | 13.00 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.82 | 23.74 | 26.56 | 26.41 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.78 | 25.88 | 28.67 | 28.38 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 26.08 | 23.96 | 27.02 | 26.69 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 34.79 | 32.45 | 36.06 | 35.68 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 12.50 | 12.18 | 13.12 | 13.05 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.57 | 9.91 | 11.36 | 10.90 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.78 | 23.81 | 27.25 | 26.70 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 28.20 | 25.99 | 29.06 | 28.98 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 26.08 | 24.01 | 29.01 | 27.15 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.89 | 32.68 | 36.47 | 35.85 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 13.09 | 14.07 | 13.87 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.44 | 11.89 | 12.98 | 12.78 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 27.62 | 25.90 | 28.74 | 28.45 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.31 | 28.90 | 32.87 | 32.47 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 28.22 | 25.84 | 29.67 | 28.90 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 38.02 | 35.71 | 44.29 | 40.55 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 12.80 | 12.33 | 13.68 | 13.32 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.70 | 10.13 | 13.53 | 11.24 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 28.51 | 26.36 | 29.52 | 29.29 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 32.49 | 29.81 | 33.49 | 33.22 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.74 | 26.91 | 30.39 | 29.88 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.51 | 35.81 | 40.09 | 39.09 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.59 | 13.15 | 14.15 | 13.89 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.72 | 12.10 | 13.89 | 13.58 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 24.47 | 24.04 | 25.26 | 25.24 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 26.62 | 26.13 | 27.28 | 27.24 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 24.74 | 24.32 | 25.30 | 25.19 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 33.04 | 32.36 | 33.94 | 33.70 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 12.86 | 12.68 | 13.27 | 13.19 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.61 | 10.16 | 11.31 | 11.28 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 24.70 | 23.90 | 29.01 | 27.83 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 27.18 | 26.35 | 27.69 | 27.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 25.19 | 24.57 | 25.52 | 25.48 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 33.62 | 32.66 | 34.25 | 34.23 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 13.82 | 13.60 | 14.17 | 13.97 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.57 | 12.07 | 13.00 | 12.90 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 47.52 | 42.69 | 48.87 | 48.21 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 52.08 | 49.38 | 60.80 | 53.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 48.17 | 46.29 | 51.00 | 49.28 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 65.38 | 59.89 | 66.40 | 66.02 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 12.68 | 12.40 | 12.93 | 12.93 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.68 | 10.07 | 11.30 | 11.21 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 47.99 | 42.38 | 49.00 | 48.97 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 52.62 | 45.18 | 54.54 | 53.95 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 48.81 | 43.60 | 49.66 | 49.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 66.18 | 60.29 | 66.58 | 66.57 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 13.39 | 13.09 | 13.57 | 13.55 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.55 | 12.20 | 13.18 | 12.85 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.78 | 2.74 | 2.83 | 2.83 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.48 | 5.41 | 5.60 | 5.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.20 | 3.15 | 3.25 | 3.23 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 6.83 | 6.79 | 6.99 | 6.94 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 11.97 | 11.84 | 12.12 | 12.12 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.38 | 0.37 | 0.41 | 0.40 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.77 | 2.73 | 2.83 | 2.81 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.49 | 5.42 | 5.54 | 5.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.22 | 3.15 | 3.28 | 3.27 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 6.89 | 6.77 | 7.25 | 7.19 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 12.68 | 12.43 | 12.89 | 12.85 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.39 | 0.37 | 0.42 | 0.42 | - | - | - | - |
