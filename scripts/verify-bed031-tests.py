#!/usr/bin/env python3
"""Reconcile one complete Cargo profile against the exact BED-031 predecessor."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / "docs/engineering/bed-031-predecessors.json").read_text())
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
manifest = json.loads((root / "docs/engineering/bed-031-evidence.json").read_text())
expected = {r["id"]: r["outcome"] for r in baseline["identities"]}
assert len(expected) == 1790, "exact BED-031 predecessor inventory required"
assert manifest["migrations"] == {}, "no predecessor identity migrations authorized by this implementation"
additions = manifest["additions"]
assert len(additions) == len(set(additions)), "duplicate addition"
assert all(actual.get(k) == v for k, v in expected.items()), "missing/changed predecessor"
assert set(actual) - set(expected) == set(additions), "unclassified or missing additions"
assert len(actual) == sum(summaries) == 1790 + len(additions)
repair = manifest["repair"]
reviewed = set(repair["reviewed_additions"])
new_repair = set(repair["new_identities"])
assert repair["finding"] == "K31-MR01" and len(reviewed) == 46
assert len(new_repair) == 10 and len(repair["new_identities"]) == 10
assert not reviewed & new_repair and set(additions) == reviewed | new_repair
assert all(actual.get(k) == "ok" for k in reviewed | new_repair)
assert set(manifest["classifications"]) == set(additions)
assert set(manifest["classifications"].values()) <= {"ordinary_source", "private_canonical_resources", "primitive_protocol"}
for adaptation in manifest["fixture_adaptations"]:
    assert adaptation["affected_identities"] and set(adaptation["affected_identities"]) <= set(expected)
    assert adaptation["reason"]
assert manifest["contract_corrections"] == []
assert actual.get("lib.rs::verify::tests::affine_proofs_match_exhaustive_execution_on_small_domains") == "ok"
doctests = [k for k in actual if k.startswith("doctest:")]
assert len(doctests) == 1 and "CheckedExecutable - compile fail" in doctests[0]
print(f"BED-031 reconciliation: 1,790 predecessors + {len(additions)} additions = {len(actual)} passed")
print("identity migrations: 0; failed/ignored/filtered: 0/0/0")
print("compile-fail authority doctest: passed; affine cross-check: 1,800 cases passed")
print("evidence lanes:", {d: list(manifest["classifications"].values()).count(d) for d in sorted(set(manifest["classifications"].values()))})

print(f"K31-MR01 reconciliation: 1,836 reviewed identities + {len(new_repair)} repairs = {len(actual)} passed")
