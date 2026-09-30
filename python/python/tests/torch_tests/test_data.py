# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: Copyright The Lance Authors

import shutil
from itertools import chain
from pathlib import Path

import lance
import lance.arrow
import numpy as np
import pyarrow as pa
import pytest
from lance.file import LanceFileReader
from lance.sampler import ShardedBatchSampler, ShardedFragmentSampler

torch = pytest.importorskip("torch")
from lance.torch.data import (  # noqa: E402
    LanceDataset,
    SafeLanceDataset,
    _bf16_to_tensor,
    _to_tensor,
)


def test_iter_over_dataset_fixed_shape_tensor(tmp_path):
    data = np.random.random((10240, 32)).astype("f")

    tensor_array = pa.FixedShapeTensorArray.from_numpy_ndarray(data)
    ids = pa.array(range(0, 10240), type=pa.int32())
    tbl = pa.Table.from_arrays([ids, tensor_array], ["ids", "vec"])

    lance.write_dataset(tbl, tmp_path / "data.lance")

    iter_over_dataset(tmp_path)


def test_iter_over_dataset_fixed_size_lists(tmp_path):
    # 10240 of 32-d vectors.
    data = np.random.random(10240 * 32).astype("f")

    fsl = pa.FixedSizeListArray.from_arrays(data, 32)
    ids = pa.array(range(0, 10240), type=pa.int32())
    tbl = pa.Table.from_arrays([ids, fsl], ["ids", "vec"])

    lance.write_dataset(tbl, tmp_path / "data.lance", max_rows_per_group=32)

    iter_over_dataset(tmp_path)


def iter_over_dataset(tmp_path):
    ds = lance.dataset(tmp_path / "data.lance")

    # test when sample size is smaller than max_takes
    torch_ds_small = LanceDataset(
        ds, batch_size=256, samples=1024, columns=["ids", "vec"], cache=True
    )

    total_rows = 0
    for batch in torch_ds_small:
        assert set(batch.keys()) == {"ids", "vec"}
        # row groups of 32 can be batched into 256 exactly.
        assert batch["vec"].shape[0] == 256
        total_rows += batch["vec"].shape[0]
        assert batch["ids"].dtype == torch.int32
        assert batch["vec"].shape[1] == 32
    assert total_rows == 1024

    # test when sample size is greater than max_takes
    torch_ds = LanceDataset(
        ds,
        batch_size=256,
        samples=4096,
        columns=["ids", "vec"],
        cache=True,
        batch_readahead=2,
    )

    total_rows = 0
    for batch in torch_ds:
        assert set(batch.keys()) == {"ids", "vec"}
        # row groups of 32 can be batched into 256 exactly.
        assert batch["vec"].shape[0] == 256
        total_rows += batch["vec"].shape[0]
        assert batch["ids"].dtype == torch.int32
        assert batch["vec"].shape[1] == 32
    assert total_rows == 4096

    shutil.rmtree(tmp_path / "data.lance")

    total_rows = 0
    # it should read from cache this time.
    for batch in torch_ds_small:
        assert set(batch.keys()) == {"ids", "vec"}
        assert batch["ids"].dtype == torch.int32
        total_rows += batch["vec"].shape[0]
        assert batch["vec"].shape[1] == 32
    assert total_rows == 1024

    total_rows = 0
    # it should read from cache this time.
    for batch in torch_ds:
        assert set(batch.keys()) == {"ids", "vec"}
        assert batch["ids"].dtype == torch.int32
        total_rows += batch["vec"].shape[0]
        assert batch["vec"].shape[1] == 32
    assert total_rows == 4096


def test_iter_filter(tmp_path):
    arr = pa.array(range(1000))
    tbl = pa.Table.from_arrays([arr], ["ids"])

    ds = lance.write_dataset(tbl, tmp_path / "data.lance", max_rows_per_group=32)

    def check(dataset):
        total_rows = 0
        for batch in dataset:
            assert torch.where(batch >= 300, True, False).all()
            total_rows += batch.size(dim=0)
            assert batch.dtype == torch.int64
        assert total_rows == 700

    # No shard_grandularity
    check(
        LanceDataset(
            ds,
            batch_size=10,
            filter="ids >= 300",
            columns=["ids"],
        )
    )

    # shard_grandularity fragment ok
    check(
        LanceDataset(
            ds,
            batch_size=10,
            filter="ids >= 300",
            columns=["ids"],
            sampler=ShardedFragmentSampler(0, 1),
        )
    )

    # sampling with filter
    with pytest.raises(NotImplementedError):
        check(
            LanceDataset(
                ds,
                batch_size=10,
                filter="ids >= 300",
                samples=100,
                columns=["ids"],
            )
        )


def test_sample_fragments(tmp_path: Path):
    arr = pa.array(range(2000))
    tbl = pa.Table.from_arrays([arr], ["ids"])

    # Write 20 files
    lance.write_dataset(tbl, tmp_path, max_rows_per_file=100)

    ds = LanceDataset(
        tmp_path,
        batch_size=25,
        columns=["ids"],
        with_row_id=True,
        sampler=ShardedFragmentSampler(rank=1, world_size=2),
    )

    all_ids = list(chain.from_iterable([batch["ids"].cpu().numpy() for batch in ds]))
    assert all_ids == [i for i in range(2000) if i // 100 % 2 == 1]


def test_sample_batches(tmp_path: Path):
    arr = pa.array(range(2000))
    tbl = pa.Table.from_arrays([arr], ["ids"])

    # Write 20 files
    lance.write_dataset(tbl, tmp_path, max_rows_per_file=100)

    ds = LanceDataset(
        tmp_path,
        batch_size=25,
        columns=["ids"],
        with_row_id=True,
        sampler=ShardedBatchSampler(rank=1, world_size=2),
    )

    all_ids = list(chain.from_iterable([batch.cpu().numpy() for batch in ds]))
    assert all_ids == [i for i in range(2000) if i // 25 % 2 == 1]


def test_filtered_sampling_odd_batch_size(tmp_path: Path):
    tbl = pa.Table.from_pydict(
        {
            "vector": pa.array(
                [[1.0, 2.0, 3.0] for _ in range(10000)], pa.list_(pa.float32(), 3)
            ),
            "filterme": [i % 2 for i in range(10000)],
        }
    )

    lance.write_dataset(tbl, tmp_path, max_rows_per_file=200)

    ds = LanceDataset(
        tmp_path,
        batch_size=38,
        columns=["vector"],
        samples=38 * 256,
        filter="vector is not null",
    )

    x = next(iter(ds))

    assert x.shape[0] == 38
    assert x.shape[1] == 3


def test_sample_batches_with_filter(tmp_path: Path):
    NUM_ROWS = 10000
    tbl = pa.Table.from_pydict(
        {
            "id": range(NUM_ROWS),
            "filterme": [i % 2 for i in range(NUM_ROWS)],
        }
    )

    lance.write_dataset(tbl, tmp_path, max_rows_per_file=2000)

    ds = LanceDataset(
        tmp_path,
        batch_size=25,
        columns=["id"],
        with_row_id=True,
        filter="filterme == 0",
        sampler=ShardedBatchSampler(rank=3, world_size=5),
    )

    # The filtered sequence is 0, 2, 4, ...
    #
    # With rank 3 and world size 5 we should get
    #
    # - - - 6  -
    # - - - 16 -
    # - - - 26 -
    # ...
    all_ids = list(chain.from_iterable([batch.cpu().numpy() for batch in ds]))
    # Half of the data is filtered out, divided amongst 5 workers s
    # each should see 1/10th of the data
    assert len(all_ids) == 1000
    assert all_ids == [6 + (10 * i) for i in range(len(all_ids))]

    # Now test with random order
    ds = LanceDataset(
        tmp_path,
        batch_size=25,
        columns=["id"],
        with_row_id=True,
        filter="filterme == 0",
        sampler=ShardedBatchSampler(rank=3, world_size=5, randomize=True),
    )

    randomized_ids = list(chain.from_iterable([batch.cpu().numpy() for batch in ds]))
    assert randomized_ids != all_ids
    randomized_ids.sort()
    assert randomized_ids == all_ids


@pytest.mark.parametrize("dtype", [np.uint8, np.int64])
def test_convert_int_tensors(tmp_path: Path, dtype):
    data = np.random.randint(0, 256, size=128 * 32, dtype=dtype)
    fsl = pa.FixedSizeListArray.from_arrays(data, 32)
    ids = pa.array(range(0, 128), type=pa.int32())
    tbl = pa.Table.from_arrays([ids, fsl], ["ids", "vec"])

    ds = lance.write_dataset(tbl, tmp_path / "data.lance", max_rows_per_group=32)

    torch_ds = LanceDataset(
        ds,
        batch_size=4,
    )
    first = next(iter(torch_ds))
    assert first["vec"].dtype == torch.uint8 if dtype == np.uint8 else torch.int64
    assert first["vec"].shape == (4, 32)


def test_blob_api(tmp_path: Path):
    ints = pa.array(range(100), type=pa.int64())
    vals = pa.array([b"0" * 1024 for _ in range(100)], pa.large_binary())
    schema = pa.schema(
        [
            pa.field("int", ints.type),
            pa.field(
                "val", pa.large_binary(), metadata={"lance-encoding:blob": "true"}
            ),
        ]
    )
    tbl = pa.Table.from_arrays([ints, vals], schema=schema)

    uri = tmp_path / "data.lance"
    dataset = lance.write_dataset(tbl, uri, data_storage_version="2.1")

    torch_ds = LanceDataset(
        uri, batch_size=4, dataset_options={"version": dataset.version}
    )
    with pytest.raises(NotImplementedError):
        next(iter(torch_ds))

    def to_tensor_fn(batch, *args, **kwargs):
        ints = torch.tensor(batch["int"].to_numpy())
        vals = []
        for blob in batch["val"]:
            blob.seek(100)
            data = blob.read(100)
            tensor = torch.tensor(np.frombuffer(data, dtype=np.uint8))
            vals.append(tensor)

            # vals.append(torch.tensor(blob))
        vals = torch.stack(vals)
        return {"int": ints, "val": vals}

    torch_ds = LanceDataset(
        dataset,
        batch_size=4,
        to_tensor_fn=to_tensor_fn,
    )
    first = next(iter(torch_ds))
    assert first["int"].dtype == torch.int64
    assert first["int"].shape == (4,)
    assert first["val"].dtype == torch.uint8
    assert first["val"].shape == (4, 100)


def test_iter_over_dataset_bfloat16(tmp_path):
    """Test that bfloat16 vector columns convert to torch.bfloat16 tensors."""
    ml_dtypes = pytest.importorskip("ml_dtypes")
    from lance.arrow import BFloat16Array

    dim = 32
    num_rows = 128
    # Create random bfloat16 vectors via float32 → bfloat16 cast
    f32_data = np.random.random(num_rows * dim).astype("f")
    bf16_data = f32_data.astype(ml_dtypes.bfloat16)

    # Build a FixedSizeList<bf16> column
    inner = BFloat16Array.from_numpy(bf16_data)
    fsl = pa.FixedSizeListArray.from_arrays(inner, dim)
    ids = pa.array(range(num_rows), type=pa.int32())
    tbl = pa.Table.from_arrays([ids, fsl], ["ids", "vec"])

    ds = lance.write_dataset(tbl, tmp_path / "data.lance", max_rows_per_group=32)

    torch_ds = LanceDataset(ds, batch_size=16, columns=["ids", "vec"])

    total_rows = 0
    for batch in torch_ds:
        assert set(batch.keys()) == {"ids", "vec"}
        assert batch["vec"].dtype == torch.bfloat16
        assert batch["vec"].shape[1] == dim
        assert batch["ids"].dtype == torch.int32
        total_rows += batch["vec"].shape[0]
    assert total_rows == num_rows


def test_scalar_bfloat16_column(tmp_path):
    """Test that a scalar bfloat16 column converts to torch.bfloat16 tensor."""
    ml_dtypes = pytest.importorskip("ml_dtypes")
    from lance.arrow import BFloat16Array

    num_rows = 64
    f32_data = np.random.random(num_rows).astype("f")
    bf16_data = f32_data.astype(ml_dtypes.bfloat16)

    arr = BFloat16Array.from_numpy(bf16_data)
    tbl = pa.Table.from_arrays([arr], ["val"])

    ds = lance.write_dataset(tbl, tmp_path / "data.lance")

    torch_ds = LanceDataset(ds, batch_size=16, columns=["val"])

    total_rows = 0
    for batch in torch_ds:
        assert batch.dtype == torch.bfloat16
        total_rows += batch.shape[0]
    assert total_rows == num_rows


def test_bf16_to_tensor_zero_copy_without_nulls():
    """Non-null bf16 arrays should alias the Arrow data buffer."""
    ml_dtypes = pytest.importorskip("ml_dtypes")
    from lance.arrow import BFloat16Array

    values = np.array([1.0, 2.0, 3.0, 4.0], dtype=ml_dtypes.bfloat16)
    arr = BFloat16Array.from_numpy(values).slice(1, 2)

    tensor = _bf16_to_tensor(arr)

    assert tensor.dtype == torch.bfloat16
    assert torch.equal(
        tensor.to(torch.float32),
        torch.tensor([2.0, 3.0], dtype=torch.float32),
    )
    assert (
        tensor.data_ptr() == arr.storage.buffers()[1].address + arr.storage.offset * 2
    )


def test_bf16_to_tensor_clones_when_nulls_present():
    """Null replacement requires a writable tensor, so the Arrow buffer is cloned."""
    arr = lance.arrow.bfloat16_array([1.0, None, 3.0])

    tensor = _bf16_to_tensor(arr)

    assert tensor.dtype == torch.bfloat16
    assert (
        tensor.data_ptr() != arr.storage.buffers()[1].address + arr.storage.offset * 2
    )
    assert tensor[0].to(torch.float32).item() == pytest.approx(1.0)
    assert torch.isnan(tensor[1])
    assert tensor[2].to(torch.float32).item() == pytest.approx(3.0)


DIM = 3
NUM_VECTORS = 6
NULL_ROWS = [1, 4]

TORCH_DTYPES = {
    "float16": torch.float16,
    "float32": torch.float32,
    "float64": torch.float64,
    "bfloat16": torch.bfloat16,
    "int32": torch.int32,
}


def _vectors(dtype: str, null_rows=(), null_items=()) -> pa.FixedSizeListArray:
    """Six 3-d vectors of `dtype` whose items are 1 to 18 in order, NULL on
    `null_rows` over whatever items sit under them, and with the items at the
    positions `null_items` NULL."""
    items = np.arange(1, NUM_VECTORS * DIM + 1)
    item_nulls = np.isin(np.arange(len(items)), null_items)
    if dtype == "bfloat16":
        children = lance.arrow.bfloat16_array(
            [None if null else float(item) for item, null in zip(items, item_nulls)]
        )
    else:
        children = pa.array(items.astype(dtype), mask=item_nulls)
    list_nulls = pa.array(np.isin(np.arange(NUM_VECTORS), null_rows))
    return pa.FixedSizeListArray.from_arrays(children, DIM, mask=list_nulls)


def _items_of(rows) -> list[int]:
    return [row * DIM + i for row in rows for i in range(DIM)]


def _expected(start: int, length: int, null_rows=(), null_items=()) -> torch.Tensor:
    """Rows `start` to `start + length` of `_vectors` as float64, NaN where a
    list or an item is NULL."""
    items = np.arange(1, NUM_VECTORS * DIM + 1, dtype=np.float64)
    items[list(null_items)] = np.nan
    rows = items.reshape(NUM_VECTORS, DIM)
    rows[list(null_rows)] = np.nan
    return torch.from_numpy(rows[start : start + length])


def _assert_rows(tensor: torch.Tensor, expected: torch.Tensor):
    torch.testing.assert_close(
        tensor.to(torch.float64), expected, rtol=0, atol=0, equal_nan=True
    )


@pytest.mark.parametrize("dtype", ["float16", "float32", "float64", "bfloat16"])
@pytest.mark.parametrize(
    "null_items", [False, True], ids=["over_stored_items", "over_null_items"]
)
@pytest.mark.parametrize(
    ("start", "length"),
    [
        pytest.param(0, 6, id="whole"),
        pytest.param(1, 5, id="from_a_null"),
        pytest.param(2, 4, id="after_a_null"),
        pytest.param(3, 3, id="unaligned"),
        pytest.param(2, 2, id="between_nulls"),
    ],
)
def test_null_float_vectors_become_nan_rows(dtype, null_items, start, length):
    arr = _vectors(dtype, NULL_ROWS, _items_of(NULL_ROWS) if null_items else ())
    stored = arr.values.to_pylist()

    tensor = _to_tensor(pa.record_batch({"vec": arr.slice(start, length)}))

    assert tensor.dtype == TORCH_DTYPES[dtype]
    _assert_rows(tensor, _expected(start, length, NULL_ROWS))
    # The NaN rows must not be written into the Arrow buffer the tensor may
    # have been viewing; `arr.to_pylist()` would hide that under the NULLs.
    assert arr.values.to_pylist() == stored


@pytest.mark.parametrize("dtype", ["uint8", "int32", "int64"])
@pytest.mark.parametrize(
    ("null_rows", "null_items"),
    [
        pytest.param(NULL_ROWS, [], id="lists_over_stored_items"),
        pytest.param(NULL_ROWS, _items_of(NULL_ROWS), id="lists_over_null_items"),
        pytest.param([], [4], id="items"),
    ],
)
def test_null_integer_vectors_become_float64_nan_rows(dtype, null_rows, null_items):
    arr = _vectors(dtype, null_rows, null_items).slice(1)

    tensor = _to_tensor(pa.record_batch({"vec": arr}))

    assert tensor.dtype == torch.float64
    _assert_rows(tensor, _expected(1, 5, null_rows, null_items))


def test_null_fixed_shape_tensors_become_nan_rows():
    tensor_type = pa.fixed_shape_tensor(pa.float32(), [DIM])
    arr = pa.ExtensionArray.from_storage(tensor_type, _vectors("float32", NULL_ROWS))

    tensor = _to_tensor(pa.record_batch({"vec": arr.slice(2)}))

    assert tensor.dtype == torch.float32
    _assert_rows(tensor, _expected(2, 4, NULL_ROWS))


@pytest.mark.parametrize(
    "dtype", ["float16", "float32", "float64", "bfloat16", "int32"]
)
def test_vectors_without_nulls_are_not_copied(dtype):
    # The rows between the NULL lists hold no NULL.
    arr = _vectors(dtype, NULL_ROWS).slice(2, 2)

    tensor = _to_tensor(pa.record_batch({"vec": arr}))

    assert tensor.dtype == TORCH_DTYPES[dtype]
    _assert_rows(tensor, _expected(2, 2))
    items = arr.values.storage if dtype == "bfloat16" else arr.values
    first_item = items.offset + arr.offset * DIM
    assert (
        tensor.data_ptr()
        == items.buffers()[1].address + first_item * tensor.element_size()
    )


@pytest.mark.parametrize("dtype", ["float16", "float32", "float64", "bfloat16"])
def test_null_vectors_read_as_nan_rows(tmp_path: Path, dtype):
    table = pa.table(
        {
            "id": pa.array(range(NUM_VECTORS), pa.int32()),
            "vec": _vectors(dtype, NULL_ROWS),
        }
    )
    lance.write_dataset(table, tmp_path, max_rows_per_file=4)

    batches = list(LanceDataset(tmp_path, batch_size=3, columns=["id", "vec"]))

    assert torch.cat([batch["id"] for batch in batches]).tolist() == list(
        range(NUM_VECTORS)
    )
    vectors = torch.cat([batch["vec"] for batch in batches])
    assert vectors.dtype == TORCH_DTYPES[dtype]
    _assert_rows(vectors, _expected(0, NUM_VECTORS, NULL_ROWS))


# Written by `write_masked_embeddings_fixture` in rust/lance; see fixtures/README.md.
MASKED_EMBEDDINGS = Path(__file__).parent / "fixtures" / "masked_embeddings.lance"
MASKED_EMBEDDINGS_LIVE_IDS = [0, 1, 2, 4, 5, 7, 8, 10, 11]
MASKED_EMBEDDINGS_PENDING_IDS = [5, 7, 8, 10, 11]
# What the flag hides under pending rows that were never published.
MASKED_EMBEDDINGS_QUERY = [5.25, 0.0, 0.0, 1.0]


def _masked_embedding_rows(ids) -> torch.Tensor:
    """What reads of the masked embedding fixture show for `ids`: only ids 0,
    2 and 4 have a visible embedding, `[id, 0, 0, 1]`. Id 1 is computed NULL
    and the others are pending over stored values."""
    return torch.tensor(
        [
            [float(row_id), 0.0, 0.0, 1.0] if row_id in (0, 2, 4) else [np.nan] * 4
            for row_id in ids
        ],
        dtype=torch.float64,
    )


@pytest.fixture
def masked_embeddings(monkeypatch) -> lance.LanceDataset:
    # Release builds refuse a dataset with cell flags without it.
    monkeypatch.setenv("LANCE_ENABLE_UNSTABLE_CELL_FLAGS", "1")
    return lance.dataset(MASKED_EMBEDDINGS)


@pytest.mark.parametrize("batch_size", [2, 16])
def test_masked_embeddings_read_as_nan_rows(masked_embeddings, batch_size):
    # Fragment 2 was never published: the flag hides the vectors it was
    # written with.
    [data_file] = masked_embeddings.get_fragment(2).data_files()
    stored = LanceFileReader(
        str(MASKED_EMBEDDINGS / "data" / data_file.path), columns=["embedding"]
    )
    assert (
        stored.read_all().to_table()["embedding"].to_pylist()
        == [MASKED_EMBEDDINGS_QUERY] * 4
    )

    batches = list(
        LanceDataset(
            masked_embeddings, batch_size=batch_size, columns=["id", "embedding"]
        )
    )

    ids = torch.cat([batch["id"] for batch in batches]).tolist()
    assert ids == MASKED_EMBEDDINGS_LIVE_IDS
    embeddings = torch.cat([batch["embedding"] for batch in batches])
    assert embeddings.dtype == torch.float32
    _assert_rows(embeddings, _masked_embedding_rows(ids))


@pytest.mark.parametrize("children", ["as_read", "populated"])
def test_masked_embeddings_do_not_rely_on_null_children(masked_embeddings, children):
    table = masked_embeddings.to_table(columns=["id", "embedding"])
    embedding = table["embedding"].combine_chunks()
    null_items = embedding.values.is_null().to_numpy(zero_copy_only=False)
    pending = np.isin(table["id"].to_numpy(), MASKED_EMBEDDINGS_PENDING_IDS)
    # Masking nulls only the slot: id 5's stale vector stays under it.
    assert not null_items[np.repeat(pending, 4)].all()
    if children == "populated":
        # As if every NULL embedding, computed or masked, were read over a
        # stored vector.
        null_rows = embedding.is_null().to_numpy(zero_copy_only=False)
        items = embedding.values.to_numpy(zero_copy_only=False).reshape(-1, 4)
        items[null_rows] = MASKED_EMBEDDINGS_QUERY
        embedding = pa.FixedSizeListArray.from_arrays(
            pa.array(items.ravel(), pa.float32()), 4, mask=embedding.is_null()
        )

    tensor = _to_tensor(pa.record_batch({"embedding": embedding}))

    _assert_rows(tensor, _masked_embedding_rows(table["id"].to_pylist()))


def test_safe_lance_dataset_worker_uses_dataset_options(tmp_path: Path):
    """Worker processes must reopen the dataset with dataset_options.

    Regression test for: worker init called lance.dataset(uri) without
    dataset_options, silently dropping version, storage_options, etc.
    """
    tbl_v1 = pa.table({"id": pa.array([1, 2, 3], pa.int64())})
    ds = lance.write_dataset(tbl_v1, tmp_path / "data.lance")
    version_1 = ds.version

    # Write a second version with different data so we can distinguish them.
    tbl_v2 = pa.table({"id": pa.array([10, 20, 30], pa.int64())})
    lance.write_dataset(tbl_v2, tmp_path / "data.lance", mode="overwrite")

    # Pin to version 1 via dataset_options.
    safe_ds = SafeLanceDataset(
        str(tmp_path / "data.lance"),
        dataset_options={"version": version_1},
    )

    # Simulate worker-process state: _ds is None so __getitems__ must reopen.
    safe_ds._ds = None
    rows = safe_ds.__getitems__([0, 1, 2])

    assert [r["id"] for r in rows] == [1, 2, 3], (
        "Worker reopened dataset without dataset_options (got version 2 data)"
    )
