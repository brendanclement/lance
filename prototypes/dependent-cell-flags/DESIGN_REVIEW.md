# Dependency-aware cell flags: design-review handoff

> **Research prototype, not for production.** The format is unstable and gated by
> `FLAG_UNSTABLE_CELL_FLAGS`; release builds refuse flagged datasets unless
> `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` is set. Nothing here is pushed or proposed as a PR.

- **Branch:** `brendan/dependency-aware-cell-flags`.
- **Baseline:** `e3671b2f5`.
- **Final code:** `b56278dc8` (the last commit that changes code).
- **Research record** (kept separately):
  - [`README.md`](README.md): the detailed log of every decision, check and departure;
  - [`bench/REPORT.md`](bench/REPORT.md) and `bench/results/`: every benchmark run with raw
    samples;
  - [`bench/results/10m-noflag-investigation/`](bench/results/10m-noflag-investigation/README.md)
    and [`bench/results/vector-masking/`](bench/results/vector-masking/README.md);
  - [`bench/results/feature-costs/`](bench/results/feature-costs/README.md): the cost of each
    operation, with the cost table.

## What it is

A *dependent* cell flag is a per-row Boolean on an output field, cleared in the same commit as any
write to the fields it is computed from. With `mask_when_false`, reads return NULL for the output
wherever the flag is false.

A refresh reads a snapshot, computes outputs outside Lance, and publishes them as an ordinary
`DataReplacement` that sets the flags true. Lance validates the publication against everything
committed since the snapshot. It either rejects the publication or publishes the safe part and
reports the rest.

## API

```rust
// Register: `summary` is computed from `title` and `body`, and reads NULL until published.
let ready = dataset.register_cell_flag("summary", "ready",
    CellFlagOptions::default().with_clear_on_write(["title", "body"]).with_mask_when_false(true)).await?;

// Refresh: stream computed rows (`_rowaddr` + output columns, optional per-output masks)
// against a fixed snapshot; the stager copies every other cell from that snapshot.
let snapshot = Arc::new(dataset.clone());
let stager = PublicationStager::try_new(snapshot.clone(), &["summary"])?;
if let Some(publication) = stager.stage(computed).await? {
    let result = CommitBuilder::new(snapshot)
        .with_dependency_conflict_policy(DependencyConflictPolicy::Skip) // default: Reject
        .execute_with_report(publication)
        .await?;
    // Follow up at exactly the version the report names, then pick up remaining pending rows
    // (moved or vacated rows) with a live scan filtered by `cell_flag_true_rows`.
    let plan = PublicationStager::try_new(Arc::new(result.dataset), &["summary"])?
        .follow_up(&result.report).await?;          // per output: reuse / recompute rows
}

// Vectors: a nullable FixedSizeList<Float16|32|64> embedding can be masked the same way;
// unindexed nearest-neighbor search is a flat search that never returns masked rows.
scanner.nearest("embedding", &query, 10)?;
```

The public surface:
- **Registration and state:** `register_cell_flag`, `replace_cell_flag`, `drop_cell_flag`,
  `cell_flags`, `cell_flag_true_rows`.
- **Transactions:** `CellFlagChanges` / `CellFlagUpdate` on `Transaction`.
- **Commit:** `CommitBuilder::with_dependency_conflict_policy` and `execute_with_report`, which
  returns a `PublicationResult { dataset, report }`.
- **Staging:** `PublicationStager` (`try_new`, `flag_id`, `stage`, `follow_up`), `ComputedBatch`
  (`new`, `with_assigned`), `FollowUpPlan`, `FollowUpRows`.

Everything is Rust only.

## Guarantees

**Invalidation.**
- A write to a watched field clears the flag on the written rows in that same commit, and records
  the clear even when the flag was already false.
- Clears propagate down chains of computed outputs.
- Registering or dropping a masking flag on a source clears the flags watching it.

**Publication.**
- A dependent flag becomes true only through a publication: a `DataReplacement` that writes its
  output, validated against its read version.
- The read version never changes across retries.
- Replacing or dropping a registration fences older publishers.

**Conflicts.**
- `Reject` (the default) fails an unsafe publication as a retryable conflict and exposes nothing.
  The one exception: rows deleted since the read are dropped and reported `RowVacated`.
- `Skip` publishes the safe groups and defers the rest, each with a reason: `InputChanged`,
  `RowVacated`, `UpstreamNotPublished`, `NewerResult`, `OutputWritten`, `FragmentRemoved` or
  `FragmentRewritten`.
- Under either policy, a replacement file never overwrites a newer result anywhere in its
  footprint.

**Report.** Rows `reusable_rows` certifies are safe to reuse at the version the report names.
Rows deferred `InputChanged` or `UpstreamNotPublished` must be recomputed. A recorded clear and an
input change are tracked as different things.

**Masking.** A false masking flag makes the cell read NULL on every read path:
- scans, filters and aggregates;
- `take`, `take_rows` and late materialization;
- SQL and flat full-text search;
- unindexed vector search.

Masking nulls a vector's list slot only: the stored vector, stale or never published, can remain
in the child array under it (`b56278dc8`).

**Torch consumers.** `lance.torch` maps a NULL vector (masked, computed NULL, or ordinary) to a NaN
row whatever values sit under its slot. Integer vectors in a batch holding a NULL become float64
with NaN rows. Arrays without NULLs stay zero-copy.

**Stager.**
- Copy-through holds by construction. Unassigned cells are copied from the same snapshot as read
  through Lance, and each copy window is checked against its row addresses.
- Assignment comes from column presence and masks, never from values, so a computed NULL is
  distinct from an unassigned cell.
- Refused before anything is written:
  - bad row addresses: out of range, deleted, duplicated, out of order or revisited;
  - wrong types;
  - outputs without a dependent flag;
  - blob and JSON outputs, and V1 datasets;
  - chains through an undeclared output, or a row that skips the declared output between two
    outputs.
- Memory is bounded per copy window, not per fragment. The bound excludes the scan's own
  read-ahead.
- A stored output and a newly added all-NULL output publish together in one file per fragment.
  The ordinary `DataReplacement` build now tombstones the stored fields and appends the file
  (`ded2e02e93a2583f4912420d2323ec3b79265d79`).

## Caller obligations (Lance cannot verify these)

- **Values:** computed values must be functions of the snapshot's inputs as read through Lance.
  Wrong values publish under true flags.
- **Chains:** a downstream output assigned in the same row as its upstream must be computed from
  that upstream value.
- **Follow-ups:**
  - Run at exactly `committed_version.unwrap_or(checked_version)`.
  - Then find the remaining pending rows with a live scan filtered by `cell_flag_true_rows`,
    because moved rows get new addresses.
  - A follow-up that assigns an upstream must also assign the declared downstream outputs on the
    same row.
- **Hand-built publications** (without the stager) must copy every unassigned row unchanged.
- **Staged files:** files of a publication that is dropped, rejected or partly deferred stay until
  `cleanup_old_versions` removes them, after 7 days by default. There is no lease.
- **Vectors:** readers must respect the list validity. A reader that takes a fixed-size list's
  values without it (a `.values` reshape, a user `to_tensor_fn`, an Arrow C Data consumer in
  another engine) can see a masked slot's stored vector.

## Unsupported (explicit errors)

**Indexes:**
- any index on a masked output: scalar, full-text or vector (IVF, HNSW, prebuilt segments), at
  build and at commit;
- masking an indexed field.

**Operations:** compaction (`Rewrite`), `Overwrite`, `DataOverlay` on a watched field, dropping or
casting a watched field, and a `Merge` that rewrites a source.

**Types:** masking a non-nullable, list, struct, blob or V1 field, or a vector whose items are not
Float16/32/64.

**Other:**
- MemWAL or LSM together with masking;
- detached or batch commits carrying flag changes;
- `Skip` without `execute_with_report`;
- Python and Java APIs (bindings drop flag changes on a round trip).

## Tests actually run

**On the final code, `b56278dc8`** (macOS), after removing child-nulling:
- `cargo fmt --all` and `cargo clippy -p lance --tests --benches -- -D warnings`: clean.
- `cargo test -p lance --lib -- cell_flag`: 422 passed, 1 ignored (the fixture generator).
- `python/python/tests/torch_tests/` and `test_torch.py`, against a `pylance` built from
  `b56278dc8`: 90 passed, 3 skipped (CUDA). The torch fixture test now reads a stale vector under a
  masked slot.

Everything below ran on `19bb42a65`, which differs only in `mask_batch`, its rustdoc and the
tests' child-value assertions.

**macOS (Apple M5 Pro):**
- `cargo fmt --all -- --check` and `cargo clippy --all --tests --benches -- -D warnings`: clean.
- `cargo test -p lance --lib`: 4584 passed, 4 ignored.
- `cargo test -p lance-table`: 580 passed, plus 2 doctests.
- `cargo test -p lance-index --lib vector::flat`: 30 passed.
- `cargo test -p lance --doc -- cell_flag staging PublicationStager ComputedBatch FollowUpPlan`: 15
  passed.
- `python/python/tests/torch_tests/` and `test_torch.py`: 90 passed, 3 skipped (CUDA).
  - These ran against the cached torch 2.11.0 through `uv run --with`. The locked torch 2.14.0 is
    not cached, and its roughly 128 MB download was not made.
  - `uv run make lint` passed on the Torch change.

**Linux (a local aarch64 Docker container, Rust 1.97):**
- `cell_flag`, `fragment_write_columns` and the DataReplacement tests: 471 passed, 1 ignored (the
  fixture generator).
- `lance-table`: 580 passed, plus 2 doctests.
- `lance-index` `vector::flat`: 30 passed.
- Doctests: 15 passed.
- `lance` benches compile.

**MSRV:** CI's msrv command, `cargo check --profile ci --workspace --tests --benches` with every
workspace feature except `protoc`, passes under Rust 1.91.0 in the same container.

**Mutation checks:** the checks listed in the README's Tests table were disabled in turn, and at
least one test failed each time.

**History caveat:** one intermediate commit, `58c874a67`, fails clippy on its own. The lint fixes
for its tests landed in `792c462d9`, and history was not rewritten.

**Not run:**
- Linux x86_64;
- `python/` beyond the torch tests;
- `java/` (no Java runtime here; only a Javadoc changed);
- torch 2.14;
- any representative performance environment.

**Flat-search fix, separately:** `brendan/flat-search-null-offset` (`8d09fb623`) is based on
`2c4934cfe`, which was `origin/main` when the fix was extracted. It merges cleanly onto the
current `origin/main`, three commits ahead. It passed on its own:
- clippy for `lance-index` and `lance`;
- 51 `vector::flat` tests;
- 318 scanner tests;
- 189 `io::exec::knn` tests.

Its new scanner test and two of its three unit cases fail on `main` without the fix.

## Performance evidence and uncertainties

Measurements of the current prototype (`b56278dc8`), on one Apple M5 Pro laptop running macOS
with a local SSD and a warm page cache, in `release-with-debug`, on 2026-09-30. None of it
describes production builds or object storage. The Linux wheels use thin LTO, 1 codegen unit,
`haswell` on x86_64, and the `metrics` features. **Performance acceptance is open:** no
representative environment or agreed threshold was available. The full cost table, its
evidence and what was not measured are in
[`bench/results/feature-costs/`](bench/results/feature-costs/README.md). At 1M rows in 10
fragments, against tables without flags:

- **Reads without flags:** 1.014–1.023× `main` at 10M rows, with intervals that include 1.0; an
  identical binary reads 1.017–1.027× `main`. Instructions +0.1%, allocations identical.
  Unresolved, as before; `2fd300ac2` stays a maintainability choice, not a proven speed-up.
- **Masked reads:** free when every fragment's flags are whole (0.98–0.99×). When fragments have
  lost rows, reads cost 1.72× even with every flag true: a per-row mask is built from the
  fragment's row-set state. 1% masked costs 1.65–1.69× equal ordinary NULLs. A fully masked
  column still decodes its stored values: 2.5–3.0× all-NULL scans, 16× takes. Masked vectors
  cost 1.01× an all-valid table at 1% masked, and 0.46× equal NULL vectors. Takes are unaffected
  unless every row is masked.
- **Writes:** unrelated writes cost 0.99–1.01×, and sparse source writes 1.00×. A dense source
  write (100k rows moved) costs 1.24–1.25× (+15 ms), and leaves 163 KB of manifest flag state for
  one flag, 323 KB for two, against 2.5 KB.
- **Refresh:** staging pending rows that sit in a new fragment is 20× faster than the
  `merge_insert` a plain table would use (0.56 ms against 11 ms). Staging 100 rows scattered in
  place rewrites every touched fragment: 65.6 ms against 23.9 ms for `merge_insert`
  `RewriteColumns`, which rewrites them too, and 117× the same rows staged in a new fragment.
  Backfills stage 3.5–5.4× slower than a plain `write_columns`. Commits (validation and
  publication) cost 0.99–1.15×. Finding pending rows with the documented live scan costs 10.7 ms
  per 1M rows. Complete cycles cost 0.84–0.96× (row-moving sparse), 1.16–1.30× (dense) and
  1.40–1.49× (in place) the plain cycle.
- **Races:** a `Reject` retry redoes the whole refresh (0.97–1.06× the first attempt), and
  recomputes twice the rows. A `Skip` follow-up of an in-place race costs 0.59–0.73× of the first
  refresh at 10 fragments and 0.20–0.22× at 40, because it restages every fragment a raced row is
  in.
- **Over time:** a publication after 32 unrelated commits costs the same as plain Lance
  (1.00–1.01×). Opens after 16–64 updates cost 1.09–1.31× (+0.02–0.05 ms), and appends and
  updates 0.96–1.05×. Holes at moved rows stay in the flag state after a refresh.
- **Flat search with any NULL vector is slow, flags or not:** 219–230 ms against 15 ms. This looks
  like a separate flat KNN issue on `main`.

What these costs come from:
- The semantics require the flag clears and conflict checks.
- Inline storage causes the manifest and transaction growth.
- Whole-fragment `DataReplacement` causes the in-place and `Skip` rewrites.
- The prototype's own code causes the stager's speed and the row-set mask path.
- The refresher's own code causes the pending-row scan.

## Decisions needed before production work

1. **Vector child values (decided: validity only).** `b56278dc8` removed child-nulling, so
   reading a masked vector correctly is a caller obligation (see above). `lance.torch` respects
   vector validity (`90c31aad8`, and `58e4a4dc1` for `KMeans`). Confirm the contract, and whether
   other consumers need it documented.
2. **`DataReplacement` partial coverage.** This changes `main`'s rule for every caller: a file
   that writes a stored field next to an unstored one now tombstones and appends instead of
   failing.
   - It needs a `main` reviewer.
   - Consider validating the new file's field ids against the schema.
   - The Java `DataReplacement` Javadoc now matches (`19bb42a65`); no Java runtime was available to
     check its formatting.
3. **Flag state storage.** Inline state grows with holes and flags. A dense update leaves
   163 KB per flag. A 50% in-place clear leaves 163 KB, and writes a 331 KB transaction twice:
   inline and as the transaction file. Holes at moved rows are permanent without compaction.
   Locally this added only 0.02–0.15 ms to opens and commits. Object-storage latency on every
   open and commit was not measured. Decide whether to spill the state to external per-fragment
   files before production.
4. **Publication granularity and stager speed.** Whole-fragment `DataReplacement` makes a refresh
   of 100 scattered rows rewrite 13.8 MiB per output, and makes `Skip` follow-ups rewrite again.
   The stager is also 2.6–5.4× slower than plain Lance writes of the same columns, though it uses
   far less memory. Decide whether a finer-grained publication, a faster stager, or both are
   needed. Measurement only so far.
5. **Index maintenance over masked outputs** (scalar, full-text and ANN) and **compaction** that
   remaps flag state. Both are refused today.
6. **Bindings.** Python and Java APIs are needed for real use.
7. **Staged-file lifecycle.** Leases, or explicit discard, for uncommitted publications.
8. **Performance acceptance.** Agree an environment (Linux x86_64 wheel builds, object storage), a
   threshold and a wheel-matching Rust profile. Run `bench/run_costs.sh` there; each rotation has
   an identical-binary control.
9. **Torch NULL representation** (`90c31aad8`).
   - Float NULL vectors become NaN rows.
   - Integer vectors with NULLs become float64 with NaN rows, a dtype that depends on the batch.
   - Accept or change this. It was tested only on the cached torch 2.11, not the locked 2.14.
10. **Upstream fixes, independent of cell flags.**
   - The flat-search null-offset fix is ready on the local branch `brendan/flat-search-null-offset`
     (`8d09fb623`), also saved as `upstream/0001-flat-search-null-offset.patch`.
   - The flat KNN slowdown with NULL vectors needs its own investigation.
  - A `take` of 1,000 rows from a table with 1% of rows moved takes 1.5 s, with or without
    flags. Not investigated.
