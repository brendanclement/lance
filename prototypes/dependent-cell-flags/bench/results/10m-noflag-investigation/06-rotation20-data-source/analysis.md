## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix@basedata | wall_ns | 1.017 | [0.997, 1.037] | 13/20 |
| fix@basedata | instructions | 1.000 | [1.000, 1.000] | 8/20 |
| fix@basedata | cycles | 1.018 | [1.004, 1.033] | 15/20 |
| baseline@fixdata | wall_ns | 1.005 | [0.984, 1.025] | 12/20 |
| baseline@fixdata | instructions | 1.003 | [1.002, 1.003] | 20/20 |
| baseline@fixdata | cycles | 0.998 | [0.976, 1.019] | 11/20 |
| fix@fixdata | wall_ns | 1.032 | [1.017, 1.047] | 15/20 |
| fix@fixdata | instructions | 1.003 | [1.003, 1.003] | 20/20 |
| fix@fixdata | cycles | 1.031 | [1.013, 1.050] | 15/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline@basedata | 24.87 | 24.21 | 9.48 |
| fix@basedata | 25.24 | 24.59 | 9.49 |
| baseline@fixdata | 24.97 | 24.16 | 9.39 |
| fix@fixdata | 25.72 | 24.64 | 9.48 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix@basedata | wall_ns | 1.011 | [0.993, 1.029] | 14/20 |
| fix@basedata | instructions | 1.000 | [1.000, 1.000] | 8/20 |
| fix@basedata | cycles | 1.016 | [1.005, 1.027] | 14/20 |
| baseline@fixdata | wall_ns | 1.006 | [0.988, 1.025] | 7/20 |
| baseline@fixdata | instructions | 1.003 | [1.002, 1.003] | 20/20 |
| baseline@fixdata | cycles | 0.999 | [0.979, 1.019] | 10/20 |
| fix@fixdata | wall_ns | 1.026 | [1.013, 1.040] | 15/20 |
| fix@fixdata | instructions | 1.003 | [1.002, 1.003] | 20/20 |
| fix@fixdata | cycles | 1.028 | [1.013, 1.043] | 15/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline@basedata | 26.91 | 26.25 | 8.97 |
| fix@basedata | 27.16 | 26.40 | 9.03 |
| baseline@fixdata | 26.97 | 26.12 | 8.91 |
| fix@fixdata | 27.73 | 26.55 | 8.99 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| fix@basedata | wall_ns | 1.009 | [0.990, 1.027] | 10/20 |
| fix@basedata | instructions | 0.999 | [0.999, 1.000] | 6/20 |
| fix@basedata | cycles | 1.004 | [0.991, 1.018] | 9/20 |
| baseline@fixdata | wall_ns | 1.009 | [0.989, 1.029] | 12/20 |
| baseline@fixdata | instructions | 1.003 | [1.002, 1.003] | 20/20 |
| baseline@fixdata | cycles | 1.007 | [0.987, 1.027] | 10/20 |
| fix@fixdata | wall_ns | 1.031 | [1.017, 1.045] | 16/20 |
| fix@fixdata | instructions | 1.002 | [1.002, 1.003] | 20/20 |
| fix@fixdata | cycles | 1.024 | [1.010, 1.037] | 16/20 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline@basedata | 24.65 | 23.95 | 9.77 |
| fix@basedata | 24.78 | 24.13 | 9.75 |
| baseline@fixdata | 24.85 | 24.01 | 9.80 |
| fix@fixdata | 25.44 | 24.57 | 9.69 |

