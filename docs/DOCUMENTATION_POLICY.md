# Bowfin Documentation Policy

**Status:** Accepted project policy.

## Core principle

> **Documentation is part of the program's human-facing contract. Code and
> documentation are reviewed together, and disagreement between them is a
> defect.**

Bowfin does not assume that source code is self-explanatory. Code is an exact
machine artifact; it is not automatically an adequate explanation for a human
reader. Comments and documentation exist to state purpose, policy, boundaries,
and meaning in language people can read directly.

This policy treats documentation as engineering work. It is not an apology for
unclear code, an optional courtesy, or material to add after implementation is
considered complete.

## Immediate source policy

The current executable prototype accepts ASCII whitespace and `//` line
comments. It does not assign special documentation meaning to `///`, `//!`,
`/** ... */`, Doxygen tags, or another structured comment spelling.

Until a focused language and tooling decision accepts documentation-comment
syntax, Bowfin source examples must use ordinary `//` comments. Repetition of a
candidate spelling does not freeze it.

Comments in public examples and production-oriented source should be
proportionate to the human knowledge the program must preserve. They should
provide, as applicable:

- the purpose of a declaration or algorithm;
- the meaning of inputs, outputs, fields, and status values;
- effects, authority requirements, trusted assumptions, and escape hatches;
- ordinary, recoverable, and terminal failure behavior;
- bounds, invariants, units, ordering, and lifecycle obligations;
- policy decisions such as precedence or classification rules;
- important algorithm phases and the reason for non-obvious sequencing;
- limitations and facts the program does **not** establish.

A comment may describe straightforward code when doing so gives the reader a
clear plain-language account of the program. Bowfin does not adopt a rule that
comments are justified only by unusually difficult code.

Documentation quality is not measured by comment count. Mechanical repetition,
misleading certainty, obsolete prose, and ornamental verbosity are defects too.
The standard is whether a careful human reader receives an accurate and useful
account of the program.

## Public declaration documentation

Public declarations should eventually carry structured documentation describing,
where applicable:

1. purpose;
2. parameters and their domain meaning;
3. return value or completion behavior;
4. effects and required authority;
5. recoverable failures and terminal tool outcomes;
6. ownership, borrowing, custody, and cleanup obligations;
7. preconditions, postconditions, invariants, bounds, and assumptions;
8. important non-guarantees.

A contract clause, type, or effect declaration does not make prose unnecessary.
The machine-readable declaration establishes the formal fact available to the
toolchain; documentation explains its application meaning to humans. Conversely,
prose never substitutes for a required type, contract, effect, permission, or
proof obligation.

## Public example spelling

Bowfin-owned public examples prefer the explicit spelling of an approved alias
whenever that spelling is implemented by the demonstrated grammar.

For the current prototype this includes, where applicable:

| Prefer publicly | Compact equivalent |
|---|---|
| `function` | `fn` |
| `mutable` | `mut` |
| `&mutable T` | `&mut T` |
| `Error(...)` | `Err(...)` |

Do not use an explicit spelling merely because BD-014 lists it if the grammar
being demonstrated does not yet implement it. Examples must not trade correctness
for appearance.

Compact aliases remain equal official language spellings. Tests may deliberately
exercise either form, and compact spelling remains useful where alias coverage is
the subject. This repository policy does not change BD-014's semantic equivalence,
the wider ecosystem default, or an external project's right to choose its own
formatter/linter policy.

## Review policy

Documentation receives substantive review alongside code.

A change is incomplete when it changes externally meaningful behavior, policy,
authority, failure handling, bounds, lifecycle, or guarantees without updating
the documentation that describes those facts.

Reviewers should ask:

- Does the documentation describe the code that will actually execute?
- Does the code implement the behavior the documentation promises?
- Are effects, authority, assumptions, and failure boundaries stated honestly?
- Are partial success and terminal outcomes distinguished?
- Are examples executable under the stated language boundary, or clearly marked
  as illustrative?
- Could a new contributor understand the intent without reconstructing it solely
  from implementation details?
- Did a change preserve or intentionally revise the human-facing contract?

When code and documentation disagree, neither wins silently. The discrepancy is
a defect requiring correction or an explicit design decision.

## Toolchain direction

Bowfin should eventually support structured documentation comments comparable in
purpose to established documentation-comment systems while remaining coherent
with Bowfin's assurance model.

A future accepted design may provide:

- a dedicated source spelling for documentation comments;
- AST preservation and canonical attachment to declarations;
- generated API documentation;
- checked links to parameters, fields, types, effects, and contracts;
- compiled documentation examples;
- lints or project policy requiring documentation for public declarations;
- documentation coverage based on meaningful declarations rather than raw lines;
- editor and audit integration;
- review-visible records of documentation changes.

Where a documentation claim is mechanically checkable, tooling should prefer
deriving or validating it against canonical compiler facts rather than trusting
duplicated prose. Non-mechanical claims remain subject to human review.

Structured documentation metadata must not:

- grant execution or resource authority;
- change ownership, type, effect, or contract semantics;
- create proof evidence merely by assertion;
- conceal trusted assumptions;
- convert a tool failure into a source outcome;
- override canonical compiler artifacts.

The exact comment spelling, attachment rules, metadata schema, generated format,
lint severity, inheritance behavior, macro/generic interaction, and compatibility
policy require a focused future language/tooling decision. This policy establishes
the direction and review culture; it does not silently settle that syntax.

## Public examples as exemplars

Flagship and reference applications should demonstrate this policy rather than
merely mention it. Their comments should form a readable explanation of the
program's purpose, phases, policy, authority, failure behavior, and limits.

Bowfin Sentinel 0 is the first planned reference application expected to apply
this policy comprehensively.
