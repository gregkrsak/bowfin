#!/usr/bin/env python3
"""Check the selected rustc with the reviewed compile-only #164250 controls.

Statuses apply to these exact controls. Unknown diagnostics are inconclusive,
including on a future compiler believed to contain the upstream repair.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
PROBES = ROOT / "tools" / "rustc-164250" / "probes"
PINNED_RELEASE = "1.80.0"
PINNED_COMMIT = "051478957371ee0084a7c0913941d2a8c4757bb9"
PINNED_DATE = "2024-07-21"
DEFAULT_HOST = "x86_64-unknown-linux-gnu"
TIMEOUT_SECONDS = 90

# Updating a fixture requires rechecking both its identity and diagnostic site.
FIXTURES = {
    "issue_164250.rs": "dfe7c0be2baf3fcf453592a495b29427d2dc47451d743d06d5e1543a48d701ef",
    "explicit_type_control.rs": "2f834c91542b2c3a7146a16ac914711f3660b756447d85532a7aa71069f0c2b2",
}


class Inconclusive(Exception):
    """The compiler or controls did not produce interpretable evidence."""


def report_output(label: str, stdout: bytes, stderr: bytes) -> None:
    # Keep both streams in the Actions log, including rejected diagnostics.
    for stream, data in (("stdout", stdout), ("stderr", stderr)):
        if data:
            print(f"--- {label} {stream} ---")
            rendered = data.decode("utf-8", errors="backslashreplace")
            print(rendered, end="" if rendered.endswith("\n") else "\n")


def invoke(label: str, argv: list[str]) -> subprocess.CompletedProcess[bytes]:
    print(f"{label} command: {json.dumps(argv)}", flush=True)
    try:
        run = subprocess.run(
            argv, cwd=ROOT, capture_output=True, timeout=TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        report_output(label, error.stdout or b"", error.stderr or b"")
        raise Inconclusive(f"{label} timed out") from error
    report_output(label, run.stdout, run.stderr)
    print(f"{label} exit status: {run.returncode}")
    return run


def verify_identity(rustc: str, version: str, commit: str, host: str) -> None:
    result = invoke("rustc identity", [rustc, "-vV"])
    if result.returncode != 0:
        raise Inconclusive("could not obtain rustc identity")
    lines = result.stdout.decode("utf-8").splitlines()
    fields = {}
    for line in lines[1:]:
        key, sep, value = line.partition(": ")
        if not sep or key in fields:
            raise Inconclusive(f"malformed rustc -vV identity field: {line!r}")
        fields[key] = value
    date = fields.get("commit-date", "")
    if (
        fields.get("binary") != "rustc"
        or fields.get("release") != version
        or fields.get("commit-hash") != commit
        or fields.get("host") != host
        or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", date)
        or (version == PINNED_RELEASE and date != PINNED_DATE)
        or not lines
        or lines[0] != f"rustc {version} ({commit[:9]} {date})"
    ):
        raise Inconclusive(f"rustc identity mismatch; expected release={version}, commit={commit}, host={host}")


def diagnostics(raw: bytes) -> list[dict]:
    messages = []
    for line in raw.decode("utf-8").splitlines():
        if not line.strip():
            continue
        entry = json.loads(line)
        if not isinstance(entry, dict) or entry.get("$message_type") != "diagnostic":
            raise Inconclusive("unexpected rustc JSON output")
        validate_diagnostic(entry, child=False)
        messages.append(entry)
    return messages


def validate_diagnostic(entry: dict, *, child: bool) -> None:
    level = entry.get("level")
    valid_levels = ("note", "help") if child else ("error", "warning", "note", "help", "failure-note")
    message = entry.get("message")
    if level not in valid_levels or not isinstance(message, str):
        raise Inconclusive("unexpected diagnostic level or message")
    if "internal compiler error" in message.lower() or "panicked at" in message.lower():
        raise Inconclusive("compiler internal error")
    if (
        "code" not in entry
        or not isinstance(entry.get("spans"), list)
        or not all(isinstance(span, dict) for span in entry["spans"])
        or not isinstance(entry.get("children"), list)
    ):
        raise Inconclusive("malformed rustc diagnostic")
    code = entry.get("code")
    if code is not None and (not isinstance(code, dict) or not isinstance(code.get("code"), str)):
        raise Inconclusive("malformed rustc diagnostic code")
    for item in entry["children"]:
        if not isinstance(item, dict):
            raise Inconclusive("malformed rustc child diagnostic")
        validate_diagnostic(item, child=True)


def intended_lifetime_error(messages: list[dict], source: Path) -> bool:
    errors = [item for item in messages if item["level"] == "error"]

    # Rustc's abort summary is an uncoded, span-less error record.
    actual = [
        item for item in errors
        if not (
            item["message"] == "aborting due to 1 previous error"
            and item["code"] is None
            and not item["spans"]
            and not item["children"]
        )
    ]
    if len(actual) != 1:
        return False
    error = actual[0]
    if (
        error["message"] != "`local` does not live long enough"
        or (error["code"] or {}).get("code") != "E0597"
    ):
        return False
    primaries = [span for span in error["spans"] if span.get("is_primary") is True]
    if len(primaries) != 1:
        return False
    span = primaries[0]
    return (
        span.get("file_name") == str(source)
        and span.get("line_start") == span.get("line_end") == 8
        and span.get("column_start") in (20, 21)
        and span.get("column_end") == 26
    )


def compile_control(rustc: str, source: Path, *, expects_borrow_diagnostic: bool) -> bool:
    """Return True for successful metadata, False for the reviewed rejection."""
    label = source.name
    output = source.with_suffix(".rmeta")
    result = invoke(
        label,
        [
            rustc,
            "--edition=2021",
            "--crate-type=lib",
            "--emit=metadata",
            "--error-format=json",
            "-o",
            str(output),
            str(source),
        ],
    )
    if result.returncode not in (0, 1):
        raise Inconclusive(f"{label} did not exit normally with a compile result")
    if result.stdout:
        raise Inconclusive(f"{label} produced unexpected stdout")
    messages = diagnostics(result.stderr)
    if result.returncode == 0:
        if any(item["level"] == "error" for item in messages):
            raise Inconclusive(f"{label} exited successfully with an error diagnostic")
        if output.is_symlink() or not output.is_file():
            raise Inconclusive(f"{label} did not create nonempty metadata")
        with output.open("rb") as metadata:
            if not metadata.read(1):
                raise Inconclusive(f"{label} created empty metadata")
        return True
    if output.exists() or output.is_symlink():
        raise Inconclusive(f"{label} failed but produced a metadata path")
    if not expects_borrow_diagnostic or not intended_lifetime_error(messages, source):
        raise Inconclusive(f"{label} failed without the reviewed E0597 borrow-site diagnostic")
    print(f"{label}: rejected with E0597 at the expected borrow site")
    return False


def confirmed_sources(directory: Path) -> dict[str, Path]:
    sources = {}
    for name, digest in FIXTURES.items():
        original = (PROBES / name).read_bytes()
        if hashlib.sha256(original).hexdigest() != digest:
            raise Inconclusive(f"unrecognized fixture bytes: {name}")
        source = directory / name
        source.write_bytes(original)
        sources[name] = source
    return sources


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--expect", choices=("affected", "fixed"), required=True)
    parser.add_argument("--expected-version")
    parser.add_argument("--expected-commit", help="full 40-digit rustc commit hash")
    parser.add_argument("--expected-host")
    args = parser.parse_args(argv)

    version = args.expected_version or PINNED_RELEASE
    if (
        args.expect == "fixed" or version != PINNED_RELEASE
    ) and not (args.expected_version and args.expected_commit and args.expected_host):
        parser.error("fixed mode or a non-pinned release requires explicit version, commit and host")
    commit = args.expected_commit or PINNED_COMMIT
    host = args.expected_host or DEFAULT_HOST
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        parser.error("--expected-commit must be a full lowercase SHA-1")
    if version == PINNED_RELEASE and commit != PINNED_COMMIT:
        parser.error("the pinned release requires its recorded official commit")

    try:
        selected = shutil.which(args.rustc)
        if selected is None:
            raise Inconclusive(f"rustc executable not found: {args.rustc!r}")
        # Preserve the selected name because rustup's rustc proxy uses argv[0].
        rustc = str(Path(selected).absolute())
        verify_identity(rustc, version, commit, host)
        with tempfile.TemporaryDirectory(prefix="bowfin-rustc-164250-") as directory:
            folder = Path(directory).resolve()
            sources = confirmed_sources(folder)
            good = folder / "known_good.rs"
            good.write_text("pub fn known_good() -> u32 { 1 }\n", encoding="utf-8")
            compile_control(rustc, good, expects_borrow_diagnostic=False)
            explicit = compile_control(
                rustc, sources["explicit_type_control.rs"], expects_borrow_diagnostic=True,
            )
            if explicit:
                print("unexpected acceptance of the explicit-type control", file=sys.stderr)
                return 1
            affected = compile_control(
                rustc, sources["issue_164250.rs"], expects_borrow_diagnostic=True,
            )
    except (Inconclusive, OSError, UnicodeError, ValueError) as error:
        print(f"rustc-164250 probe inconclusive: {error}", file=sys.stderr)
        return 2

    observed = "affected" if affected else "fixed"
    if observed != args.expect:
        print(f"rustc-164250 status changed: expected {args.expect}, observed {observed}", file=sys.stderr)
        return 1
    print(f"Rust #164250 compiler probe OK: {version}; observed={observed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
