#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
ai = root / "docs" / "ai"

order = [
    "BOWFIN_CONTEXT.md",
    "DESIGN_PRINCIPLES.md",
    "LANGUAGE_DECISIONS.md",
    "ARCHITECTURE.md",
    "OPEN_QUESTIONS.md",
]

header = '''# Bowfin AI Context Bundle

> Generated from the canonical files in `docs/ai/`.
>
> A fresh contributor or AI model should read this entire file before making major Bowfin design proposals.
>
> **Do not reinterpret Accepted decisions unless explicitly asked.**

'''

parts = [header]
for name in order:
    p = ai / name
    if not p.exists():
        raise SystemExit(f"Missing canonical context file: {p}")
    parts.append(f"\n---\n\n<!-- BEGIN {name} -->\n\n")
    parts.append(p.read_text(encoding="utf-8"))
    parts.append(f"\n<!-- END {name} -->\n")

out = ai / "BOWFIN_AI_CONTEXT_BUNDLE.md"
out.write_text("".join(parts), encoding="utf-8")
print(f"Wrote {out}")
