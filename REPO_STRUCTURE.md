# Bowfin Repository Structure

This repository contains the initial Bowfin scaffold plus an executable Rust-based
compiler prototype. Current operational GitHub home: `gregkrsak/bowfin`. The adopted destination is
`gregkrsak/bowfin`; the repository cutover has occurred. A future organization
transfer requires a separate owner decision.

```text
bowfin/
├── AGENTS.md
├── CLAUDE.md
├── CONTRIBUTING.md
├── GEMINI.md
├── LICENSE
├── NOTICE
├── README.md
├── REPO_STRUCTURE.md
├── THIRD_PARTY_NOTICES.md
├── TRADEMARKS.md
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── .github/
│   ├── copilot-instructions.md
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── workflows/
│       ├── build.yml
│       ├── docs.yml
│       ├── tests.yml
│       └── verify.yml
├── assets/
│   ├── bowfin-logo.jpeg
│   ├── diagrams/
│   │   ├── assurance-ladder.svg
│   │   ├── assurance-ladder-mobile.svg
│   │   ├── knowledge-boundaries.svg
│   │   └── knowledge-boundaries-mobile.svg
│   ├── readme-banner.png
│   └── readme-banner-mobile.png
├── legal/
│   ├── ENTITY_CONTRIBUTOR_ASSIGNMENT_AGREEMENT.md
│   ├── INDIVIDUAL_CONTRIBUTOR_ASSIGNMENT_AGREEMENT.md
│   └── README.md
├── compiler/
│   ├── Cargo.toml
│   ├── README.md
│   ├── lib.rs
│   ├── main.rs
│   ├── ast/
│   ├── codegen/
│   ├── diagnostics/
│   ├── effects/
│   ├── hir/
│   ├── lexer/
│   ├── mir/
│   ├── ownership/
│   ├── parser/
│   ├── resolve/
│   ├── types/
│   ├── tests/
│   ├── verify/
│   └── vir/
├── docs/
│   ├── README.md
│   ├── ai/
│   │   ├── ARCHITECTURE.md
│   │   ├── DECISION_TEMPLATE.md
│   │   ├── DESIGN_PRINCIPLES.md
│   │   ├── HANDOFF_PROMPT.md
│   │   ├── BOWFIN_AI_CONTEXT_BUNDLE.md
│   │   ├── BOWFIN_CONTEXT.md
│   │   ├── LANGUAGE_DECISIONS.md
│   │   ├── OPEN_QUESTIONS.md
│   │   └── README.md
│   ├── design/
│   └── language/
├── examples/
│   ├── battery.bow
│   ├── battery_ok.bow
│   ├── battery_fail.bow
│   ├── battery_sequence_fail.bow
│   ├── battery_body_fail.bow
│   └── effects.bow
├── rfcs/
├── runtime/
├── scripts/
│   ├── build-ai-context.py
│   ├── verify-repo-scaffold.py
│   └── verify-readme.py
├── std/
├── tests/
└── tools/
```

Empty implementation directories contain `.gitkeep` so Git will preserve the initial structure.

`assets/bowfin-logo.jpeg` preserves Greg's canonical source artwork unchanged.
The README uses derived desktop and mobile banners and diagrams. Its responsive
`<picture>` blocks select the mobile artwork at widths up to 767 pixels and keep
the desktop artwork as the fallback.

## Deliberately omitted for now

The scaffold does **not** choose these yet because the project has not locked them down:

- a final package/manifest format;
- a stable ABI policy.

Those should be added through explicit project decisions rather than guessed into the initial repository.

The prototype compiler uses a Cargo workspace with a pinned Rust toolchain.
That implementation choice does not settle the future Bowfin package manifest.

## Licensing and contribution ownership

The repository uses `Apache-2.0 WITH LLVM-exception`. `NOTICE`,
`TRADEMARKS.md`, and `THIRD_PARTY_NOTICES.md` record attribution, mark usage,
and third-party material. `CONTRIBUTING.md` and `legal/` distinguish original
material covered by the external contribution-assignment gate from approved
pre-existing material retained under its upstream license, and define the
individual/entity Harmony-based agreements selected with outbound-license Option
Five.

Gregory M. Krsak is the current individual rights-holder. Transfer to a future
Bowfin entity requires a documented project decision and written acceptance of
the contributor agreements' continuing obligations by the assignee.
