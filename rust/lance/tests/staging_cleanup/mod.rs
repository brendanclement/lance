// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! What a failed publication staging logs while it discards its files on
//! local disk. The lib test binary cannot observe this: its `test_log` tests
//! may install the process-wide logger first.

use std::sync::{Arc, Mutex, Once};

use arrow_array::{
    ArrayRef, Int32Array, RecordBatch, RecordBatchIterator, StringArray, UInt64Array,
};
use arrow_buffer::BooleanBuffer;
use futures::{StreamExt, stream};
use lance::dataset::WriteParams;
use lance::dataset::cell_flag::{CellFlagOptions, ComputedBatch, PublicationStager};
use lance::{Dataset, Error};
use lance_core::ROW_ADDR;
use lance_core::utils::address::RowAddress;
use lance_core::utils::tempfile::TempStrDir;

static WARNINGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct CapturedWarnings;

impl log::Log for CapturedWarnings {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            WARNINGS.lock().unwrap().push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

fn capture_warnings() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        log::set_logger(&CapturedWarnings).expect("no other logger in this test binary");
        log::set_max_level(log::LevelFilter::Warn);
    });
}

/// Warnings naming a path under `dataset`'s data directory.
fn warnings_about(dataset: &Dataset) -> Vec<String> {
    let data_dir = dataset.data_dir().to_string();
    WARNINGS
        .lock()
        .unwrap()
        .iter()
        .filter(|warning| warning.contains(&data_dir))
        .cloned()
        .collect()
}

/// Two fragments of 3 rows with a masked `summary` computed from `body`.
async fn summaries(uri: &str) -> Dataset {
    let batch = RecordBatch::try_from_iter([
        (
            "id",
            Arc::new(Int32Array::from_iter_values(1..=6)) as ArrayRef,
        ),
        (
            "body",
            Arc::new(StringArray::from_iter_values(
                (1..=6).map(|id| format!("b{id}")),
            )) as ArrayRef,
        ),
        ("summary", Arc::new(StringArray::new_null(6)) as ArrayRef),
    ])
    .unwrap();
    let schema = batch.schema();
    let mut dataset = Dataset::write(
        RecordBatchIterator::new([Ok(batch)], schema),
        uri,
        Some(WriteParams {
            max_rows_per_file: 3,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    dataset
        .register_cell_flag(
            "summary",
            "ready",
            CellFlagOptions::default()
                .with_clear_on_write(["body"])
                .with_mask_when_false(true),
        )
        .await
        .unwrap();
    dataset
}

/// Summaries of `offsets` of `fragment_id`, assigned when `is_assigned`.
fn summary_rows(fragment_id: u32, offsets: &[u32], is_assigned: bool) -> ComputedBatch {
    let addrs: Vec<u64> = offsets
        .iter()
        .map(|offset| RowAddress::new_from_parts(fragment_id, *offset).into())
        .collect();
    let summaries = offsets
        .iter()
        .map(|offset| format!("s{fragment_id}-{offset}"));
    let rows = RecordBatch::try_from_iter([
        (ROW_ADDR, Arc::new(UInt64Array::from(addrs)) as ArrayRef),
        (
            "summary",
            Arc::new(StringArray::from_iter_values(summaries)) as ArrayRef,
        ),
    ])
    .unwrap();
    ComputedBatch::new(rows).with_assigned(
        "summary",
        BooleanBuffer::collect_bool(offsets.len(), |_| is_assigned),
    )
}

fn data_files(uri: &str) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(std::path::Path::new(uri).join("data"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    files
}

/// The caller's stream fails after fragment 0's file is complete and while
/// fragment 1's is being written. The discard finds nothing to remove but the
/// complete file: neither file has blob sidecars, and the local writer never
/// created the unfinished one. So it logs nothing.
#[tokio::test]
async fn test_failed_stage_discards_missing_files_quietly() {
    capture_warnings();
    let uri = TempStrDir::default();
    let read = summaries(&uri).await;
    let before = data_files(&uri);
    let items = vec![
        Ok(summary_rows(0, &[0, 1, 2], true)),
        Ok(summary_rows(1, &[0], true)),
        Err(Error::io("the computation failed")),
    ];
    let error = PublicationStager::try_new(Arc::new(read.clone()), &["summary"])
        .unwrap()
        .stage(stream::iter(items))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::IO { .. }), "{error}");
    assert!(
        error.to_string().contains("the computation failed"),
        "{error}"
    );
    assert_eq!(data_files(&uri), before);
    assert_eq!(warnings_about(&read), Vec::<String>::new());
}

/// A delete that fails for another reason than a missing file is still
/// logged, and leaves the staged file behind.
#[cfg(unix)]
#[tokio::test]
async fn test_failed_stage_logs_a_failed_discard() {
    use std::os::unix::fs::PermissionsExt;

    // Directory permissions do not bind root, so its deletes cannot fail here.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    capture_warnings();
    let uri = TempStrDir::default();
    let read = summaries(&uri).await;
    let before = data_files(&uri);
    let data_dir = std::path::Path::new(uri.as_str()).join("data");
    let set_mode = move |mode| {
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(mode)).unwrap();
    };
    // Fragment 1 assigns nothing, so fragment 0's file is the only one staged
    // when the stream fails.
    let items = stream::iter(0..3).map({
        let set_mode = set_mode.clone();
        move |item| match item {
            0 => Ok(summary_rows(0, &[0, 1, 2], true)),
            1 => Ok(summary_rows(1, &[0], false)),
            _ => {
                set_mode(0o555);
                Err(Error::io("the computation failed"))
            }
        }
    });
    let staged = PublicationStager::try_new(Arc::new(read.clone()), &["summary"])
        .unwrap()
        .stage(items)
        .await;
    set_mode(0o755);

    let error = staged.unwrap_err();
    assert!(
        error.to_string().contains("the computation failed"),
        "{error}"
    );
    let left = data_files(&uri);
    assert_eq!(left.len(), before.len() + 1, "{left:?}");
    let warnings = warnings_about(&read);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("failed to delete staged column file"),
        "{warnings:?}"
    );
}
