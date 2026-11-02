# Bowfin AI Context Bundle

> Generated from the canonical files in `docs/ai/`.
>
> A fresh contributor or AI model should read this entire file before making major Bowfin design proposals.
>
> **Do not reinterpret Accepted decisions unless explicitly asked.**


---

<!-- BEGIN BOWFIN_CONTEXT.md -->

# Bowfin Context

> **Read this first. You are joining an existing language-design project. Do not reinterpret settled decisions unless explicitly asked.**

## Project identity

**Bowfin** is a systems programming language in development.

Adopted primary domain: `bowfin-lang.com` (external cutover deferred)
Operational legacy public sites: `klyxr.com`, `klyxr.online` (retained endpoints)
Primary source extension: `.bow` on migration staging; explicit `.klx` input retained<br>
CLI executable: `bowfin`; retained compatibility command: `klyxr`

Primary public line:

> **Systems programming you can reason about.**

Core philosophy:

> **Own what moves. Constrain what exists. Prove what matters. Make every escape hatch visible.**

Canonical public summary:

> **Bowfin is a systems programming language that treats memory safety as the foundation, then lets programmers express and verify more of the rules that make their software correct.**

## Why Bowfin exists

Bowfin begins from the premise that **memory safety is the beginning of the argument, not the end**.

Systems software also depends on rules such as:

- this value must remain within a valid range;
- this function may not allocate;
- this task must not block;
- this state transition is legal only under defined conditions;
- this API must preserve an invariant;
- this routine must not panic;
- this external assumption is trusted rather than proven.

Those facts often live in comments, tests, conventions, wrapper types, code review, runtime assertions, external tools, or institutional knowledge.

Bowfin's core proposition is:

> **More of those facts should be part of the program itself.**

Once they are part of the program, the compiler and verifier should be able to reason about them.

## Documentation philosophy

> **Documentation is part of the program's human-facing contract. Code and
> documentation are reviewed together, and disagreement between them is a
> defect.**

Bowfin does not assume that code is self-explanatory. Public declarations,
examples, and reference applications should explain purpose, domain meaning,
effects, authority, failures, lifecycle obligations, important policy,
assumptions, bounds, and non-guarantees in clear human language.

The toolchain should eventually preserve structured documentation comments,
generate reference material, compile documentation examples, and validate
mechanically checkable claims against canonical compiler facts. Documentation
metadata may never grant authority, change semantics, or create proof evidence
by assertion. Exact documentation-comment syntax remains open; the current
prototype accepts ordinary `//` line comments only.

Bowfin-owned public examples prefer implemented explicit aliases such as
`function`, `mutable`, and `Error`. Compact spellings remain equal language
aliases under BD-014. See [the documentation policy](../DOCUMENTATION_POLICY.md).

## Current implementation status

An executable compiler prototype, written in Rust, supports ordinary
value functions with statement conditionals, ownership-stable while loops,
iteration-local Move values, and ordinary multiple-field named-range records.
The specialized signed-integer verified-state/harness subset remains single-field.
Explicit resolution assigns compilation-local function, parameter, local, type,
field, and top-level binding IDs. Expression and value-flow type checking produce
canonical typed HIR. Public HIR inspection remains read-only; names/spans are
diagnostic metadata.

BED-004 introduced ordinary Bool/range value functions, immutable locals, calls,
and explicit `return expression;` (BD-019). BED-005 now permits existing records
by value in ordinary signatures, call results, locals, and returns. Ordinary and
verified functions retain one function namespace and ID table. Bare integer locals remain unsupported; BD-037 now permits direct literal
formation at independently established named-range value boundaries.

The dedicated ownership phase runs after type checking. Bool and named ranges
are `Copy`; records are non-`Copy` and move through bindings, by-value arguments,
and returns (BD-020). ParameterId/LocalId state is per function; a moved place
cannot be reused. The compiler does not silently clone. Record fields do not make
records Copy. BED-006 adds explicit shared (`&T`) and exclusive (`&mut T`)
whole-value borrowing. Shared references are Copy; mutable references move.
BED-006 introduced `let mut` / `let mutable` for exclusive borrowing without
reassignment; BD-026 later adds narrow Copy-safe local reassignment. Immutable reference locals carry private loan provenance. Loans
end after the last possible use of derived handles on each path; potentially
available handles at a join can retain a possibly active loan. Call arguments
remain held until the receiving call completes, including nested arguments.
Availability and loans are separate ID-based state within the same ownership phase.
Borrowing/cloning/reborrowing/coercion are never implicit. Ordinary owned parameters
remain immutable. Reference returns, nested references, explicit lifetimes,
auto-dereference, generalized reference projection beyond BD-039/BD-040, and partial
borrowing/moves remain unsupported (BD-021 / OQ-024). BED-027 implements BD-042's
deterministic ownership-attached cleanup through private synthetic resource leaves,
explicit lexical MIR metadata and bounded runtime release. Public source records
still have named-range fields; no resource registration or production resource API
is exposed.

BED-007 adds explicit `*reference` for directly named ordinary reference parameters
and locals. Copy referents (Bool/ranges) may be read through shared or mutable
references without consuming the handle. `*reference = expression;` requires an
exclusive mutable reference and an exact Copy referent/RHS type. Non-Copy move-out
and replacement are Ownership errors; no destruction or displaced-value semantics
are implied. Reads/writes count for last use; writes hold the loan throughout the
RHS. Copied dereference arguments do not call-hold a reference, while reference
arguments retain BED-006 holds. General owner replacement, field mutation beyond BD-038/BD-040, auto-deref,
reborrowing, and broader mutation remain unsupported (BD-022 / OQ-025).

BED-008 adds statement-only `if` / optional `else` with Bool conditions, nesting,
lexical child scopes, no visible-name shadowing, and distinct sibling LocalIds.
The condition's ownership effects occur once before independent branch snapshots.
An outer Move place remains available at the join only if every incoming path
retains it. BED-009 replaces whole-conditional loan holds with continuation-aware,
path-sensitive last-use analysis (BD-024): each branch inherits its own uses plus
the shared suffix, with edge-specific expiry after condition evaluation. Possible
active incoming loans merge conservatively; a conditionally moved handle cannot
erase a loan needed on another incoming path. Loans never reactivate along the
same path. Branch-local handles/loans do not escape. Straight-line last-use, call holds, and
write holds remain intact. Ordinary typed HIR lowers deterministically to read-only
MIR basic-block tables with explicit Branch/Goto/Return terminators. BED-010 adds initializer-only value conditionals (BD-025): mandatory else, one
expression per branch, nested complete branch results, Bool conditions, and exact
owned Bool/range/record types. Reference results and contextual integer-literal
materialization remain forbidden. Each Move leaf transfers into the same source
LocalId; moved sources retain BD-023 conditional unavailability. BD-024 path-specific
loan expiry and holds apply. MIR leaves initialize the canonical destination and
Goto a common join; no IfValue remains hidden in MIR expressions. There is no
phi, block parameter, SSA, general temporary/result slot, definite-assignment
solver, cleanup, or MIR-based ownership solver. BD-033 later adds structured early return. BED-012 later adds narrow statement loops. Conditional
values in returns/call arguments/general expression positions remain open under
OQ-022/OQ-024/OQ-026.

BED-011 adds direct `local = expression;` statements for mutable owned Copy Bool
and named-range locals (BD-026). Parameters, immutable locals, and reference locals
are Type errors; exact-typed non-Copy record replacement is an Ownership error.
RHS typing is exact and nominal; BD-037 forms direct literals at established
named-range targets only. The RHS evaluates
before mutation, completes operation holds and last-use expiry, then the write
checks that no shared/exclusive loan of the target remains active. Self/repeated
assignment is valid; BD-024 sibling-path and post-join liveness remain unchanged.
AST/HIR/MIR distinguish Assign from Let and DerefAssign, preserving the existing
LocalId. BED-011 MIR was acyclic; BED-012 introduces real backedges. Assignment expressions, compound/chained/projected
assignment, conditional-value RHSs, reference replacement, non-Copy displacement,
destruction, and general dataflow remain unsupported.

BED-012 adds pre-test statement `while` with exact Bool conditions, nested lexical
body scopes, and ordinary if/while composition (BD-027). BED-012 loop activity was owned
Copy Bool/range only: existing direct assignment and conditional initialization
compose. Reference creation/use/dereference/write-through and non-Copy activity
were rejected in conditions and bodies pending cyclic ownership/lifetime analysis.
BED-013 subsequently permits iteration-local Move activity; BED-014 also permits
iteration-local borrowing in bodies, as described below.
One permitted iteration must preserve the header's Move availability, reference
provenance and active loans after body locals are scoped away. Outside owners and
references may remain untouched; outside live loans still constrain Copy owner
reads/writes, while dead pre-loop loans stay dead. There is no fixed-point solver.
MIR uses Goto → header Branch → body/exit, with body Goto back to header. Its
validator now accepts cycles with visited tracking; structural validity does not
prove termination. Both constant-condition edges remain. For and labeled loop transfers (BD-031/BD-032 settle unlabeled continue/break),
loop values, verified loops, and general cyclic ownership/loans
remain unresolved. Ordinary loops are not executed by `check`/`verify` or proven; BD-041 `run` executes their MIR.

BED-013 permits iteration-local Move ownership in ordinary while bodies (BD-028):
fresh record call results, transfer chains, by-value consumption, and existing
acyclic branch joins. Every loop snapshots its own pre-existing Move place set;
outer-loop locals are pre-existing at an inner header and cannot move there.
Body-local availability evolves normally and is scoped away before comparing
remaining header/backedge state. Pre-existing owners cannot transfer in the loop,
non-Copy conditions remain forbidden, BED-014 refines the body-reference boundary, and non-Copy
replacement remains unresolved. Lexical cleanup is not runtime destruction.
No syntax, HIR/MIR representation, second ownership engine, or fixed-point solver
is added by BED-013. BD-035 later adds ordinary record construction; BD-041 later adds bounded MIR execution.

Ordinary functions are parsed, resolved, type-checked, and ownership-checked,
including loan and borrowed-access legality, and lowered to ordinary MIR. `check`/`verify` do not execute or prove ordinary functions; BD-041 `run` executes them through bounded preparation. A range result type does not establish that the
produced value meets its bounds. Plain `fn` does not yet implement the complete
future `safe` assurance model. Forward and recursive calls remain structurally
valid without a termination claim. Diagnostic traversal order does not settle
runtime evaluation order.

The verifier continues to recognize the original battery-contract structure by
IDs and rejects richer well-typed expressions inside that verified path with a
verifier-support diagnostic. It ignores ordinary functions as proof targets.
Its affine proof establishes subtraction safety, field ranges, and postconditions
for every admissible input of the supported structure, and tracks state between
literal-argument harness calls. Calls across function kinds are rejected; OQ-019,
OQ-021 through OQ-026 retain the broader assurance, arithmetic, function,
Copy/destruction, advanced borrowing, and general mutation questions.
General cyclic ownership/lifetimes beyond iteration-local moves/borrowing and stable carried access, advanced effects beyond frozen BED-030, runtime
contracts, VIR, MIR backend/verification lowering, SMT integration,
and machine-code generation remain unimplemented. Read `compiler/README.md`
before making claims about what the executable establishes.

## Intellectual lineage

Bowfin deliberately combines lessons from **Rust** and **Ada/SPARK**, while aiming to become its own coherent language.

### Rust contribution

Rust demonstrated that ownership and borrowing can make strong memory-safety guarantees practical in a high-performance systems language.

Bowfin takes ownership as a foundational model.

### Ada/SPARK contribution

Ada/SPARK demonstrated that constrained types, contracts, static analysis, invariants, and formal proof can be practical tools for real systems engineering.

Bowfin carries that assurance culture into an ownership-based language and contemporary developer workflow.

### Bowfin's center of gravity

A useful public shorthand is:

> **Rust makes memory safety a core language guarantee. Bowfin takes that as the starting point and extends the same compile-time philosophy to contracts, system effects, domain constraints, trust boundaries, and formal verification.**

And:

> **SPARK brings formal methods to Ada. Bowfin is designed to make provable systems programming feel native to the ownership generation.**

These are explanatory simplifications, not claims that Rust or SPARK are incapable of adjacent features.

## Assurance model

Bowfin has three escalating assurance levels:

### `safe`

Ordinary language guarantees such as ownership, initialization, reference validity, bounded access, race restrictions, and deterministic destruction.

### `checked`

Additional contracts, ranges, invariants, or related rules are enforced dynamically where appropriate.

### `verified`

Declared proof obligations must be established statically by the verifier.

The same language is used across all three levels.

Key principle:

> **Verification is not a second language you graduate into. It is an assurance level you apply where it earns its cost.**

## Core language direction

Bowfin currently includes or intends to include:

- ownership and move semantics;
- immutable and mutable borrowing;
- deterministic destruction;
- no required garbage collector;
- distinct range/domain types;
- units and dimension-aware types;
- records with invariants;
- `requires` / `ensures` contracts;
- `old(...)` in postconditions;
- quantification for verification;
- Rust-like algebraic enums and exhaustive `match`;
- `Option<T>` rather than universal null;
- `Result<T, E>` and `?`-style error propagation;
- traits/generics with an explicit static-vs-dynamic dispatch distinction;
- explicit effects such as heap allocation, blocking, I/O, time, randomness, filesystem, networking, and unsafe behavior;
- auditable `safe`, `trusted`, and `unsafe` boundaries;
- embedded and realtime profiles;
- explicit data representation for hardware/protocol work;
- a Cargo-like unified toolchain.

## Public trust model

Bowfin must be precise about what a guarantee means.

A verified function does **not** mean:

> “This program is correct.”

It means:

> **The stated properties were proven under the stated assumptions.**

Bowfin should make it easy to distinguish:

- what the compiler knows;
- what the verifier proved;
- what the programmer trusted;
- where `unsafe` exists;
- and what remains unknown.

Do not market Bowfin as eliminating bugs, proving arbitrary global correctness, or making testing unnecessary.

## Flagship examples

### Application-level correctness

```bowfin
type Percent = range 0..100;

record Battery {
    charge: Percent
}

verified function consume(
    battery: &mutable Battery,
    amount: Percent
)
    requires amount <= battery.charge
    ensures battery.charge == old(battery.charge) - amount
{
    battery.charge -= amount;
}
```

A call that attempts to consume `90` from a battery at `80` charge is memory-safe but contract-invalid.

That distinction is central to Bowfin.

### Effects

Frozen BED-030 Draft 2's first phase uses ordinary explicit upper bounds:

```bowfin
function observe() -> bool effects clock, io { return true; }
function demo() -> bool effects io, clock { return observe(); }
```

The complete declaration is caller-visible even when its body does not perform an
effect. Absence means empty. All eight canonical identities are independent;
`unsafe` adds no safety permission. Derived cleanup contracts enter only where the
existing successful ownership checker produces eligible custody evidence. A move
transfers that obligation; ordinary terminal stops add no unwind. This branch's
implementation awaits independent exact-head review and Greg's acceptance.

The following remains illustrative future policy syntax, outside BED-030:

```bowfin
#[deny_effects(heap, blocking)]
function control_loop(...)
    effects clock
{
    estimate_state(...);
}
```

A transitive allocation should be diagnosable through the call graph.

The rule should be compiler-visible rather than merely documented.

## Brand system

Current locked brand direction:

- name: **Bowfin**
- prose casing: `Bowfin`
- canonical logo: Greg's supplied fish-and-`BOWFIN` artwork at `assets/bowfin-logo.jpeg`, preserving the exact original JPEG bytes (1536 × 1536 pixels, SHA-256 `3fc1f03156b6415c652220c7b405ff7df293b65b26c31d0836878d2547b17323`)
- wordmark: white `BOWFIN`, preserving the supplied character and proportions
- canonical CLI casing: `bowfin`; `klyxr` remains the compatibility command
- adopted repository destination: `gregkrsak/bowfin`; operational repository: `gregkrsak/klyxr` until authorized cutover
- desktop and mobile derived layouts may adapt placement, scale and composition while preserving the supplied fish design and proportions
- source extension: `.bow` on migration staging; explicit `.klx` input retained
- visual mode: dark-first
- primary aesthetic: premium developer tooling, precise, modern, non-hype
- palette direction: dark/obsidian with cyan, blue, violet and the canonical fish's amber-gold accents
- tone: exact, serious, plainspoken, engineer-respecting

## AI collaboration rules

When assisting with Bowfin:

1. Preserve accepted language decisions unless explicitly asked to revisit them.
2. Distinguish **accepted**, **current direction**, **proposed**, and **open**.
3. Do not silently turn a tentative idea into a language guarantee.
4. Do not overclaim verification.
5. Prefer coherent semantics over novelty.
6. Prefer diagnostics and tooling that explain *why* a rule exists.
7. When comparing Bowfin with Rust or Ada/SPARK, be respectful and technically defensible.
8. Treat the repository, not any one conversation, as canonical project memory.


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


### BED-018 / BD-033 — Structured early return

Explicit ordinary return may end any statement path, including branches and
while bodies. Completeness means no structural closing-brace fallthrough after
loop transfers are consumed; while always retains its condition-false edge,
without constant pruning or termination claims. Returns terminate lexical blocks;
parsed and public ASTs obey the same structural checks. Exact operand typing
remains before ownership.

The finite continuation before return is operand uses only, excluding every
bypassed suffix. Current/enclosing recurrent obligations remain active throughout
actual operand evaluation. A scoped private permission bypasses only pre-existing
owned non-reference Move restrictions within the return-expression tree; it never
relaxes preceding operations, availability, loans, provenance, references or holds.
The return transaction snapshots after finite-summary installation/expiry and
before permission; all operand/final-validation failures restore that exact entry.
Permission restores on both outcomes. Successful return state has no local
successor normalization or merge. Mixed return/break/continue retains actual
surviving-path state and canonical loop false exits.

MIR lowers whole bodies with existing Return terminators; no synthetic join/final
return or MIR-validator semantic changes. No function-exit ownership join, second
checker, generalized CFG/fixed points, SSA/phi, reference returns, reborrowing or
runtime behavior is added. The verified proof path remains unchanged.


### BED-019 / BD-034 — No-value ordinary functions and call statements

Implemented from frozen Issue #39 baseline aaca52a63a40b252287935b3c7416e26f4ea50d9.
Omitting `-> Type` explicitly declares NoValue, not inference or a unit/void
expression type. NoValue functions permit `return;` or validated top-level
completion. Value functions preserve BD-033 exact returns/structural completeness.
Only named NoValue calls may be standalone call statements; value discard and
NoValue expressions are rejected. AST/resolution/HIR retain canonical IDs and
separate statement forms; signature metadata uses an optional result payload.

Statement calls have whole-operation prepared-entry rollback, including all
arguments, holds, release/expiry and final validation (B19-01). Both finite and
recurrent-reference traversal include arguments. Bare return has empty finite
continuation, pre-snapshot expiry, no terminal Move permission, balanced holds
and exact prepared-state rollback. Recurrence and loop rules remain authoritative;
completion performs no repair, cleanup or synthetic function-exit join.
MIR has honest CallNoValue/ReturnNoValue forms and exhaustive argument validation.
The single ownership checker remains above MIR. Verified functions/harness/proofs
remain unchanged; no destruction, execution, unit/void, generalized dataflow or
new lifetime semantics are implemented.


### BED-020 / BD-035 — Ordinary record construction and Copy-field reads

Frozen Issue #41 Draft 4 is implemented from baseline
02eb5364927ee9d4325fc9d4fdbe05623e3ce0d3, including K20-S01–S03,
K20-A01–A07 and K20-F01–F04. The existing one named-range-field record schema
is unchanged. `Battery { charge: expression }` produces a fresh owned Move
record from an exactly typed initializer. Bare integers do not implicitly become
named ranges, and construction proves no bounds/invariants or runtime behavior.
Canonical RecordId/FieldId selection and typing precede ownership. Construction
is an expression child in all applicable visitors and inherits complete-expression,
BD-033 return and BD-034 whole-call transactions; it creates no harness binding,
anonymous public local, hidden loan/provenance or allocator effect.

`owned.charge` reads an exact Copy range from a direct named owned parameter/local,
identified by ParameterId/LocalId. Availability and exclusive whole-record loans
are checked without consuming the owner. Shared loans remain compatible and
protected; no field loan, projected place, provenance or partial availability is
created. Reads may occur in while conditions and stable loop bodies; construction
anywhere beneath a recurring condition is rejected. Reference/temporary/nested
roots, field borrowing, mutation beyond BD-038 and broader field declaration types remain excluded.

Typed-HIR publication explicitly validates table relationships and existing
conditional positions before ownership. MIR recursively validates construction
children and excludes hidden conditionals/residual verified projections without
reconstructing canonical tables. Verified battery/harness/proof behavior is unchanged.
No generalized ownership solver, execution, allocation, destruction or broader
nominal/literal semantics is introduced.


### BED-021 / BD-036 — Complete named multiple-field construction

Ordinary records now declare one or more unique named-range fields. Canonical
FieldIds follow declaration order; complete named construction retains written
initializer order independently, including reversed order. Initializers require
exact nominal types and are evaluated in written order within the existing
complete-expression transaction. Later failure restores the entire prepared entry
and publishes no destination. Completion preserves inherited holds, recurrence and
permissions; it introduces no separate balancing stage or ownership engine.

Every declared field supports non-consuming direct named-owner Copy reads under
whole-record availability and loans. Records remain Move; there are no field
owners, partial availability, field borrowing, mutation beyond BD-038 or reference projections.
The typed-HIR boundary checks all declaration tables, including unused malformed
records, before ordered construction children. MIR preserves and recursively
checks all children. Explicit gates keep verified state/harness support single-field.
Ordinary construction and reads are transparent data primitives, not generated
factory/accessor code. Visibility, constructibility, invariants, broader field
categories, layout, destruction, execution and generalized cyclic analysis remain
unresolved. See BD-036 and compiler/README.md for the implemented boundary.


### BED-022 / BD-037 — Bounded contextual literal formation

Direct signed integer literal leaves can form exactly the canonical range required
independently by ordinary returns, either ordinary call form, established-local
assignment, established-referent write-through and record fields. Inclusive bounds
are checked during typing. Typed HIR distinguishes FormedRangeLiteral from raw
IntegerLiteral; checked table-aware validation precedes ownership. MIR preserves
and checks leaf/type structure only. The formed Copy leaf has no ownership effects.
No expectation enters recursive synthesis or conditional sibling inference; raw
operator constants, unannotated literal-local rejection, nominal identity and
verified/harness behavior are unchanged. Arithmetic bounds proofs, general
conversions/inference, annotations and broader numeric semantics remain open.


### BED-023: direct Copy-field assignment

BD-038 permits `local.field = expression;` only through a directly named mutable
owned ordinary record local and the field's exact named-range type. Contextual
literals use BD-037. The RHS completes without target reservation, then ordinary
expiry and whole-record availability/shared-or-exclusive-loan checks precede the
write. A dedicated prepared field-operation transaction restores all checker state
on failure while committing lawful RHS effects on success. The write preserves
one whole owner; no field owner/loan/provenance, partial move, replacement or
terminal permission is added. Branch/loop/return/call boundaries remain unchanged.
AST/HIR/MIR have genuine field-write representations and recursive validation.
Verified/harness mutation remains specialized and single-field; ordinary field
writes are checked/lowered, not executed or proven. Reference/temporary/nested
projection, field borrowing, invariants, visibility, destruction and generalized
mutation/cyclic analysis remain open.


### BED-024: direct Copy-field reads through named references

BD-039 implements `view.field` for directly named ordinary `&Record` / `&mut Record`
parameters and inferred locals. It copies the exact declared named-range field
through the existing whole-record loan without moving handle/record, reborrowing,
converting exclusive permission, or creating field-level state. Typed HIR/MIR
explicitly distinguish referenced reads from owned reads. Canonical table checks
and distinct field-token spans precede ownership; MIR validates structural facts.

Reads count for finite use and scoped recurrence. Final use may expire a loan;
later alias/handle uses, receiving holds and recurrent obligations retain it.
Existing expression/return/call transactions restore even prefix-expired loans
on later failure. Canonical origin/initializer-lineage integrity validates external
parameters, shared aliases and exclusive transfers using the single handle→loan
mapping. BD-040 subsequently permits writes through named exclusive record
references. Reference-backed while conditions, shared-reference writes, field borrowing, temporary/nested/dereference projection and whole-record move-out
remain unsupported. Ordinary functions remain unexecuted/unproven; verified battery
and single-field harness eligibility/proofs are unchanged.

### BED-025: direct Copy-field assignment through exclusive named references

BD-040 implements `access.field = value;` for a directly named ordinary exact
`&mut Record` parameter or local, including an immutable handle binding. The
actual record selects its canonical field and exact nominal range; direct
contextual literals form at that boundary. Shared and non-record references reject.
Unlike BD-038 owned-local RHS-first writes, this operation retains the existing
exclusive loan before the complete RHS, then revalidates the available handle,
live lineage and original LoanId before authorizing the write. Same-handle reads
are legal; original-owner conflicts and RHS target transfers reject. Last use
expires only after completion. Lawful unrelated RHS effects commit on success;
failure restores the exact prepared operation entry with the initializer index
shared. Finite/recurrent summaries include target and RHS. Stable loops, continue,
break and existing convergence integrity remain authoritative. HIR/MIR preserve
CopyReferenceFieldAssign distinctly. General projection, reborrow/coercion, field
loans, partial ownership and whole-record replacement remain excluded. Ordinary
functions are checked/lowered only; verified/harness behavior and proofs are unchanged.

## BED-026 execution addition (BD-041)

Frozen Draft 3 adds bounded `run` of the complete accepted ordinary MIR subset.
Source ingestion and the existing frontend/ownership pipeline are protected before
growth; an opaque checked artifact binds canonical HIR, MIR, and lowering origins.
Explicit frames and continuations preserve written evaluation order, moves,
short-circuiting, prefix mutations, and immediate widened named-range traps.
Execution is independent of specialized proof and harness evaluation. `check` and
`verify` retain their existing behavior. This is interpretation, not native code
generation or ordinary contract proof. APIs, limits, origins, and tool reporting
are documented in `docs/BOUNDED_EXECUTION.md`; excluded language boundaries remain
unchanged. General execution mechanisms beyond this frozen subset remain open.


## BED-027 cleanup decision (BD-042)

Frozen Draft 1 accepts RAII-like deterministic scope cleanup for canonically
resource-bearing owned values. Resource leaves have one fixed trusted non-fallible
release operation; records become resource-bearing structurally and clean their
resource-bearing fields recursively in reverse declaration order without a
user-authored destructor body. Every resource-bearing value is Move, but ordinary
Move values remain semantically inert at destruction. A move transfers the unique
cleanup responsibility; borrowing and loan expiry do not affect it.

Still-live owners clean exactly once on covered structured exits: block fallthrough,
implicit NoValue completion, value or NoValue return, break, continue, and future
typed recoverable propagation. Exited inner scopes precede outer scopes; locals
clean in reverse successful-initialization order before parameters, parameters
clean in reverse declaration order, and return-result ownership is transferred
before the callee cleans its remainder. Pending record construction owns each
successfully initialized resource-bearing field and, on future recoverable failure,
cleans the actual initialized set in reverse canonical field-declaration order.

Automatic cleanup releases authority only. Fallible flush, commit, publication,
durability, synchronization, and protocol completion remain explicit. Cleanup must
be represented through auditable validated MIR control flow, never runtime HIR
reconstruction or name-based destructor lookup. Range traps, configured stops,
integrity failures, host/tool failures, process termination, panic/unwind, and task
cancellation receive no cleanup guarantee from BD-042. The decision adds no partial
moves, non-Copy replacement, generalized projection, arbitrary destructor code,
resource API, effects system, heap, FFI, exception model, or temporary-lifetime rule.
The authorized BED-027 implementation supplies private canonical acquisition and
release descriptors, non-cloneable resource payloads, bounded shared cleanup plans,
explicit MIR scope entries/initializations/exits and protected return ownership.
Runtime activations retain actual successful initialization order; transferred
slots have explicit provenance distinct from vacancy. Pending construction tracks
exact field membership and a private bounded failure facility demonstrates abort
order without admitting source-level recoverable propagation. Cleanup stops retain
earlier effects and remaining owned payloads and cannot retry or publish success.
Undefined resource argument temporaries reject at canonical admission. Every
private fixture repeats canonical validation, ownership, lowering, correspondence
and opaque approval. Source syntax and ordinary resource-free accounting remain
unchanged. Implementation evidence and the exact predecessor inventory are in
`docs/engineering/BED-027-IMPLEMENTATION-LEDGER.md`; during K27-I01 only GitHub
Actions supplies authoritative executable evidence. Acceptance remains Greg's
decision and merge remains a human action.

## BD-043 owned Move replacement

Accepted BED-028 Draft 1 adds deterministic whole-owner replacement through existing
assignment syntax for available mutable owned Move locals. Exact record identity,
prepared checker rollback, target continuity, final loan expiry and relative loop
boundaries are authoritative. RHS results are protected before displaced cleanup;
new generations register in the target's owning activation. Old generations never
clean a newer occupant. Public records demonstrate inert replacement; private
synthetic resources demonstrate fixed canonical cleanup and identity conservation.

Earlier sections retain their historical restrictions; this narrowly refines their
direct-local replacement exclusion. References, projections, self-consuming
transforms, unavailable slots, loop-carried Move mutation, arbitrary temporaries,
user destructors and production resource APIs remain excluded. See
LANGUAGE_DECISIONS.md and the BED-028 implementation ledger for scope and
GitHub-executed evidence. Greg accepted and human-merged PR #65 as commit
`72c34593e79e7763ce16f0e18ad99595959a2454`, tree
`2b9339fbe21f705cb8c5722ff681c34680870490`.

## BD-044 expression temporary ownership

Accepted BED-029 Draft 2 gives admitted resource-bearing expression intermediates
explicit bounded ownership. Caller expressions share their source evaluation
region; callee statements and returns have their own regions. Record fields use
child regions, transferring into pending construction before child retirement.
Owned arguments evaluate left to right and transfer once on whole-call entry.
Remaining siblings retire in reverse successful creation order; complete aggregates
retain reverse canonical field-declaration cleanup order.

Results are protected before other temporaries retire and before locals, existing
assignments, returns, replacements or condition selection continue. Pending-field
transfer is the specified immediate-transfer exception. Static regions/sites and
dynamic activations/owners are distinct; temporary handles are neither LocalIds nor
borrowable places. Resource-free evaluations omit empty temporary machinery.

This refines earlier temporary exclusions only within BD-044. Temporary
borrowing, value-returning call statements, new recurring-while Move/reference
eligibility, unwind, rollback, custom destructors and public resource APIs remain
excluded. See the BED-029 implementation ledger for GitHub-executed evidence.
Greg accepted and human-merged PR #67 as commit
`5ce5dc5895efab7ba29a7cbc8e980ce58a7b24ab`, tree
`e95fb9e69d7554719d86d9662cf22cbf5151b84b`.

## Frozen BED-031 implementation boundary

Issue #72 Draft 4 adds exact ordinary `Result<success: S, failure: F>` signatures,
contextual Ok/Error/Err construction and a fixed exhaustive statement match. Error
and Err normalize to one canonical Error tag. The full ASCII case-insensitive
Result/Ok/Error/Err/match families are reserved, while only exact syntax spellings
are accepted. Every Result is Move and carries an independent unresolved outcome
obligation through locals, calls, nested payloads, branches and structured exits.
Cleanup and conditional Move unavailability cannot acknowledge it.

Finite owned bool/range/record/nested payloads, ordered variant plans, structural
cleanup-effect unions and actual machine-owned cells preserve active-only custody.
Manual propagation protects the complete outgoing Result before callee cleanup;
cleanup failure prevents delivery. Resource-free Results do not enable the private
resource catalog. No conditional Result initializer, raw named-range constructor
formation, reference/public-record/verified/harness extension, general enum/pattern,
`?`, conversion, unwind, retry, rollback or production resource API is added.
The implementation and ledger await independent review and Greg's acceptance.

## Frozen BED-032 implementation boundary

Frozen Issue #75 Draft 2 adds exact lowercase contextual `or return` only at an
ordinary Result-returning Value function's local initializer. The independently
typed inner Result and declared outer Result must share exact failure identity;
their success types may differ. Success initializes the original local in its
actual lexical scope; failure canonically rewraps the selected payload and uses
protected ordinary return, actual-scope cleanup and delivery after completion.
There is no synthetic owning match arm. Existing replacement generations yield
direct A → C → B versus explicit match A → B → C.

The existing ownership checker retains outcome/effect witnesses and all loop/loan
restrictions. Tool stops remain distinct from source Error. Contextual `or` remains
an identifier elsewhere; existing keyword aliases/trivia are preserved. No `?`,
conversion, nested propagation expression, conditional Result initializer, Result
replacement, verified/harness extension, unwind or resource API is added. The
implementation is authorized on its dedicated branch; independent review and
Greg's acceptance remain pending. Its exact baseline is the human-merged BED-031
commit 74c1bedc8f847ffaeb38ed4795ee91730e35b949, tree
2d0cc78b67a8cb69ea8feaff183dbf1b91938e63.

## Frozen BED-033 implementation boundary

Issue #78 Draft 2 is frozen; Greg separately authorized implementation on baseline
62023217b3f93e53cae302731643cbcea1fb3d17 (tree
3998bcad9b34d66e960b863eb20b7b78680da07d). The branch implements one explicitly
selected immutable sensor-demo catalog: exact nominal Reading 0..4095 and
SensorError 1..3, opaque Move Resource Sensor, canonical sensor_open/sensor_read
trusted calls and cleanup-only fixed release, all effects io. Operation-specific
grants remain separate from effects. Open Error is only 1/3; read Error is only
2/3 and preserves the Sensor. No selected catalog means no names or authority.

The defining source uses existing exact local `or return`. Owned helpers, protected
internal relay, shared observation, nested Results and structured cleanup reuse
the existing ownership/outcome/effect/temporary systems. Retire needs io for
residual cleanup; relay transfer alone does not. Mutable Sensor refs, conditional
Sensor initialization, replacement, record fields, representation/forging and
public live-Sensor egress remain excluded. Existing whole-Result obligations and
sealed private primitive cleanup are unchanged.

Explicit MIR plans and opaque approval seal catalog/profile/compiler/configuration,
grants, binding/signature/effect identities, roots, origins and custody. Only a
persistent ExecutionSession may invoke: its pre-reserved machine-owned receipt
holds actual binding payload before any fallible boundary; stopped sessions retain
unfinished responsibilities, protected Results and completed prefix. Terminal
Host/Limit/Integrity/trap/cleanup failure never becomes source Error or permits
retry/resume/unwind. Teardown is not successful release. The fixed deterministic
fixture is not hardware/FFI. BED-034's frozen completion specification now has an
implementation candidate, described below; acceptance remains pending.

CLI `run` and `audit` require `--profile sensor-demo` and an explicit --function
root; the static bounded audit follows sealed structural edges through empty-effect
helpers and both alternatives, disclosing grants versus reachable use and exact
release pairing without invocation. See docs/engineering/BED-033-TRUSTED-SENSOR.md,
its ledger and identity manifest for contracts, budgets, commands and evidence.
Implementation is not independent confirmation or acceptance; Sarah's exact-head
review and Greg's acceptance remain separate, and only humans merge.

### BED-034 frozen completion implementation candidate

Draft PR #83 implements consuming `sensor_finish(Sensor)` with exact
`Result<success:FinishReceipt,failure:SensorError>` and `{io}`. FinishReceipt is
nominal `1..1`, Copy source-formable status data, never proof or authority.
Only finish Error(3) is admitted. Both ordinary outcomes remain protected until
the exact fixed release succeeds. Pending completion custody and protected Result
are separate; terminal stops retain actual prefix and remaining duty with no
retry/resume/rollback or caller delivery. The existing checker, temporary, Result,
cleanup, effect and opaque approval systems remain the only authorities.

For this task GitHub Actions is the sole authorized Rust execution environment.
M0 used a fresh same-tree baseline gate; M5 is separately preserved before M6-M9.
See `docs/engineering/BED-034-COMPLETION-CUSTODY.md` and the implementation ledger
and manifest for source examples, exact boundaries, malformed controls, workflow
evidence and limitations. This candidate changes no Accepted decision or proof
claim and still requires Sarah's independent exact-head review and Greg's decision.

## Accepted BED-035 Stage I implementation

Frozen Issue #84 Draft 2 defines approved-MIR execution-engine equivalence.
Greg accepted the independently reviewed Stage I implementation, and human-merged
PR #85 as commit `c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717`, tree
`a76e2ab68c8d2db094f1005906bdc24b6a9b8795`. The accepted implementation head
was `837fc99e2bbb241077d4b65b32a4103b6bf99362`. All four post-merge
build/tests/verify/docs workflows passed.

The engine contract is an immutable bounded view of the same opaque approved
HIR/MIR/origin/outcome/effect/cleanup/domain/trusted composite. Versioned semantic,
logical-accounting, cut, observation and replay identities are sealed into that
approval. Ordinary empty-effect calls, both alternatives and recursive closure
are included. No new checker or ownership authority is introduced.

Opt-in conformance sessions capture real interpreter history and final custody
under separate admission. Independent validation precedes one anchored injective
dynamic correspondence and Equal/Different/Inconclusive. Capture completion alone
is not validity. Private addressed replay supplies only already approved binding
behavior; it adds no public input ABI, profile, grant, source operation or resource.
Modeled Host, unmatched surviving physical Host and loss of the execution substrate
remain distinct. Terminal advance never resumes or fabricates source Error.

Structured rooted authority and bounded human presentation have separate evidence
accounts. Existing public preparation, run, session, audit, diagnostics and reporting
boundaries remain governed by their predecessors. Version-one capacity/scan charges
are logical compatibility facts, not a native ABI or a mandate for physical frames,
allocations or dead stores.

Sarah's reviews identified K35-I01 through K35-I04. The accepted repairs resolve
canonical ordinary origin rows, separate stored reference identity from live access,
bind queued/reached work to approved producer/operand/continuation relationships,
and make malformed-observation controls reach their named protections. Sarah's
final focused confirmation resolves I01–I04, assigns no I05, and reports the exact
accepted head ready for owner acceptance. The complete debug and release profiles
each execute 2,112 identities with zero predecessor migrations.

No native backend, target, ABI, layout, emitter, linker, optimizer, runtime ABI,
transport, serialization, signing or packaging is selected. Native installation,
actual optimized/unoptimized comparisons and native physical-failure evidence
remain mandatory deferred Stage N work requiring a separately authorized BED.
Stage I acceptance and merge do not authorize Stage N. See
`docs/engineering/BED-035-STAGE-I-LEDGER.md` and its evidence manifest.

<!-- END BOWFIN_CONTEXT.md -->

---

<!-- BEGIN DESIGN_PRINCIPLES.md -->

# Bowfin Design Principles

These principles guide language and toolchain decisions.

## DP-001 — Own what moves

Ownership and borrowing are foundational, not optional library conventions.

## DP-002 — Constrain what exists

Types should be capable of describing the program's domain: bounded integers, percentages, voltages, durations, byte counts, physical units, legal state values, and invariants.

## DP-003 — Prove what matters

Formal verification should be available without requiring every function to be formally verified. Proof effort must be proportional to consequence.

## DP-004 — Make every escape hatch visible

Trust should not disappear into implementation detail. `trusted`, `unsafe`, FFI, hardware assumptions, allocation, blocking, and important effects should be discoverable and auditable.

## DP-005 — Memory safety is foundational, not sufficient

A program can be memory-safe and still violate a precondition, invariant, range, scheduling constraint, effect restriction, or protocol rule.

## DP-006 — One language, escalating assurance

`safe`, `checked`, and `verified` belong to one semantic language.

## DP-007 — Effects are part of an API

Whether a function allocates, blocks, performs I/O, accesses the clock, reads randomness, or crosses an unsafe boundary may be as important as its return type.

## DP-008 — The toolchain must explain itself

Diagnostics should explain what failed, what is known, what assumption is missing, which call introduced an effect, and what the developer can do next.

## DP-009 — Semantic honesty over marketing

Never claim that verification proves unspecified global correctness, that Bowfin eliminates bugs, or that testing is unnecessary.

## DP-010 — Explicitness without mandatory verbosity

Bowfin may provide standardized compact and explicit spellings for selected high-frequency keywords. The spellings are semantically identical.

## DP-011 — High assurance should feel like normal development

```bash
bowfin build
bowfin test
bowfin verify
bowfin audit
```

## DP-012 — Do not hide computational cost

Allocation, dynamic dispatch, blocking, and similar operational costs should not become invisible when they matter to systems programmers.

## DP-013 — Realtime claims must be defensible

Initially prove structural restrictions that the compiler can actually establish. Do not claim arbitrary WCET proof until the toolchain truly supports it.

## DP-014 — Project knowledge must be portable

If Bowfin can only be understood because one conversation remembers its history, the project is under-documented. A fresh expert should be able to read the repository context and become productive quickly.

## DP-015 — Humans execute repository merges

The final act of merging any repository pull request is reserved to a human.
AI agents may prepare changes, create branches and commits, open pull requests,
run validation, inspect exact heads, conduct merge reviews, recommend a
disposition, and record explicit human authorization. They must never execute a
merge through an API, command, user interface, automation, or equivalent action.

A human statement that a pull request is authorized for merge authorizes the AI
to record that decision and hand the ready pull request back to the human. It does
not authorize the AI to perform the merge. This boundary applies equally to code,
documentation, and governance changes.

## DP-016 — Documentation is a human-facing contract

Code is an exact machine artifact, not a complete human explanation merely
because its author finds it readable. Documentation is part of the program's
human-facing contract. Code and documentation are reviewed together, and
disagreement between them is a defect.

Public documentation should explain purpose, domain meaning, effects, authority,
failure behavior, lifecycle obligations, important policy, assumptions, bounds,
and non-guarantees in clear human language. A machine-readable type, contract,
or effect does not make that explanation unnecessary; prose does not replace the
machine-readable rule.

Bowfin should eventually preserve structured documentation comments, generate
reference material, compile documentation examples, and validate mechanically
checkable claims against canonical compiler facts. Documentation metadata may
never grant authority, change semantics, create proof evidence by assertion, or
override canonical artifacts. Exact documentation-comment syntax and tooling
require a focused future decision.

Bowfin-owned public examples prefer implemented explicit aliases under BD-014.
See [the documentation policy](../DOCUMENTATION_POLICY.md).

BD-041 applies the existing trust-boundary principles to bounded ordinary
execution: an opaque successful-ownership artifact grants authority, MIR controls
runtime flow, and explicit limits begin before source ingestion. A runtime trap
or configured stop is a tool outcome; execution never implies contract proof.

Frozen BED-033 applies DP-004/DP-007/DP-008/DP-010/DP-012 to the first selected
trusted resource slice. Concise ordinary source is supported by an explicit exact
authority account, not hidden host lookup. Grants, complete effects, recoverable
Result and terminal tool outcomes stay distinct. No-gap owned receipts and persistent
stopped sessions retain real responsibilities without promising unwind or cleanup
after failure. Structural audit, bounded work/storage and deterministic private
failure evidence expose this trust boundary. Implementation does not itself grant
independent acceptance or verify the host/device; explicit fallible completion is
deferred and human-only merge authority remains unchanged.

Frozen BED-035 applies the existing trust, explicit-cost and audit principles to
engine equivalence. Approved semantics and independently derived evidence constrain
an interpreter as well as a future engine. Complete validated observations can
establish Equal or Different; missing or malformed evidence yields Inconclusive.
Observation and rendering failure neither acknowledge an outcome nor change the
retained primary execution. A canonical cleanup plan supplies no new authority to
execute cleanup. Versioned logical charges and semantic cuts remain visible even
when a later implementation removes their physical work.

The accepted Stage I implementation selects no native design, makes no performance
or new proof claim, and supplies no Stage N execution evidence. Independent review,
owner acceptance and human merge are complete for Stage I. Any backend selection,
Stage N authorization and later human merge remain separate decisions.

<!-- END DESIGN_PRINCIPLES.md -->

---

<!-- BEGIN LANGUAGE_DECISIONS.md -->

# Bowfin Language Decisions

Statuses:

- **Accepted** — settled unless explicitly reopened.
- **Direction Accepted** — architectural direction is settled; details may evolve.
- **Provisional** — useful current choice, not fully locked.
- **Superseded** — historical only.

---

## BD-001 — Public project name

**Status:** Accepted

**Decision:** The adopted language name is **Bowfin**, under the accepted name-migration contract. This identity migration preserves decision numbers, statuses, scope and acceptance history; it creates no language rule or new decision sequence.

**CLI:** `bowfin`; retained compatibility command: `klyxr`
**Primary source extension:** `.bow` on migration staging; explicit `.klx` input retained<br>
**Adopted primary domain:** `bowfin-lang.com` (external cutover deferred)
**Operational legacy endpoints:** `klyxr.com`, `klyxr.online` (retained)

---

## BD-002 — Public positioning

**Status:** Accepted

> **Systems programming you can reason about.**

Canonical summary:

> **Bowfin is a systems programming language that treats memory safety as the foundation, then lets programmers express and verify more of the rules that make their software correct.**

Core philosophy:

> **Own what moves. Constrain what exists. Prove what matters. Make every escape hatch visible.**

---

## BD-003 — Assurance levels

**Status:** Accepted

Bowfin has three escalating assurance levels:

- `safe`
- `checked`
- `verified`

They belong to one language, not separate dialects.

---

## BD-004 — Ownership and borrowing

**Status:** Direction Accepted

Bowfin uses:

- moves;
- immutable borrowing;
- mutable borrowing;
- deterministic destruction;
- no required garbage collector.

Move semantics are the default for non-`Copy` values.

---

## BD-005 — Domain and range types

**Status:** Accepted

Bowfin supports real distinct constrained types.

```bowfin
type Percent = range 0..100;
```

Units and dimension-aware types are part of the language direction.

---

## BD-006 — Contracts and invariants

**Status:** Accepted

Bowfin supports first-class:

- `requires`
- `ensures`
- type/record invariants
- `old(...)`
- quantification where appropriate

In checked mode, contracts may become runtime checks.
In verified mode, contracts create proof obligations.

---

## BD-007 — Sum types and recoverable errors

**Status:** Direction Accepted

Bowfin uses:

- algebraic enum/sum types;
- exhaustive `match`;
- `Option<T>` instead of universal null;
- `Result<T, E>`;
- `?`-style propagation or equivalent ergonomic syntax.

---

## BD-008 — Explicit effects

**Status:** Direction Accepted

Bowfin has compiler-visible effects.

Candidate core effects include:

- `io`
- `heap`
- `blocking`
- `network`
- `filesystem`
- `clock`
- `random`
- `unsafe`

Effects propagate transitively through the call graph.

Frozen BED-030 Draft 2 ([Issue #70](https://github.com/gregkrsak/klyxr/issues/70))
fixes the first ordinary-function phase: these eight spellings are independent
canonical audit identities. Ordinary Value and NoValue `fn`/`function` signatures
may declare `effects io, clock` after the optional result type. Absence means the
empty set; unknown and duplicate identities reject. A declaration is an explicit
upper bound, including any legal unused members. Every structurally reachable
ordinary call contributes the callee's complete declaration without fixed-point
body inference. Verified and harness declarations remain excluded.

The same bound admits mechanically derived cleanup effects at eligible sites
produced by the existing ownership checker. Leaf cleanup derives from the complete
fixed release-operation contract; aggregate cleanup unions child summaries. Moving
responsibility performs no cleanup, and an empty summary preserves the obligation.
`unsafe` grants no permission. Ordinary terminal stops add no cleanup edge; private
approved abort and primitive/protocol evidence have separate sealed domains.

This branch implements the frozen first phase and awaits independent exact-head
review and Greg's acceptance. BD-008's broader Direction Accepted status is unchanged.
Production effectful APIs, effects-aware verification, policies, aliases,
hierarchies and polymorphism are not part of this phase.

---

## BD-009 — Trust domains

**Status:** Accepted

Bowfin distinguishes:

- `safe`
- `trusted`
- `unsafe`

`trusted` means an assumption is intentionally accepted and auditable.
`unsafe` means normal language safety rules may be bypassed.

---

## BD-010 — Auditability

**Status:** Direction Accepted

The toolchain should provide an audit workflow such as:

```bash
bowfin audit
```

It should ultimately expose trusted assumptions, unsafe regions, transitive trust dependencies, effect-policy violations, and assurance-relevant project information.

---

## BD-011 — Unified toolchain

**Status:** Accepted

Canonical command family:

```bash
bowfin new
bowfin build
bowfin run
bowfin test
bowfin verify
bowfin bench
bowfin doc
bowfin audit
bowfin format
bowfin lint
```

Exact subcommand details may evolve.

---

## BD-012 — Compiler semantic pipeline

**Status:** Direction Accepted

```text
source
  ↓
lexer / parser
  ↓
AST
  ↓
name resolution
  ↓
typed HIR
  ↓
types / ownership / effects
  ↓
MIR
  ├── verification path → VIR → solver
  └── codegen path      → backend → machine code
```

Verification occurs before backend transformations can redefine language semantics.

---

## BD-013 — Verification is modular

**Status:** Direction Accepted

Callers rely on callee contracts rather than reproving entire implementations on every call.

Verified loops use appropriate invariants and, where needed, termination variants.

---

## BD-014 — Long-form keyword aliases

**Status:** Accepted

Selected high-frequency keywords may have approved compact and explicit spellings.

The spellings are lexical synonyms and normalize to the same semantic token.

| Compact | Explicit |
|---|---|
| `mut` | `mutable` |
| `fn` | `function` |
| `pub` | `public` |
| `mod` | `module` |
| `impl` | `implementation` |
| `const` | `constant` |

Examples:

```bowfin
pub verified fn transfer(source: &mut Account)
```

```bowfin
public verified function transfer(source: &mutable Account)
```

They must produce identical semantics.

**Default ecosystem style:** compact.

An organization may select explicit style through formatter/linter policy, e.g.:

```toml
[style]
keyword_spelling = "explicit"
```

The compiler understands both forms. Style enforcement belongs in formatting/linting unless a project promotes the lint to an error.

**Hard rule:** aliases may never acquire semantic differences.

---

## BD-015 — Deterministic arithmetic semantics

**Status:** Direction Accepted

Optimization level must not silently change arithmetic semantics.

Current operator direction:

- ordinary `+` — checked
- `+%` — wrapping
- `+?` — result-on-overflow
- `+^` — saturating

Exact spellings remain reviewable.

---

## BD-016 — Realtime profiles

**Status:** Direction Accepted

A hard-realtime profile may reject:

- heap allocation;
- unbounded blocking;
- forbidden recursion;
- other structurally unbounded or disallowed behavior.

Initially claim only structural properties the compiler can defensibly establish.

---

## BD-017 — Public guarantee discipline

**Status:** Accepted

Bowfin never equates verification with unspecified global correctness.

Canonical wording:

> **The stated properties were proven under the stated assumptions.**

---

## BD-018 — Typed expression semantics

**Status:** Direction Accepted

Bowfin expressions are resolved and type-checked before verification or backend lowering.

Typed HIR records the semantic type of expressions.

`Bool` is a built-in semantic expression type.

Named constrained range types remain nominally distinct during expression typing;
operators do not implicitly mix different constrained type identities merely
because their representations or bounds are compatible.

Integer literals are constants and do not themselves create implicit conversions
between distinct named constrained types.

The initial executable expression subset and surface spellings remain subject to
the explicit open questions and prototype-boundary rules. Implementing an operator
in the prototype does not settle the complete Bowfin operator set.

---

## BD-019 — Explicit value returns in block-bodied functions

**Status:** Accepted

Bowfin uses explicit value returns in block-bodied functions.

A value-returning block function declares its return type with `-> Type` and
returns a value with:

```bowfin
return expression;
```

Bowfin does not use Rust-style implicit block-tail returns. The absence of a
semicolon on the final expression does not cause that expression to become the
function's return value.

Local bindings use `let` and are immutable by default; future local mutation
must be explicit. Function calls are expressions.

A future explicitly marked expression-bodied function shorthand is not ruled
out, but it would be a separate syntactic construct and would not change the
explicit-return rule for block-bodied functions.

---

## BD-020 — Core owned-value move semantics

**Status:** Accepted

Bowfin uses move-by-default ownership for non-`Copy` values.

In the initial core ownership model:

- `bool` values are `Copy`;
- named constrained range values are `Copy`;
- record values are non-`Copy` and move by default.

A non-`Copy` owned value moves when transferred through a by-value binding,
by-value function argument, or return. After a move, the previous owned place
may not be used as a value. The compiler never silently clones a moved value.

This initial `Copy` classification is semantic compiler behavior; BED-005 does
not establish the eventual user-facing `Copy` trait or customization mechanism.
A record is non-`Copy` even when all its fields are `Copy`.

Borrowing is a separate language mechanism and is not introduced by BED-005.

---

## BD-021 — Core borrowing semantics

**Status:** Accepted

Bowfin uses explicit shared (`&T`) and exclusive (`&mut T`) borrowing.
A shared loan permits multiple overlapping shared loans but conflicts with an
exclusive loan. An exclusive loan conflicts with all other active loans and with
direct access to the borrowed owner. Moves of a non-`Copy` owner are forbidden
while any loan of that owner remains active.

Stored loans remain active through the last possible use of the reference values
derived from them, rather than mechanically through the enclosing lexical block.
Shared reference values are `Copy`; mutable reference values are non-`Copy` and
move by value. Borrowing, cloning, mutable reborrowing, and reference coercions
are never inserted implicitly.

BED-006 implements only whole-value, straight-line, non-escaping borrowing.
`let mut` marked an owned local as exclusively borrowable; BED-006 itself added
no reassignment. BD-026 later permits narrow Copy-safe direct local reassignment.
BED-006 does not establish explicit lifetime syntax, reference returns, dereference
semantics, partial borrowing, or mutation through references.

---

## BD-022 — Core dereference and Copy-safe mutation

**Status:** Accepted

Bowfin uses explicit `*reference` dereference syntax. Dereferencing accesses the
referent but does not itself transfer or consume the reference handle.
Borrowed Copy values may be read through either `&T` or `&mut T`; the read copies
the referent. Borrowed non-Copy values may not be moved out through dereference.

Whole-value write-through is permitted through `&mut T` only when T is Copy.
Write-through through `&T` is forbidden. Non-Copy replacement remains forbidden
until ownership and destruction semantics for displaced values are defined.

BED-007 implements directly named ordinary reference access and dedicated
`*reference = expression;` statements only. It does not establish owned-local
reassignment, non-Copy replacement, destructors/Drop, field access or mutation,
auto-deref, reborrowing, compound assignment, assignment expressions, or control flow.
BD-026 later establishes a separate narrow Copy-safe owned-local reassignment form.

---

## BD-023 — Statement conditionals and MIR control-flow foundation

**Status:** Accepted

Bowfin initially introduces `if` / optional `else` as control-flow statements.
An `if` requires a Bool condition, creates lexical child scopes for its branches,
and rejoins after the selected branch completes. Nested statements are allowed;
conditionals do not produce values. Visible names cannot be shadowed; sibling
branches may reuse a spelling with distinct canonical local identities.

A non-Copy place visible after a join is available only if it remains available
on every reachable incoming branch. A move on any such branch prevents later
use or borrowing of that place. Mutable reference handles follow the Move rule;
shared reference handles remain Copy.

BED-008 establishes acyclic ordinary MIR basic blocks and explicit Branch, Goto,
and Return terminators. Ordinary functions still require one final function-level
return. It establishes no loops, early/multiple returns, SSA, phi nodes, block
parameters, or generalized control-flow dataflow. Value-producing conditionals
remain unresolved, and the MIR architecture leaves their future extension open.

---

## BD-024 — Path-sensitive acyclic loan liveness

**Status:** Accepted

For the current acyclic statement conditional subset, whole-value loan liveness
is control-flow-path-sensitive. A loan remains active on a path while an available
reference handle carrying its provenance may still be used on that path, or an
explicit operation hold requires it. Lexical coexistence of reference bindings
does not imply overlapping loans. Sibling-only uses do not keep a loan active
on the opposite branch.

A loan needed by any possible successor stays active through condition evaluation
and dispatch. After the split it may expire independently on an edge with no
possible future use of a surviving handle. Post-join future uses propagate into
every incoming path where that handle may remain available. Loans cannot expire
and later reactivate along the same path; both branch edges remain possible,
including constant Boolean conditions.

At joins, Move availability retains BD-023's all-path rule. Potentially active
incoming loans merge conservatively: an operation must be safe on every incoming
path. Conditional unavailability of a handle does not itself erase a loan still
needed on a path where the handle was not moved. Loans inactive on all incoming
paths are dead after the join. Shared copies retain provenance; mutable transfers
remain moves. Call/write holds remain authoritative and branch-local handles
and loans do not escape.

BED-009 refines the existing ownership phase above MIR. It adds no syntax, loops,
reborrowing, reference returns, explicit lifetimes, conditional values, destruction,
or generalized CFG lifetime inference. General MIR dataflow and cyclic loan analysis remain
unresolved under OQ-024/OQ-026; BD-027 later settles narrow statement loops.


---

## BD-025 — Value-producing conditional initialization

**Status:** Accepted

An ordinary local initializer may select an owned value with:

```bowfin
let chosen = if flag { first } else { second };
```

`let mut` / `let mutable` may use the same one-time initialization. Braces and
`else` are mandatory. Each branch contains exactly one value expression, without
a trailing semicolon or statements. A nested conditional may be the complete
branch result. Statement `if` remains separate and retains optional `else` and
lexical statement branches.

The condition must be Bool. Both branches must have exactly the same permitted
concrete owned type: Bool, a canonical named range, or a canonical record. Distinct
nominal declarations are incompatible even with identical bounds or structure.
There is no contextual integer-literal materialization, implicit conversion, or
reference-valued conditional result. Copy dereference results follow existing
rules; non-Copy move-out through references remains forbidden.

Condition ownership effects occur once before independent branch snapshots. A
Copy result preserves its source; a Move result transfers into the destination
on that path. Every reachable leaf initializes the same source local. The
destination becomes available after the join; source availability retains
BD-023's all-path rule. The same source may move into the destination on both
mutually exclusive paths, but cannot then be used again. BD-024 continuation-aware
loan liveness and call/write holds apply without relaxation.

Typed HIR retains the conditional and its exact type. The first MIR strategy
emits explicit Branch/Goto edges, initializes the canonical destination LocalId
on each mutually exclusive leaf, and rejoins without a value merge instruction.
This is one-time initialization, not reassignment or SSA. No phi, block parameters,
general temporaries, result slots, or definite-assignment solver are introduced.

BED-010 permits this form only as a complete local initializer or recursively as
a complete value branch. Returns, call arguments, unary/binary operands,
parenthesized conditional values, dereference writes, and general expression
positions remain unsupported. It settles no statement-containing value blocks,
reference lifetime joins, early returns, loops, destruction, execution, or general
MIR value-flow strategy. These boundaries remain open under OQ-022/OQ-024/OQ-026.


---

## BD-026 — Copy-safe direct local reassignment

**Status:** Accepted

An ordinary local declared with `let mut` / `let mutable` may be directly
reassigned when it holds an owned non-reference Copy value. Initially the
permitted target types are Bool and named constrained ranges:

```bowfin
let mut current = start;
current = next(current);
```

The target is one directly named, in-scope mutable local. Parameters and immutable
locals are not assignable. The RHS must have the exact same concrete type,
including nominal range identity. There is no contextual integer-literal
materialization, conversion, structural matching, or implicit clone.

Assignment is a statement with a required semicolon and produces no value.
It remains distinct from Let initialization and explicit DerefAssign write-through.
Chained/compound assignment, projected/field/index targets, assignment expressions,
and conditional values on an assignment RHS remain unsupported.

For this directly named form, RHS evaluation occurs before mutation. Existing
RHS ownership effects and call holds complete, last-use loans may expire, and
then the write requires exclusive mutation access: any active shared or exclusive
loan of the target conflicts. This allows Copy self-assignment and final reference
reads/calls on the RHS before the write when no future handle use keeps the loan
active. BD-024 continuation-aware path-sensitive expiry remains authoritative in
branches; a post-join reference use protects the owner on every relevant path.
Assignment creates no loan or new local identity, moves nothing out of the target,
and leaves the Copy owner available. This rule does not settle evaluation order
for future generalized place assignments.

Reference-local reassignment is rejected during typing because provenance/lifetime
replacement remains unresolved. Mutable record locals remain non-Copy: exact-typed
replacement is rejected during ownership checking before consuming either owner,
until displacement/destruction semantics are defined. No destruction, leak, clone,
swap, take, or replace behavior is implied.

AST, resolved statements, typed HIR, and MIR use a dedicated Assign node. HIR/MIR
retain the canonical destination LocalId, typed RHS, and statement span. Straight-line
lowering adds no blocks; branch lowering uses the current branch block. MIR remains
deterministic with explicit Branch/Goto/Return terminators (acyclic in BED-011;
BD-027 later adds loop backedges). BED-010
conditional initialization still emits mutually exclusive Let statements, not Assign.

BED-011 adds no loops/backedges, fixed-point analysis, general ownership dataflow,
SSA, phi, block parameters, general temporaries/result slots, definite assignment,
non-Copy replacement, reference replacement, execution, or verification expansion.
OQ-022/OQ-024/OQ-025/OQ-026 retain broader mutation and control-flow design.


## BD-027 — Ordinary while loops with loop-stable ownership

**Status:** Accepted

Ordinary `while condition { statements }` is a pre-test, zero-or-more-iteration
statement. The condition must be exactly Bool. Braces are required; the block has
no trailing semicolon or result value. Nested loops and statement conditionals
are permitted. Each body is a lexical child scope: declarations are ordered,
visible names cannot be shadowed, and canonical LocalIds are static source
identities, not new identities for each iteration. One final function-level return
remains required; loop-local returns are unsupported.

BED-012 permits ownership-stable owned Copy Bool/named-range activity in the
condition and body, including BD-026 direct reassignment, Copy calls/locals,
and BD-025 Copy conditional initialization. Header and backedge ownership/loan
states must agree after body-local state is scoped away. Untouched outside Move
owners and references may persist unchanged. Outside live loans still constrain
owner access and assignment; a dead pre-loop loan is not resurrected.

All reference activity in the condition/body (creation, handle use/transfer,
arguments, dereference and write-through) and, under BED-012, all non-Copy activity (including
fresh record call results and nested consumption) are rejected pending cyclic
ownership/lifetime analysis. BD-028 later permits iteration-local Move activity
in the body while preserving the condition and reference boundaries. A reference
being Copy does not bypass this boundary.
Finite future-use counts remain outside-continuation bookkeeping, not cyclic NLL.
Ownership remains above MIR with one permitted-iteration stability check and no
fixed-point solver or second checker.

MIR lowers while to existing Goto/Branch primitives: preheader to header, header
to body or exit, and body back to header. Mutable locals retain source LocalIds;
Let remains initialization and Assign remains mutation. MIR is no longer globally
acyclic. Structural validation accepts cycles, checks targets/reachability/types,
and makes no termination claim. Constant conditions do not remove either edge.

No break/continue/for/unconditional loop, labels, loop values, early returns,
phi/SSA/block parameters, generalized temporaries, destruction, reference returns,
reborrowing, execution, code generation, verified loops, invariants/variants, or
termination checking is introduced. OQ-024/OQ-026 retain cyclic lifetime/ownership
and broader control-flow questions.


---

## BD-028 — Iteration-Local Move Ownership

**Status:** Accepted

Non-Copy ownership activity may occur inside an ordinary while body when it
originates and stays entirely within the current iteration and leaves all
pre-existing ownership state unchanged at the backedge. Fresh non-Copy call
results may be bound, moved through iteration-local bindings and by-value calls,
or consumed by calls returning Copy values. Existing BD-020 move rules and
BD-023 acyclic branch joins apply normally, including use-after-move rejection.
Existing conditional initialization may select iteration-local owned values.

A body-local declaration has one static canonical LocalId and represents fresh
initialization on each dynamic iteration. Its tracked availability is scoped
away before the BD-027 header/backedge semantic comparison. This compile-time
scope cleanup introduces no runtime destruction, Drop, cleanup or resource-release
semantics. Unused owned locals retain the existing prototype scope-exit behavior.

Parameters and locals present at entry to the current loop are pre-existing state.
They cannot move through bindings, calls, or nested branches inside that loop.
Each nested loop establishes its own boundary: an outer iteration's local is
pre-existing relative to an inner loop and cannot be consumed there. It may remain
untouched by the inner loop and be consumed later in its containing iteration.
Untouched outside owners and live outside loans retain their existing protection.

The condition remains limited to BD-027 ownership-stable Copy activity. Reference
creation, use/alias/transfer/call arguments, dereference and write-through remain
forbidden in conditions and bodies. Non-Copy replacement remains forbidden even
for iteration-local owners. Exact nominal typing, source ordering, no shadowing,
non-escaping scopes and conservative constant-condition edges are unchanged.

BED-013 refines the single typed-HIR ownership checker with per-loop header place
identity, analyzes one permitted iteration, scopes away iteration-local state,
and requires semantic equality of remaining header/backedge state. No fixed-point
solver, general loop-carried Move joins, second checker, MIR ownership annotations,
SSA/phi/block parameters, syntax, conversions, execution, or termination proof is
introduced. MIR retains BD-027's existing lowering and cyclic validation.
OQ-022/OQ-023/OQ-025/OQ-026 retain broader ownership/control-flow questions;
BD-029 subsequently permits iteration-local body borrowing; cyclic lifetimes and
BD-030 subsequently permits stable carried access; generalized cyclic lifetime
inference remains open under OQ-024.


---

## BD-029 — Iteration-Local Borrowing in While Loops

**Status:** Accepted

Ordinary while bodies may create and use whole-value references whose provenance
is created after the current loop header and fully discharged before its backedge.
Both pre-existing owners and iteration-local Move owners may be borrowed, subject
to existing exact nominal types, mutability, capability and conflict rules. Shared
aliases, mutable-handle moves, reference call arguments, named dereference and
Copy-safe write-through retain BD-021/BD-022 rules, including complete call/write
holds. BD-024 path-sensitive last-use rules apply within acyclic body branches.
An iteration-local record may move after its final loan use; borrowing does not
permit moving a pre-existing non-Copy owner or materializing/replacing a non-Copy
referent.

Reference handles present at the current header may persist untouched but cannot
be read, aliased, transferred, passed to calls, dereferenced or written through
inside that loop. Their live loans still constrain owner access and new borrows;
dead pre-loop loans never resurrect. While conditions remain reference-free and
retain BD-027/BD-028 Copy-only restrictions. Constant conditions do not prune checks.
Nested loops establish relative boundaries: an outer-created reference is
pre-existing at an inner header and must remain untouched there, though it may
be used after the inner loop in its containing iteration. Compatible fresh inner
borrows remain permitted.

The existing typed-HIR checker analyzes one cloned iteration. Lexical cleanup
removes body-local handles/availability and expires their loans. Before restoring
the header's private loan allocator, it proves that no newly allocated loan or
surviving handle provenance remains. Remaining ownership availability, handle
provenance and active loans must equal the header semantically. Diagnostic names,
spans and finite continuation counts are excluded from that equality. Allocator
reuse cannot conceal escaping provenance or collide with a surviving loan.
Cleanup is compile-time bookkeeping, not runtime destruction.

This refines BD-027/BD-028's historical body-reference restriction only. No syntax,
public HIR annotations, MIR construct, second checker, cyclic lifetime/fixed-point
solver or execution is introduced. MIR retains existing deterministic Branch/Goto
backedges. BD-030 subsequently settles stable carried access with recurrent
obligations. Reference returns, reference-valued
conditional results, reference-local reassignment, reborrowing, explicit lifetimes,
partial borrowing and generalized cyclic NLL remain unresolved under OQ-024/OQ-026.


---

## BD-030 — Stable Loop-Carried Reference Access

**Status:** Accepted

A reference present at a structured loop header may be used repeatedly within
that loop when its availability, provenance, and required loan state are preserved
across every accepted backedge. If the carried reference is used on any checked
body path, its required loan remains continuously active across every accepted
backedge regardless of constant-condition or one-shot reasoning. Loop-backedge
liveness is distinct from false-exit continuation liveness. Newly created
iteration-local provenance must still be discharged before the current loop's
backedge. This does not establish generalized cyclic lifetime inference.

Shared carried handles remain Copy and may be read, passed to existing calls or
copied into iteration-local aliases. Mutable carried handles may be dereferenced
and used for existing Copy-safe writes, but remain Move: binding transfers and
by-value call arguments reject rather than becoming implicit reborrows. Referent
Copy mutation does not change handle provenance. Existing capability/conflict,
nominal typing, operation-hold and non-Copy access/replacement rules remain intact.

Recurrent loan obligations are separate from finite continuation FutureUses and
operation holds. Discovery includes both checked branches and nested bodies.
Scoped nested obligations retain the original LoanId, owner and kind continuously;
inner false-exit removes only its own frame. Backedge cleanup and exact semantic
validation precede obligation discharge. No expiry/refresh/reconstruction, fake
infinite counts or path-specific release can implement recurrence. On false exit,
ordinary suffix liveness and enclosing obligations determine whether a loan ends.
An unused lexical handle does not itself keep a loan alive.

An outer-iteration-local reference may be carried at an inner header but its
new provenance must still disappear before the outer backedge under BD-029.
Body-local aliases of existing provenance disappear without killing an obligated
carried loan. New provenance is checked before allocator normalization. Failed
reference operations preserve availability, liveness, holds, loan and provenance
state. Names/spans remain metadata; identity and scope are canonical.

BED-015 refines the existing one-iteration typed-HIR ownership checker. While
conditions remain reference-free and carried owned Move changes remain forbidden.
No syntax, MIR ownership engine, cyclic fixed-point solver, general lifetime
inference, reference reassignment/returns/results, escaping references, reborrowing,
coercion, partial borrowing, destruction, execution or termination proof is added.
OQ-024/OQ-026 retain those broader lifetime/control-flow questions.


---

## BD-031 — Structured continue in while loops

**Status:** Accepted

An unlabeled `continue;` statement transfers control to the innermost enclosing
ordinary while header. It is an independently checked explicit backedge: carried
availability, handle mappings, exact continuously active LoanId/owner/kind,
balanced operation holds, and scoped recurrent obligations must equal the header
state after iteration-local cleanup. Fresh provenance must be absent before safe
allocator normalization. A continue never discharges current or enclosing
recurrent obligations. Existing carried Move restrictions remain unchanged.

Continue has no same-iteration finite continuation, but retains the finite
post-loop continuation because the condition-false exit remains a real successor.
Path-specific summaries must establish this distinction before operations preceding
a continue. A body-local loan needed only by a skipped suffix may expire on that
path; a recurrent carried loan or post-loop use still protects the owner. Finite
future uses, recurrent obligations, and operation holds remain separate reasons
for loan activity. No loan may expire and later reactivate on one path.

Direct continue terminates its current lexical block. A statement conditional
whose checked arms both terminate has no fallthrough; a following statement in
that block is rejected structurally, including nested conditionals. Only actual
fallthrough arms participate in a suffix join; a single survivor retains its
ownership effects. No constant-path pruning or generalized unreachable-code
policy is established. A while retains its checked false exit even if every body
path continues, independently of any body fallthrough. False exit discharges only
that loop's obligation frame and applies outside suffix liveness; enclosing
obligations remain authoritative.

AST/resolution/typed HIR contain an explicit statement form. The single ownership
checker uses bounded recursive structured flow and transactional backedge cleanup;
a failure restores availability, provenance, finite uses, obligations, holds,
allocator, lexical scope, and loop target/header context. MIR reuses Goto to the
innermost header, emitting joins/suffixes only for real fallthrough paths. This
settles no labels, break, loop values, early returns, generalized CFG/fixed-point
ownership, MIR borrowing, reborrowing, destruction, execution or verification.


---

## BD-032 — Structured `break` in `while` Loops

**Status:** Accepted

An unlabeled `break;` transfers control to the continuation of the innermost
enclosing ordinary `while`. Every break edge is an explicit loop-exit edge and
must normalize to the same canonical ownership/reference exit state as the loop's
ordinary condition-false exit. Before any permitted normalization reduction that
could erase evidence, the checker must validate the corresponding ownership,
provenance, recurrent-frame, nesting, and operation-hold integrity against the
saved target-loop context. Target-loop iteration-local state is then cleaned,
exactly the target loop's recurrent-obligation frame is discharged, enclosing
recurrent frames are preserved, and the target loop's outside finite continuation
is applied before expiry and canonical-exit comparison. Recurrent protection
remains active until the break statement is reached; BED-017 does not retroactively
weaken earlier operations on a path that later breaks. An unconditional break
terminates its current lexical statement block. BED-017 introduces no labels,
break values, loop values, generalized unreachable-code analysis, generalized
cyclic ownership/lifetime fixed point, arbitrary exit-state merging, path-sensitive
relaxation of pre-existing non-Copy loop ownership, reborrowing, or new MIR terminator.

The canonical false exit is derived independently from the checked header
condition, outside finite continuation, and enclosing frames before inspecting
break candidates. Normalization is a restricted, non-repairing projection of the
actual candidate: carried availability/provenance and enclosing obligations cannot
be reset to manufacture equality. Integrity checks precede local cleanup, exact
frame discharge, expiry, and safe allocator normalization whenever those reductions
could erase corruption evidence. Holds must balance; newly allocated provenance
must be absent before allocator reuse. Failed validation restores the complete
entering candidate, including an injected defect, rather than repairing it.

Finite same-iteration suffix uses skipped by break are removed before preceding
operations on that path; outside finite uses and separate recurrent LoanId
obligations remain authoritative. Recurrence is not discharged early. Nested
break removes exactly one frame even when that frame is empty. Original carried
LoanId/owner/kind remain continuously active whenever an enclosing obligation or
post-loop use requires them; recurrence-only loans may end at exit without
reactivation. Lexical cleanup adds no runtime destruction.

Only actual fallthrough arms participate in a suffix join. Both-break and mixed
break/continue conditionals have zero lexical fallthrough. A while retains its
checked condition-false exit even if all body paths terminate or its condition is
a Boolean constant. Bottom backedges and explicit continue backedges keep the
existing exact header-state rule; break exits instead compare to canonical false
exit. Pre-existing non-Copy Move restrictions remain unchanged on terminal paths.
AST, resolution, and typed HIR preserve Break/span and canonical source identities.
MIR uses existing Goto to the innermost loop exit, with no synthetic join for
all-terminal arms. Ownership remains above MIR in the single structured checker.
OQ-022/OQ-024/OQ-025/OQ-026 retain broader control-flow and lifetime design.

## BD-033 — Structured Early Return in Ordinary Functions

**Status:** Accepted (frozen BED-018, Issue #37).

> An explicit `return expression;` may terminate any statement path in an ordinary
> function. After lexically targeted loop transfers are consumed, an ordinary function
> must have no structural fallthrough to its closing brace; every finite
> function-completing path ends in an explicit correctly typed return, without asserting
> that execution terminates. Structured `while` always retains its condition-false path
> for this analysis regardless of constant conditions. A return terminates its lexical
> block and contributes no ownership state to later branch, loop, or suffix joins. The
> finite continuation immediately before `return E;` consists only of the finite uses
> required to evaluate `E`; every bypassed lexical, loop-outside, enclosing-loop, and
> function continuation is excluded on that path. All active current and enclosing
> recurrent obligations remain authoritative throughout evaluation of the actual return
> operand, including nested calls and arguments. Terminal-return permission suppresses
> only the existing restriction on transferring pre-existing owned non-reference Move
> places within the actual return-expression tree; it does not retroactively authorize
> earlier loop-carried Move changes and does not bypass availability, borrowing,
> reference-transfer, provenance, or operation-hold checks. Exact return typing remains
> in the existing typing phase before ownership. On the ownership side, the return-entry
> snapshot is taken after operand-only finite-summary installation and ordinary expiry
> but before terminal-return permission is enabled; all fallible return-specific
> ownership validation remains inside that transaction, and terminal permission is
> restored on both success and failure. Only after successful operand and
> return-specific ownership validation does the function path terminate, with no
> canonical local successor normalization or ownership-state merge. BED-018 reuses the
> existing MIR `Return` terminator and introduces no implicit returns, divergence/bottom
> semantics, generalized unreachable analysis, arbitrary function-exit ownership
> joining, generalized cyclic ownership/lifetime fixed point, reborrowing, destruction,
> SSA/phi machinery, second type checker, or new MIR terminator.

BD-033 refines the earlier single-final-return boundaries of BD-019, BD-023,
and BD-027; their other typing, scope, loan, and loop rules remain unchanged.


## BD-034 — No-Value Ordinary Functions and Call Statements

**Status:** Accepted (frozen BED-019, Issue #39, including K19-01 and accepted implementation-pressure constraints).

> An ordinary function with no declared `-> Type` is explicitly NoValue: it
> produces no expression value and may complete either by reaching its function
> closing brace or by executing `return;`. A call to a NoValue ordinary function
> may appear only as a standalone call statement; value-producing calls may not
> be silently discarded, and NoValue calls may not enter expression typing.
> No-value call statements preserve the complete existing ordinary-call ownership
> transaction, while bare return is a terminal function edge with an empty finite
> continuation, no operand, no terminal-expression Move permission, no local
> successor, and no destruction/cleanup semantics.

Function results carry explicit NoValue or Value(T) metadata across AST,
resolution, signature lookup, typing and HIR, separate from expression types.
Omitted arrows do not infer result types. Exact argument and return typing remains
in typing; manually constructed ASTs must receive the same diagnostics. Forward
and recursive statement calls are structurally fallthrough-capable, without
termination or divergence inference. Nested closing braces remain local fallthrough.

B19-01 requires a whole prepared-entry statement-call transaction covering all
arguments, receiving holds (including nested-call preservation), release, expiry,
and fallible final validation. Failure exactly restores availability, mappings,
loans/holds/provenance, finite uses, allocator, recurrence, loop and terminal
context. Arguments participate in both finite-use counting and recurrent-reference
discovery. Neither preceding/final calls nor bare return receive BD-033 terminal
Move permission. `finite_before(return;) = ∅`; ordinary expiry precedes the bare
return snapshot, and completion validation/rollback stays within that boundary.
Current/enclosing recurrence remains authoritative. NoValue top-level fallthrough
requires balanced holds, without repair, frame normalization, synthetic exit
joining, cleanup or destruction. Existing loop false exits and local transfers,
real suffix joins, and BD-033 value-return behavior remain unchanged.

MIR uses distinct no-value call statements and honest no-value completion,
without dummy expressions, hidden result locals, or expression-type additions.
Lowering is function-kind-sensitive; already-terminal flow emits no unreachable
completion, and call argument trees receive exhaustive existing validator checks.
Ownership remains in the single structured checker above MIR. Unit/void types,
general expression statements, reference returns, destruction, generalized CFG
ownership/fixed points, interprocedural termination, and execution remain outside
this decision. Earlier BD sections retain their historical boundaries; BD-034
refines ordinary result/call/completion behavior without reopening other decisions.


## BD-035 — Ordinary Record Construction and Copy-Field Reads

**Status:** Accepted (frozen BED-020 Draft 4, Issue #41; K20-S01–S03,
K20-A01–A07 and K20-F01–F04 accepted).

> An ordinary record value may be constructed from an exactly typed field
> expression and may expose its Copy field through a direct named owned place.
> Construction produces a fresh owned record and participates in the existing
> complete-expression transaction. Copy-field access requires an available root
> that is not protected by an active exclusive whole-record loan, copies the
> field without moving or partially consuming the record, creates no field loan
> or reference provenance, and preserves the record's ownership state. Shared
> whole-record loans remain compatible with the read. Direct Copy-field reads
> may participate in ordinary Copy-valued conditions, while construction and
> other prohibited non-Copy activity remain excluded from recurring while conditions.

The current declaration schema remains exactly one field of a declared named
range. `RecordName { field: initializer }` is an ordinary expression wherever the
existing position rules permit that record value; nested conditional values do
not become legal beneath it. RecordId/FieldId and exact nominal initializer typing
are established before ownership. Equal bounds or field spellings do not confer
compatibility. Bare integer literals remain non-contextual (OQ-021); already
range-typed arithmetic may initialize a field without proving its bounds.

Construction is a fresh owned Move value, not an alias/reference, harness binding,
anonymous public local, new record type, allocation, executable constructor or
runtime hook. Initializer effects inherit complete-expression/BD-033/BD-034
rollback, including successful well-typed prefixes before a later ownership failure.
There is no parallel transaction engine. Recurrence and terminal permission rules
are unchanged; wrapping a prohibited operation in construction does not authorize it.

Copy-field roots are canonical ordinary owned ParameterIds/LocalIds only. Reads
require available roots, including all-path conditional availability, and reject
active exclusive loans required by finite uses, receiving holds or current/enclosing
recurrence. Shared loans allow immutable Copy reads without being weakened.
Reads create no field-level availability, transfer history, partial move, field
loan, provenance or hold. Whole-record moves and lawful loan expiry retain their
existing rules. Copy reads are legal in while conditions and stable body/backedge
states; construction is prohibited at any depth in a recurring condition.

The typed-HIR boundary validates canonical owner/record/field/result relationships
before ownership. Recursive visitors traverse construction children; MIR validates
structural trees, including hidden conditionals, without reconstructing HIR tables.
Public exclusive-loan witnesses require independently valid controls; recurrent
zero-finite/zero-hold protection and malformed Move-result metadata have separate
private evidence. Public field-declaration rejection is not partial-move evidence.
Verified battery behavior and harness literals remain isolated and unchanged.

Multiple/default/positional fields, broader field types, structural compatibility,
contextual literals, field borrowing/mutation, reference/temporary/nested projection,
general places, partial moves, non-Copy replacement, destruction, allocation,
execution, code generation and generalized cyclic analysis remain excluded. This
settles no general runtime initializer ordering or invariant/mixed-assurance rule.


## BD-036 — Multiple-field records and complete named construction

**Status:** Accepted

BED-021 Draft 3 (Issue #43) is frozen at baseline
`d83f0067eadd784cda2037340c9ff03276eeb0c5`, including K21-S01–S02,
K21-A01–A09 and K21-F01–F05. It extends BD-035's historical single-field
ordinary schema to one or more fields, each of a declared named-range type.
Field spellings are unique within a record. Commas separate declarations and
construction entries; a trailing comma is optional. Empty records are unsupported.

Declaration order is canonical metadata, determining FieldId allocation,
enumeration and missing-field diagnostic order. It establishes no layout, ABI,
serialization, reflection or destruction order. Named construction supplies every
field exactly once. Entries may appear in any order and retain written source
order through AST, resolution, typed HIR, ownership and MIR. Membership and
duplicates are checked in written order, completeness in declaration order, then
initializer types in written order, before ownership begins. Each initializer
must already have its selected field's exact nominal range type. Equal bounds,
structural record similarity and bare literals establish no implicit compatibility.
Copying an exact-typed value from another record's field is permitted; it does
not reuse the source FieldId as the destination's identity.

Initializers are evaluated left to right in written order inside the existing
complete-expression transaction. If a later initializer fails, all successful
prefix effects restore to the prepared entry: availability, loans, provenance,
holds, recurrence, allocator, target context and terminal permission. No destination
is published before all entries succeed. Completion adds no fallible balancing
stage, independent hold, transaction engine or permission, and preserves legitimate
enclosing call holds and recurrent obligations. BD-033 return and BD-034 whole-call
transactions remain authoritative. This orders record initializers only, not
arbitrary argument/operator evaluation, destruction or runtime unwinding.

A successful construction produces one fresh whole-record Move value, without
field owners, partial availability, field loans/provenance, harness bindings or
anonymous public locals. Any declared field supports direct named-owned
ParameterId/LocalId Copy access with its exact range result. Availability and
exclusive whole-record loans are checked; shared loans are compatible. The root
remains available until an ordinary whole-record Move. Copy reads may participate
in recurring while conditions; construction remains prohibited there at every
depth. Loop-body construction preserves existing recurrence and backedge rules.

Typed-HIR publication first validates every record/field declaration table,
including unused declarations: canonical table IDs, nonempty unique member vectors,
unique field spellings, valid range types, reciprocal ownership and exactly-once
field coverage. Checked lookup rejects malformed tables without panic or repair.
Construction validation checks nonempty complete unique field sets, canonical
membership, exact metadata, eligibility and recursive expression placement.
MIR preserves ordered typed trees and recursively validates every child without
reconstructing unavailable HIR tables. Verified state and the literal harness
remain explicitly restricted to their supported single-field shape.

Direct construction and field reads are transparent data primitives, not an
accessor/factory doctrine. Field syntax invokes no user code, property, allocation,
I/O, locking or invariant hook. Readability, constructibility and mutability remain
separate future authorities. Broader field types, defaults/positional/spread forms,
partial moves, field borrowing/mutation, reference/temporary/nested projection,
generalized places, contextual literals, invariants, visibility, layout,
destruction, execution, code generation and generalized cyclic ownership remain
outside this decision. BD-006 and OQ-004/OQ-012/OQ-016/OQ-019–OQ-026 retain their
unresolved scope. Ordinary records are checked/lowered, not executed or proven.


## BD-037 — Contextual named-range literal formation at canonical value boundaries

**Status:** Accepted (frozen BED-022 Draft 1, Issue #45; K22-A01–A09 accepted).

At an existing ordinary value boundary independently requiring exactly one
canonical named range R, a direct representable signed integer literal n forms
R iff R.minimum <= n <= R.maximum (inclusive). Eligible boundaries are ordinary
value returns, value/NoValue ordinary call arguments, assignment to an established
mutable range local, write-through to an established range referent, and named
record-field initializers. Surrounding eligibility/capability rules apply first.
Parentheses preserving the literal leaf do not defeat formation.

This is formation of a literal occurrence, not conversion of an existing value.
Nominal identity remains exact even for equal bounds. A single bounded value check
forms direct literals; its fallback synthesizes normally and checks exact types.
Expectations never propagate through operators, conditionals, names, dereferences,
field reads or call results. Nested calls/constructions may select their own
independent parameter/field expectations. Unannotated literal locals, later-use
inference, sibling inference, annotations, constant evaluation, general numeric
negation, casts/coercions and nonliteral formation remain unsupported.

Typed HIR distinguishes raw IntegerLiteral from FormedRangeLiteral, with canonical
RangeTypeId stored once in the latter's Range expression type. Table-aware HIR
validation rejects forged tags, invalid/noncanonical IDs, out-of-bounds values and
formed verified descendants without panic or repair. MIR preserves that distinction
and validates leaf/type shape recursively, without claiming unavailable table facts.
A formed literal is inert Copy: no owner, transfer, loan, provenance, hold,
allocator, finite-use, recurrence or terminal-permission effect. Bounds failure
is Type before ownership, never an ownership-rollback witness. Existing complete
transactions, written initializer order, call/write holds, target checks and
recurrent/terminal authorities remain unchanged.

Raw operator constants retain existing behavior: p <= 101 and p - 101 do not
form 101 as p's range. Formation proves only its selected literal is in range;
it proves no arithmetic result, overflow safety, runtime check or execution.
Signed i64 representation and earlier lexer/parser failures are unchanged.
Verified-body raw constants and the scalar literal harness remain separate and
unchanged, including their eligibility, phases, proofs and diagnostics. This
narrowly refines historical BD-025/BD-026/BD-035/BD-036 literal boundaries;
OQ-021/OQ-022 retain all broader arithmetic/inference/conversion questions.


## BD-038 — Direct Copy-field assignment on mutable owned record locals

**Status:** Accepted (frozen BED-023 Draft 2, Issue #49, October 9, 2026).

A directly named mutable owned ordinary record local may replace one declared
Copy named-range field with an exactly typed expression. Canonical LocalId,
RecordId and FieldId identify the operation. Parameters, immutable bindings,
reference/non-record roots, temporary/nested projection and compound/expressional
assignment remain unsupported. BD-037 contextual formation applies at the selected
exact field boundary, not recursively through arbitrary expressions.

Evaluate the complete RHS first, with no target reservation, hold or loan. Perform
ordinary expiry, then check non-consuming root availability and whole-record
mutable conflicts. Live shared/exclusive loans block the write; unused/completed
handles may expire lawfully. Held and recurrent provenance remains authoritative.
The RHS may read the old field, complete temporary borrows or consume unrelated
owners; a Move of the target makes final availability fail.

A dedicated snapshot at prepared field-operation entry covers RHS evaluation,
post-RHS expiry and both final checks. Any ownership failure restores availability,
transfer history, handles, loans/provenance/holds, finite uses, recurrence, allocator,
loop/target context and terminal permission exactly. This boundary is independent
of broader enclosing transactions. Successful lawful RHS effects remain committed.
The write preserves the same whole owner and creates no replacement transfer,
field owner/availability/provenance/loan, partial move, handle or terminal permission.

This ordinary fallthrough statement composes with existing branches, stable loops,
break/continue and returns without value joins, field-sensitive ownership or a
second checker. RHS-only finite and recurrent traversal remains complete. Public
AST/HIR/MIR have distinct field-assignment variants; table-aware HIR validates
canonical identity/membership, root mutability/type and exact recursive RHS metadata
without inference or repair. MIR honestly validates structural RHS facts without
claiming canonical tables. Verified state/harness remain unchanged and single-field.

This settles transparent direct data mutation only, not setters/properties,
reference projection, field loans, generalized places, Move-field replacement,
whole-record replacement, invariants, visibility, destruction, runtime execution,
proof or generalized cyclic analysis. Existing BD sections retain their historical
boundaries; this narrows OQ-025 only. K23-S01–S04, K23-I01–I05 and K23-A01–A05
are preserved as frozen implementation/evidence obligations.


## BD-039 — Direct Copy-field reads through named record references

**Status:** Accepted (frozen BED-024 Draft 1, Issue #51, October 9, 2026).

A directly named ordinary parameter or local of exact `&Record` or `&mut Record`
type may read a declared named-range Copy field. Selection uses the referent's
canonical RecordId and its own FieldId; the result has the field's exact nominal
range type. Identical spellings/bounds do not unify records, fields or ranges.
Owned-root Copy reads and direct mutable owned-local field assignment retain
BD-035–BD-038 semantics.

The dedicated referenced read uses the available handle's existing continuously
active whole-record loan and validated provenance. It moves neither handle nor
record, preserves shared/exclusive permission, and creates no loan, reborrow,
projected place, field provenance, hold, owner or terminal permission. Reading
through `&mut Record` is access under its exclusive authority, not conversion to
`&Record`. Whole-record non-Copy dereference extraction remains forbidden.

Each read is a real handle use in finite-use and recurrent-reference traversal.
Ordinary last-use expiry, alias/transfer lineage, receiving holds, current and
enclosing recurrent obligations, branch isolation and joins remain authoritative.
Stable body reads compose with existing loops/continue; reference-backed reads
remain forbidden in recurring conditions even though their result is Copy.
The existing complete-expression, BD-033 return and BD-034 call transactions
restore all prepared state after later failure, including loans expired by a
successful prefix. The leaf introduces no separate transaction or completion gate.

AST/resolution retain complete-expression and distinct field-token spans. Typing
classifies canonical named roots using existing inferred types and selects fields
record-locally. HIR and MIR distinguish CopyReferenceFieldRead from CopyFieldRead.
Table-aware HIR publication validates checked root/function/referent/record/field/
range relationships and exact results without repair or panic; MIR checks only
its structural leaf/result and recursively nested trees, not absent canonical
or ownership tables. Public inspection remains read-only.

Private canonical loan-origin metadata and initializer lineage validate external
parameters, shared aliases and exclusive transfers without requiring the moved
source to remain available. Place → LoanId remains the sole active-loan authority;
compatible loans are not interchangeable. Origin integrity participates in cloned
transactions, recurrent/break integrity and header/backedge comparison.

BD-040 subsequently permits direct named exclusive-reference writes. Shared-
reference field assignment, field borrowing, temporary/nested/dereference projection, auto-deref, coercion/reborrowing, partial moves/loans, broader field
types, reference returns, record replacement, properties, destruction, execution,
code generation and generalized cyclic inference remain excluded. The specialized
verified-state/harness paths, eligibility, proof counts and affine kernel are
unchanged. This refines OQ-024 only within the named Copy-field read boundary;
historical accepted sections retain their earlier scope. Merge authorization
remains separate from this accepted implementation decision.

## BD-040 — Direct Copy-field assignment through named exclusive record references

**Status:** Accepted (frozen BED-025 Draft 2, Issue #56, October 10, 2026).

A directly named ordinary parameter or local of exact `&mut Record` type may
assign one declared named-range Copy field. Binding mutability is unnecessary:
mutation authority belongs to the reference. The actual referent's canonical
RecordId selects the FieldId and exact nominal RangeTypeId. Direct contextual
literals form only at that independently established boundary (BD-037).
Shared and non-record references reject. BD-038 owned-local writes and BD-039
reference reads retain their distinct behavior.

A dedicated prepared-operation transaction validates the available target and
live canonical lineage, captures its existing exclusive LoanId, counts one target
use, and retains that loan before any expiry. It evaluates the complete RHS and
then independently revalidates target availability, live lineage and the exact
original LoanId before authorizing the write. A held loan cannot substitute for
an available handle. Success releases only the operation's hold and applies
ordinary expiry. Failure restores the complete prepared entry, including finite
uses, holds, allocator, transfers, provenance, recurrence and control context.
The immutable initializer index remains shared structural metadata.

Target preparation, retention, validation and the write allocate no loan or
ownership transfer. Lawful temporary borrowing and unrelated moves in a successful
RHS remain committed. Same-handle field reads use the existing exclusive authority;
original-owner conflicts remain forbidden across the complete operation. Exclusive
recipients validate the correct transfer edge without requiring moved sources to
remain available. Place → LoanId remains the sole active-loan identity authority.

Finite and recurrent traversal include the target and every RHS use. Stable loops,
nested frames, continue and break preserve existing lineage and discharge gates;
recurring conditions remain reference-free. HIR/MIR distinguish
CopyReferenceFieldAssign from owned CopyFieldAssign and whole-value DerefAssign.
HIR validates canonical relationships and complete RHS metadata with checked IDs;
MIR validates structural facts without canonical-table or provenance claims.

This settles only direct named Copy-field writes through existing exclusive
whole-record authority. Reborrowing, coercion, field loans/provenance, projected
places, partial ownership, compound or expression assignment, whole-record
replacement, properties/invariants, general lifetime inference, execution and
code generation remain unsupported. Verified-state and harness representations,
eligibility, proof counts and the affine kernel are unchanged. This narrowly
refines OQ-024/OQ-025; historical BD boundaries remain historical. Independent
implementation review and Greg's merge authorization remain separate stages.

## BD-041 — Deterministic bounded ordinary MIR execution

**Status:** Accepted (BED-026 Draft 3; frozen and implementation-authorized in
Issue #58, comments 6110717709 and 6110730498).

Ordinary execution is a bounded tool operation over an opaque artifact produced
by the existing complete frontend and ownership pipeline. The complete currently
accepted ordinary MIR subset executes with canonical nominal metadata, dynamic
frame/place locations, distinct consuming/non-consuming access, ordered resumable
evaluation, and no runtime borrow checker or transaction. MIR controls branches,
initializers, loops, break, continue, and returns. Shared handles copy; exclusive
handles transfer. Prefix mutations survive later traps/stops; incomplete values
are not published. Each subtraction checks its widened mathematical result before
materialization. Short-circuit operators skip all unused RHS work and fuel.

Finite preparation limits begin before input accumulation; alias depth and total
work are independently bounded. Runtime fuel charges statements, terminators,
expression nodes, and function entry. Origins and bounded traces are compilation
bound metadata. Limits are tool contracts, not nontermination proofs or language
exceptions. `run` selects a zero-parameter ordinary entry and never invokes proofs,
verified functions/harnesses, verifier state, or the affine kernel. Existing `check`
and `verify` behavior remains unchanged. See `docs/BOUNDED_EXECUTION.md` for APIs,
accounting, supported maxima, reporting, and host-failure exclusions.

This adds no executable source subset, generalized places, partial ownership,
reborrowing, runtime loans, destruction/cleanup, heap semantics, source effects,
exceptions, I/O, concurrency, or native code generation. Independent review and
separate merge authorization remain required.

## BD-042 — Deterministic scope cleanup for owned resources

**Status:** Accepted (BED-027 Draft 1, Issue #60; frozen October 11, 2026 at
baseline `06fa4ac8491ae297981051b0544299bfe849c7f2`, tree
`f2e405a4a22e4a27d525950cb47bb9cec904e3b3`).

Bowfin uses RAII-like deterministic scope cleanup for resource-bearing owned
values. A type is resource-bearing exactly when canonical compiler metadata
identifies it as a trusted resource leaf or it is a record containing at least
one resource-bearing field. Structural classification is recursive. Every
resource-bearing type is Move and cannot be Copy; an ordinary non-resource Move
type has semantically inert destruction.

A successfully initialized owned resource-bearing value carries exactly one
cleanup responsibility. Moving the value transfers that responsibility without
running cleanup at the source. Borrowing neither transfers nor duplicates it,
and reference last use or loan expiry never determines destruction. A trusted
resource API may provide an explicit consuming release operation that discharges
the responsibility early, but there is no generic manual-drop operation.

Each resource leaf has one fixed release operation selected by canonical type
identity at a trusted runtime, standard-library, or equivalent boundary. It
accepts no user callback, returns no source-visible result, cannot report a
recoverable source-level failure, cannot transfer new authority into the program,
and cannot silently perform fallible completion. Host inability to perform that
release is a runtime or tool boundary failure, not successful cleanup or completion.

A resource-bearing record is one complete whole owner of its constituent cleanup
responsibilities. Compiler-synthesized cleanup visits fields recursively in
reverse canonical declaration order, skips inert fields, and invokes no arbitrary
record-level destructor body. Whole-owner source semantics remain intact: this
does not introduce field-sensitive availability, partial moves, or independently
addressable source-level field obligations.

A cleanup responsibility begins only after successful initialization. Pending
record construction owns every resource-bearing field value that has completed;
the complete record is published only after every required field succeeds. If
future typed recoverable failure exits construction, the actual initialized field
set cleans in reverse canonical declaration order. That set need not be a
declaration-order prefix, and cleanup never restores moved sources or implies
runtime rollback.

Every still-live owned resource-bearing value cleans exactly once when its owning
lexical scope is exited through ordinary block fallthrough, implicit NoValue
completion, explicit value or NoValue return, break, continue, or future typed
recoverable propagation. Break and continue clean only scopes actually exited.
Loop-carried owners whose scope continues remain live. Owned parameters clean at
function exit unless already moved or consumed.

Cleanup order is deterministic: exited inner scopes precede outer scopes; within
one scope, locals clean in reverse successful-initialization order; function
locals precede parameters; parameters clean in reverse declaration order; and
record fields clean recursively in reverse declaration order. A value return
fully evaluates and transfers the result into protected return-result ownership
before remaining locals and parameters clean. The callee never cleans that result.

Automatic cleanup relinquishes the resource or authority. It never asserts
successful flush, commit, publication, durable synchronization, or protocol
completion. Any completion whose success matters remains an explicit fallible
program operation. Ordinary field syntax and lexical exit dispatch no arbitrary
user-authored code.

The compiler must represent cleanup through explicit auditable cleanup edges or
equivalent MIR operations on every covered structured exit. Correspondence
validation must reject malformed cleanup structure rather than repairing it.
Execution follows validated MIR; it must not reconstruct lexical cleanup by
walking HIR or resolve destructors by textual name. The exact instruction shape,
trusted registration representation, and bounded executor mechanics remain
implementation choices under these semantic constraints.

BD-042 guarantees no cleanup across range traps, configured preparation/execution
stops, admission or integrity failures, detected host/tool failures, hostile host
or process termination, panic/exception unwinding, or task cancellation. It adds
no user-authored destructor bodies or callbacks, `defer`, `finally`, garbage
collection, nondeterministic finalization, non-Copy replacement, overwrite
destruction, partial moves, generalized projection, implicit reborrowing, heap
model, FFI, operating-system resource syntax, `Result`/propagation syntax,
effects system, concurrency, cancellation, or production resource API. Resource-
bearing expression-temporary destruction points require a focused later decision
before those forms are admitted.

Sarah's architectural review found no specification finding and returned READY
FOR IMPLEMENTATION-PRESSURE REVIEW. Arnold's independent read-only pressure review
found no K27-P finding and returned READY TO FREEZE. Freeze does not authorize
implementation. A later directive must bind canonical metadata, trusted resource
admission, HIR/MIR representation, cleanup synthesis and correspondence, executor
behavior, bounds, diagnostics, temporary admission, synthetic evidence, branch/base,
validation, reporting, and phase structure. Any newly exposed semantic choice
returns to Greg rather than being decided in code.

## BD-043 — Deterministic replacement of owned Move locals

**Status:** Accepted from BED-028 Draft 1, Issue #64; human-merged October 15,
2026 as commit `72c34593e79e7763ce16f0e18ad99595959a2454`, tree
`2b9339fbe21f705cb8c5722ff681c34680870490`.

An available, directly named mutable owned non-reference Move local may receive an
exact canonical record value through the existing assignment statement. Parameters,
immutable bindings, references, projections, dereferences and unavailable slots
remain excluded. The complete RHS evaluates while the old owner stays in place;
the target may be borrowed or read non-consumingly but cannot transfer or be
explicitly consumed within its own RHS. Existing last-use, receiving-hold and
recurrent-loan rules apply. No active target loan may remain at commit. Rejection
restores the exact prepared checker state, not runtime effects.

The result enters protected machine ownership before displacement. The old whole
owner detaches and performs its fixed canonical cleanup, recursively in reverse
field declaration order if resource-bearing; inert Move destruction executes no
cleanup. Only then does the protected value publish into the same LocalId as a
new initialization generation. At later scope exit, only the current live generation
cleans, in reverse successful publication order within the target's owning dynamic
activation. A/B/C ordering is acquisition A, B, C; release displaced A; at scope exit
release current C, then B. History owns no duplicate payload.

Iteration-local owners may be replaced repeatedly. A target or moved RHS source
present at the current loop header remains protected by BD-043's relative boundary,
including nested loops and paths followed by return, break or continue. The complete
replacement-result root is one additional resource lifetime boundary; unnamed
resource arguments and arbitrary expression temporaries remain excluded.

Explicit MIR replacement authority binds canonical target/type, lexical scope,
cleanup plan, protected/displaced ownership protocol and origin. Opaque approval
and deterministic correspondence reject malformed artifacts. Terminal trap, limit,
integrity or Host outcomes preserve prefix effects and uniquely retained remaining
responsibilities, refuse resume and provide no unwind or successful continuation.
This adds no take/swap, self-consuming transformation, moved-slot reinitialization,
partial move, reference replacement, destructor callback, production resource API,
recoverable propagation, generalized place/dataflow engine or verifier expansion.
Earlier BD sections retain their historical boundaries; this decision narrowly
extends BD-026 and BD-042 and refines OQ-022/OQ-023/OQ-025/OQ-026.

## BD-044 — Deterministic resource-bearing expression temporaries

**Status:** Accepted from BED-029 Draft 2, Issue #66; human-merged October 16,
2026 as commit `5ce5dc5895efab7ba29a7cbc8e980ce58a7b24ab`, tree
`e95fb9e69d7554719d86d9662cf22cbf5151b84b`.

This narrowly refines BD-042 and BD-043 temporary boundaries while preserving
BD-041 argument order and BD-034 statement restrictions.

Every admitted resource-bearing expression intermediate has one explicit bounded
machine owner. Ordering history owns no payload. Static source regions and sites
are distinct from runtime activations and owners. Nested caller expressions remain
in their caller region; executed callee statements and return operands get their
own regions. Conditional initializers retain one region across MIR blocks. Each
record-field initializer has a child region and transfers its result into pending
construction before that child retires.

Owned arguments evaluate left to right, remain responsible while later arguments
evaluate, and transfer exactly once during valid whole-call entry. Remaining
sibling owners clean in reverse successful creation order. Cleanup inside complete
aggregates remains reverse canonical field-declaration order. Local initialization
protects the result, retires other temporaries, publishes and registers the local,
then continues. Existing assignments similarly protect, retire and perform their
pending write. Returns and BD-043 replacements use their protected holders before
operand retirement. Conditions protect their Bool and retire before branch selection.

Temporary borrowing and escape remain rejected at every representation layer.
Temporary owners are not hidden user locals or borrowable places. Recurring while
conditions gain no Move/reference eligibility. Value-returning call statements
remain rejected. Vacancy never establishes valid transfer or cleanup. Terminal
stops retain actual unfinished responsibilities and completed prefixes; no unwind,
rollback, retry, arbitrary destructor, public resource API, ownership inference or
second ownership checker is added. See the BED-029 implementation ledger and
accepted exact-head review for evidence. OQ-022/OQ-023/OQ-025/OQ-026 retain their broader questions.

## Frozen BED-031 — Typed recoverable failure and explicit handling

**Status:** Frozen Draft 4, implementation authorized in [Issue #72](https://github.com/gregkrsak/klyxr/issues/72).
This implementation awaits independent review and Greg's acceptance. BD-007's
broader Direction Accepted status is unchanged.

The fixed built-in `Result<success: S, failure: F>` is one mutually exclusive
ordinary owned Move value, including when both payloads are Copy. Payloads are
exact `bool`, named ranges, records, or finite nested Results. The labels are
mandatory in that order. `Ok(...)` constructs success; `Error(...)` and `Err(...)`
are equal official aliases normalized immediately to one canonical Error variant.
Only those exact syntax spellings and `Result`/`match` are accepted. Their complete
ASCII case-insensitive spelling families are reserved in every identifier context;
nearby names such as `ErrorCode` remain available. The labels are contextual.

Construction requires an independently established complete expected Result at
an exact ordinary return, parameter, or recursively established payload boundary.
The active payload cannot infer the inactive type. Constructor payloads create no
new BD-037 named-range literal boundary. BD-025 Result-valued conditional
initializers, Result replacement, Result references, public record Result fields,
verified functions and harness Result use remain excluded.

The closed statement `match value { Ok(binding) => { ... } Error(binding) => { ... } }`
accepts either written arm order and either failure alias. Exactly one canonical
arm of each kind is required. No wildcard, guard, nested/ref/alternative pattern,
match expression, or value-producing arm is added. Each arm has a fresh lexical
LocalId, exact payload type, and no visible-name shadowing. Matching consumes the
wrapper once, copies a Copy payload or transfers a Move payload, and executes
only the selected arm. Both bodies are statically checked. An inner Result in the
selected binding carries its own independent obligation.

The existing ownership checker tracks unresolved outcomes separately from Move
availability, loans, cleanup, effects, and vacancy. Every successful exit must
match, return, pass to an exact Result parameter, or transfer to another owner.
Checks precede scope reduction, break/continue projection and backedges; unioned
pending facts preserve unhandled branch and short-circuit alternatives. Cleanup
cannot acknowledge an outcome. Existing conservative loop Move rules remain.

Resource-bearingness is the structural disjunction of both declared payloads;
only the active payload owns responsibility. Shared bounded variant plans preserve
ordered roles and nested identities, with cleanup effects unioned over both
alternatives. Whole-wrapper cleanup has only independently sealed primitive/protocol
authority. Ordinary matching transfers cleanup to the selected binding.

Manual propagation uses explicit matching and exact Result return. The outgoing
value is protected before other temporaries and callee owners retire; failed
cleanup prevents delivery. Results add no implicit effect, authority, rollback,
retry or unwind. Tool stops never become Error. General enums/generics, Option,
`?`, conversions, reference payloads, fallible cleanup and production effectful
APIs remain deferred. See the BED-031 ledger for evidence and its limits.

## Frozen BED-032 — Explicit local Result propagation

**Status:** Frozen Draft 2, implementation authorized in Issue #75; independent
implementation review and Greg's acceptance remain separate. BD-007's broader
Direction Accepted status is unchanged.

`let value = E or return;` requires independently typed canonical
`E : Result<success: S, failure: F>` inside an ordinary Value function returning
`Result<success: T, failure: F>`. Exact canonical F is mandatory; S and T may differ.
No conversion or constructor expected-type context is inferred from propagation.
The expression executes once in its statement-local region, its complete Result
is protected and other initializer temporaries retire before tag selection.

Ok transfers/copies its active payload, consumes the wrapper and establishes the
original local in the actual enclosing lexical scope, registering only successful
initialization. Error transfers/copies exact F into canonical outer Error, protects
that result, consumes the input and uses the existing ordinary return exits.
Callee scopes/parameters clean before frame retirement and caller delivery; cleanup
stops retain the outgoing result and unfinished responsibilities without resume.
Equal complete Result types permit equivalent wrapper reuse, but this implementation
uses the same canonical rewrapping protocol for both equal and differing types.

The illustrative match in the BED is tag-dependent control flow, not a lexical
rewrite. No propagation-arm scope exists. BD-042/BD-043 over actual activations
yield direct A → C → B and unchanged explicit match A → B → C; a failed C release
retains B without an attempt. The existing ownership checker preserves independent
nested outcomes, earlier pending obligations, exact nominal types, loans, original
replacement activations and relative loop restrictions. Initializer evaluation
gains no terminal-return Move permission. The success path still requires its
own explicit function return and ordinary break/continue/backedge checks.

The exact lowercase contextual phrase does not reserve `or` globally. Existing
fn/function, mut/mutable and admitted trivia remain equivalent. Propagation is not
an expression; `?`, `try`, fallback/abort spellings, conversions, public resource
APIs, Result replacement, references, verified/harness execution, conditional
Result initializers, exceptions, unwind, retry and rollback remain excluded.
All Host, Limit, Integrity, trap and cleanup failures remain tool outcomes.

## Frozen BED-033 — First selected trusted effectful Resource slice

**Status:** Frozen Draft 2, separately implementation-authorized in Issue #78.
The branch supplies implementation evidence; independent exact-head review and
Greg's acceptance remain separate. No new Accepted BD number is assigned here.

One explicit immutable sensor-demo selection contributes callable sensor_open,
sensor_read, exact nominal ranges Reading 0..4095 and SensorError 1..3, and opaque
Move Resource Sensor before ordinary signature collection. Same-namespace collisions
reject without overwrite; cross-namespace spellings and existing case rules remain.
Selected denied rows still occupy their names; unselected rows occupy nothing.
Catalog origins are distinct from source spans. Open returns Result<Sensor,
SensorError> (Error only 1 or 3); read takes shared &Sensor and returns Result<Reading,
SensorError> (Error only 2 or 3). Both effects are io. Read Error leaves the owner
valid. Fixed cleanup-only release has io, no source Result and no source-callable
alias. Complete effects never confer grants; acquisition requires exact release.

Existing ordinary moves/parameters/internal protected returns/shared helpers,
Result payloads and BD-044 custody compose. Ownership-derived cleanup effects
require io on retire(sensor)->bool but not on relay(sensor)->Sensor transfer.
Mutable Sensor references, conditional Sensor initialization, replacement/reinit,
public record fields, representation operations and public live-Sensor egress are
excluded. Whole-Result cleanup does not acknowledge ordinary unresolved outcomes.

Canonical typed trusted calls and MIR plans bind exact signatures, grants,
bindings, cleanup/Result plans, argument destinations, roots and honest origins.
Opaque approval seals composition. Runtime has no source/HIR/name/host fallback.
Only persistent session execution may invoke; an already-owned pre-reserved receipt
holds actual payload across every fallible validation/publication boundary.
External success followed by a stop retains responsibility and completed prefix.
Host, Limit, Integrity, traps, denial and cleanup failure never become source Error.
Protected Results are delivered only after required cleanup succeeds; terminal
advance never resumes, retries or unwinds. Teardown is not successful release.

Bounded deterministic audit follows all structural call edges from sealed roots,
including zero-effect helpers and alternatives, and distinguishes grants, reachable
source-callable use and cleanup-only authority. Catalog, configuration, runtime
receipt/session, audit and diagnostic budgets are explicit even for tiny source.
The demonstration supplies no device proof, general FFI, modules, capabilities,
concurrency, verified/harness trusted execution or arbitrary resource declarations.
Release promises no flush/commit/durability/completion; fallible completion is
explicitly deferred to BED-034. Historical public-resource exclusions above are
narrowed only for this exact frozen family. See its implementation guide and ledger.

## Accepted BED-035 — Approved-MIR execution equivalence, Stage I

**Status:** Accepted Stage I. Greg accepted Sarah's final resolution of K35-I01
through K35-I04 and human-merged PR #85 as
`c2910493ad5e04fb80f5cfa0ae8d3ee80be5d717`, tree
`a76e2ab68c8d2db094f1005906bdc24b6a9b8795`. No new Accepted BD number
is assigned because this establishes an execution-engine contract and tooling
boundary without changing source-language semantics.

Execution consumes the exact opaque approved composite and bound versioned
semantic/accounting/cut/replay identities. Neither source similarity, matching
names, observed runtime behavior nor independently lowered HIR creates authority.
The complete structural ordinary closure includes empty-effect calls and both
alternatives. Existing nominal types, ordered Result roles, ownership/outcome
witnesses, effects, cleanup plans, domains and selected binding contracts remain
canonical; this adds no language expression, pattern, inference or recovery rule.

Complete separately admitted non-owning observations validate local state before
one anchored injective correspondence. Equal and Different require complete valid
evidence; incomplete, malformed, unavailable or exhausted evidence is Inconclusive,
which is a tool result. Logical activation/site/occurrence/phase addresses govern
private conformance replay and modeled Host. Earlier inherited gates retain
precedence; skipped and preempted events consume no controlled item. Controlled
responses create no root, grant, ownership, binding or cleanup edge. Public roots
remain zero-parameter and public live-resource/reference egress stays excluded.

Recoverable Error remains ordinary data. Host, Limit, Integrity, trap, cleanup
failure and excluded terminal stops retain actual prefix, protected outcomes and
remaining responsibilities without retry, rollback, source recovery or delivery.
Detected physical Host with surviving state is not automatically matched input;
substrate destruction carries no fabricated structured-retention guarantee.

Version-one public preparation/run/audit/reporting and exact accounting neighbors
are preserved. New observation, validation, replay, structured audit and presentation
accounts remain separate. Reference representation sizes define the compatibility
schedule on its supported configuration, not a portable native layout. Inlining,
preallocation and eliminated physical work remain possible under the fixed logical
relation. The interpreter is an implementation under test, not a semantic oracle.

No backend/target/ABI/emitter/optimizer/runtime/transport or packaging design is
chosen. Actual native production, installation, runtime binding, optimized and
unoptimized equality and physical-failure evidence remain mandatory Stage N
obligations under a later authorization. Stage I evidence is finite interpreter,
independent-oracle, protocol and malformed-boundary evidence, not universal proof.


The accepted Stage I evidence reconciles 2,045 predecessors, 48 original K35
controls, 11 first-repair identities and 8 final I03/I04 controls: 2,112 passing
identities in each complete debug and release profile, with zero predecessor
migrations. All exact-head and post-merge Actions gates passed. Sarah's final
focused confirmation resolves I01–I04 and assigns no I05. These finite results
support the accepted Stage I boundary; they remain neither universal conformance
proof nor native execution evidence. Stage N remains separately unauthorized.

<!-- END LANGUAGE_DECISIONS.md -->

---

<!-- BEGIN ARCHITECTURE.md -->

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

<!-- END ARCHITECTURE.md -->

---

<!-- BEGIN OPEN_QUESTIONS.md -->

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

<!-- END OPEN_QUESTIONS.md -->
