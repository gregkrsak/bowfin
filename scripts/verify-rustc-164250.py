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
from collections import Counter
import hashlib
import json
import os
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
import re
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
    offset: int
    literal: bool = False


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    kind: str
    name: str
    declaration_sha256: str
    file_sha256: str
    byte_offset: int
    reason: str
    in_macro: bool

    def identity(self) -> tuple[str, str, str, str, str, int]:
        return (self.path, self.kind, self.name, self.declaration_sha256,
                self.file_sha256, self.byte_offset)


RAW_STRING = re.compile(r'(?:br|cr|r)(#*)"')
NUMERIC_LITERAL = re.compile(
    r"(?:0x[0-9a-fA-F_]+|0b[01_]+|0o[0-7_]+|"
    r"[0-9][0-9_]*(?:\.(?:[0-9][0-9_]*|(?=[^\w.]|$)))?(?:[eE][+-]?[0-9_]+)?)"
    r"(?:[^\W\d]\w*)?"
)
GROUP_END = {"(": ")", "[": "]", "{": "}"}
DECLARATIONS = {"struct", "enum", "union"}


def _identifier_end(source: str, start: int) -> int:
    i = start + 2 if source.startswith("r#", start) else start
    if i >= len(source) or not source[i].isidentifier():
        return start
    i += 1
    while i < len(source) and ("a" + source[i]).isidentifier():
        i += 1
    return i


def _identifier(text: str) -> bool:
    return text.removeprefix("r#").isidentifier()


def tokenize(source: str) -> list[Token]:
    tokens: list[Token] = []
    i = 0
    line = 1
    length = len(source)

    def quoted_end(start: int) -> int:
        quote = source[start]
        cursor = start + 1
        while cursor < length:
            if source[cursor] == "\\":
                cursor += 2
                continue
            if source[cursor] == quote:
                return cursor + 1
            cursor += 1
        raise ValueError(f"line {line}: unterminated quoted literal")

    while i < length:
        ch = source[i]
        if ch.isspace() or (i == 0 and ch == "\ufeff"):
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
                    cursor += 1
            if depth:
                raise ValueError(f"line {line}: unterminated block comment")
            line += source.count("\n", i, cursor)
            i = cursor
            continue

        raw = RAW_STRING.match(source, i)
        literal = True
        if raw:
            closing = '"' + raw[1]
            end = source.find(closing, raw.end())
            if end < 0:
                raise ValueError(f"line {line}: unterminated raw literal")
            end += len(closing)
        elif source.startswith(('b"', 'c"', "b'"), i):
            end = quoted_end(i + 1)
        elif ch == '"':
            end = quoted_end(i)
        elif ch == "'":
            end = _identifier_end(source, i + 1)
            if end > i + 1 and (end == length or source[end] != "'"):
                literal = False  # lifetime, including '_
            else:
                end = quoted_end(i)
        elif numeric := NUMERIC_LITERAL.match(source, i):
            end = numeric.end()
        else:
            literal = False
            end = _identifier_end(source, i)
            if end == i:
                end = i + (2 if source[i:i + 2] in {"::", "->", "=>", "!="} else 1)

        # Literal spelling is opaque to the scanner, but is part of the hash.
        tokens.append(Token(source[i:end], line, i, literal))
        line += source.count("\n", i, end)
        i = end
    return tokens


def _groups(tokens: Sequence[Token]) -> dict[int, int]:
    groups = {}
    stack = []
    for i, token in enumerate(tokens):
        if token.text in GROUP_END:
            stack.append(i)
        elif token.text in GROUP_END.values():
            if not stack or GROUP_END[tokens[stack[-1]].text] != token.text:
                raise ValueError(f"line {token.line}: unbalanced Rust delimiter")
            groups[stack.pop()] = i
    if stack:
        raise ValueError(f"line {tokens[stack[-1]].line}: unclosed Rust delimiter")
    return groups


def _angle_end(tokens: Sequence[Token], groups: dict[int, int], start: int) -> int:
    cursor = start + 1
    depth = 1
    while cursor < len(tokens):
        if cursor in groups:
            cursor = groups[cursor] + 1
            continue
        text = tokens[cursor].text
        if text in {"{", "}", "]", ")", ";"}:
            break
        depth += (text == "<") - (text == ">")
        if depth == 0:
            return cursor
        cursor += 1
    raise ValueError(f"line {tokens[start].line}: unresolved generic parameter syntax")


def _top_level(
    tokens: Sequence[Token], groups: dict[int, int], start: int, stop: int,
) -> Iterable[int]:
    cursor = start
    while cursor < stop:
        yield cursor
        if cursor in groups:
            cursor = groups[cursor] + 1
        elif tokens[cursor].text == "<":
            cursor = _angle_end(tokens, groups, cursor) + 1
        else:
            cursor += 1


def _generic_bound(
    tokens: Sequence[Token], groups: dict[int, int], start: int, stop: int,
) -> bool:
    # Inspect parameter heads. A const parameter's colon is a type annotation.
    generic_bound = False
    ends = [i for i in _top_level(tokens, groups, start, stop) if tokens[i].text == ","]
    for end in [*ends, stop]:
        while start + 1 < end and tokens[start].text == "#" and tokens[start + 1].text == "[":
            start = groups[start + 1] + 1
        if start < end:
            head = tokens[start].text
            if not _identifier(head.removeprefix("'")):
                raise ValueError(f"line {tokens[start].line}: unresolved generic parameter head")
            if head != "const" and start + 1 < end:
                following = tokens[start + 1].text
                if following not in {":", "="}:
                    raise ValueError(f"line {tokens[start].line}: unresolved generic parameter")
                generic_bound |= following == ":"
        start = end + 1
    return generic_bound


def _macro_body_ranges(tokens: Sequence[Token], groups: dict[int, int]) -> list[tuple[int, int]]:
    return [
        (i + 3, groups[i + 3]) for i, token in enumerate(tokens)
        if token.text == "macro_rules" and i + 3 in groups
        and tokens[i + 1].text == "!" and _identifier(tokens[i + 2].text)
    ]


def _find_bound_aggregates(
    source: str, path: str, tokens: Sequence[Token], groups: dict[int, int],
) -> list[Finding]:
    findings = []
    macro_ranges = _macro_body_ranges(tokens, groups)
    file_hash = hashlib.sha256(source.encode("utf-8")).hexdigest()
    # Do not skip a recognized declaration body: it may contain nested items.
    for index, keyword in enumerate(tokens):
        kind = keyword.text
        if kind not in DECLARATIONS:
            continue
        cursor = index + 1
        preceding = tokens[index - 1].text if index else None
        if kind == "union" and preceding in {".", "::", "fn", "struct", "enum", "union", "$"}:
            continue  # contextual identifier in a field, path, name or metavariable
        if cursor < len(tokens) and tokens[cursor].text == "$":
            cursor += 1
            name_prefix = "$"
        else:
            name_prefix = ""
        if cursor >= len(tokens) or not _identifier(tokens[cursor].text):
            raise ValueError(f"line {keyword.line}: unresolved {kind} declaration name")
        name = name_prefix + tokens[cursor].text
        cursor += 1

        generic_bound = False
        if cursor < len(tokens) and tokens[cursor].text == "<":
            end = _angle_end(tokens, groups, cursor)
            generic_bound = _generic_bound(tokens, groups, cursor + 1, end)
            cursor = end + 1
        if kind == "struct" and cursor < len(tokens) and tokens[cursor].text == "(":
            cursor = groups[cursor] + 1

        where_bound = False
        if cursor < len(tokens) and tokens[cursor].text == "where":
            start = cursor + 1
            for cursor in _top_level(tokens, groups, start, len(tokens)):
                if tokens[cursor].text in {"{", ";"}:
                    break
            # Any nonempty predicate is conservative evidence of a constraint.
            where_bound = cursor > start
        terminators = {"{", ";"} if kind == "struct" else {"{"}
        if cursor >= len(tokens) or tokens[cursor].text not in terminators:
            raise ValueError(f"line {keyword.line}: unresolved {kind} {name} declaration header")
        end = groups.get(cursor, cursor)

        if generic_bound or where_bound:
            normalized = json.dumps([token.text for token in tokens[index:end + 1]],
                                    ensure_ascii=False, separators=(",", ":"))
            findings.append(Finding(
                path=path,
                line=keyword.line,
                kind=kind,
                name=name,
                declaration_sha256=hashlib.sha256(normalized.encode("utf-8")).hexdigest(),
                file_sha256=file_hash,
                byte_offset=len(source[:keyword.offset].encode("utf-8")),
                reason=(
                    "generic-parameter and where-clause bounds" if generic_bound and where_bound
                    else "generic-parameter bound" if generic_bound else "where-clause bound"
                ),
                in_macro=any(first < index < last for first, last in macro_ranges),
            ))
    return findings


def find_bound_aggregates(source: str, path: str) -> list[Finding]:
    tokens = tokenize(source)
    return _find_bound_aggregates(source, path, tokens, _groups(tokens))


def _path_literal(token: Token) -> str:
    text = token.text
    if token.literal and text.startswith('"') and "\\" not in text:
        value = text[1:-1]
    elif token.literal and text.startswith("r"):
        opening_quote = text.index('"')
        value = text[opening_quote + 1:-opening_quote]
    else:
        raise ValueError(f"line {token.line}: unresolved Rust source path (need a literal path)")
    if not value or any(ch in value for ch in ("\0", "\\", "\n", "\r")) or Path(value).is_absolute():
        raise ValueError(f"line {token.line}: unresolved Rust source path")
    return value


def _source_inputs(
    tokens: Sequence[Token], groups: dict[int, int],
) -> Iterable[tuple[int, str, str]]:
    macro_ranges = _macro_body_ranges(tokens, groups)
    for i, token in enumerate(tokens):
        if token.text.removeprefix("r#") == "include":
            if i + 2 not in groups or tokens[i + 1].text != "!":
                raise ValueError(f"line {token.line}: unresolved use of Rust include macro name")
            if any(first < i < last for first, last in macro_ranges):
                raise ValueError(f"line {token.line}: unresolved include! base in macro template")
            args = tokens[i + 3:groups[i + 2]]
            if args and args[-1].text == ",":
                args = args[:-1]
            if len(args) != 1:
                raise ValueError(f"line {token.line}: unresolved include! input (need one literal path)")
            yield token.line, _path_literal(args[0]), "include"
        if token.text == "$" and i + 2 < len(tokens) and (
            _identifier(tokens[i + 1].text) and tokens[i + 2].text == "!"
        ):
            raise ValueError(f"line {token.line}: unresolved dynamically named macro invocation")
        if token.text == "#" and i + 1 < len(tokens) and tokens[i + 1].text == "[":
            # Source-file-level #[path] is relative to the file's directory.
            # Inside an inline module the base differs, so leave that case closed.
            end = groups[i + 1]
            for j in range(i + 2, end - 1):
                if tokens[j].text.removeprefix("r#") != "path" or tokens[j + 1].text != "=":
                    continue
                if any(first < i < last and tokens[first].text == "{" for first, last in groups.items()):
                    raise ValueError(f"line {tokens[j].line}: unresolved nested #[path] base")
                if any(first < i < last for first, last in macro_ranges):
                    raise ValueError(f"line {tokens[j].line}: unresolved #[path] base in macro template")
                yield tokens[j].line, _path_literal(tokens[j + 2]), "path"


def _resolve_source_input(root: Path, parent: Path, name: str) -> Path:
    # Interpret '..' stepwise so that an intermediate symlink cannot change
    # the base directory before a later segment cancels its spelling.
    target = parent
    for segment in PurePosixPath(name).parts:
        target = target.parent if segment == ".." else target / segment
        if not target.is_relative_to(root) or target.is_symlink():
            raise ValueError(f"Rust source input {name!r} escapes the root or uses a symlink")
    if target != Path(os.path.normpath(parent / name)):
        raise ValueError(f"unresolved Rust source input path: {name!r}")
    if target.resolve(strict=True) != target:
        raise ValueError(f"Rust source input {name!r} aliases a different path")
    return target


def _tracked_rust_paths(root: Path) -> tuple[list[Path], int]:
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", "*.rs"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    paths = []
    excluded = 0
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8")
        if relative.startswith(EXCLUDED_PREFIXES):
            excluded += 1
        else:
            paths.append(root / relative)
    return paths, excluded


def _entry_identity(entry: dict[str, object]) -> tuple[str, str, str, str, str, int]:
    parts = [entry[field] for field in ("path", "kind", "name", "declaration_sha256", "file_sha256")]
    offset = entry["byte_offset"]
    if any(not isinstance(part, str) or not part for part in parts) or type(offset) is not int or offset < 0:
        raise ValueError(f"invalid allowlist declaration identity: {entry!r}")
    path, kind, name, digest, file_digest = parts
    file = PurePosixPath(path)
    if (file.is_absolute() or ".." in file.parts or file.as_posix() != path
        or kind not in DECLARATIONS or not _identifier(name)
        or any(not re.fullmatch("[0-9a-f]{64}", value) for value in (digest, file_digest))):
        raise ValueError(f"invalid allowlist declaration identity: {entry!r}")
    return path, kind, name, digest, file_digest, offset


def _unique_fields(pairs: list[tuple[str, object]]) -> dict[str, object]:
    fields = {}
    for key, value in pairs:
        if key in fields:
            raise ValueError(f"duplicate allowlist JSON key: {key!r}")
        fields[key] = value
    return fields


def _load_allowlist(path: Path) -> set[tuple[str, str, str, str, str, int]]:
    data = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=_unique_fields)
    if (not isinstance(data, dict) or set(data) != {"schema", "issue", "entries"}
        or type(data["schema"]) is not int or data["schema"] != 2 or data["issue"] != ISSUE):
        raise ValueError(f"invalid allowlist header: {path}")
    entries = data["entries"]
    if not isinstance(entries, list):
        raise ValueError(f"allowlist entries must be a list: {path}")
    required = {
        "path",
        "kind",
        "name",
        "declaration_sha256",
        "file_sha256",
        "byte_offset",
        "rationale",
        "review",
        "controls",
    }
    identities = set()
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
        identity = _entry_identity(entry)
        if identity in identities:
            raise ValueError(f"duplicate allowlist entry: {identity!r}")
        identities.add(identity)
    return identities


def check(
    root: Path,
    paths: Iterable[Path],
    allowlist_path: Path,
) -> tuple[list[Finding], list[str]]:
    allowed = _load_allowlist(allowlist_path)
    root = root.resolve()
    files = []
    for path in paths:
        # Use the Git spelling for coverage and refuse symlink aliases.
        file = path.absolute()
        file.relative_to(root)
        if file.resolve(strict=True) != file or not file.is_file():
            raise ValueError(f"Rust source is not a regular, unaliased file: {file}")
        files.append(file)
    selected = set(files)
    imported = set()
    spliced = set()
    path_sources = {}
    findings = []
    errors = []
    for file in files:
        relative = file.relative_to(root).as_posix()
        source = file.read_bytes().decode("utf-8")
        try:
            tokens = tokenize(source)
            groups = _groups(tokens)
            findings.extend(_find_bound_aggregates(source, relative, tokens, groups))
            for line, name, mode in _source_inputs(tokens, groups):
                target = _resolve_source_input(root, file.parent, name)
                target_relative = target.relative_to(root).as_posix()
                if (target not in selected or target.suffix != ".rs"
                    or target_relative.startswith(EXCLUDED_PREFIXES)):
                    errors.append(
                        f"{relative}:{line}: Rust source input {name!r} is outside "
                        "the scanned nonexcluded .rs paths"
                    )
                else:
                    imported.add(target_relative)
                if mode == "include":
                    spliced.add(target_relative)
                else:
                    path_sources.setdefault(relative, []).append(line)
        except (OSError, UnicodeError, ValueError) as error:
            errors.append(f"{relative}: Rust source needs review: {error}")

    for file in sorted(spliced & path_sources.keys()):
        for line in path_sources[file]:
            errors.append(f"{file}:{line}: unresolved #[path] base in code-spliced file")

    actual = Counter(finding.identity() for finding in findings)
    for finding in findings:
        identity = finding.identity()
        diagnostic = f"{finding.path}:{finding.line}:"
        if actual[identity] != 1:
            errors.append(f"{diagnostic} duplicate actual declaration identity: {identity!r}")
        elif finding.in_macro or finding.path in imported:
            # A written declaration can expand in multiple lexical contexts.
            errors.append(
                f"{diagnostic} {finding.kind} {finding.name} in a macro template or "
                "code-spliced file requires source repair, not an exception"
            )
        elif identity not in allowed:
            errors.append(
                f"{diagnostic} unreviewed {finding.kind} {finding.name} has {finding.reason}; "
                f"declaration sha256 {finding.declaration_sha256}, "
                f"file sha256 {finding.file_sha256}, byte offset {finding.byte_offset}"
            )
    for identity in sorted(allowed - actual.keys()):
        path, kind, name, digest, file_digest, offset = identity
        errors.append(
            f"stale rustc-164250 allowlist entry: {path}/{kind}/{name} "
            f"sha256={digest} file_sha256={file_digest} byte_offset={offset}"
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
    try:
        if args.paths:
            paths, excluded = args.paths, 0
        else:
            paths, excluded = _tracked_rust_paths(root)
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
        f"{len(paths)} scanned Rust files; {excluded} excluded fixtures/probes; "
        f"{len(findings)} reviewed bounded aggregates"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
