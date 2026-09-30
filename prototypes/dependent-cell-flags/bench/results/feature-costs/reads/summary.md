## Scalar reads

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | alloc MiB | base alloc MiB | peak MiB | base peak MiB | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| scan all true / no flags | 2.96 | 2.88–3.08 | 3.02 | 2.92–3.12 | 0.984 | [0.979, 0.989] | 0/16 | 150.1 | 152.3 | 18.8 | 18.9 | 13.7 | 0.0 | 2.2 | 2.1 |
| scan all true after moves / same moves | 15.96 | 15.60–16.13 | 9.21 | 9.01–9.37 | 1.734 | [1.725, 1.741] | 16/16 | 251.4 | 249.0 | 32.2 | 28.7 | 13.8 | 0.0 | 41.4 | 2.3 |
| scan 1% masked / 1% NULL | 5.50 | 5.41–5.62 | 3.25 | 3.17–3.36 | 1.686 | [1.674, 1.699] | 16/16 | 156.4 | 157.0 | 22.4 | 19.1 | 13.6 | 0.0 | 42.0 | 3.0 |
| scan 1% masked / no NULLs | 5.50 | 5.41–5.62 | 3.02 | 2.92–3.12 | 1.828 | [1.815, 1.841] | 16/16 | 156.4 | 152.3 | 22.4 | 18.9 | 13.6 | 0.0 | 42.0 | 2.1 |
| scan 50% masked / 50% NULL | 5.94 | 5.71–6.08 | 3.19 | 3.09–3.42 | 1.849 | [1.830, 1.867] | 16/16 | 156.6 | 107.3 | 22.3 | 13.2 | 13.6 | 0.0 | 163.3 | 3.0 |
| scan all masked / all NULL | 3.05 | 2.90–3.23 | 1.01 | 0.962–1.04 | 3.017 | [2.970, 3.061] | 16/16 | 154.0 | 7.1 | 18.7 | 0.7 | 13.7 | 0.0 | 3.0 | 3.0 |
| filter_null all true / no flags | 3.26 | 3.13–3.41 | 3.30 | 3.13–3.45 | 0.991 | [0.986, 0.997] | 4/16 | 158.4 | 160.6 | 18.0 | 19.0 | 13.7 | 0.0 | 2.2 | 2.1 |
| filter_null all true after moves / same moves | 16.32 | 15.96–16.53 | 9.46 | 9.27–9.75 | 1.724 | [1.714, 1.734] | 16/16 | 252.2 | 249.8 | 32.4 | 29.6 | 13.8 | 0.0 | 41.4 | 2.3 |
| filter_null 1% masked / 1% NULL | 6.08 | 5.97–6.21 | 3.68 | 3.60–3.83 | 1.653 | [1.638, 1.668] | 16/16 | 165.3 | 165.6 | 22.4 | 19.1 | 13.6 | 0.0 | 42.0 | 3.0 |
| filter_null 1% masked / no NULLs | 6.08 | 5.97–6.21 | 3.30 | 3.13–3.45 | 1.848 | [1.827, 1.870] | 16/16 | 165.3 | 160.6 | 22.4 | 19.0 | 13.6 | 0.0 | 42.0 | 2.1 |
| filter_null 50% masked / 50% NULL | 9.11 | 8.80–9.38 | 4.97 | 4.80–5.13 | 1.826 | [1.803, 1.851] | 16/16 | 187.9 | 127.2 | 22.6 | 13.4 | 13.6 | 0.0 | 163.3 | 3.0 |
| filter_null all masked / all NULL | 3.30 | 3.16–3.48 | 1.32 | 1.28–1.36 | 2.498 | [2.464, 2.530] | 16/16 | 166.1 | 19.2 | 18.7 | 0.8 | 13.7 | 0.0 | 3.0 | 3.0 |
| take all true / no flags | 5.92 | 5.74–6.56 | 5.97 | 5.71–6.70 | 0.990 | [0.983, 0.998] | 5/16 | 35.0 | 35.6 | 7.7 | 7.8 | 3.7 | 0.0 | 2.2 | 2.1 |
| take all true after moves / same moves | 1571 | 1514–1586 | 1577 | 1532–1593 | 0.997 | [0.992, 1.002] | 6/16 | 35.1 | 35.9 | 7.7 | 7.9 | 3.7 | 0.0 | 41.4 | 2.3 |
| take 1% masked / 1% NULL | 5.92 | 5.71–6.55 | 6.13 | 5.94–6.46 | 0.968 | [0.958, 0.978] | 1/16 | 34.7 | 35.8 | 7.6 | 7.8 | 3.6 | 0.0 | 42.0 | 3.0 |
| take 1% masked / no NULLs | 5.92 | 5.71–6.55 | 5.97 | 5.71–6.70 | 0.989 | [0.979, 0.998] | 4/16 | 34.7 | 35.6 | 7.6 | 7.8 | 3.6 | 0.0 | 42.0 | 2.1 |
| take 50% masked / 50% NULL | 5.93 | 5.73–6.49 | 5.97 | 5.61–6.45 | 0.995 | [0.986, 1.003] | 7/16 | 34.7 | 24.1 | 7.7 | 5.3 | 3.6 | 0.0 | 163.3 | 3.0 |
| take all masked / all NULL | 5.86 | 5.66–6.42 | 0.372 | 0.347–0.389 | 16.003 | [15.675, 16.365] | 16/16 | 35.0 | 0.7 | 3.9 | 0.2 | 3.7 | 0.0 | 3.0 | 3.0 |

## Control: <build>-copy / <build>, per workload

| workload | ratio | 95% interval |
|---|---|---|
| filter_null_masked | 0.999 | [0.980, 1.019] |
| filter_null_masked.read | 0.999 | [0.980, 1.019] |
| filter_null_null_1pct | 1.006 | [0.995, 1.017] |
| filter_null_null_1pct.read | 1.006 | [0.995, 1.017] |
| filter_null_null_50pct | 1.002 | [0.992, 1.013] |
| filter_null_null_50pct.read | 1.002 | [0.992, 1.013] |
| filter_null_null_all | 1.003 | [0.991, 1.017] |
| filter_null_null_all.read | 1.003 | [0.991, 1.017] |
| filter_null_one | 0.999 | [0.988, 1.009] |
| filter_null_one.read | 0.999 | [0.988, 1.009] |
| filter_null_partial_1pct | 0.998 | [0.991, 1.007] |
| filter_null_partial_1pct.read | 0.998 | [0.991, 1.007] |
| filter_null_partial_50pct | 1.014 | [0.993, 1.035] |
| filter_null_partial_50pct.read | 1.014 | [0.993, 1.035] |
| filter_null_plain | 0.997 | [0.982, 1.012] |
| filter_null_plain.read | 0.997 | [0.982, 1.012] |
| filter_null_plain_moved | 0.999 | [0.989, 1.009] |
| filter_null_plain_moved.read | 0.999 | [0.989, 1.009] |
| filter_null_ready_moved | 0.996 | [0.988, 1.004] |
| filter_null_ready_moved.read | 0.996 | [0.988, 1.004] |
| scan_masked | 0.997 | [0.982, 1.017] |
| scan_masked.read | 0.997 | [0.982, 1.017] |
| scan_null_1pct | 1.013 | [1.000, 1.027] |
| scan_null_1pct.read | 1.013 | [1.000, 1.027] |
| scan_null_50pct | 0.994 | [0.981, 1.010] |
| scan_null_50pct.read | 0.994 | [0.981, 1.010] |
| scan_null_all | 0.997 | [0.976, 1.017] |
| scan_null_all.read | 0.997 | [0.976, 1.017] |
| scan_one | 0.990 | [0.977, 1.003] |
| scan_one.read | 0.990 | [0.977, 1.003] |
| scan_partial_1pct | 1.003 | [0.999, 1.008] |
| scan_partial_1pct.read | 1.003 | [0.999, 1.008] |
| scan_partial_50pct | 0.999 | [0.987, 1.011] |
| scan_partial_50pct.read | 0.999 | [0.987, 1.011] |
| scan_plain | 0.996 | [0.984, 1.007] |
| scan_plain.read | 0.996 | [0.984, 1.007] |
| scan_plain_moved | 1.009 | [1.002, 1.016] |
| scan_plain_moved.read | 1.009 | [1.002, 1.016] |
| scan_ready_moved | 1.000 | [0.993, 1.007] |
| scan_ready_moved.read | 1.000 | [0.993, 1.007] |
| take_masked | 1.026 | [1.004, 1.060] |
| take_masked.read | 1.026 | [1.004, 1.060] |
| take_null_1pct | 1.008 | [0.991, 1.031] |
| take_null_1pct.read | 1.008 | [0.991, 1.031] |
| take_null_50pct | 1.021 | [0.994, 1.061] |
| take_null_50pct.read | 1.021 | [0.994, 1.061] |
| take_null_all | 1.008 | [0.968, 1.052] |
| take_null_all.read | 1.008 | [0.968, 1.052] |
| take_one | 1.024 | [1.002, 1.056] |
| take_one.read | 1.024 | [1.002, 1.056] |
| take_partial_1pct | 1.020 | [0.993, 1.060] |
| take_partial_1pct.read | 1.020 | [0.993, 1.060] |
| take_partial_50pct | 1.009 | [0.982, 1.049] |
| take_partial_50pct.read | 1.009 | [0.982, 1.049] |
| take_plain | 1.023 | [0.991, 1.071] |
| take_plain.read | 1.023 | [0.991, 1.071] |
| take_plain_moved | 1.007 | [1.000, 1.014] |
| take_plain_moved.read | 1.007 | [1.000, 1.014] |
| take_ready_moved | 1.010 | [1.001, 1.020] |
| take_ready_moved.read | 1.010 | [1.001, 1.020] |
