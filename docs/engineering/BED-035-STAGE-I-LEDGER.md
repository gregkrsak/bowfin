# BED-035 Stage I engineering ledger

Authority: Issue #84 comment 6296001700; frozen Draft 2 and its freeze record
6295819136; accepted K35-P01–P10 review 6295569435. The implementation branch was
`ked-035-approved-mir-execution-equivalence`; PR #85 was independently reviewed,
accepted by Greg and human-merged into `prod`.
Baseline commit `5f28977a42a7df1817e5cac46b3f6660a38435d3`, tree
`0b330328f3f213a26b6fc4af9f45ca06a98876b8`.

This ledger records the accepted Stage I implementation and its closeout. It
grants no permission for Stage N. GitHub Actions was the only Rust compilation
and execution environment used; no local Rust installation, repair, compiler,
formatter, linker or executable test was run.

## Focused I03 / I04 repair and confirmation

Sarah's focused confirmation (Issue #84 comment 6306899488) resolves K35-I01
and confirms the original I02 lawful last-use witnesses. It also identifies two
Major blockers at head `23dd37ca32c0ec6c9df34fc3cbc44b0a7371060a`, tree
`6536a0a21ba266ef4a29880f0bfeac02bb08ee1b`. Greg authorizes only I03/I04 repair.
All implementation history and M5 remain normal ancestors. The repaired head was
later independently confirmed, accepted and merged; none of those events grants
authority for another project step.

I03: `compare/transitions.rs` binds newly queued expressions to canonical operands
of the approved statement/terminator or already authorized parent evaluation.
It preserves the producer's frame, operand order, depth and continuation relation.
Frame cursors start at canonical entry and advance only along the producing
approved statement or selected control edge. Field staging cursors retain their
canonical production order; incomplete operand scheduling cannot start a new
action. These are existing MIR control facts, not an ownership/lifetime analysis.
Queue prefixes persist unchanged across cuts; reached tasks consume their prior
validated queued representation. Canonical parent operands, rather than agreement
between two mutable fragments, establish the relationship. Field staging and
short-circuit continuations carry their canonical operand relationships forward.
The checker resolves immutable MIR facts; it evaluates no source values, recomputes
no future uses or loans, and grants no execution or cleanup authority.

I04 evidence correction: the former duplicate and stale artifact assertions
changed capture length without its sealed count. Their observed InvalidState
proved the shape gate, not the advertised reference protections. The earlier
report's description of all eleven identities as discriminating was too strong.
Those identities are retained with corrected assertions. Private builders now
preserve length, or explicitly keep only their own malformed envelope coherent.
Production sealing remains mandatory and has its own separately labeled control.

The coordinated witness retains two independent complete sessions producing bool
false and Equal, then changes only final `Local(value)` ordinals to the earlier
`Deref(shared)` in both queued and reached records. Length, sealed count, capacities,
primary outcomes, inventory, cuts, origins and accounting remain intact. Operand
validation must reject it. A second length-preserving witness substitutes siblings
inside one approved Binary expression, so same-instruction membership alone cannot
satisfy canonical operand order. A reached-only substitution instead rejects at Queue.
The duplicate fixture keeps the original production available throughout transfer,
so Duplicate is the first failing protection. The resurrection fixture changes
only the final stopped snapshot, returns bits to the Moved source and removes the
alias's bits, so Resurrection is decisive without simultaneous duplication. A
lawfully queued/reached dereference with an unavailable referent rejects at Access.
For Operand, Queue, Duplicate, Resurrection and Access, each principal fixture
checks its named rejection and requires Equal when only that specific private
test guard is removed. An extra cursor subcontrol checks Control rejection and
shows that disabling Control reaches a separate Operand rejection; it does not
claim an Equal counterfactual. These test-only switches are thread-local,
restored by a guard, and absent from production. Actions must execute all assertions
before any test is reported as passing.

The I01 canonical ordinary origin resolution remains unchanged. Stored references
retain kind, nominal referent, dynamic frame/place, initializer/alias/transfer and
caller-argument identity without asserting current loan activity. The original
I02 positives, repeated scopes/calls/productions, exclusive transfers and wrong
frame/referent/permission controls remain. Source borrowing/lifetimes still have
one authority: the existing ownership checker (KD-021/KD-024/KD-029/KD-030/KD-039).
This corrects K35-P03/P04/P07/P08 validation/evidence under KD-041, not language rules.

All scans debit comparison Work. Canonical operand views are borrowed; queue
history uses bounded record ranges and fixed scalar producer state without new
heap allocation or recursive traversal. Existing reference history is pre-reserved,
charged and capped; exhaustion remains Inconclusive. No primary layout, accounting
schedule, custody, capture vocabulary, public API or selected-profile gate changes.
These are logical evaluation witnesses, not a requirement that a later engine
physically retain interpreter tasks or frames; K35-P09 remains binding.

Expected inventory: 2,045 predecessors + 48 original reviewed K35 controls + 11
retained focused identities + 8 new classified I03/I04 identities = 2,112 per
complete profile. The manifest and verifier pin both retained identity/classification
sets. Exact executed debug/release totals, eight run/job identities, changed files
and readbacks belong to the final exact-head repair report. Actions and Sarah's
focused confirmation remain pending for this candidate. Finite controls are not
universal proof; Stage N evidence remains deferred, not waived. PR #85 stays draft
and unmerged for Sarah's confirmation and Greg's acceptance decision.

## M0–M5 implementation and checkpoint scope

M0's exact-tree checkpoint and eight successful push/PR workflows are recorded
in `ked-035-evidence.json`. M1 published `KED-035-CONTRACT-INVENTORY.md`, including
baseline charge locations, public boundaries, domains, cuts and precedence.

M2's `execution::contract::Contract` is a bounded immutable borrowed view of the
same opaque `CheckedExecutable`. Extraction consumes the original validation and
entry gates, visits the entire structural ordinary call closure (including
empty-effect calls, recursion and unselected alternatives), and indexes exact
MIR operands/sites. The view retains all existing sealed nominal, origin,
ownership/outcome, cleanup/effect/domain and trusted facts through that checked
composite. Validation re-extracts canonical correspondence. Source similarity,
independent recompilation and independently changed schema/site/closure cannot
replace approval identity. This is no second ownership solver or execution IR.

M3 seals a versioned semantic/accounting/cut/observation/replay schema and 48
compatibility quantities into `Approval`. Reference interpreter capacities are
charged through that schedule. Accounted predecessor structs/enums have not been
expanded for instrumentation; new sidecars and scratch belong to separate
accounts. Reference-build sizes preserve the version-one implementation's logical
charges, not a native ABI. A later engine must retain logical capacity and scan
history even when physical allocation, frames or dead slots are eliminated.

M4 adds opt-in separately admitted non-owning capture on the existing Machine.
Ordinary no-profile and already selected-profile sessions use their original
execution authority. Capture is pre-reserved and records actual state, logical
continuations, counters, capacities, aliases, generations, Result parents,
active payloads, staging, replacement, temporary custody, trusted receipts,
productions, completion and retained event prefixes. Runtime custody markers
identify primary local transit and protected publication without owning, copying
or releasing a live resource. Capture failure leaves primary execution unchanged.
The terminal census and cached result come from the actual retained session;
repeated advance is read-only. Capturing does not consume reserved diagnostic
backing. A capture-complete flag alone is never a validity certificate.

M5 independently parses the flat record grammar and validates local shape,
nominal types, tags, canonical source/call/arm/region facts, active holders,
parentage, generations, aliases, payload/provenance associations, capacities,
exact compatibility storage, histories and primary delivery/stop consistency.
Both full views validate before one forward/inverse injective map is extended in
approved logical occurrence order. No raw addresses or raw cross-process IDs
are compared. Expression pointers are local immutable-MIR lookup keys only.
Malformed, truncated, foreign, stale, unmatched physical Host and exhausted
validation/comparison evidence yield Inconclusive, never Equal. A locally typed
engine defect with complete consistent evidence can yield Different.

All capture, graph, lineage, history and work traversal is bounded. Comparison
scratch is pre-reserved; successful output is a fixed scalar result. Flat Copy
records contain no recursive heap-owning payload or cleanup authority. Validation
never repairs the primary session or executes a cleanup plan. Existing whole-
Result cleanup authority remains restricted to its separately sealed domain.

## M5 evidence families

| Family | Discriminating evidence | Claim and limitation |
|---|---|---|
| Independent oracle | Hand-derived entire bool-return trace, every counter/capacity/cut, plus exact fuel 3/2 neighbors | Expected sequence is not generated by the interpreter, capture, replay or generous-run prefix. Static approved origins and sealed units are shared premises. |
| Observation boundaries | Independent exact record/storage sufficiency and one-short neighbors; exhausted capture and comparison work | Primary result/counters remain unchanged; incomplete evidence never establishes Equal. |
| Three outcomes | Complete Equal; actual well-typed wrong bool and conserving Sensor-parameter swap yield Different; malformed or incomplete views yield Inconclusive | Finite decisive fixtures, not universal equivalence proof. |
| Actual identity variation | Fresh selected realms and actual frame allocator seeds; shared alias equality and ancestry | Real interpreter executions, no native engine claim. |
| Analytic representation variation | Consistent relabeling of all dynamic record classes, including composite temporary owners and Result/provenance identities | Private schema/correspondence fixture; not independent execution or Stage N evidence. |
| Malformed independence | Foreign realm, stale generation/alias, bad tag/custody, unbound origin and coordinated corruption | Validation precedes projection; equal-looking malformed copies cannot self-certify. |
| Ordinary source | No-profile bool/range/record/Result, no-value calls, shared/mutable parameters, branch/match, short-circuit and loop fallthrough/break/continue; nested success/failure propagation | Existing source boundaries and selected-only execution retained. |
| Private protocol | Sealed synthetic Leaf acquisition, replacement and loop cleanup | Explicit private primitive/protocol evidence; grants no production source resource API. |
| Predecessors | Exact KED-034 1,981 predecessors plus 64 additions, including all inherited KED-033/032 identities, opaque-authority doctest and 1,800 affine cases | `verify-ked035-tests.py` reconciles identities rather than totals; no migrations or weakened assertions. |

## M6 — addressed replay and modeled abstract Host

`conformance/replay.rs` consumes a private bounded Input attached to the same
approval, schema, entry and ordinary execution limits. Sites use approved
function/origin or invocation identity, logical activation ancestry, site
occurrence and phase. They do not use a raw frame, ticket, allocation attempt,
source spelling, native address or backend instruction count. Positive logical
receiving growth remains an address even if a later engine preallocates; zero
growth is not a new capacity event. The analytic preallocation fixture tests this
address law without claiming a native implementation.

Static applicability, duplicate entries, response types, closed error masks,
prefix lengths and release birth associations validate before entry. Dynamic
responses are consumed only after inherited fuel/storage/depth and custody gates
reach the event. A missing reached response rejects without falling back to a
fixture. Future unvisited items remain unconsumed. Open/read/finish and existing
fixed release phases retain their original external protocol; instrumentation
adds no source operation or authority edge. Receipt-to-holder publication remains
atomic where the predecessor contract made it atomic.

Actual consumed controls and responses are independently checked against actual
receipt, production, event prefix, activation and resource association. Finish
partial failure retains its irreversible prefix; release failure retains the
protected outgoing Result without delivery. Modeled Host is a common logical
input only when its reached witness and primary stop agree. Surviving physical
Host remains unmatched/Inconclusive. Capture failure, invalid evidence and lost
substrate evidence cannot become Equal. No rollback, retry, implicit cleanup,
resume or fabricated opposite variant occurs.

M6 passed all eight exact-head workflow/job/step sets at
`79c5166f0a8178d73f1a3085f9178f89939b7e46` (37 additions, 2,082 per profile).
The preserved M5 checkpoint is
`e31f6104d8222ee201bc392a3daf87bcb03734ab`, tree
`34466f3032475e6b03931f59e61e712fab3b54e2`; its eight successful gates and
verified PR checkpoint publication preceded all M6 source changes.

## M7 — integrated authority, primary execution and presentation

`conformance/authority.rs` retains typed immutable references to the complete
approved type/MIR context, exact catalog, roots, structural call edges including
empty effects and skipped alternatives, invocation plans, cleanup facts and
temporary sites. Extraction and canonical revalidation have a separate bounded
row/work/storage account. Ordinary no-profile evidence does not obtain catalog
or execution authority. Possible structural effects and grants come from the
approved structure, never the runtime event prefix.

`Session` integrates ordinary Machine or the existing selected ExecutionSession.
The legacy run/CLI/profile/zero-parameter-root/verified/harness boundaries remain
unchanged. The only added public surface is bounded read-only contract and
conformance tooling over an existing CheckedExecutable. Controlled input and
modeled Host remain private. The summary renderer has a separate output allowance;
text, allocation failure and verbosity cannot replace primary execution or decide
semantic equality. The existing public audit renderer and its exact budgets are
unchanged. The new summary is not a serialization or external authority format.

M7's final source gate at `20d23dcafb0bceaf72e25e037c820fe6d3191143`
passed all eight workflows and every applicable job/step, including warning-free
Clippy, Rustdoc, formatting and both complete Cargo profiles. All 2,045 exact
predecessors and 48 classified additions passed: 2,093 per profile.

## M8 — complete finite evidence and documentation

`ked-035-evidence.json` is the detailed machine-readable requirement-to-code/test
matrix: §§4–18, all thirty §15 obligations, K35-P01–P10, malformed families,
M0–M9, evidence classifications, boundedness, public interfaces and deferred Stage
N evidence. `verify-ked035-contract.py` checks completeness and references.
`verify-ked035-tests.py` independently reconciles actual complete Cargo identities
in each profile, retaining the prior KED-034/033/032 maps, the opaque-authority
compile-fail doctest and all 1,800 affine cross-check cases. Existing predecessor
assertions and numeric boundaries are not weakened or migrated. Two forged
Approval fixture constructors merely supply the new original schema/schedule
fields; their malformed predicates remain unchanged.

Independent decisive evidence includes the hand-derived whole bool trace and
first-insufficient fuel trace, external event prefixes for every 1–72 fuel
neighbor of the exact predecessor direct-open/finish source, terminal mutation
and range-trap custody, actual two-resource association swaps, actual wrong read
payload with the same final Error, and completion/release-phase responsibilities.
The finite larger sample enumerates every distinct reached logical storage gate
and checks first-insufficient precedence and custody; discovery selects budgets,
not expected success snapshots. Recursive depth, short-circuit nonconsumption,
loop activation and terminal repetition are exercised. Finite fixture enumeration
is not proof of every program or every conceivable substrate failure.

Malformed evidence is altered independently of its bound authority or oracle:
schema, closure, site, nominal tag/payload, active holder, realm, alias, generation,
continuation, branch target, scope, completion phase, production/invocation,
accounting, origin and replay marker corruption all reject before projection.
Equal-looking jointly corrupted copies cannot self-certify. Full view validation
precedes mapping. A valid but wrong actual read or resource association can yield
Different; incomplete or malformed views yield Inconclusive. Analytic raw-ID
relabeling is explicitly separated from real interpreter identity variation and
from independent expected observations.

The five canonical AI documents now record accepted and merged Stage I while
preserving separate Stage N authority. The generated context bundle is regenerated
using `scripts/build-ai-context.py` and checked byte-for-byte in CI. No source
language decision number or native implementation claim is invented.

## Binding implementation constraints

| Finding | Implementation disposition |
|---|---|
| B35-P01 | Borrow and validate complete exact approved authority; no second lowering or ownership authority. |
| B35-P02 | Sealed version-one common schedule and separate observation/audit/presentation domains; exact predecessor neighbors retained. |
| B35-P03 | Bounded pre-reserved complete actual observation; failure preserves primary and prevents Equal. |
| B35-P04 | Independent local validation before one anchored injective dynamic map; raw IDs are never cross-engine equality. |
| B35-P05 | Logical site/activation/occurrence/phase and receiving-growth shadows; future engines must preserve eliminated logical work. |
| B35-P06 | Reached-only consumption, inherited gate precedence, no-gap custody, protected non-delivery and terminal non-resumption. |
| B35-P07 | Independent whole-trace and custody oracles plus actual Equal/Different/Inconclusive controls; no interpreter self-certification. |
| B35-P08 | Ordinary and selected-profile integration without broader source/runtime authority; structured audit is separate from events/text. |
| B35-P09 | Preserve the semantic relation independently of layout, frame IDs, preallocation or future optimization; no optimizer chosen. |
| B35-P10 | Mandatory native identity/installation/actual optimized and unoptimized comparisons remain deferred Stage N evidence. |

## Bounds, accounting and custody

Contract extraction admits nodes/work/storage and accounts simultaneous canonical
re-extraction buffers. Capture uses flat non-owning Copy records, finite lineage,
record/storage/work maxima and pre-reserved storage. Comparison pre-reserves
bounded nodes, links, slots, histories, event scratch and both map directions;
validation and comparisons charge work and return a fixed scalar. Replay admits
finite sites/items and conservative work for counters, ancestry and histories
bounded by execution fuel/depth; it performs no allocation during binding.
Authority rows and revalidation buffers have their own budget. Evidence disposal
owns no resource and invokes no source cleanup plan. All size arithmetic uses
checked admission or fails conservatively. Finite maxima and exact defaults are
in the implementation; reference geometry is not a native ABI guarantee.

The 48 sealed quantities preserve predecessor receiving capacity and prospective
combined sums. Accounted Frame/Task/Value/Fault/State layouts were not expanded;
Machine sidecars are outside those existing logical allocations. Whole-Result
cleanup-plan identity does not grant authority to execute cleanup. Structural
cleanup-effect unions remain static even though only one runtime payload is
active. Primary local transit/publication markers observe custody without
creating another owner or second cleanup authority.

KED-034 numeric measurements retain audit 65,617 storage / 65,584 work;
diagnostic 126 and result 20; catalog diagnostics 9,792,
protocol bytes/site 1,320, storage 1,695,808, work 102,656; combined receiving
failure 3,848 (retained 3,672, largest 176, sum growth 680); physical growth probe
mask 1,918; preparation diagnostics 44,608, identifier bytes 13, storage
18,559,608, tokens 35, work 1,344,662,663; direct-open/finish runtime fuel 72 /
storage 4,448; stopped-report audit storage 69,969 with 4,352 retained. These are
reference Rust 1.80.0 x86_64 test-build evidence, not portable physical costs.

Audit text output measured 6,193 bytes in the fresh M0 suite and 6,199 in the
expanded Stage I suite. The unchanged renderer includes process-local catalog
serials and packed operation identities in Debug text; extra preparations alter
those identities and their decimal widths. A fixed cross-run text length is not
a semantic invariant. The original tests still require exact sufficient and
one-byte-short output allowances for each actual approved artifact, and the
renderer, allowance rules and static authority are unchanged. No normalization,
old-test relaxation or authorized-boundary migration is applied.

## Public API and changed-file accounting

Public metadata additions are `execution::contract::{Schema, Schedule, Domain,
Limits, Role, Site, Contract}` and its read-only extraction/validation/accessors;
`execution::conformance::{Limits, Incomplete, Session, Comparison,
ComparisonLimits, compare}` including Session construction, advance, capture
status/count and bounded authority-summary rendering. `Schema::V1` is sealed;
public observations cannot construct Approval. `completeness()` reports capture
completion, not a semantic validity certificate. There is no public replay input,
resource constructor, new profile or engine-selection interface.

Implementation files comprise contract.rs; execution/mod.rs; Machine and its
cleanup, replacement, results, temporaries, trusted and completion modules;
conformance.rs, authority.rs, compare.rs, replay.rs and their dedicated identity,
oracle and replay tests; and the two forged-Approval fixture modules. Documentation
comprises five canonical documents, their generated bundle, this ledger, the
contract inventory and JSON evidence map. CI changes comprise docs.yml, tests.yml
and the two new KED-035 verifiers. The final M9 report gives the exact head's full
changed-file list and tree; no unrelated repository configuration is changed.

## M9 — final gate, acceptance and merge closeout

The final implementation head is
`837fc99e2bbb241077d4b65b32a4103b6bf99362`, tree
`a76e2ab68c8d2db094f1005906bdc24b6a9b8795`. Its complete debug and release
profiles each pass 2,112 identities: 2,045 predecessors, 48 original reviewed
KED-035 controls, 11 retained first-repair identities and 8 I03/I04 identities,
with zero predecessor migrations.

All eight exact-head workflows passed:

| Event | Build | Tests | Verify | Docs |
|---|---:|---:|---:|---:|
| push | 39590998868 | 39590998853 | 39590998901 | 39590998874 |
| pull request | 39591008109 | 39591008122 | 39591007962 | 39591008089 |

Arnold's identical complete reports are PR comment 6314789174 and Issue comment
6314791196. Sarah's final focused confirmation is PR comment 6315882077. It resolves
K35-I01 through K35-I04, assigns no K35-I05, and reports READY FOR OWNER ACCEPTANCE.
Greg accepted the implementation and authorized human merge.

Human-merged PR #85 is
`c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717`, tree
`a76e2ab68c8d2db094f1005906bdc24b6a9b8795`, with the frozen baseline and exact
reviewed head as parents. GitHub verifies the merge signature. All four post-merge
push workflows passed: build 39600440449, tests 39600440157, verify 39600440116
and docs 39600440427.

Some early implementation commit subjects used non-prescribed prefixes. This is
a nonfunctional repository-process deviation; history and the required M5
checkpoint remain intact. No local Rust installation, repair, compilation, linking,
formatting or executable testing occurred.

Remaining limitations are unchanged: finite fixtures are not universal equivalence
proof; private Leaf/protocol fixtures are not production source permission;
analytic relabeling/preallocation are not native tests; contract/static rows use
exact in-process immutable references, not portable transport; insufficient evidence
is deliberately Inconclusive; and no performance improvement is claimed.

Native production, target/runtime installation, pre-execution artifact mismatch
checks, actual optimized/unoptimized comparisons and physical/native/substrate
evidence remain mandatory deferred Stage N obligations. No backend, target, ABI,
layout, emitter, object format, runtime ABI, linker, optimizer, serialization,
signing or packaging mechanism is selected. Stage N requires a separately reviewed
and explicitly authorized BED.

This documentation-only closeout changes no frozen semantics, implementation,
test identity, workflow or public API. Merging its PR may close Issue #84 as
completed; it does not authorize Stage N or any later project step.
