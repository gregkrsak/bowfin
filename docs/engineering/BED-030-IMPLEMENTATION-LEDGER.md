# BED-030 implementation ledger

## Authority and baseline

Frozen Draft 2: [Issue #70](https://github.com/gregkrsak/klyxr/issues/70).
Freeze: comment 6224699062. Separate implementation authorization: comment
6224708500. Binding implementation-pressure review: comment 6222126054.
Arnold charter blob: `b04c4ba5251fdcacb5278ba26c0aaf8a06b4db7e`.

Authorized baseline commit: `edf80c1e68e6fe3e2b25f6a557fc121395e45adc`.
Baseline tree: `844afe3deacf22bc55844bc8ea92d76036cb3fc2`.
Branch: `ked-030-explicit-transitive-effects`; target: `prod`.

The unchanged branch passed all four push workflows:

- [build](https://github.com/gregkrsak/klyxr/actions/runs/38968144093)
- [tests](https://github.com/gregkrsak/klyxr/actions/runs/38968143774)
- [docs](https://github.com/gregkrsak/klyxr/actions/runs/38968143743)
- [verify](https://github.com/gregkrsak/klyxr/actions/runs/38968144001)

Job 117199623757 reconciled 1,706 historical predecessors plus 37 KED-029
additions = 1,743 passing identities separately in debug and release. Each profile
passed the compile-fail authority doctest and 1,800-case affine cross-check;
failed/ignored/filtered were zero. Build, documentation and verifier job logs and
step conclusions were inspected. This is GitHub-executed baseline evidence.

Local Git HTTPS push had no credentials; branch publication uses the connected
GitHub app. No local Rust/toolchain execution or repair is used as evidence.

## Milestone 1 — Canonical atoms and bounded sets

The eight independent atoms use canonical enum identities and a private one-byte
set. Source-name resolution is separate from runtime operation dispatch. Union,
subset checking and canonical rendering allocate no semantic authority. Wider
encodings reject before narrowing. Membership, including `unsafe`, grants no
safety permission.

Three new unit-test identities cover all atoms, duplicate-insertion detection,
unknown names, all 65,536 pairs of valid sets, and all invalid wider encodings.
GitHub milestone commit `cbd35c657ec3ee45aacdeb98f4c3a844d67ef27d`, tree
`91a58f443813d61ef29a3fefdf853845dc43d07b`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38968477589),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38968477737),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38968477669),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38968477901).
Job 117200734355 reconciled 1,743 predecessors plus three additions = 1,746
passing identities in each profile, with both authority/affine controls passed.
All four job conclusions and relevant logs were inspected.

`ked-030-predecessors.json` preserves the complete baseline identity set.
`verify-ked030-tests.py` requires every predecessor plus exactly the explicitly
listed additions in each complete profile, retaining affine/doctest controls.
Historical KED-027 through KED-029 manifests and reconcilers are unchanged.

## Milestone 2 — Ordinary grammar and canonical declarations

Contextual clauses preserve raw member and introducer spans only on ordinary AST
functions. Resolution collects canonical contracts with all signatures before
bodies. Ordinary typed HIR and MIR retain one bounded declaration representation
with at most eight member origins; publication/structural validation checks its
internal consistency. No verified or harness metadata is added. Independent
approval correspondence includes declarations through existing exact equality.

Five integration identities cover result kinds/spellings, retained metadata and
execution of unused declarations, lexical compatibility, forward/recursive
signatures, malformed lists, and verified/harness rejection controls. One unit
identity mutates declaration mask/origin copies independently. Execution remains
pending until the milestone workflows are recorded. Direct/transitive body subset
checking is milestone 3; this milestone alone is not a completed effect system.

One advisory `cargo fmt --all` invocation could not run because Cargo is absent.
No toolchain repair/probing followed. Formatting is checked by GitHub Actions.

## Remaining milestones

Direct/transitive checking; operation contracts and
derived summaries; checker-produced cleanup custody, MIR and approval; replacement
and temporaries; private abort domains; malformed/bounds/terminal evidence;
documentation and complete conformance remain unimplemented at this milestone.

No production resource API, unsafe permission, effectful proof semantics, new
ownership checker, source recoverable failure or unwind is introduced.
Implementation completion, independent acceptance and merge are not claimed.

## Milestone 2 first push and correction

Commit `4ebdf2beeb6dc80a5ad4e1f06cb8343aacc10198`, tree
`7ad86db19b49372b28d556e6a160ef908ef6a5cc`, compiled all targets.
[Build 38969016547](https://github.com/gregkrsak/klyxr/actions/runs/38969016547)
stopped at formatting; its exact rustfmt diff was applied. Clippy and Rustdoc were
not reached on that push. [Tests 38969016526](https://github.com/gregkrsak/klyxr/actions/runs/38969016526)
passed the debug test bodies but stopped in reconciliation: the five new
integration identities were listed as `effects.rs::...` instead of Cargo's actual
`tests/effects.rs::...`. Only those new manifest entries were corrected; no test
assertion or predecessor identity changed. Release was not reached.
[Docs](https://github.com/gregkrsak/klyxr/actions/runs/38969016525) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38969016520) passed.
These are implementation formatting/evidence-manifest errors, not platform
failures. No later architectural layer was added on the failing milestone.

## Milestone 2 completed

Correction commit `3db5aba640885810a04df74824160ac71efbd1f7`, tree
`096fa362e59e8d543126a40bb96faa10cf2af3ec`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38969187579),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38969187747),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38969187827),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38969187641).
Job 117203057962 reconciled 1,743 predecessors plus nine additions = 1,752
passing identities in both profiles; both affine and authority controls passed.
All four job conclusions and relevant logs were inspected before milestone 3.
Declaration origins share an immutable fixed-size array; absent clauses allocate
none. Only ordinary function representations contain those origins.

## Milestone 3 — Direct and transitive contracts

A structural visitor collects complete canonical call declarations, without
interprocedural inference. Both ordinary call forms and all expression containers
contribute. Terminal branches, loops and both short-circuit operands remain
structurally represented. Source check/verify and both execution preparation
routes enforce subset checks after ownership approval. Private synthetic
preparation enforces the same check.

MIR retains nonempty direct call sites with canonical enclosing/callee identities,
source call ordinals, masks and origins. Resource-free empty-contract calls add no
site entries. Structural and table-aware validation plus exact deterministic
correspondence protect the metadata. No runtime effects analysis is added.

Five new identities exercise all atoms/spellings/result/call forms, three-function
propagation through unused strict supersets, structural unions, recursive bounded
contract-edge diagnostics through both preparation APIs, unsafe disclosure without
ownership permission, and nine independent direct-site mutations at approval.
These new tests are authored; execution awaits this milestone's workflows.
Cleanup contracts/realization and private abort domains are still pending.

## Milestone 3 first-push compilation correction

Commit `e228d4eb69df62dfe5654b09395c19b3002499f2`, tree
`4bc5d686b9c912544dc1a29feeee3d1eb02f093a`, failed all-target compilation in
[build](https://github.com/gregkrsak/klyxr/actions/runs/38969651618) and
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38969651702): predecessor
helpers exhaustively matched the public frontend error enum and needed its new
Effects variant. Explicit Effects classifiers were added to the eight affected
helper files (ordinary_records, copy_field_assignment, reference_field_assignment,
reference_field_reads, contextual_literals, multiple_field_records, while_loops,
local_reassignment). All original expected phases, assertions and identities stay
unchanged. No test migration, weakened expectation or platform failure is claimed.
[Docs](https://github.com/gregkrsak/klyxr/actions/runs/38969651619) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38969651788) passed.
No later milestone was started on this failing compilation gate.

Milestone 3 compilation correction `46a1df5337a082c68f377c2d43008477f79f42b8`
(tree `e0d3c2341a76fd4daff78d26546087efdcafac29`) passed all-target compilation.
[Build 38969904121](https://github.com/gregkrsak/klyxr/actions/runs/38969904121)
then stopped only at rustfmt layout. Its exact diff was applied to execution
preparation call layout and new test layout. [Tests 38969904096](https://github.com/gregkrsak/klyxr/actions/runs/38969904096)
passed debug test execution/reconciliation; release was still running when this
format-only correction was prepared. [Docs](https://github.com/gregkrsak/klyxr/actions/runs/38969904112)
and [verify](https://github.com/gregkrsak/klyxr/actions/runs/38969904118) passed.

## Milestone 3 completed

Commit `6d43af6e9028d189a4fcd4d0d8bf112118232a84`, tree
`36056a9dc25f70b7c8bfe01aad4a811a92b09540`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38970021292),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38970021243),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38970021125),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38970021253).
Job 117205821633 reconciled 1,743 predecessors plus 14 additions = 1,757 passing
identities in each profile, both affine controls and the authority doctest passed.
All job steps/conclusions and relevant logs were inspected before milestone 4.

## Milestone 4 — Registered contracts and derived summaries

Each registered operation stores its complete contract, checked for exact equality
with the bound ordinary declaration. A leaf's cached cleanup summary equals its
fixed release operation's complete contract. Acquire and release contracts remain
independent. Stub bodies are shape controls, not effect authorities.

Nonempty summaries are derived through shared canonical plans with memoization,
checked record/release identities and bounded recursion. Executable summary copies
must equal that derivation. Canonical plan validation precedes summary derivation;
it already validates fields, cycles, shared graph heights and configured depth.
An empty summary vector denotes all-empty sets only; the existing resource plans
remain present and continue controlling real cleanup. It does not classify values
as inert. All-empty predecessor and resource-free paths allocate no summary cache.

Five new identities cover all atoms, distinct-child contract unions, release strict supersets, independent
operation/function/leaf/binding mutations, aggregate unions, empty-summary plans,
and independent cached-summary mutations through actual execution approval.
These new tests await milestone GitHub execution. Ownership-produced realization
checking and private execution-domain authority remain pending.

Milestone 4 first push `3f638b0e522586eb003d41b4c4929e2bd95ca150`, tree
`42be08ec26ddfbc66424fce1ca51f6db4e33526c`, passed all-target compilation and
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38970650456): job
117207905893 reconciled 1,743 predecessors plus 19 additions = 1,762 passing
identities in each profile, with both affine/authority controls passed.
[Docs](https://github.com/gregkrsak/klyxr/actions/runs/38970650381) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38970650493) passed.
[Build](https://github.com/gregkrsak/klyxr/actions/runs/38970650641) stopped at one
rustfmt call-layout diff, which was applied exactly. Clippy/Rustdoc await the
corrective push; no later layer was started on this gate.

## Milestone 4 completed

Correction commit `17b382b0a76a9995857af6ba0a20a6fd654ca37f`, tree
`4aed797145f7c234c4b8c9452db1ec811180f9df`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38970841650),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38970841711),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38970841538),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38970841567).
Job 117208545567 reconciled 1,743 predecessors plus 19 additions = 1,762 passing
identities in both profiles; affine/doctest controls passed. All job steps and
relevant logs were inspected before milestone 5.

## Milestone 5 — Cleanup realization evidence and approval

The existing structured ownership checker now carries an effects-only may-custody
product. Validated initialization/consume/replacement transitions update it;
no new move or loan permission is introduced. Fallthrough alternatives union
residual custody, while effects from terminal branches remain recorded. Both-path
transfer leaves no old responsibility. Short-circuit skip custody is retained
independently of the predecessor's conservative usability state. Return results
transfer before remaining owners contribute. Loop locals contribute before
normalization removes their safety state. Displaced replacement owners contribute
before new-generation initialization; existing runtime ordering is unchanged.

The successful checker emits canonical holder/type/realization/origin evidence.
MIR receives that evidence, and opaque approval retains its immutable original.
Pre-execution validation checks shape/type/summary/order and deterministic lowering
against the retained witness without rerunning ownership or inferring runtime
effects. All compilation/preparation routes enforce the cleanup subset rule.

Nonempty cleanup effects reserve an additional conservative envelope before
checker snapshots: holder-site combinations times simultaneous snapshot depth,
plus original frontend storage; work is bounded by a checked 640*n^4 envelope.
The holder bound is the maximum within one function, not an interprocedural
ownership model. Empty-effect predecessors retain the old envelope. Diagnostic
release ancestry has a fixed 16-visit budget and stops honestly if truncated.

Seven new identities cover every ordinary exit and replacement, missing cleanup
rejection, pass-through/both-path transfer, protected return remainders,
short-circuit residual custody, independent MIR witness mutations and a canonical-looking protected-result cleanup
forgery under a legal unused superset, exact storage
neighbors, detected witness allocation failure, and expression-transaction custody
rollback. They are authored and await this milestone's authoritative CI results.
Temporary/private-domain integration and final conformance remain pending.

Milestone 5 first push `ccff5adae8a1d111e9c490afcfba06a1c19b4b20`, tree
`223de7860f68fb1878a56d439c203f40e906ba09`, compiled the implementation but failed
all-target checking in [build](https://github.com/gregkrsak/klyxr/actions/runs/38972493921)
and [tests](https://github.com/gregkrsak/klyxr/actions/runs/38972493916). One
predecessor fault-injection test called the private seal helper directly and
needed its new successful-checker-evidence argument. The fixture now obtains
ownership/effect approval before entering its unchanged allocation-failure loop.
All Host and no-artifact assertions remain unchanged; its exact identity and
reason are recorded under fixture_adaptations. [Docs](https://github.com/gregkrsak/klyxr/actions/runs/38972493924)
and [verify](https://github.com/gregkrsak/klyxr/actions/runs/38972493928) passed.
No later layer was started on this failing gate.

Correction `a5fe8e5db7a892ac3797eb7abde53fb33952ab57`, tree
`165db26097de9d909e8a0a4179689225e2582806`, passed [tests](https://github.com/gregkrsak/klyxr/actions/runs/38972672280),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38972672271), and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38972672467).
Both profiles independently reconciled all 1,743 predecessors plus 26 additions
(1,769 tests), the authority doctest, and 1,800 affine cases, with no ignored or
filtered tests. [Build](https://github.com/gregkrsak/klyxr/actions/runs/38972672297)
passed all-target compilation and failed formatting; Clippy and Rustdoc were
therefore skipped. The next correction applies only the reviewed CI rustfmt diff
to six implementation files; no later architectural layer has begun.

Milestone 5 formatting correction `17db6bc69a4a8c444786a2274a7b65805cfce123`,
tree `8c512d07bcd67f34ac41ab85af1da86dd3341df7`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38973143032),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38973142682),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38973142667),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38973142669).
All steps and relevant logs were inspected, including formatting, warnings-denied
Clippy/Rustdoc, both 1,769-test reconciliations, both affine checks, and the doctest.

## Milestone 6 — Effectful replacement, return, temporary and entry evidence

Six new runtime identities exercise successful ownership-approved source with
nonempty fixed operation contracts. They assert exact acquisition/release identity
order across replacement generations and nested argument pass-through, protected
return versus remaining parameter cleanup, and both short-circuit alternatives.
Every fuel cut through two-argument entry checks conservation, including an actual
partially published parameter frame. Runtime storage is checked at its exact
sufficient/insufficient neighbors. A release failure preserves Complete of the
first owner, Attempt/Failure of the next, and the never-attempted suffix, and
refuses resumption. Unused network/unsafe supersets add no events.

Existing canonical temporary site plans carry the mechanically derived latent
summary by record identity; transfer preserves that plan without an independently
writable effect mask. No ordinary source temporary remains eligible at successful
region retirement: accepted record values transfer to their validated destination
or are protected returns. Private leftover injection is protocol evidence and
will receive an explicit separate sealed domain in milestone 7.

These six tests are authored; authoritative CI evidence is pending.

Milestone 6 first push `85b4a5aa12664447383845585f50a146d16a605f`, tree
`939d6bf2dac69499569c63fe4519527b3afef5c6`, passed all-target compilation
and the debug suite. [Build](https://github.com/gregkrsak/klyxr/actions/runs/38973362869)
failed only formatting; its reviewed diff is applied to the two new test-related
source files. [Docs](https://github.com/gregkrsak/klyxr/actions/runs/38973362798)
and [verify](https://github.com/gregkrsak/klyxr/actions/runs/38973362811) passed.
[Tests](https://github.com/gregkrsak/klyxr/actions/runs/38973362797) was still
running the optimized profile at correction publication. No later layer was added.

Milestone 6 formatting correction `ec63b307422a5009f34905353c62825cd6e845ce`,
tree `650fd68cb804f3a35e078c46fd54023d49b8fe83`, passed
[build](https://github.com/gregkrsak/klyxr/actions/runs/38973468097),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38973468135),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38973468098), and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38973468089).
All job steps and relevant logs were inspected: 1,743 unchanged predecessor
identities plus 32 additions = 1,775 in each complete profile, both affine
checks and doctests, formatting, Clippy/Rustdoc, CLI/docs and byte reproducibility.
The earlier first push also finished both complete profiles successfully.

## Milestone 7 — Sealed private execution domains

Only Ordinary exists in production. Test builds additionally seal Primitive or
Approved Abort before opaque approval, retaining an independent immutable domain
copy. Existing state-injection fixtures use Primitive; every predecessor identity
covered by that helper family is explicitly listed in the evidence manifest.
Their protocol assertions and outcomes remain unchanged. New effectful ordinary
conformance fixtures use a separate constructor through the same successful
ownership, complete-call-contract, and cleanup-subset pipeline.

Approved abort templates derive complete cleanup unions from the canonical
written prefix of initialized construction members, and require each containing
function's declaration before approval. Templates bind function, MIR construction
site, record/plan, creation/failure origins, exact monotonic frame and pending
identities, and call-stack depth. They cover the entire affected pending stack.
Execution validates every frame, pending instance, initialized membership and
origin before the first attempted release; it does not rerun ownership or effects.
Canonical reverse-declaration cleanup then preserves the existing inner-to-outer
container order. No callbacks, source continuation, rollback, retry or unwind is
introduced. Ordinary stops never use this authority.

Metadata count, membership, scratch derivation and both retained copies fit an
additive reservation alongside the ownership/frontend envelope. Canonical
validation re-derives summaries from already validated plans and contracts.
Independent domain/template mutations, runtime frame/pending/membership/origin
mutations, nested cross-frame completeness, ordinary pending traps, refused
ordinary temporary injection, exact work/storage neighbors, detected allocation
failure, and an approved release Attempt/Failure prefix are authored controls.
They await authoritative CI; no local Rust executable evidence is claimed.

Milestone 7 first push `cdcdd35b3bbfa02c6c63961454024f1775ffd0ad`, tree
`d3900c23fd62424102ed8e78124d10c1da74bc20`, passed all-target compilation,
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38974063260) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38974063111).
[Build](https://github.com/gregkrsak/klyxr/actions/runs/38974063170) failed
formatting. [Tests](https://github.com/gregkrsak/klyxr/actions/runs/38974063148)
reported two new-fixture defects: the trap used unsupported `+` syntax, and the
refused-injection test assumed a resource state had been allocated before any
resource operation. The trap now uses supported subtraction below its range;
the latter asserts resource state is still absent, directly proving refusal
preceded allocation/acquisition. Neither correction changes the expected semantic
outcome or any predecessor assertion. The reviewed CI formatting diff is applied.
The failed debug job did not run the release profile. No later layer was started.

Correction `cc97e06e5c52799087ea5360de99a5918e7e13b0`, tree
`9845410f015d156259c4f9d0fb29bfa060802261`, passed
[build](https://github.com/gregkrsak/klyxr/actions/runs/38974183687),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38974183714), and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38974183678).
[Tests](https://github.com/gregkrsak/klyxr/actions/runs/38974183706) exposed
one remaining new trap-fixture syntax error: local type annotations are not
supported. A typed `zero()` result now supplies the same range context for the
mutable local. All other unit controls passed; release was skipped. The trap,
no-unwind and conservation assertions are retained unchanged.

Milestone 7 trap-fixture correction `79237e75e7b88f9a165277401ff00174dae7cda9`,
tree `085cfb73cd65605b92fbca60a1d51646ac01ed51`, passed all four workflows:
[build](https://github.com/gregkrsak/klyxr/actions/runs/38974357887),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38974357743),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38974357780),
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38974357744).
All steps and relevant logs were inspected. Each profile reconciled 1,743
predecessors plus 40 additions = 1,783 passing identities, both affine checks and
the authority doctest. No predecessor identity or outcome changed.

## Milestone 8 — Final authority and failure matrix

The checking traversal now retains only its first illegal call edge without a
site-table allocation. Retained MIR direct-effect sites allocate fallibly; bounded
preparation propagates allocation failure as Host with no executable. The legacy
inspection-only infallible MIR entry retains its existing infallible API.

Approved abort authority additionally binds the exact in-progress field's MIR site
and origin in every ancestor, and requires no active field at the triggering
container. Independent mutations and a missing active-field witness reject before
any release attempt. A recursive scenario distinguishes the same function and
construction site at two dynamic frame identities and stack depths.

Six additional identities cover all eight effects at the same cleanup site; real
empty-summary resource cleanup alongside inert Move and reference controls; every
ordinary pending-construction fuel stop without implicit abort cleanup; identical
recursive sites in different frames; diagnostic admission/rendering exact neighbors;
and detected retained direct-site allocation failure. The full identity reconciler
now also audits fixture adaptation identities, explicit primitive-domain inventory,
and the absence of predecessor contract corrections.

These controls are authored and await this milestone's authoritative CI gate.

Milestone 8 first push `f3ec1a0ae08030ed69591fe8c01dab03ca41b109`, tree
`6a5800f177bc1ae1a0d6a672d63a34048860de34`, passed all-target compilation
and [docs](https://github.com/gregkrsak/klyxr/actions/runs/38974811538).
[Build](https://github.com/gregkrsak/klyxr/actions/runs/38974811932) failed
formatting. [Tests](https://github.com/gregkrsak/klyxr/actions/runs/38974811829)
found one new fixture descriptor mismatch: the predecessor fixture helper changes
Pair's first/last fields to resource-bearing Leaf, while this new control intends
an inert Pair. Its original scalar field descriptors are restored explicitly;
the Leaf catalog, empty summary and real cleanup assertions remain unchanged.
The reviewed formatting diff is applied. The debug failure skipped release;
no later layer was started on this gate.

The milestone 8 first push also passed [verify](https://github.com/gregkrsak/klyxr/actions/runs/38974811882).
Correction `fa3137c34fbce41dab0e4c63fddf65c303b52dd9`, tree
`482eeb3da1ec07ee7152a8b538c34646c7f54950`, passed
[build](https://github.com/gregkrsak/klyxr/actions/runs/38974998414),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38974998309), and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38974998459).
[Tests](https://github.com/gregkrsak/klyxr/actions/runs/38974998387) passed all
unit controls, then exposed an incorrect new renderer expectation: the canonical
renderer returns its fixed truncated diagnostic when the full text does not fit.
The new test now checks exact full-text neighbors, exact fallback text and its
sufficient/insufficient neighbors. The renderer and predecessor tests are unchanged;
release was skipped after the integration failure.

Milestone 8 renderer-expectation correction `4206de4d3114ce1fb6f109e30e0bc4e50b3d6c83`,
tree `fd198352eae41fd0db57368116287053e8c607ee`, passed
[build](https://github.com/gregkrsak/klyxr/actions/runs/38975167087),
[tests](https://github.com/gregkrsak/klyxr/actions/runs/38975167117),
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38975167078), and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38975167054).
Every step and relevant log was inspected. Each complete profile passed 1,743
predecessors plus 46 additions = 1,789 identities, the authority doctest, the
1,800-case affine check and the 129-identity explicit protocol classification audit.

## Milestone 9 — Documentation, CLI examples and final conformance

Canonical KD-008 remains Direction Accepted for the broader system. The frozen
first ordinary phase and its implementation are documented without self-accepting
the implementation or assigning a new KD number. OQ-001 distinguishes settled
first-phase clauses/contracts from deferred broader syntax and assurance. Context,
architecture, compiler scope and bounded execution describe declarations, custody,
opaque approval, private domains, bounds and failure accounting. The generated
context bundle is rebuilt from its canonical sources.

Two public resource-free examples demonstrate conservative complete declarations
and a missing three-function transitive upper bound. A new CLI identity executes
positive/negative check, verify and run paths, preserving zero ordinary proof
counts and honest assurance reporting. The CLI adds an explicit checked-effects
line while retaining predecessor success/error behavior and scope assertions.
All earlier illustrative policy syntax remains unsupported.

The implementation does not add production effectful/resource APIs, hierarchy,
aliases, policy syntax, polymorphism, unsafe authority, user destructors, unwind,
rollback/retry or effects-aware verified/harness behavior. Independent exact-head
review remains Sarah's responsibility; Greg retains acceptance and humans retain
merge authority. No issue closure, approval, prod write or merge is performed.

### Frozen conformance audit

| Requirement | Executable or retained evidence |
| --- | --- |
| Eight atoms, canonical sets/raw rejection | effects unit tests: all 65,536 pairs; wider encodings; declaration mutation controls |
| Both function kinds/spellings, contextual names, exclusions | tests/effects.rs ordinary grammar and verified/harness controls |
| Complete call declarations and three-function propagation | all-eight call-form/spelling matrix; recursive diagnostics; new CLI positive/negative examples |
| Strict supersets, no fabricated events, unsafe audit only | empty/no-op execution; exact resource events with unused network/unsafe; moved-owner rejection |
| Mechanical operation/release/leaf/aggregate equality | resources unit matrices; canonical cache/plan independent mutations |
| Every ordinary cleanup boundary | effect_authority_tests all exits, iteration/break/continue and replacement, positive and missing-effect controls |
| Empty-summary responsibility, inert values and references | effects::empty_cleanup_summary_preserves_owner_but_does_not_own_references_or_inert_moves |
| Protected return and transfer deferral | both-path/pass-through tests; exact parameter remainder/return identities; canonical-looking protected-owner forgery rejection |
| Branch/short-circuit custody | successful checker witness tests plus skipped/executed runtime single-release controls |
| Replacement, temporaries, pending and partial entry | exact generation/argument event order; every-fuel conservation with an observed partially entered parameter frame |
| Ordinary stops versus approved abort versus primitive | four-way sealed-domain controls; pending traps/limits; refused injection; 129 predecessor protocol classifications |
| Exact private binding, nested/recursive pending stacks | template/domain/membership/origin matrices; cross-frame completeness; identical sites at distinct recursive frames; in-progress-field negative control |
| Attempt/Complete/Failure/never attempted | ordinary and approved-abort failure prefixes, unique retained holders, refused resumption |
| Opaque approval/correspondence | direct-site/custody/cache matrices plus preserved authority doctest and predecessor mutations |
| Admission/diagnostic/render/work/storage/Host | exact neighbors; bounded truncation; detected direct/witness/private allocation failures with no artifact |
| Compatibility | exact predecessor inventory, no identity renames or contract corrections; complete debug/release reconciliation and affine/doctest gates |

All milestone-9 changes and the complete branch are awaiting authoritative CI.
The only local verification claimed is static diff/manifest/context inspection;
no local Rust executable result is claimed. Final exact head/tree, independent
PR workflow results and full changed-file list will be recorded on the PR and
Issue #70 after final branch-push validation.

Milestone 9 first push `b8f956daff88c8529c83f04a83a31bbb8d402f2b`, tree
`cae7fe73f79974a6dcd5d54e42a1ea588122aeae`, passed
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38975591104) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38975591074), including
the new positive/negative CLI example control. [Build](https://github.com/gregkrsak/klyxr/actions/runs/38975591075)
passed all-target compilation and failed only formatting in the new CLI test;
the reviewed CI diff is applied. [Tests](https://github.com/gregkrsak/klyxr/actions/runs/38975591090)
records the complete debug/release reconciliation for that head.
Both profiles passed all 1,743 predecessors plus 47 additions = 1,790 identities,
the authority doctest, 1,800 affine cases and the explicit protocol-domain audit.

The final whole-branch audit also tightened source-origin precision: a clause span
now covers the keyword through its last member, and declaration validation rejects
members outside that span. Existing grammar and malformed-declaration identities
now assert full-clause slicing and a foreign member endpoint. This is an origin
correspondence correction, with unchanged effect sets, eligibility and safety rules.
`origin/prod` was fetched again and still exactly matches authorized baseline
commit `edf80c1e68e6fe3e2b25f6a557fc121395e45adc`, tree
`844afe3deacf22bc55844bc8ea92d76036cb3fc2`. Static asset/scaffold, manifest,
generated-context byte and full diff-integrity checks passed. All executable
claims still depend on the final correction's GitHub gate and independent PR runs.

Origin correction `d30d506be5143f9cb272937423e2a2056651d5dd`, tree
`56a9e2a8995bcc0bc2d69cd41caa136d090242d2`, passed all-target compilation,
[docs](https://github.com/gregkrsak/klyxr/actions/runs/38975842428) and
[verify](https://github.com/gregkrsak/klyxr/actions/runs/38975842326).
[Build](https://github.com/gregkrsak/klyxr/actions/runs/38975842300) requested
one final formatting hunk in the added member-origin mutation assertion; it is
applied without behavior changes. [Tests](https://github.com/gregkrsak/klyxr/actions/runs/38975842289)
contains that head's profile evidence. The next head changes only that formatting
and this auditable gate entry; final push and independent PR results are reported
externally with their exact immutable head/tree.
