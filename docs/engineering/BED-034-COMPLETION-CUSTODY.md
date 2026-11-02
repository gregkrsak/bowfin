# Explicit fallible completion: implementation candidate

BED-034 Draft 2 is frozen in Issue #82. This implementation is a candidate on
draft PR #83, based on commit `372230df1d359b13d28f5c136130ac031507716a`, tree
`a646e9576a9b7650ff9a95275f1080b591d06986`. Independent exact-head review and
Greg's acceptance remain pending. Only a human may merge.

## Source and selected authority

The selected sensor-demo catalog adds exactly:

```text
sensor_finish(Sensor)
    -> Result<success: FinishReceipt, failure: SensorError>
    effects { io }
```

FinishReceipt is the exact nominal named range `1..1`. It is Copy status data.
Ordinary source can form it at existing named-range boundaries; it is not proof,
authority, attestation or evidence that finish occurred. Same-shaped foreign
ranges do not substitute. The defining source is `examples/sensor_finish.bow`.

The parameter consumes one whole canonical Sensor, including a lawful relay
temporary or selected Ok payload. No reference, moved owner, live conflicting
loan, record field, replacement generation or public imported handle is admitted.
The existing ownership checker, receiving holds, temporaries and Result obligations
remain authoritative. No second move/loan analysis or fake source Place exists.

The complete finish grant and exact paired cleanup-only release grant are required
independently of `effects io`. Denial retains callable namespace occupancy and
cannot fall back to a source declaration or host symbol. Open/read/release origins
and IDs keep their predecessor rows; receipt and finish have distinct new rows.

## Custody and terminal states

| Phase | Authoritative Sensor responsibility | Ordinary outcome |
|---|---|---|
| Before reservation | Actual current temporary, including helper-returned owner | None |
| Reserved | Same owner; receiver capacity exists | Reserved cell capacity |
| Consumed / attempted | Pending completion ticket; physical token in session table | Owned response receipt |
| Outcome protected / release pending | Pending completion ticket | Separate CompletionProtected Result cell |
| Releasing | Existing cleanup detached/work/current cursor | Same protected cell |
| Released | Physical token removed by exact fixed release | Protected cell, no caller delivery yet |
| Delivered | No Sensor responsibility | Active Result published to ordinary temporary |

The existing successful ownership consume path emits an invocation cleanup holder
in the same cleanup-effect witness table. Approval derives it bidirectionally from
MIR and seals holders, ordered Result, alternatives, effects, grants, fixed pair,
four origins, destination, eight phases and four event identities. Runtime binds
the original frame, realm, fresh invocation and production identity. It never
looks up a host symbol by a source name.

Before taking the current input, one checked sum covers receiving provenance,
Result cell, operand, temporary owner and transfer/register events, maximum finish
prefix events, cleanup work and real diagnostic backing storage. Owned argument
evaluation already provides both release/delivery task slots; that invariant is
checked before handoff. Actual capacities are rechecked together before consume.
The old source place is never restored after a relay transfer.

Finish success produces only Ok(FinishReceipt(1)); ordinary failure produces only
Error(SensorError(3)). Codes 1 and 2, wrong nominal identities, invalid tags or
provenance are Integrity failures. The deterministic partial-error scenario records
one irreversible prefix unit and then Error(3); it is not completion success.

Both valid alternatives protect an inert Result before the same fixed cleanup
engine releases the Sensor. Release failure retains its current cursor, real
payload, prefix and protected outcome. A fuel cut after release but before delivery
retains the outcome with no live Sensor. No protected cell is caller-visible.
Inner delivery can succeed before a later enclosing expression traps; that later
failure cannot undo the completed finish or release.

Host, Limit, Integrity and range/cleanup failures are terminal, never Error(3).
ExecutionSession retains its machine and recorded outcome. Repeated advance borrows
that outcome with no retry, resume, rollback, compensation, unwinding cleanup,
duplicate release or late delivery. Teardown is not successful language cleanup.

## Actions-only commands and static audit

These commands are for the authorized GitHub Actions environment. No local Rust
execution is authorized for this task.

```bash
cargo run --locked --bin bowfin -- run examples/sensor_finish.bow --function sample --profile sensor-demo
cargo run --locked --bin bowfin -- run examples/sensor_finish.bow --function sample --profile sensor-demo --scenario finish-io-failure
cargo run --locked --bin bowfin -- run examples/sensor_finish.bow --function sample --profile sensor-demo --scenario finish-partial-io-failure
cargo run --locked --bin bowfin -- audit examples/sensor_finish.bow --function sample --profile sensor-demo
```

The CLI integration tests execute these real paths through the built binary.
`sensor_finish_live_loan_fail.bow` and `sensor_finish_reuse_fail.bow` are rejection
controls. Existing `open-release` denies finish; `finish-release` denies open/read;
`open-finish-release` denies read. An effect declaration changes none of these grants.

Audit is a bounded, recursion-safe static account of sealed rooted structural
edges, including effect-free helpers and both branches. It distinguishes grants
from reachable use, finish from release-only cleanup, direct finish effects from
cleanup-derived release effects, both completion outcomes, release attempt,
completion and failure, and terminal non-delivery. It invokes nothing, observes no
scenario trace and grants no authority. Public session events are a separate actual
runtime prefix. A source-formed receipt and a release-only path establish no finish.

## Evidence and bounds

The ledger and `bed-034-evidence.json` map frozen obligations to code and exact test
identities. `verify-bed034-tests.py` reconciles every one of the 1,981 predecessors
and every classified addition in each complete unfiltered profile. M5 is separately
preserved and reported before M6. All eight ordinary push/PR workflows remain
required, including both 1,800-case affine checks and the authority doctest.

`completion_bound_tests` measures sufficient/insufficient preparation, combined
receiving, runtime peak/fuel, audit and output neighbors. Every observed actual
finish receiving growth gets a typed Host cut before dispatch. Fixed-release event
and work capacity are prepaid. A physical-token/sole-holder census covers every
fuel cut for success, immediate Error and partial Error. Diagnostics preserve the
existing bounded truncated fallback; its minimum size is checked separately.
Actions logs contain JSON measurements from `verify-bed034-budgets.py` for each
complete profile. These are exact finite-fixture test-build measurements at the
real production seams, not portable byte constants or a proof over all programs.

The existing predecessor catalog budget oracle remains unchanged. Selected BED-034
preparation adds explicit descriptor, protocol-copy, traversal and diagnostic
terms through the same Context. No project limit or predecessor assertion is
relaxed. A larger two-Result source fixture explicitly uses the existing supported
admission-storage maximum; ordinary production defaults remain unchanged.

This fixed in-process fixture is not a general device API, FFI, fallible destructor,
retryable resource-returning Error, typestate mechanism, durability promise, host
OOM recovery, hard realtime guarantee or new formal proof rule. Verified bodies,
contracts and harnesses gain no trusted authority. Private malformed controls are
compiled only for tests and cannot grant production or source authority.
