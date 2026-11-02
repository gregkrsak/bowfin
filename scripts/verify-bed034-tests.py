#!/usr/bin/env python3
"""Reconcile one complete Cargo profile against the exact BED-034 predecessor."""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / "docs/engineering/bed-034-predecessors.json").read_text())
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
manifest = json.loads((root / "docs/engineering/bed-034-evidence.json").read_text())
expected = {row["id"]: row["outcome"] for row in baseline["identities"]}
assert baseline["commit"] == manifest["baseline_commit"] == "372230df1d359b13d28f5c136130ac031507716a"
assert baseline["tree"] == manifest["baseline_tree"] == "a646e9576a9b7650ff9a95275f1080b591d06986"
assert len(expected) == manifest["predecessor_count"] == 1981
assert manifest["migrations"] == {} and manifest["contract_corrections"] == []
assert all(actual.get(k) == v for k, v in expected.items()), "missing or changed predecessor"
additions = manifest["additions"]
assert len(additions) == len(set(additions))
assert set(actual) - set(expected) == set(additions), "missing or unclassified BED-034 identity"
assert len(actual) == sum(summaries) == manifest["expected_per_profile"] == 1981 + len(additions)
assert set(manifest["classifications"]) == set(additions)
assert set(manifest["classifications"].values()) <= {"ordinary_source", "private_canonical_catalog", "private_catalog_envelope", "trusted_boundary", "private_malformed_authority"}
k33 = json.loads((root / "docs/engineering/bed-033-evidence.json").read_text())
old = json.loads((root / "docs/engineering/bed-033-predecessors.json").read_text())
assert {row["id"] for row in old["identities"]} | set(k33["additions"]) == set(expected)
assert len(old["identities"]) == 1875 and len(k33["additions"]) == 106
pre32 = json.loads((root / "docs/engineering/bed-032-predecessors.json").read_text())
k32 = json.loads((root / "docs/engineering/bed-032-evidence.json").read_text())
assert {row["id"] for row in pre32["identities"]} | set(k32["additions"]) == {row["id"] for row in old["identities"]}
assert len(pre32["identities"]) == 1846 and len(k32["additions"]) == 29
assert k33["migrations"] == {} and k33["contract_corrections"] == []
for key, count, prefix in [("semantic_obligations", 35, "20."), ("malformed_controls", 13, "21."), ("constraints", 9, "K33-P")]:
    assert set(k33[key]) == {prefix + f"{n:02}" for n in range(1, count + 1)}
    assert all(values and set(values) <= set(k33["additions"]) for values in k33[key].values())
assert actual.get("lib.rs::verify::tests::affine_proofs_match_exhaustive_execution_on_small_domains") == "ok"
doctests = [k for k in actual if k.startswith("doctest:")]
assert len(doctests) == 1 and "CheckedExecutable - compile fail" in doctests[0]
assert manifest["authority_doctests"] == 1 and manifest["affine_cases_per_profile"] == 1800
if manifest["stage"] == "M9":
    script_id = "python:predecessor_identity_reconciliation"
    assert set(manifest["script_assertions"]) == {script_id}
    script = manifest["script_assertions"][script_id]
    assert script["path"] == "scripts/verify-ked034-tests.py"
    assert script["assertion"] == "all(actual.get(k) == v for k, v in expected.items())"
    # The frozen evidence names its original producer. Resolve only this
    # enumerated moved script; do not rewrite the historical JSON claim.
    current_script = "scripts/verify-bed034-tests.py"
    assert script["meaning"] and script["assertion"] in (root / current_script).read_text()
    for key, count, prefix in [("semantic_obligations", 26, "13."), ("malformed_controls", 12, "14."), ("constraints", 9, "K34-P")]:
        keys = {prefix + f"{n:02}" for n in range(1, count + 1)}
        assert set(manifest[key]) == keys
        assert set(manifest["production"][key]) == keys
        assert set(manifest["obligation_descriptions"][key]) == keys
        assert all(values and len(values) == len(set(values)) and set(values) <= set(additions) | {script_id} for values in manifest[key].values())
        for obligation in keys:
            assert manifest["obligation_descriptions"][key][obligation]
            production = manifest["production"][key][obligation]
            assert production
            for seam in production:
                file = root / (current_script if seam["path"] == script["path"] else seam["path"])
                assert file.is_file() and seam["symbol"] in file.read_text(), (obligation, seam)
    assert manifest["semantic_obligations"]["13.25"] == [script_id]
    assert set(manifest["milestones"]) == {f"M{n}" for n in range(10)}
    for adaptation in manifest["fixture_adaptations"]:
        assert adaptation["reason"] and adaptation["affected_identities"]
        assert set(adaptation["affected_identities"]) <= set(expected) | set(additions)
    print("Frozen BED-034 map: 26 semantic obligations, 12 malformed families, P01-P09, M0-M9; production symbols and exact assertions present")
print(f"BED-034 {manifest['stage']} reconciliation: 1,981 exact predecessors + {len(additions)} additions = {len(actual)} passed")
print("BED-033 retained: 1,875 predecessors + 106 additions = 1,981")
print("identity migrations: 0; failed/ignored/filtered: 0/0/0")
print("compile-fail authority doctest: passed; affine cross-check: 1,800 cases passed")
