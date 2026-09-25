## Clean build: 10m reads at 26388225a built outside the nested checkout, prototype first

Source: `results/10m-reads-clean-prototype-first`

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
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M prototypes/dependent-cell-flags/bench/run_paired.sh; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-clean-baseline-first/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	prototype	2026-09-25T10:10:50Z	2026-09-25T10:11:34Z	6.64 8.57 10.10
1	baseline	2026-09-25T10:11:34Z	2026-09-25T10:12:17Z	6.47 8.27 9.92
1	prototype-flags	2026-09-25T10:12:17Z	2026-09-25T10:13:09Z	6.09 7.90 9.70
2	prototype	2026-09-25T10:13:09Z	2026-09-25T10:13:52Z	5.42 7.39 9.40
2	baseline	2026-09-25T10:13:52Z	2026-09-25T10:14:35Z	5.06 6.98 9.14
2	prototype-flags	2026-09-25T10:14:35Z	2026-09-25T10:15:27Z	4.50 6.53 8.86
3	prototype	2026-09-25T10:15:27Z	2026-09-25T10:16:11Z	4.40 6.18 8.60
3	baseline	2026-09-25T10:16:11Z	2026-09-25T10:16:54Z	5.17 6.05 8.42
3	prototype-flags	2026-09-25T10:16:54Z	2026-09-25T10:17:46Z	4.15 5.67 8.17
```

Records: 5040. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.48 | 22.60 | 24.33 | 24.14 | 60 | 25.31 | 24.46 | 25.90 | 25.82 | 1.073 | 1.069 1.084 1.073 | 24,800 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 25.35 | 24.40 | 26.52 | 25.85 | 60 | 27.62 | 26.76 | 31.55 | 28.33 | 1.090 | 1.099 1.090 1.083 | 24,800 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 23.47 | 22.96 | 24.76 | 23.90 | 60 | 25.64 | 24.74 | 27.34 | 26.53 | 1.103 | 1.110 1.080 1.103 | 24,800 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 32.91 | 31.68 | 34.23 | 33.77 | 60 | 35.26 | 33.60 | 36.75 | 36.29 | 1.067 | 1.097 1.067 1.061 | 24,800 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.96 | 12.64 | 13.18 | 13.16 | 60 | 13.65 | 13.22 | 14.11 | 13.95 | 1.051 | 1.056 1.051 1.051 | 24,800 | 24,799 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.49 | 9.87 | 11.47 | 11.00 | 60 | 10.53 | 9.81 | 12.16 | 11.40 | 1.002 | 1.026 1.002 1.001 | 24,800 | 24,799 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 23.23 | 22.35 | 24.35 | 23.77 | 60 | 25.72 | 24.62 | 27.07 | 26.59 | 1.110 | 1.110 1.117 1.105 | 24,800 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 25.64 | 24.82 | 26.46 | 26.20 | 60 | 28.23 | 27.43 | 29.34 | 28.96 | 1.107 | 1.107 1.116 1.095 | 24,800 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 23.66 | 22.78 | 24.33 | 24.09 | 60 | 25.95 | 25.15 | 27.88 | 26.64 | 1.094 | 1.108 1.091 1.094 | 24,800 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 32.71 | 31.57 | 35.42 | 34.13 | 60 | 35.03 | 33.47 | 35.96 | 35.78 | 1.066 | 1.066 1.059 1.078 | 24,800 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.87 | 13.52 | 14.06 | 14.03 | 60 | 14.64 | 14.25 | 14.99 | 14.94 | 1.061 | 1.067 1.061 1.049 | 24,800 | 24,799 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.51 | 12.09 | 12.95 | 12.81 | 60 | 12.57 | 11.62 | 13.61 | 13.16 | 1.007 | 1.028 0.993 1.007 | 24,800 | 24,799 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 25.51 | 24.53 | 26.13 | 26.03 | 60 | 27.56 | 26.42 | 28.46 | 28.23 | 1.084 | 1.087 1.084 1.071 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.24 | 28.05 | 29.90 | 29.88 | 60 | 31.20 | 29.96 | 32.35 | 31.75 | 1.061 | 1.095 1.061 1.047 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 25.74 | 25.08 | 26.43 | 26.40 | 60 | 27.91 | 26.99 | 29.25 | 28.57 | 1.079 | 1.105 1.069 1.079 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 35.56 | 34.28 | 37.22 | 36.70 | 60 | 38.35 | 37.26 | 39.51 | 39.34 | 1.081 | 1.090 1.063 1.081 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.65 | 13.41 | 13.25 | 60 | 13.75 | 13.46 | 16.32 | 13.96 | 1.055 | 1.055 1.065 1.053 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.68 | 10.01 | 11.35 | 11.14 | 60 | 10.77 | 10.33 | 13.30 | 11.24 | 1.011 | 1.011 0.990 1.014 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 25.77 | 25.17 | 28.19 | 26.47 | 60 | 28.22 | 27.31 | 31.01 | 29.73 | 1.101 | 1.101 1.117 1.084 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 29.71 | 28.74 | 30.59 | 30.47 | 60 | 32.11 | 31.15 | 32.99 | 32.68 | 1.077 | 1.092 1.077 1.072 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 26.32 | 25.51 | 27.06 | 26.95 | 60 | 28.59 | 27.49 | 29.55 | 29.25 | 1.080 | 1.108 1.068 1.080 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 35.66 | 34.62 | 37.06 | 36.68 | 60 | 38.29 | 37.27 | 39.36 | 38.93 | 1.078 | 1.083 1.060 1.078 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.85 | 13.57 | 14.30 | 14.24 | 60 | 14.71 | 14.43 | 15.02 | 14.95 | 1.066 | 1.056 1.066 1.067 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.68 | 12.07 | 13.76 | 13.58 | 60 | 12.69 | 12.16 | 13.32 | 13.08 | 1.007 | 0.994 1.007 1.009 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.31 | 24.46 | 25.90 | 25.82 | 60 | 25.17 | 24.14 | 25.97 | 25.84 | 0.995 | 0.995 0.991 1.003 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.62 | 26.76 | 31.55 | 28.33 | 60 | 27.55 | 26.56 | 28.17 | 28.04 | 0.989 | 0.989 0.989 1.003 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.64 | 24.74 | 27.34 | 26.53 | 60 | 25.42 | 24.20 | 26.38 | 26.07 | 0.981 | 0.965 1.006 0.981 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 35.26 | 33.60 | 36.75 | 36.29 | 60 | 34.19 | 33.05 | 35.54 | 35.04 | 0.968 | 0.951 0.986 0.968 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 13.65 | 13.22 | 14.11 | 13.95 | 60 | 12.97 | 12.60 | 13.36 | 13.29 | 0.944 | 0.938 0.944 0.972 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.53 | 9.81 | 12.16 | 11.40 | 60 | 10.50 | 9.85 | 11.65 | 11.28 | 0.996 | 0.978 0.996 1.018 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.72 | 24.62 | 27.07 | 26.59 | 60 | 24.93 | 23.99 | 25.78 | 25.53 | 0.965 | 0.956 0.986 0.965 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 28.23 | 27.43 | 29.34 | 28.96 | 60 | 27.50 | 26.73 | 28.20 | 28.03 | 0.977 | 0.953 0.986 0.977 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 25.95 | 25.15 | 27.88 | 26.64 | 60 | 25.45 | 24.73 | 28.32 | 26.54 | 0.972 | 0.972 1.006 0.970 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 35.03 | 33.47 | 35.96 | 35.78 | 60 | 34.29 | 32.76 | 35.76 | 35.42 | 0.977 | 0.977 0.996 0.967 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 14.64 | 14.25 | 14.99 | 14.94 | 60 | 13.94 | 13.09 | 14.32 | 14.17 | 0.948 | 0.948 0.914 0.962 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.57 | 11.62 | 13.61 | 13.16 | 60 | 12.56 | 11.74 | 13.78 | 13.30 | 1.010 | 0.969 1.010 1.012 | - | - | 24,799 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 27.56 | 26.42 | 28.46 | 28.23 | 60 | 47.62 | 39.40 | 50.38 | 49.74 | 1.719 | 1.680 1.719 1.770 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.20 | 29.96 | 32.35 | 31.75 | 60 | 53.63 | 44.26 | 56.09 | 55.45 | 1.718 | 1.628 1.718 1.751 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 27.91 | 26.99 | 29.25 | 28.57 | 60 | 48.29 | 44.01 | 51.22 | 50.54 | 1.726 | 1.685 1.764 1.726 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 38.35 | 37.26 | 39.51 | 39.34 | 60 | 67.16 | 60.48 | 69.43 | 68.82 | 1.737 | 1.737 1.666 1.775 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 13.75 | 13.46 | 16.32 | 13.96 | 60 | 12.46 | 11.69 | 13.34 | 13.17 | 0.910 | 0.857 0.910 0.951 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.77 | 10.33 | 13.30 | 11.24 | 60 | 10.61 | 10.01 | 11.14 | 11.01 | 0.981 | 0.978 1.004 0.981 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 28.22 | 27.31 | 31.01 | 29.73 | 60 | 48.55 | 41.59 | 50.72 | 50.35 | 1.704 | 1.672 1.704 1.749 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 32.11 | 31.15 | 32.99 | 32.68 | 60 | 53.76 | 47.82 | 56.31 | 55.73 | 1.683 | 1.637 1.702 1.683 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.59 | 27.49 | 29.55 | 29.25 | 60 | 48.75 | 42.70 | 51.36 | 50.87 | 1.731 | 1.671 1.756 1.731 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 38.29 | 37.27 | 39.36 | 38.93 | 60 | 67.41 | 60.36 | 69.28 | 68.96 | 1.778 | 1.726 1.778 1.793 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 14.71 | 14.43 | 15.02 | 14.95 | 60 | 13.25 | 12.63 | 13.98 | 13.81 | 0.905 | 0.867 0.905 0.928 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.69 | 12.16 | 13.32 | 13.08 | 60 | 12.70 | 12.08 | 13.55 | 13.35 | 0.997 | 0.996 0.997 1.005 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 2.80 | 2.71 | 2.90 | 2.86 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 5.45 | 5.24 | 5.59 | 5.56 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 60 | 3.29 | 3.05 | 3.44 | 3.38 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 60 | 7.09 | 6.95 | 7.49 | 7.38 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 11.66 | 10.98 | 12.47 | 12.24 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.37 | 0.36 | 0.42 | 0.40 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 2.77 | 2.67 | 2.88 | 2.85 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 5.46 | 5.26 | 5.56 | 5.54 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 3.32 | 3.18 | 3.45 | 3.40 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 7.14 | 6.94 | 7.50 | 7.30 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 12.59 | 11.97 | 13.50 | 13.09 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.39 | 0.38 | 0.44 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.48 | 22.60 | 24.33 | 24.14 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 25.35 | 24.40 | 26.52 | 25.85 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 23.47 | 22.96 | 24.76 | 23.90 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 32.91 | 31.68 | 34.23 | 33.77 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.96 | 12.64 | 13.18 | 13.16 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.49 | 9.87 | 11.47 | 11.00 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 23.23 | 22.35 | 24.35 | 23.77 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 25.64 | 24.82 | 26.46 | 26.20 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 23.66 | 22.78 | 24.33 | 24.09 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 32.71 | 31.57 | 35.42 | 34.13 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.87 | 13.52 | 14.06 | 14.03 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.51 | 12.09 | 12.95 | 12.81 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 25.51 | 24.53 | 26.13 | 26.03 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.24 | 28.05 | 29.90 | 29.88 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 25.74 | 25.08 | 26.43 | 26.40 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 35.56 | 34.28 | 37.22 | 36.70 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.65 | 13.41 | 13.25 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.68 | 10.01 | 11.35 | 11.14 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 25.77 | 25.17 | 28.19 | 26.47 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 29.71 | 28.74 | 30.59 | 30.47 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 26.32 | 25.51 | 27.06 | 26.95 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 35.66 | 34.62 | 37.06 | 36.68 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.85 | 13.57 | 14.30 | 14.24 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.68 | 12.07 | 13.76 | 13.58 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.31 | 24.46 | 25.90 | 25.82 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.62 | 26.76 | 31.55 | 28.33 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.64 | 24.74 | 27.34 | 26.53 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 35.26 | 33.60 | 36.75 | 36.29 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 13.65 | 13.22 | 14.11 | 13.95 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.53 | 9.81 | 12.16 | 11.40 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.72 | 24.62 | 27.07 | 26.59 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 28.23 | 27.43 | 29.34 | 28.96 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 25.95 | 25.15 | 27.88 | 26.64 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 35.03 | 33.47 | 35.96 | 35.78 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 14.64 | 14.25 | 14.99 | 14.94 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.57 | 11.62 | 13.61 | 13.16 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 27.56 | 26.42 | 28.46 | 28.23 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.20 | 29.96 | 32.35 | 31.75 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.91 | 26.99 | 29.25 | 28.57 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 38.35 | 37.26 | 39.51 | 39.34 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 13.75 | 13.46 | 16.32 | 13.96 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.77 | 10.33 | 13.30 | 11.24 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 28.22 | 27.31 | 31.01 | 29.73 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 32.11 | 31.15 | 32.99 | 32.68 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.59 | 27.49 | 29.55 | 29.25 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 38.29 | 37.27 | 39.36 | 38.93 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 14.71 | 14.43 | 15.02 | 14.95 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.69 | 12.16 | 13.32 | 13.08 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 60 | 25.17 | 24.14 | 25.97 | 25.84 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 60 | 27.55 | 26.56 | 28.17 | 28.04 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 60 | 25.42 | 24.20 | 26.38 | 26.07 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 60 | 34.19 | 33.05 | 35.54 | 35.04 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 60 | 12.97 | 12.60 | 13.36 | 13.29 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 60 | 10.50 | 9.85 | 11.65 | 11.28 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 60 | 24.93 | 23.99 | 25.78 | 25.53 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 60 | 27.50 | 26.73 | 28.20 | 28.03 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 60 | 25.45 | 24.73 | 28.32 | 26.54 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 60 | 34.29 | 32.76 | 35.76 | 35.42 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 60 | 13.94 | 13.09 | 14.32 | 14.17 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 60 | 12.56 | 11.74 | 13.78 | 13.30 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 60 | 47.62 | 39.40 | 50.38 | 49.74 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 60 | 53.63 | 44.26 | 56.09 | 55.45 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 60 | 48.29 | 44.01 | 51.22 | 50.54 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 60 | 67.16 | 60.48 | 69.43 | 68.82 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 60 | 12.46 | 11.69 | 13.34 | 13.17 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 60 | 10.61 | 10.01 | 11.14 | 11.01 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 60 | 48.55 | 41.59 | 50.72 | 50.35 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 60 | 53.76 | 47.82 | 56.31 | 55.73 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 60 | 48.75 | 42.70 | 51.36 | 50.87 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 60 | 67.41 | 60.36 | 69.28 | 68.96 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 60 | 13.25 | 12.63 | 13.98 | 13.81 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 60 | 12.70 | 12.08 | 13.55 | 13.35 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 60 | 2.80 | 2.71 | 2.90 | 2.86 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 60 | 5.45 | 5.24 | 5.59 | 5.56 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 60 | 3.29 | 3.05 | 3.44 | 3.38 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 60 | 7.09 | 6.95 | 7.49 | 7.38 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 60 | 11.66 | 10.98 | 12.47 | 12.24 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 60 | 0.37 | 0.36 | 0.42 | 0.40 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 60 | 2.77 | 2.67 | 2.88 | 2.85 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 60 | 5.46 | 5.26 | 5.56 | 5.54 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 60 | 3.32 | 3.18 | 3.45 | 3.40 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 60 | 7.14 | 6.94 | 7.50 | 7.30 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 60 | 12.59 | 11.97 | 13.50 | 13.09 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 60 | 0.39 | 0.38 | 0.44 | 0.42 | - | - | - | - |
