// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Cell flags: a named Boolean per cell of one top-level field.
//!
//! **Unstable prototype.** Datasets with cell flags set
//! [`lance_table::feature_flags::FLAG_UNSTABLE_CELL_FLAGS`], which release
//! builds refuse unless `LANCE_ENABLE_UNSTABLE_CELL_FLAGS` is set. The format
//! and these APIs may change without migration.
//!
//! A flag registered with `clear_on_write` sources is *dependent*: any write to
//! a source, or to the flag's own field, clears the flag for the written rows
//! in the same commit. When a source is itself the output of a dependent flag,
//! clearing that flag clears this one on the same rows, so clears follow
//! chains of computed fields. Registering or dropping a masking flag on a
//! source changes which of its cells read as NULL, so it clears the flag on
//! every row. A dependent flag becomes true only through a `DataReplacement`
//! that writes its field and carries a
//! [`CellFlagUpdate`](crate::dataset::transaction::CellFlagUpdate) setting it,
//! committed through [`CommitBuilder`] with the version the values were
//! computed from as its read version; the flag must already be registered
//! there. In such a publication, every row of a replaced fragment that the
//! transaction does not assign must be copied unchanged from the read
//! snapshot, as read through Lance. Lance treats those rows as logically
//! unchanged for invalidation, and as physically written for conflict
//! detection, so an incremental refresh keeps the rows an earlier one
//! completed. A flag published together with a dependent flag upstream of it
//! must, on the rows both assign, be computed from the upstream values the
//! same transaction publishes. [`PublicationReport`] describes how concurrent
//! transactions are checked. A flag without sources is *ordinary*: only explicit updates
//! change it, and it cannot mask its field.
//!
//! A row-moving update gives the rows it writes new addresses, so their state
//! moves with them only for the rows it lists in `moved_rows`.
//! [`UpdateBuilder`](crate::dataset::UpdateBuilder) lists every row it moves
//! and the fields it sets: ordinary flags move, and so does each dependent flag
//! unless the update sets a field it watches, directly or through a dependent
//! flag upstream of it. Other row-moving writes, such as a `merge_insert` that
//! rewrites whole rows, are refused where an ordinary flag is true, and leave
//! the moved rows unassigned for dependent flags.
//!
//! A refresh from registration to publication:
//!
//! ```
//! # use std::sync::Arc;
//! # use arrow_array::RecordBatch;
//! # use futures::stream;
//! # use lance::{Dataset, Result};
//! # use lance::dataset::cell_flag::{CellFlagOptions, DeferralReason};
//! # use lance::dataset::transaction::{
//! #     CellFlagChanges, CellFlagUpdate, Operation, TransactionBuilder,
//! # };
//! # use lance::dataset::{CommitBuilder, DependencyConflictPolicy, UpdateBuilder};
//! # use lance_select::RowAddrTreeMap;
//! # async fn example(mut dataset: Dataset, summaries: RecordBatch) -> Result<()> {
//! // `summary` is computed from `title` and `body` and reads NULL until published.
//! let ready = dataset
//!     .register_cell_flag(
//!         "summary",
//!         "ready",
//!         CellFlagOptions::default()
//!             .with_clear_on_write(["title", "body"])
//!             .with_mask_when_false(true),
//!     )
//!     .await?;
//!
//! // A refresh reads a snapshot, computes one value per physical row of fragment 0
//! // outside Lance, and stages them as a full-fragment file.
//! let read = dataset.clone();
//! let output = read.schema().project(&["summary"])?;
//! let fragment = read.get_fragment(0).expect("fragment 0 exists");
//! let group = fragment
//!     .write_columns(stream::iter([Ok(summaries)]), &output)
//!     .await?;
//! let mut computed = RowAddrTreeMap::new();
//! computed.insert_fragment(0);
//! let publication = TransactionBuilder::new(
//!     read.version().version,
//!     Operation::DataReplacement { replacements: vec![group] },
//! )
//! .cell_flag_changes(CellFlagChanges {
//!     updates: vec![CellFlagUpdate { flag_id: ready.flag_id, value: true, rows: computed }],
//!     ..Default::default()
//! })
//! .build();
//!
//! // Publish what is still valid and learn what to redo.
//! let result = CommitBuilder::new(Arc::new(read))
//!     .with_dependency_conflict_policy(DependencyConflictPolicy::Skip)
//!     .execute_with_report(publication)
//!     .await?;
//! let recompute = result.report.deferred_rows_of(ready.flag_id, DeferralReason::InputChanged);
//! let reuse = result.report.reusable_rows(ready.flag_id);
//! # let _ = (recompute, reuse);
//!
//! // An ordinary write to a source clears the flag in its own commit.
//! UpdateBuilder::new(Arc::new(result.dataset))
//!     .update_where("id = 7")?
//!     .set("body", "'new body'")?
//!     .build()?
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use lance_select::RowAddrTreeMap;
use lance_table::format::{CellFlagDefinition, Fragment};
use roaring::{RoaringBitmap, RoaringTreemap};

use crate::dataset::transaction::{
    CellFlagChanges, CellFlagMovedRows, CellFlagRegistration, Operation, TransactionBuilder,
};
use crate::dataset::write::CommitBuilder;
use crate::{Dataset, Error, Result};

mod publication;

pub use publication::{
    DeferralReason, DeferredGroup, DeferredRows, DependencyConflictPolicy, PublicationReport,
    PublicationResult,
};
pub(crate) use publication::{PublicationDeferrals, input_changed_rows, removed_fragments};

/// How a cell flag reacts to writes and reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CellFlagOptions {
    /// Top-level source fields, by name. A write to any of them, or to the
    /// flag's own field, clears the flag for the written rows. Empty makes an
    /// ordinary flag.
    pub clear_on_write: Vec<String>,
    /// Reads return NULL for the flag's field wherever the flag is false. Needs
    /// `clear_on_write` sources: writes that re-read rows through the masked
    /// scan, such as a row-moving update, write masked cells back as NULL,
    /// which only a dependent flag tolerates, since a publication sets it true
    /// together with fresh values. The field must be a nullable scalar and must
    /// not be indexed. MemWAL rows carry no flag state to mask them with, so a
    /// masking flag cannot be registered while MemWAL is initialized, MemWAL
    /// cannot be initialized or written while one is registered, and LSM reads
    /// over a version that has one fail.
    pub mask_when_false: bool,
}

impl CellFlagOptions {
    pub fn with_clear_on_write(
        mut self,
        fields: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.clear_on_write = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_mask_when_false(mut self, mask_when_false: bool) -> Self {
        self.mask_when_false = mask_when_false;
        self
    }
}

impl Dataset {
    /// Register a cell flag named `name` on the top-level field `field`.
    ///
    /// The flag starts false for every row. Field names are resolved to stable
    /// field ids now, so later renames keep the flag. The flag id is assigned
    /// at commit and never reused; staged transactions refer to flags by id.
    /// Returns the committed definition.
    ///
    /// ```
    /// # use lance::{Dataset, Result};
    /// # use lance::dataset::cell_flag::CellFlagOptions;
    /// # async fn example(dataset: &mut Dataset) -> Result<()> {
    /// let ready = dataset
    ///     .register_cell_flag(
    ///         "summary",
    ///         "ready",
    ///         CellFlagOptions::default()
    ///             .with_clear_on_write(["title", "body"])
    ///             .with_mask_when_false(true),
    ///     )
    ///     .await?;
    /// assert!(ready.is_dependent());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_cell_flag(
        &mut self,
        field: &str,
        name: &str,
        options: CellFlagOptions,
    ) -> Result<CellFlagDefinition> {
        let registration = self.cell_flag_registration(field, name, options)?;
        let field_id = registration.field_id;
        self.commit_cell_flag_changes(CellFlagChanges {
            registrations: vec![registration],
            ..Default::default()
        })
        .await?;
        self.committed_cell_flag(field_id, name)
    }

    /// Atomically drop the flag `name` on `field` and register a new one with
    /// the same name and `options`.
    ///
    /// The new flag has a new id and starts false everywhere, so transactions
    /// staged against the old id fail to commit instead of publishing values
    /// computed under the old registration.
    ///
    /// ```
    /// # use lance::{Dataset, Result};
    /// # use lance::dataset::cell_flag::CellFlagOptions;
    /// # async fn example(dataset: &mut Dataset) -> Result<()> {
    /// let old_id = dataset.cell_flag("summary", "ready").map(|flag| flag.flag_id);
    /// let ready = dataset
    ///     .replace_cell_flag(
    ///         "summary",
    ///         "ready",
    ///         CellFlagOptions::default().with_clear_on_write(["body"]),
    ///     )
    ///     .await?;
    /// assert_ne!(Some(ready.flag_id), old_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn replace_cell_flag(
        &mut self,
        field: &str,
        name: &str,
        options: CellFlagOptions,
    ) -> Result<CellFlagDefinition> {
        let registration = self.cell_flag_registration(field, name, options)?;
        let field_id = registration.field_id;
        let existing = self.registered_cell_flag(field, name)?;
        self.commit_cell_flag_changes(CellFlagChanges {
            drops: vec![existing],
            registrations: vec![registration],
            ..Default::default()
        })
        .await?;
        self.committed_cell_flag(field_id, name)
    }

    /// Drop the flag `name` on `field` and its state.
    ///
    /// ```
    /// # use lance::{Dataset, Result};
    /// # async fn example(dataset: &mut Dataset) -> Result<()> {
    /// dataset.drop_cell_flag("summary", "ready").await?;
    /// assert!(dataset.cell_flag("summary", "ready").is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn drop_cell_flag(&mut self, field: &str, name: &str) -> Result<()> {
        let existing = self.registered_cell_flag(field, name)?;
        self.commit_cell_flag_changes(CellFlagChanges {
            drops: vec![existing],
            ..Default::default()
        })
        .await
    }

    /// Every cell flag registered at this version, sorted by flag id.
    ///
    /// ```
    /// # use lance::Dataset;
    /// # fn example(dataset: &Dataset) -> Vec<&str> {
    /// dataset
    ///     .cell_flags()
    ///     .iter()
    ///     .map(|flag| flag.name.as_str())
    ///     .collect()
    /// # }
    /// ```
    pub fn cell_flags(&self) -> &[CellFlagDefinition] {
        self.manifest
            .cell_flags
            .as_deref()
            .map(|registry| registry.definitions())
            .unwrap_or_default()
    }

    /// The flag `name` on the field `field`, if registered at this version.
    ///
    /// ```
    /// # use lance::Dataset;
    /// # fn example(dataset: &Dataset) -> Option<u32> {
    /// dataset.cell_flag("summary", "ready").map(|flag| flag.flag_id)
    /// # }
    /// ```
    pub fn cell_flag(&self, field: &str, name: &str) -> Option<&CellFlagDefinition> {
        let field_id = self.schema().field(field)?.id;
        self.manifest.cell_flags.as_deref()?.find(field_id, name)
    }

    /// The rows where flag `flag_id` is true at this version, keyed by fragment
    /// id with physical row offsets. A `Full` fragment means every physical row
    /// of it, deleted rows included.
    ///
    /// ```
    /// # use lance::{Dataset, Result};
    /// # use lance_select::RowAddrSelection;
    /// # fn example(dataset: &Dataset, flag_id: u32) -> Result<bool> {
    /// let true_rows = dataset.cell_flag_true_rows(flag_id)?;
    /// let first_fragment_all_true = matches!(true_rows.get(&0), Some(RowAddrSelection::Full));
    /// # Ok(first_fragment_all_true)
    /// # }
    /// ```
    pub fn cell_flag_true_rows(&self, flag_id: u32) -> Result<RowAddrTreeMap> {
        let registry = self
            .manifest
            .cell_flags
            .as_deref()
            .filter(|registry| registry.definition(flag_id).is_some())
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "cell flag {flag_id} is not registered at version {}",
                    self.manifest.version
                ))
            })?;
        Ok(registry
            .states()
            .get(&flag_id)
            .map(|state| state.as_ref().clone())
            .unwrap_or_default())
    }

    fn cell_flag_field_id(&self, field: &str) -> Result<i32> {
        self.schema()
            .field(field)
            .map(|field| field.id)
            .ok_or_else(|| {
                Error::field_not_found(
                    field,
                    self.schema()
                        .fields
                        .iter()
                        .map(|field| field.name.clone())
                        .collect(),
                )
            })
    }

    fn cell_flag_registration(
        &self,
        field: &str,
        name: &str,
        options: CellFlagOptions,
    ) -> Result<CellFlagRegistration> {
        Ok(CellFlagRegistration {
            field_id: self.cell_flag_field_id(field)?,
            name: name.to_string(),
            clear_on_write: options
                .clear_on_write
                .iter()
                .map(|source| self.cell_flag_field_id(source))
                .collect::<Result<_>>()?,
            mask_when_false: options.mask_when_false,
        })
    }

    fn registered_cell_flag(&self, field: &str, name: &str) -> Result<u32> {
        let field_id = self.cell_flag_field_id(field)?;
        self.manifest
            .cell_flags
            .as_deref()
            .and_then(|registry| registry.find(field_id, name))
            .map(|definition| definition.flag_id)
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "field '{field}' has no cell flag named '{name}' at version {}",
                    self.manifest.version
                ))
            })
    }

    fn committed_cell_flag(&self, field_id: i32, name: &str) -> Result<CellFlagDefinition> {
        self.manifest
            .cell_flags
            .as_deref()
            .and_then(|registry| registry.find(field_id, name))
            .cloned()
            .ok_or_else(|| {
                Error::internal(format!(
                    "cell flag '{name}' on field id {field_id} is missing from version {} \
                     that registered it",
                    self.manifest.version
                ))
            })
    }

    async fn commit_cell_flag_changes(&mut self, changes: CellFlagChanges) -> Result<()> {
        let operation = Operation::UpdateConfig {
            config_updates: None,
            table_metadata_updates: None,
            schema_metadata_updates: None,
            field_metadata_updates: HashMap::new(),
        };
        let transaction = TransactionBuilder::new(self.manifest.version, operation)
            .cell_flag_changes(changes)
            .build();
        *self = CommitBuilder::new(Arc::new(self.clone()))
            .execute(transaction)
            .await?;
        Ok(())
    }
}

/// Pair every row a row-moving update wrote into `new_fragments` with the
/// address it was read from, for the commit to move its flag state.
///
/// `source_row_addrs` yields the `source_row_count` read addresses in write
/// order: the new fragments in order, each from its first offset, hold exactly
/// the rows the update read.
pub(crate) fn moved_cell_flag_rows(
    new_fragments: &[Fragment],
    source_row_count: u64,
    mut source_row_addrs: impl Iterator<Item = Result<u64>>,
) -> Result<Vec<CellFlagMovedRows>> {
    // Ids are assigned at commit, so fragments are named by position here.
    let mut row_counts = Vec::with_capacity(new_fragments.len());
    for (position, fragment) in new_fragments.iter().enumerate() {
        let physical_rows = fragment.physical_rows.ok_or_else(|| {
            Error::internal(format!(
                "new fragment {position} written by the update has no physical row count"
            ))
        })?;
        let physical_rows = u32::try_from(physical_rows).map_err(|_| {
            Error::internal(format!(
                "new fragment {position} written by the update has {physical_rows} physical \
                 rows, more than a row address holds"
            ))
        })?;
        row_counts.push(physical_rows);
    }
    let written_rows: u64 = row_counts.iter().copied().map(u64::from).sum();
    if written_rows != source_row_count {
        return Err(Error::internal(format!(
            "the update wrote {written_rows} rows into {} new fragments but read \
             {source_row_count} rows, so the moved rows cannot be matched to the addresses they \
             were read from",
            new_fragments.len(),
        )));
    }
    let mut moved_rows = Vec::with_capacity(new_fragments.len());
    let mut paired_rows = 0_u64;
    for (position, (fragment, physical_rows)) in new_fragments.iter().zip(row_counts).enumerate() {
        let first_file = fragment.files.first().ok_or_else(|| {
            Error::internal(format!(
                "new fragment {position} written by the update has no data file"
            ))
        })?;
        let empty_entry = || CellFlagMovedRows {
            fragment_path: first_file.path.clone(),
            offsets: RoaringBitmap::new(),
            source_row_addrs: RoaringTreemap::new(),
        };
        let mut entry = empty_entry();
        let mut previous_source = None;
        for offset in 0..physical_rows {
            let Some(source) = source_row_addrs.next() else {
                return Err(Error::internal(format!(
                    "the update read {source_row_count} rows, but only {paired_rows} source \
                     addresses could be listed"
                )));
            };
            let source = source?;
            paired_rows += 1;
            // Offsets and sources pair by rank, so a source below the one
            // before it starts another entry.
            if previous_source.is_some_and(|previous| source <= previous) {
                moved_rows.push(std::mem::replace(&mut entry, empty_entry()));
            }
            previous_source = Some(source);
            entry
                .offsets
                .try_push(offset)
                .and_then(|()| entry.source_row_addrs.try_push(source))
                .map_err(|_| {
                    Error::internal(format!(
                        "offset {offset} or source address {source} of new fragment {position} \
                         is not above the ones listed before it"
                    ))
                })?;
        }
        if !entry.offsets.is_empty() {
            moved_rows.push(entry);
        }
    }
    Ok(moved_rows)
}

#[cfg(test)]
mod tests {
    use lance_core::utils::address::RowAddress;
    use lance_file::version::ConcreteFileVersion;
    use rstest::rstest;

    use super::*;

    fn addr(fragment_id: u32, offset: u32) -> u64 {
        RowAddress::new_from_parts(fragment_id, offset).into()
    }

    fn new_fragment(path: &str, physical_rows: usize) -> Fragment {
        Fragment::new(0)
            .with_file(path, vec![0], vec![0], ConcreteFileVersion::V2_0, None)
            .with_physical_rows(physical_rows)
    }

    /// `(fragment_path, offsets, sources)` of each entry.
    fn entries(moved_rows: &[CellFlagMovedRows]) -> Vec<(&str, Vec<u32>, Vec<u64>)> {
        moved_rows
            .iter()
            .map(|moved| {
                (
                    moved.fragment_path.as_str(),
                    moved.offsets.iter().collect(),
                    moved.source_row_addrs.iter().collect(),
                )
            })
            .collect()
    }

    #[rstest]
    #[case::split_across_fragments(
        &[2, 3],
        vec![addr(0, 1), addr(0, 4), addr(1, 0), addr(1, 2), addr(2, 7)],
        vec![
            ("a.lance", vec![0, 1], vec![addr(0, 1), addr(0, 4)]),
            ("b.lance", vec![0, 1, 2], vec![addr(1, 0), addr(1, 2), addr(2, 7)]),
        ]
    )]
    #[case::descending_source_starts_an_entry(
        &[3, 1],
        vec![addr(3, 0), addr(1, 5), addr(1, 6), addr(0, 2)],
        vec![
            ("a.lance", vec![0], vec![addr(3, 0)]),
            ("a.lance", vec![1, 2], vec![addr(1, 5), addr(1, 6)]),
            ("b.lance", vec![0], vec![addr(0, 2)]),
        ]
    )]
    fn moved_rows_pair_offsets_with_sources_in_write_order(
        #[case] physical_rows: &[usize],
        #[case] sources: Vec<u64>,
        #[case] expected: Vec<(&str, Vec<u32>, Vec<u64>)>,
    ) {
        let fragments: Vec<Fragment> = ["a.lance", "b.lance"]
            .into_iter()
            .zip(physical_rows)
            .map(|(path, rows)| new_fragment(path, *rows))
            .collect();
        let moved_rows = moved_cell_flag_rows(
            &fragments,
            sources.len() as u64,
            sources.into_iter().map(Ok),
        )
        .unwrap();
        assert_eq!(entries(&moved_rows), expected);
    }

    #[rstest]
    #[case::fewer_rows_read(
        vec![new_fragment("a.lance", 2), new_fragment("b.lance", 3)],
        4,
        "the update wrote 5 rows into 2 new fragments but read 4 rows"
    )]
    #[case::fewer_sources_listed(
        vec![new_fragment("a.lance", 2), new_fragment("b.lance", 3)],
        5,
        "the update read 5 rows, but only 4 source addresses could be listed"
    )]
    #[case::unknown_row_count(
        vec![new_fragment("a.lance", 2), Fragment::new(0).with_file("b.lance", vec![0], vec![0], ConcreteFileVersion::V2_0, None)],
        4,
        "new fragment 1 written by the update has no physical row count"
    )]
    #[case::no_data_file(
        vec![new_fragment("a.lance", 2), Fragment::new(0).with_physical_rows(2)],
        4,
        "new fragment 1 written by the update has no data file"
    )]
    fn moved_rows_that_cannot_be_paired_are_an_error(
        #[case] fragments: Vec<Fragment>,
        #[case] source_row_count: u64,
        #[case] expected: &str,
    ) {
        let sources = (0..4).map(|offset| Ok(addr(0, offset)));
        let error = moved_cell_flag_rows(&fragments, source_row_count, sources).unwrap_err();
        assert!(matches!(error, Error::Internal { .. }), "{error}");
        assert!(error.to_string().contains(expected), "{error}");
    }
}
