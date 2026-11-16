#!/usr/bin/env python3
"""Reject unreviewed Rust aggregate declarations with generic WF bounds.

Rust issue #164250 permits some inferred MIR local types to bypass the
well-formedness requirements declared by their aggregate type.  Bowfin's pinned
Rust 1.80.0 toolchain is treated as affected.  This guard deliberately enforces
a broader source rule: a struct, enum, or union with a bound on a generic
parameter must be absent or exactly listed in the reviewed allowlist.

The scanner tokenizes Rust source, including nested comments, raw strings,
character literals, lifetimes, balanced generic parameters, tuple fields, and
where clauses.  It is intentionally independent of rustc so it does not rely on
the compiler behavior it is guarding.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
import subprocess
import sys
from typing import Iterable, Sequence


ISSUE = "https://github.com/rust-lang/rust/issues/164250"
DEFAULT_ALLOWLIST = Path("tools/rustc-164250/allowlist.json")
EXCLUDED_PREFIXES = (
    "tools/rustc-164250/fixtures/",
    "tools/rustc-164250/probes/",
)


@dataclass(frozen=True)
class Token:
    text: str
    line: int


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    kind: str
    name: str
    declaration_sha256: str
    reason: str

    def identity(self) -> tuple[str, str, str, str]:
        return (self.path, self.kind, self.name, self.declaration_sha256)


def _raw_string_end(source: str, start: int) -> int | None:
    """Return the byte index after a Rust raw string beginning at start."""
    i = start
    if source.startswith("br", i) or source.startswith("rb", i):
        i += 2
    elif source.startswith("r", i):
        i += 1
    else:
        return None
    hashes = 0
    while i < len(source) and source[i] == "#":
        hashes += 1
        i += 1
    if i >= len(source) or source[i] != '"':
        return None
    closing = '"' + ("#" * hashes)
    end = source.find(closing, i + 1)
    return len(source) if end < 0 else end + len(closing)


def tokenize(source: str) -> list[Token]:
    tokens: list[Token] = []
    i = 0
    line = 1
    length = len(source)

    def skip_quoted(start: int, quote: str) -> int:
        cursor = start + 1
        while cursor < length:
            if source[cursor] == "\\":
                cursor += 2
                continue
            if source[cursor] == quote:
                return cursor + 1
            cursor += 1
        return length

    while i < length:
        ch = source[i]
        if ch.isspace():
            if ch == "\n":
                line += 1
            i += 1
            continue

        if source.startswith("//", i):
            end = source.find("\n", i + 2)
            if end < 0:
                break
            i = end
            continue

        if source.startswith("/*", i):
            depth = 1
            cursor = i + 2
            while cursor < length and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    if source[cursor] == "\n":
                        line += 1
                    cursor += 1
            i = cursor
            continue

        raw_end = _raw_string_end(source, i)
        if raw_end is not None:
            line += source.count("\n", i, raw_end)
            i = raw_end
            continue

        if source.startswith('b"', i):
            end = skip_quoted(i + 1, '"')
            line += source.count("\n", i, end)
            i = end
            continue
        if ch == '"':
            end = skip_quoted(i, '"')
            line += source.count("\n", i, end)
            i = end
            continue

        if source.startswith("b'", i):
            end = skip_quoted(i + 1, "'")
            line += source.count("\n", i, end)
            i = end
            continue

        if ch == "'":
            # A lifetime is apostrophe + identifier without a closing quote.
            cursor = i + 1
            if cursor < length and (source[cursor].isalpha() or source[cursor] == "_"):
                cursor += 1
                while cursor < length and (
                    source[cursor].isalnum() or source[cursor] == "_"
                ):
                    cursor += 1
                if cursor >= length or source[cursor] != "'":
                    tokens.append(Token(source[i:cursor], line))
                    i = cursor
                    continue
            end = skip_quoted(i, "'")
            line += source.count("\n", i, end)
            i = end
            continue

        if ch.isalpha() or ch == "_":
            cursor = i + 1
            while cursor < length and (
                source[cursor].isalnum() or source[cursor] == "_"
            ):
                cursor += 1
            tokens.append(Token(source[i:cursor], line))
            i = cursor
            continue

        two = source[i : i + 2]
        if two in {"::", "->", "=>", "<=", ">=", "==", "!=", "&&", "||"}:
            tokens.append(Token(two, line))
            i += 2
            continue

        tokens.append(Token(ch, line))
        i += 1

    return tokens


def _split_top_level(tokens: Sequence[Token], separator: str) -> list[list[Token]]:
    result: list[list[Token]] = [[]]
    angle = paren = bracket = brace = 0
    for token in tokens:
        text = token.text
        if text == "<":
            angle += 1
        elif text == ">" and angle:
            angle -= 1
        elif text == "(":
            paren += 1
        elif text == ")" and paren:
            paren -= 1
        elif text == "[":
            bracket += 1
        elif text == "]" and bracket:
            bracket -= 1
        elif text == "{":
            brace += 1
        elif text == "}" and brace:
            brace -= 1
        if text == separator and not (angle or paren or bracket or brace):
            result.append([])
        else:
            result[-1].append(token)
    return result


def _generic_parameter_has_bound(segment: Sequence[Token]) -> bool:
    meaningful = [token.text for token in segment if token.text]
    if not meaningful or meaningful[0] == "const":
        return False
    return ":" in meaningful


def _find_matching_angle(tokens: Sequence[Token], start: int) -> int | None:
    depth = 0
    for index in range(start, len(tokens)):
        if tokens[index].text == "<":
            depth += 1
        elif tokens[index].text == ">":
            depth -= 1
            if depth == 0:
                return index
    return None


def find_bound_aggregates(source: str, path: str) -> list[Finding]:
    tokens = tokenize(source)
    findings: list[Finding] = []
    index = 0
    while index < len(tokens):
        kind = tokens[index].text
        if kind not in {"struct", "enum", "union"}:
            index += 1
            continue
        if index + 1 >= len(tokens):
            break
        name_token = tokens[index + 1]
        if not (name_token.text[:1].isalpha() or name_token.text.startswith("_")):
            index += 1
            continue

        cursor = index + 2
        generic_tokens: list[Token] = []
        if cursor < len(tokens) and tokens[cursor].text == "<":
            end = _find_matching_angle(tokens, cursor)
            if end is None:
                index += 1
                continue
            generic_tokens = list(tokens[cursor + 1 : end])
            cursor = end + 1

        # Find the header terminator while allowing tuple-struct fields before a
        # trailing where clause.  The opening brace of a braced aggregate ends
        # the header; a tuple field's parentheses are balanced first.
        paren = bracket = angle = 0
        header_end = cursor
        where_index: int | None = None
        while header_end < len(tokens):
            text = tokens[header_end].text
            if text == "(":
                paren += 1
            elif text == ")" and paren:
                paren -= 1
            elif text == "[":
                bracket += 1
            elif text == "]" and bracket:
                bracket -= 1
            elif text == "<":
                angle += 1
            elif text == ">" and angle:
                angle -= 1
            elif not (paren or bracket or angle):
                if text == "where" and where_index is None:
                    where_index = header_end
                if text in {"{", ";"}:
                    break
            header_end += 1

        generic_bound = any(
            _generic_parameter_has_bound(segment)
            for segment in _split_top_level(generic_tokens, ",")
        )
        where_bound = False
        if where_index is not None:
            where_tokens = tokens[where_index + 1 : header_end]
            where_bound = any(
                ":" in [token.text for token in segment]
                for segment in _split_top_level(where_tokens, ",")
            )

        declaration_end = header_end
        if header_end < len(tokens) and tokens[header_end].text == "{":
            brace_depth = 0
            for declaration_end in range(header_end, len(tokens)):
                if tokens[declaration_end].text == "{":
                    brace_depth += 1
                elif tokens[declaration_end].text == "}":
                    brace_depth -= 1
                    if brace_depth == 0:
                        break

        if generic_bound or where_bound:
            normalized = " ".join(
                token.text
                for token in tokens[index : min(declaration_end + 1, len(tokens))]
            )
            findings.append(
                Finding(
                    path=path,
                    line=tokens[index].line,
                    kind=kind,
                    name=name_token.text,
                    declaration_sha256=hashlib.sha256(
                        normalized.encode("utf-8")
                    ).hexdigest(),
                    reason=(
                        "generic-parameter bound"
                        if generic_bound and not where_bound
                        else "where-clause bound"
                        if where_bound and not generic_bound
                        else "generic-parameter and where-clause bounds"
                    ),
                )
            )
        index = max(index + 1, declaration_end + 1)
    return findings


def _tracked_rust_paths(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", "*.rs"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    paths = []
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8")
        if relative.startswith(EXCLUDED_PREFIXES):
            continue
        paths.append(root / relative)
    return paths


def _load_allowlist(path: Path) -> list[dict[str, object]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != 1 or data.get("issue") != ISSUE:
        raise ValueError(f"invalid allowlist header: {path}")
    entries = data.get("entries")
    if not isinstance(entries, list):
        raise ValueError(f"allowlist entries must be a list: {path}")
    required = {
        "path",
        "kind",
        "name",
        "declaration_sha256",
        "rationale",
        "review",
        "controls",
    }
    identities: set[tuple[str, str, str, str]] = set()
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != required:
            raise ValueError(f"invalid allowlist entry shape: {entry!r}")
        if not isinstance(entry["rationale"], str) or not entry["rationale"].strip():
            raise ValueError(f"allowlist rationale is empty: {entry!r}")
        if not isinstance(entry["review"], str) or not entry["review"].startswith("https://"):
            raise ValueError(f"allowlist review must be an https URL: {entry!r}")
        if not isinstance(entry["controls"], list) or not entry["controls"]:
            raise ValueError(f"allowlist controls must be nonempty: {entry!r}")
        if not all(isinstance(item, str) and item.strip() for item in entry["controls"]):
            raise ValueError(f"allowlist controls contain an empty value: {entry!r}")
        identity = (
            str(entry["path"]),
            str(entry["kind"]),
            str(entry["name"]),
            str(entry["declaration_sha256"]),
        )
        if identity in identities:
            raise ValueError(f"duplicate allowlist entry: {identity!r}")
        identities.add(identity)
    return entries


def check(
    root: Path,
    paths: Iterable[Path],
    allowlist_path: Path,
) -> tuple[list[Finding], list[str]]:
    findings: list[Finding] = []
    for path in paths:
        relative = path.resolve().relative_to(root.resolve()).as_posix()
        findings.extend(find_bound_aggregates(path.read_text(encoding="utf-8"), relative))

    entries = _load_allowlist(allowlist_path)
    allowed = {
        (
            str(entry["path"]),
            str(entry["kind"]),
            str(entry["name"]),
            str(entry["declaration_sha256"]),
        ): entry
        for entry in entries
    }
    errors: list[str] = []
    actual = {finding.identity(): finding for finding in findings}
    for finding in findings:
        if finding.identity() not in allowed:
            errors.append(
                f"{finding.path}:{finding.line}: unreviewed {finding.kind} "
                f"{finding.name} has {finding.reason}; declaration sha256 "
                f"{finding.declaration_sha256}"
            )
    for identity in sorted(set(allowed) - set(actual)):
        errors.append(
            "stale rustc-164250 allowlist entry: " + "/".join(identity[:3])
            + f" sha256={identity[3]}"
        )
    return findings, errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--allowlist", type=Path)
    parser.add_argument("paths", nargs="*", type=Path)
    args = parser.parse_args(argv)
    root = args.root.resolve()
    allowlist = args.allowlist or (root / DEFAULT_ALLOWLIST)
    paths = [path.resolve() for path in args.paths] or _tracked_rust_paths(root)
    try:
        findings, errors = check(root, paths, allowlist.resolve())
    except (OSError, UnicodeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"rustc-164250 guard failed closed: {error}", file=sys.stderr)
        return 2
    if errors:
        print("Rust #164250 source guard rejected the repository:", file=sys.stderr)
        for error in errors:
            print(f" - {error}", file=sys.stderr)
        print(f"See {ISSUE} and docs/ai/TOOLCHAIN_CONSTRAINTS.md", file=sys.stderr)
        return 1
    print(
        "Rust #164250 source guard OK: "
        f"{len(paths)} tracked Rust files, {len(findings)} reviewed bounded aggregates"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
