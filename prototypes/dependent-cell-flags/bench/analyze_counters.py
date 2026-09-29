#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Compare builds measured by the cell_flags_scan_counters bench.

Reads ``round<r>-<build>.jsonl`` files from one directory. For each workload it
prints, per build, the geometric mean over rounds of the per-round ratio to the
reference build (medians within a round), for wall time, retired instructions
and cycles, with a bootstrap 95% interval and how many rounds were above 1. It
also prints the median number of busy cores (CPU time / wall time).

    python3 analyze_counters.py <dir> --reference baseline --builds prototype,fix

A metric the bench recorded as null (unavailable on that platform or machine)
is reported as unavailable, never as a ratio. Records written before the bench
recorded CPU time in nanoseconds (no ``schema`` field) store it in Mach ticks;
``--legacy-mach-timebase 125/3`` converts them as on Apple silicon, and without
it their busy cores are unavailable. A round missing a build or the reference
is listed and left out of that build's ratios. The rows each read returned
must agree across builds, and every run must have read the same table.

Standard library only.
"""

import argparse
import collections
import fractions
import json
import math
import random
import re
import statistics
import sys
from pathlib import Path

ROUND_FILE = re.compile(r"round(\d+)-(.+)\.jsonl$")
SCHEMA = 2
METRICS = ("wall_ns", "instructions", "cycles")


def fail(message):
    sys.exit(f"analyze_counters: {message}")


def load(directory):
    """Sample records, each tagged with its round, and the run records."""
    samples = []
    runs = []
    for path in sorted(Path(directory).glob("round*-*.jsonl")):
        match = ROUND_FILE.search(path.name)
        if not match:
            continue
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            kind = record.get("record")
            if kind == "run":
                if record.get("schema") != SCHEMA:
                    fail(
                        f"{path}: run record schema {record.get('schema')}, expected {SCHEMA}"
                    )
                runs.append((path, record))
                continue
            if kind == "sample":
                if record.get("schema") != SCHEMA:
                    fail(
                        f"{path}: sample schema {record.get('schema')}, expected {SCHEMA}"
                    )
            elif kind is not None:
                fail(f"{path}: unknown record kind {kind!r}")
            record["round"] = int(match.group(1))
            samples.append(record)
    return samples, runs


def cpu_ns(record, legacy_timebase):
    """CPU time of one sample in nanoseconds, or None when unavailable."""
    if record.get("record") == "sample":
        return record["cpu_ns"]
    if legacy_timebase is None:
        return None
    return (record["user_time"] + record["system_time"]) * legacy_timebase


def geomean(values):
    return math.exp(statistics.mean(math.log(v) for v in values))


def bootstrap(values, draws=5000, seed=1):
    rng = random.Random(seed)
    means = sorted(geomean([rng.choice(values) for _ in values]) for _ in range(draws))
    return means[int(0.025 * draws)], means[int(0.975 * draws)]


def check_consistency(samples, runs):
    rows = collections.defaultdict(set)
    for record in samples:
        rows[record["workload"]].add(record["rows"])
    for workload, values in rows.items():
        if len(values) != 1:
            fail(
                f"{workload} returned different row counts across builds or rounds: {sorted(values)}"
            )
    tables = collections.defaultdict(list)
    for path, run in runs:
        tables[run["dataset"]["manifest_blake3"]].append(path.name)
    if len(tables) > 1:
        fail(f"runs read different tables (manifest blake3 -> files): {dict(tables)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("directory")
    parser.add_argument("--reference", default="baseline")
    parser.add_argument(
        "--builds", required=True, help="comma-separated builds to compare"
    )
    parser.add_argument(
        "--legacy-mach-timebase",
        type=fractions.Fraction,
        help="nanoseconds per Mach tick for records without a schema, e.g. 125/3",
    )
    args = parser.parse_args()

    samples, runs = load(args.directory)
    check_consistency(samples, runs)
    groups = collections.defaultdict(list)
    for record in samples:
        groups[(record["workload"], record["build"], record["round"])].append(record)
    rounds = sorted({record["round"] for record in samples})
    workloads = sorted({record["workload"] for record in samples})
    builds = args.builds.split(",")
    known = {record["build"] for record in samples}
    unknown = [build for build in [args.reference] + builds if build not in known]
    if unknown:
        fail(
            f"no records for {', '.join(unknown)}; recorded builds are {', '.join(sorted(known))}"
        )
    timebase = (
        None if args.legacy_mach_timebase is None else float(args.legacy_mach_timebase)
    )

    for workload in workloads:
        print(f"## {workload}")
        print("| build | metric | ratio to reference | 95% interval | rounds > 1 |")
        print("|---|---|---|---|---|")
        dropped = {}
        for build in builds:
            paired = [
                rnd
                for rnd in rounds
                if groups.get((workload, build, rnd))
                and groups.get((workload, args.reference, rnd))
            ]
            missing = [rnd for rnd in rounds if rnd not in paired]
            if missing:
                dropped[build] = missing
            for metric in METRICS:
                if any(
                    record[metric] is None
                    for rnd in paired
                    for key in (build, args.reference)
                    for record in groups[(workload, key, rnd)]
                ):
                    print(f"| {build} | {metric} | unavailable | — | — |")
                    continue
                ratios = []
                for rnd in paired:
                    ours = statistics.median(
                        r[metric] for r in groups[(workload, build, rnd)]
                    )
                    ref = statistics.median(
                        r[metric] for r in groups[(workload, args.reference, rnd)]
                    )
                    if ours <= 0 or ref <= 0:
                        fail(
                            f"{workload} round {rnd}: median {metric} of {build} is {ours}, "
                            f"of {args.reference} {ref}; a ratio needs positive values"
                        )
                    ratios.append(ours / ref)
                if not ratios:
                    continue
                low, high = bootstrap(ratios)
                above = sum(ratio > 1 for ratio in ratios)
                print(
                    f"| {build} | {metric} | {geomean(ratios):.3f} | "
                    f"[{low:.3f}, {high:.3f}] | {above}/{len(ratios)} |"
                )
        for build, missing in dropped.items():
            print(
                f"\n{build}: rounds {', '.join(map(str, missing))} lack {build} or "
                f"{args.reference} and are left out."
            )
        print()
        print("| build | median wall ms | p10 wall ms | busy cores |")
        print("|---|---|---|---|")
        for build in [args.reference] + builds:
            records = [
                r for rnd in rounds for r in groups.get((workload, build, rnd), [])
            ]
            if not records:
                continue
            walls = [r["wall_ns"] / 1e6 for r in records]
            p10 = (
                f"{statistics.quantiles(walls, n=10)[0]:.2f}" if len(walls) > 1 else "—"
            )
            cpu = [cpu_ns(r, timebase) for r in records]
            if any(value is None for value in cpu):
                busy = "unavailable"
            else:
                busy = f"{statistics.median(c / r['wall_ns'] for c, r in zip(cpu, records)):.2f}"
            print(f"| {build} | {statistics.median(walls):.2f} | {p10} | {busy} |")
        print()


if __name__ == "__main__":
    main()
