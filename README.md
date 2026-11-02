<p align="center">
  <picture>
    <source media="(max-width: 767px)" srcset="./assets/readme-banner-mobile.png">
    <img src="./assets/readme-banner.png" alt="Bowfin fish logo and wordmark — Systems programming you can reason about" width="100%">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/gregkrsak/klyxr/actions/workflows/build.yml"><img alt="Build" src="https://github.com/gregkrsak/klyxr/actions/workflows/build.yml/badge.svg"></a>
  <a href="https://github.com/gregkrsak/klyxr/actions/workflows/tests.yml"><img alt="Tests" src="https://github.com/gregkrsak/klyxr/actions/workflows/tests.yml/badge.svg"></a>
  <a href="https://github.com/gregkrsak/klyxr/actions/workflows/verify.yml"><img alt="Verify" src="https://github.com/gregkrsak/klyxr/actions/workflows/verify.yml/badge.svg"></a>
  <a href="https://github.com/gregkrsak/klyxr/actions/workflows/docs.yml"><img alt="Docs" src="https://github.com/gregkrsak/klyxr/actions/workflows/docs.yml/badge.svg"></a>
</p>

<p align="center">
  <strong>Ownership · Contracts · Effects · Verification</strong>
</p>

<p align="center">
  <a href="#why-bowfin">Why Bowfin?</a> ·
  <a href="#documentation-is-part-of-the-program">Documentation</a> ·
  <a href="#a-quick-taste">Quick taste</a> ·
  <a href="#safe--checked--verified">Assurance model</a> ·
  <a href="#how-is-bowfin-different-from-rust">vs. Rust</a> ·
  <a href="#how-is-bowfin-different-from-adaspark">vs. Ada/SPARK</a> ·
  <a href="#status">Status</a>
</p>

---

# Bowfin

> **Systems programming you can reason about.**  

**Bowfin** is a systems programming language in development that combines
**ownership-based memory safety**, **first-class contracts**, **explicit effects**,
**domain-aware types**, and **incremental formal verification**.

It starts from a simple premise:

> **Memory safety is the beginning of the argument, not the end.**

Modern systems languages have shown that major classes of bugs can be moved from runtime into the language model.
Bowfin asks the next question:

> **What other engineering truths should be part of the program itself — visible to the compiler, checkable by tools, and auditable by humans?**

---

## Why Bowfin?

Systems code depends on more than valid pointers.

Real software also has rules like:

```text
this percentage must remain between 0 and 100

this control loop may not allocate

this task must not block

this operation may occur only in the ARMED state

this function must preserve the invariant

this assumption is trusted rather than proven
```

Today those truths often live in comments, tests, wrapper types, runtime checks, coding standards, review checklists, or the heads of experienced engineers.

**Bowfin wants more of them in the language.**

<p align="center">
  <picture>
    <source media="(max-width: 767px)" srcset="./assets/diagrams/knowledge-boundaries-mobile.svg">
    <img src="./assets/diagrams/knowledge-boundaries.svg" alt="Bowfin makes the boundaries between known, proved, trusted and unsafe visible" width="100%">
  </picture>
</p>

The goal is not magical correctness.

The goal is **engineering clarity**:

- what the compiler knows,
- what the verifier proved,
- what the programmer assumed,
- and where the guarantees stop.

---

## Documentation is part of the program

> **Documentation is part of the program's human-facing contract. Code and
> documentation are reviewed together, and disagreement between them is a
> defect.**

Bowfin does not assume that source code is self-explanatory. Public declarations
and examples should explain purpose, domain meaning, effects, authority, failure
behavior, lifecycle obligations, important policy, assumptions, bounds, and
non-guarantees in language people can read directly.

Machine-readable types, contracts, and effects establish facts for the toolchain.
Documentation explains what those facts mean in the application. Neither silently
substitutes for the other.

See the [Bowfin documentation policy](docs/DOCUMENTATION_POLICY.md).

---

## A quick taste

### Constrain the domain

```text
type Percent = range 0..100;

record Battery {
    charge: Percent
}
```

No convention. No “remember this value is a percentage.”

The type says what values may exist.

### State the contract

```text
verified function consume(
    battery: &mutable Battery,
    amount: Percent
)
    requires amount <= battery.charge
    ensures battery.charge == old(battery.charge) - amount
{
    battery.charge -= amount;
}
```

Now consider:

```text
let mutable battery = Battery { charge: 80 };

consume(&mutable battery, 90);
```

Nothing here is memory-unsafe.

But it is still wrong.

A Bowfin verifier should be able to explain **why**:

```text
error: precondition cannot be established

  --> src/battery.bow:14:1

14 | consume(&mutable battery, 90);
   | ^^^^^^^^^^^^^^^^^^^^^^^^^

required:
    amount <= battery.charge

known:
    amount == 90
    battery.charge == 80

90 <= 80 is false
```

That distinction is central to Bowfin:

> **Memory-safe does not automatically mean application-correct.**

---

## Effects are part of the API

Some rules are operational rather than mathematical.

```text
#[deny_effects(heap, blocking)]
function control_loop(...)
    effects clock
{
    estimate_state(...);
}
```

If a transitive dependency allocates:

```text
error: forbidden effect `heap`

control_loop
└── estimate_state
    └── resize_buffer
        ^ introduces effect `heap`
```

The control loop did not merely *document* that allocation was forbidden.

It made the rule **compiler-visible**.

---

## Safe → Checked → Verified

<p align="center">
  <picture>
    <source media="(max-width: 767px)" srcset="./assets/diagrams/assurance-ladder-mobile.svg">
    <img src="./assets/diagrams/assurance-ladder.svg" alt="Safe to checked to verified" width="100%">
  </picture>
</p>

Bowfin is designed around **one language with escalating assurance**.

### `safe`

Ordinary Bowfin code gets the language's core guarantees:

- ownership and borrowing,
- initialization safety,
- reference validity,
- bounded memory access,
- race restrictions,
- deterministic destruction.

### `checked`

Additional constraints and contracts may be enforced dynamically.

```text
checked function withdraw(...)
    requires amount <= balance
```

### `verified`

Declared properties become proof obligations.

```text
verified function withdraw(...)
    requires amount <= balance
    ensures balance == old(balance) - amount
```

> **Verification is not a second language you graduate into.  
> It is an assurance level you apply where it earns its cost.**

---

## Own what moves. Constrain what exists. Prove what matters.

Bowfin's design philosophy is intentionally compact.

### **Own what moves**
Ownership and borrowing make memory and mutation rules structural.

### **Constrain what exists**
Range types, units, invariants, and contracts express the domain directly.

### **Prove what matters**
Formal verification is available where consequences justify it.

### **Make every escape hatch visible**
Allocation, blocking, I/O, trusted assumptions, FFI, and `unsafe` boundaries should be easy to find and audit.

---

## How is Bowfin different from Rust?

Rust proved that systems programmers do not have to choose between
**low-level control** and **strong memory safety**.

Bowfin takes that lesson seriously.

> **Rust reasons deeply about ownership.  
> Bowfin aims to extend that style of reasoning to contracts, domain constraints, effects, trust boundaries, and selected program properties.**

Many of those capabilities can be built around Rust through types, libraries, conventions, and verification tools.

Bowfin's goal is different:

**design them as one coherent language model from the beginning.**

| | Rust | Bowfin |
|---|---|---|
| Ownership / borrowing | Core | Core |
| Memory safety without GC | ✅ | ✅ |
| Algebraic enums / matching | ✅ | ✅ |
| Range/domain types | Usually modeled | First-class design goal |
| Preconditions / postconditions | External patterns/tools | First-class |
| Explicit effects | Mostly encoded indirectly | Explicit ordinary upper bounds; broader system remains a design goal |
| Formal verification | Specialized toolchains | Integrated workflow |
| Assurance progression | Not language-level | `safe → checked → verified` |
| Trust audit | Tool/project dependent | Explicit design goal |

---

## How is Bowfin different from Ada/SPARK?

Ada/SPARK demonstrated that **contracts, constrained types, static analysis, and proof can be practical engineering tools**.

Bowfin draws deeply from that tradition — but uses a different programming model.

> **SPARK brings formal methods to Ada.  
> Bowfin is designed to make provable systems programming feel native to the ownership generation.**

Bowfin combines the assurance mindset with:

- ownership and borrowing,
- algebraic data types,
- exhaustive pattern matching,
- `Result`-style error handling,
- explicit effect sets,
- modern package/tooling ergonomics,
- and incremental rather than all-or-nothing verification.

The ancestry is deliberate:

> **Rust taught Bowfin how ownership can shape a language.<br>
> Ada/SPARK taught Bowfin that specifications and proof belong in real engineering.**

Bowfin's job is not to imitate either one.

It is to explore what happens when those lessons are designed together.

---

## The toolchain we want

The command line should feel boring in the best possible way:

```bash
bowfin new flight-control
cd flight-control

bowfin build
bowfin test
bowfin verify
bowfin audit
```

### `bowfin build`
Compile the project.

### `bowfin test`
Run tests.

### `bowfin verify`
Discharge declared proof obligations.

### `bowfin audit`
Show trust boundaries, unsafe regions, effect violations, and assurance-relevant project information.

The long-term goal is **one toolchain from hello-world to high assurance**.

---

## Designed for serious systems work

Bowfin is being designed with environments like these in mind:

| Embedded / realtime | Infrastructure | High assurance |
|---|---|---|
| controllers | runtimes | aerospace |
| robotics | operating systems | automotive |
| firmware | compilers | safety-critical systems |
| hard realtime | security tooling | critical infrastructure |

But Bowfin should never require formal methods expertise merely to get started.

You should be able to use Bowfin as an ordinary safe systems language — and turn up the assurance only when the problem demands it.

---

## What Bowfin does **not** claim

Bowfin is not based on the idea that a sufficiently fancy compiler can make software infallible.

A verified Bowfin function does **not** mean:

> “This code is correct.”

It means:

> **The stated properties were established under the stated assumptions.**

That difference is not legal fine print.

It is part of the language philosophy.

---

## Benchmarking and community conduct

Performance work should illuminate engineering tradeoffs, not diminish other
projects or the people behind them.

> **Measure honestly. Publish reproducibly. Report losses alongside wins. Respect
> the people and projects that make comparison possible.**

Official Bowfin performance claims and published benchmarks must:

- disclose the relevant source, compiler and toolchain versions, target,
  configuration, assurance mode, enabled checks, methodology, and known
  limitations;
- compare semantically equivalent work and identify material differences;
- publish representative losses alongside wins rather than cherry-picking;
- distinguish language design from compiler and backend maturity; and
- discuss other languages, compilers, tools, and communities accurately,
  respectfully, and with appropriate credit.

For interpreter/native comparisons, semantic agreement comes first: both engines
must execute the same approved MIR with the same observable Bowfin behavior before
their performance results are treated as comparable.

---

## Status

> [!IMPORTANT]
> **Bowfin is an early-stage language project.**
>
> The compiler, verifier, syntax, semantics, and tooling are still being designed
> and prototyped.
>
> The executable Rust prototype supports a deliberately bounded ordinary-language
> subset: nominal ranges and records, ownership and borrowing, structured control
> flow, deterministic cleanup, explicit effects, canonical `Result` values,
> exhaustive handling, `or return`, and deterministic interpretation of approved
> MIR under configured limits. The verifier supports a narrower integer-and-record
> contract subset.
>
> The prototype does not yet provide the complete planned ownership, verification,
> runtime, standard-library, package, or native-code-generation systems.
>
> The badges at the top of this README are **live GitHub Actions status badges**
> for this repository's `build`, `tests`, `verify`, and `docs` workflows.
> They report regression and documentation checks; they do not establish
> production readiness.

Try the current prototype from the repository root with Rust/rustup installed:

```bash
cargo run --locked --bin bowfin -- verify examples/battery_ok.bow
cargo test --locked --workspace
```

See [the prototype scope and proof model](compiler/README.md) for supported syntax,
exact guarantees, and intentional failure examples. The examples elsewhere in this
README describe the broader language design; they are not all accepted by the
current prototype.

### Run an ordinary function

```sh
cargo run --locked --bin bowfin -- run examples/ordinary_run.bow --function demo
```

This prints `Pair { first: Percent(89), second: Percent(90) }`. `run` performs the
existing frontend and ownership checks through bounded preparation, then executes
ordinary MIR. It does not evaluate verified proofs or harness calls and does not
prove ordinary contracts. Named-range violations stop immediately; configured
resource stops are not nontermination proofs. See
[bounded execution](docs/BOUNDED_EXECUTION.md) for supported limits and reporting.

Current design areas include:

- [x] public language story
- [x] ownership-first systems model
- [x] `safe / checked / verified` assurance concept
- [x] contracts and invariants
- [x] explicit effect model
- [x] visible trusted / unsafe boundaries
- [x] executable frontend for the documented ordinary-language subset
- [x] structured conditionals and loops with explicit MIR control flow
- [x] deterministic cleanup and bounded expression-temporary custody
- [x] typed recoverable failure with explicit handling and propagation
- [x] bounded ordinary MIR interpretation with opaque execution authority
- [ ] complete compiler MVP
- [ ] advanced borrowing and destruction
- [ ] broader verifier integration
- [ ] native-code generation
- [ ] standard library
- [ ] package ecosystem
- [ ] self-hosting

---

### Sensor-monitoring reference application

[Sentinel 0](docs/SENTINEL.md) uses the interpreter to classify two deterministic
Sensor readings and return a typed report, with explicit completion and bounded
failure handling. Its guide gives the complete invocation, fixture selections,
application-success convention and audit limitations.

## Prototype repository layout

```text
.
├── .github/
│   └── workflows/
│       ├── build.yml
│       ├── tests.yml
│       ├── verify.yml
│       └── docs.yml
├── assets/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── compiler/
│   ├── diagrams/
│   └── readme-banner.png
├── docs/
├── examples/
│   ├── battery.bow
│   └── effects.bow
├── scripts/
│   └── verify-readme.py
└── README.md
```

---

## Run this README locally

If your browser/GFM extension renders local Markdown directly, open `README.md`.

For the most reliable relative-image behavior, serve this folder locally:

```bash
python -m http.server 8000
```

Then visit the directory through:

```text
http://localhost:8000/
```

Your Markdown extension can render the README while all local assets remain available by relative path.

---

## Contributing

Bowfin is young enough that **careful thinking matters more than churn**.

When contributing, favor:

- precision over hype,
- clarity over cleverness,
- explicitness over convention,
- useful diagnostics over compiler mysticism,
- and honest tradeoffs over sweeping guarantees.

Areas where thoughtful contributors will eventually be valuable include:

- language semantics,
- ownership and aliasing,
- contracts,
- effect systems,
- SMT / verification architecture,
- diagnostics,
- embedded and realtime constraints,
- documentation,
- tooling,
- and examples.

---

## The long view

Bowfin began with a question:

> **If ownership can make memory safety part of the language, what other important engineering truths can we move out of comments, conventions, and institutional memory — and into the program itself?**

That question leads toward contracts.

Toward domain-aware types.

Toward explicit effects.

Toward visible trust.

Toward selective proof.

And ultimately toward software where engineers can say, with much greater precision:

**this is what we know.**

---

<p align="center">
  <strong>Systems programming you can reason about.</strong>
</p>

<p align="center">
  <code>bowfin build</code> ·
  <code>bowfin test</code> ·
  <code>bowfin verify</code> ·
  <code>bowfin audit</code>
</p>

<p align="center">
  <sub>Own what moves. Constrain what exists. Prove what matters. Make every escape hatch visible.</sub>
</p>

<!-- README-END -->
