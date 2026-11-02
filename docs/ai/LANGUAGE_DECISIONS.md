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
