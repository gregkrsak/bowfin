# BED-032 implementation ledger

## Authority and exact baseline

Greg authorized implementation of frozen Issue #75 Draft 2 after the freeze record
6256067597, Sarah's confirmation 6255571162 and Arnold's pressure review 6255779241.
The specification, K32-S01 repair and all non-goals remain controlling. Independent
implementation review and Greg's acceptance remain pending; no merge is authorized.

Remote prod and its signed Git commit object were retrieved through the connected
GitHub API. The exact object hash was verified locally and origin/prod was fetched
from those Git data objects without changing prod. The unavailable historical Git
HTTPS proxy was not repaired or used as a reason for GitHub-only development.

- Baseline commit: `74c1bedc8f847ffaeb38ed4795ee91730e35b949`
- Baseline tree: `2d0cc78b67a8cb69ea8feaff183dbf1b91938e63`
- Implementation branch: `ked-032-explicit-result-propagation`
- PR target: `prod`

The current production Arnold charter was retrieved; exact-baseline AGENTS.md and
the canonical AI documents were read in the prescribed order during the preceding
pressure review and their controlling sections refreshed before implementation.
The checkout, index and source tree were clean and exact before branch creation.
No branch was recreated and no accepted predecessor content was discarded.

## Accepted environment and startup gate

One unchanged verified Rust 1.80.0 installation is used:
`/tmp/rust-clean-vywtlaad/toolchain/bin`. RUSTC and RUSTDOC explicitly select its
files. Rustc/Rustdoc are 1.80.0 (051478957 2024-07-21); Cargo is 1.80.0
(376290515 2024-07-16); Rustfmt is 1.7.0-stable and Clippy is 0.1.80.
All seven executable sizes, modes and SHA-256 hashes matched the accepted snapshots.
LLVM, rustc_driver and libstd critical library snapshots also matched exactly.
No reinstall, extraction, permission repair, target churn or process-table discovery.
The accepted ps/procfs/PID-namespace incident remains an ancillary limitation.

One new empty task target was created before startup validation:
`/tmp/klyxr-ked032-target-tw_ru4v0`. It is retained throughout this implementation.

The complete startup pass succeeded on the untouched baseline:
**1,846 exact identities in debug and 1,846 in release**, including all 1,790 original
predecessors, 46 reviewed KED-031 additions and ten MR01 identities. Zero failures,
ignored/filtered tests or migrations. Authority doctest and the 1,800-case affine
cross-check ran successfully in each profile.

| Startup command | Exit |
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
| `python3 scripts/verify-ked031-tests.py /tmp/klyxr-ked032-baseline-gate/debug.log` | 0 |
| `cargo test --locked --workspace --release` | 0 |
| `python3 scripts/verify-ked031-tests.py /tmp/klyxr-ked032-baseline-gate/release.log` | 0 |
| `cargo doc --locked --workspace --no-deps` | 0 |
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

Rustdoc used `RUSTDOCFLAGS="-D warnings"`. CLI negative controls returned the
expected 1 with preserved diagnostics. Context regeneration was byte-identical;
diff, worktree and index checks were clean. No compiler/executable anomaly occurred.

## Representation and authority

AST and resolved Let carry optional phrase/binding spans. Typed Let carries a
canonical propagation mode with exact inner and outer ResultTypeIds, shared
failure ValueType and distinct statement/phrase/binding origins. Its destination
has the inner success type. Independent initializer synthesis supplies no missing
constructor context. Exact nominal mismatch remains Type, not tool failure.

The existing ownership checker records an explicit propagation transfer and an
immutable mode witness. It evaluates the initializer once with ordinary permissions,
validates a failure-state snapshot using an empty bypassed suffix and unchanged
recurrence/holds, rejects earlier unresolved outcomes, and emits existing return
cleanup/effect sites. Only success initializes the original local and any nested
outcome obligation. No synthetic match-arm scope or branch-local reduction occurs.

MIR adds Propagate and ForwardError terminators and reuses selected BindResult.
The success block remains in the active lexical scope, retaining the original
LocalId and its canonical declaring ScopeId. The Error block lists exact ordinary
function-return exits. The original suffix is emitted once. Scope identities are
derived while structured HIR is available, including inert replacements, without
activating resource machinery for cleanup-inert propagation.

Generated Error cells bind type/tag/origin to an approved ForwardError block;
ordinary constructed cells retain temporary-site provenance. A tagged bounded
scalar preserves the predecessor cell footprint. Every encoded index is checked
before publication. Error cells reserve combined cell/value capacity while F
still belongs to protected input, then transfer payload, parent links and wrapper
state without allocation. Existing protected return and cleanup code delivers only
after completed cleanup. Both equal and differing complete types use canonical
rewrapping, avoiding a separate optimized reuse protocol.

Typed MIR validation now returns PlanError internally for traversal/cleanup
reservation as well as malformed structure. Public legacy validate only adapts
its diagnostic shape. Opaque execution approval uses variants, not diagnostic
text, through lowering, validation, sealing and revalidation. Original immutable
HIR, outcome/effect witnesses, domains and exact deterministic HIR/MIR/origin
correspondence remain the only executable approval boundary.

## Scope, cleanup, effects and custody evidence

Private canonical resources prove direct A → C → B and unchanged explicit Match
A → B → C. The direct stopped prefix is Complete(A), Attempt(C), Failure(C).
B remains uniquely held without Attempt/Complete; the outgoing result stays
protected and sample's frame does not retire. A further control replaces the
propagation success local from a real child branch and retains its original
owning activation. No cleanup scheduler was added.

Four payload classes on both alternatives are covered: Copy, inert Move,
resource-bearing and nested Result. Nested parent links and outcome obligations
remain exact; different outer success types rewrap F without conversion. Guards
clean before delivery of propagated resource-bearing Error; a failed guard release
prevents delivery and leaves its failure payload retained. Returned F has latent
caller responsibility and contributes no callee release. Cleanup effects from
implicit Error return are admitted by the existing custody product; complete callee
contracts remain caller-visible. Resource-free propagation has no resource state.

Separate sealed primitive evidence injects a sibling temporary in the approved
Match-destination region. Sibling cleanup stops before tag selection while the
complete input is protected. This is not mislabeled ordinary-source reachability.
No production source can select fault injection, resource registration or a domain.

Mutation evidence independently changes inner/outer/F identities, destinations,
scopes, dispatch alternatives, omitted/duplicate transfers, origins, exits,
protected-return metadata, a synthetic owning arm and replacement scope. HIR,
MIR and executing-cell tag/origin/parent corruptions and foreign approval all reject.
Malformed runtime probes remain primitive evidence, not valid source execution.

## Bounded protocol

Immutable new fields are fixed-size and included in existing conservative checked
frontend/outcome storage/work envelopes. New CFG growth is two blocks per site:
straight-line p sites have exactly 2*p+1 blocks; the suffix is not duplicated.
Existing token-derived block/site caps cover this mapping. No source limit is
raised and no cleanup plan is expanded by path. Witness count and snapshot storage
remain bounded; shared work counters are not refunded by checker rollback.

Actual initializer/temporary/call charges remain. Propagate dispatch and the
selected binding/transfer/entry or Error return/publication/transfer are charged;
only the selected alternative executes. Cell/history capacities remain charged
until their established retirement. Combined receiving growth is checked before
active detachment, with each physical payload counted once. Histories/handles
never own duplicates. All terminal stops refuse resume and add no unwind/retry.

Exact successful thresholds on x86_64 Rust 1.80.0 are:

| Scenario | Fuel | Working-storage bytes |
| --- | ---: | ---: |
| Copy success | 53 | 3904 |
| inert Move success | 59 | 4344 |
| resource success | 65 | 4392 |
| nested Result success | 88 | 5232 |
| Copy failure | 58 | 4080 |
| inert Move failure | 64 | 4544 |
| resource failure | 72 | 4624 |
| nested Result failure | 95 | 5504 |

Tests assert all eight implementation fuel and storage totals and every smaller fuel
cut. Every byte through the exact sufficient storage threshold is tested for
resource/nested success and failure; immediately insufficient neighbors stop with
retained holders. Actual growth-boundary Host injection covers all four payload
classes on both paths; exact sufficient/insufficient admission storage is separately
established for repeated propagation. There is no real system-OOM experiment.

## Identity preservation and development corrections

The KED-032 predecessor inventory contains exactly 1,846 normalized identities and
outcomes from the startup gate. The new reconciler separately checks preservation
of all KED-031 original/reviewed/MR01 sets and every KED-032 addition. No identity
migration, deleted test or predecessor contract correction is introduced.

Six mechanical predecessor fixture adaptations are explicit in the evidence
manifest: three HIR Let constructions set propagation=None, two MIR inspection
patterns ignore the new field, and the unchanged conditional-value path oracle
explicitly rejects the new terminators. Their assertions/outcomes are preserved.

New tests were corrected during development for existing Bool rendering syntax,
constructor literal exclusion (use an independently range-typed helper), the
original whole-statement transfer-origin boundary, and a fixture string replacement
that also matched a declaration. These were implementation/test construction
corrections, not changes to frozen semantics or predecessor expectations.

The final gate's exact commands, outcomes, reconciled totals and source audit are
recorded below after completion. Final commit/tree and eight CI URLs are published
in the linked PR/Issue #75 engineering reports once all local checks pass. No
independent confirmation, acceptance or merge is claimed by this ledger.


## Final local conformance gate

The complete final pass used the same accepted toolchain and task target. Debug
and release each passed **1,875 exact identities: 1,846 predecessors + 29 additions**.
Each profile had zero failed, ignored or filtered identities. This includes the
compile-fail authority doctest and its 1,800-case affine cross-check independently
in debug and release. The addition domains are nine ordinary-source identities,
three private canonical-resource identities and 17 primitive/protocol identities;
these domains do not claim production resource APIs.

The resource scope identity also covers replacement from an inner branch, an
actual match arm, loop normal backedge, continue, break and nested-loop return.
These use the existing lexical cleanup activations and preserve conservation.

| Final command | Exit |
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
| `python3 scripts/verify-ked032-tests.py /tmp/klyxr-ked032-final-gate/debug.log` | 0 |
| `cargo test --locked --workspace --release` | 0 |
| `python3 scripts/verify-ked032-tests.py /tmp/klyxr-ked032-final-gate/release.log` | 0 |
| `'RUSTDOCFLAGS=-D warnings' cargo doc --locked --workspace --no-deps` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_handling.klx --function success_demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- check examples/result_unhandled_fail.klx` | 1 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_ok.klx` | 0 |
| `cargo run --locked --quiet --bin klyxr -- verify examples/battery_fail.klx` | 1 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_propagation.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_propagation.klx --function failure_demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/result_propagation_mismatch.klx --function demo` | 1 |
| `cargo run --locked --quiet --bin klyxr -- run examples/ordinary_run.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/owned_replacement.klx --function demo` | 0 |
| `cargo run --locked --quiet --bin klyxr -- run examples/explicit_effects.klx --function demo` | 0 |
| `python3 scripts/verify-readme.py` | 0 |
| `python3 scripts/verify-repo-scaffold.py` | 0 |
| `python3 scripts/build-ai-context.py` | 0 |
| `git diff --check` | 0 |
| `git diff --cached --exit-code` | 0 |

Exit 1 is expected only for the three negative controls (unhandled Result,
battery precondition and nominal propagation mismatch). Positive propagation
prints Ok(Reading(42)) and Error(ReadError(1)). CLI frontend mismatches remain
class 1 and detected Host preparation failures remain class 2. The established
CLI renders the primary mismatch message; canonical identities/names remain in
the frontend diagnostic detail. A development gate assertion was aligned with
that existing rendering contract; no CLI diagnostic was weakened.

All seven tool executables again matched their accepted size/mode/SHA-256
snapshots. Generated context was byte-identical after regeneration. The index
remained the exact baseline during validation, and every modified/untracked path
belongs to this implementation. No process table inspection was attempted.

## Final source audit and limits

The complete baseline-to-working-tree diff was inspected, including new tests,
the predecessor inventory, evidence manifest, reconciler, examples and generated
context. The source feature is limited to the ordinary Let tail. Constructor
inference, explicit-match behavior, ownership authority, cleanup/replacement
ordering, effects and public resource boundaries remain the predecessor contracts.
The failure path uses ordinary return machinery; the success suffix appears once.
Actual ScopeIds, generated return provenance, independent type/origin checks and
deterministic approval correspondence reject mutated alternatives and authority.

No predecessor identity was migrated or removed. Six mechanical fixture changes
are fully enumerated above and in the manifest. Numeric protocol thresholds apply
to this x86_64 Rust 1.80.0 implementation; they are evidence, not new language
semantics. Private synthetic abort/process-termination experiments are not added:
propagation consumes only an actually produced Result and adds no interception,
handler or resume path. Existing terminal-failure predecessor evidence remains
reconciled; new cuts cover Host, configured Limit, Integrity, trap and cleanup stop.

Local validation and CI are separate evidence lanes. Git data publication uses
verified local blob/tree identities through the connected GitHub API because the
historical shell proxy is unavailable. The complete local gate above is not
replaced by remote CI. Final commit/tree, clean post-commit index/worktree and all
eight workflow URLs are recorded in the PR and Issue #75 engineering reports.
Independent review, Greg's acceptance and human merge remain pending.


A final evidence-only follow-up adds an explicit `try` rejection case and hard
assertions for the eight already-recorded working-storage totals. The complete
30-command local gate was rerun successfully after those assertions: unchanged
1,875 debug and 1,875 release identities, exact predecessor reconciliation and
all ancillary checks. The initial implementation commit's four push workflows
also passed; the engineering report identifies eight runs at the final reviewed
head separately. No implementation behavior or predecessor expectation changed.
