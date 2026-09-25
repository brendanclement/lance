#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# Layout control for the 10M read regression: the baseline against the
# baseline plus one unused function (results/10m-reads-layout-control/
# control.patch), so the only difference is code placement. Rounds
# 1..ROUNDS_PER_ORDER run the baseline first, the next ROUNDS_PER_ORDER the
# control first. The control's records are labelled `prototype`, so
# analyze.py's regression table shows control / baseline.
#
# Reproduces results/10m-reads-layout-control:
#
#   BASELINE_WORKTREE=/abs/lance-baseline CONTROL_WORKTREE=/abs/lance-layout-control \
#   BENCH_DATA_DIR=/abs/bench-data RESULTS_DIR=/abs/results/<new run> \
#     prototypes/dependent-cell-flags/bench/run_layout_control.sh
#
# CONTROL_WORKTREE is created from the baseline's commit, with the baseline's
# regression harness and control.patch, when it does not exist. Both worktrees
# must lie outside any other checkout, so that their builds get the same
# rustflags.

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
BASELINE=$(cd "${BASELINE_WORKTREE:?set BASELINE_WORKTREE to the baseline worktree with the regression harness}" && pwd)
CONTROL=${CONTROL_WORKTREE:?set CONTROL_WORKTREE to a path outside any other checkout}
RESULTS=${RESULTS_DIR:?set RESULTS_DIR to a directory for a new run}
DATA=${BENCH_DATA_DIR:-${TMPDIR:-/tmp}/lance_cell_flags_bench}
PATCH=$SCRIPT_DIR/results/10m-reads-layout-control/control.patch
PROFILE=release-with-debug
: "${ROUNDS_PER_ORDER:=3}" "${BENCH_RUN_NICE:=0}"
: "${BENCH_SCALE_ROWS:=10000000}" "${BENCH_ROWS_PER_FRAGMENT:=100000}"
: "${BENCH_SAMPLES:=3}" "${BENCH_READ_SAMPLES:=20}" "${BENCH_WARMUP:=1}"
: "${BENCH_WORKLOADS:=scan_summary_full,filter_summary_is_null_count,count_summary_vs_star,filter_id_range_project_summary,take_random_1k}"
export BENCH_SCALE_ROWS BENCH_ROWS_PER_FRAGMENT BENCH_SAMPLES BENCH_READ_SAMPLES BENCH_WARMUP
export BENCH_WORKLOADS BENCH_DATA_DIR=$DATA

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  echo "unset CARGO_TARGET_DIR: the two worktrees must build into their own target directories" >&2
  exit 2
fi
if [[ "$DATA" != /* ]]; then
  echo "BENCH_DATA_DIR must be absolute (got $DATA)" >&2
  exit 2
fi
if compgen -G "$RESULTS/round*.jsonl" > /dev/null; then
  echo "$RESULTS already holds results; move them away or set RESULTS_DIR" >&2
  exit 2
fi

if [[ ! -d "$CONTROL" ]]; then
  git -C "$BASELINE" worktree add --detach "$CONTROL" "$(git -C "$BASELINE" rev-parse HEAD)"
  cp -R "$BASELINE/rust/lance/benches/cell_flags_common" "$CONTROL/rust/lance/benches/"
  cp "$BASELINE/rust/lance/benches/cell_flags_regression.rs" "$CONTROL/rust/lance/benches/"
  cp "$BASELINE/rust/lance/Cargo.toml" "$CONTROL/rust/lance/Cargo.toml"
  git -C "$CONTROL" apply "$PATCH"
fi
PROTO=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)
for file in rust/lance/benches/cell_flags_common/mod.rs rust/lance/benches/cell_flags_regression.rs; do
  if ! cmp -s "$PROTO/$file" "$BASELINE/$file" || ! cmp -s "$BASELINE/$file" "$CONTROL/$file"; then
    echo "$file differs between $PROTO, $BASELINE and $CONTROL; the regression harness must be identical" >&2
    exit 2
  fi
done
if ! cmp -s "$BASELINE/rust/lance/Cargo.toml" "$CONTROL/rust/lance/Cargo.toml"; then
  echo "rust/lance/Cargo.toml differs between $BASELINE and $CONTROL" >&2
  exit 2
fi
if ! git -C "$CONTROL" apply --reverse --check "$PATCH" 2> /dev/null ||
  [[ "$(git -C "$CONTROL" diff --name-only HEAD -- rust/lance/src)" != rust/lance/src/dataset.rs ]]; then
  echo "$CONTROL/rust/lance/src must differ from its HEAD by exactly $PATCH" >&2
  exit 2
fi

build() {
  local dir=$1 path
  path=$( (cd "$dir" && nice -n 15 cargo bench -p lance --bench cell_flags_regression \
    --profile "$PROFILE" --no-run --message-format=json-render-diagnostics) |
    python3 -c '
import json, sys
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        continue
    if message.get("reason") == "compiler-artifact" and message.get("executable") \
            and message["target"]["name"] == "cell_flags_regression":
        print(message["executable"])
' | tail -1)
  if [[ -z "$path" || "$path" != "$dir"/* ]]; then
    echo "could not find the cell_flags_regression binary under $dir (got '$path')" >&2
    exit 1
  fi
  echo "$path"
}

rustflags_of() {
  local binary=$1 dir=$2
  python3 -c 'import json, sys; print(json.dumps(json.load(open(sys.argv[1]))["rustflags"]))' \
    "$dir/target/$PROFILE/.fingerprint/lance-${binary##*-}/test-bench-cell_flags_regression.json"
}

echo "== building baseline ($BASELINE)"
BASE_BIN=$(build "$BASELINE")
echo "== building control ($CONTROL)"
CONTROL_BIN=$(build "$CONTROL")
BASE_RUSTFLAGS=$(rustflags_of "$BASE_BIN" "$BASELINE")
CONTROL_RUSTFLAGS=$(rustflags_of "$CONTROL_BIN" "$CONTROL")
if [[ "$BASE_RUSTFLAGS" != "$CONTROL_RUSTFLAGS" ]]; then
  echo "the builds used different rustflags (baseline $BASE_RUSTFLAGS, control" \
    "$CONTROL_RUSTFLAGS); build both worktrees outside any other checkout" >&2
  exit 2
fi
BASE_SHA=$(git -C "$BASELINE" rev-parse HEAD)

mkdir -p "$RESULTS" "$DATA"
python3 - "$RESULTS/env.json" <<EOF
import json, subprocess, sys

def run(*cmd, cwd=None):
    try:
        return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        return f"unavailable: {error}"

env = {
    "scale": "10m",
    "profile": "$PROFILE",
    "hw_model": run("sysctl", "-n", "hw.model"),
    "cpu_brand": run("sysctl", "-n", "machdep.cpu.brand_string"),
    "ncpu": run("sysctl", "-n", "hw.ncpu"),
    "memsize_bytes": run("sysctl", "-n", "hw.memsize"),
    "macos": run("sw_vers", "-productVersion") + " (" + run("sw_vers", "-buildVersion") + ")",
    "rustc_baseline": run("rustc", "-V", cwd="$BASELINE"),
    "rustc_prototype": run("rustc", "-V", cwd="$CONTROL"),
    "baseline_worktree": "$BASELINE",
    "baseline_sha": "$BASE_SHA",
    "baseline_status": run("git", "status", "--porcelain", cwd="$BASELINE"),
    "prototype_worktree": "$CONTROL",
    "prototype_sha": "$BASE_SHA",
    "prototype_status": run("git", "status", "--porcelain", cwd="$CONTROL"),
    "control": "records labelled prototype are the baseline plus control.patch",
    "binaries": {"baseline": "$BASE_BIN", "prototype": "$CONTROL_BIN"},
    "rounds": 2 * int("$ROUNDS_PER_ORDER"),
    "run_nice": int("$BENCH_RUN_NICE"),
    "order": "rounds 1-$ROUNDS_PER_ORDER baseline first, then control first",
    "rustflags": $BASE_RUSTFLAGS,
    "config": {
        "BENCH_SCALE_ROWS": "$BENCH_SCALE_ROWS",
        "BENCH_ROWS_PER_FRAGMENT": "$BENCH_ROWS_PER_FRAGMENT",
        "BENCH_SAMPLES": "$BENCH_SAMPLES",
        "BENCH_READ_SAMPLES": "$BENCH_READ_SAMPLES",
        "BENCH_WARMUP": "$BENCH_WARMUP",
        "BENCH_WORKLOADS": "$BENCH_WORKLOADS",
    },
}
with open(sys.argv[1], "w") as out:
    json.dump(env, out, indent=2)
    out.write("\n")
EOF
printf 'round\tbuild\tstarted\tfinished\tloadavg_before\n' > "$RESULTS/runs.tsv"

# run <round> <build> <binary> <sha label>
run() {
  local round=$1 build=$2 binary=$3 sha=$4 started load
  local out="$RESULTS/round$round-$build.jsonl" log="$RESULTS/round$round-$build.log"
  started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  load=$(sysctl -n vm.loadavg | tr -d '{}' | xargs)
  echo "== round $round: $build, load $load"
  BENCH_BUILD="$build" BENCH_GIT_SHA="$sha" BENCH_OUT="$out" \
    nice -n "$BENCH_RUN_NICE" "$binary" > "$log" 2>&1 || {
    echo "$build failed in round $round; see $log (datasets left under $DATA)" >&2
    exit 1
  }
  printf '%s\t%s\t%s\t%s\t%s\n' "$round" "$build" "$started" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$load" >> "$RESULTS/runs.tsv"
}

for round in $(seq 1 $((2 * ROUNDS_PER_ORDER))); do
  if ((round <= ROUNDS_PER_ORDER)); then
    run "$round" baseline "$BASE_BIN" "$BASE_SHA"
    run "$round" prototype "$CONTROL_BIN" "$BASE_SHA+control.patch"
  else
    run "$round" prototype "$CONTROL_BIN" "$BASE_SHA+control.patch"
    run "$round" baseline "$BASE_BIN" "$BASE_SHA"
  fi
done

python3 "$SCRIPT_DIR/analyze.py" "$RESULTS" --out "$RESULTS/analysis.md" \
  --report "$SCRIPT_DIR/REPORT.md" \
  --title "Control: 10M reads, baseline plus one unused function (records labelled \`prototype\`), rounds 1-$ROUNDS_PER_ORDER baseline first, then control first" \
  > /dev/null
echo "== wrote $RESULTS/analysis.md and updated $SCRIPT_DIR/REPORT.md"
