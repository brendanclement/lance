#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# Read-path variant: the prototype with rust/lance/src/dataset/fragment.rs
# reverted to the baseline, run against the baseline with run_paired.sh. Only
# fragment.rs is reverted; the rest of the prototype (for example the masked
# index check in index.rs and the manifest's cell flag decoding) stays. The
# flag harness is not run, since its masking checks cannot pass without the
# fragment read path.
#
# Reproduces results/10m-reads-variant-fragment-reverted:
#
#   BASELINE_WORKTREE=/abs/lance-baseline VARIANT_WORKTREE=/abs/lance-variant \
#   BENCH_DATA_DIR=/abs/bench-data RESULTS_DIR=/abs/results/<new run> \
#     prototypes/dependent-cell-flags/bench/run_variant.sh
#
# VARIANT_WORKTREE is created at PROTOTYPE_COMMIT when it does not exist. Like
# the baseline, it must lie outside any other checkout. Any other run_paired.sh
# variable (ORDER, ROUNDS, BENCH_WORKLOADS, ...) passes through.

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
BASELINE=$(cd "${BASELINE_WORKTREE:?set BASELINE_WORKTREE to the baseline worktree}" && pwd)
VARIANT=${VARIANT_WORKTREE:?set VARIANT_WORKTREE to a path outside any other checkout}
RESULTS=${RESULTS_DIR:?set RESULTS_DIR to an absolute path for a new run}
: "${PROTOTYPE_COMMIT:=26388225a273052b3a1ff0ec705c53da1b68c7cb}"
: "${BASELINE_COMMIT:=$(git -C "$BASELINE" rev-parse HEAD)}"
: "${ORDER:=baseline-first}" "${ROUNDS:=3}"
: "${BENCH_WORKLOADS:=scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k}"
export ORDER ROUNDS BENCH_WORKLOADS BASELINE_WORKTREE RESULTS_DIR

if [[ "$RESULTS" != /* ]]; then
  echo "RESULTS_DIR must be absolute (got $RESULTS)" >&2
  exit 2
fi
if [[ ! -d "$VARIANT" ]]; then
  git -C "$SCRIPT_DIR" worktree add --detach "$VARIANT" "$PROTOTYPE_COMMIT"
  git -C "$VARIANT" checkout "$BASELINE_COMMIT" -- rust/lance/src/dataset/fragment.rs
fi
if [[ -z "$(git -C "$VARIANT" status --porcelain -- rust/lance/src/dataset/fragment.rs)" ]]; then
  echo "$VARIANT has the prototype's fragment.rs; revert it to $BASELINE_COMMIT first" >&2
  exit 2
fi

# This checkout's driver, minus the flag harness run.
python3 - "$SCRIPT_DIR/run_paired.sh" "$VARIANT/prototypes/dependent-cell-flags/bench/run_paired.sh" <<'EOF'
import sys

source, target = sys.argv[1:]
text = open(source).read()
flag_run = """  if [[ "$FLAG_EVERY_ROUND" == 1 || "$round" == "$ROUNDS" ]]; then
    run "$round" prototype-flags "$PROTO_SHA" "$PROTO_FLAGS" LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1
  fi
"""
if text.count(flag_run) != 1:
    sys.exit(f"{source}: expected the flag harness run exactly once")
comment = "  # Variant: fragment.rs is reverted to the baseline, so the flag harness cannot pass.\n"
with open(target, "w") as out:
    out.write(text.replace(flag_run, comment))
EOF

(cd "$VARIANT" && prototypes/dependent-cell-flags/bench/run_paired.sh 10m)

# run_paired.sh analyzed with the variant's analyze.py and updated the
# variant's REPORT.md; redo both with this checkout's.
python3 "$SCRIPT_DIR/analyze.py" "$RESULTS" --out "$RESULTS/analysis.md" \
  --report "$SCRIPT_DIR/REPORT.md" \
  --title "Control: 10M reads, \`${PROTOTYPE_COMMIT:0:9}\` with \`fragment.rs\` reverted to the baseline (no flag runs), $ORDER" \
  > /dev/null
echo "== wrote $RESULTS/analysis.md and updated $SCRIPT_DIR/REPORT.md"
