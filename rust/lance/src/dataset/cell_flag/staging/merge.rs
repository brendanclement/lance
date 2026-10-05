// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Merging the computed rows of one fragment into its copy-through read.
//!
//! The copy read yields every physical row of the fragment in windows, and
//! each window becomes one staged batch. For each output, a window takes the
//! copied column when no computed row assigns it there, a zero-copy slice of
//! the computed values when one run assigns every row of the window, and
//! otherwise interleaves both, placing each computed value at its offset.

use std::collections::VecDeque;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_array::{Array, ArrayRef, RecordBatch};
use arrow_buffer::{BooleanBuffer, ScalarBuffer};
use arrow_schema::SchemaRef;
use arrow_select::interleave::interleave;
use lance_arrow::RecordBatchExt;
use lance_core::ROW_ADDR;
use lance_core::datatypes::Schema;
use lance_core::utils::address::RowAddress;
use roaring::RoaringBitmap;

use super::logical_type_mismatch;
use crate::{Error, Result};

/// The computed values of one output on a run of rows.
#[derive(Debug, Clone)]
pub(super) struct Assignment {
    /// Conformed to the output's field with nullability relaxed.
    pub(super) values: ArrayRef,
    /// The rows it assigns, or `None` for every row.
    pub(super) mask: Option<BooleanBuffer>,
}

impl Assignment {
    pub(super) fn is_assigned(&self, row: usize) -> bool {
        self.mask.as_ref().is_none_or(|mask| mask.value(row))
    }

    fn slice(&self, offset: usize, len: usize) -> Self {
        Self {
            values: self.values.slice(offset, len),
            mask: self.mask.as_ref().map(|mask| mask.slice(offset, len)),
        }
    }
}

/// Computed rows of one fragment, with strictly ascending offsets once
/// validated.
#[derive(Debug, Clone)]
pub(super) struct Run {
    pub(super) fragment_id: u32,
    pub(super) addrs: ScalarBuffer<u64>,
    /// One entry per output of the publication, `None` where the run assigns
    /// none of its cells.
    pub(super) outputs: Vec<Option<Assignment>>,
}

impl Run {
    pub(super) fn slice_of(
        fragment_id: u32,
        addrs: &ScalarBuffer<u64>,
        outputs: &[Option<Assignment>],
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            fragment_id,
            addrs: addrs.slice(offset, len),
            outputs: outputs
                .iter()
                .map(|assignment| {
                    assignment
                        .as_ref()
                        .map(|assignment| assignment.slice(offset, len))
                })
                .collect(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.addrs.len()
    }

    pub(super) fn offset(&self, row: usize) -> u32 {
        RowAddress::from(self.addrs[row]).row_offset()
    }

    /// Whether `row` assigns any output.
    pub(super) fn assigns(&self, row: usize) -> bool {
        self.outputs
            .iter()
            .flatten()
            .any(|assignment| assignment.is_assigned(row))
    }

    pub(super) fn assigns_any(&self) -> bool {
        self.len() > 0
            && self.outputs.iter().flatten().any(|assignment| {
                assignment
                    .mask
                    .as_ref()
                    .is_none_or(|mask| mask.count_set_bits() > 0)
            })
    }

    /// The number of leading rows whose offset is below `end`.
    fn rows_below(&self, end: u64) -> usize {
        self.addrs
            .partition_point(|addr| u64::from(RowAddress::from(*addr).row_offset()) < end)
    }
}

/// The merge of one fragment: pushed computed rows wait until the copy window
/// that holds their offsets arrives.
#[derive(Debug)]
pub(super) struct FragmentMerge {
    fragment_id: u32,
    physical_rows: u32,
    /// The outputs as the manifest defines them.
    output_schema: Arc<Schema>,
    /// The schema of every merged batch: the outputs with nullability relaxed.
    relaxed_schema: SchemaRef,
    /// The offset of the next copy window.
    next_offset: u32,
    is_type_checked: bool,
    pending: VecDeque<Run>,
    last_pushed: Option<u32>,
    /// Per output, the offsets merged from computed rows.
    assigned: Vec<RoaringBitmap>,
}

impl FragmentMerge {
    pub(super) fn new(
        fragment_id: u32,
        physical_rows: u32,
        output_schema: Arc<Schema>,
        relaxed_schema: SchemaRef,
    ) -> Self {
        let assigned = vec![RoaringBitmap::new(); output_schema.fields.len()];
        Self {
            fragment_id,
            physical_rows,
            output_schema,
            relaxed_schema,
            next_offset: 0,
            is_type_checked: false,
            pending: VecDeque::new(),
            last_pushed: None,
            assigned,
        }
    }

    /// Whether computed rows inside the next copy window, of `len` rows, may
    /// not have been pushed yet.
    pub(super) fn needs_rows_for(&self, len: usize) -> bool {
        let end = u64::from(self.next_offset) + len as u64;
        self.last_pushed
            .is_none_or(|last| u64::from(last) + 1 < end)
    }

    /// Queue `run`, whose offsets follow every row pushed before it.
    pub(super) fn push(&mut self, run: Run) {
        if let Some(last) = run.len().checked_sub(1) {
            self.last_pushed = Some(run.offset(last));
            self.pending.push_back(run);
        }
    }

    /// Merge the pending computed rows into the copy window `copy`, the next
    /// rows of the fragment's copy-through read with their `_rowaddr`.
    pub(super) fn merge(&mut self, copy: RecordBatch) -> Result<RecordBatch> {
        let len = copy.num_rows();
        let start = self.next_offset;
        let end = u64::from(start) + len as u64;
        if end > u64::from(self.physical_rows) {
            return Err(Error::internal(format!(
                "copy-through of fragment {} returned {len} rows at offset {start}, past its {} \
                 physical rows",
                self.fragment_id, self.physical_rows
            )));
        }
        self.check_alignment(&copy)?;
        if !self.is_type_checked {
            self.check_copied_types(&copy)?;
            self.is_type_checked = true;
        }
        let copied = copy.project_by_schema(&self.relaxed_schema)?;

        // The window's computed rows are a prefix of the pending ones, whose
        // earlier windows were trimmed away.
        let mut window = Vec::new();
        for (index, run) in self.pending.iter().enumerate() {
            let rows = run.rows_below(end);
            if rows == 0 {
                break;
            }
            window.push((index, rows));
            if rows < run.len() {
                break;
            }
        }
        let mut columns = Vec::with_capacity(copied.num_columns());
        for (output, (copied, assigned)) in copied
            .columns()
            .iter()
            .zip(self.assigned.iter_mut())
            .enumerate()
        {
            columns.push(merge_column(
                self.fragment_id,
                &self.pending,
                &window,
                output,
                copied,
                start,
                assigned,
            )?);
        }
        self.trim(end);
        // `end` is at most `physical_rows`, a u32.
        self.next_offset = start + len as u32;
        Ok(RecordBatch::try_new(self.relaxed_schema.clone(), columns)?)
    }

    /// Check that the copy read covered the whole fragment and left no
    /// computed row unmerged.
    pub(super) fn check_complete(&self) -> Result<()> {
        if self.next_offset != self.physical_rows {
            return Err(Error::internal(format!(
                "copy-through of fragment {} ended after {} of {} physical rows",
                self.fragment_id, self.next_offset, self.physical_rows
            )));
        }
        if let Some(run) = self.pending.front() {
            return Err(Error::internal(format!(
                "computed row {:#x} of fragment {} lies past its {} physical rows",
                run.addrs[0], self.fragment_id, self.physical_rows
            )));
        }
        Ok(())
    }

    /// Per output, the offsets merged from computed rows.
    pub(super) fn into_assigned(self) -> Vec<RoaringBitmap> {
        self.assigned
    }

    fn check_alignment(&self, copy: &RecordBatch) -> Result<()> {
        let addrs = copy
            .column_by_name(ROW_ADDR)
            .and_then(|addrs| addrs.as_primitive_opt::<UInt64Type>())
            .ok_or_else(|| {
                Error::internal(format!(
                    "copy-through of fragment {} returned no UInt64 '{ROW_ADDR}' column",
                    self.fragment_id
                ))
            })?;
        for (row, addr) in addrs.iter().enumerate() {
            // `merge` bounds the window by `physical_rows`, a u32.
            let expected = u64::from(RowAddress::new_from_parts(
                self.fragment_id,
                self.next_offset + row as u32,
            ));
            if addr != Some(expected) {
                let returned = addr.map_or_else(|| "NULL".to_string(), |addr| format!("{addr:#x}"));
                return Err(Error::internal(format!(
                    "copy-through of fragment {} returned row address {returned} where \
                     {expected:#x} was expected",
                    self.fragment_id
                )));
            }
        }
        Ok(())
    }

    /// Computed values are checked against the manifest types, so copied ones
    /// must read back as those types too. Nullability is relaxed rather than
    /// compared: scans keep the manifest's nested nullability.
    fn check_copied_types(&self, copy: &RecordBatch) -> Result<()> {
        let copy_schema = copy.schema();
        for field in &self.output_schema.fields {
            let copied = copy_schema.field_with_name(&field.name).map_err(|_| {
                Error::internal(format!(
                    "copy-through of fragment {} returned no column '{}'",
                    self.fragment_id, field.name
                ))
            })?;
            // Unreachable: the stager refuses the outputs a scan converts,
            // blob and JSON.
            if logical_type_mismatch(copied, field).is_some() {
                return Err(Error::internal(format!(
                    "output '{}' reads back from fragment {} as {}, not as its field type {}; \
                     publication staging does not cast",
                    field.name,
                    self.fragment_id,
                    copied.data_type(),
                    field.data_type()
                )));
            }
        }
        Ok(())
    }

    fn trim(&mut self, end: u64) {
        while let Some(front) = self.pending.front_mut() {
            let rows = front.rows_below(end);
            if rows == front.len() {
                self.pending.pop_front();
                continue;
            }
            if rows > 0 {
                *front = Run::slice_of(
                    front.fragment_id,
                    &front.addrs,
                    &front.outputs,
                    rows,
                    front.len() - rows,
                );
            }
            break;
        }
    }
}

/// One output's column of a window starting at offset `start`: `copied`
/// where no computed row of `window`, `(pending index, rows in the window)`,
/// assigns it. Records the assigned offsets in `assigned`.
fn merge_column(
    fragment_id: u32,
    pending: &VecDeque<Run>,
    window: &[(usize, usize)],
    output: usize,
    copied: &ArrayRef,
    start: u32,
    assigned: &mut RoaringBitmap,
) -> Result<ArrayRef> {
    let len = copied.len();
    if let [(index, rows)] = window
        && *rows == len
        && len > 0
    {
        let run = &pending[*index];
        // Computed-row validation established strict ordering. Matching the
        // window's bounds and row count therefore proves contiguous coverage.
        // Avoid building per-row interleave indices just to return a slice.
        if run.offset(0) == start
            && u64::from(run.offset(len - 1)) + 1 == u64::from(start) + len as u64
            && let Some(assignment) = &run.outputs[output]
            && assignment
                .mask
                .as_ref()
                .is_none_or(|mask| mask.slice(0, len).count_set_bits() == len)
        {
            if assigned.max().is_some_and(|previous| previous >= start) {
                return Err(Error::internal(format!(
                    "computed offset {start} of fragment {fragment_id} does not follow the \
                     offsets merged before it"
                )));
            }
            // `merge` bounds the window by the fragment's u32 physical count.
            assigned.insert_range(start..start + len as u32);
            return Ok(assignment.values.slice(0, len));
        }
    }
    let mut sources: Vec<&ArrayRef> = vec![copied];
    let mut indices: Vec<(usize, usize)> = Vec::new();
    let mut first_hit = None;
    let mut hits = 0;
    for &(index, rows) in window {
        let run = &pending[index];
        let Some(assignment) = &run.outputs[output] else {
            continue;
        };
        let mut source = None;
        for row in (0..rows).filter(|row| assignment.is_assigned(*row)) {
            let offset = run.offset(row);
            let source = *source.get_or_insert_with(|| {
                sources.push(&assignment.values);
                sources.len() - 1
            });
            if indices.is_empty() {
                indices = (0..len).map(|position| (0, position)).collect();
            }
            indices[(offset - start) as usize] = (source, row);
            first_hit.get_or_insert(row);
            hits += 1;
            assigned.try_push(offset).map_err(|_| {
                Error::internal(format!(
                    "computed offset {offset} of fragment {fragment_id} does not follow the \
                     offsets merged before it"
                ))
            })?;
        }
    }
    match first_hit {
        None => Ok(copied.clone()),
        Some(row) if hits == len && sources.len() == 2 => Ok(sources[1].slice(row, len)),
        Some(_) => {
            let arrays: Vec<&dyn Array> = sources.iter().map(|array| array.as_ref()).collect();
            Ok(interleave(&arrays, &indices)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::builder::{ListBuilder, StringBuilder};
    use arrow_array::{Int32Array, ListArray, StringArray, StructArray, UInt64Array};
    use arrow_schema::{DataType, Field as ArrowField, Schema as ArrowSchema};
    use rstest::rstest;

    use super::*;
    use crate::dataset::fragment::relax_nullability;

    const FRAGMENT: u32 = 3;

    fn addr(offset: u32) -> u64 {
        RowAddress::new_from_parts(FRAGMENT, offset).into()
    }

    /// A merge of the nullable Utf8 outputs `a` and `b`.
    fn text_merge(physical_rows: u32) -> FragmentMerge {
        let schema = ArrowSchema::new(vec![
            ArrowField::new("a", DataType::Utf8, true),
            ArrowField::new("b", DataType::Utf8, true),
        ]);
        FragmentMerge::new(
            FRAGMENT,
            physical_rows,
            Arc::new(Schema::try_from(&schema).unwrap()),
            Arc::new(schema),
        )
    }

    /// The copy window at `start`: `copied-{output}-{offset}` for each row.
    fn text_window(start: u32, len: u32) -> RecordBatch {
        let offsets = start..start + len;
        let column = |output: &str| -> ArrayRef {
            Arc::new(StringArray::from_iter_values(
                offsets
                    .clone()
                    .map(|offset| format!("copied-{output}-{offset}")),
            ))
        };
        RecordBatch::try_from_iter([
            ("a", column("a")),
            ("b", column("b")),
            (
                ROW_ADDR,
                Arc::new(UInt64Array::from_iter_values(offsets.clone().map(addr))) as ArrayRef,
            ),
        ])
        .unwrap()
    }

    /// `window` with its `_rowaddr` replaced by `addrs`, NULLs allowed.
    fn with_row_addrs(window: RecordBatch, addrs: Vec<Option<u64>>) -> RecordBatch {
        let index = window.schema().index_of(ROW_ADDR).unwrap();
        let mut fields = window.schema().fields().to_vec();
        fields[index] = Arc::new(ArrowField::new(ROW_ADDR, DataType::UInt64, true));
        let mut columns = window.columns().to_vec();
        columns[index] = Arc::new(UInt64Array::from(addrs));
        RecordBatch::try_new(Arc::new(ArrowSchema::new(fields)), columns).unwrap()
    }

    /// What `output` computes at `offset`; every third row is a computed NULL.
    fn computed(output: &str, offset: u32) -> Option<String> {
        (offset % 3 != 2).then(|| format!("computed-{output}-{offset}"))
    }

    /// A run on `offsets` carrying each output of `masks` (`None`: not
    /// carried), with its mask over the run's rows (`None`: every row).
    fn text_run(offsets: &[u32], masks: [Option<Option<Vec<bool>>>; 2]) -> Run {
        let outputs = ["a", "b"]
            .into_iter()
            .zip(masks)
            .map(|(output, mask)| {
                mask.map(|mask| Assignment {
                    values: Arc::new(StringArray::from_iter(
                        offsets.iter().map(|offset| computed(output, *offset)),
                    )) as ArrayRef,
                    mask: mask.map(BooleanBuffer::from),
                })
            })
            .collect();
        Run {
            fragment_id: FRAGMENT,
            addrs: offsets.iter().copied().map(addr).collect(),
            outputs,
        }
    }

    /// Drive `merge` as staging does: before each window, push runs until
    /// the window's rows are in.
    fn merge_all(
        mut merge: FragmentMerge,
        windows: &[u32],
        runs: Vec<Run>,
    ) -> (Vec<RecordBatch>, Vec<RoaringBitmap>) {
        let mut runs = runs.into_iter();
        let mut merged = Vec::new();
        let mut start = 0;
        for len in windows {
            while merge.needs_rows_for(*len as usize) {
                let Some(run) = runs.next() else { break };
                merge.push(run);
            }
            let batch = merge.merge(text_window(start, *len)).unwrap();
            assert_eq!(batch.num_rows(), *len as usize, "window at {start}");
            merged.push(batch);
            start += len;
        }
        merge.check_complete().unwrap();
        (merged, merge.into_assigned())
    }

    fn texts(batches: &[RecordBatch], column: &str) -> Vec<Option<String>> {
        batches
            .iter()
            .flat_map(|batch| {
                let values = batch[column].as_string::<i32>();
                (0..values.len())
                    .map(|row| values.is_valid(row).then(|| values.value(row).to_string()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[rstest]
    #[case::run_spans_windows(6, &[2, 2, 2], vec![text_run(&[0, 1, 2, 3, 4, 5], [Some(None), None])])]
    #[case::window_spans_runs(
        6,
        &[6],
        vec![
            text_run(&[0, 1], [Some(None), Some(None)]),
            text_run(&[2, 3], [Some(None), None]),
            text_run(&[4, 5], [None, Some(None)]),
        ]
    )]
    #[case::sparse_rows(
        8,
        &[3, 3, 2],
        vec![
            text_run(&[1, 4], [Some(None), Some(Some(vec![false, true]))]),
            text_run(&[6], [None, Some(None)]),
        ]
    )]
    #[case::first_and_last_offset(6, &[4, 2], vec![text_run(&[0, 5], [Some(None), Some(None)])])]
    #[case::masked_rows_straddle_windows(
        6,
        &[2, 2, 2],
        vec![text_run(
            &[0, 1, 2, 3, 4, 5],
            [Some(Some(vec![true, false, false, true, true, false])), Some(Some(vec![false; 6]))],
        )]
    )]
    #[case::one_assigned_one_copied(4, &[4], vec![text_run(&[0, 1, 2, 3], [None, Some(None)])])]
    #[case::fully_assigned_prefixes_in_masked_runs(
        4,
        &[2, 2],
        vec![text_run(
            &[0, 1, 2, 3],
            [Some(Some(vec![true, true, false, true])), Some(Some(vec![false, false, true, true]))],
        )]
    )]
    fn merge_places_values_by_offset(
        #[case] physical_rows: u32,
        #[case] windows: &[u32],
        #[case] runs: Vec<Run>,
    ) {
        // Row by row: a computed value wherever a run assigns the output.
        let mut expected: Vec<Vec<Option<String>>> = ["a", "b"]
            .iter()
            .map(|output| {
                (0..physical_rows)
                    .map(|offset| Some(format!("copied-{output}-{offset}")))
                    .collect()
            })
            .collect();
        let mut expected_assigned = vec![RoaringBitmap::new(); 2];
        for run in &runs {
            for (output, assignment) in run.outputs.iter().enumerate() {
                let Some(assignment) = assignment else {
                    continue;
                };
                for row in (0..run.len()).filter(|row| assignment.is_assigned(*row)) {
                    let offset = run.offset(row);
                    expected[output][offset as usize] = computed(["a", "b"][output], offset);
                    expected_assigned[output].insert(offset);
                }
            }
        }

        let (merged, assigned) = merge_all(text_merge(physical_rows), windows, runs);
        assert_eq!(texts(&merged, "a"), expected[0]);
        assert_eq!(texts(&merged, "b"), expected[1]);
        assert_eq!(assigned, expected_assigned);
    }

    #[test]
    fn merge_takes_whole_windows_without_copying() {
        let run = text_run(&[0, 1, 2, 3], [Some(None), None]);
        let computed_a = run.outputs[0].as_ref().unwrap().values.clone();
        let (merged, _) = merge_all(text_merge(4), &[2, 2], vec![run]);
        let values_ptr = |array: &ArrayRef| array.as_string::<i32>().values().as_ptr();
        for batch in &merged {
            assert_eq!(values_ptr(&batch["a"]), values_ptr(&computed_a));
        }
    }

    #[rstest]
    #[case::skipped_offset(
        text_window(0, 2).slice(1, 1),
        format!("returned row address {:#x} where {:#x} was expected", addr(1), addr(0))
    )]
    #[case::other_fragment(
        {
            let window = text_window(0, 1);
            let other = u64::from(RowAddress::new_from_parts(FRAGMENT + 1, 0));
            window
                .replace_column_by_name(ROW_ADDR, Arc::new(UInt64Array::from(vec![other])))
                .unwrap()
        },
        format!(
            "returned row address {:#x} where {:#x} was expected",
            u64::from(RowAddress::new_from_parts(FRAGMENT + 1, 0)),
            addr(0)
        )
    )]
    #[case::reordered_after_first_row(
        with_row_addrs(text_window(0, 3), vec![Some(addr(0)), Some(addr(2)), Some(addr(1))]),
        format!("returned row address {:#x} where {:#x} was expected", addr(2), addr(1))
    )]
    #[case::repeated_row(
        with_row_addrs(text_window(0, 3), vec![Some(addr(0)), Some(addr(0)), Some(addr(1))]),
        format!("returned row address {:#x} where {:#x} was expected", addr(0), addr(1))
    )]
    #[case::null_after_first_row(
        with_row_addrs(text_window(0, 3), vec![Some(addr(0)), None, Some(addr(2))]),
        format!("returned row address NULL where {:#x} was expected", addr(1))
    )]
    #[case::past_physical_rows(text_window(0, 5), "returned 5 rows at offset 0, past its 4 physical rows".to_string())]
    #[case::copied_type_mismatch(
        text_window(0, 1)
            .replace_column_schema_by_name(
                "a",
                DataType::Int32,
                Arc::new(Int32Array::from(vec![1])),
            )
            .unwrap(),
        format!(
            "output 'a' reads back from fragment {FRAGMENT} as Int32, not as its field type Utf8"
        )
    )]
    fn merge_detects_invalid_copy_reads(#[case] window: RecordBatch, #[case] expected: String) {
        let mut merge = text_merge(4);
        merge.push(text_run(&[0], [Some(None), None]));
        let error = merge.merge(window).unwrap_err();
        assert!(matches!(error, Error::Internal { .. }), "{error}");
        assert!(error.to_string().contains(&expected), "{error}");
    }

    #[test]
    fn merge_detects_a_copy_read_that_ends_early() {
        let mut merge = text_merge(4);
        merge.push(text_run(&[3], [Some(None), None]));
        merge.merge(text_window(0, 2)).unwrap();
        let error = merge.check_complete().unwrap_err();
        assert!(matches!(error, Error::Internal { .. }), "{error}");
        assert!(
            error.to_string().contains(&format!(
                "copy-through of fragment {FRAGMENT} ended after 2 of 4 physical rows"
            )),
            "{error}"
        );
    }

    /// Scans keep the manifest's nested nullability while staged batches
    /// relax it, so copied and computed values meet under the relaxed type.
    #[test]
    fn merge_conforms_nested_nullability() {
        let item = Arc::new(ArrowField::new("item", DataType::Utf8, false));
        let child = ArrowField::new("x", DataType::Utf8, false);
        let manifest = ArrowSchema::new(vec![
            ArrowField::new("tags", DataType::List(item.clone()), true),
            ArrowField::new("info", DataType::Struct(vec![child.clone()].into()), true),
        ]);
        let relaxed = Arc::new(ArrowSchema::new(
            manifest
                .fields()
                .iter()
                .map(|field| relax_nullability(field))
                .collect::<Vec<_>>(),
        ));
        let tags = |values: &[&str], item: Arc<ArrowField>| -> ArrayRef {
            let mut builder = ListBuilder::new(StringBuilder::new()).with_field(item);
            for value in values {
                builder.append_value([Some(*value)]);
            }
            Arc::new(builder.finish())
        };
        let info = |values: &[&str], child: ArrowField| -> ArrayRef {
            Arc::new(StructArray::from(vec![(
                Arc::new(child),
                Arc::new(StringArray::from_iter_values(values.iter().copied())) as ArrayRef,
            )]))
        };
        let copy = RecordBatch::try_new(
            Arc::new(ArrowSchema::new(vec![
                manifest.field(0).clone(),
                manifest.field(1).clone(),
                ArrowField::new(ROW_ADDR, DataType::UInt64, true),
            ])),
            vec![
                tags(&["c0", "c1"], item),
                info(&["c0", "c1"], child),
                Arc::new(UInt64Array::from(vec![addr(0), addr(1)])),
            ],
        )
        .unwrap();
        let relaxed_item = match relaxed.field(0).data_type() {
            DataType::List(item) => item.clone(),
            other => panic!("tags relaxed to {other}"),
        };
        let relaxed_child = match relaxed.field(1).data_type() {
            DataType::Struct(children) => children[0].as_ref().clone(),
            other => panic!("info relaxed to {other}"),
        };
        let mut merge = FragmentMerge::new(
            FRAGMENT,
            2,
            Arc::new(Schema::try_from(&manifest).unwrap()),
            relaxed.clone(),
        );
        merge.push(Run {
            fragment_id: FRAGMENT,
            addrs: vec![addr(1)].into(),
            outputs: vec![
                Some(Assignment {
                    values: tags(&["v1"], relaxed_item),
                    mask: None,
                }),
                Some(Assignment {
                    values: info(&["v1"], relaxed_child),
                    mask: None,
                }),
            ],
        });

        let merged = merge.merge(copy).unwrap();
        merge.check_complete().unwrap();
        assert_eq!(merged.schema(), relaxed);
        let tags = merged["tags"].as_any().downcast_ref::<ListArray>().unwrap();
        let tag = |row: usize| tags.value(row).as_string::<i32>().value(0).to_string();
        assert_eq!((tag(0), tag(1)), ("c0".to_string(), "v1".to_string()));
        let info = merged["info"].as_struct().column(0).as_string::<i32>();
        assert_eq!((info.value(0), info.value(1)), ("c0", "v1"));
    }
}
