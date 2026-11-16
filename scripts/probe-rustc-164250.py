#!/usr/bin/env python3
"""Record whether the selected rustc exhibits Rust issue #164250.

The affected reproducer is compiled as metadata and is never executed.  The
explicit-type control must be rejected on both affected and fixed compilers.
Changing the expected status is a reviewed toolchain-policy change, not an
automatic consequence of installing a different compiler.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile


def compile_probe(rustc: str, source: Path, output: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            rustc,
            "--edition=2021",
            "--crate-type=lib",
            "--emit=metadata",
            "-o",
            str(output),
            str(source),
        ],
        capture_output=True,
        text=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--expect", choices=("affected", "fixed"), required=True)
    parser.add_argument("--expected-version")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    probes = root / "tools" / "rustc-164250" / "probes"
    version = subprocess.run(
        [args.rustc, "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    if args.expected_version and args.expected_version not in version:
        print(
            f"rustc-164250 probe failed: expected {args.expected_version!r}, got {version!r}",
            file=sys.stderr,
        )
        return 2

    with tempfile.TemporaryDirectory() as directory:
        temporary = Path(directory)
        affected = compile_probe(
            args.rustc,
            probes / "issue_164250.rs",
            temporary / "affected.rmeta",
        )
        explicit = compile_probe(
            args.rustc,
            probes / "explicit_type_control.rs",
            temporary / "explicit.rmeta",
        )

    observed = "affected" if affected.returncode == 0 else "fixed"
    if observed != args.expect:
        print(
            f"rustc-164250 status changed: expected {args.expect}, observed {observed}",
            file=sys.stderr,
        )
        if affected.stdout:
            print(affected.stdout, file=sys.stderr, end="")
        if affected.stderr:
            print(affected.stderr, file=sys.stderr, end="")
        return 1
    if explicit.returncode == 0:
        print(
            "rustc-164250 explicit-type control unexpectedly compiled",
            file=sys.stderr,
        )
        return 1

    print(f"Rust #164250 compiler probe OK: {version}; observed={observed}")
    print("Explicit-type mitigation control was rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
