# Dependency-aware cell flags (option D) prototype

> **Not for production.** This is a research prototype on an unstable, feature-gated format
> (`FLAG_UNSTABLE_CELL_FLAGS`). The on-disk encoding, APIs and conflict rules may change or be
> removed without migration. Release builds refuse flagged datasets unless
> `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` is set. It does not implement full option D.

## Status

| | |
|---|---|
| Worktree | `/Users/brendan/code/lance/.claude/worktrees/dependency-aware-cell-flags-43fc32` |
| Branch | `brendan/dependency-aware-cell-flags` (not merged, no PR) |
| Baseline | `e3671b2f5730eea927a088a42cbf30e273edc43c` (`origin/main` when the work started) |
| Final implementation | `25d3cf3e5d988271f05e50de1f47e9885f1b33ba`, the last commit that changes the prototype's code: a deferred publication file no longer certifies outputs computed from an upstream output staged in the same file, and reports them as `UpstreamNotPublished`. Later commits change only rustdoc and tests in `rust/lance` (`9f0911398`) and `prototypes/`. Before it, the last code commit was `26388225a273052b3a1ff0ec705c53da1b68c7cb`; the commits in between change only `prototypes/` (benchmark results, docs and benchmark tooling: `run_paired.sh` gained `ORDER` in `3e8c36ba0672102ad005c6de20858e9a82b32231` and the rustflags check in `f1fb31606`; `analyze.py` gained `--section`/`--title` and `bench/` the rerun scripts after that) and rustdoc in `rust/lance/src/dataset/cell_flag/publication.rs` (the `PublicationReport` conflict table in `4aaa8280c`, the `DependencyConflictPolicy::Reject` description in `4993d350b`) |
| Benchmarked | Full matrix at `21601f894` (`1m`; `10m` subset; `smoke-final`) and at `26388225a` (`1m-clean`, 1M, clean build). Reads at `26388225a` (`10m-reads-clean-*`, clean builds; `1m-reads-maskfix`, `10m-reads-maskfix`, `10m-reads-reversed`, which record `e48573011`, a results-only commit with the same code). The only code change from `21601f894` to `26388225a` is the mask builder (`fragment/cell_flag_mask.rs`). Not re-run at `25d3cf3e5` or `9f0911398` (rustdoc and tests only): `25d3cf3e5` changes only what the report says about a deferred publication file that carries chained outputs, and no benchmark workload does (none registers an output computed from another output, and each publication stages one output) |

Checks on the benchmarked code `26388225a` (run at `f1fb31606`, whose Rust, proto, Python and Java
sources equal it): `cargo fmt --all -- --check`; `cargo clippy --all --tests --benches -- -D
warnings`; `cargo check --workspace --tests --benches`; `cargo test -p lance-table` (564 passed, 2
doctests); `cargo test -p lance --lib` (4392 passed, 3 ignored); `cargo test -p lance --doc
cell_flag` (8); `cargo check` and `cargo clippy --tests -- -D warnings` for `python/`; `cargo check`
for `java/lance-jni/`. No lockfile changed. Python lint (`uv run make lint`) was not run: no Python
source changed, only the Rust binding's struct literal. Java and Python tests were not run. The
`CommitBuilder` doctests the prototype added do not match `cell_flag`; they ran later, at
`4aaa8280c` (same code as `26388225a`): `cargo test -p lance --doc -- cell_flag
with_dependency_conflict_policy execute_with_report` (10 passed).

Checks on the final code, at `25d3cf3e5` (which changes only `rust/lance`) and again at `9f0911398`
(which changes only its rustdoc and tests): `cargo fmt --all -- --check`; `cargo clippy -p
lance-table -p lance --tests --benches -- -D warnings`; `cargo test -p lance --lib -- cell_flag
conflict_resolver` (312 passed, then 313); `cargo test -p lance-table cell_flag` (100 passed);
`cargo test -p lance --doc -- cell_flag with_dependency_conflict_policy execute_with_report` (10
passed); `cargo test -p lance --lib` (4399 passed, then 4400; 3 ignored). The workspace-wide check
and clippy, `python/` and `java/lance-jni/` were not rerun: nothing outside `rust/lance` names the
report types.

## Scope

Lance tracks, per output field, a Boolean *cell flag* with a stable flag id. A registration may
declare:

- `clear_on_write`: source fields. Any logical write to a source (or to the output itself) clears
  the flag for the written rows, in the same commit as the write. In-place writes (partial-schema
  `merge_insert`) and `DataReplacement`s record the clear in the transaction even if the flag was
  already false. A row-moving `UpdateBuilder` update records the rows it moved instead
  (`moved_rows`). A concurrent publication with a group on a fragment the update moves rows out of
  fails under `Reject` (retryable); under `Skip` it defers the moved rows as `RowVacated`. A flag
  with sources is *dependent*.
- `mask_when_false`: reads return NULL for the output field where the flag is false. Only a
  dependent flag may mask.

Flags without `clear_on_write` are *ordinary*: only explicit updates change them.

A refresh reads a snapshot, computes outputs outside Lance, stages full-fragment column files, and
commits a `DataReplacement` carrying `CellFlagUpdate { value: true }` assignments. Lance validates
the assignments against every transaction since the refresh's `read_version`. The default policy,
`Reject`, fails the commit on any conflict, except that assignments of rows deleted since the read
version are dropped and the rest is committed (reported as `RowVacated`). The `Skip` policy
publishes safe groups and reports deferred rows and fragments. See `PLAN.md` for the integration
points.

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
| `PublicationReport { read_version, checked_version, committed_version, published, deferred_rows, deferred_groups }`, `published_rows`, `reusable_rows`, `deferred_rows_of` | `DeferredRows`, `DeferredGroup`, `DeferralReason::{InputChanged, RowVacated, UpstreamNotPublished, NewerResult, OutputWritten, FragmentRemoved, FragmentRewritten}` in `lance::dataset::cell_flag` |
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

**A flag published in the same transaction as a dependent flag upstream of it must, on the rows
both assign, be computed from the upstream values that transaction publishes, not from the read
snapshot.** Those rows are exempt from the clear the upstream's publication causes, so Lance
cannot notice a downstream value computed from the upstream's old (masked) value. Published in
separate transactions, the upstream's publication clears the downstream flag instead
(`publication::test_chained_outputs_published_together`). When `Skip` defers the file, the
downstream rows are reported as `UpstreamNotPublished`, not as reusable (see below).

Every true assignment is validated against the read version before the rebase: each offset must
be a physical row of its fragment, and each assigned fragment needs a group whose file writes the
flag's output. An invalid publication fails with `InvalidInput` even when a concurrent commit
would have deferred the offending group.

## Conflicts and the `Skip` policy

A publication is checked against every transaction committed in `(read_version, head]`, across
commit retries:

| Concurrent change | `Reject` | `Skip` |
|---|---|---|
| Drops or replaces a published flag | error | error |
| Invalidates assigned rows (input written, flag cleared) | retryable | rows deferred (`InputChanged`) |
| Deletes assigned rows | rows deferred (`RowVacated`) | rows deferred (`RowVacated`) |
| Moves rows out of a group's fragment (row-moving update), assigned or not | retryable | assigned rows deferred (`RowVacated`) |
| Publishes on a group's fragment and fields | retryable | group deferred (`NewerResult`) |
| Writes a group's fields on its fragment | retryable | group deferred (`OutputWritten`) |
| Removes a group's fragment | error | group deferred (`FragmentRemoved`) |
| Compacts a group's fragment (unreachable, see below) | retryable | group deferred (`FragmentRewritten`) |
| `Merge`, or indexing a replaced field | retryable | retryable |
| Overwrite, restore, MemWAL state update, dropping a replaced field | error | error |

The compaction row is defensive code with no test: the commit gate refuses compaction on any
dataset with a registered flag, and a publication fails unless its flags are registered at its
read version and not dropped since, so no compaction can commit between the read version and the
head of a publication that would otherwise succeed.

Under `Skip`, a row deferred for one flag is deferred for every flag whose output the group's file
writes. The group still installs its file, so those rows hold stale values under a false flag
(masked outputs read them as NULL); where that changes what an output reads as (an unmasked output,
or a sibling flag that was true), the commit clears the flag explicitly so that flags computed from
it are cleared too. A deferred group installs nothing and its staged file is left on storage; its
`valid_rows` are the staged values still correct at `checked_version`, except the stale rows listed
in the known gaps. Since that file never committed, a flag's staged values on the rows where the
file also assigns a flag upstream of it (one whose output it watches) were computed from an input no
reader sees: they are deferred as `UpstreamNotPublished`, not listed as valid. Those staged values
must not be reused, and later commit attempts do not check them: a follow-up recomputes them from
the upstream value it leaves on the row, the committed one or, where it also publishes the upstream
there, the one it publishes. The upstream's own values, and sibling outputs that do not watch each
other, stay valid. Only direct upstreams count: where the upstream is not assigned, the file holds
its copied snapshot value, and a later write or clear of the upstream there clears the downstream
flag too (`InputChanged`), unless it republishes the downstream flag there as well (see the known
gaps). A group whose fragment is removed or rewritten by `checked_version` is reported with that
reason, and no row of that fragment is reported as deferred or reusable. When every group is
deferred, no version is written. The read version of a publication is never advanced. A follow-up
refresh reads at `committed_version` (or `checked_version` when nothing was committed), recomputes
the `InputChanged` and `UpstreamNotPublished` rows and the rows that moved, and may reuse
`reusable_rows`, except where it also publishes an upstream output of the flag on the row: a
reusable value was computed from the committed upstream value, so there the publication contract
requires computing it from the upstream value the follow-up publishes
(`publication::test_deferred_group_defers_each_link_of_a_chain`).

`Reject` still drops assignments of rows deleted since the read version and commits the rest;
`execute_with_report` reports them as `RowVacated`.

Other cell flag conflicts, in both policies:

- Explicit updates of the same flag on overlapping rows conflict (retryable).
- Explicit updates on a fragment conflict with a transaction that moves rows out of it, removes
  it or rewrites it (retryable). A publication's assignments follow the table above instead: under
  `Reject`, moved or rewritten rows are retryable and a removed fragment is an error.
- A row-moving update conflicts with a concurrent transaction that sets a dependent flag true on
  one of its source fragments (retryable), so it rereads the published values.
- A registration conflicts with a concurrent `Overwrite` or `Restore` (incompatible), and with a
  `Project` or `Merge` that removes or re-ids a field it names (retryable).
- An `Update` that rewrites a field it read through a masking flag (a partial-schema
  `merge_insert` copying unmatched rows in place, or a row-moving update) conflicts (retryable)
  with a concurrent drop or replacement of that flag. It wrote the masked cells back as NULL; the
  drop exposes the stored values, and the retry reads them.

## Supported / Unsupported

Supported:

- Several sources per flag, sources shared by several flags, and chains (a flag watching another
  flag's output; clears propagate downstream). Cycles and a second dependent flag on one field are
  refused at registration.
- Source writes: `UpdateBuilder` (row-moving; moved rows keep every flag whose watched fields,
  directly or upstream, were not set), partial-schema `merge_insert` (`RewriteColumns`, in place,
  per-row clears), `DataReplacement` of a source or output (whole fragment), deletes, and
  `DataOverlay` of an unwatched field. A write staged before a registration still records the
  clear. A partial `merge_insert` records the offsets it patched only when the version it read has
  stable row ids or a registry, so one staged before the first registration clears whole
  fragments.
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
| Masking a non-nullable, nested, list, struct, blob or legacy (v1) field | `InvalidInput` / `NotSupported` | `lance-table` `registration_validation_errors`, `masking_registration_rejects_indexed_output_mem_wal_and_legacy_storage` |
| Any index on a masked output; masking an indexed field | `NotSupported` | `dataset_cell_flags_masking`: `test_indexing_a_masked_field_fails_before_building`, `test_masking_an_indexed_field_is_refused` |
| Row-moving writes without flag state (`merge_insert` `RewriteRows`, raw staged `Update`) on a fragment where an ordinary flag is true | `NotSupported` (retryable when staged before the flag's registration) | `dataset_cell_flags_update`: `test_stateless_row_move_is_refused_where_an_ordinary_flag_is_true`, `test_row_move_retries_over_flag_registered_after_its_read` |
| Detached commits; batch commits or dataset creation carrying flag changes | `NotSupported` | `test_detached_and_batch_commits_are_refused`, `test_creating_a_dataset_with_cell_flag_changes_is_refused` |
| Initializing MemWAL or opening a MemWAL writer where a flag masks a field; registering a masking flag where MemWAL is initialized (checked at commit, so also when staged before the other) | `NotSupported` | `dataset_cell_flags_masking::test_mem_wal_and_masking_are_exclusive`; `lance-table` `operation_gate::mem_wal_on_masked`, `masking_registration_rejects_indexed_output_mem_wal_and_legacy_storage` |
| MemWAL (LSM) reads over a base table version with a masking flag | `NotSupported` | `dataset_cell_flags_masking::test_lsm_reads_of_a_masked_dataset_are_refused` |
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
| 3 | Invalidation while the flag is already false | `publication::test_refresh_skips_rows_invalidated_while_flag_false` (3 write paths × 2 policies; replace and `merge_insert` record the clear, the row-moving update records its moved rows instead); `cell_flags::test_dependent_flag_publication_and_invalidation`; `lance-table` `derive_records_clears_when_already_false_and_against_head_registry` |
| 4 | Input A → B → A | `publication::test_input_restored_before_refresh_publishes` (3 write paths × 2 policies) |
| 5 | A newer worker's result survives an older worker | `publication::test_newer_result_survives_older_refresh`, `test_copied_rows_cannot_overwrite_newer_result`, `test_retry_that_defers_a_group_keeps_the_newer_flag` |
| 6 | An input write staged before a publication commits after it and clears it | `publication::test_input_write_staged_before_publication_clears_it`; `update::test_update_retries_over_concurrent_publication` (a row-moving update read before the publication retries over it); `cell_flags::test_write_staged_before_registration_records_clear` |
| 7 | An unrelated-field update keeps dependent outputs | `update::test_update_moves_flags_whose_watched_fields_it_does_not_set::unrelated_field`, `test_in_place_merge_insert_clears_only_flags_watching_it::unrelated_field`; `masking::test_write_predicates_see_masked_values::unwatched_field` |
| 8 | Shared inputs invalidate all and only the right outputs | `update::test_update_moves_flags_whose_watched_fields_it_does_not_set`, `test_in_place_merge_insert_clears_only_flags_watching_it`; `cell_flags::test_clears_propagate_down_dependency_chains`; `publication::test_refresh_loop_never_shows_a_stale_output`, `test_chained_outputs_published_together` |
| 9 | One unsafe fragment does not block safe ones under `Skip` | `publication::test_unsafe_fragment_does_not_block_safe_ones` (5 kinds × 2 policies), `test_refresh_loop_never_shows_a_stale_output` |
| 10 | A full-file replacement cannot overwrite newer results outside its rows | `publication::test_copied_rows_cannot_overwrite_newer_result`, `test_newer_result_survives_older_refresh`; `lance-table` `derive_counts_unassigned_rows_of_a_publication_as_copied`, `publication_exempts_only_published_inputs` |
| 11 | An assigned NULL is distinguishable from pending work | `masking::test_masked_field_reads_null_where_flag_is_false` |
| 12 | Registration replace/drop fences old work | `publication::test_registration_change_fences_old_publisher`; `cell_flags::test_dropped_flag_fences_staged_publication`, `test_publication_needs_flag_registered_at_read_version`; `lance-table` `apply_fences_changes_for_unknown_flags` |
| 13 | A commit retry after a competing commit keeps every invalidation | `publication::test_commit_retry_records_every_invalidation` (3 write paths × registration/publication competitor; the first manifest write loses its slot to a competitor committed by a `CommitHandler` wrapper), `test_publication_retry_accumulates_deferrals`, `test_retry_keeps_chained_outputs_of_a_deferred_group_unreusable` |
| 14 | A failed publication exposes no partial values or assignments | `publication::test_rejected_publication_exposes_nothing` (conflict; invalid assignment under `Reject`, and under `Skip` with a concurrent commit deferring the invalid group or every group), `test_registration_change_fences_old_publisher`, the `Reject` branches of the tests above |
| 15 | Unsupported operations fail explicitly | see the table above |
| M | Masking, `IS NULL`, `COUNT(column)` vs `COUNT(*)` | `masking::test_filters_see_masked_values`, `test_aggregates_count_masked_cells_as_null`, `test_every_reader_funnel_masks_each_batch`, `test_takes_and_late_materialization_mask`, `test_sort_and_group_by_see_masked_values` |
| S | Staged writes: the clears they record, and retries over a dropped mask they read through | `cell_flags::test_partial_merge_insert_records_offsets_only_when_read`; `update::test_rewrite_retries_over_drop_of_the_mask_it_read_through` |
| R | A deferred file's values computed from an upstream it also stages are reported for recomputation (`UpstreamNotPublished`), never as reusable; independent outputs stay reusable | `publication::test_deferred_group_recomputes_outputs_of_its_staged_upstream` (the review's case; a follow-up that follows the report: translating alone, after republishing summary, or with summary in one file), `test_deferred_group_defers_only_rows_its_upstream_assigns` (partially overlapping assignments), `test_deferred_group_defers_each_link_of_a_chain` (three outputs in one file; a follow-up restaging them in one file recomputes the reusable values whose input it republishes), `test_retry_keeps_chained_outputs_of_a_deferred_group_unreusable` (group deferred on the retry, or invalidated during it), `test_deferred_group_keeps_independent_outputs_reusable`. Each checks that every assigned row is reported once and that every reusable staged value is the function of its input at the head |

`test_refresh_loop_never_shows_a_stale_output` runs the whole loop on three fragments with
`summary <- title, body` and `translation <- body, language`, and checks every version by time
travel. Tests use `memory://` datasets, except for two that use temporary directories:
`masking::test_masked_field_reads_null_where_flag_is_false` (cold reopen) and
`cell_flags::test_clones_keep_cell_flags`. They control commit order with stale handles, staged
transactions and a `CommitHandler` that commits a competitor first; none sleeps.

```sh
cargo test -p lance-table cell_flag
cargo test -p lance --lib -- cell_flag conflict_resolver test_merge_insert_subcols_without_matches_stores_its_transaction
cargo test -p lance --doc -- cell_flag with_dependency_conflict_policy execute_with_report
```

The second command's last filter selects the one test the prototype added whose path matches
neither `cell_flag` nor `conflict_resolver`, in `dataset/write/merge_insert.rs`. The third
command's last two filters select the `CommitBuilder` examples in `dataset/write/commit.rs`.

## Departures from the design draft

The draft is `option-d-dependent-cell-flags.md` (September 24, 2026), which proposes these APIs as
extensions of Xuanwo's cell flag draft PR
[#8655](https://github.com/lance-format/lance/pull/8655), based on Weston's option D. It is not in
this repository; the copy used is `/Users/brendan/code/sophon/option-d-dependent-cell-flags.md`.

- **No Python API.** The draft's Python calls (`register_cell_flag`,
  `Transaction(cell_flag_changes=...)`, `commit(on_dependency_conflict=...)`) exist only as Rust
  methods. The prototype's questions are about the core, and bindings are thin wrappers that can
  follow once the Rust API settles. The Python binding drops cell flag changes when a transaction round-trips
  through it.
- **Registrations ride on `UpdateConfig`.** `register_cell_flag` commits an `UpdateConfig` whose
  typed `CellFlagChanges` carry the registration. `UpdateConfig` already has conflict rules
  against every operation, so no new `Operation` variant is needed, and typed changes replace
  #8655's serialized payload in `transaction_properties`: the task asks that callers never handle
  serialized flag payloads.
- **No `initial_value`.** Every flag starts false. Dependent flags must start unassigned; none of
  the requirements needs a true default for ordinary flags, which would also need a rule for rows
  appended later.
- **Masking only for dependent flags.** Row-moving updates and in-place rewrites re-read rows
  through the masked scan and write masked cells back as NULL. An ordinary masking flag set true
  later would expose that NULL instead of the hidden value; a dependent flag is set true only by a
  publication that writes fresh values.
- **Output writes invalidate.** Writing a dependent flag's output other than through its own
  publication clears it, like a source write: the flag vouched for the value that write replaced.
- **Copy-through contract.** A publication replaces whole fragments, and rows it does not assign
  must be copied from its read snapshot. A `DataReplacement` file covers every physical row of its
  fragment, so an incremental refresh must carry the rows it did not compute, and Lance cannot tell
  a copy from a new value.
- **Chained outputs published together.** A downstream output published in the same transaction
  as its upstream must be computed from the upstream's new values on the rows both assign; the
  draft computes every value from the read snapshot. Publishing both atomically exempts those rows
  from the clear the upstream's publication causes, so a downstream value computed from the
  snapshot's (masked) upstream would otherwise publish under a true flag.
- **`Skip` only for publication-only transactions**, and only through `execute_with_report`. The
  task forbids partially applying unrelated updates, and `CommitBuilder::execute` returns no
  report to say what was deferred. The policy is set on `CommitBuilder`, the Rust commit entry
  point, not passed to `commit`.
- **Fragment-group deferral.** An unsafe group is deferred whole; there is no row-subset
  publication inside a file. The draft's first step and the task both allow this; row-subset
  publication needs rewritten value files or an overlay value mapping.
- **Manifest-inline state.** Flag state is a `RowAddrTreeMap` per flag inside the manifest, rather
  than #8655's external root objects, to avoid a new file format and its IO and cleanup paths in a
  prototype. Its cost is measured (flag state size below) and listed as a bottleneck.
- **Compaction and indexes are refused** rather than preserving flags (the draft) or maintaining
  index coverage over masked outputs. Each is a separate feature (remapping flag state through a
  rewrite, index maintenance on publication), and the task allows refusing unsupported paths
  explicitly.

Known gaps: fresh-tier readers that take no base table (`LsmScanner::without_base_table`,
`ShardWriter::scan`, `MemTable::scan`) and `ShardWriter::open` never see a manifest. Since no
version holds both MemWAL and a masking flag, their rows come from a MemWAL initialized while no
field was masked, but a writer or shard left over after `drop_index` removes the MemWAL index keeps
serving its rows unmasked once a masking flag is registered. A caller-staged `DataReplacement`
computed from masked reads is not retried over a concurrent drop of the mask; it writes whatever it
read, and its whole-fragment clear keeps downstream flags safe. Compaction is refused only at commit,
after it has reserved fragment ids and written its files; a plain delete leaves flag state on
deleted rows, which still counts toward the ordinary-flag gate for row-moving writes;
`DeferredGroup::valid_rows` can list rows deleted after the read version, and rows where a
concurrent publication republished the flag together with an upstream output it watches. Nothing
clears the flag there, so the staged value, computed from the older upstream value, stays listed.
Those rows are true at the head with the newer result: a follow-up that reuses staged values only
on rows still pending is unaffected, one that restages them overwrites the newer result.

## Benchmarks

Harness: `rust/lance/benches/` (`cell_flags_regression.rs`, `dependent_cell_flags.rs`,
`cell_flags_common/`). Driver, analyzer and rerun scripts: `bench/`; `bench/README.md` has the
setup and the exact invocation of every run. Raw samples (JSONL), per-run logs, `env.json`
(machine, rustc, both SHAs, git status) and `runs.tsv` (load average before each run) are under
`bench/results/<run>/`; the `env.json` of `10m-reads-layout-control` was written by hand and is
partial (its `README.md` says what it lacks). `bench/REPORT.md` has a hand-written summary and
every generated table. Local APFS SSD on an Apple M5 Pro (18 cores, 48 GB), `release-with-debug`,
same machine and profile for both builds, baseline and prototype alternating per round. A
device-management daemon kept about one core busy during the runs. The OS page cache is not
controlled: `fresh-session` means new Lance caches only, never a cold read. The simulated function
(FNV rounds, about 1.2 µs/row) is timed separately (`udf_ms`). The permissive baseline refresh is
**not** correctness-equivalent: without dependency tracking it publishes stale values after
in-place writes (10·K stale rows) and main rejects it outright after row-moving writes.

Both worktrees must be checked out outside any other Lance checkout (see the method caveat below;
`bench/README.md` lists the harness files to copy into the baseline). Every recorded results
directory already holds rounds, which `run_paired.sh` refuses to overwrite, so a new run needs a
new `RESULTS_DIR`; its name becomes the run's `REPORT.md` section:

```sh
git worktree add --detach /abs/lance-baseline e3671b2f5
git worktree add --detach /abs/lance-proto brendan/dependency-aware-cell-flags
git -C /abs/lance-proto checkout 26388225a -- rust/   # the benchmarked code; see bench/README.md
cd /abs/lance-proto
BASELINE_WORKTREE=/abs/lance-baseline BENCH_DATA_DIR=/abs/bench-data \
RESULTS_DIR=/abs/results/1m-repeat \
  prototypes/dependent-cell-flags/bench/run_paired.sh 1m    # also: smoke, 10m; ORDER=prototype-first
```

Results at 1M rows / 10 fragments (3 paired rounds unless noted; each ratio is the median of the
per-round ratios of medians; the run is named in each row):

| Question | Result |
|---|---|
| Tables without flags | `1m-clean` (clean build): every workload 0.96–1.04× baseline: appends, sparse and dense `UpdateBuilder` updates (dense 1.04×), partial `merge_insert`, DataReplacement refresh, publication after K commits, scans, filters, counts, SQL, take. |
| Source writes with flags | `1m`: sparse `UpdateBuilder` (100 rows) 1.11× (one output) / 1.15× (two sharing `body`); unrelated field 1.16× / 1.20× (flags move with the rows); dense (10% of rows) 1.53× / 1.48×, manifest 2.8 KB → 336 KB / 502 KB, transaction 1.2 KB → 170 KB; partial `merge_insert` 1.03× / 0.98×. The one flag round of `1m-clean` measured less: sparse 1.02× / 1.08×, unrelated field 0.98× / 1.11×, dense 1.41× / 1.39×, `merge_insert` 0.98× / 0.93×. |
| Refresh publication | `1m`: clean refresh +0.8% (1.27 s, of which 1.18 s simulated UDF); publication commit after K = 0–64 unrelated commits 1.00–1.07×, growing with K like the baseline. |
| Masked reads | All flags true: 0.93–1.00× in `1m-reads-maskfix` (final mask builder), 0.98–1.05× in `1m` (`21601f894`). 1% of rows invalidated (scattered), full scan / `IS NULL` / `COUNT`: 3.0–3.2× a plain NULL column in `1m`, **1.5–1.8×** in `1m-reads-maskfix` (masked scan 9.7 ms in `1m` → 5.4 ms; the plain NULL column 3.3 ms in the same run); id-range filter and take 0.92–1.00× (`1m-reads-maskfix`). |
| Refresh under K conflicting source commits (10 rows each), `Skip` | `1m`: publishes 999,990 / 999,960 / 999,840 of 1,000,000 rows for K = 1 / 4 / 16; deferred rows are exactly the written ones (`InputChanged` in place, `RowVacated` row-moving). A concurrent write to the output field defers whole fragments: 5 / 9 / 10 of 10. `Reject` fails every one (0 published). Publication commit 0.51–1.80 ms under `Skip`, 0.14–0.46 ms for the rejected commits. |
| Saved computation | `1m`, follow-up to completion: after `Reject`, this harness's follow-up recomputes every row (2,000,000 UDF rows in total, 1.26–1.29 s), because the rejected publication returns only an error, with no report of what stayed valid. After row-level deferral, or whole-fragment deferral with `reuse_valid_staged`, only the deferred rows are (1,000,010–1,000,160 in total, 16–85 ms). After whole-fragment deferral with `recompute_all_pending`, the deferred fragments are (1,500,000 / 1,900,000 / 2,000,000 in total for K = 1 / 4 / 16, 0.63–1.27 s); reusing `reusable_rows` saves up to 999,840 recomputations (K = 16: 1.18 s → 0.57 ms of UDF). |
| Flag state size | `1m`, 1% of rows invalidated: manifest 7 KB (unflagged control) → 88 KB (one flag) / 148 KB (two); 10%: 473 KB / 789 KB (the manifest also inlines that transaction). Fresh-session open at 1% and 10% invalidated: 0.09 ms (unflagged control) → 0.13–0.15 ms locally. |

At 10M rows / 100 fragments, `10m` (`21601f894`, one round): sparse updates 1.15× / 1.19×,
unrelated-field updates 1.13× / 1.16×, clean refresh +0.6%, publication after K commits
1.00–1.08×, 858 KB of manifest for the 1%-invalidated read table, and masked reads 0.92–1.03× with
all flags true and 3.0–3.5× on full-column reads with 1% invalidated. With the final mask builder
(`10m-reads-maskfix`, `10m-reads-reversed`, `10m-reads-clean-*`): all flags true 0.94–1.04×,
full-column reads with 1% invalidated 1.68–1.80×.

**Open regression: full-column reads of tables without flags are 6–9% slower at 10M rows**
(geometric mean of `10m-reads-clean-baseline-first` and `10m-reads-clean-prototype-first`, clean
builds with rustflags identical to the baseline; selective reads 1.00–1.06×; at 1M, `1m-clean`
full-column reads are 0.96–1.03×). A baseline plus one unused function
(`10m-reads-layout-control`, 6 rounds) moves the same reads by 0.99–1.04× (per round 0.95–1.10×).
The prototype with `fragment.rs`, its fragment read path, reverted to the baseline
(`10m-reads-variant-fragment-reverted`; the masked-index check in `index.rs` and the manifest's
cell flag decoding remain) measured 1.02–1.06×. The clean slowdown exceeds the layout control by
2.6–7.7% per full-column workload (median 4.7%). On a dataset without flags, the prototype's read
path adds an `Option` check per fragment open (`CellFlagMasks::resolve`) and per read call
(`FragmentReader::resolve_cells`), an `Option<Arc<CellFlagMasks>>` field that `FragmentReader`
copies on clone, a registry check per index in `scalar_index_info` (`index.rs`), and an optional
registry in the manifest decode; the variant reverted the first three and kept the last two. A
sampling profile of the nested build (`bench/results/10m-profile/`) has the same six leading
non-wait frames in both builds and cell flag code in 2 of its 62,830 non-wait samples. Of the 18
frames with at least 1% of the non-wait samples in either build, all but `mach_absolute_time`
(1.7% against the baseline's 3.0%) differ in share by at most 0.8 percentage points, though `_free`
is 1.2% against 0.4%. No clean-build binary was profiled, and the cause is not identified.

Method caveat: this worktree sits under another Lance checkout (`/Users/brendan/code/lance`), and
cargo merged both identical `.cargo/config.toml` files, concatenating their `rustflags`, so
prototype binaries built here received their rustflags twice. The early baseline-vs-prototype runs
(`smoke-final`, `1m`, `10m`, `1m-reads-maskfix`, `10m-reads-maskfix`, `10m-reads-reversed`) mix
that build difference with the code change. Whether it also moves flags-vs-no-flags ratios is not
established: the only clean-build flag round (`1m-clean`, round 3) measured the lower write
overheads in the table above and full-column reads with 1% invalidated of 1.66–1.89×
(`1m-reads-maskfix`: 1.53–1.77×). The `*-clean` runs and both controls were built outside any other
checkout, and `run_paired.sh` now refuses builds whose rustflags differ. `bench/REPORT.md` opens
with a summary saying which run answers which question.

These are prototype measurements on local disk, not a pass of #8655's benchmark acceptance criteria;
S3 was not measured.

## Remaining risks and bottlenecks

Correctness risks:

- **Trusted publication contents.** Lance cannot verify the copy-through contract (unassigned rows
  copied unchanged) or the chained-output contract. A buggy executor that writes placeholders for
  copied rows publishes them under true flags.
- **Staged work retention.** Deferred groups' staged files are unreferenced, so
  `cleanup_old_versions` may delete them once they age out; there is no lease.
- **Unclassified future operations.** State is keyed by physical row address. Every operation must
  either be handled or refused by the commit gate; a new `Operation` variant or row-moving writer
  that bypasses it would misplace or drop flag state.
- **Known gaps** listed above: base-table-less MemWAL readers, caller-staged replacements computed
  from masked reads racing a mask drop, compaction refused only after it wrote its files, stale
  `valid_rows` after a concurrent publication of an output together with its upstream, Python
  bindings dropping cell flag changes on a round trip (the outcome is conservative), no Python/Java
  API.
- **Compatibility review needed** for a public `Transaction` field, the `CellFlagMovedRows` type, the
  reader+writer feature bit and the `FLAG_UNKNOWN` bump; the format is unstable and gated.

Performance bottlenecks:

- **Unexplained no-flag read regression at 10M rows.** Full-column reads of tables without flags
  measured 6–9% slower than the baseline in the clean builds (`10m-reads-clean-*`), 2.6–7.7%
  (median 4.7%) beyond the layout control; not seen at 1M (`1m-clean`). Needs a hardware-counter
  profile of the clean build before any claim of zero overhead.
- **Manifest-inline state.** Fragmented true sets make every commit rewrite, and every open decode,
  hundreds of KB (858 KB at 10M rows with 1% scattered invalidation, `10m`). Spill per-fragment
  state to external files, as deletion files or #8655's roots do.
- **Row-moving updates fragment state.** `UpdateBuilder` clears the old addresses of moved rows, so
  a dense update leaves a hole pattern no run compression helps; keeping dependent-flag bits on
  deleted rows (they are unreadable) would keep those fragments `Full`. Its moved-rows payload
  (`RoaringTreemap` of source addresses) adds 170 KB to the transaction at 100k moved rows.
- **Masked scans over partial state** still cost 1.5–1.8× after the run-based mask
  (`1m-reads-maskfix`; 1.68–1.80× at 10M in `10m-reads-clean-*`); the remainder is per-read offset
  materialization. A cached per-fragment validity buffer or masking in the decoder would remove
  most of it.
- **Full-fragment publication.** Republishing a few rows rewrites whole fragment files (7.5–15 MB to
  republish 10–160 rows invalidated in place at 1M, `1m`), and one output write defers a whole
  group. Row-subset or overlay publication (with a rewritten value file, not a trimmed bitmap)
  would fix both.
- **`Reject` returns only an error on a conflict.** It fails at the first conflict, so this
  harness's follow-up recomputes every row (2N UDF rows at 1M). Certifying the rest of the staged
  work as reusable would need Lance to finish validating the history since the read version instead
  of stopping at the first conflict: a separate API change, not just exposing the report `Skip`
  builds.
