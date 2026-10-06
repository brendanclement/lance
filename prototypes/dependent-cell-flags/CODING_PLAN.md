# Concrete coding work for the design review

2026-10-06. Companion to [DESIGN_REVIEW.md](DESIGN_REVIEW.md), based on production
code at `fabb89983`. The original queue was committed at `16413f22b`; experiments
1–3 now have evidence in separate local branches. Production code is unchanged.

The agent can turn the remaining design questions into small reproducible examples,
tests and measurements. Start with the existing primitives and keep each finding
independently reviewable. Keep the established concurrency guarantees and parent-only
vector NULL masking throughout.

## Current findings and remaining work

| Task | Status and consequence |
| --- | --- |
| 1: binding reset | Characterized at `cb288925b`. One `UpdateConfig` handles the tested masked siblings and downstream atomically. The application must own the binding record, complete siblings and immutable snapshot basis. Old downstream `Skip` can commit invisible stale files; do not assert that all old transactions are refused. |
| 2: replacement validation | Characterized through round 2 at `1282616ce`. Malformed real files can commit. Preflight against the file's actual footer is a candidate, not an implemented fix; valid external imports and nested logical mappings need compatibility rules before coding it. |
| 3: costs | Characterized at `b5a59bd77`. Scattered partial masks cost about 2.1–2.2x matched stored-NULL wall time. Sparse publication bytes follow touched fragments and are shared with ordinary replacement. No new processing optimization was implemented. |
| 4: activation and lifecycle | Still open. This is the next independent correctness characterization before integration; it can proceed without choosing a publication format. |

The reports and qualified comparisons are linked in DESIGN_REVIEW's evidence map. Do not merge
test/benchmark branches merely to consolidate the findings. The next review decision is whether
the first workload accepts full-fragment bytes and latency. Otherwise compare row-selective
designs below before implementing a writer. Scattered-mask optimization and bounded staging
concurrency are optional budget-driven tasks; neither reduces whole-fragment publication bytes.

For task 4, keep activation fencing and cleanup/history retention independently reviewable.
Record observed behavior through real handles and files, not just source inspection. Distinguish
retained input snapshots from retained intervening validation history, and committed files from
unreferenced staging. Missing history must never certify freshness. Characterize first, without
adding a lease/epoch format or changing the concurrency contract automatically.

## Work that can start without choosing a new format

| Order | Coding task | Concrete deliverable and pass condition |
| --- | --- | --- |
| 1 | Prove atomic binding reset using the current Rust transaction API | One `UpdateConfig` transaction changes an illustrative binding metadata record and drops/re-registers all sibling flags. Test two outputs, an existing downstream dependent output, an old worker, a concurrent source writer and competing binding resets. Every observed snapshot has a consistent binding and registry; both siblings become pending; downstream readiness clears; old work cannot make stale output visible; a conflicting binding change is not silently overwritten. |
| 2 | Characterize ordinary replacement validation | Extend existing replacement tests with a real short/long replacement file, mixed stored/all-NULL fields, deletion holes, nested V2 fields and overlay supersession. Record what is accepted and what reads return. Preserve minimal reproducers; agree logical mapping/import compatibility before a focused validation fix. Do not assume the stager protects arbitrary callers. |
| 3 | Measure the unresolved read and staging costs | Use the existing saved-binary harness for partial masks at several densities and sparse publication concentrated in one fragment versus scattered across many. Record decode/IO bytes, stage bytes, allocations and timing with matched ordinary-NULL/plain controls. Implement an optimization only when the measured processing bottleneck matters to the selected workload budget; retain unchanged publication semantics. |
| 4 | Reproduce activation and lifecycle gaps | Hold a MemWAL writer/reader open across index removal and masking registration; exercise the fresh-tier-only reader. Separately stage work, advance the table, run cleanup, and attempt publication after its files or required validation history disappear. Pass means enforced refusal/fencing or a clearly specified supported lifetime; missing history never certifies freshness. |

Task 1 is a Rust characterization fixture, not a function catalog implementation.
After these findings, implement thin bindings for the selected application path.
Their round-trip tests must preserve flag registrations, drops, assignments and row
movement payloads, the original read version, conflict policy and report semantics.
Do not start by redesigning the existing Python/Java public APIs.

## What the source audit establishes

- **Atomic reset now has a tested starting point.**
  [TransactionBuilder](../../rust/lance-table/src/transaction/builder.rs) attaches
  `CellFlagChanges` to one ordinary transaction;
  [manifest construction](../../rust/lance-table/src/transaction/manifest_build.rs)
  applies `UpdateConfig` metadata, and
  [flag application](../../rust/lance-table/src/transaction/cell_flag_commit.rs)
  handles a vector of drops and registrations. New IDs are allocated from the
  head registry at commit time. Never predict them from the staged snapshot's
  next-ID counter. An initial fixture can store stable output IDs and flag-slot
  descriptors in the binding record, then resolve actual IDs from the *same
  committed snapshot*. If the durable record must contain allocated IDs itself,
  specify commit-time resolution before adding that API.
- **Competing metadata updates already have conflict machinery.**
  [UpdateConfig conflict checking](../../rust/lance/src/io/commit/conflict_resolver.rs)
  rejects overlapping config/metadata updates. This is a building block; it does
  not by itself define binding ownership or an expected-binding precondition.
  Test sibling resets as one transaction and re-evaluate a conflicting reset
  rather than blindly resubmitting it against a newer binding.
- **Replacement validation now has real-file reproductions.**
  Experiment 2 confirms the missing commit checks for row counts and mappings,
  and shows why footer row count alone misses ragged fields. Validate actual
  logical fields, not raw nested physical-column lengths. General imports,
  schema compatibility and trustworthy footer caching need a contract before
  selecting the preflight implementation.
- **The fresh-tier guard needs a lifetime test.**
  [The LSM collector](../../rust/lance/src/dataset/mem_wal/scanner/collector.rs)
  checks masking when it has a base dataset and returns success when that
  dataset is absent. The existing snapshot guard cannot establish fencing of a
  surviving shard handle. Reproduce the transition before selecting a guard or
  a persistent mode/epoch contract.
- **Staged files and history are distinct resources.**
  [Cleanup](../../rust/lance/src/dataset/cleanup.rs) normally grants unreferenced
  in-progress files an age grace period, documented as seven days. That is not a
  renewable lease. A job also needs validation history from its read snapshot;
  retaining its input snapshot alone must not be assumed to retain every
  intervening transaction. Define both lifetimes and ambiguous-commit
  reconciliation before durable checkpoint integration.
- **Binding round trips currently lose typed changes.**
  [Python extraction](../../python/src/transaction.rs) sets
  `cell_flag_changes: None`; [Java extraction](../../java/lance-jni/src/transaction.rs)
  builds a transaction without attaching them. Source inspection confirms these
  boundaries; this review did not run a new Python/Java integration test.

## Publication experiment: three options to compare

Use a three-row example before implementing a row-selective writer. An ordinary
[overlay](../../docs/src/format/table/data_overlay_file.md) with coverage
`{2, 5, 9}` stores values `[A, NULL, C]`. Removing offset 5 from coverage while
keeping the file unchanged makes offset 9 read the value at rank 1: `NULL`, not
`C`. Coverage cannot be filtered independently of its stored value positions.

| Candidate | How it preserves alignment | Tradeoff to measure |
| --- | --- | --- |
| Existing full-fragment replacement | Preserve every physical position and protect the full file footprint | Copies unrelated cells; a newer result anywhere in that footprint can defer the whole group |
| Filtered ordinary overlay | Rewrite selected staged values as `[A, C]` with coverage `{2, 9}` | Adds staging IO and overlay read/maintenance work, but requires no function recomputation and keeps the ordinary rank contract |
| Immutable stored-row mapping plus effective coverage | Keep `{2, 5, 9}` as the value-position mapping and resolve only effective offsets `{2, 9}` | Can reuse original file bytes; requires new resolution/format semantics and explicit lifecycle, conflict, index and compaction rules |

The latter two are candidate designs. The current dependent publication path
refuses watched-field overlays; neither is enabled by this document. The
experiment must preserve computed NULLs, deletions, sibling atomicity, chain
provenance, fixed read versions, false-to-false invalidations, latest-registry
validation and protection of newer results. Narrower conflict footprints require
an explicit proof and adversarial tests, not just smaller output files.

For a simple **uncompressed payload model**, take 100 assigned rows scattered
across ten 100k-row fragments. Full replacement writes 1M row positions versus
100 for a selective representation: 10,000x. At 16 bytes/value that is 16 MB
versus 1.6 kB; for a 768-dimensional float32 vector it is 3.072 GB versus
307.2 kB. These are decimal payload sizes excluding metadata, validity, encoding
and IO overhead. They are not Lance timings or storage measurements. The actual
[Linux scalar benchmark](bench/results/linux-local-optimizations/README.md)
wrote about 13.8 MiB/output for 100 scattered results.

## Flag storage experiment and recommended first slice

Keep publication bytes and readiness metadata as separate experiments. For inline
state, measure manifest bytes, serialization/open CPU and allocation growth versus
fragment count, flag count and true-set fragmentation. For external immutable
state, estimate descriptor bytes plus changed-state bytes and count extra cold
reads, writes and cleanup objects. Whole-table scans may need every external
object; selected-column/fragment reads may need fewer. A spill threshold adds two
resolution paths. There is no implemented external-state benchmark or measured
S3 crossover in this packet; retain inline state until representative evidence
justifies a storage change.

The recommended initial integration experiment is one row-local scalar function,
optionally a two-output sibling fixture, supported V2 writers, ordinary scans/take,
and short-lived jobs on a controlled table. Keep current fragment publication and
inline state as the baseline. Exercise `Skip` reports and follow-ups; finish with
a pending-row scan rather than treating an empty report plan as all-ready. This
slice tests the contract without requiring ANN, compaction or durable checkpoint
support. A product that requires those capabilities needs their designs before
adopting this slice as its release scope.

All work stays in the standalone checkout. Keep source/history on the network
mount and build/data scratch local as documented in [CONTINUATION.md](CONTINUATION.md).
Save reproducible findings and local commits. No push, PR or Sophon submodule change.
