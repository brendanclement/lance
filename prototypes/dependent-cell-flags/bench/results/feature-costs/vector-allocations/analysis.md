## nearest_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.984, 1.018] | 1/2 |
| final-copy | instructions | 1.003 | [1.001, 1.005] | 2/2 |
| final-copy | cycles | 0.992 | [0.970, 1.015] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.966 | [0.877, 1.064] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.10 | 17.52 | 11.57 |
| final-copy | 18.21 | 17.57 | 11.45 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 29434 | 1495.6 | 307.1 |
| final-copy | 29434 | 1495.6 | 287.1 |

## nearest_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.997, 1.001] | 1/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 0.997 | [0.987, 1.008] | 1/2 |
| final-copy | allocations | 1.003 | [1.003, 1.004] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.997 | [0.996, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 230.14 | 228.63 | 1.61 |
| final-copy | 230.07 | 228.83 | 1.59 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 39053 | 2968.1 | 497.3 |
| final-copy | 39153 | 2968.1 | 495.3 |

## nearest_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.993, 0.997] | 0/2 |
| final-copy | instructions | 0.997 | [0.997, 0.998] | 0/2 |
| final-copy | cycles | 0.994 | [0.991, 0.996] | 0/2 |
| final-copy | allocations | 1.001 | [0.997, 1.004] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.988 | [0.979, 0.997] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 127.58 | 127.10 | 2.12 |
| final-copy | 126.84 | 126.22 | 2.11 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 38420 | 2718.8 | 488.7 |
| final-copy | 38422 | 2718.8 | 485.5 |

## nearest_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [0.985, 1.027] | 1/2 |
| final-copy | instructions | 1.000 | [0.998, 1.003] | 1/2 |
| final-copy | cycles | 1.014 | [0.991, 1.038] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.990 | [0.949, 1.033] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.44 | 7.12 | 7.46 |
| final-copy | 7.49 | 7.34 | 7.46 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 25427 | 520.7 | 108.6 |
| final-copy | 25428 | 520.7 | 108.3 |

## nearest_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 1.004 | [0.995, 1.014] | 1/2 |
| final-copy | allocations | 0.999 | [0.997, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.996 | [0.993, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 218.82 | 217.12 | 1.56 |
| final-copy | 219.31 | 217.44 | 1.55 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 38683 | 1511.6 | 394.8 |
| final-copy | 38633 | 1511.6 | 394.4 |

## nearest_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.996, 0.996] | 0/2 |
| final-copy | instructions | 0.999 | [0.997, 1.000] | 1/2 |
| final-copy | cycles | 0.987 | [0.972, 1.002] | 1/2 |
| final-copy | allocations | 1.001 | [0.997, 1.004] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.998 | [0.998, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 117.05 | 116.47 | 2.08 |
| final-copy | 116.62 | 115.48 | 2.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 37754 | 1262.4 | 391.1 |
| final-copy | 37804 | 1262.4 | 392.0 |

## nearest_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.994 | [0.984, 1.005] | 1/2 |
| final-copy | instructions | 1.001 | [0.999, 1.003] | 1/2 |
| final-copy | cycles | 1.003 | [0.990, 1.017] | 1/2 |
| final-copy | allocations | 1.002 | [1.002, 1.003] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.973 | [0.925, 1.024] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 15.27 | 14.84 | 13.84 |
| final-copy | 15.18 | 14.97 | 13.90 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 32418 | 993.0 | 191.9 |
| final-copy | 32468 | 993.0 | 191.3 |

## nearest_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.015 | [1.002, 1.029] | 2/2 |
| final-copy | instructions | 1.000 | [0.998, 1.003] | 1/2 |
| final-copy | cycles | 1.004 | [0.998, 1.011] | 1/2 |
| final-copy | allocations | 1.002 | [1.002, 1.003] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.054 | [1.023, 1.086] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 15.12 | 14.95 | 13.82 |
| final-copy | 15.17 | 14.98 | 13.91 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 32418 | 993.0 | 187.6 |
| final-copy | 32516 | 993.0 | 197.4 |

## scan_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.991, 1.002] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.003] | 1/2 |
| final-copy | cycles | 1.013 | [1.009, 1.018] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.904 | [0.884, 0.925] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 14.24 | 13.81 | 11.74 |
| final-copy | 14.17 | 13.93 | 11.92 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 18137 | 1481.9 | 196.3 |
| final-copy | 18137 | 1481.9 | 180.5 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.015 | [1.005, 1.025] | 2/2 |
| final-copy | instructions | 1.002 | [0.997, 1.007] | 1/2 |
| final-copy | cycles | 1.007 | [0.999, 1.014] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.017 | [1.000, 1.033] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.05 | 26.14 | 6.99 |
| final-copy | 28.09 | 27.02 | 7.01 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 18339 | 2442.5 | 499.4 |
| final-copy | 18339 | 2442.5 | 499.4 |

## scan_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [0.998, 1.015] | 1/2 |
| final-copy | instructions | 1.001 | [0.999, 1.004] | 1/2 |
| final-copy | cycles | 0.956 | [0.946, 0.966] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.995 | [0.983, 1.007] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 26.71 | 25.74 | 7.40 |
| final-copy | 27.03 | 25.71 | 6.94 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 18339 | 2442.5 | 473.6 |
| final-copy | 18339 | 2442.5 | 471.6 |

## scan_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.986, 1.015] | 1/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 1.010 | [0.996, 1.023] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.017 | [1.000, 1.034] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 3.83 | 3.69 | 6.62 |
| final-copy | 3.87 | 3.63 | 6.77 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 14132 | 506.9 | 62.8 |
| final-copy | 14132 | 506.9 | 62.9 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [1.000, 1.008] | 1/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 0.997 | [0.996, 0.998] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.960 | [0.950, 0.970] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 13.01 | 12.90 | 11.71 |
| final-copy | 13.10 | 12.84 | 11.69 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 17916 | 986.1 | 179.9 |
| final-copy | 17916 | 986.1 | 173.7 |

## scan_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.008 | [1.001, 1.015] | 2/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | cycles | 1.003 | [0.994, 1.011] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.969 | [0.966, 0.973] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 13.54 | 13.35 | 11.81 |
| final-copy | 13.66 | 13.42 | 11.86 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 17916 | 986.1 | 180.1 |
| final-copy | 17916 | 986.1 | 177.7 |

## scan_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.985, 1.015] | 1/2 |
| final-copy | instructions | 0.997 | [0.996, 0.999] | 0/2 |
| final-copy | cycles | 1.016 | [1.013, 1.019] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.952 | [0.884, 1.026] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 12.78 | 12.56 | 11.54 |
| final-copy | 12.76 | 12.64 | 11.75 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 16408 | 978.1 | 192.3 |
| final-copy | 16407 | 978.1 | 180.7 |

## scan_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.995, 0.995] | 0/2 |
| final-copy | instructions | 1.000 | [0.998, 1.001] | 1/2 |
| final-copy | cycles | 0.999 | [0.998, 1.000] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.962 | [0.955, 0.968] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 12.74 | 12.58 | 11.51 |
| final-copy | 12.71 | 12.53 | 11.63 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 16406 | 978.1 | 177.9 |
| final-copy | 16407 | 978.1 | 170.3 |

## take_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.013 | [0.817, 1.256] | 1/2 |
| final-copy | instructions | 0.998 | [0.844, 1.181] | 1/2 |
| final-copy | cycles | 1.000 | [0.785, 1.275] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.97 | 6.81 | 16.26 |
| final-copy | 7.93 | 6.93 | 16.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 19947 | 5.5 | 1.3 |
| final-copy | 19948 | 5.5 | 1.3 |

## take_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.009 | [0.830, 1.227] | 1/2 |
| final-copy | instructions | 1.010 | [0.870, 1.173] | 1/2 |
| final-copy | cycles | 0.995 | [0.783, 1.264] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.31 | 7.01 | 15.91 |
| final-copy | 8.36 | 6.90 | 15.84 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 23038 | 6.1 | 1.3 |
| final-copy | 23037 | 6.1 | 1.3 |

## take_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.022 | [0.839, 1.244] | 1/2 |
| final-copy | instructions | 0.996 | [0.867, 1.143] | 1/2 |
| final-copy | cycles | 1.018 | [0.813, 1.276] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.18 | 7.21 | 15.56 |
| final-copy | 8.16 | 7.14 | 15.68 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 23039 | 6.1 | 1.3 |
| final-copy | 23038 | 6.1 | 1.3 |

## take_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.982 | [0.978, 0.986] | 0/2 |
| final-copy | instructions | 1.001 | [0.992, 1.009] | 1/2 |
| final-copy | cycles | 0.989 | [0.978, 0.999] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.46 | 0.46 | 1.17 |
| final-copy | 0.45 | 0.44 | 1.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 6816 | 2.3 | 1.0 |
| final-copy | 6816 | 2.3 | 1.0 |

## take_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.009 | [0.814, 1.251] | 1/2 |
| final-copy | instructions | 1.011 | [0.866, 1.180] | 1/2 |
| final-copy | cycles | 1.000 | [0.783, 1.276] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.10 | 7.04 | 15.87 |
| final-copy | 8.15 | 7.09 | 15.80 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 19930 | 5.0 | 1.3 |
| final-copy | 19930 | 5.0 | 1.3 |

## take_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [0.816, 1.229] | 1/2 |
| final-copy | instructions | 1.002 | [0.855, 1.175] | 1/2 |
| final-copy | cycles | 0.987 | [0.770, 1.267] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.000, 1.002] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.39 | 7.14 | 15.92 |
| final-copy | 8.41 | 7.11 | 15.47 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 19937 | 5.0 | 1.3 |
| final-copy | 19937 | 5.0 | 1.3 |

## take_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.814, 1.238] | 1/2 |
| final-copy | instructions | 1.011 | [0.872, 1.173] | 1/2 |
| final-copy | cycles | 1.000 | [0.810, 1.233] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.89 | 6.87 | 16.29 |
| final-copy | 7.95 | 7.00 | 16.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 19753 | 5.0 | 1.2 |
| final-copy | 19754 | 5.0 | 1.2 |

## take_ready
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.794, 1.258] | 1/2 |
| final-copy | instructions | 0.998 | [0.854, 1.165] | 1/2 |
| final-copy | cycles | 1.007 | [0.785, 1.290] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.82 | 6.52 | 16.10 |
| final-copy | 7.85 | 6.69 | 16.61 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 19754 | 5.0 | 1.2 |
| final-copy | 19754 | 5.0 | 1.2 |

## scan_ready / scan_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.997 | [0.980, 1.013] | 1/2 |
| final | instructions | 0.998 | [0.996, 1.000] | 1/2 |
| final | cycles | 0.992 | [0.986, 0.999] | 0/2 |
| final | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final | peak_live_growth_bytes | 0.936 | [0.835, 1.050] | 1/2 |
| final-copy | wall_ns | 0.992 | [0.990, 0.993] | 0/2 |
| final-copy | instructions | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | cycles | 0.976 | [0.974, 0.978] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.945 | [0.914, 0.977] | 0/2 |

## scan_partial_1pct / scan_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.472 | [0.464, 0.480] | 0/2 |
| final | instructions | 0.492 | [0.489, 0.495] | 0/2 |
| final | cycles | 0.789 | [0.774, 0.805] | 0/2 |
| final | allocations | 0.977 | [0.977, 0.977] | 0/2 |
| final | allocated_bytes | 0.404 | [0.404, 0.404] | 0/2 |
| final | peak_live_growth_bytes | 0.366 | [0.359, 0.374] | 0/2 |
| final-copy | wall_ns | 0.466 | [0.465, 0.468] | 0/2 |
| final-copy | instructions | 0.491 | [0.491, 0.491] | 0/2 |
| final-copy | cycles | 0.782 | [0.773, 0.791] | 0/2 |
| final-copy | allocations | 0.977 | [0.977, 0.977] | 0/2 |
| final-copy | allocated_bytes | 0.404 | [0.404, 0.404] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.346 | [0.343, 0.348] | 0/2 |

## scan_partial_1pct / scan_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 1.018 | [1.004, 1.033] | 2/2 |
| final | instructions | 1.397 | [1.394, 1.401] | 2/2 |
| final | cycles | 1.031 | [1.019, 1.042] | 2/2 |
| final | allocations | 1.092 | [1.092, 1.092] | 2/2 |
| final | allocated_bytes | 1.008 | [1.008, 1.008] | 2/2 |
| final | peak_live_growth_bytes | 0.951 | [0.857, 1.057] | 1/2 |
| final-copy | wall_ns | 1.022 | [1.019, 1.026] | 2/2 |
| final-copy | instructions | 1.401 | [1.397, 1.404] | 2/2 |
| final-copy | cycles | 1.011 | [1.003, 1.020] | 2/2 |
| final-copy | allocations | 1.092 | [1.092, 1.092] | 2/2 |
| final-copy | allocated_bytes | 1.008 | [1.008, 1.008] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.959 | [0.921, 0.999] | 0/2 |

## scan_partial_50pct / scan_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.504 | [0.498, 0.511] | 0/2 |
| final | instructions | 0.500 | [0.500, 0.500] | 0/2 |
| final | cycles | 0.823 | [0.820, 0.827] | 0/2 |
| final | allocations | 0.977 | [0.977, 0.977] | 0/2 |
| final | allocated_bytes | 0.404 | [0.404, 0.404] | 0/2 |
| final | peak_live_growth_bytes | 0.391 | [0.375, 0.407] | 0/2 |
| final-copy | wall_ns | 0.505 | [0.504, 0.507] | 0/2 |
| final-copy | instructions | 0.500 | [0.498, 0.501] | 0/2 |
| final-copy | cycles | 0.864 | [0.858, 0.869] | 0/2 |
| final-copy | allocations | 0.977 | [0.977, 0.977] | 0/2 |
| final-copy | allocated_bytes | 0.404 | [0.404, 0.404] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.381 | [0.363, 0.400] | 0/2 |

## scan_masked / scan_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 3.695 | [3.648, 3.743] | 2/2 |
| final | instructions | 2.299 | [2.297, 2.300] | 2/2 |
| final | cycles | 6.738 | [6.633, 6.846] | 2/2 |
| final | allocations | 1.283 | [1.283, 1.283] | 2/2 |
| final | allocated_bytes | 2.923 | [2.923, 2.923] | 2/2 |
| final | peak_live_growth_bytes | 3.224 | [3.064, 3.392] | 2/2 |
| final-copy | wall_ns | 3.680 | [3.653, 3.707] | 2/2 |
| final-copy | instructions | 2.303 | [2.298, 2.307] | 2/2 |
| final-copy | cycles | 6.761 | [6.748, 6.774] | 2/2 |
| final-copy | allocations | 1.283 | [1.283, 1.284] | 2/2 |
| final-copy | allocated_bytes | 2.923 | [2.923, 2.923] | 2/2 |
| final-copy | peak_live_growth_bytes | 2.867 | [2.833, 2.902] | 2/2 |

## nearest_ready / nearest_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.991 | [0.985, 0.998] | 0/2 |
| final | instructions | 1.000 | [1.000, 1.000] | 1/2 |
| final | cycles | 0.996 | [0.993, 0.998] | 0/2 |
| final | allocations | 1.000 | [0.998, 1.002] | 1/2 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final | peak_live_growth_bytes | 0.981 | [0.926, 1.039] | 1/2 |
| final-copy | wall_ns | 1.012 | [1.002, 1.022] | 2/2 |
| final-copy | instructions | 0.999 | [0.998, 1.000] | 0/2 |
| final-copy | cycles | 0.997 | [0.987, 1.006] | 1/2 |
| final-copy | allocations | 1.000 | [0.997, 1.003] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.062 | [1.024, 1.102] | 2/2 |

## nearest_partial_1pct / nearest_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.951 | [0.950, 0.952] | 0/2 |
| final | instructions | 0.889 | [0.889, 0.890] | 0/2 |
| final | cycles | 0.920 | [0.917, 0.924] | 0/2 |
| final | allocations | 0.991 | [0.990, 0.992] | 0/2 |
| final | allocated_bytes | 0.509 | [0.509, 0.509] | 0/2 |
| final | peak_live_growth_bytes | 0.796 | [0.794, 0.799] | 0/2 |
| final-copy | wall_ns | 0.953 | [0.952, 0.953] | 0/2 |
| final-copy | instructions | 0.889 | [0.889, 0.890] | 0/2 |
| final-copy | cycles | 0.927 | [0.922, 0.931] | 0/2 |
| final-copy | allocations | 0.987 | [0.984, 0.989] | 0/2 |
| final-copy | allocated_bytes | 0.509 | [0.509, 0.509] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.795 | [0.794, 0.797] | 0/2 |

## nearest_partial_1pct / nearest_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 14.334 | [14.266, 14.402] | 2/2 |
| final | instructions | 4.270 | [4.261, 4.278] | 2/2 |
| final | cycles | 1.651 | [1.637, 1.666] | 2/2 |
| final | allocations | 1.193 | [1.193, 1.193] | 2/2 |
| final | allocated_bytes | 1.522 | [1.522, 1.522] | 2/2 |
| final | peak_live_growth_bytes | 2.030 | [1.994, 2.066] | 2/2 |
| final-copy | wall_ns | 14.435 | [14.351, 14.519] | 2/2 |
| final-copy | instructions | 4.262 | [4.261, 4.262] | 2/2 |
| final-copy | cycles | 1.652 | [1.632, 1.673] | 2/2 |
| final-copy | allocations | 1.189 | [1.188, 1.190] | 2/2 |
| final-copy | allocated_bytes | 1.522 | [1.522, 1.522] | 2/2 |
| final-copy | peak_live_growth_bytes | 2.078 | [2.005, 2.155] | 2/2 |

## nearest_partial_50pct / nearest_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.918 | [0.916, 0.920] | 0/2 |
| final | instructions | 0.832 | [0.831, 0.833] | 0/2 |
| final | cycles | 0.903 | [0.901, 0.906] | 0/2 |
| final | allocations | 0.985 | [0.984, 0.986] | 0/2 |
| final | allocated_bytes | 0.464 | [0.464, 0.464] | 0/2 |
| final | peak_live_growth_bytes | 0.799 | [0.793, 0.805] | 0/2 |
| final-copy | wall_ns | 0.919 | [0.919, 0.919] | 0/2 |
| final-copy | instructions | 0.833 | [0.833, 0.833] | 0/2 |
| final-copy | cycles | 0.897 | [0.889, 0.906] | 0/2 |
| final-copy | allocations | 0.985 | [0.984, 0.986] | 0/2 |
| final-copy | allocated_bytes | 0.464 | [0.464, 0.464] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.808 | [0.808, 0.808] | 0/2 |

## nearest_masked / nearest_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 2.453 | [2.440, 2.465] | 2/2 |
| final | instructions | 1.364 | [1.364, 1.365] | 2/2 |
| final | cycles | 3.860 | [3.778, 3.944] | 2/2 |
| final | allocations | 1.158 | [1.158, 1.158] | 2/2 |
| final | allocated_bytes | 2.872 | [2.872, 2.872] | 2/2 |
| final | peak_live_growth_bytes | 2.715 | [2.398, 3.073] | 2/2 |
| final-copy | wall_ns | 2.442 | [2.421, 2.463] | 2/2 |
| final-copy | instructions | 1.368 | [1.368, 1.368] | 2/2 |
| final-copy | cycles | 3.777 | [3.686, 3.871] | 2/2 |
| final-copy | allocations | 1.158 | [1.158, 1.158] | 2/2 |
| final-copy | allocated_bytes | 2.872 | [2.872, 2.872] | 2/2 |
| final-copy | peak_live_growth_bytes | 2.647 | [2.607, 2.688] | 2/2 |

## take_ready / take_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.993 | [0.985, 1.001] | 1/2 |
| final | instructions | 0.997 | [0.988, 1.007] | 1/2 |
| final | cycles | 0.981 | [0.970, 0.992] | 0/2 |
| final | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | wall_ns | 0.989 | [0.976, 1.001] | 1/2 |
| final-copy | instructions | 0.984 | [0.981, 0.987] | 0/2 |
| final-copy | cycles | 0.988 | [0.962, 1.015] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

## take_partial_1pct / take_null_1pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 0.997 | [0.982, 1.013] | 1/2 |
| final | instructions | 0.984 | [0.978, 0.991] | 0/2 |
| final | cycles | 0.992 | [0.990, 0.993] | 0/2 |
| final | allocations | 0.865 | [0.865, 0.865] | 0/2 |
| final | allocated_bytes | 0.819 | [0.819, 0.819] | 0/2 |
| final | peak_live_growth_bytes | 1.002 | [1.002, 1.002] | 2/2 |
| final-copy | wall_ns | 0.998 | [0.994, 1.001] | 1/2 |
| final-copy | instructions | 0.985 | [0.983, 0.986] | 0/2 |
| final-copy | cycles | 0.996 | [0.993, 1.000] | 0/2 |
| final-copy | allocations | 0.865 | [0.865, 0.865] | 0/2 |
| final-copy | allocated_bytes | 0.819 | [0.819, 0.819] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.002 | [1.001, 1.002] | 2/2 |

## take_partial_1pct / take_plain
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 1.029 | [1.028, 1.030] | 2/2 |
| final | instructions | 1.019 | [1.013, 1.025] | 2/2 |
| final | cycles | 0.995 | [0.975, 1.016] | 1/2 |
| final | allocations | 1.009 | [1.009, 1.009] | 2/2 |
| final | allocated_bytes | 1.007 | [1.007, 1.007] | 2/2 |
| final | peak_live_growth_bytes | 1.018 | [1.018, 1.019] | 2/2 |
| final-copy | wall_ns | 1.035 | [1.030, 1.039] | 2/2 |
| final-copy | instructions | 1.018 | [1.018, 1.018] | 2/2 |
| final-copy | cycles | 0.995 | [0.982, 1.009] | 1/2 |
| final-copy | allocations | 1.009 | [1.009, 1.009] | 2/2 |
| final-copy | allocated_bytes | 1.007 | [1.007, 1.007] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.018 | [1.018, 1.018] | 2/2 |

## take_partial_50pct / take_null_50pct
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 1.014 | [1.004, 1.025] | 2/2 |
| final | instructions | 0.975 | [0.963, 0.987] | 0/2 |
| final | cycles | 1.027 | [1.023, 1.030] | 2/2 |
| final | allocations | 0.865 | [0.865, 0.865] | 0/2 |
| final | allocated_bytes | 0.819 | [0.819, 0.819] | 0/2 |
| final | peak_live_growth_bytes | 1.001 | [1.000, 1.002] | 2/2 |
| final-copy | wall_ns | 0.994 | [0.977, 1.012] | 1/2 |
| final-copy | instructions | 0.981 | [0.972, 0.990] | 0/2 |
| final-copy | cycles | 0.995 | [0.975, 1.016] | 1/2 |
| final-copy | allocations | 0.865 | [0.865, 0.865] | 0/2 |
| final-copy | allocated_bytes | 0.819 | [0.819, 0.819] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.002 | [1.002, 1.002] | 2/2 |

## take_masked / take_null_all
| build | metric | ratio | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final | wall_ns | 16.905 | [15.123, 18.897] | 2/2 |
| final | instructions | 15.659 | [14.428, 16.994] | 2/2 |
| final | cycles | 266.815 | [236.733, 300.718] | 2/2 |
| final | allocations | 2.927 | [2.926, 2.927] | 2/2 |
| final | allocated_bytes | 2.373 | [2.373, 2.373] | 2/2 |
| final | peak_live_growth_bytes | 1.209 | [1.208, 1.209] | 2/2 |
| final-copy | wall_ns | 17.443 | [15.662, 19.427] | 2/2 |
| final-copy | instructions | 15.622 | [14.464, 16.874] | 2/2 |
| final-copy | cycles | 269.954 | [241.357, 301.940] | 2/2 |
| final-copy | allocations | 2.926 | [2.926, 2.927] | 2/2 |
| final-copy | allocated_bytes | 2.373 | [2.373, 2.373] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.209 | [1.209, 1.209] | 2/2 |

