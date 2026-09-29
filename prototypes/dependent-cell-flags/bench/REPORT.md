# Dependent cell flags: benchmark report

The Summary is written by hand. Every section after it is generated: section `results:<run>` by
`analyze.py` from the raw samples in `results/<run>/`, with the headings listed in
`regenerate_report.sh`, which rebuilds them all. See `README.md` for the commands, the exact
invocation of each run and what each workload measures.

## Summary

Every number below names its run and was recomputed from that run's raw samples with
`analyze.py`'s `load`, `Groups` and `paired_ratios`. A ratio is the median of the per-round ratios
of medians; a range spans the workloads, variants and cache states it names.

Prototype measurements on one local machine (Apple M5 Pro, 18 cores, 48 GB, APFS SSD), profile
`release-with-debug`, baseline `e3671b2f5`. Prototype code by run:

- `21601f894`: `smoke-final`, `1m` (full matrix), `10m` (subset, one round).
- `26388225a`, the final benchmarked code, which differs from `21601f894` only in the mask builder
  (the later fix `25d3cf3e5` changes only what the report says about a deferred publication file
  carrying chained outputs, which no workload publishes, so nothing was re-run):
  `1m-clean` (full 1M matrix), `10m-reads-clean-baseline-first`, `10m-reads-clean-prototype-first`,
  and the read reruns `1m-reads-maskfix`, `10m-reads-maskfix` and `10m-reads-reversed`, which
  record `e48573011`, a results-only commit with the same code.
- `2fd300ac2`, the final code. For reads it differs from `26388225a` only in the fragment read
  funnels, which keep `main`'s `merge_overlays` call and apply masks through a plain function:
  `10m-reads-fix-baseline-first` and `10m-reads-fix-prototype-first`. The no-flag hardware-counter
  rotations are in `results/10m-noflag-investigation/`. They have no section here; their README
  has the tables.
- Controls: `10m-reads-variant-fragment-reverted` (`26388225a` with `rust/lance/src/dataset/fragment.rs`
  reverted to the baseline; the rest of the prototype, including the masked-index check in
  `index.rs` and the manifest's cell flag decoding, stays) and `10m-reads-layout-control` (the
  baseline plus one unused function).

`results/smoke` (`e63e41628`, before the final hardening) has no section here. These are not a
pass of #8655's benchmark acceptance criteria, and S3 was not measured.

**Which run answers which question**

| Question | Authoritative run | Other runs |
|---|---|---|
| Regression on tables without flags, 1M | `1m-clean` | `1m` |
| Regression on tables without flags, 10M reads | `results/10m-noflag-investigation/` (shared tables, 20-round rotations, hardware counters) | `10m-reads-fix-*` (the fix, paired harness); `10m-reads-clean-*` with the controls `10m-reads-layout-control` and `10m-reads-variant-fragment-reverted`; `10m`, `10m-reads-maskfix`, `10m-reads-reversed` |
| Flag invalidation, publication and conflict costs | `1m`, `10m` (flags vs no flags within one nested build) | `1m-clean` (one flag round, clean build) |
| Masked reads | `1m-reads-maskfix`, `10m-reads-clean-*`, `10m-reads-fix-*` (final mask builder) | `10m-reads-maskfix`, `10m-reads-reversed`, `1m-clean` (final builder); `1m`, `10m` (earlier builder) |

**Build-configuration caveat.** The prototype worktree sits under another Lance checkout, and cargo
merges the `.cargo/config.toml` of every parent directory, concatenating `rustflags`, so every
prototype binary built there received its (identical) rustflags twice. That changes the binaries'
hash (`cc60bdbcaa7919a6`, against `bff5b3f24f8697e9` for the baseline and every build outside a
checkout) and their code layout. The runs `smoke-final`, `1m`, `10m`, `1m-reads-maskfix`,
`10m-reads-maskfix` and `10m-reads-reversed` used such binaries for the prototype, so their
baseline-vs-prototype ratios mix the code change with a build difference. These are the *nested*
runs. The `*-clean` runs and both controls were built outside any other checkout, with rustflags
identical to the baseline (`run_paired.sh` now refuses builds whose rustflags differ). Whether the
nested build also moves flags-vs-no-flags ratios is not established. The one clean-build flag
round (`1m-clean`, round 3) measured sparse `UpdateBuilder` 1.02× / 1.08× (`1m`: 1.11× / 1.15×),
unrelated-field updates 0.98× / 1.11× (`1m`: 1.16× / 1.20×), dense updates 1.41× / 1.39× (`1m`:
1.53× / 1.48×), all-true masked reads 0.88–1.01× and full-column reads with 1% invalidated
1.66–1.89× (`1m-reads-maskfix`: 0.93–1.00× and 1.53–1.77×).

**Findings**

- **Tables without flags, 1M rows** (`1m-clean`, 3 rounds): every workload 0.96–1.04× baseline
  (dense `UpdateBuilder` 1.04×, everything else 0.96–1.03×; full-column reads 0.96–1.03×).
- **Tables without flags, 10M rows, full-column reads**: a regression narrowed down but unresolved,
  and performance acceptance open. `results/10m-noflag-investigation/README.md` has the runs, their
  limitations and the representative comparison that is still needed.
  - Method: shared tables, 20-round Latin-square rotations, and the process's retired instructions
    and cycles per sample.
  - Every comparison retired nearly equal instructions, within 0.3%. Memory, cache,
    synchronization and scheduling costs were not measured.
  - In `release-with-debug`, the prototype measured 3.2–4.4% slower than `main`, with fewer cores
    busy. Narrowly scoped variants tied this to the `async fn resolve_cells` wrapper around
    `merge_overlays` in both `FragmentReader` read funnels. The mechanism was not identified.
  - `2fd300ac2` restores `main`'s call. It measured 1.000–1.001× `main` in the run that measured the
    prototype at 1.032–1.038×, and 1.000–1.032× over four rotations.
  - Behavior-free edits give comparable spreads: `main` plus one unused `FragmentReader` field
    measured 1.014–1.021×, and the fix with `main`'s `fragment.rs` 1.056–1.064×.
  - In the local `release` profile (fat LTO, 1 codegen unit), one rotation put the prototype at
    0.997–1.025×, the fix at 1.032–1.044× and `main` plus the unused field at 1.024–1.032×.
  - Neither profile matches the Linux wheels (thin LTO, 1 codegen unit, `haswell`, `metrics`).
    `2fd300ac2` is not a proven performance fix; it stays for maintainability.
  - The paired harness overstated the regression. It writes a new table per process, and writes
    are not byte-identical. Its clean runs put the prototype at **1.06–1.09×**, the
    geometric mean of `10m-reads-clean-baseline-first` (1.05–1.07×) and
    `10m-reads-clean-prototype-first` (1.06–1.11×).
  - The fix's paired runs (`10m-reads-fix-*`) measured 1.00–1.07×, with round 1 favoring the fix
    and rounds 2–3 the baseline in both orders.
  - Selective reads (id range, take) measured 1.00–1.06× in the clean runs and 0.95–1.00× in the
    fix runs.
  - The earlier controls (`10m-reads-layout-control`, `10m-reads-variant-fragment-reverted`) and
    the sampling profile of the nested build (`results/10m-profile/`) did not identify the cause.
- **Flag costs, 1M** (`1m`, 3 rounds, flags vs no flags in one build): sparse `UpdateBuilder`
  1.11× (one output) / 1.15× (two sharing an input); unrelated-field update 1.16× / 1.20× (flag
  state moves with the rows); dense update 1.53× / 1.48× with manifest 2.8 KB → 336 KB / 502 KB;
  partial `merge_insert` 1.03× / 0.98×; clean refresh +0.8%; publication after 0–64 unrelated
  commits 1.00–1.07×. At 10M (`10m`, one round): sparse 1.15× / 1.19×, unrelated-field 1.13× /
  1.16×, clean refresh +0.6%, publication after K 1.00–1.08×. The clean-build flag round is in the
  caveat above.
- **Masked reads, final mask builder**: all flags true 0.93–1.04× (`1m-reads-maskfix` 0.93–1.00×,
  `10m-reads-clean-*` 0.94–1.04×). With 1% of rows invalidated (scattered), full-column reads cost
  1.53–1.77× a plain NULL column at 1M (`1m-reads-maskfix`) and 1.68–1.80× at 10M
  (`10m-reads-clean-*`); id-range filters and takes 0.91–1.00× (both). At `2fd300ac2`
  (`10m-reads-fix-*`, one flag round per order): all true 0.92–1.04×, 1% invalidated 1.60–1.81×. With the per-row builder at
  `21601f894`: all true 0.98–1.05× (`1m`) and 0.92–1.03× (`10m`), full-column reads with 1%
  invalidated 3.02–3.22× (`1m`) and 3.02–3.46× (`10m`).
- **Refresh under concurrent writes** (`1m`, K commits of 10 rows): `Skip` publishes 999,990 /
  999,960 / 999,840 rows for K = 1 / 4 / 16 and defers exactly the written rows; a concurrent write
  of the output defers whole fragments (5 / 9 / 10 of 10). `Reject` publishes nothing. The
  publication commit takes 0.51–1.80 ms under `Skip` and 0.14–0.46 ms for the rejected commits.
  The permissive baseline without flags is **not correctness-equivalent**: it publishes 10·K stale
  values after in-place writes and is rejected after row-moving writes.
- **Saved computation** (`1m` follow-ups): after `Reject`, this harness's follow-up recomputes every
  row (2,000,000 UDF rows in total, 1.26–1.29 s), because the rejected publication returns only an
  error, with no report of what stayed valid. `Reject` stops at the first conflict, so certifying
  the rest as reusable would need Lance to finish validating the history first: a separate API
  change, not just exposing the report `Skip` builds. After row-level deferral under `Skip`, or
  whole-fragment deferral with `reuse_valid_staged`, only the deferred rows are recomputed
  (1,000,010–1,000,160 in total, 16–85 ms). After whole-fragment deferral with
  `recompute_all_pending`, the deferred fragments are (1,500,000 / 1,900,000 / 2,000,000 in total
  for K = 1 / 4 / 16, 0.63–1.27 s); reuse saves up to 999,840 recomputations (K = 16: UDF 1.18 s →
  0.57 ms).
- **Flag state size**: with 1% of rows invalidated the manifest grows from 7 KB to 88 KB (one flag)
  / 148 KB (two) at 1M rows (`1m`) and is 858 KB for the one-flag read table at 10M rows (`10m`);
  fresh-session opens go from 0.09 ms to 0.13–0.15 ms (`1m`, 1% and 10% invalidated).

<!-- BEGIN results:smoke-final -->
## Smoke: 100k rows, prototype `21601f894` (nested build)

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
<!-- END results:smoke-final -->

<!-- BEGIN results:1m -->
## 1M full matrix, prototype `21601f894` (nested build)

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
<!-- END results:1m -->

<!-- BEGIN results:10m -->
## 10M reads, clean refresh, sparse updates and publish after K, prototype `21601f894` (nested build), one round

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
<!-- END results:10m -->

<!-- BEGIN results:1m-reads-maskfix -->
## 1M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build)

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
<!-- END results:1m-reads-maskfix -->

<!-- BEGIN results:10m-reads-maskfix -->
## 10M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build), baseline first

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
<!-- END results:10m-reads-maskfix -->

<!-- BEGIN results:10m-reads-reversed -->
## 10M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build), prototype first, flags in round 3 only

Source: `results/10m-reads-reversed`

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
| flag_every_round | `False` |
| run_nice | `0` |
| config | `{"BENCH_SCALE_ROWS": "10000000", "BENCH_ROWS_PER_FRAGMENT": "100000", "BENCH_SAMPLES": "3", "BENCH_READ_SAMPLES": "20", "BENCH_WARMUP": "1", "BENCH_WORKLOADS": "scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k"}` |
| baseline_status | dirty: M rust/lance/Cargo.toml; ?? prototypes-bench-notes.md; ?? rust/lance/benches/cell_flags_common/; ?? rust/lance/benches/cell_flags_regression.rs |
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M prototypes/dependent-cell-flags/bench/run_paired.sh; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-maskfix/; ?? prototypes/dependent-cell-flags/bench/results/1m-reads-maskfix/ |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	prototype	2026-09-25T08:59:53Z	2026-09-25T09:00:37Z	3.32 6.63 7.37
1	baseline	2026-09-25T09:00:37Z	2026-09-25T09:01:19Z	3.83 6.26 7.19
2	prototype	2026-09-25T09:01:19Z	2026-09-25T09:02:02Z	4.69 6.16 7.11
2	baseline	2026-09-25T09:02:02Z	2026-09-25T09:02:45Z	5.04 5.98 6.99
3	prototype	2026-09-25T09:02:45Z	2026-09-25T09:03:29Z	4.21 5.64 6.82
3	baseline	2026-09-25T09:03:29Z	2026-09-25T09:04:12Z	7.34 6.22 6.97
3	prototype-flags	2026-09-25T09:04:12Z	2026-09-25T09:05:04Z	4.80 5.69 6.73
```

Records: 3600. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.85 | 23.07 | 25.09 | 24.69 | 60 | 25.19 | 23.43 | 26.59 | 26.31 | 1.057 | 1.082 1.017 1.057 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 26.16 | 25.08 | 37.61 | 28.65 | 60 | 27.37 | 25.47 | 28.67 | 28.35 | 1.025 | 1.069 1.021 1.025 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.10 | 23.20 | 24.91 | 24.67 | 60 | 25.44 | 24.11 | 27.29 | 26.92 | 1.047 | 1.096 1.031 1.047 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.18 | 31.59 | 34.37 | 33.96 | 60 | 34.69 | 33.37 | 36.17 | 35.75 | 1.041 | 1.064 1.032 1.041 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.70 | 12.33 | 12.99 | 12.89 | 60 | 13.21 | 12.78 | 13.84 | 13.72 | 1.038 | 1.082 1.021 1.038 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.61 | 10.05 | 12.11 | 11.59 | 60 | 10.66 | 10.10 | 11.18 | 10.99 | 0.999 | 0.999 0.983 1.013 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 23.92 | 23.17 | 24.81 | 24.62 | 60 | 25.11 | 23.69 | 26.67 | 26.42 | 1.064 | 1.073 1.007 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.30 | 25.07 | 27.25 | 26.93 | 60 | 27.51 | 25.99 | 29.48 | 29.19 | 1.059 | 1.087 1.004 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.43 | 23.58 | 25.13 | 24.92 | 60 | 25.45 | 24.18 | 27.83 | 27.28 | 1.046 | 1.089 1.018 1.046 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.56 | 32.24 | 35.61 | 34.59 | 60 | 35.11 | 32.72 | 37.24 | 36.44 | 1.038 | 1.079 1.013 1.038 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 12.91 | 13.84 | 13.80 | 60 | 14.03 | 13.58 | 14.77 | 14.64 | 1.059 | 1.082 1.018 1.059 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.50 | 11.80 | 13.53 | 12.96 | 60 | 12.65 | 11.84 | 13.77 | 13.31 | 1.007 | 1.007 1.002 1.030 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.23 | 25.40 | 26.93 | 26.72 | 60 | 26.93 | 25.63 | 28.65 | 28.53 | 1.033 | 1.068 1.003 1.033 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.51 | 27.96 | 30.72 | 30.24 | 60 | 31.06 | 29.48 | 33.21 | 32.62 | 1.064 | 1.077 1.025 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.19 | 25.46 | 27.75 | 27.39 | 60 | 27.65 | 26.47 | 28.73 | 28.66 | 1.050 | 1.050 1.040 1.066 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 35.97 | 34.71 | 37.51 | 36.91 | 60 | 37.67 | 35.24 | 39.58 | 38.92 | 1.051 | 1.064 1.035 1.051 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.63 | 12.21 | 13.16 | 13.10 | 60 | 13.20 | 12.82 | 13.90 | 13.76 | 1.052 | 1.078 1.001 1.052 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.79 | 9.91 | 11.43 | 11.26 | 60 | 10.80 | 10.27 | 11.31 | 11.29 | 0.993 | 0.993 0.989 1.017 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.44 | 25.53 | 29.11 | 27.71 | 60 | 27.65 | 26.49 | 31.16 | 28.85 | 1.052 | 1.052 1.031 1.053 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.24 | 29.35 | 32.52 | 30.97 | 60 | 31.85 | 30.52 | 33.52 | 33.11 | 1.061 | 1.079 1.013 1.061 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.20 | 25.95 | 30.30 | 28.70 | 60 | 28.08 | 26.87 | 29.58 | 29.46 | 1.053 | 1.062 0.998 1.053 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.01 | 34.73 | 40.11 | 39.06 | 60 | 37.76 | 35.68 | 39.48 | 38.77 | 1.021 | 1.003 1.021 1.060 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.56 | 13.23 | 14.24 | 13.95 | 60 | 14.05 | 13.66 | 14.77 | 14.65 | 1.044 | 1.070 1.010 1.044 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.69 | 11.67 | 13.91 | 13.27 | 60 | 12.78 | 12.29 | 13.25 | 13.19 | 1.007 | 1.007 1.019 0.988 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.19 | 23.43 | 26.59 | 26.31 | 20 | 25.33 | 24.71 | 25.84 | 25.72 | 1.006 | 1.006 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.37 | 25.47 | 28.67 | 28.35 | 20 | 27.17 | 26.38 | 28.34 | 27.77 | 0.992 | 0.992 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 25.44 | 24.11 | 27.29 | 26.92 | 20 | 24.76 | 24.35 | 25.29 | 25.29 | 0.974 | 0.974 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 34.69 | 33.37 | 36.17 | 35.75 | 20 | 33.65 | 33.05 | 34.34 | 34.19 | 0.970 | 0.970 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 13.21 | 12.78 | 13.84 | 13.72 | 20 | 13.57 | 13.39 | 13.71 | 13.69 | 1.032 | 1.032 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.66 | 10.10 | 11.18 | 10.99 | 20 | 10.63 | 10.31 | 11.10 | 11.08 | 0.990 | 0.990 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.11 | 23.69 | 26.67 | 26.42 | 20 | 25.09 | 24.48 | 25.63 | 25.54 | 0.999 | 0.999 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 27.51 | 25.99 | 29.48 | 29.19 | 20 | 27.65 | 27.02 | 28.34 | 28.10 | 1.005 | 1.005 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 25.45 | 24.18 | 27.83 | 27.28 | 20 | 25.06 | 24.31 | 26.56 | 25.68 | 0.985 | 0.985 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 35.11 | 32.72 | 37.24 | 36.44 | 20 | 33.93 | 32.55 | 34.82 | 34.75 | 0.968 | 0.968 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 14.03 | 13.58 | 14.77 | 14.64 | 20 | 14.54 | 14.22 | 14.73 | 14.71 | 1.041 | 1.041 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.65 | 11.84 | 13.77 | 13.31 | 20 | 12.52 | 12.22 | 12.99 | 12.70 | 0.976 | 0.976 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 26.93 | 25.63 | 28.65 | 28.53 | 20 | 47.48 | 44.02 | 51.55 | 51.21 | 1.766 | 1.766 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.06 | 29.48 | 33.21 | 32.62 | 20 | 52.73 | 45.75 | 54.64 | 54.34 | 1.698 | 1.698 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 27.65 | 26.47 | 28.73 | 28.66 | 20 | 46.90 | 40.77 | 48.65 | 47.95 | 1.697 | 1.697 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 37.67 | 35.24 | 39.58 | 38.92 | 20 | 66.87 | 60.45 | 67.79 | 67.62 | 1.777 | 1.777 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 13.20 | 12.82 | 13.90 | 13.76 | 20 | 13.06 | 12.69 | 13.29 | 13.22 | 0.992 | 0.992 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.80 | 10.27 | 11.31 | 11.29 | 20 | 10.65 | 10.06 | 11.06 | 11.02 | 0.979 | 0.979 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 27.65 | 26.49 | 31.16 | 28.85 | 20 | 47.85 | 43.35 | 50.04 | 49.78 | 1.736 | 1.736 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 31.85 | 30.52 | 33.52 | 33.11 | 20 | 53.25 | 46.59 | 54.68 | 54.44 | 1.678 | 1.678 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.08 | 26.87 | 29.58 | 29.46 | 20 | 47.61 | 45.42 | 49.59 | 48.81 | 1.698 | 1.698 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 37.76 | 35.68 | 39.48 | 38.77 | 20 | 67.26 | 61.23 | 69.49 | 68.50 | 1.781 | 1.781 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 14.05 | 13.66 | 14.77 | 14.65 | 20 | 13.78 | 13.48 | 14.00 | 13.99 | 0.984 | 0.984 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.78 | 12.29 | 13.25 | 13.19 | 20 | 12.58 | 12.34 | 13.39 | 13.13 | 0.990 | 0.990 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.70 | 2.65 | 2.77 | 2.75 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.53 | 5.41 | 5.81 | 5.74 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.26 | 3.21 | 3.33 | 3.32 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 7.08 | 6.97 | 7.59 | 7.37 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 12.36 | 11.84 | 12.54 | 12.47 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.37 | 0.36 | 0.40 | 0.39 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.71 | 2.67 | 2.76 | 2.73 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.53 | 5.41 | 5.66 | 5.63 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.29 | 3.22 | 3.38 | 3.34 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 7.08 | 6.99 | 7.46 | 7.34 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 13.16 | 12.86 | 13.38 | 13.30 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.40 | 0.38 | 0.43 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.85 | 23.07 | 25.09 | 24.69 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 26.16 | 25.08 | 37.61 | 28.65 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.10 | 23.20 | 24.91 | 24.67 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.18 | 31.59 | 34.37 | 33.96 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.70 | 12.33 | 12.99 | 12.89 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.61 | 10.05 | 12.11 | 11.59 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 23.92 | 23.17 | 24.81 | 24.62 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.30 | 25.07 | 27.25 | 26.93 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.43 | 23.58 | 25.13 | 24.92 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.56 | 32.24 | 35.61 | 34.59 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 12.91 | 13.84 | 13.80 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.50 | 11.80 | 13.53 | 12.96 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.23 | 25.40 | 26.93 | 26.72 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.51 | 27.96 | 30.72 | 30.24 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.19 | 25.46 | 27.75 | 27.39 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 35.97 | 34.71 | 37.51 | 36.91 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.63 | 12.21 | 13.16 | 13.10 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.79 | 9.91 | 11.43 | 11.26 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.44 | 25.53 | 29.11 | 27.71 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.24 | 29.35 | 32.52 | 30.97 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.20 | 25.95 | 30.30 | 28.70 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.01 | 34.73 | 40.11 | 39.06 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.56 | 13.23 | 14.24 | 13.95 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.69 | 11.67 | 13.91 | 13.27 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.19 | 23.43 | 26.59 | 26.31 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.37 | 25.47 | 28.67 | 28.35 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.44 | 24.11 | 27.29 | 26.92 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 34.69 | 33.37 | 36.17 | 35.75 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 13.21 | 12.78 | 13.84 | 13.72 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.66 | 10.10 | 11.18 | 10.99 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.11 | 23.69 | 26.67 | 26.42 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 27.51 | 25.99 | 29.48 | 29.19 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 25.45 | 24.18 | 27.83 | 27.28 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 35.11 | 32.72 | 37.24 | 36.44 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 14.03 | 13.58 | 14.77 | 14.64 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.65 | 11.84 | 13.77 | 13.31 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 26.93 | 25.63 | 28.65 | 28.53 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.06 | 29.48 | 33.21 | 32.62 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.65 | 26.47 | 28.73 | 28.66 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 37.67 | 35.24 | 39.58 | 38.92 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 13.20 | 12.82 | 13.90 | 13.76 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.80 | 10.27 | 11.31 | 11.29 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.65 | 26.49 | 31.16 | 28.85 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.85 | 30.52 | 33.52 | 33.11 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.08 | 26.87 | 29.58 | 29.46 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.76 | 35.68 | 39.48 | 38.77 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 14.05 | 13.66 | 14.77 | 14.65 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.78 | 12.29 | 13.25 | 13.19 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 25.33 | 24.71 | 25.84 | 25.72 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 27.17 | 26.38 | 28.34 | 27.77 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 24.76 | 24.35 | 25.29 | 25.29 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 33.65 | 33.05 | 34.34 | 34.19 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 13.57 | 13.39 | 13.71 | 13.69 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.63 | 10.31 | 11.10 | 11.08 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 25.09 | 24.48 | 25.63 | 25.54 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 27.65 | 27.02 | 28.34 | 28.10 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 25.06 | 24.31 | 26.56 | 25.68 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 33.93 | 32.55 | 34.82 | 34.75 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 14.54 | 14.22 | 14.73 | 14.71 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.52 | 12.22 | 12.99 | 12.70 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 47.48 | 44.02 | 51.55 | 51.21 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 52.73 | 45.75 | 54.64 | 54.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 46.90 | 40.77 | 48.65 | 47.95 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 66.87 | 60.45 | 67.79 | 67.62 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 13.06 | 12.69 | 13.29 | 13.22 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.65 | 10.06 | 11.06 | 11.02 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 47.85 | 43.35 | 50.04 | 49.78 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 53.25 | 46.59 | 54.68 | 54.44 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 47.61 | 45.42 | 49.59 | 48.81 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 67.26 | 61.23 | 69.49 | 68.50 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 13.78 | 13.48 | 14.00 | 13.99 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.58 | 12.34 | 13.39 | 13.13 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.70 | 2.65 | 2.77 | 2.75 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.53 | 5.41 | 5.81 | 5.74 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.26 | 3.21 | 3.33 | 3.32 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 7.08 | 6.97 | 7.59 | 7.37 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 12.36 | 11.84 | 12.54 | 12.47 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.37 | 0.36 | 0.40 | 0.39 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.71 | 2.67 | 2.76 | 2.73 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.53 | 5.41 | 5.66 | 5.63 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.29 | 3.22 | 3.38 | 3.34 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 7.08 | 6.99 | 7.46 | 7.34 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 13.16 | 12.86 | 13.38 | 13.30 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.40 | 0.38 | 0.43 | 0.42 | - | - | - | - |
<!-- END results:10m-reads-reversed -->

<!-- BEGIN results:10m-reads-variant-fragment-reverted -->
## Control: 10M reads, `26388225a` with `fragment.rs` reverted to the baseline (clean build, no flag runs), baseline first

Source: `results/10m-reads-variant-fragment-reverted`

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
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/run_paired.sh; M  rust/lance/src/dataset/fragment.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	baseline	2026-09-25T09:51:36Z	2026-09-25T09:52:19Z	20.90 16.81 11.45
1	prototype	2026-09-25T09:52:19Z	2026-09-25T09:53:04Z	12.60 15.14 11.11
2	baseline	2026-09-25T09:53:04Z	2026-09-25T09:53:47Z	8.12 13.57 10.74
2	prototype	2026-09-25T09:53:47Z	2026-09-25T09:54:30Z	6.57 12.47 10.46
3	baseline	2026-09-25T09:54:30Z	2026-09-25T09:55:13Z	6.65 11.61 10.24
3	prototype	2026-09-25T09:55:13Z	2026-09-25T09:55:57Z	5.27 10.50 9.90
```

Records: 2880. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 23.51 | 22.77 | 25.47 | 25.34 | 60 | 24.82 | 23.62 | 26.46 | 26.30 | 1.060 | 1.060 1.000 1.113 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 25.77 | 25.07 | 27.16 | 27.03 | 60 | 26.77 | 25.73 | 28.46 | 28.16 | 1.028 | 1.028 1.002 1.089 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.25 | 23.35 | 26.31 | 25.31 | 60 | 25.16 | 24.07 | 26.50 | 26.30 | 1.025 | 1.025 1.006 1.090 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 33.23 | 31.79 | 35.58 | 34.99 | 60 | 33.92 | 32.37 | 35.84 | 35.15 | 1.025 | 1.025 0.984 1.049 | 24,799 | 24,799 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 12.93 | 12.32 | 13.77 | 13.32 | 60 | 12.91 | 11.92 | 13.56 | 13.51 | 0.975 | 0.966 0.975 1.041 | 24,799 | 24,799 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.60 | 9.92 | 11.31 | 10.98 | 60 | 10.69 | 10.09 | 11.09 | 11.00 | 1.009 | 1.009 1.019 1.009 | 24,799 | 24,799 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 24.01 | 22.68 | 25.36 | 25.05 | 60 | 24.67 | 23.87 | 26.35 | 25.85 | 1.057 | 1.057 0.980 1.060 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 26.25 | 24.95 | 28.16 | 27.81 | 60 | 27.19 | 25.82 | 28.84 | 28.39 | 1.037 | 1.037 0.985 1.069 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.76 | 23.33 | 26.07 | 25.82 | 60 | 24.98 | 23.89 | 28.51 | 26.79 | 1.041 | 1.041 0.960 1.059 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 33.18 | 32.00 | 35.34 | 35.17 | 60 | 34.08 | 32.72 | 36.44 | 35.51 | 1.035 | 1.035 0.966 1.063 | 24,799 | 24,799 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.88 | 13.26 | 14.34 | 14.27 | 60 | 13.92 | 12.69 | 14.45 | 14.40 | 0.986 | 0.967 0.986 1.027 | 24,799 | 24,799 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.53 | 12.00 | 13.22 | 12.97 | 60 | 12.58 | 12.02 | 13.24 | 12.91 | 1.007 | 1.014 1.007 1.000 | 24,799 | 24,799 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.12 | 24.75 | 27.28 | 26.98 | 60 | 26.71 | 25.97 | 27.81 | 27.63 | 1.045 | 1.059 0.989 1.045 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 29.50 | 28.31 | 31.19 | 30.57 | 60 | 30.19 | 28.85 | 31.82 | 31.41 | 1.034 | 1.034 0.989 1.049 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.51 | 25.50 | 27.99 | 27.64 | 60 | 27.12 | 26.35 | 28.04 | 27.85 | 1.039 | 1.041 0.979 1.039 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 36.41 | 34.97 | 38.75 | 37.82 | 60 | 36.97 | 35.17 | 38.62 | 37.99 | 1.032 | 1.039 0.972 1.032 | 24,799 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.32 | 17.03 | 14.30 | 60 | 12.49 | 11.87 | 13.28 | 13.22 | 0.960 | 0.960 0.908 1.009 | 24,799 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.75 | 10.05 | 12.15 | 11.25 | 60 | 10.92 | 10.35 | 12.47 | 11.32 | 1.026 | 1.026 1.034 1.002 | 24,799 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.58 | 25.29 | 29.19 | 27.63 | 60 | 27.13 | 26.00 | 30.41 | 28.74 | 1.047 | 1.047 0.988 1.053 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.46 | 29.10 | 31.55 | 31.41 | 60 | 31.26 | 29.76 | 32.62 | 32.41 | 1.047 | 1.047 0.989 1.048 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.37 | 26.01 | 28.89 | 28.33 | 60 | 27.78 | 26.62 | 31.48 | 29.55 | 1.021 | 1.055 0.992 1.021 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.15 | 34.49 | 37.85 | 37.57 | 60 | 37.04 | 35.37 | 40.98 | 38.37 | 1.032 | 1.053 0.985 1.032 | 24,799 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.32 | 16.15 | 14.38 | 60 | 13.24 | 12.82 | 14.34 | 14.33 | 0.960 | 0.960 0.925 1.029 | 24,799 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.79 | 12.20 | 13.36 | 13.22 | 60 | 12.82 | 12.25 | 13.75 | 13.43 | 1.005 | 1.005 1.000 1.011 | 24,799 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

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
| baseline | scan_summary_full | populated | warm | 60 | 23.51 | 22.77 | 25.47 | 25.34 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 25.77 | 25.07 | 27.16 | 27.03 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.25 | 23.35 | 26.31 | 25.31 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 33.23 | 31.79 | 35.58 | 34.99 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 12.93 | 12.32 | 13.77 | 13.32 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.60 | 9.92 | 11.31 | 10.98 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 24.01 | 22.68 | 25.36 | 25.05 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 26.25 | 24.95 | 28.16 | 27.81 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.76 | 23.33 | 26.07 | 25.82 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 33.18 | 32.00 | 35.34 | 35.17 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.88 | 13.26 | 14.34 | 14.27 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.53 | 12.00 | 13.22 | 12.97 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.12 | 24.75 | 27.28 | 26.98 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 29.50 | 28.31 | 31.19 | 30.57 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.51 | 25.50 | 27.99 | 27.64 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.41 | 34.97 | 38.75 | 37.82 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.98 | 12.32 | 17.03 | 14.30 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.75 | 10.05 | 12.15 | 11.25 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.58 | 25.29 | 29.19 | 27.63 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 30.46 | 29.10 | 31.55 | 31.41 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.37 | 26.01 | 28.89 | 28.33 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.15 | 34.49 | 37.85 | 37.57 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.32 | 16.15 | 14.38 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.79 | 12.20 | 13.36 | 13.22 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 24.82 | 23.62 | 26.46 | 26.30 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 26.77 | 25.73 | 28.46 | 28.16 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 25.16 | 24.07 | 26.50 | 26.30 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 33.92 | 32.37 | 35.84 | 35.15 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 12.91 | 11.92 | 13.56 | 13.51 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.69 | 10.09 | 11.09 | 11.00 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 24.67 | 23.87 | 26.35 | 25.85 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 27.19 | 25.82 | 28.84 | 28.39 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.98 | 23.89 | 28.51 | 26.79 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.08 | 32.72 | 36.44 | 35.51 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 13.92 | 12.69 | 14.45 | 14.40 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.58 | 12.02 | 13.24 | 12.91 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 26.71 | 25.97 | 27.81 | 27.63 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 30.19 | 28.85 | 31.82 | 31.41 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 27.12 | 26.35 | 28.04 | 27.85 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 36.97 | 35.17 | 38.62 | 37.99 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 12.49 | 11.87 | 13.28 | 13.22 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.92 | 10.35 | 12.47 | 11.32 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 27.13 | 26.00 | 30.41 | 28.74 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.26 | 29.76 | 32.62 | 32.41 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.78 | 26.62 | 31.48 | 29.55 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.04 | 35.37 | 40.98 | 38.37 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.24 | 12.82 | 14.34 | 14.33 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.82 | 12.25 | 13.75 | 13.43 | - | - | - | - |
<!-- END results:10m-reads-variant-fragment-reverted -->

<!-- BEGIN results:10m-reads-clean-baseline-first -->
## Clean build: 10M reads at `26388225a`, baseline first

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
<!-- END results:10m-reads-clean-baseline-first -->

<!-- BEGIN results:10m-reads-clean-prototype-first -->
## Clean build: 10M reads at `26388225a`, prototype first

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
<!-- END results:10m-reads-clean-prototype-first -->

<!-- BEGIN results:1m-clean -->
## Clean build: 1M full matrix at `26388225a`, flags in round 3 only

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
<!-- END results:1m-clean -->

<!-- BEGIN results:10m-reads-layout-control -->
## Control: 10M reads, baseline plus one unused function (records labelled `prototype`), rounds 1-3 baseline first, 4-6 control first

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
<!-- END results:10m-reads-layout-control -->

<!-- BEGIN results:10m-reads-fix-baseline-first -->
## Clean build: 10M reads at `2fd300ac2`, baseline first, flags in round 3 only

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
<!-- END results:10m-reads-fix-baseline-first -->

<!-- BEGIN results:10m-reads-fix-prototype-first -->
## Clean build: 10M reads at `2fd300ac2`, prototype first, flags in round 3 only

Source: `results/10m-reads-fix-prototype-first`

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
| prototype_status | dirty: M prototypes/dependent-cell-flags/bench/REPORT.md;  M rust/lance/Cargo.toml; ?? prototypes/dependent-cell-flags/bench/results/10m-reads-fix-baseline-first/; ?? rust/lance/benches/cell_flags_scan_counters.rs |

Runs (load average before each run):

```
round	build	started	finished	loadavg_before
1	prototype	2026-09-26T05:29:05Z	2026-09-26T05:29:47Z	5.09 10.16 12.19
1	baseline	2026-09-26T05:29:47Z	2026-09-26T05:30:31Z	5.26 9.53 11.87
2	prototype	2026-09-26T05:30:31Z	2026-09-26T05:31:15Z	4.06 8.58 11.39
2	baseline	2026-09-26T05:31:15Z	2026-09-26T05:31:58Z	4.67 8.03 11.04
3	prototype	2026-09-26T05:31:58Z	2026-09-26T05:32:42Z	4.31 7.48 10.70
3	baseline	2026-09-26T05:32:42Z	2026-09-26T05:33:25Z	4.50 7.05 10.37
3	prototype-flags	2026-09-26T05:33:25Z	2026-09-26T05:34:17Z	4.85 6.75 10.09
```

Records: 3600. Simulated UDF iterations: [16]. `udf_ms` is the labeled simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.
`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS page cache is not controlled, so no state here is a cold read.

### Regression: tables without cell flags, baseline vs prototype

Same harness binary source on both builds. Ratio = prototype / baseline median wall_ms, per round (paired) and the median of those. Conflict workloads are **not correctness-equivalent** (see notes): compare latency and IO only.

| workload | variant | cache | base n | base med ms | base min | base max | base p95 | proto n | proto med ms | proto min | proto max | proto p95 | ratio (med) | ratio per round | base manifest B | proto manifest B | base txn B | proto txn B | base r_iops | proto r_iops | base written B | proto written B | outcome base / proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | populated | warm | 60 | 24.04 | 23.29 | 26.79 | 26.19 | 60 | 25.82 | 23.74 | 26.56 | 26.41 | 1.070 | 0.928 1.070 1.096 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | warm | 60 | 26.48 | 25.30 | 28.88 | 28.55 | 60 | 27.78 | 25.88 | 28.67 | 28.38 | 1.054 | 0.926 1.054 1.088 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | warm | 60 | 24.61 | 23.67 | 35.79 | 27.06 | 60 | 26.08 | 23.96 | 27.02 | 26.69 | 1.063 | 0.921 1.063 1.089 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | warm | 60 | 34.06 | 31.79 | 39.61 | 37.07 | 60 | 34.79 | 32.45 | 36.06 | 35.68 | 1.020 | 0.917 1.020 1.064 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | warm | 60 | 13.10 | 12.18 | 18.17 | 15.76 | 60 | 12.50 | 12.18 | 13.12 | 13.05 | 0.974 | 0.944 0.974 0.997 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | populated | warm | 60 | 10.64 | 10.19 | 11.09 | 11.02 | 60 | 10.57 | 9.91 | 11.36 | 10.90 | 0.988 | 0.988 0.983 0.998 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | populated | fresh-session | 60 | 24.45 | 23.02 | 26.58 | 26.31 | 60 | 25.78 | 23.81 | 27.25 | 26.70 | 1.060 | 0.926 1.060 1.097 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | populated | fresh-session | 60 | 27.15 | 25.77 | 29.42 | 29.28 | 60 | 28.20 | 25.99 | 29.06 | 28.98 | 1.043 | 0.911 1.043 1.085 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.95 | 23.99 | 33.89 | 27.28 | 60 | 26.08 | 24.01 | 29.01 | 27.15 | 1.046 | 0.925 1.046 1.102 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | populated:sql | fresh-session | 60 | 34.39 | 32.46 | 36.10 | 35.81 | 60 | 34.89 | 32.68 | 36.47 | 35.85 | 1.017 | 0.950 1.017 1.067 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | populated | fresh-session | 60 | 13.68 | 13.08 | 14.40 | 14.32 | 60 | 13.41 | 13.09 | 14.07 | 13.87 | 0.976 | 0.976 0.970 0.998 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | populated | fresh-session | 60 | 12.51 | 11.59 | 13.39 | 13.35 | 60 | 12.44 | 11.89 | 12.98 | 12.78 | 1.002 | 1.012 0.981 1.002 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | warm | 60 | 26.44 | 25.11 | 28.03 | 27.86 | 60 | 27.62 | 25.90 | 28.74 | 28.45 | 1.063 | 0.952 1.063 1.076 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | warm | 60 | 30.17 | 29.18 | 32.48 | 32.32 | 60 | 31.31 | 28.90 | 32.87 | 32.47 | 1.055 | 0.939 1.055 1.077 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.52 | 25.73 | 28.94 | 28.85 | 60 | 28.22 | 25.84 | 29.67 | 28.90 | 1.069 | 0.937 1.069 1.092 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | warm | 60 | 37.09 | 34.76 | 39.00 | 38.93 | 60 | 38.02 | 35.71 | 44.29 | 40.55 | 1.016 | 0.971 1.016 1.068 | 24,800 | 24,800 | 7,842 | 7,842 | 100 | 100 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | warm | 60 | 12.92 | 12.48 | 13.42 | 13.34 | 60 | 12.80 | 12.33 | 13.68 | 13.32 | 0.986 | 1.016 0.972 0.986 | 24,800 | 24,800 | 7,842 | 7,842 | 113 | 113 | 0 | 0 | - / - |
| take_random_1k | null_1pct | warm | 60 | 10.75 | 10.03 | 13.27 | 11.18 | 60 | 10.70 | 10.13 | 13.53 | 11.24 | 0.997 | 0.989 0.997 1.016 | 24,800 | 24,800 | 7,842 | 7,842 | 932 | 932 | 0 | 0 | - / - |
| scan_summary_full | null_1pct | fresh-session | 60 | 26.88 | 25.86 | 28.97 | 28.67 | 60 | 28.51 | 26.36 | 29.52 | 29.29 | 1.063 | 0.954 1.063 1.092 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.12 | 29.23 | 33.07 | 32.79 | 60 | 32.49 | 29.81 | 33.49 | 33.22 | 1.044 | 0.945 1.044 1.078 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.51 | 26.56 | 29.85 | 29.65 | 60 | 28.74 | 26.91 | 30.39 | 29.88 | 1.056 | 0.925 1.056 1.084 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.87 | 35.23 | 39.03 | 38.99 | 60 | 37.51 | 35.81 | 40.09 | 39.09 | 1.018 | 0.942 1.018 1.081 | 24,800 | 24,800 | 7,842 | 7,842 | 300 | 300 | 0 | 0 | - / - |
| filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.38 | 14.48 | 14.36 | 60 | 13.59 | 13.15 | 14.15 | 13.89 | 0.982 | 0.982 0.967 0.992 | 24,800 | 24,800 | 7,842 | 7,842 | 418 | 418 | 0 | 0 | - / - |
| take_random_1k | null_1pct | fresh-session | 60 | 12.67 | 12.16 | 13.24 | 13.00 | 60 | 12.72 | 12.10 | 13.89 | 13.58 | 1.003 | 1.003 0.991 1.043 | 24,800 | 24,800 | 7,842 | 7,842 | 1,132 | 1,132 | 0 | 0 | - / - |

† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).

### Flag overhead: prototype without flags vs prototype with dependent masking flags

Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by round). Updates run on a fully published table; `groups=2` adds a second output sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored NULLs) with `partial_1pct` (masked by false flags), on the same ids.

| flag workload | variant | cache | compared with | plain n | plain med ms | plain min | plain max | plain p95 | flags n | flags med ms | flags min | flags max | flags p95 | ratio (med) | ratio per round | plain commit ms | flags commit ms | plain manifest B | flags manifest B | plain txn B | flags txn B | plain written B | flags written B |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan_summary_full | all_true | warm | scan_summary_full:populated | 60 | 25.82 | 23.74 | 26.56 | 26.41 | 20 | 24.47 | 24.04 | 25.26 | 25.24 | 0.938 | 0.938 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | warm | filter_summary_is_null_count:populated | 60 | 27.78 | 25.88 | 28.67 | 28.38 | 20 | 26.62 | 26.13 | 27.28 | 27.24 | 0.948 | 0.948 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | warm | count_summary_vs_star:populated:aggregate | 60 | 26.08 | 23.96 | 27.02 | 26.69 | 20 | 24.74 | 24.32 | 25.30 | 25.19 | 0.937 | 0.937 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | warm | count_summary_vs_star:populated:sql | 60 | 34.79 | 32.45 | 36.06 | 35.68 | 20 | 33.04 | 32.36 | 33.94 | 33.70 | 0.939 | 0.939 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | warm | filter_id_range_project_summary:populated | 60 | 12.50 | 12.18 | 13.12 | 13.05 | 20 | 12.86 | 12.68 | 13.27 | 13.19 | 1.035 | 1.035 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | warm | take_random_1k:populated | 60 | 10.57 | 9.91 | 11.36 | 10.90 | 20 | 10.61 | 10.16 | 11.31 | 11.28 | 0.997 | 0.997 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | all_true | fresh-session | scan_summary_full:populated | 60 | 25.78 | 23.81 | 27.25 | 26.70 | 20 | 24.70 | 23.90 | 29.01 | 27.83 | 0.941 | 0.941 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_summary_is_null_count | all_true | fresh-session | filter_summary_is_null_count:populated | 60 | 28.20 | 25.99 | 29.06 | 28.98 | 20 | 27.18 | 26.35 | 27.69 | 27.53 | 0.949 | 0.949 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:aggregate | fresh-session | count_summary_vs_star:populated:aggregate | 60 | 26.08 | 24.01 | 29.01 | 27.15 | 20 | 25.19 | 24.57 | 25.52 | 25.48 | 0.943 | 0.943 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| count_summary_vs_star | all_true:sql | fresh-session | count_summary_vs_star:populated:sql | 60 | 34.89 | 32.68 | 36.47 | 35.85 | 20 | 33.62 | 32.66 | 34.25 | 34.23 | 0.949 | 0.949 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| filter_id_range_project_summary | all_true | fresh-session | filter_id_range_project_summary:populated | 60 | 13.41 | 13.09 | 14.07 | 13.87 | 20 | 13.82 | 13.60 | 14.17 | 13.97 | 1.035 | 1.035 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| take_random_1k | all_true | fresh-session | take_random_1k:populated | 60 | 12.44 | 11.89 | 12.98 | 12.78 | 20 | 12.57 | 12.07 | 13.00 | 12.90 | 1.004 | 1.004 | - | - | 24,800 | 26,460 | 7,842 | 8,659 | 0 | 0 |
| scan_summary_full | partial_1pct | warm | scan_summary_full:null_1pct | 60 | 27.62 | 25.90 | 28.74 | 28.45 | 20 | 47.52 | 42.69 | 48.87 | 48.21 | 1.690 | 1.690 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | warm | filter_summary_is_null_count:null_1pct | 60 | 31.31 | 28.90 | 32.87 | 32.47 | 20 | 52.08 | 49.38 | 60.80 | 53.43 | 1.632 | 1.632 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | warm | count_summary_vs_star:null_1pct:aggregate | 60 | 28.22 | 25.84 | 29.67 | 28.90 | 20 | 48.17 | 46.29 | 51.00 | 49.28 | 1.686 | 1.686 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | warm | count_summary_vs_star:null_1pct:sql | 60 | 38.02 | 35.71 | 44.29 | 40.55 | 20 | 65.38 | 59.89 | 66.40 | 66.02 | 1.697 | 1.697 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | warm | filter_id_range_project_summary:null_1pct | 60 | 12.80 | 12.33 | 13.68 | 13.32 | 20 | 12.68 | 12.40 | 12.93 | 12.93 | 1.014 | 1.014 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | warm | take_random_1k:null_1pct | 60 | 10.70 | 10.13 | 13.53 | 11.24 | 20 | 10.68 | 10.07 | 11.30 | 11.21 | 0.987 | 0.987 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | partial_1pct | fresh-session | scan_summary_full:null_1pct | 60 | 28.51 | 26.36 | 29.52 | 29.29 | 20 | 47.99 | 42.38 | 49.00 | 48.97 | 1.656 | 1.656 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_summary_is_null_count | partial_1pct | fresh-session | filter_summary_is_null_count:null_1pct | 60 | 32.49 | 29.81 | 33.49 | 33.22 | 20 | 52.62 | 45.18 | 54.54 | 53.95 | 1.602 | 1.602 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:aggregate | fresh-session | count_summary_vs_star:null_1pct:aggregate | 60 | 28.74 | 26.91 | 30.39 | 29.88 | 20 | 48.81 | 43.60 | 49.66 | 49.43 | 1.654 | 1.654 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| count_summary_vs_star | partial_1pct:sql | fresh-session | count_summary_vs_star:null_1pct:sql | 60 | 37.51 | 35.81 | 40.09 | 39.09 | 20 | 66.18 | 60.29 | 66.58 | 66.57 | 1.705 | 1.705 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| filter_id_range_project_summary | partial_1pct | fresh-session | filter_id_range_project_summary:null_1pct | 60 | 13.59 | 13.15 | 14.15 | 13.89 | 20 | 13.39 | 13.09 | 13.57 | 13.55 | 0.994 | 0.994 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| take_random_1k | partial_1pct | fresh-session | take_random_1k:null_1pct | 60 | 12.72 | 12.10 | 13.89 | 13.58 | 20 | 12.55 | 12.20 | 13.18 | 12.85 | 0.952 | 0.952 | - | - | 24,800 | 858,409 | 7,842 | 432,565 | 0 | 0 |
| scan_summary_full | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 2.78 | 2.74 | 2.83 | 2.83 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 5.48 | 5.41 | 5.60 | 5.53 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | warm | (no counterpart) | - | - | - | - | - | 20 | 3.20 | 3.15 | 3.25 | 3.23 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | warm | (no counterpart) | - | - | - | - | - | 20 | 6.83 | 6.79 | 6.99 | 6.94 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 11.97 | 11.84 | 12.12 | 12.12 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | warm | (no counterpart) | - | - | - | - | - | 20 | 0.38 | 0.37 | 0.41 | 0.40 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| scan_summary_full | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 2.77 | 2.73 | 2.83 | 2.81 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_summary_is_null_count | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 5.49 | 5.42 | 5.54 | 5.53 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:aggregate | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 3.22 | 3.15 | 3.28 | 3.27 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| count_summary_vs_star | all_pending:sql | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 6.89 | 6.77 | 7.25 | 7.19 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| filter_id_range_project_summary | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 12.68 | 12.43 | 12.89 | 12.85 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |
| take_random_1k | all_pending | fresh-session | (no counterpart) | - | - | - | - | - | 20 | 0.39 | 0.37 | 0.42 | 0.42 | - | - | - | - | - | 9,550 | - | 62 | - | 0 |

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
| baseline | scan_summary_full | populated | warm | 60 | 24.04 | 23.29 | 26.79 | 26.19 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | warm | 60 | 26.48 | 25.30 | 28.88 | 28.55 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | warm | 60 | 24.61 | 23.67 | 35.79 | 27.06 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | warm | 60 | 34.06 | 31.79 | 39.61 | 37.07 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | warm | 60 | 13.10 | 12.18 | 18.17 | 15.76 | - | - | - | - |
| baseline | take_random_1k | populated | warm | 60 | 10.64 | 10.19 | 11.09 | 11.02 | - | - | - | - |
| baseline | scan_summary_full | populated | fresh-session | 60 | 24.45 | 23.02 | 26.58 | 26.31 | - | - | - | - |
| baseline | filter_summary_is_null_count | populated | fresh-session | 60 | 27.15 | 25.77 | 29.42 | 29.28 | - | - | - | - |
| baseline | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 24.95 | 23.99 | 33.89 | 27.28 | - | - | - | - |
| baseline | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.39 | 32.46 | 36.10 | 35.81 | - | - | - | - |
| baseline | filter_id_range_project_summary | populated | fresh-session | 60 | 13.68 | 13.08 | 14.40 | 14.32 | - | - | - | - |
| baseline | take_random_1k | populated | fresh-session | 60 | 12.51 | 11.59 | 13.39 | 13.35 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | warm | 60 | 26.44 | 25.11 | 28.03 | 27.86 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | warm | 60 | 30.17 | 29.18 | 32.48 | 32.32 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 26.52 | 25.73 | 28.94 | 28.85 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | warm | 60 | 37.09 | 34.76 | 39.00 | 38.93 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | warm | 60 | 12.92 | 12.48 | 13.42 | 13.34 | - | - | - | - |
| baseline | take_random_1k | null_1pct | warm | 60 | 10.75 | 10.03 | 13.27 | 11.18 | - | - | - | - |
| baseline | scan_summary_full | null_1pct | fresh-session | 60 | 26.88 | 25.86 | 28.97 | 28.67 | - | - | - | - |
| baseline | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 31.12 | 29.23 | 33.07 | 32.79 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 27.51 | 26.56 | 29.85 | 29.65 | - | - | - | - |
| baseline | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 36.87 | 35.23 | 39.03 | 38.99 | - | - | - | - |
| baseline | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.82 | 13.38 | 14.48 | 14.36 | - | - | - | - |
| baseline | take_random_1k | null_1pct | fresh-session | 60 | 12.67 | 12.16 | 13.24 | 13.00 | - | - | - | - |
| prototype | scan_summary_full | populated | warm | 60 | 25.82 | 23.74 | 26.56 | 26.41 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | warm | 60 | 27.78 | 25.88 | 28.67 | 28.38 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | warm | 60 | 26.08 | 23.96 | 27.02 | 26.69 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | warm | 60 | 34.79 | 32.45 | 36.06 | 35.68 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | warm | 60 | 12.50 | 12.18 | 13.12 | 13.05 | - | - | - | - |
| prototype | take_random_1k | populated | warm | 60 | 10.57 | 9.91 | 11.36 | 10.90 | - | - | - | - |
| prototype | scan_summary_full | populated | fresh-session | 60 | 25.78 | 23.81 | 27.25 | 26.70 | - | - | - | - |
| prototype | filter_summary_is_null_count | populated | fresh-session | 60 | 28.20 | 25.99 | 29.06 | 28.98 | - | - | - | - |
| prototype | count_summary_vs_star | populated:aggregate | fresh-session | 60 | 26.08 | 24.01 | 29.01 | 27.15 | - | - | - | - |
| prototype | count_summary_vs_star | populated:sql | fresh-session | 60 | 34.89 | 32.68 | 36.47 | 35.85 | - | - | - | - |
| prototype | filter_id_range_project_summary | populated | fresh-session | 60 | 13.41 | 13.09 | 14.07 | 13.87 | - | - | - | - |
| prototype | take_random_1k | populated | fresh-session | 60 | 12.44 | 11.89 | 12.98 | 12.78 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | warm | 60 | 27.62 | 25.90 | 28.74 | 28.45 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | warm | 60 | 31.31 | 28.90 | 32.87 | 32.47 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | warm | 60 | 28.22 | 25.84 | 29.67 | 28.90 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | warm | 60 | 38.02 | 35.71 | 44.29 | 40.55 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | warm | 60 | 12.80 | 12.33 | 13.68 | 13.32 | - | - | - | - |
| prototype | take_random_1k | null_1pct | warm | 60 | 10.70 | 10.13 | 13.53 | 11.24 | - | - | - | - |
| prototype | scan_summary_full | null_1pct | fresh-session | 60 | 28.51 | 26.36 | 29.52 | 29.29 | - | - | - | - |
| prototype | filter_summary_is_null_count | null_1pct | fresh-session | 60 | 32.49 | 29.81 | 33.49 | 33.22 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:aggregate | fresh-session | 60 | 28.74 | 26.91 | 30.39 | 29.88 | - | - | - | - |
| prototype | count_summary_vs_star | null_1pct:sql | fresh-session | 60 | 37.51 | 35.81 | 40.09 | 39.09 | - | - | - | - |
| prototype | filter_id_range_project_summary | null_1pct | fresh-session | 60 | 13.59 | 13.15 | 14.15 | 13.89 | - | - | - | - |
| prototype | take_random_1k | null_1pct | fresh-session | 60 | 12.72 | 12.10 | 13.89 | 13.58 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | warm | 20 | 24.47 | 24.04 | 25.26 | 25.24 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | warm | 20 | 26.62 | 26.13 | 27.28 | 27.24 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | warm | 20 | 24.74 | 24.32 | 25.30 | 25.19 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | warm | 20 | 33.04 | 32.36 | 33.94 | 33.70 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | warm | 20 | 12.86 | 12.68 | 13.27 | 13.19 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | warm | 20 | 10.61 | 10.16 | 11.31 | 11.28 | - | - | - | - |
| prototype-flags | scan_summary_full | all_true | fresh-session | 20 | 24.70 | 23.90 | 29.01 | 27.83 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_true | fresh-session | 20 | 27.18 | 26.35 | 27.69 | 27.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:aggregate | fresh-session | 20 | 25.19 | 24.57 | 25.52 | 25.48 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_true:sql | fresh-session | 20 | 33.62 | 32.66 | 34.25 | 34.23 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_true | fresh-session | 20 | 13.82 | 13.60 | 14.17 | 13.97 | - | - | - | - |
| prototype-flags | take_random_1k | all_true | fresh-session | 20 | 12.57 | 12.07 | 13.00 | 12.90 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | warm | 20 | 47.52 | 42.69 | 48.87 | 48.21 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | warm | 20 | 52.08 | 49.38 | 60.80 | 53.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | warm | 20 | 48.17 | 46.29 | 51.00 | 49.28 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | warm | 20 | 65.38 | 59.89 | 66.40 | 66.02 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | warm | 20 | 12.68 | 12.40 | 12.93 | 12.93 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | warm | 20 | 10.68 | 10.07 | 11.30 | 11.21 | - | - | - | - |
| prototype-flags | scan_summary_full | partial_1pct | fresh-session | 20 | 47.99 | 42.38 | 49.00 | 48.97 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | partial_1pct | fresh-session | 20 | 52.62 | 45.18 | 54.54 | 53.95 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:aggregate | fresh-session | 20 | 48.81 | 43.60 | 49.66 | 49.43 | - | - | - | - |
| prototype-flags | count_summary_vs_star | partial_1pct:sql | fresh-session | 20 | 66.18 | 60.29 | 66.58 | 66.57 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | partial_1pct | fresh-session | 20 | 13.39 | 13.09 | 13.57 | 13.55 | - | - | - | - |
| prototype-flags | take_random_1k | partial_1pct | fresh-session | 20 | 12.55 | 12.20 | 13.18 | 12.85 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | warm | 20 | 2.78 | 2.74 | 2.83 | 2.83 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | warm | 20 | 5.48 | 5.41 | 5.60 | 5.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | warm | 20 | 3.20 | 3.15 | 3.25 | 3.23 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | warm | 20 | 6.83 | 6.79 | 6.99 | 6.94 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | warm | 20 | 11.97 | 11.84 | 12.12 | 12.12 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | warm | 20 | 0.38 | 0.37 | 0.41 | 0.40 | - | - | - | - |
| prototype-flags | scan_summary_full | all_pending | fresh-session | 20 | 2.77 | 2.73 | 2.83 | 2.81 | - | - | - | - |
| prototype-flags | filter_summary_is_null_count | all_pending | fresh-session | 20 | 5.49 | 5.42 | 5.54 | 5.53 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:aggregate | fresh-session | 20 | 3.22 | 3.15 | 3.28 | 3.27 | - | - | - | - |
| prototype-flags | count_summary_vs_star | all_pending:sql | fresh-session | 20 | 6.89 | 6.77 | 7.25 | 7.19 | - | - | - | - |
| prototype-flags | filter_id_range_project_summary | all_pending | fresh-session | 20 | 12.68 | 12.43 | 12.89 | 12.85 | - | - | - | - |
| prototype-flags | take_random_1k | all_pending | fresh-session | 20 | 0.39 | 0.37 | 0.42 | 0.42 | - | - | - | - |
<!-- END results:10m-reads-fix-prototype-first -->
