# BED-033 trusted Sensor implementation

Frozen [Issue #78 Draft 2](https://github.com/gregkrsak/klyxr/issues/78) governs this
implementation. Implementation evidence is separate from Sarah's independent
exact-head review and Greg's acceptance. Only a human may merge. This is one
compiler-supplied deterministic demonstration family, not a hardware driver,
standard library, general FFI, capability language or new proof claim.

## Source and explicit configuration

The complete executable example is [trusted_sensor.bow](../../examples/trusted_sensor.bow):

```bowfin
fn sample() -> Result<success: Reading, failure: SensorError> effects io {
    let sensor = sensor_open() or return;
    let reading = sensor_read(&sensor) or return;
    return Ok(reading);
}
```

```bash
cargo run --locked --bin bowfin -- run examples/trusted_sensor.bow --function sample --profile sensor-demo
cargo run --locked --bin bowfin -- run examples/trusted_sensor.bow --function sample --profile sensor-demo --scenario no-sample
cargo run --locked --bin bowfin -- audit examples/trusted_sensor.bow --function sample --profile sensor-demo
```

Profile selection precedes source signature collection. Without explicit selection
the catalog contributes no names or authority. Selected denied operations still
occupy callable names. Same-namespace collisions reject without overwrite; type,
callable, local-value and field namespaces remain separate. Case rules are unchanged.
Catalog declarations have honest catalog origins rather than invented source spans.

`Reading` is the exact nominal range 0..4095; `SensorError` is 1..3. Equal bounds on
a source range do not establish identity. Existing named-range Copy, contextual
literals, comparisons and subtraction apply. Result constructors acquire no new
expected-type context. Sensor is a separate opaque Move Resource, never a RecordId.

| Scenario | Open | Read | Final sample result |
| --- | --- | --- | --- |
| `sample` (default) | one fresh Sensor | Reading(42) | Ok(Reading(42)) |
| `no-sensor` | Error 1, no Sensor | not attempted | Error(SensorError(1)) |
| `open-io-failure` | Error 3, no Sensor | not attempted | Error(SensorError(3)) |
| `no-sample` | one fresh Sensor | Error 2, Sensor stays live | Error(SensorError(2)) |
| `read-io-failure` | one fresh Sensor | Error 3, Sensor stays live | Error(SensorError(3)) |

These outcomes repeat deterministically on explicit source calls; there is no
implicit retry or alternative binding. Error 1 means no selected-profile sensor;
2 means no sample for that read; 3 means recoverable fixture I/O failure. Open
cannot emit 2 and read cannot emit 1. Wrong nominal identity, range, tag, payload,
fresh identity or provenance is terminal Integrity before Result publication.

CLI configuration is deliberately closed: `--scenario` selects one table row;
`--sensor-limit` is 1..1024 (default 16); `--grants` is `all` (default), `none`,
`open-release`, `read-release` or `release`. `--function` supplies the single CLI
entry root. The API supports an immutable bounded list of explicit roots and
independent open/read/release booleans through `trusted::Selection::sensor_demo`.
Every admitted acquisition requires its exact fixed release grant and binding.
An `effects io` clause supplies no grant. Source cannot call release.

## Custody and supported composition

The same canonical type, ownership, loan, outcome, effect and temporary systems
handle trusted calls. Trusted argument destinations carry operation and argument
IDs, never synthetic source FunctionIds or ParameterIds. Typed MIR invocation
plans bind the receiver, Result, argument sites, origins, exact catalog/binding
contracts and structural edges. Opaque approval seals all copies and witnesses.
Runtime dispatch uses that approved contract, with no source/HIR/name/host fallback.

Whole owned locals, moves, owned parameters, shared aliases/non-escaping helpers,
protected internal Sensor returns, nested Results and admitted BD-044 destinations
compose with matches, `or return`, branches, loops, break, continue and returns.
`retire(sensor: Sensor) -> bool` requires io for residual parameter cleanup;
`relay(sensor: Sensor) -> Sensor` needs no io for transfer alone. That difference
comes from successful custody evidence, never runtime vacancy or event inference.

Mutable Sensor references, Sensor conditionals/replacement/reinitialization,
public record fields, representation access, arithmetic, equality, Copy/Clone,
casts, serialization and forging remain excluded. Public live-Sensor or
Sensor-bearing Result egress rejects before entry, even if an inactive Error
alternative would carry no Sensor. Whole-Result cleanup remains sealed private
primitive/protocol authority: ordinary source must resolve its outcome obligation.

Use `execution::prepare_execution_with_profile` or its bounded-reader counterpart,
then `ExecutionSession::new` and `advance`. The resource-free consuming `execute`
API remains compatible and rejects selected profile artifacts. The profile CLI
keeps its session alive through result/diagnostic rendering and writing.

Before binding dispatch the machine reserves receiving capacity jointly for
receipt payload, token identity, Result cell/provenance, operand, temporary owner,
audit prefix and stopped diagnostics. The fixed binding creates its non-Clone
token directly inside the owned receipt. Validation never leaves a raw handle
only on the Rust stack across a fallible boundary. Publication allocates nothing;
the actual token transfers to the machine's live table and its sole non-Copy
language ticket transfers through the existing custody protocol. Histories own
no tokens. Every successful fixed release occurs once and precedes caller delivery.

Host, configured Limit, Integrity, range traps and cleanup failure remain terminal
tool outcomes. The stopped session retains receipts, binding state, protected
Results, cleanup cursor, completed prefix and unfinished responsibilities.
`advance` returns a borrowed recorded outcome; repeat calls never resume, retry,
unwind or release. A protected Ok or Error retained after a terminal stop has not
been delivered. Teardown/process reclamation is not successful Bowfin release.
Automatic release has no source Result and promises no flush, commit, durability,
publication or protocol completion. Explicit fallible completion remains deferred
to BED-034; none is implemented here.

## Bounded audit and execution

The established admission envelope adds selected catalog, configuration, approval,
root and diagnostic terms even for tiny source. Runtime working storage includes
the selected state header, token/provenance/event capacities, outstanding receipt,
protected values and stopped diagnostics. Invocation/resource scans and existing
evaluation phases consume explicit fuel. The configured live Sensor limit is
checked before the excessive attempt. Existing no-profile numeric protocol oracles
are unchanged. No hard real-time, system-OOM recovery or device proof is claimed.

`CheckedExecutable::render_authority` validates the same opaque artifact and
performs bounded structural reachability from its sealed roots. Empty-effect
helpers, recursion and both alternatives participate independently of runtime
events and effects filtering. The report discloses exact identities, contracts,
assumptions, configuration, grants versus reachable operations, source/catalog
origins, opaque Resource/acquisition/release pairing, possible cleanup sites and
recoverable-versus-terminal policy. It grants nothing and invokes no binding.

Audit uses `ExecutionLimits`: output allowance plus graph scratch and builder
header are conservatively charged together; output-byte work and every graph or
metadata scan consume audit steps. Opaque revalidation uses its retained bounded
admission envelope. `ExecutionSession::render_authority` adds the retained machine
storage to the same audit storage check. Rendering/write failure cannot change
external outcomes or dispose of live custody. Result and diagnostic output retain
the existing explicit allowance; detected Host/tool or writer failure is CLI class
2, frontend/terminal Limit/Integrity/trap is class 1, and a delivered source Error
is successful execution (class 0).

All malformed/injected controls are private cfg(test) at real preparation,
allocation, receipt, publication, cleanup or reporting boundaries. They do not
manufacture public authority, require actual system OOM, catch a panic to simulate
allocation failure, or offer a source/CLI/environment production fault switch.

## Evidence

The [implementation ledger](BED-033-IMPLEMENTATION-LEDGER.md) records each checkpoint,
exact Git identities, local gates, CI jobs, environment limitations and corrections.
The [evidence manifest](bed-033-evidence.json) maps all 35 section-20 obligations,
13 section-21 controls and P01-P09 to exact classified identities. The reconciler
requires every one of the 1,875 predecessor identities plus all additions, zero
ignored/filtered/failed tests, authority doctest, both profile affine checks and
the preserved BED-032 reconciliation chain. No test migration, contract correction
or numeric-oracle relaxation is claimed. Independent acceptance remains pending.
