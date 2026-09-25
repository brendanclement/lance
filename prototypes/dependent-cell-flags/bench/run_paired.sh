#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# Paired benchmark driver for the dependent cell flag prototype.
#
# Builds the regression harness in the baseline worktree and both harnesses in
# this (prototype) worktree with the same profile, then for each round runs:
# baseline regression, prototype regression, prototype flag harness. Results go
# to results/<scale>/round<r>-<build>.jsonl next to this script, with the
# machine and toolchain in env.json and one line per run in runs.tsv. Finally
# analyze.py writes results/<scale>/analysis.md and updates REPORT.md.
#
# Usage: BASELINE_WORKTREE=/path/to/baseline ./run_paired.sh <smoke|1m|10m>
# See README.md for the environment variables.

set -euo pipefail

SCALE=${1:-}
case "$SCALE" in
  smoke)
    : "${BENCH_SCALE_ROWS:=100000}" "${BENCH_ROWS_PER_FRAGMENT:=10000}"
    : "${BENCH_SAMPLES:=3}" "${BENCH_READ_SAMPLES:=3}" "${ROUNDS:=1}"
    ;;
  1m)
    : "${BENCH_SCALE_ROWS:=1000000}" "${BENCH_ROWS_PER_FRAGMENT:=100000}"
    : "${BENCH_SAMPLES:=5}" "${BENCH_READ_SAMPLES:=20}" "${ROUNDS:=3}"
    ;;
  10m)
    : "${BENCH_SCALE_ROWS:=10000000}" "${BENCH_ROWS_PER_FRAGMENT:=100000}"
    : "${BENCH_SAMPLES:=3}" "${BENCH_READ_SAMPLES:=20}" "${ROUNDS:=1}"
    # Reads, clean refresh, sparse updates and publish_after_k; each name
    # selects the regression workload or the flag workload of that name.
    : "${BENCH_WORKLOADS:=scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k,refresh_permissive_clean,flagged_refresh_clean,update_sparse,flagged_update_sparse,update_unrelated_sparse,flagged_update_unrelated_sparse,publish_after_k_commits,flagged_publish_after_k_commits}"
    ;;
  *)
    echo "usage: BASELINE_WORKTREE=<dir> $0 <smoke|1m|10m>" >&2
    exit 2
    ;;
esac
: "${BENCH_WARMUP:=1}"
: "${BENCH_RUN_NICE:=0}"
: "${FLAG_EVERY_ROUND:=1}"
: "${ORDER:=baseline-first}"
if [[ "$ORDER" != baseline-first && "$ORDER" != prototype-first ]]; then
  echo "ORDER must be baseline-first or prototype-first, not '$ORDER'" >&2
  exit 2
fi
export BENCH_SCALE_ROWS BENCH_ROWS_PER_FRAGMENT BENCH_SAMPLES BENCH_READ_SAMPLES BENCH_WARMUP
if [[ -n "${BENCH_WORKLOADS:-}" ]]; then export BENCH_WORKLOADS; fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROTO=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)
BASELINE=${BASELINE_WORKTREE:?set BASELINE_WORKTREE to a worktree of the baseline commit}
BASELINE=$(cd "$BASELINE" && pwd)
PROFILE=release-with-debug
RESULTS=${RESULTS_DIR:-$SCRIPT_DIR/results/$SCALE}
DATA=${BENCH_DATA_DIR:-${TMPDIR:-/tmp}/lance_cell_flags_bench}
export BENCH_DATA_DIR=$DATA

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  # Cargo hashes workspace members by their path relative to the workspace
  # root, so two worktrees sharing a target directory overwrite each other's
  # artifacts and binaries.
  echo "unset CARGO_TARGET_DIR: the two worktrees must build into their own target directories" >&2
  exit 2
fi
if [[ "$DATA" != /* ]]; then
  echo "BENCH_DATA_DIR must be absolute (got $DATA)" >&2
  exit 2
fi
for file in rust/lance/benches/cell_flags_common/mod.rs rust/lance/benches/cell_flags_regression.rs; do
  if ! cmp -s "$PROTO/$file" "$BASELINE/$file"; then
    echo "$file differs between $PROTO and $BASELINE; the regression harness must be identical" >&2
    exit 2
  fi
done
mkdir -p "$RESULTS" "$DATA"
if compgen -G "$RESULTS/round*.jsonl" > /dev/null; then
  echo "$RESULTS already holds results; move them away or set RESULTS_DIR" >&2
  exit 2
fi

# Print "<bench name> <executable>" for every bench binary cargo builds.
build_benches() {
  local dir=$1
  shift
  local args=()
  for bench in "$@"; do args+=(--bench "$bench"); done
  (cd "$dir" && nice -n 15 cargo bench -p lance "${args[@]}" --profile "$PROFILE" --no-run \
    --message-format=json-render-diagnostics) |
    python3 -c '
import json, sys
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        continue
    if message.get("reason") == "compiler-artifact" and message.get("executable") \
            and "bench" in message["target"]["kind"]:
        print(message["target"]["name"], message["executable"])
'
}

bench_binary() {
  local listing=$1 name=$2 dir=$3 path
  path=$(awk -v name="$name" '$1 == name { print $2 }' <<< "$listing" | tail -1)
  if [[ -z "$path" || "$path" != "$dir"/* ]]; then
    echo "could not find the $name binary under $dir (got '$path')" >&2
    exit 1
  fi
  echo "$path"
}

echo "== building baseline ($BASELINE)"
BASE_LIST=$(build_benches "$BASELINE" cell_flags_regression)
BASE_REGRESSION=$(bench_binary "$BASE_LIST" cell_flags_regression "$BASELINE")
echo "== building prototype ($PROTO)"
PROTO_LIST=$(build_benches "$PROTO" cell_flags_regression dependent_cell_flags)
PROTO_REGRESSION=$(bench_binary "$PROTO_LIST" cell_flags_regression "$PROTO")
PROTO_FLAGS=$(bench_binary "$PROTO_LIST" dependent_cell_flags "$PROTO")

BASE_SHA=$(git -C "$BASELINE" rev-parse HEAD)
PROTO_SHA=$(git -C "$PROTO" rev-parse HEAD)

python3 - "$RESULTS/env.json" <<EOF
import json, subprocess, sys

def run(*cmd, cwd=None):
    try:
        return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        return f"unavailable: {error}"

env = {
    "scale": "$SCALE",
    "profile": "$PROFILE",
    "hw_model": run("sysctl", "-n", "hw.model"),
    "cpu_brand": run("sysctl", "-n", "machdep.cpu.brand_string"),
    "ncpu": run("sysctl", "-n", "hw.ncpu"),
    "memsize_bytes": run("sysctl", "-n", "hw.memsize"),
    "macos": run("sw_vers", "-productVersion") + " (" + run("sw_vers", "-buildVersion") + ")",
    "rustc_baseline": run("rustc", "-V", cwd="$BASELINE"),
    "rustc_prototype": run("rustc", "-V", cwd="$PROTO"),
    "baseline_worktree": "$BASELINE",
    "baseline_sha": "$BASE_SHA",
    "baseline_status": run("git", "status", "--porcelain", cwd="$BASELINE"),
    "prototype_worktree": "$PROTO",
    "prototype_sha": "$PROTO_SHA",
    "prototype_status": run("git", "status", "--porcelain", cwd="$PROTO"),
    "binaries": {
        "baseline": "$BASE_REGRESSION",
        "prototype": "$PROTO_REGRESSION",
        "prototype-flags": "$PROTO_FLAGS",
    },
    "rounds": int("$ROUNDS"),
    "flag_every_round": "$FLAG_EVERY_ROUND" == "1",
    "run_nice": int("$BENCH_RUN_NICE"),
    "order": "$ORDER",
    "config": {
        key: value
        for key, value in {
            "BENCH_SCALE_ROWS": "$BENCH_SCALE_ROWS",
            "BENCH_ROWS_PER_FRAGMENT": "$BENCH_ROWS_PER_FRAGMENT",
            "BENCH_SAMPLES": "$BENCH_SAMPLES",
            "BENCH_READ_SAMPLES": "$BENCH_READ_SAMPLES",
            "BENCH_WARMUP": "$BENCH_WARMUP",
            "BENCH_WORKLOADS": "${BENCH_WORKLOADS:-}",
            "BENCH_UDF_ITERS": "${BENCH_UDF_ITERS:-}",
            "BENCH_STABLE_ROW_IDS": "${BENCH_STABLE_ROW_IDS:-}",
            "BENCH_CONFLICT_KS": "${BENCH_CONFLICT_KS:-}",
            "BENCH_PUBLISH_KS": "${BENCH_PUBLISH_KS:-}",
        }.items()
        if value
    },
}
with open(sys.argv[1], "w") as out:
    json.dump(env, out, indent=2)
    out.write("\n")
EOF
printf 'round\tbuild\tstarted\tfinished\tloadavg_before\n' > "$RESULTS/runs.tsv"

# run <round> <build> <sha> <binary> [env...]
run() {
  local round=$1 build=$2 sha=$3 binary=$4
  shift 4
  local out="$RESULTS/round$round-$build.jsonl" log="$RESULTS/round$round-$build.log"
  local started load
  started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  load=$(sysctl -n vm.loadavg | tr -d '{}' | xargs)
  echo "== round $round: $build ($(basename "$binary")), load $load"
  env "$@" BENCH_BUILD="$build" BENCH_GIT_SHA="$sha" BENCH_OUT="$out" \
    nice -n "$BENCH_RUN_NICE" "$binary" > "$log" 2>&1 || {
    echo "$build failed in round $round; see $log (datasets left under $DATA)" >&2
    exit 1
  }
  printf '%s\t%s\t%s\t%s\t%s\n' "$round" "$build" "$started" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$load" >> "$RESULTS/runs.tsv"
}

for round in $(seq 1 "$ROUNDS"); do
  # Reversing the order separates a build difference from an effect of
  # running second (thermal state, page cache, allocator).
  if [[ "$ORDER" == prototype-first ]]; then
    run "$round" prototype "$PROTO_SHA" "$PROTO_REGRESSION"
    run "$round" baseline "$BASE_SHA" "$BASE_REGRESSION"
  else
    run "$round" baseline "$BASE_SHA" "$BASE_REGRESSION"
    run "$round" prototype "$PROTO_SHA" "$PROTO_REGRESSION"
  fi
  if [[ "$FLAG_EVERY_ROUND" == 1 || "$round" == "$ROUNDS" ]]; then
    run "$round" prototype-flags "$PROTO_SHA" "$PROTO_FLAGS" LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1
  fi
done

python3 "$SCRIPT_DIR/analyze.py" "$RESULTS" --out "$RESULTS/analysis.md" --report "$SCRIPT_DIR/REPORT.md"
echo "== wrote $RESULTS/analysis.md and updated $SCRIPT_DIR/REPORT.md"
