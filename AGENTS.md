# Bowfin Agent Instructions

Before making language-design, compiler-architecture, verification, or major tooling changes, read these files in order:

1. `docs/ai/BOWFIN_CONTEXT.md`
2. `docs/ai/DESIGN_PRINCIPLES.md`
3. `docs/ai/LANGUAGE_DECISIONS.md`
4. `docs/ai/ARCHITECTURE.md`
5. `docs/ai/OPEN_QUESTIONS.md`

`docs/ai/LANGUAGE_DECISIONS.md` is authoritative for settled decisions.

## Rules for agents

- Do **not** reinterpret an **Accepted** decision unless the user explicitly asks to reopen it.
- Distinguish **Accepted**, **Direction Accepted**, **Provisional**, **Proposed**, and **Open**.
- Do not silently turn illustrative syntax into a language guarantee.
- Do not overclaim formal verification. Bowfin proves stated properties under stated assumptions.
- When proposing a change, identify the affected BD/DP/OQ entries.
- Prefer coherent semantics, explicit trust boundaries, precise diagnostics, and auditable behavior.
- After an accepted design change, update the canonical files in `docs/ai/`.
- Rebuild the single-file context bundle with:

```bash
python scripts/build-ai-context.py
```

The repository—not conversational memory—is Bowfin's canonical project context.

## Commit messages

Use Conventional-Commit-style lowercase type prefixes:

```text
<type>: <concise imperative summary>
```

Use `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`, `ci`, or `perf`
according to the commit's dominant purpose. Prefer an existing appropriate prefix
rather than inventing one. Do not use bare unprefixed messages or split one coherent
implementation artificially just to use multiple prefixes. This convention applies
to future agent work unless project leads change it.

For example:

```text
feat: add value-producing conditional initializers (BED-010)
fix: preserve ownership across conditional value joins
```
