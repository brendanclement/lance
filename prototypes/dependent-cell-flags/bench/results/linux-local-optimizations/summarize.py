#!/usr/bin/env python3
"""Compact summary from the raw rotations, using the existing analyzer's statistics."""
import collections
import json
import statistics
import sys
from pathlib import Path

repo = Path(sys.argv[1])
root = Path(sys.argv[2])
sys.path.insert(0, str(repo / 'prototypes/dependent-cell-flags/bench'))
import analyze_counters as analyzer

result = {}
for kind in ('reads', 'mutations', 'allocations'):
    samples, runs = analyzer.load(root / kind)
    analyzer.check_consistency(samples, runs)
    groups = collections.defaultdict(list)
    for sample in samples:
        groups[sample['workload'], sample['build'], sample['round']].append(sample)
    rounds = sorted({sample['round'] for sample in samples})
    workloads = sorted({sample['workload'] for sample in samples})
    summaries = {}
    for workload in workloads:
        builds = {}
        for build in ('before', 'after', 'after-copy'):
            records = [s for rnd in rounds for s in groups[workload, build, rnd]]
            if not records:
                continue
            metrics = {}
            for metric in ('wall_ns', 'cpu_ns', 'allocations', 'allocated_bytes',
                           'peak_live_growth_bytes', 'read_bytes', 'written_bytes'):
                values = [s.get(metric) for s in records]
                metrics[metric] = (statistics.median(values)
                                   if all(v is not None for v in values) else None)
            metrics['round_wall_ms'] = [statistics.median(s['wall_ns'] for s in
                                                        groups[workload, build, rnd]) / 1e6
                                         for rnd in rounds]
            builds[build] = metrics
        ratios = {}
        for label, numerator, denominator in (
                ('after_before', 'after', 'before'),
                ('control', 'after-copy', 'after')):
            values = [statistics.median(s['wall_ns'] for s in groups[workload, numerator, rnd]) /
                      statistics.median(s['wall_ns'] for s in groups[workload, denominator, rnd])
                      for rnd in rounds]
            ratios[label] = dict(geomean=analyzer.geomean(values),
                                 interval=analyzer.bootstrap(values), per_round=values)
        summaries[workload] = dict(builds=builds, ratios=ratios)
    record = json.loads((root / kind / 'run.json').read_text())
    assert record['dataset_before'] == record['dataset_after']
    result[kind] = dict(config=record['config'], workloads=summaries)

print(json.dumps(result, indent=2))
