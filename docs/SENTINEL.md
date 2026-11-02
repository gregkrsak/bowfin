# Bowfin Sentinel 0

Sentinel is a deterministic, interpreter-run sensor-monitoring application. It
accepts two successful readings, counts disjoint warning and critical observations,
retains the peak and greatest severity, explicitly finishes every acquired Sensor
on ordinary paths, and returns a typed Report or Failure. It uses one source file
and a zero-argument `monitor` entry. Read the ordinary `//` comments in
[the application](../examples/sentinel.bow) as its human-facing contract.

## Run

From the repository root, use the following existing CLI allowances for the
complete application (not the smaller predecessor example):

```bash
cargo run --locked --bin bowfin -- run examples/sentinel.bow --function monitor --profile sensor-demo --scenario warning --sensor-limit 1 --max-tokens 2048 --max-frontend-work 10000000000000000 --max-admission-storage 1073741824 --max-diagnostic-storage 4194304
```

These are supported explicit ceilings, not measured minima or actual host RSS.
Runtime defaults remain 1,000,000 steps, 16 MiB working storage, call/expression
depth 64 and 65,536 output bytes. Preparation reserves conservative envelopes;
small runtime state does not imply small preparation allowances. No limit or
accounting schedule is changed. The integration tests execute this exact CLI
configuration in both compiler build profiles.

## Policy and result

A successful read is accepted before finish. Values 0..49 are normal, 50..79
warning, and 80..4095 critical. Critical observations do not also count as warnings.
Disposition is 0 normal, 1 warning, 2 critical, and never decreases. Peak is the
largest accepted Reading. No array, application allocation, string, clock, network,
filesystem, external configuration, general FFI or library API is involved.

With `warning`, stdout is:

```text
Ok(Report { observations: Count(2), warnings: Count(1), critical: Count(0), peak: Reading(60), disposition: Disposition(1), completed: Count(2) })
```

All fields are computed by Bowfin. The Rust host renderer supplies names,
punctuation, numeric formatting and the newline. It does not classify readings.
Warnings and critical dispositions are successful reports, not application failures.

| Scenario | Readings/outcome | Observations | Warnings | Critical | Peak | Disposition | Completed |
|---|---|---:|---:|---:|---:|---:|---:|
| sample | 42,42; Report | 2 | 0 | 0 | 42 | 0 | 2 |
| warning | 60,42; Report | 2 | 1 | 0 | 60 | 1 | 2 |
| critical | 90,60; Report | 2 | 1 | 1 | 90 | 2 | 2 |
| no-sensor | acquisition Failure, problem 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| open-io-failure | acquisition Failure, problem 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| no-sample | read Failure, problem 2 | 0 | 0 | 0 | 0 | 0 | 1 |
| read-io-failure | read Failure, problem 3 | 0 | 0 | 0 | 0 | 0 | 1 |
| finish-io-failure | completion Failure, problem 3 | 1 | 0 | 0 | 42 | 0 | 0 |
| finish-partial-io-failure | completion Failure, problem 3, committed host prefix | 1 | 0 | 0 | 42 | 0 | 0 |

The Failure record preserves those partial aggregates plus phase (1 acquisition,
2 read, 3 completion), original nominal SensorError in `problem`, `read_status`
(0 no preceding read error, 2 no sample, 3 read I/O failure), and `completion`
(0 not attempted, 1 succeeded, 2 failed). A zero peak is not a no-data indicator;
inspect the outer Result, phase/problem and observation count. `read_code` is used
only for the two admitted read errors; it is not a general SensorError conversion.

For example `no-sample` delivers:

```text
Error(Failure { phase: Stage(2), problem: SensorError(2), read_status: ReadStatus(2), completion: CompletionStatus(1), observations: Count(0), warnings: Count(0), critical: Count(0), peak: Reading(0), disposition: Disposition(0), completed: Count(1) })
```

Open/read attempts are not aggregate fields: failed acquisition creates no Sensor;
a successful acquisition leads to one read attempt in that cycle. Runtime events
provide attempt evidence separately. Completed resources and accepted observations
count different events. A Sensor can
finish successfully after its read failed, so completed greater than observations
is legitimate in Failure. Continuing loop headers have equal counts, at most two.
The guard permits a body only at counts zero or one; each increment stays in range.
These are reviewed algorithm invariants, not ordinary verifier proofs.

## Fixture and custody boundary

The finite immutable `warning` and `critical` selections use the successful Sensor
acquisition identity within one admitted session. Sensor zero has the first value,
Sensor one the second. Further opens return existing SensorError(1), including
repeated opens after exhaustion. Reading the same live Sensor repeatedly is stable.
Release can free a physical token slot but cannot reset or replay the sequence.
A fresh admitted execution resets it. No process-wide counter selects readings.

Selection is authenticated with the original approved catalog. New scenarios add
no operation, type, grant, callback or general input facility. Existing scenarios
are unchanged. The live-Sensor gate retains precedence over fixture exhaustion;
with limit one, a second simultaneous owner stops before an excessive attempt.
Sequential acquire/read/finish/release cycles fit that limit.

A successful read does not consume its Sensor. Both read paths explicitly finish.
Finish consumes the owner, protects its Ok or Error, performs fixed release, then
publishes the ordinary result. A read error followed by finish Error reports the
completion problem while retaining the read status. A sealed private conformance
test exercises that combination; it is not a public third scenario.

## Application failure versus tool failure

The CLI tool can exit zero for either `Ok(Report)` or `Error(Failure)`. Consumers
must inspect the typed outer Result; process success alone is not application
success. Warning/critical Report values mean the requested work completed.

Limit, Integrity, range trap, Host, release, admission and reporting failures are
terminal tool outcomes, not Failure records. Exit one covers non-Host tool stops;
usage/configuration and detected Host/writer failures use exit two. Add
`--max-steps 1` for an early terminal limit control. It is not a custody-after-open
test. Unit tests separately cover every fuel cut, acquired custody and release
before protected publication. There is no retry, rollback, unwinding cleanup or
late delivery after a stop; teardown is not successful Bowfin release.

## Audit and evidence

Replace `run` with `audit` in the full invocation above to inspect configuration,
grants, rooted call sites and possible completion/cleanup protocols. Static audit
executes no binding and proves no actual reading, branch, aggregate or lifecycle.
It bounds its own work/storage/output; it is not a full runtime-limit attestation.
Independent preparations do not become the same approval because their reports
look equivalent.

Typed unit assertions and CLI assertions check the application result. Separate
runtime tests associate fresh Sensor identities, imported readings, finish attempts,
protected outcomes and release-before-delivery events. Aggregate counts and
source-formable receipt data alone are not lifecycle attestations. Repeated-run
checks distinguish semantic results, equivalent authority descriptions and fresh
session identities. Private threshold controls exercise the real application at
0,49,50,79,80,4095 without adding public scenarios.

This experiment makes no real-device, formal application proof, hard realtime,
portable artifact, restart recovery or native execution claim. See the
[engineering ledger](engineering/SENTINEL-0-LEDGER.md) for exact-head evidence and
review status; implementation does not imply owner acceptance.
