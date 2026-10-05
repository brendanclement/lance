# Dependency-aware cell flags: design and tradeoffs

Review draft, 2026-10-05. **Unstable research prototype; no production acceptance or integration approval.**
The reviewed implementation is `fabb89983996569ceb0ba480696528cdd34b34b5`, with results and
handoff committed at `c9ff941b5`. The transferred prototype remains at `cd44c394c`.

**Recommendation:** retain option D's logical contract: Lance atomically invalidates outputs
when their declared inputs change, masks unavailable outputs, and validates publication of
computed results. Keep functions and jobs in LanceDB/Sophon. Treat whole-fragment publication
and inline flag state as prototype choices whose production suitability still needs a decision.
The evidence supports this direction and several local optimizations; it does not establish that
the entire branch is ready to integrate.

The review should settle the smallest useful product scope, the publication unit, the state
storage strategy, and the binding/lifecycle contract. Another optimization is useful only when
it resolves a cost that matters to those choices.

[CODING_PLAN.md](CODING_PLAN.md) turns these questions into ordered coding tasks, source-backed
integration checks and worked publication examples with explicit acceptance conditions.

## 1. Goal and observable behavior

An asynchronous worker computes `summary = f(title, body)` while ordinary table writes continue.
If `body` changes, a new snapshot must immediately show `summary` as NULL until a result based on
the new inputs publishes. An older retained snapshot can still show its old, consistent pair.
A worker based on old inputs must never make stale output visible or destroy a newer result.

Readiness is separate from Arrow validity:

| Ready flag | Stored value | Logical value and meaning |
| --- | --- | --- |
| false | Anything | NULL; result unavailable and needs computation |
| true | NULL | NULL; computation completed with a NULL result |
| true | Non-NULL | Available computed result |

`IS NULL` matches both kinds of NULL. Work selection must inspect readiness rather than use
`output IS NULL`, or legitimate NULL results will be recomputed indefinitely. Readiness certifies
declared table dependencies under the caller's computation contract; it does not detect an
external model changing, prove correct function execution, or promise deterministic results.

## 2. Why option D

The alternatives below come from the packet's saved *Derived Columns & Source Updates* document.
They are design alternatives, not claims about current upstream APIs.

| Approach | Benefit | Tradeoff for this goal | Assessment |
| --- | --- | --- | --- |
| Synchronous computation in every source write | Outputs are always available and current | Slow functions and remote resources become prerequisites for ordinary writes | Unsuitable for expensive asynchronous functions |
| A: commit preconditions | Reject results whose inputs changed during computation | A source write after publication can still leave visible stale output | Necessary publication protection, insufficient alone |
| B: LanceDB detects changes during refresh and permits stale values | Smaller engine change; stale values remain usable | Visible output can disagree with current inputs until refresh catches up | Requires a different product contract |
| C: mask using row update versions and a column calculation version | Can hide stale output with version comparisons | Row-wide tracking can invalidate unrelated outputs; incremental publication, derived writes and raw writers need precise rules | Viable alternative, less direct for field-specific dependencies |
| D: declared dependencies, atomic clearing, mandatory masking and validated publication | Enforces the contract at the source-write boundary; all supported readers agree | Adds engine state, write bookkeeping, read semantics and maintenance obligations | Recommended for the requested contract |

Physical NULL as a pending marker cannot distinguish a completed NULL result. Separate assignment
state is required regardless of publication encoding. A flag's name has no magic meaning: its dependency
and masking configuration gives it these rules. Generic explicitly writable flags remain distinct
from dependent readiness flags.

## 3. Ownership and reconciliation with the original function design

| Layer | Responsibilities |
| --- | --- |
| Lance | Stable field/flag identities; dependency graph; snapshot readiness; atomic invalidation; publication admission and outcome reports; logical masking; preservation or explicit refusal of unsupported writers/readers |
| LanceDB | Exact immutable function version; binding inputs and outputs; sibling membership; atomic binding changes using Lance identities; local execution; user query policies and refresh results |
| Sophon | Distributed execution, resources, checkpoints, retries, progress and maintenance scheduling under the same table contract |

The original first-class-functions document put dependency enforcement and query policies in
LanceDB and proposed rejecting raw Lance mutations through an application capability. D changes
that boundary: supported raw Lance writes must invalidate dependencies correctly, and ordinary
Lance reads must mask. LanceDB can add `error` and `skip` consumption policies above that baseline;
those policies and the proposed function APIs are not implemented by this prototype.

The product may still need to restrict ordinary writes to managed outputs. The prototype instead
invalidates an output written outside its validated publication. Agree that application policy
explicitly; do not present the original managed-output rejection rule as current behavior.

Replacing a flag allocates a new ID and fences old publications. That provides an engine-level
identity boundary, but does not implement a function catalog, a durable function sibling group,
or atomic binding metadata replacement. Integration must install/reset the relevant flags and
the exact function binding together. A worker-side check before or after publication is too late
to establish that atomicity.

Prefer a Boolean readiness bit and a function version recorded once in the snapshot binding.
Per-cell function-version provenance is warranted only if mixed versions within one output and
snapshot become a product requirement. This is a recommendation; provenance is not implemented.

## 4. Proposed contract and implemented mechanism

### Registration and source writes

Dependencies are row-local, within one table, and refer to stable top-level field IDs. The current
implementation supports several inputs, independent outputs and acyclic chains. Cycles and a
second dependent flag on the same output are refused. Stable row IDs are optional and are not
interchangeable with physical row addresses.

A dependent flag watches its inputs and its own output. A supported logical write clears affected
flags and descendants in the same commit as the data write. An explicit source write counts even
if the bytes are equal. Physically copying an unchanged field is not a logical input write.
Invalidation resolves the latest registry on every commit attempt, including registrations made
after a writer staged its data.

The transaction records an invalidation event even if the flag was already false. Looking only
at the current true-set would miss inputs changing while a worker was computing, including
`A -> B -> A` updates. Row-moving writes carry unaffected state to new addresses and account for
vacated old positions.

### Compute, stage and publish

1. Read inputs from fixed snapshot R and retain its output/flag identities.
2. Compute outside Lance. Stream `_rowaddr` and output arrays to `PublicationStager`.
3. Column presence and optional assignment masks identify computed cells, including computed NULLs.
4. The stager copies unassigned cells from R through Lance's logical reads and writes one
   full-fragment file containing its declared outputs for each fragment with assigned work.
5. Commit an ordinary `DataReplacement` plus typed true assignments. Validate against relevant
   commits after R and the latest registration state. A manifest retry never advances R.
6. Consume the outcome report, then follow up at exactly the version it names.

Typed flag changes travel on existing transactions; registration uses `UpdateConfig`. There is no
special output value and no UDF runtime inside Lance. The stager keeps overlapping computed batches
and a small number of copy windows rather than collecting a whole fragment. Caller batch sizes and
the scan's own decode/IO read-ahead also affect memory. Late streaming errors can follow staged writes: no publication
does not mean no bytes were written.

Copy-through is guaranteed by the stager. A hand-built publication must satisfy it itself; Lance
cannot prove that unassigned file contents are unchanged. It also cannot prove that computed
values came from R's inputs. Prefer the stager for application integration while keeping this
trust boundary visible in the lower-level API.

### Conflict policy and reports

`Reject` is the default: unsafe publication fails without partial visibility. Deleted assignments
are an exception: they can be omitted and reported as `RowVacated` while the rest commits.
`Skip` is explicitly limited to output-and-flag publications through `execute_with_report`;
it is not arbitrary partial application of mixed transactions.

`Skip` can install a safe file while withholding stale rows' assignments, or defer an unsafe
file/group entirely. A physical file that would overwrite a newer output anywhere in its
footprint is unsafe, including rows copied through rather than computed. Dropping only the flag
assignments would not protect that newer result.

| Report category | Meaning for follow-up |
| --- | --- |
| Published | Values and true assignments committed |
| Reusable staged rows | Values certified against the checked snapshot; can be reused when the follow-up's inputs permit |
| `InputChanged` | Recompute from current inputs |
| `UpstreamNotPublished` | Recompute downstream from the upstream actually used by the follow-up |
| `RowVacated` | Old address is no longer live; discover moved pending rows at their new addresses |
| `NewerResult` / `OutputWritten` | File was deferred; keep the winner, use per-output row classifications to plan remaining work |
| `FragmentRemoved` / `FragmentRewritten` | Staged physical positions are unusable; do not certify those rows as reusable |

A report is snapshot-bound, not a permanent freshness certificate. `follow_up` uses
`committed_version.unwrap_or(checked_version)`, normally excludes already-ready rows, propagates
upstream recomputation even to ready downstream rows, and leaves vacated addresses out. A subsequent live pending-row scan finds moved
rows. An empty follow-up plan therefore does not prove that every current live row is ready.

### The concurrency cases that define correctness

| Controlled commit ordering | Required result |
| --- | --- |
| Publish, then source write | New source and masked output become visible together |
| Source write, then old publication | Changed rows cannot become ready from old work |
| Clear while already false; inputs change and change back | Old work still loses eligibility |
| Source write staged before publication, committed afterward | Latest ready state is cleared correctly |
| New worker publishes, then old worker finishes | New values and readiness survive over the old file's entire footprint |
| Commit loses its manifest slot repeatedly | Checks retain the original input basis and all intervening changes |
| Registration replaced/dropped during work | Obsolete flag identities cannot publish |
| Rows move or are deleted | No resurrection or reuse of vacated physical addresses |
| Upstream and downstream staged together; file deferred | Downstream based on the unpublished upstream is not reusable |
| Competitor republishes upstream and downstream together | Upstream publication is an input change even if no downstream clear was recorded |

Chains add a caller obligation: a downstream assigned alongside its upstream must use the new
upstream value in that publication. A follow-up that republishes upstream must recompute the
relevant downstream, even if its old staged value otherwise appeared reusable. The packet's
adversarial reproducer exposed exactly this issue; current tests cover the corrected reports.

## 5. Representation and API tradeoffs

These recommendations distinguish an experimental default from a production commitment.

| Decision | Current choice and benefit | Cost / alternative | Recommendation and decision trigger |
| --- | --- | --- | --- |
| Publication representation | Ordinary `DataReplacement` plus typed flag changes reuses commit machinery | Safe eligibility is row-level, but physical publication is whole-fragment; a dedicated primitive could express mapping, group identity and reconciliation directly | Keep the current primitive for the bounded prototype. Specify a new operation only if required mapping/lifecycle contracts cannot remain clear and safe on the existing API |
| Publication granularity | Full output columns have simple mapping and bounded copy-through | Sparse work rewrites unrelated rows and one output race can defer a whole file. Filtered staged-value overlays or an explicit immutable value mapping can preserve outside rows | Decide whether fragment fallback is acceptable for the first useful workload. If not, write a separate row-selective publication design before more storage implementation |
| Output grouping | A stager's outputs publish atomically in one file | Grouping independent outputs couples their conflicts; Lance does not know which outputs are siblings of a function | Use the smallest required atomic group; define sibling membership in binding metadata and enforce it on integration |
| Flag state storage | Inline compressed true-sets avoid new objects, caches and cleanup paths | Every open decodes state; commits serialize it; holes and additional flags grow metadata. External immutable per-fragment state reduces rewritten metadata but adds requests and lifecycle work | Retain inline state as the experimental baseline. Compare inline, external and threshold-based spill on representative storage before selecting a format |
| Row identity | Physical fragment offsets match current replacement files | Moves require explicit state carry and invalidate staged positions; stable row IDs do not themselves repair file alignment | Keep `_rowaddr` as an explicit snapshot identity in this API. Require verified mapping or deferral for every physical rewrite |
| Dependency scope | Chains are implemented and covered by adversarial tests | They complicate invalidation, same-publication provenance and reuse | Preserve existing guarantees and tests. Decide whether public function bindings expose chains initially; a narrower product surface need not discard the engine work |
| Rejection vs selective progress | `Reject` is easy to consume; `Skip` retains safe progress and exposes reuse | `Skip` needs report-driven orchestration. `Reject` stops early and provides no complete reuse certificate | Keep both; use `Skip` for expensive asynchronous jobs after the executor consumes reports correctly. Do not equate rejection with mandatory full recomputation |
| Reusable staging lifecycle | Unreferenced files are eventually collected | Long jobs/retries can accumulate or lose work; no lease protects deferred files | Specify ownership, discard, history retention and expiry before durable checkpoint integration. A lease buys reuse guarantees at the cost of lifecycle metadata |

Existing overlays cannot be made row-selective by simply subtracting coverage bits: value positions
are determined by rank in the coverage bitmap. Removing a middle bit shifts later positions.
Either rewrite/filter staged values into an aligned file, without rerunning the function, or
specify separate immutable stored-row mapping and effective coverage. Both need conflict-footprint,
deletion, compaction, index and cleanup rules.

External flag storage addresses metadata, not sparse output write amplification or the partial-mask
processing path. These are separate decisions and should have separate evidence.

### Ordinary DataReplacement partial coverage

The prototype lets a stored output and a newly added metadata-only all-NULL output publish together:
it tombstones the stored V2 fields and appends their combined file. This avoids an extra publication
and permits atomic siblings, but broadens ordinary `DataReplacement` behavior for every caller.
It requires an independent maintainer review of field/schema coverage, nested mappings, versions,
row counts and overlay supersession. The builder still has an existing TODO to check replacement
file length; stager alignment does not establish validation for arbitrary callers.

The alternatives are to accept and validate this general rule, constrain the extension to an
explicitly supported publication contract, or refuse mixed layouts. Publishing siblings separately
changes their atomicity and is not an equivalent fallback. Keep this change independently reviewable.

## 6. Read masking and vectors

Masking belongs in logical value resolution before predicates, aggregates, ordering, SQL, take
and search consume the output. Query indexes and statistics must never bypass it. The current
prototype refuses indexes on masked outputs instead of maintaining their coverage.

**Settled decision: vector NULLs mask only parent list validity.** Preserve ordinary Arrow NULL
semantics and the Torch/KMeans validity-and-slice fixes. Stored child values can remain under a
NULL slot; readers must honor the parent bitmap. This is logical masking, not secure erasure.
Removing child-nulling avoided a measured large copy/allocation cost on the old machine; those
old 10–18x figures describe a superseded implementation, not current vector masking overhead.

The current Torch adapter represents NULL floating vectors as NaN rows. Nullable integer tensors
are a separate unresolved policy: casting a NULL-containing batch to float64 changes dtype and
can lose integer precision above `2**53`. Choices include preserving integers with a separate
validity mask, explicit refusal without a converter, or an explicitly accepted lossy conversion.
Prefer lossless values plus validity in a future API, but review its compatibility and tensor
consumer requirements before changing the existing adapter. Parent-only masking remains fixed.

## 7. What the measurements tell the design review

The Linux continuation compares the same generated data/history and matched before/after builds
on one Ryzen 9 3900X host. Source is on CIFS; timed data/builds are local tmpfs. These are warm
processing measurements, not network-mount, SSD, S3, cold-cache or wheel-build acceptance.
Function computation is excluded. Absolute times below are pooled sample medians; the paired
ratios and confidence intervals are in [CONTINUATION.md](CONTINUATION.md).

| Workload, 1M live rows | Before -> after, ms | Equivalent plain workload after, ms | Design implication |
| --- | ---: | ---: | --- |
| All-ready scan after row movement | 45.23 -> 22.95 | 23.39, same movement | Deleted holes do not require per-row masking of valid live rows |
| Wholly masked scalar scan | 8.44 -> 1.06 | 1.67, physically stored all-NULL | Hidden outputs can use a NULL reader; data IO fell from 14.00 MiB to zero |
| Fully assigned backfill stage, one output | 246.71 -> 127.23 | 119.25, `write_columns` | Most of the measured former backfill processing cost was avoidable |
| Fully assigned backfill, shared / chained pair | 401.04 / 403.51 -> 158.14 / 151.92 | 144.89 / 145.81 | Bulk slices help without changing concurrency or bounded streaming |
| Scalar scan, 1% masked | 16.50 -> 17.20 | 8.78, equivalent stored NULLs | Partial-mask processing remains unresolved |
| Stage 100 scattered rows in place | 146.40 -> 149.26 | 56.21, plain merge staging | No demonstrated sparse-stage gain; whole columns still rewrite |

The large improvements exceed identical-binary controls. Mutation controls are noisy enough that
small differences are not established. This phase compares combined optimizations and does not
isolate each commit's contribution. Allocation measurements count requested capacity and peak
live growth, not RSS; they exclude precomputed buffers already live at phase start.

Sparse staging still writes about **13.8 MiB per output for 100 results**. The plain in-place
comparison also rewrites whole columns, so amplification is a representation cost, not all a
flag-specific penalty. Lower CPU overhead can improve that path without lowering its bytes.

The earlier feature-wide Mac measurements add evidence the Linux phase did not rerun: dense
100k-row source updates added roughly 15 ms; inline state reached roughly 163 KiB per flag;
pending-row discovery enumerated all live addresses; and fragment count affected sparse staging
and follow-up cost. They identify questions for storage/scale evaluation, not portable targets.
No consistent no-flag regression was demonstrated by the latest Mac repetitions; representative
acceptance remains open. Follow-up time adds to the first attempt, and permissive plain races
do not provide the same freshness guarantees.

The NULL-vector flat-search slowdown and slow take after row movement occurred with and without
flags on the prototype build. They need clean-main reproduction before upstream attribution.
The independent null-bitmap-offset correctness fix is preserved separately at `8d09fb623`.

## 8. Scope and risks before integration

The implemented experimental surface includes selected nullable top-level scalar outputs and
nullable `FixedSizeList<Float16/32/64>` vectors on V2 storage, supported source writes, deletions,
multi-fragment publication, chains and report-driven follow-ups. The exact operation/error matrix
is in [README.md](README.md#supported--unsupported).

| Current boundary | Implication for a useful first release |
| --- | --- |
| All scalar, full-text and ANN indexes on masked outputs refused | Flat queries can demonstrate semantics; an embedding product needing ANN needs a separate coverage design |
| Compaction refused while flags are registered | Long-lived tables accumulate layout/state costs; remapping cannot be treated as optional for a general mutable-table release |
| Watched-field overlays, some merges, overwrite, watched-field drop/cast and other modes refused | Supported writers must be documented precisely; new writers require explicit classification |
| MemWAL/LSM with masking refused | No fresh-tier support; stale shard handles/base-table-less readers remain a documented activation gap needing closure or an enforced scope boundary |
| Rust-only flag APIs; Python/Java transaction round trips drop changes | Bindings must preserve typed changes and semantics before application publication uses them |
| No protected staged-file lease or complete checkpoint/reconciliation contract | Durable jobs need retention of staged work and validation history, plus handling of ambiguous commit outcomes |
| Unstable reader and writer gates | Release access requires the opt-in environment variable; audit supported client versions and every entry point before rollout |

The MemWAL handle gap is recorded in the packet and remains visible in the source: shard-only
readers do not consult a base table manifest. A manifest-level mutual-exclusion check alone
does not fence a handle left alive across mode changes. This review does not claim a new
end-to-end reproduction; activation/handle lifetime needs explicit validation before release.

Format/API review also includes the public transaction field, moved-row payload, sticky registry
feature bit, restore/clone behavior and stable-ID allocation. Replacing a binding must not reuse
an old publication identity. Missing validation history must fail conservatively rather than
certify unknown freshness.

Current Linux checks at `fabb89983`: 4,604 Lance library tests passed (including 442 cell-flag
cases), 580 lance-table tests, 76 doctests, formatting and Lance tests/benches Clippy. The
continuation preserves the full logs and ignored-test counts. These checks were run during the
optimization phase, not rerun for this documentation review. Historical Torch/MSRV/aarch64 checks
are separately dated; current Torch, Java, full bindings, parallel storage writers and production
performance acceptance remain unverified.

## 9. Decisions and next work

1. **Review the logical contract and ownership.** Confirm D, the caller trust boundary, the
   original snapshot through retries, parent-only vector masking, and required atomic sibling
   behavior. Reconcile the original function design's raw-writer and query-policy rules.
2. **Choose the first useful workload and supported surface.** Decide whether flat scalar/vector
   use with fragment fallback is enough for an experimental integration. If ANN, compaction or
   long-lived checkpoints are required, those designs are prerequisites.
3. **Resolve publication and state storage independently.** Use sparse refresh density, raced
   fragment counts, output width, metadata size and storage requests as decision criteria.
   State the acceptable write/retry amplification; do not silently turn an optimization into
   a new publication format.
4. **Review the general DataReplacement extension and integration contract.** Define atomic
   binding reset, sibling enforcement, typed binding round trips, retained validation history,
   staged-file ownership and commit-outcome reconciliation.
5. **Validate that chosen design.** Agree workload-specific latency, bytes, reuse and memory
   budgets; use representative Linux deployment builds and intended storage with cold/warm
   reads and concurrent writers. Keep the existing controlled-order concurrency regressions.
   Optimize partial scalar masking and sparse-stage processing if they threaten those budgets.

Split later integration into reviewable changes: independent flat-search correctness fix;
ordinary replacement semantics; flag model/gates and invalidation; publication/report safety;
masking and adapters; stager; then bindings/application integration. Preserve their shared
regressions and the frozen experimental baseline. No push, PR or Sophon pin change is part of
this review.

## 10. Packet coverage and evidence map

The review used the full packet inventory, restoration/context/continuation instructions, original
function design, Weston's A–D alternatives, option-D draft, September 17 advice, four saved agent
reports, historical design handoff/plan/research log, benchmark methods/analyses and relevant
current implementation/test paths. All 1,320 checksummed payload files matched. All 25 JSON files
and 802 JSONL files parsed successfully, covering 167,264 records including run and warm-up records.
Those integrity checks are not a rerun of the historical experiments or a formal audit of every
line in the bundle's Git history.

| Evidence | Role and precedence |
| --- | --- |
| Transfer `START_HERE.md`, `CONTEXT.md`, `NEW_AGENT_PROMPT.md`, reference snapshots and agent reports | Goal, explicit decisions, alternatives and earlier recommendations; supplied dated snapshots, no live external lookup |
| [Transferred design handoff](DESIGN_REVIEW_2026-09-30.md), [PLAN.md](PLAN.md), [README.md](README.md) | Preserved implementation history and detailed contracts; older head/test/performance claims need their revision context |
| [Feature-wide costs](bench/results/feature-costs/README.md) | Latest transferred scalar/vector/writes/races/history/scale evidence on the Mac |
| [No-flag investigation](bench/results/10m-noflag-investigation/README.md), [vector investigation](bench/results/vector-masking/README.md), [REPORT.md](bench/REPORT.md) | Controls and superseded experiments; neither old no-flag percentages nor child-nulling costs are current acceptance claims |
| [CONTINUATION.md](CONTINUATION.md), [Linux records](bench/results/linux-local-optimizations/README.md) | Current local optimization evidence, exact source/build identities and test logs |
| `rust/lance-table/src/{format/cell_flag.rs,transaction/cell_flag_commit.rs,transaction/manifest_build.rs}` | Persisted state, invalidation/gates and ordinary replacement behavior |
| `rust/lance/src/{io/commit.rs,io/commit/conflict_resolver/publication.rs,dataset/cell_flag/}` | Fixed snapshot, publication validation, reports, staging and follow-ups |
| `rust/lance/src/dataset/{fragment.rs,fragment/cell_flag_mask.rs,tests/dataset_cell_flags*.rs}` and Torch adapters | Logical reads, local fast paths, adversarial regressions and validity-aware consumers |

Precedence corrections incorporated here: chains and ordinary transactions supersede the early
no-chain/special-operation advice; parent validity supersedes child-nulling; mixed layouts are
now supported by the prototype; Linux checks supersede historical “x86_64 not run” statements;
late streaming errors may leave bytes; no storage/performance release decision has been made.
