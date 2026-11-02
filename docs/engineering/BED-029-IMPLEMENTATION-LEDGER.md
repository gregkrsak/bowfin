# BED-029 implementation ledger

Issue: https://github.com/gregkrsak/klyxr/issues/66
Frozen specification: Draft 2; freeze 6207528580; authorization 6207551736.
Accepted implementation-pressure review: 6207528400.
Baseline: `72c34593e79e7763ce16f0e18ad99595959a2454`.
Tree: `2b9339fbe21f705cb8c5722ff681c34680870490`.
Branch: `ked-029-deterministic-expression-temporaries`; target: `prod`.
Current charter blob: `b04c4ba5251fdcacb5278ba26c0aaf8a06b4db7e`.

GitHub Actions is authoritative for compilation and executable evidence. Local
Rust validation is unavailable because of confirmed Work-runtime corruption.
No local Rust, Rustfmt, Clippy, compiler or test execution is performed. Source,
Git and non-Rust text operations are local. No runtime remediation is attempted.

## Unchanged predecessor gate

Fetched origin/prod, verified exact commit/tree and clean index/worktree. Neither
local nor remote implementation branch existed. Created both at the exact baseline.
The unchanged branch passed all four push workflows:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38846904919 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38846904912 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38846904978 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38846904965 |

Tests job 116799384664 reports 1,706 passing identities in each complete debug
and release profile: 1,667 reconciled predecessors and 39 KED-028 additions.
Both authority doctests and both 1,800-case affine checks passed; failed, ignored
and filtered counts are zero. `ked-029-predecessors.json` records the exact debug
inventory. Historical inventories and verifiers remain unchanged.

## Milestone 1 — canonical source-region metadata

Sealed MIR carries separate source-region and expression-site identities, exact
types, canonical plan references, destination identities, source origins, written
child order and explicit field-child region nesting. Conditional initializer
conditions and leaves share one source region; calls do not create caller-side
regions. Metadata allocation is fallible and bounded by existing admitted syntax;
shape checks precede exact deterministic correspondence comparison.

This milestone does not admit new runtime resource arguments or claim temporary
execution support. Existing ownership traversal, call holds, borrowing, recurring
conditions and resource-free runtime remain unchanged. Four focused metadata
regressions are classified explicitly in the evidence manifest. The new inventory
verifier requires all 1,706 predecessor identities and all declared additions.

Validation pending. No implementation PR, acceptance, issue closure or merge.

First metadata commit: `d8f30671d966d4aa51b325cd9a04618ac759c53b`, tree
`e0f5bf25048029021381bb2b382ae62886ce718c`. GitHub compilation passed. Build
https://github.com/gregkrsak/klyxr/actions/runs/38847302394 failed Rustfmt; applied
the complete GitHub Rustfmt output as text. Tests
https://github.com/gregkrsak/klyxr/actions/runs/38847302408 found that the new
nested-call metadata fixture reached full resource admission before that layer
is enabled. Its original source witness is retained; it now explicitly checks
the current full-gate rejection and validates canonical record shape before
testing the static metadata lowerer directly. This does not change an accepted
language expectation or claim runtime support. Docs
https://github.com/gregkrsak/klyxr/actions/runs/38847302430 and verify
https://github.com/gregkrsak/klyxr/actions/runs/38847302405 passed. No later layer
started. Metadata repair validation pending.

Milestone 1 completed at `07c68f34ae78465313b5abbae1e50b2e391b761b`, tree
`473d4144adcd14dc38de9b64eb844eb499b00064`. All four push workflows passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38847430093),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38847430100),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38847430222),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38847430132).
Complete debug and release each reconciled 1,706 predecessors plus four new
identities = 1,710 passed. Both affine checks and authority doctests passed.

## Milestone 2 — MIR execution-point authority

Canonical source evaluation ordinals bind regions to MIR statement/terminator
points during the existing structured lowering. Neither spelling, span equality,
nor block numbering supplies region identity. Conditional-initializer conditions
and leaves use their original site-tree identities across generated blocks.
Only the initial point begins the region; selected value leaves complete it.
Field-child sites remain explicit. Creation, transfer, cleanup, retirement and
publication retain distinct origins and destination-specific protection policy.
Point storage is reserved fallibly before lowering and never grows during it.
Bounded shape validation precedes opaque exact correspondence. Runtime admission
remains closed pending the next layer. Validation pending.

First MIR-authority commit `92fb0086459fdc25c066ca4cff8ceec9ba9cae97`, tree
`de93ab1ac03695e1c9380c04ea3a8b225d3a18ea`: compilation and complete debug
tests passed. Build https://github.com/gregkrsak/klyxr/actions/runs/38847834695
reported Rustfmt differences; applied the complete CI formatting output as text.
Other runs: tests https://github.com/gregkrsak/klyxr/actions/runs/38847834629,
docs https://github.com/gregkrsak/klyxr/actions/runs/38847834638,
verify https://github.com/gregkrsak/klyxr/actions/runs/38847834686.
Three added tests cover 26 independent authority mutations, selected initializer
paths and source ordinals with resource-free predecessor statements. No runtime
layer started while this milestone's build gate was red.

Milestone 2 completed at `c352861ff6085b715e1bc52e53fb803ec2996f88`, tree
`c8858d7b95713b25927e11171885217b5866fc25`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38847970102),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38847970119),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38847970097),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38847970098).
Debug and release each: 1,706 predecessors + seven additions = 1,713 passed;
both affine checks and authority doctests passed.

## Milestone 3 — explicit temporary custody and call vertical slice

A separate lazy machine state retains dynamic region activations, payload owners,
creation histories, protected completion results and typed transfer/cleanup events.
Operand handles are not places and own no duplicate payload. Resource registration
retains the actual value stack payload throughout fallible reservations. Transfer
vacates the owner only after validation, fuel and event reservation. Parameter
capacity is reserved before entry; an explicit callee frame retains any published
prefix, while remaining arguments stay staged. Body continuation is queued only
after entry completes. Existing resource-free continuations retain their path.

The vertical slice necessarily installs common result-protection/retirement hooks
at existing MIR completion points; record field continuations transfer membership
before child retirement. Runtime boundary conformance and adversarial failure
evidence remain subsequent milestones. No completion claim is made here.

The two designated obsolete argument rejections retain their original identities
and source witnesses, now asserting successful identity-conserving execution.
Frozen sections 4-5 admit those exact owned arguments, including replacement RHS
composition. The metadata-only nested-call test now also checks normal admission
and the single ownership checker. One additional predecessor assertion changes:
the exact single-owner fuel threshold from 11 to 16, accounting for region entry,
registration, transfer, owner inspection and retirement required by frozen section
14. Its sufficient/insufficient boundary witness is preserved. The existing
later-argument publication-stop fixture recognizes SiteEval as well as Eval at
the same source-read transition; its conservation assertions remain intact.

New vertical evidence distinguishes caller temporary transfers from callee cleanup,
preserves A/B/C acquisition identities and C/B/A parameter cleanup, and traces a
single leaf through an inner protected return and outer entry. The common
conservation census includes actual temporary and protected-result payloads.
Validation pending; later failure and destination batteries are not yet claimed.

Initial runtime slice `1365998011cf0617f83a756b0937842ed7a35f74`, tree
`f8009fdbd2e7ff86cee0bc35afe7a805558cbb41`: GitHub compilation and complete
debug/release profiles passed, each reconciling 1,706 predecessors plus ten new
identities = 1,716 passed. Build
https://github.com/gregkrsak/klyxr/actions/runs/38848759716 reported formatting
differences, applied from the complete GitHub Rustfmt output. Other runs: tests
https://github.com/gregkrsak/klyxr/actions/runs/38848759654, docs
https://github.com/gregkrsak/klyxr/actions/runs/38848759450, verify
https://github.com/gregkrsak/klyxr/actions/runs/38848759628. Both affine checks
and authority doctests passed. Destination-specific return/replacement protection
integration and controlled leftover-owner evidence remain required before the
implementation can be considered complete. No later layer added while build red.

Milestone 3 completed at `18ef6029962634bbb8f894f65cc929cb563bc9e9`, tree
`69f3eabd162449414a21290115dc09da2faeee71`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38848976816),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38848976858),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38848976741),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38848976736).
Complete debug/release profiles each reconciled 1,716 identities; both affine
checks and authority doctests passed.

## Milestone 4 — call cuts and destination protection

Return operands enter the existing protected-return holder before temporary
retirement. Replacement RHS results enter the existing prepared replacement
holder before retirement or displacement. Their downstream operations consume
these already protected results without a second pop or duplicate payload.
NoValue completion stays a control marker and creates no dummy result owner.

Private boundary fixtures use the existing trusted acquisition and canonical
temporary registration to seed leftover U before source RHS evaluation. They
demonstrate unpublished local R retention, exact success neighbors, protected
return/replacement custody, and preserved RHS prefix mutation without the final
assignment write. Private A/B/C retirement is explicitly a primitive oracle,
not execution of a discarded value expression or changed source call.

Every fuel cut of a three-argument entry (including partial parameter publication)
must conserve identities and refuse resume; an exact sufficient neighbor succeeds.
Every byte storage cut of nested argument/return custody must conserve identities,
with the observed exact prospective peak sufficient. Later-argument range failure
retains the earlier acquired owner without callee entry or fabricated unwind.
Nine new test identities are enumerated in the evidence manifest. Validation
pending. Remaining assignments, pending-field stop, structured-condition, bounds,
Host-allocation and artifact batteries remain required before final conformance.

Destination/call-cut commit `eb060edf062069a9f466808bfe466d5bb1a5eabf`, tree
`27f57115cd3dc80dbb4df0d847f93cef38f6d8d9`: compilation and complete debug
and release profiles passed. Each reconciled 1,706 predecessors plus 19 additions
= 1,725 passing identities, including both affine checks and authority doctests.
Build https://github.com/gregkrsak/klyxr/actions/runs/38849550815 reported
formatting differences, applied verbatim from CI output. Other runs: tests
https://github.com/gregkrsak/klyxr/actions/runs/38849550748, docs
https://github.com/gregkrsak/klyxr/actions/runs/38849550785, verify
https://github.com/gregkrsak/klyxr/actions/runs/38849550885. No later layer added
before this milestone's four gates pass.

Milestone 4 completed at `56dc5ec39fa4d08091a7270bec618281b2947c43`, tree
`e03d5a7bcbdd984986dd75316b5d7d8dfb75701a`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38849723088),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38849722960),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38849723026),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38849722891).
Both complete profiles reconciled 1,725 passing identities; both affine checks
and authority doctests passed.

## Milestone 5 — aggregate, assignment and control-flow evidence

Private A/G/B retirement distinguishes sibling creation order from G's canonical
field plan. Pending construction witnesses stop retirement after a middle
canonical field receives a moved named source, and after the last written field
of a nested pending record. Initialized membership, moved source and incomplete
publication state are inspected directly alongside identity conservation.

All four assignment forms have exact success and retirement-stop neighbors.
Conditions retain Bool and retire before selection; a cleanup stop prevents arm
entry. Conditional initializers keep one caller activation across their selected
CFG path and never execute the other leaf. Repeated loop statements receive
fresh activations. Bool wrappers remain legal while direct/nested Move and
reference activity in recurring conditions reject. Parser temporary-borrow
rejections, source value-discard typing and private forged resource-discard
metadata are checked separately. Nine new test identities enumerate these
matrices. Validation pending; Host allocation, admission and final artifact
pressure remain before documentation and complete implementation conformance.

Control-flow/aggregate evidence commit `948c9862d87032d5433a44bbe3071498322ceb67`,
tree `3de77804616b002c445a98cff7d689d1f5f7a538`: compilation and complete debug
and release profiles passed, each reconciling 1,706 predecessors plus 28 additions
= 1,734 passing identities. Both affine checks and authority doctests passed.
Build https://github.com/gregkrsak/klyxr/actions/runs/38850141141 reported only
formatting differences, applied from the complete GitHub Rustfmt output. Other
runs: tests https://github.com/gregkrsak/klyxr/actions/runs/38850141257,
docs https://github.com/gregkrsak/klyxr/actions/runs/38850141181,
verify https://github.com/gregkrsak/klyxr/actions/runs/38850141160.
No later layer added before the four-workflow formatting repair gate passes.

Milestone 5 completed at `5423989c82a2a8fa08b0f790e0350e1c52f66aa8`, tree
`2d8b6e317fbdc29e41177e68679655e04e71457a`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38850319723),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38850319794),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38850319781),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38850319730).
Both complete profiles reconciled 1,734 identities; both affine checks and
authority doctests passed.

## Milestone 6 — allocation, admission and holder evidence

Test-only fault probes sit immediately before actual vector reservations in the
new temporary state, call/value/continuation buffers and existing cleanup holders.
A cut-by-cut fixture requires Host classification and conservation, including
partial callee publication. A separate acquisition cut proves no acquisition or
responsibility exists before successful fixed-operation publication. Static
temporary-authority allocation cuts preserve Host classification, while exact
admission work/storage neighbors preserve Configured Limit classification.

The physical identity census is cross-checked against named holder paths covering
slots, operand staging, temporary owners, every protected destination, pending
fields, detached payloads and cleanup cursors. A same-leaf witness observes caller
temporary, inner parameter, protected return, new caller temporary and outer
parameter in order; release is attributed to that outer parameter, not caller
temporary cleanup. Runtime malformed-custody controls reject wrong destinations,
foreign/missing owners, duplicate transfers and vacancy marked as owned.

Admission checks table lengths before scanning references and bounds parent-chain
nesting. Source sites and cleanup work use their sealed origins. Registration
checks combined owner/event growth before moving the staged payload. Completion
and retirement each charge before frame traversal; the exact single-owner
resource threshold is now 18 (the original 11 plus seven explicit temporary
progress charges). The historical 16-unit milestone result is retained above;
its old assertion was corrected for these previously uncharged traversals under
frozen section 14. Resource-free paths enter no temporary machinery.

Forged resource-leaf construction now rejects outside the already validated
canonical acquisition stub. This preserves the predecessor trusted-acquisition
contract when admitting new temporary positions; it creates no constructor or
resource API. Seven new test identities are explicit in the manifest. Validation
pending; no final implementation or acceptance claim.

Adversarial commit `41fce84ad8ea2dba3f1ed364c15a76f9babb17ff`, tree
`a7d7c2611f897a8f81b856d264e2372ee90c4742`: compilation and complete debug
and release profiles passed, each reconciling 1,706 predecessors plus 35 additions
= 1,741 passing identities. Both affine checks and authority doctests passed.
Build https://github.com/gregkrsak/klyxr/actions/runs/38851327364 reported only
formatting differences, applied from the complete GitHub Rustfmt output. Other
runs: tests https://github.com/gregkrsak/klyxr/actions/runs/38851327325,
docs https://github.com/gregkrsak/klyxr/actions/runs/38851327345,
verify https://github.com/gregkrsak/klyxr/actions/runs/38851327393.
No documentation/final layer begins before this four-workflow repair gate passes.

Milestone 6 completed at `59491c960327b87d43c5c73f792b720f35203cf2`, tree
`f347b20e4860741e70a361dc2baf1e2bced0472f`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38851658653),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38851658648),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38851658660),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38851658680).
Both complete profiles reconciled 1,741 passing identities; both affine checks
and authority doctests passed.

## Milestone 7 — remaining cleanup-stop pressure

The U/R fixture now checks every configured fuel cut after private U seeding,
requiring unpublished protected R across stopped retirement, exact conservation,
refused resume and an exact sufficient neighbor. The private A/G/B primitive
injects trusted-release failure independently at every leaf in canonical retirement
order, verifies completed prefixes and identifies the failed cleanup holder and
unvisited temporary A. Two new identities are explicit in the manifest.

The existing independent artifact matrix expands from 26 to 37 single-fault
mutations, adding well-formed foreign cleanup/transfer/publication origins,
foreign source sites and regions, wrong type, missing retirement/publication
origins and unrelated child registration. All original cases remain intact.
Validation pending; documentation and final branch/PR gates remain ahead.

Remaining-cut evidence commit `09770710127ef48eb84185e49e40d9ae17ad5ea8`,
tree `ccf5abde03f0c66646a02f6a6a17c7a2ced8062c`: compilation and debug
tests passed. Build https://github.com/gregkrsak/klyxr/actions/runs/38851977267
reported formatting differences, applied from the complete CI Rustfmt output.
Other runs: tests https://github.com/gregkrsak/klyxr/actions/runs/38851977323,
docs https://github.com/gregkrsak/klyxr/actions/runs/38851977322,
verify https://github.com/gregkrsak/klyxr/actions/runs/38851977308.
No later layer added while this milestone's build gate was red.

Milestone 7 completed at `cc48724d8bb1ef8b0e1a963b8ee0095ca5ff8188`, tree
`e172d43df93b2eb51ce5808ce902d73313d4d656`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38852179648),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38852179652),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38852179651),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38852179655).
Both complete profiles reconciled 1,706 predecessors plus 37 additions = 1,743
passing identities, zero failed/ignored/filtered. Both 1,800-case affine checks
and compile-fail authority doctests passed.

## Final conformance and documentation

The evidence manifest lists all 37 additions individually. Its `migrations` map
means identity renames and is empty. The separate `contract_corrections` list
records two preserved-ID admission corrections and the fuel threshold correction;
`fixture_adaptations` records the unchanged named-source stop witness adapter.
The reconciliation script now prints these categories explicitly. Historical
KED-027/KED-028 inventories and reconciliation scripts remain unchanged.

| Frozen requirement / pressure | GitHub-executed evidence |
| --- | --- |
| Source regions, CFG, canonical sites and origins | Four MIR tests, source ordinal and selected-leaf tests, 37 independent single-fault authority mutations |
| Direct/nested/multiple owned arguments, one transfer | Three vertical call tests, partial-entry fuel cuts, exact caller/parameter holder census |
| Reverse sibling order and canonical aggregate order | Private A/B/C exact C/B/A retirement, A/G/B order and each trusted-release failure |
| Protect before local publication | U/R success, Host stop and every configured fuel cut; unpublished R and completed prefix census |
| Existing assignment forms and RHS effects | All four forms, retirement-stop neighbors, prefix mutation retained without pending write |
| Protected return and KED-028 replacement | Dedicated protected-holder stop tests; predecessor A/B/C replacement witness preserved |
| Field child regions and canonical pending membership | Middle-field named move and nested pending last-written middle-field stop tests |
| Conditions and dynamic activations | Retirement before selection, selected arm only, CFG-spanning initializer, fresh loop-body activations |
| Exclusions and authority isolation | Source borrow/discard/while rejection, malformed artifacts, forged acquisition rejection, compile-fail opaque authority doctest |
| Bounds and terminal failures | Exact admission and storage neighbors, every call fuel/storage cut, actual reserve probes, failed acquisition, trusted release, refused resume |
| Conservation and cost preservation | Independent identity census of physical holders; resource-free predecessor suite; both affine checks |

Implementation-facing context, decisions, architecture, open questions, compiler
README and bounded-execution documentation now describe frozen Draft 2. The context
bundle is regenerated and checked byte-for-byte; README/assets and scaffold checks
are run locally using Python and repeated by GitHub. No local Rust, Cargo, Rustfmt,
Clippy or executable-test result is claimed: local Rust validation was unavailable
because of confirmed Work-runtime corruption. Formatting repairs above apply text
from GitHub Rustfmt logs; all executable validation belongs to GitHub Actions.

The full baseline-to-head source diff, predecessor inventory provenance, manifest,
workflow changes and generated documentation are audited before PR creation. No
frozen semantic change, non-goal expansion or deferred facility was required.
The exact publishing head/tree and completed documentation push gate, followed by
independent pull-request workflow URLs, are recorded in Issue #66 and the linked
implementation PR after those events exist. This avoids self-referential commit
identifiers in a versioned ledger. Independent implementation review and Greg's
acceptance remain outstanding; Issue #66 stays open and only a human may merge.
