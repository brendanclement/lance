## Unrelated writes

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| unrelated moving write, one / plain | 16.54 | 15.58–17.32 | 16.55 | 15.56–17.13 | 1.002 | [0.998, 1.006] | 9/16 | 77.2 | 77.2 | 7.6 | 7.5 | 6.9 | 0.0 | 3.1 | 2.3 |
| unrelated moving write, shared / plain | 16.69 | 15.69–17.13 | 16.55 | 15.56–17.13 | 1.006 | [1.000, 1.013] | 10/16 | 77.2 | 77.2 | 7.3 | 7.5 | 6.9 | 0.0 | 3.8 | 2.3 |
| unrelated moving write, chain / plain | 16.60 | 15.36–17.09 | 16.55 | 15.56–17.13 | 1.000 | [0.995, 1.006] | 8/16 | 77.1 | 77.2 | 7.5 | 7.5 | 6.9 | 0.0 | 3.8 | 2.3 |
| unrelated inplace write, one / plain | 10.45 | 10.28–10.65 | 10.52 | 10.39–10.67 | 0.995 | [0.991, 0.999] | 5/16 | 195.1 | 195.1 | 20.3 | 21.3 | 7.1 | 4.7 | 3.1 | 3.0 |
| unrelated inplace write, shared / plain | 10.44 | 10.29–10.64 | 10.52 | 10.39–10.67 | 0.991 | [0.987, 0.995] | 3/16 | 195.1 | 195.1 | 20.8 | 21.3 | 7.1 | 4.7 | 3.2 | 3.0 |
| unrelated inplace write, chain / plain | 10.48 | 10.34–10.61 | 10.52 | 10.39–10.67 | 0.996 | [0.991, 1.001] | 5/16 | 195.1 | 195.1 | 21.0 | 21.3 | 7.1 | 4.7 | 3.2 | 3.0 |

## Refresh cycles: flagged / plain merge_insert

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| moving_sparse one: whole cycle | 51.12 | 49.43–52.17 | 61.18 | 59.26–62.60 | 0.836 | [0.831, 0.841] | 0/16 | 449.2 | 502.9 | 23.6 | 20.0 | 34.3 | 0.0 | 3.1 | 2.4 |
| moving_sparse one: update | 16.56 | 15.70–17.19 | 16.54 | 15.54–17.10 | 1.003 | [0.997, 1.008] | 12/16 | 77.2 | 77.2 | 7.6 | 7.5 | 6.9 | 0.0 | — | — |
| moving_sparse one: read_pending | 11.36 | 10.74–11.61 | 9.29 | 8.89–9.66 | 1.215 | [1.196, 1.236] | 16/16 | 178.5 | 173.0 | 22.9 | 19.3 | 13.7 | 0.0 | — | — |
| moving_sparse one: locate | 11.07 | 10.85–11.42 | 14.25 | 13.58–14.73 | 0.778 | [0.768, 0.788] | 0/16 | 9.7 | 32.1 | 0.2 | 4.2 | 0.0 | 0.0 | — | — |
| moving_sparse one: stage | 0.560 | 0.538–0.593 | 11.11 | 10.93–11.42 | 0.050 | [0.050, 0.051] | 0/16 | 0.3 | 42.5 | 0.1 | 6.8 | 0.0 | 0.0 | — | — |
| moving_sparse one: commit | 0.934 | 0.919–0.965 | 0.923 | 0.904–0.954 | 1.013 | [1.005, 1.021] | 12/16 | 5.1 | 5.1 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |
| moving_sparse one: read_published | 10.79 | 10.47–11.01 | 9.01 | 8.81–9.55 | 1.189 | [1.175, 1.203] | 16/16 | 178.4 | 173.0 | 22.6 | 19.4 | 13.7 | 0.0 | — | — |
| moving_sparse shared: whole cycle | 61.47 | 60.30–62.97 | 64.23 | 61.77–65.14 | 0.957 | [0.951, 0.963] | 0/16 | 781.1 | 832.2 | 42.5 | 38.3 | 62.4 | 0.0 | 3.9 | 2.4 |
| moving_sparse shared: update | 16.50 | 15.73–16.97 | 16.54 | 15.64–16.93 | 1.002 | [0.995, 1.009] | 10/16 | 77.2 | 77.2 | 7.6 | 7.5 | 6.9 | 0.0 | — | — |
| moving_sparse shared: read_pending | 14.43 | 13.80–14.97 | 10.81 | 10.64–11.07 | 1.327 | [1.311, 1.342] | 16/16 | 344.4 | 337.6 | 41.9 | 37.4 | 27.8 | 0.0 | — | — |
| moving_sparse shared: locate | 14.88 | 14.61–15.31 | 14.34 | 13.37–14.83 | 1.041 | [1.029, 1.054] | 16/16 | 9.6 | 32.1 | 0.2 | 4.2 | 0.0 | 0.0 | — | — |
| moving_sparse shared: stage | 0.602 | 0.582–0.670 | 11.14 | 10.90–11.40 | 0.055 | [0.054, 0.056] | 0/16 | 0.4 | 42.6 | 0.1 | 6.8 | 0.0 | 0.0 | — | — |
| moving_sparse shared: commit | 0.957 | 0.935–1.01 | 0.938 | 0.909–0.981 | 1.027 | [1.020, 1.034] | 15/16 | 5.1 | 5.1 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |
| moving_sparse shared: read_published | 14.10 | 13.77–14.54 | 10.50 | 10.31–10.76 | 1.342 | [1.331, 1.355] | 16/16 | 344.3 | 337.5 | 41.0 | 36.9 | 27.8 | 0.0 | — | — |
| moving_sparse chain: whole cycle | 61.40 | 60.15–62.82 | 64.32 | 61.81–65.39 | 0.958 | [0.952, 0.964] | 0/16 | 773.0 | 827.7 | 41.4 | 37.7 | 61.7 | 0.0 | 3.9 | 2.4 |
| moving_sparse chain: update | 16.57 | 15.55–17.19 | 16.58 | 15.52–17.01 | 1.004 | [1.000, 1.009] | 11/16 | 77.1 | 77.2 | 7.5 | 7.6 | 6.9 | 0.0 | — | — |
| moving_sparse chain: read_pending | 14.43 | 13.95–14.91 | 10.76 | 10.53–11.13 | 1.334 | [1.324, 1.344] | 16/16 | 340.4 | 335.4 | 40.7 | 37.1 | 27.4 | 0.0 | — | — |
| moving_sparse chain: locate | 14.93 | 14.53–15.21 | 14.41 | 13.31–14.78 | 1.043 | [1.029, 1.060] | 16/16 | 9.6 | 32.1 | 0.1 | 4.0 | 0.0 | 0.0 | — | — |
| moving_sparse chain: stage | 0.606 | 0.589–0.659 | 11.15 | 10.95–11.29 | 0.055 | [0.054, 0.056] | 0/16 | 0.4 | 42.6 | 0.1 | 6.8 | 0.0 | 0.0 | — | — |
| moving_sparse chain: commit | 0.960 | 0.933–0.993 | 0.933 | 0.911–0.955 | 1.030 | [1.020, 1.039] | 15/16 | 5.1 | 5.1 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |
| moving_sparse chain: read_published | 14.04 | 13.66–14.34 | 10.49 | 10.17–10.60 | 1.342 | [1.332, 1.351] | 16/16 | 340.3 | 335.3 | 40.7 | 36.7 | 27.4 | 0.0 | — | — |
| moving_dense one: whole cycle | 133 | 132–134 | 114 | 113–117 | 1.161 | [1.157, 1.165] | 16/16 | 7100.7 | 7335.3 | 1205.1 | 1209.3 | 148.7 | 13.3 | 162.8 | 2.4 |
| moving_dense one: update | 75.29 | 74.58–75.99 | 60.43 | 59.33–62.10 | 1.244 | [1.236, 1.250] | 16/16 | 4970.7 | 4979.3 | 1205.1 | 1209.3 | 110.2 | 11.8 | — | — |
| moving_dense one: read_pending | 10.43 | 10.20–10.69 | 8.85 | 8.75–9.00 | 1.179 | [1.170, 1.188] | 16/16 | 954.4 | 959.0 | 138.3 | 137.0 | 15.2 | 0.0 | — | — |
| moving_dense one: locate | 11.03 | 10.87–11.21 | 11.41 | 11.27–11.66 | 0.966 | [0.961, 0.971] | 0/16 | 137.1 | 202.8 | 29.4 | 25.8 | 6.8 | 0.0 | — | — |
| moving_dense one: stage | 25.23 | 25.02–25.48 | 24.34 | 24.14–24.76 | 1.036 | [1.033, 1.039] | 16/16 | 79.6 | 232.0 | 10.3 | 27.9 | 1.4 | 1.4 | — | — |
| moving_dense one: commit | 1.03 | 1.01–1.05 | 0.971 | 0.961–1.00 | 1.059 | [1.051, 1.067] | 16/16 | 5.8 | 5.1 | 5.5 | 5.0 | 0.0 | 0.2 | — | — |
| moving_dense one: read_published | 9.90 | 9.74–10.04 | 8.30 | 8.16–8.48 | 1.192 | [1.187, 1.197] | 16/16 | 953.0 | 957.3 | 138.0 | 136.8 | 15.1 | 0.0 | — | — |
| moving_dense shared: whole cycle | 160 | 158–161 | 123 | 122–124 | 1.301 | [1.298, 1.304] | 16/16 | 9030.5 | 9212.5 | 1205.8 | 1206.1 | 181.1 | 15.1 | 323.1 | 2.4 |
| moving_dense shared: update | 75.69 | 75.14–76.90 | 60.39 | 59.74–61.78 | 1.254 | [1.250, 1.259] | 16/16 | 4978.6 | 4979.2 | 1205.8 | 1206.1 | 110.3 | 11.9 | — | — |
| moving_dense shared: read_pending | 14.78 | 14.36–14.99 | 11.95 | 11.71–12.22 | 1.234 | [1.222, 1.246] | 16/16 | 1881.4 | 1878.4 | 269.6 | 266.9 | 30.7 | 0.0 | — | — |
| moving_dense shared: locate | 13.42 | 13.26–13.56 | 11.36 | 11.20–11.56 | 1.178 | [1.173, 1.183] | 16/16 | 137.1 | 202.4 | 29.3 | 25.7 | 6.7 | 0.0 | — | — |
| moving_dense shared: stage | 40.37 | 39.99–40.54 | 26.49 | 26.23–27.07 | 1.522 | [1.516, 1.527] | 16/16 | 148.2 | 271.2 | 19.0 | 34.0 | 2.8 | 2.8 | — | — |
| moving_dense shared: commit | 1.12 | 1.08–1.14 | 0.974 | 0.953–0.990 | 1.152 | [1.142, 1.161] | 16/16 | 6.4 | 5.1 | 6.0 | 5.0 | 0.0 | 0.3 | — | — |
| moving_dense shared: read_published | 14.17 | 14.02–14.52 | 11.27 | 11.08–11.64 | 1.259 | [1.250, 1.268] | 16/16 | 1879.0 | 1876.3 | 269.5 | 266.7 | 30.6 | 0.0 | — | — |
| moving_dense chain: whole cycle | 160 | 159–161 | 123 | 122–125 | 1.296 | [1.291, 1.301] | 16/16 | 8980.0 | 9189.1 | 1203.8 | 1209.9 | 180.2 | 15.1 | 323.1 | 2.4 |
| moving_dense chain: update | 76.04 | 75.20–76.82 | 61.06 | 59.75–62.19 | 1.245 | [1.237, 1.253] | 16/16 | 4970.4 | 4979.2 | 1203.8 | 1209.9 | 110.2 | 11.9 | — | — |
| moving_dense chain: read_pending | 14.79 | 14.49–15.09 | 12.08 | 11.89–12.39 | 1.223 | [1.215, 1.230] | 16/16 | 1860.5 | 1866.8 | 263.7 | 263.5 | 30.3 | 0.0 | — | — |
| moving_dense chain: locate | 13.58 | 13.37–13.70 | 11.43 | 11.22–11.70 | 1.186 | [1.180, 1.193] | 16/16 | 137.1 | 202.5 | 29.3 | 25.7 | 6.7 | 0.0 | — | — |
| moving_dense chain: stage | 40.42 | 40.01–40.63 | 26.46 | 26.18–26.87 | 1.525 | [1.521, 1.529] | 16/16 | 147.2 | 271.0 | 19.0 | 33.9 | 2.7 | 2.8 | — | — |
| moving_dense chain: commit | 1.10 | 1.09–1.13 | 0.968 | 0.955–0.989 | 1.140 | [1.134, 1.147] | 16/16 | 6.4 | 5.1 | 6.0 | 5.0 | 0.0 | 0.3 | — | — |
| moving_dense chain: read_published | 14.03 | 13.77–14.25 | 11.17 | 11.07–11.32 | 1.255 | [1.248, 1.262] | 16/16 | 1858.4 | 1864.7 | 263.5 | 263.2 | 30.2 | 0.0 | — | — |
| inplace_sparse one: whole cycle | 142 | 140–143 | 101 | 99.93–103 | 1.400 | [1.395, 1.404] | 16/16 | 2095.9 | 2228.9 | 224.1 | 224.0 | 95.7 | 64.7 | 3.9 | 3.9 |
| inplace_sparse one: update | 59.34 | 58.40–60.38 | 59.45 | 58.63–60.07 | 0.999 | [0.995, 1.002] | 6/16 | 1327.2 | 1327.5 | 224.1 | 224.0 | 53.1 | 50.8 | — | — |
| inplace_sparse one: read_pending | 4.65 | 4.57–4.76 | 3.09 | 3.02–3.16 | 1.512 | [1.501, 1.523] | 16/16 | 158.6 | 153.1 | 22.8 | 19.2 | 13.8 | 0.0 | — | — |
| inplace_sparse one: locate | 8.18 | 8.06–8.35 | 10.97 | 9.94–11.42 | 0.751 | [0.738, 0.766] | 0/16 | 19.8 | 39.6 | 1.9 | 4.1 | 1.2 | 0.0 | — | — |
| inplace_sparse one: stage | 65.55 | 64.51–66.39 | 23.94 | 23.60–24.26 | 2.738 | [2.724, 2.752] | 16/16 | 433.0 | 551.5 | 12.9 | 102.6 | 13.7 | 13.8 | — | — |
| inplace_sparse one: commit | 1.02 | 0.994–1.06 | 1.03 | 0.997–1.06 | 0.992 | [0.981, 1.004] | 8/16 | 5.2 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| inplace_sparse one: read_published | 2.95 | 2.90–3.03 | 2.94 | 2.88–3.05 | 1.003 | [0.996, 1.011] | 10/16 | 151.8 | 151.7 | 18.9 | 18.7 | 13.9 | 0.0 | — | — |
| inplace_sparse shared: whole cycle | 168 | 166–169 | 112 | 111–114 | 1.489 | [1.485, 1.493] | 16/16 | 2809.6 | 2941.1 | 224.0 | 224.5 | 137.8 | 78.6 | 4.1 | 4.0 |
| inplace_sparse shared: update | 59.55 | 58.98–60.60 | 59.36 | 59.00–59.97 | 1.004 | [0.999, 1.008] | 10/16 | 1327.8 | 1327.6 | 224.0 | 224.5 | 53.1 | 50.8 | — | — |
| inplace_sparse shared: read_pending | 8.05 | 7.90–8.16 | 4.97 | 4.89–5.10 | 1.616 | [1.604, 1.627] | 16/16 | 312.8 | 305.9 | 41.6 | 37.9 | 27.8 | 0.0 | — | — |
| inplace_sparse shared: locate | 12.08 | 11.82–12.33 | 10.97 | 10.17–11.41 | 1.104 | [1.087, 1.124] | 16/16 | 19.8 | 39.6 | 1.9 | 4.1 | 1.2 | 0.0 | — | — |
| inplace_sparse shared: stage | 82.04 | 81.33–82.74 | 31.22 | 30.47–31.85 | 2.625 | [2.611, 2.641] | 16/16 | 839.2 | 958.1 | 23.9 | 185.2 | 27.7 | 27.8 | — | — |
| inplace_sparse shared: commit | 1.06 | 1.03–1.08 | 1.03 | 1.02–1.07 | 1.019 | [1.011, 1.027] | 15/16 | 5.2 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| inplace_sparse shared: read_published | 4.94 | 4.82–5.11 | 4.89 | 4.80–5.04 | 1.010 | [1.003, 1.017] | 10/16 | 305.0 | 305.2 | 37.5 | 37.7 | 27.8 | 0.0 | — | — |
| inplace_sparse chain: whole cycle | 165 | 163–167 | 112 | 111–113 | 1.473 | [1.470, 1.477] | 16/16 | 2796.6 | 2932.4 | 224.6 | 224.3 | 136.9 | 78.4 | 4.1 | 4.0 |
| inplace_sparse chain: update | 59.41 | 58.71–60.21 | 59.34 | 58.74–59.95 | 0.999 | [0.997, 1.001] | 6/16 | 1327.6 | 1327.6 | 224.5 | 224.3 | 53.1 | 50.8 | — | — |
| inplace_sparse chain: read_pending | 7.91 | 7.75–8.14 | 4.86 | 4.71–4.99 | 1.632 | [1.619, 1.645] | 16/16 | 309.0 | 303.7 | 39.9 | 36.3 | 27.5 | 0.0 | — | — |
| inplace_sparse chain: locate | 12.04 | 11.88–12.26 | 11.04 | 10.15–11.52 | 1.098 | [1.083, 1.118] | 16/16 | 19.8 | 39.6 | 1.9 | 4.1 | 1.2 | 0.0 | — | — |
| inplace_sparse chain: stage | 79.49 | 78.81–80.76 | 30.81 | 30.43–31.29 | 2.584 | [2.571, 2.596] | 16/16 | 832.7 | 954.3 | 24.0 | 186.8 | 27.4 | 27.6 | — | — |
| inplace_sparse chain: commit | 1.06 | 1.05–1.09 | 1.03 | 1.02–1.06 | 1.031 | [1.026, 1.037] | 16/16 | 5.2 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| inplace_sparse chain: read_published | 4.90 | 4.78–5.06 | 4.87 | 4.71–5.00 | 1.005 | [0.997, 1.012] | 9/16 | 302.4 | 302.4 | 37.2 | 37.5 | 27.6 | 0.0 | — | — |
| inplace_sparse plain one: RewriteColumns / RewriteRows | 101 | 99.93–103 | 101 | 100–103 | 0.997 | [0.994, 1.000] | 5/16 | 2228.9 | 2876.0 | 224.0 | 224.4 | 102.7 | 66.9 | 3.9 | 3.2 |
| inplace_sparse plain one: RewriteColumns / RewriteRows: merge | 23.94 | 23.60–24.26 | 17.50 | 17.01–18.11 | 1.362 | [1.352, 1.373] | 16/16 | 551.5 | 1177.6 | 102.6 | 129.4 | 18.5 | 16.1 | — | — |
| inplace_sparse plain one: RewriteColumns / RewriteRows: commit | 1.03 | 0.997–1.06 | 1.03 | 0.995–1.04 | 1.001 | [0.993, 1.009] | 7/16 | 5.2 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| inplace_sparse plain one: RewriteColumns / RewriteRows: read_published | 2.94 | 2.88–3.05 | 9.28 | 9.11–9.69 | 0.316 | [0.313, 0.318] | 0/16 | 151.7 | 173.1 | 18.7 | 19.6 | 13.8 | 0.0 | — | — |

## Whole-fragment replacement

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| one: in-place stage / moving stage | 65.55 | 64.51–66.39 | 0.560 | 0.538–0.593 | 116.988 | [115.713, 118.233] | 16/16 | 433.0 | 0.3 | 12.9 | 0.1 | 13.7 | 13.8 | — | — |
| shared: in-place stage / moving stage | 82.04 | 81.33–82.74 | 0.602 | 0.582–0.670 | 134.960 | [132.923, 136.650] | 16/16 | 839.2 | 0.4 | 23.9 | 0.1 | 27.7 | 27.8 | — | — |
| chain: in-place stage / moving stage | 79.49 | 78.81–80.76 | 0.606 | 0.589–0.659 | 130.517 | [128.832, 131.981] | 16/16 | 832.7 | 0.4 | 24.0 | 0.1 | 27.4 | 27.6 | — | — |
| one: in-place stage (rows copied) / backfill stage (rows computed) | 65.55 | 64.51–66.39 | 186 | 185–187 | 0.353 | [0.352, 0.354] | 0/16 | 433.0 | 275.3 | 12.9 | 10.3 | 13.7 | 13.8 | — | — |
| shared: in-place stage (rows copied) / backfill stage (rows computed) | 82.04 | 81.33–82.74 | 323 | 321–325 | 0.254 | [0.253, 0.254] | 0/16 | 839.2 | 529.8 | 23.9 | 18.5 | 27.7 | 27.8 | — | — |
| chain: in-place stage (rows copied) / backfill stage (rows computed) | 79.49 | 78.81–80.76 | 325 | 322–326 | 0.245 | [0.244, 0.245] | 0/16 | 832.7 | 528.2 | 24.0 | 18.5 | 27.4 | 27.6 | — | — |

## Backfills: publication / plain DataReplacement

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| backfill one: whole | 199 | 198–201 | 67.34 | 66.35–68.31 | 2.961 | [2.953, 2.970] | 16/16 | 1237.4 | 1201.0 | 201.5 | 201.3 | 80.1 | 13.8 | 2.1 | 2.0 |
| backfill one: locate | 9.56 | 9.34–9.91 | 9.43 | 9.26–9.65 | 1.015 | [1.007, 1.023] | 14/16 | 805.4 | 804.9 | 172.6 | 169.4 | 66.2 | 0.0 | — | — |
| backfill one: stage | 186 | 185–187 | 53.76 | 52.83–54.38 | 3.456 | [3.445, 3.468] | 16/16 | 275.3 | 240.5 | 10.3 | 10.3 | 0.0 | 13.8 | — | — |
| backfill one: commit | 0.990 | 0.976–1.03 | 0.967 | 0.951–1.01 | 1.023 | [1.015, 1.033] | 14/16 | 5.2 | 5.2 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |
| backfill shared: whole | 339 | 337–340 | 77.18 | 76.11–78.29 | 4.388 | [4.373, 4.402] | 16/16 | 1645.0 | 1585.5 | 236.5 | 236.2 | 94.1 | 27.8 | 2.3 | 2.0 |
| backfill shared: locate | 9.51 | 9.32–9.71 | 9.47 | 9.33–9.66 | 1.007 | [0.999, 1.016] | 11/16 | 805.5 | 804.9 | 171.3 | 171.0 | 66.3 | 0.0 | — | — |
| backfill shared: stage | 323 | 321–325 | 61.65 | 60.66–62.92 | 5.240 | [5.219, 5.261] | 16/16 | 529.8 | 470.8 | 18.5 | 18.4 | 0.0 | 27.8 | — | — |
| backfill shared: commit | 1.01 | 1.00–1.04 | 0.980 | 0.963–1.01 | 1.036 | [1.029, 1.043] | 16/16 | 5.2 | 5.2 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |
| backfill chain: whole | 341 | 338–342 | 76.37 | 75.63–77.30 | 4.456 | [4.445, 4.467] | 16/16 | 1641.4 | 1580.8 | 236.6 | 236.2 | 93.9 | 27.6 | 2.3 | 2.0 |
| backfill chain: locate | 9.46 | 9.24–9.73 | 9.73 | 9.36–9.90 | 0.976 | [0.970, 0.982] | 0/16 | 806.2 | 804.9 | 171.1 | 169.1 | 66.3 | 0.0 | — | — |
| backfill chain: stage | 325 | 322–326 | 60.48 | 59.93–61.14 | 5.371 | [5.354, 5.388] | 16/16 | 528.2 | 469.0 | 18.5 | 18.4 | 0.0 | 27.6 | — | — |
| backfill chain: commit | 1.01 | 0.999–1.05 | 0.983 | 0.966–1.00 | 1.033 | [1.026, 1.041] | 16/16 | 5.2 | 5.2 | 5.0 | 5.0 | 0.0 | 0.0 | — | — |

## Conflicts

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| moving_k1: reject publish | 0.132 | 0.122–0.150 | 19.09 | 18.66–19.62 | 0.007 | [0.007, 0.007] | 0/16 | 0.1 | 11.5 | 0.0 | 0.3 | 0.0 | 0.0 | — | — |
| moving_k1: reject extra | 20.23 | 19.86–20.84 | 19.09 | 18.66–19.62 | 1.057 | [1.054, 1.060] | 16/16 | 17.4 | 11.5 | 5.1 | 0.3 | 0.1 | 0.0 | — | — |
| moving_k1: skip publish | 0.977 | 0.935–1.02 | 19.29 | 18.94–19.84 | 0.051 | [0.050, 0.051] | 0/16 | 5.2 | 11.5 | 5.1 | 0.3 | 0.0 | 0.0 | — | — |
| moving_k1: skip extra | 19.46 | 19.20–19.93 | 19.29 | 18.94–19.84 | 1.011 | [1.008, 1.014] | 16/16 | 15.4 | 11.5 | 5.1 | 0.3 | 0.0 | 0.0 | — | — |
| inplace_k1: reject publish | 0.141 | 0.134–0.155 | 90.78 | 89.73–92.22 | 0.002 | [0.002, 0.002] | 0/16 | 0.1 | 548.5 | 0.0 | 15.2 | 0.0 | 0.0 | — | — |
| inplace_k1: reject extra | 90.94 | 89.66–92.16 | 90.78 | 89.73–92.22 | 1.000 | [0.997, 1.003] | 8/16 | 553.3 | 548.5 | 15.1 | 15.2 | 23.1 | 13.8 | — | — |
| inplace_k1: skip publish | 1.01 | 0.973–1.09 | 90.85 | 89.92–92.72 | 0.011 | [0.011, 0.011] | 0/16 | 5.3 | 548.5 | 5.1 | 15.2 | 0.0 | 0.0 | — | — |
| inplace_k1: skip extra | 57.85 | 56.78–58.70 | 90.85 | 89.92–92.72 | 0.636 | [0.634, 0.638] | 0/16 | 379.4 | 548.5 | 13.0 | 15.2 | 12.7 | 12.5 | — | — |
| inplace_k8: reject publish | 0.320 | 0.312–0.335 | 90.70 | 89.27–91.90 | 0.004 | [0.003, 0.004] | 0/16 | 0.1 | 548.5 | 0.0 | 15.2 | 0.1 | 0.0 | — | — |
| inplace_k8: reject extra | 91.42 | 89.98–93.17 | 90.70 | 89.27–91.90 | 1.008 | [1.006, 1.010] | 16/16 | 553.0 | 548.5 | 15.1 | 15.2 | 23.1 | 13.8 | — | — |
| inplace_k8: skip publish | 1.27 | 1.20–1.32 | 91.06 | 89.67–92.36 | 0.014 | [0.014, 0.014] | 0/16 | 5.5 | 548.5 | 5.1 | 15.2 | 0.1 | 0.0 | — | — |
| inplace_k8: skip extra | 65.91 | 64.75–67.34 | 91.06 | 89.67–92.36 | 0.725 | [0.723, 0.727] | 0/16 | 430.3 | 548.5 | 13.0 | 15.2 | 14.7 | 13.8 | — | — |
| inplace_k1_chain: reject publish | 0.146 | 0.137–0.160 | 115 | 113–116 | 0.001 | [0.001, 0.001] | 0/16 | 0.1 | 965.4 | 0.0 | 24.1 | 0.0 | 0.0 | — | — |
| inplace_k1_chain: reject extra | 115 | 114–116 | 115 | 113–116 | 1.000 | [0.997, 1.003] | 7/16 | 967.8 | 965.4 | 24.0 | 24.1 | 36.8 | 27.6 | — | — |
| inplace_k1_chain: skip publish | 1.09 | 1.03–1.16 | 115 | 114–117 | 0.009 | [0.009, 0.010] | 0/16 | 5.3 | 965.4 | 5.1 | 24.1 | 0.0 | 0.0 | — | — |
| inplace_k1_chain: skip extra | 68.10 | 67.54–68.87 | 115 | 114–117 | 0.592 | [0.591, 0.594] | 0/16 | 723.4 | 965.4 | 24.1 | 24.1 | 25.1 | 24.9 | — | — |

## Publication after unrelated appends

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| publish after 8 appends: one | 1.13 | 1.10–1.14 | 1.01 | 0.989–1.03 | 1.111 | [1.104, 1.118] | 16/16 | 5.3 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| publish after 8 appends: plain_one | 1.12 | 1.08–1.15 | 1.01 | 0.985–1.05 | 1.112 | [1.101, 1.122] | 16/16 | 5.3 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| publish after 32 appends: one | 1.49 | 1.46–1.56 | 1.01 | 0.989–1.03 | 1.480 | [1.468, 1.493] | 16/16 | 5.6 | 5.2 | 5.1 | 5.1 | 0.2 | 0.0 | — | — |
| publish after 32 appends: plain_one | 1.50 | 1.44–1.54 | 1.01 | 0.985–1.05 | 1.485 | [1.469, 1.502] | 16/16 | 5.6 | 5.2 | 5.1 | 5.1 | 0.1 | 0.0 | — | — |
| publish after 0 appends: one / plain | 1.01 | 0.989–1.03 | 1.01 | 0.985–1.05 | 1.007 | [0.997, 1.017] | 11/16 | 5.2 | 5.2 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| publish after 8 appends: one / plain | 1.13 | 1.10–1.14 | 1.12 | 1.08–1.15 | 1.007 | [0.996, 1.017] | 10/16 | 5.3 | 5.3 | 5.1 | 5.1 | 0.0 | 0.0 | — | — |
| publish after 32 appends: one / plain | 1.49 | 1.46–1.56 | 1.50 | 1.44–1.54 | 1.004 | [0.990, 1.020] | 8/16 | 5.6 | 5.6 | 5.1 | 5.1 | 0.2 | 0.0 | — | — |

## Accumulated history

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| reopen one / plain | 0.160 | 0.153–0.170 | 0.178 | 0.165–0.188 | 0.930 | [0.911, 0.948] | 0/16 | 0.1 | 0.1 | 0.0 | 0.0 | — | — | 2.2 | 2.1 |
| reopen shared / plain | 0.163 | 0.153–0.174 | 0.178 | 0.165–0.188 | 0.955 | [0.941, 0.970] | 2/16 | 0.1 | 0.1 | 0.0 | 0.0 | — | — | 2.3 | 2.1 |
| reopen chain / plain | 0.163 | 0.153–0.179 | 0.178 | 0.165–0.188 | 0.956 | [0.933, 0.981] | 3/16 | 0.1 | 0.1 | 0.0 | 0.0 | — | — | 2.3 | 2.1 |
| reopen one_h16 / plain_h16 | 0.217 | 0.205–0.229 | 0.164 | 0.152–0.179 | 1.312 | [1.282, 1.344] | 16/16 | 0.1 | 0.1 | 0.1 | 0.0 | — | — | 10.3 | 3.8 |
| reopen one_h64 / plain_h64 | 0.279 | 0.269–0.297 | 0.248 | 0.237–0.261 | 1.133 | [1.116, 1.149] | 16/16 | 0.3 | 0.2 | 0.1 | 0.1 | — | — | 33.5 | 8.4 |
| reopen shared_h64 / plain_h64 | 0.286 | 0.272–0.309 | 0.248 | 0.237–0.261 | 1.163 | [1.149, 1.178] | 16/16 | 0.5 | 0.2 | 0.2 | 0.1 | — | — | 58.7 | 8.4 |
| reopen chain_h64 / plain_h64 | 0.287 | 0.268–0.306 | 0.248 | 0.237–0.261 | 1.156 | [1.137, 1.174] | 16/16 | 0.5 | 0.2 | 0.2 | 0.1 | — | — | 58.7 | 8.4 |
| reopen one_dense / plain_dense | 0.211 | 0.197–0.218 | 0.173 | 0.152–0.189 | 1.273 | [1.232, 1.314] | 16/16 | 1.3 | 0.1 | 0.4 | 0.0 | — | — | 162.7 | 2.3 |
| reopen one_h16r / plain_h16r | 0.222 | 0.212–0.231 | 0.203 | 0.195–0.217 | 1.088 | [1.076, 1.099] | 16/16 | 0.1 | 0.1 | 0.1 | 0.0 | — | — | 11.8 | 5.2 |
| reopen one_h64r / plain_h64r | 0.326 | 0.301–0.333 | 0.281 | 0.266–0.298 | 1.148 | [1.131, 1.164] | 16/16 | 0.4 | 0.2 | 0.2 | 0.1 | — | — | 39.5 | 14.2 |
| reopen one_h64r / one | 0.326 | 0.301–0.333 | 0.160 | 0.153–0.170 | 2.021 | [1.986, 2.053] | 16/16 | 0.4 | 0.1 | 0.2 | 0.0 | — | — | 39.5 | 2.2 |
| reopen plain_h64r / plain | 0.281 | 0.266–0.298 | 0.178 | 0.165–0.188 | 1.637 | [1.608, 1.670] | 16/16 | 0.2 | 0.1 | 0.1 | 0.0 | — | — | 14.2 | 2.1 |
| reopen plain_h16 / plain | 0.164 | 0.152–0.179 | 0.178 | 0.165–0.188 | 0.956 | [0.933, 0.977] | 2/16 | 0.1 | 0.1 | 0.0 | 0.0 | — | — | 3.8 | 2.1 |
| reopen plain_h64 / plain | 0.248 | 0.237–0.261 | 0.178 | 0.165–0.188 | 1.441 | [1.411, 1.470] | 16/16 | 0.2 | 0.1 | 0.1 | 0.0 | — | — | 8.4 | 2.1 |
| reopen one_h16 / one | 0.217 | 0.205–0.229 | 0.160 | 0.153–0.170 | 1.349 | [1.326, 1.372] | 16/16 | 0.1 | 0.1 | 0.1 | 0.0 | — | — | 10.3 | 2.2 |
| reopen one_h64 / one | 0.279 | 0.269–0.297 | 0.160 | 0.153–0.170 | 1.755 | [1.734, 1.779] | 16/16 | 0.3 | 0.1 | 0.1 | 0.0 | — | — | 33.5 | 2.2 |
| append one / plain | 1.38 | 1.34–1.41 | 1.38 | 1.34–1.43 | 0.995 | [0.988, 1.003] | 7/16 | 5.3 | 5.3 | 5.0 | 5.0 | 0.0 | 0.0 | 2.3 | 2.1 |
| append shared / plain | 1.39 | 1.34–1.41 | 1.38 | 1.34–1.43 | 1.001 | [0.992, 1.009] | 10/16 | 5.3 | 5.3 | 5.0 | 5.0 | 0.0 | 0.0 | 2.4 | 2.1 |
| append chain / plain | 1.35 | 1.33–1.39 | 1.38 | 1.34–1.43 | 0.977 | [0.968, 0.987] | 2/16 | 5.3 | 5.3 | 5.0 | 5.0 | 0.0 | 0.0 | 2.4 | 2.1 |
| append one_h16 / plain_h16 | 1.42 | 1.38–1.45 | 1.38 | 1.34–1.43 | 1.029 | [1.021, 1.037] | 15/16 | 5.4 | 5.3 | 5.1 | 5.1 | 0.0 | 0.0 | 10.4 | 3.9 |
| append one_h64 / plain_h64 | 1.43 | 1.39–1.45 | 1.43 | 1.40–1.46 | 1.001 | [0.993, 1.008] | 11/16 | 5.5 | 5.4 | 5.2 | 5.1 | 0.0 | 0.0 | 33.6 | 8.4 |
| append shared_h64 / plain_h64 | 1.45 | 1.42–1.52 | 1.43 | 1.40–1.46 | 1.023 | [1.012, 1.034] | 13/16 | 5.5 | 5.4 | 5.2 | 5.1 | 0.0 | 0.1 | 58.8 | 8.4 |
| append chain_h64 / plain_h64 | 1.44 | 1.41–1.50 | 1.43 | 1.40–1.46 | 1.013 | [1.004, 1.022] | 13/16 | 5.5 | 5.4 | 5.2 | 5.1 | 0.0 | 0.1 | 58.8 | 8.4 |
| append one_dense / plain_dense | 1.41 | 1.37–1.44 | 1.38 | 1.35–1.42 | 1.019 | [1.010, 1.029] | 13/16 | 5.6 | 5.3 | 5.4 | 5.0 | 0.0 | 0.2 | 162.8 | 2.4 |
| append one_h16r / plain_h16r | 1.47 | 1.42–1.54 | 1.40 | 1.36–1.44 | 1.046 | [1.031, 1.061] | 14/16 | 5.4 | 5.3 | 5.1 | 5.1 | 0.0 | 0.0 | 11.9 | 5.3 |
| append one_h64r / plain_h64r | 1.44 | 1.41–1.47 | 1.50 | 1.45–1.56 | 0.963 | [0.953, 0.974] | 1/16 | 5.5 | 5.4 | 5.2 | 5.1 | 0.0 | 0.1 | 39.6 | 14.3 |
| append one_h64r / one | 1.44 | 1.41–1.47 | 1.38 | 1.34–1.41 | 1.050 | [1.043, 1.057] | 16/16 | 5.5 | 5.3 | 5.2 | 5.0 | 0.0 | 0.1 | 39.6 | 2.3 |
| append plain_h64r / plain | 1.50 | 1.45–1.56 | 1.38 | 1.34–1.43 | 1.085 | [1.071, 1.098] | 16/16 | 5.4 | 5.3 | 5.1 | 5.0 | 0.0 | 0.0 | 14.3 | 2.1 |
| append plain_h16 / plain | 1.38 | 1.34–1.43 | 1.38 | 1.34–1.43 | 0.997 | [0.989, 1.004] | 6/16 | 5.3 | 5.3 | 5.1 | 5.0 | 0.0 | 0.0 | 3.9 | 2.1 |
| append plain_h64 / plain | 1.43 | 1.40–1.46 | 1.38 | 1.34–1.43 | 1.030 | [1.018, 1.042] | 14/16 | 5.4 | 5.3 | 5.1 | 5.0 | 0.0 | 0.0 | 8.4 | 2.1 |
| append one_h16 / one | 1.42 | 1.38–1.45 | 1.38 | 1.34–1.41 | 1.030 | [1.023, 1.037] | 15/16 | 5.4 | 5.3 | 5.1 | 5.0 | 0.0 | 0.0 | 10.4 | 2.3 |
| append one_h64 / one | 1.43 | 1.39–1.45 | 1.38 | 1.34–1.41 | 1.035 | [1.028, 1.042] | 16/16 | 5.5 | 5.3 | 5.2 | 5.0 | 0.0 | 0.0 | 33.6 | 2.3 |
| update one / plain | 16.62 | 15.73–17.00 | 16.56 | 15.74–16.99 | 1.000 | [0.994, 1.006] | 7/16 | 77.1 | 77.1 | 7.6 | 7.6 | 6.9 | 0.0 | 3.1 | 2.3 |
| update shared / plain | 16.65 | 15.78–17.05 | 16.56 | 15.74–16.99 | 1.002 | [0.997, 1.009] | 7/16 | 77.1 | 77.1 | 7.5 | 7.6 | 6.9 | 0.0 | 3.8 | 2.3 |
| update chain / plain | 16.67 | 15.62–17.02 | 16.56 | 15.74–16.99 | 1.003 | [0.997, 1.011] | 7/16 | 77.1 | 77.1 | 7.6 | 7.6 | 6.9 | 0.0 | 3.8 | 2.3 |
| update one_h16 / plain_h16 | 26.25 | 25.54–26.68 | 26.09 | 24.94–26.60 | 1.007 | [1.001, 1.013] | 10/16 | 86.0 | 86.0 | 8.5 | 8.6 | 7.0 | 0.0 | 10.8 | 3.9 |
| update one_h64 / plain_h64 | 29.49 | 28.23–30.45 | 29.45 | 28.79–30.04 | 1.001 | [0.994, 1.007] | 11/16 | 103.1 | 102.8 | 10.5 | 10.4 | 7.2 | 0.1 | 34.0 | 8.5 |
| update shared_h64 / plain_h64 | 29.47 | 28.58–29.92 | 29.45 | 28.79–30.04 | 1.000 | [0.996, 1.004] | 10/16 | 103.2 | 102.8 | 10.6 | 10.4 | 7.2 | 0.1 | 59.5 | 8.5 |
| update chain_h64 / plain_h64 | 29.42 | 28.59–29.83 | 29.45 | 28.79–30.04 | 0.998 | [0.995, 1.001] | 5/16 | 103.2 | 102.8 | 10.5 | 10.4 | 7.2 | 0.1 | 59.5 | 8.5 |
| update one_dense / plain_dense | 26.23 | 25.55–26.96 | 26.08 | 25.37–27.02 | 1.003 | [0.999, 1.007] | 9/16 | 200.7 | 199.9 | 27.2 | 27.1 | 7.5 | 0.3 | 162.8 | 2.4 |
| update one_h16r / plain_h16r | 26.42 | 25.32–26.82 | 26.33 | 25.48–26.70 | 1.002 | [0.997, 1.007] | 10/16 | 86.2 | 86.2 | 8.2 | 8.6 | 7.0 | 0.0 | 12.3 | 5.3 |
| update one_h64r / plain_h64r | 29.41 | 28.37–30.19 | 30.67 | 29.76–31.30 | 0.961 | [0.957, 0.965] | 0/16 | 103.6 | 103.6 | 10.3 | 10.5 | 7.2 | 0.1 | 40.0 | 14.3 |
| update one_h64r / one | 29.41 | 28.37–30.19 | 16.62 | 15.73–17.00 | 1.780 | [1.768, 1.793] | 16/16 | 103.6 | 77.1 | 10.3 | 7.6 | 7.2 | 0.1 | 40.0 | 3.1 |
| update plain_h64r / plain | 30.67 | 29.76–31.30 | 16.56 | 15.74–16.99 | 1.853 | [1.842, 1.865] | 16/16 | 103.6 | 77.1 | 10.5 | 7.6 | 7.5 | 0.1 | 14.3 | 2.3 |
| update plain_h16 / plain | 26.09 | 24.94–26.60 | 16.56 | 15.74–16.99 | 1.575 | [1.566, 1.584] | 16/16 | 86.0 | 77.1 | 8.6 | 7.6 | 7.0 | 0.0 | 3.9 | 2.3 |
| update plain_h64 / plain | 29.45 | 28.79–30.04 | 16.56 | 15.74–16.99 | 1.781 | [1.770, 1.793] | 16/16 | 102.8 | 77.1 | 10.4 | 7.6 | 7.2 | 0.1 | 8.5 | 2.3 |
| update one_h16 / one | 26.25 | 25.54–26.68 | 16.62 | 15.73–17.00 | 1.586 | [1.574, 1.600] | 16/16 | 86.0 | 77.1 | 8.5 | 7.6 | 7.0 | 0.0 | 10.8 | 3.1 |
| update one_h64 / one | 29.49 | 28.23–30.45 | 16.62 | 15.73–17.00 | 1.782 | [1.767, 1.800] | 16/16 | 103.1 | 77.1 | 10.5 | 7.6 | 7.2 | 0.1 | 34.0 | 3.1 |

## Control: <build>-copy / <build>, per workload

| workload | ratio | 95% interval |
|---|---|---|
| after_appends_k0_one | 1.002 | [0.999, 1.005] |
| after_appends_k0_one.commit | 0.997 | [0.987, 1.007] |
| after_appends_k0_one.find | 0.997 | [0.991, 1.002] |
| after_appends_k0_one.first | 1.002 | [0.999, 1.005] |
| after_appends_k0_one.locate | 0.998 | [0.990, 1.007] |
| after_appends_k0_one.read_inputs | 0.999 | [0.981, 1.018] |
| after_appends_k0_one.stage | 1.002 | [0.998, 1.006] |
| after_appends_k0_plain_one | 1.008 | [0.998, 1.020] |
| after_appends_k0_plain_one.commit | 0.999 | [0.984, 1.017] |
| after_appends_k0_plain_one.locate | 1.028 | [0.997, 1.061] |
| after_appends_k0_plain_one.merge | 1.001 | [0.995, 1.009] |
| after_appends_k32_one | 1.000 | [0.997, 1.004] |
| after_appends_k32_one.commit | 1.005 | [0.983, 1.030] |
| after_appends_k32_one.find | 0.994 | [0.989, 0.998] |
| after_appends_k32_one.first | 1.000 | [0.997, 1.004] |
| after_appends_k32_one.locate | 0.996 | [0.989, 1.003] |
| after_appends_k32_one.read_inputs | 0.993 | [0.976, 1.008] |
| after_appends_k32_one.stage | 1.000 | [0.996, 1.005] |
| after_appends_k32_plain_one | 1.011 | [1.001, 1.023] |
| after_appends_k32_plain_one.commit | 0.994 | [0.976, 1.014] |
| after_appends_k32_plain_one.locate | 1.040 | [1.013, 1.070] |
| after_appends_k32_plain_one.merge | 0.999 | [0.991, 1.008] |
| after_appends_k8_one | 0.998 | [0.993, 1.002] |
| after_appends_k8_one.commit | 1.010 | [1.000, 1.019] |
| after_appends_k8_one.find | 0.993 | [0.987, 0.999] |
| after_appends_k8_one.first | 0.998 | [0.993, 1.001] |
| after_appends_k8_one.locate | 0.992 | [0.978, 1.007] |
| after_appends_k8_one.read_inputs | 0.986 | [0.966, 1.008] |
| after_appends_k8_one.stage | 0.997 | [0.993, 1.002] |
| after_appends_k8_plain_one | 1.008 | [0.998, 1.020] |
| after_appends_k8_plain_one.commit | 0.992 | [0.979, 1.008] |
| after_appends_k8_plain_one.locate | 1.027 | [0.997, 1.059] |
| after_appends_k8_plain_one.merge | 0.998 | [0.989, 1.008] |
| append_chain | 1.004 | [0.994, 1.017] |
| append_chain.commit | 1.006 | [0.993, 1.022] |
| append_chain.first | 1.004 | [0.997, 1.012] |
| append_chain.stage | 1.004 | [0.997, 1.012] |
| append_chain_h64 | 1.007 | [1.000, 1.017] |
| append_chain_h64.commit | 1.006 | [0.998, 1.014] |
| append_chain_h64.first | 1.007 | [0.989, 1.027] |
| append_chain_h64.stage | 1.007 | [0.989, 1.027] |
| append_one | 1.004 | [0.993, 1.014] |
| append_one.commit | 1.009 | [1.001, 1.017] |
| append_one.first | 0.991 | [0.974, 1.009] |
| append_one.stage | 0.991 | [0.974, 1.009] |
| append_one_dense | 1.011 | [1.001, 1.022] |
| append_one_dense.commit | 1.009 | [0.998, 1.021] |
| append_one_dense.first | 1.001 | [0.990, 1.012] |
| append_one_dense.stage | 1.001 | [0.990, 1.012] |
| append_one_h16 | 1.001 | [0.992, 1.009] |
| append_one_h16.commit | 0.999 | [0.988, 1.011] |
| append_one_h16.first | 1.003 | [0.983, 1.022] |
| append_one_h16.stage | 1.003 | [0.983, 1.022] |
| append_one_h16r | 0.994 | [0.973, 1.015] |
| append_one_h16r.commit | 0.991 | [0.961, 1.020] |
| append_one_h16r.first | 1.001 | [0.983, 1.020] |
| append_one_h16r.stage | 1.001 | [0.983, 1.020] |
| append_one_h64 | 1.008 | [1.001, 1.016] |
| append_one_h64.commit | 1.007 | [0.994, 1.021] |
| append_one_h64.first | 1.012 | [1.004, 1.019] |
| append_one_h64.stage | 1.012 | [1.004, 1.019] |
| append_one_h64r | 0.994 | [0.984, 1.005] |
| append_one_h64r.commit | 0.989 | [0.978, 1.000] |
| append_one_h64r.first | 1.002 | [0.990, 1.016] |
| append_one_h64r.stage | 1.002 | [0.990, 1.016] |
| append_plain | 1.003 | [0.993, 1.014] |
| append_plain.commit | 1.002 | [0.989, 1.013] |
| append_plain.first | 1.002 | [0.979, 1.024] |
| append_plain.stage | 1.002 | [0.979, 1.024] |
| append_plain_dense | 0.997 | [0.989, 1.005] |
| append_plain_dense.commit | 0.996 | [0.983, 1.009] |
| append_plain_dense.first | 1.004 | [0.997, 1.013] |
| append_plain_dense.stage | 1.004 | [0.997, 1.013] |
| append_plain_h16 | 1.007 | [0.998, 1.015] |
| append_plain_h16.commit | 1.003 | [0.991, 1.016] |
| append_plain_h16.first | 1.008 | [0.996, 1.020] |
| append_plain_h16.stage | 1.008 | [0.996, 1.020] |
| append_plain_h16r | 1.007 | [0.998, 1.014] |
| append_plain_h16r.commit | 1.004 | [0.993, 1.018] |
| append_plain_h16r.first | 1.009 | [0.987, 1.024] |
| append_plain_h16r.stage | 1.009 | [0.987, 1.024] |
| append_plain_h64 | 0.996 | [0.988, 1.004] |
| append_plain_h64.commit | 0.994 | [0.983, 1.004] |
| append_plain_h64.first | 1.006 | [0.979, 1.028] |
| append_plain_h64.stage | 1.006 | [0.979, 1.028] |
| append_plain_h64r | 0.986 | [0.967, 1.005] |
| append_plain_h64r.commit | 0.989 | [0.963, 1.017] |
| append_plain_h64r.first | 1.003 | [0.993, 1.012] |
| append_plain_h64r.stage | 1.003 | [0.993, 1.012] |
| append_shared | 1.007 | [1.000, 1.013] |
| append_shared.commit | 1.007 | [0.999, 1.014] |
| append_shared.first | 1.008 | [1.000, 1.017] |
| append_shared.stage | 1.008 | [1.000, 1.017] |
| append_shared_h64 | 1.001 | [0.990, 1.011] |
| append_shared_h64.commit | 0.999 | [0.990, 1.008] |
| append_shared_h64.first | 0.987 | [0.973, 1.004] |
| append_shared_h64.stage | 0.987 | [0.973, 1.004] |
| backfill_chain | 0.999 | [0.998, 1.001] |
| backfill_chain.commit | 1.000 | [0.990, 1.011] |
| backfill_chain.first | 0.999 | [0.997, 1.001] |
| backfill_chain.locate | 0.993 | [0.981, 1.008] |
| backfill_chain.read_inputs | 0.993 | [0.981, 1.008] |
| backfill_chain.read_published | 0.999 | [0.987, 1.014] |
| backfill_chain.stage | 0.999 | [0.997, 1.001] |
| backfill_one | 1.000 | [0.998, 1.002] |
| backfill_one.commit | 1.003 | [0.992, 1.011] |
| backfill_one.first | 1.000 | [0.998, 1.002] |
| backfill_one.locate | 1.006 | [0.992, 1.025] |
| backfill_one.read_inputs | 1.006 | [0.992, 1.025] |
| backfill_one.read_published | 0.998 | [0.985, 1.013] |
| backfill_one.stage | 0.999 | [0.997, 1.001] |
| backfill_plain_chain | 1.003 | [0.999, 1.007] |
| backfill_plain_chain.commit | 1.003 | [0.991, 1.014] |
| backfill_plain_chain.first | 1.002 | [0.998, 1.006] |
| backfill_plain_chain.locate | 0.993 | [0.979, 1.010] |
| backfill_plain_chain.read_inputs | 0.993 | [0.979, 1.010] |
| backfill_plain_chain.read_published | 1.006 | [0.996, 1.017] |
| backfill_plain_chain.stage | 1.003 | [0.997, 1.009] |
| backfill_plain_one | 1.001 | [0.996, 1.005] |
| backfill_plain_one.commit | 0.999 | [0.991, 1.007] |
| backfill_plain_one.first | 1.000 | [0.996, 1.005] |
| backfill_plain_one.locate | 0.998 | [0.984, 1.010] |
| backfill_plain_one.read_inputs | 0.998 | [0.984, 1.010] |
| backfill_plain_one.read_published | 1.000 | [0.993, 1.008] |
| backfill_plain_one.stage | 1.004 | [1.000, 1.007] |
| backfill_plain_shared | 1.004 | [0.998, 1.010] |
| backfill_plain_shared.commit | 0.992 | [0.983, 1.000] |
| backfill_plain_shared.first | 1.004 | [0.997, 1.010] |
| backfill_plain_shared.locate | 0.987 | [0.977, 1.000] |
| backfill_plain_shared.read_inputs | 0.987 | [0.977, 1.000] |
| backfill_plain_shared.read_published | 1.002 | [0.992, 1.012] |
| backfill_plain_shared.stage | 1.006 | [0.998, 1.013] |
| backfill_shared | 1.000 | [0.999, 1.001] |
| backfill_shared.commit | 0.992 | [0.979, 1.006] |
| backfill_shared.first | 1.000 | [0.999, 1.001] |
| backfill_shared.locate | 0.997 | [0.982, 1.011] |
| backfill_shared.read_inputs | 0.997 | [0.982, 1.011] |
| backfill_shared.read_published | 1.006 | [0.997, 1.014] |
| backfill_shared.stage | 1.000 | [0.999, 1.002] |
| conflict_inplace_k1_reject | 1.000 | [0.995, 1.004] |
| conflict_inplace_k1_reject.extra | 0.999 | [0.996, 1.002] |
| conflict_inplace_k1_reject.find | 0.996 | [0.992, 1.000] |
| conflict_inplace_k1_reject.first | 1.002 | [0.996, 1.008] |
| conflict_inplace_k1_reject.locate | 0.995 | [0.981, 1.005] |
| conflict_inplace_k1_reject.publish | 1.034 | [1.000, 1.065] |
| conflict_inplace_k1_reject.read_inputs | 0.995 | [0.976, 1.010] |
| conflict_inplace_k1_reject.retry_checkout | 1.007 | [0.982, 1.034] |
| conflict_inplace_k1_reject.retry_commit | 1.003 | [0.987, 1.018] |
| conflict_inplace_k1_reject.retry_find | 0.999 | [0.993, 1.005] |
| conflict_inplace_k1_reject.retry_read_inputs | 0.991 | [0.980, 1.002] |
| conflict_inplace_k1_reject.retry_stage | 1.002 | [1.000, 1.004] |
| conflict_inplace_k1_reject.stage | 1.004 | [1.000, 1.009] |
| conflict_inplace_k1_reject_chain | 1.000 | [0.995, 1.005] |
| conflict_inplace_k1_reject_chain.extra | 1.001 | [0.997, 1.006] |
| conflict_inplace_k1_reject_chain.find | 1.000 | [0.998, 1.002] |
| conflict_inplace_k1_reject_chain.first | 1.000 | [0.996, 1.004] |
| conflict_inplace_k1_reject_chain.locate | 0.998 | [0.981, 1.014] |
| conflict_inplace_k1_reject_chain.publish | 0.982 | [0.944, 1.020] |
| conflict_inplace_k1_reject_chain.read_inputs | 0.996 | [0.967, 1.024] |
| conflict_inplace_k1_reject_chain.retry_checkout | 0.965 | [0.940, 0.990] |
| conflict_inplace_k1_reject_chain.retry_commit | 0.999 | [0.983, 1.014] |
| conflict_inplace_k1_reject_chain.retry_find | 1.001 | [0.999, 1.003] |
| conflict_inplace_k1_reject_chain.retry_read_inputs | 0.986 | [0.963, 1.008] |
| conflict_inplace_k1_reject_chain.retry_stage | 1.002 | [0.999, 1.006] |
| conflict_inplace_k1_reject_chain.stage | 1.002 | [0.999, 1.005] |
| conflict_inplace_k1_skip | 0.999 | [0.994, 1.005] |
| conflict_inplace_k1_skip.extra | 1.000 | [0.994, 1.006] |
| conflict_inplace_k1_skip.find | 0.997 | [0.993, 1.003] |
| conflict_inplace_k1_skip.first | 1.002 | [0.997, 1.008] |
| conflict_inplace_k1_skip.follow_up_commit | 1.009 | [1.001, 1.017] |
| conflict_inplace_k1_skip.follow_up_plan | 1.006 | [0.996, 1.018] |
| conflict_inplace_k1_skip.follow_up_read_inputs | 0.991 | [0.978, 1.006] |
| conflict_inplace_k1_skip.follow_up_stage | 1.000 | [0.994, 1.006] |
| conflict_inplace_k1_skip.locate | 1.001 | [0.989, 1.012] |
| conflict_inplace_k1_skip.publish | 0.983 | [0.946, 1.022] |
| conflict_inplace_k1_skip.read_inputs | 1.001 | [0.987, 1.016] |
| conflict_inplace_k1_skip.stage | 1.002 | [0.996, 1.007] |
| conflict_inplace_k1_skip_chain | 0.999 | [0.996, 1.002] |
| conflict_inplace_k1_skip_chain.extra | 0.999 | [0.995, 1.003] |
| conflict_inplace_k1_skip_chain.find | 1.000 | [0.998, 1.002] |
| conflict_inplace_k1_skip_chain.first | 0.999 | [0.995, 1.003] |
| conflict_inplace_k1_skip_chain.follow_up_commit | 1.002 | [0.987, 1.017] |
| conflict_inplace_k1_skip_chain.follow_up_plan | 0.991 | [0.979, 1.001] |
| conflict_inplace_k1_skip_chain.follow_up_read_inputs | 1.025 | [1.007, 1.043] |
| conflict_inplace_k1_skip_chain.follow_up_stage | 1.000 | [0.997, 1.002] |
| conflict_inplace_k1_skip_chain.locate | 0.995 | [0.985, 1.003] |
| conflict_inplace_k1_skip_chain.publish | 0.990 | [0.963, 1.015] |
| conflict_inplace_k1_skip_chain.read_inputs | 0.992 | [0.977, 1.005] |
| conflict_inplace_k1_skip_chain.stage | 1.000 | [0.997, 1.004] |
| conflict_inplace_k8_reject | 0.999 | [0.995, 1.004] |
| conflict_inplace_k8_reject.extra | 1.000 | [0.995, 1.006] |
| conflict_inplace_k8_reject.find | 0.997 | [0.991, 1.002] |
| conflict_inplace_k8_reject.first | 1.001 | [0.995, 1.006] |
| conflict_inplace_k8_reject.locate | 0.994 | [0.980, 1.007] |
| conflict_inplace_k8_reject.publish | 1.009 | [0.995, 1.020] |
| conflict_inplace_k8_reject.read_inputs | 0.992 | [0.972, 1.011] |
| conflict_inplace_k8_reject.retry_checkout | 1.000 | [0.983, 1.020] |
| conflict_inplace_k8_reject.retry_commit | 1.004 | [0.980, 1.029] |
| conflict_inplace_k8_reject.retry_find | 0.998 | [0.991, 1.003] |
| conflict_inplace_k8_reject.retry_read_inputs | 1.001 | [0.982, 1.020] |
| conflict_inplace_k8_reject.retry_stage | 1.000 | [0.995, 1.007] |
| conflict_inplace_k8_reject.stage | 1.001 | [0.997, 1.005] |
| conflict_inplace_k8_skip | 1.003 | [0.997, 1.009] |
| conflict_inplace_k8_skip.extra | 0.999 | [0.994, 1.004] |
| conflict_inplace_k8_skip.find | 1.000 | [0.998, 1.003] |
| conflict_inplace_k8_skip.first | 1.003 | [0.997, 1.009] |
| conflict_inplace_k8_skip.follow_up_commit | 1.007 | [0.992, 1.024] |
| conflict_inplace_k8_skip.follow_up_plan | 1.010 | [0.998, 1.024] |
| conflict_inplace_k8_skip.follow_up_read_inputs | 0.999 | [0.984, 1.014] |
| conflict_inplace_k8_skip.follow_up_stage | 1.000 | [0.994, 1.004] |
| conflict_inplace_k8_skip.locate | 0.999 | [0.987, 1.013] |
| conflict_inplace_k8_skip.publish | 1.003 | [0.985, 1.022] |
| conflict_inplace_k8_skip.read_inputs | 1.000 | [0.983, 1.020] |
| conflict_inplace_k8_skip.stage | 1.003 | [0.995, 1.009] |
| conflict_moving_k1_reject | 1.000 | [0.993, 1.006] |
| conflict_moving_k1_reject.extra | 1.000 | [0.992, 1.008] |
| conflict_moving_k1_reject.find | 1.003 | [0.992, 1.011] |
| conflict_moving_k1_reject.first | 1.002 | [0.994, 1.009] |
| conflict_moving_k1_reject.locate | 1.003 | [0.993, 1.011] |
| conflict_moving_k1_reject.publish | 1.000 | [0.958, 1.050] |
| conflict_moving_k1_reject.read_inputs | 1.010 | [0.989, 1.030] |
| conflict_moving_k1_reject.retry_checkout | 1.014 | [0.973, 1.054] |
| conflict_moving_k1_reject.retry_commit | 0.979 | [0.950, 1.006] |
| conflict_moving_k1_reject.retry_find | 1.000 | [0.989, 1.009] |
| conflict_moving_k1_reject.retry_read_inputs | 1.012 | [0.999, 1.028] |
| conflict_moving_k1_reject.retry_stage | 1.007 | [0.993, 1.025] |
| conflict_moving_k1_reject.stage | 1.003 | [0.987, 1.022] |
| conflict_moving_k1_skip | 0.996 | [0.989, 1.004] |
| conflict_moving_k1_skip.extra | 0.998 | [0.989, 1.007] |
| conflict_moving_k1_skip.find | 0.996 | [0.989, 1.006] |
| conflict_moving_k1_skip.first | 0.997 | [0.990, 1.007] |
| conflict_moving_k1_skip.follow_up_commit | 0.991 | [0.972, 1.013] |
| conflict_moving_k1_skip.follow_up_find | 0.997 | [0.989, 1.006] |
| conflict_moving_k1_skip.follow_up_plan | 1.000 | [0.961, 1.032] |
| conflict_moving_k1_skip.follow_up_read_inputs | 0.993 | [0.975, 1.012] |
| conflict_moving_k1_skip.follow_up_stage | 0.977 | [0.952, 1.002] |
| conflict_moving_k1_skip.locate | 0.997 | [0.989, 1.006] |
| conflict_moving_k1_skip.publish | 1.002 | [0.969, 1.032] |
| conflict_moving_k1_skip.read_inputs | 0.992 | [0.976, 1.008] |
| conflict_moving_k1_skip.stage | 1.000 | [0.987, 1.013] |
| cycle_inplace_sparse_chain | 1.000 | [0.994, 1.005] |
| cycle_inplace_sparse_chain.commit | 1.004 | [0.991, 1.017] |
| cycle_inplace_sparse_chain.find | 1.000 | [0.995, 1.007] |
| cycle_inplace_sparse_chain.first | 1.002 | [0.994, 1.007] |
| cycle_inplace_sparse_chain.locate | 0.999 | [0.989, 1.005] |
| cycle_inplace_sparse_chain.read_inputs | 0.997 | [0.974, 1.016] |
| cycle_inplace_sparse_chain.read_pending | 1.003 | [0.990, 1.016] |
| cycle_inplace_sparse_chain.read_published | 1.003 | [0.991, 1.017] |
| cycle_inplace_sparse_chain.stage | 1.000 | [0.993, 1.006] |
| cycle_inplace_sparse_chain.update | 0.999 | [0.992, 1.007] |
| cycle_inplace_sparse_one | 1.001 | [0.997, 1.006] |
| cycle_inplace_sparse_one.commit | 1.005 | [0.988, 1.021] |
| cycle_inplace_sparse_one.find | 0.998 | [0.993, 1.003] |
| cycle_inplace_sparse_one.first | 1.003 | [1.000, 1.006] |
| cycle_inplace_sparse_one.locate | 0.996 | [0.989, 1.003] |
| cycle_inplace_sparse_one.read_inputs | 0.995 | [0.979, 1.012] |
| cycle_inplace_sparse_one.read_pending | 0.999 | [0.984, 1.014] |
| cycle_inplace_sparse_one.read_published | 0.994 | [0.983, 1.007] |
| cycle_inplace_sparse_one.stage | 1.003 | [1.000, 1.007] |
| cycle_inplace_sparse_one.update | 1.000 | [0.992, 1.008] |
| cycle_inplace_sparse_plain_chain | 1.004 | [0.998, 1.010] |
| cycle_inplace_sparse_plain_chain.commit | 0.999 | [0.992, 1.007] |
| cycle_inplace_sparse_plain_chain.locate | 1.029 | [0.998, 1.063] |
| cycle_inplace_sparse_plain_chain.merge | 0.998 | [0.991, 1.006] |
| cycle_inplace_sparse_plain_chain.read_pending | 1.005 | [0.988, 1.024] |
| cycle_inplace_sparse_plain_chain.read_published | 1.002 | [0.984, 1.024] |
| cycle_inplace_sparse_plain_chain.update | 0.998 | [0.992, 1.005] |
| cycle_inplace_sparse_plain_one | 1.003 | [0.994, 1.013] |
| cycle_inplace_sparse_plain_one.commit | 1.005 | [0.997, 1.012] |
| cycle_inplace_sparse_plain_one.locate | 1.038 | [1.005, 1.072] |
| cycle_inplace_sparse_plain_one.merge | 1.000 | [0.992, 1.007] |
| cycle_inplace_sparse_plain_one.read_pending | 1.003 | [0.992, 1.013] |
| cycle_inplace_sparse_plain_one.read_published | 0.995 | [0.983, 1.008] |
| cycle_inplace_sparse_plain_one.update | 1.000 | [0.991, 1.008] |
| cycle_inplace_sparse_plain_one_rewrite_rows | 1.003 | [0.996, 1.012] |
| cycle_inplace_sparse_plain_one_rewrite_rows.commit | 0.993 | [0.984, 1.000] |
| cycle_inplace_sparse_plain_one_rewrite_rows.locate | 1.035 | [1.009, 1.064] |
| cycle_inplace_sparse_plain_one_rewrite_rows.merge | 1.009 | [0.991, 1.029] |
| cycle_inplace_sparse_plain_one_rewrite_rows.read_pending | 1.002 | [0.981, 1.024] |
| cycle_inplace_sparse_plain_one_rewrite_rows.read_published | 1.004 | [0.992, 1.017] |
| cycle_inplace_sparse_plain_one_rewrite_rows.update | 0.998 | [0.991, 1.006] |
| cycle_inplace_sparse_plain_shared | 1.007 | [1.001, 1.012] |
| cycle_inplace_sparse_plain_shared.commit | 1.012 | [1.000, 1.021] |
| cycle_inplace_sparse_plain_shared.locate | 1.043 | [1.018, 1.071] |
| cycle_inplace_sparse_plain_shared.merge | 1.007 | [0.994, 1.019] |
| cycle_inplace_sparse_plain_shared.read_pending | 0.996 | [0.987, 1.004] |
| cycle_inplace_sparse_plain_shared.read_published | 1.006 | [0.995, 1.019] |
| cycle_inplace_sparse_plain_shared.update | 1.000 | [0.996, 1.004] |
| cycle_inplace_sparse_shared | 1.000 | [0.996, 1.006] |
| cycle_inplace_sparse_shared.commit | 1.027 | [1.020, 1.034] |
| cycle_inplace_sparse_shared.find | 1.000 | [0.996, 1.005] |
| cycle_inplace_sparse_shared.first | 1.001 | [0.998, 1.005] |
| cycle_inplace_sparse_shared.locate | 0.998 | [0.986, 1.008] |
| cycle_inplace_sparse_shared.read_inputs | 0.994 | [0.964, 1.020] |
| cycle_inplace_sparse_shared.read_pending | 1.009 | [0.997, 1.019] |
| cycle_inplace_sparse_shared.read_published | 0.997 | [0.982, 1.013] |
| cycle_inplace_sparse_shared.stage | 1.001 | [0.998, 1.005] |
| cycle_inplace_sparse_shared.update | 1.000 | [0.991, 1.010] |
| cycle_moving_dense_chain | 1.000 | [0.996, 1.005] |
| cycle_moving_dense_chain.commit | 1.006 | [0.996, 1.015] |
| cycle_moving_dense_chain.find | 1.001 | [0.999, 1.003] |
| cycle_moving_dense_chain.first | 1.001 | [0.999, 1.002] |
| cycle_moving_dense_chain.locate | 1.003 | [0.998, 1.010] |
| cycle_moving_dense_chain.read_inputs | 0.999 | [0.978, 1.023] |
| cycle_moving_dense_chain.read_pending | 0.996 | [0.992, 1.001] |
| cycle_moving_dense_chain.read_published | 0.995 | [0.987, 1.002] |
| cycle_moving_dense_chain.stage | 1.001 | [0.998, 1.002] |
| cycle_moving_dense_chain.update | 0.997 | [0.990, 1.006] |
| cycle_moving_dense_one | 1.000 | [0.996, 1.003] |
| cycle_moving_dense_one.commit | 0.995 | [0.988, 1.004] |
| cycle_moving_dense_one.find | 1.004 | [1.000, 1.009] |
| cycle_moving_dense_one.first | 1.001 | [0.998, 1.004] |
| cycle_moving_dense_one.locate | 1.004 | [1.001, 1.008] |
| cycle_moving_dense_one.read_inputs | 1.002 | [0.996, 1.011] |
| cycle_moving_dense_one.read_pending | 0.991 | [0.980, 1.004] |
| cycle_moving_dense_one.read_published | 1.009 | [1.000, 1.019] |
| cycle_moving_dense_one.stage | 0.999 | [0.995, 1.003] |
| cycle_moving_dense_one.update | 1.001 | [0.997, 1.005] |
| cycle_moving_dense_plain_chain | 0.997 | [0.991, 1.003] |
| cycle_moving_dense_plain_chain.commit | 1.003 | [0.992, 1.013] |
| cycle_moving_dense_plain_chain.locate | 0.990 | [0.978, 1.002] |
| cycle_moving_dense_plain_chain.merge | 1.000 | [0.995, 1.005] |
| cycle_moving_dense_plain_chain.read_pending | 1.000 | [0.993, 1.008] |
| cycle_moving_dense_plain_chain.read_published | 1.007 | [1.000, 1.014] |
| cycle_moving_dense_plain_chain.update | 0.990 | [0.979, 1.002] |
| cycle_moving_dense_plain_one | 1.004 | [0.996, 1.013] |
| cycle_moving_dense_plain_one.commit | 1.005 | [0.995, 1.014] |
| cycle_moving_dense_plain_one.locate | 1.002 | [0.995, 1.009] |
| cycle_moving_dense_plain_one.merge | 1.003 | [1.000, 1.007] |
| cycle_moving_dense_plain_one.read_pending | 1.004 | [0.997, 1.011] |
| cycle_moving_dense_plain_one.read_published | 1.012 | [1.002, 1.022] |
| cycle_moving_dense_plain_one.update | 1.003 | [0.991, 1.014] |
| cycle_moving_dense_plain_shared | 1.000 | [0.996, 1.005] |
| cycle_moving_dense_plain_shared.commit | 0.990 | [0.981, 1.002] |
| cycle_moving_dense_plain_shared.locate | 0.997 | [0.990, 1.006] |
| cycle_moving_dense_plain_shared.merge | 0.999 | [0.991, 1.008] |
| cycle_moving_dense_plain_shared.read_pending | 0.997 | [0.985, 1.009] |
| cycle_moving_dense_plain_shared.read_published | 1.005 | [0.995, 1.015] |
| cycle_moving_dense_plain_shared.update | 0.999 | [0.992, 1.007] |
| cycle_moving_dense_shared | 1.003 | [0.999, 1.007] |
| cycle_moving_dense_shared.commit | 1.001 | [0.993, 1.008] |
| cycle_moving_dense_shared.find | 1.000 | [0.998, 1.002] |
| cycle_moving_dense_shared.first | 1.001 | [0.999, 1.003] |
| cycle_moving_dense_shared.locate | 1.001 | [0.998, 1.003] |
| cycle_moving_dense_shared.read_inputs | 1.002 | [0.993, 1.010] |
| cycle_moving_dense_shared.read_pending | 1.005 | [0.993, 1.018] |
| cycle_moving_dense_shared.read_published | 1.003 | [0.995, 1.010] |
| cycle_moving_dense_shared.stage | 1.000 | [0.998, 1.002] |
| cycle_moving_dense_shared.update | 1.003 | [0.996, 1.009] |
| cycle_moving_sparse_chain | 1.010 | [1.000, 1.019] |
| cycle_moving_sparse_chain.commit | 1.021 | [1.007, 1.037] |
| cycle_moving_sparse_chain.find | 1.007 | [0.999, 1.015] |
| cycle_moving_sparse_chain.first | 1.009 | [1.001, 1.016] |
| cycle_moving_sparse_chain.locate | 1.008 | [1.000, 1.016] |
| cycle_moving_sparse_chain.read_inputs | 1.006 | [0.990, 1.023] |
| cycle_moving_sparse_chain.read_pending | 1.004 | [0.991, 1.016] |
| cycle_moving_sparse_chain.read_published | 1.001 | [0.987, 1.015] |
| cycle_moving_sparse_chain.stage | 1.016 | [0.999, 1.038] |
| cycle_moving_sparse_chain.update | 1.024 | [1.003, 1.047] |
| cycle_moving_sparse_one | 1.005 | [0.996, 1.017] |
| cycle_moving_sparse_one.commit | 0.997 | [0.990, 1.004] |
| cycle_moving_sparse_one.find | 0.991 | [0.978, 1.004] |
| cycle_moving_sparse_one.first | 0.994 | [0.982, 1.006] |
| cycle_moving_sparse_one.locate | 0.992 | [0.980, 1.005] |
| cycle_moving_sparse_one.read_inputs | 1.007 | [0.991, 1.022] |
| cycle_moving_sparse_one.read_pending | 1.012 | [0.987, 1.034] |
| cycle_moving_sparse_one.read_published | 0.984 | [0.972, 0.998] |
| cycle_moving_sparse_one.stage | 0.997 | [0.980, 1.018] |
| cycle_moving_sparse_one.update | 1.026 | [1.004, 1.043] |
| cycle_moving_sparse_plain_chain | 1.018 | [1.008, 1.030] |
| cycle_moving_sparse_plain_chain.commit | 1.001 | [0.996, 1.006] |
| cycle_moving_sparse_plain_chain.locate | 1.040 | [1.021, 1.062] |
| cycle_moving_sparse_plain_chain.merge | 1.003 | [0.994, 1.011] |
| cycle_moving_sparse_plain_chain.read_pending | 1.011 | [1.000, 1.022] |
| cycle_moving_sparse_plain_chain.read_published | 1.003 | [0.991, 1.016] |
| cycle_moving_sparse_plain_chain.update | 1.029 | [1.011, 1.049] |
| cycle_moving_sparse_plain_one | 1.015 | [1.006, 1.024] |
| cycle_moving_sparse_plain_one.commit | 1.016 | [1.006, 1.028] |
| cycle_moving_sparse_plain_one.locate | 1.031 | [1.016, 1.045] |
| cycle_moving_sparse_plain_one.merge | 1.001 | [0.992, 1.012] |
| cycle_moving_sparse_plain_one.read_pending | 1.009 | [0.987, 1.030] |
| cycle_moving_sparse_plain_one.read_published | 0.998 | [0.984, 1.015] |
| cycle_moving_sparse_plain_one.update | 1.027 | [1.006, 1.050] |
| cycle_moving_sparse_plain_shared | 1.008 | [0.998, 1.020] |
| cycle_moving_sparse_plain_shared.commit | 1.007 | [0.993, 1.022] |
| cycle_moving_sparse_plain_shared.locate | 1.024 | [1.003, 1.046] |
| cycle_moving_sparse_plain_shared.merge | 0.993 | [0.981, 1.004] |
| cycle_moving_sparse_plain_shared.read_pending | 1.001 | [0.990, 1.014] |
| cycle_moving_sparse_plain_shared.read_published | 0.999 | [0.985, 1.009] |
| cycle_moving_sparse_plain_shared.update | 1.017 | [0.999, 1.039] |
| cycle_moving_sparse_shared | 1.004 | [0.996, 1.013] |
| cycle_moving_sparse_shared.commit | 1.016 | [0.999, 1.036] |
| cycle_moving_sparse_shared.find | 1.003 | [0.995, 1.012] |
| cycle_moving_sparse_shared.first | 1.004 | [0.996, 1.013] |
| cycle_moving_sparse_shared.locate | 1.004 | [0.996, 1.013] |
| cycle_moving_sparse_shared.read_inputs | 1.006 | [0.980, 1.035] |
| cycle_moving_sparse_shared.read_pending | 1.001 | [0.986, 1.017] |
| cycle_moving_sparse_shared.read_published | 0.994 | [0.985, 1.005] |
| cycle_moving_sparse_shared.stage | 1.014 | [0.996, 1.037] |
| cycle_moving_sparse_shared.update | 1.025 | [1.007, 1.045] |
| reopen_chain | 1.010 | [0.973, 1.045] |
| reopen_chain.open | 1.010 | [0.973, 1.045] |
| reopen_chain_h64 | 1.017 | [0.995, 1.039] |
| reopen_chain_h64.open | 1.017 | [0.995, 1.039] |
| reopen_one | 0.991 | [0.975, 1.006] |
| reopen_one.open | 0.991 | [0.975, 1.006] |
| reopen_one_dense | 0.991 | [0.959, 1.022] |
| reopen_one_dense.open | 0.991 | [0.959, 1.022] |
| reopen_one_h16 | 1.035 | [1.015, 1.054] |
| reopen_one_h16.open | 1.035 | [1.015, 1.054] |
| reopen_one_h16r | 0.980 | [0.967, 0.998] |
| reopen_one_h16r.open | 0.980 | [0.967, 0.998] |
| reopen_one_h64 | 0.994 | [0.975, 1.015] |
| reopen_one_h64.open | 0.994 | [0.975, 1.015] |
| reopen_one_h64r | 1.002 | [0.983, 1.017] |
| reopen_one_h64r.open | 1.002 | [0.983, 1.017] |
| reopen_plain | 0.999 | [0.960, 1.039] |
| reopen_plain.open | 0.999 | [0.960, 1.039] |
| reopen_plain_dense | 0.997 | [0.964, 1.037] |
| reopen_plain_dense.open | 0.997 | [0.964, 1.037] |
| reopen_plain_h16 | 0.963 | [0.936, 0.995] |
| reopen_plain_h16.open | 0.963 | [0.936, 0.995] |
| reopen_plain_h16r | 0.979 | [0.963, 0.994] |
| reopen_plain_h16r.open | 0.979 | [0.963, 0.994] |
| reopen_plain_h64 | 1.020 | [1.011, 1.030] |
| reopen_plain_h64.open | 1.020 | [1.011, 1.030] |
| reopen_plain_h64r | 0.980 | [0.949, 1.009] |
| reopen_plain_h64r.open | 0.980 | [0.949, 1.009] |
| reopen_shared | 1.007 | [0.987, 1.021] |
| reopen_shared.open | 1.007 | [0.987, 1.021] |
| reopen_shared_h64 | 1.012 | [0.990, 1.034] |
| reopen_shared_h64.open | 1.012 | [0.990, 1.034] |
| unrelated_inplace_chain | 1.008 | [0.999, 1.016] |
| unrelated_inplace_chain.update | 1.008 | [0.999, 1.016] |
| unrelated_inplace_one | 1.004 | [0.997, 1.012] |
| unrelated_inplace_one.update | 1.004 | [0.997, 1.012] |
| unrelated_inplace_plain | 1.004 | [0.999, 1.008] |
| unrelated_inplace_plain.update | 1.004 | [0.999, 1.008] |
| unrelated_inplace_shared | 1.001 | [0.996, 1.005] |
| unrelated_inplace_shared.update | 1.001 | [0.996, 1.005] |
| unrelated_moving_chain | 1.028 | [1.006, 1.052] |
| unrelated_moving_chain.update | 1.028 | [1.006, 1.052] |
| unrelated_moving_one | 1.027 | [1.003, 1.054] |
| unrelated_moving_one.update | 1.027 | [1.003, 1.054] |
| unrelated_moving_plain | 1.029 | [1.009, 1.052] |
| unrelated_moving_plain.update | 1.029 | [1.009, 1.052] |
| unrelated_moving_shared | 1.030 | [1.011, 1.050] |
| unrelated_moving_shared.update | 1.030 | [1.011, 1.050] |
| update_chain | 1.020 | [1.000, 1.044] |
| update_chain.update | 1.020 | [1.000, 1.044] |
| update_chain_h64 | 1.009 | [0.999, 1.019] |
| update_chain_h64.update | 1.009 | [0.999, 1.019] |
| update_one | 1.020 | [1.001, 1.042] |
| update_one.update | 1.020 | [1.001, 1.042] |
| update_one_dense | 0.996 | [0.981, 1.012] |
| update_one_dense.update | 0.996 | [0.981, 1.012] |
| update_one_h16 | 1.005 | [0.993, 1.018] |
| update_one_h16.update | 1.005 | [0.993, 1.018] |
| update_one_h16r | 1.009 | [0.998, 1.021] |
| update_one_h16r.update | 1.009 | [0.998, 1.021] |
| update_one_h64 | 1.015 | [1.002, 1.029] |
| update_one_h64.update | 1.015 | [1.002, 1.029] |
| update_one_h64r | 1.012 | [0.998, 1.028] |
| update_one_h64r.update | 1.012 | [0.998, 1.028] |
| update_plain | 1.018 | [1.003, 1.034] |
| update_plain.update | 1.018 | [1.003, 1.034] |
| update_plain_dense | 1.003 | [0.989, 1.016] |
| update_plain_dense.update | 1.003 | [0.989, 1.016] |
| update_plain_h16 | 1.013 | [0.998, 1.029] |
| update_plain_h16.update | 1.013 | [0.998, 1.029] |
| update_plain_h16r | 1.008 | [1.000, 1.017] |
| update_plain_h16r.update | 1.008 | [1.000, 1.017] |
| update_plain_h64 | 1.006 | [0.994, 1.017] |
| update_plain_h64.update | 1.006 | [0.994, 1.017] |
| update_plain_h64r | 1.006 | [0.996, 1.017] |
| update_plain_h64r.update | 1.006 | [0.996, 1.017] |
| update_shared | 1.023 | [1.004, 1.045] |
| update_shared.update | 1.023 | [1.004, 1.045] |
| update_shared_h64 | 1.004 | [0.996, 1.014] |
| update_shared_h64.update | 1.004 | [0.996, 1.014] |
