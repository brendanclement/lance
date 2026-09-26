// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Conflict checks for a publication: a `DataReplacement` that sets dependent
//! cell flags true. See [`crate::dataset::cell_flag::PublicationReport`] for
//! the rules and what each policy does.
//!
//! The rebase is rebuilt on every commit attempt from the transaction the
//! previous attempt finished, and checks only the versions committed since,
//! so what one attempt defers is already gone from the transaction the next
//! one checks. The report keeps checking the valid rows of groups deferred by
//! earlier attempts.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use lance_core::{Error, Result};
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::format::cell_flag::{field_label, fragment_key, top_level_ancestors};
use lance_table::format::{CellFlagDefinition, CellFlagRegistry, DeletionFile, Fragment, Manifest};
use lance_table::transaction::ensure_cell_flags_registered_at_read_version;
use roaring::RoaringBitmap;

use super::{TransactionRebase, read_fragment_deletion_bitmap, wrong_operation_err};
use crate::Dataset;
use crate::dataset::cell_flag::{
    DeferralReason, DeferredGroup, DeferredRows, DependencyConflictPolicy, PublicationDeferrals,
    input_changed_rows, removed_fragments,
};
use crate::dataset::transaction::{
    CellFlagUpdate, DataReplacementGroup, Operation, Transaction, UpdateMode,
};
use crate::io::deletion::read_dataset_deletion_file;

#[derive(Debug)]
pub(super) struct Publication {
    policy: DependencyConflictPolicy,
    read_version: u64,
    /// The dependent flags the transaction sets true.
    flags: BTreeMap<u32, PublishedFlag>,
    groups: BTreeMap<u64, Group>,
    /// Top-level ancestor of every field id of the read schema.
    ancestors: HashMap<i32, i32>,
    /// The registry at the read version.
    registry: Arc<CellFlagRegistry>,
    /// The registry at the head the attempt checks against, which also knows
    /// the flags registered since the read version and not dropped.
    head_registry: Option<Arc<CellFlagRegistry>>,
    deferred_groups: BTreeMap<u64, (DeferralReason, u64)>,
    /// Per fragment, the rows each concurrent version made stale, in version
    /// order and disjoint.
    stale_rows: BTreeMap<u64, Vec<(u64, RoaringBitmap)>>,
    /// Per fragment, the deletion file each concurrent version left it with.
    deletion_updates: BTreeMap<u64, Vec<(u64, Option<DeletionFile>)>>,
}

#[derive(Debug)]
struct PublishedFlag {
    definition: CellFlagDefinition,
    label: String,
    /// The flags the transaction also publishes whose outputs this flag
    /// watches.
    upstreams: Vec<u32>,
}

#[derive(Debug)]
struct Group {
    /// Top-level ids of the fields the file writes.
    fields: HashSet<i32>,
    physical_rows: u32,
}

#[derive(Default)]
struct GroupWrites {
    conflicts: Vec<(u64, DeferralReason, Error)>,
    deletion_updates: Vec<(u64, Option<DeletionFile>)>,
}

fn materialize(selection: &RowAddrSelection, physical_rows: u32) -> RoaringBitmap {
    match selection {
        RowAddrSelection::Full => {
            let mut rows = RoaringBitmap::new();
            rows.insert_range(0..physical_rows);
            rows
        }
        RowAddrSelection::Partial(rows) => rows.clone(),
    }
}

fn fragment_rows(fragment: u32, rows: RoaringBitmap) -> RowAddrTreeMap {
    let mut map = RowAddrTreeMap::new();
    if !rows.is_empty() {
        map.insert_bitmap(fragment, rows);
    }
    map
}

fn assigned_rows(transaction: &Transaction, flag_id: u32) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for update in transaction
        .cell_flag_changes
        .iter()
        .flat_map(|changes| changes.updates.iter())
        .filter(|update| update.value && update.flag_id == flag_id)
    {
        rows |= &update.rows;
    }
    rows
}

impl Publication {
    /// `None` unless `transaction` is a `DataReplacement` setting a flag true
    /// that is dependent at the read version. `head` is the latest version
    /// the commit attempt loaded.
    pub(super) fn try_new(
        read_manifest: &Manifest,
        head: &Manifest,
        transaction: &Transaction,
        initial_fragments: &HashMap<u64, (Fragment, bool)>,
        policy: DependencyConflictPolicy,
    ) -> Result<Option<Self>> {
        let Operation::DataReplacement { replacements } = &transaction.operation else {
            return Ok(None);
        };
        let Some(changes) = transaction.cell_flag_changes.as_deref() else {
            return Ok(None);
        };
        ensure_cell_flags_registered_at_read_version(read_manifest, transaction)?;
        let Some(registry) = read_manifest.cell_flags.clone() else {
            return Ok(None);
        };
        let schema = &read_manifest.schema;
        let definitions: BTreeMap<u32, &CellFlagDefinition> = changes
            .updates
            .iter()
            .filter(|update| update.value)
            .filter_map(|update| registry.definition(update.flag_id))
            .filter(|definition| definition.is_dependent())
            .map(|definition| (definition.flag_id, definition))
            .collect();
        let flags: BTreeMap<u32, PublishedFlag> = definitions
            .iter()
            .map(|(flag_id, definition)| {
                let upstreams = definition
                    .clear_on_write
                    .iter()
                    .filter_map(|source| registry.dependent_flag(*source))
                    .filter(|upstream| definitions.contains_key(&upstream.flag_id))
                    .map(|upstream| upstream.flag_id)
                    .collect();
                (
                    *flag_id,
                    PublishedFlag {
                        definition: (*definition).clone(),
                        label: definition.label(schema),
                        upstreams,
                    },
                )
            })
            .collect();
        if flags.is_empty() {
            return Ok(None);
        }
        let ancestors = top_level_ancestors(schema);
        let mut groups = BTreeMap::new();
        for DataReplacementGroup(fragment_id, file) in replacements {
            let (fragment, _) = initial_fragments.get(fragment_id).ok_or_else(|| {
                Error::invalid_input(format!(
                    "the publication replaces fragment {fragment_id}, which is not in version {}, \
                     the version it read",
                    transaction.read_version
                ))
            })?;
            let physical_rows = fragment
                .physical_rows
                .and_then(|rows| u32::try_from(rows).ok())
                .ok_or_else(|| {
                    Error::invalid_input(format!(
                        "cannot check the publication on fragment {fragment_id}: its physical row \
                         count at version {} is {:?}, not a count a cell flag row address holds",
                        transaction.read_version, fragment.physical_rows
                    ))
                })?;
            let fields = file
                .fields
                .iter()
                .filter(|id| **id >= 0)
                .map(|id| ancestors.get(id).copied().unwrap_or(*id))
                .collect();
            let group = Group {
                fields,
                physical_rows,
            };
            if groups.insert(*fragment_id, group).is_some() {
                return Err(Error::invalid_input(format!(
                    "the publication replaces fragment {fragment_id} more than once"
                )));
            }
        }
        Ok(Some(Self {
            policy,
            read_version: transaction.read_version,
            flags,
            groups,
            ancestors,
            registry,
            head_registry: head.cell_flags.clone(),
            deferred_groups: BTreeMap::new(),
            stale_rows: BTreeMap::new(),
            deletion_updates: BTreeMap::new(),
        }))
    }

    pub(super) fn publishes(&self, flag_id: u32) -> bool {
        self.flags.contains_key(&flag_id)
    }

    fn top_of(&self, field_id: i32) -> i32 {
        self.ancestors.get(&field_id).copied().unwrap_or(field_id)
    }

    fn writes_group_fields(&self, group: &Group, field_ids: impl IntoIterator<Item = i32>) -> bool {
        field_ids
            .into_iter()
            .filter(|id| *id >= 0)
            .any(|id| group.fields.contains(&self.top_of(id)))
    }

    /// Dropping a published flag, or replacing it with a new id, retires the
    /// registration the values were computed under.
    fn check_fence(&self, other: &Transaction, other_version: u64) -> Result<()> {
        let Some((flag_id, flag)) = other
            .cell_flag_changes
            .iter()
            .flat_map(|changes| changes.drops.iter())
            .find_map(|flag_id| self.flags.get(flag_id).map(|flag| (flag_id, flag)))
        else {
            return Ok(());
        };
        Err(Error::incompatible_transaction_source(
            format!(
                "cell flag {flag_id} is not registered since version {other_version}: concurrent \
                 {} dropped or replaced {} after version {}, which this transaction read, so \
                 values computed under that registration cannot be published",
                other.operation.name(),
                flag.label,
                self.read_version
            )
            .into(),
        ))
    }

    fn record_group_writes(&mut self, writes: GroupWrites, other_version: u64) -> Result<()> {
        for (fragment_id, reason, error) in writes.conflicts {
            match self.policy {
                DependencyConflictPolicy::Reject => return Err(error),
                DependencyConflictPolicy::Skip => {
                    let deferral = self
                        .deferred_groups
                        .entry(fragment_id)
                        .or_insert((reason, other_version));
                    // Once the fragment is gone, none of the group's staged
                    // positions can be reused, whatever deferred it first.
                    if reason.removes_fragment() && !deferral.0.removes_fragment() {
                        *deferral = (reason, other_version);
                    }
                }
            }
        }
        for (fragment_id, deletion_file) in writes.deletion_updates {
            self.deletion_updates
                .entry(fragment_id)
                .or_default()
                .push((other_version, deletion_file));
        }
        Ok(())
    }

    /// Rows whose inputs `other` changed for a published flag, as
    /// [`input_changed_rows`] defines them, are stale wherever this
    /// transaction assigns them. Deferred groups are tracked too, so the
    /// report tells which of their staged rows stay reusable.
    fn check_staleness(
        &mut self,
        ours: &Transaction,
        other: &Transaction,
        other_version: u64,
    ) -> Result<()> {
        if other.cell_flag_changes.is_none() {
            return Ok(());
        }
        let mut found: Vec<(u64, RoaringBitmap)> = Vec::new();
        for (flag_id, flag) in &self.flags {
            let changed = input_changed_rows(
                other,
                &flag.definition,
                &self.registry,
                self.head_registry.as_deref(),
                &self.ancestors,
            );
            if changed.is_empty() {
                continue;
            }
            let stale = assigned_rows(ours, *flag_id) & &changed;
            for (fragment, selection) in stale.iter() {
                let fragment_id = u64::from(*fragment);
                // Rows outside every group are refused when the commit applies them.
                let Some(group) = self.groups.get(&fragment_id) else {
                    continue;
                };
                let mut rows = materialize(selection, group.physical_rows);
                for (_, earlier) in self
                    .stale_rows
                    .get(&fragment_id)
                    .into_iter()
                    .flatten()
                    .chain(found.iter().filter(|(id, _)| *id == fragment_id))
                {
                    rows -= earlier;
                }
                if rows.is_empty() {
                    continue;
                }
                if self.policy == DependencyConflictPolicy::Reject {
                    return Err(Error::retryable_commit_conflict_source(
                        other_version,
                        format!(
                            "{} cannot be published on {} row(s) of fragment {fragment_id}: \
                             concurrent {} at version {other_version} changed their inputs after \
                             version {}, which this transaction read. Recompute those rows, or \
                             commit with DependencyConflictPolicy::Skip to publish the rest",
                            flag.label,
                            rows.len(),
                            other.operation.name(),
                            self.read_version
                        )
                        .into(),
                    ));
                }
                found.push((fragment_id, rows));
            }
        }
        for (fragment_id, rows) in found {
            self.stale_rows
                .entry(fragment_id)
                .or_default()
                .push((other_version, rows));
        }
        Ok(())
    }

    /// `rows` without `removed`, whose partial fragments must be groups.
    fn subtract(&self, rows: &RowAddrTreeMap, removed: &RowAddrTreeMap) -> Result<RowAddrTreeMap> {
        let mut kept = RowAddrTreeMap::new();
        for (fragment, selection) in rows.iter() {
            match removed.get(fragment) {
                None => {
                    let mut unchanged = RowAddrTreeMap::new();
                    match selection {
                        RowAddrSelection::Full => unchanged.insert_fragment(*fragment),
                        RowAddrSelection::Partial(bitmap) => {
                            unchanged.insert_bitmap(*fragment, bitmap.clone())
                        }
                    }
                    kept |= &unchanged;
                }
                Some(RowAddrSelection::Full) => {}
                Some(RowAddrSelection::Partial(removed)) => {
                    let group = self.groups.get(&u64::from(*fragment)).ok_or_else(|| {
                        Error::internal(format!(
                            "publication rows were removed from fragment {fragment}, which has no \
                             replacement group"
                        ))
                    })?;
                    kept |= &fragment_rows(
                        *fragment,
                        materialize(selection, group.physical_rows) - removed,
                    );
                }
            }
        }
        Ok(kept)
    }
}

impl TransactionRebase<'_> {
    /// Checks for a publication against `other`, in place of
    /// `check_data_replacement_txn`.
    pub(super) fn check_publication_txn(
        &mut self,
        other: &Transaction,
        other_version: u64,
    ) -> Result<()> {
        let publication = self.publication()?;
        publication.check_fence(other, other_version)?;
        match &other.operation {
            Operation::Delete { .. }
            | Operation::Update { .. }
            | Operation::Rewrite { .. }
            | Operation::DataReplacement { .. }
            | Operation::DataOverlay { .. } => {
                let writes = self.group_writes(other, other_version)?;
                self.publication_mut()?
                    .record_group_writes(writes, other_version)?;
            }
            // Conflicts over the whole dataset keep main's rules in both policies.
            _ => self.check_data_replacement_txn(other, other_version)?,
        }
        let Self {
            transaction,
            cell_flags,
            ..
        } = self;
        cell_flags
            .publication
            .as_mut()
            .ok_or_else(|| wrong_operation_err(&transaction.operation))?
            .check_staleness(transaction, other, other_version)
    }

    fn publication(&self) -> Result<&Publication> {
        self.cell_flags
            .publication
            .as_ref()
            .ok_or_else(|| wrong_operation_err(&self.transaction.operation))
    }

    fn publication_mut(&mut self) -> Result<&mut Publication> {
        let operation = &self.transaction.operation;
        self.cell_flags
            .publication
            .as_mut()
            .ok_or_else(|| wrong_operation_err(operation))
    }

    /// What `other` did to the groups' footprints (every physical row of the
    /// fragment, for every field the file writes) and deletion vectors.
    fn group_writes(&self, other: &Transaction, other_version: u64) -> Result<GroupWrites> {
        let publication = self.publication()?;
        let groups = &publication.groups;
        let retryable = || self.retryable_conflict_err(other, other_version);
        let mut writes = GroupWrites::default();
        let note_deletions = |writes: &mut GroupWrites, fragments: &[Fragment]| {
            for fragment in fragments {
                let changed = self
                    .initial_fragments
                    .get(&fragment.id)
                    .is_some_and(|(initial, _)| initial.deletion_file != fragment.deletion_file);
                if groups.contains_key(&fragment.id) && changed {
                    writes
                        .deletion_updates
                        .push((fragment.id, fragment.deletion_file.clone()));
                }
            }
        };
        for (fragment_id, reason) in removed_fragments(other)
            .into_iter()
            .filter(|(id, _)| groups.contains_key(id))
        {
            let error = match reason {
                DeferralReason::FragmentRemoved => {
                    self.data_replacement_target_removed_err(fragment_id, other, other_version)
                }
                _ => retryable(),
            };
            writes.conflicts.push((fragment_id, reason, error));
        }
        match &other.operation {
            Operation::Delete {
                updated_fragments, ..
            } => note_deletions(&mut writes, updated_fragments),
            Operation::Update {
                updated_fragments,
                new_fragments,
                fields_modified,
                update_mode,
                ..
            } => {
                let moves_rows = !new_fragments.is_empty()
                    && matches!(update_mode, Some(UpdateMode::RewriteRows) | None);
                for fragment in updated_fragments {
                    let Some(group) = groups.get(&fragment.id) else {
                        continue;
                    };
                    let modified = fields_modified
                        .iter()
                        .filter_map(|id| i32::try_from(*id).ok());
                    if publication.writes_group_fields(group, modified) {
                        writes.conflicts.push((
                            fragment.id,
                            DeferralReason::OutputWritten,
                            retryable(),
                        ));
                    } else if moves_rows && publication.policy == DependencyConflictPolicy::Reject {
                        writes.conflicts.push((
                            fragment.id,
                            DeferralReason::RowVacated,
                            retryable(),
                        ));
                    }
                }
                // Under Skip, moved rows count as deleted at their old addresses.
                note_deletions(&mut writes, updated_fragments);
            }
            Operation::DataReplacement { replacements } => {
                let published = other.cell_flag_changes.as_deref();
                for DataReplacementGroup(fragment_id, file) in replacements {
                    let Some(group) = groups.get(fragment_id) else {
                        continue;
                    };
                    if !publication.writes_group_fields(group, file.fields.iter().copied()) {
                        continue;
                    }
                    let fragment = fragment_key(*fragment_id)?;
                    let publishes_output = published
                        .iter()
                        .flat_map(|changes| changes.updates.iter())
                        .any(|update| {
                            update.value
                                && update.rows.get(&fragment).is_some()
                                && publication
                                    .registry
                                    .definition(update.flag_id)
                                    .is_some_and(|flag| group.fields.contains(&flag.field_id))
                        });
                    let reason = if publishes_output {
                        DeferralReason::NewerResult
                    } else {
                        DeferralReason::OutputWritten
                    };
                    writes.conflicts.push((*fragment_id, reason, retryable()));
                }
            }
            // An overlay wins over any file under it, so installing ours would
            // not hide it, but its cells would no longer be what we assigned.
            Operation::DataOverlay {
                groups: overlay_groups,
            } => {
                for overlay_group in overlay_groups {
                    let Some(group) = groups.get(&overlay_group.fragment_id) else {
                        continue;
                    };
                    if overlay_group.overlays.iter().any(|overlay| {
                        publication
                            .writes_group_fields(group, overlay.data_file.fields.iter().copied())
                    }) {
                        writes.conflicts.push((
                            overlay_group.fragment_id,
                            DeferralReason::OutputWritten,
                            retryable(),
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(writes)
    }

    /// Drop the deferred groups, and the rows that went stale or were vacated
    /// from the assignments of every flag in their group, reporting both, and
    /// which staged values of a deferred group stay reusable. The read version
    /// is kept: the values were computed there.
    pub(super) async fn finish_publication(
        self,
        dataset: &Dataset,
    ) -> Result<(Transaction, PublicationDeferrals)> {
        let Self {
            mut transaction,
            initial_fragments,
            cell_flags,
            ..
        } = self;
        let publication = cell_flags
            .publication
            .ok_or_else(|| wrong_operation_err(&transaction.operation))?;
        let Operation::DataReplacement { replacements } = &mut transaction.operation else {
            return Err(wrong_operation_err(&transaction.operation));
        };
        let mut deferred_files = BTreeMap::new();
        replacements.retain(|DataReplacementGroup(fragment_id, data_file)| {
            let Some((reason, version)) = publication.deferred_groups.get(fragment_id) else {
                return true;
            };
            deferred_files.insert(*fragment_id, (data_file.clone(), *reason, *version));
            false
        });

        // What each version removes from a group's assignments. Sorted by
        // version below, so a row is reported for the first change to it.
        let mut removals: BTreeMap<u64, Vec<(u64, DeferralReason, RoaringBitmap)>> = publication
            .stale_rows
            .iter()
            .map(|(fragment_id, stale)| {
                let entries = stale
                    .iter()
                    .map(|(version, rows)| (*version, DeferralReason::InputChanged, rows.clone()))
                    .collect();
                (*fragment_id, entries)
            })
            .collect();
        // Deferred groups too: a row moved to a new address is gone from the
        // one its staged value was computed for, and its move may have
        // written an input without recording a clear there.
        for (fragment_id, updates) in &publication.deletion_updates {
            let (initial, _) = initial_fragments.get(fragment_id).ok_or_else(|| {
                Error::internal(format!(
                    "publication group on fragment {fragment_id} has no fragment at the read \
                     version"
                ))
            })?;
            let mut deleted = read_fragment_deletion_bitmap(dataset, initial).await?;
            for (version, deletion_file) in updates {
                let now = match deletion_file {
                    Some(deletion_file) => RoaringBitmap::from(
                        read_dataset_deletion_file(dataset, *fragment_id, deletion_file)
                            .await?
                            .as_ref(),
                    ),
                    None => RoaringBitmap::new(),
                };
                let vacated = &now - &deleted;
                if !vacated.is_empty() {
                    removals.entry(*fragment_id).or_default().push((
                        *version,
                        DeferralReason::RowVacated,
                        vacated,
                    ));
                }
                deleted |= now;
            }
        }
        if removals.is_empty() && deferred_files.is_empty() {
            return Ok((transaction, PublicationDeferrals::default()));
        }
        for entries in removals.values_mut() {
            entries.sort_by_key(|(version, reason, _)| (*version, *reason));
        }
        // Taken before the loop below trims them: a deferred file holds the
        // values staged for every row its flags assign at this attempt.
        let staged: BTreeMap<u32, RowAddrTreeMap> = publication
            .flags
            .keys()
            .map(|flag_id| (*flag_id, assigned_rows(&transaction, *flag_id)))
            .collect();
        let staged_rows = |flag_id: u32| {
            staged.get(&flag_id).ok_or_else(|| {
                Error::internal(format!(
                    "published cell flag {flag_id} has no assignment in the publication"
                ))
            })
        };

        let changes = Arc::make_mut(
            transaction
                .cell_flag_changes
                .as_mut()
                .ok_or_else(|| Error::internal("a publication lost its cell flag changes"))?,
        );
        let head_states = dataset
            .manifest
            .cell_flags
            .as_deref()
            .map(|registry| registry.states());
        let mut deferrals = PublicationDeferrals::default();
        let mut valid_rows: BTreeMap<u64, Vec<CellFlagUpdate>> = BTreeMap::new();
        let mut stale_clears = Vec::new();
        for (flag_id, flag) in &publication.flags {
            let mut removed = RowAddrTreeMap::new();
            let mut input_changed = RowAddrTreeMap::new();
            for (fragment, selection) in staged_rows(*flag_id)?.iter() {
                let fragment_id = u64::from(*fragment);
                // Rows outside every group are refused when the commit applies them.
                let Some(group) = publication.groups.get(&fragment_id) else {
                    continue;
                };
                let deferral = deferred_files
                    .get(&fragment_id)
                    .map(|(_, reason, version)| (*reason, *version));
                let mut remaining = materialize(selection, group.physical_rows);
                let mut dropped = RoaringBitmap::new();
                for (version, reason, rows) in removals.get(&fragment_id).into_iter().flatten() {
                    let hit = &remaining & rows;
                    if hit.is_empty() {
                        continue;
                    }
                    remaining -= &hit;
                    dropped |= &hit;
                    if *reason == DeferralReason::InputChanged && deferral.is_none() {
                        input_changed |= &fragment_rows(*fragment, hit.clone());
                    }
                    deferrals.rows.push(DeferredRows {
                        flag_id: *flag_id,
                        rows: fragment_rows(*fragment, hit),
                        reason: *reason,
                        conflicting_version: *version,
                    });
                }
                if let Some((reason, conflicting_version)) = deferral {
                    if !reason.removes_fragment() {
                        // Values on rows an upstream assigns here were computed
                        // from its staged value, which never committed.
                        // Elsewhere the file holds the upstream's copied
                        // snapshot value, and any later change of the upstream
                        // there is an input change of this flag
                        // (InputChanged, removed above), so only direct
                        // upstreams matter, however long the chain.
                        let mut from_staged_input = RoaringBitmap::new();
                        for upstream in &flag.upstreams {
                            if let Some(rows) = staged_rows(*upstream)?.get(fragment) {
                                from_staged_input |= materialize(rows, group.physical_rows);
                            }
                        }
                        from_staged_input &= &remaining;
                        if !from_staged_input.is_empty() {
                            remaining -= &from_staged_input;
                            deferrals.rows.push(DeferredRows {
                                flag_id: *flag_id,
                                rows: fragment_rows(*fragment, from_staged_input),
                                reason: DeferralReason::UpstreamNotPublished,
                                conflicting_version,
                            });
                        }
                        if !remaining.is_empty() {
                            valid_rows
                                .entry(fragment_id)
                                .or_default()
                                .push(CellFlagUpdate {
                                    flag_id: *flag_id,
                                    value: true,
                                    rows: fragment_rows(*fragment, remaining),
                                });
                        }
                    }
                    removed.insert_fragment(*fragment);
                } else {
                    removed |= &fragment_rows(*fragment, dropped);
                }
            }
            if removed.is_empty() {
                continue;
            }
            // The update stays, emptied or not, so its remaining groups keep
            // counting as a publication whose unassigned rows are copies.
            for update in changes
                .updates
                .iter_mut()
                .filter(|update| update.value && update.flag_id == *flag_id)
            {
                update.rows = publication.subtract(&update.rows, &removed)?;
            }
            // The group installs values it no longer assigns on these rows.
            // Where that changes what the output reads as, because it is not
            // masked or because only a sibling's input changed and this flag
            // is still true, clear the flag and so the flags computed from it.
            let visible = if flag.definition.mask_when_false {
                head_states
                    .and_then(|states| states.get(flag_id))
                    .map_or_else(RowAddrTreeMap::new, |state| input_changed & state.as_ref())
            } else {
                input_changed
            };
            if !visible.is_empty() {
                stale_clears.push(CellFlagUpdate {
                    flag_id: *flag_id,
                    value: false,
                    rows: visible,
                });
            }
        }
        // Only Skip defers groups, and under Skip every clear of a published
        // flag is one an earlier attempt added for stale values a group
        // installed. A deferred group installs nothing, and a newer result may
        // own those rows by now.
        if !deferred_files.is_empty() {
            let mut deferred_fragments = RowAddrTreeMap::new();
            for fragment_id in deferred_files.keys() {
                deferred_fragments.insert_fragment(fragment_key(*fragment_id)?);
            }
            for update in changes
                .updates
                .iter_mut()
                .filter(|update| !update.value && publication.publishes(update.flag_id))
            {
                update.rows = publication.subtract(&update.rows, &deferred_fragments)?;
            }
        }
        changes.updates.extend(stale_clears);
        deferrals.groups = deferred_files
            .into_iter()
            .map(
                |(fragment_id, (data_file, reason, conflicting_version))| DeferredGroup {
                    fragment_id,
                    data_file,
                    reason,
                    conflicting_version,
                    valid_rows: valid_rows.remove(&fragment_id).unwrap_or_default(),
                },
            )
            .collect();
        Ok((transaction, deferrals))
    }
}

/// Refuse [`DependencyConflictPolicy::Skip`] for a transaction that is not a
/// publication Skip can trim: deferring part of anything else would apply
/// its other changes partially.
pub fn ensure_skip_eligible(read_manifest: &Manifest, transaction: &Transaction) -> Result<()> {
    let refuse = |why: String| {
        Error::invalid_input(format!(
            "DependencyConflictPolicy::Skip only applies to a publication, a DataReplacement \
             whose cell flag updates all set dependent flags true and whose files write only \
             those flags' outputs: {why}"
        ))
    };
    let Operation::DataReplacement { replacements } = &transaction.operation else {
        return Err(refuse(format!(
            "this transaction is a {} operation",
            transaction.operation.name()
        )));
    };
    let changes = transaction
        .cell_flag_changes
        .as_deref()
        .filter(|changes| !changes.updates.is_empty())
        .ok_or_else(|| refuse("this DataReplacement sets no cell flag".to_string()))?;
    if !changes.registrations.is_empty()
        || !changes.drops.is_empty()
        || !changes.moved_rows.is_empty()
    {
        return Err(refuse(
            "this DataReplacement also registers, drops or moves cell flags".to_string(),
        ));
    }
    let schema = &read_manifest.schema;
    let registry = read_manifest.cell_flags.as_deref();
    let mut outputs = HashSet::new();
    for update in &changes.updates {
        let definition = registry
            .and_then(|registry| registry.definition(update.flag_id))
            .ok_or_else(|| {
                refuse(format!(
                    "cell flag {} is not registered at version {}",
                    update.flag_id, read_manifest.version
                ))
            })?;
        if !update.value || !definition.is_dependent() {
            return Err(refuse(format!(
                "it sets {}, which is {}, to {}",
                definition.label(schema),
                if definition.is_dependent() {
                    "dependent"
                } else {
                    "ordinary"
                },
                update.value
            )));
        }
        outputs.insert(definition.field_id);
    }
    let ancestors = top_level_ancestors(schema);
    for DataReplacementGroup(fragment_id, file) in replacements {
        if let Some(field_id) = file
            .fields
            .iter()
            .filter(|id| **id >= 0)
            .map(|id| ancestors.get(id).copied().unwrap_or(*id))
            .find(|id| !outputs.contains(id))
        {
            return Err(refuse(format!(
                "its file for fragment {fragment_id} writes {}, which is not the output of a flag \
                 it publishes",
                field_label(schema, field_id)
            )));
        }
    }
    Ok(())
}
