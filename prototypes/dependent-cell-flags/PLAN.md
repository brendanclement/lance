# Dependency-aware cell flags: implementation plan

Baseline: `origin/main` at `e3671b2f5730eea927a088a42cbf30e273edc43c`. Branch: `brendan/dependency-aware-cell-flags`.

Design input: the draft `option-d-dependent-cell-flags.md` (September 24, 2026, based on Weston's
option D; not in this repository, the copy used is
`/Users/brendan/code/sophon/option-d-dependent-cell-flags.md`) and the reference PR
[#8655](https://github.com/lance-format/lance/pull/8655) at head `488aecce5`. #8655 is a draft on
an old base (11.0.0-beta.12); main has since moved the transaction model into `lance-table`. We
reuse its data model ideas (stable flag ids, monotonic allocator, per-fragment true-set keyed by
physical offset, commit-time application against the head, row-overlap conflicts) but not its
transport (hidden `transaction_properties` payload + BLAKE3 digest), external root objects, query
UDF, compaction remap, or bindings.

## Integration points found on main

| Concern | Integration point |
|---|---|
| Registry + state storage | New `Manifest.cell_flags: Option<Arc<CellFlagRegistry>>` (`rust/lance-table/src/format/manifest.rs`), proto `Manifest.cell_flags = 22` (`protos/table.proto`). State is inline: per flag, a `RowAddrTreeMap` of rows whose flag is true (`Full` = every physical row). |
| Feature gate | `FLAG_UNSTABLE_CELL_FLAGS = 1 << 12` reader **and** writer bit (`rust/lance-table/src/feature_flags.rs`), gated in release builds by `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` like overlays. `ensure_can_write_manifest` already runs against the head on every commit attempt (`io/commit.rs`). |
| Typed transaction payload | New `Transaction.cell_flag_changes: Option<Arc<CellFlagChanges>>` (`rust/lance-table/src/transaction/builder.rs`), proto `Transaction.cell_flag_changes = 5` (`protos/transaction.proto`), conversions in `transaction/proto.rs`. |
| Registration | `Dataset::{register,replace,drop}_cell_flag` commit `Operation::UpdateConfig` (no config edits) carrying registrations/drops. Flag ids are assigned at commit from `next_flag_id` and never reused (`restore_old_manifest` keeps the maximum). |
| Invalidation | `derive_cell_flag_invalidations(head, txn)` in `lance-table`, called through `record_derived_cell_flag_invalidations` in `commit_transaction_with_report` (`io/commit.rs`) after the rebase and the commit gates and **before** the transaction is serialized, on every attempt, against the **head** registry. Result replaces `cell_flag_changes.derived_invalidations`, so the transaction file records the clear even when the flag was already false. A row-moving update derives no clear; it records `moved_rows` instead. |
| State application | `apply_cell_flag_changes` inside `build_manifest_with_read_version` after the per-operation match: drops, registrations, retain final fragments, derived clears, explicit updates, carried state for moved rows, normalization, feature bits. |
| Unsupported operations | `ensure_operation_allowed_with_cell_flags(head, txn)` in the commit loop of `commit_transaction_with_report`, after the rebase on every attempt. `do_commit_detached_transaction` does not call it: it refuses every detached commit on a dataset with registered flags or carrying cell flag changes. |
| Refresh conflicts | `TransactionRebase` (`io/commit/conflict_resolver.rs`). `check_txn` sends a publication to `check_publication_txn` (`conflict_resolver/publication.rs`) in place of `check_data_replacement_txn`; it handles the operations that touch the groups (`Delete`, `Update`, `Rewrite`, `DataReplacement`, `DataOverlay`) and hands the rest to main's `check_data_replacement_txn`, which is unchanged. After the per-operation check, `check_masked_reads_txn` and `check_cell_flag_changes_txn` run for every transaction. `finish` and `finish_with_report` delegate a publication to `finish_publication`, which trims deferred groups and rows and never changes `read_version`. |
| Conflict policy + report | `CommitBuilder::with_dependency_conflict_policy(DependencyConflictPolicy::{Reject, Skip})` and `CommitBuilder::execute_with_report` returning a `PublicationResult { dataset, report: PublicationReport }` (`rust/lance/src/dataset/write/commit.rs`). |
| Masking | `FileFragment::open` resolves masking flags for projected top-level fields (`CellFlagMasks::resolve`); `FragmentReader::resolve_cells` merges overlays and then nulls masked cells, in both read funnels (`new_read_impl`, `read_ranges`), before deletions, filters, aggregates and projection. |
| Row-moving update | `UpdateJob` (`write/update.rs`) records `CellFlagMovedRows` (new offsets paired by rank with a treemap of old addresses) and the fields it wrote; the commit copies the head state of ordinary flags, and of dependent flags whose watched fields were not written, onto the moved rows, and clears every flag at the old addresses. Keeping dependent state requires every version since the read to be loadable. |

## Semantics

- A flag is *dependent* when `clear_on_write` is non-empty. Its watched fields are the
  `clear_on_write` fields plus its own output field: any logical write to them clears the flag for
  the written rows, except the flag's own publication.
- Dependent flags start unassigned (false) and become true only through a publication: a
  `DataReplacement` whose files carry the output field for the assigned fragments.
- Only dependent flags may mask. Writers that re-read rows through the masked scan write masked
  cells back as NULL, which is harmless only when the flag can become true again solely through a
  publication that writes new values, and while the mask stays: an `Update` that rewrote a field
  it read through a mask retries over a concurrent drop of that mask. MemWAL rows carry no flag
  state, so MemWAL and a masking flag are never part of one version.
- A publication validates its assignments against the read version (offsets within the fragment,
  a group writing the output), then, for every assigned row, that no transaction in
  `(read_version, head]` invalidated the row, and that no transaction wrote the group's output
  fields in that fragment (the file's full physical footprint). Registration drop/replace fences
  publishers. A downstream flag published with its upstream must be computed from the upstream's
  new values on the rows both assign.
- Default policy `Reject`: any dependency conflict fails the commit; assignments of rows deleted
  since the read version are still dropped and the rest committed. Policy `Skip` (publication
  transactions only): stale rows lose their assignment (a masked output reads their stale values
  as NULL; for an unmasked output, or a sibling flag that was true, the commit clears the flag so
  the flags computed from it are cleared too), unsafe groups are deferred whole, safe groups
  publish, and the caller receives a `PublicationReport`.

## Implementation order

1. Foundation (`lance-table` types/proto/apply/derive/gate, commit wiring, registration API, one
   end-to-end test: register, publish, source write clears, read the state).
2. In parallel: read-path masking; refresh conflict checks + skip policy + report; UpdateJob carry-over.
3. The required correctness test matrix, repository checks.
4. Benchmarks (baseline worktree at the same SHA, same profile) and report.
5. Adversarial review and fixes.
