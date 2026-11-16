#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "verify-rustc-164250.py"
SPEC = importlib.util.spec_from_file_location("rustc_164250_guard", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
GUARD = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = GUARD
SPEC.loader.exec_module(GUARD)


class Rustc164250GuardTests(unittest.TestCase):
    def findings(self, source: str):
        return GUARD.find_bound_aggregates(source, "sample.rs")

    def test_generic_lifetime_bound_is_rejected(self):
        findings = self.findings("struct Q<T: 'static + Send>(Option<T>);")
        self.assertEqual([(item.kind, item.name) for item in findings], [("struct", "Q")])

    def test_where_bound_after_tuple_fields_is_rejected(self):
        findings = self.findings("struct Q<T>(T) where T: 'static;")
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].reason, "where-clause bound")

    def test_trait_bounds_used_by_array_unification_are_rejected(self):
        findings = self.findings(
            "struct InvokeOnDrop<T: Copy, F: FnMut(T)>(T, F);"
        )
        self.assertEqual(len(findings), 1)

    def test_lifetime_parameter_bound_is_rejected(self):
        findings = self.findings("enum E<'a: 'static> { Value(&'a str) }")
        self.assertEqual(len(findings), 1)

    def test_const_parameter_type_is_not_a_generic_bound(self):
        self.assertEqual(self.findings("struct Array<T, const N: usize>([T; N]);"), [])

    def test_unbounded_generic_is_allowed(self):
        self.assertEqual(self.findings("struct Plain<T>(Option<T>);"), [])

    def test_checked_in_adversarial_fixtures_reach_the_guard(self):
        fixtures = ROOT / "tools" / "rustc-164250" / "fixtures"
        expected = {
            "array_unification.rs": 1,
            "generic_bound.rs": 1,
            "lifetime_bound.rs": 1,
            "safe_unbounded.rs": 0,
            "where_bound.rs": 1,
        }
        observed = {
            path.name: len(
                GUARD.find_bound_aggregates(
                    path.read_text(encoding="utf-8"),
                    path.relative_to(ROOT).as_posix(),
                )
            )
            for path in sorted(fixtures.glob("*.rs"))
        }
        self.assertEqual(observed, expected)

    def test_comments_and_literals_do_not_create_declarations(self):
        source = r'''
            // struct Commented<T: 'static>(T);
            const TEXT: &str = "enum Stringy<T: Copy> { Value(T) }";
            const RAW: &str = r#"union Raw<T: Send> { value: T }"#;
            /* struct Blocked<T: Clone>(T); */
        '''
        self.assertEqual(self.findings(source), [])

    def test_nested_block_comments_are_ignored(self):
        source = "/* outer /* struct Q<T: Copy>(T); */ still outer */ struct P<T>(T);"
        self.assertEqual(self.findings(source), [])

    def test_macro_generated_declaration_is_visible_to_guard(self):
        source = "macro_rules! m { () => { struct Q<T: Copy>(T); } }"
        findings = self.findings(source)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].name, "Q")

    def test_braced_field_change_invalidates_declaration_digest(self):
        first = self.findings("struct Q<T: Copy> { value: T }")[0]
        second = self.findings("struct Q<T: Copy> { value: Option<T> }")[0]
        self.assertNotEqual(first.declaration_sha256, second.declaration_sha256)

    def test_exact_reviewed_allowlist_entry_is_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_path = root / "sample.rs"
            source_path.write_text("struct Q<T: Copy>(T);\n", encoding="utf-8")
            finding = GUARD.find_bound_aggregates(
                source_path.read_text(encoding="utf-8"), "sample.rs"
            )[0]
            allowlist = root / "allowlist.json"
            allowlist.write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "issue": GUARD.ISSUE,
                        "entries": [
                            {
                                "path": "sample.rs",
                                "kind": "struct",
                                "name": "Q",
                                "declaration_sha256": finding.declaration_sha256,
                                "rationale": "Reviewed construction-closed test type.",
                                "review": "https://example.invalid/review/1",
                                "controls": ["sample::explicit_type_control"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            _, errors = GUARD.check(root, [source_path], allowlist)
            self.assertEqual(errors, [])

    def test_changed_declaration_invalidates_allowlist_digest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_path = root / "sample.rs"
            source_path.write_text("struct Q<T: Copy>(T);\n", encoding="utf-8")
            allowlist = root / "allowlist.json"
            allowlist.write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "issue": GUARD.ISSUE,
                        "entries": [
                            {
                                "path": "sample.rs",
                                "kind": "struct",
                                "name": "Q",
                                "declaration_sha256": "0" * 64,
                                "rationale": "Reviewed construction-closed test type.",
                                "review": "https://example.invalid/review/1",
                                "controls": ["sample::explicit_type_control"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            _, errors = GUARD.check(root, [source_path], allowlist)
            self.assertEqual(len(errors), 2)
            self.assertTrue(any("unreviewed" in error for error in errors))
            self.assertTrue(any("stale" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
