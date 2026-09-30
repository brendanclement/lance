## nearest_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.996 | [0.985, 1.008] | 5/12 |
| zip-copy | instructions | 1.000 | [0.997, 1.003] | 5/12 |
| zip-copy | cycles | 0.992 | [0.982, 1.003] | 4/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/12 |
| zip-copy | peak_live_growth_bytes | 1.003 | [0.972, 1.033] | 6/12 |
| nullif | wall_ns | 1.014 | [1.006, 1.021] | 11/12 |
| nullif | instructions | 1.006 | [1.004, 1.008] | 11/12 |
| nullif | cycles | 0.988 | [0.975, 1.002] | 4/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | peak_live_growth_bytes | 1.032 | [1.010, 1.056] | 8/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 18.81 | 18.01 | 10.89 |
| zip-copy | 18.66 | 17.92 | 10.89 |
| nullif | 19.02 | 18.27 | 10.64 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 29434 | 1495.6 | 311.4 |
| zip-copy | 29434 | 1495.6 | 311.5 |
| nullif | 29434 | 1495.6 | 320.2 |

## nearest_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 1.000 | [0.988, 1.012] | 7/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 5/12 |
| zip-copy | cycles | 1.000 | [0.989, 1.010] | 5/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.001] | 4/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 5/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.003] | 7/12 |
| nullif | wall_ns | 0.998 | [0.985, 1.011] | 5/12 |
| nullif | instructions | 1.000 | [0.999, 1.000] | 6/12 |
| nullif | cycles | 0.998 | [0.994, 1.004] | 5/12 |
| nullif | allocations | 1.000 | [0.999, 1.001] | 6/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 8/12 |
| nullif | peak_live_growth_bytes | 1.000 | [0.995, 1.005] | 6/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 239.36 | 232.83 | 1.57 |
| zip-copy | 238.16 | 232.32 | 1.58 |
| nullif | 237.85 | 232.30 | 1.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 39152 | 2968.1 | 493.3 |
| zip-copy | 39152 | 2968.1 | 493.3 |
| nullif | 39152 | 2968.1 | 493.3 |

## nearest_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.995 | [0.979, 1.008] | 5/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 5/12 |
| zip-copy | cycles | 0.994 | [0.983, 1.006] | 5/12 |
| zip-copy | allocations | 1.001 | [1.000, 1.002] | 9/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 9/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.997, 1.003] | 6/12 |
| nullif | wall_ns | 0.994 | [0.980, 1.008] | 5/12 |
| nullif | instructions | 1.000 | [0.999, 1.001] | 6/12 |
| nullif | cycles | 0.995 | [0.987, 1.004] | 4/12 |
| nullif | allocations | 1.001 | [1.000, 1.002] | 7/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 8/12 |
| nullif | peak_live_growth_bytes | 0.997 | [0.993, 1.002] | 5/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 133.10 | 129.44 | 2.01 |
| zip-copy | 132.07 | 129.03 | 2.01 |
| nullif | 132.67 | 128.89 | 2.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 38324 | 2718.8 | 493.9 |
| zip-copy | 38325 | 2718.8 | 493.3 |
| nullif | 38325 | 2718.8 | 493.0 |

## nearest_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 1.002 | [0.977, 1.026] | 6/12 |
| zip-copy | instructions | 1.002 | [1.000, 1.003] | 8/12 |
| zip-copy | cycles | 1.006 | [0.991, 1.020] | 9/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 0.997 | [0.987, 1.008] | 5/12 |
| nullif | wall_ns | 1.009 | [0.990, 1.029] | 7/12 |
| nullif | instructions | 1.001 | [0.999, 1.003] | 7/12 |
| nullif | cycles | 0.995 | [0.984, 1.006] | 5/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | peak_live_growth_bytes | 1.002 | [0.993, 1.013] | 6/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.84 | 7.26 | 7.38 |
| zip-copy | 7.87 | 7.31 | 7.38 |
| nullif | 7.95 | 7.40 | 7.32 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 25428 | 520.7 | 110.8 |
| zip-copy | 25428 | 520.7 | 110.3 |
| nullif | 25428 | 520.7 | 110.8 |

## nearest_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 1.002 | [0.985, 1.020] | 7/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 7/12 |
| zip-copy | cycles | 1.001 | [0.995, 1.006] | 7/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.001] | 6/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 6/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.995, 1.006] | 6/12 |
| nullif | wall_ns | 0.887 | [0.875, 0.899] | 0/12 |
| nullif | instructions | 0.858 | [0.857, 0.858] | 0/12 |
| nullif | cycles | 0.918 | [0.913, 0.922] | 0/12 |
| nullif | allocations | 0.844 | [0.843, 0.844] | 0/12 |
| nullif | allocated_bytes | 0.511 | [0.511, 0.511] | 0/12 |
| nullif | peak_live_growth_bytes | 0.920 | [0.917, 0.922] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 256.00 | 249.60 | 1.46 |
| zip-copy | 254.42 | 248.72 | 1.47 |
| nullif | 226.49 | 220.92 | 1.52 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 45833 | 2955.4 | 428.3 |
| zip-copy | 45833 | 2955.4 | 428.3 |
| nullif | 38634 | 1511.6 | 394.8 |

## nearest_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.994 | [0.983, 1.006] | 3/12 |
| zip-copy | instructions | 1.001 | [1.000, 1.001] | 8/12 |
| zip-copy | cycles | 0.999 | [0.992, 1.005] | 7/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.001] | 3/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | peak_live_growth_bytes | 1.013 | [1.005, 1.022] | 11/12 |
| nullif | wall_ns | 0.747 | [0.738, 0.757] | 0/12 |
| nullif | instructions | 0.732 | [0.732, 0.733] | 0/12 |
| nullif | cycles | 0.858 | [0.853, 0.863] | 0/12 |
| nullif | allocations | 0.820 | [0.819, 0.820] | 0/12 |
| nullif | allocated_bytes | 0.531 | [0.531, 0.531] | 0/12 |
| nullif | peak_live_growth_bytes | 0.981 | [0.979, 0.983] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 163.41 | 158.62 | 1.74 |
| zip-copy | 161.43 | 157.80 | 1.75 |
| nullif | 121.44 | 118.75 | 2.01 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 46236 | 2376.7 | 400.7 |
| zip-copy | 46188 | 2376.7 | 402.6 |
| nullif | 37902 | 1262.4 | 392.7 |

## nearest_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.994 | [0.973, 1.015] | 6/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 6/12 |
| zip-copy | cycles | 1.002 | [0.991, 1.014] | 8/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.001] | 5/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 6/12 |
| zip-copy | peak_live_growth_bytes | 1.001 | [0.947, 1.052] | 8/12 |
| nullif | wall_ns | 0.997 | [0.981, 1.013] | 5/12 |
| nullif | instructions | 1.000 | [0.999, 1.000] | 2/12 |
| nullif | cycles | 0.998 | [0.985, 1.011] | 7/12 |
| nullif | allocations | 1.001 | [1.000, 1.002] | 8/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 8/12 |
| nullif | peak_live_growth_bytes | 1.012 | [0.973, 1.055] | 8/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 16.05 | 15.29 | 13.04 |
| zip-copy | 15.87 | 15.30 | 13.14 |
| nullif | 16.04 | 15.32 | 12.97 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 32467 | 993.0 | 215.9 |
| zip-copy | 32466 | 993.0 | 208.6 |
| nullif | 32467 | 993.0 | 216.0 |

## nearest_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.995 | [0.976, 1.014] | 7/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 5/12 |
| zip-copy | cycles | 1.004 | [0.997, 1.011] | 7/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.000] | 5/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 5/12 |
| zip-copy | peak_live_growth_bytes | 0.959 | [0.914, 1.004] | 5/12 |
| nullif | wall_ns | 0.995 | [0.983, 1.010] | 5/12 |
| nullif | instructions | 1.000 | [1.000, 1.000] | 6/12 |
| nullif | cycles | 1.002 | [0.992, 1.014] | 6/12 |
| nullif | allocations | 1.000 | [0.999, 1.001] | 4/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | peak_live_growth_bytes | 0.988 | [0.938, 1.044] | 5/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 15.99 | 15.34 | 12.92 |
| zip-copy | 15.91 | 15.23 | 13.10 |
| nullif | 16.00 | 15.25 | 12.96 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 32468 | 993.0 | 212.5 |
| zip-copy | 32467 | 993.0 | 204.3 |
| nullif | 32467 | 993.0 | 208.0 |

## scan_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.995 | [0.985, 1.003] | 3/12 |
| zip-copy | instructions | 1.000 | [0.998, 1.001] | 6/12 |
| zip-copy | cycles | 0.997 | [0.985, 1.006] | 8/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | peak_live_growth_bytes | 0.996 | [0.959, 1.034] | 5/12 |
| nullif | wall_ns | 0.996 | [0.988, 1.002] | 6/12 |
| nullif | instructions | 1.000 | [0.999, 1.001] | 5/12 |
| nullif | cycles | 1.002 | [0.990, 1.013] | 7/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 5/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | peak_live_growth_bytes | 0.984 | [0.948, 1.012] | 5/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 14.74 | 14.23 | 11.31 |
| zip-copy | 14.63 | 14.25 | 11.34 |
| nullif | 14.64 | 14.26 | 11.36 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 18137 | 1481.9 | 207.3 |
| zip-copy | 18137 | 1481.9 | 203.4 |
| nullif | 18137 | 1481.9 | 203.1 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.996 | [0.986, 1.009] | 3/12 |
| zip-copy | instructions | 1.001 | [0.997, 1.006] | 7/12 |
| zip-copy | cycles | 1.006 | [0.993, 1.018] | 8/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 0.999 | [0.990, 1.009] | 6/12 |
| nullif | wall_ns | 0.960 | [0.953, 0.967] | 0/12 |
| nullif | instructions | 0.935 | [0.932, 0.938] | 0/12 |
| nullif | cycles | 0.994 | [0.983, 1.008] | 3/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| nullif | peak_live_growth_bytes | 1.006 | [1.002, 1.010] | 9/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 30.16 | 28.90 | 6.55 |
| zip-copy | 29.98 | 28.98 | 6.58 |
| nullif | 28.95 | 27.98 | 6.78 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 18339 | 2442.5 | 491.3 |
| zip-copy | 18339 | 2442.5 | 495.3 |
| nullif | 18339 | 2442.5 | 495.3 |

## scan_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.995 | [0.989, 1.001] | 4/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.002] | 4/12 |
| zip-copy | cycles | 1.003 | [0.991, 1.014] | 8/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 2/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 5/12 |
| zip-copy | peak_live_growth_bytes | 1.002 | [0.987, 1.018] | 6/12 |
| nullif | wall_ns | 0.994 | [0.987, 1.001] | 2/12 |
| nullif | instructions | 0.999 | [0.998, 1.001] | 4/12 |
| nullif | cycles | 0.999 | [0.987, 1.012] | 6/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | peak_live_growth_bytes | 0.991 | [0.982, 1.002] | 3/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 27.19 | 26.34 | 6.84 |
| zip-copy | 27.01 | 26.29 | 6.90 |
| nullif | 27.06 | 26.27 | 6.87 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 18339 | 2442.5 | 472.1 |
| zip-copy | 18339 | 2442.5 | 475.2 |
| nullif | 18339 | 2442.5 | 468.0 |

## scan_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.990 | [0.973, 1.007] | 5/12 |
| zip-copy | instructions | 1.000 | [0.999, 1.001] | 7/12 |
| zip-copy | cycles | 0.987 | [0.959, 1.010] | 6/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.993 | [0.982, 1.000] | 4/12 |
| nullif | wall_ns | 0.999 | [0.987, 1.009] | 5/12 |
| nullif | instructions | 1.000 | [0.999, 1.001] | 5/12 |
| nullif | cycles | 0.999 | [0.977, 1.018] | 5/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | peak_live_growth_bytes | 0.997 | [0.983, 1.007] | 4/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 4.16 | 3.79 | 6.75 |
| zip-copy | 4.10 | 3.82 | 6.74 |
| nullif | 4.15 | 3.88 | 6.72 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 14132 | 506.9 | 62.9 |
| zip-copy | 14132 | 506.9 | 62.9 |
| nullif | 14132 | 506.9 | 62.9 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.998 | [0.982, 1.016] | 7/12 |
| zip-copy | instructions | 0.999 | [0.998, 1.000] | 4/12 |
| zip-copy | cycles | 0.998 | [0.992, 1.005] | 5/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | peak_live_growth_bytes | 0.997 | [0.992, 1.000] | 5/12 |
| nullif | wall_ns | 0.055 | [0.055, 0.056] | 0/12 |
| nullif | instructions | 0.153 | [0.153, 0.154] | 0/12 |
| nullif | cycles | 0.438 | [0.435, 0.440] | 0/12 |
| nullif | allocations | 0.720 | [0.720, 0.720] | 0/12 |
| nullif | allocated_bytes | 0.406 | [0.406, 0.406] | 0/12 |
| nullif | peak_live_growth_bytes | 0.466 | [0.463, 0.469] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 235.94 | 229.29 | 1.45 |
| zip-copy | 234.90 | 228.34 | 1.45 |
| nullif | 13.03 | 12.69 | 11.79 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 24890 | 2429.8 | 381.4 |
| zip-copy | 24889 | 2429.8 | 381.2 |
| nullif | 17917 | 986.1 | 177.7 |

## scan_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.996 | [0.981, 1.012] | 8/12 |
| zip-copy | instructions | 1.001 | [0.999, 1.002] | 7/12 |
| zip-copy | cycles | 1.000 | [0.994, 1.005] | 6/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 3/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.001] | 8/12 |
| nullif | wall_ns | 0.099 | [0.098, 0.100] | 0/12 |
| nullif | instructions | 0.192 | [0.192, 0.192] | 0/12 |
| nullif | cycles | 0.630 | [0.625, 0.636] | 0/12 |
| nullif | allocations | 0.688 | [0.688, 0.688] | 0/12 |
| nullif | allocated_bytes | 0.469 | [0.469, 0.469] | 0/12 |
| nullif | peak_live_growth_bytes | 0.471 | [0.466, 0.477] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 136.87 | 133.21 | 1.81 |
| zip-copy | 135.37 | 133.04 | 1.82 |
| nullif | 13.54 | 13.07 | 11.76 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 26025 | 2100.4 | 381.4 |
| zip-copy | 26024 | 2100.4 | 381.4 |
| nullif | 17917 | 986.1 | 178.6 |

## scan_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.998 | [0.984, 1.010] | 4/12 |
| zip-copy | instructions | 0.999 | [0.996, 1.002] | 7/12 |
| zip-copy | cycles | 0.998 | [0.988, 1.004] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 2/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/12 |
| zip-copy | peak_live_growth_bytes | 1.001 | [0.978, 1.031] | 5/12 |
| nullif | wall_ns | 0.993 | [0.986, 1.000] | 3/12 |
| nullif | instructions | 0.999 | [0.997, 1.001] | 5/12 |
| nullif | cycles | 0.998 | [0.989, 1.005] | 8/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | peak_live_growth_bytes | 1.000 | [0.986, 1.016] | 7/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 13.05 | 12.65 | 11.59 |
| zip-copy | 12.98 | 12.66 | 11.62 |
| nullif | 12.96 | 12.65 | 11.63 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 16407 | 978.1 | 177.9 |
| zip-copy | 16407 | 978.1 | 177.9 |
| nullif | 16407 | 978.1 | 177.9 |

## scan_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.997 | [0.989, 1.007] | 5/12 |
| zip-copy | instructions | 1.000 | [0.998, 1.002] | 7/12 |
| zip-copy | cycles | 1.000 | [0.991, 1.007] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| zip-copy | peak_live_growth_bytes | 0.994 | [0.990, 0.999] | 3/12 |
| nullif | wall_ns | 0.996 | [0.990, 1.003] | 3/12 |
| nullif | instructions | 0.999 | [0.998, 1.001] | 4/12 |
| nullif | cycles | 0.999 | [0.990, 1.008] | 7/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 3/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| nullif | peak_live_growth_bytes | 1.002 | [0.992, 1.012] | 7/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 12.97 | 12.61 | 11.52 |
| zip-copy | 12.92 | 12.56 | 11.56 |
| nullif | 12.91 | 12.61 | 11.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 16407 | 978.1 | 169.9 |
| zip-copy | 16407 | 978.1 | 169.9 |
| nullif | 16407 | 978.1 | 169.9 |

## take_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.994 | [0.957, 1.032] | 6/12 |
| zip-copy | instructions | 0.995 | [0.970, 1.020] | 6/12 |
| zip-copy | cycles | 0.992 | [0.958, 1.026] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 3/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 6/12 |
| nullif | wall_ns | 1.013 | [0.978, 1.047] | 8/12 |
| nullif | instructions | 1.004 | [0.979, 1.029] | 7/12 |
| nullif | cycles | 0.995 | [0.955, 1.039] | 6/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 4/12 |
| nullif | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 5/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.57 | 7.17 | 15.38 |
| zip-copy | 7.60 | 7.13 | 15.47 |
| nullif | 7.66 | 7.15 | 15.24 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 19947 | 5.5 | 1.3 |
| zip-copy | 19947 | 5.5 | 1.3 |
| nullif | 19948 | 5.5 | 1.3 |

## take_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.994 | [0.958, 1.031] | 6/12 |
| zip-copy | instructions | 0.996 | [0.970, 1.022] | 6/12 |
| zip-copy | cycles | 0.999 | [0.959, 1.036] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 4/12 |
| nullif | wall_ns | 1.006 | [0.970, 1.043] | 6/12 |
| nullif | instructions | 1.006 | [0.980, 1.034] | 8/12 |
| nullif | cycles | 1.003 | [0.960, 1.050] | 6/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 3/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.81 | 7.33 | 14.93 |
| zip-copy | 7.84 | 7.34 | 14.96 |
| nullif | 7.90 | 7.35 | 14.83 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 23037 | 6.1 | 1.3 |
| zip-copy | 23037 | 6.1 | 1.3 |
| nullif | 23037 | 6.1 | 1.3 |

## take_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 1.002 | [0.967, 1.039] | 7/12 |
| zip-copy | instructions | 0.997 | [0.971, 1.022] | 7/12 |
| zip-copy | cycles | 1.007 | [0.967, 1.044] | 9/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 5/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 6/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 5/12 |
| nullif | wall_ns | 1.009 | [0.974, 1.045] | 6/12 |
| nullif | instructions | 1.003 | [0.977, 1.030] | 7/12 |
| nullif | cycles | 1.008 | [0.965, 1.056] | 6/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 5/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 5/12 |
| nullif | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 5/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.82 | 7.33 | 14.86 |
| zip-copy | 7.85 | 7.43 | 14.97 |
| nullif | 7.86 | 7.32 | 14.83 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 23038 | 6.1 | 1.3 |
| zip-copy | 23038 | 6.1 | 1.3 |
| nullif | 23038 | 6.1 | 1.3 |

## take_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.999 | [0.983, 1.014] | 6/12 |
| zip-copy | instructions | 1.000 | [0.998, 1.003] | 6/12 |
| zip-copy | cycles | 0.989 | [0.970, 1.012] | 4/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | wall_ns | 0.997 | [0.981, 1.013] | 5/12 |
| nullif | instructions | 0.999 | [0.997, 1.002] | 4/12 |
| nullif | cycles | 0.987 | [0.972, 1.001] | 4/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 0.47 | 0.45 | 1.17 |
| zip-copy | 0.47 | 0.45 | 1.17 |
| nullif | 0.47 | 0.45 | 1.17 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 6816 | 2.3 | 1.0 |
| zip-copy | 6816 | 2.3 | 1.0 |
| nullif | 6816 | 2.3 | 1.0 |

## take_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 1.001 | [0.962, 1.041] | 7/12 |
| zip-copy | instructions | 1.005 | [0.978, 1.032] | 7/12 |
| zip-copy | cycles | 0.985 | [0.943, 1.024] | 5/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 5/12 |
| nullif | wall_ns | 1.008 | [0.971, 1.045] | 7/12 |
| nullif | instructions | 0.994 | [0.965, 1.022] | 4/12 |
| nullif | cycles | 1.023 | [0.981, 1.066] | 9/12 |
| nullif | allocations | 0.982 | [0.982, 0.982] | 0/12 |
| nullif | allocated_bytes | 0.855 | [0.855, 0.855] | 0/12 |
| nullif | peak_live_growth_bytes | 0.929 | [0.929, 0.930] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.76 | 7.17 | 14.56 |
| zip-copy | 7.88 | 7.23 | 14.77 |
| nullif | 7.80 | 7.31 | 14.91 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 20291 | 5.8 | 1.4 |
| zip-copy | 20291 | 5.8 | 1.4 |
| nullif | 19929 | 5.0 | 1.3 |

## take_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.993 | [0.963, 1.024] | 6/12 |
| zip-copy | instructions | 0.995 | [0.971, 1.019] | 6/12 |
| zip-copy | cycles | 1.001 | [0.963, 1.040] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 4/12 |
| nullif | wall_ns | 1.013 | [0.973, 1.054] | 7/12 |
| nullif | instructions | 0.991 | [0.964, 1.018] | 4/12 |
| nullif | cycles | 1.029 | [0.990, 1.076] | 7/12 |
| nullif | allocations | 0.977 | [0.977, 0.977] | 0/12 |
| nullif | allocated_bytes | 0.782 | [0.782, 0.782] | 0/12 |
| nullif | peak_live_growth_bytes | 0.859 | [0.859, 0.860] | 0/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.75 | 7.21 | 14.77 |
| zip-copy | 7.71 | 7.18 | 14.86 |
| nullif | 7.86 | 7.35 | 14.83 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 20411 | 6.4 | 1.5 |
| zip-copy | 20411 | 6.4 | 1.5 |
| nullif | 19937 | 5.0 | 1.3 |

## take_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.999 | [0.960, 1.038] | 7/12 |
| zip-copy | instructions | 0.997 | [0.973, 1.022] | 6/12 |
| zip-copy | cycles | 1.003 | [0.970, 1.036] | 9/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| nullif | wall_ns | 1.012 | [0.974, 1.050] | 7/12 |
| nullif | instructions | 1.003 | [0.976, 1.029] | 5/12 |
| nullif | cycles | 1.008 | [0.964, 1.057] | 7/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 5/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| nullif | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.44 | 7.01 | 15.57 |
| zip-copy | 7.48 | 6.98 | 15.65 |
| nullif | 7.51 | 6.97 | 15.53 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 19753 | 5.0 | 1.2 |
| zip-copy | 19753 | 5.0 | 1.2 |
| nullif | 19753 | 5.0 | 1.2 |

## take_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip-copy | wall_ns | 0.993 | [0.957, 1.031] | 6/12 |
| zip-copy | instructions | 0.994 | [0.969, 1.019] | 5/12 |
| zip-copy | cycles | 1.000 | [0.968, 1.030] | 7/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 2/12 |
| nullif | wall_ns | 1.000 | [0.964, 1.036] | 5/12 |
| nullif | instructions | 1.002 | [0.976, 1.028] | 5/12 |
| nullif | cycles | 1.005 | [0.964, 1.051] | 5/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| nullif | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/12 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| zip | 7.52 | 7.09 | 15.38 |
| zip-copy | 7.49 | 6.97 | 15.68 |
| nullif | 7.49 | 7.00 | 15.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| zip | 19753 | 5.0 | 1.2 |
| zip-copy | 19753 | 5.0 | 1.2 |
| nullif | 19753 | 5.0 | 1.2 |

## scan_ready / scan_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 0.994 | [0.987, 0.999] | 1/12 |
| zip | instructions | 0.998 | [0.997, 0.999] | 0/12 |
| zip | cycles | 0.992 | [0.987, 0.996] | 1/12 |
| zip | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| zip | peak_live_growth_bytes | 0.959 | [0.946, 0.970] | 0/12 |
| zip-copy | wall_ns | 0.993 | [0.988, 0.998] | 2/12 |
| zip-copy | instructions | 0.999 | [0.998, 0.999] | 0/12 |
| zip-copy | cycles | 0.994 | [0.991, 0.997] | 2/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.952 | [0.929, 0.972] | 0/12 |
| nullif | wall_ns | 0.997 | [0.994, 1.001] | 2/12 |
| nullif | instructions | 0.998 | [0.997, 0.998] | 0/12 |
| nullif | cycles | 0.994 | [0.991, 0.996] | 2/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | peak_live_growth_bytes | 0.960 | [0.949, 0.971] | 0/12 |

## nearest_ready / nearest_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 1.000 | [0.992, 1.009] | 5/12 |
| zip | instructions | 1.000 | [1.000, 1.000] | 8/12 |
| zip | cycles | 0.995 | [0.991, 1.000] | 5/12 |
| zip | allocations | 1.001 | [1.000, 1.002] | 6/12 |
| zip | allocated_bytes | 1.000 | [1.000, 1.000] | 8/12 |
| zip | peak_live_growth_bytes | 1.006 | [0.970, 1.043] | 7/12 |
| zip-copy | wall_ns | 1.002 | [0.997, 1.007] | 6/12 |
| zip-copy | instructions | 1.000 | [1.000, 1.000] | 7/12 |
| zip-copy | cycles | 0.997 | [0.993, 1.001] | 5/12 |
| zip-copy | allocations | 1.000 | [0.999, 1.001] | 5/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 6/12 |
| zip-copy | peak_live_growth_bytes | 0.964 | [0.931, 0.999] | 3/12 |
| nullif | wall_ns | 0.999 | [0.995, 1.002] | 6/12 |
| nullif | instructions | 1.001 | [1.000, 1.001] | 9/12 |
| nullif | cycles | 1.000 | [0.998, 1.002] | 7/12 |
| nullif | allocations | 1.000 | [0.999, 1.001] | 6/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 5/12 |
| nullif | peak_live_growth_bytes | 0.982 | [0.959, 1.008] | 3/12 |

## take_ready / take_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 1.008 | [1.001, 1.015] | 11/12 |
| zip | instructions | 1.003 | [1.000, 1.007] | 7/12 |
| zip | cycles | 1.002 | [0.997, 1.007] | 6/12 |
| zip | allocations | 1.000 | [1.000, 1.000] | 2/12 |
| zip | allocated_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| zip | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/12 |
| zip-copy | wall_ns | 1.003 | [0.998, 1.008] | 9/12 |
| zip-copy | instructions | 1.000 | [0.997, 1.003] | 6/12 |
| zip-copy | cycles | 0.998 | [0.992, 1.003] | 5/12 |
| zip-copy | allocations | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/12 |
| zip-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 3/12 |
| nullif | wall_ns | 0.997 | [0.990, 1.005] | 4/12 |
| nullif | instructions | 1.002 | [0.999, 1.005] | 9/12 |
| nullif | cycles | 0.998 | [0.992, 1.003] | 5/12 |
| nullif | allocations | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | allocated_bytes | 1.000 | [1.000, 1.000] | 0/12 |
| nullif | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/12 |

## scan_partial_1pct / scan_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 7.834 | [7.774, 7.899] | 12/12 |
| zip | instructions | 2.986 | [2.979, 2.994] | 12/12 |
| zip | cycles | 1.788 | [1.775, 1.802] | 12/12 |
| zip | allocations | 1.357 | [1.357, 1.357] | 12/12 |
| zip | allocated_bytes | 0.995 | [0.995, 0.995] | 0/12 |
| zip | peak_live_growth_bytes | 0.775 | [0.773, 0.777] | 0/12 |
| zip-copy | wall_ns | 7.854 | [7.775, 7.937] | 12/12 |
| zip-copy | instructions | 2.981 | [2.971, 2.990] | 12/12 |
| zip-copy | cycles | 1.774 | [1.754, 1.794] | 12/12 |
| zip-copy | allocations | 1.357 | [1.357, 1.357] | 12/12 |
| zip-copy | allocated_bytes | 0.995 | [0.995, 0.995] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.774 | [0.766, 0.781] | 0/12 |
| nullif | wall_ns | 0.450 | [0.448, 0.451] | 0/12 |
| nullif | instructions | 0.490 | [0.489, 0.491] | 0/12 |
| nullif | cycles | 0.787 | [0.782, 0.792] | 0/12 |
| nullif | allocations | 0.977 | [0.977, 0.977] | 0/12 |
| nullif | allocated_bytes | 0.404 | [0.404, 0.404] | 0/12 |
| nullif | peak_live_growth_bytes | 0.359 | [0.356, 0.362] | 0/12 |

## nearest_partial_1pct / nearest_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 1.071 | [1.063, 1.081] | 12/12 |
| zip | instructions | 1.039 | [1.039, 1.040] | 12/12 |
| zip | cycles | 0.998 | [0.993, 1.001] | 3/12 |
| zip | allocations | 1.171 | [1.170, 1.172] | 12/12 |
| zip | allocated_bytes | 0.996 | [0.996, 0.996] | 0/12 |
| zip | peak_live_growth_bytes | 0.870 | [0.866, 0.874] | 0/12 |
| zip-copy | wall_ns | 1.073 | [1.067, 1.079] | 12/12 |
| zip-copy | instructions | 1.039 | [1.039, 1.040] | 12/12 |
| zip-copy | cycles | 0.999 | [0.995, 1.003] | 4/12 |
| zip-copy | allocations | 1.171 | [1.170, 1.172] | 12/12 |
| zip-copy | allocated_bytes | 0.996 | [0.996, 0.996] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.870 | [0.866, 0.875] | 0/12 |
| nullif | wall_ns | 0.952 | [0.947, 0.957] | 0/12 |
| nullif | instructions | 0.891 | [0.891, 0.892] | 0/12 |
| nullif | cycles | 0.917 | [0.913, 0.920] | 0/12 |
| nullif | allocations | 0.988 | [0.987, 0.989] | 0/12 |
| nullif | allocated_bytes | 0.509 | [0.509, 0.509] | 0/12 |
| nullif | peak_live_growth_bytes | 0.801 | [0.799, 0.803] | 0/12 |

## take_partial_1pct / take_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 0.990 | [0.984, 0.996] | 3/12 |
| zip | instructions | 0.990 | [0.984, 0.995] | 1/12 |
| zip | cycles | 0.980 | [0.973, 0.988] | 1/12 |
| zip | allocations | 0.881 | [0.881, 0.881] | 0/12 |
| zip | allocated_bytes | 0.959 | [0.959, 0.959] | 0/12 |
| zip | peak_live_growth_bytes | 1.077 | [1.077, 1.078] | 12/12 |
| zip-copy | wall_ns | 0.997 | [0.991, 1.004] | 6/12 |
| zip-copy | instructions | 0.998 | [0.993, 1.003] | 6/12 |
| zip-copy | cycles | 0.967 | [0.951, 0.981] | 1/12 |
| zip-copy | allocations | 0.881 | [0.881, 0.881] | 0/12 |
| zip-copy | allocated_bytes | 0.959 | [0.959, 0.959] | 0/12 |
| zip-copy | peak_live_growth_bytes | 1.077 | [1.077, 1.078] | 12/12 |
| nullif | wall_ns | 0.992 | [0.985, 0.998] | 4/12 |
| nullif | instructions | 0.978 | [0.974, 0.982] | 0/12 |
| nullif | cycles | 1.000 | [0.993, 1.007] | 5/12 |
| nullif | allocations | 0.865 | [0.865, 0.865] | 0/12 |
| nullif | allocated_bytes | 0.819 | [0.819, 0.819] | 0/12 |
| nullif | peak_live_growth_bytes | 1.002 | [1.001, 1.002] | 12/12 |

## scan_partial_50pct / scan_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 5.039 | [4.996, 5.082] | 12/12 |
| zip | instructions | 2.606 | [2.603, 2.609] | 12/12 |
| zip | cycles | 1.360 | [1.347, 1.373] | 12/12 |
| zip | allocations | 1.419 | [1.419, 1.419] | 12/12 |
| zip | allocated_bytes | 0.860 | [0.860, 0.860] | 0/12 |
| zip | peak_live_growth_bytes | 0.804 | [0.799, 0.809] | 0/12 |
| zip-copy | wall_ns | 5.047 | [4.990, 5.108] | 12/12 |
| zip-copy | instructions | 2.607 | [2.603, 2.611] | 12/12 |
| zip-copy | cycles | 1.356 | [1.347, 1.365] | 12/12 |
| zip-copy | allocations | 1.419 | [1.419, 1.419] | 12/12 |
| zip-copy | allocated_bytes | 0.860 | [0.860, 0.860] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.803 | [0.793, 0.812] | 0/12 |
| nullif | wall_ns | 0.501 | [0.497, 0.504] | 0/12 |
| nullif | instructions | 0.500 | [0.500, 0.501] | 0/12 |
| nullif | cycles | 0.858 | [0.849, 0.866] | 0/12 |
| nullif | allocations | 0.977 | [0.977, 0.977] | 0/12 |
| nullif | allocated_bytes | 0.404 | [0.404, 0.404] | 0/12 |
| nullif | peak_live_growth_bytes | 0.382 | [0.377, 0.388] | 0/12 |

## nearest_partial_50pct / nearest_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 1.222 | [1.215, 1.230] | 12/12 |
| zip | instructions | 1.135 | [1.135, 1.136] | 12/12 |
| zip | cycles | 1.061 | [1.054, 1.069] | 12/12 |
| zip | allocations | 1.205 | [1.205, 1.206] | 12/12 |
| zip | allocated_bytes | 0.874 | [0.874, 0.874] | 0/12 |
| zip | peak_live_growth_bytes | 0.809 | [0.806, 0.812] | 0/12 |
| zip-copy | wall_ns | 1.222 | [1.217, 1.228] | 12/12 |
| zip-copy | instructions | 1.136 | [1.136, 1.137] | 12/12 |
| zip-copy | cycles | 1.066 | [1.058, 1.074] | 12/12 |
| zip-copy | allocations | 1.204 | [1.203, 1.205] | 12/12 |
| zip-copy | allocated_bytes | 0.874 | [0.874, 0.874] | 0/12 |
| zip-copy | peak_live_growth_bytes | 0.819 | [0.812, 0.828] | 0/12 |
| nullif | wall_ns | 0.918 | [0.916, 0.920] | 0/12 |
| nullif | instructions | 0.831 | [0.831, 0.832] | 0/12 |
| nullif | cycles | 0.915 | [0.911, 0.919] | 0/12 |
| nullif | allocations | 0.987 | [0.986, 0.988] | 0/12 |
| nullif | allocated_bytes | 0.464 | [0.464, 0.464] | 0/12 |
| nullif | peak_live_growth_bytes | 0.795 | [0.793, 0.797] | 0/12 |

## take_partial_50pct / take_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 0.992 | [0.985, 0.999] | 3/12 |
| zip | instructions | 0.990 | [0.986, 0.994] | 1/12 |
| zip | cycles | 0.975 | [0.960, 0.987] | 1/12 |
| zip | allocations | 0.886 | [0.886, 0.886] | 0/12 |
| zip | allocated_bytes | 1.048 | [1.048, 1.048] | 12/12 |
| zip | peak_live_growth_bytes | 1.166 | [1.165, 1.166] | 12/12 |
| zip-copy | wall_ns | 0.983 | [0.975, 0.990] | 1/12 |
| zip-copy | instructions | 0.989 | [0.984, 0.993] | 2/12 |
| zip-copy | cycles | 0.970 | [0.962, 0.977] | 0/12 |
| zip-copy | allocations | 0.886 | [0.886, 0.886] | 0/12 |
| zip-copy | allocated_bytes | 1.048 | [1.048, 1.048] | 12/12 |
| zip-copy | peak_live_growth_bytes | 1.166 | [1.165, 1.166] | 12/12 |
| nullif | wall_ns | 0.996 | [0.986, 1.005] | 5/12 |
| nullif | instructions | 0.978 | [0.974, 0.982] | 0/12 |
| nullif | cycles | 0.995 | [0.987, 1.003] | 5/12 |
| nullif | allocations | 0.865 | [0.865, 0.865] | 0/12 |
| nullif | allocated_bytes | 0.819 | [0.819, 0.819] | 0/12 |
| nullif | peak_live_growth_bytes | 1.002 | [1.001, 1.002] | 12/12 |

## scan_masked / scan_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 3.544 | [3.508, 3.583] | 12/12 |
| zip | instructions | 2.302 | [2.299, 2.305] | 12/12 |
| zip | cycles | 6.113 | [6.044, 6.172] | 12/12 |
| zip | allocations | 1.283 | [1.283, 1.283] | 12/12 |
| zip | allocated_bytes | 2.923 | [2.923, 2.923] | 12/12 |
| zip | peak_live_growth_bytes | 3.297 | [3.222, 3.385] | 12/12 |
| zip-copy | wall_ns | 3.561 | [3.497, 3.619] | 12/12 |
| zip-copy | instructions | 2.301 | [2.298, 2.305] | 12/12 |
| zip-copy | cycles | 6.174 | [6.098, 6.252] | 12/12 |
| zip-copy | allocations | 1.283 | [1.283, 1.283] | 12/12 |
| zip-copy | allocated_bytes | 2.923 | [2.923, 2.923] | 12/12 |
| zip-copy | peak_live_growth_bytes | 3.306 | [3.211, 3.400] | 12/12 |
| nullif | wall_ns | 3.534 | [3.498, 3.573] | 12/12 |
| nullif | instructions | 2.301 | [2.299, 2.304] | 12/12 |
| nullif | cycles | 6.137 | [6.078, 6.193] | 12/12 |
| nullif | allocations | 1.283 | [1.283, 1.283] | 12/12 |
| nullif | allocated_bytes | 2.923 | [2.923, 2.923] | 12/12 |
| nullif | peak_live_growth_bytes | 3.254 | [3.181, 3.330] | 12/12 |

## nearest_masked / nearest_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 2.391 | [2.356, 2.431] | 12/12 |
| zip | instructions | 1.363 | [1.361, 1.366] | 12/12 |
| zip | cycles | 3.567 | [3.536, 3.597] | 12/12 |
| zip | allocations | 1.158 | [1.158, 1.158] | 12/12 |
| zip | allocated_bytes | 2.872 | [2.872, 2.872] | 12/12 |
| zip | peak_live_growth_bytes | 2.825 | [2.798, 2.856] | 12/12 |
| zip-copy | wall_ns | 2.377 | [2.348, 2.407] | 12/12 |
| zip-copy | instructions | 1.360 | [1.358, 1.363] | 12/12 |
| zip-copy | cycles | 3.516 | [3.467, 3.562] | 12/12 |
| zip-copy | allocations | 1.158 | [1.158, 1.158] | 12/12 |
| zip-copy | allocated_bytes | 2.872 | [2.872, 2.872] | 12/12 |
| zip-copy | peak_live_growth_bytes | 2.839 | [2.785, 2.894] | 12/12 |
| nullif | wall_ns | 2.403 | [2.377, 2.434] | 12/12 |
| nullif | instructions | 1.370 | [1.369, 1.372] | 12/12 |
| nullif | cycles | 3.541 | [3.496, 3.586] | 12/12 |
| nullif | allocations | 1.158 | [1.158, 1.158] | 12/12 |
| nullif | allocated_bytes | 2.872 | [2.872, 2.872] | 12/12 |
| nullif | peak_live_growth_bytes | 2.908 | [2.870, 2.945] | 12/12 |

## take_masked / take_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| zip | wall_ns | 16.053 | [15.588, 16.605] | 12/12 |
| zip | instructions | 15.183 | [14.920, 15.493] | 12/12 |
| zip | cycles | 229.593 | [223.048, 237.367] | 12/12 |
| zip | allocations | 2.927 | [2.927, 2.927] | 12/12 |
| zip | allocated_bytes | 2.373 | [2.373, 2.373] | 12/12 |
| zip | peak_live_growth_bytes | 1.209 | [1.209, 1.210] | 12/12 |
| zip-copy | wall_ns | 15.978 | [15.763, 16.234] | 12/12 |
| zip-copy | instructions | 15.099 | [14.956, 15.255] | 12/12 |
| zip-copy | cycles | 230.292 | [226.356, 234.682] | 12/12 |
| zip-copy | allocations | 2.927 | [2.927, 2.927] | 12/12 |
| zip-copy | allocated_bytes | 2.373 | [2.373, 2.373] | 12/12 |
| zip-copy | peak_live_growth_bytes | 1.209 | [1.209, 1.210] | 12/12 |
| nullif | wall_ns | 16.312 | [15.842, 16.835] | 12/12 |
| nullif | instructions | 15.256 | [14.989, 15.569] | 12/12 |
| nullif | cycles | 231.502 | [224.254, 240.402] | 12/12 |
| nullif | allocations | 2.927 | [2.927, 2.927] | 12/12 |
| nullif | allocated_bytes | 2.373 | [2.373, 2.373] | 12/12 |
| nullif | peak_live_growth_bytes | 1.209 | [1.209, 1.209] | 12/12 |

