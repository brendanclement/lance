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
  transaction even if the flag was already false. A flag with sources is *dependent*.
- `mask_when_false`: reads return NULL for the output field where the flag is false. Only a
  dependent flag may mask.

Flags without `clear_on_write` are *ordinary*: only explicit updates change them.

A refresh reads a snapshot, computes outputs outside Lance, stages full-fragment column files, and
commits a `DataReplacement` carrying `CellFlagUpdate { value: true }` assignments. Lance validates
the assignments against every transaction since the refresh's `read_version`. The default policy
rejects any conflict. The `Skip` policy publishes safe groups and reports deferred rows and
fragments. See `PLAN.md` for the integration points.

## API

Rust only (`lance` and `lance-table`).

| Item | Where |
|---|---|
| `CellFlagOptions { clear_on_write: Vec<String>, mask_when_false: bool }`, `with_clear_on_write`, `with_mask_when_false` | `lance::dataset::cell_flag` |
| `Dataset::register_cell_flag(field, name, options) -> Result<CellFlagDefinition>` | commits an `UpdateConfig` with the registration |
| `Dataset::replace_cell_flag(field, name, options)`, `Dataset::drop_cell_flag(field, name)` | a replacement gets a new flag id |
| `Dataset::cell_flags()`, `Dataset::cell_flag(field, name)`, `Dataset::cell_flag_true_rows(flag_id) -> Result<RowAddrTreeMap>` | read the registry and state at the checked-out version |
| `CellFlagChanges { updates, registrations, drops, derived_invalidations, moved_rows, moved_rows_written_fields }`, `CellFlagUpdate { flag_id, value, rows }` | `lance::dataset::transaction`; attach with `TransactionBuilder::cell_flag_changes` |
| `CommitBuilder::with_dependency_conflict_policy(DependencyConflictPolicy::{Reject, Skip})` | default `Reject` |
| `CommitBuilder::execute_with_report(Transaction) -> Result<PublicationResult>` | `PublicationResult { dataset, report }` |
| `PublicationReport { read_version, checked_version, committed_version, published, deferred_rows, deferred_groups }`, `published_rows`, `reusable_rows`, `deferred_rows_of` | `DeferredRows`, `DeferredGroup`, `DeferralReason` in `lance::dataset::cell_flag` |
| `CellFlagDefinition`, `CellFlagRegistry` (`Manifest::cell_flags`) | `lance_table::format` |

Callers never supply `derived_invalidations`: the commit recomputes them against the head on every
attempt. `UpdateBuilder` fills `moved_rows` itself. End to end (compiled as the
`lance::dataset::cell_flag` module example):

```rust
// `summary` is computed from `title` and `body` and reads NULL until published.
let ready = dataset
    .register_cell_flag(
        "summary",
        "ready",
        CellFlagOptions::default()
            .with_clear_on_write(["title", "body"])
            .with_mask_when_false(true),
    )
    .await?;

// A refresh reads a snapshot, computes one value per physical row of fragment 0
// outside Lance, and stages them as a full-fragment file.
let read = dataset.clone();
let output = read.schema().project(&["summary"])?;
let fragment = read.get_fragment(0).expect("fragment 0 exists");
let group = fragment
    .write_columns(stream::iter([Ok(summaries)]), &output)
    .await?;
let mut computed = RowAddrTreeMap::new();
computed.insert_fragment(0);
let publication = TransactionBuilder::new(
    read.version().version,
    Operation::DataReplacement { replacements: vec![group] },
)
.cell_flag_changes(CellFlagChanges {
    updates: vec![CellFlagUpdate { flag_id: ready.flag_id, value: true, rows: computed }],
    ..Default::default()
})
.build();

// Publish what is still valid and learn what to redo.
let result = CommitBuilder::new(Arc::new(read))
    .with_dependency_conflict_policy(DependencyConflictPolicy::Skip)
    .execute_with_report(publication)
    .await?;
let recompute = result.report.deferred_rows_of(ready.flag_id, DeferralReason::InputChanged);
let reuse = result.report.reusable_rows(ready.flag_id);

// An ordinary write to a source clears the flag in its own commit.
UpdateBuilder::new(Arc::new(result.dataset))
    .update_where("id = 7")?
    .set("body", "'new body'")?
    .build()?
    .execute()
    .await?;
```

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
writes. The group still installs its file, so those rows hold stale values under a false flag
(masked outputs read them as NULL); where that changes what an output reads as (an unmasked
output, or a sibling flag that was true), the commit clears the flag explicitly so that flags
computed from it are cleared too. A deferred group installs nothing and its staged file is left on
storage; its `valid_rows` are the staged values still correct at `checked_version`. A group whose
fragment is removed or rewritten by `checked_version` is reported with that reason, and no row of
that fragment is reported as deferred or reusable. When every group is deferred, no version is
written. The read version of a publication is never advanced. A follow-up refresh reads at
`committed_version` (or `checked_version` when nothing was committed), recomputes the
`InputChanged` rows and the rows that moved, and may reuse `reusable_rows`.

`Reject` still drops assignments of rows deleted since the read version and commits the rest;
`execute_with_report` reports them as `RowVacated`.

Other cell flag conflicts, in both policies:

- Explicit updates of the same flag on overlapping rows conflict (retryable).
- Explicit updates on a fragment conflict with a transaction that moves rows out of it, removes
  it or rewrites it (retryable); a publication's assignments do so only under `Reject`.
- A row-moving update conflicts with a concurrent transaction that sets a dependent flag true on
  one of its source fragments (retryable), so it rereads the published values.
- A registration conflicts with a concurrent `Overwrite` or `Restore` (incompatible), and with a
  `Project` or `Merge` that removes or re-ids a field it names (retryable).

## Supported / Unsupported

Supported:

- Several sources per flag, sources shared by several flags, and chains (a flag watching another
  flag's output; clears propagate downstream). Cycles and a second dependent flag on one field are
  refused at registration.
- Source writes: `UpdateBuilder` (row-moving; moved rows keep every flag whose watched fields,
  directly or upstream, were not set), partial-schema `merge_insert` (`RewriteColumns`, in place,
  per-row clears), `DataReplacement` of a source or output (whole fragment), deletes, and
  `DataOverlay` of an unwatched field. A write staged before a registration still records the
  clear.
- Publication through `DataReplacement` on several fragments, incremental (copy-through), with
  sibling outputs in one file, under `Reject` or `Skip`, across commit retries.
- Registration drop and replace, which fence older publishers; registering or dropping a masking
  flag clears the flags watching its field. Restore keeps flag ids monotonic; shallow and deep
  clones keep the registry.
- Masking in scans, filters (including `IS NULL`), `count_rows` with a filter, aggregates
  (`COUNT(column)` counts masked cells as NULL, `COUNT(*)` counts rows), sort, group by, `take`,
  `take_rows`, late materialization, flat full-text search, update/delete predicates, SQL through
  `Dataset::sql` and the Lance table provider; multi-fragment with deletions, with and without
  stable row ids.

Unsupported, each failing with an explicit error (test in `dataset_cell_flags.rs` unless named):

| Operation | Error | Test |
|---|---|---|
| Compaction (`Rewrite`) | `NotSupported` | `test_unsupported_operations_fail_explicitly::compaction` |
| `Overwrite` | `NotSupported` | `…::overwrite` |
| MemWAL state update | `NotSupported` | `…::mem_wal_state_update` |
| `DataOverlay` writing a watched source or output | `NotSupported` | `…::overlay_on_source`, `…::overlay_on_output` |
| Dropping or casting (re-id'ing) a source or output | `NotSupported` | `…::drop_source`, `…::drop_output`, `…::cast_source` |
| Changing the nested fields of a watched struct | `NotSupported` | `lance-table` `nested_changes_to_a_watched_struct_are_refused` |
| `Merge` rewriting a source's data | `NotSupported` | `…::merge_rewriting_source` |
| Making a masked output non-nullable | `NotSupported` | `…::non_nullable_masked_output` |
| Setting a dependent flag true outside a `DataReplacement` that writes its output | `NotSupported` / `InvalidInput` | `…::dependent_flag_set_outside_publication`; `lance-table` `change_gate`, `publication_validation_errors` |
| Masking with an ordinary flag (no `clear_on_write`) | `NotSupported` | `…::masking_ordinary_flag`; `lance-table` `registration_validation_errors::ordinary_mask` |
| Masking a non-nullable, nested, list, struct, blob or legacy (v1) field | `InvalidInput` / `NotSupported` | `lance-table` `registration_validation_errors`, `masking_registration_rejects_indexed_output_and_legacy_storage` |
| Any index on a masked output; masking an indexed field | `NotSupported` | `dataset_cell_flags_masking`: `test_indexing_a_masked_field_fails_before_building`, `test_masking_an_indexed_field_is_refused` |
| Row-moving writes without flag state (`merge_insert` `RewriteRows`, raw staged `Update`) on a fragment where an ordinary flag is true | `NotSupported` (retryable when staged before the flag's registration) | `dataset_cell_flags_update`: `test_stateless_row_move_is_refused_where_an_ordinary_flag_is_true`, `test_row_move_retries_over_flag_registered_after_its_read` |
| Detached commits; batch commits or dataset creation carrying flag changes | `NotSupported` | `test_detached_and_batch_commits_are_refused`, `test_creating_a_dataset_with_cell_flag_changes_is_refused` |
| MemWAL (LSM) reads over a base table with a masking flag | `NotSupported` | `dataset_cell_flags_masking::test_lsm_reads_of_a_masked_dataset_are_refused` |
| `Skip` through `CommitBuilder::execute`, or on anything but a publication | `InvalidInput` | `dataset_cell_flags_publication::test_skip_needs_a_publication_and_a_report` |

Dependent flags on rows moved by a stateless row-moving write fall back to unassigned: safe, but
the work is lost. Not implemented: Python/Java bindings, the `cell_flag()` query function, index
maintenance over masked outputs, compaction remapping, overlay-based or row-subset publication,
flag state spilled out of the manifest, and `initial_value = true`.

## Tests

Requirements from the task, with the tests that cover them (files under
`rust/lance/src/dataset/tests/`; `lance-table` tests in
`rust/lance-table/src/transaction/cell_flag_commit.rs`):

| # | Requirement | Tests |
|---|---|---|
| 1 | Publish, then an input changes: the output is masked | `publication::test_input_write_masks_published_output` (replace, merge_insert, UpdateBuilder); `masking::test_source_write_masks_output_in_its_own_version`; `cell_flags::test_dependent_flag_publication_and_invalidation` |
| 2 | Input changes, then an old refresh publishes: rejected or skipped | `publication::test_refresh_staged_before_input_write` (3 write paths × 2 policies) |
| 3 | Invalidation while the flag is already false | `publication::test_refresh_skips_rows_invalidated_while_flag_false` (3 write paths × 2 policies); `cell_flags::test_dependent_flag_publication_and_invalidation`; `lance-table` `derive_records_clears_when_already_false_and_against_head_registry` |
| 4 | Input A → B → A | `publication::test_input_restored_before_refresh_publishes` (3 write paths × 2 policies) |
| 5 | A newer worker's result survives an older worker | `publication::test_newer_result_survives_older_refresh`, `test_copied_rows_cannot_overwrite_newer_result`, `test_retry_that_defers_a_group_keeps_the_newer_flag` |
| 6 | An input write staged before a publication commits after it and clears it | `publication::test_input_write_staged_before_publication_clears_it`; `update::test_update_retries_over_concurrent_publication`; `cell_flags::test_write_staged_before_registration_records_clear` |
| 7 | An unrelated-field update keeps dependent outputs | `update::test_update_moves_flags_whose_watched_fields_it_does_not_set::unrelated_field`, `test_in_place_merge_insert_clears_only_flags_watching_it::unrelated_field`; `masking::test_write_predicates_see_masked_values::unwatched_field` |
| 8 | Shared inputs invalidate all and only the right outputs | `update::test_update_moves_flags_whose_watched_fields_it_does_not_set`, `test_in_place_merge_insert_clears_only_flags_watching_it`; `cell_flags::test_clears_propagate_down_dependency_chains`; `publication::test_refresh_loop_never_shows_a_stale_output` |
| 9 | One unsafe fragment does not block safe ones under `Skip` | `publication::test_unsafe_fragment_does_not_block_safe_ones` (5 kinds × 2 policies), `test_refresh_loop_never_shows_a_stale_output` |
| 10 | A full-file replacement cannot overwrite newer results outside its rows | `publication::test_copied_rows_cannot_overwrite_newer_result`, `test_newer_result_survives_older_refresh`; `lance-table` `derive_counts_unassigned_rows_of_a_publication_as_copied`, `publication_exempts_only_published_inputs` |
| 11 | An assigned NULL is distinguishable from pending work | `masking::test_masked_field_reads_null_where_flag_is_false` |
| 12 | Registration replace/drop fences old work | `publication::test_registration_change_fences_old_publisher`; `cell_flags::test_dropped_flag_fences_staged_publication`, `test_publication_needs_flag_registered_at_read_version`; `lance-table` `apply_fences_changes_for_unknown_flags` |
| 13 | A commit retry after a competing commit keeps every invalidation | `publication::test_commit_retry_records_every_invalidation` (3 write paths × registration/publication competitor; the first manifest write loses its slot to a competitor committed by a `CommitHandler` wrapper), `test_publication_retry_accumulates_deferrals` |
| 14 | A failed publication exposes no partial values or assignments | `publication::test_rejected_publication_exposes_nothing` (conflict and invalid assignment), `test_registration_change_fences_old_publisher`, the `Reject` branches of the tests above |
| 15 | Unsupported operations fail explicitly | see the table above |
| M | Masking, `IS NULL`, `COUNT(column)` vs `COUNT(*)` | `masking::test_filters_see_masked_values`, `test_aggregates_count_masked_cells_as_null`, `test_every_reader_funnel_masks_each_batch`, `test_takes_and_late_materialization_mask`, `test_sort_and_group_by_see_masked_values` |

`test_refresh_loop_never_shows_a_stale_output` runs the whole loop on three fragments with
`summary <- title, body` and `translation <- body, language`, and checks every version by time
travel. Tests use `memory://` datasets and control commit order with stale handles, staged
transactions and a `CommitHandler` that commits a competitor first; none sleeps.

```sh
cargo test -p lance-table cell_flag
cargo test -p lance --lib -- cell_flag conflict_resolver
cargo test -p lance --doc cell_flag
```

## Departures from the design draft

- **No Python API.** The draft's Python calls exist as Rust methods. The Python binding drops
  cell flag changes when a transaction round-trips through it.
- **Registrations ride on `UpdateConfig`.** `register_cell_flag` commits an `UpdateConfig` whose
  typed `CellFlagChanges` carry the registration; there is no separate operation or hidden payload.
- **No `initial_value`.** Every flag starts false; dependent flags must.
- **Masking only for dependent flags.** Row-moving updates and in-place rewrites re-read rows
  through the masked scan and write masked cells back as NULL. An ordinary masking flag set true
  later would expose that NULL instead of the hidden value; a dependent flag is set true only by a
  publication that writes fresh values.
- **Output writes invalidate.** Writing a dependent flag's output other than through its own
  publication clears it, like a source write.
- **Copy-through contract.** A publication replaces whole fragments, and rows it does not assign
  must be copied from its read snapshot.
- **`Skip` only for publication-only transactions**, and only through `execute_with_report`. The
  policy is set on `CommitBuilder`, not passed to `commit`.
- **Fragment-group deferral.** An unsafe group is deferred whole; there is no row-subset
  publication inside a file.
- **Manifest-inline state.** Flag state is a `RowAddrTreeMap` per flag inside the manifest.
- **Compaction and indexes are refused** rather than preserving flags or maintaining index
  coverage over masked outputs.

Known gaps: `LsmScanner::without_base_table` has no manifest to consult and serves MemWAL rows
unmasked; compaction is refused only at commit, after it has reserved fragment ids and written
its files; a plain delete leaves flag state on deleted rows, which still counts toward the
ordinary-flag gate for row-moving writes; `DeferredGroup::valid_rows` can list rows deleted after
the read version.
