#!/usr/bin/env python3
import argparse
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


def srcset_urls(value):
    """Extract URL tokens, keeping commas inside URLs and skipping descriptors."""
    whitespace = " \t\n\f\r"
    position = 0
    while position < len(value):
        while position < len(value) and value[position] in whitespace + ",":
            position += 1
        start = position
        while position < len(value) and value[position] not in whitespace:
            position += 1
        url = value[start:position]
        if not url:
            break
        if url.endswith(","):
            yield url.rstrip(",")
            continue

        # Descriptors end at a comma outside parentheses. They are not paths.
        parentheses = 0
        while position < len(value):
            char = value[position]
            position += 1
            if char == "," and parentheses == 0:
                break
            if char == "(":
                parentheses += 1
            elif char == ")" and parentheses:
                parentheses -= 1
        yield url


class AssetParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.refs = set()

    def handle_starttag(self, tag, attrs):
        for name, value in attrs:
            if value is None:
                continue
            if name == "src":
                self.refs.add(value)
            elif name == "srcset":
                self.refs.update(srcset_urls(value))


root = Path(__file__).resolve().parents[1]
arguments = argparse.ArgumentParser(description="Validate README assets and policy.")
arguments.add_argument("--readme", type=Path, default=root / "README.md")
readme = arguments.parse_args().readme.read_text(encoding="utf-8")

html_assets = AssetParser()
html_assets.feed(readme)
html_assets.close()
refs = html_assets.refs | set(re.findall(r'!\[[^\]]*\]\(["\']?([^"\' )>]+)', readme))
local = [r for r in refs if not re.match(r'^[a-z]+://', r) and not r.startswith('#')]

missing = []
for ref in local:
    ref = ref.split('#', 1)[0]
    if not ref:
        continue
    path = (root / ref).resolve()
    if not path.exists():
        missing.append(ref)

policy_errors = []

for line_number, line in enumerate(readme.splitlines(), start=1):
    identifiers = re.findall(r'\b(?:BED|BD|KED|KD)-\d+\b', line)
    if identifiers:
        policy_errors.append(
            f"line {line_number}: internal decision identifier(s) "
            f"{', '.join(sorted(set(identifiers)))}"
        )

    if re.search(r'\bthis branch\b', line, flags=re.IGNORECASE):
        policy_errors.append(
            f"line {line_number}: branch-lifecycle prose belongs in an issue, "
            "pull request, or implementation ledger"
        )

    if re.search(
        r'\bawaits\b.{0,80}\b(?:review|acceptance)\b',
        line,
        flags=re.IGNORECASE,
    ):
        policy_errors.append(
            f"line {line_number}: pending-review prose must not be published "
            "as current project documentation"
        )

public_footer = """<p align="center">
  <strong>Systems programming you can reason about.</strong>
</p>

<p align="center">
  <code>bowfin build</code> ·
  <code>bowfin test</code> ·
  <code>bowfin verify</code> ·
  <code>bowfin audit</code>
</p>

<p align="center">
  <sub>Own what moves. Constrain what exists. Prove what matters. Make every escape hatch visible.</sub>
</p>

<!-- README-END -->"""

if not readme.rstrip().endswith(public_footer):
    policy_errors.append(
        "the public footer and README-END marker must remain the final README section"
    )

if missing:
    print("Missing local README assets:")
    for ref in sorted(set(missing)):
        print(" -", ref)

if policy_errors:
    print("Root README public-documentation policy violations:")
    for error in policy_errors:
        print(" -", error)
    print(
        "Move decision history to docs/ai/LANGUAGE_DECISIONS.md, exact "
        "implementation scope to compiler/README.md, and milestone evidence "
        "to docs/engineering/."
    )

if missing or policy_errors:
    sys.exit(1)

print(
    f"README local assets OK ({len(local)} references checked); "
    "public-documentation boundary OK"
)
