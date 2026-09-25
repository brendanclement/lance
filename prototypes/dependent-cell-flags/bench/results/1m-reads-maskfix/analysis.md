## Results: 1m reads after the mask fix (26388225a)

Source: `results/1m-reads-maskfix`

### Environment

| key | value |
|---|---|
| scale | `1m` |
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
| config | `{"BENCH_SCALE_ROWS": "1000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "5", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T08:50:44Z	2026-09-25T08:50:49Z	22.17 16.85 9.55
1	prototype	2026-09-25T08:50:49Z	2026-09-25T08:50:56Z	20.55 16.60 9.50
1	prototype-flags	2026-09-25T08:50:56Z	2026-09-25T08:51:05Z	18.26 16.24 9.46
2	baseline	2026-09-25T08:51:05Z	2026-09-25T08:51:10Z	22.97 17.24 9.85
2	prototype	2026-09-25T08:51:10Z	2026-09-25T08:51:15Z	21.21 16.97 9.80
2	prototype-flags	2026-09-25T08:51:15Z	2026-09-25T08:51:22Z	19.75 16.74 9.76
3	baseline	2026-09-25T08:51:22Z	2026-09-25T08:51:27Z	17.24 16.30 9.69
3	prototype	2026-09-25T08:51:27Z	2026-09-25T08:51:32Z	16.10 16.08 9.65
3	prototype-flags	2026-09-25T08:51:32Z	2026-09-25T08:51:38Z	14.89 15.83 9.59
```

Records: 5040. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 2.88 | 2.51 | 4.33 | 3.68 | 60 | 2.97 | 2.58 | 3.51 | 3.29 | 1.019 | 1.019 1.098 1.008 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 3.21 | 2.86 | 7.80 | 4.11 | 60 | 3.32 | 3.03 | 3.60 | 3.55 | 1.036 | 1.041 1.036 1.000 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 2.91 | 2.61 | 3.12 | 3.10 | 60 | 3.13 | 2.79 | 3.36 | 3.30 | 1.073 | 1.033 1.073 1.084 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 3.87 | 3.57 | 4.26 | 4.15 | 60 | 4.19 | 3.69 | 4.82 | 4.69 | 1.078 | 1.050 1.176 1.078 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 1.53 | 1.40 | 1.87 | 1.66 | 60 | 1.61 | 1.48 | 1.95 | 1.86 | 1.057 | 1.057 1.142 0.993 | 2,840 | 2,840 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 5.68 | 4.59 | 6.61 | 6.17 | 60 | 5.74 | 4.79 | 6.48 | 6.30 | 1.029 | 1.081 1.029 0.956 | 2,840 | 2,840 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 3.04 | 2.85 | 3.32 | 3.24 | 60 | 3.17 | 2.99 | 3.47 | 3.36 | 1.049 | 1.049 1.058 1.031 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 3.25 | 2.99 | 3.73 | 3.54 | 60 | 3.52 | 3.10 | 3.88 | 3.80 | 1.070 | 1.070 1.110 1.046 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 3.03 | 2.75 | 3.35 | 3.22 | 60 | 3.28 | 3.08 | 3.74 | 3.53 | 1.074 | 1.074 1.109 1.073 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 3.95 | 3.61 | 4.28 | 4.17 | 60 | 4.37 | 4.03 | 5.04 | 4.81 | 1.110 | 1.110 1.200 1.060 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 1.80 | 1.58 | 1.99 | 1.92 | 60 | 1.89 | 1.73 | 2.19 | 2.08 | 1.031 | 1.031 1.113 1.022 | 2,840 | 2,840 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 6.00 | 5.48 | 6.81 | 6.65 | 60 | 6.08 | 4.86 | 6.72 | 6.58 | 0.992 | 1.062 0.969 0.992 | 2,840 | 2,840 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 3.08 | 2.84 | 3.33 | 3.27 | 60 | 3.28 | 2.94 | 3.80 | 3.60 | 1.098 | 1.041 1.098 1.104 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 3.52 | 3.25 | 3.87 | 3.72 | 60 | 3.83 | 3.25 | 4.33 | 4.23 | 1.082 | 1.027 1.082 1.125 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.17 | 2.89 | 3.49 | 3.35 | 60 | 3.39 | 2.93 | 3.73 | 3.62 | 1.077 | 0.988 1.077 1.099 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 4.13 | 3.67 | 4.46 | 4.36 | 60 | 4.34 | 3.73 | 4.94 | 4.67 | 1.057 | 1.057 1.060 1.042 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 1.54 | 1.43 | 1.71 | 1.67 | 60 | 1.61 | 1.49 | 1.89 | 1.78 | 1.058 | 1.005 1.086 1.058 | 2,840 | 2,840 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 5.97 | 5.05 | 6.72 | 6.41 | 60 | 5.89 | 4.92 | 6.57 | 6.29 | 0.992 | 1.027 0.992 0.938 | 2,840 | 2,840 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 3.28 | 2.97 | 3.66 | 3.54 | 60 | 3.43 | 2.95 | 3.82 | 3.77 | 1.045 | 1.031 1.101 1.045 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.75 | 3.36 | 4.04 | 4.00 | 60 | 3.91 | 3.50 | 4.29 | 4.18 | 1.038 | 1.008 1.064 1.038 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.44 | 3.23 | 3.72 | 3.65 | 60 | 3.56 | 3.22 | 3.94 | 3.75 | 1.025 | 1.005 1.068 1.025 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.37 | 4.00 | 4.75 | 4.63 | 60 | 4.58 | 4.18 | 5.07 | 4.83 | 1.059 | 1.011 1.080 1.059 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.83 | 1.67 | 2.03 | 1.97 | 60 | 1.90 | 1.71 | 2.09 | 2.04 | 1.029 | 1.029 1.076 0.990 | 2,840 | 2,840 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 6.26 | 5.65 | 6.98 | 6.90 | 60 | 6.24 | 5.35 | 7.10 | 6.70 | 0.994 | 0.994 0.996 0.973 | 2,840 | 2,840 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 2.97 | 2.58 | 3.51 | 3.29 | 60 | 2.92 | 2.50 | 3.17 | 3.15 | 0.978 | 0.978 0.963 0.981 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 3.32 | 3.03 | 3.60 | 3.55 | 60 | 3.25 | 2.96 | 3.50 | 3.43 | 0.981 | 1.003 0.981 0.974 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 3.13 | 2.79 | 3.36 | 3.30 | 60 | 3.06 | 2.80 | 3.25 | 3.19 | 0.980 | 1.015 0.963 0.980 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 4.19 | 3.69 | 4.82 | 4.69 | 60 | 4.01 | 3.77 | 4.32 | 4.28 | 0.957 | 0.999 0.879 0.957 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 1.61 | 1.48 | 1.95 | 1.86 | 60 | 1.62 | 1.49 | 1.84 | 1.76 | 1.002 | 1.018 0.963 1.002 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 5.74 | 4.79 | 6.48 | 6.30 | 60 | 5.60 | 4.72 | 6.23 | 5.99 | 0.977 | 0.929 0.977 1.017 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 3.17 | 2.99 | 3.47 | 3.36 | 60 | 3.18 | 2.85 | 3.37 | 3.36 | 0.989 | 1.013 0.989 0.979 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 3.52 | 3.10 | 3.88 | 3.80 | 60 | 3.43 | 3.26 | 3.76 | 3.64 | 0.961 | 1.010 0.959 0.961 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 3.28 | 3.08 | 3.74 | 3.53 | 60 | 3.22 | 2.96 | 3.48 | 3.43 | 0.971 | 1.008 0.971 0.965 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 4.37 | 4.03 | 5.04 | 4.81 | 60 | 4.05 | 3.80 | 4.37 | 4.32 | 0.930 | 0.955 0.888 0.930 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 1.89 | 1.73 | 2.19 | 2.08 | 60 | 1.89 | 1.68 | 2.31 | 2.02 | 1.005 | 1.007 0.957 1.005 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 6.08 | 4.86 | 6.72 | 6.58 | 60 | 5.94 | 5.32 | 6.57 | 6.39 | 0.946 | 0.945 0.946 1.016 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 3.28 | 2.94 | 3.80 | 3.60 | 60 | 5.39 | 4.64 | 5.79 | 5.71 | 1.618 | 1.690 1.578 1.618 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 3.83 | 3.25 | 4.33 | 4.23 | 60 | 5.98 | 5.08 | 6.67 | 6.49 | 1.529 | 1.670 1.529 1.513 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 3.39 | 2.93 | 3.73 | 3.62 | 60 | 5.47 | 4.62 | 6.39 | 6.06 | 1.594 | 1.759 1.580 1.594 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 4.34 | 3.73 | 4.94 | 4.67 | 60 | 7.66 | 6.65 | 8.15 | 8.08 | 1.773 | 1.804 1.736 1.773 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 1.61 | 1.49 | 1.89 | 1.78 | 60 | 1.57 | 1.47 | 1.81 | 1.75 | 0.952 | 1.000 0.952 0.941 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 5.89 | 4.92 | 6.57 | 6.29 | 60 | 5.68 | 4.78 | 6.22 | 6.18 | 0.972 | 0.972 0.977 0.964 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 3.43 | 2.95 | 3.82 | 3.77 | 60 | 5.69 | 5.03 | 6.44 | 6.13 | 1.647 | 1.693 1.584 1.647 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 3.91 | 3.50 | 4.29 | 4.18 | 60 | 6.38 | 5.58 | 6.89 | 6.74 | 1.638 | 1.673 1.606 1.638 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 3.56 | 3.22 | 3.94 | 3.75 | 60 | 5.64 | 4.84 | 6.23 | 6.04 | 1.585 | 1.649 1.544 1.585 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 4.58 | 4.18 | 5.07 | 4.83 | 60 | 7.80 | 6.84 | 8.26 | 8.18 | 1.729 | 1.749 1.606 1.729 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 1.90 | 1.71 | 2.09 | 2.04 | 60 | 1.76 | 1.57 | 1.91 | 1.87 | 0.925 | 0.934 0.917 0.925 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 6.24 | 5.35 | 7.10 | 6.70 | 60 | 6.04 | 5.33 | 6.91 | 6.65 | 0.995 | 0.904 0.995 1.005 | - | - | 2,840 | 86,282 | 822 | 43,315 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.30 | 0.27 | 0.36 | 0.34 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.62 | 0.59 | 0.66 | 0.65 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 60 | 0.33 | 0.32 | 0.37 | 0.36 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 60 | 0.98 | 0.91 | 1.13 | 1.08 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 1.53 | 1.41 | 1.68 | 1.63 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.08 | 0.08 | 0.09 | 0.09 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.29 | 0.28 | 0.32 | 0.31 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.63 | 0.59 | 0.97 | 0.71 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.34 | 0.33 | 0.50 | 0.45 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 1.02 | 0.91 | 1.80 | 1.32 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 1.80 | 1.55 | 2.03 | 2.02 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.09 | 0.08 | 0.14 | 0.12 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 2.88 | 2.51 | 4.33 | 3.68 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 3.21 | 2.86 | 7.80 | 4.11 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 2.91 | 2.61 | 3.12 | 3.10 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 3.87 | 3.57 | 4.26 | 4.15 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 1.53 | 1.40 | 1.87 | 1.66 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 5.68 | 4.59 | 6.61 | 6.17 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 3.04 | 2.85 | 3.32 | 3.24 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 3.25 | 2.99 | 3.73 | 3.54 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 3.03 | 2.75 | 3.35 | 3.22 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 3.95 | 3.61 | 4.28 | 4.17 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 1.80 | 1.58 | 1.99 | 1.92 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 6.00 | 5.48 | 6.81 | 6.65 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 3.08 | 2.84 | 3.33 | 3.27 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 3.52 | 3.25 | 3.87 | 3.72 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.17 | 2.89 | 3.49 | 3.35 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 4.13 | 3.67 | 4.46 | 4.36 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 1.54 | 1.43 | 1.71 | 1.67 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 5.97 | 5.05 | 6.72 | 6.41 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 3.28 | 2.97 | 3.66 | 3.54 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.75 | 3.36 | 4.04 | 4.00 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.44 | 3.23 | 3.72 | 3.65 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.37 | 4.00 | 4.75 | 4.63 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.83 | 1.67 | 2.03 | 1.97 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 6.26 | 5.65 | 6.98 | 6.90 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 2.97 | 2.58 | 3.51 | 3.29 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 3.32 | 3.03 | 3.60 | 3.55 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 3.13 | 2.79 | 3.36 | 3.30 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 4.19 | 3.69 | 4.82 | 4.69 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 1.61 | 1.48 | 1.95 | 1.86 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 5.74 | 4.79 | 6.48 | 6.30 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 3.17 | 2.99 | 3.47 | 3.36 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 3.52 | 3.10 | 3.88 | 3.80 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 3.28 | 3.08 | 3.74 | 3.53 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 4.37 | 4.03 | 5.04 | 4.81 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 1.89 | 1.73 | 2.19 | 2.08 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 6.08 | 4.86 | 6.72 | 6.58 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 3.28 | 2.94 | 3.80 | 3.60 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 3.83 | 3.25 | 4.33 | 4.23 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.39 | 2.93 | 3.73 | 3.62 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 4.34 | 3.73 | 4.94 | 4.67 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 1.61 | 1.49 | 1.89 | 1.78 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 5.89 | 4.92 | 6.57 | 6.29 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 3.43 | 2.95 | 3.82 | 3.77 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.91 | 3.50 | 4.29 | 4.18 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.56 | 3.22 | 3.94 | 3.75 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.58 | 4.18 | 5.07 | 4.83 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.90 | 1.71 | 2.09 | 2.04 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 6.24 | 5.35 | 7.10 | 6.70 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 60 | 2.92 | 2.50 | 3.17 | 3.15 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 60 | 3.25 | 2.96 | 3.50 | 3.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 60 | 3.06 | 2.80 | 3.25 | 3.19 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 60 | 4.01 | 3.77 | 4.32 | 4.28 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 60 | 1.62 | 1.49 | 1.84 | 1.76 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 60 | 5.60 | 4.72 | 6.23 | 5.99 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 60 | 3.18 | 2.85 | 3.37 | 3.36 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 60 | 3.43 | 3.26 | 3.76 | 3.64 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 60 | 3.22 | 2.96 | 3.48 | 3.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 60 | 4.05 | 3.80 | 4.37 | 4.32 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 60 | 1.89 | 1.68 | 2.31 | 2.02 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 60 | 5.94 | 5.32 | 6.57 | 6.39 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 60 | 5.39 | 4.64 | 5.79 | 5.71 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 60 | 5.98 | 5.08 | 6.67 | 6.49 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 60 | 5.47 | 4.62 | 6.39 | 6.06 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 60 | 7.66 | 6.65 | 8.15 | 8.08 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 60 | 1.57 | 1.47 | 1.81 | 1.75 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 60 | 5.68 | 4.78 | 6.22 | 6.18 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 60 | 5.69 | 5.03 | 6.44 | 6.13 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 60 | 6.38 | 5.58 | 6.89 | 6.74 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 60 | 5.64 | 4.84 | 6.23 | 6.04 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 60 | 7.80 | 6.84 | 8.26 | 8.18 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 60 | 1.76 | 1.57 | 1.91 | 1.87 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 60 | 6.04 | 5.33 | 6.91 | 6.65 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 60 | 0.30 | 0.27 | 0.36 | 0.34 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 60 | 0.62 | 0.59 | 0.66 | 0.65 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 60 | 0.33 | 0.32 | 0.37 | 0.36 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 60 | 0.98 | 0.91 | 1.13 | 1.08 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 60 | 1.53 | 1.41 | 1.68 | 1.63 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 60 | 0.08 | 0.08 | 0.09 | 0.09 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 60 | 0.29 | 0.28 | 0.32 | 0.31 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 60 | 0.63 | 0.59 | 0.97 | 0.71 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 60 | 0.34 | 0.33 | 0.50 | 0.45 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 60 | 1.02 | 0.91 | 1.80 | 1.32 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 60 | 1.80 | 1.55 | 2.03 | 2.02 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 60 | 0.09 | 0.08 | 0.14 | 0.12 | - | - | - | - |
