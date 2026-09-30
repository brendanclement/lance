## after_appends_k0_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.995, 0.997] | 0/2 |
| final-copy | instructions | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | cycles | 0.997 | [0.985, 1.008] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.001 | [1.000, 1.002] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 77.63 | 76.82 | 2.26 |
| final-copy | 77.48 | 76.85 | 2.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 179694 | 458.1 | 13.3 |
| final-copy | 179694 | 458.6 | 13.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 15.01 | 13.74 | 414 | 13 | 5.2 | 3.9 | 1.3 | 1.3 |
| final-copy | 15.01 | 13.84 | 414 | 13 | 5.2 | 3.9 | 1.3 | 1.3 |

## after_appends_k0_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.998 | [0.995, 1.001] | 1/2 |
| final-copy | instructions | 0.996 | [0.994, 0.998] | 0/2 |
| final-copy | cycles | 1.003 | [1.001, 1.005] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.998 | [0.988, 1.008] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 51.68 | 51.14 | 6.17 |
| final-copy | 51.63 | 51.30 | 6.17 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 378657 | 597.1 | 102.8 |
| final-copy | 378660 | 597.0 | 102.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 21.79 | 16.16 | 362 | 13 | 7.5 | 3.9 | 3.6 | 3.6 |
| final-copy | 21.79 | 16.14 | 362 | 13 | 7.5 | 3.9 | 3.6 | 3.6 |

## after_appends_k32_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.993, 0.997] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 0.992 | [0.981, 1.003] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 78.72 | 77.94 | 2.26 |
| final-copy | 78.55 | 77.22 | 2.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 181557 | 458.6 | 13.3 |
| final-copy | 181556 | 458.8 | 13.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 15.18 | 13.77 | 446 | 13 | 8.1 | 6.8 | 1.3 | 1.3 |
| final-copy | 15.18 | 13.81 | 446 | 13 | 8.1 | 6.8 | 1.3 | 1.3 |

## after_appends_k32_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.003 | [0.990, 1.016] | 1/2 |
| final-copy | instructions | 0.996 | [0.995, 0.997] | 0/2 |
| final-copy | cycles | 0.992 | [0.984, 0.999] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 0.999] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.996 | [0.992, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 52.20 | 51.26 | 6.25 |
| final-copy | 52.54 | 51.62 | 6.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 380421 | 597.6 | 103.2 |
| final-copy | 380422 | 597.1 | 102.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 21.94 | 16.19 | 394 | 13 | 10.4 | 6.8 | 3.6 | 3.6 |
| final-copy | 21.94 | 16.10 | 394 | 13 | 10.4 | 6.8 | 3.6 | 3.6 |

## after_appends_k8_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.987, 0.997] | 0/2 |
| final-copy | instructions | 0.999 | [0.998, 1.000] | 0/2 |
| final-copy | cycles | 0.999 | [0.997, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 78.01 | 76.89 | 2.25 |
| final-copy | 77.44 | 76.88 | 2.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 180148 | 458.7 | 13.3 |
| final-copy | 180148 | 458.5 | 13.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 15.04 | 13.86 | 422 | 13 | 6.0 | 4.6 | 1.3 | 1.3 |
| final-copy | 15.04 | 13.80 | 422 | 13 | 6.0 | 4.6 | 1.3 | 1.3 |

## after_appends_k8_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.990, 1.001] | 1/2 |
| final-copy | instructions | 0.995 | [0.995, 0.996] | 0/2 |
| final-copy | cycles | 0.997 | [0.990, 1.004] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.997, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 51.99 | 50.40 | 6.19 |
| final-copy | 51.74 | 51.26 | 6.16 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 379086 | 597.2 | 103.1 |
| final-copy | 379090 | 596.6 | 102.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 21.82 | 16.18 | 370 | 13 | 8.2 | 4.6 | 3.6 | 3.6 |
| final-copy | 21.82 | 16.06 | 370 | 13 | 8.2 | 4.6 | 3.6 | 3.6 |

## append_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.951 | [0.920, 0.984] | 0/2 |
| final-copy | instructions | 0.996 | [0.991, 1.001] | 1/2 |
| final-copy | cycles | 0.983 | [0.970, 0.995] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.38 | 1.31 | 1.23 |
| final-copy | 1.35 | 1.29 | 1.30 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1900 | 5.3 | 5.0 |
| final-copy | 1900 | 5.3 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.01 | 1 | 4 | 2.5 | 2.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.01 | 1 | 4 | 2.5 | 2.4 | 0.1 | 0.1 |

## append_chain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.994 | [0.980, 1.008] | 1/2 |
| final-copy | instructions | 0.998 | [0.996, 1.000] | 0/2 |
| final-copy | cycles | 0.992 | [0.986, 0.998] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.48 | 1.39 | 1.24 |
| final-copy | 1.46 | 1.43 | 1.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2316 | 5.5 | 5.2 |
| final-copy | 2316 | 5.5 | 5.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.07 | 1 | 4 | 58.9 | 58.8 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.07 | 1 | 4 | 58.9 | 58.8 | 0.1 | 0.1 |

## append_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.017 | [1.010, 1.024] | 2/2 |
| final-copy | instructions | 1.005 | [0.994, 1.016] | 1/2 |
| final-copy | cycles | 0.998 | [0.989, 1.007] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.38 | 1.34 | 1.27 |
| final-copy | 1.39 | 1.35 | 1.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1893 | 5.3 | 5.0 |
| final-copy | 1893 | 5.3 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.01 | 1 | 4 | 2.4 | 2.3 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.01 | 1 | 4 | 2.4 | 2.3 | 0.1 | 0.1 |

## append_one_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [0.979, 1.026] | 1/2 |
| final-copy | instructions | 1.005 | [0.994, 1.017] | 1/2 |
| final-copy | cycles | 1.012 | [0.969, 1.057] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.43 | 1.33 | 1.24 |
| final-copy | 1.43 | 1.39 | 1.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1902 | 5.6 | 5.4 |
| final-copy | 1902 | 5.6 | 5.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.17 | 1 | 4 | 162.9 | 162.8 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.17 | 1 | 4 | 162.9 | 162.8 | 0.1 | 0.1 |

## append_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.041 | [1.026, 1.057] | 2/2 |
| final-copy | instructions | 1.005 | [0.998, 1.012] | 1/2 |
| final-copy | cycles | 1.029 | [1.024, 1.034] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.44 | 1.41 | 1.25 |
| final-copy | 1.49 | 1.45 | 1.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2005 | 5.4 | 5.1 |
| final-copy | 2006 | 5.4 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 10.6 | 10.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 10.6 | 10.4 | 0.1 | 0.1 |

## append_one_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.985 | [0.984, 0.986] | 0/2 |
| final-copy | instructions | 0.994 | [0.994, 0.995] | 0/2 |
| final-copy | cycles | 0.979 | [0.977, 0.980] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.45 | 1.43 | 1.27 |
| final-copy | 1.44 | 1.36 | 1.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2074 | 5.4 | 5.1 |
| final-copy | 2074 | 5.4 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 12.1 | 11.9 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 12.1 | 11.9 | 0.1 | 0.1 |

## append_one_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.993, 1.009] | 1/2 |
| final-copy | instructions | 1.006 | [1.000, 1.013] | 1/2 |
| final-copy | cycles | 0.998 | [0.995, 1.002] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.45 | 1.41 | 1.27 |
| final-copy | 1.45 | 1.39 | 1.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2300 | 5.5 | 5.2 |
| final-copy | 2300 | 5.5 | 5.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.05 | 1 | 4 | 33.7 | 33.6 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.05 | 1 | 4 | 33.7 | 33.6 | 0.1 | 0.1 |

## append_one_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.003 | [0.989, 1.017] | 1/2 |
| final-copy | instructions | 1.003 | [0.994, 1.011] | 1/2 |
| final-copy | cycles | 0.988 | [0.968, 1.009] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.47 | 1.40 | 1.28 |
| final-copy | 1.47 | 1.42 | 1.27 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2572 | 5.5 | 5.2 |
| final-copy | 2572 | 5.5 | 5.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.05 | 1 | 4 | 39.8 | 39.6 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.05 | 1 | 4 | 39.8 | 39.6 | 0.1 | 0.1 |

## append_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.983 | [0.982, 0.984] | 0/2 |
| final-copy | instructions | 1.004 | [1.000, 1.008] | 2/2 |
| final-copy | cycles | 0.986 | [0.986, 0.987] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.41 | 1.34 | 1.26 |
| final-copy | 1.37 | 1.32 | 1.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1868 | 5.3 | 5.0 |
| final-copy | 1868 | 5.3 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.01 | 1 | 4 | 2.3 | 2.1 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.01 | 1 | 4 | 2.3 | 2.1 | 0.1 | 0.1 |

## append_plain_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.969, 1.010] | 1/2 |
| final-copy | instructions | 0.998 | [0.990, 1.006] | 1/2 |
| final-copy | cycles | 0.987 | [0.953, 1.022] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.40 | 1.34 | 1.27 |
| final-copy | 1.40 | 1.35 | 1.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1878 | 5.3 | 5.0 |
| final-copy | 1877 | 5.3 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.01 | 1 | 4 | 2.6 | 2.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.01 | 1 | 4 | 2.6 | 2.4 | 0.1 | 0.1 |

## append_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.025 | [1.019, 1.032] | 2/2 |
| final-copy | instructions | 1.007 | [1.003, 1.011] | 2/2 |
| final-copy | cycles | 0.996 | [0.981, 1.012] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.35 | 1.33 | 1.27 |
| final-copy | 1.39 | 1.35 | 1.23 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1970 | 5.3 | 5.1 |
| final-copy | 1970 | 5.3 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 4.0 | 3.9 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 4.0 | 3.9 | 0.1 | 0.1 |

## append_plain_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.974, 1.012] | 1/2 |
| final-copy | instructions | 1.000 | [0.990, 1.010] | 1/2 |
| final-copy | cycles | 0.997 | [0.971, 1.024] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.43 | 1.35 | 1.28 |
| final-copy | 1.41 | 1.36 | 1.27 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2034 | 5.3 | 5.1 |
| final-copy | 2034 | 5.3 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 5.5 | 5.3 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 5.5 | 5.3 | 0.1 | 0.1 |

## append_plain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [0.985, 1.019] | 1/2 |
| final-copy | instructions | 1.017 | [1.011, 1.023] | 2/2 |
| final-copy | cycles | 1.022 | [1.014, 1.030] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.50 | 1.44 | 1.26 |
| final-copy | 1.48 | 1.46 | 1.27 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2262 | 5.4 | 5.1 |
| final-copy | 2262 | 5.4 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 8.6 | 8.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 8.6 | 8.4 | 0.1 | 0.1 |

## append_plain_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.979, 1.004] | 1/2 |
| final-copy | instructions | 0.986 | [0.978, 0.994] | 0/2 |
| final-copy | cycles | 0.985 | [0.976, 0.993] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.50 | 1.46 | 1.25 |
| final-copy | 1.48 | 1.44 | 1.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2518 | 5.4 | 5.1 |
| final-copy | 2519 | 5.4 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.03 | 1 | 4 | 14.5 | 14.3 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.03 | 1 | 4 | 14.5 | 14.3 | 0.1 | 0.1 |

## append_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.979 | [0.948, 1.010] | 1/2 |
| final-copy | instructions | 1.000 | [0.996, 1.003] | 1/2 |
| final-copy | cycles | 0.988 | [0.958, 1.019] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.36 | 1.31 | 1.25 |
| final-copy | 1.35 | 1.29 | 1.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1900 | 5.3 | 5.0 |
| final-copy | 1900 | 5.3 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.01 | 1 | 4 | 2.5 | 2.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.01 | 1 | 4 | 2.5 | 2.4 | 0.1 | 0.1 |

## append_shared_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [0.995, 1.017] | 1/2 |
| final-copy | instructions | 0.999 | [0.995, 1.003] | 1/2 |
| final-copy | cycles | 1.005 | [0.995, 1.014] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.47 | 1.42 | 1.28 |
| final-copy | 1.47 | 1.44 | 1.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 2316 | 5.5 | 5.2 |
| final-copy | 2317 | 5.5 | 5.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.07 | 1 | 4 | 58.9 | 58.8 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.07 | 1 | 4 | 58.9 | 58.8 | 0.1 | 0.1 |

## backfill_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | cycles | 0.996 | [0.994, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 367.77 | 365.66 | 2.33 |
| final-copy | 367.18 | 366.58 | 2.32 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 625706 | 1641.6 | 236.6 |
| final-copy | 625708 | 1641.1 | 236.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 93.97 | 27.65 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |
| final-copy | 93.95 | 27.64 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |

## backfill_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.994 | [0.991, 0.996] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 0.999] | 0/2 |
| final-copy | cycles | 1.004 | [0.996, 1.011] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 0.999 | [0.998, 1.001] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 222.67 | 221.50 | 2.67 |
| final-copy | 221.46 | 220.55 | 2.69 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 506243 | 1238.3 | 201.5 |
| final-copy | 506243 | 1237.3 | 201.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 80.15 | 13.90 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |
| final-copy | 80.09 | 13.84 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |

## backfill_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.983, 1.006] | 1/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.996 | [0.993, 1.000] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.001 | [1.000, 1.001] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 103.70 | 102.55 | 5.74 |
| final-copy | 103.41 | 101.74 | 5.72 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 580829 | 1580.4 | 236.2 |
| final-copy | 580827 | 1581.8 | 236.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 93.81 | 27.54 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |
| final-copy | 93.88 | 27.61 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |

## backfill_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.993, 0.993] | 0/2 |
| final-copy | instructions | 0.999 | [0.997, 1.000] | 1/2 |
| final-copy | cycles | 0.998 | [0.994, 1.002] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 0.999] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 90.59 | 89.37 | 5.15 |
| final-copy | 89.72 | 88.97 | 5.17 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 468831 | 1201.7 | 201.3 |
| final-copy | 468830 | 1200.6 | 201.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 80.06 | 13.80 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |
| final-copy | 80.00 | 13.74 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |

## backfill_plain_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.995, 1.003] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 2/2 |
| final-copy | cycles | 0.992 | [0.985, 0.999] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 0.999] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 104.76 | 103.30 | 5.71 |
| final-copy | 104.33 | 103.06 | 5.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 580849 | 1586.2 | 236.2 |
| final-copy | 580848 | 1584.9 | 236.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 94.02 | 27.76 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |
| final-copy | 94.18 | 27.91 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |

## backfill_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.991, 0.995] | 0/2 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 0.997 | [0.992, 1.002] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.001 | [1.001, 1.001] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 366.82 | 363.06 | 2.32 |
| final-copy | 364.32 | 363.86 | 2.33 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 625363 | 1644.7 | 236.5 |
| final-copy | 625363 | 1645.8 | 236.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 94.14 | 27.87 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |
| final-copy | 94.12 | 27.84 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |

## conflict_inplace_k1_reject
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.986 | [0.984, 0.988] | 0/2 |
| final-copy | instructions | 0.998 | [0.997, 0.999] | 0/2 |
| final-copy | cycles | 0.996 | [0.982, 1.010] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 188.60 | 185.48 | 3.36 |
| final-copy | 185.75 | 183.78 | 3.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 443432 | 1102.0 | 15.7 |
| final-copy | 443430 | 1101.6 | 15.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 46.35 | 27.82 | 3137 | 23 | 7.0 | 3.9 | 3.1 | 3.1 |
| final-copy | 46.35 | 27.73 | 3137 | 23 | 7.0 | 3.9 | 3.1 | 3.1 |

## conflict_inplace_k1_reject_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.995, 1.003] | 1/2 |
| final-copy | instructions | 0.999 | [0.997, 1.001] | 1/2 |
| final-copy | cycles | 0.991 | [0.981, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 242.82 | 239.75 | 3.75 |
| final-copy | 242.09 | 240.18 | 3.69 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 676332 | 1931.2 | 24.8 |
| final-copy | 676330 | 1933.6 | 24.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 73.76 | 55.27 | 3165 | 23 | 9.5 | 4.1 | 5.4 | 5.4 |
| final-copy | 73.76 | 55.11 | 3165 | 23 | 9.5 | 4.1 | 5.4 | 5.4 |

## conflict_inplace_k1_skip
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.988, 0.997] | 0/2 |
| final-copy | instructions | 0.998 | [0.996, 1.000] | 1/2 |
| final-copy | cycles | 0.992 | [0.973, 1.011] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 154.95 | 153.81 | 2.84 |
| final-copy | 153.80 | 152.89 | 2.84 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 372416 | 932.9 | 15.2 |
| final-copy | 372418 | 933.3 | 15.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 35.86 | 26.31 | 1694 | 25 | 4.9 | 3.9 | 1.0 | 1.0 |
| final-copy | 35.86 | 26.30 | 1694 | 25 | 4.9 | 3.9 | 1.0 | 1.0 |

## conflict_inplace_k1_skip_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.995, 1.004] | 1/2 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 0.999 | [0.985, 1.014] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.001, 1.001] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 195.73 | 194.45 | 3.46 |
| final-copy | 195.31 | 194.28 | 3.46 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 592648 | 1693.6 | 24.8 |
| final-copy | 592648 | 1693.3 | 24.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 62.02 | 52.52 | 1731 | 25 | 5.4 | 4.1 | 1.3 | 1.3 |
| final-copy | 62.03 | 52.52 | 1731 | 25 | 5.4 | 4.1 | 1.3 | 1.3 |

## conflict_inplace_k8_reject
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.989, 0.996] | 0/2 |
| final-copy | instructions | 0.998 | [0.996, 0.999] | 0/2 |
| final-copy | cycles | 0.991 | [0.974, 1.008] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.001, 1.001] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 188.00 | 187.15 | 3.43 |
| final-copy | 186.91 | 184.86 | 3.42 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 443842 | 1101.9 | 16.7 |
| final-copy | 443840 | 1101.3 | 16.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 46.43 | 27.78 | 3145 | 23 | 7.0 | 3.9 | 3.1 | 3.1 |
| final-copy | 46.43 | 27.64 | 3145 | 23 | 7.0 | 3.9 | 3.1 | 3.1 |

## conflict_inplace_k8_skip
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.991, 0.992] | 0/2 |
| final-copy | instructions | 0.999 | [0.998, 0.999] | 0/2 |
| final-copy | cycles | 0.995 | [0.984, 1.007] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 164.30 | 162.55 | 2.86 |
| final-copy | 162.91 | 160.80 | 2.86 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 394176 | 984.1 | 15.2 |
| final-copy | 394176 | 984.5 | 15.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 37.97 | 27.77 | 1810 | 26 | 5.1 | 3.9 | 1.2 | 1.2 |
| final-copy | 37.97 | 27.72 | 1810 | 26 | 5.1 | 3.9 | 1.2 | 1.2 |

## conflict_moving_k1_reject
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.984, 0.994] | 0/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 0.997 | [0.990, 1.003] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 39.61 | 38.15 | 1.06 |
| final-copy | 38.69 | 37.60 | 1.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 31608 | 28.9 | 4.4 |
| final-copy | 31608 | 28.9 | 4.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.24 | 0.06 | 30 | 6 | 7.2 | 6.9 | 0.3 | 0.3 |
| final-copy | 0.24 | 0.06 | 30 | 6 | 7.2 | 6.9 | 0.3 | 0.3 |

## conflict_moving_k1_skip
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.964 | [0.949, 0.978] | 0/2 |
| final-copy | instructions | 0.996 | [0.992, 1.000] | 0/2 |
| final-copy | cycles | 0.976 | [0.948, 1.006] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 40.35 | 38.14 | 1.06 |
| final-copy | 38.88 | 38.41 | 1.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 27306 | 32.1 | 4.3 |
| final-copy | 27306 | 32.1 | 4.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.13 | 0.04 | 25 | 8 | 7.1 | 6.9 | 0.2 | 0.2 |
| final-copy | 0.13 | 0.04 | 25 | 8 | 7.1 | 6.9 | 0.2 | 0.2 |

## cycle_inplace_sparse_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.996, 0.996] | 0/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | cycles | 0.998 | [0.995, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.997, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 205.96 | 204.59 | 5.08 |
| final-copy | 205.09 | 204.02 | 5.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1083100 | 2796.6 | 224.6 |
| final-copy | 1083101 | 2796.5 | 224.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 136.87 | 78.42 | 595 | 26 | 6.0 | 4.1 | 1.9 | 1.9 |
| final-copy | 136.89 | 78.44 | 595 | 26 | 6.0 | 4.1 | 1.9 | 1.9 |

## cycle_inplace_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.990, 0.992] | 0/2 |
| final-copy | instructions | 0.998 | [0.997, 1.000] | 0/2 |
| final-copy | cycles | 0.993 | [0.991, 0.994] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 172.17 | 171.13 | 4.60 |
| final-copy | 170.86 | 168.25 | 4.60 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 838578 | 2096.5 | 224.2 |
| final-copy | 838574 | 2094.8 | 224.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 95.75 | 64.72 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |
| final-copy | 95.63 | 64.61 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |

## cycle_inplace_sparse_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.991, 1.000] | 0/2 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 1/2 |
| final-copy | cycles | 0.998 | [0.996, 1.000] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 169.72 | 168.41 | 7.33 |
| final-copy | 168.90 | 167.10 | 7.34 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1314536 | 2932.0 | 224.3 |
| final-copy | 1314543 | 2932.7 | 224.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 143.92 | 80.71 | 543 | 26 | 7.7 | 4.0 | 3.7 | 3.7 |
| final-copy | 143.94 | 80.69 | 543 | 26 | 7.7 | 4.0 | 3.7 | 3.7 |

## cycle_inplace_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [0.990, 1.014] | 1/2 |
| final-copy | instructions | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | cycles | 1.007 | [1.000, 1.013] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.001 | [1.000, 1.001] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.000, 1.002] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 144.65 | 142.70 | 6.46 |
| final-copy | 145.23 | 142.67 | 6.49 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1036529 | 2228.2 | 223.9 |
| final-copy | 1036530 | 2229.0 | 224.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 102.66 | 66.91 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |
| final-copy | 102.68 | 67.00 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |

## cycle_inplace_sparse_plain_one_rewrite_rows
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.992, 1.006] | 1/2 |
| final-copy | instructions | 0.999 | [0.998, 1.000] | 1/2 |
| final-copy | cycles | 0.999 | [0.997, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.002 | [1.000, 1.003] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 156.10 | 153.74 | 8.20 |
| final-copy | 156.13 | 154.80 | 8.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1262205 | 2876.0 | 224.0 |
| final-copy | 1262208 | 2876.0 | 224.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 180.51 | 50.85 | 537 | 27 | 6.1 | 3.2 | 2.9 | 2.9 |
| final-copy | 180.48 | 50.82 | 537 | 27 | 6.1 | 3.2 | 2.9 | 2.9 |

## cycle_inplace_sparse_plain_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.995, 0.996] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.995 | [0.988, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 170.52 | 169.19 | 7.33 |
| final-copy | 169.53 | 168.40 | 7.31 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1314438 | 2941.1 | 224.5 |
| final-copy | 1314443 | 2941.3 | 224.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 144.59 | 80.98 | 523 | 26 | 7.7 | 4.0 | 3.7 | 3.7 |
| final-copy | 144.55 | 80.93 | 523 | 26 | 7.7 | 4.0 | 3.7 | 3.7 |

## cycle_inplace_sparse_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.993, 1.000] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.998 | [0.994, 1.002] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 208.38 | 205.60 | 5.05 |
| final-copy | 207.44 | 206.11 | 5.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1082966 | 2809.6 | 224.2 |
| final-copy | 1082969 | 2809.6 | 224.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 137.84 | 78.64 | 575 | 26 | 6.0 | 4.1 | 1.9 | 1.9 |
| final-copy | 137.78 | 78.63 | 575 | 26 | 6.0 | 4.1 | 1.9 | 1.9 |

## cycle_moving_dense_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.986 | [0.976, 0.997] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.980 | [0.978, 0.982] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 201.78 | 200.01 | 4.63 |
| final-copy | 201.00 | 197.77 | 4.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1042514 | 8980.3 | 1203.8 |
| final-copy | 1042167 | 8979.9 | 1204.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 180.23 | 15.05 | 426 | 18 | 323.4 | 323.1 | 0.2 | 0.2 |
| final-copy | 180.22 | 15.05 | 418 | 18 | 323.4 | 323.1 | 0.2 | 0.2 |

## cycle_moving_dense_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.985 | [0.976, 0.994] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 1.010 | [1.010, 1.011] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.003 | [1.000, 1.005] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 166.99 | 164.38 | 4.26 |
| final-copy | 165.76 | 163.28 | 4.34 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 875083 | 7100.9 | 1203.1 |
| final-copy | 875040 | 7100.6 | 1206.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 148.71 | 13.34 | 396 | 18 | 163.0 | 162.8 | 0.2 | 0.2 |
| final-copy | 148.71 | 13.34 | 396 | 18 | 163.0 | 162.8 | 0.2 | 0.2 |

## cycle_moving_dense_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.991, 0.999] | 0/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.992 | [0.992, 0.993] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.000, 1.003] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 165.09 | 161.62 | 5.43 |
| final-copy | 163.63 | 160.74 | 5.41 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1100560 | 9189.1 | 1209.6 |
| final-copy | 1100841 | 9189.1 | 1210.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 182.99 | 14.31 | 498 | 18 | 2.7 | 2.4 | 0.3 | 0.2 |
| final-copy | 182.94 | 14.26 | 504 | 18 | 2.7 | 2.4 | 0.3 | 0.2 |

## cycle_moving_dense_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.983 | [0.965, 1.001] | 1/2 |
| final-copy | instructions | 0.999 | [0.998, 0.999] | 0/2 |
| final-copy | cycles | 0.998 | [0.993, 1.004] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.996, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 148.91 | 147.20 | 4.83 |
| final-copy | 147.87 | 146.17 | 4.92 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 947989 | 7335.3 | 1210.4 |
| final-copy | 947738 | 7335.4 | 1208.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 152.76 | 12.93 | 476 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |
| final-copy | 152.76 | 12.93 | 470 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |

## cycle_moving_dense_plain_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.990, 0.995] | 0/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | cycles | 0.992 | [0.987, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.002 | [1.001, 1.003] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 163.97 | 163.78 | 5.41 |
| final-copy | 163.11 | 158.93 | 5.32 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1100446 | 9212.3 | 1203.4 |
| final-copy | 1100642 | 9212.6 | 1207.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 183.38 | 14.30 | 476 | 18 | 2.7 | 2.4 | 0.3 | 0.2 |
| final-copy | 183.39 | 14.31 | 480 | 18 | 2.7 | 2.4 | 0.3 | 0.2 |

## cycle_moving_dense_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.998, 1.004] | 1/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.964 | [0.962, 0.965] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.000, 1.003] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 201.33 | 198.43 | 4.63 |
| final-copy | 201.13 | 198.44 | 4.50 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1042211 | 9030.7 | 1205.1 |
| final-copy | 1042356 | 9030.4 | 1206.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 181.13 | 15.07 | 398 | 18 | 323.4 | 323.1 | 0.2 | 0.2 |
| final-copy | 181.11 | 15.08 | 400 | 18 | 323.4 | 323.1 | 0.2 | 0.2 |

## cycle_moving_sparse_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.991, 1.000] | 1/2 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 1.000 | [0.981, 1.019] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.994 | [0.994, 0.994] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 72.85 | 71.87 | 5.23 |
| final-copy | 72.87 | 71.21 | 5.27 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 383482 | 773.0 | 41.5 |
| final-copy | 383484 | 773.0 | 41.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 61.74 | 0.05 | 726 | 18 | 4.1 | 3.9 | 0.2 | 0.2 |
| final-copy | 61.74 | 0.05 | 726 | 18 | 4.1 | 3.9 | 0.2 | 0.2 |

## cycle_moving_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.973, 1.008] | 1/2 |
| final-copy | instructions | 0.999 | [0.998, 0.999] | 0/2 |
| final-copy | cycles | 0.992 | [0.964, 1.021] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 57.81 | 54.97 | 4.11 |
| final-copy | 57.03 | 54.42 | 4.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 251072 | 449.2 | 23.6 |
| final-copy | 251074 | 449.2 | 23.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 34.33 | 0.04 | 705 | 18 | 3.3 | 3.1 | 0.2 | 0.2 |
| final-copy | 34.33 | 0.04 | 705 | 18 | 3.3 | 3.1 | 0.2 | 0.2 |

## cycle_moving_sparse_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.013 | [0.988, 1.039] | 1/2 |
| final-copy | instructions | 1.000 | [0.998, 1.001] | 1/2 |
| final-copy | cycles | 1.001 | [0.977, 1.026] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.010 | [1.003, 1.017] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 79.82 | 79.05 | 5.30 |
| final-copy | 80.66 | 76.97 | 5.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 463316 | 827.7 | 37.5 |
| final-copy | 463321 | 827.7 | 37.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 66.79 | 0.04 | 770 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |
| final-copy | 66.79 | 0.04 | 770 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |

## cycle_moving_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.983, 1.011] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | cycles | 1.026 | [1.014, 1.039] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 69.42 | 67.39 | 3.93 |
| final-copy | 69.64 | 66.94 | 3.94 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 329930 | 502.9 | 20.0 |
| final-copy | 329929 | 502.9 | 20.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 39.31 | 0.04 | 750 | 18 | 2.6 | 2.4 | 0.2 | 0.2 |
| final-copy | 39.31 | 0.04 | 750 | 18 | 2.6 | 2.4 | 0.2 | 0.2 |

## cycle_moving_sparse_plain_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.984, 1.016] | 1/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 1.002 | [0.996, 1.007] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.989 | [0.975, 1.003] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 80.09 | 78.99 | 5.31 |
| final-copy | 79.87 | 76.92 | 5.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 463182 | 832.1 | 38.5 |
| final-copy | 463186 | 832.2 | 38.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 67.19 | 0.04 | 749 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |
| final-copy | 67.19 | 0.04 | 749 | 18 | 2.7 | 2.4 | 0.2 | 0.2 |

## cycle_moving_sparse_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [1.000, 1.011] | 2/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 1.001 | [0.999, 1.003] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.991 | [0.988, 0.995] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 73.53 | 72.16 | 5.14 |
| final-copy | 74.00 | 73.03 | 5.16 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 383348 | 781.1 | 42.9 |
| final-copy | 383350 | 781.1 | 42.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 62.42 | 0.05 | 705 | 18 | 4.1 | 3.9 | 0.2 | 0.2 |
| final-copy | 62.42 | 0.05 | 705 | 18 | 4.1 | 3.9 | 0.2 | 0.2 |

## filter_null_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.013 | [1.003, 1.023] | 2/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | cycles | 0.966 | [0.965, 0.967] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.996 | [0.990, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.20 | 7.13 | 9.55 |
| final-copy | 7.30 | 7.22 | 9.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 82656 | 166.1 | 18.7 |
| final-copy | 82656 | 166.1 | 18.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.70 | 0.00 | 10 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |
| final-copy | 13.70 | 0.00 | 10 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |

## filter_null_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.985, 1.001] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | cycles | 1.010 | [1.000, 1.020] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [0.994, 1.008] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.83 | 8.70 | 9.93 |
| final-copy | 8.72 | 8.65 | 10.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 99816 | 165.6 | 19.0 |
| final-copy | 99816 | 165.6 | 19.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.82 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 13.82 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## filter_null_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.985, 0.993] | 0/2 |
| final-copy | instructions | 0.999 | [0.998, 0.999] | 0/2 |
| final-copy | cycles | 0.979 | [0.972, 0.986] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.014 | [1.001, 1.028] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 9.47 | 9.33 | 8.77 |
| final-copy | 9.38 | 9.30 | 8.70 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 99626 | 127.2 | 13.4 |
| final-copy | 99626 | 127.2 | 13.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 9.31 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 9.31 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## filter_null_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.972, 1.013] | 1/2 |
| final-copy | instructions | 0.996 | [0.996, 0.996] | 0/2 |
| final-copy | cycles | 0.966 | [0.934, 0.998] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.009 | [0.957, 1.063] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.97 | 1.91 | 5.27 |
| final-copy | 1.95 | 1.80 | 5.22 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21431 | 19.2 | 0.8 |
| final-copy | 21431 | 19.2 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## filter_null_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | instructions | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | cycles | 0.986 | [0.983, 0.989] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.988 | [0.981, 0.995] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.21 | 7.13 | 9.82 |
| final-copy | 7.21 | 7.03 | 9.69 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 81895 | 158.4 | 18.3 |
| final-copy | 81896 | 158.4 | 18.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.71 | 0.00 | 10 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |
| final-copy | 13.71 | 0.00 | 10 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |

## filter_null_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.984, 1.008] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 2/2 |
| final-copy | cycles | 0.987 | [0.980, 0.993] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.991 | [0.981, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 9.30 | 8.75 | 6.88 |
| final-copy | 9.31 | 8.92 | 6.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 83995 | 165.3 | 22.4 |
| final-copy | 83996 | 165.3 | 22.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.57 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |
| final-copy | 13.57 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |

## filter_null_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [0.969, 1.042] | 1/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 0.970 | [0.861, 1.093] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.998, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 11.88 | 11.09 | 5.68 |
| final-copy | 11.75 | 11.11 | 5.51 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 83996 | 187.9 | 22.6 |
| final-copy | 83996 | 187.9 | 22.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.59 | 0.00 | 10 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |
| final-copy | 13.59 | 0.00 | 10 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |

## filter_null_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.994, 1.000] | 0/2 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 1/2 |
| final-copy | cycles | 1.006 | [1.001, 1.011] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.007 | [0.999, 1.015] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.19 | 7.10 | 9.77 |
| final-copy | 7.17 | 7.10 | 9.85 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 81895 | 160.6 | 19.0 |
| final-copy | 81896 | 160.6 | 19.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.89 | 0.00 | 10 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |
| final-copy | 13.89 | 0.00 | 10 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |

## filter_null_plain_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.994 | [0.976, 1.012] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | cycles | 1.000 | [0.985, 1.016] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.035 | [1.026, 1.045] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 12.27 | 11.61 | 5.89 |
| final-copy | 12.32 | 11.54 | 5.92 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 86550 | 249.8 | 28.7 |
| final-copy | 86550 | 249.8 | 29.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 14.14 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | 14.14 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |

## filter_null_ready_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.024 | [0.988, 1.060] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | cycles | 1.094 | [0.995, 1.204] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.17 | 17.48 | 4.05 |
| final-copy | 18.89 | 17.57 | 4.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 87135 | 252.2 | 32.4 |
| final-copy | 87135 | 252.2 | 32.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.83 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |
| final-copy | 13.83 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |

## reopen_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.987 | [0.966, 1.008] | 1/2 |
| final-copy | instructions | 0.994 | [0.984, 1.004] | 1/2 |
| final-copy | cycles | 0.989 | [0.924, 1.058] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.16 | 0.14 | 1.09 |
| final-copy | 0.16 | 0.14 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 472 | 0.1 | 0.0 |
| final-copy | 472 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 3.3 | 2.3 | 1.0 | 1.0 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 3.3 | 2.3 | 1.0 | 1.0 |

## reopen_chain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.971, 1.023] | 1/2 |
| final-copy | instructions | 0.998 | [0.993, 1.004] | 1/2 |
| final-copy | cycles | 0.987 | [0.969, 1.005] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.28 | 0.25 | 1.05 |
| final-copy | 0.28 | 0.26 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1140 | 0.5 | 0.2 |
| final-copy | 1140 | 0.5 | 0.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 61.4 | 58.7 | 2.7 | 2.7 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 61.4 | 58.7 | 2.7 | 2.7 |

## reopen_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.024 | [1.018, 1.029] | 2/2 |
| final-copy | instructions | 0.995 | [0.986, 1.004] | 1/2 |
| final-copy | cycles | 1.019 | [0.985, 1.055] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.15 | 0.14 | 1.07 |
| final-copy | 0.16 | 0.14 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 461 | 0.1 | 0.0 |
| final-copy | 461 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 3.1 | 2.2 | 0.9 | 0.9 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 3.1 | 2.2 | 0.9 | 0.9 |

## reopen_one_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.979 | [0.978, 0.980] | 0/2 |
| final-copy | instructions | 0.996 | [0.990, 1.002] | 1/2 |
| final-copy | cycles | 0.970 | [0.966, 0.974] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.21 | 0.19 | 1.07 |
| final-copy | 0.21 | 0.18 | 1.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 520 | 1.3 | 0.4 |
| final-copy | 520 | 1.3 | 0.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 328.4 | 162.7 | 165.7 | 165.7 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 328.4 | 162.7 | 165.7 | 165.7 |

## reopen_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.036 | [1.031, 1.042] | 2/2 |
| final-copy | instructions | 0.995 | [0.991, 1.000] | 0/2 |
| final-copy | cycles | 1.019 | [1.009, 1.030] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.21 | 0.19 | 1.07 |
| final-copy | 0.22 | 0.18 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 594 | 0.1 | 0.1 |
| final-copy | 594 | 0.1 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 13.1 | 10.3 | 2.7 | 2.7 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 13.1 | 10.3 | 2.7 | 2.7 |

## reopen_one_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.027 | [1.022, 1.032] | 2/2 |
| final-copy | instructions | 0.997 | [0.991, 1.003] | 1/2 |
| final-copy | cycles | 1.021 | [0.989, 1.054] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.22 | 0.21 | 1.08 |
| final-copy | 0.23 | 0.21 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 687 | 0.1 | 0.1 |
| final-copy | 687 | 0.1 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 12.0 | 11.8 | 0.2 | 0.2 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 12.0 | 11.8 | 0.2 | 0.2 |

## reopen_one_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.949 | [0.922, 0.976] | 0/2 |
| final-copy | instructions | 0.998 | [0.992, 1.005] | 1/2 |
| final-copy | cycles | 0.965 | [0.962, 0.969] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.29 | 0.25 | 1.05 |
| final-copy | 0.27 | 0.25 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1003 | 0.3 | 0.1 |
| final-copy | 1003 | 0.3 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 36.3 | 33.5 | 2.7 | 2.7 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 36.3 | 33.5 | 2.7 | 2.7 |

## reopen_one_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.984 | [0.981, 0.987] | 0/2 |
| final-copy | instructions | 1.000 | [0.994, 1.005] | 1/2 |
| final-copy | cycles | 0.965 | [0.934, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.32 | 0.29 | 1.05 |
| final-copy | 0.31 | 0.29 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1367 | 0.4 | 0.2 |
| final-copy | 1367 | 0.4 | 0.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 39.7 | 39.5 | 0.2 | 0.2 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 39.7 | 39.5 | 0.2 | 0.2 |

## reopen_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [1.001, 1.003] | 2/2 |
| final-copy | instructions | 0.997 | [0.992, 1.003] | 1/2 |
| final-copy | cycles | 0.987 | [0.937, 1.039] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.16 | 0.14 | 1.08 |
| final-copy | 0.17 | 0.14 | 1.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 445 | 0.1 | 0.0 |
| final-copy | 445 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 2.9 | 2.1 | 0.9 | 0.9 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 2.9 | 2.1 | 0.9 | 0.9 |

## reopen_plain_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.945, 1.043] | 1/2 |
| final-copy | instructions | 0.995 | [0.986, 1.004] | 1/2 |
| final-copy | cycles | 1.019 | [0.945, 1.100] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.16 | 0.14 | 1.08 |
| final-copy | 0.15 | 0.13 | 1.09 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 334 | 0.1 | 0.0 |
| final-copy | 334 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 4.3 | 2.3 | 2.0 | 2.0 |

## reopen_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.022 | [0.980, 1.065] | 1/2 |
| final-copy | instructions | 0.998 | [0.989, 1.007] | 1/2 |
| final-copy | cycles | 1.029 | [0.993, 1.066] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.16 | 0.15 | 1.08 |
| final-copy | 0.17 | 0.15 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 459 | 0.1 | 0.0 |
| final-copy | 459 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 5.7 | 3.8 | 2.0 | 2.0 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 5.7 | 3.8 | 2.0 | 2.0 |

## reopen_plain_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.025 | [1.023, 1.027] | 2/2 |
| final-copy | instructions | 0.997 | [0.991, 1.003] | 1/2 |
| final-copy | cycles | 0.962 | [0.937, 0.989] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.20 | 0.18 | 1.08 |
| final-copy | 0.20 | 0.18 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 539 | 0.1 | 0.0 |
| final-copy | 539 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 5.5 | 5.2 | 0.2 | 0.2 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 5.5 | 5.2 | 0.2 | 0.2 |

## reopen_plain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.987 | [0.984, 0.991] | 0/2 |
| final-copy | instructions | 0.998 | [0.994, 1.002] | 1/2 |
| final-copy | cycles | 0.995 | [0.984, 1.006] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.24 | 0.22 | 1.05 |
| final-copy | 0.23 | 0.23 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 863 | 0.2 | 0.1 |
| final-copy | 863 | 0.2 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 10.3 | 8.4 | 2.0 | 2.0 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 10.3 | 8.4 | 2.0 | 2.0 |

## reopen_plain_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.016 | [0.985, 1.047] | 1/2 |
| final-copy | instructions | 0.997 | [0.990, 1.003] | 1/2 |
| final-copy | cycles | 1.006 | [0.962, 1.053] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.27 | 0.25 | 1.05 |
| final-copy | 0.27 | 0.24 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1121 | 0.2 | 0.1 |
| final-copy | 1121 | 0.2 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 14.5 | 14.2 | 0.2 | 0.2 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 14.5 | 14.2 | 0.2 | 0.2 |

## reopen_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.970, 1.012] | 1/2 |
| final-copy | instructions | 0.996 | [0.989, 1.002] | 1/2 |
| final-copy | cycles | 0.998 | [0.979, 1.017] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.16 | 0.14 | 1.06 |
| final-copy | 0.16 | 0.15 | 1.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 472 | 0.1 | 0.0 |
| final-copy | 472 | 0.1 | 0.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 3.3 | 2.3 | 1.0 | 1.0 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 3.3 | 2.3 | 1.0 | 1.0 |

## reopen_shared_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.959, 1.045] | 1/2 |
| final-copy | instructions | 0.998 | [0.993, 1.004] | 1/2 |
| final-copy | cycles | 0.989 | [0.929, 1.054] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.28 | 0.24 | 1.05 |
| final-copy | 0.28 | 0.27 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 1140 | 0.5 | 0.2 |
| final-copy | 1140 | 0.5 | 0.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 61.4 | 58.7 | 2.7 | 2.7 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 61.4 | 58.7 | 2.7 | 2.7 |

## scan_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.986, 1.004] | 1/2 |
| final-copy | instructions | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | cycles | 0.999 | [0.992, 1.007] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.995 | [0.990, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.53 | 6.39 | 9.82 |
| final-copy | 6.46 | 6.42 | 9.94 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 73234 | 154.0 | 18.7 |
| final-copy | 73234 | 154.0 | 18.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.70 | 0.00 | 10 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |
| final-copy | 13.70 | 0.00 | 10 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.010 | [1.010, 1.011] | 2/2 |
| final-copy | instructions | 1.001 | [1.001, 1.001] | 2/2 |
| final-copy | cycles | 0.996 | [0.991, 1.001] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.002 | [1.001, 1.002] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.92 | 7.83 | 10.80 |
| final-copy | 8.00 | 7.83 | 10.73 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 89094 | 157.0 | 19.1 |
| final-copy | 89095 | 157.0 | 19.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.82 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 13.82 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [1.003, 1.007] | 2/2 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 1/2 |
| final-copy | cycles | 0.972 | [0.965, 0.978] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.993 | [0.984, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 7.81 | 7.71 | 10.98 |
| final-copy | 7.86 | 7.74 | 10.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 88904 | 107.3 | 13.4 |
| final-copy | 88905 | 107.3 | 13.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 9.31 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 9.31 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.969 | [0.967, 0.972] | 0/2 |
| final-copy | instructions | 0.998 | [0.991, 1.006] | 1/2 |
| final-copy | cycles | 0.989 | [0.952, 1.027] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.028 | [1.015, 1.041] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1.22 | 1.14 | 4.84 |
| final-copy | 1.19 | 1.13 | 4.86 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 12010 | 7.1 | 0.7 |
| final-copy | 12010 | 7.1 | 0.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.992, 1.005] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | cycles | 0.991 | [0.982, 1.000] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.011 | [1.001, 1.020] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.53 | 6.42 | 10.43 |
| final-copy | 6.51 | 6.29 | 10.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 71955 | 150.1 | 18.8 |
| final-copy | 71954 | 150.1 | 18.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.71 | 0.00 | 10 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |
| final-copy | 13.71 | 0.00 | 10 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.986, 1.003] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 2/2 |
| final-copy | cycles | 0.991 | [0.966, 1.017] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [0.998, 1.003] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.28 | 7.90 | 7.74 |
| final-copy | 8.21 | 7.68 | 7.78 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 73146 | 156.4 | 22.4 |
| final-copy | 73145 | 156.4 | 22.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.57 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |
| final-copy | 13.57 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |

## scan_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.978 | [0.960, 0.995] | 0/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 0.948 | [0.912, 0.987] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.988 | [0.982, 0.994] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 8.79 | 8.35 | 7.33 |
| final-copy | 8.54 | 8.24 | 6.94 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 73145 | 156.6 | 22.4 |
| final-copy | 73144 | 156.6 | 22.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.59 | 0.00 | 10 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |
| final-copy | 13.59 | 0.00 | 10 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |

## scan_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.990 | [0.987, 0.993] | 0/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | cycles | 0.990 | [0.983, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.977, 1.024] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.49 | 6.37 | 10.51 |
| final-copy | 6.43 | 6.33 | 10.46 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 71954 | 152.3 | 18.6 |
| final-copy | 71954 | 152.3 | 18.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.89 | 0.00 | 10 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |
| final-copy | 13.89 | 0.00 | 10 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |

## scan_plain_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [0.992, 1.020] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 1/2 |
| final-copy | cycles | 0.995 | [0.979, 1.011] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 11.48 | 11.26 | 5.87 |
| final-copy | 11.54 | 11.22 | 5.88 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 76698 | 249.0 | 28.7 |
| final-copy | 76697 | 249.0 | 28.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 14.14 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | 14.14 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |

## scan_ready_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.985, 1.005] | 1/2 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | cycles | 0.994 | [0.947, 1.044] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.987 | [0.978, 0.996] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 17.12 | 16.40 | 3.79 |
| final-copy | 16.98 | 16.28 | 3.77 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 77283 | 251.4 | 32.3 |
| final-copy | 77282 | 251.4 | 32.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.83 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |
| final-copy | 13.83 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |

## take_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.987 | [0.951, 1.025] | 1/2 |
| final-copy | instructions | 0.984 | [0.977, 0.991] | 0/2 |
| final-copy | cycles | 0.973 | [0.959, 0.987] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.013 | [0.950, 1.080] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 5.87 | 5.54 | 10.77 |
| final-copy | 5.86 | 4.86 | 10.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21780 | 35.0 | 3.8 |
| final-copy | 21781 | 35.0 | 4.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.67 | 0.00 | 513 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |
| final-copy | 3.67 | 0.00 | 513 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |

## take_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.993, 0.997] | 0/2 |
| final-copy | instructions | 0.995 | [0.988, 1.001] | 1/2 |
| final-copy | cycles | 0.967 | [0.924, 1.013] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.17 | 5.69 | 10.47 |
| final-copy | 6.15 | 6.02 | 10.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 25335 | 35.8 | 7.8 |
| final-copy | 25336 | 35.8 | 7.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.70 | 0.00 | 513 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 3.70 | 0.00 | 513 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## take_null_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.030 | [1.022, 1.038] | 2/2 |
| final-copy | instructions | 0.979 | [0.976, 0.983] | 0/2 |
| final-copy | cycles | 0.997 | [0.965, 1.030] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.02 | 5.17 | 10.51 |
| final-copy | 6.07 | 4.88 | 10.22 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 25327 | 24.1 | 5.3 |
| final-copy | 25327 | 24.1 | 5.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 2.51 | 0.00 | 512 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 2.51 | 0.00 | 512 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## take_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.968 | [0.917, 1.023] | 1/2 |
| final-copy | instructions | 0.998 | [0.997, 0.999] | 0/2 |
| final-copy | cycles | 1.068 | [1.013, 1.126] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.001, 1.001] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 0.38 | 0.31 | 2.41 |
| final-copy | 0.36 | 0.31 | 2.66 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 4100 | 0.7 | 0.2 |
| final-copy | 4100 | 0.7 | 0.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## take_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.003 | [0.992, 1.015] | 1/2 |
| final-copy | instructions | 0.999 | [0.987, 1.010] | 1/2 |
| final-copy | cycles | 0.998 | [0.970, 1.027] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 5.89 | 5.04 | 10.83 |
| final-copy | 5.86 | 4.89 | 10.97 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21589 | 35.0 | 7.7 |
| final-copy | 21589 | 35.0 | 7.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.67 | 0.00 | 513 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |
| final-copy | 3.67 | 0.00 | 513 | 0 | 3.1 | 2.2 | 0.9 | 0.9 |

## take_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.979 | [0.963, 0.994] | 0/2 |
| final-copy | instructions | 0.993 | [0.989, 0.997] | 0/2 |
| final-copy | cycles | 0.976 | [0.965, 0.986] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 5.91 | 5.29 | 10.83 |
| final-copy | 5.73 | 4.95 | 11.01 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21787 | 34.7 | 7.6 |
| final-copy | 21788 | 34.7 | 7.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.64 | 0.00 | 513 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |
| final-copy | 3.64 | 0.00 | 513 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |

## take_partial_50pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.976 | [0.969, 0.983] | 0/2 |
| final-copy | instructions | 0.993 | [0.984, 1.002] | 1/2 |
| final-copy | cycles | 0.980 | [0.942, 1.020] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.001] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.02 | 5.63 | 10.83 |
| final-copy | 5.85 | 5.62 | 10.81 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21791 | 34.7 | 7.7 |
| final-copy | 21791 | 34.7 | 7.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.64 | 0.00 | 513 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |
| final-copy | 3.64 | 0.00 | 513 | 0 | 486.6 | 163.3 | 323.3 | 323.3 |

## take_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.977 | [0.963, 0.992] | 0/2 |
| final-copy | instructions | 0.988 | [0.978, 0.998] | 0/2 |
| final-copy | cycles | 1.029 | [1.011, 1.047] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 6.07 | 5.41 | 10.51 |
| final-copy | 5.90 | 5.52 | 10.73 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 21589 | 35.6 | 7.8 |
| final-copy | 21589 | 35.6 | 7.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.72 | 0.00 | 513 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |
| final-copy | 3.72 | 0.00 | 513 | 0 | 2.9 | 2.1 | 0.9 | 0.9 |

## take_plain_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.007 | [0.995, 1.021] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 2/2 |
| final-copy | cycles | 1.016 | [1.004, 1.029] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [1.000, 1.002] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1547.41 | 1534.20 | 1.04 |
| final-copy | 1557.62 | 1541.34 | 1.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 22089 | 35.9 | 7.9 |
| final-copy | 22088 | 35.9 | 7.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.78 | 0.00 | 518 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | 3.78 | 0.00 | 518 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |

## take_ready_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.991, 0.994] | 0/2 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 0/2 |
| final-copy | cycles | 0.994 | [0.989, 1.000] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 1565.51 | 1539.51 | 1.05 |
| final-copy | 1544.07 | 1531.33 | 1.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 22188 | 35.1 | 7.7 |
| final-copy | 22189 | 35.1 | 7.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.70 | 0.00 | 518 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |
| final-copy | 3.70 | 0.00 | 518 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |

## unrelated_inplace_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.988, 1.011] | 1/2 |
| final-copy | instructions | 1.001 | [1.001, 1.002] | 2/2 |
| final-copy | cycles | 0.996 | [0.985, 1.007] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.012 | [0.996, 1.028] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 19.54 | 19.24 | 6.40 |
| final-copy | 19.60 | 19.26 | 6.37 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 194853 | 195.1 | 21.0 |
| final-copy | 194855 | 195.1 | 20.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.07 | 4.71 | 91 | 13 | 6.3 | 3.2 | 3.1 | 3.1 |
| final-copy | 7.07 | 4.71 | 91 | 13 | 6.3 | 3.2 | 3.1 | 3.1 |

## unrelated_inplace_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.991, 1.017] | 1/2 |
| final-copy | instructions | 0.994 | [0.991, 0.997] | 0/2 |
| final-copy | cycles | 0.996 | [0.976, 1.017] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.034 | [1.019, 1.049] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 19.66 | 19.10 | 6.48 |
| final-copy | 19.73 | 19.38 | 6.41 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 194848 | 195.1 | 20.3 |
| final-copy | 194849 | 195.1 | 20.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.07 | 4.71 | 91 | 13 | 6.2 | 3.1 | 3.1 | 3.1 |
| final-copy | 7.07 | 4.71 | 91 | 13 | 6.2 | 3.1 | 3.1 | 3.1 |

## unrelated_inplace_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.007 | [1.004, 1.010] | 2/2 |
| final-copy | instructions | 0.989 | [0.986, 0.991] | 0/2 |
| final-copy | cycles | 0.991 | [0.990, 0.992] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 1.015 | [0.996, 1.033] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 19.50 | 19.27 | 6.49 |
| final-copy | 19.64 | 19.36 | 6.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 194591 | 195.1 | 21.3 |
| final-copy | 194590 | 195.1 | 21.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.07 | 4.71 | 91 | 13 | 5.6 | 3.0 | 2.6 | 2.6 |
| final-copy | 7.07 | 4.71 | 91 | 13 | 5.6 | 3.0 | 2.6 | 2.6 |

## unrelated_inplace_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.983, 0.995] | 0/2 |
| final-copy | instructions | 0.996 | [0.991, 1.001] | 1/2 |
| final-copy | cycles | 0.986 | [0.975, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.008 | [1.004, 1.012] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 19.68 | 19.50 | 6.42 |
| final-copy | 19.46 | 19.30 | 6.42 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 194856 | 195.1 | 20.8 |
| final-copy | 194858 | 195.1 | 20.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.07 | 4.71 | 91 | 13 | 6.3 | 3.2 | 3.1 | 3.1 |
| final-copy | 7.07 | 4.71 | 91 | 13 | 6.3 | 3.2 | 3.1 | 3.1 |

## unrelated_moving_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.990 | [0.980, 1.001] | 1/2 |
| final-copy | instructions | 0.996 | [0.995, 0.997] | 0/2 |
| final-copy | cycles | 1.032 | [0.993, 1.072] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.982 | [0.941, 1.025] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.48 | 17.82 | 5.24 |
| final-copy | 18.25 | 17.75 | 5.53 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85316 | 77.1 | 7.6 |
| final-copy | 85318 | 77.1 | 7.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.87 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |
| final-copy | 6.87 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |

## unrelated_moving_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.990 | [0.964, 1.017] | 1/2 |
| final-copy | instructions | 0.999 | [0.997, 1.000] | 1/2 |
| final-copy | cycles | 1.016 | [1.008, 1.025] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.986 | [0.967, 1.005] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.56 | 17.88 | 5.32 |
| final-copy | 18.30 | 17.40 | 5.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85188 | 77.2 | 7.6 |
| final-copy | 85188 | 77.2 | 7.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.88 | 0.03 | 664 | 14 | 5.8 | 3.1 | 2.7 | 2.7 |
| final-copy | 6.88 | 0.03 | 664 | 14 | 5.8 | 3.1 | 2.7 | 2.7 |

## unrelated_moving_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.998 | [0.968, 1.029] | 1/2 |
| final-copy | instructions | 0.995 | [0.993, 0.997] | 0/2 |
| final-copy | cycles | 0.995 | [0.970, 1.020] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.010 | [0.995, 1.025] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.40 | 17.55 | 5.39 |
| final-copy | 18.13 | 17.38 | 5.34 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 84841 | 77.2 | 7.4 |
| final-copy | 84841 | 77.2 | 7.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.88 | 0.03 | 664 | 14 | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | 6.88 | 0.03 | 664 | 14 | 4.3 | 2.3 | 2.0 | 2.0 |

## unrelated_moving_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.965, 1.017] | 1/2 |
| final-copy | instructions | 0.995 | [0.992, 0.997] | 0/2 |
| final-copy | cycles | 0.975 | [0.974, 0.975] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.018 | [1.017, 1.020] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.12 | 17.36 | 5.35 |
| final-copy | 17.85 | 17.32 | 5.37 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85334 | 77.2 | 7.3 |
| final-copy | 85334 | 77.2 | 7.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.88 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |
| final-copy | 6.88 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |

## update_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.010 | [1.005, 1.016] | 2/2 |
| final-copy | instructions | 1.000 | [0.995, 1.004] | 1/2 |
| final-copy | cycles | 1.001 | [0.998, 1.003] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.017 | [0.988, 1.047] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.37 | 17.70 | 5.38 |
| final-copy | 18.51 | 18.05 | 5.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85309 | 77.1 | 7.6 |
| final-copy | 85308 | 77.1 | 7.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.85 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |
| final-copy | 6.85 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |

## update_chain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.991, 1.001] | 1/2 |
| final-copy | instructions | 0.997 | [0.994, 0.999] | 0/2 |
| final-copy | cycles | 0.971 | [0.959, 0.983] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.978 | [0.972, 0.984] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 35.44 | 34.32 | 5.18 |
| final-copy | 35.29 | 34.44 | 5.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 202521 | 103.2 | 10.5 |
| final-copy | 202522 | 103.2 | 10.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.24 | 0.11 | 1009 | 15 | 62.4 | 59.5 | 2.9 | 2.9 |
| final-copy | 7.24 | 0.11 | 1009 | 15 | 62.4 | 59.5 | 2.9 | 2.9 |

## update_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.978 | [0.942, 1.016] | 1/2 |
| final-copy | instructions | 0.997 | [0.995, 0.999] | 0/2 |
| final-copy | cycles | 1.019 | [1.018, 1.019] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.990 | [0.981, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.63 | 18.05 | 5.30 |
| final-copy | 18.25 | 17.59 | 5.35 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85172 | 77.1 | 7.6 |
| final-copy | 85172 | 77.1 | 7.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.85 | 0.03 | 664 | 14 | 5.8 | 3.1 | 2.7 | 2.7 |
| final-copy | 6.85 | 0.03 | 664 | 14 | 5.8 | 3.1 | 2.7 | 2.7 |

## update_one_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [1.003, 1.008] | 2/2 |
| final-copy | instructions | 0.995 | [0.993, 0.996] | 0/2 |
| final-copy | cycles | 1.015 | [0.999, 1.032] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.996 | [0.993, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 27.67 | 26.55 | 4.55 |
| final-copy | 27.93 | 26.79 | 4.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 93642 | 200.7 | 27.2 |
| final-copy | 93644 | 200.7 | 27.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.51 | 0.33 | 689 | 15 | 165.7 | 162.8 | 2.9 | 2.9 |
| final-copy | 7.51 | 0.33 | 689 | 15 | 165.7 | 162.8 | 2.9 | 2.9 |

## update_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.020 | [1.015, 1.026] | 2/2 |
| final-copy | instructions | 1.001 | [0.997, 1.004] | 1/2 |
| final-copy | cycles | 1.030 | [1.003, 1.058] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.978 | [0.976, 0.980] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.46 | 28.15 | 4.47 |
| final-copy | 29.04 | 27.38 | 4.61 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 118086 | 86.0 | 8.5 |
| final-copy | 118086 | 86.0 | 8.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.96 | 0.05 | 757 | 15 | 13.7 | 10.8 | 2.9 | 2.9 |
| final-copy | 6.96 | 0.05 | 757 | 15 | 13.7 | 10.8 | 2.9 | 2.9 |

## update_one_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [1.004, 1.009] | 2/2 |
| final-copy | instructions | 1.001 | [0.995, 1.008] | 1/2 |
| final-copy | cycles | 1.004 | [0.974, 1.034] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.981 | [0.956, 1.006] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.53 | 27.61 | 4.55 |
| final-copy | 28.95 | 27.87 | 4.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 118530 | 86.2 | 8.3 |
| final-copy | 118532 | 86.2 | 8.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.96 | 0.05 | 758 | 15 | 15.3 | 12.4 | 2.9 | 2.9 |
| final-copy | 6.96 | 0.05 | 758 | 15 | 15.3 | 12.4 | 2.9 | 2.9 |

## update_one_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.016 | [1.014, 1.018] | 2/2 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 1/2 |
| final-copy | cycles | 1.007 | [0.995, 1.019] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.005 | [0.995, 1.016] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 34.59 | 34.16 | 5.19 |
| final-copy | 35.09 | 34.62 | 5.24 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 202193 | 103.1 | 10.5 |
| final-copy | 202194 | 103.1 | 10.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.25 | 0.08 | 1009 | 15 | 36.8 | 34.0 | 2.9 | 2.9 |
| final-copy | 7.25 | 0.08 | 1009 | 15 | 36.8 | 34.0 | 2.9 | 2.9 |

## update_one_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [1.001, 1.003] | 2/2 |
| final-copy | instructions | 0.998 | [0.996, 1.000] | 0/2 |
| final-copy | cycles | 0.987 | [0.986, 0.989] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.010 | [0.999, 1.020] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 34.92 | 34.58 | 5.10 |
| final-copy | 35.04 | 33.84 | 5.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 203424 | 103.6 | 10.3 |
| final-copy | 203424 | 103.6 | 10.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.24 | 0.09 | 1010 | 15 | 43.0 | 40.0 | 3.0 | 2.9 |
| final-copy | 7.24 | 0.09 | 1010 | 15 | 43.0 | 40.0 | 3.0 | 2.9 |

## update_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.990, 0.997] | 0/2 |
| final-copy | instructions | 0.995 | [0.989, 1.000] | 0/2 |
| final-copy | cycles | 1.010 | [0.982, 1.038] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.982 | [0.974, 0.989] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.46 | 17.78 | 5.28 |
| final-copy | 18.27 | 17.77 | 5.31 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 84840 | 77.1 | 7.7 |
| final-copy | 84840 | 77.1 | 7.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.86 | 0.03 | 664 | 14 | 4.3 | 2.3 | 2.0 | 2.0 |
| final-copy | 6.86 | 0.03 | 664 | 14 | 4.3 | 2.3 | 2.0 | 2.0 |

## update_plain_dense
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.987 | [0.982, 0.992] | 0/2 |
| final-copy | instructions | 0.994 | [0.993, 0.995] | 0/2 |
| final-copy | cycles | 0.959 | [0.941, 0.977] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [0.998, 1.004] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.18 | 27.58 | 4.77 |
| final-copy | 27.79 | 26.61 | 4.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 93112 | 199.9 | 27.1 |
| final-copy | 93112 | 199.9 | 27.1 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.52 | 0.17 | 689 | 15 | 4.6 | 2.4 | 2.1 | 2.1 |
| final-copy | 7.52 | 0.17 | 689 | 15 | 4.6 | 2.4 | 2.1 | 2.1 |

## update_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.988, 1.019] | 1/2 |
| final-copy | instructions | 1.001 | [1.000, 1.003] | 1/2 |
| final-copy | cycles | 1.018 | [1.011, 1.024] | 2/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.025 | [1.006, 1.043] | 2/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.76 | 27.85 | 4.55 |
| final-copy | 28.87 | 27.65 | 4.61 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 117508 | 86.0 | 8.4 |
| final-copy | 117510 | 86.0 | 8.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.96 | 0.04 | 757 | 15 | 6.0 | 3.9 | 2.1 | 2.1 |
| final-copy | 6.96 | 0.04 | 757 | 15 | 6.0 | 3.9 | 2.1 | 2.1 |

## update_plain_h16r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.983, 1.018] | 1/2 |
| final-copy | instructions | 0.997 | [0.994, 1.000] | 0/2 |
| final-copy | cycles | 0.993 | [0.980, 1.006] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 1.001 | [0.994, 1.007] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 28.65 | 28.15 | 4.59 |
| final-copy | 28.56 | 27.37 | 4.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 118012 | 86.2 | 8.6 |
| final-copy | 118014 | 86.2 | 8.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.03 | 0.04 | 792 | 15 | 7.5 | 5.3 | 2.2 | 2.2 |
| final-copy | 7.03 | 0.04 | 792 | 15 | 7.5 | 5.3 | 2.2 | 2.2 |

## update_plain_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.974, 1.005] | 1/2 |
| final-copy | instructions | 0.996 | [0.988, 1.004] | 1/2 |
| final-copy | cycles | 1.003 | [0.963, 1.045] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.982 | [0.973, 0.990] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 35.37 | 34.28 | 4.98 |
| final-copy | 34.94 | 33.99 | 5.11 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 201565 | 102.8 | 10.5 |
| final-copy | 201564 | 102.8 | 10.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.24 | 0.06 | 1009 | 15 | 10.5 | 8.5 | 2.1 | 2.1 |
| final-copy | 7.24 | 0.06 | 1009 | 15 | 10.5 | 8.4 | 2.1 | 2.1 |

## update_plain_h64r
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.989, 0.996] | 0/2 |
| final-copy | instructions | 1.000 | [0.999, 1.002] | 1/2 |
| final-copy | cycles | 0.982 | [0.968, 0.997] | 0/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | peak_live_growth_bytes | 0.997 | [0.994, 0.999] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 35.61 | 35.07 | 5.34 |
| final-copy | 35.54 | 33.85 | 5.32 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 202276 | 103.6 | 10.5 |
| final-copy | 202276 | 103.6 | 10.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.51 | 0.06 | 1140 | 15 | 16.5 | 14.3 | 2.2 | 2.2 |
| final-copy | 7.51 | 0.06 | 1140 | 15 | 16.5 | 14.3 | 2.2 | 2.2 |

## update_shared
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.988 | [0.970, 1.006] | 1/2 |
| final-copy | instructions | 1.000 | [0.998, 1.002] | 1/2 |
| final-copy | cycles | 1.014 | [0.998, 1.031] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/2 |
| final-copy | peak_live_growth_bytes | 0.998 | [0.997, 1.000] | 1/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 18.48 | 17.86 | 5.30 |
| final-copy | 18.33 | 17.82 | 5.41 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 85310 | 77.1 | 7.5 |
| final-copy | 85311 | 77.1 | 7.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 6.85 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |
| final-copy | 6.85 | 0.04 | 664 | 14 | 6.5 | 3.8 | 2.7 | 2.7 |

## update_shared_h64
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.996, 1.012] | 1/2 |
| final-copy | instructions | 0.997 | [0.996, 0.998] | 0/2 |
| final-copy | cycles | 0.973 | [0.916, 1.032] | 1/2 |
| final-copy | allocations | 1.000 | [1.000, 1.000] | 1/2 |
| final-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/2 |
| final-copy | peak_live_growth_bytes | 0.993 | [0.991, 0.994] | 0/2 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| final | 35.46 | 34.36 | 5.27 |
| final-copy | 35.36 | 33.95 | 5.12 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | 202520 | 103.2 | 10.6 |
| final-copy | 202519 | 103.2 | 10.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.24 | 0.11 | 1009 | 15 | 62.4 | 59.5 | 2.9 | 2.9 |
| final-copy | 7.24 | 0.11 | 1009 | 15 | 62.4 | 59.5 | 2.9 | 2.9 |

