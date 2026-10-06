# Atomic binding reset: characterization

Handoff copy: original report at local revision `cb288925b`. The added experiment tests and
their branch remain local with Brendan; this published prototype includes the report only.
Source links below refer to the production files, not the unpublished added test cases.

2026-10-05. Coding task 1 of [CODING_PLAN.md](CODING_PLAN.md). **Unstable research prototype.**
Base `16413f22b`. The test is commit `8008fbe64` on the local branch
`brendan/cell-flag-binding-reset`. No production code changed, and nothing was pushed.

**Question:** can the current Rust transaction API change a function-binding record and reset
its outputs' readiness flags atomically, without a new engine primitive?

**Answer:** yes, for the tested contract, provided the application is the only writer of the
binding record and meets the obligations below. One `UpdateConfig` carrying the record and
`CellFlagChanges` is atomic. It fences old workers twice over, clears the downstream output and
refuses competing resets. Lance cannot tell a binding record from any other metadata key. The
three negative controls show what it accepts that the application must prevent.

## Fixture

The tests follow the sibling-output tests in
[`dataset_cell_flags_publication.rs`](../../rust/lance/src/dataset/tests/dataset_cell_flags_publication.rs).
The binding record is test-only, not a catalog. Table metadata `binding/{name}` holds
`{function, inputs, outputs}` with stable field ids. Each output's slot is the flag named `ready`
on that field. The record never stores a flag id.

| Binding | Inputs | Outputs | Flag per output |
| --- | --- | --- | --- |
| `summarize@1` | title, body | summary, keywords (siblings, one file per fragment) | dependent, masking |
| `digest@1` | summary | digest (existing downstream) | dependent, masking |

A reset is one `UpdateConfig` with `table_metadata_updates` for the record and
`CellFlagChanges { drops, registrations }`. Drops are the ids the old record resolves to in the
reset's read snapshot. The commit allocates new ids. Tests resolve them through the record's
slots in the committed snapshot and never predict them.

Every value names its function version and inputs, for example
`summarize@1:summary(t1,b1)` and `digest@1:digest(summarize@1:summary(t1,b1))`.
`check_bindings` walks every version. It fails on a bound output without its flag, a flag not
watching its binding's inputs, a dependent flag no binding names, one flag id seen under two
function versions, or a visible output that the snapshot's binding would not compute from the
inputs shown with it.

## Reproduced behavior

R is the version the workers read; the reset commits R+1 unless stated.

| Scenario | Observed |
| --- | --- |
| Successful reset | One new version holds the `summarize@2` record and two new ids, disjoint from the old ones. Both siblings and digest have no true rows, and all three outputs read NULL. The reset's transaction records `cleared(digest, every row)`. Version R still reads its `summarize@1` values. |
| Old sibling worker (`Reject`, `Skip`) | `IncompatibleTransaction`: `cell flag S1 is not registered since version R+1`. With ids re-resolved at the head but its true read version: `IncompatibleTransaction`, `... was not registered at version R`. Nothing commits. |
| Old downstream worker (digest keeps its id) | `Reject`: `RetryableCommitConflict`, `concurrent UpdateConfig at version R+1 changed their inputs`. `Skip`: commits R+2, installing its stale file under a false flag, and defers all four rows as `InputChanged` at R+1. Digest stays pending. |
| Body write staged with the reset, committed after it | The write records clears for the **new** sibling ids and digest on fragment 0. A worker that read the reset publishes fragment 1 under `Skip`; fragment 0 is deferred as `InputChanged` at the write's version. |
| Body write takes the reset's first manifest slot | The write commits with clears for the old ids. The reset rebases, retries, commits R+2 and records `cleared(digest, every row)`. A worker that read the reset publishes every row. |
| Competing resets | The loser gets `IncompatibleTransaction`, `incompatible with concurrent transaction UpdateConfig at version W`, from the shared metadata key. Its changes resubmitted with read version W fail too: `cell flag S1 is not registered at the version this transaction commits on ... so its drop cannot apply`. The winner's record and ids are unchanged. Rebuilt from W, the reset commits with new ids. |
| Refused registration: a cyclic new input, or a sibling not dropped | `InvalidInput` (`would make the dependencies cyclic` / `already has a flag of that name`). It is raised after the drops have applied to the manifest being built, and in the cyclic case after one sibling's registration too. No version is added. The record, registry definitions, allocator, states and values are unchanged, and the old worker still publishes. The attempt's `_transactions/{read}-{uuid}.txn` remains, referenced by no manifest. |

The history walk passes after every scenario. No version pairs a record with another
generation's ids, and no visible output disagrees with its snapshot's binding.

### Negative controls: Lance accepts these, and the history walk reports them

| Commit | What Lance does | First violation reported |
| --- | --- | --- |
| Record rewritten to `summarize@2` by plain `update_metadata` | Commits. The old ids stay ready, and an old worker then publishes `summarize@1` values. | Flag S1 registered for `summarize@1`; record binds `summarize@2` |
| Reset drops both sibling flags but registers only summary's | Commits. Keywords is unmasked and shows its stored `summarize@1:keywords(t1,b1)`. | Keywords has no `ready` flag |
| Worker computed under `summarize@1`, restaged against the reset with the new ids | Publishes | Summary reads `summarize@1:...` where `summarize@2` computes otherwise |

### Mutation checks (temporary, reverted)

Each mutation is one line, applied alone (M2 and M3 touch disjoint tests and shared a build).

| Mutation | New cases failing | Meaning |
| --- | --- | --- |
| M1, [`cell_flag_commit.rs:333`](../../rust/lance-table/src/transaction/cell_flag_commit.rs): `registered.chain(dropped).filter(\|_\| false).collect()`, so dropping or registering a masking flag no longer changes downstream inputs | 6 of 10, plus 1 existing | This rule alone clears digest. Without it the walk reports digest visible over a masked summary. |
| M2, [`publication.rs:235`](../../rust/lance/src/io/commit/conflict_resolver/publication.rs): `changes.drops.iter().filter(\|_\| false)`, disabling the conflict-time fence | 2 of 10, message only | The old worker is still refused, by the apply-time unknown-flag check: there are two independent fences. |
| M3, [`conflict_resolver.rs:1993`](../../rust/lance/src/io/commit/conflict_resolver.rs): `modifies_same_metadata(...) && false` | 1 of 10, message only | The losing reset is still refused, because its drops name ids the winner retired. |

## Source inspection only

- Flag changes and metadata updates apply to one in-memory manifest
  ([`manifest_build.rs:1488`, `:1541`](../../rust/lance-table/src/transaction/manifest_build.rs)).
  An error from either aborts the commit.
- The attempt writes its transaction file ([`commit.rs:1737`](../../rust/lance/src/io/commit.rs))
  before the manifest build validates registrations (`:1751`), and does not delete it on that
  error. Cleanup removes unreferenced `.txn` files after its 7-day grace period
  ([`cleanup.rs:1157`](../../rust/lance/src/dataset/cleanup.rs)).
- Only **masking** flags count as remasking
  ([`cell_flag_commit.rs:320-335`](../../rust/lance-table/src/transaction/cell_flag_commit.rs)).
  Resetting a dependent but non-masking output leaves its downstream ready on the still visible
  old values.
- A downstream worker that reads while its upstream is masked computes from NULL, and Lance
  accepts it; the next upstream publication clears it again.
- Registrations and drops require `UpdateConfig` (`cell_flag_commit.rs:544`), and a flagged field
  cannot be dropped before its flag (`:617`). Adding an output needs a schema commit first, which
  leaves an unbound all-NULL column, a consistent state. Removing an output cannot drop the flag
  and the column in one commit, and dropping the mask exposes the stored values.
- Python and Java transaction conversion drop `CellFlagChanges`
  ([`python/src/transaction.rs:921`](../../python/src/transaction.rs),
  [`java/lance-jni/src/transaction.rs:1507`](../../java/lance-jni/src/transaction.rs)).

## Conclusion

**Existing primitives guarantee** (reproduced):

- the record and the flag generation become visible atomically, in one manifest;
- new ids are allocated at commit, never reused, and named only by the committed snapshot;
- siblings become pending together and are masked in the reset's own version;
- downstream readiness clears on every row, and downstream work computed before the reset is
  rejected or deferred as `InputChanged`;
- two independent fences refuse old workers under both policies;
- a racing source write's clears are resolved against the latest registry on every attempt;
- competing resets are refused by the metadata key and, independently, by drops of retired ids;
- a refused registration changes nothing visible.

**Integration must enforce:**

1. **One writer path for binding records.** Route every change through the reset transaction and
   refuse raw writes to the record's key namespace. A record-only change leaves old ids live.
2. **The complete sibling set,** derived from the old and new records together. A missing
   registration exposes old stored values, and a missing drop is refused.
3. **Same-snapshot work.** Read the record, flag ids and inputs from one snapshot R, compute with
   R's function version and publish with read version R. Lance certifies inputs, not the
   function that computed the values.
4. **Drops resolved from the reset's own read snapshot.** Each drop then acts as an expected
   generation. Resolving current ids at commit time would remove that protection.
5. **Retry classification.** Re-read and re-evaluate after `IncompatibleTransaction`; fix the
   binding after `InvalidInput`. Retry neither blindly.
6. **Policies outside the tested path** (source inspection):
    - clear downstream explicitly (`value: false`, which propagates) when a bound output is
      dependent but not masking;
    - schedule downstream work only after its upstream is ready;
    - remove an output by overwriting it with NULL while masked, or accept exposure;
    - issue resets from Rust until the bindings round-trip typed changes.

**Additional engine primitive:** not necessary for this contract. In particular, commit-time id
resolution into the record, a metadata compare-and-swap and a dedicated binding operation are
unnecessary. A primitive would become necessary only if Lance itself must enforce binding
consistency against non-cooperating writers (the negative controls), or must remove an output and
its flag atomically. Both need an engine-known binding or sibling concept, which is a format
decision for the review rather than part of this task.

## Proposed changes (not implemented)

| Proposal | Benefit | Cost |
| --- | --- | --- |
| Delete the attempt's transaction file when the manifest build refuses it, or validate registrations before writing it | Refused resets leave no orphan bytes | A small change on the commit error path |
| Encode the generation in the flag name, such as `ready@summarize@2` (integration only) | Each snapshot is self-checking: a record-only change fails slot resolution instead of reusing old ids | Does not stop exposure or stale workers; longer names |
| An engine-owned binding/sibling record, validated with its flag resets | Enforcement against raw writers, and atomic output removal | A new storage concept with lifecycle rules |

## Verification

On the Linux host of [CONTINUATION.md](CONTINUATION.md): Rust 1.97.0, `ci` profile, source on
CIFS, build target under `/tmp/cell-flags-binding-reset`.

| Check | Result |
| --- | --- |
| Base `16413f22b`, cell-flag suites (`dataset::tests::dataset_cell_flags`) | 399 passed, 1 ignored |
| With `8008fbe64`, same suites | 409 passed, 1 ignored; the 10 new cases take 0.49 s serially |
| M1, M2 + M3 | As tabulated; reverted, tree clean |
| `cargo fmt --all -- --check`, `git diff --check` | Passed |
| `cargo clippy -p lance --tests --profile ci --locked -j 12 -- -D warnings` | Passed |

Not run: the rest of the `lance` library suite (the full test binary compiled), `lance-table`
tests (unchanged), Python and Java.

```sh
cd /home/brendan/work/lance-cell-flags
git worktree add /home/brendan/work/lance-cell-flags-binding-reset brendan/cell-flag-binding-reset
cd /home/brendan/work/lance-cell-flags-binding-reset
mkdir -p /tmp/cell-flags-binding-reset/tools
cp prototypes/dependent-cell-flags/bench/protoc_network.py /tmp/cell-flags-binding-reset/tools/protoc-network
chmod +x /tmp/cell-flags-binding-reset/tools/protoc-network
export PROTOC=/tmp/cell-flags-binding-reset/tools/protoc-network
export CARGO_TARGET_DIR=/tmp/cell-flags-binding-reset/target LANCE_ENABLE_UNSTABLE_CELL_FLAGS=1
cargo test -p lance --lib --profile ci --locked -j 12 -- dataset::tests::dataset_cell_flags
```
