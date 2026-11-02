# Bowfin Compiler and Verification Architecture

## 1. High-level pipeline

```text
source (.bow; explicit .klx input retained)
   │
   ▼
lexer
   │
   ▼
parser
   │
   ▼
AST
   │
   ▼
name resolution
   │
   ▼
expression type checking
   │
   ▼
typed HIR
   │
   ├── type checking
   ├── ownership checking
   ├── region constraints
   └── effect checking
   │
   ▼
MIR
   │
   ├──────── verification path ────────▶ VIR ─────▶ SMT / prover
   │
   └──────── code generation path ─────▶ backend ─▶ machine code
```

The backend must not define Bowfin's source-level semantics.

## 2. AST

The AST preserves source structure for diagnostics, formatting, documentation, and syntax-aware tooling.

## 3. HIR

Typed HIR resolves source conveniences into clearer semantics.

Responsibilities may include:

- resolved names;
- canonical types;
- desugaring;
- normalized keyword aliases;
- generic/trait metadata;
- effect declarations;
- contracts.

Example:

```text
mut
mutable
   ↓
KW_MUTABLE
```

Original spelling remains available for source-aware tooling.

## 4. Ownership and regions

Availability (Available/Moved) and active shared/exclusive loans are orthogonal.
The current prototype checks moves, acyclic loans, and narrow Copy loop stability and iteration-local Move ownership together in `compiler/ownership`.
Stored loans follow derived reference handles through last use; direct reference
arguments remain call-held through the receiving call. Broader regions/lifetimes
are future architecture; explicit syntax remains open under OQ-009/OQ-024.

## 5. Effect analysis

Conceptual judgment:

```text
Γ ⊢ expression : Type ! Effects
```

Effects propagate transitively.

A caller denying `heap` must not reach allocation through a hidden call chain.

## 6. MIR

The accepted long-term MIR direction is control-flow oriented and suitable for:

- ownership analysis;
- drop elaboration;
- borrow checking;
- effect propagation;
- contract instrumentation;
- verification lowering;
- backend lowering.

## 7. VIR

VIR is the restricted verification representation.

Expected concepts include:

- `assume`
- `assert`
- `forall`
- `exists`
- `old`
- `invariant`
- `variant`
- `havoc`

An initial verifier may target Z3 or another SMT solver. Solver choice is not language semantics.

## 8. Contract semantics

Conceptually:

- caller establishes `requires`;
- callee establishes `ensures`;
- invariants hold at defined semantic boundaries.

Checked mode may lower contracts to runtime checks.  
Verified mode generates proof obligations.

## 9. Loop verification

Verified loops may require invariants and termination variants where termination must be proven.

## 10. Trust tracking

`trusted` statements/declarations should retain:

- source location;
- rationale;
- direct dependents;
- transitive dependents where feasible.

A future command may support:

```bash
bowfin audit --trust
```

## 11. Unsafe boundaries

Unsafe operations should be lexically visible.

Possible future direction:

```bowfin
unsafe
    assumes ptr != null
    assumes ptr aligned_for T
{
    ...
}
```

Exact syntax remains open.

## 12. Realtime / embedded profiles

Initial support should focus on structural restrictions such as:

- no heap;
- no unbounded blocking;
- constrained recursion;
- bounded loops where policy requires it.

Do not claim arbitrary WCET analysis until it truly exists.

## 13. Runtime strategy

Bowfin should support modular runtime configurations from hosted targets to bare metal.

## 14. Implementation direction

Current direction:

- first compiler likely implemented in Rust;
- LLVM is a plausible initial code-generation backend;
- Z3 is a plausible initial SMT backend;
- self-hosting is a long-term possibility.

These are implementation choices, not permanent language semantics.

## 15. Proposed repository structure

```text
compiler/
├── lexer/
├── parser/
├── ast/
├── resolve/
├── types/
├── ownership/
├── effects/
├── hir/
├── mir/
├── vir/
├── verify/
├── codegen/
└── diagnostics/

runtime/
std/
tools/
tests/
examples/
docs/
rfcs/
```

## 16. MVP boundary

A narrow first compiler should target:

- integers and `Bool`;
- bindings and mutation;
- functions;
- records;
- `if` / `while`;
- range types;
- basic `Option` / `Result`;
- basic matching;
- moves and borrows;
- `requires` / `ensures`;
- deterministic checked arithmetic;
- machine-code backend;
- simple integer verification.

The MVP should demonstrate the difference between memory safety and application-level verified correctness before pursuing breadth.

## 17. Current executable prototype

The compiler prototype is implemented in Rust, pinned to 1.80.0. The executable
pipeline is source → lexer → parser → AST → explicit name resolution → expression
and value-flow type checking → typed HIR → core ownership, loan, and borrowed-access checking →
ordinary MIR CFG lowering (ordinary functions) or specialized
integer contract proof (verified functions) → diagnostics. This implements BD-012, BD-018, BD-019,
BD-020, BD-021, BD-022, BD-023, BD-024, BD-025, BD-026, BD-027, BD-028, BD-029, and BD-030 only for the documented
subset; the broader pipeline above remains accepted architecture/future work.

The AST preserves textual declarations, parameter types, explicit ordinary
returns, statement conditionals with optional else/nested lexical branches,
local initializers/mutability, flat source reference types, direct named
borrow/dereference targets, dedicated ordinary dereference writes, expression calls,
contracts, and ordered prototype
statements. `FunctionDecl` retains source order across ordinary and verified forms.
The resolver collects all declarations/signatures before resolving bodies, owns
name lookup and source-order local scope, and assigns strongly typed range, record,
field, function, parameter, local, and harness-binding IDs. Locals enter scope after
resolving their initializer; visible-name shadowing and cross-function local scope are rejected.
Each branch clones the visible resolver scope; siblings may reuse a spelling,
but every declaration receives a fresh LocalId from the compilation-wide allocator.
Forward and recursive ordinary calls resolve without execution or termination
analysis. Source type names resolve to canonical references; `compiler/types`
validates declared value types, operators, initializers, calls, and returns without
textual name lookup.

One canonical `FunctionId` table contains both ordinary value and verified mutation
functions, in source declaration order. Both kinds share the existing function
namespace. Ordinary signatures/materialized locals use `ValueType::Bool`,
`ValueType::Range(RangeTypeId)`, `ValueType::Record(RecordId)`,
`SharedRef(ReferentType)`, or `MutableRef(ReferentType)`; `IntegerLiteral`
remains expression-only. Expressions represent records by the same canonical
RecordId. Record arguments and returns require exact nominal identity; existing
operators do not accept record or reference values. Reference referents are
nonrecursive Bool/range/record identities. Borrow expressions carry canonical
ParameterId/LocalId targets and Shared/Mutable capability; locals carry owned
mutability. Nested reference signatures and reference returns are rejected before ownership. HIR
local and call references carry `LocalId` and `FunctionId`, with typed arguments,
initializers, and explicit returns. Nominal compatibility uses exact range/record IDs.
Names and spans are diagnostic metadata. `hir::Program` owns compilation-local
canonical tables, including locals, and exposes read-only slices/ID lookups.
Storage is crate-private; internal construction/transformations must preserve
identity and reference invariants. IDs have no cross-compilation stability.

Ordinary functions have explicit NoValue or Value(T) result metadata under BD-034.
Value functions require structurally complete explicit `return expression;` paths
under BD-033; NoValue functions permit `return;` and validated top-level completion. Immutable locals propagate concrete initializer types;
call arguments and returns require exact concrete types. Bare integer locals remain unsupported. BD-037 forms direct literal leaves at
independently established named-range return/argument/assignment/field boundaries.
BED-004 through BED-011 do not execute, prove, or dynamically enforce ordinary functions.
`Range(T)` typing does not prove bounds or overflow safety. Plain `fn` is not a
claim to implement the full future `safe` model. All mixed-kind calls are rejected.

`compiler/ownership` is a dedicated phase that inspects canonical typed HIR without
rebuilding it or adding public ownership annotations. Its private classification
is Bool/range/shared reference → Copy, record/mutable reference → Move,
regardless of record field types. It tracks only
ordinary ParameterId/LocalId places, using separate strongly typed place
variants in a per-function state table. Record parameters begin Available. Local
initializers are analyzed before introducing a fresh local owner. Value references
consume Move places through bindings, by-value calls, and returns; Copy places
remain reusable. Call results are fresh values, without public temporary IDs.

State is Available, Moved with the prior source span/transfer destination, or
conditionally unavailable after a branch join with a contributing move span.
Use-after-move produces the first precise `FrontendError::Ownership` diagnostic,
with the later use span and prior move location. Names are diagnostic metadata;
classification and state use canonical types and IDs. Children are traversed in
source order for deterministic diagnostics, without settling runtime evaluation
order. BED-008 adds independent post-condition branch snapshots and acyclic joins.
An outer Move place moved on either path becomes conditionally unavailable, with
a prior move span for diagnostics. There are no fixed points or inlined callee states.
Forward/recursive calls are checked structurally, not executed or proven terminating.
Unused owners are accepted without destruction behavior.

The ownership phase excludes the verified state parameter, fields, and BindingId
harness state. BED-006 adds private loan identity/provenance alongside availability,
with continuation-aware path-specific future-use counts (BED-009) and operation-hold counts for nested
calls and (under BED-007) whole write statements.
Shared copies and mutable moves retain loan identity across LocalIds. Loans expire
when no potentially available derived handle has future uses on the current path
and no call/write hold or BD-030 recurrent obligation retains the loan. ConditionalMove blocks handle use but does not
by itself eliminate a loan active on another incoming path. Local
transfers attach destination provenance before expiry; final-use arguments attach
call holds before expiry. Reference parameters have external provenance without
interprocedural owner reconstruction. Loan state is not published in HIR.
Exclusive borrowing requires a mutable owned local; `let mut` introduced exclusive borrowing under BED-006; BD-026 now adds Copy-safe local assignment.
BD-035 adds ordinary single-range-field construction and named-owner Copy reads.
There is no generalized reference projection beyond BD-039, reborrowing, reference return,
implicit reference coercion, partial move/borrow, user-defined Copy/Clone, or destructor. Source names
and nominal type errors must be resolved before ownership checking.

BED-007 represents `Deref { reference: Place }` with the exact canonical referent
ExprType and a dedicated `DerefAssign { reference, value, span }` statement. The
resolver identifies existing ordinary parameter/local targets; typing requires
reference operands, exclusive write capability, and exact RHS compatibility.
Ownership rejects borrowed non-Copy value materialization and non-Copy replacement.
No generalized projection/place architecture or additional frontend pass is added.
Read/write access checks handle availability without transferring it or creating
loans. Both count for last use. Copy reads can expire the original loan immediately
after access, and their result does not call-hold a reference. Write statements hold
the original loan through RHS traversal, then expire normally after completion;
RHS calls cannot invalidate the target handle and leave an accepted write.
No destruction or auto-deref is implemented. BD-038 adds narrow direct Copy-field mutation. BD-026 adds separate Copy-safe local assignment.

The verifier takes only a verified-function view from the shared HIR table;
ordinary functions are ignored as proof targets and never counted as proven.
Verified AST/HIR contracts remain expressions, and subtraction statements have
expression operands. The verifier recognizes only the original resolved battery
proof structure and parameter/literal body operands. Richer well-typed expressions
inside verified functions receive spanned verifier-support diagnostics. The
private affine kernel and its independent 1,800-case cross-check retain their
existing mathematical coverage, including independently varying numeric domains.

Ordered top-level record construction/call statements remain the prototype harness. The
checker retains distinct-record checks, range checks, and immutable-binding
rejection. Every verified body is proven independently of its call sites; its
postcondition then updates caller state in source order. Ordinary functions do
not enter that state model.

BED-009 removes BED-008's whole-conditional loan pin. Future-use accounting is
private and independent of availability, keyed by canonical ParameterId/LocalId
places. After analyzing a condition once, the remaining suffix is decomposed
into then uses, else uses, and a common continuation. Each branch starts from
the same post-condition ownership snapshot, with only its own uses plus that
continuation. Its edge expires loans with no future surviving handle uses.
Nested statement-list analysis composes this decomposition recursively, and
operation-level last-use expiry remains unchanged. No constant-path pruning occurs.

At joins, outer Move availability retains BD-023's all-path rule. Only loan
provenance that existed before the split may survive; activity is the union of
incoming states. Dead-on-all-path loans are removed. ConditionalMove can retain
possibly active provenance while rejecting any handle use. Branch-local handles
and loans are discarded; sibling-created private LoanIds are never conflated.
Future uses after the join keep needed loans continuously active along each
relevant incoming path: no loan resurrection occurs. Call/write holds remain
balanced through their operation and cannot escape statements or branch exits.

Ownership remains above MIR in `compiler/ownership`; there is no second checker,
MIR ownership pass, general dataflow framework, or fixed-point solver. This is
whole-value liveness for structured acyclic statement and initializer branches, not arbitrary CFG borrowing
or a claim of Rust borrow-checker equivalence. OQ-024/OQ-026 retain general lifetimes,
generalized cyclic lifetime analysis beyond BD-030, reborrowing, escaping references, and broader MIR ownership dataflow.

`compiler/mir` lowers only ordinary typed HIR after frontend ownership checks.
`mir::lower(&program)` leaves `compile_source`'s canonical HIR result unchanged.
MIR preserves FunctionId/ParameterId/LocalId and exact expression types. Public
inspection is read-only; BasicBlockId and canonical block storage are compilation-local.
Each block owns Let/Assign/DerefAssign statements and exactly one Goto, typed Bool Branch,
or value Return terminator. Expressions retain their typed HIR trees. Structured
recursive lowering allocates blocks deterministically, gives every fallthrough
an explicit Goto, and emits each explicit source return as a terminator. BD-033 removes final-return
peeling; only actual fallthrough paths join and no final return is synthesized. Empty else
branches use a direct false edge to the join. A structural validator checks entry,
targets, reachability, and Bool conditions; unique IDs and single
terminators follow from indexed storage and the block type. BED-012 permits cycles;
visited tracking terminates validation without claiming paths reach Return. MIR discovers no new source semantic errors. The CLI exercises lowering.

BED-010 adds initializer-only conditional values with mandatory else and exactly
one expression per branch. AST/resolved/typed HIR retain explicit IfValue nodes;
resolution uses the same visible scope without introducing branch declarations.
The destination enters scope after its whole initializer resolves. Type checking
requires Bool conditions and exact owned Bool/range/record identity; references
and IntegerLiteral results cannot materialize conditional destinations.

The existing ownership checker shares snapshot/split/join helpers for statement
and value conditionals. It evaluates the condition once, gives each branch its
own future uses plus the continuation, and transfers every leaf into the same
canonical source LocalId. Copy sources survive; moved sources become conditionally
unavailable after the join. The destination is inserted as available only after
the whole initializer completes. BD-024 provenance and operation holds are unchanged.

MIR value lowering emits explicit Branch blocks with distinct outgoing targets.
Every nested leaf emits an existing Let statement for the same source destination
and Goto to a common control-flow join. Multiple mutually exclusive initializations
are not reassignment. No hidden IfValue survives inside any MIR expression; the
structural validator checks this. Deterministic construction and mandatory value
branches guarantee one initialization per reachable path without a general
definite-assignment solver. This first destination-local strategy is not a final
general MIR solution for all future value joins (OQ-026).

The executable flow is typed HIR → ownership/path-sensitive acyclic loans → MIR
statement CFGs and destination-local value joins. MIR still has no backend or
verification consumer. The verified battery path excludes ordinary conditionals.

BED-011 extends ordinary statements with Assign. The AST preserves the direct target
spelling, RHS, and complete span. Resolution binds the target to a canonical Place;
typing rejects parameters, references, and immutable locals, requires exact RHS
identity, and publishes only a LocalId target. It creates no declaration or new ID.
Exact-typed non-Copy record replacement reaches the ownership rejection boundary.

For mutable Bool/range targets, the existing ownership checker analyzes the RHS
under current state, completes call holds and last-use expiry, then checks the
write against every active shared/exclusive loan of that owner. This RHS-first,
write-second rule allows final RHS reference uses to end before mutation. BD-024
branch-specific future counts include RHS uses and the continuation; loan merging,
Move/ConditionalMove availability, call holds, and write-through holds are unchanged.
Non-Copy replacement fails before either owner is consumed or displaced.

MIR Assign retains the source LocalId, typed RHS, and span; it is distinct from
Let initialization and DerefAssign. Straight-line lowering adds no blocks; branch
assignments stay in the current block. Expression validation also inspects Assign
RHSs for hidden IfValue nodes. BED-012 subsequently replaces MIR cycle rejection with cycle-safe reachability. There is no
second ownership engine, general place model, reference provenance replacement,
destruction or fixed-point infrastructure. BED-012 adds loop backedges as described below.

The implemented ordinary path now includes initialization, conditional initialization,
Copy-safe mutable-local reassignment, and explicit dereference write-through →
ownership/path-sensitive acyclic loans and Copy loop stability/iteration-local Move → MIR Let/Assign/DerefAssign statements
with explicit Branch/Goto/Return terminators. Ordinary code remains non-executable
and unverified, and the specialized battery path remains unchanged.

There are no phi nodes, block parameters/results, SSA, general temporaries,
cleanup edges, execution, or MIR backend/verifier consumers. The specialized
verified path stays on HIR and retains its proof counts and numerical kernel.
OQ-026 records future control-flow/value-flow/MIR dataflow design.

VIR, ownership beyond core moves and acyclic whole-value loans, advanced
borrowing, effect analysis, destruction, SMT integration, runtime lowering,
and code generation remain unimplemented. This subset does not settle broader
HIR, modules, generalized inference/coercions, arithmetic enforcement, or mixed
assurance semantics.

For the supported precondition `amount <= state.field` and postcondition
`state.field == old(state.field) - amount`, the proof establishes signed i64
subtraction safety, range preservation at each assignment, and the final equality
over every admissible input. It uses affine extrema at the vertices of the
range rectangle clipped by the precondition. This is a specialized proof with
the implementation in its trusted base, not an SMT proof or proof certificate.
Empty admissible domains are explicitly rejected by this prototype.

The exact grammar, mathematical argument, CLI behavior, and limitations live in
`compiler/README.md`. The broader language's representation, assurance-boundary,
invariant, and snapshot rules remain open. Relevant entries: BD-005, BD-006, BD-012,
BD-013, BD-014, BD-015, BD-017, BD-018, BD-019, BD-020, BD-021, BD-022, BD-023, BD-024, BD-025, BD-026, BD-027, BD-028, DP-008, DP-009, and OQ-018 through OQ-026.


### BED-012: while and cyclic MIR

AST, resolved statements and typed HIR preserve explicit While condition/body/span.
Resolution gives the body a child scope using the existing canonical LocalId
allocator; type checking establishes exact Bool before ownership. Ordinary branch
and loop bodies cannot contain returns. Verified syntax remains separate.

The existing HIR ownership checker enters a private loop mode and analyzes one
permitted iteration from a header snapshot. BED-012 restricted nested expressions/statements to
owned Copy values; BD-028 subsequently permits iteration-local Move in bodies,
as described below. BD-029 subsequently permits iteration-local body references;
pre-existing-handle use, reference conditions and non-Copy conditions reject with
source-oriented future cyclic-analysis diagnostics. The check compares outer Move
availability, handle provenance, loans (including balanced operation holds), and
loan allocation state at the backedge after body-local state is scoped away.
Nested loops use the same check independently. FutureUses only advances the finite
outside suffix; prohibited recurring handles are never approximated by one count.
Outside live loans are preserved and still checked by Copy owner access/assignment.
There is no fixed-point, MIR ownership pass, or second engine.

Deterministic lowering reserves a header/body/exit, emits Goto from the preheader,
Branch at the header, and Goto from the completed body back to the header. Nested
if/value-initialization joins and nested loops compose recursively. Existing typed
expressions and canonical source IDs survive; no MIR While instruction is needed.
The validator uses visited marks to accept cycles while retaining target,
reachability, Bool and hidden-IfValue checks. It makes no termination claim and
retains body/exit edges for constant conditions. The ordinary pipeline is source
→ AST → resolution → typed HIR → ownership/acyclic loans/Copy loop stability →
MIR CFG. Backend and verification lowering remain unimplemented; the specialized
battery proof path and numerical kernel remain unchanged.


### BED-013: iteration-local Move ownership

The existing checker replaces the BED-012 blanket non-Copy loop-body rejection
with a private per-loop scope containing canonical pre-existing Move places and
condition/body mode. Condition analysis retains the strict Copy boundary. Body
expressions may return fresh records; existing consumption checks permit transfer
only when the source is not in the current loop's header set. A forbidden transfer
rejects before changing availability or consuming future-use counts. Non-Copy
assignment still rejects before RHS analysis. BD-029 subsequently refines the
body-reference restriction without relaxing pre-existing-handle or condition checks.

Branch clones inherit the same loop boundary. A nested loop establishes a new
header from the current state, protecting outer-iteration locals; afterward the
outer boundary is restored. Each loop checks one cloned iteration, scopes away
body-local availability/provenance, and compares remaining state semantically with
its header. Source names, diagnostic move sites, spans and finite FutureUses counts
remain excluded from equality. Body-local Available/Moved/ConditionalMove states
may evolve normally and disappear at lexical exit without runtime destruction.
Outside handles/loans remain unchanged at accepted backedges and constrain owner
access as before; BD-030 distinguishes false-exit liveness.

No AST, resolver, type, HIR or MIR representation change is required. Existing
Let/call expressions and Branch/Goto backedges retain static LocalIds, exact types,
deterministic construction and cyclic validation. Ownership remains above MIR
without generalized fixed points or a second engine. BD-029 adds iteration-local
borrowing; OQ-024/OQ-026 retain loop-carried ownership/reference-use analysis; OQ-023/OQ-025 retain
replacement/destruction. Ordinary functions remain non-executable and unverified.


### BED-014: iteration-local borrowing

BD-029 permits fresh body references to pre-existing owned Copy/record values and
iteration-local Move owners. Existing shared aliases, mutable-handle transfers,
call holds, named dereference, Copy-safe writes and acyclic path-sensitive last-use
rules apply. A borrowed local record can move after its loan ends. Each loop
captures canonical header handle identities and loan provenance: these handles
were unusable under BED-014 but could persist untouched. BD-030 subsequently
permits stable carried access without loan resurrection.
Live header loans continue constraining compatible owner access/new borrows.
While conditions remain reference-free; non-Copy condition/transfer and replacement
boundaries are unchanged. Nested loops classify outer-created handles as
pre-existing and restore the outer boundary afterward.

After one cloned iteration, lexical cleanup removes body handles and availability,
then expires loans. The checker proves no iteration-created loan or surviving
handle provenance remains before restoring the header allocator; semantic header
comparison still checks availability, handles, active loans and allocation state.
Names/spans and finite FutureUses counts remain diagnostic/continuation metadata.
This permits safe reuse of transient private LoanIds without hiding provenance.
No generalized cyclic NLL, fixed-point solver, second checker, syntax, HIR/MIR
surface change, runtime destruction, execution or verification is introduced.
BED-015 subsequently permits stable carried access through scoped recurrent
obligations; generalized cyclic lifetimes remain open under OQ-024/OQ-026.


### BED-015: stable loop-carried reference access

BD-030 distinguishes straight-line references, iteration-local loop references,
and stable loop-carried references. Existing header shared handles may be read,
passed to calls and copied into body-local aliases; mutable handles may be read
and used for Copy-safe writes, but cannot move into bindings or by-value calls.
Reference conditions, reference replacement, reborrowing and carried owned Move
changes remain unsupported. Constant and apparently one-shot loops do not weaken
recurrent protection.

The same typed-HIR checker discovers canonical header reference uses in both
branches and nested bodies. Private scoped sets of original LoanIds provide
recurrent retention independently of finite FutureUses and operation holds.
Branches inherit all frames: a skipped use cannot expire the recurrent loan.
Loans retain their identity/owner/kind continuously, without refreshing, sentinel
counts or repeated body analysis. Nested loops add frames and discharge only their
own obligations. Outer-created provenance may be carried by an inner loop but
must still die before the outer backedge under BD-029.

Lexical cleanup removes body-local aliases and new provenance; allocator
normalization cannot conceal surviving new loans. Exact backedge validation runs
with obligations still active. Only then is the current frame discharged and
false-exit suffix liveness applied, preserving enclosing obligations and post-loop
uses. With neither, the loan can end and owner access resume. Compound expressions
and statements restore their pre-operation snapshot on error, including partially
consumed counts and operation holds. Canonical HIR/MIR and public APIs are unchanged.
No generalized cyclic lifetime/fixed-point solver, second checker, MIR borrowing,
execution, destruction or termination proof is implemented.


### BED-016: structured continue and explicit backedges

AST, resolved statements and typed HIR preserve Continue/span. Parser and resolver lexical
loop depth reject use outside an ordinary while; recursive loop/branch structure
identifies the innermost target. Structural block checks reject any suffix after
a direct continue or an all-terminating conditional. Loops still have a false
exit, including constant conditions and all-continuing bodies. No general
unreachable-code analysis is implemented.

The existing ownership checker carries immutable header snapshots and finite
outside continuations on a private nested target stack. Statement analysis returns
fallthrough or a previously validated continue; conditionals analyze each arm in
source order and join only surviving fallthrough states. A single surviving arm
keeps its actual effects while branch-local state is scoped away. Each explicit
continue runs the existing lexical provenance cleanup, safe allocator normalization,
and exact header comparison transactionally with recurrent frames still active.
Normal bottom-of-body backedges use that same invariant. No continue discharges
an obligation frame; failed analysis restores its complete entering snapshot.

Finite future-use summaries are computed backwards over structured syntax before
forward ownership operations. A continue substitutes its loop's outside finite
continuation for the skipped same-iteration suffix. Statement branches receive
separate summaries; a pre-dispatch summary retains uses possible on either edge.
Expression-level counts and existing call/write holds remain unchanged. Recurrent
LoanId sets remain separate from FutureUses and retain exact provenance even with
zero finite counts and zero holds. The independently checked header-condition false
state, not a fabricated body fallthrough, supplies the loop exit. Only its current
frame is discharged; enclosing frames and post-loop finite uses still protect loans.

MIR maintains a nested header stack and returns an optional live tail. Continue
finishes the current block with the existing Goto to that header. No new terminator
or ownership annotation is introduced. Only real fallthrough arms create joins or
lower a suffix, so all emitted blocks remain reachable under the unchanged
validator. The header false exit still exists when all body paths continue.
Canonical source IDs, deterministic lowering, and the verified proof path remain
unchanged. No generalized CFG/fixed-point machinery or competing checker is added.


### BED-017: structured break and canonical loop exits

AST, resolution, and typed ordinary HIR contain Break/span. Parser and resolver
reject placement outside while, labels/values, and suffixes after a direct break
or recursively all-terminal conditional. Actual edge class and lexical
fallthrough are distinct: mixed Break/Continue arms have no surviving suffix
state. The single HIR ownership checker retains immutable header, canonical false
exit, and outside-continuation snapshots on a finite nested target stack.

The four structured loop edges have distinct contracts:

| Edge | Ownership/reference boundary | Recurrent frames | Existing MIR |
| --- | --- | --- | --- |
| Ordinary bottom backedge | Exact header state after local cleanup | Retain current and enclosing | Goto header |
| Explicit continue backedge | Independently validate exact header state | Retain current and enclosing | Goto header |
| Ordinary condition-false exit | Independently derive canonical exit before body candidates | Discharge exactly current; retain enclosing | Header Branch false target |
| Explicit break exit | Validate actual normalized candidate against saved canonical exit | Discharge exactly target; retain enclosing | Goto innermost exit |

Canonical false exit comes from the checked condition state, outside finite uses,
and outer obligations. A candidate never supplies or replaces it. Break first
checks invariants whose evidence could disappear: exact frame membership/position,
outer context, carried handle mapping and original active LoanId/owner/kind, and
balanced operation holds. This is bounded integrity checking, not raw header
state equality. Then the actual candidate installs captured outside uses before
expiry, scopes target-iteration locals, pops exactly the target frame/context,
expires with outer obligations and finite uses, proves fresh provenance absent,
and safely normalizes its allocator. Semantic comparison does not repair carried
availability or provenance. All steps are transactional; failure restores the
actual entering snapshot, including faults and canonical-exit target context.

Finite backward summaries cut skipped same-iteration suffixes before preceding
operations. Recurrent sets remain separate and active until break itself; post-loop
finite uses independently protect loans. Nested exits preserve outer frames and
continuous LoanIds, including when the removed inner frame is empty. A recurrent-only
loan may end when no outer or outside retention reason remains. No resurrection,
pre-existing Move relaxation, generalized fixed point, or second checker is added.

MIR maintains header/exit pairs and optional live tails. Break finishes its block
with existing Goto to the innermost exit; continue still targets the header. No
all-terminal suffix join is emitted; nested target restoration is deterministic.
The validator is unchanged. Constant and all-breaking bodies retain false edges.
Private tests inspect normalized candidates before loop consumption and inject
pre-reduction and post-cleanup faults. The specialized verifier and its 1,800-case
affine cross-check remain unchanged; ordinary code is not executed or proven.


## Structured early return ownership boundary (BED-018 / BD-033)

The existing parser/AST → resolver → types → typed HIR → ownership → MIR pipeline
is unchanged. The resolver validates lexical terminal suffixes, including for
publicly constructed ASTs. Types establishes no structural closing-brace
fallthrough after consuming loop-local transfers and exactly types every return
before ownership. A while always contributes its condition-false path; constants
do not prove completeness or termination.

Backward finite summaries use `finite_before(return E) = finite_uses(E)`:
all bypassed continuations are excluded before analyzing preceding operations.
Separate current/enclosing recurrent LoanId obligations remain authoritative
throughout operand evaluation. Private terminal-return context, independent of
Transfer::Return, bypasses only the pre-existing owned non-reference Move gate
inside the actual operand tree. Availability, loans/provenance, reference gates,
call/write holds and all other checks remain authoritative.

The return transaction snapshots only after operand-summary installation and
ordinary expiry, before enabling terminal permission. Nested evaluation and final
fallible hold validation roll back to that exact entry on failure. Permission is
restored on both outcomes. Return flow has no local successor; its state is never
joined or normalized to a loop header/false exit. Actual surviving branches keep
their own effects. Mixed return/break/continue works in the existing bounded
structured checker; there is no synthetic function-exit ownership state.

Whole-body structural MIR lowering reuses existing Return, Branch and Goto.
All-terminal conditionals have no join and no synthetic final Return. Existing
MIR validation is semantically unchanged, including cycle support and no claim
that execution reaches a Return. No second checker, generalized CFG/fixed-point
ownership engine, SSA/phi nodes, reborrowing, destruction or backend is added.


## No-value result and completion boundary (BED-019 / BD-034)

AST/resolution/signatures/HIR represent `NoValue | Value(T)` using an optional
result payload (`None` means NoValue, never inference). Expression and value
types have no unit/void case. Distinct bare-return and named-call statement
nodes preserve source structure and spans. Typing uses canonical FunctionId
signature metadata to reject wrong return forms and call positions, including
public-AST construction, before ownership. The compile_source result remains
canonical typed HIR; MIR inspection remains downstream through mir::lower.

The existing ownership checker handles statement calls with a complete
prepared-entry transaction including all arguments, holds, release, expiry and
final balanced-hold validation. Both finite-use and recurrent-reference traversals
inspect arguments. Nested calls release only their own holds. Bare return installs
empty finite liveness, expires, snapshots, validates, and terminates; failure
restores that prepared snapshot. It has no operand or BD-033 terminal permission.
Recurrence, loan provenance, reference availability and holds remain unchanged.
Top-level NoValue fallthrough validates balanced holds without synthetic exit
state, repair or cleanup. BD-033 value-return transactions are preserved.

MIR adds CallNoValue statements with canonical typed arguments and ReturnNoValue
terminators. Live top-level tails complete only for NoValue functions; nested
braces remain local fallthrough and all-terminal branches create no extra block.
Validator traversal checks every call argument for unlowered conditional values.
No result expression/local, second checker, CFG ownership solver, fixed point,
execution, destruction or verified-path expansion is implemented.


## Ordinary record expression boundary (BED-020–021 / BD-035–036)

Source AST records contain ordered, individually spanned field declarations.
RecordConstruct retains a vector of individually spanned textual field entries
and their expression children, plus a constructor-identifier span. Existing source FieldAccess remains source
oriented. Contextual syntactic recognition distinguishes field-colon construction
from an identifier condition followed by a statement block; final binary/unary
operands and verified contract/body boundaries preserve block parsing. No parser
type-name lookup is used. Malformed value-position constructions receive targeted
messages. Reference, temporary and nested projections remain unsupported.

Resolution selects canonical RecordId/FieldId for construction and resolves a
field-read owner to ParameterId/LocalId. For inferred locals, existing typed
lowering finalizes field membership through that owner's actual RecordId; no
second inference engine or post-HIR spelling lookup is introduced. HIR represents
RecordConstruct { record, fields: Vec<RecordFieldInit> } and
CopyFieldRead { owner, record, field }. Canonical record.fields follows declaration
order; each construction.fields follows written initializer order. FieldId allocation
is in declaration order. Membership/duplicate checks precede completeness, then
exact typing of every initializer in written order. The construction type is that
exact record; each initializer/read result is its exact selected field range. No harness BindingId or anonymous LocalId models fresh results.

`types::check` invokes `Program::validate_record_expressions` at typed-HIR
publication before ownership. This crate-private, table-aware invariant boundary
first validates every declaration table, including unused records and fields,
with nonempty field vectors, unique IDs and spellings, reciprocal exactly-once
membership and valid named-range types. Checked indexing rejects malformed tables
without panic or repair. Expression validation then checks complete unique
construction sets, canonical owner eligibility/function membership, record/field membership,
exact result/initializer metadata and ordinary-versus-verified representation,
recursively enforcing existing conditional-value positions. It performs no
inference, ownership evaluation or generalized validation framework.

The single existing ownership checker evaluates construction children inside
its complete-expression transaction. BD-033 and BD-034 enclosing transactions
retain their original prepared-entry boundaries. A dedicated non-consuming
Copy-field-read operation checks availability/exclusive whole-record loans and
accounts for the written root use; it never records a transfer, field place,
field loan or independent provenance. Shared loans remain protected by their
existing finite/held/recurrent requirements. Construction participates in both
finite-use and recurrent-reference traversal. Reads are Copy activity in recurring
conditions; recursive construction evaluation rejects non-Copy construction there.

MIR retains these typed trees and canonical IDs without new statements or
terminators. Its structural validator descends every construction child through
all expression containers, statements and terminators, rejecting hidden conditional
values and residual verified projection representations. It does not re-prove
canonical relationships without HIR tables. Ordinary operations are not executed
or proven; the specialized verifier and literal harness path remain single-field,
with explicit eligibility gates before scalar selection. No ordinary construction
becomes a harness binding. Construction completion adds no balancing stage or
hold and never releases inherited operation holds. All entries evaluate inside
the existing expression transaction in written order, preserving BD-033/BD-034
rollback and exact loop recurrence protections.


## Contextual literal publication boundary (BED-022 / BD-037)

`types::check_value` is one bounded literal-aware check shared by ordinary value
returns, both call forms, direct assignment, write-through and record initializers.
Its exact canonical expectation comes only from existing signature/target/field
metadata. It directly compares represented i64 literals with inclusive bounds;
otherwise it invokes unchanged expression synthesis and strict exact-value checking.
It never passes an expected type into general recursive synthesis or retries it.
AST, resolution, numeric parsing and canonical tables remain unchanged.

HIR adds FormedRangeLiteral(i64), with identity stored once as ExprType::Range(id).
Raw IntegerLiteral retains its expression-only tag. The existing publication walker
validates both tags, checked canonical range lookup and inclusive formed bounds,
recursively across all containers and statements; formed verified nodes reject.
Ownership explicitly treats formed leaves as zero finite/reference uses and no
state/provenance effects. Existing transactions and operation completion still run.
MIR clones these typed leaves and checks structural discriminator/type consistency,
without range-table or bounds claims. No extra statement, owner, temporary, CFG
edge, pass, rollback engine or ownership mechanism is introduced. Specialized
verified constants and harness scalar literals remain isolated.


## Direct field-operation boundary (BED-023 / BD-038)

AST FieldAssign keeps textual root/field names and their spans, the complete RHS
and statement span. Resolution lexically resolves the root and complete RHS before
typing; it retains a ParameterId long enough to give a targeted Type diagnostic.
Typing classifies local category before mutability, selects the actual record's
canonical field and applies existing exact/contextual value checking at that field.
HIR CopyFieldAssign carries LocalId/RecordId/FieldId and all source spans. The
existing table-aware publication pass checks canonical containing-function/local
identity, mutability, owned record type, reciprocal field membership, valid range
metadata, exact RHS type and all recursive expression invariants without repair.

The existing ownership checker counts/discovers only RHS uses, never a synthetic
target use. One bounded prepared field-operation snapshot wraps ordinary expression
evaluation, expiry, non-consuming availability and whole-record mutable-conflict
checks. It neither reserves the target nor creates a loan/hold. On failure the
prepared entry is restored exactly; on success lawful RHS moves/expiry/allocation
remain committed. Enclosing transactions keep their distinct entries. The write
adds no ownership state, transfer, field availability/provenance/loan or terminal
permission. Finite/held/recurrent liveness, nested frames, stable backedges and
actual-return-only terminal permission remain unchanged.

MIR CopyFieldAssign retains the canonical IDs and full typed RHS in source order.
Its validator checks RHS named-range shape and every recursive child, not canonical
HIR table facts. No setter, reconstruction, generalized place, new terminator,
phi, second checker or fixed-point engine is introduced. Verified subtraction and
harness paths remain isolated. Ordinary mutation is checked/lowered, not executed
or proven; broader projection, mutation, invariants and destruction remain open.


## Referenced Copy-field access boundary (BED-024 / BD-039)

AST FieldAccess now retains the complete expression and a distinct field-token
span. Resolution preserves the canonical ParameterId/LocalId root plus written
field spelling/span. Existing expression typing classifies owned/shared/exclusive
roots from actual inferred types, selects the field in the canonical referent
record, and publishes CopyReferenceFieldRead { reference, record, field } with
exact Range result metadata. Owned CopyFieldRead remains a separate operation.

The existing HIR publication walker uses checked canonical tables for containing
function, exact reference-to-record root, reciprocal field membership and exact
range result, and rejects ordinary referenced reads in verified expressions.
MIR retains this typed leaf, checking Range result shape and every container;
it cannot establish HIR-table facts or loan validity without those tables/state.
No new pass, instruction, terminator, temporary or generalized place is introduced.

The bounded ownership sibling checks loop-handle condition protection, checked
reference type, availability and live lineage; it decrements finite use, invokes
ordinary expiry and returns no LoanId result. Thus copied arguments create no
receiving hold. The whole-record dereference helper still rejects Move referents.
Both finite/recurrent visitors explicitly count the reference handle. Shared and
exclusive permissions remain unchanged; holds and scoped recurrence retain loans
continuously through calls/backedges and false-exit normalization as before.

Loan.origin records only the canonical external ParameterId or first stored-borrow
LocalId (direct ephemeral borrows are unbound). Active identity remains exclusively
in Place → LoanId. Validation walks the current function's canonical initializer
lineage, checking borrow origin/owner/type/permission and same-loan alias/transfer
links. Exclusive source transfers are checked without requiring moved sources to
remain available. No second expected-LoanId map or field provenance is added.
Cloning, rollback assertions, semantic backedge comparisons and break integrity
include origin metadata. Branches retain only incoming provenance, as before.

Existing complete-expression rollback restores an earlier read's finite-use and
loan expiry after a distinct later child failure. BD-033/BD-034 enclosing boundaries
remain separate and unchanged; no leaf transaction or global hold-balancing rule
is added. Owned reads/assignment and verified/harness eligibility/proofs remain
unchanged. Broader projection, reference mutation, reborrowing and generalized
cyclic analysis remain unresolved; ordinary code is checked/lowered only.

## Exclusive referenced Copy-field assignment (BED-025 / BD-040)

The existing parser, AST and resolved FieldAssign retain the named root, field,
complete RHS and individual spans. Typing classifies the actual root type before
owned-local restrictions: exact mutable record references produce the distinct
CopyReferenceFieldAssign with Place/RecordId/FieldId, full TypedExpr and spans.
Shared and non-record references receive targeted Type diagnostics. HIR checks
canonical root/function membership, exact referent, reciprocal field/range tables,
nominal RHS type and every descendant. MIR preserves this operation and its RHS;
its structural validator has no canonical or loan tables.

The existing ownership checker adds a bounded operation transaction after
statement-list liveness and entry expiry. Read-only target authorization checks
loop eligibility, availability, exact reference type and existing live lineage.
It captures the original LoanId, consumes one finite target use without expiry,
and adds one hold to that loan before evaluating the entire RHS. Final read-only
authorization independently checks availability and canonical lineage resolving
to the same original LoanId. Only then is writing authorized; the operation
releases exactly its own hold and expires normally. No runtime field state is
modeled. Failure restores the complete prepared snapshot; success commits lawful
RHS transfers and temporary allocation. Nested calls release their own holds.

Both summary visitors include the target and every RHS occurrence. The immutable
initializer-index walker recognizes this non-declaration without adding an entry;
Rc metadata remains shared across snapshots. Existing join candidate validation,
semantic loop-edge checks and pre-discharge break gates remain intact. No second
loan map/checker, field identity in Place, projected authority, reborrow, coercion,
generalized dataflow, verifier/harness extension or CFG machinery is introduced.

## BED-026 bounded ordinary execution (BD-041)

`bounded` provides preparation limits and shared, non-refundable lineage counters.
Budget-aware lexer/parser/ownership entry points reuse existing semantic engines;
conservative preflight envelopes protect resolution, typing, checking, lowering,
and rejected representations. `mir::lower_with_origins` preserves block origins;
CFG reachability validation is iterative. `execution` owns the opaque immutable
successful-ownership artifact and exact HIR/MIR/origin correspondence validation.
Canonical FunctionId lookup accommodates verified/ordinary ID gaps. Private
runtime frames use monotonic IDs and canonical places; explicit continuations
execute only MIR flow. Storage growth, recursion, work, fuel, diagnostics, and
rendering are bounded. Typed values distinguish raw operands, ranges, records,
and references. There are no runtime loans, snapshots, or rollback. `main` routes
`run` to bounded reader preparation and execution independently of `verify_report`.
Private stopped-machine tests inspect prefix effects and unpublished destinations.
Retained verified-body operands receive the same bounded structural admission as
contracts before recursive validation and authority comparison, without proof
evaluation. Detached runtime payloads are reserved together with receiving buffer
growth. The private CLI reporting boundary preserves renderer host failures and
uses borrowed static emergency diagnostics without a second String allocation.
See `docs/BOUNDED_EXECUTION.md` for the full tool contract and host-failure boundary.

## BED-027 deterministic scope cleanup (BD-042)

BED-027 implements the frozen BD-042 architecture through a private synthetic
resource facility. Canonical typed representation distinguishes
Copy/Move, owner/reference, and resource-bearing/inert properties. A trusted
resource leaf carries canonical acquisition/release identity; recursive record
classification produces a bounded shared cleanup plan in reverse declaration
order. Ordinary Move values and references have no executable cleanup plan.

`hir::resources` uses canonical ResourceId/OperationId and FunctionId descriptors;
names never classify or dispatch resources. `Field` holds structural ValueType,
while source resolution continues to require named-range fields. Plans share
child RecordId references rather than expanding nested trees. Catalog validation
rejects cycles, foreign fields, unsupported pairings and noncanonical operation
bodies. A trusted acquire stub and owned release parameter cannot become arbitrary
callbacks.

`mir::cleanup` preserves lexical scope identity before structured HIR becomes flat
MIR control flow. Function cleanup Flow contains scopes, owner initialization
sites, exact exit sets, parameter order, protected-result metadata and origins.
Deterministic HIR/MIR/origin correspondence binds the complete artifact to opaque
approval. Dynamic scope activations separate
repeated loop iterations and retain successful-local-initialization order. Runtime
responsibility follows the actually executed ownership path and live/transferred
payload state, never the ownership checker's conservative `ConditionalMove` join
classification. Validated duplicate cleanup remains malformed even if a prior
operation would leave a slot vacant.

Return execution requires a protected result boundary: evaluate and transfer the
result, clean the callee's remaining scopes and parameters, retire the frame, then
deliver the result. Loop-body cleanup must precede header reevaluation and static
slot reuse. Pending construction must retain canonical FieldId membership and the
actual initialized field set; publication transfers those payloads into one whole
aggregate owner, while abort for future recoverable failure visits the initialized
set in reverse canonical declaration order. None of this creates source-visible
field ownership or rollback.

Cleanup operations and plans participate in opaque authority, HIR/MIR/origin
correspondence, admission limits, runtime storage accounting, and fuel. Configured
exhaustion or host release failure stops with prior release effects preserved and
cannot be reported as successful completion. A private constant-work synthetic
resource leaf and release-event trace are the first acceptable evidence mechanism;
they do not create public resource syntax or operating-system I/O.

`execution::machine::cleanup` owns the lazy private runtime state. Non-cloneable
Leaf/Aggregate payloads retain fresh acquisition identities through moves. Scope
histories provide order but own no duplicate payload. Transferred and Cleaned slot
states distinguish valid discharge from malformed vacancy. Protected outgoing,
detached, current-cleanup and work-list payloads remain accounted and retained on
terminal stops. Fixed trusted operations record Acquire/Attempt/Complete/Failure;
frame retirement is recorded only after successful cleanup. Release has no user
callback or operating-system handle.

PendingNext/PendingField continuations evaluate and commit fields in written
order. Exact initialized membership and full canonical field-set completion gate
publication. A private synthetic field-failure control aborts nested pending
containers inside-out through shared reverse-declaration plans, then stops the
tool. It neither rolls back moved sources nor unwinds ordinary scopes. Resource
temporaries are admitted only at existing local, return or pending-field boundaries;
unnamed resource call arguments reject. Public resource results reject before
acquisition. There is no new source failure system or resource syntax.

Admission bounds metadata, expression counts and type depth before recursive
authority checks and reserves the expanded frontend envelope. Runtime charges
scope operations, owner inspections, field traversal, construction and fixed
release. Prospective storage includes simultaneous detached payload and receiving
buffer growth. Lazy activation reserves State before publication. Resource-free
functions receive no cleanup Flow; an unused private catalog preserves the prior
fuel/storage path. Existing single ownership analysis, borrowing, replacement,
loop-carried Move restrictions and specialized verifier isolation remain unchanged.

Private malformed-artifact and conservation controls run through the canonical
approval boundary. The implementation ledger records milestone SHA/run evidence,
failures and repairs, predecessor identity reconciliation and final review status.

## BD-043 owned replacement

Canonical typed Assign metadata distinguishes Copy writes from owned record
replacement without names. HIR publication checks target/function membership,
mutability, owned category and exact RHS type. The single ownership checker wraps
Move replacement in a prepared-entry transaction and installs an operation-local
non-consumption restriction at canonical consume. This is neither a synthetic
loan nor a second ownership map. Final expiry/exclusivity and current-loop header
checks preserve existing borrowing and relative iteration rules.

Structured lowering emits a compound ReplaceOwned MIR operation with LocalId,
RecordId, declaring ScopeId, canonical cleanup-plan reference or inert marker,
complete RHS and replacement origin. Lexical scope metadata also binds inert
replacement targets. Functions without replacement preserve their existing
metadata boundary. Existing admission envelopes bound descriptors and lexical
vectors; opaque deterministic HIR/MIR/origin correspondence is the execution gate.

Dynamic histories identify initialization generations independently of frame/local
slot associations. Activation identities distinguish recurring static scopes.
Discharged history entries cannot clean a later occupant; a live entry must match
the slot's current generation. Frame-associated prepared contexts form a bounded
stack for recursive RHS calls. Their protected results are separate from protected
returns, pending fields and cleanup cursors. Both new and displaced payloads and
all metadata capacities participate in prospective storage accounting.

Runtime replacement evaluates, protects, reserves, detaches, discharges, then
publishes/registers. The final slot installation, association update and reserved
history append are one allocation-free transition. No source continuation runs
between them. Failures retain actual payload holders and completed release prefixes;
history, vacancy and release attempts are never counted as completed responsibility.
The executor consumes MIR authority and does not reconstruct ownership or HIR scopes.
Public resource APIs and all frozen non-goals remain excluded. Evidence and precise
CI identities are recorded in `docs/engineering/BED-028-IMPLEMENTATION-LEDGER.md`.

## BD-044 temporary custody and authority

Fallible canonical MIR metadata binds source evaluation ordinals, regions, sites,
canonical types/plans, destinations and creation/transfer/cleanup/retirement origins.
Ordinals include resource-free predecessors, while empty regions are omitted. MIR
points bind source regions across CFG blocks; conditional initializer leaves finish
the same activation. Region/site identities are separate from LocalId and Place.
Exact HIR/MIR/origin correspondence and opaque approval seal the complete protocol;
bounded validation rejects malformed tables before following their references.

The machine lazily allocates temporary state. Each dynamic activation contains
explicit Owned/Transferred/Cleaning/Cleaned custody and payload holders. Operand
handles identify owners without duplicating payloads. Combined prospective growth
is checked before detachment where feasible. Whole-call entry publishes an explicit
callee frame before incremental parameter transfer; a stopped partial entry retains
both published parameters and untransferred arguments, without executing the body.

Completion protects results before reverse sibling retirement. Return and replacement
results use their existing dedicated holders. Local/assignment/condition completion
uses activation protection until publication or selection. Pending fields receive
immediate canonical membership before child retirement. Existing cleanup cursors
retain a stopped leaf and completed prefixes. Vacancy is checked against explicit
custody and never substitutes for it. Fresh activations distinguish repeated source
regions; no host destructor implements Bowfin cleanup. Capacities and payload storage
are bounded and charged without counting non-owning handles as duplicate payloads.
Resource-free paths retain their previous accounting. The BED-029 ledger records
identity-sensitive tests, independent artifact mutations and private allocation probes.

## Frozen BED-030 explicit effects implementation

Canonical bounded sets and immutable declaration origins pass from raw contextual
clauses through collected ordinary signatures, HIR and MIR. Complete declarations
make forward and recursive edges checkable without traversing callee bodies or
computing inferred minimum contracts. Both call forms and all structural branches
contribute; strict supersets remain part of the API. Verified/harness metadata and
proof behavior are unchanged.

Trusted operation and bound-function contracts must agree exactly. Canonical
cleanup plans derive leaf release effects and memoized aggregate unions. A separate
may-custody product inside the existing successful ownership checker records
realization holders, types, kinds and origins. It does not grant move/loan permission
or use ConditionalMove, vacancy or runtime events as an authority oracle. Branch
residual custody, terminal evidence and short-circuit skip alternatives remain
distinct from usability. Return transfer precedes remaining-owner cleanup; existing
replacement generations and temporary/pending/parameter transfers retain their order.

Successful checker output is attached to MIR and retained immutably by opaque
approval. Bounded validation checks each independent contract/summary/site/origin
copy, exact deterministic lowering and compilation identity. The machine consumes
validated custody and plans without rerunning ownership or effect analysis. Fixed
effect identities and admitted supersets do not generate runtime events.

Only Ordinary exists in production. Test-only Approved Abort templates admit the
whole pending stack's initialized canonical membership and derived effects before
approval, binding exact frame/pending identities, stack depth, construction and
in-progress field sites and origins. Runtime checks every binding before attempting
any release. Primitive fixtures are separately sealed and make no unchanged-source
claim. Ordinary traps/limits/Integrity/Host stops add no cleanup edge. Retained
metadata, recursive derivations, diagnostics and validation fit conservative checked
work/storage envelopes; detected fallible allocation failures remain Host.

The implementation ledger and identity manifest record GitHub-executed milestones.
Specification freeze does not constitute independent implementation acceptance.

## Frozen BED-031 Result identities and custody

Result syntax uses a bounded flat postorder type graph, interned as ordered
`ResultTypeId` payload pairs. The canonical two-tag enum has no Err alternative;
both source failure tokens normalize before AST publication. Constructors and
statement matches have distinct AST/resolved/HIR forms. Result is a ValueType,
never a RecordId or ReferentType. Complete tables, including unused rows, reject
invalid IDs, duplicate pairs, cycles, foreign payload categories and depth violations.

The existing ownership checker carries pending outcome facts independently of
availability and cleanup custody. Its successful analysis retains an append-only
bounded witness trail across branch snapshots. Transfer destinations, constructor
and pattern roles, bindings, origins and validated exits are sealed with the
original HIR. Opaque approval compares independently retained witness copies and
exact HIR/MIR/origin products; execution reruns neither ownership nor exhaustiveness.

MIR has a canonical Result dispatch terminator and explicit selected-binding
transfer, with match/pattern origins and normal lexical arm scopes/exits. Temporary
sites independently retain Result-plan identities even for cleanup-inert wrappers.
Variant plans reference shared record or nested Result plans in ordered roles;
resource-bearingness and unioned effects are separate from nested plan identity.

A lazy, flat machine cell table owns actual active Result payloads. Non-cloneable
operand/slot handles refer to cells and never duplicate payloads. Cells stage before
payload evaluation, retain constructor correspondence to approved MIR, and pass
through active, match-protected, consumed or delivered states. Nested cells have
exact parent links. Reservation precedes detach; dispatch protection precedes
binding reservation and transfer. Calls and protected returns retain those cells
through all fallible transitions. Cleanup-inert Results do not activate the resource
catalog. Ordinary range-field records retain their public representation in Result
regions through separate inert field staging. The public output is a bounded flat
active chain with one Bool/range/record leaf; private resource results cannot cross
the public execution boundary. Only sealed primitive/protocol authority traverses
a whole wrapper for active-payload cleanup.

## BED-032 contextual local propagation

AST/resolved Let retain optional exact phrase/binding spans. Typed Let retains an
approved propagation mode binding inner/outer ResultTypeIds, exact failure type
and source origins; local type is the inner success type. No expected Result type
is supplied to initializer synthesis. Table-aware publication and bounded shape
validation independently check function/local/type/origin relationships.

The existing ownership checker consumes the input through an explicit propagation
transfer witness. A failure-state snapshot removes bypassed suffix finite uses,
checks all remaining outcomes/holds and records existing return cleanup effects.
Only success initializes the original local and flows into the suffix; nested
Result obligations remain independent. No synthetic arm reduction or second checker
is used. Runtime tags never establish static authority.

Structured lowering emits Propagate and ForwardError terminators plus the existing
selected binding transfer. Success continues the same lexical scope; only Error
has ordinary function-return exits. Canonical declaring ScopeIds are derived while
structured HIR is present even when no runtime cleanup Flow is needed. No owning
activation is invented. Replacement metadata keeps the target's original scope.

The flat cell table retains incoming protection and active payloads. Forwarded
Error cells bind their type/tag/origin to the approved ForwardError block; ordinary
constructors retain their approved temporary-site identity. A bounded tagged scalar
keeps this distinction in the predecessor cell footprint. Receiving capacities
are reserved together before detachment; commit allocates nothing. Existing
protected return/cleanup machinery prevents early delivery and retains stopped
payloads. Inert paths activate no resource catalog.

Bounded MIR validation carries typed PlanError internally, including traversal
and cleanup-validation allocation failures. The legacy read-only validate API
only adapts diagnostics; execution approval uses variants rather than matching
diagnostic text. Exact deterministic HIR/MIR/origin comparison and immutable
outcome/effect witnesses seal both branches and generated transfer sites.

## BED-033 selected trusted catalog and persistent invocation custody

Immutable Selection enters collection before signatures. Compilation-scoped
ResourceTypeId and TrustedOperationId are distinct from RecordId and FunctionId;
imported named ranges retain canonical RangeIds and Catalog declaration origins.
Every row, including unused/denied rows, is bounded and validated. Typed TrustedCall
and trusted-argument destinations reuse the existing argument, hold, ownership,
outcome, custody and temporary systems. CleanupType distinguishes Record/Resource;
Result variant plans carry Resource children without changing private whole-wrapper
cleanup authority. Resource slot moves use the existing Transferred state.

Approved runtime metadata is independent of HIR: exact target/compiler/catalog/
profile/configuration, grants, bindings, contracts, roots, complete structural edges
and MIR invocation/site/receiver/origin correspondence are retained and independently
compared by opaque approval. Trusted runtime meaning uses only those contracts.
There are no synthetic ordinary stubs, fabricated ParameterIds or fallback dispatch.

ExecutionSession is the sole production installer of binding state. Its inline
stopped/complete status retains the machine through reporting. Before invocation,
combined capacities cover the receipt payload, fresh token slot, Result cell and
provenance, operand, immediate temporary registration, audit prefix and stopped
diagnostics. The binding constructs its non-Clone Sensor directly in the owned
receipt. Malformed or stopped publication keeps it there; allocation-free commit
moves it into the live token table and publishes the sole non-Copy logical ticket.
Trusted Result authority records actual invocation provenance. History is never
physical ownership. Fixed release consumes exactly the matched token only on
success. Every later stop retains protected values and unfinished cleanup; repeated
advance borrows the recorded outcome without execution or implicit cleanup.

A static bounded authority renderer follows sealed structural edges from explicit
roots using bounded visited storage, independent of direct-effect filtering or
fixture trace. It records complete contracts/configuration/grants/bindings, source
and catalog origins, acquisition/release pairing and possible cleanup sites. Audit
scratch/output and retained session storage share one conservative envelope;
rendering is read-only and failure cannot alter a recorded outcome. Existing bounded
reader/preparation and CLI diagnostics are reused by explicit profile run/audit.
Private cfg(test) controls act only at genuine allocation/receipt/cleanup/reporting
boundaries. No second ownership/effect checker, host registry, FFI, device proof or
BED-034 fallible completion was introduced by that baseline. Independent
review/acceptance remains separate. The later candidate is described below.

## BED-034 completion implementation candidate

The frozen Draft 2 candidate adds one owned completion row and singleton nominal
FinishReceipt status range to the same closed catalog. Existing row origins remain
stable. The successful ownership consume gate emits a distinct invocation holder
in its existing cleanup-effect evidence; it is not a source Place or move permission.
MIR-associated approval independently seals current input, completion holder,
protected-result role, exact release, effects/grants, origins, destinations, phases
and event identities. No checker is duplicated and runtime dispatch is ID-bound.

One pre-dispatch envelope reserves provenance, Result and temporary capacities,
events, fixed cleanup work and actual diagnostic backing storage together. The
current owner may be a protected helper return and is never restored to a historical
local. A pending protocol retains the logical ticket while the physical Sensor
stays in the session token table. The inert ordinary outcome occupies a separate
CompletionProtected Result cell. Fixed release uses the existing cleanup cursor;
only its success enables the delivery continuation. Session realm, fresh invocation,
frame and production identities are validated independently of payload shape.

Terminal stops retain exact prefix, current responsibility and protected outcome;
repeat advance cannot run any continuation. Source-formed receipt data is not
execution evidence. Static rooted audit discloses possible finish and release-only
sites and independent effect derivations without running a binding. Actions-only
conformance, malformed controls and measured finite-fixture bounds are recorded
in the BED-034 engineering guide, ledger and manifest. This is an implementation
candidate awaiting independent review and acceptance, not a new proof guarantee.

## Accepted BED-035 Stage I implementation

Frozen Issue #84 Draft 2 and directive 6296001700 govern the implementation
developed from baseline 5f28977a42a7df1817e5cac46b3f6660a38435d3.
Greg accepted the independently reviewed Stage I head
837fc99e2bbb241077d4b65b32a4103b6bf99362, and human-merged PR #85 as
c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717 with tree
a76e2ab68c8d2db094f1005906bdc24b6a9b8795.
`execution::contract::Contract` borrows the same checked executable, consumes its
canonical/opaque gates, indexes exact MIR operands/sites and builds bounded
structural ordinary closure. `Approval` seals semantic/accounting/cut/observation/
replay schemas and the version-one compatibility schedule. Accounted predecessor
runtime structs/enums are not expanded for conformance instrumentation.

`execution::conformance::Session` retains the actual ordinary or selected-profile
interpreter session. Pre-reserved flat Copy evidence records contain metadata,
never owning Sensor tokens, recursive heap payloads or release authority. Actual
slots/continuations/counters, aliases, Result parents and active payloads,
initialization/generation/scope associations, pending/replacement/temporary custody,
receipts/provenance/completion, protected outcomes and event histories are observed
read-only. Transit markers describe primary local custody without creating it.
Failure of capture leaves the primary execution intact. `completeness()` reports
capture status only; independent validation and comparison establish evidence.

`conformance/compare.rs` validates both full traces and final censuses before one
bounded forward/inverse correspondence. Exact static IDs are not alpha-renamed.
Dynamic IDs preserve ancestry, alias equality, sole holders, generations, resource
birth and release/delivery association. Missing coverage or budget yields
Inconclusive, never truncated equality. `conformance/replay.rs` is a private
conformance context, not a public source/runtime input interface. It binds the
same approval and canonical site inventory, retains logical activation ancestry
and per-site occurrences, and supplies only admitted selected binding responses.
Its metadata is pre-admitted separately; no fallible observer allocation is inserted
inside an atomic handoff. Positive logical receiving growth has an abstract Host
cut after its storage gate; a physical allocator callback is not its identity.
Open/read keep a shorter phase set around atomic publication. Completion preserves
consumed input, committed prefix, protected Result, fixed release and non-delivery
as distinct existing phases. Repeated terminal advancement consumes nothing.

`conformance/authority.rs` adds bounded typed rooted static content from sealed
facts, independently of runtime events and text. Full catalog/grants, structural
edges, operations, plans, effects and possible cleanup remain inspectable. Its
bounded summary renderer is an evidence presentation and never replaces primary
execution. The existing public `audit::render` path and predecessor byte/work/storage
boundaries remain unchanged. Logical execution, observation/comparison, replay,
structured audit, renderer, physical resource and provenance domains stay distinct.

M5 is preserved at e31f6104d8222ee201bc392a3daf87bcb03734ab, tree
34466f3032475e6b03931f59e61e712fab3b54e2, with every applicable Actions workflow
successful before M6. Independent full bool/fuel traces, protocol expectations,
semantic-defect controls, malformed independence and finite cut enumeration are
recorded in the engineering ledger/manifest. Exact BED-034/033/032 predecessors,
opaque-authority doctest and affine cases are reconciled in debug/release Actions.
No local Rust execution is used. No native implementation, physical optimized
execution or portable artifact identity is claimed; B35-P10 remains Stage N.

### Accepted BED-035 observation-validation repairs

Sarah's first Stage I review identified K35-I01 and K35-I02. The first repair
resolves source-global FunctionIds through the actual ordinary MIR/origin row.
Stored reference bits retain exact canonical initializer, alias/transfer and
caller-argument identity after last use without implying a current loan or forcing
an owner to remain available after lawful Move. Reached access and held operands
still require available referents. Bounded initialization/Move history distinguishes
repeated borrow-site executions and rejects duplicate or resurrected exclusive
representations. This metadata is not a lifetime engine; the existing ownership
checker remains the only loan authority.

Sarah's next confirmation resolved I01 and retained the I02 lawful positives while
identifying I03 and I04. The final accepted repair binds queued operands to approved
producer transitions and preserves each validated queue until reached consumption.
Canonical cursor, parent, operand-order and continuation relationships supply
authority independently of two observation fragments agreeing. Private controls
separately discriminate Seal, Control, Operand, Queue, Duplicate, Resurrection
and Access; the prior seal-masked evidence claims are explicitly withdrawn.

Sarah's final focused confirmation resolves K35-I01 through K35-I04, assigns no
K35-I05, and finds the exact head ready for owner acceptance. Complete debug and
release profiles each pass 2,112 identities with zero predecessor migrations.
The repair changes no frozen semantics, public API, source ownership/lifetime rule,
primary interpreter behavior or Stage N boundary. Post-merge build, tests, verify
and docs workflows all pass on c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717.
