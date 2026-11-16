# Bowfin Host Toolchain Constraints

This file records active compiler-host and build-tool constraints that can affect
the integrity of Bowfin's implementation. These are engineering constraints, not
Bowfin source-language semantics. A clean build does not override them.

Statuses:

- **Active** — every affected change must satisfy the stated source and CI gates;
- **Monitoring** — no current source restriction, but the recorded probe remains;
- **Retired** — historical only, with the reviewed retirement evidence retained.

## TC-001 — Rust local-type well-formedness defect (#164250)

**Status:** Active

**Applies to:** all repository Rust source, including tests, tools, generated
source checked into the repository, and BED-038 Stage N work before acceptance

**Pinned host compiler:** Rust 1.80.0 (`051478957 2024-07-21`)

**Upstream issue:** [rust-lang/rust#164250](https://github.com/rust-lang/rust/issues/164250)

**Proposed upstream repair:** [rust-lang/rust#164277](https://github.com/rust-lang/rust/pull/164277)

Rust may accept an inferred MIR local type without proving all well-formedness
requirements declared by that type. The reported form constructs a generic
aggregate with an unconstraining value and later narrows the inferred parameter
through field assignment. Upstream repair work also exercises other local-type
unification forms. Therefore Bowfin does not treat a search for one textual
reproducer, the absence of a malicious destructor, Miri success, or ordinary
`cargo check` success as a complete protection.

At the time this entry was created on 2026-11-16, the issue and repair pull
request were open. Bowfin treats its pinned compiler as affected unless the
repository probe and a reviewed toolchain change establish otherwise.

### Required source discipline

Repository Rust source may not introduce a `struct`, `enum`, or `union` whose
generic parameters or `where` clause impose a bound unless the exact declaration
has received focused review and appears in the machine-checked allowlist.

For any reviewed exception:

1. every well-formedness-bearing generic parameter is fixed explicitly at local
   construction or by an argument that fully constrains it;
2. fields carrying such parameters are private and are not used to change the
   effective inferred local type after construction;
3. mutation occurs only through an API whose signature restates the required
   bounds;
4. `unsafe`, `Drop`, trait-object conversion, cleanup, runtime authority, and
   evidence custody never rely solely on rustc rechecking an inferred local type
   after mutation or unification;
5. focused positive and negative controls cover construction, mutation,
   destruction, substitution, and any other relevant unification path.

The broad declaration rule is deliberate. It prevents the known family before a
future contributor has to recognize every exploitable expression shape.

### Mechanical enforcement

`scripts/verify-rustc-164250.py` independently tokenizes tracked Rust source.
It inspects declaration tokens inside macro bodies and nested items, and fails
closed for unresolved declaration syntax that could carry a bound. Rust code
splicing must resolve to a tracked Rust input already covered by the default
scan. The isolated scanner fixtures and probe crates are outside that coverage
and must never be spliced into scanned code. String and byte includes are data.

The allowlist at `tools/rustc-164250/allowlist.json` is empty. A future entry
must identify exactly one declaration, including its path, kind, name,
keyword byte offset, normalized-token SHA-256, and containing source-file
SHA-256. Opaque literal tokens preserve their spelling, including ABI strings.
The file digest also binds same-file attributes, scopes, and use sites. Macro
templates and spliced source declarations cannot use entries under this
mechanism. Cross-file call sites and toolchain identities remain part of
focused review. Duplicate actual matches, duplicate entries, and stale entries
fail closed.

`scripts/probe-rustc-164250.py` checks the selected compiler's identity and
verifies that it can compile a known-good input to metadata. Under the current
pin the affected-shape control must produce metadata, and the explicit-type
control must produce the intended lifetime diagnostic at the relevant source
site. Fixed mode requires a classified rejection of both controls. The driver
reports diagnostic evidence and fails closed on unrelated errors. These probes
record compiler behavior. The source guard is the preventive gate.

The guard, probe driver controls, and compile-only probe run on every push and
pull request. The full evidence and maintenance procedure are recorded in
`docs/engineering/RUST-164250.md`.

### Change and retirement rule

An upstream merge alone does not retire TC-001. A later change must identify the
first adopted fixed Rust release, invert the compiler probe to require rejection,
run the complete Bowfin suite, inspect regressions, update the engineering record,
receive independent review, and obtain Greg's decision. The conservative source
discipline remains in force unless that same reviewed decision expressly narrows
or retires it. Downgrading or replacing the host compiler reopens the applicability
check.

This gate addresses one known defect family. It is not a claim that rustc, Bowfin,
or the host environment contains no other defect.
