## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.032 | [1.016, 1.049] | 16/20 |
| prototype | instructions | 1.001 | [1.000, 1.002] | 15/20 |
| prototype | cycles | 1.007 | [0.986, 1.028] | 11/20 |
| fix | wall_ns | 1.000 | [0.984, 1.017] | 8/20 |
| fix | instructions | 1.000 | [0.999, 1.000] | 7/20 |
| fix | cycles | 1.015 | [0.997, 1.033] | 14/20 |
| funnel | wall_ns | 0.991 | [0.973, 1.010] | 8/20 |
| funnel | instructions | 1.000 | [1.000, 1.001] | 10/20 |
| funnel | cycles | 0.992 | [0.975, 1.010] | 8/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 25.15 | 24.22 | 9.47 |
| prototype | 26.19 | 25.27 | 9.15 |
| fix | 25.17 | 24.17 | 9.58 |
| funnel | 24.95 | 24.21 | 9.40 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.032 | [1.017, 1.047] | 15/20 |
| prototype | instructions | 1.001 | [1.001, 1.002] | 17/20 |
| prototype | cycles | 1.012 | [0.993, 1.031] | 12/20 |
| fix | wall_ns | 1.001 | [0.983, 1.018] | 9/20 |
| fix | instructions | 1.000 | [0.999, 1.000] | 6/20 |
| fix | cycles | 1.019 | [1.001, 1.037] | 13/20 |
| funnel | wall_ns | 0.999 | [0.982, 1.016] | 11/20 |
| funnel | instructions | 1.000 | [1.000, 1.001] | 11/20 |
| funnel | cycles | 0.998 | [0.984, 1.013] | 8/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 26.98 | 26.10 | 8.92 |
| prototype | 28.14 | 27.27 | 8.73 |
| fix | 27.07 | 25.91 | 9.09 |
| funnel | 27.06 | 26.25 | 8.86 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.038 | [1.022, 1.053] | 17/20 |
| prototype | instructions | 1.000 | [1.000, 1.001] | 9/20 |
| prototype | cycles | 1.005 | [0.988, 1.022] | 13/20 |
| fix | wall_ns | 1.001 | [0.982, 1.019] | 12/20 |
| fix | instructions | 0.999 | [0.999, 1.000] | 7/20 |
| fix | cycles | 1.009 | [0.988, 1.029] | 12/20 |
| funnel | wall_ns | 0.996 | [0.977, 1.016] | 11/20 |
| funnel | instructions | 1.000 | [0.999, 1.001] | 11/20 |
| funnel | cycles | 0.988 | [0.975, 1.002] | 8/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.63 | 23.96 | 9.81 |
| prototype | 25.86 | 24.91 | 9.45 |
| fix | 24.73 | 23.73 | 9.88 |
| funnel | 24.69 | 23.98 | 9.63 |

