#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Compact cost tables from cell_flags_costs rotations.

    python3 summarize_costs.py <timed dir> [--allocations <dir>] [--only REGEX]

Every process of the rotation is one replicate: the build and its
identical-binary control (``<build>-copy``) run the same executable, so both
count. For each comparison it prints the median wall time of each side, the
range of per-process medians (the run-to-run variation), and the geometric
mean over processes of the per-process ratio of medians with a bootstrap 95%
interval. A ``.phase`` suffix names one phase of a workload. With
``--allocations``, a rotation run with ``BENCH_COUNT_ALLOCATIONS=1``, it adds
each side's median allocation volume and peak live growth, and from the
timed rotation the median object-store bytes and the manifest and
transaction sizes. The control's ratio to its build, per workload, is the
noise floor (``--control``).

Standard library only; reads records with analyze_counters.load.
"""

import argparse
import collections
import re
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from analyze_counters import bootstrap, geomean, load  # noqa: E402

READS = [
    (f"{read} {state}", f"{read}_{flagged}", f"{read}_{plain}")
    for read in ("scan", "filter_null", "take")
    for state, flagged, plain in (
        ("all true / no flags", "one", "plain"),
        ("all true after moves / same moves", "ready_moved", "plain_moved"),
        ("1% masked / 1% NULL", "partial_1pct", "null_1pct"),
        ("1% masked / no NULLs", "partial_1pct", "plain"),
        ("50% masked / 50% NULL", "partial_50pct", "null_50pct"),
        ("all masked / all NULL", "masked", "null_all"),
    )
]
TOPOLOGIES = ("one", "shared", "chain")
SHAPES = ("moving_sparse", "moving_dense", "inplace_sparse")
# (flagged phase, plain phase): a plain refresh finds its rows with a filter
# (`locate`) and writes with merge_insert (`merge`).
CYCLE_PHASES = (
    ("update", "update"),
    ("read_pending", "read_pending"),
    ("find", None),
    ("locate", "locate"),
    ("stage", "merge"),
    ("commit", "commit"),
    ("read_published", "read_published"),
)
WRITES = [
    (
        f"unrelated {kind} write, {topology} / plain",
        f"unrelated_{kind}_{topology}",
        f"unrelated_{kind}_plain",
    )
    for kind in ("moving", "inplace")
    for topology in TOPOLOGIES
]
CYCLES = [
    (
        f"{shape} {topology}: {phase or 'whole cycle'}",
        f"cycle_{shape}_{topology}{'.' + phase if phase else ''}",
        f"cycle_{shape}_plain_{topology}{'.' + plain if phase else ''}",
    )
    for shape in SHAPES
    for topology in TOPOLOGIES
    for phase, plain in ((None, None),) + CYCLE_PHASES
    if plain or not phase
] + [
    (
        f"inplace_sparse plain one: RewriteColumns / RewriteRows{': ' + phase if phase else ''}",
        f"cycle_inplace_sparse_plain_one{'.' + phase if phase else ''}",
        f"cycle_inplace_sparse_plain_one_rewrite_rows{'.' + phase if phase else ''}",
    )
    for phase in (None, "merge", "commit", "read_published")
]
# The same 100 computed rows staged into one new 100-row fragment (moving)
# or merged into every 100k-row fragment they sit in (in place); and every
# row computed (backfill) against nearly every row copied through (in place),
# writing the same fragments.
WHOLE_FRAGMENT = [
    (
        f"{topology}: in-place stage / moving stage",
        f"cycle_inplace_sparse_{topology}.stage",
        f"cycle_moving_sparse_{topology}.stage",
    )
    for topology in TOPOLOGIES
] + [
    (
        f"{topology}: in-place stage (rows copied) / backfill stage (rows computed)",
        f"cycle_inplace_sparse_{topology}.stage",
        f"backfill_{topology}.stage",
    )
    for topology in TOPOLOGIES
]
BACKFILLS = [
    (
        f"backfill {topology}: {phase or 'whole'}",
        f"backfill_{topology}{'.' + phase if phase else ''}",
        f"backfill_plain_{topology}{'.' + phase if phase else ''}",
    )
    for topology in TOPOLOGIES
    for phase in (None, "locate", "stage", "commit")
]
CONFLICTS = [
    (
        f"{race}{suffix}: {policy} {part}",
        f"conflict_{race}_{policy}{suffix}.{part}",
        f"conflict_{race}_{policy}{suffix}.first",
    )
    for race, suffix in (
        ("moving_k1", ""),
        ("inplace_k1", ""),
        ("inplace_k8", ""),
        ("inplace_k1", "_chain"),
    )
    for policy in ("reject", "skip")
    for part in ("publish", "extra")
]
APPENDS = [
    (
        f"publish after {k} appends: {side}",
        f"after_appends_k{k}_{side}.commit",
        f"after_appends_k0_{side}.commit",
    )
    for k in (8, 32)
    for side in ("one", "plain_one")
] + [
    (
        f"publish after {k} appends: one / plain",
        f"after_appends_k{k}_one.commit",
        f"after_appends_k{k}_plain_one.commit",
    )
    for k in (0, 8, 32)
]
HISTORY = [
    (f"{op} {table} / {plain}", f"{op}_{table}", f"{op}_{plain}")
    for op in ("reopen", "append", "update")
    for table, plain in (
        ("one", "plain"),
        ("shared", "plain"),
        ("chain", "plain"),
        ("one_h16", "plain_h16"),
        ("one_h64", "plain_h64"),
        ("shared_h64", "plain_h64"),
        ("chain_h64", "plain_h64"),
        ("one_dense", "plain_dense"),
        ("one_h16r", "plain_h16r"),
        ("one_h64r", "plain_h64r"),
        ("one_h64r", "one"),
        ("plain_h64r", "plain"),
        ("plain_h16", "plain"),
        ("plain_h64", "plain"),
        ("one_h16", "one"),
        ("one_h64", "one"),
    )
]
SECTIONS = [
    ("Scalar reads", READS),
    ("Unrelated writes", WRITES),
    ("Refresh cycles: flagged / plain merge_insert", CYCLES),
    ("Whole-fragment replacement", WHOLE_FRAGMENT),
    ("Backfills: publication / plain DataReplacement", BACKFILLS),
    ("Conflicts", CONFLICTS),
    ("Publication after unrelated appends", APPENDS),
    ("Accumulated history", HISTORY),
]


# Sums of a sample's phases, as extra phases: `locate`, finding and reading
# what to recompute; `first`, a raced refresh's work up to its publication;
# `extra`, what its retry or follow-up adds.
DERIVED = {
    "locate": lambda names: [n for n in names if n in ("find", "read_inputs")],
    "first": lambda names: [n for n in names if n in ("find", "read_inputs", "stage")],
    "extra": lambda names: names[names.index("publish") + 1 :]
    if "publish" in names
    else [],
}
SUMMED = (
    "wall_ns",
    "cpu_ns",
    "allocations",
    "allocated_bytes",
    "read_bytes",
    "written_bytes",
    "read_iops",
    "write_iops",
)


def add_derived(samples):
    derived = []
    for record in samples:
        names = record.get("phase_order")
        if not names:
            continue
        for name, select in DERIVED.items():
            parts = [record["phases"][n] for n in select(names)]
            if not parts:
                continue
            extra = dict(record, workload=f"{record['workload']}.{name}")
            extra.pop("phase_order")
            for metric in SUMMED:
                values = [part.get(metric) for part in parts]
                extra[metric] = None if None in values else sum(values)
            peaks = [part.get("peak_live_growth_bytes") for part in parts]
            extra["peak_live_growth_bytes"] = None if None in peaks else max(peaks)
            for size in (
                "manifest_bytes",
                "manifest_struct_bytes",
                "inline_transaction_bytes",
                "transaction_bytes",
            ):
                extra[size] = None
            derived.append(extra)
    return samples + derived


def by_process(samples, metric):
    """{workload: {(round, build): [values]}}"""
    groups = collections.defaultdict(lambda: collections.defaultdict(list))
    for record in samples:
        value = record.get(metric)
        if value is not None:
            groups[record["workload"]][(record["round"], record["build"])].append(value)
    return groups


def fmt_ms(ns):
    ms = ns / 1e6
    return f"{ms:.3f}" if ms < 1 else f"{ms:.2f}" if ms < 100 else f"{ms:.0f}"


def side(groups, workload):
    runs = groups.get(workload)
    if not runs:
        return None
    medians = [statistics.median(values) for values in runs.values()]
    pooled = statistics.median(v for values in runs.values() for v in values)
    return pooled, min(medians), max(medians), runs


def ratio(numerator_runs, denominator_runs):
    keys = sorted(set(numerator_runs) & set(denominator_runs))
    ratios = []
    for key in keys:
        ours = statistics.median(numerator_runs[key])
        ref = statistics.median(denominator_runs[key])
        if ours <= 0 or ref <= 0:
            return None
        ratios.append(ours / ref)
    if not ratios:
        return None
    low, high = bootstrap(ratios)
    return geomean(ratios), low, high, sum(r > 1 for r in ratios), len(ratios)


def median_of(samples, workload, metric):
    values = [
        r[metric]
        for r in samples
        if r["workload"] == workload and r.get(metric) is not None
    ]
    return statistics.median(values) if values else None


def mib(value):
    return "—" if value is None else f"{value / (1 << 20):.1f}"


def kib(value):
    return "—" if value is None else f"{value / (1 << 10):.1f}"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("timed")
    parser.add_argument("--allocations")
    parser.add_argument("--only", help="print only comparisons whose label matches")
    parser.add_argument(
        "--control",
        action="store_true",
        help="also print each workload's control ratio",
    )
    args = parser.parse_args()

    samples = add_derived(load(args.timed)[0])
    walls = by_process(samples, "wall_ns")
    allocated = counted = None
    if args.allocations:
        counted = add_derived(load(args.allocations)[0])
        allocated = True
    workloads = set(walls)
    for title, comparisons in SECTIONS:
        rows = [c for c in comparisons if c[1] in workloads and c[2] in workloads]
        if args.only:
            rows = [c for c in rows if re.search(args.only, c[0])]
        if not rows:
            continue
        print(f"## {title}\n")
        header = "| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 |"
        rule = "|---|---|---|---|---|---|---|---|"
        if allocated:
            header += " alloc MiB | base alloc MiB | peak MiB | base peak MiB |"
            rule += "---|---|---|---|"
        header += " read MiB | written MiB | manifest KiB | base manifest KiB |"
        # Manifest KiB is the manifest proper, without the inline transaction.
        rule += "---|---|---|---|"
        print(header)
        print(rule)
        for label, numerator, denominator in rows:
            ours, base = side(walls, numerator), side(walls, denominator)
            result = ratio(ours[3], base[3])
            cells = [
                label,
                fmt_ms(ours[0]),
                f"{fmt_ms(ours[1])}–{fmt_ms(ours[2])}",
                fmt_ms(base[0]),
                f"{fmt_ms(base[1])}–{fmt_ms(base[2])}",
            ]
            if result:
                mean, low, high, above, n = result
                cells += [f"{mean:.3f}", f"[{low:.3f}, {high:.3f}]", f"{above}/{n}"]
            else:
                cells += ["—", "—", "—"]
            if allocated:
                cells += [
                    mib(median_of(counted, numerator, "allocated_bytes")),
                    mib(median_of(counted, denominator, "allocated_bytes")),
                    mib(median_of(counted, numerator, "peak_live_growth_bytes")),
                    mib(median_of(counted, denominator, "peak_live_growth_bytes")),
                ]
            cells += [
                mib(median_of(samples, numerator, "read_bytes")),
                mib(median_of(samples, numerator, "written_bytes")),
                kib(median_of(samples, numerator, "manifest_struct_bytes")),
                kib(median_of(samples, denominator, "manifest_struct_bytes")),
            ]
            print(f"| {' | '.join(cells)} |")
        print()

    if args.control:
        print("## Control: <build>-copy / <build>, per workload\n")
        print("| workload | ratio | 95% interval |")
        print("|---|---|---|")
        builds = sorted({r["build"] for r in samples})
        copies = [b for b in builds if b.endswith("-copy")]
        for copy in copies:
            build = copy[: -len("-copy")]
            for workload in sorted(workloads):
                runs = walls[workload]
                # Every round runs each label once.
                ratios = [
                    statistics.median(runs[(rnd, copy)])
                    / statistics.median(runs[(rnd, build)])
                    for rnd in sorted({key[0] for key in runs})
                    if (rnd, copy) in runs and (rnd, build) in runs
                ]
                if ratios:
                    low, high = bootstrap(ratios)
                    print(
                        f"| {workload} | {geomean(ratios):.3f} | [{low:.3f}, {high:.3f}] |"
                    )


if __name__ == "__main__":
    main()
