## Clean build: 10m reads at 26388225a built outside the nested checkout (rustflags identical to the baseline), baseline first

Source: `results/10m-reads-clean-baseline-first`

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
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M prototypes/dependent-cell-flags/bench/run_paired.sh |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T10:03:48Z	2026-09-25T10:04:32Z	23.37 17.89 13.25
1	prototype	2026-09-25T10:04:32Z	2026-09-25T10:05:17Z	12.97 15.87 12.75
1	prototype-flags	2026-09-25T10:05:17Z	2026-09-25T10:06:11Z	7.69 14.07 12.24
2	baseline	2026-09-25T10:06:11Z	2026-09-25T10:06:54Z	5.73 12.46 11.76
2	prototype	2026-09-25T10:06:54Z	2026-09-25T10:07:37Z	4.71 11.33 11.37
2	prototype-flags	2026-09-25T10:07:37Z	2026-09-25T10:08:29Z	5.35 10.54 11.08
3	baseline	2026-09-25T10:08:29Z	2026-09-25T10:09:12Z	6.01 9.89 10.80
3	prototype	2026-09-25T10:09:12Z	2026-09-25T10:09:56Z	5.78 9.26 10.52
3	prototype-flags	2026-09-25T10:09:56Z	2026-09-25T10:10:48Z	6.08 8.80 10.28
```

Records: 5040. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.54 | 22.87 | 24.34 | 24.18 | 60 | 25.01 | 24.14 | 25.85 | 25.74 | 1.058 | 1.056 1.058 1.070 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 25.73 | 25.11 | 27.30 | 26.43 | 60 | 27.31 | 26.47 | 28.12 | 27.82 | 1.054 | 1.054 1.048 1.073 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 23.94 | 23.26 | 24.92 | 24.61 | 60 | 25.39 | 24.41 | 26.33 | 26.10 | 1.059 | 1.048 1.059 1.073 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.06 | 31.76 | 34.44 | 34.16 | 60 | 35.11 | 33.84 | 39.39 | 36.39 | 1.057 | 1.057 1.051 1.086 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.94 | 12.57 | 13.35 | 13.32 | 60 | 13.65 | 12.82 | 15.28 | 13.97 | 1.036 | 1.029 1.074 1.036 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.59 | 9.99 | 13.46 | 13.06 | 60 | 10.69 | 9.99 | 11.20 | 11.15 | 1.002 | 1.027 0.982 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 23.79 | 22.75 | 25.91 | 25.47 | 60 | 25.34 | 24.26 | 26.82 | 25.93 | 1.072 | 1.022 1.072 1.072 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.08 | 25.04 | 27.05 | 26.86 | 60 | 27.87 | 26.91 | 28.58 | 28.41 | 1.063 | 1.063 1.072 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.51 | 23.57 | 26.07 | 25.73 | 60 | 25.62 | 24.97 | 27.23 | 26.24 | 1.057 | 1.057 1.060 1.029 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.60 | 31.63 | 40.29 | 39.13 | 60 | 34.84 | 33.34 | 36.76 | 35.81 | 1.049 | 1.052 1.049 0.983 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.81 | 13.60 | 14.16 | 14.09 | 60 | 14.59 | 13.78 | 15.22 | 14.95 | 1.059 | 1.029 1.069 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.43 | 11.92 | 13.16 | 12.88 | 60 | 12.60 | 12.16 | 13.36 | 13.18 | 1.016 | 1.034 0.985 1.016 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 25.97 | 25.40 | 26.60 | 26.45 | 60 | 27.20 | 25.74 | 29.15 | 28.85 | 1.054 | 1.096 1.054 1.021 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.53 | 28.65 | 30.96 | 29.96 | 60 | 31.06 | 29.77 | 33.07 | 32.77 | 1.064 | 1.092 1.064 1.029 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.33 | 25.76 | 27.18 | 27.09 | 60 | 27.82 | 26.40 | 29.52 | 29.21 | 1.058 | 1.099 1.058 1.023 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 36.34 | 35.22 | 38.20 | 37.09 | 60 | 38.04 | 35.92 | 40.58 | 39.84 | 1.046 | 1.083 1.046 1.023 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 13.09 | 12.75 | 13.46 | 13.27 | 60 | 13.67 | 13.00 | 14.05 | 13.95 | 1.045 | 1.017 1.057 1.045 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.71 | 10.00 | 12.89 | 11.08 | 60 | 11.03 | 10.13 | 12.93 | 12.27 | 1.044 | 1.080 1.009 1.044 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.50 | 25.39 | 29.25 | 27.07 | 60 | 27.98 | 26.40 | 30.46 | 29.62 | 1.062 | 1.099 1.062 1.013 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.23 | 29.19 | 31.28 | 30.98 | 60 | 32.00 | 30.73 | 33.94 | 33.81 | 1.072 | 1.116 1.072 1.017 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.17 | 26.20 | 28.13 | 27.89 | 60 | 28.36 | 27.41 | 29.95 | 29.87 | 1.062 | 1.077 1.062 1.020 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.34 | 34.98 | 37.81 | 37.34 | 60 | 38.03 | 36.05 | 39.84 | 39.74 | 1.059 | 1.068 1.059 1.014 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.99 | 13.77 | 14.20 | 14.16 | 60 | 14.68 | 13.94 | 15.12 | 14.95 | 1.054 | 1.017 1.058 1.054 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.80 | 12.31 | 13.58 | 13.29 | 60 | 12.72 | 12.12 | 13.25 | 13.04 | 0.991 | 0.991 0.990 1.008 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.01 | 24.14 | 25.85 | 25.74 | 60 | 25.37 | 24.24 | 27.67 | 26.34 | 1.012 | 1.003 1.028 1.012 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.31 | 26.47 | 28.12 | 27.82 | 60 | 27.44 | 26.39 | 28.59 | 28.46 | 1.008 | 0.991 1.008 1.022 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.39 | 24.41 | 26.33 | 26.10 | 60 | 25.42 | 24.54 | 26.59 | 26.14 | 1.007 | 0.995 1.007 1.008 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 35.11 | 33.84 | 39.39 | 36.39 | 60 | 34.46 | 32.75 | 36.28 | 35.74 | 0.974 | 0.970 0.995 0.974 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 13.65 | 12.82 | 15.28 | 13.97 | 60 | 13.03 | 12.59 | 14.20 | 13.73 | 0.975 | 0.975 0.940 1.000 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.69 | 9.99 | 11.20 | 11.15 | 60 | 10.85 | 10.18 | 12.39 | 12.00 | 1.041 | 0.988 1.084 1.041 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.34 | 24.26 | 26.82 | 25.93 | 60 | 25.16 | 24.34 | 26.25 | 25.96 | 0.992 | 0.988 1.008 0.992 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 27.87 | 26.91 | 28.58 | 28.41 | 60 | 27.83 | 26.76 | 29.22 | 28.80 | 1.004 | 0.989 1.004 1.014 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 25.62 | 24.97 | 27.23 | 26.24 | 60 | 25.61 | 24.79 | 27.13 | 26.64 | 1.008 | 0.978 1.008 1.011 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 34.84 | 33.34 | 36.76 | 35.81 | 60 | 34.50 | 33.31 | 36.05 | 35.65 | 0.988 | 0.982 0.999 0.988 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 14.59 | 13.78 | 15.22 | 14.95 | 60 | 14.01 | 13.54 | 14.83 | 14.67 | 0.977 | 0.977 0.949 0.991 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.60 | 12.16 | 13.36 | 13.18 | 60 | 12.59 | 11.90 | 13.73 | 13.49 | 0.993 | 1.031 0.989 0.993 | - | - | 24,800 | 26,459 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 27.20 | 25.74 | 29.15 | 28.85 | 60 | 47.73 | 40.83 | 49.79 | 49.14 | 1.706 | 1.651 1.706 1.837 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.06 | 29.77 | 33.07 | 32.77 | 60 | 53.31 | 45.93 | 56.25 | 55.20 | 1.717 | 1.633 1.717 1.759 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 27.82 | 26.40 | 29.52 | 29.21 | 60 | 47.70 | 42.13 | 50.93 | 49.65 | 1.731 | 1.616 1.731 1.744 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 38.04 | 35.92 | 40.58 | 39.84 | 60 | 67.66 | 60.82 | 68.86 | 68.52 | 1.785 | 1.706 1.785 1.837 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 13.67 | 13.00 | 14.05 | 13.95 | 60 | 12.91 | 12.52 | 13.47 | 13.39 | 0.964 | 0.964 0.928 0.966 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 11.03 | 10.13 | 12.93 | 12.27 | 60 | 10.69 | 9.98 | 11.68 | 11.38 | 0.957 | 0.919 0.994 0.957 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 27.98 | 26.40 | 30.46 | 29.62 | 60 | 47.73 | 42.57 | 51.29 | 49.65 | 1.711 | 1.636 1.711 1.769 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 32.00 | 30.73 | 33.94 | 33.81 | 60 | 53.57 | 46.92 | 56.29 | 56.14 | 1.685 | 1.609 1.685 1.715 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.36 | 27.41 | 29.95 | 29.87 | 60 | 48.49 | 42.34 | 51.35 | 50.35 | 1.744 | 1.581 1.744 1.755 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 38.03 | 36.05 | 39.84 | 39.74 | 60 | 67.47 | 61.28 | 69.93 | 69.08 | 1.799 | 1.709 1.799 1.853 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 14.68 | 13.94 | 15.12 | 14.95 | 60 | 13.53 | 13.19 | 18.10 | 15.53 | 0.949 | 0.949 0.909 0.951 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.72 | 12.12 | 13.25 | 13.04 | 60 | 12.70 | 11.86 | 13.61 | 13.45 | 0.999 | 1.001 0.999 0.976 | - | - | 24,800 | 858,408 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 2.77 | 2.49 | 2.95 | 2.89 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 5.50 | 5.29 | 8.73 | 5.66 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 60 | 3.31 | 3.20 | 3.45 | 3.41 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 60 | 7.22 | 7.10 | 8.11 | 7.57 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 12.04 | 11.56 | 12.71 | 12.58 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.37 | 0.36 | 0.43 | 0.40 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 2.77 | 2.49 | 2.93 | 2.92 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 5.49 | 5.31 | 5.70 | 5.64 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 3.30 | 3.20 | 3.53 | 3.45 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 7.24 | 7.07 | 7.67 | 7.51 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 12.92 | 12.35 | 13.70 | 13.39 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.39 | 0.37 | 0.44 | 0.43 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.54 | 22.87 | 24.34 | 24.18 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 25.73 | 25.11 | 27.30 | 26.43 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 23.94 | 23.26 | 24.92 | 24.61 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.06 | 31.76 | 34.44 | 34.16 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.94 | 12.57 | 13.35 | 13.32 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.59 | 9.99 | 13.46 | 13.06 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 23.79 | 22.75 | 25.91 | 25.47 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.08 | 25.04 | 27.05 | 26.86 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.51 | 23.57 | 26.07 | 25.73 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.60 | 31.63 | 40.29 | 39.13 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.81 | 13.60 | 14.16 | 14.09 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.43 | 11.92 | 13.16 | 12.88 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 25.97 | 25.40 | 26.60 | 26.45 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.53 | 28.65 | 30.96 | 29.96 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.33 | 25.76 | 27.18 | 27.09 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.34 | 35.22 | 38.20 | 37.09 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 13.09 | 12.75 | 13.46 | 13.27 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.71 | 10.00 | 12.89 | 11.08 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.50 | 25.39 | 29.25 | 27.07 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.23 | 29.19 | 31.28 | 30.98 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.17 | 26.20 | 28.13 | 27.89 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.34 | 34.98 | 37.81 | 37.34 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.99 | 13.77 | 14.20 | 14.16 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.80 | 12.31 | 13.58 | 13.29 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.01 | 24.14 | 25.85 | 25.74 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.31 | 26.47 | 28.12 | 27.82 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.39 | 24.41 | 26.33 | 26.10 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 35.11 | 33.84 | 39.39 | 36.39 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 13.65 | 12.82 | 15.28 | 13.97 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.69 | 9.99 | 11.20 | 11.15 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.34 | 24.26 | 26.82 | 25.93 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 27.87 | 26.91 | 28.58 | 28.41 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 25.62 | 24.97 | 27.23 | 26.24 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.84 | 33.34 | 36.76 | 35.81 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 14.59 | 13.78 | 15.22 | 14.95 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.60 | 12.16 | 13.36 | 13.18 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 27.20 | 25.74 | 29.15 | 28.85 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.06 | 29.77 | 33.07 | 32.77 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.82 | 26.40 | 29.52 | 29.21 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 38.04 | 35.92 | 40.58 | 39.84 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 13.67 | 13.00 | 14.05 | 13.95 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 11.03 | 10.13 | 12.93 | 12.27 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.98 | 26.40 | 30.46 | 29.62 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 32.00 | 30.73 | 33.94 | 33.81 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.36 | 27.41 | 29.95 | 29.87 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 38.03 | 36.05 | 39.84 | 39.74 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 14.68 | 13.94 | 15.12 | 14.95 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.72 | 12.12 | 13.25 | 13.04 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 60 | 25.37 | 24.24 | 27.67 | 26.34 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 60 | 27.44 | 26.39 | 28.59 | 28.46 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 60 | 25.42 | 24.54 | 26.59 | 26.14 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 60 | 34.46 | 32.75 | 36.28 | 35.74 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 60 | 13.03 | 12.59 | 14.20 | 13.73 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 60 | 10.85 | 10.18 | 12.39 | 12.00 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 60 | 25.16 | 24.34 | 26.25 | 25.96 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 60 | 27.83 | 26.76 | 29.22 | 28.80 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 60 | 25.61 | 24.79 | 27.13 | 26.64 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 60 | 34.50 | 33.31 | 36.05 | 35.65 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 60 | 14.01 | 13.54 | 14.83 | 14.67 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 60 | 12.59 | 11.90 | 13.73 | 13.49 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 60 | 47.73 | 40.83 | 49.79 | 49.14 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 60 | 53.31 | 45.93 | 56.25 | 55.20 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 60 | 47.70 | 42.13 | 50.93 | 49.65 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 60 | 67.66 | 60.82 | 68.86 | 68.52 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 60 | 12.91 | 12.52 | 13.47 | 13.39 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 60 | 10.69 | 9.98 | 11.68 | 11.38 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 60 | 47.73 | 42.57 | 51.29 | 49.65 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 60 | 53.57 | 46.92 | 56.29 | 56.14 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 60 | 48.49 | 42.34 | 51.35 | 50.35 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 60 | 67.47 | 61.28 | 69.93 | 69.08 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 60 | 13.53 | 13.19 | 18.10 | 15.53 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 60 | 12.70 | 11.86 | 13.61 | 13.45 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 60 | 2.77 | 2.49 | 2.95 | 2.89 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 60 | 5.50 | 5.29 | 8.73 | 5.66 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 60 | 3.31 | 3.20 | 3.45 | 3.41 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 60 | 7.22 | 7.10 | 8.11 | 7.57 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 60 | 12.04 | 11.56 | 12.71 | 12.58 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 60 | 0.37 | 0.36 | 0.43 | 0.40 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 60 | 2.77 | 2.49 | 2.93 | 2.92 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 60 | 5.49 | 5.31 | 5.70 | 5.64 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 60 | 3.30 | 3.20 | 3.53 | 3.45 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 60 | 7.24 | 7.07 | 7.67 | 7.51 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 60 | 12.92 | 12.35 | 13.70 | 13.39 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 60 | 0.39 | 0.37 | 0.44 | 0.43 | - | - | - | - |
