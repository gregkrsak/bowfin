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
