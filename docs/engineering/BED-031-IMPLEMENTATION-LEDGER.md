# BED-031 implementation ledger

## Authority and provenance

Frozen [Issue #72 Draft 4](https://github.com/gregkrsak/klyxr/issues/72), decision
6252193611, Sarah confirmation 6252268406, freeze 6252312068, implementation
6252329672, and pressure review 6251651535 control. Greg's complete fresh local-first
directive supersedes the earlier emergency GitHub-only policy. The production
Arnold charter was read at governance revision
`2887c9239fb6f82ded4fd7b2d8f5f592b725cccc`.

Exact authorized predecessor commit: `63a89f2062a33e5cec452df1ee0b3b2f27781f55`.
Tree: `653afdad686e1d5280427e11ff233463d4dc9dab`.
Branch: `ked-031-typed-recoverable-failure`; PR target: `prod`.
The unchanged predecessor gate is accepted evidence and was not repeated as a
startup requirement. Its exact 1,790 identities are retained in
`ked-031-predecessors.json`, derived from the unchanged KED-030 inventory plus
its 47 classified additions. Historical manifests and reconcilers are preserved.

The resume audit recorded `git status --short`, inspected all existing tracked
and untracked implementation changes, and ran `git diff --check`. They belonged
to KED-031. The existing worktree was continued without reset, clean, deletion,
overwrite or branch recreation. Local changes during implementation are expected.

## Implementation

One early canonical Error tag serves both exact source aliases Error and Err.
The lexer reserves the complete ASCII case-insensitive Result/Ok/Error/Err/match
families. Parser and resolver both enforce identifier reservation. Source type
syntax uses a finite postorder graph; canonical ordered payload pairs use a
separate ResultTypeId DAG. No RecordId or reference referent counterfeits Result.
Complete table validation includes unused rows, exact IDs, categories, uniqueness,
ordering, cycles and depth. Contextual constructors require a complete exact type;
raw named-range literal formation and conditional Result initialization remain
excluded. Pattern coverage normalizes aliases before checking.

The existing ownership checker alone tracks Move/loan permission and unresolved
outcome facts. Facts survive conservative conditional Move state and short-circuit
skips. Every successful scope/arm/function/loop exit validates duties before state
reduction. Transfers record exact destinations. Nested payload bindings initialize
independent duties. One bounded append-only trail records initialization,
construction, match bindings/origins, transfer and validated exit evidence across
branch snapshots. It is retained independently in opaque approval. No MIR or
runtime ownership checker is introduced.

Ordered variant plans retain shared canonical record/nested Result references.
Resource-bearingness and cleanup-effect unions are separate properties. Nested
inert Results retain plan identity without release work. Both arm bodies are
statically checked; only the selected binding/body runs. MIR explicitly represents
tag dispatch, selected payload transfer, lexical arm scopes and source origins.
Exact deterministic HIR/MIR/origin correspondence and independently retained
outcome/plan/effect authority reject artifact substitutions before execution.

Flat machine cells own actual Result payloads, including cleanup-inert values.
Non-cloneable handles, parent links and validated constructor-site correspondence
protect payloads through evaluation, publication, scrutinee retirement, dispatch,
selected-binding reservation, transfer, arm entry, calls and protected returns.
Combined prospective growth precedes detach. A failed callee cleanup retains the
outgoing Error Result and prevents caller delivery. No stop becomes Error and no
resume, unwind, rollback or retry is manufactured. Recurring arm bindings reuse
only inert storage from completed lexical activations; live resource or Result
owners cannot be overwritten.

Whole-wrapper cleanup is limited to the independently sealed primitive/protocol
domain and traverses only the actual active payload. Ordinary source must exhaustively
match or transfer its outcome. The public output representation is a bounded flat
active chain and one Bool/range/inert-record payload. Resource-bearing public
Result entries reject before execution or acquisition. Result-only execution does
not activate the resource catalog. No production resource API is exposed.

## Evidence domains and malformed controls

`ked-031-evidence.json` lists every addition, one conservative evidence-domain
classification per identity, all preserved-ID fixture adaptations, and zero
contract corrections or identity migrations. Mixed failure injection/artifact
mutation tests are classified as primitive/protocol even when they also contain
ordinary controls. A test using private canonical resource registration does not
establish a public source resource API. Inherited private-domain classifications
remain recorded in the KED-030 manifest.

- Ordinary source: 26 identities in `compiler/tests/results.rs`, covering mixed
  failure aliases and arm orders, contextual and exact payload types, namespace
  and capitalization, KD-025/KD-037 exclusions, scope and structured exits, nested
  duties, conditional/short-circuit loss, propagation, effects, public records,
  recurring arm activation and bounded public output.
- Private canonical resources: five identities covering success/inert failure,
  inert success/resource failure, equal resource types, nesting, effects on both
  alternatives, replacement generations, payload construction traps and public
  entry exclusion.
- Primitive/protocol and adversarial artifacts: 15 identities cover actual stopped
  holders, every fuel/storage/allocation cut, cleanup failure and protected returns,
  separate whole-wrapper authority, tables, witnesses, HIR/MIR, origins, plans,
  effects, admission and rendering. Distinct resource alternatives union filesystem
  and clock cleanup contracts independently of the active runtime alternative.

Mutation matrices reject invalid/duplicate/foreign/cyclic/over-depth Result IDs,
wrong nominal payloads, missing/duplicate/reordered/wrong-target arms, foreign or
duplicate bindings, wrong function/type/origins, missing/fabricated/duplicate
transfers, erased/reordered witnesses, foreign/duplicated/swapped plan roles,
shrunk effect unions, forged domain authority, incorrect scope parent/init/exit
metadata, and protected-return metadata changes. Runtime probes independently
reject unknown/Err-derived tags, same-payload-type tag flips, missing/wrong payload,
foreign origins/parents and consumed cells. Single active-payload storage excludes
a two-inline-payload representation structurally; duplicated detach and binding
artifacts reject. Nested outcome omission and conditional laundering reject in
the source ownership phase. Existing replacement-generation and approved-abort
mutants remain mandatory predecessors, rather than becoming new source authority.

## Development corrections and fixture adaptations

Ordinary compiler errors, test failures and formatting diagnostics were diagnosed
locally. Corrections included the all-empty record cleanup-summary sentinel,
public inert-record staging in Result regions, recurring inert binding reuse,
pre-execution exclusion of private resource Result returns, and combined outcome
plus cleanup admission reservations. The existing resource terminal diagnostic was
preserved. None required a semantic deviation or architectural reinterpretation.

Source fixtures using result or ok now use unreserved names; the shared value-flow
example uses returned. Corresponding source-name/span expectations were updated.
Exhaustive predecessor enum visitors explicitly reject unexpected Result forms;
manual AST/private seal scaffolding supplies the new empty metadata. Assertions,
identities, proof counts and all predecessor outcomes remain required. No valid
test was removed, ignored, filtered, weakened or contract-corrected.

## Environment and limits

The existing checksum-verified Rust 1.80.0 installation is unchanged:
`/tmp/rust-clean-vywtlaad/toolchain/bin/` contains rustc/rustdoc 1.80.0
(051478957 2024-07-21), cargo 1.80.0 (376290515 2024-07-16), rustfmt
1.7.0-stable and Clippy 0.1.80 (0514789 2024-07-21). All seven compiler/component
executables were compared with their previously verified size, mode and SHA-256
snapshot before the final pass. No toolchain installation or repair was performed.
One target, `/tmp/klyxr-ked031-target-lrwxlvcf`, serves all development/final checks.
No Cargo target cleaning, artifact copying or target substitution was used.

The accepted `/usr/bin/ps` failure `fatal library error, lookup self` is an ancillary
process-namespace/procfs visibility limitation, not failed Klyxr validation. Greg
and Miles authorized resumption. No ps/procps retry, process-table discovery or
repair is used; command runners manage known child sessions. No Rust integrity,
mode, truncation, loader or crash anomaly was observed during this implementation.
This does not claim that every utility in the host environment is healthy.

Outcome/type/proof storage is bounded before analysis. Result programs additionally
reserve quadratic witness storage and snapshot capacity; nonempty cleanup evidence
combines its storage/work reservation with that Result envelope. Every runtime
cell, continuation, temporary and payload capacity is prospectively charged. Flat
nested type/custody/output metadata avoids recursively owned Result chains.
Consumed runtime cells remain charged metadata: a sufficiently long loop may hit
the configured storage limit. Host abort, process termination and external memory
corruption remain outside structured reporting. These are tool limits, not proof
of termination or ordinary program correctness.

## Validation and publication disposition

Final local commands, profile counts, ancillary results and exact remote workflow
identities are recorded below after execution. Focused development runs are not
substitutes for the complete final profiles. Source inspection/editing and local
execution are distinct from GitHub-executed evidence. No authored test is counted
as passing until it executes.

Implementation completion is separate from independent review, Greg's acceptance,
issue closure and merge. No merge or direct prod write is authorized or performed.

## Completed final local validation

All commands below ran locally with the verified Rust 1.80.0 tools and the single
development target. Exit 1 is expected only for the two negative CLI controls.

| Command | Exit | Seconds |
| --- | ---: | ---: |
| `rustc -vV` | 0 | 0.01 |
| `cargo -vV` | 0 | 0.014 |
| `rustdoc -V` | 0 | 0.009 |
| `rustfmt --version` | 0 | 0.008 |
| `clippy-driver --version` | 0 | 0.01 |
| `cargo check --locked --workspace --all-targets` | 0 | 1.08 |
| `cargo fmt --all -- --check` | 0 | 0.445 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | 4.722 |
| `cargo test --locked --workspace` | 0 | 5.311 |
| `python3 scripts/verify-ked031-tests.py /tmp/ked031-final-gate/debug.log` | 0 | 0.036 |
| `cargo test --locked --workspace --release` | 0 | 20.848 |
| `python3 scripts/verify-ked031-tests.py /tmp/ked031-final-gate/release.log` | 0 | 0.03 |
| `cargo doc --locked --workspace --no-deps` | 0 | 1.115 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function demo` | 0 | 0.009 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function success_demo` | 0 | 0.008 |
| `cargo run --locked --quiet --bin klyxr -- check examples/result_unhandled_fail.klx` | 1 | 0.006 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_ok.klx` | 0 | 0.007 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_fail.klx` | 1 | 0.007 |
| `python3 scripts/verify-readme.py` | 0 | 0.023 |
| `python3 scripts/verify-repo-scaffold.py` | 0 | 0.024 |
| `python3 scripts/build-ai-context.py` | 0 | 0.027 |
| `git diff --check` | 0 | 0.023 |
| `git diff --cached --exit-code` | 0 | 0.003 |
| `git status --short` | 0 | 0.003 |

`cargo doc` used `RUSTDOCFLAGS="-D warnings"`. Rust executable paths resolve
under `/tmp/rust-clean-vywtlaad/toolchain/bin`; RUSTC and RUSTDOC explicitly select
those exact files. Toolchain integrity matched all seven verified executable
snapshots. No debug/release executable failed to load or crashed.

Each complete profile: **1,790 predecessors + 46 additions = 1,836 passing
identities**, zero failed/ignored/filtered; zero identity migrations. The compile-fail
authority doctest passed in each profile. The preserved affine identity executed
its 1,800-case cross-check in each profile.

| Target | Debug | Release |
| --- | ---: | ---: |
| `lib.rs` | 539 | 539 |
| `main.rs` | 0 | 0 |
| `tests/borrowing.rs` | 32 | 32 |
| `tests/cli.rs` | 27 | 27 |
| `tests/conditional_values.rs` | 55 | 55 |
| `tests/conditionals.rs` | 31 | 31 |
| `tests/contextual_literals.rs` | 21 | 21 |
| `tests/copy_field_assignment.rs` | 46 | 46 |
| `tests/deref_mutation.rs` | 32 | 32 |
| `tests/effects.rs` | 10 | 10 |
| `tests/execution_cli.rs` | 6 | 6 |
| `tests/expressions.rs` | 13 | 13 |
| `tests/iteration_local_borrowing.rs` | 74 | 74 |
| `tests/iteration_local_moves.rs` | 55 | 55 |
| `tests/local_reassignment.rs` | 45 | 45 |
| `tests/multiple_field_records.rs` | 35 | 35 |
| `tests/no_value_functions.rs` | 121 | 121 |
| `tests/ordinary_records.rs` | 97 | 97 |
| `tests/owned_replacement.rs` | 12 | 12 |
| `tests/ownership.rs` | 17 | 17 |
| `tests/path_sensitive_loans.rs` | 30 | 30 |
| `tests/reference_field_assignment.rs` | 48 | 48 |
| `tests/reference_field_reads.rs` | 59 | 59 |
| `tests/regressions.rs` | 15 | 15 |
| `tests/resolution.rs` | 7 | 7 |
| `tests/results.rs` | 26 | 26 |
| `tests/stable_loop_references.rs` | 56 | 56 |
| `tests/structured_break.rs` | 85 | 85 |
| `tests/structured_continue.rs` | 67 | 67 |
| `tests/structured_return.rs` | 95 | 95 |
| `tests/value_flow.rs` | 18 | 18 |
| `tests/while_loops.rs` | 61 | 61 |
| `doctest:klyxr_compiler` | 1 | 1 |

Result failure example: `Error(ErrorCode(1))`; success example: `Reading(42)`.
Unhandled outcome rejects with its Ownership diagnostic. Battery positive control
proves one function and checks one call; negative control preserves all four
precondition/value diagnostic fragments. The complete CLI integration target also
ran in both profiles. README assets and scaffold checks passed; generated context
was byte-identical after regeneration. Final pre-commit diff integrity passed;
the index was unchanged and all worktree changes were scoped implementation work.

The final gate runner initially used an incorrect success-output substring for
the positive battery control; that command itself exited 0. The assertion was
corrected to the actual proven-function/call fields, and the complete gate above
passed after the final scope-mutation tests were included. This was evidence-harness
work, not a compiler, source-semantic or platform failure.

Git HTTPS remote inspection failed before publication because its configured local
proxy endpoint refused connection. No proxy repair or repeat toolchain gate was
attempted. The connected GitHub API independently confirmed prod at the exact
baseline and the implementation branch absent. Publication uses Git data objects,
checks every uploaded blob SHA and the complete tree against the locally staged
validated content, and then creates the dedicated branch. This transport limitation
is distinct from local Rust validation and from the accepted ps incident.

## K31-MR01 focused merge-readiness repair

Greg authorized this repair after Sarah's review of PR #73. Before editing,
the connected GitHub API confirmed the existing remote branch
`ked-031-typed-recoverable-failure` at reviewed commit
`a48b48fe3f65e32d88bd0628f13849f3289d0e9d`, tree
`945fa7434bdc96691d20a855232dbcead998f39a`; the local checkout matched and was clean.
The repair continues that history. Frozen Draft 4 and prod remain unchanged.

Result syntax-node, resolver scratch and canonical-table allocations now return
shared typed ResourceFailure values. Bounded resolution records those values in
the existing preparation context, as bounded parsing already can. The public
preparation boundary prefers that typed failure over its frontend diagnostic
carrier; it performs no diagnostic-text classification and publishes no
executable. Canonical interning distinguishes genuine Source errors from Resource
errors. Configured syntax depth/nodes and canonical count/nesting failures retain
Limit; detected allocation failures retain Host. The existing CLI Host reporter
retains exit class 2. Legacy frontend diagnostic API shapes remain unchanged.
No allocation retry, panic interception, system-OOM experiment, competing
preparation authority, ownership/effects/cleanup/MIR/runtime redesign or accepted
syntax expansion is introduced.

Ten new `execution::result_admission_tests` identities are recorded explicitly in
`ked-031-evidence.json`. Each is sealed primitive/protocol evidence, with private
AST/canonical controls where an earlier source limit otherwise intervenes. The
three allocation-boundary tests use public preparation and assert Host with no
executable; a fourth checks later failures after successful reservation work.
The CLI control passes each actual preparation failure to its production reporter.
Other controls cover exact/excessive count, syntax depth, canonical depth and node
budgets, plus unchanged malformed-source/nominal diagnostic classifications.
At canonical count 4,096 an existing pair still deduplicates; the next new pair
is Limit. Canonical depth 32 succeeds and 33 is Limit. Existing configured source
traversal 32 includes the owned leaf (31 wrappers succeed, 32 fail). The binary
syntax-tree node boundary is 4,095/4,097 against ceiling 4,096. A late canonical
depth failure follows 32 successful interning steps and retains Limit.

All 1,790 predecessor identities and the 46 reviewed additions remain mandatory,
with no weakened assertions, identity migrations or new fixture adaptations.
The reconciler separately checks their union with the ten repair additions:
1,846 passing identities are required per complete profile. The evidence manifest
retains the reviewed exact head/tree and explicitly lists both addition sets.
Focused development validation passed all ten new tests and warnings-denied
Clippy. Two new origin assertions were corrected to the resolver's existing
whole-function origin before that pass; no predecessor assertion changed.

The previously accepted `/usr/bin/ps` `fatal library error, lookup self` incident
remains an ancillary PID namespace/procfs visibility limitation. No process-table
discovery or repair was performed during this repair. The accepted toolchain is
used unchanged. Independent confirmation of K31-MR01 remains Sarah's task against
the repaired exact head; this ledger records implementation evidence only.

### Completed MR01 local conformance gate

The unchanged accepted Rust 1.80.0 installation under
`/tmp/rust-clean-vywtlaad/toolchain/bin` and the established target
`/tmp/klyxr-ked031-target-lrwxlvcf` ran the following complete pass. All seven
verified tool executable sizes, modes and SHA-256 hashes matched their accepted
snapshots. Negative CLI controls intentionally return 1.

| Command | Exit |
| --- | ---: |
| `rustc -vV` | 0 |
| `cargo -vV` | 0 |
| `rustdoc -V` | 0 |
| `rustfmt --version` | 0 |
| `clippy-driver --version` | 0 |
| `cargo check --locked --workspace --all-targets` | 0 |
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --locked --workspace` | 0 |
| `python3 scripts/verify-ked031-tests.py /tmp/klyxr-mr01-final-gate/debug.log` | 0 |
| `cargo test --locked --workspace --release` | 0 |
| `python3 scripts/verify-ked031-tests.py /tmp/klyxr-mr01-final-gate/release.log` | 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function success_demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- check examples/result_unhandled_fail.klx` | 1 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_ok.klx` | 0 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_fail.klx` | 1 |
| `python3 scripts/verify-readme.py` | 0 |
| `python3 scripts/verify-repo-scaffold.py` | 0 |
| `python3 scripts/build-ai-context.py` | 0 |
| `git diff --check` | 0 |
| `git diff --cached --exit-code` | 0 |
| `git status --short` | 0 |

Both complete profiles passed **1,846 identities = 1,790 exact predecessors +
46 reviewed additions + 10 MR01 additions**. The library target is now 549 per
profile; every other target retains the totals in the original table above.
The reconciler checks every identity and result, including all reviewed additions,
with zero failures, ignored/filtered tests or migrations. Both 1,800-case affine
checks and both authority compile-fail doctests passed. Warnings-denied Rustdoc
and Clippy passed. Result examples produced `Error(ErrorCode(1))` and
`Reading(42)`; unresolved outcome and battery rejection diagnostics retained their
expected exit 1 and exact diagnostic controls. Documentation assets/scaffold
passed and generated context was byte-identical. No compiler/test/artifact
integrity or execution anomaly occurred. Pre-staging diff integrity passed,
the index was unchanged and all current paths belong to this focused repair.

The complete baseline-to-repaired content is inspected as the accepted reviewed
implementation plus the focused MR01 delta; unchanged files and predecessor
manifest retain their reviewed blob identities. Publication will fast-forward
the same branch using GitHub Git data objects, with local/remote blob and whole
tree identity comparison, and leave PR #73 open. The final engineering reports
on PR #73 and Issue #72 record the resulting exact commit/tree and all eight
push/PR workflow URLs; Sarah's independent focused confirmation remains pending.

Additional complete-example controls all exited 0:

| Command | Exit |
| --- | ---: |
| `cargo run --locked --quiet --bin klyxr -- check examples/battery_ok.klx` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/ordinary_run.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/owned_replacement.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- check examples/explicit_effects.klx` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/explicit_effects.klx --function demo` | 0 |

The compact `verified fn consume` and explicit `verified function consume`
example-presence checks passed. Final tool integrity comparison again matched all
seven accepted snapshots. Documentation checks are repeated after this evidence
appendix; Rust implementation content has not changed since the full gate.
