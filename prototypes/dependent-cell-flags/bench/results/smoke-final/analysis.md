## Results: smoke

Source: `results/smoke-final`

### Environment

| key | value |
|---|---|
| scale | `smoke` |
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
| rounds | `1` |
| flag_every_round | `True` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "100000", "BENCH_ROWS_PER_FRAGMENT": "10000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "3", "BENCH_WARMUP": "1"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T06:46:48Z	2026-09-25T06:46:59Z	25.78 21.47 13.67
1	prototype	2026-09-25T06:46:59Z	2026-09-25T06:47:11Z	22.64 20.95 13.57
1	prototype-flags	2026-09-25T06:47:11Z	2026-09-25T06:48:10Z	18.25 20.06 13.39
```

Records: 603. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| append | default | warm | 3 | 4.67 | 4.63 | 4.71 | - | 3 | 4.72 | 4.63 | 4.73 | - | 1.010 | 1.010 | 1,475 | 1,475 | 134 | 134 | 1 | 1 | 725,890 | 726,210 | - / - |
| update_sparse | default | warm | 3 | 5.61 | 5.49 | 5.68 | - | 3 | 5.39 | 5.16 | 5.43 | - | 0.959 | 0.959 | 2,702 | 2,698 | 1,191 | 1,189 | 161 | 161 | 16,788 | 16,782 | - / - |
| update_dense | default | warm | 3 | 7.73 | 7.58 | 8.02 | - | 3 | 7.88 | 7.77 | 7.91 | - | 1.019 | 1.019 | 2,724 | 2,722 | 1,202 | 1,201 | 35 | 35 | 246,171 | 246,104 | - / - |
| update_unrelated_sparse | default | warm | 3 | 5.61 | 5.44 | 5.99 | - | 3 | 5.40 | 4.97 | 5.45 | - | 0.961 | 0.961 | 2,702 | 2,698 | 1,191 | 1,189 | 161 | 161 | 23,592 | 23,586 | - / - |
| merge_insert_partial_sparse | default | warm | 3 | 13.98 | 13.08 | 14.18 | - | 3 | 13.47 | 13.25 | 13.72 | - | 0.963 | 0.963 | 4,087 | 4,087 | 1,888 | 1,888 | 31 | 31 | 5,317,499 | 5,316,219 | - / - |
| refresh_permissive_clean | default | warm | 3 | 159.17 | 158.49 | 159.59 | - | 3 | 150.78 | 149.67 | 151.43 | - | 0.947 | 0.947 | 2,821 | 2,820 | 822 | 822 | 11 | 11 | 1,520,787 | 1,531,602 | committed / committed |
| refresh_permissive_conflicts_1 † | update_row_moving | warm | 3 | 157.02 | 156.86 | 158.15 | - | 3 | 153.52 | 152.93 | 154.12 | - | 0.978 | 0.978 | - | - | - | - | 56 | 56 | 1,523,828 | 1,515,313 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_1 † | merge_insert_in_place | warm | 3 | 164.13 | 163.14 | 164.61 | - | 3 | 157.36 | 155.28 | 160.30 | - | 0.959 | 0.959 | 3,291 | 3,291 | 822 | 822 | 33 | 33 | 4,197,587 | 4,172,755 | committed(stale=10) / committed(stale=10) |
| refresh_permissive_conflicts_4 † | update_row_moving | warm | 3 | 168.65 | 168.22 | 168.78 | - | 3 | 164.99 | 164.78 | 165.03 | - | 0.978 | 0.978 | - | - | - | - | 237 | 237 | 1,559,393 | 1,568,295 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_4 † | merge_insert_in_place | warm | 3 | 190.07 | 189.91 | 192.27 | - | 3 | 187.69 | 186.88 | 189.55 | - | 0.987 | 0.987 | 3,762 | 3,761 | 822 | 822 | 163 | 163 | 17,501,427 | 17,509,939 | committed(stale=40) / committed(stale=40) |
| refresh_permissive_conflicts_16 † | update_row_moving | warm | 3 | 227.19 | 226.64 | 227.71 | - | 3 | 223.53 | 223.35 | 224.07 | - | 0.984 | 0.984 | - | - | - | - | 1,078 | 1,078 | 1,739,034 | 1,736,501 | conflict:retryable / conflict:retryable |
| refresh_permissive_conflicts_16 † | merge_insert_in_place | warm | 3 | 339.94 | 339.93 | 341.16 | - | 3 | 341.02 | 338.95 | 371.47 | - | 1.003 | 1.003 | 3,764 | 3,764 | 823 | 823 | 765 | 765 | 78,668,554 | 78,650,075 | committed(stale=160) / committed(stale=160) |
| publish_after_k_commits | k=0 | warm | 3 | 0.92 | 0.92 | 0.93 | - | 3 | 0.93 | 0.91 | 1.01 | - | 1.013 | 1.013 | 2,824 | 2,823 | 823 | 823 | 1 | 1 | 3,662 | 3,661 | committed / committed |
| publish_after_k_commits | k=1 | warm | 3 | 0.94 | 0.91 | 0.95 | - | 3 | 0.87 | 0.85 | 0.88 | - | 0.924 | 0.924 | 2,914 | 2,915 | 823 | 823 | 2 | 2 | 3,752 | 3,753 | committed / committed |
| publish_after_k_commits | k=4 | warm | 3 | 1.01 | 1.00 | 1.07 | - | 3 | 0.98 | 0.97 | 1.05 | - | 0.969 | 0.969 | 3,188 | 3,188 | 823 | 823 | 5 | 5 | 4,026 | 4,026 | committed / committed |
| publish_after_k_commits | k=16 | warm | 3 | 1.22 | 1.12 | 1.23 | - | 3 | 1.27 | 1.20 | 1.43 | - | 1.041 | 1.041 | 4,297 | 4,297 | 823 | 823 | 17 | 17 | 5,135 | 5,135 | committed / committed |
| publish_after_k_commits | k=64 | warm | 3 | 1.81 | 1.75 | 2.04 | - | 3 | 2.12 | 1.80 | 2.20 | - | 1.169 | 1.169 | 8,713 | 8,713 | 823 | 823 | 65 | 65 | 9,551 | 9,551 | committed / committed |
| scan_summary_full | populated | warm | 3 | 0.74 | 0.63 | 0.76 | - | 3 | 0.75 | 0.74 | 0.77 | - | 1.013 | 1.013 | 2,820 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 3 | 0.88 | 0.83 | 0.91 | - | 3 | 0.79 | 0.77 | 0.81 | - | 0.899 | 0.899 | 2,820 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 3 | 0.73 | 0.73 | 0.73 | - | 3 | 0.77 | 0.71 | 0.77 | - | 1.047 | 1.047 | 2,820 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 3 | 1.19 | 1.18 | 1.25 | - | 3 | 1.20 | 1.10 | 1.22 | - | 1.009 | 1.009 | 2,820 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 3 | 0.75 | 0.74 | 0.85 | - | 3 | 0.76 | 0.72 | 0.79 | - | 1.008 | 1.008 | 2,820 | 2,820 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 3 | 1.31 | 1.29 | 1.32 | - | 3 | 1.28 | 1.27 | 1.33 | - | 0.979 | 0.979 | 2,820 | 2,820 | 822 | 822 | 13 | 13 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 3 | 0.91 | 0.78 | 0.92 | - | 3 | 0.89 | 0.78 | 0.90 | - | 0.984 | 0.984 | 2,820 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 3 | 0.94 | 0.90 | 1.13 | - | 3 | 0.94 | 0.87 | 0.97 | - | 0.996 | 0.996 | 2,820 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 3 | 0.91 | 0.81 | 0.93 | - | 3 | 0.95 | 0.79 | 0.98 | - | 1.044 | 1.044 | 2,820 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 3 | 1.30 | 1.28 | 1.38 | - | 3 | 1.36 | 1.22 | 1.41 | - | 1.045 | 1.045 | 2,820 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 3 | 1.07 | 0.98 | 1.12 | - | 3 | 1.09 | 1.01 | 1.09 | - | 1.020 | 1.020 | 2,820 | 2,820 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 3 | 1.40 | 1.40 | 1.48 | - | 3 | 1.50 | 1.46 | 1.54 | - | 1.076 | 1.076 | 2,820 | 2,820 | 822 | 822 | 33 | 33 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 3 | 0.80 | 0.78 | 0.85 | - | 3 | 0.80 | 0.78 | 0.88 | - | 0.999 | 0.999 | 2,819 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 3 | 0.91 | 0.89 | 0.93 | - | 3 | 0.89 | 0.87 | 0.93 | - | 0.978 | 0.978 | 2,819 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 3 | 0.77 | 0.70 | 0.82 | - | 3 | 0.81 | 0.73 | 0.85 | - | 1.058 | 1.058 | 2,819 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 3 | 1.20 | 1.14 | 1.21 | - | 3 | 1.26 | 1.19 | 1.29 | - | 1.044 | 1.044 | 2,819 | 2,820 | 822 | 822 | 10 | 10 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 3 | 0.80 | 0.80 | 0.91 | - | 3 | 0.83 | 0.69 | 0.84 | - | 1.033 | 1.033 | 2,819 | 2,820 | 822 | 822 | 11 | 11 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 3 | 1.32 | 1.31 | 1.36 | - | 3 | 1.35 | 1.32 | 1.36 | - | 1.018 | 1.018 | 2,819 | 2,820 | 822 | 822 | 13 | 13 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 3 | 0.92 | 0.83 | 0.93 | - | 3 | 0.96 | 0.95 | 1.04 | - | 1.041 | 1.041 | 2,819 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 3 | 1.08 | 1.04 | 1.09 | - | 3 | 1.10 | 1.08 | 1.14 | - | 1.015 | 1.015 | 2,819 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 3 | 0.95 | 0.88 | 0.97 | - | 3 | 0.99 | 0.88 | 1.07 | - | 1.033 | 1.033 | 2,819 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 3 | 1.37 | 1.25 | 1.43 | - | 3 | 1.48 | 1.29 | 1.53 | - | 1.079 | 1.079 | 2,819 | 2,820 | 822 | 822 | 30 | 30 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 3 | 1.02 | 0.99 | 1.07 | - | 3 | 1.09 | 0.92 | 1.15 | - | 1.068 | 1.068 | 2,819 | 2,820 | 822 | 822 | 43 | 43 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 3 | 1.53 | 1.51 | 1.61 | - | 3 | 1.49 | 1.48 | 1.50 | - | 0.971 | 0.971 | 2,819 | 2,820 | 822 | 822 | 33 | 33 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | warm | update_sparse:default | 3 | 5.39 | 5.16 | 5.43 | - | 3 | 6.03 | 5.95 | 6.12 | - | 1.120 | 1.120 | - | - | 2,698 | 5,565 | 1,189 | 2,638 | 16,782 | 24,005 |
| flagged_update_dense | groups=1 | warm | update_dense:default | 3 | 7.88 | 7.77 | 7.91 | - | 3 | 10.06 | 9.87 | 10.24 | - | 1.278 | 1.278 | - | - | 2,722 | 72,986 | 1,201 | 30,444 | 246,104 | 491,016 |
| flagged_update_unrelated_sparse | groups=1 | warm | update_unrelated_sparse:default | 3 | 5.40 | 4.97 | 5.45 | - | 3 | 5.85 | 5.71 | 6.15 | - | 1.083 | 1.083 | - | - | 2,698 | 5,576 | 1,189 | 2,639 | 23,586 | 30,821 |
| flagged_merge_insert_partial_sparse | groups=1 | warm | merge_insert_partial_sparse:default | 3 | 13.47 | 13.25 | 13.72 | - | 3 | 13.51 | 12.99 | 13.63 | - | 1.003 | 1.003 | 0.89 | 0.98 | 4,087 | 7,113 | 1,888 | 3,501 | 5,316,219 | 5,319,322 |
| flagged_refresh_clean | groups=1 | warm | refresh_permissive_clean:default | 3 | 150.78 | 149.67 | 151.43 | - | 3 | 148.01 | 146.54 | 149.02 | - | 0.982 | 0.982 | 0.91 | 0.94 | 2,820 | 3,035 | 822 | 916 | 1,531,602 | 1,532,038 |
| flagged_publish_after_k_commits | k=0 | warm | publish_after_k_commits:k=0 | 3 | 0.93 | 0.91 | 1.01 | - | 3 | 0.97 | 0.93 | 1.00 | - | 1.042 | 1.042 | 0.93 | 0.97 | 2,823 | 3,039 | 823 | 917 | 3,661 | 3,971 |
| flagged_publish_after_k_commits | k=1 | warm | publish_after_k_commits:k=1 | 3 | 0.87 | 0.85 | 0.88 | - | 3 | 0.82 | 0.81 | 0.86 | - | 0.940 | 0.940 | 0.87 | 0.82 | 2,915 | 3,131 | 823 | 917 | 3,753 | 4,063 |
| flagged_publish_after_k_commits | k=4 | warm | publish_after_k_commits:k=4 | 3 | 0.98 | 0.97 | 1.05 | - | 3 | 1.09 | 1.06 | 1.20 | - | 1.110 | 1.110 | 0.98 | 1.09 | 3,188 | 3,406 | 823 | 917 | 4,026 | 4,338 |
| flagged_publish_after_k_commits | k=16 | warm | publish_after_k_commits:k=16 | 3 | 1.27 | 1.20 | 1.43 | - | 3 | 1.23 | 1.16 | 1.23 | - | 0.963 | 0.963 | 1.27 | 1.23 | 4,297 | 4,511 | 823 | 917 | 5,135 | 5,443 |
| flagged_publish_after_k_commits | k=64 | warm | publish_after_k_commits:k=64 | 3 | 2.12 | 1.80 | 2.20 | - | 3 | 1.92 | 1.92 | 2.14 | - | 0.908 | 0.908 | 2.12 | 1.92 | 8,713 | 8,928 | 823 | 917 | 9,551 | 9,861 |
| flagged_update_sparse | groups=2 | warm | update_sparse:default | 3 | 5.39 | 5.16 | 5.43 | - | 3 | 6.47 | 6.45 | 6.52 | - | 1.202 | 1.202 | - | - | 2,698 | 7,706 | 1,189 | 3,378 | 16,782 | 29,345 |
| flagged_update_dense | groups=2 | warm | update_dense:default | 3 | 7.88 | 7.77 | 7.91 | - | 3 | 10.62 | 10.28 | 10.84 | - | 1.348 | 1.348 | - | - | 2,722 | 114,731 | 1,201 | 31,185 | 246,104 | 692,152 |
| flagged_update_unrelated_sparse | groups=2 | warm | update_unrelated_sparse:default | 3 | 5.40 | 4.97 | 5.45 | - | 3 | 6.41 | 6.28 | 6.65 | - | 1.187 | 1.187 | - | - | 2,698 | 7,725 | 1,189 | 3,379 | 23,586 | 36,169 |
| flagged_merge_insert_partial_sparse | groups=2 | warm | merge_insert_partial_sparse:default | 3 | 13.47 | 13.25 | 13.72 | - | 3 | 13.16 | 12.96 | 13.24 | - | 0.977 | 0.977 | 0.89 | 1.02 | 4,087 | 9,705 | 1,888 | 4,693 | 5,316,219 | 5,326,562 |
| scan_summary_full | all_true | warm | scan_summary_full:populated | 3 | 0.75 | 0.74 | 0.77 | - | 3 | 0.69 | 0.67 | 0.82 | - | 0.917 | 0.917 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 3 | 0.79 | 0.77 | 0.81 | - | 3 | 0.80 | 0.79 | 0.85 | - | 1.009 | 1.009 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 3 | 0.77 | 0.71 | 0.77 | - | 3 | 0.73 | 0.73 | 0.77 | - | 0.960 | 0.960 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 3 | 1.20 | 1.10 | 1.22 | - | 3 | 1.21 | 1.12 | 1.29 | - | 1.006 | 1.006 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 3 | 0.76 | 0.72 | 0.79 | - | 3 | 0.81 | 0.81 | 0.82 | - | 1.072 | 1.072 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 3 | 1.28 | 1.27 | 1.33 | - | 3 | 1.31 | 1.30 | 1.32 | - | 1.019 | 1.019 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 3 | 0.89 | 0.78 | 0.90 | - | 3 | 0.87 | 0.85 | 0.90 | - | 0.975 | 0.975 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 3 | 0.94 | 0.87 | 0.97 | - | 3 | 1.06 | 1.01 | 1.18 | - | 1.123 | 1.123 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 3 | 0.95 | 0.79 | 0.98 | - | 3 | 0.90 | 0.82 | 0.94 | - | 0.942 | 0.942 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 3 | 1.36 | 1.22 | 1.41 | - | 3 | 1.38 | 1.28 | 1.48 | - | 1.014 | 1.014 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 3 | 1.09 | 1.01 | 1.09 | - | 3 | 1.01 | 1.00 | 1.04 | - | 0.932 | 0.932 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 3 | 1.50 | 1.46 | 1.54 | - | 3 | 1.48 | 1.44 | 1.62 | - | 0.984 | 0.984 | - | - | 2,820 | 3,034 | 822 | 916 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 3 | 0.80 | 0.78 | 0.88 | - | 3 | 1.11 | 0.96 | 1.15 | - | 1.383 | 1.383 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 3 | 0.89 | 0.87 | 0.93 | - | 3 | 1.25 | 1.12 | 1.40 | - | 1.413 | 1.413 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 3 | 0.81 | 0.73 | 0.85 | - | 3 | 1.15 | 0.92 | 1.32 | - | 1.423 | 1.423 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 3 | 1.26 | 1.19 | 1.29 | - | 3 | 1.66 | 1.64 | 1.78 | - | 1.322 | 1.322 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 3 | 0.83 | 0.69 | 0.84 | - | 3 | 0.78 | 0.74 | 0.83 | - | 0.942 | 0.942 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 3 | 1.35 | 1.32 | 1.36 | - | 3 | 1.32 | 1.29 | 1.39 | - | 0.979 | 0.979 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 3 | 0.96 | 0.95 | 1.04 | - | 3 | 1.37 | 1.24 | 1.38 | - | 1.431 | 1.431 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 3 | 1.10 | 1.08 | 1.14 | - | 3 | 1.49 | 1.32 | 1.53 | - | 1.358 | 1.358 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 3 | 0.99 | 0.88 | 1.07 | - | 3 | 1.22 | 1.16 | 1.34 | - | 1.237 | 1.237 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 3 | 1.48 | 1.29 | 1.53 | - | 3 | 1.63 | 1.61 | 1.79 | - | 1.104 | 1.104 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 3 | 1.09 | 0.92 | 1.15 | - | 3 | 1.01 | 0.99 | 1.11 | - | 0.925 | 0.925 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 3 | 1.49 | 1.48 | 1.50 | - | 3 | 1.48 | 1.46 | 1.52 | - | 0.994 | 0.994 | - | - | 2,820 | 14,288 | 822 | 7,121 | 0 | 0 |
| flagged_update_title_sparse | groups=1 | warm | (no counterpart) | - | - | - | - | - | 3 | 5.90 | 5.73 | 6.00 | - | - | - | - | - | - | 5,568 | - | 2,639 | - | 28,745 |
| flagged_update_title_sparse | groups=2 | warm | (no counterpart) | - | - | - | - | - | 3 | 6.42 | 6.16 | 6.47 | - | - | - | - | - | - | 7,717 | - | 3,379 | - | 34,093 |
| flagged_refresh_clean | groups=2 | warm | (no counterpart) | - | - | - | - | - | 3 | 278.19 | 277.97 | 278.61 | - | - | - | - | 1.90 | - | 3,885 | - | 916 | - | 3,032,092 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 3 | 0.16 | 0.14 | 0.18 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 3 | 0.29 | 0.28 | 0.30 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 3 | 0.15 | 0.14 | 0.16 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 3 | 0.55 | 0.49 | 0.59 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 3 | 0.75 | 0.69 | 0.76 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 3 | 0.09 | 0.09 | 0.10 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.14 | 0.14 | 0.14 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.27 | 0.26 | 0.29 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.14 | 0.14 | 0.16 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.55 | 0.54 | 0.58 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.98 | 0.92 | 1.01 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 3 | 0.08 | 0.08 | 0.09 | - | - | - | - | - | - | 1,340 | - | 62 | - | 0 |

Flag effects of the source writes (asserted exact in every sample; first sample shown):

| workload | variant | rows written | flags cleared | flags carried | derived invalidation rows | moved rows |
|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | 100 | {"summary": 100} | {"summary": 0} | {"summary": 0} | 100 |
| flagged_update_dense | groups=1 | 10,000 | {"summary": 10000} | {"summary": 0} | {"summary": 0} | 10,000 |
| flagged_update_unrelated_sparse | groups=1 | 100 | {"summary": 0} | {"summary": 100} | {"summary": 0} | 100 |
| flagged_update_title_sparse | groups=1 | 100 | {"summary": 100} | {"summary": 0} | {"summary": 0} | 100 |
| flagged_merge_insert_partial_sparse | groups=1 | 100 | {"summary": 100} | null | {"summary": 100} | - |
| flagged_update_sparse | groups=2 | 100 | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | {"summary": 0, "translation": 0} | 100 |
| flagged_update_dense | groups=2 | 10,000 | {"summary": 10000, "translation": 10000} | {"summary": 0, "translation": 0} | {"summary": 0, "translation": 0} | 10,000 |
| flagged_update_unrelated_sparse | groups=2 | 100 | {"summary": 0, "translation": 0} | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | 100 |
| flagged_update_title_sparse | groups=2 | 100 | {"summary": 100, "translation": 0} | {"summary": 0, "translation": 100} | {"summary": 0, "translation": 0} | 100 |
| flagged_merge_insert_partial_sparse | groups=2 | 100 | {"summary": 100, "translation": 100} | null | {"summary": 100, "translation": 100} | - |

### Refresh under concurrent source writes

A full `summary` refresh staged at V, K source commits of 10 scattered rows each, then the publication at V. `commit ms` is the conflict-checked publication commit. `merge_insert_output_in_place` writes `summary` itself (an output override); it is the only source write here that defers whole groups.

| workload | variant | n | outcome | rows assigned | rows published | rows deferred | deferred by reason | fragments deferred | fragment reasons | valid rows in deferred groups | commit med ms | commit p95 | wall med ms | udf ms | source commits ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_refresh_conflicts_1 | update_row_moving:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.16 | - | 157.51 | 118.40 | 3.86 |
| flagged_refresh_conflicts_1 | update_row_moving:skip | 3 | committed_partial | 100,000 | 99,990 | 10 | {"RowVacated": 10} | 0 | {} | 0 | 1.19 | - | 155.88 | 118.47 | 3.66 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.15 | - | 160.18 | 118.01 | 8.57 |
| flagged_refresh_conflicts_1 | merge_insert_in_place:skip | 3 | committed_partial | 100,000 | 99,990 | 10 | {"InputChanged": 10} | 0 | {} | 0 | 0.92 | - | 156.46 | 117.51 | 8.21 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.14 | - | 152.12 | 118.02 | 4.24 |
| flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | 3 | committed_partial | 100,000 | 50,000 | 10 | {"InputChanged": 10} | 5 | {"OutputWritten": 5} | 49,990 | 0.91 | - | 153.06 | 118.16 | 4.29 |
| flagged_refresh_conflicts_4 | update_row_moving:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.20 | - | 164.61 | 117.86 | 16.67 |
| flagged_refresh_conflicts_4 | update_row_moving:skip | 3 | committed_partial | 100,000 | 99,960 | 40 | {"RowVacated": 40} | 0 | {} | 0 | 1.53 | - | 164.61 | 117.58 | 16.47 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.23 | - | 183.99 | 117.88 | 36.24 |
| flagged_refresh_conflicts_4 | merge_insert_in_place:skip | 3 | committed_partial | 100,000 | 99,960 | 40 | {"InputChanged": 40} | 0 | {} | 0 | 1.08 | - | 184.70 | 118.00 | 36.05 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.21 | - | 165.99 | 117.93 | 18.14 |
| flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | 3 | deferred_all | 100,000 | 0 | 40 | {"InputChanged": 40} | 10 | {"OutputWritten": 10} | 99,960 | 0.26 | - | 167.02 | 118.42 | 18.60 |
| flagged_refresh_conflicts_16 | update_row_moving:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.39 | - | 220.79 | 117.52 | 73.02 |
| flagged_refresh_conflicts_16 | update_row_moving:skip | 3 | committed_partial | 100,000 | 99,840 | 160 | {"RowVacated": 160} | 0 | {} | 0 | 1.95 | - | 224.44 | 118.32 | 73.49 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.37 | - | 336.27 | 118.00 | 188.09 |
| flagged_refresh_conflicts_16 | merge_insert_in_place:skip | 3 | committed_partial | 100,000 | 99,840 | 160 | {"InputChanged": 160} | 0 | {} | 0 | 1.37 | - | 336.20 | 117.96 | 187.11 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | 3 | conflict:retryable | 100,000 | 0 | - | {} | - | {} | - | 0.44 | - | 227.18 | 117.85 | 79.40 |
| flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | 3 | deferred_all | 100,000 | 0 | 160 | {"InputChanged": 160} | 10 | {"OutputWritten": 10} | 99,840 | 0.54 | - | 228.90 | 117.97 | 80.00 |

Permissive publication without flags (regression harness), **NOT correctness-equivalent**: it has no dependency tracking, so in-place writes publish stale summaries (`stale`) and row-moving writes are rejected by main's rules:

| build | workload | variant | n | outcome | rows published | stale rows | commit med ms | wall med ms |
|---|---|---|---|---|---|---|---|---|
| baseline | refresh_permissive_conflicts_1 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.13 | 157.02 |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.14 | 153.52 |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | 3 | committed(stale=10) | 100,000 | 10 | 0.92 | 164.13 |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | 3 | committed(stale=10) | 100,000 | 10 | 0.95 | 157.36 |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.18 | 168.65 |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.21 | 164.99 |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | 3 | committed(stale=40) | 100,000 | 40 | 1.01 | 190.07 |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | 3 | committed(stale=40) | 100,000 | 40 | 1.00 | 187.69 |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.38 | 227.19 |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | 3 | conflict:retryable | 0 | 0 | 0.37 | 223.53 |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | 3 | committed(stale=160) | 100,000 | 160 | 1.21 | 339.94 |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | 3 | committed(stale=160) | 100,000 | 160 | 1.22 | 341.02 |

### Follow-up refresh to completion: saved and repeated computation

From the state the conflicted publication left, each strategy refreshes every pending row and publishes; afterwards every flag is asserted true and every value equal to the UDF of its current inputs. `recompute_all_pending` ignores the report; `reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject returns an error, so it has no report). `total UDF rows` = rows the conflicted publication computed + rows the follow-up recomputed; with N rows, anything above N is repeated computation.

| workload | variant | n | rows computed first | rows published first | rows recomputed | rows reused | rows copied through | total UDF rows | udf med ms | stage med ms | commit med ms | wall med ms | written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 118.46 | 22.51 | 1.02 | 152.27 | 1,601,486 |
| flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 118.15 | 22.32 | 1.00 | 151.44 | 1,629,262 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | 3 | 100,000 | 99,990 | 10 | 0 | 49,990 | 100,010 | 0.03 | 0.32 | 0.92 | 5.86 | 3,411 |
| flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | 3 | 100,000 | 99,990 | 10 | 0 | 49,990 | 100,010 | 0.03 | 0.34 | 0.92 | 6.23 | 3,411 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.69 | 20.84 | 0.94 | 147.84 | 1,521,119 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.60 | 20.65 | 0.98 | 147.69 | 1,511,967 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | 3 | 100,000 | 99,990 | 10 | 0 | 49,990 | 100,010 | 0.03 | 10.27 | 0.91 | 14.83 | 772,115 |
| flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | 3 | 100,000 | 99,990 | 10 | 0 | 49,990 | 100,010 | 0.03 | 10.19 | 0.89 | 14.88 | 782,866 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 118.16 | 21.11 | 0.95 | 148.90 | 1,529,826 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.71 | 20.83 | 0.99 | 148.52 | 1,531,810 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | 3 | 100,000 | 50,000 | 50,000 | 0 | 0 | 150,000 | 58.98 | 10.56 | 0.91 | 74.96 | 769,669 |
| flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 3 | 100,000 | 50,000 | 10 | 49,990 | 0 | 100,010 | 0.03 | 10.35 | 0.95 | 15.44 | 772,805 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.76 | 22.75 | 1.01 | 151.73 | 1,715,757 |
| flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.87 | 22.71 | 0.96 | 151.90 | 1,706,730 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | 3 | 100,000 | 99,960 | 40 | 0 | 99,960 | 100,040 | 0.08 | 0.92 | 0.91 | 10.92 | 6,593 |
| flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | 3 | 100,000 | 99,960 | 40 | 0 | 99,960 | 100,040 | 0.08 | 0.94 | 0.86 | 11.26 | 6,593 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.60 | 20.75 | 0.92 | 147.35 | 1,533,880 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.99 | 20.74 | 0.96 | 148.19 | 1,502,904 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | 3 | 100,000 | 99,960 | 40 | 0 | 99,960 | 100,040 | 0.08 | 20.61 | 1.01 | 28.96 | 1,532,254 |
| flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | 3 | 100,000 | 99,960 | 40 | 0 | 99,960 | 100,040 | 0.08 | 20.51 | 0.98 | 28.92 | 1,511,389 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.82 | 21.04 | 1.00 | 148.60 | 1,549,048 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 118.03 | 20.96 | 0.98 | 148.98 | 1,523,000 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.66 | 20.99 | 0.93 | 148.44 | 1,534,391 |
| flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 3 | 100,000 | 0 | 40 | 99,960 | 0 | 100,040 | 0.08 | 20.58 | 0.94 | 29.72 | 1,543,672 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.72 | 25.60 | 1.07 | 157.35 | 1,721,095 |
| flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.84 | 26.07 | 1.09 | 158.33 | 1,700,413 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | 3 | 100,000 | 99,840 | 160 | 0 | 99,840 | 100,160 | 0.25 | 3.67 | 1.04 | 16.99 | 18,700 |
| flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | 3 | 100,000 | 99,840 | 160 | 0 | 99,840 | 100,160 | 0.24 | 3.68 | 0.99 | 17.85 | 18,699 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.89 | 20.80 | 0.98 | 147.63 | 1,514,233 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.86 | 21.30 | 1.00 | 148.39 | 1,540,089 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | 3 | 100,000 | 99,840 | 160 | 0 | 99,840 | 100,160 | 0.23 | 20.51 | 0.99 | 29.26 | 1,532,607 |
| flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | 3 | 100,000 | 99,840 | 160 | 0 | 99,840 | 100,160 | 0.23 | 20.49 | 0.99 | 29.27 | 1,529,535 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.88 | 21.16 | 0.95 | 148.86 | 1,524,025 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 117.88 | 21.06 | 0.96 | 149.05 | 1,533,817 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | 3 | 100,000 | 0 | 100,000 | 0 | 0 | 200,000 | 118.10 | 20.86 | 0.97 | 148.96 | 1,521,720 |
| flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | 3 | 100,000 | 0 | 160 | 99,840 | 0 | 100,160 | 0.22 | 20.47 | 0.97 | 29.82 | 1,542,840 |

### Publication commit latency after K unrelated commits

`wall = commit_ms` of the publication only. Conflict checks read every transaction since the read version.

| k | base n | base med ms | base p95 | proto n | proto med ms | proto p95 | flags n | flags med ms | flags p95 | flags / proto (med) |
|---|---|---|---|---|---|---|---|---|---|---|
| k=0 | 3 | 0.92 | - | 3 | 0.93 | - | 3 | 0.97 | - | 1.042 |
| k=1 | 3 | 0.94 | - | 3 | 0.87 | - | 3 | 0.82 | - | 0.940 |
| k=4 | 3 | 1.01 | - | 3 | 0.98 | - | 3 | 1.09 | - | 1.110 |
| k=16 | 3 | 1.22 | - | 3 | 1.27 | - | 3 | 1.23 | - | 0.963 |
| k=64 | 3 | 1.81 | - | 3 | 2.12 | - | 3 | 1.92 | - | 0.908 |

### Flag state size as the true set fragments

Head after one in-place `body` write invalidating the given fraction of scattered rows. `groups=0` is the unflagged control with the same data files. `wall` is a fresh-session open (OS page cache not controlled).

| variant | n | manifest B | txn B | flag state B (serialized true sets) | open med ms | open min | open max | open p95 | open r_iops | open read B |
|---|---|---|---|---|---|---|---|---|---|---|
| groups=0:invalidated=0pct | 3 | 2,782 | - | 0 | 0.10 | 0.09 | 0.11 | - | 1 | 2,782 |
| groups=0:invalidated=0.1pct | 3 | 7,045 | 3,368 | 0 | 0.10 | 0.10 | 0.11 | - | 1 | 4,096 |
| groups=0:invalidated=1pct | 3 | 7,045 | 3,368 | 0 | 0.09 | 0.07 | 0.11 | - | 1 | 4,096 |
| groups=0:invalidated=10pct | 3 | 7,046 | 3,368 | 0 | 0.10 | 0.10 | 0.11 | - | 1 | 4,096 |
| groups=1:invalidated=0pct | 3 | 2,903 | - | 84 | 0.14 | 0.13 | 0.15 | - | 1 | 2,903 |
| groups=1:invalidated=0.1pct | 3 | 8,592 | 4,241 | 634 | 0.11 | 0.11 | 0.14 | - | 3 | 8,427 |
| groups=1:invalidated=1pct | 3 | 15,768 | 7,861 | 4,190 | 0.13 | 0.11 | 0.16 | - | 3 | 11,983 |
| groups=1:invalidated=10pct | 3 | 83,880 | 43,865 | 36,294 | 0.12 | 0.11 | 0.13 | - | 3 | 44,091 |
| groups=2:invalidated=0pct | 3 | 3,013 | - | 168 | 0.11 | 0.10 | 0.13 | - | 1 | 3,013 |
| groups=2:invalidated=0.1pct | 3 | 9,705 | 4,693 | 1,268 | 0.13 | 0.12 | 0.14 | - | 3 | 9,088 |
| groups=2:invalidated=1pct | 3 | 22,237 | 10,113 | 8,380 | 0.12 | 0.10 | 0.13 | - | 3 | 16,200 |
| groups=2:invalidated=10pct | 3 | 140,456 | 64,119 | 72,588 | 0.15 | 0.15 | 0.16 | - | 3 | 80,413 |

### Every group (wall_ms)

One row per (build, workload, variant, cache) in the input.

| build | workload | variant | cache | n | med ms | min | max | p95 | udf med ms | stage med ms | commit med ms | outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | append | default | warm | 3 | 4.67 | 4.63 | 4.71 | - | - | 3.86 | 0.82 | - |
| baseline | update_sparse | default | warm | 3 | 5.61 | 5.49 | 5.68 | - | - | - | - | - |
| baseline | update_dense | default | warm | 3 | 7.73 | 7.58 | 8.02 | - | - | - | - | - |
| baseline | update_unrelated_sparse | default | warm | 3 | 5.61 | 5.44 | 5.99 | - | - | - | - | - |
| baseline | merge_insert_partial_sparse | default | warm | 3 | 13.98 | 13.08 | 14.18 | - | - | 13.10 | 0.89 | - |
| baseline | refresh_permissive_clean | default | warm | 3 | 159.17 | 158.49 | 159.59 | - | 125.69 | 23.49 | 1.11 | committed |
| baseline | refresh_permissive_conflicts_1 | update_row_moving | warm | 3 | 157.02 | 156.86 | 158.15 | - | 125.10 | 21.76 | 0.13 | conflict:retryable |
| baseline | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 3 | 164.13 | 163.14 | 164.61 | - | 124.97 | 21.94 | 0.92 | committed(stale=10) |
| baseline | refresh_permissive_conflicts_4 | update_row_moving | warm | 3 | 168.65 | 168.22 | 168.78 | - | 124.26 | 21.63 | 0.18 | conflict:retryable |
| baseline | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 3 | 190.07 | 189.91 | 192.27 | - | 124.73 | 21.94 | 1.01 | committed(stale=40) |
| baseline | refresh_permissive_conflicts_16 | update_row_moving | warm | 3 | 227.19 | 226.64 | 227.71 | - | 124.77 | 22.10 | 0.38 | conflict:retryable |
| baseline | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 3 | 339.94 | 339.93 | 341.16 | - | 124.19 | 21.86 | 1.21 | committed(stale=160) |
| baseline | publish_after_k_commits | k=0 | warm | 3 | 0.92 | 0.92 | 0.93 | - | - | - | 0.92 | committed |
| baseline | publish_after_k_commits | k=1 | warm | 3 | 0.94 | 0.91 | 0.95 | - | - | - | 0.94 | committed |
| baseline | publish_after_k_commits | k=4 | warm | 3 | 1.01 | 1.00 | 1.07 | - | - | - | 1.01 | committed |
| baseline | publish_after_k_commits | k=16 | warm | 3 | 1.22 | 1.12 | 1.23 | - | - | - | 1.22 | committed |
| baseline | publish_after_k_commits | k=64 | warm | 3 | 1.81 | 1.75 | 2.04 | - | - | - | 1.81 | committed |
| baseline | scan_summary_full | populated | warm | 3 | 0.74 | 0.63 | 0.76 | - | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 3 | 0.88 | 0.83 | 0.91 | - | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 3 | 0.73 | 0.73 | 0.73 | - | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 3 | 1.19 | 1.18 | 1.25 | - | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 3 | 0.75 | 0.74 | 0.85 | - | - | - | - | - |
| baseline | take_random_1k | populated | warm | 3 | 1.31 | 1.29 | 1.32 | - | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 3 | 0.91 | 0.78 | 0.92 | - | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 3 | 0.94 | 0.90 | 1.13 | - | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 3 | 0.91 | 0.81 | 0.93 | - | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 3 | 1.30 | 1.28 | 1.38 | - | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 3 | 1.07 | 0.98 | 1.12 | - | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 3 | 1.40 | 1.40 | 1.48 | - | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 3 | 0.80 | 0.78 | 0.85 | - | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 3 | 0.91 | 0.89 | 0.93 | - | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 3 | 0.77 | 0.70 | 0.82 | - | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 3 | 1.20 | 1.14 | 1.21 | - | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 3 | 0.80 | 0.80 | 0.91 | - | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 3 | 1.32 | 1.31 | 1.36 | - | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 3 | 0.92 | 0.83 | 0.93 | - | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 3 | 1.08 | 1.04 | 1.09 | - | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 3 | 0.95 | 0.88 | 0.97 | - | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 3 | 1.37 | 1.25 | 1.43 | - | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 3 | 1.02 | 0.99 | 1.07 | - | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 3 | 1.53 | 1.51 | 1.61 | - | - | - | - | - |
| prototype | append | default | warm | 3 | 4.72 | 4.63 | 4.73 | - | - | 3.92 | 0.77 | - |
| prototype | update_sparse | default | warm | 3 | 5.39 | 5.16 | 5.43 | - | - | - | - | - |
| prototype | update_dense | default | warm | 3 | 7.88 | 7.77 | 7.91 | - | - | - | - | - |
| prototype | update_unrelated_sparse | default | warm | 3 | 5.40 | 4.97 | 5.45 | - | - | - | - | - |
| prototype | merge_insert_partial_sparse | default | warm | 3 | 13.47 | 13.25 | 13.72 | - | - | 12.55 | 0.89 | - |
| prototype | refresh_permissive_clean | default | warm | 3 | 150.78 | 149.67 | 151.43 | - | 121.99 | 21.08 | 0.91 | committed |
| prototype | refresh_permissive_conflicts_1 | update_row_moving | warm | 3 | 153.52 | 152.93 | 154.12 | - | 121.83 | 21.31 | 0.14 | conflict:retryable |
| prototype | refresh_permissive_conflicts_1 | merge_insert_in_place | warm | 3 | 157.36 | 155.28 | 160.30 | - | 120.61 | 20.81 | 0.95 | committed(stale=10) |
| prototype | refresh_permissive_conflicts_4 | update_row_moving | warm | 3 | 164.99 | 164.78 | 165.03 | - | 121.08 | 20.98 | 0.21 | conflict:retryable |
| prototype | refresh_permissive_conflicts_4 | merge_insert_in_place | warm | 3 | 187.69 | 186.88 | 189.55 | - | 121.59 | 21.46 | 1.00 | committed(stale=40) |
| prototype | refresh_permissive_conflicts_16 | update_row_moving | warm | 3 | 223.53 | 223.35 | 224.07 | - | 121.60 | 21.60 | 0.37 | conflict:retryable |
| prototype | refresh_permissive_conflicts_16 | merge_insert_in_place | warm | 3 | 341.02 | 338.95 | 371.47 | - | 121.56 | 21.48 | 1.22 | committed(stale=160) |
| prototype | publish_after_k_commits | k=0 | warm | 3 | 0.93 | 0.91 | 1.01 | - | - | - | 0.93 | committed |
| prototype | publish_after_k_commits | k=1 | warm | 3 | 0.87 | 0.85 | 0.88 | - | - | - | 0.87 | committed |
| prototype | publish_after_k_commits | k=4 | warm | 3 | 0.98 | 0.97 | 1.05 | - | - | - | 0.98 | committed |
| prototype | publish_after_k_commits | k=16 | warm | 3 | 1.27 | 1.20 | 1.43 | - | - | - | 1.27 | committed |
| prototype | publish_after_k_commits | k=64 | warm | 3 | 2.12 | 1.80 | 2.20 | - | - | - | 2.12 | committed |
| prototype | scan_summary_full | populated | warm | 3 | 0.75 | 0.74 | 0.77 | - | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 3 | 0.79 | 0.77 | 0.81 | - | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 3 | 0.77 | 0.71 | 0.77 | - | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 3 | 1.20 | 1.10 | 1.22 | - | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 3 | 0.76 | 0.72 | 0.79 | - | - | - | - | - |
| prototype | take_random_1k | populated | warm | 3 | 1.28 | 1.27 | 1.33 | - | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 3 | 0.89 | 0.78 | 0.90 | - | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 3 | 0.94 | 0.87 | 0.97 | - | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 3 | 0.95 | 0.79 | 0.98 | - | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 3 | 1.36 | 1.22 | 1.41 | - | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 3 | 1.09 | 1.01 | 1.09 | - | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 3 | 1.50 | 1.46 | 1.54 | - | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 3 | 0.80 | 0.78 | 0.88 | - | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 3 | 0.89 | 0.87 | 0.93 | - | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 3 | 0.81 | 0.73 | 0.85 | - | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 3 | 1.26 | 1.19 | 1.29 | - | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 3 | 0.83 | 0.69 | 0.84 | - | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 3 | 1.35 | 1.32 | 1.36 | - | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 3 | 0.96 | 0.95 | 1.04 | - | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 3 | 1.10 | 1.08 | 1.14 | - | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 3 | 0.99 | 0.88 | 1.07 | - | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 3 | 1.48 | 1.29 | 1.53 | - | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 3 | 1.09 | 0.92 | 1.15 | - | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 3 | 1.49 | 1.48 | 1.50 | - | - | - | - | - |
| prototype-flags | flagged_update_sparse | groups=1 | warm | 3 | 6.03 | 5.95 | 6.12 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=1 | warm | 3 | 10.06 | 9.87 | 10.24 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=1 | warm | 3 | 5.85 | 5.71 | 6.15 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=1 | warm | 3 | 5.90 | 5.73 | 6.00 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=1 | warm | 3 | 13.51 | 12.99 | 13.63 | - | - | 12.49 | 0.98 | - |
| prototype-flags | flagged_refresh_clean | groups=1 | warm | 3 | 148.01 | 146.54 | 149.02 | - | 117.69 | 20.73 | 0.94 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:reject | warm | 3 | 157.51 | 152.56 | 165.45 | - | 118.40 | 23.16 | 0.16 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:recompute_all_pending | warm | 3 | 152.27 | 151.31 | 152.53 | - | 118.46 | 22.51 | 1.02 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:reject:reuse_valid_staged | warm | 3 | 151.44 | 151.16 | 151.97 | - | 118.15 | 22.32 | 1.00 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | update_row_moving:skip | warm | 3 | 155.88 | 155.15 | 156.37 | - | 118.47 | 22.67 | 1.19 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:recompute_all_pending | warm | 3 | 5.86 | 5.77 | 5.91 | - | 0.03 | 0.32 | 0.92 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | update_row_moving:skip:reuse_valid_staged | warm | 3 | 6.23 | 5.78 | 6.33 | - | 0.03 | 0.34 | 0.92 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:reject | warm | 3 | 160.18 | 154.88 | 169.26 | - | 118.01 | 23.77 | 0.15 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 3 | 147.84 | 147.51 | 160.81 | - | 117.69 | 20.84 | 0.94 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 3 | 147.69 | 147.32 | 150.35 | - | 117.60 | 20.65 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_in_place:skip | warm | 3 | 156.46 | 154.62 | 157.10 | - | 117.51 | 20.85 | 0.92 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 3 | 14.83 | 14.69 | 14.97 | - | 0.03 | 10.27 | 0.91 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 3 | 14.88 | 14.76 | 15.24 | - | 0.03 | 10.19 | 0.89 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:reject | warm | 3 | 152.12 | 151.92 | 156.11 | - | 118.02 | 20.84 | 0.14 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 3 | 148.90 | 148.87 | 151.65 | - | 118.16 | 21.11 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 3 | 148.52 | 146.99 | 149.59 | - | 117.71 | 20.83 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_1 | merge_insert_output_in_place:skip | warm | 3 | 153.06 | 152.75 | 153.40 | - | 118.16 | 20.90 | 0.91 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 3 | 74.96 | 74.75 | 75.07 | - | 58.98 | 10.56 | 0.91 | committed |
| prototype-flags | flagged_refresh_conflicts_1_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 3 | 15.44 | 15.31 | 15.65 | - | 0.03 | 10.35 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:reject | warm | 3 | 164.61 | 161.82 | 164.70 | - | 117.86 | 20.95 | 0.20 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:recompute_all_pending | warm | 3 | 151.73 | 151.00 | 152.12 | - | 117.76 | 22.75 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:reject:reuse_valid_staged | warm | 3 | 151.90 | 149.41 | 155.11 | - | 117.87 | 22.71 | 0.96 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | update_row_moving:skip | warm | 3 | 164.61 | 163.83 | 166.10 | - | 117.58 | 20.63 | 1.53 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:recompute_all_pending | warm | 3 | 10.92 | 10.83 | 11.13 | - | 0.08 | 0.92 | 0.91 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | update_row_moving:skip:reuse_valid_staged | warm | 3 | 11.26 | 11.14 | 11.26 | - | 0.08 | 0.94 | 0.86 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:reject | warm | 3 | 183.99 | 182.59 | 184.38 | - | 117.88 | 20.82 | 0.23 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 3 | 147.35 | 145.27 | 148.05 | - | 117.60 | 20.75 | 0.92 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 3 | 148.19 | 145.51 | 148.35 | - | 117.99 | 20.74 | 0.96 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_in_place:skip | warm | 3 | 184.70 | 184.64 | 189.65 | - | 118.00 | 20.88 | 1.08 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 3 | 28.96 | 28.92 | 29.28 | - | 0.08 | 20.61 | 1.01 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 3 | 28.92 | 28.41 | 29.06 | - | 0.08 | 20.51 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:reject | warm | 3 | 165.99 | 165.63 | 166.15 | - | 117.93 | 20.85 | 0.21 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 3 | 148.60 | 148.24 | 148.99 | - | 117.82 | 21.04 | 1.00 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 3 | 148.98 | 148.98 | 149.55 | - | 118.03 | 20.96 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_4 | merge_insert_output_in_place:skip | warm | 3 | 167.02 | 164.85 | 169.58 | - | 118.42 | 21.20 | 0.26 | deferred_all |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 3 | 148.44 | 148.36 | 149.48 | - | 117.66 | 20.99 | 0.93 | committed |
| prototype-flags | flagged_refresh_conflicts_4_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 3 | 29.72 | 29.71 | 32.78 | - | 0.08 | 20.58 | 0.94 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:reject | warm | 3 | 220.79 | 220.12 | 221.01 | - | 117.52 | 20.98 | 0.39 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:recompute_all_pending | warm | 3 | 157.35 | 154.51 | 157.62 | - | 117.72 | 25.60 | 1.07 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:reject:reuse_valid_staged | warm | 3 | 158.33 | 157.37 | 170.64 | - | 117.84 | 26.07 | 1.09 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | update_row_moving:skip | warm | 3 | 224.44 | 219.80 | 245.57 | - | 118.32 | 21.51 | 1.95 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:recompute_all_pending | warm | 3 | 16.99 | 16.13 | 17.19 | - | 0.25 | 3.67 | 1.04 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | update_row_moving:skip:reuse_valid_staged | warm | 3 | 17.85 | 16.44 | 17.96 | - | 0.24 | 3.68 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:reject | warm | 3 | 336.27 | 332.36 | 336.94 | - | 118.00 | 20.92 | 0.37 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:recompute_all_pending | warm | 3 | 147.63 | 147.44 | 147.70 | - | 117.89 | 20.80 | 0.98 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:reject:reuse_valid_staged | warm | 3 | 148.39 | 145.85 | 148.56 | - | 117.86 | 21.30 | 1.00 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_in_place:skip | warm | 3 | 336.20 | 336.01 | 336.75 | - | 117.96 | 21.15 | 1.37 | committed_partial |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:recompute_all_pending | warm | 3 | 29.26 | 28.38 | 29.43 | - | 0.23 | 20.51 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_in_place:skip:reuse_valid_staged | warm | 3 | 29.27 | 29.22 | 29.34 | - | 0.23 | 20.49 | 0.99 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:reject | warm | 3 | 227.18 | 222.78 | 227.62 | - | 117.85 | 20.70 | 0.44 | conflict:retryable |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:recompute_all_pending | warm | 3 | 148.86 | 147.74 | 155.01 | - | 117.88 | 21.16 | 0.95 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:reject:reuse_valid_staged | warm | 3 | 149.05 | 146.41 | 149.44 | - | 117.88 | 21.06 | 0.96 | committed |
| prototype-flags | flagged_refresh_conflicts_16 | merge_insert_output_in_place:skip | warm | 3 | 228.90 | 227.83 | 229.35 | - | 117.97 | 21.16 | 0.54 | deferred_all |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:recompute_all_pending | warm | 3 | 148.96 | 147.26 | 149.13 | - | 118.10 | 20.86 | 0.97 | committed |
| prototype-flags | flagged_refresh_conflicts_16_followup | merge_insert_output_in_place:skip:reuse_valid_staged | warm | 3 | 29.82 | 29.67 | 30.37 | - | 0.22 | 20.47 | 0.97 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=0 | warm | 3 | 0.97 | 0.93 | 1.00 | - | - | - | 0.97 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=1 | warm | 3 | 0.82 | 0.81 | 0.86 | - | - | - | 0.82 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=4 | warm | 3 | 1.09 | 1.06 | 1.20 | - | - | - | 1.09 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=16 | warm | 3 | 1.23 | 1.16 | 1.23 | - | - | - | 1.23 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=64 | warm | 3 | 1.92 | 1.92 | 2.14 | - | - | - | 1.92 | committed |
| prototype-flags | flagged_update_sparse | groups=2 | warm | 3 | 6.47 | 6.45 | 6.52 | - | - | - | - | - |
| prototype-flags | flagged_update_dense | groups=2 | warm | 3 | 10.62 | 10.28 | 10.84 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=2 | warm | 3 | 6.41 | 6.28 | 6.65 | - | - | - | - | - |
| prototype-flags | flagged_update_title_sparse | groups=2 | warm | 3 | 6.42 | 6.16 | 6.47 | - | - | - | - | - |
| prototype-flags | flagged_merge_insert_partial_sparse | groups=2 | warm | 3 | 13.16 | 12.96 | 13.24 | - | - | 12.14 | 1.02 | - |
| prototype-flags | flagged_refresh_clean | groups=2 | warm | 3 | 278.19 | 277.97 | 278.61 | - | 216.45 | 43.41 | 1.90 | committed |
| prototype-flags | scan_summary_full | all_true | warm | 3 | 0.69 | 0.67 | 0.82 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 3 | 0.80 | 0.79 | 0.85 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 3 | 0.73 | 0.73 | 0.77 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 3 | 1.21 | 1.12 | 1.29 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 3 | 0.81 | 0.81 | 0.82 | - | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 3 | 1.31 | 1.30 | 1.32 | - | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 3 | 0.87 | 0.85 | 0.90 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 3 | 1.06 | 1.01 | 1.18 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 3 | 0.90 | 0.82 | 0.94 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 3 | 1.38 | 1.28 | 1.48 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 3 | 1.01 | 1.00 | 1.04 | - | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 3 | 1.48 | 1.44 | 1.62 | - | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 3 | 1.11 | 0.96 | 1.15 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 3 | 1.25 | 1.12 | 1.40 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 3 | 1.15 | 0.92 | 1.32 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 3 | 1.66 | 1.64 | 1.78 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 3 | 0.78 | 0.74 | 0.83 | - | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 3 | 1.32 | 1.29 | 1.39 | - | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 3 | 1.37 | 1.24 | 1.38 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 3 | 1.49 | 1.32 | 1.53 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 3 | 1.22 | 1.16 | 1.34 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 3 | 1.63 | 1.61 | 1.79 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 3 | 1.01 | 0.99 | 1.11 | - | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 3 | 1.48 | 1.46 | 1.52 | - | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 3 | 0.16 | 0.14 | 0.18 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 3 | 0.29 | 0.28 | 0.30 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 3 | 0.15 | 0.14 | 0.16 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 3 | 0.55 | 0.49 | 0.59 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 3 | 0.75 | 0.69 | 0.76 | - | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 3 | 0.09 | 0.09 | 0.10 | - | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 3 | 0.14 | 0.14 | 0.14 | - | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 3 | 0.27 | 0.26 | 0.29 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 3 | 0.14 | 0.14 | 0.16 | - | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 3 | 0.55 | 0.54 | 0.58 | - | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 3 | 0.98 | 0.92 | 1.01 | - | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 3 | 0.08 | 0.08 | 0.09 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0pct | fresh-session | 3 | 0.10 | 0.09 | 0.11 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=0.1pct | fresh-session | 3 | 0.10 | 0.10 | 0.11 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=1pct | fresh-session | 3 | 0.09 | 0.07 | 0.11 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=0:invalidated=10pct | fresh-session | 3 | 0.10 | 0.10 | 0.11 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0pct | fresh-session | 3 | 0.14 | 0.13 | 0.15 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=0.1pct | fresh-session | 3 | 0.11 | 0.11 | 0.14 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=1pct | fresh-session | 3 | 0.13 | 0.11 | 0.16 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=1:invalidated=10pct | fresh-session | 3 | 0.12 | 0.11 | 0.13 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0pct | fresh-session | 3 | 0.11 | 0.10 | 0.13 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=0.1pct | fresh-session | 3 | 0.13 | 0.12 | 0.14 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=1pct | fresh-session | 3 | 0.12 | 0.10 | 0.13 | - | - | - | - | - |
| prototype-flags | flag_state_size | groups=2:invalidated=10pct | fresh-session | 3 | 0.15 | 0.15 | 0.16 | - | - | - | - | - |
