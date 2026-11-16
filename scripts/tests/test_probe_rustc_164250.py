#!/usr/bin/env python3
"""Driver controls with a simulated subprocess. No Rust compiler is invoked."""

from __future__ import annotations

from contextlib import redirect_stderr, redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPTS = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("rustc_164250_probe", SCRIPTS / "probe-rustc-164250.py")
assert SPEC is not None and SPEC.loader is not None
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)

RUSTC = "/selected/bin/rustc"
PIN_HASH = "051478957371ee0084a7c0913941d2a8c4757bb9"
HOST = "x86_64-unknown-linux-gnu"
FUTURE_HASH = "0123456789abcdef0123456789abcdef01234567"
FIXED_ARGS = [
    "--expect", "fixed", "--expected-version", "90.1.0",
    "--expected-commit", FUTURE_HASH, "--expected-host", HOST,
]


def identity(*, version="1.80.0", commit=PIN_HASH, date="2024-07-21", host=HOST):
    return (
        f"rustc {version} ({commit[:9]} {date})\n"
        f"binary: rustc\ncommit-hash: {commit}\ncommit-date: {date}\n"
        f"host: {host}\nrelease: {version}\nLLVM version: 18.1.7\n"
    ).encode("utf-8")


def diagnostic(message, *, code=None, level="error", spans=None, children=None):
    return {
        "$message_type": "diagnostic", "message": message, "level": level,
        "code": None if code is None else {"code": code, "explanation": None},
        "spans": [] if spans is None else spans,
        "children": [] if children is None else children,
        "rendered": message,
    }


def json_stream(*records):
    return b"".join(f"{json.dumps(record)}\n".encode("utf-8") for record in records)


def lifetime_diagnostic(source):
    return diagnostic("`local` does not live long enough", code="E0597", spans=[{
        "file_name": str(source),
        "line_start": 8,
        "line_end": 8,
        "column_start": 20,
        "column_end": 26,
        "is_primary": True,
    }])


def borrowing_rejection(source):
    return json_stream(
        lifetime_diagnostic(source),
        diagnostic("aborting due to 1 previous error"),
        diagnostic("For more information about this error, try `rustc --explain E0597`.",
                   level="failure-note"),
    )


class DriverTests(unittest.TestCase):
    def setUp(self):
        self.identity_bytes = identity()
        self.version_status = 0
        self.outcomes = {
            # rc, diagnostics, artifact (bytes or None).
            "known_good.rs": (0, b"", b"validated metadata"),
            "explicit_type_control.rs": (1, borrowing_rejection, None),
            "issue_164250.rs": (0, b"", b"validated metadata"),
        }
        self.commands = []

    def fake_run(self, args, *, cwd, capture_output, timeout):
        self.commands.append(args)
        self.assertEqual(args[0], RUSTC)
        self.assertEqual(cwd, PROBE.ROOT)
        self.assertTrue(capture_output)
        self.assertGreater(timeout, 0)
        if args[1:] == ["-vV"]:
            return subprocess.CompletedProcess(args, self.version_status, self.identity_bytes, b"")
        self.assertEqual(args[1:6], [
            "--edition=2021", "--crate-type=lib", "--emit=metadata", "--error-format=json", "-o"
        ])
        self.assertEqual(len(args), 8)
        source = Path(args[7])
        self.assertTrue(source.is_file())
        output = Path(args[6])
        self.assertEqual(output, source.with_suffix(".rmeta"))
        self.assertFalse(output.exists())
        outcome = self.outcomes[source.name]
        if isinstance(outcome, Exception):
            raise outcome
        status, stderr, artifact = outcome
        if artifact is not None:
            output.write_bytes(artifact)
        return subprocess.CompletedProcess(
            args, status, b"", stderr(source) if callable(stderr) else stderr,
        )

    def run_driver(self, args=None, *, selected=RUSTC):
        stdout, stderr = io.StringIO(), io.StringIO()
        if args is None:
            args = ["--expect", "affected", "--expected-version", "1.80.0"]
        with (patch.object(PROBE.shutil, "which", return_value=selected),
              patch.object(PROBE.subprocess, "run", side_effect=self.fake_run),
              redirect_stdout(stdout), redirect_stderr(stderr)):
            status = PROBE.main(args)
        return status, stdout.getvalue(), stderr.getvalue()

    def require_inconclusive(self, args=None, *, selected=RUSTC):
        status, out, err = self.run_driver(args, selected=selected)
        self.assertEqual(status, 2, out + err)
        self.assertNotIn("observed=", out + err)
        self.assertNotIn("compiler probe OK", out + err)
        self.assertIn("inconclusive", err)
        return out, err

    def set_fixed(self):
        self.identity_bytes = identity(version="90.1.0", commit=FUTURE_HASH)
        self.outcomes["issue_164250.rs"] = (1, borrowing_rejection, None)

    def test_pinned_affected_and_explicit_control(self):
        status, out, err = self.run_driver()
        self.assertEqual(status, 0, out + err)
        self.assertIn("observed=affected", out)
        self.assertIn("explicit_type_control.rs: rejected with E0597", out)
        self.assertIn('"code": "E0597"', out)
        self.assertEqual([Path(argv[-1]).name for argv in self.commands], [
            "-vV", "known_good.rs", "explicit_type_control.rs", "issue_164250.rs",
        ])

    def test_fixed_requires_exact_rejection_in_both_controls(self):
        self.set_fixed()
        status, out, err = self.run_driver(FIXED_ARGS)
        self.assertEqual(status, 0, out + err)
        self.assertIn("observed=fixed", out)
        self.assertIn("issue_164250.rs: rejected with E0597", out)
        self.assertIn("explicit_type_control.rs: rejected with E0597", out)

    def test_fixed_identity_arguments_are_required(self):
        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            PROBE.main(["--expect", "fixed"])
        self.assertEqual(error.exception.code, 2)

    def test_wrong_expectation_is_failure_even_for_real_acceptance(self):
        self.set_fixed()
        self.outcomes["issue_164250.rs"] = (0, b"", b"valid")
        status, out, err = self.run_driver(FIXED_ARGS)
        self.assertEqual(status, 1, out + err)
        self.assertIn("observed affected", err)

    def test_identity_substrings_and_other_hosts_do_not_match(self):
        for bad in (identity(version="11.80.0"), identity(host="aarch64-apple-darwin"),
                    identity(commit="b" * 40), identity(date="2024-08-12"), b"1.80.0\n"):
            with self.subTest(identity=bad):
                self.identity_bytes = bad
                self.commands.clear()
                self.require_inconclusive()
                self.assertEqual(len(self.commands), 1)

    def test_missing_compiler_and_unsuccessful_version_query(self):
        self.require_inconclusive(selected=None)
        self.assertEqual(self.commands, [])
        self.version_status = 101
        self.require_inconclusive()
        self.assertEqual(len(self.commands), 1)

    def test_known_good_compile_failure_stops_before_status_test(self):
        self.outcomes["known_good.rs"] = (1, borrowing_rejection, None)
        out, _ = self.require_inconclusive()
        self.assertIn("known_good.rs stderr", out)
        self.assertEqual(len(self.commands), 2)

    def test_success_without_usable_metadata_is_not_evidence(self):
        for name in ("known_good.rs", "issue_164250.rs"):
            for artifact in (None, b""):
                with self.subTest(name=name, artifact=artifact):
                    old = self.outcomes[name]
                    self.outcomes[name] = (0, b"", artifact)
                    self.require_inconclusive()
                    self.outcomes[name] = old

    def test_nonzero_write_and_input_failures_never_observe_status(self):
        for name in ("explicit_type_control.rs", "issue_164250.rs"):
            for message in ("couldn't read input", "could not write output; permission denied"):
                with self.subTest(name=name, message=message):
                    self.set_fixed()
                    with patch.dict(self.outcomes, {name: (1, json_stream(diagnostic(message)), None)}):
                        out, _ = self.require_inconclusive(FIXED_ARGS)
                    self.assertIn(message, out)

    def test_signals_and_compiler_crashes_remain_inconclusive(self):
        self.set_fixed()
        for status in (-9, -11, 101):
            with self.subTest(status=status):
                self.outcomes["issue_164250.rs"] = (status, borrowing_rejection, None)
                self.require_inconclusive(FIXED_ARGS)

    def test_internal_error_even_when_mixed_with_expected_error(self):
        self.set_fixed()
        self.outcomes["issue_164250.rs"] = (
            1, lambda source: borrowing_rejection(source) + json_stream(
                diagnostic("internal compiler error: synthetic unexpected state", level="bug")
            ), None,
        )
        out, _ = self.require_inconclusive(FIXED_ARGS)
        self.assertIn("internal compiler error", out)

    def test_wrong_diagnostic_wrong_file_and_wrong_primary_site(self):
        self.set_fixed()
        for key, bad in (
            ("file_name", "/other/issue_164250.rs"), ("line_start", 7),
            ("is_primary", False), ("column_start", 5), ("column_end", 25),
        ):
            with self.subTest(key=key):
                def wrong(source, key=key, bad=bad):
                    error = lifetime_diagnostic(source)
                    error["spans"][0][key] = bad
                    return json_stream(error)

                self.outcomes["issue_164250.rs"] = (1, wrong, None)
                self.require_inconclusive(FIXED_ARGS)
        self.outcomes["explicit_type_control.rs"] = (
            1, json_stream(diagnostic("mismatched types", code="E0308")), None,
        )
        self.require_inconclusive(FIXED_ARGS)

    def test_expected_error_does_not_hide_unrelated_or_malformed_diagnostics(self):
        self.set_fixed()
        for other in (
            b"not json\n",
            json_stream(diagnostic("expected item")),
            json_stream({"$message_type": "diagnostic", "level": "error", "message": "missing fields"}),
        ):
            with self.subTest(other=other):
                self.outcomes["issue_164250.rs"] = (
                    1, lambda source, other=other: borrowing_rejection(source) + other, None,
                )
                self.require_inconclusive(FIXED_ARGS)

    def test_metadata_on_rejection_is_inconclusive(self):
        self.set_fixed()
        self.outcomes["issue_164250.rs"] = (1, borrowing_rejection, b"partial")
        self.require_inconclusive(FIXED_ARGS)

    def test_success_with_error_is_inconclusive(self):
        self.outcomes["issue_164250.rs"] = (0, borrowing_rejection, b"unexpected")
        self.require_inconclusive()

    def test_missing_input_and_malformed_fixture_are_rejected_before_compile(self):
        with tempfile.TemporaryDirectory() as directory:
            temp = Path(directory)
            with patch.object(PROBE, "PROBES", temp):
                self.require_inconclusive()
                shutil.copyfile(PROBE.ROOT / "tools/rustc-164250/probes/issue_164250.rs",
                                temp / "issue_164250.rs")
                (temp / "explicit_type_control.rs").write_text("incomplete input", encoding="utf-8")
                self.require_inconclusive()
        self.assertEqual(len(self.commands), 2)
        self.assertTrue(all(argv[1:] == ["-vV"] for argv in self.commands))

    def test_timeout_retains_partial_diagnostic_output(self):
        self.outcomes["explicit_type_control.rs"] = subprocess.TimeoutExpired(
            [RUSTC], 90, stderr=b"partial compiler diagnostic\n",
        )
        out, err = self.require_inconclusive()
        self.assertIn("partial compiler diagnostic", out)
        self.assertIn("timed out", err)

    def test_failed_output_write_raises_inconclusive(self):
        self.outcomes["issue_164250.rs"] = OSError("output path inaccessible")
        _, err = self.require_inconclusive()
        self.assertIn("output path inaccessible", err)

    def test_explicit_control_acceptance_fails(self):
        self.outcomes["explicit_type_control.rs"] = (0, b"", b"invalid acceptance")
        status, out, err = self.run_driver()
        self.assertEqual(status, 1, out + err)
        self.assertIn("unexpected acceptance", err)
        self.assertEqual(len(self.commands), 3)


if __name__ == "__main__":
    unittest.main()
