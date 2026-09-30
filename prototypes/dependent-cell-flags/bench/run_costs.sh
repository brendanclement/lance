#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# The rotations behind bench/results/feature-costs/, one step per argument:
#
#   run_costs.sh create      tables for every step, with the final build's binaries
#   run_costs.sh reads       cell_flags_costs scalar reads, timed
#   run_costs.sh mutations   every other cell_flags_costs workload, timed
#   run_costs.sh allocations every cell_flags_costs workload, allocations counted
#   run_costs.sh scaling     a subset at 4M rows and at 40 fragments of 1M rows
#   run_costs.sh vector      cell_flags_vector_masking, timed and counted
#   run_costs.sh noflag      cell_flags_scan_counters against main, timed and counted
#
# FINAL and MAIN are git checkouts outside any other checkout (the driver
# refuses nested ones): FINAL at the prototype commit under test, MAIN at the
# baseline with the counters bench files copied in. OUT receives one
# directory per rotation; ROOTS holds the tables. Run the steps one at a
# time on an otherwise idle machine.
set -euo pipefail

: "${FINAL:?set FINAL to the prototype checkout}"
: "${MAIN:?set MAIN to the baseline checkout}"
: "${ROOTS:?set ROOTS to the table directory}"
: "${OUT:?set OUT to the results directory}"
: "${VECTOR_TABLES:?set VECTOR_TABLES to the cell_flags_vector_masking root}"
PROFILE=release-with-debug
HERE=$(cd "$(dirname "$0")" && pwd)
DRIVER="$HERE/run_counters_rotation.py"

bench_binary() {
  (cd "$FINAL" && cargo bench -p lance --bench "$1" --profile "$PROFILE" --locked --no-run \
    --message-format=json-render-diagnostics 2>/dev/null) |
    python3 -c 'import json, sys
for line in sys.stdin:
    m = json.loads(line)
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == sys.argv[1]:
        print(m["executable"])' "$1"
}

costs_workloads() {
  BENCH_COUNTERS_MODE=list "$(bench_binary cell_flags_costs)" | grep -E "$1" | paste -sd, -
}

READS='^(scan|filter_null|take)_'
SCALING_TABLES=plain,one,partial_1pct,null_1pct,plain_h16,one_h16
SCALING_WORKLOADS=scan_one,scan_plain,scan_partial_1pct,scan_null_1pct,filter_null_partial_1pct,filter_null_null_1pct,take_partial_1pct,take_null_1pct,unrelated_moving_plain,unrelated_moving_one,cycle_moving_sparse_one,cycle_moving_sparse_plain_one,cycle_moving_dense_one,cycle_moving_dense_plain_one,cycle_inplace_sparse_one,cycle_inplace_sparse_plain_one,conflict_inplace_k1_reject,conflict_inplace_k1_skip,after_appends_k32_one,after_appends_k32_plain_one,reopen_plain,reopen_one,reopen_plain_h16,reopen_one_h16,append_plain,append_one,append_plain_h16,append_one_h16,update_plain,update_one,update_plain_h16,update_one_h16

costs() {
  local name=$1 root=$2 workloads=$3 rounds=$4 samples=$5 warmup=$6
  python3 "$DRIVER" --bench cell_flags_costs --dataset "$root" --out "$OUT/$name" \
    --profile "$PROFILE" --build final="$FINAL" --control final \
    --rounds "$rounds" --samples "$samples" --warmup "$warmup" --workloads "$workloads"
}

case "${1:?step}" in
  create)
    costs_binary=$(bench_binary cell_flags_costs)
    BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI="$ROOTS/costs-1m" \
      BENCH_SCALE_ROWS=1000000 BENCH_ROWS_PER_FRAGMENT=100000 "$costs_binary"
    BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI="$ROOTS/costs-4m" BENCH_TABLES=$SCALING_TABLES \
      BENCH_SCALE_ROWS=4000000 BENCH_ROWS_PER_FRAGMENT=100000 "$costs_binary"
    BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI="$ROOTS/costs-1m-40frag" BENCH_TABLES=$SCALING_TABLES \
      BENCH_SCALE_ROWS=1000000 BENCH_ROWS_PER_FRAGMENT=25000 "$costs_binary"
    BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI="$ROOTS/articles-10m" \
      BENCH_SCALE_ROWS=10000000 "$(bench_binary cell_flags_scan_counters)"
    ;;
  reads)
    costs reads "$ROOTS/costs-1m" "$(costs_workloads "$READS")" 8 30 2
    ;;
  mutations)
    costs mutations "$ROOTS/costs-1m" "$(costs_workloads "^" | tr , '\n' | grep -Ev "$READS" | paste -sd, -)" 8 10 2
    ;;
  allocations)
    BENCH_COUNT_ALLOCATIONS=1 costs allocations "$ROOTS/costs-1m" "$(costs_workloads "^")" 2 4 2
    ;;
  scaling)
    BENCH_SCALE_ROWS=4000000 costs scaling-4m "$ROOTS/costs-4m" "$SCALING_WORKLOADS" 4 6 2
    costs scaling-1m-40frag "$ROOTS/costs-1m-40frag" "$SCALING_WORKLOADS" 4 6 2
    ;;
  vector)
    python3 "$DRIVER" --bench cell_flags_vector_masking --dataset "$VECTOR_TABLES" \
      --out "$OUT/vector" --profile "$PROFILE" --build final="$FINAL" --control final \
      --rounds 8 --samples 20 --warmup 2
    BENCH_COUNT_ALLOCATIONS=1 python3 "$DRIVER" --bench cell_flags_vector_masking \
      --dataset "$VECTOR_TABLES" --out "$OUT/vector-allocations" --profile "$PROFILE" \
      --build final="$FINAL" --control final --rounds 2 --samples 4 --warmup 2
    ;;
  noflag)
    # Both builds must run the same bench source, allocator included.
    for file in cell_flags_scan_counters.rs cell_flags_common/mod.rs \
      cell_flags_common/counters.rs cell_flags_common/alloc.rs; do
      cmp "$MAIN/rust/lance/benches/$file" "$FINAL/rust/lance/benches/$file"
    done
    python3 "$DRIVER" --dataset "$ROOTS/articles-10m" --out "$OUT/noflag" --profile "$PROFILE" \
      --build main="$MAIN" --build final="$FINAL" --control final \
      --rounds 6 --samples 30 --warmup 2
    BENCH_COUNT_ALLOCATIONS=1 python3 "$DRIVER" --dataset "$ROOTS/articles-10m" \
      --out "$OUT/noflag-allocations" --profile "$PROFILE" \
      --build main="$MAIN" --build final="$FINAL" --control final \
      --rounds 3 --samples 4 --warmup 2
    ;;
  *)
    echo "unknown step $1" >&2
    exit 2
    ;;
esac
