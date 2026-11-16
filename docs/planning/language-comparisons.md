# Bowfin Language Comparison Strategy

**Status:** Planning note  
**Recorded:** 2026-11-16  
**Scope:** Future Bowfin README and website positioning. This note is not canonical language semantics, approved public copy, or a project-authority decision.

## Purpose

Bowfin should answer the comparison questions readers are likely to arrive with—including questions prompted by fashionable or newly visible languages—without pretending that every language is solving the same problem.

The comparison should:

- acknowledge the reader's existing mental map;
- explain Bowfin's design center clearly;
- distinguish language features from tooling, verification systems, and certified toolchains;
- give competing languages full credit where they are the better choice;
- avoid a green-check/red-X trophy chart;
- state Bowfin's present maturity candidly.

The goal is not to “defeat” other languages. It is to help a reader understand **what Bowfin is for, what tradeoffs it makes, and why it exists**.

## Recommended language set

### README: concise core comparison

The README should keep the set small enough to scan:

| Language or ecosystem | Why readers may ask about it | Comparison emphasis |
|---|---|---|
| **Bowfin** | The subject of the page | Safety and reliability as a systems-language design center; explicit ownership, effects, authority, trust boundaries, domain types, and proportional verification |
| **Rust** | The dominant contemporary memory-safe systems-language reference point | Ownership and borrowing, unsafe boundaries, concurrency, ecosystem maturity, and the distinction between Rust itself and its assurance ecosystem |
| **Ada + SPARK** | The established high-assurance and safety-critical reference point | Strong typing, contracts, proof, certification history, runtime model, and the important distinction between Ada and the SPARK subset/toolset |
| **Bend 2** | A visible, novel language associated with automatic parallelism and proof-oriented ideas | Pure/affine/dependent typing, laws and proofs, automatic parallel execution, runtime/backends, youth, and current limitations |
| **Zig** | A popular low-level alternative to C and Rust | Explicit control, manual resource management, simplicity, compile-time facilities, and a different safety philosophy |
| **Mojo** | A highly visible performance language with Python-family syntax | Performance portability, ownership features, accelerator ambitions, tooling maturity, and target audience |
| **C and C++** | The baseline systems-programming comparison that readers already know | Control, interoperability, ubiquity, undefined behavior, manual safety burden, and migration practicality |

Rust, Ada/SPARK, Bend 2, Zig, Mojo, and C/C++ cover the questions most likely to exist in readers' minds now. They belong in the main comparison before more specialized languages are added.

### Website: deeper profiles

The website can support individual comparison pages or expandable profiles for the README languages, plus:

| Language or family | Why it is useful to include |
|---|---|
| **Hylo** | Mutable value semantics and a research-driven approach to safe, efficient systems programming |
| **Austral** | Linear types, capabilities, and unusually explicit authority/resource reasoning |
| **Dafny, F\*, and Lean 4** | Proof-oriented languages and tools that clarify the boundary between a programming language, a verifier, and a theorem prover |
| **Carbon** | A likely “what about this?” question because of its C++-successor positioning; best kept on a watchlist while it develops |
| **Move** | Resource-oriented semantics and strong asset invariants; useful when discussing domain-specific safety, though not a general-purpose systems-language peer in the same sense |

### Rust assurance ecosystem

A fair Rust comparison must not stop at the base language. The website should separately acknowledge:

- **Verus** for deductive verification of Rust programs;
- **Kani** for model checking Rust;
- **Ferrocene** for a qualified Rust toolchain and safety-critical use.

These are not interchangeable with one another, and none should be silently treated as “Rust by itself.” That distinction is central to an honest comparison with Bowfin's intended integrated assurance story.

### Watchlist and exclusions

- **Vale** should not be presented as a current active competitor; its official site identifies it as archived.
- **Valen**, Vale's emerging successor, belongs on the watchlist until it is mature enough for a stable factual comparison.
- Other languages should be added only when they represent a materially different design answer or a question readers are demonstrably asking. Popularity alone is not enough.

## Comparison dimensions

Every comparison should use the same factual axes:

1. **Primary goal** — What problem is the language designed to solve?
2. **Memory and resource model** — Ownership, borrowing, tracing, linearity, manual management, or another model.
3. **Concurrency and parallelism** — Explicit versus automatic; data-race story; scheduling/runtime assumptions.
4. **Verification and assurance boundary** — What the language guarantees, what optional tools prove, and what remains trusted.
5. **Effects and authority** — How side effects, capabilities, privileges, and ambient authority are represented or constrained.
6. **Unsafe or escape mechanisms** — Whether they exist, how they are bounded, and how they affect the assurance claim.
7. **Runtime, compiler, and backend** — Native compilation, runtime dependencies, supported targets, interoperability, and implementation trust.
8. **Toolchain and ecosystem maturity** — Libraries, debugging, deployment, stability, adoption, and production evidence.
9. **Best-fit use cases** — Where this language is the stronger choice today.
10. **Bowfin distinction** — The specific design difference, stated without dismissing the other language.

## Editorial rules

- **Use primary sources.** Link directly to the language's own documentation, specification, repository, or official project site.
- **Date-stamp the comparison.** Bend, Mojo, Carbon, Hylo, Austral, Valen, and Bowfin can change quickly.
- **Pin material claims when practical.** For durable technical claims, cite a release, commit, or archived documentation revision rather than an undated landing page.
- **Separate fact from inference.** Clearly label Bowfin's interpretation of another project's design rather than presenting it as that project's own claim.
- **Separate language, tools, and certification.** For example: Ada is not SPARK; Rust is not Verus, Kani, or Ferrocene; a verifier is not a compiler guarantee.
- **Say where the other language wins.** A useful comparison helps readers choose, even when the answer is not Bowfin.
- **Do not overclaim Bowfin.** Distinguish implemented behavior, frozen specification, planned architecture, and aspiration.
- **Compare design centers before feature lists.** Superficially similar syntax or individual features can conceal fundamentally different goals and trust models.
- **Avoid winner grids.** A row of checkmarks rewards vague definitions and hides tradeoffs.

## Proposed information architecture

### README

Add a compact section titled **“Where Bowfin Fits”**:

1. a short paragraph describing Bowfin's design center;
2. a compact comparison table covering the core set;
3. one candid maturity statement;
4. a link to the full website comparisons.

The README should orient, not litigate every language-design question.

### Website

Create a comparison hub with:

- a shared methodology and “last reviewed” date;
- one profile per language or ecosystem;
- a consistent set of comparison axes;
- primary-source links;
- a “better fit when…” section for the other language;
- a “Bowfin differs because…” section;
- a watchlist for fast-moving projects.

This structure lets Bowfin answer trendy questions without rebuilding the README every time a new language becomes visible.

## Initial source set

These are starting points for the eventual fact-check; final public copy should cite the most precise stable page or pinned revision available.

- [Rust language](https://www.rust-lang.org/)
- [Verus guide](https://verus-lang.github.io/verus/guide/)
- [Kani documentation](https://model-checking.github.io/kani/)
- [Ferrocene](https://ferrocene.dev/en/)
- [AdaCore SPARK Reference Manual](https://docs.adacore.com/live/wave/spark2014/html/spark2014_rm/introduction.html)
- [Bend repository](https://github.com/bendlang/bend)
- [Zig overview](https://ziglang.org/learn/overview/)
- [Mojo](https://mojolang.org/)
- [Mojo ownership documentation](https://mojolang.org/docs/manual/values/ownership/)
- [Hylo introduction](https://hylo-lang.org/introduction/)
- [Austral](https://austral-lang.org/)
- [Dafny](https://dafny.org/)
- [F\*](https://fstar-lang.org/)
- [Lean](https://lean-lang.org/learn/)
- [Carbon documentation](https://docs.carbon-lang.dev/)
- [Move book](https://move-language.github.io/move/)
- [Vale archive notice](https://vale.dev/)

## Recommended next step

When Greg approves this structure, prepare two separate drafts:

1. the exact concise README section; and
2. the website comparison-hub copy and per-language page template.

Before publication, fact-check every material statement against current primary sources and classify every Bowfin statement as implemented, specified, planned, or aspirational.
