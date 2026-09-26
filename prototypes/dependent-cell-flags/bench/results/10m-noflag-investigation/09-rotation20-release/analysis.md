## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 0.997 | [0.974, 1.020] | 10/20 |
| prototype | instructions | 0.999 | [0.999, 1.000] | 5/20 |
| prototype | cycles | 0.987 | [0.969, 1.005] | 7/20 |
| fix | wall_ns | 1.039 | [1.017, 1.060] | 16/20 |
| fix | instructions | 1.002 | [1.001, 1.002] | 18/20 |
| fix | cycles | 1.040 | [1.025, 1.056] | 17/20 |
| base-field | wall_ns | 1.025 | [1.012, 1.037] | 15/20 |
| base-field | instructions | 1.001 | [1.000, 1.001] | 11/20 |
| base-field | cycles | 1.005 | [0.993, 1.016] | 13/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 25.13 | 24.41 | 9.30 |
| prototype | 25.12 | 23.63 | 9.23 |
| fix | 26.41 | 24.93 | 9.30 |
| base-field | 25.94 | 25.02 | 9.13 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 0.999 | [0.978, 1.019] | 10/20 |
| prototype | instructions | 0.999 | [0.999, 1.000] | 6/20 |
| prototype | cycles | 0.992 | [0.975, 1.011] | 9/20 |
| fix | wall_ns | 1.032 | [1.013, 1.049] | 17/20 |
| fix | instructions | 1.001 | [1.001, 1.002] | 17/20 |
| fix | cycles | 1.037 | [1.022, 1.053] | 18/20 |
| base-field | wall_ns | 1.024 | [1.013, 1.035] | 18/20 |
| base-field | instructions | 1.001 | [1.000, 1.001] | 12/20 |
| base-field | cycles | 1.009 | [0.996, 1.023] | 13/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 27.23 | 26.64 | 8.80 |
| prototype | 27.29 | 25.83 | 8.74 |
| fix | 28.41 | 26.69 | 8.82 |
| base-field | 28.04 | 27.14 | 8.67 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.025 | [1.004, 1.045] | 14/20 |
| prototype | instructions | 1.000 | [0.999, 1.001] | 10/20 |
| prototype | cycles | 1.026 | [1.010, 1.043] | 15/20 |
| fix | wall_ns | 1.044 | [1.023, 1.063] | 17/20 |
| fix | instructions | 1.002 | [1.001, 1.002] | 17/20 |
| fix | cycles | 1.045 | [1.029, 1.061] | 18/20 |
| base-field | wall_ns | 1.032 | [1.020, 1.044] | 18/20 |
| base-field | instructions | 1.001 | [1.000, 1.001] | 13/20 |
| base-field | cycles | 1.018 | [1.001, 1.035] | 15/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.77 | 24.06 | 9.53 |
| prototype | 25.67 | 24.19 | 9.54 |
| fix | 26.11 | 24.70 | 9.52 |
| base-field | 25.68 | 24.88 | 9.40 |

