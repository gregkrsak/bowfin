#!/usr/bin/env python3
"""Retain measured finite-fixture neighbors from the complete Actions profile."""
import json
from pathlib import Path
import sys

directory = Path(sys.argv[1])
expected = {"catalog", "preparation", "runtime", "combined-envelope", "host-growth", "audit-output", "stopped-report"}
assert {p.stem for p in directory.glob("*.json")} == expected
evidence = {name: json.loads((directory / (name + ".json")).read_text()) for name in sorted(expected)}
assert evidence["host-growth"]["actual_mask"] == evidence["host-growth"]["tested_mask"] > 0
assert bin(evidence["host-growth"]["actual_mask"]).count("1") == 9
combined = evidence["combined-envelope"]
assert combined["sum_growth"] > combined["largest_growth"] > 0
assert combined["independent_fit_combined_failure"] == combined["retained"] + combined["largest_growth"]
assert len(evidence["runtime"]) == 3
assert all(row["storage"] > 1 and row["fuel"] > 1 for row in evidence["runtime"])
assert {row["scenario"] for row in evidence["runtime"]} == {"Sample", "FinishIoFailure", "FinishPartialIoFailure"}
assert evidence["stopped-report"]["audit_storage"] > evidence["stopped-report"]["retained"] > 0
print("BED-034 measured budget neighbors: " + json.dumps(evidence, sort_keys=True))
