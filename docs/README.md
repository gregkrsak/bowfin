# Bowfin documentation

This prototype directory is intentionally small.

Planned public documentation areas:

- Language tour
- Ownership and borrowing
- Domain-aware types
- Contracts and invariants
- Effects
- `safe` / `checked` / `verified`
- Trust and unsafe boundaries
- Embedded and realtime profiles
- Compiler architecture
- RFCs

## Documentation placement

The root `README.md` is Bowfin's public front door. Keep it readable without
project-internal decision history or implementation-process context. Describe the
current executable capability in ordinary language and link to deeper material.

Use the established documentation layers consistently:

- `README.md`: public purpose, design direction, concise current capability,
  basic commands, contribution posture, and project principles;
- `compiler/README.md`: exact implemented syntax, semantic boundaries,
  diagnostics, examples, and known exclusions;
- `docs/ai/LANGUAGE_DECISIONS.md`: accepted and directional language decisions;
- `docs/engineering/`: implementation ledgers, milestone identities, test
  evidence, and exact review state;
- GitHub issues and pull requests: drafts, pressure reviews, branch lifecycle,
  acceptance, and merge history.

Do not place BED/BD identifiers, draft or freeze history, branch-specific status,
pending-review language, or implementation-ledger prose in the root README.
`scripts/verify-readme.py` enforces this public-documentation boundary and keeps
the branded footer as the final section.
