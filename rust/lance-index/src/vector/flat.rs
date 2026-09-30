// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Flat Vector Index.
//!

use std::sync::Arc;

use arrow::{array::AsArray, buffer::NullBuffer};
use arrow_array::{Array, ArrayRef, Float32Array, RecordBatch, make_array};
use arrow_schema::{DataType, Field as ArrowField};
use lance_arrow::*;
use lance_core::{Error, ROW_ID, Result};
use lance_linalg::distance::{DistanceType, multivec_distance};
use tracing::instrument;

use super::DIST_COL;

pub mod index;
pub(crate) mod pairwise;
pub mod storage;
pub mod transform;

fn distance_field() -> ArrowField {
    ArrowField::new(DIST_COL, DataType::Float32, true)
}

/// Get a column from a RecordBatch, supporting nested field paths.
///
/// This function handles:
/// - Simple column names: "column"
/// - Nested paths: "parent.child" or "parent.child.grandchild"
/// - Backtick-escaped field names: "parent.`field.with.dots`"
fn get_column_from_batch(batch: &RecordBatch, column: &str) -> Result<ArrayRef> {
    // Try to get the column directly first (fast path for simple columns)
    if let Some(col) = batch.column_by_name(column) {
        return Ok(col.clone());
    }

    // Parse the field path using Lance's field path parsing logic
    // This properly handles backtick-escaped field names
    let parts = lance_core::datatypes::parse_field_path(column)
        .map_err(|e| Error::schema(format!("Failed to parse field path '{}': {}", column, e)))?;

    if parts.is_empty() {
        return Err(Error::schema(format!(
            "Invalid empty field path: {}",
            column
        )));
    }

    // Get the root column
    let mut current_array: ArrayRef = batch
        .column_by_name(&parts[0])
        .ok_or_else(|| {
            Error::schema(format!(
                "Column '{}' does not exist in batch (looking for root field '{}')",
                column, parts[0]
            ))
        })?
        .clone();

    // Navigate through nested struct fields
    for part in &parts[1..] {
        let struct_array = current_array
            .as_any()
            .downcast_ref::<arrow_array::StructArray>()
            .ok_or_else(|| {
                Error::schema(format!(
                    "Cannot access nested field '{}' in column '{}': parent is not a struct",
                    part, column
                ))
            })?;

        current_array = struct_array
            .column_by_name(part)
            .ok_or_else(|| {
                Error::schema(format!(
                    "Nested field '{}' does not exist in column '{}'",
                    part, column
                ))
            })?
            .clone();
    }

    Ok(current_array)
}

#[instrument(level = "debug", skip_all)]
pub async fn compute_distance(
    key: ArrayRef,
    dt: DistanceType,
    column: &str,
    mut batch: RecordBatch,
) -> Result<RecordBatch> {
    if batch.column_by_name(DIST_COL).is_some() {
        // Ignore the distance calculated from inner vector index.
        batch = batch.drop_column(DIST_COL)?;
    }

    let vectors = get_column_from_batch(&batch, column)?;

    let validity_buffer = if let Some(rowids) = batch.column_by_name(ROW_ID) {
        NullBuffer::union(rowids.nulls(), vectors.nulls())
    } else {
        vectors.nulls().cloned()
    };

    tokio::task::spawn_blocking(move || {
        // A selection vector may have been applied to _rowid column, so we need to
        // push that onto vectors if possible.

        let vectors = vectors
            .into_data()
            .into_builder()
            .nulls(validity_buffer)
            .build()
            .map(make_array)?;
        let distances = match vectors.data_type() {
            DataType::FixedSizeList(_, _) => {
                let vectors = vectors.as_fixed_size_list();
                dt.arrow_batch_func()(key.as_ref(), vectors)? as ArrayRef
            }
            DataType::List(_) => {
                let vectors = vectors.as_list();
                let dists = multivec_distance(key.as_ref(), vectors, dt)?;
                Arc::new(Float32Array::from(dists))
            }
            _ => {
                unreachable!()
            }
        };

        batch
            .try_with_column(distance_field(), distances)
            .map_err(|e| Error::execution(format!("Failed to adding distance column: {}", e)))
    })
    .await?
}

#[cfg(test)]
mod tests {
    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, UInt64Array};
    use rstest::rstest;

    use super::*;

    /// A slice's null buffer starts at the slice's offset into its parent's,
    /// as in the batches of a scan rechunked by bytes. Vectors 1 and 6 are
    /// NULL, so reading the slice's validity from the parent's first bit
    /// would score vector 6 and drop vector 5. Row id 7 is NULL, for a
    /// deleted row.
    #[rstest]
    #[case::without_row_ids(None, [Some(16.0), Some(25.0), None, Some(49.0)])]
    #[case::row_ids_without_nulls(
        Some(UInt64Array::from_iter_values(0..8)),
        [Some(16.0), Some(25.0), None, Some(49.0)]
    )]
    #[case::row_ids_with_nulls(
        Some(UInt64Array::from_iter((0..8).map(|row| (row != 7).then_some(row)))),
        [Some(16.0), Some(25.0), None, None]
    )]
    #[tokio::test]
    async fn test_distances_of_a_sliced_batch_keep_its_nulls(
        #[case] row_ids: Option<UInt64Array>,
        #[case] expected: [Option<f32>; 4],
    ) {
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            (0..8).map(|row| (row % 5 != 1).then_some([Some(row as f32), Some(0.0)])),
            2,
        );
        let mut columns = vec![("vector", Arc::new(vectors) as ArrayRef)];
        if let Some(row_ids) = row_ids {
            columns.push((ROW_ID, Arc::new(row_ids) as ArrayRef));
        }
        let batch = RecordBatch::try_from_iter(columns).unwrap().slice(4, 4);
        let key = Arc::new(Float32Array::from(vec![0.0, 0.0]));

        let batch = compute_distance(key, DistanceType::L2, "vector", batch)
            .await
            .unwrap();
        let distances: Vec<_> = batch[DIST_COL]
            .as_primitive::<Float32Type>()
            .iter()
            .collect();
        assert_eq!(distances, expected);
    }
}
