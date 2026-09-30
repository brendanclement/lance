## Scalar reads

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| scan all true / no flags | 9.60 | 9.37–9.75 | 9.41 | 9.30–9.55 | 1.012 | [1.001, 1.020] | 6/8 | 55.2 | 0.0 | 7.4 | 7.1 |
| scan 1% masked / 1% NULL | 19.67 | 19.49–20.09 | 10.66 | 10.24–10.87 | 1.852 | [1.832, 1.874] | 8/8 | 55.0 | 0.0 | 166.8 | 10.8 |
| scan 1% masked / no NULLs | 19.67 | 19.49–20.09 | 9.41 | 9.30–9.55 | 2.090 | [2.070, 2.109] | 8/8 | 55.0 | 0.0 | 166.8 | 7.1 |
| filter_null 1% masked / 1% NULL | 21.55 | 21.31–22.20 | 11.99 | 11.53–12.22 | 1.807 | [1.783, 1.828] | 8/8 | 55.0 | 0.0 | 166.8 | 10.8 |
| take 1% masked / 1% NULL | 8.98 | 8.82–9.34 | 9.02 | 8.66–9.39 | 0.996 | [0.974, 1.019] | 4/8 | 3.7 | 0.0 | 166.8 | 10.8 |

## Unrelated writes

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| unrelated moving write, one / plain | 43.55 | 39.84–44.44 | 43.59 | 39.65–44.33 | 1.002 | [0.994, 1.010] | 6/8 | 22.5 | 0.1 | 9.4 | 7.8 |

## Refresh cycles: flagged / plain merge_insert

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| moving_sparse one: whole cycle | 147 | 144–149 | 185 | 179–187 | 0.795 | [0.790, 0.801] | 0/8 | 133.0 | 0.1 | 9.5 | 7.9 |
| moving_sparse one: update | 43.59 | 39.38–44.13 | 43.53 | 39.49–44.20 | 1.001 | [0.995, 1.006] | 5/8 | 22.5 | 0.1 | — | — |
| moving_sparse one: read_pending | 32.94 | 32.19–34.76 | 30.16 | 29.60–30.69 | 1.104 | [1.087, 1.122] | 8/8 | 55.3 | 0.0 | — | — |
| moving_sparse one: locate | 36.95 | 36.78–37.06 | 48.46 | 44.51–48.91 | 0.770 | [0.759, 0.788] | 0/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: stage | 0.616 | 0.606–0.638 | 33.50 | 32.75–33.62 | 0.019 | [0.018, 0.019] | 0/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: commit | 1.07 | 1.03–1.08 | 1.04 | 1.03–1.07 | 1.023 | [1.007, 1.038] | 6/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: read_published | 31.69 | 31.46–32.21 | 29.42 | 28.67–30.58 | 1.077 | [1.062, 1.094] | 8/8 | 55.2 | 0.0 | — | — |
| moving_dense one: whole cycle | 512 | 508–514 | 394 | 393–400 | 1.295 | [1.290, 1.300] | 8/8 | 597.1 | 53.4 | 649.3 | 8.0 |
| moving_dense one: update | 259 | 257–261 | 204 | 203–205 | 1.274 | [1.269, 1.279] | 8/8 | 442.3 | 47.1 | — | — |
| moving_dense one: read_pending | 37.69 | 36.66–38.02 | 31.73 | 31.43–32.40 | 1.176 | [1.166, 1.185] | 8/8 | 61.3 | 0.0 | — | — |
| moving_dense one: locate | 44.23 | 43.85–44.40 | 40.52 | 40.16–41.26 | 1.087 | [1.081, 1.092] | 8/8 | 27.1 | 0.0 | — | — |
| moving_dense one: stage | 132 | 132–133 | 86.69 | 86.00–89.45 | 1.522 | [1.507, 1.533] | 8/8 | 5.5 | 5.7 | — | — |
| moving_dense one: commit | 1.24 | 1.22–1.26 | 1.09 | 1.06–1.10 | 1.140 | [1.127, 1.153] | 8/8 | 0.0 | 0.6 | — | — |
| moving_dense one: read_published | 36.92 | 35.89–37.22 | 31.10 | 30.80–31.41 | 1.182 | [1.170, 1.194] | 8/8 | 60.9 | 0.0 | — | — |
| inplace_sparse one: whole cycle | 440 | 438–443 | 278 | 277–281 | 1.580 | [1.572, 1.588] | 8/8 | 369.2 | 246.5 | 14.0 | 14.1 |
| inplace_sparse one: update | 159 | 158–160 | 159 | 158–160 | 1.003 | [0.999, 1.006] | 6/8 | 204.3 | 193.9 | — | — |
| inplace_sparse one: read_pending | 12.32 | 12.08–12.51 | 9.72 | 9.56–9.98 | 1.265 | [1.252, 1.276] | 8/8 | 55.6 | 0.0 | — | — |
| inplace_sparse one: locate | 23.57 | 23.33–23.72 | 32.56 | 29.47–32.77 | 0.729 | [0.718, 0.748] | 0/8 | 1.5 | 0.0 | — | — |
| inplace_sparse one: stage | 234 | 232–237 | 66.80 | 66.26–67.40 | 3.503 | [3.483, 3.524] | 8/8 | 52.4 | 52.5 | — | — |
| inplace_sparse one: commit | 1.15 | 1.14–1.17 | 1.21 | 1.16–1.23 | 0.957 | [0.943, 0.973] | 1/8 | 0.0 | 0.0 | — | — |
| inplace_sparse one: read_published | 9.61 | 9.47–9.74 | 9.45 | 9.33–9.60 | 1.014 | [1.006, 1.023] | 7/8 | 55.4 | 0.0 | — | — |

## Whole-fragment replacement

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| one: in-place stage / moving stage | 234 | 232–237 | 0.616 | 0.606–0.638 | 377.153 | [371.981, 381.817] | 8/8 | 52.4 | 52.5 | — | — |

## Conflicts

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| inplace_k1: reject publish | 0.171 | 0.158–0.180 | 316 | 313–318 | 0.001 | [0.001, 0.001] | 0/8 | 0.0 | 0.0 | — | — |
| inplace_k1: reject extra | 311 | 309–312 | 316 | 313–318 | 0.984 | [0.981, 0.987] | 0/8 | 66.2 | 55.4 | — | — |
| inplace_k1: skip publish | 1.16 | 1.09–1.23 | 316 | 315–320 | 0.004 | [0.004, 0.004] | 0/8 | 0.0 | 0.0 | — | — |
| inplace_k1: skip extra | 63.60 | 63.14–64.39 | 316 | 315–320 | 0.201 | [0.201, 0.202] | 0/8 | 14.1 | 13.9 | — | — |

## Publication after unrelated appends

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| publish after 32 appends: one / plain | 1.62 | 1.59–1.65 | 1.67 | 1.52–1.93 | 0.956 | [0.902, 1.013] | 3/8 | 0.4 | 0.0 | — | — |

## Accumulated history

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| reopen one / plain | 0.222 | 0.208–0.230 | 0.227 | 0.217–0.232 | 0.977 | [0.953, 1.000] | 2/8 | — | — | 7.4 | 7.1 |
| reopen one_h16 / plain_h16 | 0.248 | 0.245–0.253 | 0.225 | 0.214–0.236 | 1.101 | [1.079, 1.121] | 8/8 | — | — | 16.8 | 9.2 |
| reopen plain_h16 / plain | 0.225 | 0.214–0.236 | 0.227 | 0.217–0.232 | 1.004 | [0.982, 1.031] | 3/8 | — | — | 9.2 | 7.1 |
| reopen one_h16 / one | 0.248 | 0.245–0.253 | 0.222 | 0.208–0.230 | 1.131 | [1.114, 1.150] | 8/8 | — | — | 16.8 | 7.4 |
| append one / plain | 1.55 | 1.53–1.58 | 1.53 | 1.49–1.59 | 1.011 | [0.991, 1.031] | 6/8 | 0.0 | 0.0 | 7.5 | 7.2 |
| append one_h16 / plain_h16 | 1.52 | 1.45–1.58 | 1.60 | 1.54–1.63 | 0.948 | [0.926, 0.965] | 0/8 | 0.0 | 0.0 | 16.8 | 9.3 |
| append plain_h16 / plain | 1.60 | 1.54–1.63 | 1.53 | 1.49–1.59 | 1.038 | [1.023, 1.056] | 8/8 | 0.0 | 0.0 | 9.3 | 7.2 |
| append one_h16 / one | 1.52 | 1.45–1.58 | 1.55 | 1.53–1.58 | 0.973 | [0.958, 0.989] | 2/8 | 0.0 | 0.0 | 16.8 | 7.5 |
| update one / plain | 43.52 | 39.41–44.08 | 43.25 | 39.49–44.56 | 1.003 | [0.996, 1.010] | 5/8 | 22.5 | 0.1 | 9.4 | 7.8 |
| update one_h16 / plain_h16 | 75.38 | 71.88–76.52 | 75.86 | 71.94–77.02 | 0.996 | [0.988, 1.003] | 2/8 | 22.6 | 0.1 | 17.2 | 9.3 |
| update plain_h16 / plain | 75.86 | 71.94–77.02 | 43.25 | 39.49–44.56 | 1.756 | [1.726, 1.787] | 8/8 | 22.6 | 0.1 | 9.3 | 7.8 |
| update one_h16 / one | 75.38 | 71.88–76.52 | 43.52 | 39.41–44.08 | 1.744 | [1.727, 1.769] | 8/8 | 22.6 | 0.1 | 17.2 | 9.4 |

## Control: <build>-copy / <build>, per workload

| workload | ratio | 95% interval |
|---|---|---|
| after_appends_k32_one | 0.996 | [0.981, 1.005] |
| after_appends_k32_one.commit | 1.005 | [0.991, 1.019] |
| after_appends_k32_one.find | 0.998 | [0.989, 1.005] |
| after_appends_k32_one.first | 0.996 | [0.980, 1.005] |
| after_appends_k32_one.locate | 0.998 | [0.987, 1.011] |
| after_appends_k32_one.read_inputs | 1.000 | [0.986, 1.018] |
| after_appends_k32_one.stage | 0.996 | [0.979, 1.006] |
| after_appends_k32_plain_one | 1.004 | [0.993, 1.020] |
| after_appends_k32_plain_one.commit | 0.943 | [0.905, 0.983] |
| after_appends_k32_plain_one.locate | 1.020 | [0.983, 1.087] |
| after_appends_k32_plain_one.merge | 1.002 | [0.995, 1.012] |
| append_one | 1.002 | [0.990, 1.014] |
| append_one.commit | 1.003 | [0.984, 1.019] |
| append_one.first | 1.009 | [0.991, 1.041] |
| append_one.stage | 1.009 | [0.991, 1.041] |
| append_one_h16 | 0.990 | [0.969, 1.009] |
| append_one_h16.commit | 0.980 | [0.954, 1.012] |
| append_one_h16.first | 0.990 | [0.971, 1.004] |
| append_one_h16.stage | 0.990 | [0.971, 1.004] |
| append_plain | 1.000 | [0.956, 1.033] |
| append_plain.commit | 0.993 | [0.961, 1.012] |
| append_plain.first | 1.029 | [1.016, 1.043] |
| append_plain.stage | 1.029 | [1.016, 1.043] |
| append_plain_h16 | 1.021 | [0.992, 1.052] |
| append_plain_h16.commit | 1.022 | [0.997, 1.048] |
| append_plain_h16.first | 1.020 | [0.999, 1.041] |
| append_plain_h16.stage | 1.020 | [0.999, 1.041] |
| conflict_inplace_k1_reject | 0.998 | [0.995, 1.001] |
| conflict_inplace_k1_reject.extra | 1.001 | [0.997, 1.005] |
| conflict_inplace_k1_reject.find | 1.003 | [0.993, 1.015] |
| conflict_inplace_k1_reject.first | 0.995 | [0.992, 0.998] |
| conflict_inplace_k1_reject.locate | 1.002 | [0.991, 1.013] |
| conflict_inplace_k1_reject.publish | 1.022 | [0.924, 1.089] |
| conflict_inplace_k1_reject.read_inputs | 1.003 | [0.989, 1.018] |
| conflict_inplace_k1_reject.retry_checkout | 0.974 | [0.907, 1.040] |
| conflict_inplace_k1_reject.retry_commit | 1.014 | [1.005, 1.025] |
| conflict_inplace_k1_reject.retry_find | 1.002 | [0.988, 1.014] |
| conflict_inplace_k1_reject.retry_read_inputs | 1.005 | [0.992, 1.024] |
| conflict_inplace_k1_reject.retry_stage | 1.000 | [0.995, 1.005] |
| conflict_inplace_k1_reject.stage | 0.993 | [0.990, 0.996] |
| conflict_inplace_k1_skip | 0.997 | [0.990, 1.003] |
| conflict_inplace_k1_skip.extra | 0.990 | [0.983, 0.998] |
| conflict_inplace_k1_skip.find | 1.003 | [0.990, 1.017] |
| conflict_inplace_k1_skip.first | 0.997 | [0.990, 1.001] |
| conflict_inplace_k1_skip.follow_up_commit | 0.985 | [0.949, 1.042] |
| conflict_inplace_k1_skip.follow_up_plan | 1.021 | [1.015, 1.033] |
| conflict_inplace_k1_skip.follow_up_read_inputs | 0.971 | [0.949, 0.990] |
| conflict_inplace_k1_skip.follow_up_stage | 0.990 | [0.984, 0.998] |
| conflict_inplace_k1_skip.locate | 1.005 | [0.996, 1.017] |
| conflict_inplace_k1_skip.publish | 0.993 | [0.969, 1.037] |
| conflict_inplace_k1_skip.read_inputs | 0.997 | [0.984, 1.011] |
| conflict_inplace_k1_skip.stage | 0.997 | [0.989, 1.003] |
| cycle_inplace_sparse_one | 0.995 | [0.991, 0.998] |
| cycle_inplace_sparse_one.commit | 1.012 | [1.000, 1.021] |
| cycle_inplace_sparse_one.find | 1.003 | [1.001, 1.005] |
| cycle_inplace_sparse_one.first | 0.993 | [0.989, 0.997] |
| cycle_inplace_sparse_one.locate | 0.999 | [0.995, 1.003] |
| cycle_inplace_sparse_one.read_inputs | 1.003 | [0.992, 1.016] |
| cycle_inplace_sparse_one.read_pending | 1.002 | [0.997, 1.008] |
| cycle_inplace_sparse_one.read_published | 1.004 | [0.994, 1.019] |
| cycle_inplace_sparse_one.stage | 0.993 | [0.987, 0.997] |
| cycle_inplace_sparse_one.update | 0.993 | [0.990, 0.996] |
| cycle_inplace_sparse_plain_one | 1.000 | [0.995, 1.004] |
| cycle_inplace_sparse_plain_one.commit | 0.986 | [0.968, 1.010] |
| cycle_inplace_sparse_plain_one.locate | 1.025 | [0.999, 1.077] |
| cycle_inplace_sparse_plain_one.merge | 1.002 | [0.997, 1.006] |
| cycle_inplace_sparse_plain_one.read_pending | 0.988 | [0.968, 0.999] |
| cycle_inplace_sparse_plain_one.read_published | 0.997 | [0.993, 1.001] |
| cycle_inplace_sparse_plain_one.update | 0.995 | [0.992, 1.000] |
| cycle_moving_dense_one | 0.997 | [0.995, 0.998] |
| cycle_moving_dense_one.commit | 0.991 | [0.982, 1.000] |
| cycle_moving_dense_one.find | 0.999 | [0.997, 1.000] |
| cycle_moving_dense_one.first | 0.999 | [0.997, 1.000] |
| cycle_moving_dense_one.locate | 0.999 | [0.995, 1.001] |
| cycle_moving_dense_one.read_inputs | 1.006 | [0.991, 1.016] |
| cycle_moving_dense_one.read_pending | 0.992 | [0.974, 1.003] |
| cycle_moving_dense_one.read_published | 0.998 | [0.993, 1.005] |
| cycle_moving_dense_one.stage | 0.999 | [0.997, 1.001] |
| cycle_moving_dense_one.update | 0.998 | [0.993, 1.003] |
| cycle_moving_dense_plain_one | 0.999 | [0.991, 1.003] |
| cycle_moving_dense_plain_one.commit | 1.000 | [0.993, 1.007] |
| cycle_moving_dense_plain_one.locate | 0.996 | [0.989, 1.003] |
| cycle_moving_dense_plain_one.merge | 0.997 | [0.982, 1.010] |
| cycle_moving_dense_plain_one.read_pending | 1.001 | [0.990, 1.008] |
| cycle_moving_dense_plain_one.read_published | 0.998 | [0.988, 1.008] |
| cycle_moving_dense_plain_one.update | 0.996 | [0.992, 1.000] |
| cycle_moving_sparse_one | 1.003 | [0.994, 1.014] |
| cycle_moving_sparse_one.commit | 1.000 | [0.990, 1.011] |
| cycle_moving_sparse_one.find | 1.001 | [0.998, 1.003] |
| cycle_moving_sparse_one.first | 1.001 | [0.999, 1.003] |
| cycle_moving_sparse_one.locate | 1.001 | [0.999, 1.003] |
| cycle_moving_sparse_one.read_inputs | 1.003 | [0.982, 1.025] |
| cycle_moving_sparse_one.read_pending | 0.966 | [0.954, 0.979] |
| cycle_moving_sparse_one.read_published | 0.997 | [0.986, 1.010] |
| cycle_moving_sparse_one.stage | 1.017 | [0.992, 1.038] |
| cycle_moving_sparse_one.update | 1.031 | [1.004, 1.075] |
| cycle_moving_sparse_plain_one | 1.017 | [1.008, 1.028] |
| cycle_moving_sparse_plain_one.commit | 0.988 | [0.971, 1.005] |
| cycle_moving_sparse_plain_one.locate | 1.031 | [1.009, 1.073] |
| cycle_moving_sparse_plain_one.merge | 1.005 | [1.001, 1.009] |
| cycle_moving_sparse_plain_one.read_pending | 0.999 | [0.984, 1.014] |
| cycle_moving_sparse_plain_one.read_published | 1.025 | [0.997, 1.053] |
| cycle_moving_sparse_plain_one.update | 1.031 | [1.006, 1.071] |
| filter_null_null_1pct | 0.992 | [0.965, 1.023] |
| filter_null_null_1pct.read | 0.992 | [0.965, 1.023] |
| filter_null_partial_1pct | 0.999 | [0.974, 1.017] |
| filter_null_partial_1pct.read | 0.999 | [0.974, 1.017] |
| reopen_one | 1.006 | [0.960, 1.074] |
| reopen_one.open | 1.006 | [0.960, 1.074] |
| reopen_one_h16 | 1.005 | [0.989, 1.021] |
| reopen_one_h16.open | 1.005 | [0.989, 1.021] |
| reopen_plain | 0.985 | [0.953, 1.025] |
| reopen_plain.open | 0.985 | [0.953, 1.025] |
| reopen_plain_h16 | 0.965 | [0.955, 0.975] |
| reopen_plain_h16.open | 0.965 | [0.955, 0.975] |
| scan_null_1pct | 1.005 | [0.981, 1.019] |
| scan_null_1pct.read | 1.005 | [0.981, 1.019] |
| scan_one | 0.987 | [0.979, 0.996] |
| scan_one.read | 0.987 | [0.979, 0.996] |
| scan_partial_1pct | 0.996 | [0.979, 1.007] |
| scan_partial_1pct.read | 0.996 | [0.979, 1.007] |
| scan_plain | 0.998 | [0.990, 1.011] |
| scan_plain.read | 0.998 | [0.990, 1.011] |
| take_null_1pct | 0.992 | [0.952, 1.034] |
| take_null_1pct.read | 0.992 | [0.952, 1.034] |
| take_partial_1pct | 0.985 | [0.964, 1.000] |
| take_partial_1pct.read | 0.985 | [0.964, 1.000] |
| unrelated_moving_one | 1.032 | [1.000, 1.070] |
| unrelated_moving_one.update | 1.032 | [1.000, 1.070] |
| unrelated_moving_plain | 1.030 | [0.997, 1.073] |
| unrelated_moving_plain.update | 1.030 | [0.997, 1.073] |
| update_one | 1.025 | [0.999, 1.073] |
| update_one.update | 1.025 | [0.999, 1.073] |
| update_one_h16 | 1.010 | [0.987, 1.042] |
| update_one_h16.update | 1.010 | [0.987, 1.042] |
| update_plain | 1.037 | [1.003, 1.075] |
| update_plain.update | 1.037 | [1.003, 1.075] |
| update_plain_h16 | 1.000 | [0.974, 1.040] |
| update_plain_h16.update | 1.000 | [0.974, 1.040] |
