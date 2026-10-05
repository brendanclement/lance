# Linux local optimization results — 2026-10-05

The [continuation handoff](../../../CONTINUATION.md) contains the interpretation,
code changes, contract, test provenance and remaining decisions. These are warm
Linux x86_64 tmpfs measurements, with source on CIFS, not production acceptance.

## Records

- `reads/`, `mutations/`, `allocations/`: raw per-process JSONL samples, warmups,
  `run.json`, `runs.tsv`, `order.tsv`, logs, source patches, generated `analysis.md`
  and identical-binary `control.md`.
- `baseline-smoke/`: the baseline-only reproduction before Rust edits, with an
  identical baseline-binary control; preliminary, not used for final conclusions.
- `summary.json`: compact metrics derived from those records by `summarize.py`,
  using the existing analyzer's paired ratios and bootstrap implementation.
- `verification/`: final Rust tests, Clippy, build and Git-integrity logs, plus
  `checks.json`. Historical tests are not substituted for these local checks.
- `protobuf-adapter-verification.json`: matching descriptor hashes for all six crates.
- `replay.sh`: exact read, mutation and allocation selections and rotation settings.

The original absolute paths in records identify this run. Multi-GiB executables,
Cargo targets, generated data and scratch copies are excluded from Git. `/tmp`
paths are ephemeral. There are 5,040 read samples, 864 mutation samples and 324
allocation-counting samples, excluding warmups. Root file hashes matched throughout.

## Reproduce

Use standalone sibling checkouts outside any other Cargo checkout, distinct local
build targets, and the same repository profile, compiler, default features and
flags. The recorded before source tree is exactly commit `1f70175dd` (Rust code
unchanged from transferred `cd44c394c`); after code is `fabb89983`. Build each with:

```sh
PROTOC=/tmp/cell-flags-tools/protoc-network CARGO_TARGET_DIR=/tmp/cell-flags-build \
  cargo bench -p lance --bench cell_flags_costs --profile release-with-debug --locked --no-run -j 12
```

Change the target path for the other checkout. The handoff explains the network
protoc adapter. After each completed build, before changing its source, save its
reported executable using `bench/run_saved_counters.py save --name before` or
`--name after`, with `--checkout`, `--executable` and `--out /tmp/cell-flags-snapshots`.
The saved JSON captures the source tree, patch, build fingerprint and executable hash.
Snapshots must describe the actual completed build. Do not infer a past build from
current source. The before and after binary hashes of this run are recorded in every
`run.json`; their saved immutable copies still live under `/tmp/cell-flags-snapshots`
for this session.

Generate a fresh shared dataset once with the before binary (use a new path):

```sh
BENCH_COUNTERS_MODE=create BENCH_COUNTERS_URI=/tmp/cell-flags-data-new \
BENCH_SCALE_ROWS=1000000 BENCH_ROWS_PER_FRAGMENT=100000 \
BENCH_TABLES=plain,one,ready_moved,plain_moved,partial_1pct,null_1pct,masked,null_all,pending_one,pending_shared,pending_chain,plain_empty,shared,chain \
  /tmp/cell-flags-snapshots/before-79ab713d41b9
```

The create pass checks visible-data digests of matched read pairs. Both builds then
use that same dataset root, never separate regenerated tables. For this session's
existing data and snapshots:

```sh
bash prototypes/dependent-cell-flags/bench/results/linux-local-optimizations/replay.sh \
  /home/brendan/work/lance-cell-flags /tmp/cell-flags-data \
  /tmp/cell-flags-snapshots /tmp/cell-flags-results/replay-new
```

Use a new output root and do not compile during measurements. The script checks
matching settings, immutable binary hashes, workload counts and root table hashes,
then generates analysis and control tables. To regenerate the compact summary:

```sh
python3 prototypes/dependent-cell-flags/bench/results/linux-local-optimizations/summarize.py \
  /home/brendan/work/lance-cell-flags /tmp/cell-flags-results/replay-new \
  > /tmp/cell-flags-results/replay-new/summary.json
```

Linux process CPU time is available; instructions/cycles are explicitly unavailable.
Allocation volume/peak count requested capacity, not RSS. Function computation is
outside every phase. Linux tmpfs working copies use ordinary `cp -R --reflink=auto`
plus a read of every file and a fresh Lance session, rather than APFS clonefile.
Background load is recorded and controls show substantial noise in some phases;
only the large, repeated effects support conclusions here.
