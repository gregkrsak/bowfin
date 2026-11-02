# BED-027 / BD-042 implementation ledger

Issue: https://github.com/gregkrsak/klyxr/issues/60
Branch: `ked-027-deterministic-scope-cleanup`
Starting commit: `8d5b60ac9f46ba44d6df0392b505ebf8343eba5e`
Starting tree: `0d73ae4ed7ba8d11f4c62896193cdbacee57db63`

Greg authorized GitHub Actions as the authoritative executable environment during
K27-I01. Local formatting, checking and Clippy are advisory only. No local test
or conformance claim is made. Command-line push lacked credentials; branch
creation and subsequent publication use the connected GitHub Git-data API.

## Exact predecessor baseline

The unchanged branch triggered all four push workflows at the exact starting SHA.
All passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38296578935 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38296578865 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38296578897 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38296578863 |

Tests: 1,580 unit/integration identities plus the `CheckedExecutable`
compile-fail doctest, all passing in debug and release (1,581 in each).
No failed, ignored or filtered predecessor tests. Rust/Cargo 1.80.0;
workspace/all-target checking, formatting, Clippy with warnings denied,
README assets, accepted/rejected battery CLI checks and the CLI suite passed.
The affine exhaustive-domain cross-check ran in both profiles.

## Milestone 1 — canonical resource foundations

Private canonical resource and trusted acquisition/release identities; structural
owned field types for internal fixtures; bounded acyclic, shared reverse-declaration
cleanup plans. Source record declarations still admit named-range fields only.
Reference fields and public resource registration are not admitted. Every record
and trusted leaf remains Move under the existing single ownership checker.

Nine new catalog controls cover reverse field order, name independence, inert Move
records, foreign identities, invalid trusted pairing, duplicate leaves, borrowed
release parameters, cycles, and foreign fields. These are classification/schema
controls, not resource execution evidence. Existing predecessor assertions retain
their range-identity and malformed-metadata witnesses through the explicit
`Field::range_type()` accessor after structural field typing.

Runtime cleanup, lexical activation, resource payloads and synthetic execution are
not implemented at this milestone. No PR or merge is authorized by this ledger.

Milestone 1 commit: `60cee544039ecd22b7ed01576490231e2a17cfd1`.
All four push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38297326807 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38297326828 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38297326804 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38297326820 |

Debug/release: 1,590 passed each. All 1,581 predecessor identities and outcomes
remain present; nine new identities passed in each profile. No failures or ignored
tests. Local fmt/check/Clippy passed as advisory only.

## Milestone 2 — lexical MIR cleanup metadata

The existing structured lowerer now emits private MIR scope descriptors, static
scope-entry sites, explicit successful-initialization sites, exact inner-to-outer
exit sets and synthesized origins. Function metadata retains reverse-ordered owned
record parameters and a protected-return boundary. Resource-free source lowering
retains its previous representation with no cleanup metadata.

MIR validation rejects duplicate exits, missing activations and missing cleanup
origins. Opaque execution correspondence compares the new metadata as part of the
complete deterministic MIR artifact. Admission bounds the new vectors before
recursive comparison. Trusted acquisition/release stubs reject arbitrary user
bodies, and canonical fixture construction repeats ordinary HIR validation and
ownership checking before lowering. Four new lowering/malformed-metadata controls
are authored; runtime cleanup is still pending.

Milestone 2 commit: `0bf7542f029040ce270c57b0ca398045e9df5a8f`.
All four push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38297981588 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38297981569 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38297981517 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38297981600 |

Debug/release: 1,594 passed each (1,581 predecessors plus thirteen new tests).
Local fmt/check/Clippy passed as advisory only.

## Milestone 3 — private synthetic leaf vertical slice

A non-cloneable runtime leaf retains canonical resource/type identity and a fresh
acquisition identity. Fixed trusted operations dispatch by FunctionId and catalog
identity; they do not execute the stub body or invoke callbacks. Scope activation
histories come from explicit MIR, including successful initialization and exit
sets. A transferred resource slot is distinguished from mere vacancy; cleanup
rejects an invalid owner state. Only resource-bearing local initialization and
parameter identities participate in cleanup.

The protected result, current cleanup payload, event trace, activation histories
and frame metadata are included in runtime storage accounting. Acquisition,
release attempts/completions/failures and frame retirement have a deterministic
private event trace. No source-visible production API or live-resource public
result is admitted. Private fixtures re-enter canonical admission, ownership,
lowering, opaque approval and correspondence; they are not public constructors.

Seven new vertical controls cover implicit completion, explicit NoValue return,
borrowed handles, name-independent trusted dispatch, public-result rejection,
missing cleanup, and forged cleanup plans. Aggregate payloads, pending construction
and the full boundary/failure batteries remain later milestones.

Milestone 3 commit: `2127503186654e618260fad063df9509e4ab4f34`.
All four push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38298735533 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38298735645 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38298735624 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38298735531 |

Debug/release: 1,601 passed each (1,581 predecessors plus twenty new tests).
No local executable evidence is used.

## Milestones 4–5 — transfer and structured-exit conformance

The vertical runtime already includes the inseparable transfer, parameter and
protected-result machinery. This milestone independently exercises those contracts
and structured control flow before aggregate payloads are added. An explicit guard
rejects reuse of an owner slot before its activation has completed cleanup.
Terminal trusted-release failure cannot resume scope cleanup.

Eighteen new tests cover same-scope A/B/C transfer ordering, explicit release,
reverse parameters, locals-before-parameters, parameter moves, protected returns,
parameter return, both branch-dependent consumption paths, skipped and executed
short-circuit consumption, nested fallthrough, repeated loop iterations, continue,
break, nested loops, nested return, conditional initialization and borrow-rule
preservation. No language semantics or predecessor assertion is weakened.

Milestones 4–5 initial commit: `2fbc0720f955a9da9d90abc6b7eb69bc85326ce0`.
Build/docs/verify passed. Tests run
https://github.com/gregkrsak/klyxr/actions/runs/38299407839 failed two new
short-circuit fixtures during resolution: source spelled the accepted built-in
`bool` as `Bool`. Classified as a test-construction defect, not an implementation
or semantic defect. The correction changes only that spelling and preserves both
skipped-consumption and executed-consumption assertions. No later architectural
layer is being added until the corrected complete workflow gate passes.

Milestones 4–5 correction: `85a122266e0cde4897b15192d7fd91982e9f0366`.
All four corrected push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38299632722 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38299632462 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38299633208 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38299632633 |

## Milestone 6 — aggregates and pending construction

Private aggregate payloads cannot clone resource leaves. Pending constructions
retain canonical record/field IDs, exact initialized membership, owned field
payloads and completion state. Each field is evaluated and committed in written
order; complete publication requires the full canonical field set. Cleanup uses
shared reverse-declaration plans and explicit bounded continuations.

The private synthetic failure control aborts pending construction and then stops
the tool. It is separate from range traps, configured stops, integrity failures
and host failures, and adds no recoverable source syntax, general propagation or
unwinding. Initialized pending resource fields clean in reverse canonical order,
not evaluation order. Nested pending containers abort inside-out. Moved sources
remain transferred and are never restored.

Retained detached payloads, pending fields, aggregate buffers, cleanup work,
protected return results and synthetic events participate in prospective storage
accounting. Field initialization, cleanup traversal and fixed release charge fuel.
Ten new aggregate controls cover complete reverse-order cleanup, a non-last
resource field, the exact third/first/second-fails witness, aggregate move/return,
nested complete/pending structures, source non-restoration, projection rejection
and incomplete-publication rejection.

Advisory local checking/Clippy completed but also reported a corrupt incremental
cache artifact. No repair, target experiment or local executable test was attempted;
GitHub remains the only authoritative executable environment.

Milestone 6 initial commit: `bf03f406220397508c84135f348c346d2bef8cf3`.
Build/docs/verify passed. Tests run
https://github.com/gregkrsak/klyxr/actions/runs/38301192993 stopped ten new aggregate
fixtures: the private fixture's placeholder-field removal renumbered the canonical
field table but had not remapped every expression and additional record. Classified
as fixture-translation failure against the existing canonical FieldId contract.
The correction remaps constructor/read/write field identities and record member
vectors; no cleanup-order, partial-construction or source-restoration assertion is
changed. Later milestones remain gated on a complete corrected workflow result.

Milestone 6 correction: `c25b4dad318bc60aa32141d04729b085197f9de7`.
All four corrected push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38301659243 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38301659298 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38301659138 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38301659270 |

## Milestone 7 — bounds, malformed artifacts and reporting

Only functions with owned resource locals/parameters or protected resource results
receive cleanup flow. The resource runtime starts lazily, so an unused private
catalog retains resource-free fuel and storage behavior. Resource-free callers can
invoke resource-owning helpers; reference-only callees carry no responsibility.

Canonical admission checks initializer/destination and statement-call parameter
identity, rejects undefined resource argument temporaries, bounds newly registered
metadata/nodes/type depth, and reserves the expanded frontend envelope. Synthesized
scope entries retain the source scope's span; empty function cleanup no longer
uses a generic start-of-file placeholder. Resource/Copy mismatches cannot obtain
approval. The payload enum remains non-cloneable and its Copy accessor rejects
leaves and aggregates.

Twenty-four new tests cover exact sufficient/insufficient fuel and storage,
controlled release failure and no retry, protected-return retention, resource-free
accounting, canonical Copy conflicts, wrong/foreign owners, wrong scopes and exit
sets, invalid initialization/origins, inert/reference cleanup targets, loop-edge
omission/reordering, foreign/reordered plans, trusted-operation pairing, undefined
temporaries, graph/name bounds, replacement rejection, renderer allocation failure,
writer failure, and conservation across every fuel stop in the A/B/C transfer
witness. Matrix tests preserve multiple independently malformed controls.

Milestone 7 commit: `c3b38909ab3a1662d0da93f05f26a5b84278d7cc`.
All four push workflows passed:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38305366758 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38305366703 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38305366688 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38305366727 |

Debug/release: 1,653 passed each (1,581 predecessors plus 72 new tests).
Identity reconciliation against the baseline job log found every predecessor
passing in each profile, including the compile-fail doctest. Cargo stderr target
headers can arrive before the preceding stdout tests; reconciliation associates
complete test groups with the ordered target list rather than arrival timestamps.

Three further conservation controls exercise aggregate pending construction,
protected aggregate return, complete cleanup at every fuel stop, exact storage
threshold and sampled lower storage stops, and controlled host failure during
partial abort. They retain remaining leaves rather than retrying or restoring
sources. Local formatting and Clippy are advisory only.

Additional conservation commit: `ba6207aa1baa12860cb4b3047b9d4b30c789d0ba`.
Build (38305853409) and tests (38305853260) failed to compile a new retry assertion
that cloned the deliberately non-cloneable private Event log. Docs (38305853346)
and verify (38305853303) passed. Classified as an ordinary test compilation defect.
The correction snapshots Debug text and checks exact equality after the rejected
retry; it does not add Clone or alter the resource payload or event contract.
Local Clippy passed after the correction, as advisory only.

The first correction publication (`fc25ade22981441d94ada804190fbcd1670e2a14`)
contained the ledger but omitted the staged test correction because the publisher
listed only unstaged paths. No compiler design change was involved. Publication
was repaired to enumerate all paths changed from HEAD; the exact non-cloning
assertion was published at `48236c4d22b7ead4f660c413fe8a31760baba795`.
Its build (38305977868), docs (38305977877) and verify (38305977916) passed.
Tests (38305978022) failed the new aggregate storage assertion: lazy State activation
installed its accounted bytes before reserving them. Classified as an implementation
accounting defect. The repair reserves State bytes before activation; the exact
storage and unique-leaf conservation assertions remain unchanged.

Storage repair: `a747e568686dd7a52c820d792b14ee3681f8f87a`.
All four push workflows passed: build 38306153091, tests 38306153088,
docs 38306153312, verify 38306153269. Debug/release each passed 1,656 tests:
all 1,581 predecessors and 75 new identities, with no failures/ignored/filtered.
The baseline identity inventory and reconciliation script were independently
checked against both decoded GitHub profile logs; that is analysis of remote
execution evidence, not local Rust test execution.

Final source audit found a shared-plan graph-depth defect: a cached child could
skip descendant depth on a later parent walk. The repair memoizes subtree heights
and checks the full prospective depth on cache hits, retaining shared plans.
A new control rejects a three-level graph at depth two and accepts it at depth
three. No cleanup order, ownership rule or semantic boundary changes.

Depth repair commit: `17adb10aaf0eec4a76890332a46650f736c5d87a`.
Build 38306434532 failed Clippy's eight-argument helper finding. Classified as an
ordinary implementation lint defect; depth/current-maximum are now one bounds
pair without a lint suppression. Local Clippy passed after this correction.
Tests 38306434486, docs 38306434474 and verify 38306434482 are retained evidence.

## Complete push evidence index

Every linked run is retained, including failed attempts. No infrastructure rerun
was used; fixes are separately committed and re-run the complete workflow set.

| Milestone | Commit | build | tests | docs | verify | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | `8d5b60ac9f46ba44d6df0392b505ebf8343eba5e` | [38296578935](https://github.com/gregkrsak/klyxr/actions/runs/38296578935) | [38296578865](https://github.com/gregkrsak/klyxr/actions/runs/38296578865) | [38296578897](https://github.com/gregkrsak/klyxr/actions/runs/38296578897) | [38296578863](https://github.com/gregkrsak/klyxr/actions/runs/38296578863) | pass |
| M1 | `60cee544039ecd22b7ed01576490231e2a17cfd1` | [38297326807](https://github.com/gregkrsak/klyxr/actions/runs/38297326807) | [38297326828](https://github.com/gregkrsak/klyxr/actions/runs/38297326828) | [38297326804](https://github.com/gregkrsak/klyxr/actions/runs/38297326804) | [38297326820](https://github.com/gregkrsak/klyxr/actions/runs/38297326820) | pass |
| M2 | `0bf7542f029040ce270c57b0ca398045e9df5a8f` | [38297981588](https://github.com/gregkrsak/klyxr/actions/runs/38297981588) | [38297981569](https://github.com/gregkrsak/klyxr/actions/runs/38297981569) | [38297981517](https://github.com/gregkrsak/klyxr/actions/runs/38297981517) | [38297981600](https://github.com/gregkrsak/klyxr/actions/runs/38297981600) | pass |
| M3 | `2127503186654e618260fad063df9509e4ab4f34` | [38298735533](https://github.com/gregkrsak/klyxr/actions/runs/38298735533) | [38298735645](https://github.com/gregkrsak/klyxr/actions/runs/38298735645) | [38298735624](https://github.com/gregkrsak/klyxr/actions/runs/38298735624) | [38298735531](https://github.com/gregkrsak/klyxr/actions/runs/38298735531) | pass |
| M4–5 fixtures | `2fbc0720f955a9da9d90abc6b7eb69bc85326ce0` | [38299407894](https://github.com/gregkrsak/klyxr/actions/runs/38299407894) | [38299407839](https://github.com/gregkrsak/klyxr/actions/runs/38299407839) | [38299407807](https://github.com/gregkrsak/klyxr/actions/runs/38299407807) | [38299407860](https://github.com/gregkrsak/klyxr/actions/runs/38299407860) | tests fail |
| M4–5 corrected | `85a122266e0cde4897b15192d7fd91982e9f0366` | [38299632722](https://github.com/gregkrsak/klyxr/actions/runs/38299632722) | [38299632462](https://github.com/gregkrsak/klyxr/actions/runs/38299632462) | [38299633208](https://github.com/gregkrsak/klyxr/actions/runs/38299633208) | [38299632633](https://github.com/gregkrsak/klyxr/actions/runs/38299632633) | pass |
| M6 fixtures | `bf03f406220397508c84135f348c346d2bef8cf3` | [38301193001](https://github.com/gregkrsak/klyxr/actions/runs/38301193001) | [38301192993](https://github.com/gregkrsak/klyxr/actions/runs/38301192993) | [38301193008](https://github.com/gregkrsak/klyxr/actions/runs/38301193008) | [38301193007](https://github.com/gregkrsak/klyxr/actions/runs/38301193007) | tests fail |
| M6 corrected | `c25b4dad318bc60aa32141d04729b085197f9de7` | [38301659243](https://github.com/gregkrsak/klyxr/actions/runs/38301659243) | [38301659298](https://github.com/gregkrsak/klyxr/actions/runs/38301659298) | [38301659138](https://github.com/gregkrsak/klyxr/actions/runs/38301659138) | [38301659270](https://github.com/gregkrsak/klyxr/actions/runs/38301659270) | pass |
| M7 | `c3b38909ab3a1662d0da93f05f26a5b84278d7cc` | [38305366758](https://github.com/gregkrsak/klyxr/actions/runs/38305366758) | [38305366703](https://github.com/gregkrsak/klyxr/actions/runs/38305366703) | [38305366688](https://github.com/gregkrsak/klyxr/actions/runs/38305366688) | [38305366727](https://github.com/gregkrsak/klyxr/actions/runs/38305366727) | pass |
| conservation compile | `ba6207aa1baa12860cb4b3047b9d4b30c789d0ba` | [38305853409](https://github.com/gregkrsak/klyxr/actions/runs/38305853409) | [38305853260](https://github.com/gregkrsak/klyxr/actions/runs/38305853260) | [38305853346](https://github.com/gregkrsak/klyxr/actions/runs/38305853346) | [38305853303](https://github.com/gregkrsak/klyxr/actions/runs/38305853303) | build/tests fail |
| omitted staged fix | `fc25ade22981441d94ada804190fbcd1670e2a14` | [38305928194](https://github.com/gregkrsak/klyxr/actions/runs/38305928194) | [38305928509](https://github.com/gregkrsak/klyxr/actions/runs/38305928509) | [38305928238](https://github.com/gregkrsak/klyxr/actions/runs/38305928238) | [38305928555](https://github.com/gregkrsak/klyxr/actions/runs/38305928555) | build/tests fail |
| conservation assertion | `48236c4d22b7ead4f660c413fe8a31760baba795` | [38305977868](https://github.com/gregkrsak/klyxr/actions/runs/38305977868) | [38305978022](https://github.com/gregkrsak/klyxr/actions/runs/38305978022) | [38305977877](https://github.com/gregkrsak/klyxr/actions/runs/38305977877) | [38305977916](https://github.com/gregkrsak/klyxr/actions/runs/38305977916) | tests fail |
| storage corrected | `a747e568686dd7a52c820d792b14ee3681f8f87a` | [38306153091](https://github.com/gregkrsak/klyxr/actions/runs/38306153091) | [38306153088](https://github.com/gregkrsak/klyxr/actions/runs/38306153088) | [38306153312](https://github.com/gregkrsak/klyxr/actions/runs/38306153312) | [38306153269](https://github.com/gregkrsak/klyxr/actions/runs/38306153269) | pass |
| depth repair | `17adb10aaf0eec4a76890332a46650f736c5d87a` | [38306434532](https://github.com/gregkrsak/klyxr/actions/runs/38306434532) | [38306434486](https://github.com/gregkrsak/klyxr/actions/runs/38306434486) | [38306434474](https://github.com/gregkrsak/klyxr/actions/runs/38306434474) | [38306434482](https://github.com/gregkrsak/klyxr/actions/runs/38306434482) | build fail |
| depth corrected | `c7df86870256d60cf630c212d503299cdc43004b` | [38306538179](https://github.com/gregkrsak/klyxr/actions/runs/38306538179) | [38306538085](https://github.com/gregkrsak/klyxr/actions/runs/38306538085) | [38306538097](https://github.com/gregkrsak/klyxr/actions/runs/38306538097) | [38306538120](https://github.com/gregkrsak/klyxr/actions/runs/38306538120) | pass |

## Final conformance and audit

The final gate adds warnings-denied Rustdoc, repository scaffold validation and
byte-for-byte context regeneration to the existing four workflows. Tests capture
complete Cargo logs and explicitly compare canonical test identities/outcomes to
`ked-027-predecessors.json`, extracted from baseline job 114968984399. Doctest line
numbers are source coordinates, not identities, and are normalized; the compile-fail
witness remains required. Cargo target/group ordering avoids stdout/stderr timing
misclassification. All expected new categories and the affine check are required.

Expected final profile: 1,580 predecessor unit/integration tests, one predecessor
compile-fail doctest and 76 new KD-042 unit tests = 1,657. New categories: catalog
10; lexical MIR 4; vertical leaf/authority 7; transfer/structured exits 18; aggregate/
pending construction 10; bounds/malformed artifacts 16; Copy/reporting 6; conservation
5. The 1,800-case affine kernel test must pass independently in debug and release.
CLI suites include 26 proof-scope/regression tests and five execution CLI tests
covering exact record/loop output, range trap, invalid limits, bounded I/O, host
input failure and independence from failing specialized harness evaluation.

Final diff audit checks resource identities and classification, single ownership
authority, lexical MIR/origin correspondence, deterministic structured-exit order,
protected returns, exact pending membership, no source restoration, shared graph
bounds, prospective storage, terminal effects and conservation. Canonical decision
semantics (`LANGUAGE_DECISIONS.md`) and deferred-question documents are unchanged.
Predecessor source-test changes adapt field access to structural ValueType while
retaining test identities and canonical range/malformed witnesses. No dependency,
public resource registration, source resource syntax, OS handle, callback, unwinding,
rollback, projection, replacement destruction or general failure system is added.
No architectural/semantic pressure remains deferred from this implementation audit.
Independent conformance review and Greg's acceptance remain outstanding.

Local commands (advisory only): `cargo fmt --all`, `cargo fmt --all -- --check`,
`cargo check --locked --workspace --all-targets`,
`cargo clippy --locked --workspace --all-targets -- -D warnings`. Successful local
static results are not executable evidence; corruption warnings were not repaired.
An ordinary test compile error and a helper-argument Clippy finding were corrected
and independently gated in GitHub. Local `python scripts/verify-readme.py`,
`python scripts/verify-repo-scaffold.py`, `python scripts/build-ai-context.py`,
byte comparisons, identity reconciliation of GitHub logs, and `git diff --check`
are source/static verification. No local Cargo tests or generated Rust binaries
were executed in this assignment. Command-line push was blocked by unavailable
credentials; connected GitHub Git-data publication preserved reviewed content,
parents and exact tree comparisons. The omitted staged correction is recorded above.

This implementation remains private and synthetic. Resource-free public syntax
and applicable runtime costs are preserved. K27-I01 remains a local execution
platform limitation. Only GitHub Actions supplies executable conformance evidence.
No merge, Issue #60 closure, self-approval or acceptance action is authorized.


Depth correction `c7df86870256d60cf630c212d503299cdc43004b` passed all four
workflows. Both profiles executed and passed all 1,657 identities (76 new),
including the graph-depth control, compile-fail doctest and affine kernel check.
The complete source diff and implementation-facing documentation were audited
before the final documentation/conformance commit. The exact prod commit/tree
were reconfirmed unchanged. Generated context and local scaffold/asset checks
passed; their authoritative checks are included in the final workflow gate.


## Focused Sarah review repair — K27-MR01 through K27-MR03

Authorized starting head: `78ea9cb8da9ab4c752652b62a9666f0895abfb62`,
tree `d45edb730653ddb79f677608f816c6da55d35382`; existing PR #63 targets
`prod` on `ked-027-deterministic-scope-cleanup`. Remote head/tree and clean
checkout were verified before modification. Sarah's complete independent report
was read from Greg's saved review; GitHub PR reviews/comments were empty.
Charter: governance commit `0cc5fa2fa6743d263407c86603c2c2c9feb9810a`,
blob `15d58be9110a38454e44956f1f6845a2ad2cf4ce`.

- MR01: canonical plans govern live-overwrite rejection; nested inert construction
  activates pending storage independently of unrelated resources. Three tests
  cover two loop iterations with/without an outer owner, exact release counts,
  no inert initialization metadata, and retained live-overwrite rejection.
- MR02: value-stack and destination-slot publication retain resource ownership in
  the already-reserved detached slot before every fallible receiving reservation.
  Two tests cover full later-argument value storage and new-slot growth, transferred
  sources, unique retained leaves, no cleanup on stop, and terminal resume refusal.
  Existing pending/protected/current publication and call-entry paths were audited:
  payloads remain retained until their receiving allocation succeeds.
- MR03: typed plan errors distinguish malformed state, graph/depth bounds, and
  detected host allocation failures. All actual plan reservation boundaries use
  one fallible helper with thread-local test injection. Five tests cover initial
  construction, later lowering after schema validation, sealing with no artifact,
  revalidation after prior success, and independent malformed/limit controls.
  Bounded record validation checks the catalog through the typed path before the
  unchanged record-expression checks. The public source-only legacy lowerer
  remains infallible over resource-free HIR; private synthetic cleanup lowering
  uses the fallible origin-producing entry point, computing plans once per program.

Expected gate: 1,581 predecessor identities + 76 original KD-042 identities +
10 repair identities = 1,667 per profile. Inventory, duplicate/outcome checks,
namespace restrictions, authority doctest, and affine cross-check remain intact.

Local platform evidence: default PATH has no Cargo/Rust binaries. The existing
installation at `/workspace/scratch/14316a87a750/rust-1.80.0/bin` was located
without modifying it. `rustc -vV` and `rustfmt --version` fail because
`librustc_driver-198d7b2c3cb4a0cb.so` cannot be read; `cargo -vV` segfaults.
No reinstallation, permission restoration, artifact repair, or repeated runtime
recovery was attempted. Rust executable validation is blocked locally. Python
README/scaffold checks and diff checks passed. New tests are authored, not yet
reported as executed; exact-head GitHub evidence will be recorded on PR #63 and
Issue #60 after the gate. No merge or self-approval is authorized or performed.

### Repair gate 1 and formatter correction

Repair source published as `a2faeca5359a890dc7c9303b633a75427a128b58`
(tree `8ad9afa770639fa0aac8718a93c5a77c1923d354`). Both push and PR tests,
docs and verify passed. Debug and release each reconciled 1,667 passing identities,
including every repair regression, zero failures/ignores/filters and the authority
doctest. Push test log: job `115927852804`, run `38585943253`.

Push build `38585943159` and PR build `38585952121` compiled successfully,
then failed formatting. Clippy/Rustdoc were skipped and are not claimed passed.
All 37 rustfmt hunks were applied exactly from push build job `115927852797`;
this is GitHub-generated formatting evidence, not local Rust execution.
No semantic test or witness changed. The next exact head must pass the complete
eight-run gate; no workflow rerun or runtime repair was used.

Publication note: automatic review initially rejected CLI publication for missing
trusted destination/authorization evidence. Repository metadata then confirmed
Greg's public repository, owner and push permission; the entire outgoing diff
was inspected. The permitted retry failed due to missing CLI HTTPS credentials.
The connected GitHub integration published the same independently matching tree
with a guarded, non-forced update of the existing branch. The local unpublished
commit was synchronized to the published equivalent tree without discarding work.
