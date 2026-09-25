## Clean build: 1m full matrix at 26388225a built outside the nested checkout

Source: `results/1m-clean`

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
| prototype_sha | `26388225a273052b3a1ff0ec705c53da1b68c7cb` |
| rounds | `3` |
| flag_every_round | `False` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "1000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "5", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M prototypes/dependent-cell-flags/bench/run_paired.sh; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-clean-baseline-first/; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-clean-prototype-first/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T10:17:47Z	2026-09-25T10:19:36Z	4.72 5.61 7.99
1	prototype	2026-09-25T10:19:36Z	2026-09-25T10:21:25Z	4.06 5.05 7.48
2	baseline	2026-09-25T10:21:25Z	2026-09-25T10:23:13Z	3.70 4.69 7.05
2	prototype	2026-09-25T10:23:13Z	2026-09-25T10:25:02Z	3.84 4.39 6.65
3	baseline	2026-09-25T10:25:02Z	2026-09-25T10:26:50Z	3.56 4.14 6.27
3	prototype	2026-09-25T10:26:50Z	2026-09-25T10:28:38Z	3.80 3.98 5.96
3	prototype-flags	2026-09-25T10:28:38Z	2026-09-25T10:39:52Z	3.04 3.61 5.56
```

Records: 4705. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| append | default | warm | 15 | 4.70 | 4.57 | 4.83 | - | 15 | 4.70 | 4.59 | 4.87 | - | 1.010 | 1.011 1.010 0.991 | 1,495 | 1,495 | 134 | 134 | 1 | 1 | 729,239 | 729,749 | - / - |
| update_sparse | default | warm | 15 | 12.10 | 11.30 | 12.83 | - | 15 | 12.10 | 11.54 | 12.90 | - | 0.997 | 0.997 0.973 1.021 | 2,737 | 2,738 | 1,209 | 1,209 | 256 | 256 | 16,777 | 16,778 | - / - |
| update_dense | default | warm | 15 | 36.34 | 35.74 | 37.78 | - | 15 | 38.17 | 37.44 | 39.47 | - | 1.040 | 1.076 1.040 1.027 | 2,806 | 2,804 | 1,243 | 1,242 | 87 | 87 | 2,276,672 | 2,276,605 | - / - |
| update_unrelated_sparse | default | warm | 15 | 12.20 | 11.72 | 13.09 | - | 15 | 12.28 | 11.69 | 17.40 | - | 0.994 | 1.024 0.994 0.979 | 2,737 | 2,738 | 1,209 | 1,209 | 256 | 256 | 23,581 | 23,582 | - / - |
| merge_insert_partial_sparse | default | warm | 15 | 58.71 | 57.49 | 61.45 | - | 15 | 59.87 | 58.36 | 61.27 | - | 0.999 | 1.039 0.999 0.997 | 4,147 | 4,147 | 1,918 | 1,918 | 31 | 31 | 53,253,799 | 53,269,543 | - / - |
| refresh_permissive_clean | default | warm | 15 | 1261.56 | 1259.31 | 1294.85 | - | 15 | 1262.90 | 1257.33 | 1283.99 | - | 1.000 | 0.986 1.000 1.004 | 2,841 | 2,841 | 822 | 822 | 21 | 21 | 15,035,717 | 15,055,812 | committed / committed |
| refresh_permissive_conflicts_1 † | update_row_moving | warm | 15 | 1267.47 | 1264.90 | 1291.11 | - | 15 | 1266.52 | 1262.78 | 1300.87 | - | 0.999 | 0.990 0.999 1.003 | - | - | - | - | 73 | 73 | 15,078,326 | 15,045,872 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_1 † | merge_insert_in_place | warm | 15 | 1293.52 | 1288.42 | 1300.43 | - | 15 | 1292.87 | 1290.42 | 1298.36 | - | 0.999 | 0.999 0.999 1.000 | 3,316 | 3,316 | 822 | 822 | 43 | 43 | 41,680,229 | 41,674,339 | committed(stale=10) / committed(stale=10) |
| refresh_permissive_conflicts_4 † | update_row_moving | warm | 15 | 1300.21 | 1296.94 | 1306.07 | - | 15 | 1299.39 | 1293.70 | 1324.32 | - | 0.998 | 0.997 0.998 0.998 | - | - | - | - | 260 | 260 | 15,080,602 | 15,109,167 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_4 † | merge_insert_in_place | warm | 15 | 1424.11 | 1420.06 | 1508.78 | - | 15 | 1423.73 | 1415.82 | 1450.32 | - | 1.000 | 1.001 0.998 1.000 | 3,699 | 3,699 | 823 | 823 | 209 | 209 | 190,801,515 | 190,751,787 | committed(stale=40) / committed(stale=40) |
| refresh_permissive_conflicts_16 † | update_row_moving | warm | 15 | 1458.62 | 1449.43 | 1505.97 | - | 15 | 1455.25 | 1451.93 | 1468.45 | - | 0.997 | 0.998 0.997 0.997 | - | - | - | - | 1,111 | 1,111 | 15,240,087 | 15,239,389 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_16 † | merge_insert_in_place | warm | 15 | 2137.09 | 2129.81 | 2176.11 | - | 15 | 2132.48 | 2117.30 | 2156.04 | - | 0.996 | 0.994 0.998 0.996 | 3,795 | 3,795 | 823 | 823 | 920 | 920 | 797,950,121 | 798,053,608 | committed(stale=160) / committed(stale=160) |
| publish_after_k_commits | k=0 | warm | 15 | 1.01 | 0.94 | 1.03 | - | 15 | 1.00 | 0.98 | 1.07 | - | 1.006 | 0.996 1.010 1.006 | 2,845 | 2,845 | 823 | 823 | 1 | 1 | 3,683 | 3,683 | committed / committed |
| publish_after_k_commits | k=1 | warm | 15 | 0.90 | 0.82 | 1.02 | - | 15 | 0.91 | 0.83 | 0.99 | - | 1.012 | 1.013 0.977 1.012 | 2,937 | 2,937 | 823 | 823 | 2 | 2 | 3,775 | 3,775 | committed / committed |
| publish_after_k_commits | k=4 | warm | 15 | 1.00 | 0.96 | 1.21 | - | 15 | 1.02 | 0.97 | 1.18 | - | 0.995 | 0.930 0.995 1.087 | 3,213 | 3,213 | 823 | 823 | 5 | 5 | 4,051 | 4,051 | committed / committed |
| publish_after_k_commits | k=16 | warm | 15 | 1.17 | 1.08 | 1.33 | - | 15 | 1.22 | 1.12 | 1.32 | - | 1.023 | 0.980 1.023 1.050 | 4,317 | 4,317 | 823 | 823 | 17 | 17 | 5,155 | 5,155 | committed / committed |
| publish_after_k_commits | k=64 | warm | 15 | 1.77 | 1.70 | 2.02 | - | 15 | 1.82 | 1.76 | 3.39 | - | 1.033 | 1.036 1.033 1.023 | 8,733 | 8,733 | 823 | 823 | 65 | 65 | 9,571 | 9,571 | committed / committed |
| scan_summary_full | populated | warm | 60 | 2.71 | 2.38 | 2.96 | 2.86 | 60 | 2.66 | 2.30 | 3.00 | 2.90 | 0.984 | 0.945 0.984 1.044 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 2.97 | 2.75 | 3.15 | 3.12 | 60 | 2.93 | 2.69 | 3.18 | 3.08 | 0.984 | 0.962 1.004 0.984 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 2.77 | 2.58 | 3.04 | 2.92 | 60 | 2.77 | 2.50 | 2.93 | 2.91 | 1.004 | 0.967 1.050 1.004 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 3.66 | 3.33 | 4.04 | 3.85 | 60 | 3.71 | 3.39 | 4.43 | 3.97 | 1.019 | 0.991 1.019 1.045 | 2,840 | 2,840 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 1.47 | 1.36 | 1.59 | 1.55 | 60 | 1.43 | 1.31 | 1.62 | 1.54 | 0.973 | 0.960 0.973 0.998 | 2,840 | 2,840 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 5.71 | 4.85 | 6.27 | 6.15 | 60 | 5.76 | 4.87 | 6.28 | 6.08 | 1.034 | 0.979 1.034 1.038 | 2,840 | 2,840 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 2.90 | 2.68 | 3.12 | 3.07 | 60 | 2.86 | 2.59 | 3.14 | 3.04 | 0.987 | 0.967 1.031 0.987 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 3.17 | 2.92 | 3.50 | 3.36 | 60 | 3.16 | 2.96 | 3.46 | 3.37 | 1.001 | 0.982 1.031 1.001 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.97 | 2.62 | 3.18 | 3.14 | 60 | 2.94 | 2.68 | 3.43 | 3.18 | 1.025 | 0.944 1.025 1.032 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 3.85 | 3.54 | 4.28 | 4.21 | 60 | 3.85 | 3.65 | 4.26 | 4.10 | 1.017 | 0.952 1.023 1.017 | 2,840 | 2,840 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 1.79 | 1.55 | 3.88 | 2.88 | 60 | 1.70 | 1.53 | 1.85 | 1.80 | 0.960 | 0.649 0.960 1.007 | 2,840 | 2,840 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 5.97 | 5.45 | 6.53 | 6.45 | 60 | 5.92 | 5.33 | 6.76 | 6.51 | 1.014 | 0.959 1.014 1.052 | 2,840 | 2,840 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 3.07 | 2.79 | 3.34 | 3.25 | 60 | 2.99 | 2.62 | 3.21 | 3.20 | 0.958 | 0.937 1.027 0.958 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 3.42 | 3.08 | 3.74 | 3.65 | 60 | 3.40 | 3.07 | 3.71 | 3.64 | 0.998 | 0.955 1.010 0.998 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.11 | 2.80 | 3.42 | 3.30 | 60 | 3.09 | 2.92 | 3.36 | 3.27 | 0.998 | 0.968 0.998 1.031 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 4.01 | 3.53 | 4.39 | 4.25 | 60 | 3.97 | 3.65 | 4.44 | 4.33 | 0.994 | 0.990 0.996 0.994 | 2,840 | 2,839 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 1.46 | 1.35 | 1.67 | 1.57 | 60 | 1.43 | 1.36 | 1.59 | 1.52 | 0.979 | 0.972 0.979 0.998 | 2,840 | 2,839 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 5.79 | 5.02 | 6.34 | 6.29 | 60 | 5.91 | 4.66 | 6.56 | 6.43 | 1.018 | 0.993 1.022 1.018 | 2,840 | 2,839 | 822 | 822 | 513 | 513 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 3.17 | 2.75 | 3.57 | 3.43 | 60 | 3.16 | 2.75 | 3.45 | 3.38 | 1.016 | 0.949 1.018 1.016 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.60 | 3.22 | 3.82 | 3.75 | 60 | 3.56 | 3.26 | 3.81 | 3.73 | 0.993 | 0.993 0.979 0.997 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.27 | 3.00 | 3.58 | 3.45 | 60 | 3.25 | 2.98 | 3.44 | 3.39 | 1.006 | 0.964 1.006 1.008 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.18 | 3.77 | 4.58 | 4.46 | 60 | 4.16 | 3.93 | 4.51 | 4.45 | 1.000 | 0.980 1.000 1.029 | 2,840 | 2,839 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.73 | 1.52 | 1.92 | 1.86 | 60 | 1.72 | 1.55 | 1.93 | 1.81 | 0.999 | 0.999 0.986 1.006 | 2,840 | 2,839 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 6.13 | 5.58 | 6.71 | 6.45 | 60 | 6.04 | 5.50 | 6.66 | 6.49 | 0.994 | 0.991 0.998 0.994 | 2,840 | 2,839 | 822 | 822 | 533 | 533 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | warm | update_sparse:default | 15 | 12.10 | 11.54 | 12.90 | - | 5 | 12.34 | 11.90 | 12.53 | - | 1.019 | 1.019 | - | - | 2,738 | 5,787 | 1,209 | 2,739 | 16,778 | 24,264 |
| flagged_update_dense | groups=1 | warm | update_dense:default | 15 | 38.17 | 37.44 | 39.47 | - | 5 | 54.01 | 53.39 | 54.27 | - | 1.407 | 1.407 | - | - | 2,804 | 336,125 | 1,242 | 169,609 | 2,276,605 | 4,319,335 |
| flagged_update_unrelated_sparse | groups=1 | warm | update_unrelated_sparse:default | 15 | 12.28 | 11.69 | 17.40 | - | 5 | 11.88 | 11.68 | 12.41 | - | 0.982 | 0.982 | - | - | 2,738 | 5,795 | 1,209 | 2,739 | 23,582 | 31,076 |
| flagged_merge_insert_partial_sparse | groups=1 | warm | merge_insert_partial_sparse:default | 15 | 59.87 | 58.36 | 61.27 | - | 5 | 59.32 | 58.76 | 60.55 | - | 0.979 | 0.979 | 1.02 | 1.03 | 4,147 | 7,433 | 1,918 | 3,691 | 53,269,543 | 53,264,234 |
| flagged_refresh_clean | groups=1 | warm | refresh_permissive_clean:default | 15 | 1262.90 | 1257.33 | 1283.99 | - | 5 | 1266.66 | 1259.80 | 1268.97 | - | 1.001 | 1.001 | 1.03 | 1.03 | 2,841 | 3,055 | 822 | 916 | 15,055,812 | 14,946,169 |
| flagged_publish_after_k_commits | k=0 | warm | publish_after_k_commits:k=0 | 15 | 1.00 | 0.98 | 1.07 | - | 5 | 1.00 | 0.97 | 1.03 | - | 1.001 | 1.001 | 1.00 | 1.00 | 2,845 | 3,060 | 823 | 917 | 3,683 | 3,993 |
| flagged_publish_after_k_commits | k=1 | warm | publish_after_k_commits:k=1 | 15 | 0.91 | 0.83 | 0.99 | - | 5 | 0.95 | 0.88 | 1.19 | - | 1.043 | 1.043 | 0.91 | 0.95 | 2,937 | 3,152 | 823 | 917 | 3,775 | 4,085 |
| flagged_publish_after_k_commits | k=4 | warm | publish_after_k_commits:k=4 | 15 | 1.02 | 0.97 | 1.18 | - | 5 | 1.06 | 0.99 | 1.16 | - | 0.992 | 0.992 | 1.02 | 1.06 | 3,213 | 3,428 | 823 | 917 | 4,051 | 4,361 |
| flagged_publish_after_k_commits | k=16 | warm | publish_after_k_commits:k=16 | 15 | 1.22 | 1.12 | 1.32 | - | 5 | 1.23 | 1.19 | 1.28 | - | 0.997 | 0.997 | 1.22 | 1.23 | 4,317 | 4,532 | 823 | 917 | 5,155 | 5,465 |
| flagged_publish_after_k_commits | k=64 | warm | publish_after_k_commits:k=64 | 15 | 1.82 | 1.76 | 3.39 | - | 5 | 1.88 | 1.84 | 1.94 | - | 1.039 | 1.039 | 1.82 | 1.88 | 8,733 | 8,948 | 823 | 917 | 9,571 | 9,881 |
| flagged_update_sparse | groups=2 | warm | update_sparse:default | 15 | 12.10 | 11.54 | 12.90 | - | 5 | 13.06 | 12.72 | 13.14 | - | 1.079 | 1.079 | - | - | 2,738 | 8,027 | 1,209 | 3,478 | 16,778 | 29,702 |
| flagged_update_dense | groups=2 | warm | update_dense:default | 15 | 38.17 | 37.44 | 39.47 | - | 5 | 53.38 | 53.13 | 54.02 | - | 1.390 | 1.390 | - | - | 2,804 | 501,791 | 1,242 | 170,346 | 2,276,605 | 5,914,388 |
| flagged_update_unrelated_sparse | groups=2 | warm | update_unrelated_sparse:default | 15 | 12.28 | 11.69 | 17.40 | - | 5 | 13.41 | 12.80 | 13.96 | - | 1.109 | 1.109 | - | - | 2,738 | 8,044 | 1,209 | 3,479 | 23,582 | 36,524 |
| flagged_merge_insert_partial_sparse | groups=2 | warm | merge_insert_partial_sparse:default | 15 | 59.87 | 58.36 | 61.27 | - | 5 | 56.65 | 56.09 | 57.15 | - | 0.935 | 0.935 | 1.02 | 1.08 | 4,147 | 10,205 | 1,918 | 4,963 | 53,269,543 | 53,305,078 |
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 2.66 | 2.30 | 3.00 | 2.90 | 20 | 2.56 | 2.29 | 2.71 | 2.71 | 0.923 | 0.923 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 2.93 | 2.69 | 3.18 | 3.08 | 20 | 2.92 | 2.71 | 3.09 | 3.01 | 0.975 | 0.975 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 2.77 | 2.50 | 2.93 | 2.91 | 20 | 2.62 | 2.52 | 2.72 | 2.69 | 0.936 | 0.936 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 3.71 | 3.39 | 4.43 | 3.97 | 20 | 3.54 | 3.33 | 3.91 | 3.88 | 0.943 | 0.943 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 1.43 | 1.31 | 1.62 | 1.54 | 20 | 1.43 | 1.32 | 1.55 | 1.51 | 0.982 | 0.982 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 5.76 | 4.87 | 6.28 | 6.08 | 20 | 5.67 | 4.60 | 6.09 | 6.02 | 0.989 | 0.989 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 2.86 | 2.59 | 3.14 | 3.04 | 20 | 2.76 | 2.58 | 2.92 | 2.90 | 0.956 | 0.956 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 3.16 | 2.96 | 3.46 | 3.37 | 20 | 2.92 | 2.74 | 3.15 | 3.14 | 0.914 | 0.914 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 2.94 | 2.68 | 3.43 | 3.18 | 20 | 2.71 | 2.59 | 2.87 | 2.83 | 0.882 | 0.882 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 3.85 | 3.65 | 4.26 | 4.10 | 20 | 3.59 | 3.35 | 3.83 | 3.79 | 0.925 | 0.925 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 1.70 | 1.53 | 1.85 | 1.80 | 20 | 1.72 | 1.60 | 1.83 | 1.79 | 1.010 | 1.010 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 5.92 | 5.33 | 6.76 | 6.51 | 20 | 5.92 | 5.34 | 6.45 | 6.37 | 0.962 | 0.962 | - | - | 2,840 | 3,054 | 822 | 916 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 2.99 | 2.62 | 3.21 | 3.20 | 20 | 5.29 | 4.58 | 5.62 | 5.51 | 1.792 | 1.792 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 3.40 | 3.07 | 3.71 | 3.64 | 20 | 5.83 | 5.15 | 6.28 | 6.17 | 1.722 | 1.722 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 3.09 | 2.92 | 3.36 | 3.27 | 20 | 5.26 | 4.54 | 5.67 | 5.65 | 1.678 | 1.678 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 3.97 | 3.65 | 4.44 | 4.33 | 20 | 7.53 | 6.68 | 7.69 | 7.67 | 1.892 | 1.892 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 1.43 | 1.36 | 1.59 | 1.52 | 20 | 1.43 | 1.37 | 1.57 | 1.52 | 0.991 | 0.991 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 5.91 | 4.66 | 6.56 | 6.43 | 20 | 5.62 | 4.79 | 6.14 | 6.00 | 0.970 | 0.970 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 3.16 | 2.75 | 3.45 | 3.38 | 20 | 5.30 | 4.52 | 5.57 | 5.55 | 1.668 | 1.668 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 3.56 | 3.26 | 3.81 | 3.73 | 20 | 5.96 | 5.20 | 6.47 | 6.38 | 1.661 | 1.661 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 3.25 | 2.98 | 3.44 | 3.39 | 20 | 5.44 | 4.71 | 5.85 | 5.67 | 1.666 | 1.666 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 4.16 | 3.93 | 4.51 | 4.45 | 20 | 7.59 | 6.81 | 7.89 | 7.81 | 1.820 | 1.820 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 1.72 | 1.55 | 1.93 | 1.81 | 20 | 1.61 | 1.53 | 1.77 | 1.67 | 0.939 | 0.939 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 6.04 | 5.50 | 6.66 | 6.49 | 20 | 6.04 | 5.28 | 6.59 | 6.41 | 1.017 | 1.017 | - | - | 2,839 | 86,283 | 822 | 43,315 | 0 | 0 |
| flagged_update_title_sparse | groups=1 | warm | (no counterpart) | - | - | - | - | - | 5 | 11.99 | 11.66 | 12.77 | - | - | - | - | - | - | 5,786 | - | 2,738 | - | 28,998 |
| flagged_update_title_sparse | groups=2 | warm | (no counterpart) | - | - | - | - | - | 5 | 12.69 | 12.60 | 13.29 | - | - | - | - | - | - | 8,035 | - | 3,478 | - | 34,446 |
| flagged_refresh_clean | groups=2 | warm | (no counterpart) | - | - | - | - | - | 5 | 2334.56 | 2330.32 | 2338.39 | - | - | - | - | 2.00 | - | 3,905 | - | 916 | - | 29,810,367 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.29 | 0.29 | 0.32 | 0.31 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.61 | 0.59 | 0.65 | 0.64 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 0.33 | 0.32 | 0.34 | 0.34 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 0.97 | 0.91 | 1.04 | 1.03 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 1.43 | 1.35 | 1.53 | 1.51 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.08 | 0.07 | 0.09 | 0.08 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.29 | 0.28 | 0.30 | 0.30 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.61 | 0.59 | 0.64 | 0.62 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.33 | 0.33 | 0.35 | 0.34 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.96 | 0.91 | 1.02 | 1.01 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 1.65 | 1.52 | 1.82 | 1.77 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.09 | 0.07 | 0.12 | 0.11 | - | - | - | - | - | 1,360 | - | 62 | - | 0 |

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
| flagged_refresh_conflicts_1 | update_row_moving:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.14 | - | 1276.42 | 1179.75 | 5.94 |
| flagged_refresh_conflicts_1 | update_row_moving:skip | 5 | committed_partial | 1,000,000 | 999,990 | 10 | {"RowVacated": 10} | 0 | {} | 0 | 1.21 | - | 1276.09 | 1179.03 | 5.85 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.15 | - | 1299.70 | 1178.56 | 31.79 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:skip | 5 | committed_partial | 1,000,000 | 999,990 | 10 | {"InputChanged": 10} | 0 | {} | 0 | 1.00 | - | 1298.99 | 1177.14 | 31.53 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.14 | - | 1270.98 | 1173.74 | 8.77 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | 5 | committed_partial | 1,000,000 | 500,000 | 10 | {"InputChanged": 10} | 5 | {"OutputWritten": 5} | 499,990 | 1.01 | - | 1274.23 | 1176.11 | 8.78 |
| flagged_refresh_conflicts_4 | update_row_moving:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.21 | - | 1307.04 | 1178.93 | 39.82 |
| flagged_refresh_conflicts_4 | update_row_moving:skip | 5 | committed_partial | 1,000,000 | 999,960 | 40 | {"RowVacated": 40} | 0 | {} | 0 | 1.44 | - | 1309.19 | 1177.50 | 39.67 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.23 | - | 1425.60 | 1175.01 | 162.10 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:skip | 5 | committed_partial | 1,000,000 | 999,960 | 40 | {"InputChanged": 40} | 0 | {} | 0 | 1.13 | - | 1428.19 | 1178.98 | 159.75 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.21 | - | 1306.44 | 1177.85 | 39.76 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | 5 | committed_partial | 1,000,000 | 100,000 | 40 | {"InputChanged": 40} | 9 | {"OutputWritten": 9} | 899,960 | 0.95 | - | 1307.97 | 1178.86 | 39.95 |
| flagged_refresh_conflicts_16 | update_row_moving:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.43 | - | 1464.00 | 1177.08 | 197.73 |
| flagged_refresh_conflicts_16 | update_row_moving:skip | 5 | committed_partial | 1,000,000 | 999,840 | 160 | {"RowVacated": 160} | 0 | {} | 0 | 1.81 | - | 1464.55 | 1177.54 | 195.33 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.47 | - | 2130.52 | 1178.78 | 862.85 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:skip | 5 | committed_partial | 1,000,000 | 999,840 | 160 | {"InputChanged": 160} | 0 | {} | 0 | 1.47 | - | 2125.72 | 1176.92 | 861.93 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | 5 | conflict:retryable | 1,000,000 | 0 | - | {} | - | {} | - | 0.38 | - | 1456.20 | 1179.73 | 186.37 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | 5 | deferred_all | 1,000,000 | 0 | 160 | {"InputChanged": 160} | 10 | {"OutputWritten": 10} | 999,840 | 0.47 | - | 1447.70 | 1175.92 | 184.47 |

Permissive publication without flags (regression harness), **NOT correctness-equivalent**: it has no dependency tracking, so in-place writes publish stale summaries (`stale`) and row-moving writes are rejected by main's rules:

| build | workload | variant | n | outcome | rows published | stale rows | commit med ms | wall med ms |
|---|---|---|---|---|---|---|---|---|
| baseline | refresh_permissive_conflicts_1 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.13 | 1267.47 |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.13 | 1266.52 |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | 15 | committed(stale=10) | 1,000,000 | 10 | 0.94 | 1293.52 |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | 15 | committed(stale=10) | 1,000,000 | 10 | 0.93 | 1292.87 |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.21 | 1300.21 |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.21 | 1299.39 |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | 15 | committed(stale=40) | 1,000,000 | 40 | 1.03 | 1424.11 |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | 15 | committed(stale=40) | 1,000,000 | 40 | 1.03 | 1423.73 |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.41 | 1458.62 |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | 15 | conflict:retryable | 0 | 0 | 0.42 | 1455.25 |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | 15 | committed(stale=160) | 1,000,000 | 160 | 1.26 | 2137.09 |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | 15 | committed(stale=160) | 1,000,000 | 160 | 1.28 | 2132.48 |

### Follow-up refresh to completion: saved and repeated computation

From the state the conflicted publication left, each strategy refreshes every pending row and publishes; afterwards every flag is asserted true and every value equal to the UDF of its current inputs. `recompute_all_pending` ignores the report; `reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject returns an error, so it has no report). `total UDF rows` = rows the conflicted publication computed + rows the follow-up recomputed; with N rows, anything above N is repeated computation.

| workload | variant | n | rows computed first | rows published first | rows recomputed | rows reused | rows copied through | total UDF rows | udf med ms | stage med ms | commit med ms | wall med ms | written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1174.04 | 62.92 | 1.04 | 1271.75 | 15,239,813 |
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.73 | 63.12 | 1.02 | 1273.92 | 15,342,727 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | 5 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 0.36 | 0.95 | 16.05 | 3,484 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | 5 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 0.37 | 1.05 | 16.49 | 3,485 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.08 | 58.76 | 1.00 | 1265.70 | 15,137,880 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1174.42 | 57.39 | 1.06 | 1263.21 | 14,834,393 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.19 | 29.02 | 0.99 | 42.03 | 7,571,303 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 999,990 | 10 | 0 | 499,990 | 1,000,010 | 0.17 | 29.52 | 0.98 | 42.86 | 7,583,271 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.47 | 58.16 | 1.03 | 1268.87 | 15,027,540 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.91 | 57.59 | 1.03 | 1266.80 | 14,918,804 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 500,000 | 500,000 | 0 | 0 | 1,500,000 | 586.27 | 28.90 | 1.01 | 631.78 | 7,570,532 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 500,000 | 10 | 499,990 | 0 | 1,000,010 | 0.18 | 28.56 | 0.98 | 43.24 | 7,576,100 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.78 | 67.14 | 1.03 | 1280.43 | 15,405,755 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.18 | 67.42 | 1.09 | 1284.80 | 15,402,045 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | 5 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.33 | 1.02 | 0.94 | 29.86 | 6,677 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | 5 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.33 | 1.05 | 0.92 | 30.32 | 6,677 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1179.01 | 58.00 | 1.02 | 1268.58 | 14,957,589 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1177.45 | 57.95 | 1.05 | 1266.86 | 15,046,677 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.39 | 51.77 | 1.02 | 73.86 | 13,611,675 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 999,960 | 40 | 0 | 899,960 | 1,000,040 | 0.35 | 51.16 | 1.04 | 74.57 | 13,582,619 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1174.98 | 57.01 | 1.01 | 1263.44 | 14,863,756 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1174.23 | 57.83 | 1.02 | 1262.02 | 15,029,388 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 100,000 | 900,000 | 0 | 0 | 1,900,000 | 1059.20 | 52.93 | 1.04 | 1140.48 | 13,722,588 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 100,000 | 40 | 899,960 | 0 | 1,000,040 | 0.34 | 52.01 | 1.06 | 76.69 | 13,519,516 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.00 | 71.79 | 1.17 | 1290.85 | 15,554,165 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1176.28 | 71.81 | 1.18 | 1291.28 | 15,457,569 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | 5 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.53 | 3.77 | 1.05 | 42.16 | 18,821 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | 5 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.52 | 3.63 | 1.07 | 42.08 | 18,822 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.98 | 57.30 | 0.98 | 1264.45 | 14,937,525 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1178.34 | 58.17 | 1.06 | 1269.33 | 15,046,772 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.64 | 57.09 | 1.04 | 84.24 | 14,953,053 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 999,840 | 160 | 0 | 999,840 | 1,000,160 | 0.58 | 57.09 | 1.04 | 84.16 | 14,941,085 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1175.51 | 57.28 | 1.06 | 1263.54 | 14,946,092 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1177.26 | 57.82 | 1.03 | 1265.81 | 15,027,629 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | 5 | 1,000,000 | 0 | 1,000,000 | 0 | 0 | 2,000,000 | 1180.04 | 58.59 | 1.01 | 1267.50 | 15,037,676 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 5 | 1,000,000 | 0 | 160 | 999,840 | 0 | 1,000,160 | 0.55 | 56.66 | 1.00 | 83.85 | 15,146,349 |

### Publication commit latency after K unrelated commits

`wall = commit_ms` of the publication only. Conflict checks read every transaction since the read version.

| k | base n | base med ms | base p95 | proto n | proto med ms | proto p95 | flags n | flags med ms | flags p95 | flags / proto (med) |
|---|---|---|---|---|---|---|---|---|---|---|
| k=0 | 15 | 1.01 | - | 15 | 1.00 | - | 5 | 1.00 | - | 1.001 |
| k=1 | 15 | 0.90 | - | 15 | 0.91 | - | 5 | 0.95 | - | 1.043 |
| k=4 | 15 | 1.00 | - | 15 | 1.02 | - | 5 | 1.06 | - | 0.992 |
| k=16 | 15 | 1.17 | - | 15 | 1.22 | - | 5 | 1.23 | - | 0.997 |
| k=64 | 15 | 1.77 | - | 15 | 1.82 | - | 5 | 1.88 | - | 1.039 |

### Flag state size as the true set fragments

Head after one in-place `body` write invalidating the given fraction of scattered rows. `groups=0` is the unflagged control with the same data files. `wall` is a fresh-session open (OS page cache not controlled).

| variant | n | manifest B | txn B | flag state B (serialized true sets) | open med ms | open min | open max | open p95 | open r_iops | open read B |
|---|---|---|---|---|---|---|---|---|---|---|
| groups=0:invalidated=0pct | 20 | 2,803 | - | 0 | 0.08 | 0.07 | 0.13 | 0.12 | 1 | 2,803 |
| groups=0:invalidated=0.1pct | 20 | 7,106 | 3,398 | 0 | 0.08 | 0.07 | 0.11 | 0.11 | 1 | 4,096 |
| groups=0:invalidated=1pct | 20 | 7,106 | 3,398 | 0 | 0.08 | 0.07 | 0.12 | 0.12 | 1 | 4,096 |
| groups=0:invalidated=10pct | 20 | 7,107 | 3,398 | 0 | 0.08 | 0.07 | 0.11 | 0.10 | 1 | 4,096 |
| groups=1:invalidated=0pct | 20 | 2,923 | - | 84 | 0.08 | 0.08 | 0.13 | 0.13 | 1 | 2,923 |
| groups=1:invalidated=0.1pct | 20 | 16,124 | 8,051 | 4,326 | 0.11 | 0.10 | 0.15 | 0.15 | 3 | 12,149 |
| groups=1:invalidated=1pct | 20 | 87,747 | 44,055 | 39,942 | 0.12 | 0.10 | 0.17 | 0.15 | 3 | 47,768 |
| groups=1:invalidated=10pct | 20 | 473,434 | 305,519 | 164,164 | 0.13 | 0.11 | 0.18 | 0.17 | 3 | 171,991 |
| groups=2:invalidated=0pct | 20 | 3,032 | - | 168 | 0.09 | 0.07 | 0.12 | 0.11 | 1 | 3,032 |
| groups=2:invalidated=0.1pct | 20 | 22,808 | 10,383 | 8,652 | 0.11 | 0.10 | 0.15 | 0.15 | 3 | 16,501 |
| groups=2:invalidated=1pct | 20 | 148,053 | 64,389 | 79,884 | 0.14 | 0.12 | 0.16 | 0.16 | 3 | 87,740 |
| groups=2:invalidated=10pct | 20 | 788,693 | 456,585 | 328,328 | 0.15 | 0.14 | 0.18 | 0.17 | 3 | 336,184 |

### Every group (wall_ms)

One row per (build, workload, variant, cache) in the input.

| build | workload | variant | cache | n | med ms | min | max | p95 | udf med ms | stage med ms | commit med ms | outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | append | default | warm | 15 | 4.70 | 4.57 | 4.83 | - | - | 3.84 | 0.87 | - |
| baseline | update_sparse | default | warm | 15 | 12.10 | 11.30 | 12.83 | - | - | - | - | - |
| baseline | update_dense | default | warm | 15 | 36.34 | 35.74 | 37.78 | - | - | - | - | - |
| baseline | update_unrelated_sparse | default | warm | 15 | 12.20 | 11.72 | 13.09 | - | - | - | - | - |
| baseline | merge_insert_partial_sparse | default | warm | 15 | 58.71 | 57.49 | 61.45 | - | - | 57.56 | 1.04 | - |
| baseline | refresh_permissive_clean | default | warm | 15 | 1261.56 | 1259.31 | 1294.85 | - | 1186.03 | 59.52 | 1.01 | committed |
| baseline | refresh_permissive_conflicts_1 | update_row_moving | warm | 15 | 1267.47 | 1264.90 | 1291.11 | - | 1186.18 | 59.84 | 0.13 | conflict:retryable |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 15 | 1293.52 | 1288.42 | 1300.43 | - | 1185.21 | 59.32 | 0.94 | committed(stale=10) |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | warm | 15 | 1300.21 | 1296.94 | 1306.07 | - | 1186.10 | 58.62 | 0.21 | conflict:retryable |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 15 | 1424.11 | 1420.06 | 1508.78 | - | 1187.11 | 59.43 | 1.03 | committed(stale=40) |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | warm | 15 | 1458.62 | 1449.43 | 1505.97 | - | 1186.29 | 59.52 | 0.41 | conflict:retryable |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 15 | 2137.09 | 2129.81 | 2176.11 | - | 1186.89 | 59.16 | 1.26 | committed(stale=160) |
| baseline | publish_after_k_commits | k=0 | warm | 15 | 1.01 | 0.94 | 1.03 | - | - | - | 1.01 | committed |
| baseline | publish_after_k_commits | k=1 | warm | 15 | 0.90 | 0.82 | 1.02 | - | - | - | 0.90 | committed |
| baseline | publish_after_k_commits | k=4 | warm | 15 | 1.00 | 0.96 | 1.21 | - | - | - | 1.00 | committed |
| baseline | publish_after_k_commits | k=16 | warm | 15 | 1.17 | 1.08 | 1.33 | - | - | - | 1.17 | committed |
| baseline | publish_after_k_commits | k=64 | warm | 15 | 1.77 | 1.70 | 2.02 | - | - | - | 1.77 | committed |
| baseline | scan_summary_full | populated | warm | 60 | 2.71 | 2.38 | 2.96 | 2.86 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 2.97 | 2.75 | 3.15 | 3.12 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 2.77 | 2.58 | 3.04 | 2.92 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 3.66 | 3.33 | 4.04 | 3.85 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 1.47 | 1.36 | 1.59 | 1.55 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 5.71 | 4.85 | 6.27 | 6.15 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 2.90 | 2.68 | 3.12 | 3.07 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 3.17 | 2.92 | 3.50 | 3.36 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.97 | 2.62 | 3.18 | 3.14 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 3.85 | 3.54 | 4.28 | 4.21 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 1.79 | 1.55 | 3.88 | 2.88 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 5.97 | 5.45 | 6.53 | 6.45 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 3.07 | 2.79 | 3.34 | 3.25 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 3.42 | 3.08 | 3.74 | 3.65 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.11 | 2.80 | 3.42 | 3.30 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 4.01 | 3.53 | 4.39 | 4.25 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 1.46 | 1.35 | 1.67 | 1.57 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 5.79 | 5.02 | 6.34 | 6.29 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 3.17 | 2.75 | 3.57 | 3.43 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.60 | 3.22 | 3.82 | 3.75 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.27 | 3.00 | 3.58 | 3.45 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.18 | 3.77 | 4.58 | 4.46 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.73 | 1.52 | 1.92 | 1.86 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 6.13 | 5.58 | 6.71 | 6.45 | - | - | - | - |
| prototype | append | default | warm | 15 | 4.70 | 4.59 | 4.87 | - | - | 3.89 | 0.83 | - |
| prototype | update_sparse | default | warm | 15 | 12.10 | 11.54 | 12.90 | - | - | - | - | - |
| prototype | update_dense | default | warm | 15 | 38.17 | 37.44 | 39.47 | - | - | - | - | - |
| prototype | update_unrelated_sparse | default | warm | 15 | 12.28 | 11.69 | 17.40 | - | - | - | - | - |
| prototype | merge_insert_partial_sparse | default | warm | 15 | 59.87 | 58.36 | 61.27 | - | - | 58.80 | 1.02 | - |
| prototype | refresh_permissive_clean | default | warm | 15 | 1262.90 | 1257.33 | 1283.99 | - | 1186.23 | 59.48 | 1.03 | committed |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | warm | 15 | 1266.52 | 1262.78 | 1300.87 | - | 1185.52 | 59.37 | 0.13 | conflict:retryable |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 15 | 1292.87 | 1290.42 | 1298.36 | - | 1184.83 | 59.14 | 0.93 | committed(stale=10) |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | warm | 15 | 1299.39 | 1293.70 | 1324.32 | - | 1184.67 | 59.13 | 0.21 | conflict:retryable |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 15 | 1423.73 | 1415.82 | 1450.32 | - | 1184.84 | 59.20 | 1.03 | committed(stale=40) |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | warm | 15 | 1455.25 | 1451.93 | 1468.45 | - | 1184.67 | 59.04 | 0.42 | conflict:retryable |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 15 | 2132.48 | 2117.30 | 2156.04 | - | 1184.88 | 59.31 | 1.28 | committed(stale=160) |
| prototype | publish_after_k_commits | k=0 | warm | 15 | 1.00 | 0.98 | 1.07 | - | - | - | 1.00 | committed |
| prototype | publish_after_k_commits | k=1 | warm | 15 | 0.91 | 0.83 | 0.99 | - | - | - | 0.91 | committed |
| prototype | publish_after_k_commits | k=4 | warm | 15 | 1.02 | 0.97 | 1.18 | - | - | - | 1.02 | committed |
| prototype | publish_after_k_commits | k=16 | warm | 15 | 1.22 | 1.12 | 1.32 | - | - | - | 1.22 | committed |
| prototype | publish_after_k_commits | k=64 | warm | 15 | 1.82 | 1.76 | 3.39 | - | - | - | 1.82 | committed |
| prototype | scan_summary_full | populated | warm | 60 | 2.66 | 2.30 | 3.00 | 2.90 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 2.93 | 2.69 | 3.18 | 3.08 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 2.77 | 2.50 | 2.93 | 2.91 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 3.71 | 3.39 | 4.43 | 3.97 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 1.43 | 1.31 | 1.62 | 1.54 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 5.76 | 4.87 | 6.28 | 6.08 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 2.86 | 2.59 | 3.14 | 3.04 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 3.16 | 2.96 | 3.46 | 3.37 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 2.94 | 2.68 | 3.43 | 3.18 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 3.85 | 3.65 | 4.26 | 4.10 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 1.70 | 1.53 | 1.85 | 1.80 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 5.92 | 5.33 | 6.76 | 6.51 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 2.99 | 2.62 | 3.21 | 3.20 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 3.40 | 3.07 | 3.71 | 3.64 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 3.09 | 2.92 | 3.36 | 3.27 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 3.97 | 3.65 | 4.44 | 4.33 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 1.43 | 1.36 | 1.59 | 1.52 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 5.91 | 4.66 | 6.56 | 6.43 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 3.16 | 2.75 | 3.45 | 3.38 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 3.56 | 3.26 | 3.81 | 3.73 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 3.25 | 2.98 | 3.44 | 3.39 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 4.16 | 3.93 | 4.51 | 4.45 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 1.72 | 1.55 | 1.93 | 1.81 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 6.04 | 5.50 | 6.66 | 6.49 | - | - | - | - |
| prototype-flags | flagged_update_sparse | groups=1 | warm | 5 | 12.34 | 11.90 | 12.53 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=1 | warm | 5 | 54.01 | 53.39 | 54.27 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=1 | warm | 5 | 11.88 | 11.68 | 12.41 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=1 | warm | 5 | 11.99 | 11.66 | 12.77 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=1 | warm | 5 | 59.32 | 58.76 | 60.55 | - | - | 58.28 | 1.03 | - |
| prototype-flags | flagged_refresh_clean | groups=1 | warm | 5 | 1266.66 | 1259.80 | 1268.97 | - | 1176.85 | 57.88 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:reject | warm | 5 | 1276.42 | 1272.19 | 1278.24 | - | 1179.75 | 59.40 | 0.14 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | warm | 5 | 1271.75 | 1270.42 | 1275.21 | - | 1174.04 | 62.92 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | warm | 5 | 1273.92 | 1269.26 | 1277.72 | - | 1175.73 | 63.12 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:skip | warm | 5 | 1276.09 | 1266.07 | 1302.86 | - | 1179.03 | 59.18 | 1.21 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | warm | 5 | 16.05 | 15.69 | 16.32 | - | 0.17 | 0.36 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | warm | 5 | 16.49 | 16.16 | 16.83 | - | 0.17 | 0.37 | 1.05 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:reject | warm | 5 | 1299.70 | 1294.80 | 1301.53 | - | 1178.56 | 58.20 | 0.15 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 5 | 1265.70 | 1263.01 | 1268.72 | - | 1176.08 | 58.76 | 1.00 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 5 | 1263.21 | 1261.68 | 1266.92 | - | 1174.42 | 57.39 | 1.06 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:skip | warm | 5 | 1298.99 | 1293.81 | 1305.72 | - | 1177.14 | 58.65 | 1.00 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 5 | 42.03 | 40.79 | 45.67 | - | 0.19 | 29.02 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 5 | 42.86 | 40.53 | 43.28 | - | 0.17 | 29.52 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | warm | 5 | 1270.98 | 1270.12 | 1276.13 | - | 1173.74 | 58.19 | 0.14 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 5 | 1268.87 | 1267.02 | 1269.60 | - | 1179.47 | 58.16 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 5 | 1266.80 | 1262.78 | 1268.79 | - | 1175.91 | 57.59 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | warm | 5 | 1274.23 | 1271.88 | 1279.88 | - | 1176.11 | 58.40 | 1.01 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 5 | 631.78 | 628.45 | 635.34 | - | 586.27 | 28.90 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 5 | 43.24 | 42.76 | 44.31 | - | 0.18 | 28.56 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:reject | warm | 5 | 1307.04 | 1302.75 | 1308.52 | - | 1178.93 | 58.12 | 0.21 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | warm | 5 | 1280.43 | 1275.64 | 1285.47 | - | 1175.78 | 67.14 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | warm | 5 | 1284.80 | 1278.04 | 1286.88 | - | 1179.18 | 67.42 | 1.09 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:skip | warm | 5 | 1309.19 | 1299.65 | 1323.39 | - | 1177.50 | 58.50 | 1.44 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | warm | 5 | 29.86 | 29.44 | 30.25 | - | 0.33 | 1.02 | 0.94 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | warm | 5 | 30.32 | 29.91 | 30.76 | - | 0.33 | 1.05 | 0.92 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:reject | warm | 5 | 1425.60 | 1420.14 | 1431.38 | - | 1175.01 | 58.29 | 0.23 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 5 | 1268.58 | 1261.81 | 1271.02 | - | 1179.01 | 58.00 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 5 | 1266.86 | 1262.78 | 1270.04 | - | 1177.45 | 57.95 | 1.05 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:skip | warm | 5 | 1428.19 | 1417.98 | 1432.84 | - | 1178.98 | 58.14 | 1.13 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 5 | 73.86 | 72.70 | 76.00 | - | 0.39 | 51.77 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 5 | 74.57 | 71.76 | 75.25 | - | 0.35 | 51.16 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | warm | 5 | 1306.44 | 1303.18 | 1311.45 | - | 1177.85 | 58.12 | 0.21 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 5 | 1263.44 | 1261.07 | 1266.10 | - | 1174.98 | 57.01 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 5 | 1262.02 | 1260.79 | 1265.86 | - | 1174.23 | 57.83 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | warm | 5 | 1307.97 | 1301.45 | 1347.60 | - | 1178.86 | 58.39 | 0.95 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 5 | 1140.48 | 1133.40 | 1147.54 | - | 1059.20 | 52.93 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 5 | 76.69 | 75.70 | 89.65 | - | 0.34 | 52.01 | 1.06 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:reject | warm | 5 | 1464.00 | 1461.37 | 1479.45 | - | 1177.08 | 58.43 | 0.43 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | warm | 5 | 1290.85 | 1286.11 | 1294.01 | - | 1176.00 | 71.79 | 1.17 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | warm | 5 | 1291.28 | 1284.74 | 1294.43 | - | 1176.28 | 71.81 | 1.18 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:skip | warm | 5 | 1464.55 | 1459.09 | 1468.73 | - | 1177.54 | 58.45 | 1.81 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | warm | 5 | 42.16 | 41.35 | 42.59 | - | 0.53 | 3.77 | 1.05 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | warm | 5 | 42.08 | 41.63 | 42.84 | - | 0.52 | 3.63 | 1.07 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:reject | warm | 5 | 2130.52 | 2120.28 | 2148.76 | - | 1178.78 | 57.19 | 0.47 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 5 | 1264.45 | 1262.76 | 1271.69 | - | 1175.98 | 57.30 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 5 | 1269.33 | 1263.43 | 1269.86 | - | 1178.34 | 58.17 | 1.06 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:skip | warm | 5 | 2125.72 | 2121.26 | 2134.01 | - | 1176.92 | 57.96 | 1.47 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 5 | 84.24 | 84.22 | 84.87 | - | 0.64 | 57.09 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 5 | 84.16 | 83.17 | 86.11 | - | 0.58 | 57.09 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | warm | 5 | 1456.20 | 1450.29 | 1482.74 | - | 1179.73 | 58.75 | 0.38 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 5 | 1263.54 | 1261.04 | 1296.83 | - | 1175.51 | 57.28 | 1.06 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 5 | 1265.81 | 1263.51 | 1268.59 | - | 1177.26 | 57.82 | 1.03 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | warm | 5 | 1447.70 | 1446.72 | 1453.87 | - | 1175.92 | 58.56 | 0.47 | deferred_all |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 5 | 1267.50 | 1262.29 | 1270.81 | - | 1180.04 | 58.59 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 5 | 83.85 | 80.88 | 85.09 | - | 0.55 | 56.66 | 1.00 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=0 | warm | 5 | 1.00 | 0.97 | 1.03 | - | - | - | 1.00 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=1 | warm | 5 | 0.95 | 0.88 | 1.19 | - | - | - | 0.95 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=4 | warm | 5 | 1.06 | 0.99 | 1.16 | - | - | - | 1.06 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=16 | warm | 5 | 1.23 | 1.19 | 1.28 | - | - | - | 1.23 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=64 | warm | 5 | 1.88 | 1.84 | 1.94 | - | - | - | 1.88 | committed |
| prototype-flags | flagged_update_sparse | groups=2 | warm | 5 | 13.06 | 12.72 | 13.14 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=2 | warm | 5 | 53.38 | 53.13 | 54.02 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=2 | warm | 5 | 13.41 | 12.80 | 13.96 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=2 | warm | 5 | 12.69 | 12.60 | 13.29 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=2 | warm | 5 | 56.65 | 56.09 | 57.15 | - | - | 55.51 | 1.08 | - |
| prototype-flags | flagged_refresh_clean | groups=2 | warm | 5 | 2334.56 | 2330.32 | 2338.39 | - | 2158.55 | 117.28 | 2.00 | committed |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 2.56 | 2.29 | 2.71 | 2.71 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 2.92 | 2.71 | 3.09 | 3.01 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 2.62 | 2.52 | 2.72 | 2.69 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 3.54 | 3.33 | 3.91 | 3.88 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 1.43 | 1.32 | 1.55 | 1.51 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 5.67 | 4.60 | 6.09 | 6.02 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 2.76 | 2.58 | 2.92 | 2.90 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 2.92 | 2.74 | 3.15 | 3.14 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 2.71 | 2.59 | 2.87 | 2.83 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 3.59 | 3.35 | 3.83 | 3.79 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 1.72 | 1.60 | 1.83 | 1.79 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 5.92 | 5.34 | 6.45 | 6.37 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 5.29 | 4.58 | 5.62 | 5.51 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 5.83 | 5.15 | 6.28 | 6.17 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 5.26 | 4.54 | 5.67 | 5.65 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 7.53 | 6.68 | 7.69 | 7.67 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 1.43 | 1.37 | 1.57 | 1.52 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 5.62 | 4.79 | 6.14 | 6.00 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 5.30 | 4.52 | 5.57 | 5.55 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 5.96 | 5.20 | 6.47 | 6.38 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 5.44 | 4.71 | 5.85 | 5.67 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 7.59 | 6.81 | 7.89 | 7.81 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 1.61 | 1.53 | 1.77 | 1.67 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 6.04 | 5.28 | 6.59 | 6.41 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 0.29 | 0.29 | 0.32 | 0.31 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 0.61 | 0.59 | 0.65 | 0.64 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 0.33 | 0.32 | 0.34 | 0.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 0.97 | 0.91 | 1.04 | 1.03 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 1.43 | 1.35 | 1.53 | 1.51 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.08 | 0.07 | 0.09 | 0.08 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 0.29 | 0.28 | 0.30 | 0.30 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 0.61 | 0.59 | 0.64 | 0.62 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 0.33 | 0.33 | 0.35 | 0.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 0.96 | 0.91 | 1.02 | 1.01 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 1.65 | 1.52 | 1.82 | 1.77 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.09 | 0.07 | 0.12 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0pct | fresh-session | 20 | 0.08 | 0.07 | 0.13 | 0.12 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0.1pct | fresh-session | 20 | 0.08 | 0.07 | 0.11 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=1pct | fresh-session | 20 | 0.08 | 0.07 | 0.12 | 0.12 | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=10pct | fresh-session | 20 | 0.08 | 0.07 | 0.11 | 0.10 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0pct | fresh-session | 20 | 0.08 | 0.08 | 0.13 | 0.13 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0.1pct | fresh-session | 20 | 0.11 | 0.10 | 0.15 | 0.15 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=1pct | fresh-session | 20 | 0.12 | 0.10 | 0.17 | 0.15 | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=10pct | fresh-session | 20 | 0.13 | 0.11 | 0.18 | 0.17 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0pct | fresh-session | 20 | 0.09 | 0.07 | 0.12 | 0.11 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0.1pct | fresh-session | 20 | 0.11 | 0.10 | 0.15 | 0.15 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=1pct | fresh-session | 20 | 0.14 | 0.12 | 0.16 | 0.16 | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=10pct | fresh-session | 20 | 0.15 | 0.14 | 0.18 | 0.17 | - | - | - | - |
