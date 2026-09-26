## count_summary_aggregate
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.018 | [0.989, 1.050] | 6/10 |
| prototype | instructions | 1.001 | [0.999, 1.002] | 5/10 |
| prototype | cycles | 1.006 | [0.979, 1.037] | 5/10 |
| noresolve | wall_ns | 1.025 | [0.995, 1.055] | 7/10 |
| noresolve | instructions | 1.001 | [1.000, 1.002] | 8/10 |
| noresolve | cycles | 1.036 | [1.017, 1.054] | 9/10 |
| funnel | wall_ns | 0.992 | [0.975, 1.010] | 4/10 |
| funnel | instructions | 1.001 | [1.000, 1.002] | 6/10 |
| funnel | cycles | 1.009 | [0.985, 1.034] | 5/10 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 25.11 | 23.83 | 9.47 |
| prototype | 25.76 | 24.01 | 9.26 |
| noresolve | 25.75 | 23.37 | 9.42 |
| funnel | 24.75 | 24.17 | 9.64 |

## filter_summary_is_null_count
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.022 | [0.995, 1.053] | 5/10 |
| prototype | instructions | 1.001 | [1.000, 1.002] | 6/10 |
| prototype | cycles | 1.019 | [0.988, 1.054] | 7/10 |
| noresolve | wall_ns | 1.027 | [0.995, 1.057] | 7/10 |
| noresolve | instructions | 1.001 | [1.000, 1.002] | 8/10 |
| noresolve | cycles | 1.037 | [1.016, 1.059] | 8/10 |
| funnel | wall_ns | 1.000 | [0.980, 1.016] | 5/10 |
| funnel | instructions | 1.001 | [1.000, 1.002] | 6/10 |
| funnel | cycles | 1.010 | [0.983, 1.038] | 7/10 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 27.09 | 25.69 | 8.87 |
| prototype | 27.94 | 26.27 | 8.82 |
| noresolve | 27.94 | 25.79 | 8.90 |
| funnel | 27.04 | 26.45 | 8.99 |

## scan_summary_full
| build | metric | ratio to reference | 95% interval | rounds > 1 |
|---|---|---|---|---|
| prototype | wall_ns | 1.011 | [0.981, 1.040] | 6/10 |
| prototype | instructions | 1.000 | [1.000, 1.001] | 3/10 |
| prototype | cycles | 1.010 | [0.982, 1.037] | 7/10 |
| noresolve | wall_ns | 1.014 | [0.987, 1.040] | 6/10 |
| noresolve | instructions | 1.001 | [1.000, 1.001] | 8/10 |
| noresolve | cycles | 1.021 | [1.007, 1.038] | 8/10 |
| funnel | wall_ns | 0.978 | [0.955, 1.000] | 3/10 |
| funnel | instructions | 1.001 | [1.000, 1.002] | 6/10 |
| funnel | cycles | 1.003 | [0.981, 1.028] | 5/10 |

| build | median wall ms | p10 wall ms | busy cores |
|---|---|---|---|
| baseline | 25.13 | 23.75 | 9.65 |
| prototype | 25.47 | 24.37 | 9.69 |
| noresolve | 25.43 | 24.16 | 9.69 |
| funnel | 24.34 | 23.62 | 9.96 |

