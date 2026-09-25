// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! What a publication committed and what it deferred.

use std::sync::Arc;

use lance_select::{RowAddrTreeMap, RowSetOps};
use lance_table::format::DataFile;

use crate::Dataset;
use crate::dataset::transaction::{CellFlagUpdate, Operation, Transaction};

/// How a publication treats rows and groups that concurrent transactions made
/// stale or unsafe to publish.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DependencyConflictPolicy {
    /// Fail the commit, leaving the dataset as it was, when a concurrent
    /// transaction changed the inputs of an assigned row, moved an assigned
    /// row, or wrote, removed or rewrote a group's fragment. Assignments of
    /// rows deleted since the read version are still dropped and the rest is
    /// committed; [`CommitBuilder::execute_with_report`](crate::dataset::CommitBuilder::execute_with_report)
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
    /// A field the flag watches was written on the row, or the flag was
    /// cleared there, after the read version. Also reported for the other
    /// flags whose outputs the same file writes, which publish together. The
    /// row must be recomputed.
    InputChanged,
    /// The row was deleted, or moved to a new address by a row-moving update,
    /// after the read version. Its old address no longer holds a live row.
    RowVacated,
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
    /// The true assignments the group staged, one per flag, whose inputs did
    /// not change up to `checked_version`: their staged values are still
    /// correct results, which a refresh restaging the fragment against the
    /// head can reuse. Rows deleted since the read version are not removed.
    /// Empty when `reason` is that the fragment was removed or rewritten. The
    /// rows whose inputs changed are in [`PublicationReport::deferred_rows`].
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
/// physically written for conflict detection. At commit, every transaction
/// since the read version is checked:
///
/// ```text
/// concurrent change                               Reject      Skip
/// drops or replaces a published flag              error       error
/// invalidates assigned rows (input written)       retryable   rows deferred (InputChanged)
/// deletes assigned rows                           rows deferred (RowVacated)
/// moves assigned rows (row-moving update)         retryable   rows deferred (RowVacated)
/// publishes on a group's fragment and fields      retryable   group deferred (NewerResult)
/// writes a group's fields on its fragment         retryable   group deferred (OutputWritten)
/// removes a group's fragment                      error       group deferred (FragmentRemoved)
/// compacts a group's fragment                     retryable   group deferred (FragmentRewritten)
/// merge, overwrite, restore, dropping or indexing
///   a replaced field, MemWAL state updates        error       error
/// ```
///
/// A row deferred for one flag is deferred for every flag whose output its
/// group's file writes. A group whose rows are deferred still installs its
/// file: the stale values stay stored under a false flag. A deferred group
/// installs nothing, and its staged file stays on storage for the caller to
/// reuse. Rows whose inputs changed are reported as deferred rows whether or
/// not their group was installed, so [`Self::reusable_rows`] is exactly the
/// staged work that is still correct. The exception is a group whose fragment
/// was removed or rewritten: it is reported with that reason, whatever
/// deferred it first, and no row of its fragment is reported as deferred or
/// reusable, since none is left at its staged address.
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
    /// can reuse these values instead of recomputing them, and must recompute
    /// the rows deferred for [`DeferralReason::InputChanged`].
    ///
    /// ```
    /// # use lance::dataset::cell_flag::{DeferralReason, PublicationReport};
    /// # use lance_select::RowSetOps;
    /// # fn example(report: &PublicationReport, flag_id: u32) -> bool {
    /// let reusable = report.reusable_rows(flag_id);
    /// let to_recompute = report.deferred_rows_of(flag_id, DeferralReason::InputChanged);
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
    pub(crate) fn record(
        &mut self,
        transactions: &[(u64, Arc<Transaction>)],
        deferrals: PublicationDeferrals,
    ) {
        self.check_deferred_groups(transactions);
        for deferred in deferrals.rows {
            self.push_deferred_rows(deferred);
        }
        self.deferred_groups.extend(deferrals.groups);
        self.deferred_groups.sort_by_key(|group| group.fragment_id);
        self.drop_rows_of_removed_fragments();
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
    /// checks the groups it installs. That rebase no longer sees them, since
    /// the transaction it rebases dropped them.
    fn check_deferred_groups(&mut self, transactions: &[(u64, Arc<Transaction>)]) {
        let mut stale_rows = Vec::new();
        for group in self
            .deferred_groups
            .iter_mut()
            .filter(|group| !group.reason.removes_fragment())
        {
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
                // flag of the group is stale for all of them.
                let mut stale = RowAddrTreeMap::new();
                for valid in &group.valid_rows {
                    stale |= &(valid.rows.clone() & &invalidated_rows(transaction, valid.flag_id));
                }
                if stale.is_empty() {
                    continue;
                }
                for valid in &mut group.valid_rows {
                    let hit = valid.rows.clone() & &stale;
                    if hit.is_empty() {
                        continue;
                    }
                    valid.rows -= &hit;
                    stale_rows.push(DeferredRows {
                        flag_id: valid.flag_id,
                        rows: hit,
                        reason: DeferralReason::InputChanged,
                        conflicting_version: *version,
                    });
                }
                group.valid_rows.retain(|valid| !valid.rows.is_empty());
            }
        }
        for deferred in stale_rows {
            self.push_deferred_rows(deferred);
        }
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

/// Rows `transaction` clears `flag_id` on, explicitly or through the clears
/// its writes imply. A publication that assigns any of them computed them from
/// inputs that have since changed.
pub fn invalidated_rows(transaction: &Transaction, flag_id: u32) -> RowAddrTreeMap {
    let mut rows = RowAddrTreeMap::new();
    let Some(changes) = transaction.cell_flag_changes.as_deref() else {
        return rows;
    };
    for update in changes
        .derived_invalidations
        .iter()
        .chain(changes.updates.iter().filter(|update| !update.value))
        .filter(|update| update.flag_id == flag_id)
    {
        rows |= &update.rows;
    }
    rows
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
