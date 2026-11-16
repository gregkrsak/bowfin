#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
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

def reviewed_entry(finding):
    return {
        "path": finding.path,
        "kind": finding.kind,
        "name": finding.name,
        "declaration_sha256": finding.declaration_sha256,
        "file_sha256": finding.file_sha256,
        "byte_offset": finding.byte_offset,
        "rationale": "Reviewed construction-closed test type.",
        "review": "https://example.invalid/review/1",
        "controls": ["sample::explicit_type_control"],
    }


class DefaultGuardTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.git("init", "-q")
        self.allowlist()

    def git(self, *args):
        subprocess.run(["git", *args], cwd=self.root, check=True, capture_output=True)

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def allowlist(self, *entries):
        return self.write(
            GUARD.DEFAULT_ALLOWLIST,
            json.dumps({"schema": 2, "issue": GUARD.ISSUE, "entries": list(entries)}),
        )

    def run_guard(self, *, stage=True):
        if stage:
            self.git("add", "-A")
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root)],
            cwd=self.root, text=True, capture_output=True, check=False,
        )

    def assert_guard(self, expected_code, message):
        result = self.run_guard()
        self.assertEqual(result.returncode, expected_code, result.stdout + result.stderr)
        self.assertIn(message, result.stdout + result.stderr)

    def test_default_rejects_bound_on_raw_identifier(self):
        self.write("lib.rs", "struct r#type<T: Copy>(T);")
        self.assert_guard(1, "unreviewed struct r#type")

    def test_default_allows_lawful_raw_identifier(self):
        self.write("lib.rs", "struct r#type<T>(T);")
        self.assert_guard(0, "1 scanned Rust files")

    def test_default_rejects_bound_on_macro_metavariable_name(self):
        self.write("lib.rs", "macro_rules! m { ($name:ident) => { struct $name<T: Copy>(T); }; } m!(Q);")
        self.assert_guard(1, "struct $name in a macro template")

    def test_default_allows_lawful_macro_metavariable_name(self):
        self.write("lib.rs", "macro_rules! m { ($name:ident) => { struct $name<T>(T); }; } m!(Q);")
        self.assert_guard(0, "0 reviewed bounded aggregates")

    def test_default_rejects_unresolved_macro_declaration_header(self):
        self.write("lib.rs", "macro_rules! m { ($name:ident, $g:tt) => { struct $name $g; }; }")
        self.assert_guard(1, "unresolved struct $name declaration header")

    def test_default_rejects_nested_const_item(self):
        self.write("lib.rs", "struct Outer([u8; { struct Inner<T: Copy>(T); 1 }]);")
        self.assert_guard(1, "unreviewed struct Inner")

    def test_default_allows_lawful_nested_items_and_const_parameter(self):
        self.write("lib.rs", "struct Outer<const N: usize = { 1 << 2 }>([u8; { struct Inner<T>(T); N }]);")
        self.assert_guard(0, "0 reviewed bounded aggregates")

    def test_default_checks_literal_rust_include_and_ignores_data_include(self):
        self.write("src/lib.rs", 'include!(r#"../value.rs"#); const DATA: &str = include_str!("../data.inc");')
        self.write("value.rs", "struct Q<T>(T);")
        self.write("data.inc", "struct Data<T: Copy>(T);")
        self.assert_guard(0, "2 scanned Rust files")
        self.write("value.rs", "struct Q<T: Copy>(T);")
        self.assert_guard(1, "value.rs:1: struct Q in a macro template or code-spliced file")

    def test_default_rejects_include_of_excluded_fixture_and_non_rust_input(self):
        for name in ("tools/rustc-164250/fixtures/data.rs", "value.inc"):
            with self.subTest(name=name):
                self.write("lib.rs", f'include!("{name}");')
                self.write(name, "struct Q<T>(T);")
                self.assert_guard(1, f"Rust source input '{name}' is outside")

    def test_default_rejects_indirection_in_rust_source_include(self):
        cases = {
            'include!(concat!("value", ".rs"));': "unresolved include! input",
            'use std::include as splice; splice!("value.rs");': "unresolved use of Rust include",
            'macro_rules! m { ($f:ident) => { $f!("value.rs"); }; }': "unresolved dynamically named macro",
            'macro_rules! m { () => { include!("value.rs"); }; }': "unresolved include! base",
        }
        self.write("value.rs", "struct Q<T>(T);")
        for code, expected in cases.items():
            with self.subTest(code=code):
                self.write("lib.rs", code)
                self.assert_guard(1, expected)

    def test_default_rejects_raw_include_of_non_rust_input(self):
        self.write("lib.rs", 'r#include!("value.inc");')
        self.write("value.inc", "struct Q<T>(T);")
        self.assert_guard(1, "Rust source input 'value.inc' is outside")

    def test_default_rejects_untracked_source_include(self):
        self.write("lib.rs", 'include!("value.rs");')
        self.git("add", "-A")
        self.write("value.rs", "struct Q<T>(T);")
        result = self.run_guard(stage=False)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("Rust source input 'value.rs' is outside", result.stderr)

    def test_default_rejects_symlink_at_final_or_cancelled_segment(self):
        self.write("value.rs", "struct Q<T>(T);")
        (self.root / "alias.inc").symlink_to("value.rs")
        (self.root / "alias-dir").symlink_to(".", target_is_directory=True)
        for name in ("alias.inc", "alias-dir/../value.rs"):
            with self.subTest(name=name):
                self.write("lib.rs", f'include!("{name}");')
                self.assert_guard(1, f"Rust source input '{name}' escapes the root or uses a symlink")

    def test_default_checks_literal_path_attribute_under_cfg_attr(self):
        self.write("lib.rs", '#[cfg_attr(test, path = "value.rs")] mod value;')
        self.write("value.rs", "struct Q<T>(T);")
        self.assert_guard(0, "2 scanned Rust files")
        self.write("lib.rs", '#[cfg_attr(test, path = "value.inc")] mod value;')
        self.write("value.inc", "struct Q<T>(T);")
        self.assert_guard(1, "Rust source input 'value.inc' is outside")

    def test_default_rejects_path_attribute_with_context_dependent_base(self):
        self.write("value.rs", "struct Q<T>(T);")
        self.write("lib.rs", 'mod a { #[path = "value.rs"] mod value; }')
        self.assert_guard(1, "unresolved nested #[path] base")
        self.write("lib.rs", 'include!("nested.rs");')
        self.write("nested.rs", '#[path = "value.rs"] mod value;')
        self.assert_guard(1, "unresolved #[path] base in code-spliced file")

    def test_exact_reviewed_allowlist_entry_is_accepted(self):
        source = "// π means byte offsets differ from character offsets.\n#[cfg(test)] pub struct Q<T: Copy>(T);"
        self.write("lib.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        self.assertEqual(finding.byte_offset, source.encode().index(b"struct"))
        self.allowlist(reviewed_entry(finding))
        self.assert_guard(0, "1 reviewed bounded aggregates")

    def test_abi_literal_drift_changes_declaration_digest_and_invalidates_entry(self):
        original = 'struct Q<T: Copy> { value: T, callback: extern "C" fn() }'
        updated = original.replace('"C"', '"Rust"')
        first, = GUARD.find_bound_aggregates(original, "lib.rs")
        second, = GUARD.find_bound_aggregates(updated, "lib.rs")
        self.assertNotEqual(first.declaration_sha256, second.declaration_sha256)
        self.write("lib.rs", original)
        self.allowlist(reviewed_entry(first))
        self.assert_guard(0, "1 reviewed bounded aggregates")
        self.write("lib.rs", updated)
        result = self.run_guard()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("unreviewed struct Q", result.stderr)
        self.assertIn("stale", result.stderr)

    def test_changed_declaration_invalidates_allowlist_digest(self):
        source = "struct Q<T: Copy>(T);"
        self.write("lib.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        self.allowlist(reviewed_entry(finding))
        self.assert_guard(0, "1 reviewed bounded aggregates")
        self.write("lib.rs", "struct Q<T: Copy>(Option<T>);")
        result = self.run_guard()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("unreviewed struct Q", result.stderr)
        self.assertIn("stale", result.stderr)

    def test_changed_attributes_invalidate_full_file_identity(self):
        source = "#[cfg(test)] struct Q<T: Copy>(T);"
        self.write("lib.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        self.allowlist(reviewed_entry(finding))
        self.assert_guard(0, "1 reviewed bounded aggregates")
        self.write("lib.rs", source.replace("test", "unix"))
        self.assert_guard(1, "stale")

    def test_duplicate_scope_needs_its_own_entry_even_at_same_file_digest(self):
        declaration = "struct Q<T: Copy>(T);"
        source = f"mod a {{ {declaration} }} mod b {{ {declaration} }}"
        self.write("lib.rs", source)
        first, second = GUARD.find_bound_aggregates(source, "lib.rs")
        self.assertEqual(first.declaration_sha256, second.declaration_sha256)
        self.assertEqual(first.file_sha256, second.file_sha256)
        self.assertNotEqual(first.byte_offset, second.byte_offset)
        self.allowlist(reviewed_entry(first))
        self.assert_guard(1, "unreviewed struct Q")
        self.allowlist(reviewed_entry(first), reviewed_entry(second))
        self.assert_guard(0, "2 reviewed bounded aggregates")

    def test_duplicate_actual_matches_fail_bijection(self):
        source = "struct Q<T: Copy>(T);"
        file = self.write("lib.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        allowlist = self.allowlist(reviewed_entry(finding))
        _, errors = GUARD.check(self.root, [file, file], allowlist)
        self.assertTrue(any("duplicate actual declaration identity" in error for error in errors))

    def test_duplicate_allowlist_matches_fail_bijection(self):
        source = "struct Q<T: Copy>(T);"
        self.write("lib.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        entry = reviewed_entry(finding)
        self.allowlist(entry, entry)
        self.assert_guard(2, "duplicate allowlist entry")

    def test_stale_entry_fails_without_current_candidate(self):
        source = "struct Q<T: Copy>(T);"
        finding, = GUARD.find_bound_aggregates(source, "lib.rs")
        self.write("lib.rs", "struct Q<T>(T);")
        self.allowlist(reviewed_entry(finding))
        self.assert_guard(1, "stale")

    def test_exact_entry_cannot_cover_a_file_newly_spliced_as_rust(self):
        source = "struct Q<T: Copy>(T);"
        self.write("value.rs", source)
        finding, = GUARD.find_bound_aggregates(source, "value.rs")
        self.allowlist(reviewed_entry(finding))
        self.assert_guard(0, "1 reviewed bounded aggregates")
        self.write("lib.rs", 'include!("value.rs");')
        self.assert_guard(1, "code-spliced file requires source repair")


if __name__ == "__main__":
    unittest.main()
