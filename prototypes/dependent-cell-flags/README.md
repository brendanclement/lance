# Dependency-aware cell flags (option D) prototype

> **Not for production.** This is a research prototype on an unstable, feature-gated format
> (`FLAG_UNSTABLE_CELL_FLAGS`). The on-disk encoding, APIs and conflict rules may change or be
> removed without migration. Release builds refuse flagged datasets unless
> `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` is set. It does not implement full option D.

## Scope

Lance tracks, per output field, a Boolean *cell flag* with a stable flag id. A registration may
declare:

- `clear_on_write`: source fields. Any logical write to a source (or to the output itself) clears
  the flag for the written rows, in the same commit as the write. The clear is recorded in the
  transaction even if the flag was already false.
- `mask_when_false`: reads return NULL for the output field where the flag is false.

Flags without `clear_on_write` are *ordinary*: only explicit updates change them.

A refresh reads a snapshot, computes outputs outside Lance, stages full-fragment column files, and
commits a `DataReplacement` carrying `CellFlagUpdate { value: true }` assignments. Lance validates
the assignments against every transaction since the refresh's `read_version`. The default policy
rejects any conflict. The `Skip` policy publishes safe groups and reports deferred rows and
fragments.

Supported in this prototype:

- Multiple source fields per flag; several flags sharing sources.
- Source writes through `UpdateBuilder` (row-moving), partial-schema `merge_insert`
  (`RewriteColumns`, in place), `DataReplacement` of source columns, append and delete.
- Refresh publication through `DataReplacement`, including sibling outputs in one file.
- Masking in scans, filters (including `IS NULL`), aggregates, `take`, and SQL over the Lance
  table provider, on multi-fragment datasets with deletions.
- Registration replace and drop, which fence older publishers.
- `Reject` and `Skip` conflict policies.

## Unsupported operations

On a dataset with registered cell flags these fail with an explicit error rather than silently
producing wrong results:

- Compaction (`Rewrite`), `Overwrite`, detached commits, MemWAL state updates.
- `DataOverlay` writes that touch a flagged field.
- Dropping, casting, or rewriting (via `Merge`) a source or output field. Making a masked output
  non-nullable.
- Creating any index on a masked output field. Registering a masking flag on an indexed field.
- Row-moving writes that cannot carry flag state (`merge_insert` in `RewriteRows` mode, raw staged
  `Update` operations) on fragments where an *ordinary* flag is true. Dependent flags on moved rows
  fall back to unassigned, which is safe but loses completed work.
- Masked outputs that are non-nullable, nested, list/struct, blob, or on legacy (v1) storage.
- MemWAL (LSM) reads over a base table with a masking flag: `LsmScanner`, the point-lookup,
  vector and full-text planners, and `contains_pks`. MemWAL rows carry no flag state, so every
  such read fails, even with no shards or no masked column projected. Known gap: a fresh-tier-only
  reader (`LsmScanner::without_base_table`) has no manifest to consult, so it is not refused and
  serves MemWAL rows unmasked.
- Assigning a dependent flag true outside a `DataReplacement` that writes its output.
- `Skip` through `CommitBuilder::execute` (it cannot return the report); use
  `execute_with_report`.

Not implemented: Python/Java bindings, the `cell_flag()` query function, index maintenance over
masked outputs, compaction remapping, overlay-based or row-subset publication, flag state spilled
out of the manifest, and `initial_value = true`.

See `PLAN.md` for integration points. The API summary, test and benchmark commands, and results
are added below as the work lands.

## Publication contract

A *publication* is a `DataReplacement` whose `CellFlagUpdate`s set dependent flags true. Each
replaced fragment is a *group*: one staged file holding the outputs for every physical row.

**In a publication, every row of a replaced fragment that the transaction does not assign must be
copied unchanged from the read snapshot (as read through Lance). Lance treats those rows as
logically unchanged for invalidation, and as physically written for conflict detection.**

An incremental refresh that computes only pending rows therefore keeps the rows an earlier refresh
completed, while any concurrent write to the group's file footprint still conflicts. For a
dependent flag `f` and a publication group `g`, over the watched fields of `f` that `g` writes:

- `f`'s own output, when the transaction publishes `f`, contributes nothing;
- the output of another dependent flag `u` the transaction publishes contributes the rows `u`
  assigns in `g`;
- any other written watched field (a plain source write, an unpublished output) contributes the
  whole fragment.

`f` is cleared on the union of the contributions minus the rows `f` itself assigns in `g`. A
whole-fragment contribution that meets rows `f` assigns is refused with `InvalidInput`: the
transaction would publish values its own write invalidates. Groups of non-publications clear the
whole fragment.

## Conflicts and the `Skip` policy

A publication is checked against every transaction committed in `(read_version, head]`, across
commit retries:

| Concurrent change | `Reject` | `Skip` |
|---|---|---|
| Drops or replaces a published flag | error | error |
| Invalidates assigned rows (input written, flag cleared) | retryable | rows deferred (`InputChanged`) |
| Deletes assigned rows | rows deferred (`RowVacated`) | rows deferred (`RowVacated`) |
| Moves assigned rows (row-moving update) | retryable | rows deferred (`RowVacated`) |
| Publishes on a group's fragment and fields | retryable | group deferred (`NewerResult`) |
| Writes a group's fields on its fragment | retryable | group deferred (`OutputWritten`) |
| Removes a group's fragment | error | group deferred (`FragmentRemoved`) |
| Compacts a group's fragment | retryable | group deferred (`FragmentRewritten`) |
| Merge, overwrite, restore, MemWAL state update, dropping or indexing a replaced field | error | error |

Under `Skip`, a row deferred for one flag is deferred for every flag whose output the group's file
writes. The group still installs its file, so those rows hold stale values under a false flag;
where that changes what an output reads as (an unmasked output, or a sibling flag that was true),
the commit clears the flag explicitly so that flags computed from it are cleared too. A deferred
group installs nothing and its staged file is left on storage. The sibling rule also holds for a
deferred group's `valid_rows` when a later commit retry finds them stale. A group whose fragment is
removed or rewritten by `checked_version` is reported with that reason, whatever deferred it first,
and no row of that fragment is reported as deferred or reusable. When every group is deferred, no
version is written. The read version of a publication is never advanced.

`Reject` still drops assignments of rows deleted since the read version and commits the rest;
`execute_with_report` reports them as `RowVacated`.

`Skip` applies only to a publication whose updates all set dependent flags true and whose files
write only those flags' outputs; anything else is refused with `InvalidInput`, as is `Skip` through
`CommitBuilder::execute`.

Other cell flag conflicts, in both policies:

- Explicit updates of the same flag on overlapping rows conflict (retryable). A publication's
  own assignments are checked by the rules above instead.
- Explicit updates on a fragment conflict with a transaction that moves rows out of it, removes
  it or rewrites it (retryable); a publication's assignments do so only under `Reject`.
- A row-moving update conflicts with a concurrent transaction that sets a dependent flag true on
  one of its source fragments (retryable).

## Publication API

```rust
let result = CommitBuilder::new(dataset)
    .with_dependency_conflict_policy(DependencyConflictPolicy::Skip)
    .execute_with_report(publication)
    .await?;
// result.dataset: the committed version, or the head when nothing was committed.
let report = result.report;
let recompute = report.deferred_rows_of(flag_id, DeferralReason::InputChanged);
let reuse = report.reusable_rows(flag_id);
```

`PublicationReport` holds `read_version`, `checked_version` (the head the final attempt was checked
against), `committed_version`, `published` (the committed true assignments), `deferred_rows`
(`DeferredRows { flag_id, rows, reason, conflicting_version }`) and `deferred_groups`
(`DeferredGroup { fragment_id, data_file, reason, conflicting_version, valid_rows }`). A follow-up
refresh reads at `committed_version`, or at `checked_version` when nothing was committed,
recomputes the `InputChanged` rows, and may reuse the staged values of `reusable_rows` (the
published rows plus each deferred group's `valid_rows`). `DependencyConflictPolicy`,
`PublicationReport` and `PublicationResult` are re-exported from `lance::dataset`.

Tests:

```sh
cargo test -p lance --lib dataset_cell_flags_publication
cargo test -p lance --lib conflict_resolver
cargo test -p lance-table cell_flag_commit
```
