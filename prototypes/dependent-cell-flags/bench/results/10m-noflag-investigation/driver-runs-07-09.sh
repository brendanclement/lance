#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# Historical: the driver of runs 07-09, committed as bench/run_counters_rotation.sh.
# It takes the first matching binary under target/$PROFILE/deps, which is
# ambiguous when several builds share a directory, and records no provenance.
# New runs use bench/run_counters_rotation.py.
#
# Rotate several builds of the cell_flags_scan_counters bench over one shared
# dataset in a 4x4 Latin-square order, so every build runs in every position
# equally often. Each build must be a checkout outside any other Lance checkout
# (see README.md) with the bench built:
#   cargo bench -p lance --bench cell_flags_scan_counters --profile release-with-debug --no-run
#
# Usage:
#   DATASET=/abs/table OUT=/abs/results ROUNDS=20 \
#   BUILDS="baseline=/abs/base prototype=/abs/proto fix=/abs/fix funnel=/abs/var" \
#     ./run_counters_rotation.sh
# Exactly four builds. Create DATASET once with BENCH_COUNTERS_MODE=create.
# PROFILE (default release-with-debug) selects the target directory to take
# each binary from, e.g. PROFILE=release for fat-LTO builds made with
# `--profile release`.

set -euo pipefail

: "${DATASET:?set DATASET}" "${OUT:?set OUT}" "${BUILDS:?set BUILDS}"
: "${ROUNDS:=20}" "${SAMPLES:=30}"
: "${WORKLOADS:=scan_summary_full,filter_summary_is_null_count,count_summary_aggregate}"
: "${PROFILE:=release-with-debug}"
read -r -a entries <<< "$BUILDS"
if [[ ${#entries[@]} -ne 4 ]]; then
  echo "BUILDS must name exactly four builds (got ${#entries[@]})" >&2
  exit 2
fi
if compgen -G "$OUT/round*.jsonl" > /dev/null; then
  echo "$OUT already holds results" >&2
  exit 2
fi
mkdir -p "$OUT"

binary_of() {
  local dir=$1 binary
  binary=$(find "$dir/target/$PROFILE/deps" -maxdepth 1 -type f -perm -u+x \
    -name 'cell_flags_scan_counters-*' ! -name '*.d' | head -1)
  if [[ -z "$binary" ]]; then
    echo "no cell_flags_scan_counters binary under $dir/target/$PROFILE/deps" >&2
    exit 1
  fi
  echo "$binary"
}

names=()
binaries=()
for entry in "${entries[@]}"; do
  names+=("${entry%%=*}")
  binaries+=("$(binary_of "${entry#*=}")")
done

square=("0 1 2 3" "1 3 0 2" "2 0 3 1" "3 2 1 0")
: > "$OUT/order.tsv"
for round in $(seq 1 "$ROUNDS"); do
  for index in ${square[$(( (round - 1) % 4 ))]}; do
    printf '%s\t%s\t%s\n' "$round" "${names[$index]}" "${binaries[$index]}" >> "$OUT/order.tsv"
  done
done

measure() {
  BENCH_COUNTERS_MODE=measure BENCH_COUNTERS_URI="$DATASET" BENCH_READ_SAMPLES="$1" \
    BENCH_WARMUP="$2" BENCH_WORKLOADS="$3" BENCH_BUILD="$4" BENCH_OUT="$5" "$6"
}

# One unrecorded warm-up per build.
for index in 0 1 2 3; do
  measure 10 0 scan_summary_full warmup /dev/null "${binaries[$index]}" > /dev/null
done
while IFS=$'\t' read -r round name binary; do
  measure "$SAMPLES" 2 "$WORKLOADS" "$name" "$OUT/round$round-$name.jsonl" "$binary" \
    > "$OUT/round$round-$name.log" 2>&1
  printf '%s\t%s\t%s\n' "$round" "$name" "$(sysctl -n vm.loadavg)" >> "$OUT/runs.tsv"
done < "$OUT/order.tsv"
echo "wrote $OUT"
