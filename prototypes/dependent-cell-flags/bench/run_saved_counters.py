#!/usr/bin/env python3
"""Snapshot and rotate already-built benches without placing targets in a checkout.

Save after a build, before changing its source:
  run_saved_counters.py save --name before --checkout /abs/lance \
      --executable /abs/target/release-with-debug/deps/cell_flags_costs-HASH \
      --out /abs/snapshots

Then use the existing rotation driver's arguments, with snapshot JSON files
instead of checkouts in --build. Binaries are hashed and copied before running;
the existing settings, table-integrity and raw-record checks still apply.
Targets and binaries stay outside the repository. Only use snapshots of builds
completed from the recorded checkout; this does not reconstruct past source.
"""

import argparse
import errno
import json
import os
import shutil
import sys
from pathlib import Path

import run_counters_rotation as rotation


def save():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", required=True)
    parser.add_argument("--checkout", required=True, type=Path)
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args(sys.argv[2:])
    if not rotation.LABEL.fullmatch(args.name):
        parser.error("name must contain only letters, digits, dot, underscore or dash")
    checkout = args.checkout.resolve()
    executable = args.executable.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    destination = out / f"{args.name}.json"
    if destination.exists():
        parser.error(f"snapshot already exists: {destination}")
    bench, unit_hash = executable.name.rsplit("-", 1)
    fingerprint = json.loads((executable.parent.parent / ".fingerprint" /
                              f"lance-{unit_hash}" / f"test-bench-{bench}.json").read_text())
    own, parents, home = rotation.cargo_configs(checkout)
    if parents:
        parser.error(f"inherited Cargo configs: {parents}")
    digest = rotation.sha256_file(executable)
    binary = out / f"{args.name}-{digest[:12]}"
    shutil.copy2(executable, binary)
    binary.chmod(0o555)
    entry = {
        "name": args.name,
        "checkout": str(checkout),
        "source": rotation.source_identity(checkout, out / f"{args.name}.patch"),
        "executable": str(executable),
        "binary": str(binary),
        "sha256": digest,
        "profile_name": executable.parent.parent.name,
        "profile": fingerprint["profile"],
        "features": fingerprint["features"],
        "fingerprint_rustflags": fingerprint["rustflags"],
        "fingerprint_features": fingerprint["features"],
        "rustc": rotation.run(["rustc", "-vV"], cwd=checkout),
        "cargo": rotation.run(["cargo", "-vV"], cwd=checkout),
        "cargo_configs": {str(path): rotation.sha256_file(path) for path in own + home},
        "environment": rotation.build_environment(),
        "protoc_adapter_sha256": rotation.sha256_file(
            checkout / "prototypes/dependent-cell-flags/bench/protoc_network.py"),
    }
    destination.write_text(json.dumps(entry, indent=2) + "\n")
    print(destination)


def saved_build(name, snapshot, profile, out, bench):
    entry = json.loads(snapshot.read_text())
    if entry["name"] != name or entry["profile_name"] != profile:
        rotation.fail(f"snapshot name/profile does not match: {snapshot}")
    if not Path(entry["executable"]).name.startswith(f"{bench}-"):
        rotation.fail(f"snapshot is not {bench}: {snapshot}")
    binary = Path(entry["binary"])
    if rotation.sha256_file(binary) != entry["sha256"]:
        rotation.fail(f"snapshot binary hash changed: {binary}")
    destination = out / "bins" / binary.name
    # Snapshots are immutable and may be several GiB with debug symbols.
    # Reuse their bytes across rotations while retaining the hash checks.
    try:
        os.link(binary, destination)
    except OSError as error:
        if error.errno != errno.EXDEV:
            raise
        shutil.copy2(binary, destination)
    shutil.copy2(entry["source"]["patch"], out / "source" / f"{name}.patch")
    entry["binary"] = str(destination)
    return entry


if __name__ == "__main__":
    if sys.argv[1:2] == ["save"]:
        save()
    else:
        rotation.build = saved_build
        rotation.main()
