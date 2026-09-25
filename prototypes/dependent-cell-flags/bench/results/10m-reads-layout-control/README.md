# Layout control: notes on the recorded run (supplementary)

The baseline (`e3671b2f5`) against the baseline plus `control.patch`, an unused function in
`rust/lance/src/dataset.rs`. Records of the control build are labelled `prototype`. The run was
driven by an inline loop, not `run_paired.sh`; `../../run_layout_control.sh` is that loop as a
committed script.

- `env.json` was written by hand by the loop and is partial. It has `scale`, `profile`,
  `baseline_sha`, a free-text `prototype_sha`, `binaries`, `rounds` and `order`; it has no machine,
  `rustc`, git status, rustflags or harness config, and the records carry `git_sha: null`.
- Machine and time: the same session and machine as the other runs, directly after `1m-clean`
  (`runs.tsv`: 10:54–11:03 UTC on 2026-09-25).
- Harness settings, from the loop: `BENCH_SCALE_ROWS=10000000 BENCH_ROWS_PER_FRAGMENT=100000
  BENCH_SAMPLES=3 BENCH_READ_SAMPLES=20 BENCH_WARMUP=1`, `BENCH_WORKLOADS` the five read workloads,
  no `nice`; rounds 1–3 baseline first, 4–6 control first.
- The control worktree held the baseline's regression harness and `rust/lance/Cargo.toml`, and
  `control.patch` was its only source change.
- Build flags: both worktrees were in the session's scratch directory under `/private/tmp`,
  outside any other checkout, and the control binary has the baseline's file hash
  (`cell_flags_regression-bff5b3f24f8697e9`). The same prototype code built with doubled rustflags
  got `cc60bdbcaa7919a6` and without them `bff5b3f24f8697e9`, so the equal hash implies the same
  rustflags and features. The fingerprint itself was not recorded.
