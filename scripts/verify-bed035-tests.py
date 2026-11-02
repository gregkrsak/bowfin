#!/usr/bin/env python3
"""Reconcile a complete Cargo profile through M3's source-extension controls.

Historical manifests remain unchanged. Only the named crate doctest identity is
mapped; target inventory also accounts for both zero-test command binaries.
"""
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
# M2 changes only this predecessor's Cargo crate target, not its test body or
# compile-fail authority assertion. Never normalize arbitrary prefixes or names.
identity_migrations = {
    "doctest:klyxr_compiler::compiler/execution/mod.rs - execution::CheckedExecutable - compile fail":
    "doctest:bowfin_compiler::compiler/execution/mod.rs - execution::CheckedExecutable - compile fail",
}
observed = actual
inverse = {current: legacy for legacy, current in identity_migrations.items()}
assert len(inverse) == len(identity_migrations) == 1, "non-injective M2 identity map"
for legacy, successor in identity_migrations.items():
    assert legacy not in observed and successor in observed, "missing or conflicting M2 doctest migration"
actual = {inverse.get(identity, identity): outcome for identity, outcome in observed.items()}
assert len(actual) == len(observed), "M2 mapping conflates observed identities"
manifest = json.loads((root / "docs/engineering/bed-034-evidence.json").read_text())
expected = {row["id"]: row["outcome"] for row in baseline["identities"]}
assert baseline["commit"] == manifest["baseline_commit"] == "372230df1d359b13d28f5c136130ac031507716a"
assert baseline["tree"] == manifest["baseline_tree"] == "a646e9576a9b7650ff9a95275f1080b591d06986"
assert len(expected) == manifest["predecessor_count"] == 1981
assert manifest["migrations"] == {} and manifest["contract_corrections"] == []
assert all(actual.get(k) == v for k, v in expected.items()), "missing or changed predecessor"
additions = manifest["additions"]
k35 = json.loads((root / "docs/engineering/bed-035-evidence.json").read_text())
new = k35["additions"]
sentinel = json.loads((root / "docs/engineering/sentinel-0-evidence.json").read_text())
application = sentinel["additions"]
# M1 adds only these five machine-identity controls. Historical KED/Sentinel
# manifests retain their original counts and meaning; they are not M1 records.
m1 = [
    "lib.rs::execution::audit::migration_tests::m1_machine_identity_table_is_exact_injective_and_accounts_for_output_bytes",
    "lib.rs::execution::audit::migration_tests::m1_all_nonempty_legacy_machine_identity_mixes_reject",
    "lib.rs::execution::audit::migration_tests::m1_unknown_wrong_role_and_neighbor_machine_identities_reject",
    "lib.rs::execution::audit::migration_tests::m1_machine_names_cannot_substitute_original_opaque_approval",
    "lib.rs::execution::audit::migration_tests::m1_canonical_machine_names_grant_no_operations_or_extra_roots",
]
m2 = [
    "tests/toolchain_identity.rs::m2_version_and_usage_identify_canonical_and_compatibility_commands",
    "tests/toolchain_identity.rs::m2_normal_check_verify_and_run_streams_are_identical",
    "tests/toolchain_identity.rs::m2_selected_results_and_audit_preserve_m1_authority",
    "tests/toolchain_identity.rs::m2_failures_and_option_validation_preserve_results",
    "tests/toolchain_identity.rs::m2_actual_output_byte_neighbors_are_command_independent",
    "tests/toolchain_identity.rs::m2_cargo_metadata_selects_canonical_and_retains_alias",
]
m3 = {
    "tests/source_extension.rs::m3_corpus_frontend_expectations_are_explicit": "corpus_frontend_expectations",
    "tests/source_extension.rs::m3_corpus_runs_and_audits_have_independent_outcomes": "corpus_execution_expectations",
    "tests/source_extension.rs::m3_legacy_and_canonical_paths_preserve_both_commands": "legacy_command_and_suffix_compatibility",
    "tests/source_extension.rs::m3_same_stem_and_other_explicit_paths_select_actual_bytes": "explicit_path_selection",
    "tests/source_extension.rs::m3_missing_paths_and_diagnostic_spans_remain_honest": "diagnostic_origin_and_input_failure",
    "tests/source_extension.rs::m3_equal_length_suffixes_preserve_actual_byte_neighbors": "actual_byte_boundary",
}
historical = set(expected) | set(additions) | set(new) | set(application)
assert len(m1) == len(set(m1)) == 5, "duplicate or unexpected M1 control count"
assert not set(m1) & historical, "M1 control reuses a historical identity"
assert len(m2) == len(set(m2)) == 6, "duplicate or unexpected M2 control count"
assert not set(m2) & (historical | set(m1)), "M2 control reuses a predecessor identity"
assert len(m3) == 6, "unexpected M3 control count"
assert not set(m3) & (historical | set(m1) | set(m2)), "M3 control reuses a predecessor identity"
assert set(m3.values()) == {
    "corpus_frontend_expectations", "corpus_execution_expectations",
    "legacy_command_and_suffix_compatibility", "explicit_path_selection",
    "diagnostic_origin_and_input_failure", "actual_byte_boundary",
}, "unclassified M3 control"
# A second binary must neither duplicate a nonempty test group nor hide a
# predecessor behind an unchanged aggregate total. Check Cargo targets even when
# they contain no tests; only these two binary groups may be empty.
expected_targets = {identity.split("::", 1)[0] for identity in historical | set(m1) | set(m2) | set(m3)}
expected_targets.remove("doctest:klyxr_compiler")
expected_targets |= {"doctest:bowfin_compiler", "main.rs", "compat_main.rs"}
assert len(targets) == len(set(targets)) == len(expected_targets) == 39, "duplicate or missing Cargo target"
assert set(targets) == expected_targets, "changed Cargo target inventory"
for target, group in zip(targets, groups):
    assert bool(group) == (target not in {"main.rs", "compat_main.rs"}), "unexpected empty or repeated binary coverage"
assert len(historical) == sentinel["expected_per_profile"] == 2126
assert sentinel["predecessor_count"] == 2112 and sentinel["migrations"] == {}
assert len(application) == len(set(application))
assert set(sentinel["classifications"]) == set(application)
assert set(sentinel["classifications"].values()) <= {
    "selected_application_and_fixture", "private_addressed_combined_failure",
    "private_receipt_threshold_boundary", "public_cli",
}
assert not set(application) & (set(expected) | set(additions) | set(new))
assert len(new) == len(set(new))
assert set(k35["classifications"]) == set(new), "unclassified BED-035 evidence"
assert set(k35["classifications"].values()) <= {
    "independent_oracle", "physical_host_classification", "structured_authority", "private_addressed_replay", "private_malformed_observation", "private_malformed_contract",
    "private_approval_identity", "private_canonical_protocol", "analytic_representation",
    "private_engine_control", "logical_or_observation_boundary", "selected_source", "ordinary_source",
}, "unknown BED-035 evidence class"
assert not set(new) & (set(expected) | set(additions))
assert len(additions) == len(set(additions))
assert set(actual) - set(expected) == set(additions) | set(new) | set(application) | set(m1) | set(m2) | set(m3), "missing or unclassified complete-profile identity"
assert manifest["expected_per_profile"] == 1981 + len(additions)
assert len(actual) == sum(summaries) == manifest["expected_per_profile"] + len(new) + len(application) + len(m1) + len(m2) + len(m3)
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
print(f"BED-034 {manifest['stage']} reconciliation: 1,981 exact predecessors + {len(additions)} additions = {manifest['expected_per_profile']} retained")
print("BED-033 retained: 1,875 predecessors + 106 additions = 1,981")
print("Historical KED manifest migrations: 0; failed/ignored/filtered: 0/0/0")
print("compile-fail authority doctest: passed; affine cross-check: 1,800 cases passed")

print(f"BED-035 {k35['stage']} reconciliation: 2045 exact predecessors + {len(new)} additions = {2045 + len(new)} retained")

assert len(actual) - len(m1) - len(m2) - len(m3) == sentinel["expected_per_profile"]
print(f"Sentinel 0 reconciliation: 2112 exact predecessors + {len(application)} classified additions = {len(historical)} retained; migrations: 0")
assert len(actual) - len(m2) - len(m3) == sentinel["expected_per_profile"] + len(m1) == 2131
print(f"Bowfin M1 reconciliation: {len(historical)} retained predecessors + {len(m1)} machine-identity controls = 2131 retained through the explicit M2 map")
assert len(actual) - len(m3) == 2131 + len(m2) == 2137
print("Bowfin M2 reconciliation: 2131 predecessors retained; one exact crate-doctest identity migration; all 6 command controls retained; 2137 predecessor identities")
for legacy, successor in identity_migrations.items():
    print(f"M2 one-to-one identity: {legacy} -> {successor}")
assert len(actual) == len(observed) == 2137 + len(m3) == 2143
print("Bowfin M3 reconciliation: all 2137 predecessors retained; no new identity migrations; 6 classified source-extension controls; 2143 passed across 39 distinct Cargo targets (two empty binaries)")
for identity, classification in m3.items():
    print(f"M3 control: {identity}; class={classification}")
