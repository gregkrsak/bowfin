# BED-028 implementation ledger

Issue: https://github.com/gregkrsak/klyxr/issues/64
Frozen specification: Draft 1; freeze comment 6195981810.
Accepted pressure review: comment 6195811443.
Baseline: `01f80fa636e8a06bd5b4a145307a7f0cd3626d04`.
Baseline tree: `55d2a06fde7cdf990bbb4f364baf82b49ecf8c06`.
Branch: `ked-028-deterministic-move-replacement`; PR target: `prod`.
Arnold charter retrieved at governance revision
`2887c9239fb6f82ded4fd7b2d8f5f592b725cccc`, blob
`b04c4ba5251fdcacb5278ba26c0aaf8a06b4db7e`.

Greg authorized GitHub Actions as the authoritative compilation and executable
validation environment. Local Rust validation is unavailable because of confirmed
Work-runtime extraction corruption. No local Rust execution, installation,
extraction, integrity probing or remediation is performed during this assignment.
Source edits, Python text processing and Git operations are local. Shell push
lacks credentials; connected GitHub Git-data operations publish reviewed commits.

## Unchanged baseline gate

Fetched origin/prod and verified exact commit/tree. Index and worktree were clean.
No authorized remote branch existed; created it unchanged at the exact baseline.
All four push workflows passed independently:

| Workflow | Run |
| --- | --- |
| build | https://github.com/gregkrsak/klyxr/actions/runs/38787690440 |
| tests | https://github.com/gregkrsak/klyxr/actions/runs/38787690500 |
| docs | https://github.com/gregkrsak/klyxr/actions/runs/38787690573 |
| verify | https://github.com/gregkrsak/klyxr/actions/runs/38787690531 |

The complete tests job (116603833362) supplied both debug and release identity
inventories: exactly 1,667 each, identical identities and outcomes, all `ok`.
`ked-028-predecessors.json` preserves this exact inventory, including the authority
compile-fail doctest. Both profiles execute the 1,800-case affine cross-check.
The original KED-027 inventory and verifier are preserved as historical evidence.

## Milestone 1 — prepared checker replacement

Canonical assignment validation checks mutable owned target identity and exact
RHS type. The existing single ownership checker snapshots prepared entry, validates
target availability and relative loop eligibility, prevents target consumption
through its canonical consume path, evaluates the complete RHS, expires ordinary
loans and checks final whole-owner exclusivity. Its scoped restriction creates
no synthetic loan and participates in complete rollback. Copy assignment retains
its existing path. Runtime replacement authority and generations are later layers;
this milestone does not claim complete KED-028 execution.

### Predecessor corrections

The four designated rejection-to-acceptance migrations are listed exactly in
`ked-028-evidence.json`; assertions now establish acceptance and RHS transfer.
The pressure review's `boundary_tests` namespace is corrected to the observed
`bounds_and_artifact_tests` identity without changing its witness.

One additional obsolete rejection was found in the exact inventory:
`tests/iteration_local_moves.rs::iteration_local_replacement`. It initializes a
mutable owner inside the current loop iteration and replaces it with `fresh()`.
Frozen §12 expressly accepts this form. The test keeps its identity and source,
but now asserts acceptance. This is recorded separately as an outcome correction,
not hidden among the four designated renames. No frozen semantic change is made.

Header-target, header-source, break and return witnesses remain Ownership
rejections, with diagnostics updated to the narrower controlling loop boundary.
The CLI non-Copy dereference fixture remains unchanged.

New focused checker tests cover final and future shared/exclusive uses,
self-consumption at nested RHS depth, moved/conditional targets, exact nominal
and reference boundaries, branches, iteration-local exits and nested-loop targets.
New identities require explicit manifest membership; counts alone cannot pass CI.

Milestone validation is pending. No PR, acceptance, issue closure or merge.

### First milestone CI and repair

Commit `bf3b3686ad66fcba5174e54642bf24681765184f`, tree
`feb1857c91492db115ea4f9e80e8cca3fa91469a`:
build https://github.com/gregkrsak/klyxr/actions/runs/38788406841 (formatting failed;
workspace/all-target compilation passed), tests
https://github.com/gregkrsak/klyxr/actions/runs/38788407172 (one new fixture failed),
docs https://github.com/gregkrsak/klyxr/actions/runs/38788406871 (passed), verify
https://github.com/gregkrsak/klyxr/actions/runs/38788406935 (passed).

Applied the GitHub Rustfmt diffs as text, without local Rust execution.
`later_exclusive_handle_blocks_commit` originally passed the exclusive handle by
value and then used it again; KD-021 correctly rejected the moved handle, rather
than retaining it as a live loan at replacement commit. Its intended live-loan
witness now reads a Copy field non-consumingly on the RHS and uses the handle
later. The original fixture remains as a separate moved-handle rejection test.
No accepted ownership contract was relaxed. No later architectural layer added.

Milestone 1 repair commit: `4da4fb30801c75a7d2549222f87a96d37ca1c389`, tree
`0d794d917c649b4416cd725c0a7efe7dbce3df64`. All four workflows passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38788670378),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38788670419),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38788674432),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38788670442).
Both debug and release: 1,680 passing identities, comprising the exact 1,667
predecessors with explicit corrections and thirteen additions; no failed,
ignored or filtered tests. Both affine checks and authority doctests passed.

## Milestone 2 — explicit MIR replacement authority

Compound ReplaceOwned operations retain canonical local/record identity,
declaring lexical scope, fixed canonical cleanup-plan reference (or inert), full
RHS, replacement origin and span. Structured lowering binds an outer target to
its owning scope, even inside a branch. Exact opaque correspondence prevents
missing, duplicated, reordered, wrong-target/type/scope/plan/origin artifacts from
executing. Existing admission bounds scope and owner metadata. Only the complete
replacement RHS result gains a resource lifetime boundary; argument temporaries
remain rejected. No second ownership engine is introduced.

The executor explicitly refuses ReplaceOwned pending the next validated runtime
layer. This intermediate milestone does not claim executable replacement support.
Generation machinery and runtime evidence follow only after its four CI gates.

Milestone 2 first commit `230e82a2c2f5dee8d7ee1c62c0546ad6422ba2f3`, tree
`572aeb3436fde0b89dc8d8a3c54d64be809a0921`: build
https://github.com/gregkrsak/klyxr/actions/runs/38789163988 and tests
https://github.com/gregkrsak/klyxr/actions/runs/38789163790 failed compilation of
an existing exhaustive MIR inspection in `tests/conditionals.rs`. The new variant
requires an explicit arm. The repair rejects ReplaceOwned in that Copy-only
fixture, preserving and strengthening its existing assertion. Docs
https://github.com/gregkrsak/klyxr/actions/runs/38789163922 and verify
https://github.com/gregkrsak/klyxr/actions/runs/38789163867 passed.
No later layer started.

MIR exhaustive-match repair: `cf2ecd9173edb0a1c9f2aac3606515cff65208e7`, tree
`96e6c53c6dd284f0a011401258670a1400875ec9`. Compilation and debug tests passed;
build https://github.com/gregkrsak/klyxr/actions/runs/38789306742 reported only
Rustfmt differences. Applied those CI-produced formatting changes as text.
Other runs: tests https://github.com/gregkrsak/klyxr/actions/runs/38789306768,
docs https://github.com/gregkrsak/klyxr/actions/runs/38789306792,
verify https://github.com/gregkrsak/klyxr/actions/runs/38789306763.

Milestone 2 formatting commit `e9a1fe23290a129693796295f8acdd736683f404`, tree
`2d5837215e0e60f4d62b18c7649c32cb07a0fc14`: all four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38789460451),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38789460403),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38789460449),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38789460499).
The complete debug/release jobs passed exact inventory reconciliation.

## Milestone 3 — generation foundation

Dynamic activation histories now identify each initialization generation, with a
separate frame/local-to-current-generation association. Histories own no payload.
Before cleanup, the live entry must match the slot association; a stale entry
cannot release the current occupant. Global checked generation sequences and
activation identities distinguish repeated dynamic instances of one LocalId.
Association/history capacity growth is reserved together and counted in storage.
Existing initialization and cleanup fuel charges remain unchanged.

New controls inspect repeated loop activations and inject a stale generation
while verifying exact resource-identity conservation. Replacement execution remains
disabled until this foundation passes all four workflows.

Generation foundation commit `8070752722794f315a17ceb7adc0fa8f52bebe1c`, tree
`3163553a4ab322b631485f1825eba48fa191e987`: build
https://github.com/gregkrsak/klyxr/actions/runs/38789765099 compiled successfully
and reported Rustfmt differences, applied from CI output. Other milestone runs:
tests https://github.com/gregkrsak/klyxr/actions/runs/38789765184,
docs https://github.com/gregkrsak/klyxr/actions/runs/38789765162,
verify https://github.com/gregkrsak/klyxr/actions/runs/38789765094.

Generation formatting commit `2089b209c1f39de04e6e6aa9432a3877d3ed266e`, tree
`01ee9ff91c8532a3873c69c3e9938b3a7e90f469`: all four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38789899836),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38789899824),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38789899834),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38789899688).
Both complete test profiles and exact inventory reconciliation passed.

## Milestone 4 — protected replacement vertical slice

A bounded stack of prepared replacement contexts is allocated before RHS work.
Old ownership stays in its slot while RHS evaluation runs. The complete new value
moves infallibly from its existing holder into separate protected storage before
fallible generation reservation or displacement. Detached old ownership is retained
explicitly, then transferred into the existing canonical cleanup cursors. Successful
cleanup discharges the old generation. Publication, slot association and reserved
history append form one allocation-free transition in the target's owning activation.
Ordinary continuation remains inaccessible on any terminal failure.

Inert public records use the same protocol without executable cleanup. Copy writes
retain their original route. Protected replacement results do not reuse return,
construction or cleanup scratch holders. All context/history capacities and both
payloads count simultaneously toward storage; progress and skipped discharged
history entries charge fuel. Resume and completion reject unfinished replacement.

First runtime evidence covers exact A/B/C acquisition and release identities,
repeated inert execution, distinct-source transfer, repeated leaf replacement,
every fuel cut with identity conservation, exact sufficient fuel and trusted
release Host failure retaining N separately. Aggregate/recursive and full boundary
batteries remain the next gate after validation.

Vertical-slice commit `137556365cff34347039c1cd154fbeed884117c3`, tree
`9ecce8663a071cda81ec327f6cc57d914c118b09`: compilation and debug tests passed,
including the identity-sensitive A/B/C event sequence and all fuel-cut conservation
controls. Build https://github.com/gregkrsak/klyxr/actions/runs/38790373840 reported
only formatting differences, applied from GitHub Rustfmt output. Other runs:
tests https://github.com/gregkrsak/klyxr/actions/runs/38790373495,
docs https://github.com/gregkrsak/klyxr/actions/runs/38790373820,
verify https://github.com/gregkrsak/klyxr/actions/runs/38790373805.

Vertical-slice formatting commit `b6db8ea45d49e60f00958ec22a676b3c824b48b5`, tree
`ebe0788c44996ecd14ec5b1e6d0f447b3363256c`: all four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38790503395),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38790503411),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38790503299),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38790503367).
Both complete debug/release profiles and identity reconciliation passed.

## Milestone 5 — aggregate, recursive and terminal conformance

Private tests exercise reversed written aggregate construction versus canonical
reverse-field cleanup, nested aggregates, recursive RHS replacement contexts,
protected returns, branch owning-activation ordering, later locals, covered
structured exits, repeated dynamic iterations, later moves and explicit release.
The existing private aggregate fixture builder now transforms assignment RHSs as
well as initializers and returns; no source field-type facility is added.

Identity-multiset conservation runs at every recursive task and every fuel cut of
aggregate, nested and recursive witnesses. A test-only prospective-storage meter
records the maximum combined reservation. Every byte budget below the measured
successful requirement must stop, and the exact sufficient budget must succeed.
Each stopped state must conserve all acquired leaf identities.

Test-only Host probes at RHS/protection/preparation/detachment/discharge boundaries
model detected reservation/progress failures. The normal executor charges the
same phase work; these probes create no production API. Per-leaf release Host
failures cover all eight nested-aggregate identities and preserve completed prefixes.
All terminal cuts refuse resume and never restore a moved RHS source. Publication
and generation registration are an indivisible allocation-free transition after
all fallible checks; there is no invented failure cut between those writes.
RHS field-mutation prefix effects survive a later range trap while the old target
remains in place and no new result is published.

Conformance commit `0257405611a711308a3cbb779234ef55c0229bc2`, tree
`af05aa105607b8dd7d8e1bc088d873d271a7691d`: compilation and the full debug profile
passed. Build https://github.com/gregkrsak/klyxr/actions/runs/38790992946 reported
formatting differences, applied from CI output. Other runs:
tests https://github.com/gregkrsak/klyxr/actions/runs/38790992862,
docs https://github.com/gregkrsak/klyxr/actions/runs/38790992731,
verify https://github.com/gregkrsak/klyxr/actions/runs/38790992870.
Two additional controls independently verify prepared-entry restoration after a
successful unrelated RHS transfer followed by final-loan rejection, and rejection
of a Copy assignment retagged as Move replacement. No later runtime layer added.

Milestone 5 completed at `c206d6d82ebad46afa8a8d4212f0d08b8e2d3506`, tree
`99edfab9e3079eb2448dac53efac31dfa282baa0`. All four passed:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38791182338),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38791182324),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38791182249),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38791182273).
Both complete profiles reconciled 1,667 predecessor identities (the four explicit
renames and separately documented outcome correction) plus 38 new identities:
1,705 passing tests in each profile, including the authority doctest and both
1,800-case affine executions. No failed, ignored or filtered tests.

## Final documentation and public CLI gate

Canonical context, language decisions, architecture and open questions describe
the frozen extension without assigning a new KD number. Historical sections remain
historical and their narrow supersession is explicit. The compiler README and
bounded-execution contract document eligibility, phase fuel, simultaneous storage,
terminal states and non-goals. The context bundle is regenerated reproducibly.
The public example produces Reading.value = Percent(59) after source transfer and
last-use RHS borrowing. A CLI regression checks that exact result and preserves
`check`/`verify`'s no-ordinary-contract-proof reporting.

Local non-Rust checks: diff whitespace validation, README asset verification,
scaffold verification and context generation passed. All compilation, formatting,
Clippy, tests, Rustdoc, doctest and CLI execution evidence is from GitHub Actions.
Final completed-branch and independent PR gates remain required before handoff.
