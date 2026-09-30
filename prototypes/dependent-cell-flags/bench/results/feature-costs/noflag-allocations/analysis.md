## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.994 | [0.984, 1.004] | 1/3 |
| final | instructions | 1.001 | [1.001, 1.002] | 3/3 |
| final | cycles | 0.984 | [0.978, 0.992] | 0/3 |
| final | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| final | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 2/3 |
| final-copy | wall_ns | 0.986 | [0.979, 0.994] | 0/3 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 2/3 |
| final-copy | cycles | 0.990 | [0.983, 1.002] | 1/3 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| main | 62.91 | 62.32 | 10.99 |
| final | 62.37 | 62.00 | 10.92 |
| final-copy | 61.94 | 61.59 | 11.01 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| main | 703438 | 1616.4 | 30.8 |
| final | 703438 | 1616.4 | 30.8 |
| final-copy | 703438 | 1616.4 | 30.8 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.996 | [0.994, 0.997] | 0/3 |
| final | instructions | 1.001 | [1.001, 1.001] | 3/3 |
| final | cycles | 0.992 | [0.986, 1.001] | 1/3 |
| final | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| final | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 2/3 |
| final-copy | wall_ns | 0.992 | [0.980, 1.009] | 1/3 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 3/3 |
| final-copy | cycles | 0.989 | [0.976, 1.004] | 1/3 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| main | 68.42 | 68.07 | 10.50 |
| final | 68.08 | 67.77 | 10.45 |
| final-copy | 67.58 | 67.20 | 10.48 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| main | 790441 | 1660.7 | 30.8 |
| final | 790440 | 1660.7 | 30.8 |
| final-copy | 790441 | 1660.7 | 30.8 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.994 | [0.985, 0.999] | 0/3 |
| final | instructions | 1.001 | [1.000, 1.001] | 2/3 |
| final | cycles | 0.990 | [0.985, 0.997] | 0/3 |
| final | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| final | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 1/3 |
| final-copy | wall_ns | 0.995 | [0.983, 1.008] | 1/3 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 2/3 |
| final-copy | cycles | 0.989 | [0.984, 0.998] | 0/3 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.002] | 1/3 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| main | 63.01 | 62.41 | 11.21 |
| final | 62.47 | 62.02 | 11.21 |
| final-copy | 62.66 | 61.89 | 11.15 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| main | 697332 | 1577.9 | 30.8 |
| final | 697332 | 1577.9 | 30.8 |
| final-copy | 697332 | 1577.9 | 30.8 |

