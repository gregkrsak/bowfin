# Bounded ordinary MIR execution (BED-026 Draft 3)

`cargo run --locked --bin bowfin -- run examples/ordinary_run.bow --function demo`
prints `Pair { first: Percent(89), second: Percent(90) }`. Initializers run in
written order; rendering uses canonical declaration order. `loop_demo` returns
`Percent(0)`; `trap_demo` stops at the inner subtraction with computed `-101`.

This command executes the complete accepted ordinary subset after the existing
parse, resolve, type, and ownership passes. It never calls proof evaluation,
`verify_report`, the affine kernel, specialized functions, or harness calls.
Frontend errors anywhere still block execution. `check` and `verify` retain their
existing behavior and output. Running an ordinary function proves no contract.

## Public authority and runtime

`execution::prepare_execution(&str, AdmissionLimits)` and
`prepare_execution_from_reader(Read, AdmissionLimits)` are the only artifact
constructors. Caller-owned `&str` is checked before source-dependent work; the
reader accumulates at most the admitted source size and reads one lookahead byte.
`CheckedExecutable` has no public fields or mutators. It retains an immutable
successful-ownership witness, canonical typed HIR, the same compilation's MIR,
and origins. Private preparation identities and exact deterministic correspondence
reject component replacement, including numerically aligned foreign IDs.
`compile_source`, public HIR inspection, and `mir::lower` do not grant authority.
`execute(&CheckedExecutable, FunctionId, ExecutionLimits)` validates the artifact
and zero-parameter ordinary entry. MIR functions are found by stored FunctionId,
including files with verified declarations in the same ID namespace.

An explicit continuation stack executes MIR statements and terminators. HIR
statement trees are used only in admission/lowering. Frame IDs increase with
checked arithmetic; references contain a frame ID and canonical local/parameter
place, permission, and referent type. Slots are uninitialized, available, or
vacant. Shared handles copy; exclusive handles and records transfer. Reads of
fields, dereferences, borrows, and assignment targets do not consume their roots.
Each loop `Let` initializes afresh and publishes only on completed evaluation.
No runtime loans, borrow checking, snapshots, or transactions execute.
BED-027 adds private synthetic cleanup described below; public source data remains
resource-free.
A callee's reference slot and caller referent are distinct. Retired IDs are never
reused; references cannot be public return values.

Arguments and record fields evaluate written left to right; unary operands run
first; binary operands run left then right. `&&`/`||` skip the entire unused RHS.
Assignment completes the RHS before publication; reference targets resolve
before the RHS. Selected-field commit preserves lawful RHS mutation to other
fields. Successful prefix effects survive traps and configured stops. A pending
record or assignment is never partly published. Every run starts with fresh state.

Every named-range subtraction widens both signed i64 operands to i128 and checks
the mathematical difference against the canonical inclusive range before forming
a value. Invalid intermediates trap immediately, including under comparisons or
later arithmetic. Raw integers remain distinct from formed nominal values.

## Version-one limits

Every override is positive and at most its supported maximum. Defaults and
accounting are versioned tool contracts, not source semantics. Configuration
arithmetic and counters use checked operations. CLI options are listed below.

| Limit / CLI override | Default | Supported maximum |
| --- | ---: | ---: |
| source bytes / `--max-source-bytes` | 65,536 | 1,048,576 |
| tokens including EOF / `--max-tokens` | 1,024 | 4,096 |
| identifier bytes / `--max-identifier-bytes` | 128 | 256 |
| frontend work / `--max-frontend-work` | 100,000,000,000,000 | 1,000,000,000,000,000,000 |
| traversal depth / `--max-traversal-depth` | 24 | 32 |
| alias-lineage depth / `--max-alias-depth` | 48 | 64 |
| alias-lineage work / `--max-alias-work` | 1,000,000 | 10,000,000 |
| admission storage / `--max-admission-storage` | 134,217,728 | 1,073,741,824 |
| diagnostic storage / `--max-diagnostic-storage` | 1,048,576 | 4,194,304 |
| output bytes / `--max-output-bytes` | 65,536 | 1,048,576 |
| execution steps / `--max-steps` | 1,000,000 | 100,000,000 |
| call depth / `--max-call-depth` | 64 | 256 |
| expression depth / `--max-expression-depth` | 64 | 128 |
| execution storage / `--max-working-storage` | 16,777,216 | 268,435,456 |
| trace entries / `--max-trace-depth` | 16 | 64 |

Admission reserves conservative envelopes **before** unchanged semantic passes,
not after compilation. With actual token count n, source size s, configured
traversal depth d and identifier cap i, frontend work reserves `128*n^4+s`,
working storage reserves `(16*d+32)*n*(i+512)+8*s`, and diagnostic storage reserves
`4*n*(i+128)+512`. Lexer storage is separately reserved as `128*s` before lexing.
These conservative bounds cover repeated table/suffix walks, expression and
record comparisons, simultaneous bounded scopes/checker snapshots, representations,
clones, lowering, and the immutable authority witness. Reservations can stop a
small input despite unused actual capacity; a supported maximum is a ceiling,
not a promise every combination of maxima fits. They bound successful and rejected
paths, including diagnostic construction and disposal. Parser call depth is
checked before recursion; expression depth is checked at construction (including
flat left-associated trees) before later visitors or recursive disposal. MIR
reachability is iterative. Canonical bounded shape checks cover ordinary bodies,
verified contracts, and retained verified-body operands before recursive record
validation, authority comparison, or correspondence validation. This structural
guard neither evaluates proofs nor executes verified bodies or harnesses. Other
retained tables and harness metadata contain scalar identities and values rather
than additional expression subtrees. Alias traversal has separate actual depth and
cumulative work counters shared across checker snapshots; analysis rollback cannot refund
these counters. Token, identifier, input, explicit runtime buffers, trace, and
rendering growth use fallible reservation. Existing bounded semantic collections
remain behind the pre-reserved envelope.

Runtime storage accounts for vector capacities, frame slots, record payloads,
pending arguments, and continuation buffers before growth. Detached record
payloads and receiving value-buffer or destination-slot growth are reserved
together with checked arithmetic before allocation or publication. Parameter and
frame growth are likewise reserved together while argument payloads remain
counted. Record construction and return publication reuse vacated value entries.
Error traces reserve separate bounded capacity of at most 64 entries so exhaustion
can still be reported.
Rendering is bounded before each write, including deterministic truncation. The
CLI has borrowed, fixed emergency diagnostics when even the truncation text
cannot fit or the renderer detects allocation failure; reporting those failures
does not allocate another String or retry ordinary rendering. Host/tool
classification takes precedence if a host/tool failure is detected either by the
original operation or while rendering or writing its diagnostic. Configured
rendering exhaustion remains class 1 only when no host/tool failure has already
occurred. An original host failure followed by rendering allowance exhaustion
uses a static host-identifying truncation message and retains class 2.
Host allocator overhead and fixed stack/error bookkeeping are outside byte-buffer
accounting. Arbitrary allocator abort, external corruption, process kill, and
hardware failure are outside structured reporting; detected reader, allocation,
and writer failures are classified as host/tool failures, not limit exhaustion.

Fuel charges one unit before each statement, terminator (including synthetic
Goto), evaluated expression node, and function entry. Continuation actions do not
add fuel. A skipped RHS has no node, call, mutation, or fuel charge. Boundary
precedence is fuel, then call/expression depth, then storage, then detected host
allocation failure. Wall-clock timeout is not semantic fuel. Limit exhaustion is
not a proof of nontermination. Invalid/zero/overflowing overrides reject before
source ingestion.

## Origins and diagnostics

Each block terminator has an origin produced by lowering. Explicit returns,
breaks, and continues use their statement span. Conditional initialization uses
the enclosing initializer statement; structural branches and synthetic preheaders,
backedges and joins use their enclosing if/while. Implicit completion uses the
function boundary. Statements retain source spans; traps additionally retain the
failing expression. Nested frames retain canonical callees and call-site spans.
Origin correspondence is checked before execution, with missing, malformed and
foreign origins rejected. Traces have deterministic bounded truncation.

Results are `Bool(...)`, named-range `Name(value)`, ordinary records with nominal
field values, or `NoValue`. Errors distinguish frontend/admission/integrity,
named-range traps, configured limits, and detected host/tool failures. They are
tool outcomes, not source-level exceptions, catchable results, effects, or I/O.
Exit status is 0 on success, 1 on frontend/integrity/trap/limit stops, and 2 on
usage/configuration or detected host/tool failures. No host addresses or Rust
runtime diagnostics are emitted by the structured error renderer.

## BED-027 private cleanup extension

The frozen BD-042 extension separates canonical resource identity, owned/reference
identity and Copy/Move classification. Trusted acquisition/release descriptors and
shared acyclic cleanup plans are private. Source record fields still require named
ranges, and the CLI/public data boundary never admits a live resource result.

Resource functions carry explicit lexical MIR Flow and synthesized origins.
Activations retain successful owner-initialization history, exact inner-to-outer
exits and reverse owned-parameter order. Runtime payloads own responsibility;
explicit transfer markers justify skipping former owners, while mere vacancy
does not. A protected return is retained outside callee cleanup, and delivered only
after cleanup and frame retirement complete. Borrowed handles and inert Move
records have no cleanup plan. Unused private catalogs preserve resource-free fuel
and storage behavior.

Pending aggregates evaluate fields in written order and publish only after exact
canonical membership is complete. Complete cleanup and synthetic partial abort
visit reverse canonical declaration order, with abort filtered by initialized
membership. Independent nested pending containers abort inside-out. Moving into a
pending field never restores the old source on failure. This private synthetic
failure facility demonstrates BD-042 and does not add source recoverable propagation
or general unwinding. Unnamed resource arguments without a defined lifetime reject.

Type classification shares child plans and caps its acyclic depth at 32 (also
subject to configured admission traversal depth), record tables at 4,096 and field
tables at 8,192. Execution admission retains the stricter configured token-based
table/node bounds and identifier cap and reserves its expanded frontend envelope.
Cleanup State, activation/history buffers, events, pending fields, protected/current/
detached payloads and work-list capacities participate in checked prospective
working-storage accounting. State is reserved before lazy activation. Detachment
and receiving-container growth are counted together. Resource publication first
retains the owner in the reserved detached slot; receiving-buffer or slot
reservation failure leaves that payload retained without restoring its source.
Inert nested-record construction uses canonical field structure independently of
whether unrelated resources have activated runtime state. Reinitialization tests
canonical cleanup responsibility rather than the aggregate representation tag.
Cleanup-plan reservation errors remain typed as Host, Integrity, or configured
Limit through bounded shape validation, lowering, sealing, and revalidation.
Later validation never assumes that an earlier successful allocation makes the
next plan-construction attempt infallible. Error traces and reporting
retain the existing separate bounded diagnostic contract.

In addition to BD-041 charges, fuel is charged before scope entry/exit and owner
inspection, successful initialization recording, pending field progress, aggregate
traversal and fixed trusted operations. Resource-free continuations retain their
previous costs. Release attempts reserve event storage before any effect and record
completion exactly once. Configured exhaustion or controlled host release failure
retains prior effects and all uncompleted payloads, cannot retry, and cannot publish
ordinary success. Disposal of host interpreter state is not a release effect.

The 86 new BD-042 tests (76 original and 10 focused repair regressions) cover canonical/malformed authority, structured exits,
transfers/returns, aggregates/pending abort, exact fuel/storage thresholds, terminal
failure/reporting and unique-leaf conservation. GitHub debug and release profiles
also reconcile the 1,581 predecessor identities and each run the 1,800-case affine
cross-check. See `docs/engineering/BED-027-IMPLEMENTATION-LEDGER.md` for executed
evidence; no local executable conformance is claimed during K27-I01.

## BED-028 replacement protocol and accounting

Eligible direct mutable Move locals execute an explicit compound ReplaceOwned MIR
operation. Only the complete RHS result gains a new resource boundary; call arguments
still receive their existing independent temporary checks. The target remains
available during RHS execution, including lawful Copy-field mutations through an
exclusive borrow. Traps preserve those prefix effects without displaced cleanup
or replacement publication.

Prepared context storage is reserved before RHS work. The complete result transfers
infallibly from an existing accounted holder into separate replacement protection.
History capacity is reserved before detachment. The old owner then enters an explicit
displaced state and existing canonical cleanup cursors. Old-generation discharge
requires completed cleanup; an attempt or vacant slot is insufficient. Only after
successful discharge do slot installation, current-generation association and reserved
history append occur together without allocation or a failure boundary between them.
Registration uses the target's declaring activation, including replacement inside
an inner branch. Protected returns and pending construction remain separate holders.

In addition to ordinary statement/expression/call costs, replacement charges one
unit at prepared entry, one before result protection, and one at each protected,
prepared, detached and discharged phase boundary. Existing cleanup work charges
remain; each skipped discharged history entry charges one unit at scope exit.
Copy-assignment accounting is unchanged. Generation histories and slot associations,
context stacks, protected/displaced payload backing storage, existing cleanup cursors,
value/task capacity and events all count toward checked prospective storage. Checked
monotonic identities prevent generation or activation reuse. Repeated replacement
history is retained until activation exit; it is bounded by storage and charged work.

Configured stops and detected Host failures retain all unfinished responsibilities
and completed release prefixes, refuse resume and never fabricate rollback or
successful continuation. Publication/registration is indivisible; every actual
fallible cut is tested through fuel/storage exhaustion and private Host probes.
The probes are test facilities, not production callbacks or recoverable source errors.
No cleanup guarantee is added for BD-042's excluded abrupt outcomes.

The exact 1,667-test predecessor inventory, four designated identity migrations,
one separately justified iteration-local outcome correction and every new identity
are reconciled in each complete debug/release profile. Both profile executions of
the 1,800-case affine check and the authority compile-fail doctest remain required.
See [the BED-028 ledger](engineering/BED-028-IMPLEMENTATION-LEDGER.md).

## BED-029 temporary-region accounting

Temporary execution charges before activation entry, successful registration,
completion/protection, ownership transfer, retirement traversal, owner inspection
and final retirement. The private single-owner cleanup fixture requires exactly
18 units, versus its predecessor's 11; the seven extra progress charges are frozen
BED-029 section 14 work. Empty resource-free regions are omitted and preserve their
previous fuel/storage behavior. Existing expression, call and cleanup charges remain.

Temporary activation, owner and event capacities, protected payloads, call preparation,
partial parameter publication and existing cleanup cursors participate in prospective
storage checks. Non-owning operand handles duplicate no payload. Combined growth is
reserved before detachment where feasible; every actual fallible handoff retains an
explicit holder. Static table lengths and parent-chain traversal are bounded before
reference inspection. Monotonic activation identities cannot silently wrap or reuse.

Configured fuel/storage stops, detected allocation failures and trusted-release
failures retain unfinished responsibilities and completed prefixes, refuse resume
and add no unwind or rollback. Private probes inject failures immediately before
actual fallible reserves and canonical release operations; they are test facilities,
not public callbacks. Exact sufficient/insufficient neighbors, every configured call
cut, U/R unpublished-local cuts, partial pending fields, return/replacement protection
and identity-sensitive holder censuses are recorded in the [BED-029 ledger](engineering/BED-029-IMPLEMENTATION-LEDGER.md).

Each complete debug/release profile reconciles all 1,706 predecessor identities and
37 additions, with two preserved-ID admission corrections and one justified fuel-cost
correction. There are no identity renames. Both 1,800-case affine checks and the
compile-fail authority doctest remain required. GitHub Actions is authoritative;
local Rust validation was unavailable because of confirmed Work-runtime corruption.

## Frozen BED-030 effect authority and bounds

Explicit ordinary declarations are bounded eight-identity upper bounds. Complete
callee declarations and mechanically derived cleanup summaries are checked during
preparation. Fixed descriptor/function/release/leaf/aggregate copies must agree.
Successful ownership analysis produces custody/realization witnesses; exact MIR
and origin correspondence plus an immutable original witness seal approval.
Missing, foreign, duplicated, reordered or cross-compilation authority rejects.
Execution consumes validated metadata without rerunning ownership/effect analysis.

Raw lists, immutable origins, direct sites, derived record summaries and retained
custody witnesses fit bounded frontend storage/work. Nonempty cleanup reserves an
additional holder-site/snapshot envelope; its work ceiling is a checked 640*n^4
plus source bytes. Empty-summary predecessors retain their existing envelope.
Recursive type plans and summaries have canonical depth/cycle checks; diagnostic
release ancestry has a fixed 16-visit budget with honest truncation. Diagnostic
storage and full/fallback rendering have exact sufficient/insufficient controls.
Detected direct-site, summary, witness or private-metadata allocation failures
remain typed Host failures and publish no executable.

Only Ordinary is available in production. Private Approved Abort fixtures admit
the complete possible initialized-member cleanup in every affected pending frame
before approval. Their bounded templates bind canonical membership/plan, containing
function, construction/in-progress sites and origins, exact monotonic frame and
pending identities, and stack depth. Runtime validates the entire stack before
the first attempted release. Primitive state-injection fixtures are separately
sealed protocol evidence. The manifest explicitly lists 129 predecessor identities
in that evidence domain without changing their assertions or outcomes.

Ordinary traps, configured limits, Integrity and Host stops add no cleanup edge or
unwind. In the fixed release protocol, Attempt survives Failure and is distinct
from Complete; later never-attempted owners remain held. Acquire denotes successful
payload acquisition, not failed preparatory tool allocation. Transfer, replacement
generations, pending fields, incremental parameter entry and protected results
retain their original payload custody/order. No declaration-driven events or new
source fuel charges are introduced. `unsafe` bypasses no rule; interpreter memory
and diagnostic output do not become source `heap` or `io`.

The [BED-030 ledger](engineering/BED-030-IMPLEMENTATION-LEDGER.md) records complete
debug/release identity reconciliation, all corrections and authoritative GitHub
workflows. Specification freeze is separate from independent implementation review
and acceptance. No local Rust executable evidence is claimed.

## Frozen BED-031 typed outcomes

`cargo run --locked --bin bowfin -- run examples/result_handling.bow --function demo`
prints `Error(ErrorCode(1))`; `success_demo` prints `Reading(42)`. Both Error and
Err are accepted source aliases. Public Result output uses a flat active chain
and one Bool/range/ordinary-record leaf, avoiding recursively owned output trees.
Depth, canonical nominal identities, selected types and output allowance are checked.
Private resource-bearing Result entries reject before execution/acquisition.

Type syntax and canonical tables have finite postorder identities and maximum
nesting 32, additionally bounded by configured admission depth/token limits.
Result programs reserve the existing frontend envelope plus `512*n^2 + 128*d*h`
bytes for outcome facts/snapshots and immutable witness copies, and a checked
`256*n^4` work ceiling; n is the larger existing node envelope or canonical holder
count, d is configured traversal depth, and h is the holder/type/function count.
When nonempty cleanup evidence is also present, both storage reservations combine
and the work ceiling is `896*n^4` plus source bytes.
The retained event count cannot exceed the reserved quadratic bound. Non-Result
programs retain their predecessor envelope. Variant tables and exact correspondence
include ordered payload roles, nested identities, effects, arms, bindings and origins.

Lazy cell, match-protection and inert record-staging capacities are prospectively
charged, including physical payload storage once. Cells retain actual payloads
through fallible publication, call entry, protected return and selected binding
reservation; handles and history never count as second payloads. Nested payload
links remain flat. Each Result cell binds the approved constructor's tag/type/origin,
including equal-typed alternatives. Fuel charges constructor evaluation/publication,
dispatch, active transfer and arm entry, plus existing region/call/cleanup work.
No unselected body or inactive payload executes. Consumed cells remain bounded
accounted metadata; long-running loops can exhaust storage without an unwind.

Outcome obligations are compile-time evidence from the one ownership checker.
Runtime enforces approved custody transitions rather than reconstructing ownership
or match coverage. All structured errors stop ordinary resume. No trap, limit,
Host, integrity or cleanup failure becomes Error; protected failure returns cannot
reach callers after failed cleanup. Whole-wrapper traversal is separately sealed
primitive/protocol evidence. See the BED-031 ledger and explicit evidence lanes.

### B31-MR01 Result admission classification repair

Result syntax-node storage, resolution scratch and canonical-table reservations
use fallible reservation and retain typed Host failures through bounded public
preparation. Configured Result syntax depth/node and canonical count/nesting
exhaustion retain typed Limit failures. Neither category publishes an executable
or becomes a source Error. The CLI's existing Host reporting path returns class 2.
Malformed labels, unknown or excluded payload types and nominal mismatches remain
frontend diagnostics; no syntax, ownership or runtime representation is changed.

Private test-build injection selects only actual growth at each of the three
reservation sites, including failure after earlier successful admission work.
It is absent from production and cannot be selected by source or CLI options.
The canonical table admits exactly 4,096 distinct ordered pairs and nesting 32;
its next distinct pair or nesting 33 is a configured Limit. Existing source
syntax traversal also counts its owned leaf: at configured depth 32 it admits
31 nested Result wrappers and rejects 32. The separate syntax-node control
admits 4,095 nodes and rejects the next well-formed binary-tree size, 4,097,
against the 4,096-node ceiling. These private controls isolate individual limits
from earlier token/syntax ceilings; they do not grant execution authority.

## BED-032 propagation protocol and bounds

The local declaration's approved mode retains exact inner/outer/F identities and
phrase/binding origins. Its success edge establishes the ordinary local in the
actual scope; Error rewraps into the exact outer Result and takes ordinary protected
return exits. Source Error does not catch tool stops. Diagnostic text classifies
no new failure; typed MIR validation preserves Host/Limit/Integrity through sealing
and revalidation. Existing CLI Host class 2 is retained.

MIR retains two extra blocks per propagation site and stores the suffix once.
With p sites in a straight-line body the function has 2*p+1 blocks. Actual lexical
ScopeIds remain separate from CFG blocks. Shared Result/record plans, bounded
outcome witnesses, origins, cleanup exits and independent approval copies fit
the checked conservative frontend/outcome envelopes. New fields have fixed size,
new block/site counts remain within existing token-derived caps, and dynamic
continuations/cells/history remain capacity-accounted. No source limit is raised.

The same Result cell footprint retains constructor-site versus generated-return
provenance in one bounded tagged scalar. Source sites/blocks must fit their bounded
namespace before encoding. On Error, cell and receiving-value capacities are
prospectively reserved together while the input still owns F; transfer, parent-link
update, canonical output publication and input consumption then allocate nothing.
The output is protected by the existing ordinary return machinery before cleanup.

Fuel charges the propagating terminator, evaluated initializer, dispatch, selected
binding statement/transfer and continuation entry on Ok. Error charges its return
terminator, outgoing reservation/publication progress and active transfer, then
existing actual cleanup/return work. Temporary activation, retirement, calls and
resource initialization retain their existing charges. An unselected alternative
executes and charges no payload/body/release work; its immutable metadata remains
bounded. Consumed cell/history capacity remains charged during repeated execution.

The ledger records exact x86_64 Rust 1.80.0 fuel/storage thresholds for all four
payload classes on both alternatives. Tests check each fuel cut, actual allocation
growth failure and every byte below the successful peak in representative resource
and nested cases. Selected failures preserve one physical payload owner and
completed prefixes, publish no continuation and refuse resume. Separate primitive
fixtures test sibling retirement before dispatch without claiming ordinary-source
reachability. See the BED-032 ledger and identity manifest for evidence domains.
