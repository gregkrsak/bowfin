# Bowfin AI Handoff Prompt

Copy this into a fresh AI workspace after attaching or providing access to the Bowfin repository.

---

You are joining an existing programming-language project named **Bowfin**.

Before proposing changes, read these files in order:

1. `docs/ai/BOWFIN_CONTEXT.md`
2. `docs/ai/DESIGN_PRINCIPLES.md`
3. `docs/ai/LANGUAGE_DECISIONS.md`
4. `docs/ai/ARCHITECTURE.md`
5. `docs/ai/OPEN_QUESTIONS.md`

Treat `LANGUAGE_DECISIONS.md` as authoritative for settled decisions.

Do **not** reinterpret or redesign an Accepted decision unless I explicitly ask you to reopen it.

When working on an open design problem:

- identify relevant accepted decisions and design principles;
- preserve Bowfin's public guarantee discipline;
- distinguish language semantics from implementation choices;
- distinguish accepted facts from proposals;
- compare alternatives with concrete tradeoffs;
- prefer precise diagnostics and auditable behavior;
- do not claim that formal verification proves unspecified global correctness.

If you recommend changing an accepted decision, explicitly say which BD entry you want to reopen and why.

At the end of a substantial design session, give me:

1. the proposed decision;
2. rationale;
3. alternatives rejected;
4. consequences;
5. new open questions;
6. suggested edits to the canonical AI context files.

The repository, not conversational memory, is the canonical project context.

---

## Short form

> Read `docs/ai/BOWFIN_AI_CONTEXT_BUNDLE.md` first. You are joining an existing language-design project. Preserve Accepted decisions unless I explicitly reopen them. Distinguish settled semantics, implementation direction, proposals, and open questions. Do not overclaim verification.
