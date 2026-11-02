#!/usr/bin/env python3
"""Reconcile one complete Cargo profile against the exact BED-027 predecessor."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / "docs/engineering/bed-027-predecessors.json").read_text())
targets = []
groups = []
current = None
summaries = []
for line in Path(sys.argv[1]).read_text().splitlines():
    # Also accepts decoded Actions job logs for independent evidence inspection.
    line = re.sub(r"^\ufeff?\d{4}-\d\d-\d\dT\S+\s+", "", line)
    match = re.search(r"Running (?:unittests )?([^ ]+) \(", line)
    if match:
        targets.append(match[1])
    match = re.search(r"Doc-tests (\S+)", line)
    if match:
        targets.append("doctest:" + match[1])
    if re.fullmatch(r"running \d+ tests?", line):
        assert current is None, "overlapping test groups"
        current = []
    match = re.fullmatch(r"test (.+) \.\.\. (ok|FAILED|ignored)(?:.*)", line)
    if match:
        assert current is not None, "test outside group"
        current.append((re.sub(r" \(line \d+\)", "", match[1]), match[2]))
    if line.startswith("test result:"):
        assert current is not None, "summary outside group"
        assert re.search(r"; 0 failed; 0 ignored; 0 measured; 0 filtered out;", line), line
        passed = int(re.search(r"ok\. (\d+) passed", line)[1])
        assert passed == len(current), "summary/identity count mismatch"
        groups.append(current)
        summaries.append(passed)
        current = None

# Cargo stdout and stderr may be buffered independently. Their arrival timestamps
# do not define target ownership; Cargo's target and completed-group orders do.
assert current is None and len(targets) == len(groups), "incomplete Cargo profile"
actual = {}
for target, group in zip(targets, groups):
    for name, outcome in group:
        identity = target + "::" + name
        assert identity not in actual, "duplicate identity: " + identity
        assert outcome == "ok", identity + ": " + outcome
        actual[identity] = outcome
expected = {r["id"]: r["outcome"] for r in baseline["identities"]}
assert len(expected) == 1581
assert all(actual.get(k) == v for k, v in expected.items()), "missing/changed predecessor"
new = sorted(set(actual) - set(expected))
assert len(new) == 86, f"expected 86 BD-042 tests, found {len(new)}"
assert all(k.startswith(("lib.rs::hir::resources::", "lib.rs::mir::cleanup::", "lib.rs::execution::machine::cleanup::")) for k in new), "unclassified new test"
assert len(actual) == sum(summaries) == 1667
affine = "lib.rs::verify::tests::affine_proofs_match_exhaustive_execution_on_small_domains"
assert actual.get(affine) == "ok", "missing 1,800-case affine cross-check"
doctests = [k for k in actual if k.startswith("doctest:")]
assert len(doctests) == 1 and "CheckedExecutable - compile fail" in doctests[0]
print("BED-027 reconciliation: 1,581 predecessors + 86 new = 1,667 passed")
print("compile-fail doctest: passed; affine cross-check: 1,800 cases passed")
print("failed/ignored/filtered: 0/0/0")
categories = {}
for identity in new:
    category = identity.rsplit("::", 1)[0]
    categories[category] = categories.get(category, 0) + 1
for category, count in sorted(categories.items()):
    print(f"new category: {category}: {count}")
