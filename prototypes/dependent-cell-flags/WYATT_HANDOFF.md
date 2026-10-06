# Cell flags prototype: review handoff

This branch preserves the dependency-aware prototype. This handoff branch starts
at `667d654ca` and adds documentation/navigation only; the last production change is `fabb89983`.
The prototype has not been rebased onto current upstream main.

## Start here

1. Read [DESIGN_REVIEW.md](DESIGN_REVIEW.md) for the logical contract, alternatives, ownership,
   measured tradeoffs and integration gaps.
2. Read [README.md](README.md) for supported operations, API examples and correctness details.
3. Read [CONTINUATION.md](CONTINUATION.md) for the Linux changes, verification and build setup.
4. Read the three experiment reports linked below, then [CODING_PLAN.md](CODING_PLAN.md).

```sh
git clone --branch brendan/cell-flags-handoff-2026-10-06 https://github.com/brendanclement/lance.git lance-cell-flags
cd lance-cell-flags
```

Keep this checkout standalone, outside Sophon and any other Cargo checkout.
Follow the repository's AGENTS.md and language-specific
guides. Do not replace Sophon's pinned submodules to inspect this prototype.

## Xuanwo's work and design context

Xuanwo's [draft PR #8655](https://github.com/lance-format/lance/pull/8655), documented handoff head
`488aecce58e4e2cf19b3f67b34d9ee4f28b125b6`, is a separate generic cell-flags implementation:
stable identities, explicit mutations, query expressions, immutable external state, lifecycle
handling and Rust/Python/Java APIs. It excludes implicit dependency invalidation and mandatory
masking of ordinary values. Compare its storage, lifecycle, transaction and rollout contracts
with this prototype before choosing what to integrate; the implementations are not interchangeable.

Its PR records a complete Local FS/S3 benchmark matrix winning 123/201 primary metrics and failing
the frozen acceptance rule. Later candidate-only profiling is not a final-head pass. Benchmark,
final review, compatibility, writer rollout and main-integration work remain outstanding.
The [private benchmark repository](https://github.com/Xuanwo/lance-cell-flags-benchmark-20260815)
and retained EC2 artifacts need access from Xuanwo. This fork does not contain that branch or its
benchmark artifacts; the PR is the source of those handoff details.

The original [field-assignment discussion](https://github.com/lance-format/lance/discussions/8520)
provides additional context. Brendan also has saved internal function-design documents and
Weston's A-D alternatives in the original transfer packet. Those internal documents are not
copied into this public fork. The design review reconciles their relevant contracts and explains
why D was selected: preconditions protect publication during computation, while automatic
invalidation also handles source changes after publication.

## Contract and implemented mechanism

Readiness is distinct from value validity:

| Readiness | Stored value | Logical result |
| --- | --- | --- |
| false | Anything | NULL; unavailable and pending computation |
| true | NULL | NULL; computation completed with a NULL result |
| true | Non-NULL | Available computed result |

Work selection must use readiness, not `output IS NULL`. Lance tracks declared table dependencies
and freshness; LanceDB/Sophon own immutable function identity, computation and scheduling.

The prototype supports multiple inputs, independent and sibling outputs, acyclic chains,
supported source mutations, deletion/row movement, logical masking, and report-driven follow-ups.
Publication uses ordinary `DataReplacement` with typed flag assignments. `PublicationStager`
copies unassigned positions from its fixed read snapshot and writes whole output columns for
each touched fragment. Flag state is inline in the manifest. Both representation choices remain
subject to a production decision.

Preserve these guarantees:

- Source values and flag clearing commit atomically; invalidation is recorded even if the flag
  was already false, including an input changing and changing back.
- Manifest retries retain the original read version and resolve the latest registry. Replaced
  flag identities fence older workers.
- Newer results are protected across the whole replacement footprint, including copied rows.
  Under `Skip`, unsafe files/groups may be deferred rather than merely dropping assignments.
- Deleted/moved addresses cannot resurrect rows. A report is bound to its checked/committed
  snapshot; an empty follow-up plan does not certify all current rows as ready.
- Downstream results assigned with their upstream must use the new upstream values. Deferred or
  competing upstream publication must be accounted for when deciding downstream reuse.
- **Vector NULLs mask parent list validity only.** Retain the Arrow-validity, Torch and KMeans
  fixes. Stored child values may remain and consumers must honor validity.

Lance does not prove function execution, correct computed values or hand-built copy-through
contents. The stager is the experimental application path. Application sibling/binding ownership
is required even though Lance handles flag identity and atomic transactions.

## Review experiments and exact revisions

All experiments start from `16413f22b` and change tests/benchmarks/evidence only. Their reports
are copied into this branch for reading; the original experiment revisions below remain local.
Replaying the added probes or cost cases requires those separate local revisions and records.

| Local experiment branch / revision | Included report and result |
| --- | --- |
| `brendan/cell-flag-binding-reset` / `cb288925b` | [BINDING_RESET.md](BINDING_RESET.md): one existing `UpdateConfig` atomically resets the tested masked siblings and binding metadata, including downstream clearing. Negative controls define application obligations. Old downstream `Skip` may commit an invisible stale file beneath a false flag. |
| `brendan/cell-flag-replacement-validation` / `1282616ce` | [REPLACEMENT_VALIDATION.md](REPLACEMENT_VALIDATION.md): malformed real files can commit with incorrect lengths, mappings, relabeling or ragged fields. A footer row count alone is insufficient. The proposed structural validator remains unimplemented. |
| `brendan/cell-flag-cost-review` / `b5a59bd77` | [COST_REVIEW.md](COST_REVIEW.md): density/layout and publication concentration controls, quiet replication, profiles and checks. Use the qualifications in DESIGN_REVIEW when interpreting shorthand conclusions. Its additional raw records remain local. |

The independent flat-search NULL-offset fix is preserved locally at `8d09fb623` and as the
[extracted patch](upstream/0001-flat-search-null-offset.patch) already in this branch. Keep its
review separate from the cell-flags feature; no additional branch is published for it.

The transferred prototype head `cd44c394c` and historical experimental baseline `e3671b2f5`
remain reachable in the handoff branch's history. Full production verification at `fabb89983`
records 4,604 passing Lance library tests, 580 lance-table tests, relevant doctests, formatting
and Clippy. Later experiment checks have their own revisions and counts; they are not a rerun
of every integration surface.

## Measured tradeoffs and remaining decisions

- Scattered partial masks take about 2.1-2.2x matched stored-NULL scan wall time on the current
  code. Profile/source evidence identifies work on the scan's polling task; it does not prove
  CPU shares or a recoverable 2x speedup. Matched clustered comparisons reach 7% overhead.
- Refreshing 100 scalar rows writes 1.37 MiB in one fragment versus 13.83 MiB across ten.
  Ordinary replacement shares these bytes. Faster processing cannot remove this amplification.
- Concurrent fragment staging may reduce latency at a memory/CPU cost. The merge comparison
  illustrates that tradeoff, without predicting a parallel stager's performance.
- Inline flag state reaches 163 KiB for the measured scattered partial state, excluding the
  inline transaction payload. Open/serialization cost needs measurement before choosing external
  state; external flags do not reduce output bytes or stale-value decode.

The latest measurements use 1M rows, 100k-row fragments, narrow strings and warm tmpfs data.
Function computation is excluded. They do not establish cold disk/S3, wide-vector, deployment-build
or raced-workload acceptance. Production scope should specify output width, refresh distribution,
bytes, latency, memory and retry budgets before choosing publication granularity.

Structural replacement validation is an integration prerequisite. The candidate preflight must
read actual uploaded-file metadata, preserve legitimate imports/nested logical mappings, check
schema compatibility and physical row coverage, and install the validated descriptor in every
manifest arm. Immutable-object and trustworthy-footer-cache rules need review too.

Task 4 remains open: reproduce surviving MemWAL handles across activation, and publication after
cleanup removes staged files or required intervening validation history. Retaining an input
snapshot is not necessarily retaining that history; missing history must never certify freshness.

Bindings currently lose typed changes on round trips; complete thin application bindings after
the selected Rust contract settles. Masked-output indexes and compaction are currently refused.
ANN, additional writers, durable checkpoints/leases, mode fencing and compatibility/rollout need
explicit scope decisions. The application's initial computed-on-computed creation requirement
and automatic cascading job scheduling should be scoped separately.

## Code map and reproduction

| Area | Starting files |
| --- | --- |
| Persisted state and atomic invalidation | `rust/lance-table/src/format/cell_flag.rs`, `rust/lance-table/src/transaction/cell_flag_commit.rs` |
| Registry API, stager and follow-up planning | `rust/lance/src/dataset/cell_flag.rs`, `rust/lance/src/dataset/cell_flag/staging.rs` |
| Publication admission and reports | `rust/lance/src/io/commit/conflict_resolver/publication.rs`, `rust/lance/src/dataset/cell_flag/publication.rs` |
| Read masking | `rust/lance/src/dataset/fragment/cell_flag_mask.rs`, `rust/lance/src/dataset/fragment.rs` |
| Adversarial regression suites | `rust/lance/src/dataset/tests/dataset_cell_flags*.rs` |

CONTINUATION and each results README contain build/replay commands and exact source identities.
Use repository profiles and matched builds/data with identical-binary controls. Keep source/history
on the network mount if needed, with Cargo targets and benchmark data local. The documented protoc
adapter handles the current CIFS symlink placeholders. `/tmp` binaries, datasets and build caches
are ephemeral; committed records and replay instructions are durable.
