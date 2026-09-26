## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.044 | [1.023, 1.065] | 17/20 |
| prototype | instructions | 1.001 | [1.000, 1.001] | 14/20 |
| prototype | cycles | 1.023 | [0.998, 1.047] | 13/20 |
| funnel | wall_ns | 1.013 | [0.994, 1.033] | 12/20 |
| funnel | instructions | 1.000 | [1.000, 1.001] | 14/20 |
| funnel | cycles | 1.015 | [0.996, 1.032] | 14/20 |
| layout | wall_ns | 1.027 | [1.005, 1.049] | 14/20 |
| layout | instructions | 1.000 | [0.999, 1.000] | 10/20 |
| layout | cycles | 0.997 | [0.973, 1.021] | 12/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.73 | 23.94 | 9.41 |
| prototype | 26.14 | 25.13 | 9.18 |
| funnel | 25.17 | 24.37 | 9.42 |
| layout | 25.75 | 24.71 | 9.14 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.034 | [1.015, 1.053] | 16/20 |
| prototype | instructions | 1.001 | [1.000, 1.001] | 15/20 |
| prototype | cycles | 1.023 | [1.000, 1.046] | 14/20 |
| funnel | wall_ns | 1.008 | [0.993, 1.023] | 14/20 |
| funnel | instructions | 1.000 | [1.000, 1.001] | 15/20 |
| funnel | cycles | 1.010 | [0.994, 1.025] | 13/20 |
| layout | wall_ns | 1.023 | [1.006, 1.040] | 15/20 |
| layout | instructions | 1.000 | [0.999, 1.000] | 9/20 |
| layout | cycles | 0.997 | [0.975, 1.018] | 11/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 26.93 | 26.03 | 8.86 |
| prototype | 28.11 | 27.13 | 8.76 |
| funnel | 27.29 | 26.44 | 8.90 |
| layout | 27.80 | 26.79 | 8.66 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.033 | [1.015, 1.051] | 15/20 |
| prototype | instructions | 1.000 | [1.000, 1.001] | 9/20 |
| prototype | cycles | 1.004 | [0.984, 1.025] | 12/20 |
| funnel | wall_ns | 1.007 | [0.988, 1.026] | 13/20 |
| funnel | instructions | 1.000 | [1.000, 1.001] | 9/20 |
| funnel | cycles | 1.000 | [0.984, 1.016] | 8/20 |
| layout | wall_ns | 1.025 | [1.007, 1.043] | 14/20 |
| layout | instructions | 1.000 | [0.999, 1.000] | 11/20 |
| layout | cycles | 1.001 | [0.978, 1.021] | 10/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.62 | 23.72 | 9.69 |
| prototype | 25.68 | 24.74 | 9.45 |
| funnel | 24.87 | 24.10 | 9.64 |
| layout | 25.40 | 24.60 | 9.45 |

