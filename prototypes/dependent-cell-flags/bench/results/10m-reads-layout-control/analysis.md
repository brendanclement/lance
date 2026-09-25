## Control: 10m reads, baseline plus one unused function ("prototype" = the control build), rounds 1-3 baseline first, 4-6 control first

Source: `results/10m-reads-layout-control`

### Environment

| key | value |
|---|---|
| scale | `10m-reads-layout-control` |
| profile | `release-with-debug` |
| baseline_sha | `e3671b2f5730eea927a088a42cbf30e273edc43c` |
| prototype_sha | `e3671b2f5 + one unused function in rust/lance/src/dataset.rs (layout control; records labelled prototype)` |
| rounds | `6` |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T10:54:17Z	2026-09-25T10:55:00Z	21.14 11.73 8.12
1	prototype	2026-09-25T10:55:00Z	2026-09-25T10:55:45Z	13.34 10.85 7.96
2	baseline	2026-09-25T10:55:45Z	2026-09-25T10:56:27Z	8.21 9.81 7.72
2	prototype	2026-09-25T10:56:27Z	2026-09-25T10:57:10Z	6.46 9.07 7.55
3	baseline	2026-09-25T10:57:10Z	2026-09-25T10:57:52Z	5.52 8.48 7.40
3	prototype	2026-09-25T10:57:52Z	2026-09-25T10:58:36Z	6.10 8.20 7.35
4	prototype	2026-09-25T10:58:36Z	2026-09-25T10:59:19Z	7.92 8.41 7.47
4	baseline	2026-09-25T10:59:19Z	2026-09-25T11:00:02Z	6.13 7.89 7.32
5	prototype	2026-09-25T11:00:02Z	2026-09-25T11:00:45Z	5.07 7.36 7.15
5	baseline	2026-09-25T11:00:45Z	2026-09-25T11:01:30Z	4.82 6.97 7.02
6	prototype	2026-09-25T11:01:30Z	2026-09-25T11:02:15Z	5.49 6.81 6.96
6	baseline	2026-09-25T11:02:16Z	2026-09-25T11:03:00Z	5.71 6.66 6.89
```

Records: 5760. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 120 | 23.52 | 22.33 | 25.89 | 25.38 | 120 | 24.23 | 22.77 | 25.27 | 25.12 | 1.004 | 1.007 1.000 1.069 0.980 0.960 1.040 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 120 | 25.78 | 24.54 | 27.82 | 27.29 | 120 | 26.77 | 25.05 | 29.30 | 27.71 | 1.024 | 1.025 1.007 1.067 1.022 0.987 1.051 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 120 | 24.06 | 22.32 | 25.80 | 25.56 | 120 | 24.80 | 23.24 | 26.25 | 25.73 | 1.019 | 1.009 1.021 1.074 1.018 0.974 1.056 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 120 | 33.30 | 31.89 | 35.97 | 34.92 | 120 | 33.91 | 32.32 | 35.81 | 35.15 | 1.027 | 1.016 1.039 1.060 0.991 0.965 1.044 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 120 | 13.07 | 12.57 | 13.57 | 13.42 | 120 | 13.28 | 12.26 | 14.18 | 13.77 | 1.011 | 1.025 0.987 1.058 0.970 0.996 1.027 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 120 | 10.52 | 9.74 | 12.04 | 11.12 | 120 | 10.66 | 10.01 | 11.81 | 11.36 | 1.008 | 1.009 0.996 1.007 1.046 0.995 1.040 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 120 | 23.80 | 22.66 | 26.39 | 25.37 | 120 | 24.47 | 23.16 | 26.24 | 25.42 | 1.016 | 1.020 1.012 1.073 0.998 0.962 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 120 | 26.62 | 25.03 | 28.78 | 28.07 | 120 | 26.91 | 25.65 | 27.98 | 27.80 | 1.007 | 1.042 1.006 1.066 1.009 0.947 0.999 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 120 | 24.54 | 22.87 | 26.86 | 25.63 | 120 | 24.66 | 23.57 | 26.21 | 25.64 | 1.010 | 1.021 1.014 1.045 0.986 0.979 1.006 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 120 | 33.47 | 31.34 | 36.52 | 34.96 | 120 | 33.31 | 31.68 | 35.68 | 34.77 | 0.992 | 0.999 1.000 1.046 0.973 0.973 0.986 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 120 | 13.99 | 13.18 | 15.27 | 14.83 | 120 | 13.72 | 13.25 | 14.82 | 14.63 | 0.976 | 1.019 0.989 1.076 0.963 0.933 0.941 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 120 | 12.71 | 12.09 | 13.92 | 13.35 | 120 | 12.52 | 11.69 | 13.07 | 12.88 | 0.985 | 0.975 0.995 1.004 1.001 0.960 0.971 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 120 | 26.18 | 24.85 | 34.34 | 27.29 | 120 | 26.56 | 25.22 | 28.86 | 28.47 | 1.014 | 1.026 0.997 1.038 1.002 0.989 1.081 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 120 | 29.63 | 28.08 | 31.12 | 30.95 | 120 | 30.53 | 29.21 | 33.26 | 32.79 | 1.035 | 1.052 1.006 1.061 1.018 0.989 1.105 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 120 | 26.35 | 25.02 | 29.60 | 27.64 | 120 | 27.28 | 25.77 | 30.96 | 28.68 | 1.021 | 1.035 1.006 1.073 1.006 1.001 1.069 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 120 | 36.26 | 34.23 | 39.47 | 37.63 | 120 | 36.84 | 35.42 | 38.88 | 38.12 | 1.028 | 1.026 1.031 1.035 0.980 1.010 1.037 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 120 | 13.19 | 12.52 | 14.01 | 13.72 | 120 | 12.98 | 12.15 | 13.83 | 13.69 | 0.983 | 1.018 0.992 1.077 0.929 0.957 0.974 | 24,799 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 120 | 10.78 | 9.96 | 12.94 | 11.16 | 120 | 10.78 | 9.69 | 12.30 | 11.75 | 1.006 | 0.989 1.037 1.033 0.992 0.990 1.020 | 24,799 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 120 | 26.26 | 25.04 | 29.66 | 27.62 | 120 | 27.24 | 25.81 | 30.02 | 28.88 | 1.035 | 1.049 1.012 1.067 1.000 1.021 1.087 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 120 | 30.54 | 28.79 | 32.01 | 31.48 | 120 | 31.46 | 29.54 | 34.44 | 33.63 | 1.036 | 1.053 0.995 1.066 1.018 1.007 1.090 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 120 | 27.24 | 25.43 | 28.76 | 28.38 | 120 | 28.05 | 26.37 | 31.01 | 29.51 | 1.032 | 1.041 0.984 1.080 0.996 1.024 1.077 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 120 | 36.18 | 34.50 | 38.05 | 37.42 | 120 | 36.90 | 35.29 | 42.83 | 38.76 | 1.030 | 1.043 1.022 1.038 1.001 0.979 1.045 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 120 | 14.01 | 13.31 | 14.75 | 14.39 | 120 | 14.02 | 13.14 | 15.05 | 14.77 | 1.001 | 1.006 0.997 1.067 0.935 0.959 1.039 | 24,799 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 120 | 12.71 | 11.97 | 13.47 | 13.13 | 120 | 12.61 | 11.97 | 14.19 | 13.50 | 0.999 | 0.970 1.000 1.001 0.997 0.969 1.036 | 24,799 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

No flag records.

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
| baseline | scan_summary_full | populated | warm | 120 | 23.52 | 22.33 | 25.89 | 25.38 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 120 | 25.78 | 24.54 | 27.82 | 27.29 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 120 | 24.06 | 22.32 | 25.80 | 25.56 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 120 | 33.30 | 31.89 | 35.97 | 34.92 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 120 | 13.07 | 12.57 | 13.57 | 13.42 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 120 | 10.52 | 9.74 | 12.04 | 11.12 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 120 | 23.80 | 22.66 | 26.39 | 25.37 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 120 | 26.62 | 25.03 | 28.78 | 28.07 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 120 | 24.54 | 22.87 | 26.86 | 25.63 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 120 | 33.47 | 31.34 | 36.52 | 34.96 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 120 | 13.99 | 13.18 | 15.27 | 14.83 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 120 | 12.71 | 12.09 | 13.92 | 13.35 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 120 | 26.18 | 24.85 | 34.34 | 27.29 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 120 | 29.63 | 28.08 | 31.12 | 30.95 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 120 | 26.35 | 25.02 | 29.60 | 27.64 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 120 | 36.26 | 34.23 | 39.47 | 37.63 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 120 | 13.19 | 12.52 | 14.01 | 13.72 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 120 | 10.78 | 9.96 | 12.94 | 11.16 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 120 | 26.26 | 25.04 | 29.66 | 27.62 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 120 | 30.54 | 28.79 | 32.01 | 31.48 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 120 | 27.24 | 25.43 | 28.76 | 28.38 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 120 | 36.18 | 34.50 | 38.05 | 37.42 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 120 | 14.01 | 13.31 | 14.75 | 14.39 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 120 | 12.71 | 11.97 | 13.47 | 13.13 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 120 | 24.23 | 22.77 | 25.27 | 25.12 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 120 | 26.77 | 25.05 | 29.30 | 27.71 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 120 | 24.80 | 23.24 | 26.25 | 25.73 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 120 | 33.91 | 32.32 | 35.81 | 35.15 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 120 | 13.28 | 12.26 | 14.18 | 13.77 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 120 | 10.66 | 10.01 | 11.81 | 11.36 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 120 | 24.47 | 23.16 | 26.24 | 25.42 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 120 | 26.91 | 25.65 | 27.98 | 27.80 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 120 | 24.66 | 23.57 | 26.21 | 25.64 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 120 | 33.31 | 31.68 | 35.68 | 34.77 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 120 | 13.72 | 13.25 | 14.82 | 14.63 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 120 | 12.52 | 11.69 | 13.07 | 12.88 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 120 | 26.56 | 25.22 | 28.86 | 28.47 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 120 | 30.53 | 29.21 | 33.26 | 32.79 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 120 | 27.28 | 25.77 | 30.96 | 28.68 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 120 | 36.84 | 35.42 | 38.88 | 38.12 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 120 | 12.98 | 12.15 | 13.83 | 13.69 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 120 | 10.78 | 9.69 | 12.30 | 11.75 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 120 | 27.24 | 25.81 | 30.02 | 28.88 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 120 | 31.46 | 29.54 | 34.44 | 33.63 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 120 | 28.05 | 26.37 | 31.01 | 29.51 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 120 | 36.90 | 35.29 | 42.83 | 38.76 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 120 | 14.02 | 13.14 | 15.05 | 14.77 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 120 | 12.61 | 11.97 | 14.19 | 13.50 | - | - | - | - |
