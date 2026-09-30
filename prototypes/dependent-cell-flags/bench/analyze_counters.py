#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Compare builds measured by a counters bench.

Reads ``round<r>-<build>.jsonl`` files from one directory. For each workload it
prints, per build, the geometric mean over rounds of the per-round ratio to the
reference build (medians within a round), for wall time, retired instructions
and cycles, with a bootstrap 95% interval and how many rounds were above 1. It
also prints the median number of busy cores (CPU time / wall time). Where the
samples carry allocation counters (cell_flags_vector_masking), their ratios and
medians are printed too.

    python3 analyze_counters.py <dir> --reference baseline --builds prototype,fix

``--pair A:B`` (repeatable) also compares two workloads within each build: the
geometric mean over rounds of median(A) / median(B), as for builds.

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
ALLOCATION_METRICS = ("allocations", "allocated_bytes", "peak_live_growth_bytes")


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


def metrics_of(records):
    """The metrics to report: the counters, and the allocation counters
    where the records carry them."""
    records = list(records)
    return METRICS + tuple(
        metric
        for metric in ALLOCATION_METRICS
        if any(metric in record for record in records)
    )


def ratio_rows(label, metrics, numerators, denominators, rounds, describe):
    """Table rows of the per-round ratios of median numerator to median
    denominator. Returns the rounds that lack either side."""
    paired = [rnd for rnd in rounds if numerators.get(rnd) and denominators.get(rnd)]
    for metric in metrics:
        if any(
            record.get(metric) is None
            for rnd in paired
            for side in (numerators, denominators)
            for record in side[rnd]
        ):
            print(f"| {label} | {metric} | unavailable | — | — |")
            continue
        ratios = []
        for rnd in paired:
            ours = statistics.median(r[metric] for r in numerators[rnd])
            ref = statistics.median(r[metric] for r in denominators[rnd])
            if ours <= 0 or ref <= 0:
                fail(
                    f"{describe} round {rnd}: median {metric} is {ours} over {ref}; "
                    "a ratio needs positive values"
                )
            ratios.append(ours / ref)
        if not ratios:
            continue
        low, high = bootstrap(ratios)
        above = sum(ratio > 1 for ratio in ratios)
        print(
            f"| {label} | {metric} | {geomean(ratios):.3f} | "
            f"[{low:.3f}, {high:.3f}] | {above}/{len(ratios)} |"
        )
    return [rnd for rnd in rounds if rnd not in paired]


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
    parser.add_argument(
        "--pair",
        action="append",
        default=[],
        metavar="A:B",
        help="also compare workload A to workload B within each build",
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
    pairs = []
    for pair in args.pair:
        numerator, separator, denominator = pair.partition(":")
        missing = [w for w in (numerator, denominator) if w not in workloads]
        if not separator or missing:
            fail(
                f"--pair takes A:B of recorded workloads, got {pair!r}; "
                f"recorded workloads are {', '.join(workloads)}"
            )
        pairs.append((numerator, denominator))
    timebase = (
        None if args.legacy_mach_timebase is None else float(args.legacy_mach_timebase)
    )

    def by_round(workload, build):
        return {rnd: groups.get((workload, build, rnd)) for rnd in rounds}

    for workload in workloads:
        metrics = metrics_of(r for r in samples if r["workload"] == workload)
        print(f"## {workload}")
        print("| build | metric | ratio to reference | 95% interval | rounds > 1 |")
        print("|---|---|---|---|---|")
        dropped = {}
        for build in builds:
            missing = ratio_rows(
                build,
                metrics,
                by_round(workload, build),
                by_round(workload, args.reference),
                rounds,
                f"{workload} {build} / {args.reference}",
            )
            if missing:
                dropped[build] = missing
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
        if metrics[len(METRICS) :]:
            print(
                "| build | median allocations | median allocated MiB | "
                "median peak live growth MiB |"
            )
            print("|---|---|---|---|")
            for build in [args.reference] + builds:
                records = [
                    r for rnd in rounds for r in groups.get((workload, build, rnd), [])
                ]
                if not records:
                    continue
                cells = []
                for metric, scale, digits in (
                    ("allocations", 1, 0),
                    ("allocated_bytes", 1 << 20, 1),
                    ("peak_live_growth_bytes", 1 << 20, 1),
                ):
                    values = [r.get(metric) for r in records]
                    cells.append(
                        "unavailable"
                        if any(value is None for value in values)
                        else f"{statistics.median(values) / scale:.{digits}f}"
                    )
                print(f"| {build} | {' | '.join(cells)} |")
            print()

    for numerator, denominator in pairs:
        metrics = metrics_of(
            r for r in samples if r["workload"] in (numerator, denominator)
        )
        print(f"## {numerator} / {denominator}")
        print("| build | metric | ratio | 95% interval | rounds > 1 |")
        print("|---|---|---|---|---|")
        dropped = {}
        for build in [args.reference] + builds:
            missing = ratio_rows(
                build,
                metrics,
                by_round(numerator, build),
                by_round(denominator, build),
                rounds,
                f"{build} {numerator} / {denominator}",
            )
            if missing:
                dropped[build] = missing
        for build, missing in dropped.items():
            print(
                f"\n{build}: rounds {', '.join(map(str, missing))} lack "
                f"{numerator} or {denominator} and are left out."
            )
        print()


if __name__ == "__main__":
    main()
