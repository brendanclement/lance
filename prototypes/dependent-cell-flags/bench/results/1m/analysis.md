## Results: 1m

Source: `results/1m`

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
| prototype_sha | `21601f894547b101d8966953c4e158ecd7d97523` |
| rounds | `3` |
| flag_every_round | `True` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "1000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "5", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md; ?? prototypes/dependent-cell-flags/bench/results/smoke-final/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T06:48:12Z	2026-09-25T06:50:00Z	9.19 17.09 12.72
1	prototype	2026-09-25T06:50:00Z	2026-09-25T06:51:48Z	4.41 13.06 11.65
1	prototype-flags	2026-09-25T06:51:48Z	2026-09-25T07:03:03Z	2.73 9.89 10.58
2	baseline	2026-09-25T07:03:03Z	2026-09-25T07:04:51Z	1.86 2.89 5.92
2	prototype	2026-09-25T07:04:51Z	2026-09-25T07:06:39Z	2.25 2.75 5.50
2	prototype-flags	2026-09-25T07:06:39Z	2026-09-25T07:17:54Z	2.03 2.52 5.07
3	baseline	2026-09-25T07:17:54Z	2026-09-25T07:19:43Z	2.26 2.41 3.65
3	prototype	2026-09-25T07:19:43Z	2026-09-25T07:21:31Z	2.44 2.45 3.52
3	prototype-flags	2026-09-25T07:21:31Z	2026-09-25T07:32:46Z	2.36 2.42 3.38
```

Records: 7335. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| append | default | warm | 15 | 4.52 | 4.37 | 5.07 | - | 15 | 4.62 | 4.39 | 5.14 | - | 1.033 | 1.062 1.033 0.987 | 1,495 | 1,495 | 134 | 134 | 1 | 1 | 729,239 | 729,175 | - / - |
| update_sparse | default | warm | 15 | 12.06 | 11.63 | 12.67 | - | 15 | 11.90 | 10.69 | 12.53 | - | 0.966 | 0.943 0.966 1.015 | 2,738 | 2,737 | 1,209 | 1,209 | 256 | 256 | 16,778 | 16,777 | - / - |
| update_dense | default | warm | 15 | 36.23 | 35.71 | 37.52 | - | 15 | 36.90 | 34.84 | 37.60 | - | 1.019 | 0.994 1.021 1.019 | 2,806 | 2,805 | 1,243 | 1,243 | 87 | 87 | 2,276,726 | 2,276,416 | - / - |
| update_unrelated_sparse | default | warm | 15 | 12.10 | 11.04 | 12.89 | - | 15 | 11.74 | 10.56 | 12.64 | - | 0.976 | 0.914 0.976 0.988 | 2,739 | 2,740 | 1,210 | 1,210 | 256 | 256 | 23,584 | 23,585 | - / - |
| merge_insert_partial_sparse | default | warm | 15 | 59.36 | 57.30 | 63.93 | - | 15 | 58.62 | 57.43 | 60.43 | - | 0.992 | 0.944 1.019 0.992 | 4,147 | 4,147 | 1,918 | 1,918 | 31 | 31 | 53,280,807 | 53,283,174 | - / - |
| refresh_permissive_clean | default | warm | 15 | 1263.76 | 1254.43 | 1281.86 | - | 15 | 1258.66 | 1253.86 | 1290.55 | - | 0.994 | 0.990 0.994 1.000 | 2,841 | 2,841 | 822 | 822 | 21 | 21 | 15,040,773 | 15,043,589 | committed / committed |
| refresh_permissive_conflicts_1 † | update_row_moving | warm | 15 | 1266.87 | 1259.55 | 1277.16 | - | 15 | 1263.39 | 1257.40 | 1267.20 | - | 0.998 | 0.994 0.998 1.001 | - | - | - | - | 73 | 73 | 15,056,953 | 15,032,501 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_1 † | merge_insert_in_place | warm | 15 | 1294.73 | 1289.00 | 1380.68 | - | 15 | 1290.52 | 1284.59 | 1294.36 | - | 0.998 | 0.990 0.998 0.999 | 3,316 | 3,316 | 822 | 822 | 43 | 43 | 41,651,811 | 41,620,389 | committed(stale=10) / committed(stale=10) |
| refresh_permissive_conflicts_4 † | update_row_moving | warm | 15 | 1299.41 | 1292.40 | 1342.39 | - | 15 | 1297.17 | 1291.41 | 1299.23 | - | 0.997 | 0.992 1.000 0.997 | - | - | - | - | 260 | 260 | 15,078,730 | 14,975,203 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_4 † | merge_insert_in_place | warm | 15 | 1419.23 | 1413.94 | 1439.88 | - | 15 | 1419.35 | 1413.72 | 1428.13 | - | 1.002 | 0.997 1.002 1.004 | 3,699 | 3,699 | 823 | 823 | 209 | 209 | 190,742,182 | 190,812,330 | committed(stale=40) / committed(stale=40) |
| refresh_permissive_conflicts_16 † | update_row_moving | warm | 15 | 1453.65 | 1444.59 | 1474.21 | - | 15 | 1453.00 | 1446.16 | 1469.24 | - | 1.001 | 1.001 1.000 1.004 | - | - | - | - | 1,111 | 1,111 | 15,249,810 | 15,273,414 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_16 † | merge_insert_in_place | warm | 15 | 2124.14 | 2113.79 | 2131.63 | - | 15 | 2125.19 | 2115.70 | 2148.62 | - | 1.002 | 0.997 1.008 1.002 | 3,795 | 3,795 | 823 | 823 | 920 | 920 | 797,985,641 | 797,945,192 | committed(stale=160) / committed(stale=160) |
| publish_after_k_commits | k=0 | warm | 15 | 0.99 | 0.96 | 1.04 | - | 15 | 0.98 | 0.95 | 1.18 | - | 1.001 | 0.996 1.001 1.023 | 2,845 | 2,845 | 823 | 823 | 1 | 1 | 3,683 | 3,683 | committed / committed |
| publish_after_k_commits | k=1 | warm | 15 | 0.90 | 0.86 | 0.97 | - | 15 | 0.89 | 0.81 | 0.99 | - | 0.978 | 0.961 0.978 1.045 | 2,937 | 2,937 | 823 | 823 | 2 | 2 | 3,775 | 3,775 | committed / committed |
| publish_after_k_commits | k=4 | warm | 15 | 1.00 | 0.92 | 1.08 | - | 15 | 1.03 | 1.00 | 1.10 | - | 1.010 | 1.102 0.983 1.010 | 3,213 | 3,213 | 823 | 823 | 5 | 5 | 4,051 | 4,051 | committed / committed |
| publish_after_k_commits | k=16 | warm | 15 | 1.23 | 1.13 | 1.45 | - | 15 | 1.22 | 1.16 | 1.41 | - | 0.993 | 0.986 0.993 1.054 | 4,317 | 4,317 | 823 | 823 | 17 | 17 | 5,155 | 5,155 | committed / committed |
| publish_after_k_commits | k=64 | warm | 15 | 1.82 | 1.70 | 1.93 | - | 15 | 1.83 | 1.74 | 1.95 | - | 1.016 | 1.016 1.026 0.983 | 8,733 | 8,733 | 823 | 823 | 65 | 65 | 9,571 | 9,571 | committed / committed |
| scan_summary_full | populated | warm | 60 | 2.59 | 2.27 | 2.90 | 2.79 | 60 | 2.58 | 2.37 | 2.88 | 2.82 | 1.025 | 0.968 1.025 1.030 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 2.85 | 2.64 | 3.13 | 3.06 | 60 | 2.88 | 2.65 | 3.20 | 3.05 | 1.009 | 1.009 1.017 0.993 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 2.65 | 2.36 | 2.88 | 2.79 | 60 | 2.69 | 2.43 | 3.00 | 2.87 | 1.022 | 1.022 1.019 1.024 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 3.50 | 3.29 | 4.00 | 3.79 | 60 | 3.58 | 3.32 | 4.00 | 3.90 | 1.022 | 1.024 1.022 1.006 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 1.42 | 1.35 | 1.52 | 1.49 | 60 | 1.43 | 1.34 | 1.58 | 1.52 | 1.011 | 0.983 1.011 1.043 | 2,840 | 2,840 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 5.69 | 4.75 | 6.53 | 6.21 | 60 | 5.66 | 4.48 | 6.69 | 6.42 | 0.991 | 0.991 1.044 0.985 | 2,840 | 2,840 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 2.76 | 2.46 | 3.00 | 2.95 | 60 | 2.83 | 2.54 | 3.15 | 3.04 | 1.034 | 1.034 1.039 1.012 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 2.97 | 2.72 | 3.20 | 3.12 | 60 | 3.02 | 2.81 | 3.45 | 3.23 | 1.027 | 1.027 1.044 1.007 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.83 | 2.63 | 3.11 | 2.97 | 60 | 2.84 | 2.63 | 3.12 | 3.03 | 1.001 | 0.997 1.023 1.001 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 3.74 | 3.38 | 4.40 | 4.08 | 60 | 3.73 | 3.48 | 4.16 | 4.00 | 0.996 | 0.984 0.996 1.006 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 1.72 | 1.52 | 1.87 | 1.83 | 60 | 1.74 | 1.58 | 1.94 | 1.86 | 1.003 | 0.993 1.018 1.003 | 2,840 | 2,840 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 6.00 | 5.33 | 6.68 | 6.41 | 60 | 5.99 | 5.23 | 6.82 | 6.55 | 0.989 | 0.989 1.036 0.984 | 2,840 | 2,840 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 2.99 | 2.73 | 3.31 | 3.20 | 60 | 3.00 | 2.72 | 3.31 | 3.24 | 1.005 | 0.986 1.030 1.005 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 3.35 | 3.04 | 3.72 | 3.56 | 60 | 3.40 | 2.99 | 3.81 | 3.75 | 1.017 | 1.017 1.036 0.991 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.03 | 2.75 | 3.27 | 3.22 | 60 | 3.03 | 2.82 | 3.28 | 3.26 | 1.009 | 1.018 1.009 1.009 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 3.88 | 3.58 | 4.26 | 4.18 | 60 | 3.90 | 3.58 | 4.17 | 4.12 | 1.013 | 0.984 1.027 1.013 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 1.45 | 1.35 | 1.58 | 1.54 | 60 | 1.45 | 1.34 | 1.68 | 1.59 | 0.993 | 0.993 1.024 0.989 | 2,840 | 2,839 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 5.95 | 4.82 | 6.63 | 6.54 | 60 | 5.89 | 5.05 | 6.77 | 6.59 | 0.990 | 0.990 1.033 0.960 | 2,840 | 2,839 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 3.15 | 2.80 | 3.38 | 3.28 | 60 | 3.20 | 2.79 | 3.68 | 3.50 | 0.997 | 0.997 1.060 0.985 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.49 | 3.20 | 3.72 | 3.69 | 60 | 3.48 | 3.26 | 3.96 | 3.75 | 0.996 | 0.996 0.995 1.010 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.19 | 2.96 | 3.43 | 3.38 | 60 | 3.19 | 2.94 | 3.39 | 3.35 | 1.000 | 1.005 1.000 0.987 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.05 | 3.69 | 4.42 | 4.37 | 60 | 4.04 | 3.81 | 4.30 | 4.27 | 0.992 | 1.020 0.961 0.992 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.73 | 1.56 | 1.85 | 1.83 | 60 | 1.74 | 1.55 | 1.89 | 1.85 | 1.020 | 1.027 1.020 0.975 | 2,840 | 2,839 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 6.26 | 5.51 | 6.87 | 6.80 | 60 | 6.30 | 5.59 | 7.08 | 6.78 | 1.016 | 1.044 1.016 0.957 | 2,840 | 2,839 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | warm | update_sparse:default | 15 | 11.90 | 10.69 | 12.53 | - | 15 | 13.06 | 12.33 | 13.94 | - | 1.110 | 1.141 1.085 1.110 | - | - | 2,737 | 5,785 | 1,209 | 2,738 | 16,777 | 24,261 |
| flagged_update_dense | groups=1 | warm | update_dense:default | 15 | 36.90 | 34.84 | 37.60 | - | 15 | 55.59 | 54.72 | 59.15 | - | 1.527 | 1.545 1.527 1.506 | - | - | 2,805 | 336,126 | 1,243 | 169,610 | 2,276,416 | 4,277,613 |
| flagged_update_unrelated_sparse | groups=1 | warm | update_unrelated_sparse:default | 15 | 11.74 | 10.56 | 12.64 | - | 15 | 13.34 | 12.78 | 14.07 | - | 1.158 | 1.211 1.114 1.158 | - | - | 2,740 | 5,794 | 1,210 | 2,738 | 23,585 | 31,074 |
| flagged_merge_insert_partial_sparse | groups=1 | warm | merge_insert_partial_sparse:default | 15 | 58.62 | 57.43 | 60.43 | - | 15 | 60.85 | 59.00 | 62.38 | - | 1.034 | 1.034 1.014 1.049 | 0.98 | 1.06 | 4,147 | 7,433 | 1,918 | 3,691 | 53,283,174 | 53,268,586 |
| flagged_refresh_clean | groups=1 | warm | refresh_permissive_clean:default | 15 | 1258.66 | 1253.86 | 1290.55 | - | 15 | 1268.54 | 1263.43 | 1284.15 | - | 1.008 | 1.009 1.003 1.008 | 1.00 | 1.02 | 2,841 | 3,055 | 822 | 916 | 15,043,589 | 15,071,864 |
| flagged_publish_after_k_commits | k=0 | warm | publish_after_k_commits:k=0 | 15 | 0.98 | 0.95 | 1.18 | - | 15 | 1.06 | 0.97 | 1.24 | - | 1.075 | 1.135 1.075 0.989 | 0.98 | 1.06 | 2,845 | 3,060 | 823 | 917 | 3,683 | 3,993 |
| flagged_publish_after_k_commits | k=1 | warm | publish_after_k_commits:k=1 | 15 | 0.89 | 0.81 | 0.99 | - | 15 | 0.90 | 0.83 | 1.14 | - | 1.018 | 1.018 1.028 0.956 | 0.89 | 0.90 | 2,937 | 3,152 | 823 | 917 | 3,775 | 4,085 |
| flagged_publish_after_k_commits | k=4 | warm | publish_after_k_commits:k=4 | 15 | 1.03 | 1.00 | 1.10 | - | 15 | 1.04 | 0.98 | 1.13 | - | 1.022 | 1.005 1.022 1.029 | 1.03 | 1.04 | 3,213 | 3,428 | 823 | 917 | 4,051 | 4,361 |
| flagged_publish_after_k_commits | k=16 | warm | publish_after_k_commits:k=16 | 15 | 1.22 | 1.16 | 1.41 | - | 15 | 1.23 | 1.16 | 1.33 | - | 1.004 | 1.004 0.965 1.052 | 1.22 | 1.23 | 4,317 | 4,532 | 823 | 917 | 5,155 | 5,465 |
| flagged_publish_after_k_commits | k=64 | warm | publish_after_k_commits:k=64 | 15 | 1.83 | 1.74 | 1.95 | - | 15 | 1.90 | 1.80 | 2.06 | - | 1.030 | 1.030 1.027 1.048 | 1.83 | 1.90 | 8,733 | 8,948 | 823 | 917 | 9,571 | 9,881 |
| flagged_update_sparse | groups=2 | warm | update_sparse:default | 15 | 11.90 | 10.69 | 12.53 | - | 15 | 13.65 | 13.07 | 15.04 | - | 1.151 | 1.220 1.118 1.151 | - | - | 2,737 | 8,026 | 1,209 | 3,478 | 16,777 | 29,701 |
| flagged_update_dense | groups=2 | warm | update_dense:default | 15 | 36.90 | 34.84 | 37.60 | - | 15 | 54.04 | 53.47 | 55.35 | - | 1.484 | 1.507 1.452 1.484 | - | - | 2,805 | 501,795 | 1,243 | 170,348 | 2,276,416 | 5,914,074 |
| flagged_update_unrelated_sparse | groups=2 | warm | update_unrelated_sparse:default | 15 | 11.74 | 10.56 | 12.64 | - | 15 | 14.05 | 13.02 | 14.78 | - | 1.203 | 1.279 1.159 1.203 | - | - | 2,740 | 8,045 | 1,210 | 3,479 | 23,585 | 36,525 |
| flagged_merge_insert_partial_sparse | groups=2 | warm | merge_insert_partial_sparse:default | 15 | 58.62 | 57.43 | 60.43 | - | 15 | 57.56 | 56.51 | 58.43 | - | 0.985 | 0.985 0.957 0.992 | 0.98 | 1.07 | 4,147 | 10,206 | 1,918 | 4,963 | 53,283,174 | 53,291,511 |
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 2.58 | 2.37 | 2.88 | 2.82 | 60 | 2.71 | 2.33 | 2.96 | 2.89 | 1.037 | 1.037 0.998 1.071 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 2.88 | 2.65 | 3.20 | 3.05 | 60 | 2.99 | 2.77 | 3.23 | 3.20 | 1.026 | 1.026 1.013 1.089 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 2.69 | 2.43 | 3.00 | 2.87 | 60 | 2.75 | 2.54 | 2.97 | 2.93 | 1.001 | 1.001 0.997 1.078 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 3.58 | 3.32 | 4.00 | 3.90 | 60 | 3.69 | 3.40 | 4.29 | 4.14 | 1.048 | 1.080 1.005 1.048 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 1.43 | 1.34 | 1.58 | 1.52 | 60 | 1.42 | 1.30 | 1.60 | 1.54 | 1.002 | 1.022 1.002 0.945 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 5.66 | 4.48 | 6.69 | 6.42 | 60 | 5.79 | 4.75 | 6.33 | 6.25 | 1.018 | 1.061 0.938 1.018 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 2.83 | 2.54 | 3.15 | 3.04 | 60 | 2.86 | 2.55 | 3.12 | 3.04 | 1.018 | 1.018 0.961 1.057 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 3.02 | 2.81 | 3.45 | 3.23 | 60 | 3.14 | 2.97 | 3.39 | 3.33 | 1.029 | 1.029 0.989 1.080 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 2.84 | 2.63 | 3.12 | 3.03 | 60 | 2.92 | 2.60 | 3.12 | 3.08 | 1.031 | 1.031 0.988 1.041 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 3.73 | 3.48 | 4.16 | 4.00 | 60 | 3.78 | 3.46 | 4.27 | 4.14 | 1.002 | 1.067 1.000 1.002 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 1.74 | 1.58 | 1.94 | 1.86 | 60 | 1.70 | 1.56 | 1.85 | 1.80 | 0.983 | 1.020 0.983 0.947 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 5.99 | 5.23 | 6.82 | 6.55 | 60 | 6.07 | 5.32 | 6.76 | 6.65 | 1.054 | 1.054 0.916 1.094 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 3.00 | 2.72 | 3.31 | 3.24 | 60 | 9.66 | 8.87 | 11.06 | 10.24 | 3.221 | 3.319 3.107 3.221 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 3.40 | 2.99 | 3.81 | 3.75 | 60 | 10.32 | 9.12 | 13.01 | 11.40 | 3.033 | 2.998 3.033 3.142 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 3.03 | 2.82 | 3.28 | 3.26 | 60 | 9.58 | 8.74 | 10.37 | 10.22 | 3.148 | 3.148 3.101 3.203 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 3.90 | 3.58 | 4.17 | 4.12 | 60 | 12.42 | 11.37 | 12.78 | 12.70 | 3.166 | 3.166 3.078 3.278 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 1.45 | 1.34 | 1.68 | 1.59 | 60 | 1.43 | 1.33 | 1.62 | 1.56 | 1.001 | 1.001 0.953 1.008 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 5.89 | 5.05 | 6.77 | 6.59 | 60 | 5.68 | 5.12 | 6.18 | 6.15 | 0.994 | 0.994 0.892 1.019 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 3.20 | 2.79 | 3.68 | 3.50 | 60 | 9.81 | 8.91 | 10.78 | 10.46 | 3.146 | 3.146 2.908 3.228 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 3.48 | 3.26 | 3.96 | 3.75 | 60 | 10.37 | 9.33 | 11.47 | 10.83 | 3.020 | 3.022 2.851 3.020 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 3.19 | 2.94 | 3.39 | 3.35 | 60 | 9.73 | 8.50 | 10.69 | 10.28 | 3.055 | 3.055 2.938 3.188 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 4.04 | 3.81 | 4.30 | 4.27 | 60 | 12.56 | 11.46 | 12.87 | 12.79 | 3.087 | 2.991 3.087 3.214 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 1.74 | 1.55 | 1.89 | 1.85 | 60 | 1.61 | 1.45 | 1.78 | 1.77 | 0.948 | 0.948 0.887 0.948 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 6.30 | 5.59 | 7.08 | 6.78 | 60 | 6.01 | 5.22 | 6.47 | 6.30 | 0.956 | 0.956 0.907 1.011 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| flagged_update_title_sparse | groups=1 | warm | (no counterpart) | - | - | - | - | - | 15 | 13.58 | 12.98 | 15.57 | - | - | - | - | - | - | 5,786 | - | 2,738 | - | 28,998 |
| flagged_update_title_sparse | groups=2 | warm | (no counterpart) | - | - | - | - | - | 15 | 14.09 | 12.75 | 15.13 | - | - | - | - | - | - | 8,033 | - | 3,477 | - | 34,443 |
| flagged_refresh_clean | groups=2 | warm | (no counterpart) | - | - | - | - | - | 15 | 2335.45 | 2332.44 | 2353.98 | - | - | - | - | 2.05 | - | 3,905 | - | 916 | - | 29,664,320 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.29 | 0.27 | 0.34 | 0.33 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.60 | 0.56 | 0.65 | 0.63 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 60 | 0.33 | 0.31 | 0.37 | 0.35 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 60 | 0.98 | 0.93 | 1.15 | 1.09 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 1.43 | 1.34 | 1.65 | 1.53 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 60 | 0.08 | 0.07 | 0.09 | 0.09 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.29 | 0.27 | 0.34 | 0.31 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.60 | 0.55 | 0.66 | 0.63 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.32 | 0.32 | 0.35 | 0.35 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.97 | 0.91 | 1.11 | 1.05 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 1.68 | 1.53 | 1.90 | 1.79 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 60 | 0.08 | 0.08 | 0.14 | 0.12 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |

Flag effects of the source writes (asserted exact in every sample; first sample shown):

| workload | variant | rows written | flags cleared | flags carried | derived invalidation rows | moved rows |
|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | 100 | {"summary": 100} | {"summary": 0} | {"summary": 0} | 100 |
| flagged_update_dense | groups=1 | 100,000 | {"summary": 100000} | {"summary": 0} | {"summary": 0} | 100,000 |
| flagged_update_unrelated_sparse | groups=1 | 100 | {"summary": 0} | {"summary": 100} | {"summary": 0} | 100 |
| flagged_update_title_sparse | groups=1 | 100 | {"summary": 100} | {"summary": 0} | {"summary": 0} | 100 |
| flagged_merge_insert_partial_sparse | groups=1 | 100 | {"summary": 100} | null | {"summary": 100} | - |
| flagged_update_sparse | groups=2 | 100 | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | {"summary": 0, "translation": 0} | 100 |
| flagged_update_dense | groups=2 | 100,000 | {"summary": 100000, "translation": 100000} | {"summary": 0, "translation": 0} | {"summary": 0, "translation": 0} | 100,000 |
| flagged_update_unrelated_sparse | groups=2 | 100 | {"summary": 0, "translation": 0} | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | 100 |
| flagged_update_title_sparse | groups=2 | 100 | {"summary": 100, "translation": 0} | {"summary": 0, "translation": 100} | {"summary": 0, "translation": 0} | 100 |
| flagged_merge_insert_partial_sparse | groups=2 | 100 | {"summary": 100, "translation": 100} | null | {"summary": 100, "translation": 100} | - |

### Refresh under concurrent source writes

A full `summary` refresh staged at V, K source commits of 10 scattered rows each, then the publication at V. `commit ms` is the conflict-checked publication commit. `merge_insert_output_in_place` writes `summary` itself (an output override); it is the only source write here that defers whole groups.

| workload | variant | n | outcome | rows assigned | rows published | rows deferred | deferred by reason | fragments deferred | fragment reasons | valid rows in deferred groups | commit med ms | commit p95 | wall med ms | udf ms | source commits ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_refresh_conflicts_1 | update_row_moving:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.14 | - | 1273.52 | 1176.71 | 6.14 |
| flagged_refresh_conflicts_1 | update_row_moving:skip | 15 | committed_partial | 1,000,000 | 999,990 | 10 | {"RowVacated": 10} | 0 | {} | 0 | 1.19 | - | 1274.09 | 1176.97 | 6.05 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.14 | - | 1302.85 | 1179.23 | 33.41 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:skip | 15 | committed_partial | 1,000,000 | 999,990 | 10 | {"InputChanged": 10} | 0 | {} | 0 | 0.97 | - | 1299.58 | 1175.20 | 33.00 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.15 | - | 1277.78 | 1179.38 | 9.29 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | 15 | committed_partial | 1,000,000 | 500,000 | 10 | {"InputChanged": 10} | 5 | {"OutputWritten": 5} | 499,990 | 0.95 | - | 1278.24 | 1176.18 | 9.20 |
| flagged_refresh_conflicts_4 | update_row_moving:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.22 | - | 1305.04 | 1177.55 | 39.90 |
| flagged_refresh_conflicts_4 | update_row_moving:skip | 15 | committed_partial | 1,000,000 | 999,960 | 40 | {"RowVacated": 40} | 0 | {} | 0 | 1.40 | - | 1306.04 | 1175.79 | 39.55 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.22 | - | 1432.72 | 1179.69 | 164.30 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:skip | 15 | committed_partial | 1,000,000 | 999,960 | 40 | {"InputChanged": 40} | 0 | {} | 0 | 1.09 | - | 1431.53 | 1178.47 | 162.78 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.21 | - | 1309.13 | 1180.05 | 40.81 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | 15 | committed_partial | 1,000,000 | 100,000 | 40 | {"InputChanged": 40} | 9 | {"OutputWritten": 9} | 899,960 | 1.07 | - | 1307.12 | 1177.17 | 40.04 |
| flagged_refresh_conflicts_16 | update_row_moving:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.43 | - | 1462.58 | 1177.37 | 195.76 |
| flagged_refresh_conflicts_16 | update_row_moving:skip | 15 | committed_partial | 1,000,000 | 999,840 | 160 | {"RowVacated": 160} | 0 | {} | 0 | 1.80 | - | 1463.37 | 1177.89 | 194.82 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.46 | - | 2130.17 | 1177.79 | 864.42 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:skip | 15 | committed_partial | 1,000,000 | 999,840 | 160 | {"InputChanged": 160} | 0 | {} | 0 | 1.39 | - | 2126.04 | 1178.40 | 860.22 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | 15 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.38 | - | 1453.62 | 1178.92 | 184.87 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | 15 | deferred_all | 1,000,000 | 0 | 160 | {"InputChanged": 160} | 10 | {"OutputWritten": 10} | 999,840 | 0.51 | - | 1451.94 | 1178.22 | 184.27 |

Permissive publication without flags (regression harness), **NOT correctness-equivalent**: it has no dependency tracking, so in-place writes publish stale summaries (`stale`) and row-moving writes are rejected by main's rules:

| build | workload | variant | n | outcome | rows published | stale rows | commit med ms | wall med ms |
|---|---|---|---|---|---|---|---|---|
| baseline | refresh_permissive_conflicts_1 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.14 | 1266.87 |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.14 | 1263.39 |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | 15 | committed(stale=10) | 1,000,000 | 10 | 0.90 | 1294.73 |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | 15 | committed(stale=10) | 1,000,000 | 10 | 0.93 | 1290.52 |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.21 | 1299.41 |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.21 | 1297.17 |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | 15 | committed(stale=40) | 1,000,000 | 40 | 1.00 | 1419.23 |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | 15 | committed(stale=40) | 1,000,000 | 40 | 1.02 | 1419.35 |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.42 | 1453.65 |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.40 | 1453.00 |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | 15 | committed(stale=160) | 1,000,000 | 160 | 1.27 | 2124.14 |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | 15 | committed(stale=160) | 1,000,000 | 160 | 1.25 | 2125.19 |

### Follow-up refresh to completion: saved and repeated computation

From the state the conflicted publication left, each strategy refreshes every pending row and publishes; afterwards every flag is asserted true and every value equal to the UDF of its current inputs. `recompute_all_pending` ignores the report; `reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject returns an error, so it has no report). `total UDF rows` = rows the conflicted publication computed + rows the follow-up recomputed; with N rows, anything above N is repeated computation.

| workload | variant | n | rows computed first | rows published first | rows recomputed | rows reused | rows copied through | total UDF rows | udf med ms | stage med ms | commit med ms | wall med ms | written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.47 | 62.72 | 1.02 | 1276.49 | 15,255,684 |
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.31 | 62.83 | 1.03 | 1278.13 | 15,224,582 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | 15 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 0.36 | 0.98 | 16.41 | 3,485 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | 15 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 0.38 | 0.95 | 16.63 | 3,486 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1177.24 | 58.31 | 1.03 | 1268.16 | 15,111,577 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1178.87 | 58.16 | 1.03 | 1268.95 | 15,113,113 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.18 | 29.14 | 0.99 | 42.30 | 7,482,471 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 28.53 | 1.01 | 42.08 | 7,480,871 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.75 | 57.61 | 1.02 | 1265.88 | 14,945,876 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.64 | 57.52 | 1.04 | 1264.89 | 14,938,580 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 500,000 | 500,000 | 0 | 0 | 1,500,000 | 589.52 | 28.70 | 1.04 | 634.78 | 7,469,284 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 500,000 | 10 | 499,990 | 0 | 1,000,010 | 0.18 | 28.58 | 0.99 | 43.58 | 7,565,475 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.26 | 67.41 | 1.04 | 1284.49 | 15,488,125 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.38 | 67.19 | 1.10 | 1284.58 | 15,391,738 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | 15 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.33 | 1.02 | 0.95 | 29.68 | 6,675 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | 15 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.33 | 1.04 | 0.94 | 29.90 | 6,675 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.91 | 58.38 | 1.03 | 1267.43 | 15,084,885 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.62 | 58.02 | 1.07 | 1266.09 | 15,045,844 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.38 | 52.13 | 1.01 | 74.74 | 13,614,427 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.35 | 51.51 | 1.01 | 74.55 | 13,515,227 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.66 | 57.62 | 1.03 | 1265.60 | 14,907,532 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1178.50 | 57.78 | 1.04 | 1268.24 | 14,951,308 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 100,000 | 900,000 | 0 | 0 | 1,900,000 | 1060.51 | 51.70 | 1.04 | 1140.39 | 13,428,187 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 100,000 | 40 | 899,960 | 0 | 1,000,040 | 0.34 | 51.25 | 1.01 | 76.33 | 13,499,804 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.84 | 71.24 | 1.15 | 1289.72 | 15,425,063 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1177.25 | 71.42 | 1.18 | 1291.46 | 15,364,967 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | 15 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.53 | 3.81 | 0.99 | 41.54 | 18,821 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | 15 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.52 | 3.70 | 1.02 | 41.86 | 18,821 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1178.27 | 57.88 | 1.04 | 1268.21 | 15,059,892 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.82 | 58.02 | 1.05 | 1266.96 | 14,961,460 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.63 | 57.11 | 1.03 | 83.32 | 15,041,373 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.57 | 57.73 | 1.04 | 85.04 | 15,131,293 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.32 | 58.24 | 1.04 | 1267.16 | 15,057,837 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.62 | 58.50 | 1.06 | 1269.11 | 15,043,757 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | 15 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.81 | 58.07 | 1.04 | 1266.75 | 15,032,813 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 15 | 1,000,000 | 0 | 160 | 999,840 | 0 | 1,000,160 | 0.57 | 57.17 | 1.04 | 84.70 | 15,039,213 |

### Publication commit latency after K unrelated commits

`wall = commit_ms` of the publication only. Conflict checks read every transaction since the read version.

| k | base n | base med ms | base p95 | proto n | proto med ms | proto p95 | flags n | flags med ms | flags p95 | flags / proto (med) |
|---|---|---|---|---|---|---|---|---|---|---|
| k=0 | 15 | 0.99 | - | 15 | 0.98 | - | 15 | 1.06 | - | 1.075 |
| k=1 | 15 | 0.90 | - | 15 | 0.89 | - | 15 | 0.90 | - | 1.018 |
| k=4 | 15 | 1.00 | - | 15 | 1.03 | - | 15 | 1.04 | - | 1.022 |
| k=16 | 15 | 1.23 | - | 15 | 1.22 | - | 15 | 1.23 | - | 1.004 |
| k=64 | 15 | 1.82 | - | 15 | 1.83 | - | 15 | 1.90 | - | 1.030 |

### Flag state size as the true set fragments

Head after one in-place `body` write invalidating the given fraction of scattered rows. `groups=0` is the unflagged control with the same data files. `wall` is a fresh-session open (OS page cache not controlled).

| variant | n | manifest B | txn B | flag state B (serialized true sets) | open med ms | open min | open max | open p95 | open r_iops | open read B |
|---|---|---|---|---|---|---|---|---|---|---|
| groups=0:invalidated=0pct | 60 | 2,802 | - | 0 | 0.08 | 0.07 | 0.14 | 0.12 | 1 | 2,802 |
| groups=0:invalidated=0.1pct | 60 | 7,106 | 3,398 | 0 | 0.08 | 0.07 | 0.12 | 0.11 | 1 | 4,096 |
| groups=0:invalidated=1pct | 60 | 7,106 | 3,398 | 0 | 0.09 | 0.07 | 0.12 | 0.11 | 1 | 4,096 |
| groups=0:invalidated=10pct | 60 | 7,107 | 3,398 | 0 | 0.09 | 0.07 | 0.12 | 0.11 | 1 | 4,096 |
| groups=1:invalidated=0pct | 60 | 2,922 | - | 84 | 0.08 | 0.07 | 0.14 | 0.12 | 1 | 2,922 |
| groups=1:invalidated=0.1pct | 60 | 16,123 | 8,051 | 4,326 | 0.11 | 0.09 | 0.17 | 0.15 | 3 | 12,148 |
| groups=1:invalidated=1pct | 60 | 87,746 | 44,055 | 39,942 | 0.13 | 0.11 | 0.18 | 0.16 | 3 | 47,767 |
| groups=1:invalidated=10pct | 60 | 473,434 | 305,519 | 164,164 | 0.13 | 0.11 | 0.18 | 0.17 | 3 | 171,991 |
| groups=2:invalidated=0pct | 60 | 3,033 | - | 168 | 0.08 | 0.07 | 0.14 | 0.11 | 1 | 3,033 |
| groups=2:invalidated=0.1pct | 60 | 22,808 | 10,383 | 8,652 | 0.12 | 0.10 | 0.15 | 0.15 | 3 | 16,501 |
| groups=2:invalidated=1pct | 60 | 148,052 | 64,389 | 79,884 | 0.14 | 0.12 | 0.27 | 0.18 | 3 | 87,739 |
| groups=2:invalidated=10pct | 60 | 788,692 | 456,585 | 328,328 | 0.15 | 0.14 | 0.22 | 0.19 | 3 | 336,183 |

### Every group (wall_ms)

One row per (build, workload, variant, cache) in the input.

| build | workload | variant | cache | n | med ms | min | max | p95 | udf med ms | stage med ms | commit med ms | outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | append | default | warm | 15 | 4.52 | 4.37 | 5.07 | - | - | 3.77 | 0.77 | - |
| baseline | update_sparse | default | warm | 15 | 12.06 | 11.63 | 12.67 | - | - | - | - | - |
| baseline | update_dense | default | warm | 15 | 36.23 | 35.71 | 37.52 | - | - | - | - | - |
| baseline | update_unrelated_sparse | default | warm | 15 | 12.10 | 11.04 | 12.89 | - | - | - | - | - |
| baseline | merge_insert_partial_sparse | default | warm | 15 | 59.36 | 57.30 | 63.93 | - | - | 58.37 | 1.01 | - |
| baseline | refresh_permissive_clean | default | warm | 15 | 1263.76 | 1254.43 | 1281.86 | - | 1187.01 | 58.90 | 1.02 | committed |
| baseline | refresh_permissive_conflicts_1 | update_row_moving | warm | 15 | 1266.87 | 1259.55 | 1277.16 | - | 1184.62 | 59.20 | 0.14 | conflict:retryable |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 15 | 1294.73 | 1289.00 | 1380.68 | - | 1185.67 | 59.33 | 0.90 | committed(stale=10) |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | warm | 15 | 1299.41 | 1292.40 | 1342.39 | - | 1185.97 | 58.78 | 0.21 | conflict:retryable |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 15 | 1419.23 | 1413.94 | 1439.88 | - | 1183.90 | 58.56 | 1.00 | committed(stale=40) |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | warm | 15 | 1453.65 | 1444.59 | 1474.21 | - | 1185.30 | 58.44 | 0.42 | conflict:retryable |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 15 | 2124.14 | 2113.79 | 2131.63 | - | 1183.25 | 58.26 | 1.27 | committed(stale=160) |
| baseline | publish_after_k_commits | k=0 | warm | 15 | 0.99 | 0.96 | 1.04 | - | - | - | 0.99 | committed |
| baseline | publish_after_k_commits | k=1 | warm | 15 | 0.90 | 0.86 | 0.97 | - | - | - | 0.90 | committed |
| baseline | publish_after_k_commits | k=4 | warm | 15 | 1.00 | 0.92 | 1.08 | - | - | - | 1.00 | committed |
| baseline | publish_after_k_commits | k=16 | warm | 15 | 1.23 | 1.13 | 1.45 | - | - | - | 1.23 | committed |
| baseline | publish_after_k_commits | k=64 | warm | 15 | 1.82 | 1.70 | 1.93 | - | - | - | 1.82 | committed |
| baseline | scan_summary_full | populated | warm | 60 | 2.59 | 2.27 | 2.90 | 2.79 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 2.85 | 2.64 | 3.13 | 3.06 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 2.65 | 2.36 | 2.88 | 2.79 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 3.50 | 3.29 | 4.00 | 3.79 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 1.42 | 1.35 | 1.52 | 1.49 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 5.69 | 4.75 | 6.53 | 6.21 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 2.76 | 2.46 | 3.00 | 2.95 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 2.97 | 2.72 | 3.20 | 3.12 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.83 | 2.63 | 3.11 | 2.97 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 3.74 | 3.38 | 4.40 | 4.08 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 1.72 | 1.52 | 1.87 | 1.83 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 6.00 | 5.33 | 6.68 | 6.41 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 2.99 | 2.73 | 3.31 | 3.20 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 3.35 | 3.04 | 3.72 | 3.56 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.03 | 2.75 | 3.27 | 3.22 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 3.88 | 3.58 | 4.26 | 4.18 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 1.45 | 1.35 | 1.58 | 1.54 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 5.95 | 4.82 | 6.63 | 6.54 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 3.15 | 2.80 | 3.38 | 3.28 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.49 | 3.20 | 3.72 | 3.69 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.19 | 2.96 | 3.43 | 3.38 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.05 | 3.69 | 4.42 | 4.37 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.73 | 1.56 | 1.85 | 1.83 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 6.26 | 5.51 | 6.87 | 6.80 | - | - | - | - |
| prototype | append | default | warm | 15 | 4.62 | 4.39 | 5.14 | - | - | 3.78 | 0.81 | - |
| prototype | update_sparse | default | warm | 15 | 11.90 | 10.69 | 12.53 | - | - | - | - | - |
| prototype | update_dense | default | warm | 15 | 36.90 | 34.84 | 37.60 | - | - | - | - | - |
| prototype | update_unrelated_sparse | default | warm | 15 | 11.74 | 10.56 | 12.64 | - | - | - | - | - |
| prototype | merge_insert_partial_sparse | default | warm | 15 | 58.62 | 57.43 | 60.43 | - | - | 57.60 | 0.98 | - |
| prototype | refresh_permissive_clean | default | warm | 15 | 1258.66 | 1253.86 | 1290.55 | - | 1185.10 | 58.48 | 1.00 | committed |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | warm | 15 | 1263.39 | 1257.40 | 1267.20 | - | 1183.36 | 58.16 | 0.14 | conflict:retryable |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 15 | 1290.52 | 1284.59 | 1294.36 | - | 1183.75 | 58.02 | 0.93 | committed(stale=10) |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | warm | 15 | 1297.17 | 1291.41 | 1299.23 | - | 1182.76 | 57.85 | 0.21 | conflict:retryable |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 15 | 1419.35 | 1413.72 | 1428.13 | - | 1182.96 | 58.54 | 1.02 | committed(stale=40) |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | warm | 15 | 1453.00 | 1446.16 | 1469.24 | - | 1185.01 | 58.45 | 0.40 | conflict:retryable |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 15 | 2125.19 | 2115.70 | 2148.62 | - | 1183.20 | 58.19 | 1.25 | committed(stale=160) |
| prototype | publish_after_k_commits | k=0 | warm | 15 | 0.98 | 0.95 | 1.18 | - | - | - | 0.98 | committed |
| prototype | publish_after_k_commits | k=1 | warm | 15 | 0.89 | 0.81 | 0.99 | - | - | - | 0.89 | committed |
| prototype | publish_after_k_commits | k=4 | warm | 15 | 1.03 | 1.00 | 1.10 | - | - | - | 1.03 | committed |
| prototype | publish_after_k_commits | k=16 | warm | 15 | 1.22 | 1.16 | 1.41 | - | - | - | 1.22 | committed |
| prototype | publish_after_k_commits | k=64 | warm | 15 | 1.83 | 1.74 | 1.95 | - | - | - | 1.83 | committed |
| prototype | scan_summary_full | populated | warm | 60 | 2.58 | 2.37 | 2.88 | 2.82 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 2.88 | 2.65 | 3.20 | 3.05 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 2.69 | 2.43 | 3.00 | 2.87 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 3.58 | 3.32 | 4.00 | 3.90 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 1.43 | 1.34 | 1.58 | 1.52 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 5.66 | 4.48 | 6.69 | 6.42 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 2.83 | 2.54 | 3.15 | 3.04 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 3.02 | 2.81 | 3.45 | 3.23 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.84 | 2.63 | 3.12 | 3.03 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 3.73 | 3.48 | 4.16 | 4.00 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 1.74 | 1.58 | 1.94 | 1.86 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 5.99 | 5.23 | 6.82 | 6.55 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 3.00 | 2.72 | 3.31 | 3.24 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 3.40 | 2.99 | 3.81 | 3.75 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.03 | 2.82 | 3.28 | 3.26 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 3.90 | 3.58 | 4.17 | 4.12 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 1.45 | 1.34 | 1.68 | 1.59 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 5.89 | 5.05 | 6.77 | 6.59 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 3.20 | 2.79 | 3.68 | 3.50 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.48 | 3.26 | 3.96 | 3.75 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.19 | 2.94 | 3.39 | 3.35 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.04 | 3.81 | 4.30 | 4.27 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.74 | 1.55 | 1.89 | 1.85 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 6.30 | 5.59 | 7.08 | 6.78 | - | - | - | - |
| prototype-flags | flagged_update_sparse | groups=1 | warm | 15 | 13.06 | 12.33 | 13.94 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=1 | warm | 15 | 55.59 | 54.72 | 59.15 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=1 | warm | 15 | 13.34 | 12.78 | 14.07 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=1 | warm | 15 | 13.58 | 12.98 | 15.57 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=1 | warm | 15 | 60.85 | 59.00 | 62.38 | - | - | 59.76 | 1.06 | - |
| prototype-flags | flagged_refresh_clean | groups=1 | warm | 15 | 1268.54 | 1263.43 | 1284.15 | - | 1178.35 | 58.51 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:reject | warm | 15 | 1273.52 | 1268.88 | 1277.63 | - | 1176.71 | 58.27 | 0.14 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | warm | 15 | 1276.49 | 1271.16 | 1279.72 | - | 1179.47 | 62.72 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | warm | 15 | 1278.13 | 1272.45 | 1282.17 | - | 1179.31 | 62.83 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:skip | warm | 15 | 1274.09 | 1267.96 | 1293.35 | - | 1176.97 | 58.28 | 1.19 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | warm | 15 | 16.41 | 15.74 | 22.51 | - | 0.17 | 0.36 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | warm | 15 | 16.63 | 16.19 | 16.99 | - | 0.17 | 0.38 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:reject | warm | 15 | 1302.85 | 1293.62 | 1313.35 | - | 1179.23 | 58.48 | 0.14 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 15 | 1268.16 | 1263.48 | 1289.99 | - | 1177.24 | 58.31 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 15 | 1268.95 | 1257.43 | 1272.08 | - | 1178.87 | 58.16 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:skip | warm | 15 | 1299.58 | 1294.38 | 1323.82 | - | 1175.20 | 58.32 | 0.97 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 15 | 42.30 | 41.12 | 47.71 | - | 0.18 | 29.14 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 15 | 42.08 | 40.88 | 43.57 | - | 0.17 | 28.53 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | warm | 15 | 1277.78 | 1270.39 | 1281.29 | - | 1179.38 | 58.20 | 0.15 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 15 | 1265.88 | 1261.48 | 1269.36 | - | 1176.75 | 57.61 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 15 | 1264.89 | 1261.62 | 1270.50 | - | 1175.64 | 57.52 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | warm | 15 | 1278.24 | 1271.74 | 1280.63 | - | 1176.18 | 58.36 | 0.95 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 15 | 634.78 | 629.99 | 645.08 | - | 589.52 | 28.70 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 15 | 43.58 | 42.04 | 44.62 | - | 0.18 | 28.58 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:reject | warm | 15 | 1305.04 | 1301.16 | 1311.53 | - | 1177.55 | 57.97 | 0.22 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | warm | 15 | 1284.49 | 1279.98 | 1293.49 | - | 1179.26 | 67.41 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | warm | 15 | 1284.58 | 1278.79 | 1303.94 | - | 1179.38 | 67.19 | 1.10 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:skip | warm | 15 | 1306.04 | 1299.79 | 1346.05 | - | 1175.79 | 58.04 | 1.40 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | warm | 15 | 29.68 | 29.00 | 30.48 | - | 0.33 | 1.02 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | warm | 15 | 29.90 | 29.42 | 31.22 | - | 0.33 | 1.04 | 0.94 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:reject | warm | 15 | 1432.72 | 1424.97 | 1436.76 | - | 1179.69 | 58.32 | 0.22 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 15 | 1267.43 | 1262.19 | 1273.96 | - | 1176.91 | 58.38 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 15 | 1266.09 | 1258.26 | 1272.32 | - | 1176.62 | 58.02 | 1.07 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:skip | warm | 15 | 1431.53 | 1423.61 | 1449.88 | - | 1178.47 | 57.98 | 1.09 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 15 | 74.74 | 73.39 | 76.62 | - | 0.38 | 52.13 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 15 | 74.55 | 73.17 | 80.11 | - | 0.35 | 51.51 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | warm | 15 | 1309.13 | 1304.88 | 1327.63 | - | 1180.05 | 58.15 | 0.21 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 15 | 1265.60 | 1258.47 | 1274.21 | - | 1176.66 | 57.62 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 15 | 1268.24 | 1261.84 | 1329.71 | - | 1178.50 | 57.78 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | warm | 15 | 1307.12 | 1304.52 | 1324.74 | - | 1177.17 | 57.72 | 1.07 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 15 | 1140.39 | 1135.32 | 1144.72 | - | 1060.51 | 51.70 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 15 | 76.33 | 74.44 | 78.08 | - | 0.34 | 51.25 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:reject | warm | 15 | 1462.58 | 1454.10 | 1465.86 | - | 1177.37 | 58.41 | 0.43 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | warm | 15 | 1289.72 | 1286.15 | 1299.34 | - | 1176.84 | 71.24 | 1.15 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | warm | 15 | 1291.46 | 1285.70 | 1314.52 | - | 1177.25 | 71.42 | 1.18 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:skip | warm | 15 | 1463.37 | 1457.62 | 1487.23 | - | 1177.89 | 58.53 | 1.80 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | warm | 15 | 41.54 | 40.54 | 42.85 | - | 0.53 | 3.81 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | warm | 15 | 41.86 | 40.88 | 43.53 | - | 0.52 | 3.70 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:reject | warm | 15 | 2130.17 | 2120.26 | 2158.65 | - | 1177.79 | 58.02 | 0.46 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 15 | 1268.21 | 1261.78 | 1270.00 | - | 1178.27 | 57.88 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 15 | 1266.96 | 1262.44 | 1295.21 | - | 1176.82 | 58.02 | 1.05 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:skip | warm | 15 | 2126.04 | 2120.16 | 2215.31 | - | 1178.40 | 58.50 | 1.39 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 15 | 83.32 | 81.08 | 85.73 | - | 0.63 | 57.11 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 15 | 85.04 | 82.68 | 86.69 | - | 0.57 | 57.73 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | warm | 15 | 1453.62 | 1449.58 | 1494.49 | - | 1178.92 | 58.70 | 0.38 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 15 | 1267.16 | 1261.04 | 1292.24 | - | 1176.32 | 58.24 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 15 | 1269.11 | 1262.78 | 1283.93 | - | 1176.62 | 58.50 | 1.06 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | warm | 15 | 1451.94 | 1444.09 | 1457.53 | - | 1178.22 | 58.37 | 0.51 | deferred_all |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 15 | 1266.75 | 1261.57 | 1272.21 | - | 1176.81 | 58.07 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 15 | 84.70 | 82.54 | 86.57 | - | 0.57 | 57.17 | 1.04 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=0 | warm | 15 | 1.06 | 0.97 | 1.24 | - | - | - | 1.06 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=1 | warm | 15 | 0.90 | 0.83 | 1.14 | - | - | - | 0.90 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=4 | warm | 15 | 1.04 | 0.98 | 1.13 | - | - | - | 1.04 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=16 | warm | 15 | 1.23 | 1.16 | 1.33 | - | - | - | 1.23 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=64 | warm | 15 | 1.90 | 1.80 | 2.06 | - | - | - | 1.90 | committed |
| prototype-flags | flagged_update_sparse | groups=2 | warm | 15 | 13.65 | 13.07 | 15.04 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=2 | warm | 15 | 54.04 | 53.47 | 55.35 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=2 | warm | 15 | 14.05 | 13.02 | 14.78 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=2 | warm | 15 | 14.09 | 12.75 | 15.13 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=2 | warm | 15 | 57.56 | 56.51 | 58.43 | - | - | 56.51 | 1.07 | - |
| prototype-flags | flagged_refresh_clean | groups=2 | warm | 15 | 2335.45 | 2332.44 | 2353.98 | - | 2159.09 | 116.72 | 2.05 | committed |
| prototype-flags | scan_summary_full | all_true | warm | 60 | 2.71 | 2.33 | 2.96 | 2.89 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 60 | 2.99 | 2.77 | 3.23 | 3.20 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 60 | 2.75 | 2.54 | 2.97 | 2.93 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 60 | 3.69 | 3.40 | 4.29 | 4.14 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 60 | 1.42 | 1.30 | 1.60 | 1.54 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 60 | 5.79 | 4.75 | 6.33 | 6.25 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 60 | 2.86 | 2.55 | 3.12 | 3.04 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 60 | 3.14 | 2.97 | 3.39 | 3.33 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 60 | 2.92 | 2.60 | 3.12 | 3.08 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 60 | 3.78 | 3.46 | 4.27 | 4.14 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 60 | 1.70 | 1.56 | 1.85 | 1.80 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 60 | 6.07 | 5.32 | 6.76 | 6.65 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 60 | 9.66 | 8.87 | 11.06 | 10.24 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 60 | 10.32 | 9.12 | 13.01 | 11.40 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 60 | 9.58 | 8.74 | 10.37 | 10.22 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 60 | 12.42 | 11.37 | 12.78 | 12.70 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 60 | 1.43 | 1.33 | 1.62 | 1.56 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 60 | 5.68 | 5.12 | 6.18 | 6.15 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 60 | 9.81 | 8.91 | 10.78 | 10.46 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 60 | 10.37 | 9.33 | 11.47 | 10.83 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 60 | 9.73 | 8.50 | 10.69 | 10.28 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 60 | 12.56 | 11.46 | 12.87 | 12.79 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 60 | 1.61 | 1.45 | 1.78 | 1.77 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 60 | 6.01 | 5.22 | 6.47 | 6.30 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 60 | 0.29 | 0.27 | 0.34 | 0.33 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 60 | 0.60 | 0.56 | 0.65 | 0.63 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 60 | 0.33 | 0.31 | 0.37 | 0.35 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 60 | 0.98 | 0.93 | 1.15 | 1.09 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 60 | 1.43 | 1.34 | 1.65 | 1.53 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 60 | 0.08 | 0.07 | 0.09 | 0.09 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 60 | 0.29 | 0.27 | 0.34 | 0.31 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 60 | 0.60 | 0.55 | 0.66 | 0.63 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 60 | 0.32 | 0.32 | 0.35 | 0.35 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 60 | 0.97 | 0.91 | 1.11 | 1.05 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 60 | 1.68 | 1.53 | 1.90 | 1.79 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 60 | 0.08 | 0.08 | 0.14 | 0.12 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0pct | fresh-session | 60 | 0.08 | 0.07 | 0.14 | 0.12 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0.1pct | fresh-session | 60 | 0.08 | 0.07 | 0.12 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=1pct | fresh-session | 60 | 0.09 | 0.07 | 0.12 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=10pct | fresh-session | 60 | 0.09 | 0.07 | 0.12 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0pct | fresh-session | 60 | 0.08 | 0.07 | 0.14 | 0.12 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0.1pct | fresh-session | 60 | 0.11 | 0.09 | 0.17 | 0.15 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=1pct | fresh-session | 60 | 0.13 | 0.11 | 0.18 | 0.16 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=10pct | fresh-session | 60 | 0.13 | 0.11 | 0.18 | 0.17 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0pct | fresh-session | 60 | 0.08 | 0.07 | 0.14 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0.1pct | fresh-session | 60 | 0.12 | 0.10 | 0.15 | 0.15 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=1pct | fresh-session | 60 | 0.14 | 0.12 | 0.27 | 0.18 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=10pct | fresh-session | 60 | 0.15 | 0.14 | 0.22 | 0.19 | - | - | - | - |
