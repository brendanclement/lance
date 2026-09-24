// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Commit-time cell flag rules, evaluated against the head manifest.
//!
//! ```text
//! ensure_operation_allowed_with_cell_flags  refuse what the registry cannot follow
//! derive_cell_flag_invalidations            which rows a write clears
//! apply_cell_flag_changes                   the next manifest's registry and state
//! ```
//!
//! All of it reads manifests only; nothing here does I/O.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use arrow_schema::DataType;
use lance_core::datatypes::{Field, Schema};
use lance_core::{Error, Result};
use lance_file::version::ConcreteFileVersion;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use roaring::RoaringBitmap;

use crate::feature_flags::{ENABLE_UNSTABLE_CELL_FLAGS_ENV, cell_flags_enabled};
use crate::format::{CellFlagDefinition, CellFlagRegistry, Fragment, IndexMetadata, Manifest};
use crate::transaction::{
    CellFlagChanges, CellFlagRegistration, CellFlagUpdate, DataReplacementGroup, Operation,
    Transaction, UpdateMode, UpdatedFragmentOffsets,
};

/// Every field id of `schema` mapped to the id of its top-level ancestor.
/// Flags watch top-level fields while data files list leaf ids.
fn top_level_ancestors(schema: &Schema) -> HashMap<i32, i32> {
    let mut ancestors = HashMap::new();
    for top in &schema.fields {
        let mut stack = vec![top];
        while let Some(field) = stack.pop() {
            ancestors.insert(field.id, top.id);
            stack.extend(field.children.iter());
        }
    }
    ancestors
}

fn top_level_field(schema: &Schema, field_id: i32) -> Option<&Field> {
    schema.fields.iter().find(|field| field.id == field_id)
}

fn field_label(schema: &Schema, field_id: i32) -> String {
    match schema.field_path(field_id) {
        Ok(path) => format!("'{path}' (field id {field_id})"),
        Err(_) => format!("field id {field_id}"),
    }
}

fn flag_label(schema: &Schema, definition: &CellFlagDefinition) -> String {
    format!(
        "cell flag '{}' (flag id {}) on {}",
        definition.name,
        definition.flag_id,
        field_label(schema, definition.field_id)
    )
}

fn fragment_key(fragment_id: u64) -> Result<u32> {
    u32::try_from(fragment_id).map_err(|_| {
        Error::invalid_input(format!(
            "fragment id {fragment_id} does not fit the 32 bits a cell flag row address holds"
        ))
    })
}

/// Cells an operation writes in place: `field_ids` over the rows `offsets`
/// of one fragment, or every physical row of it when `offsets` is `None`.
struct WrittenCells<'a> {
    fragment_id: u64,
    field_ids: &'a [i32],
    offsets: Option<&'a RoaringBitmap>,
    /// Set for a `DataReplacement` group, which may publish flag outputs.
    is_replacement: bool,
}

fn written_cells<'a>(
    operation: &'a Operation,
    fields_modified: &'a [i32],
) -> Vec<WrittenCells<'a>> {
    match operation {
        Operation::Update {
            updated_fragments,
            update_mode,
            updated_fragment_offsets,
            ..
        } => {
            // A row-moving update gives the rows new addresses where every flag
            // starts false; it only writes in place when it names fields.
            let in_place = matches!(update_mode, Some(UpdateMode::RewriteColumns))
                || !fields_modified.is_empty();
            if !in_place {
                return Vec::new();
            }
            let offsets = updated_fragment_offsets
                .as_ref()
                .map(|UpdatedFragmentOffsets(offsets)| offsets);
            updated_fragments
                .iter()
                .map(|fragment| WrittenCells {
                    fragment_id: fragment.id,
                    field_ids: fields_modified,
                    offsets: offsets.and_then(|offsets| offsets.get(&fragment.id)),
                    is_replacement: false,
                })
                .collect()
        }
        // A replacement file covers every physical row of its fragment.
        Operation::DataReplacement { replacements } => replacements
            .iter()
            .map(|DataReplacementGroup(fragment_id, file)| WrittenCells {
                fragment_id: *fragment_id,
                field_ids: &file.fields,
                offsets: None,
                is_replacement: true,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn insert_selection(rows: &mut RowAddrTreeMap, fragment: u32, selection: RowAddrSelection) {
    match selection {
        RowAddrSelection::Full => rows.insert_fragment(fragment),
        RowAddrSelection::Partial(bitmap) if bitmap.is_empty() => {}
        RowAddrSelection::Partial(bitmap) => rows.insert_bitmap(fragment, bitmap),
    }
}

fn all_rows(physical_rows: u32) -> RoaringBitmap {
    let mut bitmap = RoaringBitmap::new();
    bitmap.insert_range(0..physical_rows);
    bitmap
}

fn physical_rows_of(fragment: &Fragment) -> Result<u32> {
    let physical_rows = fragment.physical_rows.ok_or_else(|| {
        Error::invalid_input(format!(
            "cannot update cell flag state of fragment {}: its physical row count is unknown",
            fragment.id
        ))
    })?;
    u32::try_from(physical_rows).map_err(|_| {
        Error::invalid_input(format!(
            "fragment {} has {physical_rows} physical rows, more than a cell flag row \
             address holds",
            fragment.id
        ))
    })
}

/// The clears implied by `txn`'s writes to the watched fields of the dependent
/// flags registered at `head`.
///
/// Uses the head registry rather than the read version's, so a flag
/// registered after the write was staged is still cleared. Never looks at flag
/// state: a write is recorded even where the flag is already false, which is
/// what lets a concurrent publisher that read before the write notice it.
///
/// A `DataReplacement` that sets a flag true is its publication, so the rows
/// it assigns in a group that writes the flag's output are not recorded
/// against that flag. Its other rows in that fragment are: their output value
/// was overwritten without being published.
pub fn derive_cell_flag_invalidations(
    head: &Manifest,
    txn: &Transaction,
) -> Result<Vec<CellFlagUpdate>> {
    let Some(registry) = head.cell_flags.as_deref() else {
        return Ok(Vec::new());
    };
    let dependent: Vec<&CellFlagDefinition> = registry
        .definitions()
        .iter()
        .filter(|definition| definition.is_dependent())
        .collect();
    if dependent.is_empty() {
        return Ok(Vec::new());
    }
    let fields_modified: Vec<i32> = match &txn.operation {
        Operation::Update {
            fields_modified, ..
        } => fields_modified
            .iter()
            .map(|id| {
                i32::try_from(*id).map_err(|_| {
                    Error::invalid_input(format!("Update names invalid field id {id}"))
                })
            })
            .collect::<Result<_>>()?,
        _ => Vec::new(),
    };
    let writes = written_cells(&txn.operation, &fields_modified);
    if writes.is_empty() {
        return Ok(Vec::new());
    }

    let ancestors = top_level_ancestors(&head.schema);
    let top_of = |id: &i32| ancestors.get(id).copied().unwrap_or(*id);
    let head_fragments: HashMap<u64, &Fragment> = head
        .fragments
        .iter()
        .map(|fragment| (fragment.id, fragment))
        .collect();
    let published = txn
        .cell_flag_changes
        .as_deref()
        .map(published_rows)
        .unwrap_or_default();

    let mut invalidations = Vec::new();
    for definition in dependent {
        let watched: HashSet<i32> = definition.watched_field_ids().collect();
        let mut rows = RowAddrTreeMap::new();
        for write in &writes {
            let writes_watched = write
                .field_ids
                .iter()
                .any(|id| *id >= 0 && watched.contains(&top_of(id)));
            if !writes_watched {
                continue;
            }
            let fragment = fragment_key(write.fragment_id)?;
            let mut written = match write.offsets {
                Some(offsets) => RowAddrSelection::Partial(offsets.clone()),
                None => RowAddrSelection::Full,
            };
            let publishes_output = write.is_replacement
                && write
                    .field_ids
                    .iter()
                    .any(|id| *id >= 0 && top_of(id) == definition.field_id);
            if publishes_output
                && let Some(assigned) = published
                    .get(&definition.flag_id)
                    .and_then(|assigned| assigned.get(&fragment))
            {
                written = match (assigned, head_fragments.get(&write.fragment_id)) {
                    (RowAddrSelection::Full, _) => continue,
                    (RowAddrSelection::Partial(assigned), Some(head_fragment)) => {
                        match physical_rows_of(head_fragment) {
                            Ok(count) => RowAddrSelection::Partial(all_rows(count) - assigned),
                            // Unknown size: record the whole fragment. The state
                            // is unaffected since updates apply after clears.
                            Err(_) => RowAddrSelection::Full,
                        }
                    }
                    (RowAddrSelection::Partial(_), None) => RowAddrSelection::Full,
                };
            }
            let mut written_rows = RowAddrTreeMap::new();
            insert_selection(&mut written_rows, fragment, written);
            rows |= written_rows;
        }
        if !rows.is_empty() {
            invalidations.push(CellFlagUpdate {
                flag_id: definition.flag_id,
                value: false,
                rows,
            });
        }
    }
    Ok(invalidations)
}

/// Union of the rows each flag is explicitly set true on.
fn published_rows(changes: &CellFlagChanges) -> HashMap<u32, RowAddrTreeMap> {
    let mut published: HashMap<u32, RowAddrTreeMap> = HashMap::new();
    for update in changes.updates.iter().filter(|update| update.value) {
        *published.entry(update.flag_id).or_default() |= &update.rows;
    }
    published
}

/// Refuse a transaction whose operation or cell flag changes the head's
/// registry cannot follow.
///
/// Runs on every commit attempt against the head, so it also catches writes
/// staged before the first flag was registered.
pub fn ensure_operation_allowed_with_cell_flags(head: &Manifest, txn: &Transaction) -> Result<()> {
    let registry = head
        .cell_flags
        .as_deref()
        .filter(|registry| !registry.definitions().is_empty());
    let changes = txn.cell_flag_changes.as_deref();
    if registry.is_none() && changes.is_none() {
        return Ok(());
    }
    if let Some(changes) = changes {
        ensure_changes_allowed(head, &txn.operation, changes)?;
    }
    if let Some(registry) = registry {
        ensure_operation_allowed(head, registry, txn)?;
    }
    Ok(())
}

fn ensure_changes_allowed(
    head: &Manifest,
    operation: &Operation,
    changes: &CellFlagChanges,
) -> Result<()> {
    let name = operation.name();
    if (!changes.registrations.is_empty() || !changes.drops.is_empty())
        && !matches!(operation, Operation::UpdateConfig { .. })
    {
        return Err(Error::not_supported(format!(
            "cell flag registrations and drops must be committed with an UpdateConfig \
             operation, not {name}"
        )));
    }
    if (!changes.carried.is_empty() || !changes.carried_from_fragments.is_empty())
        && !matches!(operation, Operation::Update { .. })
    {
        return Err(Error::not_supported(format!(
            "carried cell flag state can only be committed with an Update operation, not {name}"
        )));
    }
    if !changes.updates.is_empty()
        && !matches!(
            operation,
            Operation::UpdateConfig { .. } | Operation::DataReplacement { .. }
        )
    {
        return Err(Error::not_supported(format!(
            "explicit cell flag updates can only be committed with an UpdateConfig or a \
             DataReplacement operation, not {name}"
        )));
    }
    if !matches!(operation, Operation::DataReplacement { .. })
        && let Some(registry) = head.cell_flags.as_deref()
        && let Some(definition) = changes
            .updates
            .iter()
            .filter(|update| update.value)
            .filter_map(|update| registry.definition(update.flag_id))
            .find(|definition| definition.is_dependent())
    {
        return Err(Error::not_supported(format!(
            "{} is dependent, so only a DataReplacement that writes {} can set it true, not {name}",
            flag_label(&head.schema, definition),
            field_label(&head.schema, definition.field_id),
        )));
    }
    Ok(())
}

fn refuse(
    head: &Manifest,
    operation: &Operation,
    definition: &CellFlagDefinition,
    why: &str,
) -> Error {
    Error::not_supported(format!(
        "{} is not supported on a dataset with {}: {why}",
        operation.name(),
        flag_label(&head.schema, definition)
    ))
}

fn ensure_operation_allowed(
    head: &Manifest,
    registry: &CellFlagRegistry,
    txn: &Transaction,
) -> Result<()> {
    let operation = &txn.operation;
    let Some(first) = registry.definitions().first() else {
        return Ok(());
    };
    let ancestors = top_level_ancestors(&head.schema);
    let top_of = |id: &i32| ancestors.get(id).copied().unwrap_or(*id);
    let flagged: HashSet<i32> = registry.referenced_field_ids().collect();
    let flag_on_field = |field_id: i32| {
        registry
            .definitions()
            .iter()
            .find(|definition| {
                definition.field_id == field_id || definition.clear_on_write.contains(&field_id)
            })
            .unwrap_or(first)
    };
    match operation {
        Operation::Rewrite { .. } => Err(refuse(
            head,
            operation,
            first,
            "compaction would move rows without remapping their cell flag state",
        )),
        Operation::Overwrite { .. } => Err(refuse(
            head,
            operation,
            first,
            "an overwrite can replace or drop flagged fields without clearing their flags",
        )),
        Operation::UpdateMemWalState { .. } => Err(refuse(
            head,
            operation,
            first,
            "MemWAL compaction writes rows that cell flags do not track",
        )),
        Operation::DataOverlay { groups } => {
            for overlay in groups.iter().flat_map(|group| group.overlays.iter()) {
                for field_id in overlay.data_file.fields.iter().filter(|id| **id >= 0) {
                    let top = top_of(field_id);
                    if let Some(definition) = registry
                        .definitions()
                        .iter()
                        .filter(|definition| definition.is_dependent())
                        .find(|definition| definition.watched_field_ids().any(|id| id == top))
                    {
                        return Err(refuse(
                            head,
                            operation,
                            definition,
                            &format!(
                                "the overlay writes {}, which the flag watches",
                                field_label(&head.schema, top)
                            ),
                        ));
                    }
                }
            }
            Ok(())
        }
        Operation::Merge {
            fragments, schema, ..
        } => {
            for field_id in &flagged {
                match top_level_field(schema, *field_id) {
                    None => {
                        return Err(refuse(
                            head,
                            operation,
                            flag_on_field(*field_id),
                            &format!(
                                "the merge removes {} or changes its id",
                                field_label(&head.schema, *field_id)
                            ),
                        ));
                    }
                    Some(field)
                        if !field.nullable && registry.masking_flag(*field_id).is_some() =>
                    {
                        return Err(refuse(
                            head,
                            operation,
                            flag_on_field(*field_id),
                            &format!(
                                "the merge makes the masked output {} non-nullable",
                                field_label(&head.schema, *field_id)
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
            let head_fragments: HashMap<u64, &Fragment> = head
                .fragments
                .iter()
                .map(|fragment| (fragment.id, fragment))
                .collect();
            for fragment in fragments {
                let Some(previous) = head_fragments.get(&fragment.id) else {
                    continue;
                };
                let previous_paths = Transaction::fragment_field_paths(previous);
                let paths = Transaction::fragment_field_paths(fragment);
                let changed = previous_paths
                    .keys()
                    .chain(paths.keys())
                    .copied()
                    .filter(|id| flagged.contains(&top_of(id)))
                    .find(|id| previous_paths.get(id) != paths.get(id));
                if let Some(field_id) = changed {
                    let top = top_of(&field_id);
                    return Err(refuse(
                        head,
                        operation,
                        flag_on_field(top),
                        &format!(
                            "the merge rewrites the data of {} in fragment {}",
                            field_label(&head.schema, top),
                            fragment.id
                        ),
                    ));
                }
            }
            Ok(())
        }
        Operation::Project { schema, .. } => {
            for field_id in &flagged {
                match top_level_field(schema, *field_id) {
                    None => {
                        return Err(refuse(
                            head,
                            operation,
                            flag_on_field(*field_id),
                            &format!(
                                "the projection drops {}; drop the flag first",
                                field_label(&head.schema, *field_id)
                            ),
                        ));
                    }
                    Some(field)
                        if !field.nullable && registry.masking_flag(*field_id).is_some() =>
                    {
                        return Err(refuse(
                            head,
                            operation,
                            flag_on_field(*field_id),
                            &format!(
                                "masked cells read as NULL, so the masked output {} must stay \
                                 nullable",
                                field_label(&head.schema, *field_id)
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
            Ok(())
        }
        Operation::CreateIndex { new_indices, .. } => {
            for index in new_indices {
                if let Some(definition) = index
                    .fields
                    .iter()
                    .find_map(|field_id| registry.masking_flag(*field_id))
                {
                    return Err(refuse(
                        head,
                        operation,
                        definition,
                        &format!(
                            "index '{}' would serve the values the flag masks",
                            index.name
                        ),
                    ));
                }
            }
            Ok(())
        }
        Operation::Update {
            removed_fragment_ids,
            updated_fragments,
            new_fragments,
            update_mode,
            ..
        } => {
            let moves_rows = !matches!(update_mode, Some(UpdateMode::RewriteColumns))
                && !new_fragments.is_empty();
            if !moves_rows {
                return Ok(());
            }
            let carried_from: HashSet<u64> = txn
                .cell_flag_changes
                .as_deref()
                .map(|changes| changes.carried_from_fragments.iter().copied().collect())
                .unwrap_or_default();
            let sources = removed_fragment_ids
                .iter()
                .copied()
                .chain(updated_fragments.iter().map(|fragment| fragment.id))
                .filter(|fragment_id| !carried_from.contains(fragment_id));
            for fragment_id in sources {
                let Ok(fragment) = fragment_key(fragment_id) else {
                    continue;
                };
                if let Some(definition) = registry.definitions().iter().find(|definition| {
                    !definition.is_dependent()
                        && registry.true_rows(definition.flag_id, fragment).is_some()
                }) {
                    return Err(refuse(
                        head,
                        operation,
                        definition,
                        &format!(
                            "the update moves rows out of fragment {fragment_id}, where the flag \
                             is true, without carrying the flag's state"
                        ),
                    ));
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn unknown_flag(flag_id: u32, change: &str) -> Error {
    Error::incompatible_transaction_source(
        format!(
            "cell flag {flag_id} is not registered at the version this transaction commits \
             on: it was dropped or replaced after the transaction was staged, so its {change} \
             cannot apply"
        )
        .into(),
    )
}

fn is_maskable_type(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Boolean
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float16
            | DataType::Float32
            | DataType::Float64
            | DataType::Decimal32(..)
            | DataType::Decimal64(..)
            | DataType::Decimal128(..)
            | DataType::Decimal256(..)
            | DataType::Utf8
            | DataType::LargeUtf8
            | DataType::Binary
            | DataType::LargeBinary
            | DataType::Date32
            | DataType::Date64
            | DataType::Time32(_)
            | DataType::Time64(_)
            | DataType::Timestamp(..)
    )
}

/// Whether making `sources` feed `output` closes a cycle through the
/// registry's dependent flags, i.e. `output` already feeds one of `sources`.
fn creates_dependency_cycle(registry: &CellFlagRegistry, output: i32, sources: &[i32]) -> bool {
    let mut stack = vec![output];
    let mut seen = HashSet::new();
    while let Some(field_id) = stack.pop() {
        if sources.contains(&field_id) {
            return true;
        }
        if !seen.insert(field_id) {
            continue;
        }
        stack.extend(
            registry
                .definitions()
                .iter()
                .filter(|definition| definition.clear_on_write.contains(&field_id))
                .map(|definition| definition.field_id),
        );
    }
    false
}

fn validate_registration(
    registry: &CellFlagRegistry,
    registration: &CellFlagRegistration,
    manifest: &Manifest,
    final_indices: &[IndexMetadata],
) -> Result<()> {
    let schema = &manifest.schema;
    let name = &registration.name;
    let field_id = registration.field_id;
    if name.is_empty() {
        return Err(Error::invalid_input(format!(
            "cannot register a cell flag on {} with an empty name",
            field_label(schema, field_id)
        )));
    }
    let Some(output) = top_level_field(schema, field_id) else {
        return Err(Error::invalid_input(format!(
            "cannot register cell flag '{name}': {} is not a top-level field of the schema",
            field_label(schema, field_id)
        )));
    };
    if let Some(existing) = registry.find(field_id, name) {
        return Err(Error::invalid_input(format!(
            "cannot register cell flag '{name}': {} already has a flag of that name (flag id {})",
            field_label(schema, field_id),
            existing.flag_id
        )));
    }
    for source in &registration.clear_on_write {
        if *source == field_id {
            return Err(Error::invalid_input(format!(
                "cannot register cell flag '{name}': its output {} cannot also be one of its \
                 clear_on_write sources; writes to the output clear a dependent flag anyway",
                field_label(schema, field_id)
            )));
        }
        if top_level_field(schema, *source).is_none() {
            return Err(Error::invalid_input(format!(
                "cannot register cell flag '{name}': clear_on_write source {} is not a \
                 top-level field of the schema",
                field_label(schema, *source)
            )));
        }
    }
    if !registration.clear_on_write.is_empty() {
        if let Some(existing) = registry.dependent_flag(field_id) {
            return Err(Error::invalid_input(format!(
                "cannot register cell flag '{name}': {} already has a dependent flag, '{}' \
                 (flag id {})",
                field_label(schema, field_id),
                existing.name,
                existing.flag_id
            )));
        }
        if creates_dependency_cycle(registry, field_id, &registration.clear_on_write) {
            return Err(Error::invalid_input(format!(
                "cannot register cell flag '{name}': {} already feeds one of its clear_on_write \
                 sources {:?} through other dependent flags, which would make the dependencies \
                 cyclic",
                field_label(schema, field_id),
                registration.clear_on_write
            )));
        }
    }
    if registration.mask_when_false {
        if let Some(existing) = registry.masking_flag(field_id) {
            return Err(Error::invalid_input(format!(
                "cannot register masking cell flag '{name}': {} is already masked by '{}' \
                 (flag id {})",
                field_label(schema, field_id),
                existing.name,
                existing.flag_id
            )));
        }
        if !output.nullable {
            return Err(Error::invalid_input(format!(
                "cannot register masking cell flag '{name}': masked cells read as NULL, but {} \
                 is not nullable",
                field_label(schema, field_id)
            )));
        }
        if output.is_blob() || !is_maskable_type(&output.data_type()) {
            return Err(Error::not_supported(format!(
                "cannot register masking cell flag '{name}': {} has type {}{}, and masking \
                 supports only boolean, numeric, decimal, string, binary and temporal fields",
                field_label(schema, field_id),
                output.data_type(),
                if output.is_blob() { " (blob)" } else { "" }
            )));
        }
        if let Some(index) = final_indices
            .iter()
            .find(|index| index.fields.contains(&field_id))
        {
            return Err(Error::not_supported(format!(
                "cannot register masking cell flag '{name}': index '{}' on {} would serve the \
                 values the flag masks",
                index.name,
                field_label(schema, field_id)
            )));
        }
        if manifest.data_storage_format.version == ConcreteFileVersion::V1 {
            return Err(Error::not_supported(format!(
                "cannot register masking cell flag '{name}': the dataset uses the legacy (v1) \
                 storage format"
            )));
        }
    }
    Ok(())
}

/// A dependent flag is published by writing its output: every fragment it is
/// set true on needs a replacement group whose file carries the output.
fn validate_publication(
    schema: &Schema,
    definition: &CellFlagDefinition,
    update: &CellFlagUpdate,
    operation: &Operation,
) -> Result<()> {
    let Operation::DataReplacement { replacements } = operation else {
        return Err(Error::not_supported(format!(
            "{} is dependent, so only a DataReplacement that writes {} can set it true, not {}",
            flag_label(schema, definition),
            field_label(schema, definition.field_id),
            operation.name()
        )));
    };
    let ancestors = top_level_ancestors(schema);
    for fragment in update.rows.iter().map(|(fragment, _)| u64::from(*fragment)) {
        let publishes = replacements
            .iter()
            .any(|DataReplacementGroup(fragment_id, file)| {
                *fragment_id == fragment
                    && file
                        .fields
                        .iter()
                        .any(|id| ancestors.get(id).copied().unwrap_or(*id) == definition.field_id)
            });
        if !publishes {
            return Err(Error::invalid_input(format!(
                "{} is set true on fragment {fragment}, but the DataReplacement has no group for \
                 that fragment whose file writes {}",
                flag_label(schema, definition),
                field_label(schema, definition.field_id)
            )));
        }
    }
    Ok(())
}

/// Reject rows outside the physical rows of their fragment. A fragment that is
/// not in the next manifest fails only when rows are being set true there.
fn validate_rows(
    rows: &RowAddrTreeMap,
    value: bool,
    flag_id: u32,
    fragments: &HashMap<u32, &Fragment>,
) -> Result<()> {
    for (fragment_id, selection) in rows.iter() {
        let Some(fragment) = fragments.get(fragment_id) else {
            if value {
                return Err(Error::invalid_input(format!(
                    "cell flag {flag_id} is set true on fragment {fragment_id}, which is not in \
                     the dataset"
                )));
            }
            continue;
        };
        if let RowAddrSelection::Partial(offsets) = selection
            && let Some(max) = offsets.max()
        {
            let physical_rows = physical_rows_of(fragment)?;
            if max >= physical_rows {
                return Err(Error::invalid_input(format!(
                    "cell flag {flag_id} update names offset {max} of fragment {fragment_id}, \
                     which has {physical_rows} physical rows"
                )));
            }
        }
    }
    Ok(())
}

fn clear_rows(
    state: &RowAddrTreeMap,
    cleared: &RowAddrTreeMap,
    fragments: &HashMap<u32, &Fragment>,
) -> Result<RowAddrTreeMap> {
    let mut kept = RowAddrTreeMap::new();
    for (fragment_id, selection) in state.iter() {
        let remaining = match (selection, cleared.get(fragment_id)) {
            (_, None) => selection.clone(),
            (_, Some(RowAddrSelection::Full)) => continue,
            (RowAddrSelection::Partial(rows), Some(RowAddrSelection::Partial(cleared))) => {
                RowAddrSelection::Partial(rows - cleared)
            }
            (RowAddrSelection::Full, Some(RowAddrSelection::Partial(cleared))) => {
                let fragment = fragments.get(fragment_id).ok_or_else(|| {
                    Error::internal(format!(
                        "cell flag state names fragment {fragment_id}, which is not in the dataset"
                    ))
                })?;
                RowAddrSelection::Partial(all_rows(physical_rows_of(fragment)?) - cleared)
            }
        };
        insert_selection(&mut kept, *fragment_id, remaining);
    }
    Ok(kept)
}

/// `Full` for a fragment every physical row of which is set; empty entries
/// dropped.
fn normalize(state: &RowAddrTreeMap, fragments: &HashMap<u32, &Fragment>) -> RowAddrTreeMap {
    let mut normalized = RowAddrTreeMap::new();
    for (fragment_id, selection) in state.iter() {
        let selection = match selection {
            RowAddrSelection::Partial(rows)
                if fragments
                    .get(fragment_id)
                    .and_then(|fragment| physical_rows_of(fragment).ok())
                    .is_some_and(|count| {
                        rows.len() == u64::from(count) && rows.max().is_none_or(|max| max < count)
                    }) =>
            {
                RowAddrSelection::Full
            }
            other => other.clone(),
        };
        insert_selection(&mut normalized, *fragment_id, selection);
    }
    normalized
}

fn state_mut(registry: &mut CellFlagRegistry, flag_id: u32) -> &mut RowAddrTreeMap {
    Arc::make_mut(registry.states_mut().entry(flag_id).or_default())
}

impl Transaction {
    /// Apply this transaction's cell flag changes to `manifest`, the manifest
    /// it is building, in the order [`CellFlagChanges`] documents.
    ///
    /// `manifest` must already carry the head's registry and hold the final
    /// schema and fragments. Registrations are validated against those and
    /// against `final_indices`; `current_manifest` is the head.
    pub(crate) fn apply_cell_flag_changes(
        &self,
        manifest: &mut Manifest,
        final_indices: &[IndexMetadata],
        current_manifest: Option<&Manifest>,
    ) -> Result<()> {
        let changes = self.cell_flag_changes.as_deref();
        if manifest.cell_flags.is_none() && changes.is_none() {
            return Ok(());
        }
        let Some(head) = current_manifest else {
            return Err(Error::not_supported(
                "cell flag changes cannot be committed while creating a dataset",
            ));
        };
        let mut registry = manifest.cell_flags.as_deref().cloned().unwrap_or_default();
        let fragment_list = manifest.fragments.clone();
        let fragments: HashMap<u32, &Fragment> = fragment_list
            .iter()
            .filter_map(|fragment| {
                u32::try_from(fragment.id)
                    .ok()
                    .map(|fragment_id| (fragment_id, fragment))
            })
            .collect();
        let mut touched = BTreeSet::new();

        if let Some(changes) = changes {
            for flag_id in &changes.drops {
                if registry.remove(*flag_id).is_none() {
                    return Err(unknown_flag(*flag_id, "drop"));
                }
            }
            // A build that does not understand the feature flag would write a
            // dataset it then refuses to open.
            if !changes.registrations.is_empty() && !cell_flags_enabled() {
                return Err(Error::not_supported(format!(
                    "cell flags are an unstable feature this build only enables when \
                     {ENABLE_UNSTABLE_CELL_FLAGS_ENV} is set"
                )));
            }
            for registration in &changes.registrations {
                validate_registration(&registry, registration, manifest, final_indices)?;
                let flag_id = registry.allocate_flag_id()?;
                registry.push_definition(CellFlagDefinition::new(
                    flag_id,
                    registration.field_id,
                    registration.name.clone(),
                    registration.clear_on_write.clone(),
                    registration.mask_when_false,
                ));
            }
        }

        // State follows its fragment: rows of a removed fragment are gone.
        for (flag_id, state) in registry.states_mut().iter_mut() {
            if state
                .iter()
                .any(|(fragment_id, _)| !fragments.contains_key(fragment_id))
            {
                Arc::make_mut(state).retain_fragments(fragments.keys().copied());
                touched.insert(*flag_id);
            }
        }

        if let Some(changes) = changes {
            for invalidation in &changes.derived_invalidations {
                // A flag dropped in this transaction has nothing left to clear.
                if registry.definition(invalidation.flag_id).is_none() {
                    continue;
                }
                let Some(state) = registry.states().get(&invalidation.flag_id) else {
                    continue;
                };
                let kept = clear_rows(state, &invalidation.rows, &fragments)?;
                *state_mut(&mut registry, invalidation.flag_id) = kept;
                touched.insert(invalidation.flag_id);
            }

            for update in &changes.updates {
                let Some(definition) = registry.definition(update.flag_id) else {
                    return Err(unknown_flag(update.flag_id, "update"));
                };
                validate_rows(&update.rows, update.value, update.flag_id, &fragments)?;
                if update.value {
                    if definition.is_dependent() {
                        validate_publication(
                            &manifest.schema,
                            definition,
                            update,
                            &self.operation,
                        )?;
                    }
                    *state_mut(&mut registry, update.flag_id) |= &update.rows;
                } else if let Some(state) = registry.states().get(&update.flag_id) {
                    let kept = clear_rows(state, &update.rows, &fragments)?;
                    *state_mut(&mut registry, update.flag_id) = kept;
                }
                touched.insert(update.flag_id);
            }

            if !changes.carried.is_empty() {
                let head_fragment_ids: HashSet<u64> =
                    head.fragments.iter().map(|fragment| fragment.id).collect();
                let new_fragments: HashMap<&str, &Fragment> = fragment_list
                    .iter()
                    .filter(|fragment| !head_fragment_ids.contains(&fragment.id))
                    .filter_map(|fragment| {
                        fragment
                            .files
                            .first()
                            .map(|file| (file.path.as_str(), fragment))
                    })
                    .collect();
                for carried in &changes.carried {
                    if registry.definition(carried.flag_id).is_none() {
                        continue;
                    }
                    let fragment = new_fragments
                        .get(carried.fragment_path.as_str())
                        .ok_or_else(|| {
                            Error::invalid_input(format!(
                                "carried state of cell flag {} names fragment file '{}', which \
                                 is not the first data file of a fragment this transaction adds",
                                carried.flag_id, carried.fragment_path
                            ))
                        })?;
                    let fragment_id = fragment_key(fragment.id)?;
                    let mut rows = RowAddrTreeMap::new();
                    rows.insert_bitmap(fragment_id, carried.offsets.clone());
                    validate_rows(&rows, true, carried.flag_id, &fragments)?;
                    *state_mut(&mut registry, carried.flag_id) |= &rows;
                    touched.insert(carried.flag_id);
                }
            }
        }

        for flag_id in touched {
            let Some(state) = registry.states().get(&flag_id) else {
                continue;
            };
            let normalized = normalize(state, &fragments);
            if normalized.is_empty() {
                registry.states_mut().remove(&flag_id);
            } else {
                registry.states_mut().insert(flag_id, Arc::new(normalized));
            }
        }

        manifest.cell_flags = Some(Arc::new(registry));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::overlay::{DataOverlayFile, OverlayCoverage};
    use crate::format::{DataFile, DataStorageFormat};
    use crate::transaction::test_support::{default_build_config, sample_index_metadata};
    use crate::transaction::{
        CarriedCellFlags, DataOverlayGroup, RewriteGroup, TransactionBuilder,
    };
    use arrow_schema::{Field as ArrowField, Fields, Schema as ArrowSchema};
    use rstest::rstest;
    use std::collections::HashMap as StdHashMap;

    const ID: i32 = 0;
    const TITLE: i32 = 1;
    const BODY: i32 = 2;
    const SUMMARY: i32 = 3;
    const META: i32 = 4;
    const LANG: i32 = 5;
    const TAGS: i32 = 6;
    const BLOB: i32 = 8;
    const ROWS: usize = 10;

    fn schema() -> Schema {
        let blob_metadata =
            StdHashMap::from([(lance_arrow::BLOB_META_KEY.to_string(), "true".to_string())]);
        let arrow = ArrowSchema::new(vec![
            ArrowField::new("id", DataType::Int32, false),
            ArrowField::new("title", DataType::Utf8, true),
            ArrowField::new("body", DataType::Utf8, true),
            ArrowField::new("summary", DataType::Utf8, true),
            ArrowField::new(
                "meta",
                DataType::Struct(Fields::from(vec![ArrowField::new(
                    "lang",
                    DataType::Utf8,
                    true,
                )])),
                true,
            ),
            ArrowField::new(
                "tags",
                DataType::List(Arc::new(ArrowField::new("item", DataType::Utf8, true))),
                true,
            ),
            ArrowField::new("blob", DataType::LargeBinary, true).with_metadata(blob_metadata),
        ]);
        let schema = Schema::try_from(&arrow).unwrap();
        assert_eq!(schema.field("meta.lang").unwrap().id, LANG);
        assert_eq!(schema.field("blob").unwrap().id, BLOB);
        schema
    }

    fn fragment(id: u64) -> Fragment {
        let mut fragment = Fragment::new(id).with_file(
            format!("base-{id}.lance"),
            vec![ID, TITLE, BODY, SUMMARY, META, LANG],
            vec![0, 1, 2, 3, 4, 5],
            ConcreteFileVersion::V2_0,
            None,
        );
        fragment.physical_rows = Some(ROWS);
        fragment
    }

    fn manifest() -> Manifest {
        let mut manifest = Manifest::new(
            schema(),
            Arc::new(vec![fragment(0), fragment(1)]),
            DataStorageFormat::new(ConcreteFileVersion::V2_0),
            StdHashMap::new(),
        );
        manifest.max_fragment_id = Some(1);
        manifest
    }

    fn registration_of(
        field_id: i32,
        name: &str,
        sources: &[i32],
        mask: bool,
    ) -> CellFlagRegistration {
        CellFlagRegistration {
            field_id,
            name: name.to_string(),
            clear_on_write: sources.to_vec(),
            mask_when_false: mask,
        }
    }

    fn register_txn(read_version: u64, registrations: Vec<CellFlagRegistration>) -> Transaction {
        TransactionBuilder::new(read_version, update_config())
            .cell_flag_changes(CellFlagChanges {
                registrations,
                ..Default::default()
            })
            .build()
    }

    fn update_config() -> Operation {
        Operation::UpdateConfig {
            config_updates: None,
            table_metadata_updates: None,
            schema_metadata_updates: None,
            field_metadata_updates: StdHashMap::new(),
        }
    }

    fn commit(head: &Manifest, txn: &Transaction) -> Result<Manifest> {
        commit_with_indices(head, txn, vec![])
    }

    fn commit_with_indices(
        head: &Manifest,
        txn: &Transaction,
        indices: Vec<IndexMetadata>,
    ) -> Result<Manifest> {
        ensure_operation_allowed_with_cell_flags(head, txn)?;
        let mut txn = txn.clone();
        let derived = derive_cell_flag_invalidations(head, &txn)?;
        let mut changes = txn
            .cell_flag_changes
            .as_deref()
            .cloned()
            .unwrap_or_default();
        changes.derived_invalidations = derived;
        txn.cell_flag_changes = (!changes.is_empty()).then(|| Arc::new(changes));
        let (manifest, _) =
            txn.build_manifest(Some(head), indices, "txn", &default_build_config())?;
        Ok(manifest)
    }

    /// `summary.ready` (dependent on title and body, masking) and
    /// `title.reviewed` (ordinary), both true everywhere.
    fn flagged_manifest() -> Manifest {
        let head = manifest();
        let registered = commit(
            &head,
            &register_txn(
                1,
                vec![
                    registration_of(SUMMARY, "ready", &[BODY, TITLE], true),
                    registration_of(TITLE, "reviewed", &[], false),
                ],
            ),
        )
        .unwrap();
        let mut all = RowAddrTreeMap::new();
        all.insert_fragment(0);
        all.insert_fragment(1);
        let publish = TransactionBuilder::new(
            2,
            Operation::DataReplacement {
                replacements: vec![
                    DataReplacementGroup(0, summary_file("s0")),
                    DataReplacementGroup(1, summary_file("s1")),
                ],
            },
        )
        .cell_flag_changes(CellFlagChanges {
            updates: vec![
                CellFlagUpdate {
                    flag_id: 1,
                    value: true,
                    rows: all.clone(),
                },
                CellFlagUpdate {
                    flag_id: 2,
                    value: true,
                    rows: all,
                },
            ],
            ..Default::default()
        })
        .build();
        commit(&registered, &publish).unwrap()
    }

    fn summary_file(path: &str) -> DataFile {
        DataFile::new(
            path,
            vec![SUMMARY],
            vec![0],
            ConcreteFileVersion::V2_0,
            None,
            None,
        )
    }

    fn column_file(path: &str, field_ids: Vec<i32>) -> DataFile {
        let columns = (0..field_ids.len() as i32).collect();
        DataFile::new(
            path,
            field_ids,
            columns,
            ConcreteFileVersion::V2_0,
            None,
            None,
        )
    }

    fn registry_of(manifest: &Manifest) -> &CellFlagRegistry {
        manifest.cell_flags.as_deref().unwrap()
    }

    fn rows(entries: &[(u32, Option<&[u32]>)]) -> RowAddrTreeMap {
        let mut rows = RowAddrTreeMap::new();
        for (fragment, offsets) in entries {
            match offsets {
                None => rows.insert_fragment(*fragment),
                Some(offsets) => {
                    rows.insert_bitmap(*fragment, RoaringBitmap::from_iter(offsets.iter().copied()))
                }
            }
        }
        rows
    }

    fn rewrite_columns(
        fields_modified: Vec<u32>,
        offsets: Option<StdHashMap<u64, RoaringBitmap>>,
    ) -> Operation {
        Operation::Update {
            removed_fragment_ids: vec![],
            updated_fragments: vec![fragment(0)],
            new_fragments: vec![],
            fields_modified,
            compacted_sstables: vec![],
            fields_for_preserving_frag_bitmap: vec![],
            update_mode: Some(UpdateMode::RewriteColumns),
            inserted_rows_filter: None,
            updated_fragment_offsets: offsets.map(UpdatedFragmentOffsets),
        }
    }

    fn replacement(fragment_id: u64, file: DataFile) -> Operation {
        Operation::DataReplacement {
            replacements: vec![DataReplacementGroup(fragment_id, file)],
        }
    }

    #[test]
    fn registration_assigns_monotonic_ids_and_sets_feature_flag() {
        let head = flagged_manifest();
        let registry = registry_of(&head);
        assert_eq!(registry.next_flag_id(), 3);
        let ready = registry.find(SUMMARY, "ready").unwrap();
        assert_eq!(ready.flag_id, 1);
        assert_eq!(ready.clear_on_write, vec![TITLE, BODY]);
        assert_eq!(
            registry.true_rows(1, 0),
            Some(&RowAddrSelection::Full),
            "a replacement group publishes its own flag"
        );
        assert_ne!(
            head.reader_feature_flags & crate::feature_flags::FLAG_UNSTABLE_CELL_FLAGS,
            0
        );

        // Replacing drops the old id and never hands it out again.
        let replace = TransactionBuilder::new(3, update_config())
            .cell_flag_changes(CellFlagChanges {
                drops: vec![1],
                registrations: vec![registration_of(SUMMARY, "ready", &[BODY], true)],
                ..Default::default()
            })
            .build();
        let replaced = commit(&head, &replace).unwrap();
        let registry = registry_of(&replaced);
        assert!(registry.definition(1).is_none());
        assert!(!registry.states().contains_key(&1));
        assert_eq!(registry.find(SUMMARY, "ready").unwrap().flag_id, 3);
        assert_eq!(registry.next_flag_id(), 4);
    }

    #[rstest]
    #[case::rewrite_columns_with_offsets(
        rewrite_columns(vec![BODY as u32], Some(StdHashMap::from([(0, RoaringBitmap::from_iter([2_u32, 5]))]))),
        Some(rows(&[(0, Some(&[2, 5]))]))
    )]
    #[case::rewrite_columns_without_offsets(
        rewrite_columns(vec![TITLE as u32], None),
        Some(rows(&[(0, None)]))
    )]
    #[case::rewrite_columns_of_output(
        rewrite_columns(vec![SUMMARY as u32], None),
        Some(rows(&[(0, None)]))
    )]
    #[case::rewrite_columns_of_unwatched_field(rewrite_columns(vec![ID as u32], None), None)]
    #[case::replacement_of_source(replacement(1, column_file("b", vec![BODY])), Some(rows(&[(1, None)])))]
    #[case::replacement_of_output_without_publication(
        replacement(0, summary_file("s")),
        Some(rows(&[(0, None)]))
    )]
    #[case::replacement_of_unwatched_field(replacement(0, column_file("i", vec![ID])), None)]
    fn derive_clears_written_cells(
        #[case] operation: Operation,
        #[case] expected: Option<RowAddrTreeMap>,
    ) {
        let head = flagged_manifest();
        let txn = Transaction::new(3, operation, None);
        let derived = derive_cell_flag_invalidations(&head, &txn).unwrap();
        let expected: Vec<CellFlagUpdate> = expected
            .into_iter()
            .map(|rows| CellFlagUpdate {
                flag_id: 1,
                value: false,
                rows,
            })
            .collect();
        assert_eq!(derived, expected);

        // Applying the derived clears leaves exactly the unwritten rows true.
        let next = commit(&head, &txn).unwrap();
        let mut remaining = rows(&[(0, None), (1, None)]);
        for invalidation in &expected {
            remaining = clear_rows(
                &remaining,
                &invalidation.rows,
                &head
                    .fragments
                    .iter()
                    .map(|fragment| (fragment.id as u32, fragment))
                    .collect(),
            )
            .unwrap();
        }
        let state = registry_of(&next)
            .states()
            .get(&1)
            .map(|state| state.as_ref().clone());
        assert_eq!(state.unwrap_or_default(), remaining);
        // Ordinary flags only change explicitly.
        assert_eq!(
            registry_of(&next).true_rows(2, 0),
            Some(&RowAddrSelection::Full)
        );
    }

    #[test]
    fn derive_matches_leaf_writes_to_watched_ancestor() {
        let head = commit(
            &manifest(),
            &register_txn(1, vec![registration_of(SUMMARY, "ready", &[META], false)]),
        )
        .unwrap();
        let txn = Transaction::new(2, replacement(0, column_file("lang", vec![LANG])), None);
        let derived = derive_cell_flag_invalidations(&head, &txn).unwrap();
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].rows, rows(&[(0, None)]));
    }

    #[test]
    fn derive_records_clears_when_already_false_and_against_head_registry() {
        // The write was staged against version 1, before the flag existed.
        let staged = Transaction::new(1, rewrite_columns(vec![BODY as u32], None), None);
        let head = commit(
            &manifest(),
            &register_txn(1, vec![registration_of(SUMMARY, "ready", &[BODY], true)]),
        )
        .unwrap();
        assert!(registry_of(&head).states().is_empty(), "never published");
        let derived = derive_cell_flag_invalidations(&head, &staged).unwrap();
        assert_eq!(
            derived,
            vec![CellFlagUpdate {
                flag_id: 1,
                value: false,
                rows: rows(&[(0, None)]),
            }]
        );
        assert!(
            derive_cell_flag_invalidations(&manifest(), &staged)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn derive_exempts_only_the_rows_a_publication_assigns() {
        let head = commit(
            &manifest(),
            &register_txn(
                1,
                vec![
                    registration_of(SUMMARY, "ready", &[BODY], true),
                    // Chained: a flag on title that watches summary.
                    registration_of(TITLE, "fresh", &[SUMMARY], false),
                ],
            ),
        )
        .unwrap();
        let publish = TransactionBuilder::new(2, replacement(0, summary_file("s")))
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: true,
                    rows: rows(&[(0, Some(&[0, 1, 2]))]),
                }],
                ..Default::default()
            })
            .build();
        let derived = derive_cell_flag_invalidations(&head, &publish).unwrap();
        let unassigned: Vec<u32> = (3..ROWS as u32).collect();
        assert_eq!(
            derived,
            vec![
                CellFlagUpdate {
                    flag_id: 1,
                    value: false,
                    rows: rows(&[(0, Some(&unassigned))]),
                },
                CellFlagUpdate {
                    flag_id: 2,
                    value: false,
                    rows: rows(&[(0, None)]),
                },
            ]
        );
        let next = commit(&head, &publish).unwrap();
        assert_eq!(
            registry_of(&next).true_rows(1, 0),
            Some(&RowAddrSelection::Partial(RoaringBitmap::from_iter([
                0_u32, 1, 2
            ])))
        );

        // A full publication records nothing against its own flag.
        let publish_all = TransactionBuilder::new(2, replacement(0, summary_file("s")))
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: true,
                    rows: rows(&[(0, None)]),
                }],
                ..Default::default()
            })
            .build();
        let derived = derive_cell_flag_invalidations(&head, &publish_all).unwrap();
        assert_eq!(
            derived.iter().map(|u| u.flag_id).collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn apply_normalizes_full_fragments_and_drops_empty_state() {
        let head = commit(
            &manifest(),
            &register_txn(1, vec![registration_of(TITLE, "reviewed", &[], false)]),
        )
        .unwrap();
        let every_offset: Vec<u32> = (0..ROWS as u32).collect();
        let set = TransactionBuilder::new(2, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: true,
                    rows: rows(&[(0, Some(&every_offset)), (1, Some(&[4]))]),
                }],
                ..Default::default()
            })
            .build();
        let set = commit(&head, &set).unwrap();
        assert_eq!(
            registry_of(&set).states().get(&1).unwrap().as_ref(),
            &rows(&[(0, None), (1, Some(&[4]))])
        );

        // Clearing part of a Full fragment materializes its physical rows.
        let clear = TransactionBuilder::new(3, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: false,
                    rows: rows(&[(0, Some(&[9])), (1, None)]),
                }],
                ..Default::default()
            })
            .build();
        let cleared = commit(&set, &clear).unwrap();
        let expected: Vec<u32> = (0..9).collect();
        assert_eq!(
            registry_of(&cleared).states().get(&1).unwrap().as_ref(),
            &rows(&[(0, Some(&expected))])
        );

        let clear_all = TransactionBuilder::new(4, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: false,
                    rows: rows(&[(0, None)]),
                }],
                ..Default::default()
            })
            .build();
        let empty = commit(&cleared, &clear_all).unwrap();
        assert!(registry_of(&empty).states().is_empty());
    }

    #[test]
    fn apply_drops_state_of_removed_fragments() {
        let head = flagged_manifest();
        let delete = Transaction::new(
            3,
            Operation::Delete {
                updated_fragments: vec![],
                deleted_fragment_ids: vec![0],
                predicate: "true".to_string(),
            },
            None,
        );
        let next = commit(&head, &delete).unwrap();
        for flag_id in [1, 2] {
            assert_eq!(
                registry_of(&next).states().get(&flag_id).unwrap().as_ref(),
                &rows(&[(1, None)])
            );
        }
    }

    #[test]
    fn apply_carries_state_onto_new_fragments() {
        let head = flagged_manifest();
        // Id 0 is the placeholder the commit replaces with a fresh id.
        let mut moved = fragment(0);
        moved.files[0].path = "moved.lance".to_string();
        moved.physical_rows = Some(3);
        let update = TransactionBuilder::new(
            3,
            Operation::Update {
                removed_fragment_ids: vec![0],
                updated_fragments: vec![],
                new_fragments: vec![moved],
                fields_modified: vec![],
                compacted_sstables: vec![],
                fields_for_preserving_frag_bitmap: vec![],
                update_mode: Some(UpdateMode::RewriteRows),
                inserted_rows_filter: None,
                updated_fragment_offsets: None,
            },
        )
        .cell_flag_changes(CellFlagChanges {
            carried: vec![CarriedCellFlags {
                flag_id: 2,
                fragment_path: "moved.lance".to_string(),
                offsets: RoaringBitmap::from_iter([0_u32, 2]),
            }],
            carried_from_fragments: vec![0],
            ..Default::default()
        })
        .build();
        let next = commit(&head, &update).unwrap();
        // The new fragment takes the next id after the high-water mark.
        assert_eq!(
            registry_of(&next).states().get(&2).unwrap().as_ref(),
            &rows(&[(1, None), (2, Some(&[0, 2]))])
        );
        // Dependent flags are not carried: moved rows start unassigned.
        assert_eq!(
            registry_of(&next).states().get(&1).unwrap().as_ref(),
            &rows(&[(1, None)])
        );
    }

    #[rstest]
    #[case::empty_name(registration_of(SUMMARY, "", &[], false), true, "with an empty name")]
    #[case::nested_output(registration_of(LANG, "x", &[], false), true, "is not a top-level field")]
    #[case::missing_output(registration_of(42, "x", &[], false), true, "is not a top-level field")]
    #[case::duplicate_name(registration_of(SUMMARY, "ready", &[], false), true, "already has a flag of that name")]
    #[case::source_is_output(registration_of(BODY, "x", &[BODY], false), true, "cannot also be one of its")]
    #[case::nested_source(registration_of(BODY, "x", &[LANG], false), true, "source 'meta.lang' (field id 5) is not a top-level")]
    #[case::second_dependent(registration_of(SUMMARY, "other", &[ID], false), true, "already has a dependent flag")]
    #[case::second_mask(registration_of(SUMMARY, "other", &[], true), true, "is already masked by")]
    #[case::cycle(registration_of(TITLE, "x", &[SUMMARY], false), true, "cyclic")]
    #[case::non_nullable_mask(registration_of(ID, "x", &[], true), true, "is not nullable")]
    #[case::nested_type_mask(registration_of(TAGS, "x", &[], true), false, "masking supports only")]
    #[case::blob_mask(registration_of(BLOB, "x", &[], true), false, "(blob)")]
    fn registration_validation_errors(
        #[case] registration: CellFlagRegistration,
        #[case] is_invalid_input: bool,
        #[case] expected: &str,
    ) {
        // `summary.ready` depends on title; `body.derived` on summary; so a
        // flag on title watching summary closes a cycle.
        let head = commit(
            &manifest(),
            &register_txn(
                1,
                vec![
                    registration_of(SUMMARY, "ready", &[TITLE], true),
                    registration_of(BODY, "derived", &[SUMMARY], false),
                ],
            ),
        )
        .unwrap();
        let error = commit(&head, &register_txn(2, vec![registration])).unwrap_err();
        if is_invalid_input {
            assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        } else {
            assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        }
        assert!(error.to_string().contains(expected), "{error}");
    }

    #[test]
    fn masking_registration_rejects_indexed_output_and_legacy_storage() {
        let mut index = sample_index_metadata("summary_idx");
        index.fields = vec![SUMMARY];
        let error = commit_with_indices(
            &manifest(),
            &register_txn(1, vec![registration_of(SUMMARY, "ready", &[], true)]),
            vec![index],
        )
        .unwrap_err();
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(error.to_string().contains("index 'summary_idx'"), "{error}");

        let mut legacy = manifest();
        legacy.data_storage_format = DataStorageFormat::new(ConcreteFileVersion::V1);
        let error = commit(
            &legacy,
            &register_txn(1, vec![registration_of(SUMMARY, "ready", &[], true)]),
        )
        .unwrap_err();
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(error.to_string().contains("legacy (v1)"), "{error}");
    }

    #[test]
    fn apply_fences_changes_for_unknown_flags() {
        let head = flagged_manifest();
        for (changes, expected) in [
            (
                CellFlagChanges {
                    drops: vec![7],
                    ..Default::default()
                },
                "its drop cannot apply",
            ),
            (
                CellFlagChanges {
                    updates: vec![CellFlagUpdate {
                        flag_id: 7,
                        value: false,
                        rows: rows(&[(0, None)]),
                    }],
                    ..Default::default()
                },
                "its update cannot apply",
            ),
        ] {
            let txn = TransactionBuilder::new(3, update_config())
                .cell_flag_changes(changes)
                .build();
            let error = commit(&head, &txn).unwrap_err();
            assert!(
                matches!(error, Error::IncompatibleTransaction { .. }),
                "{error}"
            );
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[rstest]
    #[case::wrong_fragment(replacement(1, summary_file("s")), rows(&[(0, None)]), "has no group for that fragment")]
    #[case::group_without_output(replacement(0, column_file("b", vec![BODY])), rows(&[(0, None)]), "has no group for that fragment")]
    #[case::offset_out_of_range(replacement(0, summary_file("s")), rows(&[(0, Some(&[10]))]), "which has 10 physical rows")]
    #[case::missing_fragment(replacement(0, summary_file("s")), rows(&[(0, None), (5, None)]), "fragment 5, which is not in the dataset")]
    fn publication_validation_errors(
        #[case] operation: Operation,
        #[case] assigned: RowAddrTreeMap,
        #[case] expected: &str,
    ) {
        let head = flagged_manifest();
        let txn = TransactionBuilder::new(3, operation)
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 1,
                    value: true,
                    rows: assigned,
                }],
                ..Default::default()
            })
            .build();
        let error = commit(&head, &txn).unwrap_err();
        assert!(matches!(error, Error::InvalidInput { .. }), "{error}");
        assert!(error.to_string().contains(expected), "{error}");
    }

    fn overlay_on(field_id: i32) -> Operation {
        Operation::DataOverlay {
            groups: vec![DataOverlayGroup {
                fragment_id: 0,
                overlays: vec![DataOverlayFile {
                    data_file: DataFile::new_legacy_from_fields("o.lance", vec![field_id], None),
                    coverage: OverlayCoverage::dense(RoaringBitmap::from_iter([0_u32])),
                    committed_version: 0,
                }],
            }],
        }
    }

    fn project_without(dropped: i32) -> Operation {
        let schema = schema();
        let kept: Vec<i32> = schema
            .fields
            .iter()
            .map(|field| field.id)
            .filter(|id| *id != dropped)
            .collect();
        Operation::Project {
            schema: schema.project_by_ids(&kept, true),
            preserves_nullability: true,
        }
    }

    fn project_non_nullable(field_id: i32) -> Operation {
        let mut schema = schema();
        schema.mut_field_by_id(field_id).unwrap().nullable = false;
        Operation::Project {
            schema,
            preserves_nullability: false,
        }
    }

    fn merge_rewriting(field_id: i32) -> Operation {
        let head = flagged_manifest();
        let mut rewritten = head.fragments[0].clone();
        rewritten.files[0].fields = rewritten.files[0]
            .fields
            .iter()
            .map(|id| if *id == field_id { -2 } else { *id })
            .collect::<Vec<_>>()
            .into();
        rewritten
            .files
            .push(column_file("new.lance", vec![field_id]));
        Operation::Merge {
            fragments: vec![rewritten, head.fragments[1].clone()],
            schema: schema(),
            preserves_nullability: true,
        }
    }

    fn merge_adding_column() -> Operation {
        let mut schema = schema();
        let mut added = schema.fields[1].clone();
        added.name = "extra".to_string();
        added.id = 20;
        schema.fields.push(added);
        let head = flagged_manifest();
        let mut extended = head.fragments[0].clone();
        extended.files.push(column_file("extra.lance", vec![20]));
        Operation::Merge {
            fragments: vec![extended, head.fragments[1].clone()],
            schema,
            preserves_nullability: true,
        }
    }

    fn create_index_on(field_id: i32) -> Operation {
        let mut index = sample_index_metadata("idx");
        index.fields = vec![field_id];
        Operation::CreateIndex {
            new_indices: vec![index],
            removed_indices: vec![],
        }
    }

    fn row_moving_update(from: u64) -> Operation {
        let mut moved = fragment(0);
        moved.physical_rows = Some(1);
        Operation::Update {
            removed_fragment_ids: vec![],
            updated_fragments: vec![fragment(from)],
            new_fragments: vec![moved],
            fields_modified: vec![],
            compacted_sstables: vec![],
            fields_for_preserving_frag_bitmap: vec![],
            update_mode: Some(UpdateMode::RewriteRows),
            inserted_rows_filter: None,
            updated_fragment_offsets: None,
        }
    }

    #[rstest]
    #[case::rewrite(Operation::Rewrite { groups: vec![RewriteGroup { old_fragments: vec![fragment(0)], new_fragments: vec![fragment(9)] }], rewritten_indices: vec![], frag_reuse_index: None }, Some("compaction would move rows"))]
    #[case::overwrite(Operation::Overwrite { fragments: vec![], schema: schema(), config_upsert_values: None, initial_bases: None }, Some("Overwrite is not supported"))]
    #[case::mem_wal(Operation::UpdateMemWalState { compacted_sstables: vec![] }, Some("MemWAL compaction"))]
    #[case::overlay_on_source(overlay_on(BODY), Some("the overlay writes 'body'"))]
    #[case::overlay_on_output(overlay_on(SUMMARY), Some("the overlay writes 'summary'"))]
    #[case::overlay_on_unwatched(overlay_on(ID), None)]
    #[case::merge_rewrites_source(merge_rewriting(BODY), Some("rewrites the data of 'body'"))]
    #[case::merge_adds_column(merge_adding_column(), None)]
    #[case::project_drops_source(project_without(TITLE), Some("drops 'title'"))]
    #[case::project_drops_unflagged(project_without(TAGS), None)]
    #[case::project_tightens_mask(project_non_nullable(SUMMARY), Some("must stay nullable"))]
    #[case::index_on_masked(create_index_on(SUMMARY), Some("index 'idx' would serve"))]
    #[case::index_on_source(create_index_on(BODY), None)]
    #[case::moves_ordinary_flag(row_moving_update(0), Some("moves rows out of fragment 0"))]
    fn operation_gate(#[case] operation: Operation, #[case] refusal: Option<&str>) {
        let head = flagged_manifest();
        let txn = Transaction::new(3, operation, None);
        let result = ensure_operation_allowed_with_cell_flags(&head, &txn);
        match refusal {
            None => result.unwrap(),
            Some(expected) => {
                let error = result.unwrap_err();
                assert!(matches!(error, Error::NotSupported { .. }), "{error}");
                assert!(error.to_string().contains(expected), "{error}");
                assert!(error.to_string().contains("flag id"), "{error}");
            }
        }
        // Without any registered flag the same operations are unaffected.
        ensure_operation_allowed_with_cell_flags(&manifest(), &txn).unwrap();
    }

    #[test]
    fn row_moving_update_is_allowed_when_state_is_carried_or_dependent() {
        let head = flagged_manifest();
        let carried = TransactionBuilder::new(3, row_moving_update(0))
            .cell_flag_changes(CellFlagChanges {
                carried_from_fragments: vec![0],
                ..Default::default()
            })
            .build();
        ensure_operation_allowed_with_cell_flags(&head, &carried).unwrap();

        // Only the dependent flag is true: moving rows just unassigns it.
        let clear_reviewed = TransactionBuilder::new(3, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 2,
                    value: false,
                    rows: rows(&[(0, None), (1, None)]),
                }],
                ..Default::default()
            })
            .build();
        let head = commit(&head, &clear_reviewed).unwrap();
        let moving = Transaction::new(4, row_moving_update(0), None);
        ensure_operation_allowed_with_cell_flags(&head, &moving).unwrap();
    }

    #[rstest]
    #[case::registration_on_replacement(
        replacement(0, summary_file("s")),
        CellFlagChanges { registrations: vec![registration_of(TITLE, "x", &[], false)], ..Default::default() },
        "must be committed with an UpdateConfig operation, not DataReplacement"
    )]
    #[case::drop_on_update(
        rewrite_columns(vec![], None),
        CellFlagChanges { drops: vec![1], ..Default::default() },
        "not Update"
    )]
    #[case::carried_on_replacement(
        replacement(0, summary_file("s")),
        CellFlagChanges { carried_from_fragments: vec![0], ..Default::default() },
        "only be committed with an Update operation"
    )]
    #[case::updates_on_append(
        Operation::Append { fragments: vec![] },
        CellFlagChanges { updates: vec![CellFlagUpdate { flag_id: 2, value: true, rows: rows(&[(0, None)]) }], ..Default::default() },
        "not Append"
    )]
    #[case::dependent_true_on_update_config(
        update_config(),
        CellFlagChanges { updates: vec![CellFlagUpdate { flag_id: 1, value: true, rows: rows(&[(0, None)]) }], ..Default::default() },
        "only a DataReplacement that writes 'summary'"
    )]
    fn change_gate(
        #[case] operation: Operation,
        #[case] changes: CellFlagChanges,
        #[case] expected: &str,
    ) {
        let head = flagged_manifest();
        let txn = TransactionBuilder::new(3, operation)
            .cell_flag_changes(changes)
            .build();
        let error = ensure_operation_allowed_with_cell_flags(&head, &txn).unwrap_err();
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(error.to_string().contains(expected), "{error}");
    }

    #[test]
    fn ordinary_flag_updates_through_update_config() {
        let head = flagged_manifest();
        let txn = TransactionBuilder::new(3, update_config())
            .cell_flag_changes(CellFlagChanges {
                updates: vec![CellFlagUpdate {
                    flag_id: 2,
                    value: false,
                    rows: rows(&[(1, Some(&[3]))]),
                }],
                ..Default::default()
            })
            .build();
        let next = commit(&head, &txn).unwrap();
        let expected: Vec<u32> = (0..ROWS as u32).filter(|offset| *offset != 3).collect();
        assert_eq!(
            registry_of(&next).states().get(&2).unwrap().as_ref(),
            &rows(&[(0, None), (1, Some(&expected))])
        );
    }

    #[test]
    fn creating_a_dataset_with_cell_flag_changes_is_refused() {
        let schema = schema();
        let txn = TransactionBuilder::new(
            0,
            Operation::Overwrite {
                fragments: vec![],
                schema,
                config_upsert_values: None,
                initial_bases: None,
            },
        )
        .cell_flag_changes(CellFlagChanges {
            drops: vec![1],
            ..Default::default()
        })
        .build();
        let error = txn
            .build_manifest(None, vec![], "txn", &default_build_config())
            .unwrap_err();
        assert!(matches!(error, Error::NotSupported { .. }), "{error}");
        assert!(
            error.to_string().contains("while creating a dataset"),
            "{error}"
        );
    }
}
