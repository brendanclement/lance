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
