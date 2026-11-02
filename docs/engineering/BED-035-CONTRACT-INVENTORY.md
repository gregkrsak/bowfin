# BED-035 Stage I: M1 contract and compatibility inventory

Authority: frozen Issue #84 Draft 2, freeze 6295819136, implementation directive
6296001700 and accepted B35-P01–P10. This inventory does not authorize a migration.
Baseline: `5f28977a42a7df1817e5cac46b3f6660a38435d3`, tree
`0b330328f3f213a26b6fc4af9f45ca06a98876b8`.

## M0 evidence

Empty checkpoint `0443dfd32dec70e0c72f1a0a886b9eedc284fb9c` retains the
exact baseline tree. Push and pull-request build/tests/verify/docs all succeeded.
The evidence manifest records all eight runs. Each complete debug/release profile
reconciled 1,981 exact KED-034 predecessors plus 64 additions: 2,045 identities.
No local Rust command was run. PR #85 remains draft. Worktree/index were clean
before M1. M0 reports: Issue comment 6299558481 and PR comment 6299557947.

## Complete sealed fact inventory

| Frozen clause | Existing authority | Required contract treatment |
|---|---|---|
| 4, 16 | execution::Approval and CheckedExecutable; Rc identity; program/outcome/cleanup/domain/trusted copies | Bind schema, schedules and cut vocabulary to this same opaque approval; reject foreign or altered components. |
| 4 | Program ranges/records/fields/parameters/locals/Result/resource tables | Preserve exact static identity and nominal roles; share approved tables, never infer them from observations. |
| 4, 10 | MIR statements/terminators and typed operands, origins, lexical cleanup and temporary Flow | Extract complete ordinary call closure and canonical expression/site order; include empty-effect calls, both alternatives, recursion with visited state. |
| 4, 8, 9 | ownership::outcomes::Witness; effects::custody::Site; cleanup/variant plans | Consume successful-checker evidence without another ownership engine. |
| 4, 10, 14 | trusted::Approved/Invocation/CompletionPlan; Selection/Catalog/contracts/grants/roots/edges/assumptions | Preserve operation-specific exact authority and full completion/release protocol; audit is static, not an event log. |
| 5, 9 | Machine frames/slots/values/tasks; Result cells; temporary activations; cleanup generations/pending/protected/current/work; replacement contexts | Separately admitted non-owning history and final-state census; capture actual current holders and logical parent/association edges. |
| 5–7, 13 | ExecutionSession Status/realm/receipt/entries/productions/events/completion | Validate before projection; one anchored injective dynamic correspondence; never compare raw realm numbers. |
| 6–8, 17 | Invocation site/region/destination/origins, CompletionPhase and actual custody seams | Version logical site/activation/occurrence/phase addresses; replay supplies behavior, never authority. |
| 5, 8, 14 | bounded, audit::render, render::Output, run_reporting | Separate complete logical observations from provenance, physical measurement and bounded presentation. |
| 12, 15, 18 | no native engine exists | Stage N is deferred, not passed; no target, ABI, emitter or runtime selected. |

## Accounting domains and precedence

The manifest retains the baseline textual locations of charge/reserve/size terms,
including test-only occurrences for provenance. Runtime JSON serialization is not
introduced by this documentation/evidence file.

| Domain | Account and sufficiency | Refund/retirement and precedence |
|---|---|---|
| Common preparation | bounded reader/lexer/tokens/identifier/traversal, conservative frontend/work/storage/diagnostics, outcome and catalog envelopes | Existing non-refundable shared work survives checker rollback; admission and canonical correspondence precede execution. No new comparison budget changes these public boundaries. |
| Logical fuel | Statement, terminator, evaluated expression and function entry plus Result/temporary/cleanup/replacement/trusted progress/scans | Successful charges accumulate; skipped work is uncharged. Fuel gates precede the action they protect. A failed charge does not execute it. |
| Logical depth | Ordinary activation count and expression traversal depth | Activation lifetime follows approved calls even under inlining; preserve local gate order before capacity/Host gates. |
| Compatibility storage | Existing vector capacities/backing, protected/detached payloads, frames, operand/task buffers, Result/temporary/resource/trusted state | Count simultaneous retained payload and receiving growth. Capacities remain charged until actual prescribed retirement. Empty/unused predecessor state remains lazy. Do not add fields to accounted layouts without protecting old charges. |
| Trusted progress | Invocation charge plus 8*m+8 scan units; release progress plus m+4 scan units for retained token-table m | Physically compacting entries cannot erase their abstract compatibility history. Existing current-holder and grant gates remain ordered. |
| Completion reservation | Simultaneous provenance/events/operands/Result/temporary/cleanup capacity and pre-reserved diagnostic backing before handoff | No fallible gap after taking the current owner. Partial prefix, protected Result, fixed release and delivery remain distinct. |
| New conformance observation/comparison | Separately admitted record/value/edge/history/lineage/work/output bounds, pre-reserved before capture | Failure marks evidence incomplete/Inconclusive without altering primary execution or granting cleanup. No unbounded history inference or retry. |
| Audit | Existing rooted structural reachability/work/output plus retained-session envelope | Existing public byte/work boundaries remain; structured conformance content gets its own named accounting. |
| Presentation/writer | Selected renderer byte allowance and writer Host path | Reporting does not rewrite retained execution. Host precedence and static emergency diagnostics remain exactly inherited. |
| Backend production/physical engine | No new implementation here | Distinct from source logical limits; native-only evidence deferred. |

Reference representation-derived quantities may be preserved as explicitly bound
version-one compatibility charges. Another engine cannot derive them from its own
physical types. Baseline measurements are scoped to their recorded build, not a
new portable ABI. Any conflict requiring public-boundary migration is a stop.

## Logical cuts and state transitions

The cut inventory must include ordinary statements/terminators/expressions and
call/loop occurrences; assignment publication; scope entry/initialization/exits;
record child transfer; Result construction/protection/tag selection/payload
transfer/arm entry; propagation selection/forwarding; replacement prepared,
protected, detached, discharged and publication phases; temporary entry,
registration, transfer, retirement and protection; acquisition/read envelope,
handoff/attempt/response validation/protection/publication; completion reserved,
consumed, attempted, outcome-protected, release-pending/releasing/released/delivered;
and actual fixed-release attempt/completion/failure.

Sites and applicable phases derive from approved MIR and existing protocols.
Source spelling, native instruction, allocator growth or a bare global invocation
ordinal is insufficient. Logical activation ancestry and per-site occurrence are
retained independently of physical frames. A preempting inherited gate consumes
no later replay or modeled failure. Legitimate future items after a stop remain
unused. Unsupported phase/site pairs reject at admission; reached malformed
responses retain the actual prefix/current custody and stop without fallback.

## Public boundary inventory

Keep source syntax/types/ownership/effects unchanged; zero-parameter public roots;
no reference or live-resource egress; selected profile required for trusted
ExecutionSession, no selected profile in ordinary execute; exact grants/catalog
occupancy and cleanup-only release; no verified/harness execution; no source
engine identity; no new user binding, input ABI or production replay authority.
Contract/capture adapters may use private APIs and separately admitted metadata.
Normal run does not acquire observation history or enable a catalog just to make
comparison possible. Terminal advance remains idempotent and non-resuming.

## Evidence status

M1 is static inventory. M2–M8 consumed implementation and finite evidence are
recorded in `BED-035-STAGE-I-LEDGER.md` and the complete JSON matrix. The distinct
M5 checkpoint obtained all eight applicable successful Actions gates before M6
source changes. M6 and M7 gates also passed. M9 requires fresh exact-final-head
checks and identical verified report publications on PR #85 and Issue #84.
Independent expected observations are separate from analytic relabeling and
finite fixture enumeration. Stage N actual native production, installation,
identity rejection and optimized/unoptimized execution evidence remains deferred.
Sarah's review and Greg's acceptance remain pending.
