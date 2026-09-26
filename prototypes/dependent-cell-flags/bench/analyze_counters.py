#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Compare builds measured by the cell_flags_scan_counters bench.

Reads ``round<r>-<build>.jsonl`` files from one directory. For each workload it
prints, per build, the geometric mean over rounds of the per-round ratio to the
reference build (medians within a round), for wall time, retired instructions
and cycles, with a bootstrap 95% interval and how many rounds were above 1. It
also prints the average number of busy cores (CPU time / wall time).

    python3 analyze_counters.py <dir> --reference baseline --builds prototype,fix

Standard library only.
"""

import argparse
import collections
import json
import math
import random
import re
import statistics
from pathlib import Path

ROUND_FILE = re.compile(r"round(\d+)-(.+)\.jsonl$")
# ri_user_time / ri_system_time are Mach absolute-time ticks: 125/3 ns on Apple silicon.
NS_PER_TICK = 125 / 3


def load(directory):
    records = []
    for path in sorted(Path(directory).glob("round*-*.jsonl")):
        match = ROUND_FILE.search(path.name)
        if not match:
            continue
        for line in path.read_text().splitlines():
            if line.strip():
                record = json.loads(line)
                record["round"] = int(match.group(1))
                records.append(record)
    return records


def geomean(values):
    return math.exp(statistics.mean(math.log(v) for v in values))


def bootstrap(values, draws=5000, seed=1):
    rng = random.Random(seed)
    means = sorted(geomean([rng.choice(values) for _ in values]) for _ in range(draws))
    return means[int(0.025 * draws)], means[int(0.975 * draws)]


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("directory")
    parser.add_argument("--reference", default="baseline")
    parser.add_argument(
        "--builds", required=True, help="comma-separated builds to compare"
    )
    args = parser.parse_args()

    records = load(args.directory)
    groups = collections.defaultdict(list)
    for record in records:
        groups[(record["workload"], record["build"], record["round"])].append(record)
    rounds = sorted({record["round"] for record in records})
    workloads = sorted({record["workload"] for record in records})
    builds = args.builds.split(",")

    for workload in workloads:
        print(f"## {workload}")
        print("| build | metric | ratio to reference | 95% interval | rounds > 1 |")
        print("|---|---|---|---|---|")
        for build in builds:
            for metric in ("wall_ns", "instructions", "cycles"):
                ratios = []
                for rnd in rounds:
                    ours = groups.get((workload, build, rnd))
                    ref = groups.get((workload, args.reference, rnd))
                    if ours and ref:
                        ratios.append(
                            statistics.median(r[metric] for r in ours)
                            / statistics.median(r[metric] for r in ref)
                        )
                if not ratios:
                    continue
                low, high = bootstrap(ratios)
                above = sum(ratio > 1 for ratio in ratios)
                print(
                    f"| {build} | {metric} | {geomean(ratios):.3f} | "
                    f"[{low:.3f}, {high:.3f}] | {above}/{len(ratios)} |"
                )
        print()
        print("| build | median wall ms | p10 wall ms | busy cores |")
        print("|---|---|---|---|")
        for build in [args.reference] + builds:
            samples = [
                r for rnd in rounds for r in groups.get((workload, build, rnd), [])
            ]
            if not samples:
                continue
            walls = [r["wall_ns"] / 1e6 for r in samples]
            busy = statistics.median(
                (r["user_time"] + r["system_time"]) * NS_PER_TICK / r["wall_ns"]
                for r in samples
            )
            print(
                f"| {build} | {statistics.median(walls):.2f} | "
                f"{statistics.quantiles(walls, n=10)[0]:.2f} | {busy:.2f} |"
            )
        print()


if __name__ == "__main__":
    main()
