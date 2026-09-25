// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Cell-flag helpers for `dependent_cell_flags.rs`: flagged tables, a refresh
//! executor built on the prototype's public API, and the correctness checks
//! every flag workload runs.
//!
//! Only the flag harness includes this file, with
//! `#[path = "cell_flags_common/flags.rs"] mod flags;`. It uses prototype-only
//! APIs, so `mod.rs` and the regression harness stay identical on the
//! baseline and the prototype.

#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use arrow_array::builder::StringBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_array::{Array, ArrayRef, Int64Array, RecordBatch, StringArray};
use arrow_schema::Schema as ArrowSchema;
use futures::{TryStreamExt, stream};
use lance::dataset::cell_flag::{CellFlagOptions, DeferralReason, PublicationReport};
use lance::dataset::transaction::{
    CellFlagChanges, CellFlagUpdate, DataReplacementGroup, Operation, Transaction,
    TransactionBuilder,
};
use lance::dataset::{CommitBuilder, Dataset, DependencyConflictPolicy, PublicationResult};
use lance::session::Session;
use lance_core::ROW_ADDR;
use lance_select::{RowAddrSelection, RowAddrTreeMap, RowSetOps};
use lance_table::format::Fragment;
use roaring::{RoaringBitmap, RoaringTreemap};

use crate::common::{
    BenchConfig, Fingerprint, OutputColumn, SimulatedUdf, create_articles_dataset, fingerprint,
    id_in_predicate, ms_since, timed,
};

/// Name of the dependent flag registered on each output.
pub const FLAG_NAME: &str = "ready";
/// Tag of the state where the flags are registered and every row is pending.
pub const PENDING_TAG: &str = "pending";
/// Tag of the state where every output is published (every flag true).
pub const PUBLISHED_TAG: &str = "published";

// ---------------------------------------------------------------------------
// Flagged tables
// ---------------------------------------------------------------------------

/// The dependent `ready` flag of one output: cleared by writes to the
/// output's inputs, and masking the output while false.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlaggedOutput {
    pub column: OutputColumn,
    pub flag_id: u32,
}

impl FlaggedOutput {
    /// Whether a write to `field` clears this flag.
    pub fn watches(&self, field: &str) -> bool {
        self.column.inputs().contains(&field) || self.column.name() == field
    }
}

pub async fn register_flags(dataset: &mut Dataset, columns: &[OutputColumn]) -> Vec<FlaggedOutput> {
    let mut outputs = Vec::new();
    for column in columns {
        let definition = dataset
            .register_cell_flag(
                column.name(),
                FLAG_NAME,
                CellFlagOptions::default()
                    .with_clear_on_write(column.inputs())
                    .with_mask_when_false(true),
            )
            .await
            .expect("register dependent masking flag");
        assert!(definition.is_dependent());
        outputs.push(FlaggedOutput {
            column: *column,
            flag_id: definition.flag_id,
        });
    }
    outputs
}

/// The `ready` flags registered on `columns` at `dataset`'s version.
pub fn flagged_outputs(dataset: &Dataset, columns: &[OutputColumn]) -> Vec<FlaggedOutput> {
    columns
        .iter()
        .map(|column| FlaggedOutput {
            column: *column,
            flag_id: dataset
                .cell_flag(column.name(), FLAG_NAME)
                .unwrap_or_else(|| panic!("no '{FLAG_NAME}' flag on {}", column.name()))
                .flag_id,
        })
        .collect()
}

/// What a tagged state holds, to check that every sample starts from it.
#[derive(Debug, Clone)]
pub struct TagState {
    pub version: u64,
    pub fragments: Arc<Vec<Fragment>>,
    pub true_rows: Vec<RowAddrTreeMap>,
    pub fingerprint: Fingerprint,
}

impl TagState {
    pub async fn of(dataset: &Dataset, tag: &str, outputs: &[FlaggedOutput]) -> Self {
        let tagged = dataset.checkout_version(tag).await.expect("checkout tag");
        Self {
            version: tagged.version().version,
            fragments: tagged.manifest().fragments.clone(),
            true_rows: true_rows_of(&tagged, outputs),
            fingerprint: fingerprint(&tagged).await,
        }
    }

    /// Assert that `dataset` holds this state. The content fingerprint scans
    /// every column, so it is optional.
    pub async fn assert_matches(
        &self,
        dataset: &Dataset,
        outputs: &[FlaggedOutput],
        check_content: bool,
    ) {
        assert_eq!(
            dataset.manifest().fragments,
            self.fragments,
            "restoring a tag must reproduce its fragments"
        );
        assert_eq!(
            true_rows_of(dataset, outputs),
            self.true_rows,
            "restoring a tag must reproduce its flag state"
        );
        if check_content {
            assert_eq!(
                fingerprint(dataset).await,
                self.fingerprint,
                "restoring a tag must reproduce its content"
            );
        }
    }
}

pub fn true_rows_of(dataset: &Dataset, outputs: &[FlaggedOutput]) -> Vec<RowAddrTreeMap> {
    outputs
        .iter()
        .map(|output| {
            dataset
                .cell_flag_true_rows(output.flag_id)
                .expect("flag registered")
        })
        .collect()
}

/// Create the articles table, register a dependent masking flag on each of
/// `columns` (tagged [`PENDING_TAG`]), then publish every output with one
/// refresh each (tagged [`PUBLISHED_TAG`] when `publish`). Any existing
/// directory at `uri` is removed.
pub async fn create_flagged_dataset(
    uri: &Path,
    config: &BenchConfig,
    columns: &[OutputColumn],
    udf: SimulatedUdf,
    publish: bool,
) -> (Dataset, Vec<FlaggedOutput>) {
    let mut dataset = create_articles_dataset(uri, config, Arc::new(Session::default())).await;
    let outputs = register_flags(&mut dataset, columns).await;
    dataset
        .tags()
        .create(PENDING_TAG, dataset.version().version)
        .await
        .expect("tag pending state");
    if !publish {
        return (dataset, outputs);
    }
    for output in &outputs {
        let staged = stage_pending(Arc::new(dataset.clone()), *output, udf, None, false).await;
        let (result, _) = staged.publish(DependencyConflictPolicy::Reject).await;
        let result = result.expect("publish the initial refresh");
        let counts = check_report(&staged, &result.report);
        assert_eq!(counts.published, config.scale_rows, "initial refresh");
        dataset = result.dataset;
    }
    dataset
        .tags()
        .create(PUBLISHED_TAG, dataset.version().version)
        .await
        .expect("tag published state");
    (dataset, outputs)
}

/// Start a sample from the state tagged `tag`: restore it (metadata only) and
/// remove every untagged version and the files only they referenced, as
/// `reset_to_initial` does for the initial tag.
pub async fn reset_to_tag(dataset: &Dataset, tag: &str) -> Dataset {
    let mut restored = dataset.checkout_version(tag).await.expect("checkout tag");
    restored.restore().await.expect("restore tagged version");
    restored
        .cleanup_old_versions(chrono::Duration::zero(), Some(true), Some(false))
        .await
        .expect("clean up previous samples");
    restored
}

/// An `(id, <column>)` source batch writing `value(id)` into `column` for `ids`.
pub fn column_source_batch(
    dataset: &Dataset,
    column: &str,
    ids: &[u64],
    value: impl Fn(u64) -> String,
) -> RecordBatch {
    let full = ArrowSchema::from(dataset.schema());
    let positions = ["id", column].map(|name| full.index_of(name).expect("source column exists"));
    let schema = Arc::new(
        full.project(&positions)
            .expect("project merge source schema"),
    );
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from_iter_values(
                ids.iter().map(|id| *id as i64),
            )),
            Arc::new(StringArray::from_iter_values(
                ids.iter().map(|id| value(*id)),
            )),
        ],
    )
    .expect("merge source batch")
}

// ---------------------------------------------------------------------------
// Row sets
// ---------------------------------------------------------------------------

/// Physical row count of every fragment at `dataset`'s version, by fragment id.
pub fn physical_rows(dataset: &Dataset) -> BTreeMap<u32, u32> {
    dataset
        .manifest()
        .fragments
        .iter()
        .map(|fragment| {
            (
                u32::try_from(fragment.id).expect("fragment id fits a row address"),
                u32::try_from(fragment.physical_rows.expect("physical row count"))
                    .expect("physical rows fit a row address"),
            )
        })
        .collect()
}

/// `rows` as row addresses, with `Full` fragments expanded by `physical_rows`.
pub fn materialize(rows: &RowAddrTreeMap, physical_rows: &BTreeMap<u32, u32>) -> RoaringTreemap {
    let mut out = RoaringTreemap::new();
    for (fragment, selection) in rows.iter() {
        let base = u64::from(*fragment) << 32;
        match selection {
            RowAddrSelection::Full => {
                let rows = physical_rows.get(fragment).unwrap_or_else(|| {
                    panic!(
                        "a Full selection names fragment {fragment}, which is not in the version"
                    )
                });
                out.insert_range(base..base + u64::from(*rows));
            }
            RowAddrSelection::Partial(offsets) => {
                out.extend(offsets.iter().map(|offset| base | u64::from(offset)));
            }
        }
    }
    out
}

pub fn count_rows(rows: &RowAddrTreeMap, physical_rows: &BTreeMap<u32, u32>) -> u64 {
    rows.iter()
        .map(|(fragment, selection)| match selection {
            RowAddrSelection::Full => u64::from(physical_rows[fragment]),
            RowAddrSelection::Partial(offsets) => offsets.len(),
        })
        .sum()
}

/// Rows the transaction of `dataset`'s version clears for each flag through
/// derived invalidations. `physical_rows` must be those of the version the
/// transaction read.
pub async fn derived_invalidations(
    dataset: &Dataset,
    outputs: &[FlaggedOutput],
    physical_rows: &BTreeMap<u32, u32>,
) -> Vec<u64> {
    let transaction = dataset
        .read_transaction()
        .await
        .expect("read transaction")
        .expect("the version has a transaction");
    outputs
        .iter()
        .map(|output| {
            transaction
                .cell_flag_changes
                .iter()
                .flat_map(|changes| changes.derived_invalidations.iter())
                .filter(|update| update.flag_id == output.flag_id)
                .map(|update| {
                    assert!(!update.value, "derived invalidations only clear");
                    count_rows(&update.rows, physical_rows)
                })
                .sum()
        })
        .collect()
}

/// Rows a row-moving update listed as moved, from its transaction.
pub async fn moved_rows(dataset: &Dataset) -> u64 {
    let transaction = dataset
        .read_transaction()
        .await
        .expect("read transaction")
        .expect("the version has a transaction");
    transaction
        .cell_flag_changes
        .iter()
        .flat_map(|changes| changes.moved_rows.iter())
        .map(|moved| moved.offsets.len())
        .sum()
}

// ---------------------------------------------------------------------------
// Refresh executor
// ---------------------------------------------------------------------------

/// Where a refresh's time and rows went. `udf_ms` is the simulated UDF only;
/// `assemble_ms` is classifying rows and building the staged columns.
#[derive(Debug, Clone, Default)]
pub struct RefreshStats {
    /// Fragments with a false flag somewhere, which the refresh scanned.
    pub fragments_scanned: u64,
    /// Fragments with live pending rows, which got a staged file.
    pub fragments_staged: u64,
    /// Rows in the staged files (every physical row of a staged fragment).
    pub rows_staged: u64,
    /// Live rows with a false flag, all assigned by the publication.
    pub rows_pending: u64,
    pub rows_computed: u64,
    pub rows_reused: u64,
    /// Completed rows copied through from the snapshot, read via Lance.
    pub rows_copied: u64,
    pub read_ms: f64,
    pub udf_ms: f64,
    pub assemble_ms: f64,
    pub stage_ms: f64,
}

/// A refresh of one output staged at one snapshot, not yet committed.
pub struct StagedPublication {
    pub output: FlaggedOutput,
    pub snapshot: Arc<Dataset>,
    pub replacements: Vec<DataReplacementGroup>,
    /// Rows assigned true: every live pending row of each staged fragment.
    pub assigned: RowAddrTreeMap,
    /// Staged values by fragment id, indexed by physical offset. Filled only
    /// when requested, for a follow-up refresh to reuse.
    pub values: HashMap<u32, ArrayRef>,
    pub stats: RefreshStats,
}

impl StagedPublication {
    pub fn read_version(&self) -> u64 {
        self.snapshot.version().version
    }

    pub fn transaction(&self) -> Transaction {
        TransactionBuilder::new(
            self.read_version(),
            Operation::DataReplacement {
                replacements: self.replacements.clone(),
            },
        )
        .cell_flag_changes(CellFlagChanges {
            updates: vec![CellFlagUpdate {
                flag_id: self.output.flag_id,
                value: true,
                rows: self.assigned.clone(),
            }],
            ..Default::default()
        })
        .build()
    }

    /// Commit the publication at its read version under `policy`, returning
    /// the commit latency.
    pub async fn publish(
        &self,
        policy: DependencyConflictPolicy,
    ) -> (lance::Result<PublicationResult>, f64) {
        timed(
            CommitBuilder::new(self.snapshot.clone())
                .with_dependency_conflict_policy(policy)
                .execute_with_report(self.transaction()),
        )
        .await
    }
}

/// Staged values a follow-up refresh takes instead of calling the UDF.
pub struct Reuse<'a> {
    pub values: &'a HashMap<u32, ArrayRef>,
    /// Row addresses whose staged values are still correct.
    pub rows: RoaringTreemap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowSource {
    CopyThrough,
    Reuse,
    Compute,
}

/// Refresh `output` at `snapshot`: for every fragment with pending rows (flag
/// false, not deleted), compute them with the simulated UDF (or take them
/// from `reuse`), copy every completed row through as read via Lance, and
/// stage one full-fragment file with `FileFragment::write_columns`. Deleted
/// offsets are staged as NULL. Fragments are processed sequentially.
///
/// The copy-through reads the masked output, so it also checks masking on
/// the fragment scan path: every pending row must read NULL.
pub async fn stage_pending(
    snapshot: Arc<Dataset>,
    output: FlaggedOutput,
    udf: SimulatedUdf,
    reuse: Option<&Reuse<'_>>,
    keep_values: bool,
) -> StagedPublication {
    let column = output.column;
    let true_rows = snapshot
        .cell_flag_true_rows(output.flag_id)
        .expect("flag registered at the snapshot");
    let target_schema = snapshot
        .schema()
        .project(&[column.name()])
        .expect("output column is declared");
    let target_arrow = Arc::new(ArrowSchema::from(&target_schema));
    let [first, second] = column.inputs();
    let mut staged = StagedPublication {
        output,
        snapshot: snapshot.clone(),
        replacements: Vec::new(),
        assigned: RowAddrTreeMap::new(),
        values: HashMap::new(),
        stats: RefreshStats::default(),
    };
    let stats = &mut staged.stats;
    for fragment in snapshot.get_fragments() {
        let fragment_id = u32::try_from(fragment.id()).expect("fragment id fits a row address");
        let physical_rows = u32::try_from(
            fragment
                .metadata()
                .physical_rows
                .expect("physical row count"),
        )
        .expect("physical rows fit a row address");
        let mut candidates = RoaringBitmap::new();
        candidates.insert_range(0..physical_rows);
        match true_rows.get(&fragment_id) {
            Some(RowAddrSelection::Full) => continue,
            Some(RowAddrSelection::Partial(done)) => candidates -= done,
            None => {}
        }
        if candidates.is_empty() {
            continue;
        }
        stats.fragments_scanned += 1;
        let copy_through = candidates.len() < u64::from(physical_rows);
        let mut projection = vec!["id", first, second];
        if copy_through {
            projection.push(column.name());
        }
        let mut scanner = fragment.scan();
        scanner
            .project(&projection)
            .expect("project refresh columns")
            .with_row_address()
            .scan_in_order(true);

        let read_start = Instant::now();
        let mut inputs = scanner.try_into_stream().await.expect("scan fragment");
        stats.read_ms += ms_since(read_start);
        let mut batches: Vec<RecordBatch> = Vec::new();
        let mut assigned = RoaringBitmap::new();
        let mut next_offset = 0u32;
        loop {
            let read_start = Instant::now();
            let Some(batch) = inputs.try_next().await.expect("read fragment batch") else {
                stats.read_ms += ms_since(read_start);
                break;
            };
            stats.read_ms += ms_since(read_start);

            let assemble_start = Instant::now();
            let offsets: Vec<u32> = batch[ROW_ADDR]
                .as_primitive::<UInt64Type>()
                .values()
                .iter()
                .map(|addr| {
                    assert_eq!(
                        addr >> 32,
                        u64::from(fragment_id),
                        "row of another fragment"
                    );
                    *addr as u32
                })
                .collect();
            let sources: Vec<RowSource> = offsets
                .iter()
                .map(|offset| {
                    if !candidates.contains(*offset) {
                        RowSource::CopyThrough
                    } else if reuse.is_some_and(|reuse| {
                        reuse
                            .rows
                            .contains((u64::from(fragment_id) << 32) | u64::from(*offset))
                    }) {
                        RowSource::Reuse
                    } else {
                        RowSource::Compute
                    }
                })
                .collect();
            stats.assemble_ms += ms_since(assemble_start);

            let firsts = batch[first].as_string::<i32>();
            let seconds = batch[second].as_string::<i32>();
            let udf_start = Instant::now();
            let computed: Vec<String> = sources
                .iter()
                .enumerate()
                .filter(|(_, source)| **source == RowSource::Compute)
                .map(|(row, _)| udf.compute(column, firsts.value(row), seconds.value(row)))
                .collect();
            stats.udf_ms += ms_since(udf_start);

            let assemble_start = Instant::now();
            let stored = copy_through.then(|| batch[column.name()].as_string::<i32>());
            let reused = reuse.map(|reuse| {
                reuse
                    .values
                    .get(&fragment_id)
                    .map(|values| values.as_string::<i32>())
            });
            let mut computed = computed.into_iter();
            let mut builder = StringBuilder::with_capacity(batch.num_rows(), batch.num_rows() * 32);
            for (row, (offset, source)) in offsets.iter().zip(&sources).enumerate() {
                assert!(
                    *offset >= next_offset,
                    "fragment scan is not in offset order"
                );
                // Offsets the scan skipped are deleted rows.
                while next_offset < *offset {
                    builder.append_null();
                    next_offset += 1;
                }
                next_offset = offset + 1;
                if let Some(stored) = stored
                    && *source != RowSource::CopyThrough
                {
                    assert!(
                        stored.is_null(row),
                        "{} reads a value at pending row {offset} of fragment {fragment_id}: \
                         masking failed",
                        column.name()
                    );
                }
                match source {
                    RowSource::CopyThrough => {
                        let stored = stored.expect("copy-through projects the output");
                        builder.append_option(stored.is_valid(row).then(|| stored.value(row)));
                        stats.rows_copied += 1;
                    }
                    RowSource::Reuse => {
                        let values = reused.flatten().unwrap_or_else(|| {
                            panic!("row {offset} of fragment {fragment_id} is reusable, but no value was staged for it")
                        });
                        let position = *offset as usize;
                        builder.append_option(
                            values.is_valid(position).then(|| values.value(position)),
                        );
                        assigned.insert(*offset);
                        stats.rows_reused += 1;
                    }
                    RowSource::Compute => {
                        builder.append_value(computed.next().expect("one value per computed row"));
                        assigned.insert(*offset);
                        stats.rows_computed += 1;
                    }
                }
            }
            batches.push(
                RecordBatch::try_new(target_arrow.clone(), vec![Arc::new(builder.finish())])
                    .expect("output batch matches the declared column"),
            );
            stats.assemble_ms += ms_since(assemble_start);
        }
        if next_offset < physical_rows {
            let mut builder = StringBuilder::new();
            for _ in next_offset..physical_rows {
                builder.append_null();
            }
            batches.push(
                RecordBatch::try_new(target_arrow.clone(), vec![Arc::new(builder.finish())])
                    .expect("output batch matches the declared column"),
            );
        }
        // Every pending candidate may have been deleted.
        if assigned.is_empty() {
            continue;
        }
        stats.rows_pending += assigned.len();
        stats.rows_staged += u64::from(physical_rows);
        stats.fragments_staged += 1;
        let columns: Vec<ArrayRef> = batches
            .iter()
            .map(|batch| batch.column(0).clone())
            .collect();

        let stage_start = Instant::now();
        let replacement = fragment
            .write_columns(stream::iter(batches.into_iter().map(Ok)), &target_schema)
            .await
            .expect("stage column file");
        stats.stage_ms += ms_since(stage_start);

        if keep_values {
            let parts: Vec<&dyn Array> = columns.iter().map(|array| array.as_ref()).collect();
            staged.values.insert(
                fragment_id,
                arrow_select::concat::concat(&parts).expect("concatenate staged values"),
            );
        }
        let mut rows = RowAddrTreeMap::new();
        if assigned.len() == u64::from(physical_rows) {
            rows.insert_fragment(fragment_id);
        } else {
            rows.insert_bitmap(fragment_id, assigned);
        }
        staged.assigned |= &rows;
        staged.replacements.push(replacement);
    }
    staged
}

// ---------------------------------------------------------------------------
// Publication report accounting
// ---------------------------------------------------------------------------

/// What happened to the rows a publication assigned.
#[derive(Debug, Clone, Default)]
pub struct ReportCounts {
    pub assigned: u64,
    pub published: u64,
    /// Deferred rows by reason (`Debug` name of [`DeferralReason`]).
    pub deferred_rows: BTreeMap<String, u64>,
    /// Deferred groups (fragments) by reason.
    pub deferred_groups: BTreeMap<String, u64>,
    /// Assigned rows of deferred groups whose staged values stay reusable.
    pub valid_rows_in_deferred_groups: u64,
    /// Assigned rows of groups whose fragment was removed or rewritten.
    pub rows_in_removed_groups: u64,
    /// Rows [`PublicationReport::reusable_rows`] reports.
    pub reusable: u64,
}

impl ReportCounts {
    pub fn deferred(&self) -> u64 {
        self.deferred_rows.values().sum()
    }

    pub fn deferred_of(&self, reason: DeferralReason) -> u64 {
        self.deferred_rows
            .get(&format!("{reason:?}"))
            .copied()
            .unwrap_or(0)
    }

    pub fn groups_deferred(&self) -> u64 {
        self.deferred_groups.values().sum()
    }
}

/// Check `report` against what `staged` assigned and count it. Every
/// assigned row must be published, deferred, held by a deferred group, or on
/// a removed fragment, and no published row may also be deferred.
pub fn check_report(staged: &StagedPublication, report: &PublicationReport) -> ReportCounts {
    assert_eq!(
        report.read_version,
        staged.read_version(),
        "report read version"
    );
    let flag_id = staged.output.flag_id;
    let physical = physical_rows(&staged.snapshot);
    let assigned = materialize(&staged.assigned, &physical);
    let published = materialize(&report.published_rows(flag_id), &physical);
    assert!(
        report
            .published
            .iter()
            .all(|update| update.flag_id == flag_id && update.value),
        "the report publishes only the flag the transaction set"
    );

    let mut counts = ReportCounts {
        assigned: assigned.len(),
        published: published.len(),
        ..Default::default()
    };
    let mut deferred = RoaringTreemap::new();
    for rows in report
        .deferred_rows
        .iter()
        .filter(|rows| rows.flag_id == flag_id)
    {
        let rows_of = materialize(&rows.rows, &physical);
        *counts
            .deferred_rows
            .entry(format!("{:?}", rows.reason))
            .or_default() += rows_of.len();
        deferred |= rows_of;
    }
    let mut valid = RoaringTreemap::new();
    let mut removed = RoaringTreemap::new();
    for group in &report.deferred_groups {
        *counts
            .deferred_groups
            .entry(format!("{:?}", group.reason))
            .or_default() += 1;
        for update in group
            .valid_rows
            .iter()
            .filter(|update| update.flag_id == flag_id)
        {
            valid |= materialize(&update.rows, &physical);
        }
        if matches!(
            group.reason,
            DeferralReason::FragmentRemoved | DeferralReason::FragmentRewritten
        ) {
            let fragment = u32::try_from(group.fragment_id).expect("fragment id fits");
            let base = u64::from(fragment) << 32;
            let mut fragment_rows = RoaringTreemap::new();
            fragment_rows.insert_range(base..base + u64::from(physical[&fragment]));
            removed |= &assigned & &fragment_rows;
        }
    }
    counts.valid_rows_in_deferred_groups = valid.len();
    counts.rows_in_removed_groups = removed.len();
    counts.reusable = materialize(&report.reusable_rows(flag_id), &physical).len();

    assert!(
        published.is_subset(&assigned),
        "published rows the transaction did not assign"
    );
    assert!(
        (&published & &deferred).is_empty(),
        "rows reported both published and deferred"
    );
    assert!(
        (&published & &valid).is_empty(),
        "rows reported both published and held by a deferred group"
    );
    let accounted = &(&(&published | &deferred) | &valid) | &removed;
    assert_eq!(
        accounted.len(),
        assigned.len(),
        "published + deferred + deferred-group rows must account for every assigned row"
    );
    assert_eq!(accounted, assigned, "report rows outside the assignment");
    counts
}

// ---------------------------------------------------------------------------
// Correctness checks
// ---------------------------------------------------------------------------

/// Outcome of [`check_visible_outputs`].
#[derive(Debug, Clone, Default)]
pub struct VisibleCheck {
    pub rows: u64,
    /// Per output: rows whose flag is true.
    pub true_rows: Vec<u64>,
    /// Per output: visible values that differ from the UDF of the row's
    /// current inputs.
    pub stale: Vec<u64>,
}

impl VisibleCheck {
    pub fn total_stale(&self) -> u64 {
        self.stale.iter().sum()
    }
}

/// Read every output with its current inputs (all rows, or the rows whose id
/// is in `ids`) and check the masking contract: a row whose flag is false
/// reads NULL, and a row whose flag is true reads a value. Visible values that
/// differ from the UDF of the current inputs are counted as stale.
pub async fn check_visible_outputs(
    dataset: &Dataset,
    outputs: &[FlaggedOutput],
    udf: SimulatedUdf,
    ids: Option<&[u64]>,
) -> VisibleCheck {
    let true_rows = true_rows_of(dataset, outputs);
    let mut projection = vec!["id", "title", "body", "language"];
    projection.extend(outputs.iter().map(|output| output.column.name()));
    let mut scanner = dataset.scan();
    scanner
        .project(&projection)
        .expect("project check columns")
        .with_row_address();
    if let Some(ids) = ids {
        scanner
            .filter(&id_in_predicate(ids))
            .expect("filter check ids");
    }
    let mut stream = scanner.try_into_stream().await.expect("scan check");
    let mut check = VisibleCheck {
        rows: 0,
        true_rows: vec![0; outputs.len()],
        stale: vec![0; outputs.len()],
    };
    while let Some(batch) = stream.try_next().await.expect("read check batch") {
        let addrs = batch[ROW_ADDR].as_primitive::<UInt64Type>();
        check.rows += batch.num_rows() as u64;
        for (index, output) in outputs.iter().enumerate() {
            let [first, second] = output.column.inputs();
            let firsts = batch[first].as_string::<i32>();
            let seconds = batch[second].as_string::<i32>();
            let values = batch[output.column.name()].as_string::<i32>();
            for row in 0..batch.num_rows() {
                let addr = addrs.value(row);
                if true_rows[index].contains(addr) {
                    check.true_rows[index] += 1;
                    assert!(
                        values.is_valid(row),
                        "{} is NULL at row {addr:#x} although its flag is true",
                        output.column.name()
                    );
                    if values.value(row)
                        != udf.compute(output.column, firsts.value(row), seconds.value(row))
                    {
                        check.stale[index] += 1;
                    }
                } else {
                    assert!(
                        values.is_null(row),
                        "{} reads a value at row {addr:#x} although its flag is false: masking \
                         failed",
                        output.column.name()
                    );
                }
            }
        }
    }
    check
}

/// Assert that a refresh completed: every row of `dataset` has every flag
/// true and reads the UDF of its current inputs. Returns the rows checked.
pub async fn assert_complete(
    dataset: &Dataset,
    outputs: &[FlaggedOutput],
    udf: SimulatedUdf,
    expected_rows: u64,
) -> u64 {
    let check = check_visible_outputs(dataset, outputs, udf, None).await;
    assert_eq!(check.rows, expected_rows, "rows after the refresh");
    for (index, output) in outputs.iter().enumerate() {
        assert_eq!(
            check.true_rows[index],
            expected_rows,
            "{} has pending rows after the refresh completed",
            output.column.name()
        );
        assert_eq!(
            check.stale[index],
            0,
            "{} shows values computed from old inputs after the refresh completed",
            output.column.name()
        );
    }
    check.rows
}
