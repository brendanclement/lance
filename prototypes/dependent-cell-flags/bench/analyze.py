#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Summarize cell-flag benchmark JSONL into markdown tables (stdlib only).

Reads `round<r>-<build>.jsonl` (plus `env.json`, `runs.tsv`) from each results
directory given, and prints per (build, workload, variant, cache) sample counts,
medians, min, max and p95 (n >= 20), paired per-round median ratios between
builds, and the flag-specific tables. Every group in the input appears in the
output; nothing is filtered for being unfavorable.

    analyze.py results/1m [results/10m ...] [--out analysis.md] [--report REPORT.md]

`--report` replaces the `<!-- BEGIN results:<scale> -->` ... `<!-- END ... -->`
section of that file (creating it if needed) with the tables.
"""

import argparse
import json
import math
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

BASELINE, PROTOTYPE, FLAGS = "baseline", "prototype", "prototype-flags"
BUILD_ORDER = [BASELINE, PROTOTYPE, FLAGS]
ROUND_FILE = re.compile(r"round(\d+)-(.+)\.jsonl$")

# Flag workload -> the no-flag regression workload it is compared with. Reads
# are mapped by variant instead (see FLAG_READ_VARIANTS).
FLAG_PAIRS = {
    "flagged_update_sparse": "update_sparse",
    "flagged_update_dense": "update_dense",
    "flagged_update_unrelated_sparse": "update_unrelated_sparse",
    "flagged_merge_insert_partial_sparse": "merge_insert_partial_sparse",
    "flagged_refresh_clean": "refresh_permissive_clean",
    "flagged_publish_after_k_commits": "publish_after_k_commits",
}
FLAG_READ_VARIANTS = {"all_true": "populated", "partial_1pct": "null_1pct"}
READS = [
    "scan_summary_full",
    "filter_summary_is_null_count",
    "count_summary_vs_star",
    "filter_id_range_project_summary",
    "take_random_1k",
]


def load(results_dir):
    records = []
    for path in sorted(results_dir.glob("round*.jsonl")):
        match = ROUND_FILE.search(path.name)
        if not match:
            continue
        round_id = int(match.group(1))
        with path.open() as lines:
            for number, line in enumerate(lines, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    record = json.loads(line)
                except ValueError as error:
                    sys.exit(f"{path}:{number}: {error}")
                record["round"] = round_id
                records.append(record)
    env_path = results_dir / "env.json"
    env = json.loads(env_path.read_text()) if env_path.exists() else {}
    runs_path = results_dir / "runs.tsv"
    runs = runs_path.read_text().splitlines() if runs_path.exists() else []
    return records, env, runs


def p95(values):
    ordered = sorted(values)
    return ordered[math.ceil(0.95 * len(ordered)) - 1]


def stats(values):
    values = [value for value in values if value is not None]
    if not values:
        return None
    return {
        "n": len(values),
        "median": statistics.median(values),
        "min": min(values),
        "max": max(values),
        "p95": p95(values) if len(values) >= 20 else None,
    }


def fmt(value, digits=2):
    if value is None:
        return "-"
    if isinstance(value, float) and not value.is_integer():
        return f"{value:.{digits}f}"
    return f"{int(value):,}"


def fmt_count(value):
    """Bytes, rows and request counts: medians of an even n are rounded."""
    return "-" if value is None else f"{round(value):,}"


def fmt_ratio(value):
    return "-" if value is None else f"{value:.3f}"


def table(headers, rows):
    lines = [
        "| " + " | ".join(headers) + " |",
        "|" + "|".join("---" for _ in headers) + "|",
    ]
    for row in rows:
        lines.append("| " + " | ".join(str(cell) for cell in row) + " |")
    return "\n".join(lines)


def field(record, name):
    if name in record:
        return record[name]
    return record.get("extra", {}).get(name)


class Groups:
    """Records grouped by (build, workload, variant, cache_state)."""

    def __init__(self, records):
        self.groups = defaultdict(list)
        self.order = []
        for record in records:
            key = (
                record["build"],
                record["workload"],
                record["variant"],
                record["cache_state"],
            )
            if key not in self.groups:
                self.order.append(key)
            self.groups[key].append(record)

    def get(self, build, workload, variant, cache):
        return self.groups.get((build, workload, variant, cache), [])

    def keys(self):
        return sorted(
            self.order,
            key=lambda key: (
                BUILD_ORDER.index(key[0])
                if key[0] in BUILD_ORDER
                else len(BUILD_ORDER),
                self.order.index(key),
            ),
        )

    def workload_keys(self, build=None):
        """(workload, variant, cache) in first-seen order."""
        seen = []
        for key in self.order:
            if build is not None and key[0] != build:
                continue
            if key[1:] not in seen:
                seen.append(key[1:])
        return seen


def metric(records, name):
    return stats([field(record, name) for record in records])


def median_of(records, name):
    result = metric(records, name)
    return None if result is None else result["median"]


def paired_ratios(numerator, denominator, name="wall_ms"):
    """Per-round ratio of medians, numerator / denominator, for rounds in both."""
    by_round = lambda records: {  # noqa: E731
        round_id: [
            field(record, name) for record in records if record["round"] == round_id
        ]
        for round_id in {record["round"] for record in records}
    }
    top, bottom = by_round(numerator), by_round(denominator)
    ratios = []
    for round_id in sorted(set(top) & set(bottom)):
        top_values = [value for value in top[round_id] if value is not None]
        bottom_values = [value for value in bottom[round_id] if value is not None]
        if top_values and bottom_values and statistics.median(bottom_values) > 0:
            ratios.append(
                statistics.median(top_values) / statistics.median(bottom_values)
            )
    return ratios


def ratio_cells(numerator, denominator, name="wall_ms"):
    ratios = paired_ratios(numerator, denominator, name)
    if not ratios:
        return "-", "-"
    return fmt_ratio(statistics.median(ratios)), " ".join(
        fmt_ratio(ratio) for ratio in ratios
    )


def outcomes(records):
    labels = []
    for record in records:
        outcome = record.get("outcome")
        if outcome is None:
            continue
        stale = record.get("stale_rows")
        label = f"{outcome}(stale={stale})" if stale else outcome
        if label not in labels:
            labels.append(label)
    return ",".join(labels) or "-"


def timing_cells(result):
    if result is None:
        return ["-"] * 5
    return [
        result["n"],
        fmt(result["median"]),
        fmt(result["min"]),
        fmt(result["max"]),
        fmt(result["p95"]),
    ]


def section_environment(env, runs, records):
    lines = ["### Environment", ""]
    if env:
        keys = [
            "scale",
            "profile",
            "hw_model",
            "cpu_brand",
            "ncpu",
            "memsize_bytes",
            "macos",
            "rustc_baseline",
            "rustc_prototype",
            "baseline_sha",
            "prototype_sha",
            "rounds",
            "flag_every_round",
            "run_nice",
            "config",
        ]
        rows = [
            [
                key,
                f"`{json.dumps(env[key]) if isinstance(env[key], dict) else env[key]}`",
            ]
            for key in keys
            if key in env
        ]
        for key in ["baseline_status", "prototype_status"]:
            if env.get(key):
                rows.append([key, "dirty: " + env[key].replace("\n", "; ")])
        lines.append(table(["key", "value"], rows))
    else:
        lines.append("No env.json found.")
    if runs:
        lines += ["", "Runs (load average before each run):", "", "```", *runs, "```"]
    udf = sorted({record.get("udf_iterations") for record in records} - {None})
    lines += [
        "",
        f"Records: {len(records)}. Simulated UDF iterations: {udf}. `udf_ms` is the labeled "
        "simulated UDF (FNV-1a rounds), reported apart from read, stage and commit time.",
        "`warm` reuses one Session; `fresh-session` opens with a new Session per sample. The OS "
        "page cache is not controlled, so no state here is a cold read.",
    ]
    return "\n".join(lines)


def section_regression(groups):
    lines = [
        "### Regression: tables without cell flags, baseline vs prototype",
        "",
        "Same harness binary source on both builds. Ratio = prototype / baseline median "
        "wall_ms, per round (paired) and the median of those. Conflict workloads are **not "
        "correctness-equivalent** (see notes): compare latency and IO only.",
        "",
    ]
    rows = []
    for workload, variant, cache in groups.workload_keys():
        base = groups.get(BASELINE, workload, variant, cache)
        proto = groups.get(PROTOTYPE, workload, variant, cache)
        if not base and not proto:
            continue
        median_ratio, per_round = ratio_cells(proto, base)
        not_equivalent = any(
            "NOT CORRECTNESS-EQUIVALENT" in (r.get("notes") or "") for r in base + proto
        )
        rows.append(
            [
                workload + (" †" if not_equivalent else ""),
                variant,
                cache,
                *timing_cells(metric(base, "wall_ms")),
                *timing_cells(metric(proto, "wall_ms")),
                median_ratio,
                per_round,
                fmt_count(median_of(base, "manifest_bytes")),
                fmt_count(median_of(proto, "manifest_bytes")),
                fmt_count(median_of(base, "txn_bytes")),
                fmt_count(median_of(proto, "txn_bytes")),
                fmt_count(median_of(base, "read_iops")),
                fmt_count(median_of(proto, "read_iops")),
                fmt_count(median_of(base, "written_bytes")),
                fmt_count(median_of(proto, "written_bytes")),
                outcomes(base) + " / " + outcomes(proto),
            ]
        )
    headers = [
        "workload",
        "variant",
        "cache",
        "base n",
        "base med ms",
        "base min",
        "base max",
        "base p95",
        "proto n",
        "proto med ms",
        "proto min",
        "proto max",
        "proto p95",
        "ratio (med)",
        "ratio per round",
        "base manifest B",
        "proto manifest B",
        "base txn B",
        "proto txn B",
        "base r_iops",
        "proto r_iops",
        "base written B",
        "proto written B",
        "outcome base / proto",
    ]
    lines.append(
        table(headers, rows) if rows else "No baseline or prototype regression records."
    )
    lines += [
        "",
        "† baseline is NOT correctness-equivalent to the prototype (no dependency tracking).",
    ]
    return "\n".join(lines)


def flag_pairs(groups):
    """(flag key, no-flag key) pairs, and flag keys with no counterpart."""
    pairs, unpaired = [], []
    for workload, variant, cache in groups.workload_keys(FLAGS):
        counterpart = None
        if workload in FLAG_PAIRS:
            base_variant = "default"
            if workload == "flagged_publish_after_k_commits":
                base_variant = variant
            elif workload == "flagged_refresh_clean" and variant != "groups=1":
                base_variant = None
            if base_variant is not None:
                counterpart = (FLAG_PAIRS[workload], base_variant, cache)
        elif workload in READS:
            state, _, api = variant.partition(":")
            if state in FLAG_READ_VARIANTS:
                base_variant = FLAG_READ_VARIANTS[state] + (":" + api if api else "")
                counterpart = (workload, base_variant, cache)
        if counterpart:
            pairs.append(((workload, variant, cache), counterpart))
        elif not workload.startswith(("flagged_refresh_conflicts", "flag_state_size")):
            unpaired.append((workload, variant, cache))
    return pairs, unpaired


def section_flag_overhead(groups):
    lines = [
        "### Flag overhead: prototype without flags vs prototype with dependent masking flags",
        "",
        "Both from the prototype build. Ratio = flags / no-flags median wall_ms (paired by "
        "round). Updates run on a fully published table; `groups=2` adds a second output "
        "sharing `body`. Reads compare `populated` with `all_true` and `null_1pct` (stored "
        "NULLs) with `partial_1pct` (masked by false flags), on the same ids.",
        "",
    ]
    pairs, unpaired = flag_pairs(groups)
    rows = []
    for (workload, variant, cache), (base_workload, base_variant, _) in pairs:
        flags = groups.get(FLAGS, workload, variant, cache)
        plain = groups.get(PROTOTYPE, base_workload, base_variant, cache)
        median_ratio, per_round = ratio_cells(flags, plain)
        rows.append(
            [
                workload,
                variant,
                cache,
                f"{base_workload}:{base_variant}",
                *timing_cells(metric(plain, "wall_ms")),
                *timing_cells(metric(flags, "wall_ms")),
                median_ratio,
                per_round,
                fmt(median_of(plain, "commit_ms")),
                fmt(median_of(flags, "commit_ms")),
                fmt_count(median_of(plain, "manifest_bytes")),
                fmt_count(median_of(flags, "manifest_bytes")),
                fmt_count(median_of(plain, "txn_bytes")),
                fmt_count(median_of(flags, "txn_bytes")),
                fmt_count(median_of(plain, "written_bytes")),
                fmt_count(median_of(flags, "written_bytes")),
            ]
        )
    for workload, variant, cache in unpaired:
        flags = groups.get(FLAGS, workload, variant, cache)
        rows.append(
            [
                workload,
                variant,
                cache,
                "(no counterpart)",
                *["-"] * 5,
                *timing_cells(metric(flags, "wall_ms")),
                "-",
                "-",
                "-",
                fmt(median_of(flags, "commit_ms")),
                "-",
                fmt_count(median_of(flags, "manifest_bytes")),
                "-",
                fmt_count(median_of(flags, "txn_bytes")),
                "-",
                fmt_count(median_of(flags, "written_bytes")),
            ]
        )
    headers = [
        "flag workload",
        "variant",
        "cache",
        "compared with",
        "plain n",
        "plain med ms",
        "plain min",
        "plain max",
        "plain p95",
        "flags n",
        "flags med ms",
        "flags min",
        "flags max",
        "flags p95",
        "ratio (med)",
        "ratio per round",
        "plain commit ms",
        "flags commit ms",
        "plain manifest B",
        "flags manifest B",
        "plain txn B",
        "flags txn B",
        "plain written B",
        "flags written B",
    ]
    lines.append(table(headers, rows) if rows else "No flag records.")

    effect_rows = []
    for workload, variant, cache in groups.workload_keys(FLAGS):
        if not workload.startswith(("flagged_update", "flagged_merge_insert")):
            continue
        records = groups.get(FLAGS, workload, variant, cache)
        first = records[0]["extra"]
        effect_rows.append(
            [
                workload,
                variant,
                fmt_count(median_of(records, "rows_written")),
                json.dumps(first.get("flags_cleared")),
                json.dumps(first.get("flags_carried")),
                json.dumps(first.get("derived_invalidation_rows")),
                fmt_count(first.get("moved_rows")),
            ]
        )
    if effect_rows:
        lines += [
            "",
            "Flag effects of the source writes (asserted exact in every sample; first sample shown):",
            "",
            table(
                [
                    "workload",
                    "variant",
                    "rows written",
                    "flags cleared",
                    "flags carried",
                    "derived invalidation rows",
                    "moved rows",
                ],
                effect_rows,
            ),
        ]
    return "\n".join(lines)


def section_conflicts(groups):
    lines = [
        "### Refresh under concurrent source writes",
        "",
        "A full `summary` refresh staged at V, K source commits of 10 scattered rows each, "
        "then the publication at V. `commit ms` is the conflict-checked publication commit. "
        "`merge_insert_output_in_place` writes `summary` itself (an output override); it is "
        "the only source write here that defers whole groups.",
        "",
    ]
    rows = []
    for workload, variant, cache in groups.workload_keys(FLAGS):
        if not re.fullmatch(r"flagged_refresh_conflicts_\d+", workload):
            continue
        records = groups.get(FLAGS, workload, variant, cache)
        first = records[0]["extra"]
        commit = metric(records, "commit_ms")
        rows.append(
            [
                workload,
                variant,
                len(records),
                outcomes(records),
                fmt_count(median_of(records, "rows_assigned")),
                fmt_count(median_of(records, "rows_published")),
                fmt_count(median_of(records, "rows_deferred")),
                json.dumps(first.get("rows_deferred_by_reason", {})),
                fmt_count(median_of(records, "fragments_deferred")),
                json.dumps(first.get("fragments_deferred_by_reason", {})),
                fmt_count(median_of(records, "valid_rows_in_deferred_groups")),
                fmt(commit and commit["median"]),
                fmt(commit and commit["p95"]),
                fmt(median_of(records, "wall_ms")),
                fmt(median_of(records, "udf_ms")),
                fmt(median_of(records, "source_commits_ms")),
            ]
        )
    headers = [
        "workload",
        "variant",
        "n",
        "outcome",
        "rows assigned",
        "rows published",
        "rows deferred",
        "deferred by reason",
        "fragments deferred",
        "fragment reasons",
        "valid rows in deferred groups",
        "commit med ms",
        "commit p95",
        "wall med ms",
        "udf ms",
        "source commits ms",
    ]
    lines.append(table(headers, rows) if rows else "No flagged conflict records.")

    base_rows = []
    for workload, variant, cache in groups.workload_keys():
        if not workload.startswith("refresh_permissive_conflicts"):
            continue
        for build in (BASELINE, PROTOTYPE):
            records = groups.get(build, workload, variant, cache)
            if records:
                base_rows.append(
                    [
                        build,
                        workload,
                        variant,
                        len(records),
                        outcomes(records),
                        fmt_count(median_of(records, "rows_published")),
                        fmt_count(median_of(records, "stale_rows")),
                        fmt(median_of(records, "commit_ms")),
                        fmt(median_of(records, "wall_ms")),
                    ]
                )
    if base_rows:
        lines += [
            "",
            "Permissive publication without flags (regression harness), **NOT "
            "correctness-equivalent**: it has no dependency tracking, so in-place writes publish "
            "stale summaries (`stale`) and row-moving writes are rejected by main's rules:",
            "",
            table(
                [
                    "build",
                    "workload",
                    "variant",
                    "n",
                    "outcome",
                    "rows published",
                    "stale rows",
                    "commit med ms",
                    "wall med ms",
                ],
                base_rows,
            ),
        ]
    return "\n".join(lines)


def section_followups(groups):
    lines = [
        "### Follow-up refresh to completion: saved and repeated computation",
        "",
        "From the state the conflicted publication left, each strategy refreshes every pending "
        "row and publishes; afterwards every flag is asserted true and every value equal to the "
        "UDF of its current inputs. `recompute_all_pending` ignores the report; "
        "`reuse_valid_staged` reuses staged values of `PublicationReport::reusable_rows` (Reject "
        "returns an error, so it has no report). `total UDF rows` = rows the conflicted "
        "publication computed + rows the follow-up recomputed; with N rows, anything above N is "
        "repeated computation.",
        "",
    ]
    rows = []
    for workload, variant, cache in groups.workload_keys(FLAGS):
        if not workload.endswith("_followup"):
            continue
        records = groups.get(FLAGS, workload, variant, cache)
        conflict_variant = variant.rsplit(":", 1)[0]
        conflicted = groups.get(
            FLAGS, workload[: -len("_followup")], conflict_variant, cache
        )
        first_pass = median_of(conflicted, "rows_assigned")
        recomputed = median_of(records, "rows_recomputed")
        total = (
            None
            if first_pass is None or recomputed is None
            else first_pass + recomputed
        )
        rows.append(
            [
                workload,
                variant,
                len(records),
                fmt_count(first_pass),
                fmt_count(median_of(conflicted, "rows_published")),
                fmt_count(recomputed),
                fmt_count(median_of(records, "rows_reused")),
                fmt_count(median_of(records, "rows_copied_through")),
                fmt_count(total),
                fmt(median_of(records, "udf_ms")),
                fmt(median_of(records, "stage_ms")),
                fmt(median_of(records, "commit_ms")),
                fmt(median_of(records, "wall_ms")),
                fmt_count(median_of(records, "written_bytes")),
            ]
        )
    headers = [
        "workload",
        "variant",
        "n",
        "rows computed first",
        "rows published first",
        "rows recomputed",
        "rows reused",
        "rows copied through",
        "total UDF rows",
        "udf med ms",
        "stage med ms",
        "commit med ms",
        "wall med ms",
        "written B",
    ]
    lines.append(table(headers, rows) if rows else "No follow-up records.")
    return "\n".join(lines)


def section_publish_after(groups):
    lines = [
        "### Publication commit latency after K unrelated commits",
        "",
        "`wall = commit_ms` of the publication only. Conflict checks read every transaction "
        "since the read version.",
        "",
    ]
    rows = []
    variants = []
    for build, workload in (
        (BASELINE, "publish_after_k_commits"),
        (PROTOTYPE, "publish_after_k_commits"),
        (FLAGS, "flagged_publish_after_k_commits"),
    ):
        for key_workload, variant, cache in groups.workload_keys(build):
            if key_workload == workload and variant not in variants:
                variants.append(variant)
    for variant in variants:
        row = [variant]
        for build, workload in (
            (BASELINE, "publish_after_k_commits"),
            (PROTOTYPE, "publish_after_k_commits"),
            (FLAGS, "flagged_publish_after_k_commits"),
        ):
            result = metric(groups.get(build, workload, variant, "warm"), "commit_ms")
            row += [
                fmt(result and result["n"]),
                fmt(result and result["median"]),
                fmt(result and result["p95"]),
            ]
        base = groups.get(PROTOTYPE, "publish_after_k_commits", variant, "warm")
        flags = groups.get(FLAGS, "flagged_publish_after_k_commits", variant, "warm")
        row.append(ratio_cells(flags, base, "commit_ms")[0])
        rows.append(row)
    headers = [
        "k",
        "base n",
        "base med ms",
        "base p95",
        "proto n",
        "proto med ms",
        "proto p95",
        "flags n",
        "flags med ms",
        "flags p95",
        "flags / proto (med)",
    ]
    lines.append(table(headers, rows) if rows else "No publish_after_k records.")
    return "\n".join(lines)


def section_flag_state(groups):
    lines = [
        "### Flag state size as the true set fragments",
        "",
        "Head after one in-place `body` write invalidating the given fraction of scattered "
        "rows. `groups=0` is the unflagged control with the same data files. `wall` is a "
        "fresh-session open (OS page cache not controlled).",
        "",
    ]
    rows = []
    for workload, variant, cache in groups.workload_keys(FLAGS):
        if workload != "flag_state_size":
            continue
        records = groups.get(FLAGS, workload, variant, cache)
        first = records[0]
        open_ms = metric(records, "wall_ms")
        rows.append(
            [
                variant,
                len(records),
                fmt_count(first["manifest_bytes"]),
                fmt_count(first["txn_bytes"]),
                fmt_count(first["extra"].get("flag_state_serialized_bytes")),
                fmt(open_ms["median"]),
                fmt(open_ms["min"]),
                fmt(open_ms["max"]),
                fmt(open_ms["p95"]),
                fmt_count(median_of(records, "read_iops")),
                fmt_count(median_of(records, "read_bytes")),
            ]
        )
    headers = [
        "variant",
        "n",
        "manifest B",
        "txn B",
        "flag state B (serialized true sets)",
        "open med ms",
        "open min",
        "open max",
        "open p95",
        "open r_iops",
        "open read B",
    ]
    lines.append(table(headers, rows) if rows else "No flag_state_size records.")
    return "\n".join(lines)


def section_all(groups):
    lines = [
        "### Every group (wall_ms)",
        "",
        "One row per (build, workload, variant, cache) in the input.",
        "",
    ]
    rows = []
    for build, workload, variant, cache in groups.keys():
        records = groups.get(build, workload, variant, cache)
        rows.append(
            [
                build,
                workload,
                variant,
                cache,
                *timing_cells(metric(records, "wall_ms")),
                fmt(median_of(records, "udf_ms")),
                fmt(median_of(records, "stage_ms")),
                fmt(median_of(records, "commit_ms")),
                outcomes(records),
            ]
        )
    headers = [
        "build",
        "workload",
        "variant",
        "cache",
        "n",
        "med ms",
        "min",
        "max",
        "p95",
        "udf med ms",
        "stage med ms",
        "commit med ms",
        "outcome",
    ]
    lines.append(table(headers, rows))
    return "\n".join(lines)


def analyze(results_dir):
    records, env, runs = load(results_dir)
    if not records:
        sys.exit(f"no round*.jsonl records under {results_dir}")
    scale = env.get("scale", results_dir.name)
    groups = Groups(records)
    parts = [
        f"## Results: {scale}",
        f"Source: `{results_dir}`",
        section_environment(env, runs, records),
        section_regression(groups),
        section_flag_overhead(groups),
        section_conflicts(groups),
        section_followups(groups),
        section_publish_after(groups),
        section_flag_state(groups),
        section_all(groups),
    ]
    return scale, "\n\n".join(parts) + "\n"


def update_report(report, scale, body):
    begin, end = f"<!-- BEGIN results:{scale} -->", f"<!-- END results:{scale} -->"
    text = (
        report.read_text()
        if report.exists()
        else (
            "# Dependent cell flags: benchmark report\n\n"
            "Generated by `analyze.py`; each scale's section is replaced when it is re-run. See "
            "`README.md` for the commands and what each workload measures.\n"
        )
    )
    block = f"{begin}\n{body}{end}\n"
    if begin in text and end in text:
        start = text.index(begin)
        stop = text.index(end) + len(end)
        text = text[:start] + block.rstrip("\n") + text[stop:]
    else:
        text = text.rstrip("\n") + "\n\n" + block
    report.write_text(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("results", nargs="+", type=Path)
    parser.add_argument(
        "--out", type=Path, help="write the markdown here (default: stdout)"
    )
    parser.add_argument(
        "--report", type=Path, help="update this REPORT.md's scale sections"
    )
    args = parser.parse_args()
    outputs = []
    for results_dir in args.results:
        scale, body = analyze(results_dir)
        outputs.append(body)
        if args.report:
            update_report(args.report, scale, body)
    text = "\n".join(outputs)
    if args.out:
        args.out.write_text(text)
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
