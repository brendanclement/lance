// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Read-time masking for cell flags registered with `mask_when_false`: the
//! flag's field reads as NULL wherever the flag is false.
//!
//! [`FragmentReader`](super::FragmentReader) masks physical rows right after
//! merging overlays, before deletions, filters, aggregates or projections see
//! a value. Internal writers that re-read rows through it (`UpdateJob`,
//! `merge_insert` rewriting unmatched rows, `add_columns`, copy-through with
//! `read_physical_slice`) therefore write masked cells back as NULL. That is
//! safe because only dependent flags mask: a false flag means the cell has no
//! value, and only a publication, which writes new values, sets it true.

use std::sync::Arc;

use arrow_array::{Array, BooleanArray, RecordBatch, new_null_array};
use arrow_buffer::{BooleanBuffer, BooleanBufferBuilder};
use arrow_select::nullif::nullif;
use futures::{FutureExt, StreamExt};
use lance_core::datatypes::Schema;
use lance_core::utils::deletion::DeletionVector;
use lance_core::{Error, Result};
use lance_file::version::ConcreteFileVersion;
use lance_select::{RowAddrSelection, RowAddrTreeMap};
use lance_table::utils::stream::{ReadBatchTask, ReadBatchTaskStream};
use roaring::RoaringBitmap;

use crate::Dataset;

#[derive(Debug)]
enum ColumnMask {
    /// The flag is false on every row of the fragment.
    AllMasked,
    /// The flag's true rows, which include some but not all of this fragment's.
    /// Shared with the manifest rather than copied on every open.
    Partial(Arc<RowAddrTreeMap>),
}

#[derive(Debug)]
struct MaskedColumn {
    name: String,
    mask: ColumnMask,
    /// Only deleted physical slots are false. Readers retaining deleted
    /// slots must still apply this mask.
    is_live_valid: bool,
}

/// The projected fields of one fragment that a masking cell flag hides on at
/// least one row. A field whose flag is true on every row is left out.
#[derive(Debug)]
pub(super) struct CellFlagMasks {
    fragment_id: u32,
    columns: Vec<MaskedColumn>,
}

impl CellFlagMasks {
    /// The masks `fragment_id` needs for the top-level fields of `projection`,
    /// or `None` when every projected cell reads its stored value.
    pub(super) fn resolve(
        dataset: &Dataset,
        fragment_id: u64,
        projection: &Schema,
    ) -> Result<Option<Self>> {
        let Some(registry) = dataset.manifest.cell_flags.as_deref() else {
            return Ok(None);
        };
        reject_masked_v1_read(dataset, fragment_id, projection)?;
        let fragment_id = u32::try_from(fragment_id).map_err(|_| {
            Error::internal(format!(
                "fragment id {fragment_id} exceeds the u32 range cell flag state is keyed by"
            ))
        })?;
        let mut columns = Vec::new();
        for field in &projection.fields {
            let Some(flag) = registry.masking_flag(field.id) else {
                continue;
            };
            let state = registry.states().get(&flag.flag_id);
            let mask = match state.and_then(|state| Some((state, state.get(&fragment_id)?))) {
                Some((_, RowAddrSelection::Full)) => continue,
                Some((state, RowAddrSelection::Partial(_))) => ColumnMask::Partial(state.clone()),
                None => ColumnMask::AllMasked,
            };
            columns.push(MaskedColumn {
                name: field.name.clone(),
                mask,
                is_live_valid: false,
            });
        }
        Ok((!columns.is_empty()).then_some(Self {
            fragment_id,
            columns,
        }))
    }

    pub(super) fn with_live_rows(
        mut self,
        physical_rows: usize,
        deleted: Option<&DeletionVector>,
    ) -> Result<Arc<Self>> {
        let physical_rows = u32::try_from(physical_rows).map_err(|_| {
            Error::internal(format!(
                "fragment {} has {physical_rows} physical rows, outside cell flag offset bounds",
                self.fragment_id
            ))
        })?;
        for column in &mut self.columns {
            if let ColumnMask::Partial(state) = &column.mask
                && let Some(RowAddrSelection::Partial(rows)) = state.get(&self.fragment_id)
            {
                column.is_live_valid = covers_live_rows(rows, physical_rows, deleted);
            }
        }
        Ok(Arc::new(self))
    }

    /// Fields that can be supplied by a NULL reader instead of decoded.
    /// Partial masks still require the stored values of their true rows.
    pub(super) fn all_masked_projection(&self, projection: &Schema) -> Option<Schema> {
        if !self
            .columns
            .iter()
            .any(|column| matches!(column.mask, ColumnMask::AllMasked))
        {
            return None;
        }
        let mut masked = projection.clone();
        masked.fields.retain(|field| {
            self.columns.iter().any(|column| {
                column.name == field.name && matches!(column.mask, ColumnMask::AllMasked)
            })
        });
        Some(masked)
    }

    pub(super) fn is_needed(&self, include_deleted: bool) -> bool {
        self.columns
            .iter()
            .any(|column| include_deleted || !column.is_live_valid)
    }

    /// Only a partially masked field needs each row's physical offset.
    pub(super) fn needs_offsets(&self, include_deleted: bool) -> bool {
        self.columns.iter().any(|column| {
            (include_deleted || !column.is_live_valid)
                && matches!(column.mask, ColumnMask::Partial(_))
        })
    }

    /// Null the masked cells of every batch of `stream`, which yields physical
    /// rows in the order of `offsets_in_frag`. The offsets are required when
    /// [`Self::needs_offsets`].
    pub(super) fn apply(
        self: Arc<Self>,
        stream: ReadBatchTaskStream,
        offsets_in_frag: Option<Arc<Vec<u32>>>,
        include_deleted: bool,
    ) -> ReadBatchTaskStream {
        let mut rows_seen = 0usize;
        stream
            .map(move |task| {
                let num_rows = task.num_rows as usize;
                let start = rows_seen;
                rows_seen += num_rows;
                let masks = self.clone();
                let offsets_in_frag = offsets_in_frag.clone();
                let inner = task.task;
                ReadBatchTask {
                    num_rows: task.num_rows,
                    task: async move {
                        let batch = inner.await?;
                        let batch_offsets = offsets_in_frag
                            .as_deref()
                            .map(|offsets| {
                                offsets.get(start..start + num_rows).ok_or_else(|| {
                                    Error::internal(format!(
                                        "fragment {} read yielded rows {start}..{} but planned \
                                         only {} physical offsets",
                                        masks.fragment_id,
                                        start + num_rows,
                                        offsets.len()
                                    ))
                                })
                            })
                            .transpose()?;
                        masks.mask_batch(batch, batch_offsets, include_deleted)
                    }
                    .boxed(),
                }
            })
            .boxed()
    }

    fn mask_batch(
        &self,
        batch: RecordBatch,
        batch_offsets: Option<&[u32]>,
        include_deleted: bool,
    ) -> Result<RecordBatch> {
        let schema = batch.schema();
        let num_rows = batch.num_rows();
        let mut columns = batch.columns().to_vec();
        for masked in &self.columns {
            if !include_deleted && masked.is_live_valid {
                continue;
            }
            let index = schema.index_of(&masked.name).map_err(|_| {
                Error::internal(format!(
                    "cannot mask field '{}' of fragment {}: the batch read has no such column \
                     (it has {:?})",
                    masked.name,
                    self.fragment_id,
                    schema.fields().iter().map(|f| f.name()).collect::<Vec<_>>()
                ))
            })?;
            columns[index] = match &masked.mask {
                ColumnMask::AllMasked if columns[index].null_count() == num_rows => continue,
                ColumnMask::AllMasked => new_null_array(columns[index].data_type(), num_rows),
                ColumnMask::Partial(state) => {
                    let Some(RowAddrSelection::Partial(true_rows)) = state.get(&self.fragment_id)
                    else {
                        return Err(Error::internal(format!(
                            "cannot mask field '{}' of fragment {}: its cell flag state no \
                             longer holds a partial selection for the fragment",
                            masked.name, self.fragment_id
                        )));
                    };
                    let offsets = batch_offsets
                        .filter(|offsets| offsets.len() == num_rows)
                        .ok_or_else(|| {
                            Error::internal(format!(
                                "cannot mask field '{}' of fragment {}: a batch of {num_rows} \
                                 rows came with {:?} physical offsets",
                                masked.name,
                                self.fragment_id,
                                batch_offsets.map(<[u32]>::len)
                            ))
                        })?;
                    // Flags change in runs of rows, so most batches fall
                    // entirely inside a run and skip the per-row lookups.
                    let (Some(&first), Some(&last)) = (offsets.iter().min(), offsets.iter().max())
                    else {
                        continue;
                    };
                    let true_in_span = true_rows.range_cardinality(first..=last);
                    if true_in_span == u64::from(last - first) + 1 {
                        continue;
                    }
                    if true_in_span == 0 {
                        new_null_array(columns[index].data_type(), num_rows)
                    } else {
                        let is_masked = if is_contiguous(offsets) {
                            masked_in_span(true_rows, first, last)
                        } else {
                            BooleanBuffer::collect_bool(num_rows, |row| {
                                !true_rows.contains(offsets[row])
                            })
                        };
                        if is_masked.count_set_bits() == 0 {
                            continue;
                        }
                        nullif(columns[index].as_ref(), &BooleanArray::new(is_masked, None))?
                    }
                }
            };
        }
        Ok(RecordBatch::try_new(schema, columns)?)
    }
}

/// Count coverage inside physical bounds, then check only the deleted holes.
/// Equal true/live cardinalities alone are insufficient: a true deleted row
/// could hide a false live row. Physical copy-through views have no deletions,
/// so their false slots never qualify for this fast path.
fn covers_live_rows(
    true_rows: &RoaringBitmap,
    physical_rows: u32,
    deleted: Option<&DeletionVector>,
) -> bool {
    let holes = u64::from(physical_rows) - true_rows.range_cardinality(0..physical_rows);
    if holes == 0 {
        return true;
    }
    let Some(deleted) = deleted else {
        return false;
    };
    if holes > deleted.len() as u64 {
        return false;
    }
    deleted
        .iter()
        .filter(|row| *row < physical_rows && !true_rows.contains(*row))
        .count() as u64
        == holes
}

fn is_contiguous(offsets: &[u32]) -> bool {
    offsets
        .windows(2)
        .all(|pair| pair[0].checked_add(1) == Some(pair[1]))
}

/// Which rows of `first..=last` are masked, built from the runs of true rows
/// in the span: a per-row `contains` probe dominated scans of partly true
/// fragments.
fn masked_in_span(true_rows: &RoaringBitmap, first: u32, last: u32) -> BooleanBuffer {
    let mut masked = BooleanBufferBuilder::new((last - first) as usize + 1);
    // The run of true rows not yet appended: `run_start..run_end`.
    let (mut run_start, mut run_end) = (first, first);
    for row in true_rows.range(first..=last) {
        if row != run_end {
            masked.append_n((run_end - run_start) as usize, false);
            masked.append_n((row - run_end) as usize, true);
            run_start = row;
        }
        run_end = row + 1;
    }
    masked.append_n((run_end - run_start) as usize, false);
    masked.append_n((last + 1 - run_end) as usize, true);
    masked.finish()
}

/// Registration refuses masking on legacy (v1) storage, whose pushdown scan
/// answers filters from page statistics of the stored values. Refuse reads of
/// a masked field there too, should one ever appear.
pub(super) fn reject_masked_v1_read(
    dataset: &Dataset,
    fragment_id: u64,
    projection: &Schema,
) -> Result<()> {
    if dataset.manifest.data_storage_format.lance_file_format() != ConcreteFileVersion::V1 {
        return Ok(());
    }
    let Some(registry) = dataset.manifest.cell_flags.as_deref() else {
        return Ok(());
    };
    let Some((field, flag)) = projection.fields.iter().find_map(|field| {
        registry
            .masking_flag(field.id)
            .map(|flag| (field.name.as_str(), flag))
    }) else {
        return Ok(());
    };
    Err(Error::not_supported(format!(
        "cannot read field '{field}' of fragment {fragment_id}: cell flag '{}' (flag id {}) \
         masks it, and the legacy (v1) storage format cannot mask cells",
        flag.name, flag.flag_id
    )))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::full(&[0, 1, 2, 3], &[], true)]
    #[case::deleted_hole(&[0, 2, 3], &[1], true)]
    #[case::true_deleted_row_and_false_live_row(&[0, 2, 3], &[0], false)]
    #[case::one_live_hole(&[0, 3], &[1], false)]
    #[case::physical_copy_view(&[0, 2, 3], &[], false)]
    #[case::outside_true_offset(&[0, 2, 3, 99], &[1], true)]
    #[case::outside_deleted_offset(&[0, 2, 3], &[99], false)]
    fn live_coverage_requires_every_live_physical_offset(
        #[case] true_offsets: &[u32],
        #[case] deleted_offsets: &[u32],
        #[case] expected: bool,
        #[values(false, true)] use_bitmap: bool,
    ) {
        let rows = true_offsets.iter().copied().collect();
        let deleted = if use_bitmap {
            DeletionVector::Bitmap(deleted_offsets.iter().copied().collect())
        } else {
            DeletionVector::Set(deleted_offsets.iter().copied().collect())
        };
        assert_eq!(covers_live_rows(&rows, 4, Some(&deleted)), expected);
    }

    #[rstest]
    #[case::all_true(0..100, 0, 99)]
    #[case::leading_and_trailing_gaps(10..20, 0, 30)]
    #[case::span_inside_a_run(0..100, 40, 60)]
    #[case::single_true_row(200..300, 0, 99)]
    fn masked_in_span_matches_per_row_probe(
        #[case] true_range: std::ops::Range<u32>,
        #[case] first: u32,
        #[case] last: u32,
    ) {
        let mut true_rows: RoaringBitmap = true_range.collect();
        // Scattered holes and isolated true rows, so runs start and end
        // inside the span.
        for row in (3..300).step_by(7) {
            true_rows.remove(row);
        }
        true_rows.insert(first + 1);
        let expected = BooleanBuffer::collect_bool((last - first + 1) as usize, |row| {
            !true_rows.contains(first + row as u32)
        });
        assert_eq!(masked_in_span(&true_rows, first, last), expected);
    }

    #[test]
    fn contiguity_needs_consecutive_ascending_offsets() {
        assert!(is_contiguous(&[]));
        assert!(is_contiguous(&[5]));
        assert!(is_contiguous(&[5, 6, 7]));
        assert!(!is_contiguous(&[5, 7, 8]));
        assert!(!is_contiguous(&[6, 5]));
        assert!(!is_contiguous(&[5, 5]));
    }
}
