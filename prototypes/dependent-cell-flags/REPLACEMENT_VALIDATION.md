# DataReplacement validation: characterization

Handoff copy: original report at local revision `1282616ce`. The added experiment tests and
their branch remain local with Brendan; this published prototype includes the report only.

2026-10-05. Coding task 2 of [CODING_PLAN.md](CODING_PLAN.md). **Unstable research prototype.**
Base `16413f22b`; local branch `brendan/cell-flag-replacement-validation`:

- `330dba801`: ordinary replacement tests in `fragment_write_columns.rs`;
- `8a4b8e637`: publication test in `dataset_cell_flags_publication.rs`;
- `c70df79e8`: round-2 probes and the footer oracle in `fragment_write_columns.rs`.

Production code is unchanged, and nothing was pushed.

**Question:** can malformed replacement files become visible, and is the prototype's mixed-field
extension (`ded2e02e9`) correctly validated? Round 2 asks which structural checks are
sufficient.

**Answer:** yes, they become visible. A real Lance file of the wrong length, or one described
with field ids, column indices or a mapping it was not written for, commits through the normal
API in every manifest arm. Only `validate()` reports a wrong footer row count, and nothing
reports a relabel, swapped indices or unequal column lengths.

- These gaps predate the extension: they reproduce at `ded2e02e9^` (`4ac8492d2`), and the
  manifest build's TODO comes from upstream #3408.
- The extension adds one more: it accepts descriptors that mix a stored and an unstored field,
  including one claiming a stored field its file does not contain, which was refused before.
- A hand-built dependent publication inherits the gap and sets its flag true over misaligned
  values.
- **Round 2 refines the recommendation.** Raw field-id equality is unsound: it rejects the valid
  V2.1 struct control and passes swapped indices. A footer row count alone also misses unequal
  column lengths. The minimum check compares each descriptor with the mapping and row counts
  derived from its file's own footer.

## Where each property is validated

The commit path runs:

1. `write_columns` (staging);
2. `CommitBuilder`;
3. [`commit_transaction_with_report`](../../rust/lance/src/io/commit.rs) and its publication
   preflight;
4. per attempt, the rebase
   ([`check_data_replacement_txn`](../../rust/lance/src/io/commit/conflict_resolver.rs), which
   compares fragment ids and fields only);
5. the [manifest build](../../rust/lance-table/src/transaction/manifest_build.rs);
6. the [storage contract](../../rust/lance/src/dataset/versions/mod.rs);
7. the manifest write.

`validate_operation` returns `Ok` for every `DataReplacement`. No step after staging opens the
replacement file.

| Property | Staging (`write_columns`) | Commit | Read | `validate()` |
| --- | --- | --- | --- | --- |
| Footer row count | Refused: `has N rows but the fragment has M physical rows` | **Not checked** (TODO, `manifest_build.rs:1132`) | Only where a read touches a missing position (`lance-file/src/reader.rs:1829`) | `data file has incorrect length` |
| Per-column row counts | Batches have equal columns | **Not checked** | Each projection's length comes from its columns' page metadata; unequal projected fields are refused | **Not checked** (footer count only) |
| Logical field coverage | Fields resolved against the manifest | Groups must write the same fields; the fragment must change. The file is never opened | Decoder errors on a type mismatch | Field order and duplicates only |
| Physical mapping | Layout taken from the manifest | Field and column-index counts must match; V2.1+ shape rules against the schema (`versions/mod.rs:496`); none for V2.0. **The exact swap keeps the stored `column_indices`** | Column count (`reader.rs:1136`) | No |
| File version | V1/V2 family; an exact V2 version is allowed | Must decode; V1/V2 mixing refused. A claim is never compared with the footer | The footer decides V2 decoding | V1/V2 mixing |
| Base-field supersession | — | Exact swap; push of uncovered fields; tombstone-and-append, which the extension opens to any partly covered layout; files left with no live field dropped | — | — |
| Overlay supersession | — | Overlays committed at or before the read version are tombstoned on replaced fields; newer ones are kept | A newer overlay wins its cells | — |

## Results: round 1

Fragment 1 has 4 physical rows. Each file is real: it was written by `write_columns` for a
neighbouring fragment of the needed length, so staging accepted it, and then paired with
fragment 1. In every accepted case the version advances by one. Reads of `id`, which open no
replaced file, and the retained previous version stay readable. Every refused case leaves the
version, the fragment list and the values unchanged. The last column is the pre-extension
revision `4ac8492d2`, run with the same test file.

| Scenario | Commit | Scan of the field | Take | `validate()` | At `4ac8492d2` |
| --- | --- | --- | --- | --- | --- |
| Control: an exact file on every fragment | Accepted | `v = id × 10` on all three fragments | Correct | Ok | Same |
| 3-row file over an all-NULL `v` (push) | Accepted | `InvalidInput`: `cannot read Ranges([0..4]) from columns with 3 rows` | A row the file holds reads its value by position; the last row is `InvalidInput` | `CorruptFile`: `Expected: 4 Got: 3` | Same |
| 5-row file (push) | Accepted | **Silent**: the file's first four values | **Silent** | `CorruptFile`: `Got: 5` | Same |
| 3 live-row values, offset 1 deleted | Accepted | `InvalidInput`: `Ranges([0..1, 2..4])` | `InvalidInput` | `CorruptFile` | Same |
| 3 live-row values, offset 3 deleted | Accepted | **Silent**: values by position | **Silent** | `CorruptFile` | Same |
| 3-row file over a stored `v` (swap) | Accepted | `InvalidInput` | `InvalidInput` | `CorruptFile` | Same |
| 3-row `[v, w]` file over a stored `v` and an all-NULL `w` | Accepted | `InvalidInput` | `InvalidInput` | `CorruptFile` | Refused: `Expected to modify the fragment` |
| `w`'s file described as `v` (swap) | Accepted | **Silent**: `w`'s values as `v` | — | **Ok** | Same |
| A Utf8 file described as the Int32 `v` | Accepted | `CorruptFile`: `invalid variable-width layout for Int32` | — | Ok | Same |
| `w`'s one-column file described as `[v, w]` over a stored `v` | Accepted; the stored `v` file is tombstoned | `v` **silently** reads `w`'s values; `w` is `InvalidInput`: `column index 1 but there are only 1 columns` | — | Ok | Refused: `Expected to modify the fragment` |
| Control: a V2.1 struct file replacing a V2.0 one | Accepted; the V2.0 file is dropped | Correct | — | Ok | Same |
| The same file claiming V2.0 | Accepted; the manifest records 2.0 | Correct (cosmetic claim) | — | Ok | Same |
| The same file with the stored V2.0 mapping, claiming V2.0 | Accepted (swap) | `InvalidInput`: `column index 2 but there are only 2 columns` | — | Ok | Same |
| The same mapping, claiming V2.1 | Refused: `non-leaf fields should have column_index=-1` | — | — | — | Same |
| Claims V1 / version 2.9 | Refused: `mixes V1 and V2 data files` / `Unknown Lance storage version` | — | — | — | Same |
| Two field ids, one column index | Refused: `has 2 field ids but 1 column indices` | — | — | — | Refused earlier: `Expected to modify the fragment` |
| Groups writing different fields / an unknown fragment | Refused: `must have the same fields` / `not found in existing fragments` | — | — | — | Same |
| Overlays on `v` and `u`, then `v` replaced from a read after them | Accepted | The `v` overlay is removed and replaced; the `u` overlay is kept | — | Ok | Same |
| The same replacement read before the overlays | Accepted | Both kept; the newer `v` overlay still wins its cell | — | Ok | Same |
| Hand-built publication: a 3-row summary file on a 2-row fragment, flag set true | Accepted; flag true | **Silent**: values staged for ids 5 and 6 read as ready summaries of ids 1 and 2 | — | `CorruptFile` | Not run |
| The same with a 1-row file | Accepted; flag true | `InvalidInput` on every scan of `summary` | — | `CorruptFile` | Not run |

The prototype also refuses new overlays on fields a dependent flag watches
(`dataset_cell_flags::test_unsupported_operations_fail_explicitly`). That is separate from the
ordinary supersession above, which still applies to overlays that predate a flag.

## Results: round 2

Each test reads the replacement file's actual footer with `FileReader::read_all_metadata` and
records which candidate checks the descriptor fails. The candidates are:

- **version**: the claimed version equals the footer's;
- **rows**: the footer row count equals the fragment's physical rows;
- **bounds**: every column index is below the footer's column count;
- **column rows**: every column's page lengths sum to the footer's row count;
- **mapping**: `(fields, column_indices)` equals `data_file_columns(footer version, footer
  schema)`.

Version, rows and bounds were the round-1 proposal. The tests assert every outcome below.

**Probe 1: struct footers.** In both the V2.0 and the V2.1 struct files the footer schema
holds `[point, x, y]` (ids 1, 2, 3). Their descriptors differ:

- the V2.0 descriptor is `[point, x, y]` → columns `[0, 1, 2]`;
- the V2.1 descriptor is `[x, y]` → `[0, 1]`, because V2.1 gives a struct parent no column.

Raw field-id equality therefore **rejects the valid V2.1 control**, while each descriptor equals
its own version's `data_file_columns`. The footers have 3 and 2 physical columns. Every column
of both files, including the V2.0 parent column, sums to the footer's 2 rows.

**Probe 2: swapped column indices.** A real `[a, b]` file of two Int32 columns with distinct
values is written at the correct length and field ids, and its indices are swapped to
`[1, 0]`. The footer's ids equal the descriptor's `fields`, so raw id equality passes too.

**Probe 3: unequal column lengths.** A real file is written with `FileWriter::write_column`:
`a` has 4 rows and `b` has 3. The footer's row count is 4, matching the fragment. Its
descriptor comes from the public footer-derived API, `Dataset::create_data_file`, which
accepts it.

| Case | Commit | Reads | `validate()` | Fails |
| --- | --- | --- | --- | --- |
| Control: an exact file on every fragment | Accepted | Correct | Ok | — |
| Control: stored `v` plus all-NULL `w` in one file, plain and compacted | Accepted | Correct | Ok | — |
| Control: V2.0 struct file / V2.1 struct file, as written | Accepted | Correct | Ok | — |
| Probe 2 over all-NULL `a`, `b` (push) | Accepted | **Silent**: `a` reads `b`'s values and `b` reads `a`'s | Ok | mapping |
| Probe 2 over `a`, `b` stored together (exact swap) | Accepted; the manifest keeps the stored `[0, 1]` | Correct, because the swap discards the descriptor's indices | Ok | mapping |
| Probe 2 over a stored `a` and an all-NULL `b` (tombstone-and-append) | Accepted | **Silent** swap, as in the push case | Ok | mapping |
| Probe 3 | Accepted | Scan of `a` correct. Scan of `b`: `InvalidInput`, `cannot read Ranges([0..4]) from columns with 3 rows`. Scan of `a, b`: `cannot read columns of differing lengths together (a=4, b=3)`. Take of `b`'s row 0 **silently** returns 21; row 3 fails | **Ok** | column rows |
| Round-1 wrong-length files (all six) | Accepted | As in round 1 | `CorruptFile` | rows |
| Round-1 relabels, same or other type | Accepted | As in round 1 | Ok | mapping |
| Round-1 `[v, w]` descriptor for a `w` file | Accepted | As in round 1 | Ok | bounds, mapping |
| Round-1 V2.1 struct claiming V2.0 | Accepted | Correct | Ok | version |
| Round-1 V2.1 struct with the V2.0 mapping, claiming V2.0 / V2.1 | Accepted / refused | As in round 1 | Ok | version, bounds, mapping / bounds, mapping |

## Confirmed findings

1. **Wrong-length files commit (pre-existing).** All three manifest arms are affected. A read
   fails where it touches a missing position, with a misleading `InvalidInput`. Otherwise it is
   silently misaligned: a long file, a deleted tail, or a take within the file. Reproduced at
   `4ac8492d2` for the push and swap arms; the TODO comes from upstream #3408.
2. **Field ids are trusted (pre-existing).** A same-type relabel silently serves another
   column's data and passes `validate()`.
3. **V2.0 mappings are trusted (pre-existing).** A V2.1 file described with the V2.0 mapping
   commits through the swap, which keeps the old column indices. Its struct is then unreadable.
4. **The extension accepts more malformed descriptors (introduced by `ded2e02e9`).** A
   descriptor mixing a stored and an unstored field is accepted. One that claims a stored field
   its file lacks tombstones the real column. The same descriptor is refused at `4ac8492d2`.
5. **Publication inherits finding 1.** A true flag vouches for values from a misaligned file.
6. **Column indices are trusted.** Swapped indices with correct ids, lengths and version pass
   every round-1 check, and the push and tombstone arms publish them: a silent swap. The exact
   swap ignores the incoming indices and republishes the stored ones, which hides a bad
   descriptor rather than validating it.
7. **Unequal column lengths are trusted.** Neither the commit, `validate()` nor
   `create_data_file` compares per-column lengths. Reads fail only for projections that touch
   the short column, and takes within it succeed.
8. **Raw id equality is the wrong check.** It rejects a valid V2.1 struct and accepts swapped
   indices. The version-aware mapping accepts every control and catches every malformed field
   or index mapping in both rounds. Length and version faults need their own checks.

Not failures: a version claim is cosmetic for V2 decoding. V1, unknown or self-inconsistent
descriptors are refused before a manifest is written. Overlay supersession follows its
documented rule.

## Source-only observations and inferences (not exercised)

- **Source:** no arm has ever checked length. The tombstone path for a field stored in a wider
  (compacted) file therefore also accepts wrong lengths before the extension, though this
  experiment exercised only the push and swap arms there.
- **Source:** between `e3671b2f5` (upstream at the start of the prototype) and `4ac8492d2`, the
  DataReplacement arm, the storage contract and the `lance-file` reader are textually
  unchanged. Findings 1–3 should therefore reproduce on upstream main. This was not run.
- **Inference:** findings 6 and 7 go through the push and swap arms, which are unchanged since
  `4ac8492d2`, so they should predate the extension. The tombstone case of probe 2 mixes a stored
  and an unstored field and should be refused there. Round 2 was not rerun at `4ac8492d2`.
- **Source:** unequal column lengths are legal in the file format. The reader advises reading
  "each column (or equal-length group) separately", and `write_column` produces them. Only a
  *table data file* must give every column its fragment's physical rows, so the rule belongs to
  the table layer.
- **Inference:** summing page lengths per physical column is right for scalars and structs.
  It would wrongly reject a V2.0 list, whose item column counts items. A per-field length check
  should reuse the reader's projection-length logic (`reader/structural.rs:320`), which already
  computes field lengths by version and refuses unequal fields.
- **Inference:** `create_data_file` matches footer fields to the dataset by name and child
  count (`dataset.rs:2324`), then takes types and layout from the dataset schema (`:2408`). A file
  whose footer names a field correctly but with another type would pass it; this was not probed.

## Minimum sufficient structural checks

For each replacement group, read the file's footer with `read_all_metadata`. That is the
file's own schema, version, row count and column metadata, not the manifest's schema or the
fragment's row count supplied in its place. Then require:

1. **Version.** The footer's version is V2 and equals the claimed version. *Confirmed needed:* a
   V2.0 claim on a V2.1 file defeats the V2.1 mapping rules.
2. **Mapping.** The descriptor's `(fields, column_indices)` equals `data_file_columns(footer
   version, footer schema)`. That function is version-aware: V2.0 gives every field a column,
   V2.1+ only leaves, packed structs and blobs. *Confirmed sufficient:* it catches every
   relabel, the swapped indices, the extra field and both V2.0-mapping lies, and passes every
   control. Bounds checks become redundant.
3. **Field definitions.** Every footer field matches the dataset field with the same id: type,
   nullability, children and storage markers. *Inference:* needed when a footer reuses the right
   id with another type; no probe here.
4. **Rows.** The footer's row count equals the fragment's physical rows (*confirmed*, finding 1).
   Every field's own length equals it too (*confirmed*, finding 7). Compute field lengths with the
   reader's version-aware logic, not raw page sums (*inference*, lists).
5. **Exact swap.** Validate the incoming descriptor, then publish it. With checks 1 and 2 the
   stored and incoming mappings are equal by construction (*inference*: same fields, same
   version). Today the swap silently keeps the stored mapping (*confirmed*).

These checks cannot establish the following, which remains the caller's responsibility:

- that values were computed correctly and from the right inputs, and, for publications, that
  unassigned rows were copied through;
- **positional identity**: a correctly shaped file staged for another fragment of the same
  physical row count passes every check (*inference*);
- that the object at `path` does not change after validation;
- the integrity of encoded pages, since no checksum is verified.

## Recommended validation boundary

Run the checks above **once per transaction in the Lance commit path**: in
`commit_transaction_with_report`, before the retry loop, beside the existing read-version
preflight. Read each file through the fragment's path resolution (including `base_id`) and
refuse with `InvalidInput`, naming the fragment, the path, and the expected and actual values.

- **Coverage.** It is the narrowest point with object-store access and the read snapshot. One
  check covers `write_columns`, the stager, `create_data_file`, hand-built descriptors and
  relayed transactions.
- **Retries.** Files are immutable, and a fragment's physical row count is fixed for the life
  of its id; a concurrent removal or rewrite already conflicts. The IO is needed once per
  transaction, not per attempt.
- **Why not the manifest build.** The manifest build is IO-free, and `DataFile` holds no row
  count. Checking there would need a format field or trusted metadata.
- **IO.** `read_all_metadata` costs one tail read per file (`read_tail`: one GET of the reader's
  block size). It needs a size lookup only when `file_size_bytes` is unknown, and one more GET
  when the metadata overflows the tail. Column metadata and the footer schema come from the same
  read. A publication over N fragments costs N reads, issued in parallel.

### Cache provenance

Cached metadata is equivalent to a footer read only under the conditions below (source
inspection, `fragment.rs`):

- **Two caches, both keyed by path only.** `get_file_metadata` caches a `CachedFileMetadata`
  decoded from the footer by `read_all_metadata`. `get_file_metadata_index` caches a
  `FileMetadataIndex`. Both live in `file_metadata_cache(path)`, keyed by the path alone, with no
  size or etag.
- **The index can come from the manifest.** When a read projects under a quarter of a file's
  columns, the index is built from `known_schema`: the descriptor's schema and the fragment's
  `physical_rows` from the manifest (`fragment.rs:1514-1530`). An index of that provenance
  restates the manifest, so checking a descriptor against it is vacuous.
- **What a substitute must be.** Only a `CachedFileMetadata` decoded from the object's own
  bytes can stand in for a footer read. The object at that path must also be unchanged since.
  Lance writes data files once under unique names, but a path-only key cannot detect an external
  overwrite.
- **Writer seeding.** Seeding the cache at staging, as round 1 suggested, has the writer's
  provenance rather than storage's. It is equivalent only if the entry is recorded after the
  upload succeeds and is decoded from the footer bytes the writer emitted.
- **Scope.** The cache is per session. A file staged by another process or session is never
  cached, so its commit must read the footer.

| Boundary | Covers | Cost | Gap |
| --- | --- | --- | --- |
| Staging only (today) | `write_columns`, the stager | None | Hand-built, relayed or `create_data_file` descriptors |
| Read-time open check | Turns silent misreads into errors | None extra; footers are read at open | Malformed versions still commit and fail late |
| **Commit footer checks 1–5 (recommended)** | Every caller and every arm | One tail read per file and transaction | Caller trust above |
| `validate()` (today) | Footer row count | Reads every file | Not on the commit path; misses findings 2–4, 6 and 7 |

### Compatibility questions

1. **Footer field ids.** Files from `write_columns` carry the dataset's ids (confirmed for flat
   and struct columns). Files from external writers may not, and `create_data_file` matches them
   by name. Should the check require dataset ids (rename-safe), match names (external-writer
   friendly but rename-fragile), or accept both under a stated rule?
2. **Cosmetic version claims.** These read correctly today. Should the check refuse them or
   normalize the descriptor to the footer's version?
3. **Descriptor normal form.** The V2.1+ storage contract allows non-leaf entries with index −1,
   which the writer and `data_file_columns` omit. Should the check compare after dropping them,
   or require the canonical form?
4. **Blob footer orphans, packed structs, lists and maps.** `create_data_file` handles blob
   descriptor orphans; the mapping and length checks must agree with it and need coverage.
5. **V1 datasets.** Their exact-swap path has legacy footers. Under the repository's legacy
   policy, should V1 replacements be left unvalidated or refused?
6. **Stricter commits.** They will reject descriptors accepted today, including through Python
   `LanceOperation.DataReplacement` and `create_data_file`. Is a release note sufficient, or is a
   transition needed?
7. **`create_data_file`.** Should it adopt the same checks, since it already reads the footer
   but ignores per-column lengths?

## Impact on acceptance

- **The general DataReplacement extension.** Findings 1–3 and 6–7 are pre-existing gaps that
  affect every caller. They argue for commit-time validation before DataReplacement is relied on
  as an integration surface. The extension widens what commits without adding validation
  (finding 4). Accept it only together with the mapping check, which catches finding 4. With
  honest descriptors its structural behavior is correct, per its existing tests and the controls
  here.
- **Dependent publication.** Stager-built publications are unaffected: they write exact rows
  and the declared outputs by construction. Hand-built publications inherit findings 1, 6 and 7,
  and worse, their flag certifies the result (finding 5). Until validation exists, the stager is
  the only safe publication path. Passing stager tests does not establish safety for arbitrary
  callers.
- **Both** therefore depend on the same commit-time checks.

## Reproduce

Fedora Linux x86_64, the host of [CONTINUATION.md](CONTINUATION.md). Rust 1.97.0 (repository
pin), `ci` profile, `--locked`, source on CIFS, build target under `/tmp`.

```sh
cd /home/brendan/work/lance-cell-flags
git worktree add /home/brendan/work/lance-cell-flags-replacement-validation brendan/cell-flag-replacement-validation
cd /home/brendan/work/lance-cell-flags-replacement-validation
mkdir -p /tmp/cell-flags-replacement-validation/tools
cp prototypes/dependent-cell-flags/bench/protoc_network.py /tmp/cell-flags-replacement-validation/tools/protoc-network
chmod +x /tmp/cell-flags-replacement-validation/tools/protoc-network
export PROTOC=/tmp/cell-flags-replacement-validation/tools/protoc-network
export CARGO_TARGET_DIR=/tmp/cell-flags-replacement-validation/target LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1
cargo test -p lance --lib --profile ci --locked -j 12 -- dataset::tests::fragment_write_columns dataset::tests::dataset_cell_flags
```

For attribution, check out `ded2e02e9^` in a detached worktree, copy in this branch's
`fragment_write_columns.rs`, and run the new replacement tests there. After switching a shared
target back to this branch, run `cargo clean -p lance-table -p lance --profile ci`. Cargo
otherwise reused the other revision's `lance-table`: its dep-info paths are workspace-relative
and the checked-out sources were older than the build.

| Check | Round 1 (`799b2bdc5`) | Round 2 (`c70df79e8`) |
| --- | --- | --- |
| `fragment_write_columns` and `dataset_cell_flags*` | 461 passed, 1 ignored | 465 passed, 1 ignored: 4 new cases, and footer assertions on 16 existing ones |
| Cases touched by the probes, serially | 23 new cases in 0.36 s | 20 cases in 0.31 s |
| `4ac8492d2`, the round-1 replacement tests | 18 passed; the 3 failures are the refusals in the table | Not rerun |
| `cargo fmt --all -- --check`, `git diff --check` | Passed | Passed |
| `cargo clippy -p lance --tests --profile ci --locked -j 12 -- -D warnings` | Passed | Passed |

A host reboot on 2026-10-05 wiped `/tmp`, including the round-1 logs. Round 2 rebuilt from a
fresh target and re-ran the round-1 suite (461 passed, 1 ignored) before adding probes. Its logs
are in `/tmp/cell-flags-replacement-validation/logs/`: `baseline-799b2bdc5.log`, the
`probes-*.log` runs, and `final-tests.log`, `final-serial.log` and `final-clippy.log`. `/tmp` is
ephemeral; the tables above are the durable record.
