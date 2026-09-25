# Dependency-aware cell flags: implementation plan

Baseline: `origin/main` at `e3671b2f5730eea927a088a42cbf30e273edc43c`. Branch: `codex/dependency-aware-cell-flags`.

Design input: `option-d-dependent-cell-flags.md` (Weston's option D) and the reference PR
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
| Invalidation | `derive_cell_flag_invalidations(head, txn)` in `lance-table`, called in `commit_transaction` after `rebase.finish` and **before** the transaction is serialized, on every attempt, against the **head** registry. Result replaces `cell_flag_changes.derived_invalidations`, so the transaction file records the clear even when the flag was already false. |
| State application | `apply_cell_flag_changes` inside `build_manifest_with_read_version` after the per-operation match: drops, registrations, retain final fragments, derived clears, explicit updates, carried state for moved rows, normalization, feature bits. |
| Unsupported operations | `ensure_operation_allowed_with_cell_flags(head, txn)` next to `ensure_can_write_manifest` in every commit attempt (normal and detached). |
| Refresh conflicts | `TransactionRebase` (`io/commit/conflict_resolver.rs`): a cell-flag preamble in `check_txn` plus per-group handling in `check_data_replacement_txn`; `finish` trims deferred groups/rows for DataReplacement and never changes `read_version`. |
| Conflict policy + report | `CommitBuilder::with_dependency_conflict_policy(DependencyConflictPolicy::{Reject, Skip})` and `CommitBuilder::execute_with_report` returning a `PublicationReport` (`rust/lance/src/dataset/write/commit.rs`). |
| Masking | `FileFragment::open` resolves masking flags for projected top-level fields; `FragmentReader` nulls masked cells right after the overlay merge in both read funnels (`new_read_impl`, `read_ranges`), before deletions, filters, aggregates and projection. |
| Row-moving update | `UpdateJob` (`write/update.rs`) records `CellFlagMovedRows` (old address -> new row) and the fields it wrote; the commit copies the head state of ordinary flags, and of dependent flags whose watched fields were not written, onto the moved rows. |

## Semantics

- A flag is *dependent* when `clear_on_write` is non-empty. Its watched fields are the
  `clear_on_write` fields plus its own output field: any logical write to them clears the flag for
  the written rows, except the flag's own publication.
- Dependent flags start unassigned (false) and become true only through a publication: a
  `DataReplacement` whose files carry the output field for the assigned fragments.
- A publication validates, for every assigned row, that no transaction in `(read_version, head]`
  invalidated the row, and that no transaction wrote the group's output fields in that fragment
  (the file's full physical footprint). Registration drop/replace fences publishers.
- Default policy `Reject`: any dependency conflict fails the commit. Policy `Skip` (publication
  transactions only): stale rows lose their assignment (their stale values stay masked), unsafe
  groups are deferred whole, safe groups publish, and the caller receives a `PublicationReport`.

## Implementation order

1. Foundation (`lance-table` types/proto/apply/derive/gate, commit wiring, registration API, one
   end-to-end test: register, publish, source write clears, read the state).
2. In parallel: read-path masking; refresh conflict checks + skip policy + report; UpdateJob carry-over.
3. The required correctness test matrix, repository checks.
4. Benchmarks (baseline worktree at the same SHA, same profile) and report.
5. Adversarial review and fixes.
