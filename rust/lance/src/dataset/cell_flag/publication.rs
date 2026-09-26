// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! What a publication committed and what it deferred.

use std::collections::HashMap;
use std::sync::Arc;

use lance_select::{RowAddrTreeMap, RowSetOps};
use lance_table::format::cell_flag::{fragment_key, top_level_ancestors};
use lance_table::format::{CellFlagDefinition, CellFlagRegistry, DataFile, DeletionFile};
use roaring::RoaringBitmap;

use crate::dataset::transaction::{CellFlagUpdate, DataReplacementGroup, Operation, Transaction};
use crate::io::deletion::read_dataset_deletion_file;
use crate::{Dataset, Error, Result};

/// How a publication treats rows and groups that concurrent transactions made
/// stale or unsafe to publish.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DependencyConflictPolicy {
    /// Fail the commit, leaving the dataset as it was, when a concurrent
    /// transaction changed the inputs of an assigned row, moved any row out of
    /// a group's fragment, or wrote, removed or rewrote a group's fragment.
    /// Assignments of rows deleted since the read version are still dropped and
    /// the rest is committed; [`CommitBuilder::execute_with_report`](crate::dataset::CommitBuilder::execute_with_report)
    /// reports them as [`DeferralReason::RowVacated`].
    #[default]
    Reject,
    /// Publish what is still valid and report the rest as deferred. Only a
    /// publication qualifies: every cell flag update sets a dependent flag
    /// true, and every file writes only the outputs of the flags it publishes.
    /// Commit it with [`CommitBuilder::execute_with_report`](crate::dataset::CommitBuilder::execute_with_report).
    Skip,
}

/// Why a publication did not publish some of its rows or groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeferralReason {
    /// An input of the flag changed on the row after the read version: a
    /// field it watches was written there, the flag was cleared there, or a
    /// publication gave the output of an upstream flag (one whose output it
    /// watches) a new value there, also when that publication republished
    /// this flag on the row and so recorded no clear of it. Also reported for
    /// the other flags whose outputs the same file writes, which publish
    /// together. The row must be recomputed.
    InputChanged,
    /// The row was deleted, or moved to a new address by a row-moving update,
    /// after the read version. Its old address no longer holds a live row.
    RowVacated,
    /// The flag's value on the row was computed from the output of an
    /// upstream flag (one whose output it watches) staged in the same file
    /// and assigned there too. The file was deferred, so that input never
    /// committed. The staged value must not be reused, and later commit
    /// attempts do not check it: recompute it from the upstream value the
    /// follow-up leaves on the row, the committed one or, where the follow-up
    /// also publishes the upstream there, the one it publishes.
    UpstreamNotPublished,
    /// A concurrent publication set a flag whose output the group writes, on
    /// the group's fragment. Installing the older file would overwrite the
    /// newer result.
    NewerResult,
    /// A concurrent transaction wrote a field the group writes, on the
    /// group's fragment, without publishing it.
    OutputWritten,
    /// The group's fragment was removed.
    FragmentRemoved,
    /// Compaction rewrote the group's fragment, so its physical offsets no
    /// longer address the same rows.
    FragmentRewritten,
}

impl DeferralReason {
    /// Whether the group's fragment is gone, so no staged position of it
    /// addresses a row any more.
    pub(crate) fn removes_fragment(self) -> bool {
        matches!(self, Self::FragmentRemoved | Self::FragmentRewritten)
    }
}

/// Rows of one flag that a publication assigned but did not publish. None is
/// on the fragment of a group deferred because that fragment was removed or
/// rewritten: the group accounts for all of its rows.
#[derive(Debug, Clone, PartialEq)]
pub struct DeferredRows {
    pub flag_id: u32,
    /// Keyed by fragment id with physical offsets at the read version.
    pub rows: RowAddrTreeMap,
    pub reason: DeferralReason,
    /// The version whose transaction caused the deferral.
    pub conflicting_version: u64,
}

/// A replacement group a publication did not install.
#[derive(Debug, Clone, PartialEq)]
pub struct DeferredGroup {
    pub fragment_id: u64,
    /// The staged file. It is not deleted, so its values can be reused where
    /// their inputs did not change.
    pub data_file: DataFile,
    /// The first reason found, unless the fragment was later removed or
    /// rewritten up to `checked_version`: that then takes precedence.
    pub reason: DeferralReason,
    /// The version whose transaction caused `reason`.
    pub conflicting_version: u64,
    /// The true assignments the group staged, one per flag, whose staged
    /// values are still correct results at `checked_version`: the row is
    /// still at its address, no input of the flag changed there (see
    /// [`DeferralReason::InputChanged`]), and the value was not computed from
    /// an upstream output this file assigns
    /// ([`DeferralReason::UpstreamNotPublished`]). A refresh restaging the
    /// fragment against the head can reuse them, except on rows where it also
    /// publishes an upstream output of the flag: there the value must be
    /// computed from the upstream value it publishes. A row a concurrent
    /// publication republished the flag on, without changing its inputs,
    /// stays listed: the staged value is computed from the inputs the head
    /// shows. Empty when `reason` is that the fragment was removed or
    /// rewritten. The group's other assigned rows are in
    /// [`PublicationReport::deferred_rows`].
    pub valid_rows: Vec<CellFlagUpdate>,
}

/// The outcome of committing a transaction through
/// [`CommitBuilder::execute_with_report`](crate::dataset::CommitBuilder::execute_with_report).
///
/// A *publication* is a `DataReplacement` that sets dependent cell flags true:
/// a refresh that read a snapshot, computed the flags' outputs, staged one
/// full-fragment file per fragment (a *group*) and assigns the rows it
/// computed. Every row of a replaced fragment that it does not assign must be
/// copied unchanged from the read snapshot, as read through Lance. Lance
/// treats those rows as logically unchanged for invalidation, and as
/// physically written for conflict detection. A flag published together with
/// a dependent flag upstream of it must, on the rows both assign, be computed
/// from the upstream values the publication writes rather than from the
/// snapshot. Assignments are checked against the read version first: an
/// offset beyond its fragment, or a fragment without a group writing the
/// flag's output, fails the commit with `InvalidInput` under either policy.
/// Then every transaction since the read version is checked:
///
/// ```text
/// concurrent change                               Reject      Skip
/// drops or replaces a published flag              error       error
/// changes inputs of assigned rows                 retryable   rows deferred (InputChanged)
/// deletes assigned rows                           rows deferred (RowVacated)
/// moves rows out of a group's fragment            retryable   assigned rows deferred (RowVacated)
///   (row-moving update), assigned or not
/// publishes on a group's fragment and fields      retryable   group deferred (NewerResult)
/// writes a group's fields on its fragment         retryable   group deferred (OutputWritten)
/// removes a group's fragment                      error       group deferred (FragmentRemoved)
/// compacts a group's fragment (unreachable:       retryable   group deferred (FragmentRewritten)
///   compaction is refused while a flag is registered)
/// merge, or indexing a replaced field             retryable   retryable
/// overwrite, restore, MemWAL state updates,       error       error
///   dropping a replaced field
/// ```
///
/// A row deferred for one flag is deferred for every flag whose output its
/// group's file writes. A group whose rows are deferred still installs its
/// file: the stale values stay stored under a false flag. A deferred group
/// installs nothing, and its staged file stays on storage for the caller to
/// reuse. Since its file never committed, a flag's values on the rows where
/// the file also assigns a flag upstream of it, one whose output it watches,
/// were computed from an input no reader sees: they are deferred as
/// [`DeferralReason::UpstreamNotPublished`], while the upstream's own values
/// stay reusable. Rows whose inputs changed
/// ([`DeferralReason::InputChanged`], which a commit need not have recorded as
/// a clear of the flag), and rows deleted or moved, are reported as deferred
/// rows whether or not their group was installed, so [`Self::reusable_rows`]
/// is the staged work that is still correct. A group whose fragment was
/// removed or rewritten is reported with that reason, whatever deferred it
/// first, and no row of its fragment is reported as deferred or reusable,
/// since none is left at its staged address.
///
/// Deferrals accumulate across commit retries. A follow-up refresh of the
/// deferred work must read at `committed_version`, or at `checked_version`
/// when nothing was committed: any earlier snapshot conflicts with this
/// publication's own groups.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicationReport {
    /// The version the transaction's values were computed from.
    pub read_version: u64,
    /// The head the final commit attempt was checked against.
    pub checked_version: u64,
    /// The version committed, or `None` when every group was deferred and no
    /// version was written.
    pub committed_version: Option<u64>,
    /// The true assignments that were committed, one per flag.
    pub published: Vec<CellFlagUpdate>,
    /// Sorted by flag id, conflicting version and reason.
    pub deferred_rows: Vec<DeferredRows>,
    /// Sorted by fragment id.
    pub deferred_groups: Vec<DeferredGroup>,
}

impl PublicationReport {
    pub(crate) fn new(read_version: u64) -> Self {
        Self {
            read_version,
            checked_version: read_version,
            committed_version: None,
            published: Vec::new(),
            deferred_rows: Vec::new(),
            deferred_groups: Vec::new(),
        }
    }

    /// Rows of `flag_id` whose staged values were committed.
    pub fn published_rows(&self, flag_id: u32) -> RowAddrTreeMap {
        union_of(&self.published, flag_id)
    }

    /// Rows of `flag_id` whose staged values are still correct at
    /// `checked_version`: the published rows and the
    /// [`DeferredGroup::valid_rows`]. A follow-up refresh reading at
    /// `committed_version`, or `checked_version` when nothing was committed,
    /// can reuse these values instead of recomputing them, also where a newer
    /// result already set the flag true, and must recompute the rows deferred
    /// for [`DeferralReason::InputChanged`] or
    /// [`DeferralReason::UpstreamNotPublished`]. A reusable value was computed
    /// from the committed values of the flag's inputs, so where the follow-up
    /// also publishes an upstream output of the flag on the row, it must
    /// compute the value from the upstream value it publishes instead.
    ///
    /// ```
    /// # use lance::dataset::cell_flag::{DeferralReason, PublicationReport};
    /// # use lance_select::RowSetOps;
    /// # fn example(report: &PublicationReport, flag_id: u32) -> bool {
    /// let reusable = report.reusable_rows(flag_id);
    /// let to_recompute = report.deferred_rows_of(flag_id, DeferralReason::InputChanged)
    ///     | report.deferred_rows_of(flag_id, DeferralReason::UpstreamNotPublished);
    /// (reusable & &to_recompute).is_empty()
    /// # }
    /// ```
    pub fn reusable_rows(&self, flag_id: u32) -> RowAddrTreeMap {
        let mut rows = self.published_rows(flag_id);
        for group in &self.deferred_groups {
            rows |= &union_of(&group.valid_rows, flag_id);
        }
        rows
    }

    /// Rows of `flag_id` deferred for `reason`, from every conflicting version.
    pub fn deferred_rows_of(&self, flag_id: u32, reason: DeferralReason) -> RowAddrTreeMap {
        let mut rows = RowAddrTreeMap::new();
        for deferred in self
            .deferred_rows
            .iter()
            .filter(|deferred| deferred.flag_id == flag_id && deferred.reason == reason)
        {
            rows |= &deferred.rows;
        }
        rows
    }

    /// Record what a commit attempt that checked `transactions` deferred.
    /// `read` is the dataset at the read version.
    pub(crate) async fn record(
        &mut self,
        read: &Dataset,
        transactions: &[(u64, Arc<Transaction>)],
        deferrals: PublicationDeferrals,
    ) -> Result<()> {
        if !self.deferred_groups.is_empty() {
            self.check_deferred_groups(read, transactions).await?;
        }
        for deferred in deferrals.rows {
            self.push_deferred_rows(deferred);
        }
        self.deferred_groups.extend(deferrals.groups);
        self.deferred_groups.sort_by_key(|group| group.fragment_id);
        self.drop_rows_of_removed_fragments();
        Ok(())
    }

    fn push_deferred_rows(&mut self, deferred: DeferredRows) {
        match self.deferred_rows.iter_mut().find(|existing| {
            existing.flag_id == deferred.flag_id
                && existing.reason == deferred.reason
                && existing.conflicting_version == deferred.conflicting_version
        }) {
            Some(existing) => existing.rows |= &deferred.rows,
            None => self.deferred_rows.push(deferred),
        }
        self.deferred_rows.sort_by_key(|deferred| {
            (
                deferred.flag_id,
                deferred.conflicting_version,
                deferred.reason,
            )
        });
    }

    /// Check groups deferred by an earlier commit attempt against
    /// `transactions`, the versions a later attempt checks, as its rebase
    /// checks the groups it installs: for input changes, and for rows
    /// deleted or moved. That rebase no longer sees them, since the
    /// transaction it rebases dropped them.
    async fn check_deferred_groups(
        &mut self,
        read: &Dataset,
        transactions: &[(u64, Arc<Transaction>)],
    ) -> Result<()> {
        let registry = read.manifest.cell_flags.as_deref().ok_or_else(|| {
            Error::internal(format!(
                "a publication deferred groups, but version {}, which it read, registers no \
                 cell flag",
                read.manifest.version
            ))
        })?;
        let ancestors = top_level_ancestors(&read.manifest.schema);
        let mut stale_rows = Vec::new();
        for group in self
            .deferred_groups
            .iter_mut()
            .filter(|group| !group.reason.removes_fragment())
        {
            let fragment = fragment_key(group.fragment_id)?;
            let read_deletion_file = read
                .manifest
                .fragments
                .iter()
                .find(|fragment| fragment.id == group.fragment_id)
                .ok_or_else(|| {
                    Error::internal(format!(
                        "deferred publication group on fragment {}, which is not in version {}, \
                         the version the publication read",
                        group.fragment_id, read.manifest.version
                    ))
                })?
                .deletion_file
                .as_ref();
            let deleted_at_read = if transactions.iter().any(|(_, transaction)| {
                new_deletion_file(transaction, group.fragment_id, read_deletion_file).is_some()
            }) {
                deleted_rows(read, group.fragment_id, read_deletion_file).await?
            } else {
                RoaringBitmap::new()
            };
            for (version, transaction) in transactions {
                if let Some((_, reason)) = removed_fragments(transaction)
                    .into_iter()
                    .find(|(fragment_id, _)| *fragment_id == group.fragment_id)
                {
                    group.reason = reason;
                    group.conflicting_version = *version;
                    group.valid_rows.clear();
                    break;
                }
                // Sibling outputs publish together, so a row stale for one
                // flag of the group is stale for all of them. Input changes
                // come first, as the rebase sorts a version's removals.
                let mut changed = RowAddrTreeMap::new();
                for valid in &group.valid_rows {
                    let flag = registry.definition(valid.flag_id).ok_or_else(|| {
                        Error::internal(format!(
                            "cell flag {} of a deferred publication group is not registered at \
                             version {}, which the publication read",
                            valid.flag_id, read.manifest.version
                        ))
                    })?;
                    changed |= &(valid.rows.clone()
                        & &input_changed_rows(transaction, flag, registry, &ancestors));
                }
                let mut removals = vec![(DeferralReason::InputChanged, changed)];
                if let Some(deletion_file) =
                    new_deletion_file(transaction, group.fragment_id, read_deletion_file)
                {
                    let vacated = deleted_rows(read, group.fragment_id, deletion_file).await?
                        - &deleted_at_read;
                    let mut rows = RowAddrTreeMap::new();
                    if !vacated.is_empty() {
                        rows.insert_bitmap(fragment, vacated);
                    }
                    removals.push((DeferralReason::RowVacated, rows));
                }
                for (reason, rows) in removals {
                    if rows.is_empty() {
                        continue;
                    }
                    for valid in &mut group.valid_rows {
                        let hit = valid.rows.clone() & &rows;
                        if hit.is_empty() {
                            continue;
                        }
                        valid.rows -= &hit;
                        stale_rows.push(DeferredRows {
                            flag_id: valid.flag_id,
                            rows: hit,
                            reason,
                            conflicting_version: *version,
                        });
                    }
                }
                group.valid_rows.retain(|valid| !valid.rows.is_empty());
            }
        }
        for deferred in stale_rows {
            self.push_deferred_rows(deferred);
        }
        Ok(())
    }

    /// A removed or rewritten fragment keeps no row at the addresses reported
    /// for it, by this attempt or an earlier one, so its group accounts for
    /// them.
    fn drop_rows_of_removed_fragments(&mut self) {
        let mut removed = RowAddrTreeMap::new();
        for group in self
            .deferred_groups
            .iter()
            .filter(|group| group.reason.removes_fragment())
        {
            // Row addresses hold 32-bit fragment ids, so a larger one has no rows.
            if let Ok(fragment) = u32::try_from(group.fragment_id) {
                removed.insert_fragment(fragment);
            }
        }
        for deferred in &mut self.deferred_rows {
            deferred.rows -= &removed;
        }
        self.deferred_rows
            .retain(|deferred| !deferred.rows.is_empty());
    }

    /// Record that `transaction` was committed as `version`.
    pub(crate) fn record_commit(&mut self, version: u64, transaction: &Transaction) {
        self.committed_version = Some(version);
        let mut published: Vec<CellFlagUpdate> = Vec::new();
        for update in transaction
            .cell_flag_changes
            .iter()
            .flat_map(|changes| changes.updates.iter())
            .filter(|update| update.value)
        {
            match published
                .iter_mut()
                .find(|existing| existing.flag_id == update.flag_id)
            {
                Some(existing) => existing.rows |= &update.rows,
                None => published.push(update.clone()),
            }
        }
        published.retain(|update| !update.rows.is_empty());
        published.sort_by_key(|update| update.flag_id);
        self.published = published;
    }
}

fn union_of(updates: &[CellFlagUpdate], flag_id: u32) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    for update in updates.iter().filter(|update| update.flag_id == flag_id) {
        rows |= &update.rows;
    }
    rows
}

/// Rows on which `transaction` changed an input of the dependent flag `flag`:
/// a field `flag` watches, as a masked read shows it, took a new value there,
/// so a value of `flag` computed before `transaction` is stale there. Every
/// check of whether a staged value is still correct uses this function.
///
/// A recorded clear, a derived invalidation or an explicit `value: false`
/// update, is an instruction to set a flag false in that commit, not the set
/// of rows whose inputs changed. The two differ where a publication sets true
/// both `flag` and an upstream flag, a dependent flag whose output `flag`
/// watches: the upstream's output takes a new value on the rows it assigns,
/// but the copy-through exemption records no clear of `flag` where the same
/// transaction republishes it. The rows are:
///
/// - those `transaction` clears `flag` on, explicitly or through the clears
///   its writes imply;
/// - those a `DataReplacement` sets an upstream flag true on, whether or not
///   it also sets `flag` true there. The rows it does not assign are copies,
///   which keep their values. `registry`, the read version's, resolves the
///   upstream. A flag registered after the read version counts as an
///   upstream on the fragments where the transaction's file writes a field
///   `flag` watches, as `ancestors` (the top-level ancestor of every field id
///   of the read schema) maps the file's fields: a publication writes the
///   output of each flag it assigns on every fragment it assigns it on.
///
/// Rows `transaction` deletes or moves to new addresses are not listed: their
/// old addresses no longer hold a live row, and the checks report them as
/// [`DeferralReason::RowVacated`] from the deletion vectors.
pub fn input_changed_rows(
    transaction: &Transaction,
    flag: &CellFlagDefinition,
    registry: &CellFlagRegistry,
    ancestors: &HashMap<i32, i32>,
) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    let Some(changes) = transaction.cell_flag_changes.as_deref() else {
        return rows;
    };
    for clear in changes
        .derived_invalidations
        .iter()
        .chain(&changes.updates)
        .filter(|update| !update.value && update.flag_id == flag.flag_id)
    {
        rows |= &clear.rows;
    }
    let Operation::DataReplacement { replacements } = &transaction.operation else {
        return rows;
    };
    let writes_watched_field: Vec<u32> = replacements
        .iter()
        .filter(|DataReplacementGroup(_, file)| {
            file.fields.iter().any(|field_id| {
                let top = ancestors.get(field_id).unwrap_or(field_id);
                flag.clear_on_write.contains(top)
            })
        })
        .filter_map(|DataReplacementGroup(fragment_id, _)| u32::try_from(*fragment_id).ok())
        .collect();
    for assignment in changes.updates.iter().filter(|update| update.value) {
        match registry.definition(assignment.flag_id) {
            Some(upstream) => {
                if upstream.is_dependent() && flag.clear_on_write.contains(&upstream.field_id) {
                    rows |= &assignment.rows;
                }
            }
            None => {
                let mut on_watched = assignment.rows.clone();
                on_watched.retain_fragments(writes_watched_field.iter().copied());
                rows |= &on_watched;
            }
        }
    }
    rows
}

/// The deletion file `transaction` leaves fragment `fragment_id` with, when
/// it updates the fragment to a deletion file other than `current`.
fn new_deletion_file<'t>(
    transaction: &'t Transaction,
    fragment_id: u64,
    current: Option<&DeletionFile>,
) -> Option<Option<&'t DeletionFile>> {
    match &transaction.operation {
        Operation::Delete {
            updated_fragments, ..
        }
        | Operation::Update {
            updated_fragments, ..
        } => updated_fragments
            .iter()
            .find(|fragment| fragment.id == fragment_id)
            .map(|fragment| fragment.deletion_file.as_ref())
            .filter(|deletion_file| *deletion_file != current),
        _ => None,
    }
}

async fn deleted_rows(
    dataset: &Dataset,
    fragment_id: u64,
    deletion_file: Option<&DeletionFile>,
) -> Result<RoaringBitmap> {
    match deletion_file {
        Some(deletion_file) => Ok(RoaringBitmap::from(
            read_dataset_deletion_file(dataset, fragment_id, deletion_file)
                .await?
                .as_ref(),
        )),
        None => Ok(RoaringBitmap::new()),
    }
}

/// Fragments `transaction` removes or rewrites, with the reason a publication
/// group on one of them is deferred.
pub fn removed_fragments(transaction: &Transaction) -> Vec<(u64, DeferralReason)> {
    match &transaction.operation {
        Operation::Delete {
            deleted_fragment_ids: removed,
            ..
        }
        | Operation::Update {
            removed_fragment_ids: removed,
            ..
        } => removed
            .iter()
            .map(|fragment_id| (*fragment_id, DeferralReason::FragmentRemoved))
            .collect(),
        Operation::Rewrite { groups, .. } => groups
            .iter()
            .flat_map(|group| group.old_fragments.iter())
            .map(|fragment| (fragment.id, DeferralReason::FragmentRewritten))
            .collect(),
        _ => Vec::new(),
    }
}

/// What one commit attempt's rebase deferred.
#[derive(Debug, Default)]
pub struct PublicationDeferrals {
    pub rows: Vec<DeferredRows>,
    pub groups: Vec<DeferredGroup>,
}

/// A commit through
/// [`CommitBuilder::execute_with_report`](crate::dataset::CommitBuilder::execute_with_report).
#[derive(Debug, Clone)]
pub struct PublicationResult {
    /// The committed version, or the head it was checked against when nothing
    /// was committed.
    pub dataset: Dataset,
    pub report: PublicationReport,
}
