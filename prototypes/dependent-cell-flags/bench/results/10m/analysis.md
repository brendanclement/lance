## Results: 10m

Source: `results/10m`

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
| prototype_sha | `21601f894547b101d8966953c4e158ecd7d97523` |
| rounds | `1` |
| flag_every_round | `True` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k,refresh_permissive_clean,flagged_refresh_clean,update_sparse,flagged_update_sparse,update_unrelated_sparse,flagged_update_unrelated_sparse,publish_after_k_commits,flagged_publish_after_k_commits"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md; ?? prototypes/dependent-cell-flags/bench/results/1m/; ?? prototypes/dependent-cell-flags/bench/results/smoke-final/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T08:08:55Z	2026-09-25T08:14:53Z	2.73 2.41 2.39
1	prototype	2026-09-25T08:14:53Z	2026-09-25T08:20:51Z	3.83 2.70 2.49
1	prototype-flags	2026-09-25T08:20:51Z	2026-09-25T08:30:05Z	3.50 2.73 2.53
```

Records: 1761. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| update_sparse | default | warm | 3 | 71.36 | 70.72 | 73.54 | - | 3 | 66.60 | 66.05 | 67.34 | - | 0.933 | 0.933 | 17,383 | 17,373 | 6,803 | 6,798 | 494 | 494 | 73,248 | 73,233 | - / - |
| update_unrelated_sparse | default | warm | 3 | 73.41 | 72.15 | 74.28 | - | 3 | 67.92 | 67.70 | 67.96 | - | 0.925 | 0.925 | 17,372 | 17,374 | 6,797 | 6,798 | 494 | 494 | 80,036 | 80,039 | - / - |
| refresh_permissive_clean | default | warm | 3 | 12621.58 | 12598.98 | 12626.78 | - | 3 | 12576.37 | 12570.71 | 12610.00 | - | 0.996 | 0.996 | 24,801 | 24,801 | 7,842 | 7,842 | 201 | 201 | 150,412,832 | 150,638,497 | committed / committed |
| publish_after_k_commits | k=0 | warm | 3 | 1.21 | 1.20 | 1.48 | - | 3 | 1.23 | 1.22 | 1.23 | - | 1.011 | 1.011 | 24,801 | 24,801 | 7,842 | 7,842 | 1 | 1 | 32,657 | 32,657 | committed / committed |
| publish_after_k_commits | k=1 | warm | 3 | 1.07 | 1.06 | 1.08 | - | 3 | 1.11 | 1.09 | 1.17 | - | 1.034 | 1.034 | 24,892 | 24,892 | 7,842 | 7,842 | 2 | 2 | 32,748 | 32,748 | committed / committed |
| publish_after_k_commits | k=4 | warm | 3 | 1.17 | 1.15 | 1.19 | - | 3 | 1.20 | 1.14 | 1.22 | - | 1.027 | 1.027 | 25,165 | 25,165 | 7,842 | 7,842 | 5 | 5 | 33,021 | 33,021 | committed / committed |
| publish_after_k_commits | k=16 | warm | 3 | 1.33 | 1.32 | 1.39 | - | 3 | 1.39 | 1.36 | 1.43 | - | 1.039 | 1.039 | 26,274 | 26,275 | 7,842 | 7,842 | 17 | 17 | 34,131 | 34,132 | committed / committed |
| publish_after_k_commits | k=64 | warm | 3 | 2.00 | 1.90 | 2.12 | - | 3 | 2.04 | 2.02 | 2.08 | - | 1.022 | 1.022 | 30,693 | 30,693 | 7,843 | 7,843 | 65 | 65 | 38,551 | 38,551 | committed / committed |
| scan_summary_full | populated | warm | 20 | 22.16 | 21.76 | 22.65 | 22.64 | 20 | 24.17 | 23.71 | 24.94 | 24.50 | 1.091 | 1.091 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 20 | 24.21 | 23.47 | 24.85 | 24.73 | 20 | 26.05 | 25.78 | 26.83 | 26.68 | 1.076 | 1.076 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 20 | 22.51 | 22.08 | 23.25 | 22.82 | 20 | 24.50 | 24.06 | 25.37 | 25.09 | 1.088 | 1.088 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 20 | 31.64 | 30.75 | 33.05 | 32.62 | 20 | 33.99 | 33.08 | 34.88 | 34.33 | 1.074 | 1.074 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 20 | 12.35 | 11.87 | 12.62 | 12.54 | 20 | 12.34 | 12.11 | 12.71 | 12.66 | 0.999 | 0.999 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 20 | 11.67 | 10.62 | 12.17 | 12.07 | 20 | 10.58 | 10.17 | 11.09 | 10.87 | 0.906 | 0.906 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 20 | 22.52 | 21.99 | 23.03 | 22.98 | 20 | 24.70 | 24.26 | 25.26 | 25.14 | 1.097 | 1.097 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 20 | 24.62 | 24.14 | 24.97 | 24.93 | 20 | 27.04 | 26.34 | 27.69 | 27.37 | 1.098 | 1.098 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 20 | 22.78 | 22.32 | 23.17 | 23.11 | 20 | 25.08 | 23.91 | 26.19 | 25.51 | 1.101 | 1.101 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 20 | 31.53 | 30.12 | 32.44 | 32.32 | 20 | 33.99 | 33.31 | 35.79 | 34.57 | 1.078 | 1.078 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 20 | 12.90 | 12.60 | 13.42 | 13.03 | 20 | 13.00 | 12.66 | 13.13 | 13.11 | 1.008 | 1.008 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 20 | 12.70 | 12.22 | 13.58 | 13.22 | 20 | 12.41 | 11.94 | 12.97 | 12.93 | 0.977 | 0.977 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 20 | 24.59 | 24.05 | 25.22 | 25.05 | 20 | 25.88 | 25.34 | 26.88 | 26.66 | 1.053 | 1.053 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 20 | 28.50 | 27.98 | 29.41 | 29.16 | 20 | 28.88 | 27.67 | 29.51 | 29.38 | 1.013 | 1.013 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 20 | 24.82 | 24.36 | 26.11 | 25.89 | 20 | 25.95 | 25.14 | 27.58 | 26.43 | 1.045 | 1.045 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 20 | 35.13 | 33.84 | 36.16 | 35.92 | 20 | 36.19 | 35.49 | 37.41 | 36.77 | 1.030 | 1.030 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 20 | 12.06 | 11.78 | 12.25 | 12.25 | 20 | 12.23 | 11.91 | 12.68 | 12.42 | 1.014 | 1.014 | 24,799 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 20 | 10.90 | 10.56 | 11.44 | 11.13 | 20 | 10.95 | 10.35 | 11.44 | 11.15 | 1.004 | 1.004 | 24,799 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 20 | 24.94 | 24.42 | 25.43 | 25.20 | 20 | 26.15 | 25.78 | 26.73 | 26.54 | 1.049 | 1.049 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 20 | 28.50 | 27.76 | 29.03 | 28.97 | 20 | 30.28 | 29.06 | 31.69 | 31.66 | 1.062 | 1.062 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 20 | 25.03 | 24.44 | 26.47 | 25.66 | 20 | 27.61 | 26.13 | 29.06 | 28.36 | 1.103 | 1.103 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 20 | 34.42 | 33.89 | 35.34 | 35.12 | 20 | 36.38 | 35.22 | 41.18 | 40.51 | 1.057 | 1.057 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 20 | 12.93 | 12.82 | 13.20 | 13.17 | 20 | 13.09 | 12.83 | 13.45 | 13.24 | 1.012 | 1.012 | 24,799 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 20 | 12.94 | 12.37 | 13.50 | 13.41 | 20 | 13.05 | 12.26 | 14.20 | 13.62 | 1.008 | 1.008 | 24,799 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | warm | update_sparse:default | 3 | 66.60 | 66.05 | 67.34 | - | 3 | 76.68 | 76.31 | 77.12 | - | 1.151 | 1.151 | - | - | 17,373 | 34,193 | 6,798 | 13,326 | 73,233 | 99,489 |
| flagged_update_unrelated_sparse | groups=1 | warm | update_unrelated_sparse:default | 3 | 67.92 | 67.70 | 67.96 | - | 3 | 76.66 | 75.65 | 77.49 | - | 1.129 | 1.129 | - | - | 17,374 | 34,198 | 6,798 | 13,324 | 80,039 | 106,296 |
| flagged_refresh_clean | groups=1 | warm | refresh_permissive_clean:default | 3 | 12576.37 | 12570.71 | 12610.00 | - | 3 | 12650.08 | 12640.93 | 12658.05 | - | 1.006 | 1.006 | 1.25 | 1.28 | 24,801 | 26,461 | 7,842 | 8,659 | 150,638,497 | 150,023,181 |
| flagged_publish_after_k_commits | k=0 | warm | publish_after_k_commits:k=0 | 3 | 1.23 | 1.22 | 1.23 | - | 3 | 1.32 | 1.23 | 1.40 | - | 1.078 | 1.078 | 1.23 | 1.32 | 24,801 | 26,461 | 7,842 | 8,659 | 32,657 | 35,134 |
| flagged_publish_after_k_commits | k=1 | warm | publish_after_k_commits:k=1 | 3 | 1.11 | 1.09 | 1.17 | - | 3 | 1.11 | 1.09 | 1.12 | - | 1.002 | 1.002 | 1.11 | 1.11 | 24,892 | 26,552 | 7,842 | 8,659 | 32,748 | 35,225 |
| flagged_publish_after_k_commits | k=4 | warm | publish_after_k_commits:k=4 | 3 | 1.20 | 1.14 | 1.22 | - | 3 | 1.29 | 1.23 | 1.30 | - | 1.075 | 1.075 | 1.20 | 1.29 | 25,165 | 26,825 | 7,842 | 8,659 | 33,021 | 35,498 |
| flagged_publish_after_k_commits | k=16 | warm | publish_after_k_commits:k=16 | 3 | 1.39 | 1.36 | 1.43 | - | 3 | 1.42 | 1.38 | 1.50 | - | 1.023 | 1.023 | 1.39 | 1.42 | 26,275 | 27,935 | 7,842 | 8,659 | 34,132 | 36,609 |
| flagged_publish_after_k_commits | k=64 | warm | publish_after_k_commits:k=64 | 3 | 2.04 | 2.02 | 2.08 | - | 3 | 2.14 | 2.13 | 2.40 | - | 1.046 | 1.046 | 2.04 | 2.14 | 30,693 | 32,353 | 7,843 | 8,660 | 38,551 | 41,028 |
| flagged_update_sparse | groups=2 | warm | update_sparse:default | 3 | 66.60 | 66.05 | 67.34 | - | 3 | 79.17 | 78.57 | 79.79 | - | 1.189 | 1.189 | - | - | 17,373 | 48,959 | 6,798 | 17,912 | 73,233 | 121,300 |
| flagged_update_unrelated_sparse | groups=2 | warm | update_unrelated_sparse:default | 3 | 67.92 | 67.70 | 67.96 | - | 3 | 78.49 | 78.15 | 78.92 | - | 1.156 | 1.156 | - | - | 17,374 | 48,975 | 6,798 | 17,912 | 80,039 | 128,120 |
| scan_summary_full | all_true | warm | scan_summary_full:populated | 20 | 24.17 | 23.71 | 24.94 | 24.50 | 20 | 22.79 | 22.38 | 23.39 | 23.39 | 0.943 | 0.943 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 20 | 26.05 | 25.78 | 26.83 | 26.68 | 20 | 24.86 | 24.26 | 25.39 | 25.33 | 0.954 | 0.954 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 20 | 24.50 | 24.06 | 25.37 | 25.09 | 20 | 23.31 | 22.83 | 23.95 | 23.84 | 0.951 | 0.951 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 20 | 33.99 | 33.08 | 34.88 | 34.33 | 20 | 31.50 | 30.53 | 32.60 | 32.49 | 0.927 | 0.927 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 20 | 12.34 | 12.11 | 12.71 | 12.66 | 20 | 11.68 | 11.51 | 11.90 | 11.80 | 0.946 | 0.946 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 20 | 10.58 | 10.17 | 11.09 | 10.87 | 20 | 10.54 | 9.84 | 11.13 | 10.93 | 0.996 | 0.996 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 20 | 24.70 | 24.26 | 25.26 | 25.14 | 20 | 22.85 | 22.21 | 23.25 | 23.16 | 0.925 | 0.925 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 20 | 27.04 | 26.34 | 27.69 | 27.37 | 20 | 25.16 | 24.64 | 26.01 | 25.75 | 0.931 | 0.931 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 20 | 25.08 | 23.91 | 26.19 | 25.51 | 20 | 23.11 | 22.54 | 24.39 | 23.95 | 0.922 | 0.922 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 20 | 33.99 | 33.31 | 35.79 | 34.57 | 20 | 31.52 | 30.29 | 32.59 | 32.19 | 0.928 | 0.928 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 20 | 13.00 | 12.66 | 13.13 | 13.11 | 20 | 12.40 | 12.26 | 12.51 | 12.50 | 0.954 | 0.954 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 20 | 12.41 | 11.94 | 12.97 | 12.93 | 20 | 12.79 | 11.92 | 13.78 | 13.70 | 1.031 | 1.031 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 20 | 25.88 | 25.34 | 26.88 | 26.66 | 20 | 88.45 | 84.61 | 92.28 | 91.99 | 3.417 | 3.417 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 20 | 28.88 | 27.67 | 29.51 | 29.38 | 20 | 90.54 | 88.51 | 155.25 | 122.85 | 3.135 | 3.135 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 20 | 25.95 | 25.14 | 27.58 | 26.43 | 20 | 89.89 | 83.32 | 95.44 | 93.50 | 3.464 | 3.464 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 20 | 36.19 | 35.49 | 37.41 | 36.77 | 20 | 115.61 | 108.43 | 123.30 | 117.44 | 3.194 | 3.194 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 20 | 12.23 | 11.91 | 12.68 | 12.42 | 20 | 11.73 | 11.45 | 12.19 | 12.10 | 0.960 | 0.960 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 20 | 10.95 | 10.35 | 11.44 | 11.15 | 20 | 10.87 | 10.49 | 11.24 | 11.20 | 0.994 | 0.994 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 20 | 26.15 | 25.78 | 26.73 | 26.54 | 20 | 86.11 | 84.65 | 93.07 | 91.48 | 3.293 | 3.293 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 20 | 30.28 | 29.06 | 31.69 | 31.66 | 20 | 91.38 | 89.68 | 96.29 | 95.66 | 3.017 | 3.017 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 20 | 27.61 | 26.13 | 29.06 | 28.36 | 20 | 92.96 | 87.29 | 95.71 | 95.59 | 3.366 | 3.366 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 20 | 36.38 | 35.22 | 41.18 | 40.51 | 20 | 116.14 | 108.00 | 116.86 | 116.86 | 3.192 | 3.192 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 20 | 13.09 | 12.83 | 13.45 | 13.24 | 20 | 12.41 | 12.13 | 12.82 | 12.71 | 0.948 | 0.948 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 20 | 13.05 | 12.26 | 14.20 | 13.62 | 20 | 12.95 | 12.43 | 13.30 | 13.17 | 0.993 | 0.993 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| flagged_refresh_clean | groups=2 | warm | (no counterpart) | - | - | - | - | - | 3 | 23355.33 | 23347.03 | 23365.12 | - | - | - | - | 2.72 | - | 34,692 | - | 8,659 | - | 297,017,446 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.69 | 2.61 | 2.76 | 2.74 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.46 | 5.34 | 5.58 | 5.55 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.20 | 3.10 | 3.23 | 3.23 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 7.36 | 6.94 | 7.89 | 7.76 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 11.02 | 10.73 | 11.40 | 11.10 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.37 | 0.35 | 0.44 | 0.39 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.69 | 2.64 | 2.75 | 2.75 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.46 | 5.27 | 5.53 | 5.52 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.18 | 3.13 | 3.27 | 3.25 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 7.29 | 6.97 | 7.87 | 7.59 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 12.09 | 11.85 | 12.41 | 12.25 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.40 | 0.38 | 0.45 | 0.43 | - | - | - | - | - | 9,549 | - | 62 | - | 0 |

Flag effects of the source writes (asserted exact in every sample; first sample shown):

| workload | variant | rows written | flags cleared | flags carried | derived invalidation rows | moved rows |
|---|---|---|---|---|---|---|
| flagged_update_sparse | groups=1 | 100 | {"summary": 100} | {"summary": 0} | {"summary": 0} | 100 |
| flagged_update_unrelated_sparse | groups=1 | 100 | {"summary": 0} | {"summary": 100} | {"summary": 0} | 100 |
| flagged_update_sparse | groups=2 | 100 | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | {"summary": 0, "translation": 0} | 100 |
| flagged_update_unrelated_sparse | groups=2 | 100 | {"summary": 0, "translation": 0} | {"summary": 100, "translation": 100} | {"summary": 0, "translation": 0} | 100 |

### Refresh under concurrent source writes

A full `summary` refresh staged at V, K source commits of 10 scattered rows each, then the publication at V. `commit ms` is the conflict-checked publication commit. `merge_insert_output_in_place` writes `summary` itself (an output override); it is the only source write here that defers whole groups.

No flagged conflict records.

### Follow-up refresh to completion: saved and repeated computation

From the state the conflicted publication left, each strategy refreshes every pending row and publishes; afterwards every flag is asserted true and every value equal to the UDF of its current inputs. `recompute_all_pending` ignores the report; `reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject returns an error, so it has no report). `total UDF rows` = rows the conflicted publication computed + rows the follow-up recomputed; with N rows, anything above N is repeated computation.

No follow-up records.

### Publication commit latency after K unrelated commits

`wall = commit_ms` of the publication only. Conflict checks read every transaction since the read version.

| k | base n | base med ms | base p95 | proto n | proto med ms | proto p95 | flags n | flags med ms | flags p95 | flags / proto (med) |
|---|---|---|---|---|---|---|---|---|---|---|
| k=0 | 3 | 1.21 | - | 3 | 1.23 | - | 3 | 1.32 | - | 1.078 |
| k=1 | 3 | 1.07 | - | 3 | 1.11 | - | 3 | 1.11 | - | 1.002 |
| k=4 | 3 | 1.17 | - | 3 | 1.20 | - | 3 | 1.29 | - | 1.075 |
| k=16 | 3 | 1.33 | - | 3 | 1.39 | - | 3 | 1.42 | - | 1.023 |
| k=64 | 3 | 2.00 | - | 3 | 2.04 | - | 3 | 2.14 | - | 1.046 |

### Flag state size as the true set fragments

Head after one in-place `body` write invalidating the given fraction of scattered rows. `groups=0` is the unflagged control with the same data files. `wall` is a fresh-session open (OS page cache not controlled).

No flag_state_size records.

### Every group (wall_ms)

One row per (build, workload, variant, cache) in the input.

| build | workload | variant | cache | n | med ms | min | max | p95 | udf med ms | stage med ms | commit med ms | outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | update_sparse | default | warm | 3 | 71.36 | 70.72 | 73.54 | - | - | - | - | - |
| baseline | update_unrelated_sparse | default | warm | 3 | 73.41 | 72.15 | 74.28 | - | - | - | - | - |
| baseline | refresh_permissive_clean | default | warm | 3 | 12621.58 | 12598.98 | 12626.78 | - | 11861.69 | 601.35 | 1.22 | committed |
| baseline | publish_after_k_commits | k=0 | warm | 3 | 1.21 | 1.20 | 1.48 | - | - | - | 1.21 | committed |
| baseline | publish_after_k_commits | k=1 | warm | 3 | 1.07 | 1.06 | 1.08 | - | - | - | 1.07 | committed |
| baseline | publish_after_k_commits | k=4 | warm | 3 | 1.17 | 1.15 | 1.19 | - | - | - | 1.17 | committed |
| baseline | publish_after_k_commits | k=16 | warm | 3 | 1.33 | 1.32 | 1.39 | - | - | - | 1.33 | committed |
| baseline | publish_after_k_commits | k=64 | warm | 3 | 2.00 | 1.90 | 2.12 | - | - | - | 2.00 | committed |
| baseline | scan_summary_full | populated | warm | 20 | 22.16 | 21.76 | 22.65 | 22.64 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 20 | 24.21 | 23.47 | 24.85 | 24.73 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 20 | 22.51 | 22.08 | 23.25 | 22.82 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 20 | 31.64 | 30.75 | 33.05 | 32.62 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 20 | 12.35 | 11.87 | 12.62 | 12.54 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 20 | 11.67 | 10.62 | 12.17 | 12.07 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 20 | 22.52 | 21.99 | 23.03 | 22.98 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 20 | 24.62 | 24.14 | 24.97 | 24.93 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 20 | 22.78 | 22.32 | 23.17 | 23.11 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 20 | 31.53 | 30.12 | 32.44 | 32.32 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 20 | 12.90 | 12.60 | 13.42 | 13.03 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 20 | 12.70 | 12.22 | 13.58 | 13.22 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 20 | 24.59 | 24.05 | 25.22 | 25.05 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 20 | 28.50 | 27.98 | 29.41 | 29.16 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 20 | 24.82 | 24.36 | 26.11 | 25.89 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 20 | 35.13 | 33.84 | 36.16 | 35.92 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 20 | 12.06 | 11.78 | 12.25 | 12.25 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 20 | 10.90 | 10.56 | 11.44 | 11.13 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 20 | 24.94 | 24.42 | 25.43 | 25.20 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 20 | 28.50 | 27.76 | 29.03 | 28.97 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 20 | 25.03 | 24.44 | 26.47 | 25.66 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 20 | 34.42 | 33.89 | 35.34 | 35.12 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 20 | 12.93 | 12.82 | 13.20 | 13.17 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 20 | 12.94 | 12.37 | 13.50 | 13.41 | - | - | - | - |
| prototype | update_sparse | default | warm | 3 | 66.60 | 66.05 | 67.34 | - | - | - | - | - |
| prototype | update_unrelated_sparse | default | warm | 3 | 67.92 | 67.70 | 67.96 | - | - | - | - | - |
| prototype | refresh_permissive_clean | default | warm | 3 | 12576.37 | 12570.71 | 12610.00 | - | 11831.28 | 582.48 | 1.25 | committed |
| prototype | publish_after_k_commits | k=0 | warm | 3 | 1.23 | 1.22 | 1.23 | - | - | - | 1.23 | committed |
| prototype | publish_after_k_commits | k=1 | warm | 3 | 1.11 | 1.09 | 1.17 | - | - | - | 1.11 | committed |
| prototype | publish_after_k_commits | k=4 | warm | 3 | 1.20 | 1.14 | 1.22 | - | - | - | 1.20 | committed |
| prototype | publish_after_k_commits | k=16 | warm | 3 | 1.39 | 1.36 | 1.43 | - | - | - | 1.39 | committed |
| prototype | publish_after_k_commits | k=64 | warm | 3 | 2.04 | 2.02 | 2.08 | - | - | - | 2.04 | committed |
| prototype | scan_summary_full | populated | warm | 20 | 24.17 | 23.71 | 24.94 | 24.50 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 20 | 26.05 | 25.78 | 26.83 | 26.68 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 20 | 24.50 | 24.06 | 25.37 | 25.09 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 20 | 33.99 | 33.08 | 34.88 | 34.33 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 20 | 12.34 | 12.11 | 12.71 | 12.66 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 20 | 10.58 | 10.17 | 11.09 | 10.87 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 20 | 24.70 | 24.26 | 25.26 | 25.14 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 20 | 27.04 | 26.34 | 27.69 | 27.37 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 20 | 25.08 | 23.91 | 26.19 | 25.51 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 20 | 33.99 | 33.31 | 35.79 | 34.57 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 20 | 13.00 | 12.66 | 13.13 | 13.11 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 20 | 12.41 | 11.94 | 12.97 | 12.93 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 20 | 25.88 | 25.34 | 26.88 | 26.66 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 20 | 28.88 | 27.67 | 29.51 | 29.38 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 20 | 25.95 | 25.14 | 27.58 | 26.43 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 20 | 36.19 | 35.49 | 37.41 | 36.77 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 20 | 12.23 | 11.91 | 12.68 | 12.42 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 20 | 10.95 | 10.35 | 11.44 | 11.15 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 20 | 26.15 | 25.78 | 26.73 | 26.54 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 20 | 30.28 | 29.06 | 31.69 | 31.66 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 20 | 27.61 | 26.13 | 29.06 | 28.36 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 20 | 36.38 | 35.22 | 41.18 | 40.51 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 20 | 13.09 | 12.83 | 13.45 | 13.24 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 20 | 13.05 | 12.26 | 14.20 | 13.62 | - | - | - | - |
| prototype-flags | flagged_update_sparse | groups=1 | warm | 3 | 76.68 | 76.31 | 77.12 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=1 | warm | 3 | 76.66 | 75.65 | 77.49 | - | - | - | - | - |
| prototype-flags | flagged_refresh_clean | groups=1 | warm | 3 | 12650.08 | 12640.93 | 12658.05 | - | 11776.34 | 579.41 | 1.28 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=0 | warm | 3 | 1.32 | 1.23 | 1.40 | - | - | - | 1.32 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=1 | warm | 3 | 1.11 | 1.09 | 1.12 | - | - | - | 1.11 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=4 | warm | 3 | 1.29 | 1.23 | 1.30 | - | - | - | 1.29 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=16 | warm | 3 | 1.42 | 1.38 | 1.50 | - | - | - | 1.42 | committed |
| prototype-flags | flagged_publish_after_k_commits | k=64 | warm | 3 | 2.14 | 2.13 | 2.40 | - | - | - | 2.14 | committed |
| prototype-flags | flagged_update_sparse | groups=2 | warm | 3 | 79.17 | 78.57 | 79.79 | - | - | - | - | - |
| prototype-flags | flagged_update_unrelated_sparse | groups=2 | warm | 3 | 78.49 | 78.15 | 78.92 | - | - | - | - | - |
| prototype-flags | flagged_refresh_clean | groups=2 | warm | 3 | 23355.33 | 23347.03 | 23365.12 | - | 21613.46 | 1172.07 | 2.72 | committed |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 22.79 | 22.38 | 23.39 | 23.39 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 24.86 | 24.26 | 25.39 | 25.33 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 23.31 | 22.83 | 23.95 | 23.84 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 31.50 | 30.53 | 32.60 | 32.49 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 11.68 | 11.51 | 11.90 | 11.80 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.54 | 9.84 | 11.13 | 10.93 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 22.85 | 22.21 | 23.25 | 23.16 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 25.16 | 24.64 | 26.01 | 25.75 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 23.11 | 22.54 | 24.39 | 23.95 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 31.52 | 30.29 | 32.59 | 32.19 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 12.40 | 12.26 | 12.51 | 12.50 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.79 | 11.92 | 13.78 | 13.70 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 88.45 | 84.61 | 92.28 | 91.99 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 90.54 | 88.51 | 155.25 | 122.85 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 89.89 | 83.32 | 95.44 | 93.50 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 115.61 | 108.43 | 123.30 | 117.44 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 11.73 | 11.45 | 12.19 | 12.10 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.87 | 10.49 | 11.24 | 11.20 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 86.11 | 84.65 | 93.07 | 91.48 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 91.38 | 89.68 | 96.29 | 95.66 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 92.96 | 87.29 | 95.71 | 95.59 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 116.14 | 108.00 | 116.86 | 116.86 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 12.41 | 12.13 | 12.82 | 12.71 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.95 | 12.43 | 13.30 | 13.17 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.69 | 2.61 | 2.76 | 2.74 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.46 | 5.34 | 5.58 | 5.55 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.20 | 3.10 | 3.23 | 3.23 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 7.36 | 6.94 | 7.89 | 7.76 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 11.02 | 10.73 | 11.40 | 11.10 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.37 | 0.35 | 0.44 | 0.39 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.69 | 2.64 | 2.75 | 2.75 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.46 | 5.27 | 5.53 | 5.52 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.18 | 3.13 | 3.27 | 3.25 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 7.29 | 6.97 | 7.87 | 7.59 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 12.09 | 11.85 | 12.41 | 12.25 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.40 | 0.38 | 0.45 | 0.43 | - | - | - | - |
