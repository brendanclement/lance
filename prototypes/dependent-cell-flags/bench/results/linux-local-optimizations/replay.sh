#!/usr/bin/env bash
# Replays measurements from immutable snapshots; build/save them first.
set -euo pipefail
repo=${1:?standalone checkout required}
dataset=${2:?shared benchmark dataset required}
snapshots=${3:?directory holding before.json and after.json required}
out=${4:?new output root required}
driver="$repo/prototypes/dependent-cell-flags/bench/run_saved_counters.py"
analyzer="$repo/prototypes/dependent-cell-flags/bench/analyze_counters.py"
common=(--bench cell_flags_costs --dataset "$dataset"
  --build "before=$snapshots/before.json" --build "after=$snapshots/after.json"
  --control after --warmup 2)
reads=scan_ready_moved,scan_plain_moved,filter_null_ready_moved,filter_null_plain_moved,scan_masked,scan_null_all,filter_null_masked,filter_null_null_all,take_masked,take_null_all,scan_partial_1pct,scan_null_1pct,scan_one,scan_plain
mutations=backfill_one,backfill_plain_one,backfill_shared,backfill_plain_shared,backfill_chain,backfill_plain_chain,cycle_inplace_sparse_one,cycle_inplace_sparse_plain_one
allocations=scan_ready_moved,scan_plain_moved,scan_masked,scan_null_all,scan_partial_1pct,scan_null_1pct,backfill_one,backfill_plain_one,backfill_chain,backfill_plain_chain,cycle_inplace_sparse_one,cycle_inplace_sparse_plain_one
python3 "$driver" "${common[@]}" --out "$out/reads" --rounds 6 --samples 20 --workloads "$reads"
python3 "$driver" "${common[@]}" --out "$out/mutations" --rounds 6 --samples 6 --workloads "$mutations"
BENCH_COUNT_ALLOCATIONS=1 python3 "$driver" "${common[@]}" --out "$out/allocations" --rounds 3 --samples 3 --workloads "$allocations"
for name in reads mutations allocations; do
  python3 "$analyzer" "$out/$name" --reference before --builds after --spread > "$out/$name/analysis.md"
  python3 "$analyzer" "$out/$name" --reference after --builds after-copy --spread > "$out/$name/control.md"
done
