# `masked_embeddings.lance`

A dataset whose `embedding`, a `FixedSizeList<Float32, 4>`, is masked by the
dependent cell flag `embedding.ready`, which watches `body`. It is the
`masked_embeddings` dataset of
`rust/lance/src/dataset/tests/dataset_cell_flags_masking.rs`, written to disk so
that the `lance.torch` tests in `../test_data.py` can read masked embeddings:
Python has no cell flag API.

```text
fragment  id  stored embedding        reads as
0         0   [0, 0, 0, 1]            [0, 0, 0, 1]
0         1   NULL (computed)         NULL
0         2   [2, 0, 0, 1]            [2, 0, 0, 1]
1         4   [4, 0, 0, 1]            [4, 0, 0, 1]
1         5   [5, 0, 0, 1] (stale)    NULL (pending)
1         7   NULL (never computed)   NULL (pending)
2         8   [5.25, 0, 0, 1]         NULL (pending, never published)
2         10  [5.25, 0, 0, 1]         NULL (pending, never published)
2         11  [5.25, 0, 0, 1]         NULL (pending, never published)
```

Ids 3, 6 and 9 are deleted.

Cell flags are an unstable format, so this fixture is disposable: when the
format changes, regenerate it rather than keeping readers compatible with it.
From the repository root:

```bash
rm -rf python/python/tests/torch_tests/fixtures/masked_embeddings.lance
LANCE_MASKED_EMBEDDINGS_FIXTURE=$PWD/python/python/tests/torch_tests/fixtures/masked_embeddings.lance \
    cargo test -p lance --lib write_masked_embeddings_fixture -- --ignored
```

Release builds read it only with `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` set, which
the tests set.
