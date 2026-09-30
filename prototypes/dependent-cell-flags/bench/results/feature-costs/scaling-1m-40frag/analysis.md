## after_appends_k32_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.009 | [1.006, 1.013] | 4/4 |
| final-copy | instructions | 1.001 | [1.001, 1.002] | 4/4 |
| final-copy | cycles | 1.009 | [0.997, 1.019] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 133.74 | 132.98 | 133.24–134.20 | 1.60 |
| final-copy | 134.91 | 133.92 | 134.71–135.56 | 1.61 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 15.51 | 13.59 | 747 | 42 | 21.1 | 17.0 | 4.1 | 4.1 |
| final-copy | 15.51 | 13.60 | 747 | 42 | 21.1 | 17.0 | 4.1 | 4.1 |

## after_appends_k32_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.990, 1.003] | 2/4 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 4/4 |
| final-copy | cycles | 1.004 | [0.991, 1.015] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 45.77 | 45.12 | 45.59–46.00 | 7.26 |
| final-copy | 45.74 | 44.62 | 44.94–46.18 | 7.29 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 22.06 | 15.90 | 782 | 42 | 30.7 | 17.1 | 13.6 | 13.6 |
| final-copy | 22.06 | 15.89 | 782 | 42 | 30.7 | 17.1 | 13.6 | 13.6 |

## append_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.996 | [0.987, 1.004] | 2/4 |
| final-copy | instructions | 1.002 | [0.991, 1.007] | 3/4 |
| final-copy | cycles | 0.997 | [0.982, 1.011] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.60 | 1.49 | 1.58–1.64 | 1.16 |
| final-copy | 1.61 | 1.52 | 1.55–1.62 | 1.18 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 7.6 | 7.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 7.6 | 7.4 | 0.1 | 0.1 |

## append_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.999 | [0.969, 1.024] | 2/4 |
| final-copy | instructions | 1.001 | [0.989, 1.010] | 3/4 |
| final-copy | cycles | 1.000 | [0.947, 1.046] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.54 | 1.49 | 1.51–1.58 | 1.19 |
| final-copy | 1.56 | 1.46 | 1.51–1.58 | 1.19 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.03 | 1 | 4 | 16.6 | 16.4 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.03 | 1 | 4 | 16.6 | 16.4 | 0.1 | 0.1 |

## append_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.997 | [0.987, 1.006] | 2/4 |
| final-copy | instructions | 0.994 | [0.991, 0.999] | 1/4 |
| final-copy | cycles | 1.002 | [0.985, 1.018] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.57 | 1.45 | 1.55–1.61 | 1.17 |
| final-copy | 1.57 | 1.50 | 1.56–1.60 | 1.19 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 0.00 | 0.02 | 1 | 4 | 7.2 | 7.1 | 0.1 | 0.1 |
| final-copy | 0.00 | 0.02 | 1 | 4 | 7.2 | 7.1 | 0.1 | 0.1 |

## append_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.016 | [1.010, 1.025] | 4/4 |
| final-copy | instructions | 0.998 | [0.984, 1.006] | 3/4 |
| final-copy | cycles | 1.017 | [1.005, 1.028] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 1.60 | 1.49 | 1.57–1.62 | 1.19 |
| final-copy | 1.62 | 1.55 | 1.59–1.66 | 1.18 |

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
| final-copy | wall_ns | 1.008 | [1.001, 1.017] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 3/4 |
| final-copy | cycles | 1.018 | [1.000, 1.040] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 299.84 | 298.01 | 298.93–300.94 | 2.37 |
| final-copy | 301.51 | 298.50 | 299.63–307.28 | 2.39 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 46.84 | 27.88 | 3606 | 83 | 20.3 | 14.3 | 6.0 | 6.0 |
| final-copy | 46.84 | 27.91 | 3606 | 83 | 20.3 | 14.3 | 6.0 | 6.0 |

## conflict_inplace_k1_skip
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [1.002, 1.008] | 4/4 |
| final-copy | instructions | 1.001 | [1.001, 1.003] | 4/4 |
| final-copy | cycles | 1.021 | [1.005, 1.038] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 185.37 | 184.31 | 184.98–186.76 | 2.21 |
| final-copy | 186.54 | 184.98 | 185.65–187.03 | 2.23 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 27.30 | 17.45 | 2068 | 56 | 15.3 | 14.3 | 1.1 | 1.1 |
| final-copy | 27.31 | 17.46 | 2068 | 56 | 15.3 | 14.3 | 1.1 | 1.1 |

## cycle_inplace_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.998, 1.010] | 2/4 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 2/4 |
| final-copy | cycles | 1.009 | [0.998, 1.021] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 194.74 | 192.65 | 193.89–195.09 | 3.58 |
| final-copy | 195.17 | 193.65 | 194.72–197.04 | 3.58 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 95.16 | 63.28 | 1232 | 84 | 18.2 | 14.1 | 4.1 | 4.1 |
| final-copy | 95.17 | 63.28 | 1232 | 84 | 18.2 | 14.1 | 4.1 | 4.1 |

## cycle_inplace_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.004 | [0.998, 1.011] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 4/4 |
| final-copy | cycles | 1.012 | [1.007, 1.019] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 105.96 | 104.51 | 105.06–106.64 | 7.63 |
| final-copy | 106.64 | 104.60 | 105.80–107.24 | 7.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 101.58 | 65.57 | 1267 | 84 | 27.8 | 14.1 | 13.6 | 13.6 |
| final-copy | 101.61 | 65.58 | 1267 | 84 | 27.8 | 14.1 | 13.6 | 13.6 |

## cycle_moving_dense_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.005 | [1.000, 1.010] | 2/4 |
| final-copy | instructions | 1.000 | [1.000, 1.000] | 1/4 |
| final-copy | cycles | 1.003 | [0.992, 1.011] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 133.52 | 131.97 | 132.99–134.17 | 3.72 |
| final-copy | 134.23 | 132.20 | 133.65–135.07 | 3.74 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 149.70 | 13.83 | 989 | 48 | 329.0 | 328.8 | 0.2 | 0.2 |
| final-copy | 149.70 | 13.83 | 979 | 48 | 329.0 | 328.8 | 0.2 | 0.2 |

## cycle_moving_dense_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.006 | [1.001, 1.010] | 3/4 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 2/4 |
| final-copy | cycles | 1.006 | [1.000, 1.012] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 120.49 | 118.86 | 119.82–120.96 | 4.33 |
| final-copy | 121.02 | 119.65 | 120.04–121.96 | 4.32 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 153.03 | 13.00 | 1189 | 48 | 8.1 | 7.9 | 0.2 | 0.2 |
| final-copy | 153.02 | 12.99 | 1195 | 48 | 8.1 | 7.9 | 0.2 | 0.2 |

## cycle_moving_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.009 | [0.994, 1.024] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 3/4 |
| final-copy | cycles | 1.021 | [1.010, 1.030] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 55.47 | 54.45 | 54.96–56.05 | 4.15 |
| final-copy | 55.97 | 54.47 | 55.19–56.46 | 4.17 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 35.05 | 0.08 | 1246 | 47 | 9.3 | 9.1 | 0.2 | 0.2 |
| final-copy | 35.05 | 0.08 | 1246 | 47 | 9.3 | 9.1 | 0.2 | 0.2 |

## cycle_moving_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.990 | [0.970, 1.005] | 2/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 4/4 |
| final-copy | cycles | 1.004 | [0.990, 1.021] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 67.87 | 66.79 | 67.63–68.36 | 3.95 |
| final-copy | 67.59 | 65.45 | 65.63–68.44 | 4.00 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 39.66 | 0.08 | 1411 | 47 | 8.0 | 7.8 | 0.2 | 0.2 |
| final-copy | 39.66 | 0.08 | 1411 | 47 | 8.0 | 7.8 | 0.2 | 0.2 |

## filter_null_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.008 | [0.981, 1.028] | 3/4 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 3/4 |
| final-copy | cycles | 0.999 | [0.973, 1.025] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 4.35 | 4.17 | 4.29–4.38 | 7.90 |
| final-copy | 4.38 | 4.21 | 4.23–4.43 | 7.83 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.85 | 0.00 | 40 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |
| final-copy | 13.85 | 0.00 | 40 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |

## filter_null_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.000 | [0.990, 1.014] | 1/4 |
| final-copy | instructions | 1.001 | [1.000, 1.001] | 4/4 |
| final-copy | cycles | 1.010 | [0.987, 1.034] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 6.81 | 6.39 | 6.64–6.87 | 4.66 |
| final-copy | 6.82 | 6.28 | 6.62–6.92 | 4.68 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.85 | 0.00 | 40 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |
| final-copy | 13.85 | 0.00 | 40 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |

## reopen_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.019 | [0.994, 1.050] | 3/4 |
| final-copy | instructions | 1.009 | [1.004, 1.014] | 4/4 |
| final-copy | cycles | 1.028 | [1.008, 1.048] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.19 | 0.18 | 0.19–0.20 | 1.08 |
| final-copy | 0.19 | 0.18 | 0.19–0.20 | 1.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 10.9 | 7.3 | 3.6 | 3.6 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 10.9 | 7.3 | 3.6 | 3.6 |

## reopen_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.960, 1.022] | 2/4 |
| final-copy | instructions | 1.006 | [1.000, 1.014] | 3/4 |
| final-copy | cycles | 0.999 | [0.991, 1.015] | 1/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.24 | 0.22 | 0.23–0.25 | 1.04 |
| final-copy | 0.23 | 0.22 | 0.23–0.24 | 1.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 23.8 | 16.3 | 7.4 | 7.4 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 23.8 | 16.3 | 7.4 | 7.4 |

## reopen_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.982, 1.021] | 2/4 |
| final-copy | instructions | 1.006 | [1.000, 1.013] | 3/4 |
| final-copy | cycles | 1.039 | [1.018, 1.059] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.21 | 0.17 | 0.20–0.21 | 1.06 |
| final-copy | 0.20 | 0.17 | 0.20–0.22 | 1.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 10.2 | 7.0 | 3.2 | 3.2 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 10.2 | 7.0 | 3.2 | 3.2 |

## reopen_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.991 | [0.976, 1.005] | 1/4 |
| final-copy | instructions | 1.006 | [0.998, 1.013] | 3/4 |
| final-copy | cycles | 1.019 | [0.982, 1.054] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 0.20 | 0.19 | 0.20–0.22 | 1.04 |
| final-copy | 0.21 | 0.19 | 0.20–0.21 | 1.06 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | unavailable | unavailable | unavailable | unavailable | 15.5 | 9.2 | 6.3 | 6.3 |
| final-copy | unavailable | unavailable | unavailable | unavailable | 15.5 | 9.2 | 6.3 | 6.3 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.013 | [0.997, 1.029] | 3/4 |
| final-copy | instructions | 1.000 | [1.000, 1.001] | 3/4 |
| final-copy | cycles | 0.999 | [0.972, 1.029] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 3.75 | 3.56 | 3.68–3.80 | 8.75 |
| final-copy | 3.78 | 3.65 | 3.76–3.81 | 8.56 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.85 | 0.00 | 40 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |
| final-copy | 13.85 | 0.00 | 40 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |

## scan_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.014 | [1.004, 1.029] | 4/4 |
| final-copy | instructions | 1.001 | [1.000, 1.002] | 2/4 |
| final-copy | cycles | 1.014 | [0.983, 1.033] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 3.34 | 3.25 | 3.31–3.39 | 9.07 |
| final-copy | 3.36 | 3.20 | 3.32–3.47 | 9.14 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.85 | 0.00 | 40 | 0 | 10.9 | 7.3 | 3.6 | 3.6 |
| final-copy | 13.85 | 0.00 | 40 | 0 | 10.9 | 7.3 | 3.6 | 3.6 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.007 | [0.997, 1.020] | 3/4 |
| final-copy | instructions | 1.000 | [0.999, 1.000] | 1/4 |
| final-copy | cycles | 1.005 | [0.979, 1.031] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 6.15 | 5.98 | 6.11–6.16 | 5.02 |
| final-copy | 6.20 | 5.79 | 6.10–6.29 | 5.03 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.85 | 0.00 | 40 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |
| final-copy | 13.85 | 0.00 | 40 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |

## scan_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.007 | [0.976, 1.041] | 2/4 |
| final-copy | instructions | 1.003 | [1.002, 1.004] | 4/4 |
| final-copy | cycles | 1.015 | [0.991, 1.036] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 3.29 | 3.16 | 3.24–3.41 | 9.17 |
| final-copy | 3.33 | 3.18 | 3.20–3.48 | 9.10 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 13.75 | 0.00 | 40 | 0 | 10.2 | 7.0 | 3.2 | 3.2 |
| final-copy | 13.75 | 0.00 | 40 | 0 | 10.2 | 7.0 | 3.2 | 3.2 |

## take_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.014 | [0.984, 1.045] | 2/4 |
| final-copy | instructions | 1.013 | [1.008, 1.019] | 4/4 |
| final-copy | cycles | 1.024 | [0.987, 1.062] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 7.02 | 6.49 | 6.88–7.17 | 9.01 |
| final-copy | 7.18 | 6.70 | 6.91–7.35 | 9.21 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.76 | 0.00 | 520 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |
| final-copy | 3.76 | 0.00 | 520 | 0 | 21.0 | 10.7 | 10.3 | 10.3 |

## take_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.023 | [1.000, 1.054] | 3/4 |
| final-copy | instructions | 1.007 | [0.999, 1.020] | 3/4 |
| final-copy | cycles | 1.031 | [0.989, 1.082] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 6.82 | 6.54 | 6.70–6.91 | 9.38 |
| final-copy | 6.93 | 6.64 | 6.85–7.18 | 9.43 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 3.75 | 0.00 | 520 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |
| final-copy | 3.75 | 0.00 | 520 | 0 | 101.5 | 50.3 | 51.3 | 51.3 |

## unrelated_moving_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.993 | [0.981, 1.006] | 2/4 |
| final-copy | instructions | 1.003 | [0.999, 1.010] | 2/4 |
| final-copy | cycles | 0.998 | [0.982, 1.015] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 23.81 | 23.39 | 23.67–24.03 | 6.24 |
| final-copy | 23.69 | 22.83 | 23.05–23.98 | 6.30 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.28 | 0.07 | 1113 | 43 | 17.5 | 9.0 | 8.5 | 8.4 |
| final-copy | 7.28 | 0.07 | 1113 | 43 | 17.5 | 9.0 | 8.5 | 8.4 |

## unrelated_moving_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.998 | [0.975, 1.017] | 2/4 |
| final-copy | instructions | 1.008 | [1.006, 1.011] | 4/4 |
| final-copy | cycles | 1.016 | [1.003, 1.030] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 23.60 | 23.07 | 23.40–23.78 | 6.26 |
| final-copy | 23.75 | 22.88 | 22.92–23.97 | 6.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.28 | 0.07 | 1113 | 43 | 14.9 | 7.7 | 7.2 | 7.2 |
| final-copy | 7.28 | 0.07 | 1113 | 43 | 14.9 | 7.7 | 7.2 | 7.2 |

## update_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.963, 1.011] | 2/4 |
| final-copy | instructions | 1.005 | [1.003, 1.007] | 4/4 |
| final-copy | cycles | 1.023 | [1.008, 1.032] | 3/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 23.00 | 22.40 | 22.90–23.16 | 6.03 |
| final-copy | 23.07 | 21.64 | 21.77–23.48 | 6.20 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.25 | 0.07 | 1076 | 39 | 16.5 | 8.9 | 7.6 | 7.6 |
| final-copy | 7.25 | 0.07 | 1076 | 39 | 16.5 | 8.9 | 7.6 | 7.6 |

## update_one_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 1.001 | [0.993, 1.007] | 3/4 |
| final-copy | instructions | 1.003 | [1.001, 1.006] | 3/4 |
| final-copy | cycles | 1.020 | [1.007, 1.036] | 4/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 33.01 | 32.36 | 32.83–33.64 | 4.78 |
| final-copy | 33.17 | 32.13 | 32.50–33.93 | 4.88 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.37 | 0.08 | 1199 | 40 | 24.6 | 16.8 | 7.8 | 7.8 |
| final-copy | 7.37 | 0.08 | 1199 | 40 | 24.6 | 16.8 | 7.8 | 7.8 |

## update_plain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.989 | [0.961, 1.006] | 2/4 |
| final-copy | instructions | 1.006 | [1.002, 1.009] | 3/4 |
| final-copy | cycles | 0.998 | [0.982, 1.014] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 22.98 | 22.39 | 22.74–23.20 | 6.09 |
| final-copy | 22.94 | 21.74 | 21.76–23.11 | 6.11 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.24 | 0.06 | 1076 | 39 | 14.1 | 7.6 | 6.5 | 6.5 |
| final-copy | 7.24 | 0.06 | 1076 | 39 | 14.1 | 7.6 | 6.5 | 6.5 |

## update_plain_h16
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| final-copy | wall_ns | 0.992 | [0.982, 1.006] | 1/4 |
| final-copy | instructions | 1.002 | [1.000, 1.006] | 3/4 |
| final-copy | cycles | 0.996 | [0.979, 1.014] | 2/4 |
| final-copy | allocations | unavailable | — | — |
| final-copy | allocated_bytes | unavailable | — | — |
| final-copy | peak_live_growth_bytes | unavailable | — | — |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| final | 33.36 | 32.94 | 33.21–33.50 | 4.85 |
| final-copy | 33.12 | 32.24 | 32.54–33.67 | 4.87 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| final | unavailable | unavailable | unavailable |
| final-copy | unavailable | unavailable | unavailable |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| final | 7.37 | 0.07 | 1199 | 40 | 15.9 | 9.3 | 6.6 | 6.6 |
| final-copy | 7.37 | 0.07 | 1199 | 40 | 15.9 | 9.3 | 6.6 | 6.6 |

