// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Staging publications of dependent cell flag outputs computed at one
//! snapshot, and planning their follow-ups.

mod merge;

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_array::{Array, ArrayRef, RecordBatch};
use arrow_buffer::BooleanBuffer;
use arrow_schema::{Field as ArrowField, FieldRef, Schema as ArrowSchema, SchemaRef};
use futures::{Stream, StreamExt, TryStreamExt, stream};
use lance_arrow::RecordBatchExt;
use lance_arrow::json::has_json_fields;
use lance_core::datatypes::{Field, NullabilityComparison, Schema, SchemaCompareOptions};
use lance_core::utils::address::RowAddress;
use lance_core::utils::deletion::DeletionVector;
use lance_core::{ROW_ADDR, ROW_ID};
use lance_file::version::ConcreteFileVersion;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::format::{CellFlagDefinition, CellFlagRegistry, Fragment};
use roaring::RoaringBitmap;

use self::merge::{Assignment, FragmentMerge, Run};
use super::{DeferralReason, PublicationReport};
use crate::dataset::fragment::{
    FileFragment, discard_staged_file, duplicate_field_path, relax_nullability,
};
use crate::dataset::scanner::{BATCH_SIZE_FALLBACK, get_default_batch_size};
use crate::dataset::transaction::{
    CellFlagChanges, CellFlagUpdate, DataReplacementGroup, Operation, Transaction,
    TransactionBuilder,
};
use crate::{Dataset, Error, Result};

/// Decoded copy windows the stager takes from the copy-through scan at once,
/// the one being merged included. The scan reads ahead on its own.
const COPY_READ_AHEAD: usize = 2;

/// Stages a publication of dependent cell flag outputs whose values were
/// computed at one snapshot.
///
/// [`Self::stage`] merges streamed computed rows with a copy-through read of
/// the snapshot into one full-fragment file per fragment on which a row is
/// assigned, and returns the `DataReplacement` that publishes them, read at
/// the snapshot's version. Commit it through
/// [`CommitBuilder`](crate::dataset::CommitBuilder) on that same snapshot.
/// Every file writes all outputs, in schema order, so the outputs of one
/// stager publish together: a concurrent publication or write of any of them
/// on a staged fragment conflicts with the whole group. Declare the smallest
/// set that must publish atomically.
///
/// Every cell a computed batch does not assign is copied from the snapshot as
/// read through Lance: overlays merged, NULL where a masking flag is false,
/// and deleted positions kept. Lance checks row addresses, types and layout,
/// but not that values were computed from the declared inputs, nor that a
/// downstream output assigned in the same row as its upstream was computed
/// from the upstream value assigned there.
///
/// [`Self::follow_up`] turns the [`PublicationReport`] of a commit into the
/// rows a follow-up stages at the version the report names.
///
/// ```
/// # use std::sync::Arc;
/// # use arrow_array::RecordBatch;
/// # use arrow_buffer::BooleanBuffer;
/// # use futures::stream;
/// # use lance::{Dataset, Result};
/// # use lance::dataset::cell_flag::{ComputedBatch, PublicationStager};
/// # use lance::dataset::{CommitBuilder, DependencyConflictPolicy};
/// # async fn example(snapshot: Arc<Dataset>, rows: RecordBatch, pending: BooleanBuffer) -> Result<()> {
/// // `rows` holds `_rowaddr` and freshly computed `summary` values for rows of
/// // `snapshot`, scanned in order; `pending` marks the ones to publish.
/// let stager = PublicationStager::try_new(snapshot.clone(), &["summary"])?;
/// let computed = ComputedBatch::new(rows).with_assigned("summary", pending);
/// let Some(publication) = stager.stage(stream::iter([Ok(computed)])).await? else {
///     return Ok(()); // no row assigned
/// };
/// let result = CommitBuilder::new(snapshot)
///     .with_dependency_conflict_policy(DependencyConflictPolicy::Skip)
///     .execute_with_report(publication)
///     .await?;
///
/// // Plan the rows the report certifies or defers, at the version it names.
/// // Rows it vacated are not planned: a moved row is pending at its new
/// // address, for a scan of pending rows to pick up.
/// let follow_up = PublicationStager::try_new(Arc::new(result.dataset), &["summary"])?;
/// let plan = follow_up.follow_up(&result.report).await?;
/// let rows = plan.rows("summary").expect("summary is staged");
/// # let _ = (&rows.reuse, &rows.recompute);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PublicationStager {
    snapshot: Arc<Dataset>,
    /// In schema order.
    outputs: Arc<[Output]>,
    /// Indices into `outputs`, each after the declared outputs it watches.
    upstream_first: Arc<[usize]>,
    /// The output fields as the snapshot's manifest defines them.
    output_schema: Arc<Schema>,
    relaxed_schema: SchemaRef,
    /// Every field id of `output_schema`, as a staged file lists them.
    output_field_ids: Arc<HashSet<i32>>,
    copy_batch_size: u32,
}

#[derive(Debug)]
struct Output {
    field: Field,
    /// A one-field schema of `field` with nullability relaxed.
    relaxed: SchemaRef,
    flag_id: u32,
    /// Indices into the stager's outputs of those this output's flag watches.
    upstreams: Vec<usize>,
}

/// Computed rows: a non-null UInt64 `_rowaddr` column of physical row
/// addresses at the stager's snapshot, as a scan with
/// [`Scanner::with_row_address`](crate::dataset::scanner::Scanner::with_row_address)
/// returns them, and one column per output the batch carries, matched by name.
///
/// An output column assigns every row of the batch, or only the rows its
/// [`Self::with_assigned`] mask sets. A NULL on an assigned row is a computed
/// NULL: it is published and its flag set true. The value on an unassigned row
/// is ignored, and that cell is copied from the snapshot, as is every output
/// the batch does not carry.
///
/// Across the stream, the rows of each fragment arrive in one contiguous run
/// (fragments in any order) with strictly ascending offsets, so a row appears
/// at most once and carries every output it assigns. A scan of the snapshot
/// in order satisfies this.
///
/// ```
/// # use arrow_array::RecordBatch;
/// # use arrow_buffer::BooleanBuffer;
/// # use lance::dataset::cell_flag::ComputedBatch;
/// # fn example(rows: RecordBatch) -> ComputedBatch {
/// // Assign `summary` on the first row only, and `translation` on every row.
/// let first_only = BooleanBuffer::collect_bool(rows.num_rows(), |row| row == 0);
/// ComputedBatch::new(rows).with_assigned("summary", first_only)
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ComputedBatch {
    rows: RecordBatch,
    assigned: Vec<(String, BooleanBuffer)>,
}

impl ComputedBatch {
    /// Rows on which every output column of `rows` is assigned.
    pub fn new(rows: RecordBatch) -> Self {
        Self {
            rows,
            assigned: Vec::new(),
        }
    }

    /// Assign `output` only on the rows whose bit is set, one bit per row.
    pub fn with_assigned(mut self, output: impl Into<String>, rows: BooleanBuffer) -> Self {
        self.assigned.push((output.into(), rows));
        self
    }
}

impl From<RecordBatch> for ComputedBatch {
    fn from(rows: RecordBatch) -> Self {
        Self::new(rows)
    }
}

/// What a follow-up of a [`PublicationReport`] stages, per output of the
/// stager that planned it.
///
/// ```
/// # use lance::dataset::cell_flag::FollowUpPlan;
/// # use lance_select::RowSetOps;
/// # fn example(plan: &FollowUpPlan) -> bool {
/// plan.rows("translation")
///     .is_some_and(|rows| rows.recompute.contains(0))
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FollowUpPlan {
    /// In the stager's output order.
    rows: Vec<(String, FollowUpRows)>,
}

impl FollowUpPlan {
    /// The planned rows of `output`, or `None` when the stager does not stage it.
    pub fn rows(&self, output: &str) -> Option<&FollowUpRows> {
        self.rows
            .iter()
            .find(|(name, _)| name == output)
            .map(|(_, rows)| rows)
    }

    /// Whether no output has a row to reuse or recompute: the report leaves
    /// nothing to reuse or recompute, not that every row is published. See
    /// [`PublicationStager::follow_up`] for the rows a plan leaves out.
    pub fn is_empty(&self) -> bool {
        self.rows
            .iter()
            .all(|(_, rows)| rows.reuse.is_empty() && rows.recompute.is_empty())
    }
}

/// Rows of one output, live at the follow-up's snapshot, keyed by fragment id
/// with physical offsets and never `Full`. `reuse` and `recompute` are
/// disjoint.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FollowUpRows {
    /// Rows whose staged value the report still certifies and whose flag is
    /// false at the snapshot: assign the earlier value again.
    pub reuse: RowAddrTreeMap,
    /// Rows to compute from the snapshot, or from an upstream value assigned
    /// in the same row: those the report defers as
    /// [`DeferralReason::InputChanged`] or
    /// [`DeferralReason::UpstreamNotPublished`] whose flag is false at the
    /// snapshot, and every row on which the plan assigns a declared output
    /// this one watches, since publishing that upstream clears this output
    /// where it is not assigned too.
    pub recompute: RowAddrTreeMap,
}

impl PublicationStager {
    /// A stager of `outputs`: top-level fields of `snapshot`, each the output
    /// of a dependent cell flag registered there. `snapshot`'s version is the
    /// read version of every publication it stages.
    ///
    /// Fails with `InvalidInput` when `outputs` is empty or repeats a name, or
    /// names a reserved column, a field that is not top-level, or one without
    /// a dependent flag, or when one output watches another through the
    /// output of a dependent flag not declared, whose clear the commit would
    /// carry onto the rows the downstream output publishes; and with
    /// `NotSupported` for an output that is or holds a blob or JSON field, or
    /// a dataset in the legacy (v1) storage format.
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use lance::{Dataset, Result};
    /// # use lance::dataset::cell_flag::PublicationStager;
    /// # fn example(snapshot: Arc<Dataset>) -> Result<Option<u32>> {
    /// let stager = PublicationStager::try_new(snapshot, &["summary", "translation"])?;
    /// # Ok(stager.flag_id("summary"))
    /// # }
    /// ```
    pub fn try_new(snapshot: Arc<Dataset>, outputs: &[&str]) -> Result<Self> {
        let version = snapshot.manifest.version;
        if snapshot.manifest.data_storage_format.version == ConcreteFileVersion::V1 {
            return Err(Error::not_supported(format!(
                "publication staging needs a V2 data storage format, but version {version} uses \
                 the legacy (v1) format"
            )));
        }
        if outputs.is_empty() {
            return Err(Error::invalid_input(
                "a publication stages at least one output",
            ));
        }
        let schema = snapshot.schema();
        let registry = snapshot.manifest.cell_flags.as_deref();
        let mut names = HashSet::with_capacity(outputs.len());
        let mut flags = HashMap::with_capacity(outputs.len());
        for name in outputs {
            if !names.insert(*name) {
                return Err(Error::invalid_input(format!(
                    "output '{name}' is named more than once"
                )));
            }
            if lance_core::is_system_column(name) {
                return Err(Error::invalid_input(format!(
                    "output '{name}' is a reserved column"
                )));
            }
            let field = schema
                .fields
                .iter()
                .find(|field| field.name == *name)
                .ok_or_else(|| {
                    Error::invalid_input(format!(
                        "output '{name}' is not a top-level field of version {version}"
                    ))
                })?;
            if has_blob(field) {
                return Err(Error::not_supported(format!(
                    "output '{name}' is a blob column, which publication staging does not support"
                )));
            }
            // Scans return JSON as Arrow JSON, so the copy-through would not
            // read back the stored type.
            if has_json_fields(&ArrowField::from(field)) {
                return Err(Error::not_supported(format!(
                    "output '{name}' is or holds a JSON field, which publication staging does not \
                     support"
                )));
            }
            let flag = registry
                .and_then(|registry| registry.dependent_flag(field.id))
                .ok_or_else(|| {
                    Error::invalid_input(format!(
                        "output '{name}' has no dependent cell flag registered at version \
                         {version}; register one with clear_on_write sources before publishing it"
                    ))
                })?;
            flags.insert(field.id, flag);
        }
        if let Some(registry) = registry {
            check_undeclared_intermediates(schema, registry, &flags)?;
        }
        let ids: Vec<i32> = flags.keys().copied().collect();
        let output_schema = schema.project_by_ids(&ids, true);
        let index_of: HashMap<i32, usize> = output_schema
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.id, index))
            .collect();
        let mut resolved = Vec::with_capacity(output_schema.fields.len());
        for field in &output_schema.fields {
            let flag = flags[&field.id];
            let relaxed = relax_nullability(&ArrowField::from(field));
            resolved.push(Output {
                field: field.clone(),
                relaxed: Arc::new(ArrowSchema::new(vec![relaxed])),
                flag_id: flag.flag_id,
                upstreams: flag
                    .clear_on_write
                    .iter()
                    .filter_map(|source| index_of.get(source).copied())
                    .collect(),
            });
        }
        let upstream_first = upstream_first(&resolved)?;
        let relaxed_schema = Arc::new(ArrowSchema::new(
            resolved
                .iter()
                .map(|output| output.relaxed.field(0).clone())
                .collect::<Vec<_>>(),
        ));
        Ok(Self {
            snapshot,
            outputs: resolved.into(),
            upstream_first: upstream_first.into(),
            output_field_ids: Arc::new(output_schema.field_ids().into_iter().collect()),
            output_schema: Arc::new(output_schema),
            relaxed_schema,
            copy_batch_size: get_default_batch_size()
                .and_then(|rows| u32::try_from(rows).ok())
                .unwrap_or(BATCH_SIZE_FALLBACK as u32),
        })
    }

    /// Rows per copy-through read, and so per staged batch.
    #[cfg(test)]
    pub(crate) fn with_copy_batch_size(mut self, rows: u32) -> Self {
        self.copy_batch_size = rows;
        self
    }

    /// The dependent flag that publishes `output`, or `None` if this stager
    /// does not stage it.
    ///
    /// ```
    /// # use lance::{Dataset, Result};
    /// # use lance::dataset::cell_flag::PublicationStager;
    /// # fn example(stager: &PublicationStager, dataset: &Dataset) -> Result<()> {
    /// if let Some(flag_id) = stager.flag_id("summary") {
    ///     let completed = dataset.cell_flag_true_rows(flag_id)?;
    /// #   let _ = completed;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn flag_id(&self, output: &str) -> Option<u32> {
        self.output_index(output)
            .map(|index| self.outputs[index].flag_id)
    }

    /// Stage one full-fragment file per fragment on which `computed` assigns a
    /// row, copying every other cell of the outputs from the snapshot, and
    /// return the publication: a `DataReplacement` read at the snapshot's
    /// version that sets each output's flag true on exactly the rows assigned.
    /// An output that assigns no row still gets a `CellFlagUpdate`, with no
    /// rows: its cells in the staged files are copies, and the update is what
    /// tells the commit so, keeping its flag and its downstream flags. Returns
    /// `Ok(None)`, and writes nothing, when no row is assigned.
    ///
    /// Fragments are written one at a time, each while `computed` streams its
    /// rows, and `computed` is pulled only as far as the copy window being
    /// written needs. Whatever the fragment size, the stager holds only the
    /// caller batches overlapping that window, two decoded copy windows and
    /// one staged batch, besides the writer's buffers and the per-output
    /// assignment bitmaps. The copy-through scan's own decode and IO
    /// read-ahead, at the scanner's default
    /// [`batch_readahead`](crate::dataset::scanner::Scanner::batch_readahead)
    /// and [`io_buffer_size`](crate::dataset::scanner::Scanner::io_buffer_size),
    /// is outside that bound.
    ///
    /// Fails with `InvalidInput`, before anything commits, when a batch breaks
    /// the [`ComputedBatch`] contract, carries values of the wrong type, or
    /// assigns a row deleted at the snapshot, a row past its fragment's
    /// physical rows, or NULL to a non-nullable output, or assigns on one row
    /// an output and one it depends on but not a declared output between them,
    /// whose clear the commit would carry on to the downstream output; and when some outputs
    /// are stored in a fragment's data files and others are not, which one
    /// replacement file cannot express. Every file staged by the call is
    /// deleted on any error, including one from `computed`.
    ///
    /// Nothing else deletes them. The files of a returned transaction that is
    /// dropped or fails to commit (as under
    /// [`DependencyConflictPolicy::Reject`](super::DependencyConflictPolicy::Reject)
    /// on a retryable conflict), the files of groups the commit defers, and
    /// the files a dropped `stage` future already wrote stay in the data
    /// directory until [`Dataset::cleanup_old_versions`] removes them as
    /// unreferenced: only once they are 7 days old, unless `delete_unverified`
    /// is set, which is safe only while no other write is in progress.
    ///
    /// An output added as an all-NULL (metadata-only) column is not stored on
    /// the fragments where another output was already published. Publishing
    /// it there takes an extra, separate commit of it alone first; only then
    /// can the two refresh together, so on such fragments their first
    /// publication is not atomic.
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use arrow_array::RecordBatch;
    /// # use futures::StreamExt;
    /// # use lance::{Dataset, Result};
    /// # use lance::dataset::cell_flag::PublicationStager;
    /// # async fn example(
    /// #     snapshot: Arc<Dataset>,
    /// #     summarize: impl Fn(&RecordBatch) -> Result<RecordBatch> + Send,
    /// # ) -> Result<()> {
    /// let stager = PublicationStager::try_new(snapshot.clone(), &["summary"])?;
    /// // An ordered scan of the inputs with row addresses; `summarize`, the
    /// // caller's computation, returns `_rowaddr` and `summary` for each batch.
    /// let mut scan = snapshot.scan();
    /// scan.project(&["title", "body"])?.with_row_address();
    /// let computed = scan
    ///     .try_into_stream()
    ///     .await?
    ///     .map(move |scanned| scanned.and_then(|scanned| summarize(&scanned)));
    /// let publication = stager.stage(computed).await?;
    /// # let _ = publication;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stage<S, B>(&self, computed: S) -> Result<Option<Transaction>>
    where
        S: Stream<Item = Result<B>> + Send,
        B: Into<ComputedBatch> + Send,
    {
        let computed = std::pin::pin!(computed);
        let mut rows = ComputedRows {
            stager: self,
            stream: computed,
            is_exhausted: false,
            queued: VecDeque::new(),
            finished: RoaringBitmap::new(),
            fragment: None,
        };
        let mut groups = Vec::new();
        let mut assigned = vec![RowAddrTreeMap::new(); self.outputs.len()];
        let staged = self
            .stage_fragments(&mut rows, &mut groups, &mut assigned)
            .await
            .and_then(|()| self.publication(&mut groups, assigned));
        match staged {
            Ok(publication) => Ok(publication),
            Err(error) => {
                for DataReplacementGroup(_, file) in &groups {
                    let path = self.snapshot.data_dir().join(file.path.as_str());
                    discard_staged_file(&self.snapshot, &path).await;
                }
                Err(error)
            }
        }
    }

    /// The rows a follow-up of `report` reuses or recomputes, per output.
    /// This stager must read the version `report` names:
    /// `committed_version`, or `checked_version` when nothing was committed.
    ///
    /// Every set holds only rows live at the snapshot, so rows deleted since
    /// the report was made are left out. Rows whose flag is already true
    /// there, published or set by a newer result, are left to copy through,
    /// except where the plan assigns a declared output they watch. Stage the
    /// plan through [`Self::stage`] with the earlier values for `reuse` and
    /// fresh ones for `recompute`; the staged files the report lists are not
    /// read back.
    ///
    /// The plan covers only rows the report certifies reusable or defers as
    /// [`DeferralReason::InputChanged`] or
    /// [`DeferralReason::UpstreamNotPublished`]. Rows vacated by a delete or a
    /// row-moving update, and fragments removed or rewritten, are not
    /// planned. A moved row is pending at its new address, where a refresh of
    /// pending rows, a live scan without the rows of
    /// [`Dataset::cell_flag_true_rows`], picks it up.
    ///
    /// Fails with `InvalidInput` when this stager reads another version, or
    /// when the report certifies rows to reuse for a flag whose output it
    /// does not stage. The rows it only defers for such a flag are left to a
    /// refresh of that output's pending rows. So where the outputs cannot
    /// stage together, as on a fragment that stores only some of them, a
    /// stager of the stored ones can plan the report.
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use lance::{Result};
    /// # use lance::dataset::cell_flag::{PublicationResult, PublicationStager};
    /// # async fn example(result: PublicationResult) -> Result<bool> {
    /// let stager = PublicationStager::try_new(Arc::new(result.dataset), &["summary"])?;
    /// let plan = stager.follow_up(&result.report).await?;
    /// # Ok(plan.is_empty())
    /// # }
    /// ```
    pub async fn follow_up(&self, report: &PublicationReport) -> Result<FollowUpPlan> {
        let version = self.snapshot.manifest.version;
        let required = report.committed_version.unwrap_or(report.checked_version);
        if version != required {
            return Err(Error::invalid_input(format!(
                "a follow-up of the publication that read version {} must read version \
                 {required}, where it committed (or was last checked when nothing committed), \
                 but this stager reads version {version}",
                report.read_version
            )));
        }
        self.check_report_flags(report)?;
        let mut live = LiveRows {
            snapshot: &self.snapshot,
            fragments: HashMap::new(),
        };
        let mut planned = vec![FollowUpRows::default(); self.outputs.len()];
        for &index in self.upstream_first.iter() {
            let output = &self.outputs[index];
            let true_rows = self.snapshot.cell_flag_true_rows(output.flag_id)?;
            let mut valid = RowAddrTreeMap::new();
            for update in report
                .deferred_groups
                .iter()
                .flat_map(|group| &group.valid_rows)
                .filter(|update| update.flag_id == output.flag_id)
            {
                valid |= &update.rows;
            }
            let redo = report.deferred_rows_of(output.flag_id, DeferralReason::InputChanged)
                | report.deferred_rows_of(output.flag_id, DeferralReason::UpstreamNotPublished);
            let mut reuse = live.of(&valid).await? - &true_rows;
            let mut recompute = live.of(&redo).await? - &true_rows;
            for &upstream in &output.upstreams {
                recompute |= &planned[upstream].reuse;
                recompute |= &planned[upstream].recompute;
            }
            reuse -= &recompute;
            planned[index] = FollowUpRows { reuse, recompute };
        }
        Ok(FollowUpPlan {
            rows: self
                .outputs
                .iter()
                .map(|output| output.field.name.clone())
                .zip(planned)
                .collect(),
        })
    }

    fn output_index(&self, name: &str) -> Option<usize> {
        self.outputs
            .iter()
            .position(|output| output.field.name == name)
    }

    async fn stage_fragments<S, B>(
        &self,
        rows: &mut ComputedRows<'_, S>,
        groups: &mut Vec<DataReplacementGroup>,
        assigned: &mut [RowAddrTreeMap],
    ) -> Result<()>
    where
        S: Stream<Item = Result<B>> + Send,
        B: Into<ComputedBatch> + Send,
    {
        while let Some(first_addr) = rows.next_addr().await? {
            let (fragment, physical_rows) = rows.open(first_addr).await?;
            let fragment_id = RowAddress::from(first_addr).fragment_id();
            // No file is opened for a fragment none of whose rows assigns.
            let mut first = None;
            while let Some(run) = rows.next_run().await? {
                if run.assigns_any() {
                    first = Some(run);
                    break;
                }
            }
            let Some(first) = first else {
                rows.close();
                continue;
            };
            self.check_layout(fragment.metadata())?;
            let mut merge = FragmentMerge::new(
                fragment_id,
                physical_rows,
                self.output_schema.clone(),
                self.relaxed_schema.clone(),
            );
            merge.push(first);
            let copy = fragment
                .read_physical_slice_with_row_addr(
                    0..u64::from(physical_rows),
                    &self.output_schema,
                    self.copy_batch_size,
                )
                .await?
                .buffered(COPY_READ_AHEAD);
            let group = fragment
                .write_columns(merged_windows(rows, &mut merge, copy), &self.output_schema)
                .await?;
            groups.push(group);
            for (flag_rows, mut offsets) in assigned.iter_mut().zip(merge.into_assigned()) {
                if !offsets.is_empty() {
                    offsets.optimize();
                    flag_rows.insert_bitmap(fragment_id, offsets);
                }
            }
            rows.close();
        }
        Ok(())
    }

    fn publication(
        &self,
        groups: &mut Vec<DataReplacementGroup>,
        assigned: Vec<RowAddrTreeMap>,
    ) -> Result<Option<Transaction>> {
        let Some(DataReplacementGroup(_, first)) = groups.first() else {
            return Ok(None);
        };
        if groups
            .iter()
            .any(|DataReplacementGroup(_, file)| file.fields != first.fields)
        {
            let fields: Vec<(u64, &[i32])> = groups
                .iter()
                .map(|DataReplacementGroup(fragment_id, file)| (*fragment_id, &file.fields[..]))
                .collect();
            return Err(Error::internal(format!(
                "staged files write different fields: {fields:?}"
            )));
        }
        groups.sort_by_key(|DataReplacementGroup(fragment_id, _)| *fragment_id);
        let updates = self
            .outputs
            .iter()
            .zip(assigned)
            .map(|(output, rows)| CellFlagUpdate {
                flag_id: output.flag_id,
                value: true,
                rows,
            })
            .collect();
        let replacements = std::mem::take(groups);
        Ok(Some(
            TransactionBuilder::new(
                self.snapshot.manifest.version,
                Operation::DataReplacement { replacements },
            )
            .cell_flag_changes(CellFlagChanges {
                updates,
                ..Default::default()
            })
            .build(),
        ))
    }

    // TODO: the smallest extension that lifts the mixed-layout refusal is in
    // the manifest build's DataReplacement: where the new file covers V2
    // files only partially, tombstone the covered fields in the existing files
    // and append the new file, as its fully covered path already does.
    /// Refuse the layouts the manifest build cannot replace in one file: it
    /// swaps or tombstones the outputs where every one is stored, or appends
    /// the file where none is, and fails otherwise.
    fn check_layout(&self, fragment: &Fragment) -> Result<()> {
        let schema = self.snapshot.schema();
        let replaced = self.output_field_ids.as_ref();
        let mut covered = HashSet::new();
        for file in &fragment.files {
            covered.extend(file.schema(schema).field_ids());
            covered.extend(file.fields.iter().copied());
        }
        if covered.is_disjoint(replaced) {
            return Ok(());
        }
        let names = |outputs: &[&Output]| {
            outputs
                .iter()
                .map(|output| output.field.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let all: Vec<&Output> = self.outputs.iter().collect();
        if replaced.is_subset(&covered) {
            // The build tombstones fields in V2 files only.
            if fragment.files.iter().all(|file| {
                file.file_version()
                    .is_ok_and(|version| version != ConcreteFileVersion::V1)
                    || file.fields.iter().all(|field| !replaced.contains(field))
            }) {
                return Ok(());
            }
            return Err(Error::invalid_input(format!(
                "outputs [{}] cannot be replaced on fragment {}: a legacy (v1) data file stores \
                 them",
                names(&all),
                fragment.id
            )));
        }
        let (stored, unstored): (Vec<&Output>, Vec<&Output>) =
            all.iter().copied().partition(|output| {
                Schema {
                    fields: vec![output.field.clone()],
                    metadata: Default::default(),
                }
                .field_ids()
                .iter()
                .all(|id| covered.contains(id))
            });
        Err(Error::invalid_input(format!(
            "outputs [{}] cannot be replaced together on fragment {}: [{}] are stored in its \
             data files but [{}] are not, so one commit cannot publish them together there; \
             publish [{}] on its own first, then stage them together",
            names(&all),
            fragment.id,
            names(&stored),
            names(&unstored),
            names(&unstored)
        )))
    }

    /// Split a computed batch into per-fragment runs after checking its
    /// columns and masks. Rows are checked as their fragment reads them.
    fn split(&self, batch: ComputedBatch) -> Result<Vec<Run>> {
        let ComputedBatch { rows, assigned } = batch;
        let num_rows = rows.num_rows();
        let schema = rows.schema();
        if let Some(duplicate) = duplicate_field_path(schema.fields(), "") {
            return Err(Error::invalid_input(format!(
                "computed batch has column '{duplicate}' twice"
            )));
        }
        let addrs = rows.column_by_name(ROW_ADDR).ok_or_else(|| {
            let hint = if rows.column_by_name(ROW_ID).is_some() {
                "; row ids are not physical addresses: scan with with_row_address()"
            } else {
                ""
            };
            Error::invalid_input(format!("computed batch has no '{ROW_ADDR}' column{hint}"))
        })?;
        let addrs = addrs.as_primitive_opt::<UInt64Type>().ok_or_else(|| {
            Error::invalid_input(format!(
                "computed batch has a '{ROW_ADDR}' of type {}, expected UInt64",
                addrs.data_type()
            ))
        })?;
        if addrs.null_count() > 0
            && let Some(row) = (0..num_rows).find(|row| addrs.is_null(*row))
        {
            return Err(Error::invalid_input(format!(
                "computed batch has a null '{ROW_ADDR}' at batch row {row}"
            )));
        }
        let mut columns: Vec<Option<ArrayRef>> = vec![None; self.outputs.len()];
        for (field, column) in schema.fields().iter().zip(rows.columns()) {
            if field.name() == ROW_ADDR {
                continue;
            }
            let Some(index) = self.output_index(field.name()) else {
                let outputs: Vec<&str> = self
                    .outputs
                    .iter()
                    .map(|output| output.field.name.as_str())
                    .collect();
                return Err(Error::invalid_input(format!(
                    "computed batch has column '{}', which is not an output of this publication \
                     (outputs: [{}])",
                    field.name(),
                    outputs.join(", ")
                )));
            };
            columns[index] = Some(self.conform(index, field, column)?);
        }
        if num_rows > 0 && columns.iter().all(Option::is_none) {
            return Err(Error::invalid_input(format!(
                "computed batch of {num_rows} rows carries no output column"
            )));
        }
        let mut masks: Vec<Option<BooleanBuffer>> = vec![None; self.outputs.len()];
        for (name, mask) in assigned {
            let Some(index) = self
                .output_index(&name)
                .filter(|index| columns[*index].is_some())
            else {
                return Err(Error::invalid_input(format!(
                    "assignment mask for '{name}' names no column of the batch"
                )));
            };
            if masks[index].is_some() {
                return Err(Error::invalid_input(format!(
                    "assignment mask for '{name}' is given twice"
                )));
            }
            if mask.len() != num_rows {
                return Err(Error::invalid_input(format!(
                    "assignment mask for '{name}' has {} bits for a batch of {num_rows} rows",
                    mask.len()
                )));
            }
            masks[index] = Some(mask);
        }
        let outputs: Vec<Option<Assignment>> = columns
            .into_iter()
            .zip(masks)
            .map(|(values, mask)| {
                let values = values?;
                match mask {
                    Some(mask) if mask.count_set_bits() == 0 => None,
                    Some(mask) if mask.count_set_bits() == mask.len() => {
                        Some(Assignment { values, mask: None })
                    }
                    mask => Some(Assignment { values, mask }),
                }
            })
            .collect();
        let addrs = addrs.values();
        let mut runs = Vec::new();
        let mut start = 0;
        while start < num_rows {
            let fragment_id = RowAddress::from(addrs[start]).fragment_id();
            let len = addrs[start..]
                .iter()
                .take_while(|addr| RowAddress::from(**addr).fragment_id() == fragment_id)
                .count();
            runs.push(Run::slice_of(fragment_id, addrs, &outputs, start, len));
            start += len;
        }
        Ok(runs)
    }

    /// `column`, computed values of output `index`, conformed to its field
    /// with nullability relaxed, as the staged batches carry it.
    fn conform(&self, index: usize, field: &FieldRef, column: &ArrayRef) -> Result<ArrayRef> {
        let output = &self.outputs[index];
        if let Some(mismatch) = logical_type_mismatch(field, &output.field) {
            return Err(Error::invalid_input(format!(
                "computed values for output '{}' do not match its field at version {}: {mismatch}",
                output.field.name, self.snapshot.manifest.version
            )));
        }
        let values = RecordBatch::try_new(
            Arc::new(ArrowSchema::new(vec![field.clone()])),
            vec![column.clone()],
        )?;
        Ok(values.project_by_schema(&output.relaxed)?.column(0).clone())
    }

    /// Refuse a report that certifies rows of a flag this stager does not
    /// stage, whose values its plan would lose. A deferred row of such a flag
    /// needs no plan: it is false at the snapshot, where a refresh of pending
    /// rows finds it, or true with a valid value.
    fn check_report_flags(&self, report: &PublicationReport) -> Result<()> {
        let mut flags = BTreeSet::new();
        for update in report
            .deferred_groups
            .iter()
            .flat_map(|group| &group.valid_rows)
            .filter(|update| !update.rows.is_empty())
        {
            flags.insert(update.flag_id);
        }
        let Some(flag_id) = flags
            .into_iter()
            .find(|flag_id| !self.outputs.iter().any(|output| output.flag_id == *flag_id))
        else {
            return Ok(());
        };
        let schema = self.snapshot.schema();
        let definition = self
            .snapshot
            .manifest
            .cell_flags
            .as_deref()
            .and_then(|registry| registry.definition(flag_id));
        Err(match definition {
            Some(definition) => Error::invalid_input(format!(
                "the report certifies rows of {} to reuse, but this stager does not stage its \
                 output '{}'; declare it, or plan its follow-up with a stager that does",
                definition.label(schema),
                schema
                    .field_path(definition.field_id)
                    .unwrap_or_else(|_| definition.field_id.to_string())
            )),
            None => Error::invalid_input(format!(
                "the report certifies rows of cell flag {flag_id} to reuse, but it is not \
                 registered at version {}",
                self.snapshot.manifest.version
            )),
        })
    }
}

/// `actual`'s difference from `expected` in logical type, shape and names,
/// nullability and child order aside, or `None` when they agree.
fn logical_type_mismatch(actual: &ArrowField, expected: &Field) -> Option<String> {
    let options = SchemaCompareOptions {
        compare_nullability: NullabilityComparison::Ignore,
        ignore_field_order: true,
        ..Default::default()
    };
    match Field::try_from(actual) {
        Ok(actual) if actual.compare_with_options(expected, &options) => None,
        Ok(actual) => Some(
            actual
                .explain_difference(expected, &options)
                .unwrap_or_else(|| {
                    format!("{} is not {}", actual.data_type(), expected.data_type())
                }),
        ),
        Err(error) => Some(error.to_string()),
    }
}

fn has_blob(field: &Field) -> bool {
    field.is_blob() || field.children.iter().any(has_blob)
}

/// Refuse declared outputs, keyed by field id in `flags`, of which one
/// watches another through outputs of dependent flags not declared: the
/// commit clears those flags where the upstream is published, and the clear
/// reaches the downstream flag on the rows this publication sets true.
fn check_undeclared_intermediates(
    schema: &Schema,
    registry: &CellFlagRegistry,
    flags: &HashMap<i32, &CellFlagDefinition>,
) -> Result<()> {
    let name = |field_id: i32| {
        schema
            .field_by_id(field_id)
            .map_or_else(|| field_id.to_string(), |field| field.name.clone())
    };
    for downstream in schema
        .fields
        .iter()
        .filter(|field| flags.contains_key(&field.id))
    {
        // Each source with the undeclared outputs between it and `downstream`.
        let mut sources: Vec<(i32, Vec<i32>)> = flags[&downstream.id]
            .clear_on_write
            .iter()
            .map(|source| (*source, Vec::new()))
            .collect();
        let mut visited = HashSet::new();
        while let Some((source, mut through)) = sources.pop() {
            if flags.contains_key(&source) {
                if through.is_empty() {
                    continue;
                }
                through.reverse();
                let through = through
                    .into_iter()
                    .map(|field_id| format!("'{}'", name(field_id)))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Error::invalid_input(format!(
                    "output '{}' depends on '{}' through {through}, which this stager does not \
                     stage; declare {through} too, or stage '{}' in a separate publication",
                    downstream.name,
                    name(source),
                    downstream.name
                )));
            }
            let Some(intermediate) = registry.dependent_flag(source) else {
                continue;
            };
            if !visited.insert(source) {
                continue;
            }
            through.push(source);
            sources.extend(
                intermediate
                    .clear_on_write
                    .iter()
                    .map(|next| (*next, through.clone())),
            );
        }
    }
    Ok(())
}

/// `outputs` indices, each after the declared outputs it watches.
fn upstream_first(outputs: &[Output]) -> Result<Vec<usize>> {
    let mut order = Vec::with_capacity(outputs.len());
    let mut is_placed = vec![false; outputs.len()];
    while order.len() < outputs.len() {
        let placed_before = order.len();
        for (index, output) in outputs.iter().enumerate() {
            if !is_placed[index] && output.upstreams.iter().all(|upstream| is_placed[*upstream]) {
                is_placed[index] = true;
                order.push(index);
            }
        }
        if order.len() == placed_before {
            // Registration refuses cycles, so only a corrupt registry has one.
            let cycle: Vec<&str> = outputs
                .iter()
                .zip(&is_placed)
                .filter(|(_, is_placed)| !**is_placed)
                .map(|(output, _)| output.field.name.as_str())
                .collect();
            return Err(Error::internal(format!(
                "the cell flags of outputs [{}] form a cycle",
                cycle.join(", ")
            )));
        }
    }
    Ok(order)
}

/// The caller's computed rows: split into per-fragment runs as batches
/// arrive, and checked against the snapshot as the open fragment takes them.
struct ComputedRows<'a, S> {
    stager: &'a PublicationStager,
    stream: Pin<&'a mut S>,
    is_exhausted: bool,
    /// Runs of the last batch not taken yet.
    queued: VecDeque<Run>,
    /// Fragments whose run ended.
    finished: RoaringBitmap,
    fragment: Option<OpenFragment>,
}

/// The fragment whose run is being read, as the snapshot has it.
struct OpenFragment {
    id: u32,
    physical_rows: u32,
    deleted: Option<Arc<DeletionVector>>,
    last_offset: Option<u32>,
}

impl<S, B> ComputedRows<'_, S>
where
    S: Stream<Item = Result<B>> + Send,
    B: Into<ComputedBatch> + Send,
{
    /// The address of the next computed row, or `None` once the stream ends.
    async fn next_addr(&mut self) -> Result<Option<u64>> {
        while self.queued.is_empty() && !self.is_exhausted {
            match self.stream.next().await {
                Some(batch) => self.queued.extend(self.stager.split(batch?.into())?),
                None => self.is_exhausted = true,
            }
        }
        Ok(self.queued.front().map(|run| run.addrs[0]))
    }

    /// Open the fragment of the row at `first_addr`, which starts its run.
    /// Returns the snapshot's fragment and its physical row count.
    async fn open(&mut self, first_addr: u64) -> Result<(FileFragment, u32)> {
        let version = self.stager.snapshot.manifest.version;
        let fragment_id = RowAddress::from(first_addr).fragment_id();
        if self.finished.contains(fragment_id) {
            return Err(Error::invalid_input(format!(
                "computed rows for fragment {fragment_id} arrived after its run ended; send each \
                 fragment's rows together, once"
            )));
        }
        let fragment = self
            .stager
            .snapshot
            .get_fragment(fragment_id as usize)
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "computed row {first_addr:#x} names fragment {fragment_id}, which is not in \
                     version {version}"
                ))
            })?;
        let physical_rows = fragment.metadata().physical_rows;
        let physical_rows = physical_rows
            .and_then(|rows| u32::try_from(rows).ok())
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "cannot stage fragment {fragment_id}: its physical row count at version \
                     {version} is {physical_rows:?}, not a count a cell flag row address holds"
                ))
            })?;
        self.fragment = Some(OpenFragment {
            id: fragment_id,
            physical_rows,
            deleted: fragment.get_deletion_vector().await?,
            last_offset: None,
        });
        Ok((fragment, physical_rows))
    }

    /// The next rows of the open fragment, checked, or `None` once its run
    /// ended.
    async fn next_run(&mut self) -> Result<Option<Run>> {
        self.next_addr().await?;
        let Some(open) = self.fragment.as_mut() else {
            return Ok(None);
        };
        let Some(run) = self.queued.pop_front() else {
            return Ok(None);
        };
        if run.fragment_id != open.id {
            self.queued.push_front(run);
            return Ok(None);
        }
        open.check(self.stager, &run)?;
        Ok(Some(run))
    }

    fn close(&mut self) {
        if let Some(open) = self.fragment.take() {
            self.finished.insert(open.id);
        }
    }
}

impl OpenFragment {
    fn check(&mut self, stager: &PublicationStager, run: &Run) -> Result<()> {
        let version = stager.snapshot.manifest.version;
        let fragment_id = self.id;
        for row in 0..run.len() {
            let offset = run.offset(row);
            if let Some(previous) = self.last_offset
                && offset <= previous
            {
                return Err(Error::invalid_input(format!(
                    "computed row {:#x} (fragment {fragment_id}, offset {offset}) follows offset \
                     {previous}; offsets must strictly ascend within a fragment, and a row appears \
                     once, carrying every output it assigns",
                    run.addrs[row]
                )));
            }
            if offset >= self.physical_rows {
                return Err(Error::invalid_input(format!(
                    "computed row {:#x} names offset {offset} of fragment {fragment_id}, which has \
                     {} physical rows at version {version}",
                    run.addrs[row], self.physical_rows
                )));
            }
            self.last_offset = Some(offset);
        }
        // The commit sets a flag true on a row deleted at the read version
        // without complaint, publishing a value no reader sees.
        if let Some(deleted) = &self.deleted
            && let Some(row) =
                (0..run.len()).find(|row| run.assigns(*row) && deleted.contains(run.offset(*row)))
        {
            return Err(Error::invalid_input(format!(
                "computed row {:#x} (fragment {fragment_id}, offset {}) is deleted at version \
                 {version}; a deleted row cannot be published",
                run.addrs[row],
                run.offset(row)
            )));
        }
        // Values on unassigned rows are never written, so only an assigned
        // NULL breaks the field. Nested non-null children are the writer's to
        // enforce.
        for (output, assignment) in stager.outputs.iter().zip(&run.outputs) {
            let Some(assignment) = assignment else {
                continue;
            };
            if output.field.nullable || assignment.values.null_count() == 0 {
                continue;
            }
            if let Some(row) = (0..run.len())
                .find(|row| assignment.values.is_null(*row) && assignment.is_assigned(*row))
            {
                return Err(Error::invalid_input(format!(
                    "output '{}' is not nullable, but the computed batch assigns NULL at row {:#x}",
                    output.field.name, run.addrs[row]
                )));
            }
        }
        check_chains(stager, run)
    }
}

/// Refuse a row that assigns an output and one it depends on but not a
/// declared output between them: publishing the upstream clears the one
/// between there, and the commit carries that clear on to the downstream
/// output this row sets true.
fn check_chains(stager: &PublicationStager, run: &Run) -> Result<()> {
    if stager
        .outputs
        .iter()
        .all(|output| output.upstreams.is_empty())
    {
        return Ok(());
    }
    let count = stager.outputs.len();
    let mut is_assigned = vec![false; count];
    // Per output the commit clears on the row, the assigned output that
    // clears it.
    let mut cleared_by: Vec<Option<usize>> = vec![None; count];
    for row in (0..run.len()).filter(|row| run.assigns(*row)) {
        for (index, assignment) in run.outputs.iter().enumerate() {
            is_assigned[index] = assignment
                .as_ref()
                .is_some_and(|assignment| assignment.is_assigned(row));
            cleared_by[index] = None;
        }
        for &index in stager.upstream_first.iter() {
            let upstreams = &stager.outputs[index].upstreams;
            let cleared_upstream = upstreams
                .iter()
                .find_map(|&upstream| cleared_by[upstream].map(|root| (upstream, root)));
            if !is_assigned[index] {
                cleared_by[index] = cleared_upstream.map(|(_, root)| root).or_else(|| {
                    upstreams
                        .iter()
                        .copied()
                        .find(|upstream| is_assigned[*upstream])
                });
            } else if let Some((between, root)) = cleared_upstream {
                let name = |index: usize| stager.outputs[index].field.name.as_str();
                return Err(Error::invalid_input(format!(
                    "computed row {:#x} assigns '{}' and '{}' but not '{}' between them: \
                     publishing '{}' clears '{}' there, and with it '{}'; assign '{}' on the row \
                     too, or leave '{}' unassigned",
                    run.addrs[row],
                    name(root),
                    name(index),
                    name(between),
                    name(root),
                    name(between),
                    name(index),
                    name(between),
                    name(index)
                )));
            }
        }
    }
    Ok(())
}

/// The staged batches of the open fragment: its copy-through windows with
/// the computed rows merged in, pulled from `rows` only as each window needs
/// them. The stream borrows `rows` and `merge`, which
/// [`FileFragment::write_columns`] drives to the end before returning.
fn merged_windows<'a, S, B, C>(
    rows: &'a mut ComputedRows<'_, S>,
    merge: &'a mut FragmentMerge,
    copy: C,
) -> impl Stream<Item = Result<RecordBatch>> + Send + 'a
where
    S: Stream<Item = Result<B>> + Send,
    B: Into<ComputedBatch> + Send,
    C: Stream<Item = Result<RecordBatch>> + Send + Unpin + 'a,
{
    stream::try_unfold((rows, merge, copy), |(rows, merge, mut copy)| async move {
        loop {
            let Some(window) = copy.try_next().await? else {
                merge.check_complete()?;
                // Taking the rest of the run checks it: every row left
                // lies past the fragment's physical rows.
                if let Some(run) = rows.next_run().await? {
                    return Err(Error::internal(format!(
                        "computed row {:#x} remains after the copy-through of its fragment ended",
                        run.addrs[0]
                    )));
                }
                return Ok(None);
            };
            if window.num_rows() == 0 {
                continue;
            }
            while merge.needs_rows_for(window.num_rows()) {
                let Some(run) = rows.next_run().await? else {
                    break;
                };
                merge.push(run);
            }
            let merged = merge.merge(window)?;
            return Ok(Some((merged, (rows, merge, copy))));
        }
    })
}

/// Rows of a report that are live at the follow-up's snapshot.
struct LiveRows<'a> {
    snapshot: &'a Dataset,
    /// Per fragment id, its physical rows and deleted offsets, or `None` when
    /// the snapshot does not have it.
    fragments: HashMap<u32, Option<(u32, RoaringBitmap)>>,
}

impl LiveRows<'_> {
    /// `rows` without the fragments the snapshot lacks and the rows it
    /// deletes, with `Full` expanded to physical offsets.
    async fn of(&mut self, rows: &RowAddrTreeMap) -> Result<RowAddrTreeMap> {
        let mut live = RowAddrTreeMap::new();
        for (fragment_id, selection) in rows.iter() {
            let Some((physical_rows, deleted)) = self.fragment(*fragment_id).await? else {
                continue;
            };
            let mut offsets = match selection {
                RowAddrSelection::Full => {
                    let mut offsets = RoaringBitmap::new();
                    offsets.insert_range(0..*physical_rows);
                    offsets
                }
                RowAddrSelection::Partial(offsets) => offsets.clone(),
            };
            offsets -= deleted;
            if !offsets.is_empty() {
                live.insert_bitmap(*fragment_id, offsets);
            }
        }
        Ok(live)
    }

    async fn fragment(&mut self, fragment_id: u32) -> Result<&Option<(u32, RoaringBitmap)>> {
        if !self.fragments.contains_key(&fragment_id) {
            let loaded = match self.snapshot.get_fragment(fragment_id as usize) {
                None => None,
                Some(fragment) => {
                    let version = self.snapshot.manifest.version;
                    let physical_rows = fragment.metadata().physical_rows;
                    let physical_rows = physical_rows
                        .and_then(|rows| u32::try_from(rows).ok())
                        .ok_or_else(|| {
                            Error::invalid_input(format!(
                                "cannot plan fragment {fragment_id}: its physical row count at \
                                 version {version} is {physical_rows:?}, not a count a cell flag \
                                 row address holds"
                            ))
                        })?;
                    let deleted = fragment
                        .get_deletion_vector()
                        .await?
                        .map(|deleted| RoaringBitmap::from(deleted.as_ref()))
                        .unwrap_or_default();
                    Some((physical_rows, deleted))
                }
            };
            self.fragments.insert(fragment_id, loaded);
        }
        Ok(&self.fragments[&fragment_id])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use arrow_array::{RecordBatchIterator, StringArray, UInt64Array};

    use super::*;
    use crate::dataset::cell_flag::CellFlagOptions;

    /// With two-row copy windows and one-row caller batches, each merged
    /// window of an 8-row fragment pulls the caller's stream only through its
    /// own rows, never the whole fragment ahead.
    #[tokio::test]
    async fn merged_windows_pull_computed_rows_lazily() {
        let batch = RecordBatch::try_from_iter([
            (
                "body",
                Arc::new(StringArray::from_iter_values(
                    (0..8).map(|offset| format!("b{offset}")),
                )) as ArrayRef,
            ),
            ("summary", Arc::new(StringArray::new_null(8)) as ArrayRef),
        ])
        .unwrap();
        let schema = batch.schema();
        let mut dataset = Dataset::write(
            RecordBatchIterator::new([Ok(batch)], schema),
            "memory://",
            None,
        )
        .await
        .unwrap();
        dataset
            .register_cell_flag(
                "summary",
                "ready",
                CellFlagOptions::default().with_clear_on_write(["body"]),
            )
            .await
            .unwrap();
        let stager = PublicationStager::try_new(Arc::new(dataset), &["summary"])
            .unwrap()
            .with_copy_batch_size(2);
        let pulled = Arc::new(AtomicUsize::new(0));
        let computed = stream::iter(0..8u32).map({
            let pulled = pulled.clone();
            move |offset| {
                pulled.fetch_add(1, Ordering::SeqCst);
                let addr = u64::from(RowAddress::new_from_parts(0, offset));
                let rows = RecordBatch::try_from_iter([
                    (
                        ROW_ADDR,
                        Arc::new(UInt64Array::from(vec![addr])) as ArrayRef,
                    ),
                    (
                        "summary",
                        Arc::new(StringArray::from(vec![format!("s{offset}")])) as ArrayRef,
                    ),
                ])
                .unwrap();
                Ok::<_, Error>(ComputedBatch::new(rows))
            }
        });

        // Open the fragment and start its merge as `stage_fragments` does.
        let mut rows = ComputedRows {
            stager: &stager,
            stream: std::pin::pin!(computed),
            is_exhausted: false,
            queued: VecDeque::new(),
            finished: RoaringBitmap::new(),
            fragment: None,
        };
        let first_addr = rows.next_addr().await.unwrap().unwrap();
        let (fragment, physical_rows) = rows.open(first_addr).await.unwrap();
        let mut merge = FragmentMerge::new(
            0,
            physical_rows,
            stager.output_schema.clone(),
            stager.relaxed_schema.clone(),
        );
        merge.push(rows.next_run().await.unwrap().unwrap());
        let copy = fragment
            .read_physical_slice_with_row_addr(
                0..u64::from(physical_rows),
                &stager.output_schema,
                stager.copy_batch_size,
            )
            .await
            .unwrap()
            .buffered(COPY_READ_AHEAD);

        let mut windows = std::pin::pin!(merged_windows(&mut rows, &mut merge, copy));
        let mut pulled_after_window = Vec::new();
        let mut merged = Vec::new();
        while let Some(window) = windows.try_next().await.unwrap() {
            pulled_after_window.push(pulled.load(Ordering::SeqCst));
            merged.extend(
                window["summary"]
                    .as_string::<i32>()
                    .iter()
                    .map(|value| value.map(str::to_string)),
            );
        }
        // Window w holds offsets 2w and 2w + 1, the first 2(w + 1) batches.
        assert_eq!(pulled_after_window, vec![2, 4, 6, 8]);
        assert_eq!(
            merged,
            (0..8)
                .map(|offset| Some(format!("s{offset}")))
                .collect::<Vec<_>>()
        );
    }
}
