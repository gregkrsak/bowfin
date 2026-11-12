# BED-038 N1 — Authenticated Native Product and Invocation Association

**Record ID:** `B38-N1-SNCC-01-R1`
**Status:** **PROPOSED NON-NORMATIVE ENGINEERING CONTINUITY RECORD — NOT ADOPTED**
**Date:** 2026-11-12 (Pacific)
**Scope:** BED-038 Stage N1 plan, authenticated product, direct-entry, invocation, and evidence relationships
**Repairs:** `B38-N1-SNCC-01-F01`, `B38-N1-SNCC-01-F02`, and `B38-N1-SNCC-01-F03` from Sarah's independent review
**Predecessor:** `B38-N1-SNCC-01`, retained unchanged for audit

## 1. Purpose, authority, and non-authority

This note separates: (a) obligations already imposed by the frozen Stage N1 composite; (b) optional implementation and hardening techniques; (c) guidance for future target-profile design; and (d) dated implementation status. It creates no new admission gate, execution mechanism, failure classification, target requirement, or authority.

The controlling Stage N1 specification remains exactly:

`FC6 + EV2 + FA1 + FA2-R1 + FA3 + FA4 + FA5 + FA6 + FA7-R1 + FA8`

Original FA7 and superseded FA2 remain excluded. Exact identities and links are retained in the [FA8 composite refreeze](https://github.com/gregkrsak/bowfin/issues/117#issuecomment-6787731375).

This record is not a normative appendix, amendment, refreeze, owner interpretation, implementation authorization, or evidence acceptance. It does not create FA9. If this record conflicts with the controlling composite, the composite controls and this record requires correction.

No statement here accepts an implementation, checkpoint, source tree, plan, product, executable, invocation, native result, or evidence. No statement grants commit, push, PR, workflow, merge, production, Stage N2, or Stage N3 authority.

## 2. Faithful core principle

> Complete independent plan validation, authenticated build and product associations, required executable inspection, and the actual direct invocation cannot be replaced by partial checks, matching output, or agreement between producers and checkers.

Native entry requires the original approval, complete independent plan validation, authenticated build and product associations, required executable inspection, and invocation admission specified by the frozen composite. The actual direct call and its evidence must concern the selected authenticated product and inspected entry under the associations required by the frozen composite.

An artifact digest, product record, pathname, descriptor, symbol, relocation, address, function pointer, metadata observation, or association field has the evidentiary role assigned to it by the composite. None is irrelevant merely because it is insufficient alone to prove the complete plan/build/product/invocation relationship.

This record does not add physical-file-object identity, continuous write exclusion, mutation-history attestation, descriptor-bound launch, process-side mapping attestation, or a particular capability type as separate N1 requirements. Those techniques may be proposed and reviewed as implementation choices, subject to the existing command, material, import, accounting, authority, and evidence boundaries.

## 3. Clause-linked frozen invariants

The following are explanatory groupings of existing obligations, not new gates or classifications.

### 3.1 Complete independent plan validation before authoritative emission

The plan validator must establish every plan property required by the composite directly from the approved original MIR and controlling inputs. Necessary-only structural approval, a plan hash, a matching emitter table, or producer/checker agreement cannot certify omitted operations, geometry, events, stopped prefixes, generations, holds, liveness, maxima, association, canonical coloring, or fixed-row fit.

Only a completely validated plan may authoritatively reach either emitter. Diagnostic plan production and partial checking may occur earlier while authority gates remain closed.

For an implementation that uses a private `ValidatedPlan` seal, possession of that seal must not claim more than the completed validation establishes. A named seal type, a unique constructor, a staged internal constructor, or another private representation is an implementation choice. Private post-validation construction is consistent with this record; an incomplete check minting a complete seal is not.

### 3.2 Authenticated materials, build, and selected product

The selected build and its evidence must satisfy and record the exact source, original approval, root, target, backend, mode, plan, schedule, tools, material projection, startup and dependency inputs, arguments, environment, output locations, profile, link procedure, and product associations required by the composite.

For each prescribed matrix build/profile, the unique selected Cargo `compiler-artifact` for `bed_038_n1` identifies the executable subjected to the required inspection and matrix command. This is not a claim that one physical inode is globally unique, that every invocation receives a new executable, or that byte-identical products are automatically interchangeable.

Matching bytes alone do not manufacture missing build provenance or original approval. Conversely, a different physical object is not an independent rejection reason when the complete frozen selected-product and provenance requirements are otherwise lawfully established. Whether a copy, transfer, reopen, or reacquisition can retain those requirements depends on the exact frozen command and evidence contract; this record authorizes none.

### 3.3 Required executable and responsibility inspection

The exact selected executable and its contributing generated source, IR/C, objects, disassembly, symbols, relocations, imports, helpers, startup material, loader, and required DSOs receive the inspection required by FC6.

The proof includes both:

1. the coordinator's fixed direct-call relationship to the selected generated native entry; and
2. the generated entry's permitted downward call, relocation, import, and responsibility closure.

Symbol presence alone is insufficient. Inspection of the final ELF alone does not erase required generated-source, object, disassembly, material, provider, version, or relocation obligations.

The native routes are statically linked into the prescribed test executable, but that executable is not a wholly static Linux binary: the frozen loader and exactly permitted DSOs remain part of the contract. This record does not authorize runtime loading or lookup of a Bowfin native product, an interpreter callback, a plan dispatcher, or another substitute for the fixed direct calls.

### 3.4 Actual selected-product invocation and admission

The prescribed matrix command must execute the selected authenticated product under the environment, loader, and assumptions fixed by the composite. Inspection of product A cannot authorize an unrelated product B. The selected native entry, invocation, carrier, original approval, plan, product, root, limits, schedule, and fresh process-local nonzero nonce must remain correctly associated as required by the composite.

The frozen admission, nonce, association, carrier, and evidence rules apply at their respective specified boundaries; this record does not move any check. No preparation object may enable semantic native work that bypasses the applicable pre-entry gates, and actual execution evidence remains subject to its required post-execution validation before view construction or comparison. No diagnostic or synthetic data may substitute for the actual invocation and locally validated evidence.

The internal enforcement design is not selected here. A private one-shot owner may implement these obligations, but its type traits, decomposition, preparation timing, reuse of immutable build metadata, and internal account representation are implementation choices. An inert preparation object may exist before final admission if it cannot enable entry. A persistent coordinator may serve multiple lawful invocations if each invocation receives its required fresh nonce, associations, carrier/evidence separation, and validation.

### 3.5 Actual evidence before semantic comparison

Generated execution owns the semantic carrier writes and sealing required by the composite. Local association, event, slot, header, terminal, census, and replay validation must succeed to the specified extent before a native validated view can participate in comparison.

Missing, malformed, exhausted, crossed, fabricated, or otherwise invalid evidence cannot become Equal or Different merely because another engine agrees. This record adds no failure tag or comparison classification.

## 4. Required dependencies, not a new total order

The frozen prerequisites are conjunctive, but this record is not a new total execution order. Only dependency orders expressly required by the composite are mandatory:

1. complete plan validation before authoritative emission;
2. required material authentication before building;
3. the selected no-run build and required inspection before matrix execution;
4. required nonce issuance and pre-entry admission before semantic native work, with generated-entry and subsequent evidence checks remaining at their frozen boundaries; and
5. sealed actual-evidence validation before comparison.

Independent diagnostic preparation, parsers, inspectors, graph experiments, custody experiments, private type scaffolding, and other non-authorizing work may occur in any compatible order while authority gates remain closed. Valid reusable build/product facts need not be recreated for each invocation unless the frozen contract specifically requires it.

The reported `bed_038_n1_matrix` failure because complete engine-neutral validated views are unavailable is consistent with a deliberately closed readiness gate. It remains a failed test, not a passing suite, native evidence, acceptance, or proof that surrounding source has no regression. The gate must not be weakened merely to obtain green output.

## 5. Optional implementation and hardening techniques

The following techniques may support the frozen invariants on a compatible hosted target. They are not universal N1 requirements and are not selected by this record.

| Technique | Possible purpose | Boundary |
|---|---|---|
| Retained executable descriptor or object handle | Reduce pathname substitution risk or carry an object association across stages | Does not alone prove build provenance, immutable history, mapped code, or invocation association; must remain compatible with the frozen command and product selection. |
| Write exclusion or mutation-history evidence | Prevent or detect changes during a stated interval | Any claim must state its threat model, interval, mechanism, and evidence; a digest or descriptor alone does not prove continuous exclusion or history. |
| Runtime mapped-image correlation | Add evidence connecting a running image or selected entry with an authenticated product | Not prescribed as a separate FC6 operation; must not widen the loader, mapping, import, accounting, or execution model. |
| Private one-shot capability or owner type | Make bypass, reuse, or crossed authority difficult to express | Type traits and construction timing do not themselves prove correct admission, product association, accounting, or single use. |

If one of these techniques is chosen, defects in its own claimed association remain defects. For example, an implementation relying on a descriptor cannot treat a reused descriptor number as the retained object. That conditional control does not make descriptor custody mandatory for implementations that do not use it.

The build must produce the completed artifact required for final inspection, and the evidence must continue to concern the actual selected product at use. A digest or descriptor retains its expressly assigned evidentiary role but does not alone prove the complete plan/build/invocation association. Any stronger mutation-history or write-exclusion claim must state its threat model, interval, mechanism, and evidence.

The existing authenticated build/run workflow and test coordinator may retain additional custody evidence as an implementation technique. This record creates no new process, daemon, service, privilege, namespace, or execution topology.

The prescribed matrix command must execute the selected authenticated product. The composite does not independently require descriptor-bound launch, retained-inode identity, or another physical-object mechanism.

The actual selected entry and invocation must remain correctly associated with the authenticated product. The composite does not prescribe a separate runtime mapping-correlation operation; such a technique may be proposed if compatible with the frozen execution, material, import, accounting, and authority boundaries.

## 6. The original eight-link chain as a reviewed hardening sketch

The original candidate proposed this sequence:

1. The independently validated plan feeds the authenticated emit/build process.
2. The trusted build produces one exact final ELF executable.
3. All writers settle and continuous write exclusion is established.
4. A trusted supervisor retains that exact executable object.
5. ELF inspection proves the generated entry and rooted direct-call relationship.
6. The kernel starts that exact retained executable object.
7. The process correlates its mapped executable image with the retained object.
8. Only then does the coordinator mint its one-shot native owner and directly call the proven entry.

This sequence is preserved for continuity as an **adversarially reviewed, non-normative hosted-Linux hardening sketch — not adopted and not a derived N1 requirement**. It is not the frozen admission contract and not a total required order.

Its faithful classification is:

| Link | Status under the frozen composite |
|---|---|
| 1 | Required in substance: complete independent validation precedes authoritative emission. A named seal representation is optional. |
| 2 | Required in substance for the selected matrix executable per build/profile; not global physical-object uniqueness or a fresh executable per invocation. |
| 3 | Completed output before final inspection is derived. Continuous exclusion/history proof is an optional stronger technique unless separately adopted. |
| 4 | Existing build/run workflow and coordinator roles are real. Mandatory retention of one OS object or a new supervisor topology is optional/new, not derived. |
| 5 | Required, including caller-to-entry association and the generated entry's downward responsibility/import closure. |
| 6 | Actual execution of the selected authenticated product is required. Identity with a separately retained physical object or descriptor-bound launch is optional/new. |
| 7 | Correct actual product/entry association is required. A separate process-side mapping-attestation operation is optional/new. |
| 8 | Admission, fresh nonce, association, fixed direct call, and actual evidence are required. A particular one-shot type and minting schedule are optional/new. |

Retaining any optional element as a mandatory Stage N1 obligation would require an explicit normative decision defining its threat model, identity relation, interval, mechanisms, accounting, failure behavior, and compatibility with the frozen execution contract.

## 7. Target-neutral obligation and target-specific evidence profiles

This section distinguishes the evidence obligation from any one operating-system mechanism. For current Stage N1, the frozen x86-64 Linux composite alone governs. This section adds no target, backend, qualification, evidence result, threat guarantee, or exception to that composite.

“Target profile” here refers to the claimed execution-and-evidence boundary. It is not merely a compiler-host identity, output triple, or generated-program resource budget. Build/evidence infrastructure and the generated execution target may have different capabilities and responsibilities.

A proposed review principle for future target profiles is:

> Evidence must truthfully associate the authenticated selected product, the entry established for that product, the actual invocation, and the resulting execution evidence within the target profile's expressly declared threat assumptions and trusted computing base. This record neither presumes nor selects any particular filesystem, descriptor, process, loader, virtual-memory, mapping-introspection, cryptographic, or capability-type mechanism as universal.

This is non-normative guidance for future target design. For continuity and review, three useful—non-exhaustive—relationships are:

| Profile relationship | Meaning |
|---|---|
| Equivalent evidence mechanism | Different machinery is independently shown to establish the same expressly defined obligations and assurance claim as the profile or mechanism to which it is compared. Equivalence is a reviewed project conclusion, not a self-declared property, and is not an exception merely because the mechanism differs. |
| Narrower assurance profile | The target establishes a deliberately smaller claim under explicit exclusions or stronger environmental assumptions. It must not inherit labels or assurances belonging to a stronger profile. |
| Experimental or unqualified target | The target cannot yet establish the association required for a claimed evidence level. It may remain useful for development, but it must not present its results as qualified native, production, or equivalent evidence. |

Lower capability does not itself imply lower assurance. Under a separately reviewed profile and declared threat model, a fixed image in protected ROM or flash may make pathname replacement, descriptor reuse, or dynamic mapping inapplicable rather than unproved. Conversely, a feature-rich hosted system does not establish trustworthy product-to-invocation association merely because it offers more inspection APIs.

Illustrative mechanisms may include:

- hosted ELF build, inspection, selection, and controlled invocation evidence;
- fixed firmware image, linker manifest, entry address, and declared boot/flash assumptions;
- RTOS module identity, loader association, and selected export binding; or
- WebAssembly module identity, engine instantiation, and exported-entry association.

These examples are neither accepted profiles nor declarations that the listed facts are sufficient. Each target and evidence level requires its own authorized definition and review.

A future profile should state, as applicable:

1. authenticated product identity and provenance;
2. how the invoked entry is bound to that product;
3. the launch, load, boot, or activation boundary;
4. mutation, replacement, remapping, and replay assumptions;
5. invocation and resulting-evidence association;
6. the trusted computing base and excluded threats;
7. the consequences of incomplete or failed association; and
8. which costs are borne by build/evidence infrastructure versus generated target code.

A target-specific proof mechanism does not change Bowfin source-language semantics. It does not automatically qualify another target, weaken an existing profile, or amend the current N1 contract. Any new target profile or assurance claim remains subject to its own project decision and evidence process.

Build- and evidence-runner mechanisms should not silently become generated-program runtime requirements. Heavy host-side verification need not impose target runtime cost when the accepted profile explicitly assigns that work to the host and preserves the stated assurance boundary.

## 8. Observation, association, custody, and authority terminology

These terms are useful but do not exhaust the implementation designs allowed by FC6:

- **Observation:** a fact measured at a stated point.
- **Association:** evidence that facts concern the entities required by the composite.
- **Custody:** a bounded claim that specified replacement, mutation, reuse, or crossing was excluded or detected over a stated interval.
- **Authority:** the ability to cause an otherwise gated operation.

A snapshot may contribute evidence without proving continuous history. Original bytes restored after a mutation do not themselves prove lawful history. Conversely, a diagnostic experiment that changes metadata does not prove that an existing metadata recheck accepts the restored file.

Authenticated hashes, product records, descriptors, paths, metadata, symbols, and pointers retain their assigned narrower evidentiary roles. Saying one is insufficient alone for the complete relationship does not make it irrelevant.

## 9. Dated implementation boundary that motivated this record

At the reported 2026-11-12 `217ea8887665c3ef5145c1c7d0473cc8beeaa0be` source checkpoint:

- the necessary-only structural approval did not establish a complete plan;
- no truthful construction path for the complete validated-plan boundary had been demonstrated;
- authenticated product/direct-entry/actual-invocation association and complete native views remained unavailable;
- diagnostic retained-file and change/restore probes did not establish an authenticated bypass or a universal custody requirement;
- production admission remained closed; and
- the matrix readiness gate failed because complete engine-neutral validated views were unavailable.

These are dated implementation observations and reported evidence boundaries, not permanent architecture, source acceptance, or proof that a complete implementation is impossible.

## 10. Explicit non-changes

This continuity record does not change:

- Bowfin source-language semantics;
- the admitted N1 roots, MIR variants, or closures;
- the validated-plan schema, product schema, or carrier ABI;
- slots, rows, events, counts, capacity, or event ordering;
- logical accounting geometry, limits, neighbors, or shared-budget requirements;
- FA7-R1 holds, FA8 vacancy ordering, coloring, or the sixteen-row proof obligation;
- backend, target, ABI, toolchain, material closure, link, loader, import, or static-call policy;
- comparison classifications or peer-independent local validation;
- the authorized source branch, PR, workflow, merge, production, or stage boundaries; or
- the distinction among analytical expectation, diagnostic results, GitHub Actions evidence, native evidence, and owner acceptance.

## 11. Proposed durable placement after independent review and owner disposition

If independent review confirms this exact R1 and Greg later adopts it as a non-normative continuity record, the proposed placement is:

1. complete record at `docs/engineering/BED-038-NATIVE-PRODUCT-INVOCATION-CONTINUITY.md`;
2. the concise faithful core principle and a link in `docs/ai/ARCHITECTURE.md`;
3. rebuilt `docs/ai/BOWFIN_AI_CONTEXT_BUNDLE.md`; and
4. an exact publication receipt retaining UTF-8 byte count, SHA-256, final-newline state, and later merged commit/tree/blob identities.

The full BED-specific record should not be duplicated in the Miles charter. The compiler repository remains canonical for Bowfin architecture and implementation context.

Independent review, owner adoption, documentation preparation, PR creation, and human merge are separate steps. This candidate authorizes none of them by itself.
