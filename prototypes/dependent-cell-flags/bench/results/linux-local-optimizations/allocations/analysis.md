## backfill_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.509 | [0.496, 0.516] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.995 | [0.995, 0.995] | 0/3 |
| after | allocated_bytes | 0.976 | [0.975, 0.977] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [0.999, 1.000] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 506.69 | 487.63 | 495.00–532.58 | 2.73 |
| after | 260.00 | 237.71 | 255.22–264.23 | 4.11 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 625150 | 1640.7 | 236.2 |
| after | 622058 | 1601.0 | 236.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 93.76 | 27.49 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |
| after | 93.78 | 27.51 | 141 | 13 | 3.8 | 2.3 | 1.5 | 1.5 |

## backfill_chain.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.981 | [0.920, 1.042] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 1.05 | 0.91 | 0.97–1.10 | 1.02 |
| after | 1.06 | 0.91 | 0.95–1.09 | 1.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 1474 | 5.2 | 5.0 |
| after | 1474 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.01 | 1 | 3 |
| after | 0.00 | 0.01 | 1 | 3 |

## backfill_chain.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.996 | [0.953, 1.038] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.991 | [0.977, 1.008] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 63.33 | 60.04 | 63.04–69.26 | 9.97 |
| after | 65.54 | 52.44 | 63.56–66.02 | 8.75 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 343023 | 804.5 | 172.9 |
| after | 343023 | 804.5 | 170.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 66.26 | 0.00 | 90 | 0 |
| after | 66.26 | 0.00 | 90 | 0 |

## backfill_chain.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.020 | [0.902, 1.128] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 0.996 | [0.991, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 0.997 | [0.989, 1.011] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 28.03 | 24.50 | 25.87–29.80 | 8.89 |
| after | 27.61 | 24.04 | 26.89–29.18 | 7.82 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 137618 | 300.8 | 38.5 |
| after | 137617 | 301.0 | 38.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 27.50 | 0.00 | 50 | 0 |
| after | 27.52 | 0.00 | 50 | 0 |

## backfill_chain.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.399 | [0.388, 0.406] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.978 | [0.978, 0.978] | 0/3 |
| after | allocated_bytes | 0.928 | [0.924, 0.936] | 0/3 |
| after | peak_live_growth_bytes | 0.997 | [0.993, 1.003] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 412.16 | 396.20 | 406.17–439.44 | 1.27 |
| after | 165.36 | 159.52 | 163.23–170.63 | 1.68 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 143036 | 530.4 | 18.5 |
| after | 139945 | 490.5 | 18.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 27.49 | 0 | 10 |
| after | 0.00 | 27.50 | 0 | 10 |

## backfill_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.614 | [0.598, 0.630] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.997 | [0.997, 0.997] | 0/3 |
| after | allocated_bytes | 0.984 | [0.982, 0.985] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 343.42 | 319.73 | 323.25–382.59 | 2.68 |
| after | 220.48 | 194.99 | 203.74–228.63 | 3.77 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 505927 | 1238.5 | 201.6 |
| after | 504377 | 1218.2 | 201.6 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 80.20 | 13.87 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |
| after | 80.19 | 13.86 | 121 | 13 | 3.3 | 2.1 | 1.1 | 1.1 |

## backfill_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.934 | [0.886, 1.006] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 1.09 | 0.89 | 0.95–1.27 | 1.02 |
| after | 1.03 | 0.93 | 0.96–1.13 | 1.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 1167 | 5.2 | 5.0 |
| after | 1167 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.00 | 1 | 3 |
| after | 0.00 | 0.00 | 1 | 3 |

## backfill_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.005 | [0.980, 1.039] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.007 | [0.996, 1.023] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 64.48 | 60.30 | 61.95–71.54 | 8.52 |
| after | 64.08 | 49.22 | 60.75–71.32 | 8.71 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 343206 | 806.1 | 173.3 |
| after | 343206 | 806.1 | 173.4 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 66.32 | 0.00 | 90 | 0 |
| after | 66.32 | 0.00 | 90 | 0 |

## backfill_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.992 | [0.923, 1.102] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 0.999 | [0.991, 1.008] | 1/3 |
| after | peak_live_growth_bytes | 0.973 | [0.966, 0.985] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 15.36 | 13.36 | 13.67–17.05 | 7.51 |
| after | 15.06 | 11.65 | 13.98–15.75 | 6.72 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 72915 | 151.8 | 20.2 |
| after | 72915 | 151.7 | 19.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.88 | 0.00 | 30 | 0 |
| after | 13.87 | 0.00 | 30 | 0 |

## backfill_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.505 | [0.501, 0.511] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.983 | [0.983, 0.983] | 0/3 |
| after | allocated_bytes | 0.927 | [0.925, 0.929] | 0/3 |
| after | peak_live_growth_bytes | 0.998 | [0.998, 0.998] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 262.48 | 243.47 | 247.05–283.56 | 0.99 |
| after | 139.60 | 120.36 | 124.33–145.03 | 0.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 88639 | 275.4 | 10.3 |
| after | 87089 | 255.3 | 10.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 13.87 | 0 | 10 |
| after | 0.00 | 13.86 | 0 | 10 |

## backfill_plain_chain
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.989 | [0.943, 1.026] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.001] | 2/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.001] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 245.73 | 224.24 | 240.59–260.42 | 4.39 |
| after | 246.20 | 241.84 | 245.48–246.87 | 4.44 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 580587 | 1581.2 | 236.3 |
| after | 580586 | 1582.1 | 236.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 93.97 | 27.64 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |
| after | 93.91 | 27.58 | 141 | 13 | 2.9 | 2.0 | 0.8 | 0.8 |

## backfill_plain_chain.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.011 | [0.920, 1.163] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 0.95 | 0.81 | 0.83–1.00 | 1.02 |
| after | 0.96 | 0.81 | 0.92–0.97 | 1.02 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 924 | 5.2 | 5.0 |
| after | 924 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.00 | 1 | 3 |
| after | 0.00 | 0.00 | 1 | 3 |

## backfill_plain_chain.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.969 | [0.905, 1.025] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.000 | [0.994, 1.007] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 67.03 | 56.04 | 64.50–73.94 | 8.99 |
| after | 66.12 | 64.02 | 65.66–66.89 | 9.56 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 343125 | 805.5 | 172.4 |
| after | 343124 | 805.5 | 172.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 66.32 | 0.00 | 90 | 0 |
| after | 66.32 | 0.00 | 90 | 0 |

## backfill_plain_chain.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.013 | [0.976, 1.055] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 0.994 | [0.989, 1.001] | 1/3 |
| after | peak_live_growth_bytes | 0.951 | [0.911, 0.994] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 28.25 | 23.36 | 26.43–29.28 | 9.34 |
| after | 28.57 | 22.92 | 27.88–28.84 | 8.15 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 137618 | 302.6 | 39.9 |
| after | 137617 | 301.9 | 36.7 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 27.65 | 0.00 | 50 | 0 |
| after | 27.59 | 0.00 | 50 | 0 |

## backfill_plain_chain.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.009 | [0.955, 1.059] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.001 | [0.991, 1.009] | 2/3 |
| after | peak_live_growth_bytes | 1.004 | [0.995, 1.013] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 153.31 | 140.90 | 146.30–159.12 | 1.75 |
| after | 151.93 | 146.54 | 150.86–155.00 | 1.73 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 98920 | 467.4 | 18.4 |
| after | 98921 | 469.6 | 18.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 27.63 | 0 | 10 |
| after | 0.00 | 27.58 | 0 | 10 |

## backfill_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.015 | [0.952, 1.076] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 3/3 |
| after | allocated_bytes | 1.000 | [0.997, 1.003] | 1/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 201.58 | 186.58 | 188.53–232.22 | 4.25 |
| after | 202.64 | 192.28 | 199.02–221.13 | 4.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 468634 | 1202.1 | 201.4 |
| after | 468635 | 1201.6 | 201.4 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 80.12 | 13.79 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |
| after | 80.08 | 13.75 | 121 | 13 | 2.8 | 2.0 | 0.8 | 0.8 |

## backfill_plain_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.049 | [1.019, 1.068] | 3/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 0.93 | 0.84 | 0.87–1.01 | 1.02 |
| after | 0.94 | 0.84 | 0.93–1.03 | 1.03 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 902 | 5.2 | 5.0 |
| after | 902 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.00 | 1 | 3 |
| after | 0.00 | 0.00 | 1 | 3 |

## backfill_plain_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.019 | [0.959, 1.076] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.005 | [0.986, 1.018] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 62.30 | 58.99 | 60.00–72.87 | 9.71 |
| after | 64.57 | 50.73 | 63.92–69.89 | 9.33 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 343124 | 805.5 | 170.8 |
| after | 343125 | 805.5 | 172.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 66.32 | 0.00 | 90 | 0 |
| after | 66.32 | 0.00 | 90 | 0 |

## backfill_plain_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.978 | [0.955, 1.004] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 0.999 | [0.984, 1.018] | 1/3 |
| after | peak_live_growth_bytes | 0.996 | [0.978, 1.016] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 14.84 | 13.88 | 14.05–16.73 | 7.68 |
| after | 14.90 | 13.08 | 13.71–15.98 | 7.16 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 72915 | 150.8 | 20.2 |
| after | 72915 | 150.4 | 20.2 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.80 | 0.00 | 30 | 0 |
| after | 13.77 | 0.00 | 30 | 0 |

## backfill_plain_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.023 | [0.994, 1.075] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [0.996, 1.005] | 1/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 126.76 | 112.62 | 112.79–134.10 | 0.98 |
| after | 122.34 | 115.30 | 115.73–134.23 | 0.99 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 51693 | 240.6 | 10.3 |
| after | 51693 | 240.4 | 10.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 13.78 | 0 | 10 |
| after | 0.00 | 13.75 | 0 | 10 |

## cycle_inplace_sparse_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.976 | [0.903, 1.035] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 0.999 | [0.999, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 0.998 | [0.995, 1.003] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 372.25 | 355.85 | 365.14–437.44 | 3.65 |
| after | 383.65 | 352.62 | 366.98–394.83 | 3.66 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 839411 | 2105.1 | 224.5 |
| after | 839411 | 2103.2 | 224.5 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 96.39 | 64.72 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |
| after | 96.26 | 64.59 | 545 | 26 | 5.2 | 3.9 | 1.3 | 1.3 |

## cycle_inplace_sparse_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.036 | [1.008, 1.065] | 3/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 1.03 | 0.91 | 0.99–1.26 | 1.04 |
| after | 1.07 | 0.95 | 1.03–1.34 | 1.03 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 1492 | 5.2 | 5.0 |
| after | 1492 | 5.2 | 5.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.01 | 1 | 3 |
| after | 0.00 | 0.01 | 1 | 3 |

## cycle_inplace_sparse_one.find
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.964 | [0.941, 0.989] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.031 | [0.959, 1.138] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 14.02 | 13.17 | 13.23–14.21 | 1.06 |
| after | 13.16 | 12.95 | 13.01–13.66 | 1.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 5691 | 8.3 | 0.1 |
| after | 5691 | 8.3 | 0.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.00 | 0 | 0 |
| after | 0.00 | 0.00 | 0 | 0 |

## cycle_inplace_sparse_one.read_inputs
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.130 | [0.966, 1.313] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.001 | [1.000, 1.001] | 3/3 |
| after | peak_live_growth_bytes | 1.003 | [1.001, 1.004] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 5.06 | 4.60 | 4.71–5.17 | 5.37 |
| after | 5.67 | 4.80 | 4.96–6.19 | 5.25 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 19360 | 11.4 | 1.8 |
| after | 19361 | 11.4 | 1.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 1.21 | 0.00 | 373 | 0 |
| after | 1.21 | 0.00 | 373 | 0 |

## cycle_inplace_sparse_one.read_pending
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.950 | [0.877, 0.990] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.990 | [0.976, 1.015] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 19.65 | 16.27 | 17.24–22.05 | 6.42 |
| after | 17.81 | 16.19 | 17.07–19.34 | 5.26 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 74703 | 162.0 | 22.8 |
| after | 74704 | 162.0 | 22.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 14.08 | 0.00 | 40 | 0 |
| after | 14.08 | 0.00 | 40 | 0 |

## cycle_inplace_sparse_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.983 | [0.958, 1.034] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 0.992 | [0.986, 0.997] | 0/3 |
| after | peak_live_growth_bytes | 0.990 | [0.946, 1.023] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 14.39 | 11.94 | 13.65–14.52 | 7.47 |
| after | 14.02 | 11.02 | 13.09–14.88 | 5.93 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73154 | 152.7 | 20.1 |
| after | 73154 | 151.2 | 20.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.95 | 0.00 | 30 | 0 |
| after | 13.82 | 0.00 | 30 | 0 |

## cycle_inplace_sparse_one.stage
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.001 | [0.948, 1.049] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 0.999 | [0.998, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 154.31 | 148.70 | 150.73–179.72 | 1.42 |
| after | 160.47 | 150.76 | 151.82–170.39 | 1.31 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 150873 | 436.8 | 12.9 |
| after | 150874 | 436.1 | 12.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.99 | 13.94 | 10 | 10 |
| after | 13.99 | 13.81 | 10 | 10 |

## cycle_inplace_sparse_one.update
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.969 | [0.841, 1.100] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 2/3 |
| after | peak_live_growth_bytes | 0.998 | [0.995, 1.003] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 168.12 | 153.95 | 157.84–202.98 | 5.91 |
| after | 170.78 | 146.25 | 165.53–173.55 | 5.83 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 514137 | 1328.5 | 224.5 |
| after | 514135 | 1328.6 | 224.5 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 53.15 | 50.80 | 91 | 13 |
| after | 53.15 | 50.79 | 91 | 13 |

## cycle_inplace_sparse_plain_one
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.992 | [0.952, 1.039] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [0.999, 1.001] | 2/3 |
| after | peak_live_growth_bytes | 1.002 | [0.999, 1.007] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 298.35 | 273.04 | 277.54–313.33 | 5.93 |
| after | 297.16 | 238.24 | 288.48–304.52 | 5.98 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 1038770 | 2222.0 | 225.1 |
| after | 1038770 | 2222.7 | 225.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 102.07 | 66.88 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |
| after | 102.09 | 67.02 | 493 | 26 | 7.5 | 3.9 | 3.6 | 3.6 |

## cycle_inplace_sparse_plain_one.commit
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.976 | [0.871, 1.052] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 0/3 |
| after | peak_live_growth_bytes | 1.000 | [1.000, 1.000] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 0.91 | 0.81 | 0.88–1.14 | 1.03 |
| after | 0.93 | 0.77 | 0.91–1.00 | 1.04 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 917 | 5.2 | 5.1 |
| after | 917 | 5.2 | 5.1 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.01 | 1 | 3 |
| after | 0.00 | 0.01 | 1 | 3 |

## cycle_inplace_sparse_plain_one.locate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.024 | [0.895, 1.181] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.001] | 3/3 |
| after | peak_live_growth_bytes | 1.018 | [0.998, 1.030] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 16.16 | 14.15 | 14.78–18.37 | 3.50 |
| after | 16.50 | 13.71 | 16.07–17.46 | 3.75 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 53320 | 39.5 | 4.3 |
| after | 53319 | 39.5 | 4.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 3.25 | 0.00 | 301 | 0 |
| after | 3.25 | 0.00 | 301 | 0 |

## cycle_inplace_sparse_plain_one.merge
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.951 | [0.884, 1.011] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [0.999, 1.001] | 2/3 |
| after | peak_live_growth_bytes | 1.000 | [0.993, 1.010] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 87.06 | 79.02 | 84.70–94.52 | 6.03 |
| after | 83.53 | 64.56 | 81.55–89.29 | 6.40 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 323562 | 548.6 | 102.9 |
| after | 323563 | 548.6 | 103.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 18.17 | 16.12 | 30 | 10 |
| after | 18.17 | 16.13 | 30 | 10 |

## cycle_inplace_sparse_plain_one.read_pending
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.011 | [0.999, 1.030] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.966 | [0.940, 1.016] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 14.02 | 12.12 | 13.41–14.44 | 7.95 |
| after | 14.42 | 10.57 | 13.46–14.44 | 7.09 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73926 | 149.5 | 19.9 |
| after | 73926 | 149.6 | 18.9 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.69 | 0.00 | 40 | 0 |
| after | 13.69 | 0.00 | 40 | 0 |

## cycle_inplace_sparse_plain_one.read_published
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.983 | [0.929, 1.018] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [0.990, 1.010] | 2/3 |
| after | peak_live_growth_bytes | 1.044 | [1.033, 1.051] | 3/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 14.43 | 12.77 | 13.48–15.07 | 7.32 |
| after | 13.73 | 10.47 | 13.56–14.40 | 8.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73443 | 151.5 | 18.6 |
| after | 73443 | 151.6 | 20.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.84 | 0.00 | 30 | 0 |
| after | 13.86 | 0.00 | 30 | 0 |

## cycle_inplace_sparse_plain_one.update
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.009 | [0.961, 1.069] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.002 | [0.999, 1.007] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 169.96 | 148.03 | 152.18–175.44 | 6.04 |
| after | 168.30 | 137.94 | 162.62–170.13 | 5.91 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 513603 | 1327.9 | 225.1 |
| after | 513600 | 1328.2 | 225.3 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 53.13 | 50.79 | 91 | 13 |
| after | 53.13 | 50.79 | 91 | 13 |

## scan_masked
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.099 | [0.088, 0.126] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.105 | [0.105, 0.105] | 0/3 |
| after | allocated_bytes | 0.030 | [0.030, 0.030] | 0/3 |
| after | peak_live_growth_bytes | 0.039 | [0.036, 0.041] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 13.31 | 10.42 | 11.27–14.42 | 5.62 |
| after | 1.27 | 0.99 | 1.17–1.42 | 1.66 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73191 | 157.6 | 19.4 |
| after | 7673 | 4.7 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 14.00 | 0.00 | 10 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |
| after | 0.00 | 0.00 | 0 | 0 | 326.3 | 3.0 | 323.3 | 323.3 |

## scan_masked.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.099 | [0.088, 0.126] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.105 | [0.105, 0.105] | 0/3 |
| after | allocated_bytes | 0.030 | [0.030, 0.030] | 0/3 |
| after | peak_live_growth_bytes | 0.039 | [0.036, 0.041] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 13.31 | 10.42 | 11.27–14.42 | 5.62 |
| after | 1.27 | 0.99 | 1.17–1.42 | 1.66 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73191 | 157.6 | 19.4 |
| after | 7673 | 4.7 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 14.00 | 0.00 | 10 | 0 |
| after | 0.00 | 0.00 | 0 | 0 |

## scan_null_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.972 | [0.904, 1.030] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.956 | [0.948, 0.968] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 17.26 | 11.77 | 16.88–17.46 | 7.37 |
| after | 16.76 | 14.82 | 15.79–17.39 | 7.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 89051 | 156.6 | 19.8 |
| after | 89051 | 156.6 | 18.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 13.79 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| after | 13.79 | 0.00 | 10 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_1pct.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.972 | [0.904, 1.030] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 0/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.956 | [0.948, 0.968] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 17.26 | 11.77 | 16.88–17.46 | 7.37 |
| after | 16.76 | 14.82 | 15.79–17.39 | 7.65 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 89051 | 156.6 | 19.8 |
| after | 89051 | 156.6 | 18.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.79 | 0.00 | 10 | 0 |
| after | 13.79 | 0.00 | 10 | 0 |

## scan_null_all
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.007 | [0.927, 1.186] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.001 | [1.000, 1.001] | 3/3 |
| after | peak_live_growth_bytes | 1.001 | [0.959, 1.081] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 2.40 | 1.82 | 2.06–2.77 | 3.96 |
| after | 2.45 | 1.93 | 2.32–2.57 | 4.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 11975 | 7.1 | 0.9 |
| after | 11975 | 7.1 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |
| after | 0.00 | 0.00 | 0 | 0 | 5.6 | 3.0 | 2.6 | 2.6 |

## scan_null_all.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.007 | [0.927, 1.186] | 1/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 1/3 |
| after | allocated_bytes | 1.001 | [1.000, 1.001] | 3/3 |
| after | peak_live_growth_bytes | 1.001 | [0.959, 1.081] | 1/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 2.40 | 1.82 | 2.06–2.77 | 3.96 |
| after | 2.45 | 1.93 | 2.32–2.57 | 4.38 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 11975 | 7.1 | 0.8 |
| after | 11975 | 7.1 | 0.8 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 0.00 | 0.00 | 0 | 0 |
| after | 0.00 | 0.00 | 0 | 0 |

## scan_partial_1pct
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.015 | [0.894, 1.095] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.034 | [0.985, 1.104] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 20.62 | 18.19 | 19.31–22.36 | 5.23 |
| after | 21.14 | 18.08 | 20.00–21.29 | 5.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73100 | 158.0 | 22.9 |
| after | 73101 | 158.0 | 24.0 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 13.71 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |
| after | 13.71 | 0.00 | 10 | 0 | 84.4 | 42.0 | 42.4 | 42.3 |

## scan_partial_1pct.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.015 | [0.894, 1.095] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 1.034 | [0.985, 1.104] | 2/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 20.62 | 18.19 | 19.31–22.36 | 5.23 |
| after | 21.14 | 18.08 | 20.00–21.29 | 5.07 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 73100 | 158.0 | 22.9 |
| after | 73101 | 158.0 | 24.0 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.71 | 0.00 | 10 | 0 |
| after | 13.71 | 0.00 | 10 | 0 |

## scan_plain_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.019 | [0.996, 1.039] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.996 | [0.992, 0.999] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 24.58 | 22.76 | 24.25–25.64 | 4.43 |
| after | 25.45 | 23.93 | 25.21–26.24 | 4.49 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 76634 | 246.0 | 32.3 |
| after | 76635 | 246.0 | 32.2 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 13.96 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |
| after | 13.96 | 0.00 | 11 | 0 | 4.3 | 2.3 | 2.0 | 2.0 |

## scan_plain_moved.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 1.019 | [0.996, 1.039] | 2/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 1.000 | [1.000, 1.000] | 2/3 |
| after | allocated_bytes | 1.000 | [1.000, 1.000] | 3/3 |
| after | peak_live_growth_bytes | 0.996 | [0.992, 0.999] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 24.58 | 22.76 | 24.25–25.64 | 4.43 |
| after | 25.45 | 23.93 | 25.21–26.24 | 4.49 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 76634 | 246.0 | 32.3 |
| after | 76635 | 246.0 | 32.2 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.96 | 0.00 | 11 | 0 |
| after | 13.96 | 0.00 | 11 | 0 |

## scan_ready_moved
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.543 | [0.515, 0.563] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.993 | [0.993, 0.993] | 0/3 |
| after | allocated_bytes | 0.969 | [0.969, 0.970] | 0/3 |
| after | peak_live_growth_bytes | 0.862 | [0.847, 0.873] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 45.45 | 42.02 | 43.94–48.71 | 3.27 |
| after | 25.07 | 22.99 | 24.72–25.09 | 4.22 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 77250 | 252.0 | 36.2 |
| after | 76690 | 244.3 | 31.3 |

| build | median read MiB | median written MiB | median read requests | median write requests | median manifest file KiB | median manifest proper KiB | median inline transaction KiB | median transaction file KiB |
|---|---|---|---|---|---|---|---|---|
| before | 13.86 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |
| after | 13.86 | 0.00 | 11 | 0 | 71.2 | 41.4 | 29.9 | 29.9 |

## scan_ready_moved.read
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| after | wall_ns | 0.543 | [0.515, 0.563] | 0/3 |
| after | instructions | unavailable | — | — |
| after | cycles | unavailable | — | — |
| after | allocations | 0.993 | [0.993, 0.993] | 0/3 |
| after | allocated_bytes | 0.969 | [0.969, 0.970] | 0/3 |
| after | peak_live_growth_bytes | 0.862 | [0.847, 0.873] | 0/3 |

| build | median wall ms | p10 wall ms | round medians ms | busy cores |
|---|---|---|---|---|
| before | 45.45 | 42.02 | 43.94–48.71 | 3.27 |
| after | 25.07 | 22.99 | 24.72–25.09 | 4.22 |

| build | median allocations | median allocated MiB | median peak live growth MiB |
|---|---|---|---|
| before | 77250 | 252.0 | 36.2 |
| after | 76690 | 244.3 | 31.2 |

| build | median read MiB | median written MiB | median read requests | median write requests |
|---|---|---|---|---|
| before | 13.86 | 0.00 | 11 | 0 |
| after | 13.86 | 0.00 | 11 | 0 |

