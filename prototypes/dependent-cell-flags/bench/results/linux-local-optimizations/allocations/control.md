## backfill_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.997 | [0.930, 1.096] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 260.00 | 237.71 | 255.22–264.23 | 4.11 |
| after-copy | 256.89 | 235.41 | 239.14–279.71 | 4.37 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 622058 | 1601.0 | 236.2 |
| after-copy | 622058 | 1600.7 | 236.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 93.78 | 27.51 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |
| after-copy | 93.79 | 27.52 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |

## backfill_chain.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.083 | [1.026, 1.160] | 3/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 1.06 | 0.91 | 0.95–1.09 | 1.02 |
| after-copy | 1.12 | 0.97 | 1.02–1.17 | 1.01 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 1474 | 5.2 | 5.0 |
| after-copy | 1474 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.01 | 1 | 3 |
| after-copy | 0.00 | 0.01 | 1 | 3 |

## backfill_chain.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.002 | [0.944, 1.048] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.008 | [0.999, 1.026] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 65.54 | 52.44 | 63.56–66.02 | 8.75 |
| after-copy | 67.07 | 59.65 | 59.98–68.59 | 9.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 343023 | 804.5 | 170.9 |
| after-copy | 343023 | 804.5 | 172.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 66.26 | 0.00 | 90 | 0 |
| after-copy | 66.26 | 0.00 | 90 | 0 |

## backfill_chain.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.993 | [0.956, 1.052] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [0.997, 1.002] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.004 | [0.918, 1.121] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 27.61 | 24.04 | 26.89–29.18 | 7.82 |
| after-copy | 28.28 | 25.32 | 26.16–28.45 | 8.80 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 137617 | 301.0 | 38.3 |
| after-copy | 137617 | 301.1 | 39.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 27.52 | 0.00 | 50 | 0 |
| after-copy | 27.53 | 0.00 | 50 | 0 |

## backfill_chain.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.991 | [0.926, 1.110] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.001 | [1.000, 1.004] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.002 | [0.995, 1.010] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 165.36 | 159.52 | 163.23–170.63 | 1.68 |
| after-copy | 160.38 | 148.29 | 153.04–181.23 | 1.71 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 139945 | 490.5 | 18.5 |
| after-copy | 139944 | 490.5 | 18.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 27.50 | 0 | 10 |
| after-copy | 0.00 | 27.52 | 0 | 10 |

## backfill_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.004 | [0.940, 1.094] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 0.998 | [0.997, 0.999] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 220.48 | 194.99 | 203.74–228.63 | 3.77 |
| after-copy | 214.93 | 204.14 | 204.41–222.98 | 3.93 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 504377 | 1218.2 | 201.6 |
| after-copy | 504376 | 1216.6 | 201.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 80.19 | 13.86 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |
| after-copy | 80.09 | 13.76 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |

## backfill_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.961 | [0.845, 1.162] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 1.03 | 0.93 | 0.96–1.13 | 1.02 |
| after-copy | 0.96 | 0.89 | 0.90–1.11 | 1.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 1167 | 5.2 | 5.0 |
| after-copy | 1167 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 1 | 3 |
| after-copy | 0.00 | 0.00 | 1 | 3 |

## backfill_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.998 | [0.950, 1.061] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 0.998 | [0.994, 1.004] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 64.08 | 49.22 | 60.75–71.32 | 8.71 |
| after-copy | 67.72 | 59.31 | 60.00–68.32 | 9.28 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 343206 | 806.1 | 173.4 |
| after-copy | 343207 | 806.1 | 173.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 66.32 | 0.00 | 90 | 0 |
| after-copy | 66.32 | 0.00 | 90 | 0 |

## backfill_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.993 | [0.905, 1.119] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 0.991 | [0.985, 0.995] | 0/3 |
| after-copy | peak_live_growth_bytes | 0.990 | [0.918, 1.078] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 15.06 | 11.65 | 13.98–15.75 | 6.72 |
| after-copy | 15.22 | 13.42 | 13.63–15.64 | 7.76 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 72915 | 151.7 | 19.9 |
| after-copy | 72915 | 150.5 | 19.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.87 | 0.00 | 30 | 0 |
| after-copy | 13.77 | 0.00 | 30 | 0 |

## backfill_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.999 | [0.906, 1.110] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 0.998 | [0.996, 0.999] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 139.60 | 120.36 | 124.33–145.03 | 0.99 |
| after-copy | 131.37 | 129.74 | 130.10–138.04 | 0.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 87089 | 255.3 | 10.3 |
| after-copy | 87089 | 254.8 | 10.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 13.86 | 0 | 10 |
| after-copy | 0.00 | 13.76 | 0 | 10 |

## backfill_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.987 | [0.932, 1.023] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | allocated_bytes | 1.000 | [0.999, 1.001] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [0.999, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 246.20 | 241.84 | 245.48–246.87 | 4.44 |
| after-copy | 247.26 | 228.85 | 229.18–252.51 | 4.48 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 580586 | 1582.1 | 236.4 |
| after-copy | 580587 | 1581.9 | 236.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 93.91 | 27.58 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |
| after-copy | 93.85 | 27.53 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |

## backfill_plain_chain.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.035 | [1.002, 1.075] | 3/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 0.96 | 0.81 | 0.92–0.97 | 1.02 |
| after-copy | 0.96 | 0.89 | 0.94–1.04 | 1.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 924 | 5.2 | 5.0 |
| after-copy | 924 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 1 | 3 |
| after-copy | 0.00 | 0.00 | 1 | 3 |

## backfill_plain_chain.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.948 | [0.901, 0.990] | 0/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.002 | [0.997, 1.006] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 66.12 | 64.02 | 65.66–66.89 | 9.56 |
| after-copy | 63.79 | 56.40 | 59.16–65.47 | 9.08 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 343124 | 805.5 | 172.7 |
| after-copy | 343124 | 805.5 | 173.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 66.32 | 0.00 | 90 | 0 |
| after-copy | 66.32 | 0.00 | 90 | 0 |

## backfill_plain_chain.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.949 | [0.882, 1.010] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [0.996, 1.003] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.056 | [0.965, 1.118] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 28.57 | 22.92 | 27.88–28.84 | 8.15 |
| after-copy | 27.06 | 25.20 | 25.44–28.16 | 8.67 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 137617 | 301.9 | 36.7 |
| after-copy | 137617 | 301.2 | 39.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 27.59 | 0.00 | 50 | 0 |
| after-copy | 27.54 | 0.00 | 50 | 0 |

## backfill_plain_chain.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.000 | [0.950, 1.029] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | allocated_bytes | 1.004 | [0.999, 1.014] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.002 | [0.990, 1.010] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 151.93 | 146.54 | 150.86–155.00 | 1.73 |
| after-copy | 154.51 | 143.28 | 143.35–158.47 | 1.75 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 98921 | 469.6 | 18.5 |
| after-copy | 98923 | 469.4 | 18.6 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 27.58 | 0 | 10 |
| after-copy | 0.00 | 27.52 | 0 | 10 |

## backfill_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.985 | [0.922, 1.076] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [0.997, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 202.64 | 192.28 | 199.02–221.13 | 4.07 |
| after-copy | 203.98 | 189.58 | 195.01–214.23 | 4.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 468635 | 1201.6 | 201.4 |
| after-copy | 468634 | 1201.5 | 201.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 80.08 | 13.75 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |
| after-copy | 80.08 | 13.75 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |

## backfill_plain_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.945 | [0.893, 1.051] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 0.94 | 0.84 | 0.93–1.03 | 1.03 |
| after-copy | 0.93 | 0.83 | 0.83–0.99 | 1.03 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 902 | 5.2 | 5.0 |
| after-copy | 902 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 1 | 3 |
| after-copy | 0.00 | 0.00 | 1 | 3 |

## backfill_plain_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.015 | [0.945, 1.092] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.004 | [0.987, 1.023] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 64.57 | 50.73 | 63.92–69.89 | 9.33 |
| after-copy | 69.32 | 59.32 | 61.05–70.80 | 9.15 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 343125 | 805.5 | 172.5 |
| after-copy | 343125 | 805.5 | 171.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 66.32 | 0.00 | 90 | 0 |
| after-copy | 66.32 | 0.00 | 90 | 0 |

## backfill_plain_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.983 | [0.898, 1.072] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 0.998 | [0.983, 1.008] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.989 | [0.963, 1.012] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 14.90 | 13.08 | 13.71–15.98 | 7.16 |
| after-copy | 14.35 | 13.30 | 13.53–15.98 | 7.59 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 72915 | 150.4 | 20.2 |
| after-copy | 72915 | 150.4 | 20.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.77 | 0.00 | 30 | 0 |
| after-copy | 13.76 | 0.00 | 30 | 0 |

## backfill_plain_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.989 | [0.885, 1.093] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 0.999 | [0.996, 1.002] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 122.34 | 115.30 | 115.73–134.23 | 0.99 |
| after-copy | 123.26 | 113.81 | 118.82–126.48 | 0.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 51693 | 240.4 | 10.3 |
| after-copy | 51693 | 240.4 | 10.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 13.75 | 0 | 10 |
| after-copy | 0.00 | 13.75 | 0 | 10 |

## cycle_inplace_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.992 | [0.938, 1.069] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.002] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 383.65 | 352.62 | 366.98–394.83 | 3.66 |
| after-copy | 383.67 | 350.17 | 354.39–392.45 | 3.82 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 839411 | 2103.2 | 224.5 |
| after-copy | 839411 | 2103.8 | 223.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 96.26 | 64.59 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |
| after-copy | 96.31 | 64.65 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |

## cycle_inplace_sparse_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.993 | [0.818, 1.243] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 1.07 | 0.95 | 1.03–1.34 | 1.03 |
| after-copy | 1.04 | 0.90 | 0.99–1.28 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 1492 | 5.2 | 5.0 |
| after-copy | 1492 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.01 | 1 | 3 |
| after-copy | 0.00 | 0.01 | 1 | 3 |

## cycle_inplace_sparse_one.find
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.024 | [0.991, 1.052] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 0.983 | [0.920, 1.057] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 13.16 | 12.95 | 13.01–13.66 | 1.07 |
| after-copy | 13.54 | 12.90 | 13.40–13.76 | 1.05 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 5691 | 8.3 | 0.1 |
| after-copy | 5691 | 8.3 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 0 | 0 |
| after-copy | 0.00 | 0.00 | 0 | 0 |

## cycle_inplace_sparse_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.914 | [0.773, 1.211] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after-copy | peak_live_growth_bytes | 0.999 | [0.999, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 5.67 | 4.80 | 4.96–6.19 | 5.25 |
| after-copy | 4.80 | 4.26 | 4.78–6.01 | 5.09 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 19361 | 11.4 | 1.8 |
| after-copy | 19360 | 11.4 | 1.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 1.21 | 0.00 | 373 | 0 |
| after-copy | 1.21 | 0.00 | 373 | 0 |

## cycle_inplace_sparse_one.read_pending
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.964 | [0.870, 1.150] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.996 | [0.941, 1.078] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 17.81 | 16.19 | 17.07–19.34 | 5.26 |
| after-copy | 16.83 | 15.16 | 15.30–20.48 | 5.54 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 74704 | 162.0 | 22.8 |
| after-copy | 74704 | 162.0 | 23.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 14.08 | 0.00 | 40 | 0 |
| after-copy | 14.08 | 0.00 | 40 | 0 |

## cycle_inplace_sparse_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.014 | [0.971, 1.094] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.003 | [0.998, 1.008] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.968 | [0.921, 1.004] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 14.02 | 11.02 | 13.09–14.88 | 5.93 |
| after-copy | 13.68 | 11.42 | 12.85–15.24 | 7.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 73154 | 151.2 | 20.0 |
| after-copy | 73153 | 151.7 | 19.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.82 | 0.00 | 30 | 0 |
| after-copy | 13.87 | 0.00 | 30 | 0 |

## cycle_inplace_sparse_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.983 | [0.927, 1.094] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 160.47 | 150.76 | 151.82–170.39 | 1.31 |
| after-copy | 158.02 | 146.65 | 150.28–166.15 | 1.37 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 150874 | 436.1 | 12.9 |
| after-copy | 150873 | 436.3 | 12.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.99 | 13.81 | 10 | 10 |
| after-copy | 13.99 | 13.85 | 10 | 10 |

## cycle_inplace_sparse_one.update
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.997 | [0.928, 1.049] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [0.998, 1.002] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 170.78 | 146.25 | 165.53–173.55 | 5.83 |
| after-copy | 170.18 | 153.07 | 161.08–173.70 | 6.03 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 514135 | 1328.6 | 224.5 |
| after-copy | 514135 | 1328.7 | 223.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 53.15 | 50.79 | 91 | 13 |
| after-copy | 53.15 | 50.82 | 91 | 13 |

## cycle_inplace_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.022 | [1.010, 1.044] | 3/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.997 | [0.992, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 297.16 | 238.24 | 288.48–304.52 | 5.98 |
| after-copy | 301.32 | 289.25 | 292.20–317.79 | 5.76 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 1038770 | 2222.7 | 225.3 |
| after-copy | 1038773 | 2224.0 | 225.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 102.09 | 67.02 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |
| after-copy | 102.18 | 67.04 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |

## cycle_inplace_sparse_plain_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.979 | [0.946, 1.035] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 0.93 | 0.77 | 0.91–1.00 | 1.04 |
| after-copy | 0.91 | 0.83 | 0.88–0.95 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 917 | 5.2 | 5.1 |
| after-copy | 917 | 5.2 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.01 | 1 | 3 |
| after-copy | 0.00 | 0.01 | 1 | 3 |

## cycle_inplace_sparse_plain_one.locate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.968 | [0.861, 1.086] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 3/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.978 | [0.960, 1.007] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 16.50 | 13.71 | 16.07–17.46 | 3.75 |
| after-copy | 15.94 | 13.24 | 15.03–17.45 | 3.79 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 53319 | 39.5 | 4.3 |
| after-copy | 53322 | 39.5 | 4.2 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 3.25 | 0.00 | 301 | 0 |
| after-copy | 3.25 | 0.00 | 301 | 0 |

## cycle_inplace_sparse_plain_one.merge
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.028 | [0.987, 1.082] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.001 | [1.000, 1.001] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.988 | [0.965, 1.005] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 83.53 | 64.56 | 81.55–89.29 | 6.40 |
| after-copy | 88.01 | 81.51 | 82.93–90.41 | 5.93 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 323563 | 548.6 | 103.0 |
| after-copy | 323563 | 549.1 | 101.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 18.17 | 16.13 | 30 | 10 |
| after-copy | 18.17 | 16.22 | 30 | 10 |

## cycle_inplace_sparse_plain_one.read_pending
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.019 | [0.928, 1.099] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 0.996 | [0.948, 1.035] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 14.42 | 10.57 | 13.46–14.44 | 7.09 |
| after-copy | 14.97 | 12.13 | 12.48–15.86 | 5.85 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 73926 | 149.6 | 18.9 |
| after-copy | 73926 | 149.6 | 19.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.69 | 0.00 | 40 | 0 |
| after-copy | 13.69 | 0.00 | 40 | 0 |

## cycle_inplace_sparse_plain_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.016 | [0.954, 1.080] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.006 | [0.998, 1.012] | 2/3 |
| after-copy | peak_live_growth_bytes | 0.975 | [0.917, 1.026] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 13.73 | 10.47 | 13.56–14.40 | 8.38 |
| after-copy | 13.82 | 12.54 | 13.74–15.11 | 7.79 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 73443 | 151.6 | 20.0 |
| after-copy | 73444 | 152.7 | 18.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.86 | 0.00 | 30 | 0 |
| after-copy | 13.95 | 0.00 | 30 | 0 |

## cycle_inplace_sparse_plain_one.update
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.019 | [0.980, 1.060] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 0.997 | [0.992, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 168.30 | 137.94 | 162.62–170.13 | 5.91 |
| after-copy | 170.53 | 155.42 | 165.83–178.57 | 5.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 513600 | 1328.2 | 225.3 |
| after-copy | 513601 | 1328.1 | 225.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 53.13 | 50.79 | 91 | 13 |
| after-copy | 53.13 | 50.81 | 91 | 13 |

## scan_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.937 | [0.819, 1.053] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 1.27 | 0.99 | 1.17–1.42 | 1.66 |
| after-copy | 1.21 | 0.99 | 1.16–1.23 | 2.15 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 7673 | 4.7 | 0.8 |
| after-copy | 7673 | 4.7 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 0.00 | 0.00 | 0 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |
| after-copy | 0.00 | 0.00 | 0 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |

## scan_masked.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.937 | [0.819, 1.053] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 1.27 | 0.99 | 1.17–1.42 | 1.66 |
| after-copy | 1.21 | 0.99 | 1.16–1.23 | 2.15 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 7673 | 4.7 | 0.8 |
| after-copy | 7673 | 4.7 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 0 | 0 |
| after-copy | 0.00 | 0.00 | 0 | 0 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.002 | [0.936, 1.086] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.055 | [1.035, 1.081] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 16.76 | 14.82 | 15.79–17.39 | 7.65 |
| after-copy | 16.58 | 12.82 | 16.28–17.14 | 8.12 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 89051 | 156.6 | 18.8 |
| after-copy | 89051 | 156.6 | 19.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 13.79 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| after-copy | 13.79 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_1pct.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.002 | [0.936, 1.086] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after-copy | peak_live_growth_bytes | 1.055 | [1.035, 1.081] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 16.76 | 14.82 | 15.79–17.39 | 7.65 |
| after-copy | 16.58 | 12.82 | 16.28–17.14 | 8.12 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 89051 | 156.6 | 18.8 |
| after-copy | 89051 | 156.6 | 19.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.79 | 0.00 | 10 | 0 |
| after-copy | 13.79 | 0.00 | 10 | 0 |

## scan_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.900 | [0.855, 0.951] | 0/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.035 | [1.010, 1.062] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 2.45 | 1.93 | 2.32–2.57 | 4.38 |
| after-copy | 2.19 | 1.63 | 2.19–2.21 | 4.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 11975 | 7.1 | 0.8 |
| after-copy | 11976 | 7.1 | 0.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| after-copy | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_all.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.900 | [0.855, 0.951] | 0/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.036 | [1.010, 1.062] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 2.45 | 1.93 | 2.32–2.57 | 4.38 |
| after-copy | 2.19 | 1.63 | 2.19–2.21 | 4.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 11975 | 7.1 | 0.8 |
| after-copy | 11976 | 7.1 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 0.00 | 0.00 | 0 | 0 |
| after-copy | 0.00 | 0.00 | 0 | 0 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.976 | [0.930, 1.014] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.003 | [0.979, 1.038] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 21.14 | 18.08 | 20.00–21.29 | 5.07 |
| after-copy | 19.97 | 18.76 | 19.79–20.87 | 5.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 73101 | 158.0 | 24.0 |
| after-copy | 73101 | 158.0 | 23.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 13.71 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |
| after-copy | 13.71 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |

## scan_partial_1pct.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.976 | [0.930, 1.014] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 1.003 | [0.979, 1.038] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 21.14 | 18.08 | 20.00–21.29 | 5.07 |
| after-copy | 19.97 | 18.76 | 19.79–20.87 | 5.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 73101 | 158.0 | 24.0 |
| after-copy | 73101 | 158.0 | 23.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.71 | 0.00 | 10 | 0 |
| after-copy | 13.71 | 0.00 | 10 | 0 |

## scan_plain_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.008 | [0.958, 1.039] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 0.977 | [0.969, 0.989] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 25.45 | 23.93 | 25.21–26.24 | 4.49 |
| after-copy | 25.87 | 23.97 | 24.37–27.03 | 4.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 76635 | 246.0 | 32.2 |
| after-copy | 76634 | 246.0 | 31.9 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 13.96 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |
| after-copy | 13.96 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |

## scan_plain_moved.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 1.008 | [0.958, 1.039] | 2/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 1/3 |
| after-copy | peak_live_growth_bytes | 0.977 | [0.969, 0.989] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 25.45 | 23.93 | 25.21–26.24 | 4.49 |
| after-copy | 25.87 | 23.97 | 24.37–27.03 | 4.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 76635 | 246.0 | 32.2 |
| after-copy | 76634 | 246.0 | 31.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.96 | 0.00 | 11 | 0 |
| after-copy | 13.96 | 0.00 | 11 | 0 |

## scan_ready_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.985 | [0.941, 1.040] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.046 | [1.028, 1.066] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 25.07 | 22.99 | 24.72–25.09 | 4.22 |
| after-copy | 25.15 | 22.86 | 23.61–26.08 | 4.77 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 76690 | 244.3 | 31.3 |
| after-copy | 76689 | 244.3 | 32.7 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| after | 13.86 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |
| after-copy | 13.86 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |

## scan_ready_moved.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after-copy | wall_ns | 0.985 | [0.941, 1.040] | 1/3 |
| after-copy | instructions | unavailable | — | — |
| after-copy | cycles | unavailable | — | — |
| after-copy | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after-copy | peak_live_growth_bytes | 1.046 | [1.028, 1.066] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| after | 25.07 | 22.99 | 24.72–25.09 | 4.22 |
| after-copy | 25.15 | 22.86 | 23.61–26.08 | 4.77 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| after | 76690 | 244.3 | 31.2 |
| after-copy | 76689 | 244.3 | 32.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| after | 13.86 | 0.00 | 11 | 0 |
| after-copy | 13.86 | 0.00 | 11 | 0 |

