## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix | wall_ns | 1.032 | [1.017, 1.045] | 18/20 |
| fix | instructions | 1.000 | [1.000, 1.000] | 14/20 |
| fix | cycles | 1.027 | [1.012, 1.044] | 15/20 |
| base-field | wall_ns | 1.019 | [1.008, 1.029] | 17/20 |
| base-field | instructions | 1.000 | [1.000, 1.001] | 15/20 |
| base-field | cycles | 1.021 | [0.998, 1.045] | 14/20 |
| fragbase-field | wall_ns | 1.032 | [1.018, 1.045] | 19/20 |
| fragbase-field | instructions | 1.000 | [1.000, 1.001] | 17/20 |
| fragbase-field | cycles | 1.001 | [0.988, 1.013] | 11/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.93 | 24.34 | 9.45 |
| fix | 25.87 | 25.04 | 9.39 |
| base-field | 25.50 | 24.77 | 9.43 |
| fragbase-field | 25.86 | 25.09 | 9.11 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix | wall_ns | 1.028 | [1.012, 1.041] | 17/20 |
| fix | instructions | 1.000 | [1.000, 1.000] | 11/20 |
| fix | cycles | 1.029 | [1.014, 1.045] | 16/20 |
| base-field | wall_ns | 1.021 | [1.008, 1.033] | 15/20 |
| base-field | instructions | 1.000 | [1.000, 1.001] | 13/20 |
| base-field | cycles | 1.023 | [1.000, 1.046] | 15/20 |
| fragbase-field | wall_ns | 1.027 | [1.012, 1.039] | 18/20 |
| fragbase-field | instructions | 1.001 | [1.000, 1.001] | 17/20 |
| fragbase-field | cycles | 1.001 | [0.989, 1.013] | 11/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 26.93 | 26.23 | 8.92 |
| fix | 27.80 | 26.88 | 8.93 |
| base-field | 27.63 | 26.64 | 8.91 |
| fragbase-field | 27.78 | 27.00 | 8.69 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix | wall_ns | 1.022 | [1.006, 1.037] | 16/20 |
| fix | instructions | 1.000 | [0.999, 1.000] | 7/20 |
| fix | cycles | 1.013 | [0.997, 1.031] | 12/20 |
| base-field | wall_ns | 1.014 | [1.003, 1.025] | 13/20 |
| base-field | instructions | 1.000 | [1.000, 1.001] | 12/20 |
| base-field | cycles | 1.020 | [0.999, 1.040] | 14/20 |
| fragbase-field | wall_ns | 1.026 | [1.010, 1.040] | 17/20 |
| fragbase-field | instructions | 1.000 | [0.999, 1.000] | 10/20 |
| fragbase-field | cycles | 0.991 | [0.976, 1.006] | 9/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 24.71 | 24.13 | 9.75 |
| fix | 25.35 | 24.46 | 9.66 |
| base-field | 25.24 | 24.39 | 9.81 |
| fragbase-field | 25.57 | 24.78 | 9.40 |

