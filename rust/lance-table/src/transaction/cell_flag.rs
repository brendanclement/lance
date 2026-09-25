// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Cell flag changes a transaction commits alongside its operation.
//!
//! **Unstable**, like the registry in [`crate::format::cell_flag`]. Flags are
//! referenced by id rather than by name, so a transaction staged against a
//! registration fails once that registration is dropped or replaced.

use lance_core::deepsize::{Context, DeepSizeOf};
use lance_select::RowAddrTreeMap;
use roaring::RoaringBitmap;

use crate::format::CellFlagRegistry;

/// Set a flag to `value` for `rows`.
///
/// Setting a dependent flag true publishes its output, and needs a
/// `DataReplacement` whose files write that output for the assigned
/// fragments. Every row of a replaced fragment that the transaction does not
/// assign must be copied unchanged from the read snapshot, as read through
/// Lance. Lance treats those rows as logically unchanged for invalidation,
/// and as physically written for conflict detection.
#[derive(Debug, Clone, PartialEq, DeepSizeOf)]
pub struct CellFlagUpdate {
    pub flag_id: u32,
    pub value: bool,
    /// Keyed by fragment id with physical row offsets. A `Full` fragment means
    /// every physical row of it.
    pub rows: RowAddrTreeMap,
}

/// Register a flag. Its id is assigned at commit, from the head's allocator.
#[derive(Debug, Clone, PartialEq, DeepSizeOf)]
pub struct CellFlagRegistration {
    /// Id of the top-level output field.
    pub field_id: i32,
    pub name: String,
    /// Ids of top-level source fields. Non-empty makes the flag dependent.
    pub clear_on_write: Vec<i32>,
    pub mask_when_false: bool,
}

/// Rows a row-moving update rewrote into one fragment it adds.
///
/// This maps addresses rather than carrying flag values: the commit copies the
/// state each source row has at the head, so a flag changed by a transaction
/// that committed after this update was staged still follows the row. Only
/// ordinary flags are copied; dependent flags start false on moved rows.
#[derive(Debug, Clone, PartialEq)]
pub struct CellFlagMovedRows {
    /// Path of the first data file of the new fragment, which names it until
    /// the commit assigns its id.
    pub fragment_path: String,
    /// Physical offsets in the new fragment that hold moved rows.
    pub offsets: RoaringBitmap,
    /// Row address (fragment id in the upper 32 bits, physical offset in the
    /// lower) each moved row had at the version the update read, one per
    /// entry of `offsets` in ascending offset order.
    pub source_row_addrs: Vec<u64>,
}

impl DeepSizeOf for CellFlagMovedRows {
    fn deep_size_of_children(&self, context: &mut Context) -> usize {
        self.fragment_path.deep_size_of_children(context)
            + self.offsets.serialized_size()
            + self.source_row_addrs.deep_size_of_children(context)
    }
}

/// All cell flag changes of one transaction.
///
/// Applied at commit in this order: `drops`, `registrations`,
/// `derived_invalidations`, `updates`, then `moved_rows`.
#[derive(Debug, Clone, PartialEq, Default, DeepSizeOf)]
pub struct CellFlagChanges {
    /// Caller-supplied updates, applied in order.
    pub updates: Vec<CellFlagUpdate>,
    pub registrations: Vec<CellFlagRegistration>,
    /// Ids of the flags to drop.
    pub drops: Vec<u32>,
    /// Clears implied by this transaction's changes to the watched fields of
    /// dependent flags, including clears that propagate from an upstream
    /// dependent flag to the flags watching its output. The commit recomputes
    /// them against the head on every attempt, replacing whatever the caller
    /// supplied, so they are recorded even where the flag was already false.
    /// Every entry has `value == false`.
    pub derived_invalidations: Vec<CellFlagUpdate>,
    /// Rows a row-moving update moved into new fragments. A row-moving update
    /// is refused on a fragment where an ordinary flag has true rows unless
    /// some entry here moves rows out of that fragment.
    pub moved_rows: Vec<CellFlagMovedRows>,
}

impl CellFlagChanges {
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
            && self.registrations.is_empty()
            && self.drops.is_empty()
            && self.derived_invalidations.is_empty()
            && self.moved_rows.is_empty()
    }

    /// Whether any explicit update sets a flag to true. Moved rows do not
    /// count: they copy the head's state rather than values read earlier.
    pub fn sets_any_flag(&self) -> bool {
        self.updates.iter().any(|update| update.value)
    }

    /// Whether an explicit update sets a flag that `registry` defines as
    /// dependent to true, i.e. this transaction publishes computed outputs.
    pub fn publishes_dependent_flags(&self, registry: &CellFlagRegistry) -> bool {
        self.updates.iter().any(|update| {
            update.value
                && registry
                    .definition(update.flag_id)
                    .is_some_and(|definition| definition.is_dependent())
        })
    }
}
