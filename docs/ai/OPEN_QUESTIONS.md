# Bowfin Open Questions

These questions are intentionally unresolved.

When one is settled, add/update a decision in `LANGUAGE_DECISIONS.md`.

## OQ-001 — Exact effect syntax

Frozen BED-030 Draft 2 settles the first ordinary-function phase: a contextual
`effects` clause declares an explicit upper bound, absence means empty, and calls
propagate complete declared contracts. Derived cleanup participates at ownership-
approved realization sites. See BD-008 and Issue #70; implementation review and
acceptance remain separate from specification freeze.

Broader effect polymorphism, groups/aliases, generic interaction, project policy,
effects-aware verified/harness behavior and richer diagnostic presentation remain
open or deferred. No attribute policy syntax is admitted by this first phase.

## OQ-002 — Heap-managed ownership vocabulary

Earlier exploration considered `heap T`, `shared T`, and `pinned T`. Names are not locked.

## OQ-003 — Trait/generic syntax

Static vs dynamic dispatch distinction is desired. Exact declarations, associated types, bounds, specialization, and coherence are open.

## OQ-004 — Package/module/interface syntax

Ada-style interface/implementation separation is attractive. Compilation-unit and source-layout details remain open.

## OQ-005 — Manifest format

`bowfin.toml` is the reserved current tooling preference; `klyxr.toml` is the legacy reservation. No configuration loader exists, and manifest behavior remains undecided.

## OQ-006 — Panic and exception model

`Result<T,E>` is preferred for recoverable errors. Exact panic/unwind/abort/no-panic behavior remains open.

## OQ-007 — Arithmetic operator spellings

Checked/wrapping/result/saturating semantics are directionally accepted. Exact operator spellings remain reviewable.

## OQ-008 — Verification backend

Z3 is a strong initial candidate. Solver abstraction, proof certificates, reproducibility, and trusted-base policy remain open.

## OQ-009 — Explicit region/lifetime syntax

Inference should handle most code. Exact explicit syntax remains open.

## OQ-010 — Concurrency model

Ownership-safe concurrency plus Ada-inspired tasking is the direction. Exact tasking, channels, structured concurrency, scheduler metadata, and shared-state primitives remain open.

## OQ-011 — HardRealtime profile

Need a precise list of forbidden effects, allocation rules, recursion rules, blocking rules, boundedness requirements, ISR rules, and stack-analysis policy.

## OQ-012 — FFI and representation

Need exact rules for layout, ABI declarations, bitfields, volatile, memory-mapped I/O, C interoperability, and trust at foreign boundaries.

## OQ-013 — Standard library scope

Need staged core/no-runtime, alloc-capable, hosted, networking/filesystem, and realtime-safe layers.

## OQ-014 — Keyword alias expansion

BD-014 accepts six initial pairs. Do not add aliases casually. Whether more pairs are justified remains open.

## OQ-015 — Units syntax

Units are part of the direction. Exact declarations, derived dimensions, literals, conversions, simplification, and diagnostics remain open.

## OQ-016 — ABI and library evolution

Need policy for ABI stability, semantic versioning, contract changes, effect-set compatibility, and verified API evolution.

## OQ-017 — Self-hosting milestones

First implementation likely uses Rust. Criteria and trust story for self-hosting remain open.

## OQ-018 — Syntax is not frozen by repetition

Many current examples are illustrative. Always distinguish accepted semantic concepts from provisional surface spelling.

## OQ-019 — Mixed assurance call boundaries

Need exact rules for calls among `safe`, `checked`, and `verified` code: who
establishes preconditions, which postconditions may be assumed, when runtime
checks are required or may be removed, and how unproved callees enter the trust
report. BD-003, BD-006, and BD-013 settle the direction, not these details. The
prototype's `check` and `verify` commands share a restricted proof pass; they do
not implement different assurance levels.

## OQ-020 — Mutation framing, invariant boundaries, and snapshots

Need exact rules for the state a function may change, when record invariants must
hold, and what `old(...)` snapshots for nested records, aliases, heap values, and
exceptional exits. The current prototype models only one mutable field and its
entry integer value; that scope does not settle the general rules.

## OQ-021 — General constrained arithmetic expression semantics

The initial expression layer supports a deliberately narrow constrained-range arithmetic model.

Still open:

- broader numeric base types;
- default numeric literal typing;
- generalized literal coercion/context rules;
- arithmetic result typing beyond the initial range-preserving model;
- exact runtime behavior for constrained arithmetic outside verified code;
- interaction with checked/wrapping/result/saturating arithmetic families;
- conversions between constrained types;
- units/dimension-aware arithmetic.

BED-003 through BED-009 do not settle these questions implicitly.


## OQ-022 — General function and local-value semantics

BED-004 established a straight-line value-function core with Bool/range values.
BED-005 extends it with by-value records and the basic binding/call/return move
rules accepted in BD-020. Those core move rules are no longer wholly unresolved;
this does not settle the broader function/local semantics below.
BED-006 adds references and `let mut` / `let mutable` for owned locals, authorizing
exclusive borrowing only. It introduces no assignment or mutable owned parameters.
The core straight-line, non-escaping loan rules now fall under BD-021; advanced
reference semantics remain under OQ-024. BED-007 adds explicit dereference and
Copy-safe write-through as a distinct accepted operation (BD-022). `let mut`
did not enable direct owned-local reassignment under BED-007. BED-011 now settles
only direct reassignment of mutable Bool/named-range Copy locals (BD-026).

BED-008 adds ordinary statement `if` / optional `else`, with nested lexical
branch scopes. BED-018 / BD-033 now settles structured early returns and
no structural closing-brace fallthrough; loop false exits remain possible. BED-010 settles owned/Copy
conditional values only in complete local initializer position, with exact branch
types and mandatory else (BD-025). Statement-containing value blocks and general
conditional-expression positions remain unresolved.

Still open:

- whether unqualified `fn` is the final surface spelling of the base `safe` assurance level;
- explicit `safe fn` syntax, if any;
- first-class unit/void values and types (BD-034 settles NoValue function results);
- conditional values in returns, call arguments, and general expression positions;
- statement-containing value blocks and broader owned branch-result semantics;
- non-Copy replacement, reference-local reassignment, and broader mutable-local semantics;
- compound assignment, projected places, and conditional values on assignment RHSs;
- mutable owned parameters;
- reference returns and escaping-reference interfaces;
- expression-bodied function shorthand;
- literal typing at unannotated bindings and broader contextual propagation (BD-037 settles direct literals at canonical value boundaries);
- broader value categories and record/reference interfaces beyond core owned records and non-escaping references;
- function values, closures, and higher-order calls;
- function overloading;
- interaction of calls and owned places with advanced borrowing and control flow;
- runtime enforcement of constrained return values outside verified code.

OQ-019 remains authoritative for mixed `safe` / `checked` / `verified` call
boundaries. OQ-021 remains authoritative for unresolved constrained arithmetic
and literal-conversion rules. Copy customization and destruction remain under OQ-023.


BED-012 permits nested statement while for ownership-stable Copy state only.
BED-012 originally left loop-local and early/multiple returns unresolved;
BD-033 now settles structured early return. Loop values and broader loop control
remain unresolved.

BED-013 now permits iteration-local Move values in while bodies (BD-028),
including fresh record results, transfers and existing branch joins. Conditions
remain Copy-only; scopes are unchanged; BD-033 later permits structurally complete early returns. General
loop-carried ownership and cyclic reference use remain unresolved. BED-014 permits
iteration-local body borrowing (BD-029), with reference-free while conditions and
historically no use of handles present at the current header; BD-030 refines this
with stable carried access.


BED-015 settles stable loop-carried reference access only (BD-030): scoped
recurrent obligations retain exact original provenance across every accepted
backedge, and false-exit liveness discharges only the current frame. Outer frames
and post-loop uses remain authoritative. Mutable carried handles cannot transfer;
while conditions stay reference-free. Generalized cyclic lifetime inference,
fixed points, MIR ownership, provenance replacement/reborrowing, reference returns,
escaping references and broader control flow remain unresolved.


BED-016 settles unlabeled statement continue to the innermost while header
(BD-031). Each explicit backedge preserves exact carried state and all recurrent
frames before cleanup validation completes. Continue removes only the skipped
same-iteration finite suffix; post-loop finite uses remain required. The real
false exit exists even when every body path continues and discharges only its own
frame. Structured no-fallthrough suffix rejection is narrow, not a general
unreachable-code policy. BD-033 settles structured early returns. Labels/loop values, generalized
cyclic ownership and lifetime inference, fixed points, MIR ownership, reborrowing,
destruction and execution remain unresolved.

BED-017 settles unlabeled statement break to the innermost while continuation
(BD-032). Each independently validated break candidate normalizes to the saved
condition-false canonical exit. Pre-reduction integrity checks protect carried
provenance, exact frame nesting and balanced holds before cleanup/expiry can erase
evidence. Exactly the target frame ends; enclosing obligations and outside finite
uses remain authoritative. Skipped iteration suffixes do not weaken recurrence
before break. Mixed break/continue arms have zero fallthrough, without arbitrary
exit merging or terminal-path relaxation of pre-existing Move restrictions.
Labeled transfers, break/loop values, generalized cyclic ownership/lifetime fixed
points, MIR ownership, unreachable-code policy, reborrowing, destruction, execution
and termination proof remain unresolved.

## OQ-023 — Copy customization, cloning, partial moves, and destruction

BED-005 establishes the core owned `Copy` / move distinction. BED-006 classifies
shared references as Copy and mutable references as Move; neither directive
settles user customization. BED-007 blocks non-Copy write replacement specifically
because displaced-value semantics remain unresolved. BD-042 now settles a bounded
resource-only deterministic cleanup model, not general user-programmable Drop.

Still open:

- eventual user-facing `Copy` trait or equivalent;
- eligibility rules for user-defined `Copy`;
- explicit cloning/duplication APIs;
- automatic or declared Copy behavior for records, enums, tuples, and arrays;
- partial moves from aggregate fields;
- ownership of destructured values;
- whether any constrained user-declared resource-leaf mechanism is eventually
  safe and useful beyond compiler-known trusted leaves;
- destruction of non-resource Move values if future representations require work;
- interaction between cleanup and non-Copy replacement/overwrite;
- destruction points for resource-bearing expression temporaries;
- interaction between destruction and panic/unwind;
- resource-owning standard-library types;
- ownership representation across FFI boundaries.

Core borrowing is accepted in BD-021. Advanced borrowing and lifetime rules
remain separate questions under BD-004, OQ-009, and OQ-024. No runtime destruction or resource release is implemented
by the core ownership checker.

BED-013 scopes away iteration-local Move availability at the backedge without
introducing destruction or resource-release semantics. BD-042 later requires
runtime cleanup for resource-bearing owners on covered scope exits while leaving
Copy customization, partial moves, arbitrary Drop, temporaries, and cleanup/unwind
open.

## OQ-024 — Advanced borrowing, reborrowing, dereference, and escaping references

BED-006 settles only whole-value, straight-line, non-escaping core borrowing.
BED-007 settles basic explicit named dereference and Copy-safe whole-value
write-through under BD-022. Advanced borrowing and dereference coercions remain
open; general mutation/replacement is tracked separately under OQ-025.

BED-008 introduced acyclic statement branches. BED-009 settles continuation-aware,
path-sensitive whole-value loan expiry across that current conditional subset
(BD-024). It does not establish arbitrary CFG lifetime inference, general NLL
completeness, or borrowing across loops. Reborrowing, reference-valued branch
results, escaping references, and advanced lifetime relationships remain open.
BED-010 permits only owned/Copy conditional initialization; reference-valued
conditional results, reference lifetime joins, escaping branch references,
reference-valued block parameters, and reborrowing remain unresolved.

Still open:

- reference returns and escaping-reference semantics;
- explicit lifetime syntax, parameterization, and elision;
- reborrowing syntax and semantics;
- whether and when `&mut T` may coerce to `&T`;
- auto-deref and dereference coercions;
- generalized field access through references beyond BD-039/BD-040 direct named Copy-field reads/writes;
- field/partial borrowing and disjoint field loans;
- reference mutation beyond Copy-safe whole-value write-through and BD-040 named Copy-field writes;
- two-phase borrows;
- temporary lifetime extension and borrowing arbitrary temporary expressions;
- cyclic loan/lifetime analysis across loops/backedges and general CFG lifetime inference;
- reference-valued branch results;
- borrowing interaction with closures and destructors;
- interior mutability and smart-pointer borrowing;
- FFI reference/lifetime boundaries.

OQ-009 remains authoritative for the broader explicit lifetime syntax question.

BED-012 leaves reference activity through while conditions/bodies unsupported.
Untouched outside handles/loans persist unchanged and still constrain Copy owner
access; this is not cyclic lifetime inference or reference use across backedges.

BED-014 permits iteration-local borrowing in while bodies (BD-029), refining
BED-013's historical restriction. Fresh body provenance must be gone at the
backedge before allocator restoration. Untouched header handles/loans persist
unchanged and constrain new borrows. BD-030 subsequently permits stable carried
use, but all while-condition reference activity, generalized cyclic NLL and
fixed-point lifetime inference remain unsupported. Nested headers make outer-created handles pre-existing.
Reborrowing, reference returns/results/reassignment, explicit lifetimes and
field/partial loans remain unresolved.


BED-015 settles stable loop-carried reference access only (BD-030): scoped
recurrent obligations retain exact original provenance across every accepted
backedge, and false-exit liveness discharges only the current frame. Outer frames
and post-loop uses remain authoritative. Mutable carried handles cannot transfer;
while conditions stay reference-free. Generalized cyclic lifetime inference,
fixed points, MIR ownership, provenance replacement/reborrowing, reference returns,
escaping references and broader control flow remain unresolved.


BED-016 settles unlabeled statement continue to the innermost while header
(BD-031). Each explicit backedge preserves exact carried state and all recurrent
frames before cleanup validation completes. Continue removes only the skipped
same-iteration finite suffix; post-loop finite uses remain required. The real
false exit exists even when every body path continues and discharges only its own
frame. Structured no-fallthrough suffix rejection is narrow, not a general
unreachable-code policy. BD-033 settles structured early returns. Labels/loop values, generalized
cyclic ownership and lifetime inference, fixed points, MIR ownership, reborrowing,
destruction and execution remain unresolved.

BED-017 settles unlabeled statement break to the innermost while continuation
(BD-032). Each independently validated break candidate normalizes to the saved
condition-false canonical exit. Pre-reduction integrity checks protect carried
provenance, exact frame nesting and balanced holds before cleanup/expiry can erase
evidence. Exactly the target frame ends; enclosing obligations and outside finite
uses remain authoritative. Skipped iteration suffixes do not weaken recurrence
before break. Mixed break/continue arms have zero fallthrough, without arbitrary
exit merging or terminal-path relaxation of pre-existing Move restrictions.
Labeled transfers, break/loop values, generalized cyclic ownership/lifetime fixed
points, MIR ownership, unreachable-code policy, reborrowing, destruction, execution
and termination proof remain unresolved.

## OQ-025 — General mutation, place expressions, and replacement semantics

BED-007 settles only explicit dereference and Copy-safe whole-value write-through.
BED-008 permits those operations inside `if` branches. It does not settle general
mutation across loops or complex control flow, or non-Copy replacement/destruction.
BED-011 settles only direct mutable Bool/named-range local Copy reassignment
(BD-026), with exact typing, RHS-first/write-second ordering, and active-loan checks.
It introduces no generalized place evaluation-order rule.

Still open:

- non-Copy replacement and the fate/destruction of displaced values;
- reference-local reassignment and provenance/lifetime replacement;
- general place-expression architecture;
- field/projected assignment beyond BD-038/BD-040 direct Copy-field writes, partial mutation, and broader aggregate mutation;
- compound assignment;
- swap/take/replace primitives;
- assignment-expression semantics, if any;
- richer assignment evaluation-order guarantees;
- mutation through future reborrows;
- mutation across control flow;
- interaction with destructors and unwind.

OQ-023 remains authoritative for Copy customization and destruction. OQ-024
remains authoritative for advanced borrowing, reborrowing, dereference coercions,
escaping references, and field/partial borrowing.

BED-012 permits BD-026 owned Copy reassignment in nested while/if bodies only
when ownership and loan state is stable. General mutation across cyclic ownership,
non-Copy replacement and destruction remain unresolved.

BED-013 permits transfers of iteration-local Move values, not replacement.
Direct reassignment of any non-Copy owner, including body-local owners, remains
unsupported pending displacement/destruction semantics. BED-014 permits existing
Copy-safe write-through using fresh body references, not replacement or destruction.
BD-030 permits stable carried Copy-safe access/write-through; generalized cyclic mutation
and lifetime analysis remain open.


BED-015 settles stable loop-carried reference access only (BD-030): scoped
recurrent obligations retain exact original provenance across every accepted
backedge, and false-exit liveness discharges only the current frame. Outer frames
and post-loop uses remain authoritative. Mutable carried handles cannot transfer;
while conditions stay reference-free. Generalized cyclic lifetime inference,
fixed points, MIR ownership, provenance replacement/reborrowing, reference returns,
escaping references and broader control flow remain unresolved.


BED-016 settles unlabeled statement continue to the innermost while header
(BD-031). Each explicit backedge preserves exact carried state and all recurrent
frames before cleanup validation completes. Continue removes only the skipped
same-iteration finite suffix; post-loop finite uses remain required. The real
false exit exists even when every body path continues and discharges only its own
frame. Structured no-fallthrough suffix rejection is narrow, not a general
unreachable-code policy. BD-033 settles structured early returns. Labels/loop values, generalized
cyclic ownership and lifetime inference, fixed points, MIR ownership, reborrowing,
destruction and execution remain unresolved.

BED-017 settles unlabeled statement break to the innermost while continuation
(BD-032). Each independently validated break candidate normalizes to the saved
condition-false canonical exit. Pre-reduction integrity checks protect carried
provenance, exact frame nesting and balanced holds before cleanup/expiry can erase
evidence. Exactly the target frame ends; enclosing obligations and outside finite
uses remain authoritative. Skipped iteration suffixes do not weaken recurrence
before break. Mixed break/continue arms have zero fallthrough, without arbitrary
exit merging or terminal-path relaxation of pre-existing Move restrictions.
Labeled transfers, break/loop values, generalized cyclic ownership/lifetime fixed
points, MIR ownership, unreachable-code policy, reborrowing, destruction, execution
and termination proof remain unresolved.

## OQ-026 — General control flow, conditional values, and MIR dataflow

BED-008 establishes only acyclic statement conditionals and the first ordinary MIR CFG.

BED-009 improves loan precision across acyclic statement branches. Ownership
remains in the semantic typed-HIR → MIR stage, using one existing ownership
checker. Full MIR-based ownership dataflow and loop/fixed-point analysis remain
unresolved, as do general conditional expressions, phi/block parameters,
definite initialization, and unreachable-path analysis. BD-033 settles structured
early returns without generalizing ownership dataflow.

BED-010 settles owned/Copy value production across acyclic joins only in local
initializer position. Its first MIR strategy initializes the existing source
LocalId independently on every mutually exclusive leaf path. This is not the
final general MIR solution for all future value joins; no phi/block parameters,
SSA, general temporary values, or definite-assignment solver are introduced.

Still open:

- general conditional expressions and branch-result type unification beyond exact types;
- join values without a source local destination, arbitrary temporaries, and SSA;
- block parameters / phi-like representations;
- generalized cyclic ownership/loans and fixed-point analysis, `for`, loop values and labels (BD-031/BD-032 settle unlabeled `continue`/`break`);
- divergence / bottom types and `match` lowering (BD-033 settles structured early returns);
- definite initialization and uninitialized locals;
- full MIR-based ownership dataflow and maximally precise path-sensitive loan analysis;
- short-circuit Boolean lowering / evaluation guarantees and unreachable-code policy;
- generalized cleanup representation beyond BD-042's covered structured exits,
  resource-bearing expression temporaries, and exceptional / unwind control flow;
- eventual MIR expression-lowering granularity.

BED-011 adds a distinct acyclic MIR Assign statement for already initialized Copy
locals. This prepares ordinary loop-carried Copy state for a separately authorized
future directive; it adds no loops, backedges, fixed points, SSA, or definite assignment.

BED-012 settles pre-test statement while and cyclic MIR for ownership-stable
owned Copy state (BD-027), including nested loops and if/while composition.
MIR is no longer globally acyclic; structural validity is not a termination proof.
The checker remains above MIR and compares header/backedge state for one permitted
iteration, without generalized cyclic dataflow. BED-013 now permits iteration-local
Move activity, but loop-carried non-Copy changes,
cyclic reference liveness, verified loops/invariants/variants and termination
checking remain open, alongside the questions above.

BED-008/BED-010/BED-011/BED-012 MIR choices do not settle or preclude these broader features.


BED-013 settles only iteration-local Move activity with per-loop header identity
(BD-028). Outer-iteration locals are loop-carried relative to an inner header;
pre-existing owners cannot change availability across any backedge. The existing
HIR-stage checker scopes away body-local state and checks semantic equality for
one iteration. It does not add general cyclic ownership convergence, fixed-point
dataflow, reference inference, MIR ownership, or a new IR representation.


BED-014 adds only iteration-local borrowing to that one-iteration HIR model
(BD-029). Finite continuation-aware liveness applies inside each permitted body;
new handles/loans must be discharged before the backedge and allocator restoration.
BD-030 subsequently settles stable carried access. It does not settle generalized cyclic NLL,
fixed-point ownership/loan dataflow, MIR ownership, reference-result joins,
termination, verified loops or any broader control-flow question above.


BED-015 settles stable loop-carried reference access only (BD-030): scoped
recurrent obligations retain exact original provenance across every accepted
backedge, and false-exit liveness discharges only the current frame. Outer frames
and post-loop uses remain authoritative. Mutable carried handles cannot transfer;
while conditions stay reference-free. Generalized cyclic lifetime inference,
fixed points, MIR ownership, provenance replacement/reborrowing, reference returns,
escaping references and broader control flow remain unresolved.


BED-016 settles unlabeled statement continue to the innermost while header
(BD-031). Each explicit backedge preserves exact carried state and all recurrent
frames before cleanup validation completes. Continue removes only the skipped
same-iteration finite suffix; post-loop finite uses remain required. The real
false exit exists even when every body path continues and discharges only its own
frame. Structured no-fallthrough suffix rejection is narrow, not a general
unreachable-code policy. BD-033 settles structured early returns. Labels/loop values, generalized
cyclic ownership and lifetime inference, fixed points, MIR ownership, reborrowing,
destruction and execution remain unresolved.

BED-017 settles unlabeled statement break to the innermost while continuation
(BD-032). Each independently validated break candidate normalizes to the saved
condition-false canonical exit. Pre-reduction integrity checks protect carried
provenance, exact frame nesting and balanced holds before cleanup/expiry can erase
evidence. Exactly the target frame ends; enclosing obligations and outside finite
uses remain authoritative. Skipped iteration suffixes do not weaken recurrence
before break. Mixed break/continue arms have zero fallthrough, without arbitrary
exit merging or terminal-path relaxation of pre-existing Move restrictions.
Labeled transfers, break/loop values, generalized cyclic ownership/lifetime fixed
points, MIR ownership, unreachable-code policy, reborrowing, destruction, execution
and termination proof remain unresolved.


### BD-033 refinement of OQ-022/OQ-024/OQ-025/OQ-026

BED-018 settles explicit structured early return and structural function
completeness in the current ordinary subset. Operand-only finite liveness excludes
bypassed continuations; current and enclosing recurrence remains authoritative
through actual operand evaluation. Terminal permission only relaxes the
pre-existing owned non-reference Move gate inside the return operand, never prior
operations, borrowing, reference transfer or replacement/destruction rules.
Reference returns, escaping lifetimes, reborrowing, general cyclic ownership/NLL,
function-exit state joins, unreachable/constant-path analysis, bottom/divergence,
first-class unit/void values, SSA/phi, and full MIR dataflow remain unresolved.
BD-034 subsequently settles NoValue ordinary function results and call statements. No runtime
cleanup/destruction is implied by return or loan expiry.


### BD-034 refinement of OQ-022/OQ-023/OQ-024/OQ-026

BED-019 settles explicit NoValue ordinary results, standalone NoValue calls,
bare return and validated function-boundary completion. Omitted arrows never
infer results; Value(T) retains BD-033 structural completeness. NoValue is not
a unit/void expression type. Branch/body fallthrough, loop false exits, local
break/continue, recurrence and all-path Move availability remain unchanged.
Consumed values and loan expiry imply no destruction or runtime cleanup (OQ-023).
Reference returns, escaping lifetimes, reborrowing, generalized cyclic ownership,
function-exit convergence, unit/void values, divergence/termination analysis,
SSA/phi, full MIR dataflow and execution remain unresolved. Historical accepted
sections describe their original boundaries; BD-034 does not reopen them.


### BD-035 refinement of OQ-020/OQ-021/OQ-022/OQ-023/OQ-024/OQ-025/OQ-026

BED-020 settles ordinary construction of the existing single named-range-field
record form and direct named-owner Copy-field reads. This does not settle
multi-field completeness/ordering/layout, broader field types, contextual literals,
constrained arithmetic proofs, invariant enforcement or mixed-assurance calls.
OQ-021 remains authoritative. Canonical field reads use whole-owner availability
and loans; they establish no partial move, field availability, field loan, reference
projection, auto-deref, generalized place or non-Copy replacement rules. Reads may
participate in Copy-valued while conditions, while construction remains forbidden
throughout recurring conditions. No destruction, cleanup, allocation, execution,
code generation or generalized cyclic inference is implied. Historical accepted
sections retain their earlier boundaries; BD-035 does not reopen other decisions.


### BD-036 refinement of OQ-004/OQ-012/OQ-016/OQ-019–OQ-026

BED-021 settles one-or-more named-range fields, complete named construction,
canonical declaration order and written initializer order. It refines BD-035's
historical single-field ordinary boundary, while verified state/harness support
remains single-field. Each initializer is exactly typed; full-expression rollback
and whole-record ownership apply. Copy reads do not introduce field places or
loans. This establishes record-initializer evaluation order only.

Visibility/readability/constructibility/mutability remain separate unresolved
module/interface authorities. Broader field types, defaults, partial moves,
field borrowing/mutation, reference projection, contextual literals, invariant
boundaries, mixed assurance, layout/ABI, destruction and execution remain open.
BD-006's invariant direction is preserved. OQ-026 remains authoritative for general
control flow, conditional values, MIR dataflow and generalized cyclic analysis;
no new ownership convergence or competing checker is introduced.


### BD-037 refinement of OQ-021/OQ-022

BED-022 settles only direct represented signed literal formation at independently
established canonical named-range ordinary returns, call arguments, assignment,
write-through and named-field initializers. No sibling/backward inference, local
annotations, general expected-type propagation, nonliteral formation, constant
folding, conversion or arithmetic-result bounds enforcement is settled. Raw
operator constants and specialized verifier/harness semantics remain unchanged.
Historical BD/BED sections retain their original no-contextual-formation boundaries.


### BD-038 refinement of OQ-025

BED-023 settles only direct Copy-field assignment through a mutable named owned
ordinary record local: exact nominal type, RHS-first/no-reservation ordering,
prepared-operation rollback, and post-RHS whole-record availability/loan checks.
Successful writes preserve one whole owner, without field-sensitive state or
terminal permission. Broader place/projection evaluation, reference-field mutation,
field loans, compound assignment, Move replacement, invariants, visibility,
destruction and generalized cyclic analysis remain unresolved. Verified/harness
mutation remains unchanged. BD-038 adds no execution or ordinary proof claim.


### BD-039 refinement of OQ-024/OQ-025

BED-024 settles direct Copy-field reads through named ordinary shared/exclusive
record references using existing whole-record provenance. This is not general
reference projection, auto-deref, conversion/reborrowing or field borrowing.
Whole-record non-Copy dereference, reference-field assignment, temporary/nested/
dereference projection, partial moves/loans, reference returns, lifetime syntax,
properties, destruction, execution and generalized cyclic inference remain open.
Owned-field assignment remains BD-038; the verified/harness path is unchanged.

### BD-040 refinement of OQ-024/OQ-025

BED-025 settles direct Copy-field assignment through a named ordinary exact
`&mut Record` handle using its continuously retained existing whole-record loan.
It requires final available-handle, live-lineage and original-LoanId authorization
and exact prepared-entry rollback on failure, while preserving lawful successful
RHS effects. This extends BD-039 reads and complements BD-038 owned-local writes.
General reference mutation/projection, reborrowing/coercion, field loans, partial
ownership, generalized places, replacement, properties/invariants, destruction,
cyclic lifetime inference and verified/harness expansion remain open or excluded.

### BD-041 refinement of execution questions

BED-026 Draft 3 settles bounded interpretation of the complete accepted ordinary
MIR subset, its authority artifact, written evaluation order, immediate range
traps, non-rollback, resource limits, and origins. Execution does not prove ordinary
contracts. Native code generation, destruction, source effects/exceptions/I/O,
concurrency, generalized places, reborrowing, and runtime verified/harness execution
remain outside this accepted boundary. Earlier statements that execution is open
now apply to those extensions rather than the frozen BD-041 interpreter.

### BD-042 refinement of OQ-006/OQ-022/OQ-023/OQ-025/OQ-026

BED-027 Draft 1 settles deterministic ownership-attached cleanup for canonically
resource-bearing values. It fixes unique movable responsibility, structural record
classification, reverse-order cleanup, covered structured exits, protected return
results, pending-construction field cleanup, explicit fallible completion, and
auditable MIR control flow without user-authored destructors. Borrowing and loan
expiry never own or trigger cleanup. Ordinary non-resource Move destruction is
inert under this decision.

Still open are implementation, trusted production resource APIs, constrained
custom resource admission, resource-bearing temporary lifetimes beyond BD-044's
admitted regions and destinations, typed recoverable
propagation syntax, non-Copy replacement, partial moves, panic/unwind/fatal-outcome
policy, cancellation, FFI, and broader generalized cleanup/control-flow dataflow.
BD-042 provides no guarantee across its excluded abrupt outcomes and authorizes
no compiler or runtime work by freeze alone.

### BD-043 refinement of OQ-022/OQ-023/OQ-025/OQ-026

Accepted BED-028 Draft 1 settles exact-typed whole-value replacement only for available,
directly named mutable owned non-reference Move locals. It fixes protected RHS
ownership, displaced canonical cleanup and new-generation ordering in the target's
owning activation. Iteration-local replacement is permitted; pre-existing loop
owners remain excluded. Ordinary non-resource Move destruction stays inert.

Reference/projected/dereferenced replacement, moved-slot reinitialization,
self-consuming transformations, take/swap/exchange, partial ownership, arbitrary
resource temporaries, user destructors, recoverable propagation, production resource
APIs and generalized cyclic ownership/dataflow remain unresolved or excluded.
The specialized verifier and proof counts are unchanged. Historical statements
that non-Copy replacement is open now apply to those broader facilities.

### BD-044 refinement of expression-temporary questions

Accepted BED-029 Draft 2 settles bounded ownership, region retirement and destination custody
for admitted resource-bearing expression intermediates. This narrows the historical
arbitrary-temporary exclusions above, without settling temporary borrowing, lifetime
extension, value-returning call statements, recurring-while Move/reference activity,
public resource admission, arbitrary destructors, recoverable propagation, unwind,
rollback or generalized ownership/dataflow. Existing aggregate field order and
BD-043 replacement remain authoritative. The accepted implementation introduces no
new verifier proof claim.

### BED-031 refinement of OQ-006/OQ-022/OQ-023/OQ-026

Frozen Draft 4 settles the first ordinary exact Result family, explicit exhaustive
two-arm statement matching, independent outcome obligations, active-payload cleanup,
and manual protected-return propagation. Error and Err are equal exact aliases for
one failure variant. This implementation awaits independent review and acceptance.
Historical wholly-open Result statements are narrowed by this fixed phase.

General enums, Option, general patterns and match expressions, propagation sugar,
implicit error conversion, reference payloads, Result public record fields,
verified/harness Result use, production fallible APIs, panic/unwind/cancellation
and generalized ownership/dataflow remain excluded or open. Result grants no unsafe
permission and no tool-stop recovery or rollback.

### BED-032 refinement of propagation questions

Frozen Draft 2 narrows Result propagation to exact contextual `or return` at an
ordinary local initializer with exact shared failure identity, actual enclosing
scope, independent initializer typing and protected ordinary return. It does not
settle `?`, general postfix/nested propagation, implicit conversion, universal
errors, generalized sums/patterns, verified/harness Result execution, production
fallible APIs, panic/unwind/cancellation or generalized cyclic ownership.
Earlier wholly-open propagation references now apply to those deferred extensions.
Implementation evidence and independent acceptance remain distinct.

### BED-033 refinement of trusted-operation and resource questions

Frozen Issue #78 Draft 2 narrows OQ-001/OQ-006/OQ-012/OQ-013/OQ-022/OQ-023/OQ-026
only for the one explicitly selected compiler-supplied sensor-demo family. The
implementation covers canonical trusted calls, exact existing nominal ranges,
opaque owned Sensor/shared observations, Result recovery, ownership-derived io
cleanup, no-gap receipts, persistent terminal custody and bounded rooted authority
audit. Historical wholly-open public-resource statements now apply beyond this
exact slice. Implementation evidence remains distinct from independent acceptance.

General FFI/ABI, user-defined resources/destructors, standard-library layering,
modules/interface/manifest syntax, capabilities, concurrency, realtime guarantees,
reference escape, mutable Sensor refs, Sensor conditionals/replacement/record fields,
public live-resource egress, native execution and verified trusted execution remain
open or excluded. Automatic release promises no flush/commit/durability/completion;
explicit fallible completion and ownership after its failure remain deferred to
BED-034. No such follow-on work is authorized by BED-033 implementation.

### BED-035 accepted Stage I engine-equivalence boundary

Frozen Issue #84 Draft 2 fixes the approved-MIR semantic relation independently of
any backend. The accepted Stage I implementation extracts and validates the
interpreter contract, separates logical and evidence accounts, captures and
validates bounded observations, compares anchored identities, and supplies private
addressed replay and modeled Host evidence. Sarah resolved K35-I01 through K35-I04;
Greg accepted the implementation; human-merged PR #85 is
`c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717`, tree
`a76e2ab68c8d2db094f1005906bdc24b6a9b8795`.

Native backend, target, ABI/data layout, emitter, optimizer, object/runtime/linker,
portable artifact binding, transport, serialization, signing and packaging remain
unselected. Actual native installation, optimized/unoptimized execution, interpreter/
native equality and native physical-failure evidence remain mandatory Stage N
questions for a separately authorized BED. Analytic preallocation/address fixtures
are protocol evidence only. Incomplete observations and substrate destruction do
not acquire equality or survival guarantees. Public inputs, resources, profiles,
verified/harness execution and existing predecessor boundaries are not widened.


## OQ-027 — Structured documentation comments

DP-016 and the repository documentation policy accept documentation as part of
the program's human-facing contract. The current executable prototype recognizes
ordinary `//` line comments as trivia and assigns no special documentation
meaning to `///`, `//!`, block comments, Doxygen tags, or another spelling.

Still open:

- exact documentation-comment and module/file-comment syntax;
- lexical and AST attachment rules;
- whether ordinary comments are preserved for formatting independently of
  structured documentation;
- the documentation markup and structured metadata model;
- generated-documentation format and stable links;
- compiled documentation examples and their assurance/profile context;
- checked references to parameters, fields, types, effects, contracts, and
  trusted assumptions;
- formatter/linter policy and the severity of missing or stale documentation;
- inheritance through traits, implementations, modules, and future generics;
- versioning and compatibility treatment for documentation contracts;
- editor, audit, and review integration.

Documentation metadata must not grant authority, change language semantics,
create proof evidence by assertion, conceal trust, or override canonical compiler
artifacts. Mechanically checkable claims should be derived from or validated
against canonical facts where practical; broader prose remains subject to human
review.
