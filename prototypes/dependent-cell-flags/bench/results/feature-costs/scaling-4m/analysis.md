## after_appends_k32_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.981, 1.005] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.003] | 3/4 |
| final-copy | cycles | 1.001 | [0.998, 1.003] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 261.84 | 260.05 | 260.72–267.92 | 1.56 |
| final-copy | 261.50 | 259.13 | 260.84–263.61 | 1.56 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 54.66 | 52.65 | 776 | 41 | 21.1 | 16.9 | 4.2 | 4.2 |
| final-copy | 54.66 | 52.60 | 776 | 41 | 21.1 | 16.9 | 4.2 | 4.2 |

## after_appends_k32_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.993, 1.020] | 2/4 |
| final-copy | instructions | 1.000 | [0.998, 1.002] | 2/4 |
| final-copy | cycles | 1.005 | [1.002, 1.009] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 102.47 | 98.62 | 99.22–103.20 | 6.21 |
| final-copy | 102.56 | 101.15 | 101.80–103.19 | 6.23 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 84.06 | 62.24 | 802 | 41 | 30.4 | 17.0 | 13.4 | 13.4 |
| final-copy | 84.06 | 62.37 | 802 | 41 | 30.4 | 17.0 | 13.4 | 13.4 |

## append_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.002 | [0.990, 1.014] | 2/4 |
| final-copy | instructions | 1.003 | [1.001, 1.006] | 3/4 |
| final-copy | cycles | 1.011 | [0.993, 1.030] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.55 | 1.49 | 1.54–1.58 | 1.18 |
| final-copy | 1.56 | 1.47 | 1.53–1.58 | 1.17 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 7.6 | 7.5 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 7.6 | 7.5 | 0.1 | 0.1 |

## append_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.990 | [0.969, 1.009] | 1/4 |
| final-copy | instructions | 0.995 | [0.989, 0.999] | 0/4 |
| final-copy | cycles | 0.982 | [0.970, 0.996] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.53 | 1.44 | 1.46–1.58 | 1.19 |
| final-copy | 1.50 | 1.42 | 1.45–1.55 | 1.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.03 | 1 | 4 | 17.0 | 16.8 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.03 | 1 | 4 | 17.0 | 16.8 | 0.1 | 0.1 |

## append_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.956, 1.033] | 3/4 |
| final-copy | instructions | 1.004 | [1.000, 1.011] | 3/4 |
| final-copy | cycles | 1.013 | [0.969, 1.053] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.53 | 1.43 | 1.52–1.59 | 1.17 |
| final-copy | 1.54 | 1.44 | 1.49–1.59 | 1.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 7.3 | 7.2 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 7.3 | 7.2 | 0.1 | 0.1 |

## append_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.021 | [0.992, 1.052] | 3/4 |
| final-copy | instructions | 1.002 | [0.999, 1.006] | 2/4 |
| final-copy | cycles | 1.010 | [0.987, 1.042] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.58 | 1.49 | 1.54–1.62 | 1.20 |
| final-copy | 1.62 | 1.54 | 1.58–1.63 | 1.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 9.4 | 9.3 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 9.4 | 9.3 | 0.1 | 0.1 |

## conflict_inplace_k1_reject
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.998 | [0.995, 1.001] | 2/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 2/4 |
| final-copy | cycles | 1.004 | [0.999, 1.009] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 626.63 | 621.14 | 623.65–628.87 | 2.15 |
| final-copy | 625.97 | 621.27 | 624.60–626.70 | 2.16 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 133.16 | 110.76 | 5328 | 83 | 20.7 | 14.4 | 6.3 | 6.3 |
| final-copy | 133.16 | 110.59 | 5328 | 83 | 20.7 | 14.4 | 6.3 | 6.3 |

## conflict_inplace_k1_skip
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.990, 1.003] | 1/4 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 2/4 |
| final-copy | cycles | 1.003 | [1.001, 1.005] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 380.75 | 378.08 | 379.44–384.50 | 2.05 |
| final-copy | 379.93 | 377.77 | 379.14–381.43 | 2.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 81.00 | 69.27 | 2929 | 56 | 15.4 | 14.4 | 1.1 | 1.1 |
| final-copy | 81.00 | 69.30 | 2929 | 56 | 15.4 | 14.4 | 1.1 | 1.1 |

## cycle_inplace_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.995 | [0.991, 0.998] | 0/4 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 1/4 |
| final-copy | cycles | 0.993 | [0.991, 0.996] | 0/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 439.90 | 436.91 | 438.69–443.35 | 4.13 |
| final-copy | 438.48 | 435.42 | 438.26–438.96 | 4.13 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 369.22 | 246.55 | 1257 | 82 | 18.2 | 14.0 | 4.2 | 4.2 |
| final-copy | 369.15 | 246.44 | 1257 | 82 | 18.2 | 14.0 | 4.2 | 4.2 |

## cycle_inplace_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.995, 1.004] | 3/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 0/4 |
| final-copy | cycles | 1.002 | [0.996, 1.008] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 278.32 | 275.39 | 276.62–280.70 | 7.33 |
| final-copy | 278.19 | 276.04 | 278.05–278.59 | 7.34 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 398.30 | 256.16 | 1283 | 82 | 27.5 | 14.1 | 13.4 | 13.4 |
| final-copy | 398.42 | 256.29 | 1283 | 82 | 27.5 | 14.1 | 13.4 | 13.4 |

## cycle_moving_dense_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.995, 0.998] | 0/4 |
| final-copy | instructions | 1.000 | [0.998, 1.001] | 1/4 |
| final-copy | cycles | 0.999 | [0.993, 1.004] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 512.88 | 506.36 | 510.54–513.78 | 3.12 |
| final-copy | 511.48 | 505.98 | 507.77–512.48 | 3.13 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 597.05 | 53.41 | 1492 | 48 | 649.5 | 649.3 | 0.2 | 0.2 |
| final-copy | 597.12 | 53.42 | 1502 | 48 | 649.5 | 649.3 | 0.2 | 0.2 |

## cycle_moving_dense_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.991, 1.003] | 3/4 |
| final-copy | instructions | 0.999 | [0.999, 1.000] | 0/4 |
| final-copy | cycles | 1.000 | [0.996, 1.005] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 394.41 | 389.75 | 392.90–399.96 | 3.94 |
| final-copy | 394.42 | 389.99 | 393.77–397.22 | 3.93 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 612.69 | 51.78 | 1758 | 48 | 8.3 | 8.0 | 0.2 | 0.2 |
| final-copy | 612.64 | 51.72 | 1760 | 48 | 8.3 | 8.0 | 0.2 | 0.2 |

## cycle_moving_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.003 | [0.994, 1.014] | 2/4 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/4 |
| final-copy | cycles | 1.004 | [1.000, 1.008] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 147.07 | 143.12 | 144.05–148.83 | 3.10 |
| final-copy | 147.02 | 144.14 | 146.45–148.30 | 3.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 133.01 | 0.08 | 1280 | 46 | 9.7 | 9.5 | 0.2 | 0.2 |
| final-copy | 133.01 | 0.08 | 1280 | 46 | 9.7 | 9.5 | 0.2 | 0.2 |

## cycle_moving_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.017 | [1.008, 1.028] | 4/4 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 3/4 |
| final-copy | cycles | 1.001 | [1.000, 1.004] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 184.11 | 178.77 | 179.26–185.39 | 2.95 |
| final-copy | 186.49 | 182.62 | 185.33–187.47 | 2.92 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 153.12 | 0.08 | 1445 | 46 | 8.1 | 7.9 | 0.2 | 0.2 |
| final-copy | 153.12 | 0.08 | 1445 | 46 | 8.1 | 7.9 | 0.2 | 0.2 |

## filter_null_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.965, 1.023] | 1/4 |
| final-copy | instructions | 1.001 | [0.999, 1.002] | 3/4 |
| final-copy | cycles | 1.017 | [1.003, 1.030] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 12.06 | 11.40 | 11.74–12.22 | 9.38 |
| final-copy | 11.94 | 11.36 | 11.53–12.17 | 9.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.14 | 0.00 | 40 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |
| final-copy | 55.14 | 0.00 | 40 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |

## filter_null_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.974, 1.017] | 3/4 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 3/4 |
| final-copy | cycles | 1.005 | [0.996, 1.014] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 21.68 | 20.56 | 21.36–22.20 | 4.53 |
| final-copy | 21.51 | 20.35 | 21.31–21.92 | 4.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.05 | 0.00 | 40 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |
| final-copy | 55.05 | 0.00 | 40 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |

## reopen_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [0.960, 1.074] | 1/4 |
| final-copy | instructions | 0.999 | [0.992, 1.009] | 1/4 |
| final-copy | cycles | 0.999 | [0.965, 1.057] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.22 | 0.20 | 0.21–0.22 | 1.06 |
| final-copy | 0.22 | 0.19 | 0.21–0.23 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 11.0 | 7.4 | 3.6 | 3.6 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 11.0 | 7.4 | 3.6 | 3.6 |

## reopen_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [0.989, 1.021] | 2/4 |
| final-copy | instructions | 1.001 | [0.999, 1.004] | 3/4 |
| final-copy | cycles | 0.986 | [0.957, 1.015] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.25 | 0.24 | 0.25–0.25 | 1.04 |
| final-copy | 0.25 | 0.24 | 0.25–0.25 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 24.6 | 16.8 | 7.9 | 7.9 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 24.6 | 16.8 | 7.9 | 7.9 |

## reopen_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.985 | [0.953, 1.025] | 1/4 |
| final-copy | instructions | 0.996 | [0.994, 0.998] | 0/4 |
| final-copy | cycles | 0.981 | [0.951, 1.018] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.23 | 0.19 | 0.22–0.23 | 1.08 |
| final-copy | 0.22 | 0.20 | 0.22–0.23 | 1.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 10.3 | 7.1 | 3.3 | 3.3 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 10.3 | 7.1 | 3.3 | 3.3 |

## reopen_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.965 | [0.955, 0.975] | 0/4 |
| final-copy | instructions | 0.998 | [0.995, 1.001] | 1/4 |
| final-copy | cycles | 0.972 | [0.940, 1.005] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.23 | 0.21 | 0.22–0.24 | 1.04 |
| final-copy | 0.22 | 0.21 | 0.21–0.23 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 15.7 | 9.2 | 6.5 | 6.5 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 15.7 | 9.2 | 6.5 | 6.5 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [0.981, 1.019] | 3/4 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 2/4 |
| final-copy | cycles | 1.011 | [0.997, 1.019] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 10.64 | 10.06 | 10.57–10.66 | 10.46 |
| final-copy | 10.75 | 10.16 | 10.24–10.87 | 10.57 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.14 | 0.00 | 40 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |
| final-copy | 55.14 | 0.00 | 40 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |

## scan_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.987 | [0.979, 0.996] | 0/4 |
| final-copy | instructions | 1.000 | [0.999, 1.001] | 2/4 |
| final-copy | cycles | 1.001 | [0.981, 1.021] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 9.66 | 9.10 | 9.52–9.75 | 9.92 |
| final-copy | 9.47 | 8.92 | 9.37–9.71 | 10.22 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.23 | 0.00 | 40 | 0 | 11.0 | 7.4 | 3.6 | 3.6 |
| final-copy | 55.23 | 0.00 | 40 | 0 | 11.0 | 7.4 | 3.6 | 3.6 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.979, 1.007] | 3/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 1/4 |
| final-copy | cycles | 1.000 | [0.993, 1.007] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 19.68 | 18.46 | 19.55–20.09 | 4.99 |
| final-copy | 19.67 | 18.81 | 19.49–19.82 | 4.91 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.05 | 0.00 | 40 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |
| final-copy | 55.05 | 0.00 | 40 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |

## scan_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.998 | [0.990, 1.011] | 1/4 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 4/4 |
| final-copy | cycles | 1.009 | [0.985, 1.021] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 9.45 | 8.82 | 9.39–9.55 | 10.15 |
| final-copy | 9.33 | 8.85 | 9.30–9.55 | 10.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 55.12 | 0.00 | 40 | 0 | 10.3 | 7.1 | 3.3 | 3.3 |
| final-copy | 55.12 | 0.00 | 40 | 0 | 10.3 | 7.1 | 3.3 | 3.3 |

## take_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.952, 1.034] | 2/4 |
| final-copy | instructions | 1.006 | [0.999, 1.012] | 3/4 |
| final-copy | cycles | 1.016 | [1.004, 1.032] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 8.98 | 8.59 | 8.76–9.34 | 11.43 |
| final-copy | 9.14 | 8.55 | 8.66–9.39 | 11.51 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.66 | 0.00 | 850 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |
| final-copy | 3.66 | 0.00 | 850 | 0 | 21.2 | 10.8 | 10.4 | 10.4 |

## take_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.985 | [0.964, 1.000] | 2/4 |
| final-copy | instructions | 1.004 | [0.999, 1.009] | 3/4 |
| final-copy | cycles | 1.003 | [0.990, 1.017] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 9.09 | 8.59 | 8.92–9.34 | 11.32 |
| final-copy | 8.93 | 8.32 | 8.82–9.04 | 11.52 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.66 | 0.00 | 850 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |
| final-copy | 3.66 | 0.00 | 850 | 0 | 336.0 | 166.8 | 169.2 | 169.2 |

## unrelated_moving_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.032 | [1.000, 1.070] | 3/4 |
| final-copy | instructions | 1.000 | [0.996, 1.005] | 2/4 |
| final-copy | cycles | 1.010 | [0.986, 1.032] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 43.15 | 39.74 | 39.84–44.07 | 4.19 |
| final-copy | 43.75 | 42.76 | 43.35–44.44 | 4.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.48 | 0.07 | 1145 | 42 | 17.9 | 9.4 | 8.5 | 8.5 |
| final-copy | 22.48 | 0.07 | 1145 | 42 | 17.9 | 9.4 | 8.5 | 8.5 |

## unrelated_moving_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.030 | [0.997, 1.073] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.003] | 3/4 |
| final-copy | cycles | 0.995 | [0.968, 1.021] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 42.88 | 39.49 | 39.65–44.33 | 4.17 |
| final-copy | 43.83 | 42.92 | 43.30–44.10 | 4.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.48 | 0.07 | 1145 | 42 | 14.9 | 7.8 | 7.1 | 7.1 |
| final-copy | 22.48 | 0.07 | 1145 | 42 | 14.9 | 7.8 | 7.1 | 7.1 |

## update_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.025 | [0.999, 1.073] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 3/4 |
| final-copy | cycles | 1.016 | [0.993, 1.039] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 43.09 | 39.28 | 39.41–44.03 | 4.13 |
| final-copy | 43.60 | 42.33 | 43.26–44.08 | 4.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.48 | 0.07 | 1136 | 41 | 17.7 | 9.4 | 8.3 | 8.3 |
| final-copy | 22.48 | 0.07 | 1136 | 41 | 17.7 | 9.4 | 8.3 | 8.3 |

## update_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.010 | [0.987, 1.042] | 3/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 2/4 |
| final-copy | cycles | 1.002 | [0.980, 1.027] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 74.85 | 71.62 | 71.88–76.52 | 2.87 |
| final-copy | 75.50 | 74.04 | 75.05–76.03 | 2.84 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.60 | 0.08 | 1256 | 41 | 25.5 | 17.2 | 8.3 | 8.3 |
| final-copy | 22.60 | 0.08 | 1256 | 41 | 25.5 | 17.2 | 8.3 | 8.3 |

## update_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.037 | [1.003, 1.075] | 4/4 |
| final-copy | instructions | 1.001 | [0.999, 1.005] | 2/4 |
| final-copy | cycles | 0.998 | [0.980, 1.010] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 42.86 | 39.17 | 39.49–43.78 | 4.17 |
| final-copy | 43.69 | 42.03 | 43.42–44.56 | 4.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.48 | 0.07 | 1136 | 41 | 14.7 | 7.8 | 6.9 | 6.9 |
| final-copy | 22.48 | 0.07 | 1136 | 41 | 14.7 | 7.8 | 6.9 | 6.9 |

## update_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.974, 1.040] | 1/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 2/4 |
| final-copy | cycles | 0.994 | [0.986, 1.002] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 76.09 | 70.93 | 71.94–77.02 | 2.87 |
| final-copy | 75.54 | 74.05 | 74.62–76.24 | 2.86 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.60 | 0.07 | 1256 | 41 | 16.2 | 9.3 | 6.9 | 6.9 |
| final-copy | 22.60 | 0.07 | 1256 | 41 | 16.2 | 9.3 | 6.9 | 6.9 |

