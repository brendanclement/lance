// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Cell flags: named Boolean state for every cell of one top-level field.
//!
//! **Unstable.** The encoding may change without migration and is gated by
//! [`crate::feature_flags::FLAG_UNSTABLE_CELL_FLAGS`].
//!
//! ```text
//! CellFlagRegistry
//! ├── definitions   one CellFlagDefinition per flag, sorted by flag id
//! ├── next_flag_id  id for the next registration; never reused
//! └── states        flag id -> RowAddrTreeMap of rows whose flag is true
//!                   (fragment id -> physical offsets, Full = every row)
//! ```
//!
//! A flag, or a fragment of a flag, with no entry in `states` is false.

use std::collections::BTreeMap;
use std::sync::Arc;

use lance_core::deepsize::DeepSizeOf;
use lance_core::{Error, Result};
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};

use crate::format::pb;

/// A registered cell flag. Immutable once registered.
#[derive(Debug, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct CellFlagDefinition {
    /// Dataset-unique id, assigned at commit and never reused.
    pub flag_id: u32,
    /// Id of the top-level field the flag is scoped to (its output field).
    pub field_id: i32,
    /// Name, unique among the flags of `field_id`.
    pub name: String,
    /// Ids of the top-level source fields, sorted and deduplicated. When
    /// non-empty the flag is dependent: a write to a source field, or to the
    /// output field itself, clears the flag for the written rows.
    pub clear_on_write: Vec<i32>,
    /// Reads return NULL for the output field wherever the flag is false.
    pub mask_when_false: bool,
}

impl CellFlagDefinition {
    /// Build a definition, normalizing `clear_on_write` to a sorted set.
    pub fn new(
        flag_id: u32,
        field_id: i32,
        name: impl Into<String>,
        mut clear_on_write: Vec<i32>,
        mask_when_false: bool,
    ) -> Self {
        clear_on_write.sort_unstable();
        clear_on_write.dedup();
        Self {
            flag_id,
            field_id,
            name: name.into(),
            clear_on_write,
            mask_when_false,
        }
    }

    /// Whether writes clear this flag, i.e. it declares source fields.
    pub fn is_dependent(&self) -> bool {
        !self.clear_on_write.is_empty()
    }

    /// The fields whose writes clear a dependent flag: its sources and its own
    /// output field. Only meaningful when [`Self::is_dependent`].
    pub fn watched_field_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.clear_on_write
            .iter()
            .copied()
            .chain(std::iter::once(self.field_id))
    }
}

/// Every registered cell flag of a dataset and the rows where each is true.
#[derive(Debug, Clone, PartialEq, DeepSizeOf)]
pub struct CellFlagRegistry {
    definitions: Vec<CellFlagDefinition>,
    next_flag_id: u32,
    states: BTreeMap<u32, Arc<RowAddrTreeMap>>,
}

impl Default for CellFlagRegistry {
    fn default() -> Self {
        Self {
            definitions: Vec::new(),
            // Zero stays unused so an unset proto field never names a flag.
            next_flag_id: 1,
            states: BTreeMap::new(),
        }
    }
}

impl CellFlagRegistry {
    /// Definitions, sorted by flag id.
    pub fn definitions(&self) -> &[CellFlagDefinition] {
        &self.definitions
    }

    /// The id the next registration receives.
    pub fn next_flag_id(&self) -> u32 {
        self.next_flag_id
    }

    /// Rows whose flag is true, per flag. Flags without true rows are absent.
    pub fn states(&self) -> &BTreeMap<u32, Arc<RowAddrTreeMap>> {
        &self.states
    }

    /// The definition of `flag_id`, if it is registered.
    pub fn definition(&self, flag_id: u32) -> Option<&CellFlagDefinition> {
        self.definitions
            .binary_search_by_key(&flag_id, |definition| definition.flag_id)
            .ok()
            .map(|pos| &self.definitions[pos])
    }

    /// The flag named `name` on `field_id`.
    pub fn find(&self, field_id: i32, name: &str) -> Option<&CellFlagDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.field_id == field_id && definition.name == name)
    }

    /// The flag that masks `field_id`, if any. There is at most one.
    pub fn masking_flag(&self, field_id: i32) -> Option<&CellFlagDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.field_id == field_id && definition.mask_when_false)
    }

    /// The dependent flag of `field_id`, if any. There is at most one.
    pub fn dependent_flag(&self, field_id: i32) -> Option<&CellFlagDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.field_id == field_id && definition.is_dependent())
    }

    /// Rows of `fragment_id` whose flag is true, or `None` when none are.
    pub fn true_rows(&self, flag_id: u32, fragment_id: u32) -> Option<&RowAddrSelection> {
        self.states.get(&flag_id)?.get(&fragment_id)
    }

    /// Every field id a definition refers to, as output or source.
    pub fn referenced_field_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.definitions.iter().flat_map(|definition| {
            std::iter::once(definition.field_id).chain(definition.clear_on_write.iter().copied())
        })
    }

    /// Allocate the next flag id.
    pub(crate) fn allocate_flag_id(&mut self) -> Result<u32> {
        let flag_id = self.next_flag_id;
        self.next_flag_id = flag_id.checked_add(1).ok_or_else(|| {
            Error::invalid_input(format!(
                "cannot register a cell flag: flag id {flag_id} is the last one available"
            ))
        })?;
        Ok(flag_id)
    }

    /// Raise the allocator to at least `next_flag_id`, as a high-water mark.
    pub(crate) fn raise_next_flag_id(&mut self, next_flag_id: u32) {
        self.next_flag_id = self.next_flag_id.max(next_flag_id);
    }

    /// Add a definition whose id was just allocated, so it sorts last.
    pub(crate) fn push_definition(&mut self, definition: CellFlagDefinition) {
        debug_assert!(
            self.definitions
                .last()
                .is_none_or(|last| last.flag_id < definition.flag_id),
            "cell flag ids are allocated in increasing order"
        );
        self.definitions.push(definition);
    }

    /// Remove a flag and its state, returning its definition.
    pub(crate) fn remove(&mut self, flag_id: u32) -> Option<CellFlagDefinition> {
        let pos = self
            .definitions
            .binary_search_by_key(&flag_id, |definition| definition.flag_id)
            .ok()?;
        self.states.remove(&flag_id);
        Some(self.definitions.remove(pos))
    }

    pub(crate) fn states_mut(&mut self) -> &mut BTreeMap<u32, Arc<RowAddrTreeMap>> {
        &mut self.states
    }
}

impl From<&CellFlagDefinition> for pb::CellFlagDefinition {
    fn from(definition: &CellFlagDefinition) -> Self {
        Self {
            flag_id: definition.flag_id,
            field_id: definition.field_id,
            name: definition.name.clone(),
            clear_on_write: definition.clear_on_write.clone(),
            mask_when_false: definition.mask_when_false,
        }
    }
}

impl TryFrom<pb::CellFlagDefinition> for CellFlagDefinition {
    type Error = Error;

    fn try_from(message: pb::CellFlagDefinition) -> Result<Self> {
        if message.name.is_empty() {
            return Err(Error::invalid_input(format!(
                "cell flag {} on field id {} has an empty name",
                message.flag_id, message.field_id
            )));
        }
        Ok(Self::new(
            message.flag_id,
            message.field_id,
            message.name,
            message.clear_on_write,
            message.mask_when_false,
        ))
    }
}

pub(crate) fn serialize_row_addr_tree_map(rows: &RowAddrTreeMap) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(rows.serialized_size());
    rows.serialize_into(&mut bytes)
        .expect("serializing into a Vec cannot fail");
    bytes
}

impl From<&CellFlagRegistry> for pb::CellFlagRegistry {
    fn from(registry: &CellFlagRegistry) -> Self {
        Self {
            definitions: registry
                .definitions
                .iter()
                .map(pb::CellFlagDefinition::from)
                .collect(),
            next_flag_id: registry.next_flag_id,
            states: registry
                .states
                .iter()
                .map(|(flag_id, true_rows)| pb::CellFlagState {
                    flag_id: *flag_id,
                    true_rows: serialize_row_addr_tree_map(true_rows),
                })
                .collect(),
        }
    }
}

impl TryFrom<pb::CellFlagRegistry> for CellFlagRegistry {
    type Error = Error;

    fn try_from(message: pb::CellFlagRegistry) -> Result<Self> {
        let definitions = message
            .definitions
            .into_iter()
            .map(CellFlagDefinition::try_from)
            .collect::<Result<Vec<_>>>()?;
        if let Some(pair) = definitions
            .windows(2)
            .find(|pair| pair[0].flag_id >= pair[1].flag_id)
        {
            return Err(Error::invalid_input(format!(
                "cell flag registry definitions must have strictly increasing ids, \
                 found {} before {}",
                pair[0].flag_id, pair[1].flag_id
            )));
        }
        let max_flag_id = definitions
            .last()
            .map_or(0, |definition| definition.flag_id);
        if message.next_flag_id <= max_flag_id {
            return Err(Error::invalid_input(format!(
                "cell flag registry next_flag_id {} must exceed every registered id \
                 (the largest is {max_flag_id})",
                message.next_flag_id
            )));
        }
        let mut states = BTreeMap::new();
        for state in message.states {
            if definitions
                .binary_search_by_key(&state.flag_id, |definition| definition.flag_id)
                .is_err()
            {
                return Err(Error::invalid_input(format!(
                    "cell flag registry has state for flag {}, which is not registered",
                    state.flag_id
                )));
            }
            let true_rows =
                RowAddrTreeMap::deserialize_from(state.true_rows.as_slice()).map_err(|error| {
                    Error::invalid_input(format!(
                        "invalid true rows for cell flag {}: {error}",
                        state.flag_id
                    ))
                })?;
            if true_rows.is_empty() {
                continue;
            }
            if states.insert(state.flag_id, Arc::new(true_rows)).is_some() {
                return Err(Error::invalid_input(format!(
                    "cell flag registry has more than one state for flag {}",
                    state.flag_id
                )));
            }
        }
        Ok(Self {
            definitions,
            next_flag_id: message.next_flag_id,
            states,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roaring::RoaringBitmap;

    fn registry_with_state() -> CellFlagRegistry {
        let mut registry = CellFlagRegistry::default();
        for (field_id, name, clear_on_write, mask) in [
            (3, "ready", vec![2, 1, 2], true),
            (1, "reviewed", vec![], false),
        ] {
            let flag_id = registry.allocate_flag_id().unwrap();
            registry.push_definition(CellFlagDefinition::new(
                flag_id,
                field_id,
                name,
                clear_on_write,
                mask,
            ));
        }
        let mut rows = RowAddrTreeMap::new();
        rows.insert_fragment(0);
        rows.insert_bitmap(4, RoaringBitmap::from_iter([1_u32, 7]));
        registry.states_mut().insert(1, Arc::new(rows));
        registry
    }

    #[test]
    fn registry_round_trips_through_proto() {
        let registry = registry_with_state();
        let decoded = CellFlagRegistry::try_from(pb::CellFlagRegistry::from(&registry)).unwrap();
        assert_eq!(decoded, registry);
        assert_eq!(decoded.next_flag_id(), 3);
        let ready = decoded.definition(1).unwrap();
        assert_eq!(ready.clear_on_write, vec![1, 2]);
        assert!(ready.is_dependent());
        assert_eq!(ready.watched_field_ids().collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(decoded.masking_flag(3), Some(ready));
        assert_eq!(decoded.dependent_flag(3), Some(ready));
        assert_eq!(decoded.find(1, "reviewed").unwrap().flag_id, 2);
        assert!(decoded.dependent_flag(1).is_none());
        assert_eq!(decoded.true_rows(1, 0), Some(&RowAddrSelection::Full));
        assert!(decoded.true_rows(1, 1).is_none());
        assert!(decoded.true_rows(2, 0).is_none());
    }

    #[test]
    fn manifest_carries_registry_through_proto() {
        use crate::format::{DataStorageFormat, Manifest};
        use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
        use lance_core::datatypes::Schema;
        use std::collections::HashMap;

        let arrow_schema = ArrowSchema::new(vec![ArrowField::new("id", DataType::Int32, false)]);
        let mut manifest = Manifest::new(
            Schema::try_from(&arrow_schema).unwrap(),
            Arc::new(vec![]),
            DataStorageFormat::default(),
            HashMap::new(),
        );
        let without = Manifest::try_from(pb::Manifest::from(&manifest)).unwrap();
        assert!(without.cell_flags.is_none());
        assert!(pb::Manifest::from(&manifest).cell_flags.is_none());

        manifest.cell_flags = Some(Arc::new(registry_with_state()));
        let decoded = Manifest::try_from(pb::Manifest::from(&manifest)).unwrap();
        assert_eq!(decoded.cell_flags, manifest.cell_flags);
        let next = Manifest::new_from_previous(&decoded, decoded.schema.clone(), Arc::new(vec![]));
        assert_eq!(next.cell_flags, manifest.cell_flags);
    }

    #[test]
    fn registry_decode_rejects_inconsistent_messages() {
        let valid = pb::CellFlagRegistry::from(&registry_with_state());
        let mut stale_allocator = valid.clone();
        stale_allocator.next_flag_id = 2;
        let mut orphan_state = valid.clone();
        orphan_state.states[0].flag_id = 9;
        let mut unsorted = valid.clone();
        unsorted.definitions.reverse();
        let mut unnamed = valid;
        unnamed.definitions[0].name.clear();
        for (message, expected) in [
            (stale_allocator, "must exceed every registered id"),
            (orphan_state, "which is not registered"),
            (unsorted, "strictly increasing ids"),
            (unnamed, "has an empty name"),
        ] {
            let error = CellFlagRegistry::try_from(message).unwrap_err();
            assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }
}
