#!/usr/bin/env python3
"""Reconcile one complete Cargo profile against the exact BED-032 predecessor."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / "docs/engineering/bed-032-predecessors.json").read_text())
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
manifest = json.loads((root / "docs/engineering/bed-032-evidence.json").read_text())
expected = {row["id"]:row["outcome"] for row in baseline["identities"]}
assert baseline["commit"] == manifest["baseline_commit"] == "74c1bedc8f847ffaeb38ed4795ee91730e35b949"
assert baseline["tree"] == manifest["baseline_tree"] == "2d0cc78b67a8cb69ea8feaff183dbf1b91938e63"
assert len(expected) == manifest["predecessor_count"] == 1846
assert manifest["migrations"] == {} and manifest["contract_corrections"] == []
assert all(actual.get(k)==v for k,v in expected.items()), "missing or changed predecessor"
additions=manifest["additions"]
assert len(additions)==len(set(additions))==29
assert set(actual)-set(expected)==set(additions), "missing or unclassified BED-032 identity"
assert len(actual)==sum(summaries)==1846+len(additions)
assert set(manifest["classifications"])==set(additions)
assert set(manifest["classifications"].values()) <= {"ordinary_source","private_canonical_resources","primitive_protocol"}
old=json.loads((root / "docs/engineering/bed-031-predecessors.json").read_text())
k31=json.loads((root / "docs/engineering/bed-031-evidence.json").read_text())
assert {r["id"] for r in old["identities"]} | set(k31["additions"]) == set(expected)
assert len(k31["repair"]["reviewed_additions"])==46 and len(k31["repair"]["new_identities"])==10
assert actual.get("lib.rs::verify::tests::affine_proofs_match_exhaustive_execution_on_small_domains")=="ok"
doctests=[k for k in actual if k.startswith("doctest:")]
assert len(doctests)==1 and "CheckedExecutable - compile fail" in doctests[0]
print(f"BED-032 reconciliation: 1,846 exact predecessors + {len(additions)} additions = {len(actual)} passed")
print("BED-031 reconciliation retained: 1,790 original + 46 reviewed + 10 MR01 = 1,846")
print("identity migrations: 0; failed/ignored/filtered: 0/0/0")
print("compile-fail authority doctest: passed; affine cross-check: 1,800 cases passed")
print("evidence lanes:", {d:list(manifest["classifications"].values()).count(d) for d in sorted(set(manifest["classifications"].values()))})

for adaptation in manifest["fixture_adaptations"]:
    assert adaptation["reason"] and adaptation["affected_identities"]
    assert set(adaptation["affected_identities"]) <= set(expected)
