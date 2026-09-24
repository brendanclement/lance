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

/// Flag state carried onto rows that a row-moving update wrote into a new
/// fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct CarriedCellFlags {
    pub flag_id: u32,
    /// Path of the first data file of the new fragment, which names it until
    /// the commit assigns its id.
    pub fragment_path: String,
    /// Physical offsets in the new fragment whose flag is true.
    pub offsets: RoaringBitmap,
}

impl DeepSizeOf for CarriedCellFlags {
    fn deep_size_of_children(&self, context: &mut Context) -> usize {
        self.fragment_path.deep_size_of_children(context) + self.offsets.serialized_size()
    }
}

/// All cell flag changes of one transaction.
///
/// Applied at commit in this order: `drops`, `registrations`,
/// `derived_invalidations`, `updates`, then `carried`.
#[derive(Debug, Clone, PartialEq, Default, DeepSizeOf)]
pub struct CellFlagChanges {
    /// Caller-supplied updates, applied in order.
    pub updates: Vec<CellFlagUpdate>,
    pub registrations: Vec<CellFlagRegistration>,
    /// Ids of the flags to drop.
    pub drops: Vec<u32>,
    /// Clears implied by this transaction's writes to the watched fields of
    /// dependent flags. The commit recomputes them against the head on every
    /// attempt, replacing whatever the caller supplied, so they are recorded
    /// even where the flag was already false. Every entry has `value == false`.
    pub derived_invalidations: Vec<CellFlagUpdate>,
    pub carried: Vec<CarriedCellFlags>,
    /// Fragments that rows moved out of and whose flag state is carried by
    /// `carried`. A row-moving update is refused on a fragment where an
    /// ordinary flag has true rows unless the fragment is listed here.
    pub carried_from_fragments: Vec<u64>,
}

impl CellFlagChanges {
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
            && self.registrations.is_empty()
            && self.drops.is_empty()
            && self.derived_invalidations.is_empty()
            && self.carried.is_empty()
            && self.carried_from_fragments.is_empty()
    }

    /// Whether any explicit update sets a flag to true.
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
