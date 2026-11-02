#!/usr/bin/env python3
"""Reconcile one complete Cargo profile against the exact BED-028 predecessor."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / "docs/engineering/bed-028-predecessors.json").read_text())
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
def reconcile(actual, baseline, migrations, additions):
    expected = {r["id"]: r["outcome"] for r in baseline["identities"]}
    assert len(expected) == 1667, "exact BED-028 baseline inventory required"
    for old, new in migrations.items():
        assert old in expected and new not in expected
        expected[new] = expected.pop(old)
    assert all(actual.get(k) == v for k, v in expected.items()), "missing/changed predecessor"
    new = set(actual) - set(expected)
    assert new == set(additions), "unclassified additions: " + repr(sorted(new ^ set(additions)))
    assert len(actual) == sum(summaries) == 1667 + len(additions)
    affine = "lib.rs::verify::tests::affine_proofs_match_exhaustive_execution_on_small_domains"
    assert actual.get(affine) == "ok", "missing 1,800-case affine cross-check"
    doctests = [k for k in actual if k.startswith("doctest:")]
    assert len(doctests) == 1 and "CheckedExecutable - compile fail" in doctests[0]
    print(f"BED-028 reconciliation: 1,667 predecessors ({len(migrations)} explicit migrations) + {len(additions)} new = {len(actual)} passed")
    print("compile-fail doctest: passed; affine cross-check: 1,800 cases passed")
    print("failed/ignored/filtered: 0/0/0")

manifest = json.loads((root / "docs/engineering/bed-028-evidence.json").read_text())
reconcile(actual, baseline, manifest["migrations"], manifest["additions"])
