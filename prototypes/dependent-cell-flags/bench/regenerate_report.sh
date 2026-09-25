#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors
#
# Regenerates every generated section of REPORT.md from results/<run>/, with
# the section named after the run and the heading below. The hand-written
# Summary at the top is left alone.
#
# Usage: ./regenerate_report.sh [REPORT.md]   (default: REPORT.md next to this script)

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPORT=${1:-$SCRIPT_DIR/REPORT.md}

section() {
  local run=$1 title=$2
  python3 "$SCRIPT_DIR/analyze.py" "$SCRIPT_DIR/results/$run" --report "$REPORT" \
    --section "$run" --title "$title" > /dev/null
}

section smoke-final 'Smoke: 100k rows, prototype `21601f894` (nested build)'
section 1m '1M full matrix, prototype `21601f894` (nested build)'
section 10m '10M reads, clean refresh, sparse updates and publish after K, prototype `21601f894` (nested build), one round'
section 1m-reads-maskfix '1M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build)'
section 10m-reads-maskfix '10M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build), baseline first'
section 10m-reads-reversed '10M reads after the mask fix, `26388225a` code recorded as `e48573011` (nested build), prototype first, flags in round 3 only'
section 10m-reads-variant-fragment-reverted 'Control: 10M reads, `26388225a` with `fragment.rs` reverted to the baseline (clean build, no flag runs), baseline first'
section 10m-reads-clean-baseline-first 'Clean build: 10M reads at `26388225a`, baseline first'
section 10m-reads-clean-prototype-first 'Clean build: 10M reads at `26388225a`, prototype first'
section 1m-clean 'Clean build: 1M full matrix at `26388225a`, flags in round 3 only'
section 10m-reads-layout-control 'Control: 10M reads, baseline plus one unused function (records labelled `prototype`), rounds 1-3 baseline first, 4-6 control first'
echo "regenerated the results sections of $REPORT"
