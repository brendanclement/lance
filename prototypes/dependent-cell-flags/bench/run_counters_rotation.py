#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
"""Rotate builds of the cell_flags_scan_counters bench over one shared table.

Builds each checkout, takes the single bench executable cargo reports for it,
copies that executable read-only into the results directory, and runs the
copies in a 4x4 Latin-square order, so every build runs in every position
equally often. run.json records what identifies each build and the table:
- the executable's sha256;
- the source revision, the working-tree state and a patch of local changes;
- rustc and cargo versions, the profile, rustflags, features and the cargo
  configuration files that applied;
- a sha256 over every file of the table, taken before and after the rotation.

    run_counters_rotation.py --dataset /abs/table --out /abs/results \\
        --build main=/abs/lance-main --build prototype=/abs/lance-proto \\
        --build fix=/abs/lance-fix --control main

There are exactly four labels: four builds, or three builds and --control,
which runs one build's executable a second time under the label <build>-copy.
Checkouts must not be nested under another checkout, whose cargo
configuration cargo would merge into the build. Builds must have the same
rustc, profile, rustflags and features unless --allow-build-differences is
given. Create the table once with BENCH_COUNTERS_MODE=create.

Do not copy a target directory between checkouts: cargo decides freshness by
file times, so a copied target can yield another checkout's executable. The
driver refuses two builds with the same executable, but it cannot detect a
partly stale one.

Standard library only.
"""

import argparse
import datetime
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

BENCH = "cell_flags_scan_counters"
PACKAGE = "lance"
SCHEMA = 2
LATIN_SQUARE = [[0, 1, 2, 3], [1, 3, 0, 2], [2, 0, 3, 1], [3, 2, 1, 0]]
LABEL = re.compile(r"^[A-Za-z0-9._-]+$")
# Cargo writes both `release` and `bench` into target/release, and `dev` and
# `test` into target/debug.
PROFILE_DIRS = {"dev": "debug", "test": "debug", "bench": "release"}


def fail(message):
    sys.exit(f"run_counters_rotation: {message}")


def run(args, cwd=None, env=None, capture=True):
    result = subprocess.run(
        args,
        cwd=cwd,
        env=env,
        capture_output=capture,
        text=True,
        stdin=subprocess.DEVNULL,
    )
    if result.returncode != 0:
        fail(
            f"{' '.join(map(str, args))} failed ({result.returncode}): {result.stderr}"
        )
    return result.stdout


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as file:
        for chunk in iter(lambda: file.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")


def cargo_env():
    """The environment for cargo and the bench. A shared CARGO_TARGET_DIR would
    make checkouts overwrite each other's artifacts."""
    env = dict(os.environ)
    env.pop("CARGO_TARGET_DIR", None)
    return env


def build_environment():
    names = (
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTUP_TOOLCHAIN",
    )
    return {
        name: value
        for name, value in sorted(os.environ.items())
        if name in names or name.startswith(("CARGO_BUILD_", "CARGO_PROFILE_"))
    }


def cargo_configs(checkout):
    """The cargo configuration files cargo merges for a build in `checkout`:
    the checkout's own, those of its parent directories, and CARGO_HOME's."""

    def configs_in(directory):
        return [
            directory / ".cargo" / name
            for name in ("config.toml", "config")
            if (directory / ".cargo" / name).is_file()
        ]

    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    home = [
        path
        for path in (cargo_home / "config.toml", cargo_home / "config")
        if path.is_file()
    ]
    parents = [
        path
        for directory in checkout.parents
        for path in configs_in(directory)
        if path not in home
    ]
    return configs_in(checkout), parents, home


def working_tree_hash(checkout):
    """A git tree hash of the working tree, untracked files included, computed
    through a temporary index so the checkout's own index is untouched."""
    with tempfile.TemporaryDirectory() as scratch:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(scratch) / "index"))
        run(["git", "read-tree", "HEAD"], cwd=checkout, env=env)
        run(["git", "add", "-A"], cwd=checkout, env=env)
        return run(["git", "write-tree"], cwd=checkout, env=env).strip()


def source_identity(checkout, patch_path):
    status = run(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=checkout
    )
    patch = run(["git", "diff", "HEAD", "--binary"], cwd=checkout)
    patch_path.write_text(patch)
    untracked = run(
        ["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=checkout
    )
    return {
        "head": run(["git", "rev-parse", "HEAD"], cwd=checkout).strip(),
        "status": status.splitlines(),
        "working_tree": working_tree_hash(checkout),
        "patch": str(patch_path),
        "untracked": {
            name: sha256_file(checkout / name)
            for name in sorted(filter(None, untracked.split("\0")))
        },
    }


def build(name, checkout, profile, out):
    checkout = checkout.resolve()
    toplevel = run(["git", "rev-parse", "--show-toplevel"], cwd=checkout).strip()
    if Path(toplevel).resolve() != checkout:
        fail(
            f"build {name}: {checkout} is not the top level of a git checkout ({toplevel} is)"
        )
    own, parents, home = cargo_configs(checkout)
    if parents:
        fail(
            f"build {name}: {checkout} is nested under another cargo configuration "
            f"({', '.join(map(str, parents))}); cargo would merge it into this build"
        )

    source = source_identity(checkout, out / "source" / f"{name}.patch")
    env = cargo_env()
    log_path = out / "logs" / f"build-{name}.log"
    command = [
        "cargo",
        "bench",
        "-p",
        PACKAGE,
        "--bench",
        BENCH,
        "--profile",
        profile,
        "--locked",
        "--no-run",
        "--message-format=json-render-diagnostics",
    ]
    with open(log_path, "w") as log:
        result = subprocess.run(
            command,
            cwd=checkout,
            env=env,
            stdout=subprocess.PIPE,
            stderr=log,
            text=True,
            stdin=subprocess.DEVNULL,
        )
    if result.returncode != 0:
        fail(f"build {name}: cargo failed ({result.returncode}); see {log_path}")
    artifacts = []
    for line in result.stdout.splitlines():
        message = json.loads(line)
        target = message.get("target") or {}
        if (
            message.get("reason") == "compiler-artifact"
            and target.get("name") == BENCH
            and "bench" in target.get("kind", [])
            and message.get("executable")
        ):
            artifacts.append(message)
    if len(artifacts) != 1:
        found = ", ".join(artifact["executable"] for artifact in artifacts) or "none"
        fail(
            f"build {name}: expected exactly one {BENCH} executable, cargo reported {found}"
        )
    artifact = artifacts[0]
    executable = Path(artifact["executable"]).resolve()
    if checkout not in executable.parents:
        fail(
            f"build {name}: executable {executable} is outside the checkout {checkout}"
        )
    if working_tree_hash(checkout) != source["working_tree"]:
        fail(f"build {name}: the checkout changed while it was building")

    unit_hash = executable.name.rsplit("-", 1)[1]
    profile_dir = executable.parent.parent
    fingerprint_path = (
        profile_dir
        / ".fingerprint"
        / f"{PACKAGE}-{unit_hash}"
        / f"test-bench-{BENCH}.json"
    )
    if not fingerprint_path.is_file():
        fail(
            f"build {name}: no cargo fingerprint at {fingerprint_path}; cannot read its rustflags"
        )
    fingerprint = json.loads(fingerprint_path.read_text())
    expected_dir = PROFILE_DIRS.get(profile, profile)
    if profile_dir.name != expected_dir:
        fail(
            f"build {name}: profile {profile} built into {profile_dir}, expected .../{expected_dir}"
        )

    digest = sha256_file(executable)
    copy = out / "bins" / f"{name}-{digest[:12]}"
    shutil.copy2(executable, copy)
    copy.chmod(0o555)
    if sha256_file(copy) != digest:
        fail(f"build {name}: the copy of {executable} does not match it")
    return {
        "name": name,
        "checkout": str(checkout),
        "source": source,
        "executable": str(executable),
        "binary": str(copy),
        "sha256": digest,
        "fresh": artifact.get("fresh"),
        "profile_name": profile,
        "profile": artifact.get("profile"),
        "features": artifact.get("features"),
        "fingerprint_rustflags": fingerprint.get("rustflags"),
        "fingerprint_features": fingerprint.get("features"),
        "rustc": run(["rustc", "-vV"], cwd=checkout, env=env),
        "cargo": run(["cargo", "-vV"], cwd=checkout, env=env),
        "cargo_configs": {str(path): sha256_file(path) for path in own + home},
        "environment": build_environment(),
    }


def dataset_identity(dataset):
    """A sha256 over the relative path, size and content hash of every file."""
    digest = hashlib.sha256()
    files = 0
    total = 0
    for path in sorted(p for p in dataset.rglob("*") if p.is_file()):
        size = path.stat().st_size
        relative = path.relative_to(dataset).as_posix()
        digest.update(f"{relative}\0{size}\0{sha256_file(path)}\n".encode())
        files += 1
        total += size
    return {
        "path": str(dataset),
        "files": files,
        "bytes": total,
        "sha256": digest.hexdigest(),
    }


def host_identity():
    host = {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cpu_count": os.cpu_count(),
        "python": platform.python_version(),
        "cpu": None,
        "memory_bytes": None,
    }
    if sys.platform == "darwin":
        for key, sysctl in (
            ("cpu", "machdep.cpu.brand_string"),
            ("memory_bytes", "hw.memsize"),
        ):
            result = subprocess.run(
                ["sysctl", "-n", sysctl], capture_output=True, text=True
            )
            if result.returncode == 0:
                host[key] = result.stdout.strip()
    elif sys.platform.startswith("linux"):
        cpuinfo = Path("/proc/cpuinfo")
        if cpuinfo.is_file():
            models = [
                line.split(":", 1)[1].strip()
                for line in cpuinfo.read_text().splitlines()
                if line.lower().startswith(("model name", "cpu model"))
            ]
            host["cpu"] = models[0] if models else None
        meminfo = Path("/proc/meminfo")
        if meminfo.is_file():
            for line in meminfo.read_text().splitlines():
                if line.startswith("MemTotal:"):
                    host["memory_bytes"] = int(line.split()[1]) * 1024
    return host


def check_build_settings(builds, allow_differences):
    keys = (
        "rustc",
        "profile",
        "fingerprint_rustflags",
        "fingerprint_features",
        "features",
    )
    first = builds[0]
    for other in builds[1:]:
        differing = [key for key in keys if other[key] != first[key]]
        if differing and not allow_differences:
            fail(
                f"builds {first['name']} and {other['name']} differ in {', '.join(differing)}; "
                "pass --allow-build-differences if that is intended"
            )
    seen = {}
    for entry in builds:
        if entry["sha256"] in seen:
            fail(
                f"builds {seen[entry['sha256']]} and {entry['name']} produced the same "
                "executable; use --control for an identical-binary control"
            )
        seen[entry["sha256"]] = entry["name"]


def check_run_file(path, label, binary, digest, samples, workloads, manifests):
    lines = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if not lines or lines[0].get("record") != "run":
        fail(f"{path}: the first record is not a run record")
    header = lines[0]
    if header.get("schema") != SCHEMA:
        fail(f"{path}: record schema {header.get('schema')}, expected {SCHEMA}")
    if header.get("build") != label or header.get("binary_sha256") != digest:
        fail(
            f"{path}: the run record names build {header.get('build')} / {header.get('binary_sha256')}"
        )
    if Path(header.get("binary_path", "")).resolve() != binary.resolve():
        fail(
            f"{path}: the run record names binary {header.get('binary_path')}, expected {binary}"
        )
    manifest = header["dataset"]["manifest_blake3"]
    manifests.setdefault(manifest, []).append(str(path))
    if len(manifests) != 1:
        fail(f"runs read different tables: {json.dumps(manifests)}")
    count = sum(1 for line in lines[1:] if line.get("record") == "sample")
    if count != samples * len(workloads):
        fail(f"{path}: {count} samples, expected {samples * len(workloads)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dataset", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--build", action="append", default=[], metavar="NAME=CHECKOUT")
    parser.add_argument(
        "--control", metavar="NAME", help="also run this build as <NAME>-copy"
    )
    parser.add_argument("--profile", default="release-with-debug")
    parser.add_argument("--rounds", type=int, default=20)
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--warmup", type=int, default=2)
    parser.add_argument(
        "--workloads",
        default="scan_summary_full,filter_summary_is_null_count,count_summary_aggregate",
    )
    parser.add_argument("--allow-build-differences", action="store_true")
    args = parser.parse_args()

    if args.rounds <= 0 or args.rounds % 4:
        fail(
            f"--rounds must be a positive multiple of 4 to keep the Latin square balanced, got {args.rounds}"
        )
    if args.samples <= 0 or args.warmup < 0:
        fail("--samples must be positive and --warmup non-negative")
    dataset = args.dataset.resolve()
    if not dataset.is_dir():
        fail(f"--dataset {dataset} is not a directory")
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        fail(f"--out {out} already holds files")

    requested = []
    for entry in args.build:
        name, separator, checkout = entry.partition("=")
        if not separator or not LABEL.match(name) or not checkout:
            fail(
                f"--build takes NAME=CHECKOUT with NAME of [A-Za-z0-9._-], got {entry!r}"
            )
        requested.append((name, Path(checkout)))
    names = [name for name, _ in requested]
    labels = names + ([f"{args.control}-copy"] if args.control else [])
    if args.control and args.control not in names:
        fail(f"--control {args.control} is not a --build name")
    if len(set(labels)) != len(labels) or len(labels) != 4:
        fail(f"need exactly four distinct labels, got {labels}")
    workloads = args.workloads.split(",")
    for directory in ("bins", "logs", "source", "warmup"):
        (out / directory).mkdir(parents=True, exist_ok=True)

    record = {
        "started": now(),
        "config": {
            "profile": args.profile,
            "rounds": args.rounds,
            "samples": args.samples,
            "warmup": args.warmup,
            "workloads": workloads,
            "control": args.control,
            "allow_build_differences": args.allow_build_differences,
        },
        "host": host_identity(),
        "dataset_before": dataset_identity(dataset),
    }
    builds = [build(name, checkout, args.profile, out) for name, checkout in requested]
    check_build_settings(builds, args.allow_build_differences)
    by_label = {entry["name"]: entry for entry in builds}
    if args.control:
        by_label[f"{args.control}-copy"] = by_label[args.control]
    record["builds"] = builds
    record["labels"] = {label: by_label[label]["name"] for label in labels}
    (out / "run.json").write_text(json.dumps(record, indent=2) + "\n")

    order = [
        (round_number, labels[index])
        for round_number in range(1, args.rounds + 1)
        for index in LATIN_SQUARE[(round_number - 1) % 4]
    ]
    with open(out / "order.tsv", "w") as order_file:
        order_file.write("round\tlabel\tbinary\n")
        for round_number, label in order:
            order_file.write(f"{round_number}\t{label}\t{by_label[label]['binary']}\n")

    def measure(label, jsonl, log, samples, warmup, selected):
        entry = by_label[label]
        env = dict(
            cargo_env(),
            BENCH_COUNTERS_MODE="measure",
            BENCH_COUNTERS_URI=str(dataset),
            BENCH_READ_SAMPLES=str(samples),
            BENCH_WARMUP=str(warmup),
            BENCH_WORKLOADS=",".join(selected),
            BENCH_BUILD=label,
            BENCH_OUT=str(jsonl),
            BENCH_BINARY_SHA256=entry["sha256"],
            BENCH_GIT_SHA=entry["source"]["head"],
        )
        with open(log, "w") as log_file:
            return subprocess.run(
                [entry["binary"]],
                env=env,
                stdin=subprocess.DEVNULL,
                stdout=log_file,
                stderr=subprocess.STDOUT,
            ).returncode

    for label in labels:
        status = measure(
            label,
            out / "warmup" / f"{label}.jsonl",
            out / "logs" / f"warmup-{label}.log",
            5,
            0,
            workloads[:1],
        )
        if status != 0:
            fail(f"warm-up of {label} exited with {status}; see {out / 'logs'}")

    manifests = {}
    with open(out / "runs.tsv", "w") as runs:
        runs.write(
            "round\tlabel\tstarted\tfinished\tloadavg1\tloadavg5\tloadavg15\texit_status\n"
        )
        for round_number, label in order:
            jsonl = out / f"round{round_number}-{label}.jsonl"
            load = os.getloadavg()
            started = now()
            status = measure(
                label,
                jsonl,
                out / "logs" / f"round{round_number}-{label}.log",
                args.samples,
                args.warmup,
                workloads,
            )
            runs.write(
                f"{round_number}\t{label}\t{started}\t{now()}\t"
                f"{load[0]:.2f}\t{load[1]:.2f}\t{load[2]:.2f}\t{status}\n"
            )
            runs.flush()
            if status != 0:
                fail(f"round {round_number} of {label} exited with {status}")
            entry = by_label[label]
            check_run_file(
                jsonl,
                label,
                Path(entry["binary"]),
                entry["sha256"],
                args.samples,
                workloads,
                manifests,
            )

    for entry in builds:
        if sha256_file(entry["binary"]) != entry["sha256"]:
            fail(f"binary {entry['binary']} changed during the rotation")
    record["dataset_after"] = dataset_identity(dataset)
    record["finished"] = now()
    (out / "run.json").write_text(json.dumps(record, indent=2) + "\n")
    if record["dataset_after"] != record["dataset_before"]:
        fail(
            "the table changed during the rotation; see dataset_before and dataset_after in run.json"
        )
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
