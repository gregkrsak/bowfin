# BED-033 implementation ledger

## Authority and fixed baseline

Greg separately authorized implementation of frozen Draft 2. Governing records:

- [Issue #78](https://github.com/gregkrsak/klyxr/issues/78)
- [Freeze and mandatory constraints](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258324041)
- [Implementation-pressure review](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258241641)
- [Sarah's focused confirmation](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258020728)
- [Sarah's original review](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6257518197)
- [Auditability-first doctrine](https://github.com/gregkrsak/klyxr/issues/52)

Baseline commit: `62023217b3f93e53cae302731643cbcea1fb3d17`.
Baseline tree: `3998bcad9b34d66e960b863eb20b7b78680da07d`.
Branch: `ked-033-trusted-effectful-operations`; PR target: `prod`.

Remote prod and the verified signed Git object matched the assigned baseline.
The object was fetched through the connected Git API, its exact Git hash verified
locally, and the branch created once from that exact commit. The preceding
checkout/index were clean. No accepted implementation was discarded or rewritten.
The production Arnold charter remains governance commit
`2887c9239fb6f82ded4fd7b2d8f5f592b725cccc`, blob
`b04c4ba5251fdcacb5278ba26c0aaf8a06b4db7e`. Exact-tree AGENTS.md and the five
canonical documents were read in order during the immediately preceding pressure
review; their identical tree and controlling requirements remain verified.

Freeze does not constitute implementation acceptance. Greg retains acceptance
authority; only a human may merge. No KED-034 work is authorized.

## M0 environment and predecessor evidence

The accepted Rust 1.80.0 installation remains available at
`/tmp/rust-clean-vywtlaad/toolchain/bin`. All seven compiler/component executable
sizes, modes and SHA-256 hashes matched the accepted snapshot before use. No
installation, repair, permission change or process-table inspection was performed.
One task-owned target is used: `/tmp/klyxr-ked033-target-ltwgzadw`.

The complete local predecessor gate passed at the untouched exact baseline:
**1,875 identities in debug and 1,875 in release**, independently reconciled by
`scripts/verify-ked032-tests.py` as 1,846 predecessors plus 29 KED-032 additions.
The compile-fail authority doctest and 1,800-case affine cross-check passed in each
profile. Locked all-target checking, formatting, warnings-denied Clippy/Rustdoc,
Result and propagation CLI controls, ordinary/replacement/effect examples, battery
positive/negative diagnostics, README assets, scaffold and byte-identical generated
context all passed. Worktree, index and diff checks were clean. Negative controls
returned expected class 1; every positive command returned zero.

This uses the normal local workflow while it remains reliable. GitHub Actions is
authoritative if the known local execution problem recurs; no toolchain repair or
experimental target replacement is permitted. Historical ps/procfs namespace
limitations remain ancillary; ps and related process-table utilities are not used.
The historical shell Git proxy is not repaired: verified local Git objects may be
published using the connected API. CI is not substituted for a falsely claimed
local result. Exact milestone commit/tree and run/job evidence is recorded as
it becomes available, without self-referential commit hashes in this file.

## Mandatory constraint map

| Constraint | Implementation milestones | Evidence location planned |
| --- | --- | --- |
| P01 catalog collection and honest origins | M1 | trusted catalog/resolver tests; manifest |
| P02 genuine Resource category and exclusions | M2, M4 | type/publication/plan/eligibility tests |
| P03 canonical calls and argument destinations | M3, M4 | signature/ownership/hold/outcome tests |
| P04 no-gap receipt and trusted Result provenance | M4, M5, M7 | actual invocation/receipt/publication probes |
| P05 persistent stopped session | M5, M7 | session/census/terminal non-delivery tests |
| P06 ownership-derived cleanup effects | M3, M6 | retire versus relay and structured-exit tests |
| P07 exact grants, release and binding approval | M4, M7 | independent authority/descriptor mutations |
| P08 explicit roots and structural audit reachability | M4, M8 | graph/root/grant/audit tests |
| P09 complete bounded accounting | M1-M9 | exact neighbors, allocation probes, audit/diagnostic tests |

No acquisition, including the deterministic fixture, may execute before M5 has
both the pre-reserved receipt and persistent terminal-session owner in place.

## Frozen section 20 evidence plan

These are planned obligations, not passing tests at M0. S = ordinary source through
the genuine selected catalog/binding; F = fault at an actual boundary; M = private
malformed artifact. The final manifest will enumerate exact identities and lanes.

| Items | Required witness | Milestones / lanes |
| --- | --- | --- |
| 1-5 | exact operation resolution, denied/missing entries, collisions, cross-namespace and case/unselected controls | M1, M3, M7 / S,M |
| 6-8 | nominal range domains/operations, exact operation Error subsets, invalid imported values retained with Integrity | M1, M5, M7 / S,F,M |
| 9-10 | open Ok fresh unique owner; each Error no owner or cleanup | M5-M7 / S,F |
| 11-12 | read alternatives preserve the Sensor and aliases | M5-M7 / S,F |
| 13-15 | full propagation, protected Ok/Error cleanup, Limit/failure non-delivery | M6-M7 / S,F |
| 16-17 | holds, moves, duplicate cleanup, forged/stale handles | M3, M6-M7 / S,M |
| 18-19 | parameters/relay, excluded references/conditional/replacement/fields/egress | M2-M7 / S,M |
| 20-23 | complete effects, retire/relay, grants versus io, exact cleanup-only binding | M3-M7 / S,M |
| 24-29 | typed terminal separation, no fallback, correspondence, no-attempt/pre-prefix and retained post-prefix stops | M4-M7 / S,F,M |
| 30-32 | every structured exit/destination and exact resource/work/storage neighbors | M6-M7 / S,F |
| 33-35 | rooted grant/use audit, bounded rendering, verified/harness exclusion and proof preservation | M8-M9 / S,F,M |

## Frozen section 21 malformed controls plan

Each row requires independent controls, not one compound corrupted artifact.

| Mutation | Milestone / expected boundary |
| --- | --- |
| operation/resource IDs | M4,M7 / pre-dispatch rejection or retained malformed receipt |
| catalog/profile identity | M4,M7 / immutable approval |
| parameter order/type/mode | M3,M4,M7 / signature and destinations |
| result/Result-plan identity | M2,M4,M7 / publication and correspondence |
| effects | M3,M4,M7 / complete independent contracts |
| acquisition/release pairing | M2,M4,M7 / exact cleanup-only grant/binding |
| fresh identity | M5,M7 / receipt-held payload and session realm |
| source/HIR/MIR/catalog origins | M1,M4,M7 / exact provenance |
| runtime binding identity | M4,M5,M7 / no substitution/fallback |
| destination/protected holder | M4-M7 / no duplicate transfer or premature delivery |
| cleanup plan/realization | M2-M7 / actual scopes and completed prefix |
| returned tag/active payload | M5,M7 / Integrity without lost custody |
| authority membership | M4,M7,M8 / independent copies and rooted authority |

## Milestone status

M0: baseline and branch established; ledger created before semantic changes.
M1-M4: catalog admission, canonical Resource eligibility, trusted-call typing and
explicit invocation/root/runtime approval are implemented; validation below.
M5 joint receipt/session custody is implemented and source fixture execution is
enabled only through that seam. M6 composition and M7 failures are implemented; M8 audit/CLI and M9 evidence/documentation are implemented; final publication and independent review remain pending. The consuming convenience
executor still rejects profile artifacts, and no ordinary/host fallback exists.

### M0 published checkpoint

Commit `3c88f3f38a86182ad5193db9b16521f6ee9fd272`, tree
`ba6e05ba571b00dd96e50b1fab233e2d66091a3f`, establishes this ledger. All four
push workflows and every job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39190661833](https://github.com/gregkrsak/klyxr/actions/runs/39190661833) | 117862503061 |
| tests | [39190661856](https://github.com/gregkrsak/klyxr/actions/runs/39190661856) | 117862503113 |
| verify | [39190661865](https://github.com/gregkrsak/klyxr/actions/runs/39190661865) | 117862503206 |
| docs | [39190661926](https://github.com/gregkrsak/klyxr/actions/runs/39190661926) | 117862503128 |

[Issue progress record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258480234).

### M1 implementation details and evidence

One explicit immutable sensor-demo selection records its scenario, independent
open/read/release grants and bounded named entry roots. It supplies only fixed
compiler-owned rows; ordinary source cannot author descriptors or bindings.
Range rows precede source declaration/signature collection, with exact domains
and nominal identities. Resource and operation IDs carry distinct checked
collection serials; matching numeric row offsets in another collection are not
authority. Range declarations now distinguish Source spans from Catalog origins;
no catalog row is assigned a synthetic source location.

Selected operations occupy only callable names and selected types occupy only
type names. Denied rows still occupy their names, collection rejects collisions
without overwriting the catalog, and ordinary values/fields retain their separate
namespaces and original case rules. Imported ranges retain existing literal,
subtraction, comparison, Copy and field behavior. Every row is validated even
when unused/denied; fixed release has no source-callable name. No private resource
record fixture or acquisition path is activated.

The source-independent admission envelope adds 65,536 bytes plus 4,096 bytes per
root plus 16 times root-name bytes, 65,536 work plus 4,096 work per root, and 4,096
diagnostic bytes plus 16 times root-name bytes. The conservative terms cover
catalog rows, selected configuration, retained approval copies, root scans and
diagnostics. They are summed with frontend, outcome and cleanup envelopes, rather
than compared in isolation. Exact sufficient and immediately insufficient
combined neighbors are tested independently. The no-profile terms stay zero.

Sixteen new identities are enumerated and classified in
`ked-033-evidence.json`; `ked-033-predecessors.json` preserves all 1,875 exact
baseline identities and `verify-ked033-tests.py` reconciles both profiles without
permitting unclassified additions. Predecessor fixtures receive only their honest
Source-origin wrapper; no expectation or numeric protocol oracle changes.
Ordinary development corrections were limited to test fixtures initially using
unsupported syntax (`+`, missing `range`, and a reserved Result-family name).
Those fixtures were corrected to the existing syntax rather than changing it.

The complete M1 local gate passed: **1,891 exact debug identities and 1,891
release identities** (1,875 predecessors plus 16 additions), authority doctest,
both 1,800-case affine checks, all-target checking, formatting, warnings-denied
Clippy/Rustdoc, all predecessor CLI/examples/rejection diagnostics, README assets,
scaffold, context reproducibility and diff integrity. All existing numeric
protocol oracles passed unchanged. The development diff is the authorized M1
change, not a new claim of baseline cleanliness; index/worktree identity is
reconfirmed at the published checkpoint.

M1 does not claim typed Sensor eligibility, trusted execution, stopped custody,
structural audit, or completion of P02-P09. Those remain later milestones.

### M1 published checkpoint

Commit `dfc63bb22166eb3335d4ae870977b69ca393caec`, tree
`34116e0ffc71c91c03fc7b6a90a4abcda161a10b`. All four push workflows and every
job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39191315743](https://github.com/gregkrsak/klyxr/actions/runs/39191315743) | 117864549739 |
| tests | [39191315678](https://github.com/gregkrsak/klyxr/actions/runs/39191315678) | 117864549494 |
| verify | [39191315629](https://github.com/gregkrsak/klyxr/actions/runs/39191315629) | 117864549001 |
| docs | [39191315649](https://github.com/gregkrsak/klyxr/actions/runs/39191315649) | 117864549143 |

[Issue progress record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258579732).

### M2 implementation details and evidence

Sensor now has its own canonical Resource type in value, expression, referent,
resolution and Result-child vocabularies. It has no RecordId, public fields or
representation constructor. The fixed descriptor records its exact cleanup-only
operation, io contract, catalog origin and conservative 64-byte binding-payload
bound. Typed cleanup references distinguish Record from Resource without changing
the private synthetic catalog or enabling its record-field permissions.

The existing ownership checker treats Resource owners as Move and shared handles
as Copy, preserves last-use/held-argument behavior and rejects duplicate moves.
Canonical resource interface validation checks unused/foreign types and excludes
mutable Resource references. Mutable borrows, conditional initialization,
replacement, fields, representation inspection, arithmetic, comparison and
dereference cannot construct or duplicate Sensor. Resource-bearing public Result
egress rejects before machine entry, even on an inactive resource alternative.
Ordinary owned helper transfers, non-escaping shared helpers and Resource Result
payloads pass through the existing pipeline.

The existing custody product now carries typed plans. Its actual successful
transfer gate removes a relayed Sensor before parameter retirement; a retained
Sensor instead emits io cleanup evidence. Thus retire requires io while relay
does not, already proven without invoking a fixture or introducing a second
ownership/effect checker. MIR cleanup and temporary plans preserve Resource
identity. The private runtime value shape is present but deliberately cannot be
constructed yet; receipt/session dispatch remains disabled until M5.

Nine additional identities bring the inventory to 25 KED-033 additions. Complete
local validation passed: **1,900 exact debug identities and 1,900 release
identities**, all 1,875 predecessors, authority doctest, both 1,800-case affine
checks, check/fmt/Clippy/Rustdoc, CLI/examples/rejection diagnostics, assets,
scaffold, context reproducibility and diff integrity. Numeric protocol oracles
remain unchanged. Existing malformed Record-plan fixtures receive only the typed
Record wrapper, retaining their original corrupt IDs, expectations and outcomes.

Ordinary development corrections included extending the canonical borrow
validator for shared Resource referents and checking foreign Resource interfaces
at the typed-HIR publication boundary rather than adding type inference to the
ownership checker. No frozen source behavior was revised. M3-M9, actual trusted
calls/receipts/binding, structural audit and independent acceptance remain open.

### M2 published checkpoint

Commit `243b79b2ca43e85260cbac54e5dd7cec8b0c606c`, tree
`4abc931ff7e2ca6d5e2eeb50d3677b370f05d4f6`. All four push workflows and every
job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39192143108](https://github.com/gregkrsak/klyxr/actions/runs/39192143108) | 117867064199 |
| tests | [39192143077](https://github.com/gregkrsak/klyxr/actions/runs/39192143077) | 117867064472 |
| verify | [39192143109](https://github.com/gregkrsak/klyxr/actions/runs/39192143109) | 117867066174 |
| docs | [39192143093](https://github.com/gregkrsak/klyxr/actions/runs/39192143093) | 117867064306 |

[Issue progress record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258673128).

### M3 implementation details and evidence

The defining source now prepares through an early canonical catalog, exact
imported Result contracts, distinct resolved/HIR TrustedCall nodes and typed
trusted argument destinations. No source functions or ParameterIds stand in for
trusted operations. The fixed source-callable operation contracts are canonical
Result rows collected before ordinary signatures, including unused/denied rows.

Source and trusted calls share argument checking and the existing loan-hold
transaction. All finite-use/reference-use visitors, temporary child traversal,
MIR validation and HIR publication traverse the new node. The existing direct
effect collector has a tagged canonical callee; catalog io is checked independently
of grants and contributes its complete contract. Catalog diagnostics cite catalog
origins, never an invented source declaration. Cleanup effects remain the same
successful residual-custody product, including read propagation failure and the
ordinary protected return.

Twelve additions bring the KED-033 inventory to 37. They establish exact trusted
typing, no ordinary stubs/fake parameters, nominal Result payloads, unchanged
constructor literal rules, independent effects/grants, shared aliases and nested
receiving-call holds, no discarded/unresolved outcome, verified-scope exclusion,
typed MIR destinations and independent foreign/cleanup-only/type/arity mutations.
No fixture invocation is used for these preparation witnesses. The consuming
public convenience executor rejects profile artifacts before machine entry;
profile execution must use the future persistent session. The runtime TrustedCall
arm also rejects explicitly, with no ordinary or host fallback.

The complete local M3 gate passed: **1,912 exact debug identities and 1,912
release identities**, all 1,875 predecessors, authority doctest, both 1,800-case
affine checks, checking/fmt/Clippy/Rustdoc, CLI/examples/rejection diagnostics,
assets/scaffold, byte-identical context and diff integrity. Numeric protocol
oracles remain unchanged. An existing malformed Source-callee fixture receives
only its typed Source wrapper. New test-development corrections used the genuine
fallible MIR lowering seam for temporary metadata and matched the existing
Result-payload diagnostic; neither production semantics nor old expectations
were changed.

M4 exact invocation/root/runtime-binding approval and M5 joint receipt/session
custody are still required before any deterministic or external acquisition.

### M3 published checkpoint

Commit `2820c59510bbc6dd39ddfa44a4447d0bc35f16f3`, tree
`dc89ad71fd38c676f9eea312bb20ac5260b710b1`. All four push workflows and every
job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39192956029](https://github.com/gregkrsak/klyxr/actions/runs/39192956029) | 117869523553 |
| tests | [39192956034](https://github.com/gregkrsak/klyxr/actions/runs/39192956034) | 117869523578 |
| verify | [39192956047](https://github.com/gregkrsak/klyxr/actions/runs/39192956047) | 117869523717 |
| docs | [39192956021](https://github.com/gregkrsak/klyxr/actions/runs/39192956021) | 117869523659 |

[Issue progress record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258781592).

### M4 implementation details and evidence

Opaque approval now independently retains a runtime contract separate from HIR,
including exact target/compiler/catalog/profile/assumption identities, selected
configuration and grants, Resource/release/binding descriptors, imported range
descriptors and approved entry roots. The existing MIR TrustedCall nodes are
indexed by explicit invocation plans: function/block/statement, temporary site and
region, operation/Result identity, argument sites, typed receiver, source origin
and catalog origin. Plans derive from actual MIR and existing custody metadata,
not a second semantic analysis or an ordinary-function stub.

Preparation validates exact root names as zero-argument ordinary functions. Entry
selection and public execution enforce the sealed root set. A separate structural
edge inventory includes every ordinary/trusted call, empty-effect helpers and
both alternatives. It is independent of filtered effect sites; M8 will derive
the rooted audit account from it. No call dispatch searches names or another
profile. Execution remains disabled pending M5.

Sixteen additional identities bring the manifest to 53. Independent mutations
cover target/compiler/catalog/profile/configuration, grants, bindings, release
pairing, Resource identity/payload bound, cleanup-only source authority,
signatures, effects, origins, imported range identity/bounds, assumptions,
invocation omission/duplication/reordering/foreign IDs, receiver/region/argument
sites, roots, graph edges and authority copies. Even consistently changed runtime
approval copies must match exact MIR derivation. Shape bounds precede deep
comparison. Deterministic test-only failures at four actual growth boundaries
(root, edge, argument and invocation reservations) each propagate as Host and
publish no executable; zero-sized reservations do not trigger injection.

The complete local M4 gate passed: **1,928 exact debug identities and 1,928
release identities**, all 1,875 predecessors, authority doctest, both 1,800-case
affine checks, check/fmt/Clippy/Rustdoc, CLI/examples/rejection diagnostics, assets,
scaffold, byte-identical context and diff integrity. Numeric protocol oracles
remain unchanged. No deterministic or external acquisition has executed.
No semantic deviation, new checker, fallback or implementation blocker is reported.

### M4 published checkpoint

Commit `4ec483803d111a09f3d32cac283c8d8d523cf4db`, tree
`0ecd76e570c0418a2d0501aa30400f1c81285633`. All four push workflows and every
job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39193893211](https://github.com/gregkrsak/klyxr/actions/runs/39193893211) | 117872352894 |
| tests | [39193893276](https://github.com/gregkrsak/klyxr/actions/runs/39193893276) | 117872353039 |
| verify | [39193893237](https://github.com/gregkrsak/klyxr/actions/runs/39193893237) | 117872352788 |
| docs | [39193893272](https://github.com/gregkrsak/klyxr/actions/runs/39193893272) | 117872353014 |

[Issue progress record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6258996923).

### M5 joint custody gate and first real source execution

The no-gap receipt and opaque persistent ExecutionSession were implemented and
compiled together before the first fixture execution. The sole production path
that installs trusted runtime state is session construction from the existing
opaque executable and shared entry gate. Public consuming execution still rejects
profile artifacts. A stopped session retains the whole machine, receipt, binding
tokens, Result cells/provenance, protected values, cleanup cursors, completed
effect prefix and unfinished responsibilities. Repeated advancement returns its
recorded borrowed outcome without executing, retrying or releasing anything.

Before dispatch, combined managed capacity is reserved for identity/token slots,
actual receipt payload, Result cells and provenance, operand, immediate temporary
owner/event registration, audit prefix and stopped diagnostics. The fixed binding
creates a non-Clone token directly inside the already machine-owned receipt;
there is no host handle held only on the Rust stack across a fallible boundary.
Tag, operation, range identity/bounds, Error subset and fresh session-scoped
identity are validated while that holder owns the actual payload. Allocation-free
commit transfers the token to a machine-owned live table and publishes its sole
non-Copy language ticket and canonical Result cell. Histories never own tokens.
Trusted Result authority references real invocation provenance, not a constructor.

Later receiver or cleanup failures retain the token, its tickets/cells and all
other custody in the stopped session. The fixed release validates exact binding,
grant, resource and live identity and has no source completion Result. Teardown
and process reclamation are not successful Klyxr release and provide no unwind
guarantee. The demonstration remains a fixed deterministic fixture, not a device
driver or general FFI.

The selected profile has an audited configurable live-Sensor limit (default 16,
supported configuration 1 through 1,024). It is part of the sealed configuration,
never ambient source authority. Runtime accounting includes the selected state
header, token/provenance/event capacities, outstanding receipt, output and stopped
diagnostic storage. Bounded invocation/resource scans are charged explicitly;
the predecessor no-profile managed-buffer accounting remains unchanged.

Twelve new identities establish the full defining source's Ok and all four
recoverable fixture outcomes, cleanup before delivery, no attempt before admission
failure, retained receipt after external success plus Host/Integrity/publication
Limit, invalid imported reading rejection, protected Error after release failure,
and every fuel cut. Tests use real source/catalog/approval/session; no HIR/token
pre-injection, system OOM, panic catching or source-selectable fault switch exists.
Private faults are compiled only for tests at the real boundary. All normal and
faulted runs preserve exact terminal non-resumption and the completed effect prefix.

The complete local M5 gate passed: **1,940 exact debug identities and 1,940
release identities** (1,875 predecessors plus 65 additions), authority doctest,
both 1,800-case affine checks, check/fmt/Clippy/Rustdoc, CLI/examples/rejection
diagnostics, assets/scaffold, context reproducibility and diff integrity. Existing
numeric protocol oracles remain unchanged. No toolchain or artifact anomaly was
observed. M6 composition, the full M7 cut/mutation matrix, M8 audit/CLI and M9
documentation/final reconciliation are still outstanding; no acceptance is claimed.

### M5 published checkpoint and K33-I01 resolution

The preserved staged snapshot was committed without changing a byte as
`5dce4db52a638e3c90e737d87d176a633b32a9ee`, tree
`8483f4a4e5e90eba29632736a542ca3c71093d4c`. All four push runs and every step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39207396019](https://github.com/gregkrsak/klyxr/actions/runs/39207396019) | 117913119813 |
| tests | [39207396028](https://github.com/gregkrsak/klyxr/actions/runs/39207396028) | 117913119863 |
| verify | [39207396047](https://github.com/gregkrsak/klyxr/actions/runs/39207396047) | 117913120100 |
| docs | [39207396052](https://github.com/gregkrsak/klyxr/actions/runs/39207396052) | 117913120205 |

K33-I01 recorded prod's documentation advancement to
`10f89f925c84f53bf9f4599c0c88bc7b75b614b2` in
[the pause record](https://github.com/gregkrsak/klyxr/issues/78#issuecomment-6259067524).
Greg expressly confirmed that authorized documentation work does not revoke the
frozen implementation baseline, directed preservation/publication of M5 and
continuation through M6-M9, and prohibited rebasing or merging prod without
separate direction. That blocker is resolved by this authorization. Implementation
continues from the original baseline; actual final integration conflicts will be
reported, and merge remains human only.

### M6 source composition

Eleven real source/session identities cover explicit matches and Err normalization,
read Error followed by a second explicit read through a shared alias, owned helper
retirement, effect-free protected Sensor relay, shared observation helpers, nested
Result obligations, branch cleanup, iteration-local continue/break/normal exit,
early loop return, protected argument temporaries, written argument order and
public Resource/Resource-bearing Result egress rejection before session entry.

These tests exposed one ordinary implementation omission: taking a Resource slot
marked it vacant instead of transferred. Resource now uses the existing transferred
slot state alongside Record and Result, so relay and nested protected returns
retire the old owner without releasing the new owner. No new ownership checker,
cleanup authority or source rule was introduced. Initial source fixtures using
unsupported named arguments and non-total returns were corrected to existing
accepted grammar; predecessor tests and numeric oracles remain unchanged.

Future entries must record meaningful changes, validation and exact remote evidence,
ordinary development corrections, any K33-I escalation, predecessor adaptations,
accounting constants, documentation/context outcomes and residual limitations.

M6 complete local gate: **1,951 exact debug identities and 1,951 release identities**
(1,875 predecessors plus 76 additions); all established check/fmt/Clippy/Rustdoc,
authority doctest, both 1,800-case affine checks, CLI/examples/rejections, assets,
scaffold, context reproducibility and diff integrity passed. No artifact anomaly.

### M6 published checkpoint

Commit `13646882875e9abb09df9b12ca69c83eaee585fa`, tree
`4cecd62406a64826fb599260bba353eaf69fe1b4`. Every push job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39207717304](https://github.com/gregkrsak/klyxr/actions/runs/39207717304) | 117914078187 |
| tests | [39207717416](https://github.com/gregkrsak/klyxr/actions/runs/39207717416) | 117914078464 |
| verify | [39207717308](https://github.com/gregkrsak/klyxr/actions/runs/39207717308) | 117914078129 |
| docs | [39207717272](https://github.com/gregkrsak/klyxr/actions/runs/39207717272) | 117914077916 |

### M7 actual failure cuts and malformed receipts

Sixteen additional boundary identities independently exercise seven actual
receiving allocation growth boundaries (identity, provenance, invocation audit,
operand, Result cell, temporary owner and temporary event) and release-audit
growth. Each detected allocation failure is Host; receiving failure precedes any
external attempt, while release-audit failure retains the owner without a release
attempt. Faults are sealed cfg(test), only fire on actual growth and never retry.
No source/production switch, real OOM experiment or caught panic is used.

Malformed receipts independently cover foreign and duplicate fresh identity,
Error carrying a Sensor, wrong imported nominal range, in-bounds but unenumerated
operation Error and wrong invocation provenance. Actual tokens stay in their
machine-owned receipt; the duplicate control retains two physical responsibilities
rather than collapsing them by equal numeric ID. Receipt invocation must now equal
the immediately realized invocation, not merely an earlier number in the prefix.
Opaque approval mutation coverage from M4 remains in force before session entry.

Exact combined runtime storage/fuel neighbors, tiny-source session header plus
stopped diagnostics, the live-Sensor count neighbor, legitimate Error followed by
Limit, protected Ok/Error after cleanup Host, CLI class 2, range traps and rendering
failure are covered. Every fuel cut through partial owned-argument entry and the
completed release prefix conserves acquired minus released tokens; repeat advance
never changes steps or events. No terminal result is delivered as source Error,
and no failed release is reported complete. Whole-Result cleanup authority and all
predecessor protocol constants remain unchanged.

M7 complete local gate: **1,967 exact debug identities and 1,967 release identities**
(1,875 predecessors plus 92 additions). All established gates including both affine
checks, authority doctest, warnings-denied tools and byte-identical context passed.
No compiler, executable or integrity anomaly occurred.

### M7 published checkpoint

Commit `d710972441e279f7e5af205163dd43c9f29e2b2f`, tree
`60a0cf355047250cccff4d69b4cbcabd80a81292`. Every push job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39208052932](https://github.com/gregkrsak/klyxr/actions/runs/39208052932) | 117915093110 |
| tests | [39208052949](https://github.com/gregkrsak/klyxr/actions/runs/39208052949) | 117915092986 |
| verify | [39208052948](https://github.com/gregkrsak/klyxr/actions/runs/39208052948) | 117915092934 |
| docs | [39208052983](https://github.com/gregkrsak/klyxr/actions/runs/39208052983) | 117915093515 |

### M8 deterministic authority account and profile CLI

The read-only audit validates opaque approval and computes structural reachability
from exactly the sealed roots through the complete canonical edge table. It uses
a bounded visited array and at most one pass per function, including recursion,
zero-effect calls and both alternatives. It never executes the fixture or infers
authority from effects filtering or observed events. Every fixed catalog row,
grant, binding, signature, effect, range contract, scenario, live-count limit,
root, reachable trusted site, source/catalog origin, opaque Resource pairing and
possible lexical/temporary cleanup site is disclosed. Immutable v1 contract and
assumption identities accompany the complete fixed contract. Cleanup-only release
is distinguished from source-callable use, and unused grants remain visible.

Audit scratch plus the full output allowance and builder header share a conservative
storage envelope. The session method adds retained machine storage to that same
envelope. Output bytes and metadata/graph scans are charged against audit work;
opaque revalidation retains its established bounded admission envelope. Scratch
and output growth remain fallible Host boundaries; exact work/storage/output
neighbors and both allocation controls are tested. Rendering is read-only: even
allocation, output allowance or writer failure cannot mutate the recorded external
outcome, release an owner, resume a stopped session or deliver a protected Result.

The existing bounded reader now accepts optional explicit immutable selection;
the resource-free path remains unchanged. CLI `run` and static `audit` take
`--profile sensor-demo`, one `--function` root, optional fixed `--scenario`,
`--sensor-limit`, and closed `--grants` selections. No profile is implicit. The
profile run path keeps its opaque session alive through result/diagnostic rendering
and writing. All five fixture outcomes have CLI evidence; denied read remains a
frontend failure despite io, malformed configuration is class 2, terminal limits
are class 1, and legitimate source Error remains successful execution. The defining
source is now `examples/trusted_sensor.klx`. There is no general manifest language,
runtime lookup, second profile, device driver or production fault switch.

M8 complete local gate: **1,977 exact debug identities and 1,977 release identities**
(1,875 predecessors plus 102 additions), authority doctest, both 1,800-case affine
checks and every established ancillary gate passed, including actual sensor CLI
success/Error/audit/denied controls. One supplementary harness expected the word
“denied”; the unchanged correct diagnostic is “not granted”. The expected substring
was corrected against the preserved exit-1 log, then the remaining documentation
and integrity gates completed. No implementation diagnostic was weakened, no test
identity or numeric oracle changed, and no artifact anomaly occurred.

### M8 published checkpoint

Commit `8f475e37dc29191b7525b6ee5af52554bf089382`, tree
`dc1e67ef8573c5f3e41083c85c4622c22aed5c5a`. Every push job/step passed:

| Workflow | Run | Job |
| --- | --- | --- |
| build | [39208587758](https://github.com/gregkrsak/klyxr/actions/runs/39208587758) | 117916718294 |
| tests | [39208587714](https://github.com/gregkrsak/klyxr/actions/runs/39208587714) | 117916718084 |
| verify | [39208587712](https://github.com/gregkrsak/klyxr/actions/runs/39208587712) | 117916717885 |
| docs | [39208587718](https://github.com/gregkrsak/klyxr/actions/runs/39208587718) | 117916718102 |

### M9 reconciliation and implementation-facing documentation

Four final identities complete Sensor-specific malformed evidence: forged logical
ticket and stale shared reference before read dispatch, duplicate release rejection
after successful fixed cleanup, independent canonical Resource Result-plan child/
effect/identity mutations, and independent protected-return/parameter/scope cleanup
realization mutations. These pass through genuine opaque approval or session
boundaries. Private controls never pre-create a Sensor or discard its physical
responsibility. Source eligibility and production authority are unchanged.

The exact manifest now enumerates 106 additions and maps each of the 35 section-20
obligations, 13 section-21 controls and P01-P09 to existing exact test identities.
The reconciler enforces complete nonempty maps and exact classifications/counts,
retains all 1,875 predecessor identities and the KED-032 chain, requires the authority
doctest and both-profile affine sentinel, and accepts no migration/contract correction.
Evidence lanes distinguish ordinary source, private catalog/envelope, malformed
authority and actual trusted runtime boundaries; they are tests under stated
fixture assumptions, not formal correctness or device proofs.

All five canonical AI files describe frozen semantics, the implemented slice,
remaining exclusions, ownership-derived effects, exact trust/receipt/session/audit
boundaries, deferred fallible completion and pending independent acceptance.
The generated bundle is rebuilt from those files. The compiler guide links the
focused [fixture and bounded-execution guide](BED-033-TRUSTED-SENSOR.md), which
documents actual commands, fixed assumptions, budgets, error classes, stopped
custody versus successful release, and no production fault authority.

The prod documentation changes remain separate. A read-only comparison at
`10f89f925c84f53bf9f4599c0c88bc7b75b614b2` shows only README.md, docs/README.md and
scripts/verify-readme.py changed since the frozen baseline; this branch changes
none of those paths. No rebase, merge or prod update has been performed. Final
GitHub integration status and exact head/tree/CI are reported externally after
publication to avoid self-referential commit identities in this ledger.

### Final local conformance result

The complete M9 local gate passed **1,981 exact debug identities and 1,981 release
identities: 1,875 predecessors plus 106 additions**, with zero failed, ignored,
filtered or migrated identities. Lanes: 53 ordinary-source, 3 private canonical
catalog, 13 private catalog/envelope, 11 private malformed-authority and 26 trusted
boundary identities. The KED-032 chain remains 1,846 plus 29. The authority
compile-fail doctest and 1,800-case affine cross-check passed independently in
both profiles. All seven accepted Rust 1.80.0 executable modes/sizes/SHA-256 hashes
were unchanged before the final gate. No actual compiler/artifact anomaly occurred.

| Exact command | Outcome |
| --- | --- |
| `cargo check --locked --workspace --all-targets` | 0 |
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --locked --workspace` | 0; 1,981 identities including doctest |
| `python3 scripts/verify-ked033-tests.py /tmp/klyxr-ked033-m9-gate/debug.log` | 0; exact reconciliation and all frozen maps |
| `cargo test --locked --workspace --release` | 0; same 1,981 identities |
| `python3 scripts/verify-ked033-tests.py /tmp/klyxr-ked033-m9-gate/release.log` | 0; exact reconciliation and all frozen maps |
| `RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps` | 0 |
| `python3 scripts/verify-readme.py` | 0; assets |
| `python3 scripts/verify-repo-scaffold.py` | 0 |
| `python3 scripts/build-ai-context.py` | 0; byte-identical regeneration |
| `git diff --check` and `git diff --cached --exit-code` | 0 before staging |

The gate also executes each established CLI example through `cargo run --locked
--quiet --bin klyxr --`: result_handling demo and success_demo, result_propagation
demo and failure_demo, ordinary_run demo, owned_replacement demo, explicit_effects
demo and battery_ok verification all returned 0 with their exact expected results.
result_unhandled_fail check, result_propagation_mismatch demo and battery_fail
verification returned expected 1 with their original diagnostics. The trusted
sample run and no-sample run returned 0 with Ok(Reading(42)) and
Error(SensorError(2)); its audit returned 0 with rooted authority; read denial under
open-release grants returned 1 with “not granted”. All five scenarios and invalid
configuration/terminal CLI classes are additionally tested as exact identities.

Generated context SHA-256:
`c99dc023ad072013fe21cba290c0268418fef3eae4cc98c18dc770d128efaab6`.
The complete baseline-to-final changes were inspected for scope, preservation of
predecessor expectations, typed boundaries and diff integrity. No tracked change
outside KED-033 is included. Local worktree/index become clean only after the final
coherent commit; final identity, remote cleanliness and CI runs are recorded in
the Issue/PR engineering report. No semantic deviation, KED-034 work, acceptance,
Issue closure, prod modification or merge is claimed.
