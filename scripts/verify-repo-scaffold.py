#!/usr/bin/env python3
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
readme = (root / "README.md").read_text(encoding="utf-8")

expected_workflows = ["build.yml", "tests.yml", "verify.yml", "docs.yml"]
repo_slug = "gregkrsak/bowfin"

errors = []

# All expected workflows must exist.
for workflow in expected_workflows:
    if not (root / ".github" / "workflows" / workflow).is_file():
        errors.append(f"missing workflow: .github/workflows/{workflow}")

# README must use live GitHub Actions badges for every workflow.
for workflow in expected_workflows:
    badge_url = f"https://github.com/{repo_slug}/actions/workflows/{workflow}/badge.svg"
    action_url = f"https://github.com/{repo_slug}/actions/workflows/{workflow}"
    if badge_url not in readme:
        errors.append(f"README missing live badge URL for {workflow}")
    if action_url not in readme:
        errors.append(f"README missing workflow link for {workflow}")

# Static badge assets/references must not return.
if (root / "assets" / "badges").exists():
    errors.append("assets/badges/ should not exist")
if "assets/badges/" in readme:
    errors.append("README still references static badge assets")

# Hero must exist locally.
if not (root / "assets" / "readme-banner.png").is_file():
    errors.append("missing assets/readme-banner.png")

if errors:
    print("Scaffold verification failed:")
    for error in errors:
        print(" -", error)
    sys.exit(1)

print("Scaffold verification OK")
print(" - live workflow badge URLs: build, tests, verify, docs")
print(" - workflow files exist")
print(" - no static badge assets/references")
print(" - hero banner path exists")
