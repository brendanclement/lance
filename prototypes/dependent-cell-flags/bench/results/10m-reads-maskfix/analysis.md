## Results: 10m reads after the mask fix (26388225a), baseline first

Source: `results/10m-reads-maskfix`

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
| flag_every_round | `True` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md; ?? prototypes/dependent-cell-flags/bench/results/1m-reads-maskfix/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T08:51:40Z	2026-09-25T08:52:23Z	13.86 15.60 9.55
1	prototype	2026-09-25T08:52:23Z	2026-09-25T08:53:06Z	7.90 13.75 9.18
1	prototype-flags	2026-09-25T08:53:06Z	2026-09-25T08:53:58Z	6.42 12.52 8.95
2	baseline	2026-09-25T08:53:58Z	2026-09-25T08:54:40Z	5.57 11.30 8.71
2	prototype	2026-09-25T08:54:40Z	2026-09-25T08:55:24Z	6.75 10.90 8.68
2	prototype-flags	2026-09-25T08:55:24Z	2026-09-25T08:56:16Z	5.69 10.00 8.46
3	baseline	2026-09-25T08:56:16Z	2026-09-25T08:56:59Z	6.21 9.38 8.31
3	prototype	2026-09-25T08:56:59Z	2026-09-25T08:57:42Z	5.80 8.77 8.13
3	prototype-flags	2026-09-25T08:57:42Z	2026-09-25T08:58:34Z	5.97 8.32 8.00
```

Records: 5040. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.27 | 22.53 | 24.87 | 23.98 | 60 | 24.89 | 24.07 | 25.93 | 25.67 | 1.070 | 1.044 1.095 1.070 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 25.50 | 24.52 | 26.47 | 26.27 | 60 | 27.40 | 26.44 | 28.77 | 28.26 | 1.064 | 1.048 1.101 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 23.88 | 23.18 | 24.79 | 24.46 | 60 | 25.61 | 24.48 | 26.73 | 26.44 | 1.065 | 1.053 1.105 1.065 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.17 | 31.90 | 34.44 | 34.12 | 60 | 35.13 | 34.04 | 36.52 | 36.15 | 1.053 | 1.053 1.082 1.049 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 13.22 | 12.36 | 13.77 | 13.58 | 60 | 13.35 | 12.95 | 13.71 | 13.66 | 1.007 | 1.007 1.030 1.003 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.61 | 9.90 | 11.29 | 11.03 | 60 | 10.59 | 9.84 | 11.08 | 10.97 | 0.994 | 0.994 1.008 0.994 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 23.42 | 22.77 | 24.33 | 23.98 | 60 | 25.24 | 24.33 | 26.17 | 25.75 | 1.079 | 1.070 1.087 1.079 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 25.75 | 25.09 | 26.85 | 26.38 | 60 | 28.02 | 27.04 | 28.96 | 28.73 | 1.082 | 1.082 1.108 1.069 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 23.90 | 22.99 | 24.93 | 24.83 | 60 | 25.77 | 24.92 | 28.26 | 27.06 | 1.068 | 1.068 1.111 1.060 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.10 | 31.63 | 35.30 | 34.41 | 60 | 35.34 | 34.04 | 42.71 | 38.82 | 1.068 | 1.068 1.080 1.065 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.89 | 13.23 | 16.03 | 14.22 | 60 | 14.21 | 13.87 | 14.78 | 14.57 | 1.019 | 1.019 1.055 1.014 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.57 | 11.87 | 13.60 | 13.24 | 60 | 12.39 | 11.89 | 12.89 | 12.83 | 0.986 | 0.968 0.992 0.986 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 25.46 | 24.67 | 26.23 | 26.06 | 60 | 27.33 | 26.66 | 28.29 | 28.12 | 1.067 | 1.055 1.094 1.067 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.01 | 27.83 | 30.20 | 29.71 | 60 | 31.21 | 30.12 | 33.22 | 32.10 | 1.080 | 1.059 1.103 1.080 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 25.90 | 25.05 | 27.71 | 26.84 | 60 | 27.81 | 26.71 | 29.03 | 28.70 | 1.068 | 1.037 1.100 1.068 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 35.91 | 34.79 | 37.42 | 37.08 | 60 | 38.06 | 37.08 | 39.14 | 38.68 | 1.063 | 1.048 1.078 1.063 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 13.06 | 12.37 | 13.49 | 13.31 | 60 | 13.44 | 13.06 | 13.80 | 13.69 | 1.032 | 1.032 1.057 1.030 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.76 | 10.06 | 11.91 | 11.31 | 60 | 10.87 | 10.18 | 12.91 | 11.47 | 1.014 | 1.000 1.016 1.014 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 25.75 | 24.44 | 28.17 | 26.65 | 60 | 28.00 | 27.34 | 30.38 | 28.71 | 1.089 | 1.073 1.106 1.089 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 29.72 | 28.71 | 31.57 | 30.59 | 60 | 32.10 | 31.02 | 33.14 | 32.69 | 1.077 | 1.060 1.104 1.077 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 26.36 | 25.63 | 27.06 | 26.96 | 60 | 28.18 | 27.39 | 29.23 | 28.97 | 1.063 | 1.052 1.102 1.063 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 35.66 | 34.54 | 37.95 | 37.23 | 60 | 37.76 | 37.08 | 38.78 | 38.59 | 1.064 | 1.037 1.064 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.94 | 13.18 | 14.43 | 14.22 | 60 | 14.34 | 14.02 | 14.66 | 14.57 | 1.027 | 1.027 1.062 1.022 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.70 | 11.89 | 13.82 | 13.69 | 60 | 12.84 | 12.31 | 13.84 | 13.62 | 1.016 | 0.967 1.050 1.016 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 24.89 | 24.07 | 25.93 | 25.67 | 60 | 24.21 | 23.03 | 25.44 | 24.93 | 0.971 | 0.968 0.971 0.975 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.40 | 26.44 | 28.77 | 28.26 | 60 | 26.45 | 25.52 | 27.42 | 27.16 | 0.965 | 0.960 0.965 0.975 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.61 | 24.48 | 26.73 | 26.44 | 60 | 24.70 | 23.54 | 26.03 | 25.60 | 0.958 | 0.949 0.958 0.975 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 35.13 | 34.04 | 36.52 | 36.15 | 60 | 33.92 | 32.36 | 35.06 | 34.87 | 0.964 | 0.958 0.964 0.982 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 13.35 | 12.95 | 13.71 | 13.66 | 60 | 13.32 | 13.00 | 13.57 | 13.50 | 1.006 | 0.976 1.010 1.006 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.59 | 9.84 | 11.08 | 10.97 | 60 | 10.58 | 10.01 | 11.16 | 10.97 | 1.001 | 1.001 0.997 1.001 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.24 | 24.33 | 26.17 | 25.75 | 60 | 24.60 | 23.65 | 25.73 | 25.51 | 0.976 | 0.960 0.976 0.995 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 28.02 | 27.04 | 28.96 | 28.73 | 60 | 27.17 | 26.10 | 28.38 | 27.86 | 0.971 | 0.950 0.971 0.989 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 25.77 | 24.92 | 28.26 | 27.06 | 60 | 25.32 | 23.66 | 28.76 | 26.54 | 0.980 | 0.952 0.980 0.988 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 35.34 | 34.04 | 42.71 | 38.82 | 60 | 33.90 | 32.25 | 35.35 | 35.06 | 0.966 | 0.923 0.966 0.986 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 14.21 | 13.87 | 14.78 | 14.57 | 60 | 14.25 | 13.94 | 14.68 | 14.59 | 1.001 | 0.993 1.001 1.008 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.39 | 11.89 | 12.89 | 12.83 | 60 | 12.47 | 11.67 | 13.42 | 13.27 | 1.008 | 1.008 0.996 1.033 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 27.33 | 26.66 | 28.29 | 28.12 | 60 | 48.38 | 43.15 | 53.54 | 50.09 | 1.760 | 1.725 1.760 1.797 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.21 | 30.12 | 33.22 | 32.10 | 60 | 53.29 | 46.53 | 56.02 | 55.23 | 1.717 | 1.717 1.688 1.726 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 27.81 | 26.71 | 29.03 | 28.70 | 60 | 48.10 | 41.90 | 51.38 | 50.84 | 1.713 | 1.698 1.713 1.792 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 38.06 | 37.08 | 39.14 | 38.68 | 60 | 67.20 | 61.28 | 69.45 | 68.79 | 1.788 | 1.758 1.788 1.794 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 13.44 | 13.06 | 13.80 | 13.69 | 60 | 13.21 | 12.88 | 13.70 | 13.45 | 0.984 | 0.964 0.996 0.984 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.87 | 10.18 | 12.91 | 11.47 | 60 | 10.74 | 10.18 | 12.07 | 11.67 | 0.985 | 0.993 0.985 0.984 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 28.00 | 27.34 | 30.38 | 28.71 | 60 | 48.90 | 42.53 | 50.72 | 50.38 | 1.742 | 1.714 1.742 1.755 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 32.10 | 31.02 | 33.14 | 32.69 | 60 | 53.86 | 45.37 | 56.22 | 55.54 | 1.684 | 1.663 1.684 1.694 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.18 | 27.39 | 29.23 | 28.97 | 60 | 49.07 | 42.08 | 51.56 | 50.34 | 1.718 | 1.712 1.718 1.778 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 37.76 | 37.08 | 38.78 | 38.59 | 60 | 67.66 | 61.41 | 69.31 | 68.79 | 1.803 | 1.782 1.806 1.803 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 14.34 | 14.02 | 14.66 | 14.57 | 60 | 13.89 | 13.53 | 14.21 | 14.17 | 0.970 | 0.945 0.970 0.985 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.84 | 12.31 | 13.84 | 13.62 | 60 | 12.61 | 12.13 | 13.89 | 13.36 | 0.982 | 0.998 0.952 0.982 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 2.71 | 2.63 | 2.80 | 2.78 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 5.45 | 5.31 | 5.65 | 5.61 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 60 | 3.28 | 3.21 | 6.43 | 3.37 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 60 | 7.14 | 6.85 | 7.47 | 7.38 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 12.42 | 12.19 | 12.68 | 12.59 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.37 | 0.36 | 0.44 | 0.41 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 2.71 | 2.58 | 2.77 | 2.75 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 5.43 | 5.30 | 5.66 | 5.62 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 3.30 | 3.20 | 3.45 | 3.39 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 7.18 | 6.95 | 7.43 | 7.36 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 13.28 | 12.99 | 13.62 | 13.57 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.39 | 0.37 | 0.46 | 0.43 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.27 | 22.53 | 24.87 | 23.98 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 25.50 | 24.52 | 26.47 | 26.27 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 23.88 | 23.18 | 24.79 | 24.46 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.17 | 31.90 | 34.44 | 34.12 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 13.22 | 12.36 | 13.77 | 13.58 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.61 | 9.90 | 11.29 | 11.03 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 23.42 | 22.77 | 24.33 | 23.98 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 25.75 | 25.09 | 26.85 | 26.38 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 23.90 | 22.99 | 24.93 | 24.83 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.10 | 31.63 | 35.30 | 34.41 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.89 | 13.23 | 16.03 | 14.22 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.57 | 11.87 | 13.60 | 13.24 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 25.46 | 24.67 | 26.23 | 26.06 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.01 | 27.83 | 30.20 | 29.71 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 25.90 | 25.05 | 27.71 | 26.84 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 35.91 | 34.79 | 37.42 | 37.08 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 13.06 | 12.37 | 13.49 | 13.31 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.76 | 10.06 | 11.91 | 11.31 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 25.75 | 24.44 | 28.17 | 26.65 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 29.72 | 28.71 | 31.57 | 30.59 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 26.36 | 25.63 | 27.06 | 26.96 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 35.66 | 34.54 | 37.95 | 37.23 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.94 | 13.18 | 14.43 | 14.22 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.70 | 11.89 | 13.82 | 13.69 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 24.89 | 24.07 | 25.93 | 25.67 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.40 | 26.44 | 28.77 | 28.26 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.61 | 24.48 | 26.73 | 26.44 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 35.13 | 34.04 | 36.52 | 36.15 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 13.35 | 12.95 | 13.71 | 13.66 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.59 | 9.84 | 11.08 | 10.97 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.24 | 24.33 | 26.17 | 25.75 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 28.02 | 27.04 | 28.96 | 28.73 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 25.77 | 24.92 | 28.26 | 27.06 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 35.34 | 34.04 | 42.71 | 38.82 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 14.21 | 13.87 | 14.78 | 14.57 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.39 | 11.89 | 12.89 | 12.83 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 27.33 | 26.66 | 28.29 | 28.12 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.21 | 30.12 | 33.22 | 32.10 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.81 | 26.71 | 29.03 | 28.70 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 38.06 | 37.08 | 39.14 | 38.68 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 13.44 | 13.06 | 13.80 | 13.69 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.87 | 10.18 | 12.91 | 11.47 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 28.00 | 27.34 | 30.38 | 28.71 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 32.10 | 31.02 | 33.14 | 32.69 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.18 | 27.39 | 29.23 | 28.97 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.76 | 37.08 | 38.78 | 38.59 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 14.34 | 14.02 | 14.66 | 14.57 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.84 | 12.31 | 13.84 | 13.62 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 60 | 24.21 | 23.03 | 25.44 | 24.93 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 60 | 26.45 | 25.52 | 27.42 | 27.16 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 60 | 24.70 | 23.54 | 26.03 | 25.60 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 60 | 33.92 | 32.36 | 35.06 | 34.87 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 60 | 13.32 | 13.00 | 13.57 | 13.50 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 60 | 10.58 | 10.01 | 11.16 | 10.97 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 60 | 24.60 | 23.65 | 25.73 | 25.51 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 60 | 27.17 | 26.10 | 28.38 | 27.86 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 60 | 25.32 | 23.66 | 28.76 | 26.54 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 60 | 33.90 | 32.25 | 35.35 | 35.06 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 60 | 14.25 | 13.94 | 14.68 | 14.59 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 60 | 12.47 | 11.67 | 13.42 | 13.27 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 60 | 48.38 | 43.15 | 53.54 | 50.09 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 60 | 53.29 | 46.53 | 56.02 | 55.23 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 60 | 48.10 | 41.90 | 51.38 | 50.84 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 60 | 67.20 | 61.28 | 69.45 | 68.79 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 60 | 13.21 | 12.88 | 13.70 | 13.45 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 60 | 10.74 | 10.18 | 12.07 | 11.67 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 60 | 48.90 | 42.53 | 50.72 | 50.38 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 60 | 53.86 | 45.37 | 56.22 | 55.54 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 60 | 49.07 | 42.08 | 51.56 | 50.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 60 | 67.66 | 61.41 | 69.31 | 68.79 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 60 | 13.89 | 13.53 | 14.21 | 14.17 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 60 | 12.61 | 12.13 | 13.89 | 13.36 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 60 | 2.71 | 2.63 | 2.80 | 2.78 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 60 | 5.45 | 5.31 | 5.65 | 5.61 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 60 | 3.28 | 3.21 | 6.43 | 3.37 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 60 | 7.14 | 6.85 | 7.47 | 7.38 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 60 | 12.42 | 12.19 | 12.68 | 12.59 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 60 | 0.37 | 0.36 | 0.44 | 0.41 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 60 | 2.71 | 2.58 | 2.77 | 2.75 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 60 | 5.43 | 5.30 | 5.66 | 5.62 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 60 | 3.30 | 3.20 | 3.45 | 3.39 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 60 | 7.18 | 6.95 | 7.43 | 7.36 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 60 | 13.28 | 12.99 | 13.62 | 13.57 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 60 | 0.39 | 0.37 | 0.46 | 0.43 | - | - | - | - |
