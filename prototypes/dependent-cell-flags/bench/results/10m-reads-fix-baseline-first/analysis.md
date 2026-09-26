## Results: 10m-reads-fix-baseline-first

Source: `results/10m-reads-fix-baseline-first`

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
| prototype_status | dirty: M rust/lance/Cargo.toml; ?? rust/lance/benches/cell_flags_scan_counters.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-26T05:23:44Z	2026-09-26T05:24:32Z	26.24 20.98 15.76
1	prototype	2026-09-26T05:24:32Z	2026-09-26T05:25:17Z	14.79 18.67 15.17
2	baseline	2026-09-26T05:25:17Z	2026-09-26T05:25:59Z	8.83 16.54 14.57
2	prototype	2026-09-26T05:26:00Z	2026-09-26T05:26:42Z	6.36 14.79 14.02
3	baseline	2026-09-26T05:26:42Z	2026-09-26T05:27:25Z	5.54 13.49 13.58
3	prototype	2026-09-26T05:27:25Z	2026-09-26T05:28:09Z	4.88 12.17 13.08
3	prototype-flags	2026-09-26T05:28:09Z	2026-09-26T05:29:02Z	5.25 11.22 12.68
```

Records: 3600. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 24.49 | 23.27 | 37.57 | 26.43 | 60 | 25.48 | 24.06 | 26.71 | 26.30 | 1.049 | 0.946 1.049 1.088 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 26.60 | 25.34 | 28.70 | 28.06 | 60 | 26.84 | 26.00 | 28.62 | 28.56 | 1.021 | 0.966 1.021 1.063 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.58 | 23.44 | 26.52 | 26.32 | 60 | 25.20 | 24.03 | 26.98 | 26.64 | 1.016 | 0.970 1.016 1.077 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.79 | 32.16 | 35.97 | 34.72 | 60 | 34.03 | 32.36 | 35.71 | 35.06 | 0.999 | 0.985 0.999 1.026 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.88 | 12.34 | 13.24 | 13.09 | 60 | 12.33 | 11.89 | 16.55 | 15.12 | 0.981 | 0.981 0.942 1.039 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.70 | 9.51 | 16.62 | 11.32 | 60 | 10.58 | 10.00 | 11.28 | 11.04 | 0.987 | 0.988 0.973 0.987 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 24.34 | 23.48 | 26.18 | 25.92 | 60 | 24.55 | 22.99 | 26.35 | 25.97 | 1.012 | 0.964 1.012 1.032 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.49 | 25.52 | 27.58 | 27.21 | 60 | 26.66 | 25.93 | 27.54 | 27.40 | 1.005 | 1.026 1.005 0.994 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.35 | 23.19 | 26.10 | 25.13 | 60 | 24.99 | 23.75 | 25.64 | 25.60 | 1.032 | 1.044 1.032 0.989 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.24 | 32.28 | 35.77 | 34.12 | 60 | 33.44 | 32.06 | 35.51 | 34.88 | 1.007 | 1.031 1.007 0.989 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.77 | 13.13 | 14.14 | 14.00 | 60 | 13.20 | 12.86 | 13.62 | 13.44 | 0.951 | 0.984 0.951 0.949 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.58 | 12.03 | 13.30 | 13.00 | 60 | 12.61 | 11.91 | 13.54 | 12.96 | 1.002 | 1.004 0.985 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.19 | 25.03 | 27.10 | 26.79 | 60 | 26.50 | 25.60 | 27.06 | 26.93 | 1.007 | 1.025 1.007 1.000 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 30.05 | 28.85 | 30.86 | 30.55 | 60 | 29.83 | 28.78 | 30.77 | 30.70 | 0.994 | 0.994 0.997 0.994 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.76 | 26.08 | 27.66 | 27.40 | 60 | 26.87 | 25.94 | 27.49 | 27.35 | 1.004 | 0.998 1.011 1.004 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 36.27 | 35.33 | 37.11 | 36.95 | 60 | 36.03 | 34.90 | 49.85 | 37.13 | 0.994 | 0.994 1.000 0.984 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 13.02 | 12.45 | 13.33 | 13.30 | 60 | 12.41 | 12.22 | 12.67 | 12.61 | 0.952 | 0.979 0.952 0.942 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.91 | 10.26 | 11.46 | 11.31 | 60 | 10.71 | 10.15 | 11.32 | 11.20 | 0.981 | 0.981 0.968 1.000 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.74 | 26.00 | 29.21 | 27.64 | 60 | 27.13 | 26.36 | 29.75 | 28.14 | 1.012 | 1.026 1.012 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.64 | 29.68 | 31.55 | 31.06 | 60 | 31.05 | 30.17 | 32.12 | 31.82 | 1.012 | 1.024 1.012 1.009 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.08 | 26.46 | 27.80 | 27.61 | 60 | 28.00 | 26.70 | 30.65 | 30.15 | 1.038 | 1.038 1.010 1.080 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.07 | 35.32 | 37.41 | 36.89 | 60 | 36.30 | 35.01 | 38.80 | 37.03 | 1.004 | 1.016 1.004 1.003 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.90 | 13.18 | 14.16 | 14.11 | 60 | 13.34 | 12.98 | 13.68 | 13.60 | 0.956 | 0.980 0.956 0.956 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.77 | 12.11 | 13.45 | 13.31 | 60 | 12.72 | 11.99 | 13.66 | 13.19 | 0.987 | 0.985 0.987 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.48 | 24.06 | 26.71 | 26.30 | 20 | 23.86 | 23.17 | 24.36 | 24.29 | 0.920 | 0.920 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 26.84 | 26.00 | 28.62 | 28.56 | 20 | 26.05 | 25.64 | 26.83 | 26.59 | 0.923 | 0.923 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.20 | 24.03 | 26.98 | 26.64 | 20 | 24.12 | 23.93 | 25.01 | 24.74 | 0.919 | 0.919 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 34.03 | 32.36 | 35.71 | 35.06 | 20 | 32.04 | 31.06 | 32.92 | 32.80 | 0.925 | 0.925 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 12.33 | 11.89 | 16.55 | 15.12 | 20 | 12.65 | 12.35 | 12.87 | 12.76 | 0.934 | 0.934 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.58 | 10.00 | 11.28 | 11.04 | 20 | 10.55 | 9.81 | 11.22 | 11.22 | 1.011 | 1.011 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 24.55 | 22.99 | 26.35 | 25.97 | 20 | 23.92 | 23.36 | 24.67 | 24.59 | 0.952 | 0.952 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 26.66 | 25.93 | 27.54 | 27.40 | 20 | 26.71 | 26.17 | 27.16 | 27.05 | 1.006 | 1.006 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 24.99 | 23.75 | 25.64 | 25.60 | 20 | 24.42 | 23.89 | 24.95 | 24.87 | 1.002 | 1.002 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 33.44 | 32.06 | 35.51 | 34.88 | 20 | 32.36 | 31.60 | 34.39 | 33.09 | 0.973 | 0.973 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 13.20 | 12.86 | 13.62 | 13.44 | 20 | 13.60 | 13.42 | 13.85 | 13.78 | 1.030 | 1.030 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.61 | 11.91 | 13.54 | 12.96 | 20 | 12.43 | 11.92 | 12.95 | 12.92 | 0.994 | 0.994 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 26.50 | 25.60 | 27.06 | 26.93 | 20 | 45.79 | 40.76 | 47.44 | 47.33 | 1.728 | 1.728 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 29.83 | 28.78 | 30.77 | 30.70 | 20 | 50.41 | 44.32 | 53.56 | 53.06 | 1.679 | 1.679 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 26.87 | 25.94 | 27.49 | 27.35 | 20 | 47.57 | 43.80 | 51.38 | 49.20 | 1.769 | 1.769 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 36.03 | 34.90 | 49.85 | 37.13 | 20 | 65.20 | 58.99 | 66.25 | 65.97 | 1.808 | 1.808 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 12.41 | 12.22 | 12.67 | 12.61 | 20 | 12.68 | 12.35 | 12.84 | 12.84 | 1.017 | 1.017 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.71 | 10.15 | 11.32 | 11.20 | 20 | 10.57 | 10.20 | 11.02 | 10.93 | 0.979 | 0.979 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 27.13 | 26.36 | 29.75 | 28.14 | 20 | 47.18 | 39.14 | 48.75 | 48.35 | 1.741 | 1.741 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 31.05 | 30.17 | 32.12 | 31.82 | 20 | 51.60 | 48.22 | 52.95 | 52.44 | 1.665 | 1.665 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.00 | 26.70 | 30.65 | 30.15 | 20 | 47.63 | 41.46 | 48.94 | 48.72 | 1.624 | 1.624 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 36.30 | 35.01 | 38.80 | 37.03 | 20 | 65.06 | 59.98 | 66.08 | 66.00 | 1.793 | 1.793 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 13.34 | 12.98 | 13.68 | 13.60 | 20 | 13.38 | 13.18 | 20.82 | 19.60 | 0.997 | 0.997 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.72 | 11.99 | 13.66 | 13.19 | 20 | 12.65 | 12.27 | 13.15 | 13.03 | 0.999 | 0.999 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.71 | 2.66 | 2.76 | 2.74 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.52 | 5.32 | 5.73 | 5.71 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.17 | 3.12 | 3.25 | 3.24 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 6.82 | 6.73 | 6.93 | 6.92 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 11.77 | 11.62 | 11.95 | 11.93 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.36 | 0.36 | 0.41 | 0.39 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.71 | 2.66 | 2.75 | 2.73 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.45 | 5.38 | 5.54 | 5.52 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.19 | 3.15 | 3.25 | 3.24 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 6.85 | 6.78 | 7.18 | 7.01 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 12.57 | 12.34 | 12.69 | 12.68 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.38 | 0.37 | 0.42 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 24.49 | 23.27 | 37.57 | 26.43 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 26.60 | 25.34 | 28.70 | 28.06 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.58 | 23.44 | 26.52 | 26.32 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.79 | 32.16 | 35.97 | 34.72 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.88 | 12.34 | 13.24 | 13.09 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.70 | 9.51 | 16.62 | 11.32 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 24.34 | 23.48 | 26.18 | 25.92 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.49 | 25.52 | 27.58 | 27.21 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.35 | 23.19 | 26.10 | 25.13 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.24 | 32.28 | 35.77 | 34.12 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.77 | 13.13 | 14.14 | 14.00 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.58 | 12.03 | 13.30 | 13.00 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.19 | 25.03 | 27.10 | 26.79 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 30.05 | 28.85 | 30.86 | 30.55 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.76 | 26.08 | 27.66 | 27.40 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.27 | 35.33 | 37.11 | 36.95 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 13.02 | 12.45 | 13.33 | 13.30 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.91 | 10.26 | 11.46 | 11.31 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.74 | 26.00 | 29.21 | 27.64 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.64 | 29.68 | 31.55 | 31.06 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.08 | 26.46 | 27.80 | 27.61 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.07 | 35.32 | 37.41 | 36.89 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.90 | 13.18 | 14.16 | 14.11 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.77 | 12.11 | 13.45 | 13.31 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.48 | 24.06 | 26.71 | 26.30 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 26.84 | 26.00 | 28.62 | 28.56 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.20 | 24.03 | 26.98 | 26.64 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 34.03 | 32.36 | 35.71 | 35.06 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 12.33 | 11.89 | 16.55 | 15.12 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.58 | 10.00 | 11.28 | 11.04 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 24.55 | 22.99 | 26.35 | 25.97 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 26.66 | 25.93 | 27.54 | 27.40 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.99 | 23.75 | 25.64 | 25.60 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.44 | 32.06 | 35.51 | 34.88 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 13.20 | 12.86 | 13.62 | 13.44 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.61 | 11.91 | 13.54 | 12.96 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 26.50 | 25.60 | 27.06 | 26.93 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 29.83 | 28.78 | 30.77 | 30.70 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.87 | 25.94 | 27.49 | 27.35 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.03 | 34.90 | 49.85 | 37.13 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 12.41 | 12.22 | 12.67 | 12.61 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.71 | 10.15 | 11.32 | 11.20 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.13 | 26.36 | 29.75 | 28.14 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.05 | 30.17 | 32.12 | 31.82 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.00 | 26.70 | 30.65 | 30.15 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.30 | 35.01 | 38.80 | 37.03 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.34 | 12.98 | 13.68 | 13.60 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.72 | 11.99 | 13.66 | 13.19 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 23.86 | 23.17 | 24.36 | 24.29 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 26.05 | 25.64 | 26.83 | 26.59 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 24.12 | 23.93 | 25.01 | 24.74 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 32.04 | 31.06 | 32.92 | 32.80 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 12.65 | 12.35 | 12.87 | 12.76 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.55 | 9.81 | 11.22 | 11.22 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 23.92 | 23.36 | 24.67 | 24.59 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 26.71 | 26.17 | 27.16 | 27.05 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 24.42 | 23.89 | 24.95 | 24.87 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 32.36 | 31.60 | 34.39 | 33.09 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 13.60 | 13.42 | 13.85 | 13.78 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.43 | 11.92 | 12.95 | 12.92 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 45.79 | 40.76 | 47.44 | 47.33 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 50.41 | 44.32 | 53.56 | 53.06 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 47.57 | 43.80 | 51.38 | 49.20 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 65.20 | 58.99 | 66.25 | 65.97 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 12.68 | 12.35 | 12.84 | 12.84 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.57 | 10.20 | 11.02 | 10.93 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 47.18 | 39.14 | 48.75 | 48.35 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 51.60 | 48.22 | 52.95 | 52.44 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 47.63 | 41.46 | 48.94 | 48.72 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 65.06 | 59.98 | 66.08 | 66.00 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 13.38 | 13.18 | 20.82 | 19.60 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.65 | 12.27 | 13.15 | 13.03 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.71 | 2.66 | 2.76 | 2.74 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.52 | 5.32 | 5.73 | 5.71 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.17 | 3.12 | 3.25 | 3.24 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 6.82 | 6.73 | 6.93 | 6.92 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 11.77 | 11.62 | 11.95 | 11.93 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.36 | 0.36 | 0.41 | 0.39 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.71 | 2.66 | 2.75 | 2.73 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.45 | 5.38 | 5.54 | 5.52 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.19 | 3.15 | 3.25 | 3.24 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 6.85 | 6.78 | 7.18 | 7.01 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 12.57 | 12.34 | 12.69 | 12.68 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.38 | 0.37 | 0.42 | 0.42 | - | - | - | - |
