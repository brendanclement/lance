## Scalar reads

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| scan all true / no flags | 3.36 | 3.31–3.47 | 3.31 | 3.20–3.48 | 1.016 | [1.006, 1.026] | 6/8 | 13.9 | 0.0 | 7.3 | 7.0 |
| scan 1% masked / 1% NULL | 6.17 | 6.10–6.29 | 3.77 | 3.68–3.81 | 1.636 | [1.622, 1.650] | 8/8 | 13.8 | 0.0 | 50.3 | 10.7 |
| scan 1% masked / no NULLs | 6.17 | 6.10–6.29 | 3.31 | 3.20–3.48 | 1.857 | [1.828, 1.885] | 8/8 | 13.8 | 0.0 | 50.3 | 7.0 |
| filter_null 1% masked / 1% NULL | 6.81 | 6.62–6.92 | 4.36 | 4.23–4.43 | 1.562 | [1.545, 1.578] | 8/8 | 13.8 | 0.0 | 50.3 | 10.7 |
| take 1% masked / 1% NULL | 6.86 | 6.70–7.18 | 7.10 | 6.88–7.35 | 0.972 | [0.959, 0.985] | 1/8 | 3.7 | 0.0 | 50.3 | 10.7 |

## Unrelated writes

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| unrelated moving write, one / plain | 23.78 | 23.05–24.03 | 23.67 | 22.92–23.97 | 1.004 | [1.001, 1.009] | 7/8 | 7.3 | 0.1 | 9.0 | 7.7 |

## Refresh cycles: flagged / plain merge_insert

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| moving_sparse one: whole cycle | 55.63 | 54.96–56.46 | 67.73 | 65.63–68.44 | 0.823 | [0.816, 0.830] | 0/8 | 35.0 | 0.1 | 9.1 | 7.8 |
| moving_sparse one: update | 23.71 | 22.65–23.95 | 23.64 | 22.78–24.03 | 0.998 | [0.990, 1.006] | 4/8 | 7.3 | 0.1 | — | — |
| moving_sparse one: read_pending | 10.73 | 10.15–11.19 | 9.33 | 8.72–9.52 | 1.148 | [1.121, 1.173] | 8/8 | 13.9 | 0.0 | — | — |
| moving_sparse one: locate | 9.82 | 9.78–9.92 | 14.73 | 13.53–14.90 | 0.674 | [0.662, 0.691] | 0/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: stage | 0.556 | 0.544–0.571 | 10.49 | 10.36–10.59 | 0.053 | [0.052, 0.054] | 0/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: commit | 1.04 | 1.03–1.07 | 1.02 | 0.993–1.05 | 1.019 | [1.006, 1.033] | 6/8 | 0.0 | 0.0 | — | — |
| moving_sparse one: read_published | 10.02 | 9.21–10.38 | 8.69 | 8.45–9.06 | 1.140 | [1.112, 1.165] | 8/8 | 13.9 | 0.0 | — | — |
| moving_dense one: whole cycle | 134 | 133–135 | 121 | 120–122 | 1.109 | [1.104, 1.115] | 8/8 | 149.7 | 13.8 | 328.8 | 7.9 |
| moving_dense one: update | 61.12 | 60.43–61.86 | 46.49 | 45.89–47.15 | 1.313 | [1.306, 1.318] | 8/8 | 110.9 | 12.1 | — | — |
| moving_dense one: read_pending | 16.50 | 15.93–16.73 | 14.89 | 14.71–15.12 | 1.102 | [1.089, 1.116] | 8/8 | 15.5 | 0.0 | — | — |
| moving_dense one: locate | 15.24 | 15.07–15.30 | 16.26 | 16.10–16.58 | 0.936 | [0.930, 0.941] | 0/8 | 6.7 | 0.0 | — | — |
| moving_dense one: stage | 25.22 | 25.13–25.38 | 28.88 | 28.72–29.13 | 0.875 | [0.872, 0.879] | 0/8 | 1.4 | 1.4 | — | — |
| moving_dense one: commit | 1.11 | 1.08–1.12 | 1.07 | 1.03–1.08 | 1.039 | [1.029, 1.049] | 8/8 | 0.0 | 0.3 | — | — |
| moving_dense one: read_published | 14.86 | 14.63–15.00 | 13.10 | 12.97–13.32 | 1.130 | [1.126, 1.134] | 8/8 | 15.2 | 0.0 | — | — |
| inplace_sparse one: whole cycle | 195 | 194–197 | 106 | 105–107 | 1.835 | [1.830, 1.839] | 8/8 | 95.2 | 63.3 | 14.1 | 14.1 |
| inplace_sparse one: update | 54.02 | 53.48–55.00 | 53.92 | 53.11–54.89 | 1.002 | [0.997, 1.006] | 4/8 | 52.2 | 49.7 | — | — |
| inplace_sparse one: read_pending | 5.64 | 5.46–5.92 | 4.01 | 3.96–4.11 | 1.411 | [1.394, 1.429] | 8/8 | 14.2 | 0.0 | — | — |
| inplace_sparse one: locate | 11.41 | 11.19–11.56 | 14.21 | 13.22–14.44 | 0.807 | [0.792, 0.828] | 0/8 | 1.3 | 0.0 | — | — |
| inplace_sparse one: stage | 119 | 118–119 | 29.14 | 28.82–29.38 | 4.078 | [4.062, 4.096] | 8/8 | 13.5 | 13.6 | — | — |
| inplace_sparse one: commit | 1.14 | 1.12–1.14 | 1.15 | 1.07–1.23 | 0.985 | [0.960, 1.010] | 2/8 | 0.0 | 0.0 | — | — |
| inplace_sparse one: read_published | 3.75 | 3.64–3.82 | 3.82 | 3.77–3.89 | 0.979 | [0.969, 0.987] | 0/8 | 14.0 | 0.0 | — | — |

## Whole-fragment replacement

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| one: in-place stage / moving stage | 119 | 118–119 | 0.556 | 0.544–0.571 | 213.583 | [211.130, 215.831] | 8/8 | 13.5 | 13.6 | — | — |

## Conflicts

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| inplace_k1: reject publish | 0.162 | 0.154–0.174 | 152 | 151–154 | 0.001 | [0.001, 0.001] | 0/8 | 0.0 | 0.0 | — | — |
| inplace_k1: reject extra | 148 | 148–150 | 152 | 151–154 | 0.974 | [0.972, 0.976] | 0/8 | 23.2 | 13.9 | — | — |
| inplace_k1: skip publish | 1.18 | 1.08–1.21 | 152 | 151–153 | 0.008 | [0.007, 0.008] | 0/8 | 0.0 | 0.0 | — | — |
| inplace_k1: skip extra | 32.95 | 32.53–33.27 | 152 | 151–153 | 0.217 | [0.216, 0.218] | 0/8 | 3.6 | 3.5 | — | — |

## Publication after unrelated appends

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| publish after 32 appends: one / plain | 1.64 | 1.59–1.77 | 1.64 | 1.57–1.76 | 0.997 | [0.960, 1.041] | 4/8 | 0.4 | 0.0 | — | — |

## Accumulated history

| comparison | ms | range | base ms | range | ratio | 95% interval | processes > 1 | read MiB | written MiB | manifest KiB | base manifest KiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| reopen one / plain | 0.193 | 0.186–0.204 | 0.203 | 0.203–0.215 | 0.934 | [0.911, 0.958] | 0/8 | — | — | 7.3 | 7.0 |
| reopen one_h16 / plain_h16 | 0.234 | 0.225–0.246 | 0.207 | 0.203–0.216 | 1.136 | [1.116, 1.158] | 8/8 | — | — | 16.3 | 9.2 |
| reopen plain_h16 / plain | 0.207 | 0.203–0.216 | 0.203 | 0.203–0.215 | 0.995 | [0.973, 1.015] | 5/8 | — | — | 9.2 | 7.0 |
| reopen one_h16 / one | 0.234 | 0.225–0.246 | 0.193 | 0.186–0.204 | 1.210 | [1.190, 1.231] | 8/8 | — | — | 16.3 | 7.3 |
| append one / plain | 1.60 | 1.55–1.64 | 1.57 | 1.55–1.61 | 1.011 | [1.001, 1.021] | 6/8 | 0.0 | 0.0 | 7.4 | 7.1 |
| append one_h16 / plain_h16 | 1.55 | 1.51–1.58 | 1.61 | 1.57–1.66 | 0.966 | [0.946, 0.984] | 1/8 | 0.0 | 0.0 | 16.4 | 9.3 |
| append plain_h16 / plain | 1.61 | 1.57–1.66 | 1.57 | 1.55–1.61 | 1.018 | [1.009, 1.028] | 7/8 | 0.0 | 0.0 | 9.3 | 7.1 |
| append one_h16 / one | 1.55 | 1.51–1.58 | 1.60 | 1.55–1.64 | 0.972 | [0.955, 0.988] | 1/8 | 0.0 | 0.0 | 16.4 | 7.4 |
| update one / plain | 23.03 | 21.77–23.48 | 22.96 | 21.76–23.20 | 1.006 | [1.001, 1.011] | 6/8 | 7.2 | 0.1 | 8.9 | 7.6 |
| update one_h16 / plain_h16 | 33.13 | 32.50–33.93 | 33.24 | 32.54–33.67 | 1.000 | [0.993, 1.006] | 3/8 | 7.4 | 0.1 | 16.8 | 9.3 |
| update plain_h16 / plain | 33.24 | 32.54–33.67 | 22.96 | 21.76–23.20 | 1.455 | [1.441, 1.470] | 8/8 | 7.4 | 0.1 | 9.3 | 7.6 |
| update one_h16 / one | 33.13 | 32.50–33.93 | 23.03 | 21.77–23.48 | 1.446 | [1.436, 1.461] | 8/8 | 7.4 | 0.1 | 16.8 | 8.9 |

## Control: <build>-copy / <build>, per workload

| workload | ratio | 95% interval |
|---|---|---|
| after_appends_k32_one | 1.009 | [1.006, 1.013] |
| after_appends_k32_one.commit | 1.037 | [1.000, 1.088] |
| after_appends_k32_one.find | 1.002 | [0.995, 1.009] |
| after_appends_k32_one.first | 1.008 | [1.005, 1.012] |
| after_appends_k32_one.locate | 1.007 | [0.992, 1.018] |
| after_appends_k32_one.read_inputs | 1.010 | [0.994, 1.024] |
| after_appends_k32_one.stage | 1.008 | [1.005, 1.011] |
| after_appends_k32_plain_one | 0.997 | [0.990, 1.003] |
| after_appends_k32_plain_one.commit | 1.008 | [0.963, 1.043] |
| after_appends_k32_plain_one.locate | 0.977 | [0.956, 0.997] |
| after_appends_k32_plain_one.merge | 1.004 | [0.991, 1.016] |
| append_one | 0.996 | [0.987, 1.004] |
| append_one.commit | 0.992 | [0.986, 0.997] |
| append_one.first | 1.003 | [0.994, 1.016] |
| append_one.stage | 1.003 | [0.994, 1.016] |
| append_one_h16 | 0.999 | [0.969, 1.024] |
| append_one_h16.commit | 1.001 | [0.978, 1.024] |
| append_one_h16.first | 0.990 | [0.942, 1.021] |
| append_one_h16.stage | 0.990 | [0.942, 1.021] |
| append_plain | 0.997 | [0.987, 1.006] |
| append_plain.commit | 0.996 | [0.968, 1.020] |
| append_plain.first | 1.005 | [0.991, 1.019] |
| append_plain.stage | 1.005 | [0.991, 1.019] |
| append_plain_h16 | 1.016 | [1.010, 1.025] |
| append_plain_h16.commit | 1.024 | [1.010, 1.038] |
| append_plain_h16.first | 0.985 | [0.980, 0.990] |
| append_plain_h16.stage | 0.985 | [0.980, 0.990] |
| conflict_inplace_k1_reject | 1.008 | [1.001, 1.017] |
| conflict_inplace_k1_reject.extra | 1.005 | [1.001, 1.009] |
| conflict_inplace_k1_reject.find | 1.000 | [0.993, 1.010] |
| conflict_inplace_k1_reject.first | 1.005 | [1.000, 1.011] |
| conflict_inplace_k1_reject.locate | 1.012 | [0.995, 1.035] |
| conflict_inplace_k1_reject.publish | 1.010 | [0.964, 1.059] |
| conflict_inplace_k1_reject.read_inputs | 1.017 | [0.995, 1.050] |
| conflict_inplace_k1_reject.retry_checkout | 1.022 | [0.987, 1.058] |
| conflict_inplace_k1_reject.retry_commit | 1.008 | [0.997, 1.022] |
| conflict_inplace_k1_reject.retry_find | 0.995 | [0.986, 1.004] |
| conflict_inplace_k1_reject.retry_read_inputs | 1.009 | [0.983, 1.038] |
| conflict_inplace_k1_reject.retry_stage | 1.004 | [1.001, 1.006] |
| conflict_inplace_k1_reject.stage | 1.004 | [0.999, 1.009] |
| conflict_inplace_k1_skip | 1.005 | [1.002, 1.008] |
| conflict_inplace_k1_skip.extra | 1.007 | [1.001, 1.014] |
| conflict_inplace_k1_skip.find | 1.007 | [1.000, 1.015] |
| conflict_inplace_k1_skip.first | 1.004 | [1.003, 1.006] |
| conflict_inplace_k1_skip.follow_up_commit | 1.083 | [1.051, 1.121] |
| conflict_inplace_k1_skip.follow_up_plan | 1.004 | [0.966, 1.045] |
| conflict_inplace_k1_skip.follow_up_read_inputs | 0.993 | [0.956, 1.018] |
| conflict_inplace_k1_skip.follow_up_stage | 1.005 | [0.997, 1.012] |
| conflict_inplace_k1_skip.locate | 1.010 | [0.999, 1.020] |
| conflict_inplace_k1_skip.publish | 1.019 | [0.992, 1.050] |
| conflict_inplace_k1_skip.read_inputs | 1.012 | [1.002, 1.023] |
| conflict_inplace_k1_skip.stage | 1.004 | [1.000, 1.008] |
| cycle_inplace_sparse_one | 1.004 | [0.998, 1.010] |
| cycle_inplace_sparse_one.commit | 0.997 | [0.985, 1.007] |
| cycle_inplace_sparse_one.find | 1.004 | [0.989, 1.019] |
| cycle_inplace_sparse_one.first | 1.001 | [0.995, 1.007] |
| cycle_inplace_sparse_one.locate | 1.001 | [0.990, 1.010] |
| cycle_inplace_sparse_one.read_inputs | 0.998 | [0.986, 1.016] |
| cycle_inplace_sparse_one.read_pending | 1.012 | [0.981, 1.059] |
| cycle_inplace_sparse_one.read_published | 1.023 | [1.011, 1.042] |
| cycle_inplace_sparse_one.stage | 1.001 | [0.995, 1.008] |
| cycle_inplace_sparse_one.update | 1.012 | [1.004, 1.023] |
| cycle_inplace_sparse_plain_one | 1.004 | [0.998, 1.011] |
| cycle_inplace_sparse_plain_one.commit | 1.038 | [0.980, 1.113] |
| cycle_inplace_sparse_plain_one.locate | 0.976 | [0.953, 0.993] |
| cycle_inplace_sparse_plain_one.merge | 1.010 | [1.008, 1.013] |
| cycle_inplace_sparse_plain_one.read_pending | 1.017 | [1.004, 1.030] |
| cycle_inplace_sparse_plain_one.read_published | 1.007 | [0.995, 1.020] |
| cycle_inplace_sparse_plain_one.update | 1.007 | [0.994, 1.019] |
| cycle_moving_dense_one | 1.005 | [1.000, 1.010] |
| cycle_moving_dense_one.commit | 0.986 | [0.979, 0.993] |
| cycle_moving_dense_one.find | 0.999 | [0.991, 1.004] |
| cycle_moving_dense_one.first | 1.000 | [0.996, 1.004] |
| cycle_moving_dense_one.locate | 0.998 | [0.989, 1.004] |
| cycle_moving_dense_one.read_inputs | 0.998 | [0.985, 1.011] |
| cycle_moving_dense_one.read_pending | 1.013 | [1.000, 1.035] |
| cycle_moving_dense_one.read_published | 1.007 | [0.998, 1.017] |
| cycle_moving_dense_one.stage | 1.002 | [0.999, 1.006] |
| cycle_moving_dense_one.update | 1.008 | [0.998, 1.014] |
| cycle_moving_dense_plain_one | 1.006 | [1.001, 1.010] |
| cycle_moving_dense_plain_one.commit | 0.976 | [0.971, 0.981] |
| cycle_moving_dense_plain_one.locate | 1.009 | [1.005, 1.013] |
| cycle_moving_dense_plain_one.merge | 1.001 | [0.997, 1.006] |
| cycle_moving_dense_plain_one.read_pending | 1.000 | [0.991, 1.010] |
| cycle_moving_dense_plain_one.read_published | 1.011 | [1.004, 1.019] |
| cycle_moving_dense_plain_one.update | 1.005 | [0.998, 1.018] |
| cycle_moving_sparse_one | 1.009 | [0.994, 1.024] |
| cycle_moving_sparse_one.commit | 1.000 | [0.991, 1.010] |
| cycle_moving_sparse_one.find | 1.000 | [0.994, 1.007] |
| cycle_moving_sparse_one.first | 1.003 | [0.995, 1.010] |
| cycle_moving_sparse_one.locate | 1.002 | [0.994, 1.009] |
| cycle_moving_sparse_one.read_inputs | 1.034 | [1.005, 1.065] |
| cycle_moving_sparse_one.read_pending | 1.015 | [0.981, 1.051] |
| cycle_moving_sparse_one.read_published | 1.043 | [1.023, 1.065] |
| cycle_moving_sparse_one.stage | 1.025 | [1.014, 1.037] |
| cycle_moving_sparse_one.update | 0.997 | [0.967, 1.014] |
| cycle_moving_sparse_plain_one | 0.990 | [0.970, 1.005] |
| cycle_moving_sparse_plain_one.commit | 0.992 | [0.975, 1.021] |
| cycle_moving_sparse_plain_one.locate | 0.981 | [0.951, 1.002] |
| cycle_moving_sparse_plain_one.merge | 1.000 | [0.992, 1.011] |
| cycle_moving_sparse_plain_one.read_pending | 1.011 | [0.963, 1.069] |
| cycle_moving_sparse_plain_one.read_published | 0.993 | [0.980, 1.008] |
| cycle_moving_sparse_plain_one.update | 0.981 | [0.969, 0.990] |
| filter_null_null_1pct | 1.008 | [0.981, 1.028] |
| filter_null_null_1pct.read | 1.008 | [0.981, 1.028] |
| filter_null_partial_1pct | 1.000 | [0.990, 1.014] |
| filter_null_partial_1pct.read | 1.000 | [0.990, 1.014] |
| reopen_one | 1.019 | [0.994, 1.050] |
| reopen_one.open | 1.019 | [0.994, 1.050] |
| reopen_one_h16 | 0.991 | [0.960, 1.022] |
| reopen_one_h16.open | 0.991 | [0.960, 1.022] |
| reopen_plain | 1.001 | [0.982, 1.021] |
| reopen_plain.open | 1.001 | [0.982, 1.021] |
| reopen_plain_h16 | 0.991 | [0.976, 1.005] |
| reopen_plain_h16.open | 0.991 | [0.976, 1.005] |
| scan_null_1pct | 1.013 | [0.997, 1.029] |
| scan_null_1pct.read | 1.013 | [0.997, 1.029] |
| scan_one | 1.014 | [1.004, 1.029] |
| scan_one.read | 1.014 | [1.004, 1.029] |
| scan_partial_1pct | 1.007 | [0.997, 1.020] |
| scan_partial_1pct.read | 1.007 | [0.997, 1.020] |
| scan_plain | 1.007 | [0.976, 1.041] |
| scan_plain.read | 1.007 | [0.976, 1.041] |
| take_null_1pct | 1.014 | [0.984, 1.045] |
| take_null_1pct.read | 1.014 | [0.984, 1.045] |
| take_partial_1pct | 1.023 | [1.000, 1.054] |
| take_partial_1pct.read | 1.023 | [1.000, 1.054] |
| unrelated_moving_one | 0.993 | [0.981, 1.006] |
| unrelated_moving_one.update | 0.993 | [0.981, 1.006] |
| unrelated_moving_plain | 0.998 | [0.975, 1.017] |
| unrelated_moving_plain.update | 0.998 | [0.975, 1.017] |
| update_one | 0.992 | [0.963, 1.011] |
| update_one.update | 0.992 | [0.963, 1.011] |
| update_one_h16 | 1.001 | [0.993, 1.007] |
| update_one_h16.update | 1.001 | [0.993, 1.007] |
| update_plain | 0.989 | [0.961, 1.006] |
| update_plain.update | 0.989 | [0.961, 1.006] |
| update_plain_h16 | 0.992 | [0.982, 1.006] |
| update_plain_h16.update | 0.992 | [0.982, 1.006] |
