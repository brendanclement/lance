#!/bin/bash
set -euo pipefail
S=/private/tmp/claude-501/-Users-brendan-code-lance--claude-worktrees-dependency-aware-cell-flags-43fc32/74020bc7-5e3b-4fa2-a700-40b87904a7b1/scratchpad
OUT=$S/counters-data-rotation20; mkdir -p $OUT
BIN=cell_flags_scan_counters-0e7dfad2f7e8d4d9
names=(baseline@basedata fix@basedata baseline@fixdata fix@fixdata)
bins=($S/regr-base/target/release-with-debug/deps/$BIN $S/regr-fix/target/release-with-debug/deps/$BIN $S/regr-base/target/release-with-debug/deps/$BIN $S/regr-fix/target/release-with-debug/deps/$BIN)
data=($S/bench-data/counters/articles_10m $S/bench-data/counters/articles_10m $S/bench-data/counters/articles_10m_fixwritten $S/bench-data/counters/articles_10m_fixwritten)
square=("0 1 2 3" "1 3 0 2" "2 0 3 1" "3 2 1 0")
for i in 0 1 2 3; do BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=${data[$i]} BENCH_READ_SAMPLES=10 BENCH_WARMUP=0 BENCH_WORKLOADS=scan_summary_full BENCH_BUILD=warmup BENCH_OUT=/dev/null ${bins[$i]} > /dev/null; done
for r in $(seq 1 20); do
  for i in ${square[$(( (r - 1) % 4 ))]}; do
    BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI=${data[$i]} BENCH_READ_SAMPLES=30 BENCH_WARMUP=2 BENCH_WORKLOADS=scan_summary_full,filter_summary_is_null_count,count_summary_aggregate BENCH_BUILD=${names[$i]} BENCH_OUT=$OUT/round$r-${names[$i]}.jsonl ${bins[$i]} > /dev/null
    printf '%s\t%s\t%s\n' $r ${names[$i]} "$(sysctl -n vm.loadavg)" >> $OUT/runs.tsv
  done
done
echo done
