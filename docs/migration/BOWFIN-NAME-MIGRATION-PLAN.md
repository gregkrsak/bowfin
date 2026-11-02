# Bowfin Name Migration Plan — M0 Draft 2, Amendment 3 — Proposed

**Status:** Proposed contract-only simplification. Amendment 2 remains the accepted contract until Greg explicitly accepts and a human merges this amendment into `bowfin-migration-staging`. Section 15 replaces the formal M0.1 prerequisite with phase-local verification; it does not declare M0.1 passed or authorize M1 implementation.

**Amendment 3 prepared:** 2026-10-31

**Amendment 3 baseline:** staging `609822762abd46c13ecd6187ccc8510695e52b2f`, tree `ebaf5e4b7b6a7a09acc693db52e3cdc5e0ec2239`; prior contract blob `03cd2c5b584a996ebfb873491f0d2888c984cfb9`.

**Reading rule:** Sections 1–14 below retain Amendment 2's text and historical preparation dates. Section 15 states the proposed precedence and replacement gates; until adoption, Section 13 remains controlling.

**Prepared:** 2026-10-23  
**Draft 2 owner decisions recorded:** 2026-10-23  
**Amendment 2 Draft 1 prepared:** 2026-10-24  
**Amendment 2 Draft 2 repairs:** 2026-10-24  
**Repository baseline:** `gregkrsak/klyxr` at `prod` commit `3c0a540df7d5ad75510db4ecad80e27fb4d0903a`, tree `928accaa6c71068a6bce273f73f8c152b4c4d348`  
**Migration integration branch:** `bowfin-migration-staging`

## 1. Objective and controlling principle

Rename the live project from **Klyxr** to **Bowfin** without changing language behavior, weakening assurance, manufacturing authority, corrupting historical evidence, or silently breaking promised consumers.

The migration is an identity change, not a language-feature release. Every changed identity must have a typed, injective correspondence; every compatibility path must name its consumer and authority boundary; and every phase must be independently reviewable and reversible within its stated boundary.

The controlling question remains:

> What new power does this create, where is that power bounded, and can we explain the answer plainly?

For this migration, the required answer is: **no new execution, approval, replay, custody, or resource authority is created by changing or translating a name.**

## 2. Greg’s resolved owner decisions

The following choices bind Draft 2 and later migration work:

1. **Bowfin** and `bowfin` are the final adopted live project names.
2. `KED-###` becomes `BED-###` for live engineering-decision identities. `KD-###` is a distinct namespace and becomes `BD-###` for live language-decision identities. The two families must never be conflated by their numeric suffixes.
3. Existing decision numbers, status, scope, issue association, acceptance history, and original freeze evidence are preserved one-to-one. The next new engineering decision is `BED-037`; the next new language decision is `BD-045` unless a pre-cutover decision legitimately advances either sequence.
4. Project control identities such as `K35-P01` and `K35-I01` become enumerated `B35-P01` and `B35-I01` identities. No generic `K` replacement is permitted.
5. `DP-###`, `OQ-###`, commit hashes, tree hashes, blob hashes, historical payloads, recorded commands, verbatim reports, and unrelated third-party identifiers remain unchanged.
6. `.bow` is the selected canonical Bowfin source extension **conditional on passing the bounded extension gate in this contract**. Until that gate and M3 are accepted, `.klx` remains canonical in production.
7. `bowfin-lang.com` is the primary Bowfin domain. The acquired `.org`, `.net`, `.info`, and `.online` names are secondary redirect candidates. `klyxr.com` and `klyxr.online` remain controlled compatibility entry points.
8. The planned repository destination is `gregkrsak/bowfin`. A future organization transfer requires a separate owner decision.
9. `.klx` input and the `klyxr` command alias remain supported until a future accepted BED explicitly removes them. New documentation and generated output use Bowfin identities after their phase gates.
10. Existing canonical KED and KD files may move to BED and BD filenames and live headings through the manifest. Historical evidence fields retain their original tokens.
11. The compiler repository and the separate AI-governance repository are distinct authority surfaces. Governance migration requires its own explicitly authorized branch and pull request; compiler-repository work cannot silently change governance authority.
13. Greg remains the sole merge authority during the migration. Base-target enforcement is therefore an explicit, recorded owner merge gate rather than a production-resident workflow or repository-settings control. If merge authority broadens or automated merging is introduced, migration work stops until this control is re-evaluated.

These decisions select the destination. They do not authorize M1 or later implementation.

## 3. Branch and integration policy

`bowfin-migration-staging` is the sole integration target for M0–M7. No phase branch may target `prod`.

| Phase branch | Identity boundary |
| --- | --- |
| `bowfin-m0-migration-contract` | Contract, owner decisions, mapping and gate definitions |
| `bowfin-m1-machine-compatibility` | Machine identifiers and historical compatibility |
| `bowfin-m2-toolchain-identity` | Rust package, crate, binary, and control identities |
| `bowfin-m3-source-extension` | Source extension, examples, and tooling hooks |
| `bowfin-m4-docs-governance` | Decisions, documentation, AI context, and coordinated governance work |
| `bowfin-m5-visual-identity` | Logo, banner, diagrams, and supporting visual identity |
| `bowfin-m6-public-cutover` | External-cutover preparation only; no live switch before production acceptance |
| `bowfin-m7-stabilization` | Exact-candidate stabilization, evidence, and release recommendation |

All eight branches were initially parked at the baseline above. Before work begins on M1–M7, the phase branch must be refreshed from the current accepted staging head. Its PR must prove:

- its base is the exact accepted staging commit and tree;
- its head descends from that base;
- its target is `bowfin-migration-staging`, never `prod`;
- the resulting integration tree is recorded and independently checked;
- no accepted staging work was replaced by a similar-looking cherry-pick; and
- any intervening `prod` change was incorporated into staging and re-reviewed before dependent work continued.

Base-target protection is an explicit owner merge gate. Every M0–M7 PR must record its head and base branch names and SHAs. The implementer and independent reviewer must verify that the base is `bowfin-migration-staging`; Greg must verify the displayed target immediately before merging. Any mismatch blocks acceptance and merge.

Because Greg is the sole merge authority during this migration, no workflow on `prod` or repository-settings change is required for this control. If merge authority broadens, automated merging is introduced, or this assumption becomes false, migration work stops until an independently reviewed automated control is established.

The only permitted staging-to-`prod` PR is the separately identified final production-candidate PR, and only Greg may merge it.

## 4. Namespace-qualified identity contract

### 4.1 Required mapping families

| Kind | Legacy identity | Canonical identity | Policy |
| --- | --- | --- | --- |
| Display brand | `Klyxr` | `Bowfin` | Replace on live surfaces |
| Lowercase brand | `klyxr` | `bowfin` | Replace on live surfaces |
| Uppercase brand | `KLYXR` | `BOWFIN` | Replace on live surfaces |
| Engineering decision | `KED-###` | `BED-###` | Typed one-to-one map |
| Language decision | `KD-###` | `BD-###` | Separate typed one-to-one map |
| Project control | `K##-*` | `B##-*` | Enumerated one-to-one map; never inferred globally |
| Test identity | Project-specific legacy test name | Enumerated Bowfin test name | One predecessor maps to exactly one successor unless an accepted report explains otherwise |
| CLI executable | `klyxr` | `bowfin` | Canonical command plus retained compatibility alias |
| Rust package | `klyxr-compiler` | `bowfin-compiler` | Intentional Cargo package-name break; repository is not registry-published |
| Rust crate path | `klyxr_compiler` | `bowfin_compiler` | Update source, tests, doctests, and harnesses together |
| Source suffix | `.klx` | `.bow` after gate | Legacy input retained; canonical output becomes `.bow` |
| Markdown fence | `klyxr` | `bowfin` | Documentation/tooling identity only |
| Configuration name | `klyxr.toml` | `bowfin.toml` | Reserved names only; no configuration loader currently exists |
| Machine namespace | `klyxr.*` | `bowfin.*` | Versioned, field-specific mapping; never treated as authority |
| Compiler identity | `klyxr-compiler/<version>` | `bowfin-compiler/<version>` | Permitted provenance change with explicit correspondence |
| Environment variable | Project-prefixed `K...` names, including `K34_EVIDENCE_DIR` | Individually selected `B...` names | Enumerate by exact spelling and consumer |
| Filename/script stem | `KLYXR`, `klyxr`, `KED`, `ked`, `KD`, or `kd` forms | Typed Bowfin/BED/BD form | Rename only after occurrence classification |
| AI context | `KLYXR_CONTEXT.md`, `KLYXR_AI_CONTEXT_BUNDLE.md` | `BOWFIN_CONTEXT.md`, `BOWFIN_AI_CONTEXT_BUNDLE.md` | Rename canonical inputs, generator, and generated output atomically |
| Repository | `gregkrsak/klyxr` | `gregkrsak/bowfin` | External cutover only |
| Primary domain | `klyxr.com` | `bowfin-lang.com` | External cutover only; old domain retained |

The `KLX` monogram and every spelling or case variant found in the repository or external inventory must receive an explicit row before implementation. `DP`, `OQ`, unrelated variables, units, prose letters, third-party identifiers, and externally defined names remain unchanged unless separately enumerated and accepted.

### 4.2 Manifest schema and validity

Before M1, M0 must produce or require a machine-readable manifest whose every entry records:

- namespace kind;
- original identity and source location;
- canonical identity or intentional retention;
- occurrence role: live, compatibility, or historical;
- owning migration phase;
- consumer and producer, when machine-readable;
- support/removal policy;
- validation or test identity; and
- historical evidence relationship, when applicable.

Mappings must be injective within each namespace. Duplicate keys, duplicate canonical values that would conflate distinct identities, untyped numeric-only correspondence, unknown kinds, and ambiguous occurrence roles are errors.

A rename preserves the decision or control’s status, scope, issue association, and acceptance history. It does not create a new decision, repeat a freeze, or rewrite an old evidentiary claim.

### 4.3 Field-level occurrence roles

Classification applies to individual structured fields and occurrences, not merely to whole files or directories:

1. **Live** fields become Bowfin identities.
2. **Compatibility** fields retain or accept a legacy representation according to the per-surface matrix.
3. **Historical** fields remain byte-truthful when they record prior hashes, commands, outputs, control identities, reports, or measurements.

A JSON document may legitimately contain all three roles. Verifiers must not become green by rewriting both a historical expectation and the observation it was meant to check.

Original historical payloads and digests remain untouched. A new live adapter or manifest may record a correspondence to them, but may not recompute an old claim under new spelling and present it as the original evidence.

## 5. Inventory and completeness gate

The baseline inventory found:

- 276 tracked files;
- 87 text files and 883 occurrences containing `Klyxr`, `klyxr`, or `KLYXR`;
- 47 files and 713 occurrences containing KED forms;
- 17 files and 122 occurrences containing known K-control forms;
- 75 `.klx` files;
- 49 `klyxr` Markdown fences;
- 14 verification/build scripts containing known legacy literals;
- 8 old-domain references; and
- known `klyxr.*` and `klyxr-compiler/<version>` machine identities.

These counts are a lower bound, not completion evidence. Before M1, the exhaustive inventory must additionally cover:

- all `KD`/`kd`, `KLX`, environment-variable, filename, script, test, package, crate, binary, configuration, and case-variant families;
- structured-field roles inside evidence and predecessor records;
- generated context inputs and outputs;
- repository settings, workflows, templates, releases, tags, badges, raw links, submodules, Pages/custom domains, and hosted-Action consumers that actually exist;
- the separate governance repository and its canonical charter/instructions;
- domains, package registries, editor/tooling identifiers, app-store or social identities that actually exist; and
- raster/vector text by manual inspection or OCR.

Nonexistent integrations are marked **not applicable**. The migration does not create a configuration system, package release, editor architecture, website, or public replay format merely to satisfy an inventory checklist.

The inventory gate passes only when an independently reviewed occurrence manifest accounts for every discovered legacy occurrence or records a bounded scanner limitation. Historical allowlists may contain exact paths, fields, and reasons; broad directory or wildcard exemptions are prohibited.

## 6. Per-surface compatibility and authority matrix

| Surface | Existing producer/consumer | Accepted input | Canonical output | Conflict or rejection behavior | Authority boundary and support |
| --- | --- | --- | --- | --- | --- |
| Source path | CLI opens the explicitly supplied path; no sibling discovery exists | `.klx` and, after M3, `.bow` | Docs/generated examples use `.bow` | No same-stem conflict exists because no discovery occurs; future discovery requires a separate BED | File suffix grants no authority; `.klx` remains until a future BED removes it |
| CLI command | Installed/build binary invoked by user or tests | `bowfin`; retained `klyxr` alias after M2 | Help/version identify Bowfin; legacy alias help/version identifies itself as compatibility | Normal execution adds no unsolicited stdout or stderr banner; arguments, exit status, and result streams remain equivalent | Alias delegates to the same implementation and remains until a future BED removes it |
| Cargo package | Workspace Cargo manifests; currently `publish = false` | Repository manifests are updated to `bowfin-compiler` | `bowfin-compiler` | Direct consumers requesting package `klyxr-compiler` must update; a CLI shim does not claim Cargo compatibility | No registry publication is implied or authorized |
| Rust crate | Repository source, doctests, and tests | `bowfin_compiler` after M2 | `bowfin_compiler` | Legacy crate-path references fail and receive migration documentation | Internal source identity only; no authority change |
| Configuration filename | No configuration loader currently exists | None | None | `klyxr.toml` and `bowfin.toml` are reserved spellings, not accepted runtime inputs | Migration must not invent configuration loading or discovery |
| Machine/provenance identifiers | Trusted catalogs, profiles, targets, compiler IDs, assumptions, audit and evidence records | Exact legacy values only in declared archival/compatibility readers; exact Bowfin values in current producers | New records use accepted versioned `bowfin.*` identities | Unknown values reject; mixed values reject unless a field-level manifest entry and existing validation prove one coherent record | Translation is descriptive only and cannot establish approval, grants, custody, replay identity, schema validity, or execution permission |
| Approval-bearing correspondence | Existing `Contract::same_approval` and associated validators | The same original opaque approval required by the governing implementation | No synthetic approval output | Independently compiled equality or spelling equality is insufficient and rejects | The migration must not weaken same-original-approval semantics |
| Private addressed replay | Existing bounded conformance mechanism | Only its already-authorized private inputs | Existing private results with mapped presentation where accepted | No public serialized replay import is created | Reading or translating a name cannot import resource authority or recover executable state |
| Historical evidence | Existing evidence/verifier consumers | Original historical bytes plus explicit mapping metadata where needed | New evidence records Bowfin identity and its predecessor relationship | Digest mismatch, unknown mapping, duplicate mapping, or role ambiguity rejects | Historical inspection is not executable admission |
| Decisions and controls | Human and script consumers | Historical legacy identities and live mapped identities in their declared roles | Live references use BED/BD/B identities | Numeric suffix alone is never sufficient; cross-namespace ambiguity rejects | Mapping preserves status/history and grants no new authority |
| Repository/domain URLs | Git clients, badges, raw links, Pages, hosted Actions, web users where applicable | Tested old redirects and new canonical URLs after cutover | New URLs | Each facility is tested separately; successful Git redirect does not prove Pages, raw links, or hosted Actions | Routing does not authorize code or release state |

Global prohibition: **name translation cannot manufacture, recover, merge, renew, or substitute executable approval, grants, operation association, replay identity, custody, schema validity, or resource authority.** No persisted executable authority, public replay facility, source-discovery system, or configuration facility enters through this migration.

## 7. Permitted observable-difference relation

“Rename-only” does not mean byte-identical presentation. The migration compares old and new behavior through the following reviewed relation.

### 7.1 Must remain unchanged

- language acceptance except for the separately gated source filename surface;
- nominal typing, ownership, borrowing, movement, and resource-bearingness;
- grants, actual resource responsibility, and approval requirements;
- effect and cleanup order;
- execution results and terminal classification;
- non-resumption behavior;
- logical execution charges and bounded-execution semantics;
- diagnostic class, causal meaning, and source span except for explicitly mapped path or source-text displacement; and
- predecessor coverage, except for enumerated one-to-one identity changes.

### 7.2 Permitted only when enumerated

- displayed project, package, crate, command, decision, control, test, path, URL, provenance, and machine-namespace spelling;
- byte length of rendered identity-bearing text;
- source-origin offsets caused solely by reviewed edits to comments or branding text;
- help/version output for the canonical command and retained alias; and
- actual-byte boundary results that differ solely because the honestly rendered Bowfin spelling has a different byte length.

Actual-byte limits remain honestly enforced. The implementation may neither undercharge Bowfin bytes nor silently raise a production limit to force old and new outcomes to match. For every affected boundary, evidence records the legacy sufficient/insufficient neighbors and the mapped Bowfin neighbors separately, preserving original measurements.

### 7.3 Forbidden normalization

No global normalization may erase or ignore authority fields, custody events, failures, byte-limit behavior, unknown differences, or unmapped identifiers. Migration differential evidence uses an explicit migration relation and independent expected results; it must not weaken same-original-approval by comparing independently compiled old and new programs as though spelling equality established approval identity.

## 8. Phase plan and gates

### M0 — Migration contract

Deliver this contract, the namespace-qualified manifest requirements, the occurrence-inventory gate, the compatibility matrix, the permitted-difference relation, and the release state machine. Sarah or another independent reviewer recommends disposition; Greg alone authorizes freeze.

**Exit:** all major contract findings resolved; owner decisions recorded; complete-manifest work is either included in M0 or is a mandatory independently reviewed precondition that blocks M1.

### M1 — Machine compatibility

Implement exact old/new handling for the machine and evidence surfaces that actually exist. Add legacy fixtures, field-role validation, mapping injectivity checks, conflict rejection, and tests proving that translation cannot establish authority.

**Must not include:** CLI, source-file, documentation, branding, domain, or repository renames.

**Exit:** required historical inputs remain readable without changed meaning; new machine output is unambiguously Bowfin; replay, digest, approval, and custody consequences are documented and tested.

### M2 — Toolchain identity

Rename the Rust package, crate, and canonical binary; add the retained command alias; update Cargo metadata, lockfile, imports, doctests, harness variables, help/version text, and project-control identities through the manifest. Supporting verifier changes required to keep the phase testable travel with this phase.

**Exit:** debug and release suites account for every predecessor; alias result streams do not diverge; only mapped identities change.

### M3 — Source extension and tooling surface

Complete the bounded `.bow` collision matrix, record its limitations, and—only if it passes—rename canonical examples and all exact consumers. Continue explicit `.klx` input. Update fences and existing editor/tooling hooks; do not invent absent infrastructure.

The extension matrix covers Windows, macOS, Linux, common archive/antivirus/DLP/email behavior available for testing, GitHub/GitLab, VS Code, Sublime Text, tree-sitter, MIME/web-server defaults, and known format/tool collisions. It does not claim universal clearance.

**Exit:** canonical sources use `.bow`; promised legacy input works; the full corpus parses, verifies, and executes under independently expected results.

### M4 — Decisions, documentation, and governance

Rename live KED→BED and KD→BD files and references through the typed map. Update AI context sources, generator, generated bundle, README, repository structure, documentation, Sentinel 0 guide, templates, code fences, scripts, and workflows. Preserve exact historical fields. The new checked-in context bundle must reproduce byte-for-byte from the new canonical inputs; it is not expected to equal the old bundle.

Coordinate a separately authorized governance-repository branch and PR. Record its commit/tree and accepted relationship to the compiler staging candidate. Neither repository silently confers authority on the other.

**Exit:** no unapproved live Klyxr/KED/KD/project-K identities remain in covered text surfaces; the governance dependency is explicitly reconciled.

### M5 — Visual identity

Replace live wordmarks, banners, and supporting visual identity assets.


**Exit:** the visible project identity is coherent and contains no live Klyxr branding.

### M6 — External-cutover preparation

Prepare—but do not execute—the domain, repository, redirect, badge, raw-link, Pages/custom-domain, hosted-Action, clone, release, and announcement runbook. Inventory only facilities that exist. Prove new domain destinations in a noncanonical or otherwise non-disruptive state where practical.

**Exit:** every external action has a target, prerequisite, verification, and recovery step. Public routing and repository identity remain unchanged.

### M7 — Stabilization and exact candidate

Run the full debug/release/format/docs/static-verification matrix, predecessor reconciliation, denylist/allowlist scan, legacy-input checks, clean clone/install, Sentinel 0, context generation, image inspection, and external runbook rehearsal on an exact staging head/tree. Publish the final migration ledger and independent recommendation.

**Exit:** Greg may accept or reject the exact production candidate. Review recommends; it does not authorize.

## 9. Release and cutover state machine

| State | Required transition | Prohibited before transition completes |
| --- | --- | --- |
| R0 — M0 frozen | Independent recommendation plus Greg’s explicit freeze | M1 implementation |
| R1 — Phases integrated | M1–M6 individually reviewed and accepted into staging from exact predecessors | Direct phase merge to `prod`; live external cutover |
| R2 — Candidate fixed | M7 records exact staging commit/tree, mapping version, tests, exceptions, artwork evidence, and runbook | Candidate drift without renewed evidence |
| R3 — Recommended | Independent review recommends the exact R2 candidate | Treating review as owner authority |
| R4 — Production accepted | Greg explicitly accepts; a human merges the sole staging-to-`prod` PR | Repository rename, canonical DNS switch, public Bowfin release claim |
| R5 — Production verified | Exact merged `prod` commit/tree passes required post-merge checks | External cutover |
| R6 — Cutover authorized | Greg separately authorizes the external runbook | Any external write not named in that authorization |
| R7 — Public cutover | Canonical Bowfin routing, repository rename, redirects, metadata, and announcement are executed in recorded order | Declaring completion before clean-environment verification |
| R8 — Post-cutover verified | Old and new entry points, CI, issues/PRs, releases/tags, raw links, badges, domains, and applicable consumers pass | Removal of compatibility surfaces without a future BED |

An alias-first public preview is allowed only by a separate owner decision that plainly states production is still Klyxr and makes no premature Bowfin-tool availability claim.

## 10. Verification and evidence requirements

Every phase report records:

- accepted staging base commit/tree;
- reviewed phase head/tree;
- mapping-manifest version and digest;
- exact identity boundary and field roles changed;
- historical exceptions;
- tests, workflows, and independent expected results;
- resulting staging integration tree;
- content rollback and external-recovery procedures; and
- confirmation that no unrelated feature or authority work occurred.

Completion additionally requires:

- all predecessor identities preserved or mapped one-to-one with no unexplained migration;
- zero unapproved live legacy identities under case-insensitive and typed scans;
- no broad historical allowlist wildcard;
- all promised legacy inputs verified;
- actual-byte limits tested honestly at mapped neighbors;
- context generation reproducible from canonical inputs;
- Sentinel 0 behavior preserved under the permitted-difference relation;
- clean clone/install and all repository workflows green on exact head;
- raster and vector branding manually inspected; and
- external facilities tested individually rather than inferred from Git redirect behavior.

## 11. Rollback and recovery

Repository-content rollback and external-state recovery are different operations.

Before R4, an accepted phase may be reverted from staging using its recorded integration boundary, followed by dependent-phase invalidation and re-review. A revert cannot be described as restoring an external or emitted state that never changed.

After R4 but before R7, production content may be forward-repaired or reverted under a separately reviewed plan. No public cutover has yet occurred.

After R7:

- do not recreate or reuse the old GitHub repository name;
- retain old domains and required redirects;
- prefer a forward repair over repeated repository renames;
- preserve both names’ routing and historical access during a critical-release disablement;
- separately recover DNS, repository metadata, badges, hosted services, cached pages, announcements, and emitted artifacts where applicable; and
- never claim that reverting Git content retracts an installed binary, downloaded artifact, cached page, published package, or previously issued runtime authority.

No package publication is part of this plan without separate authorization.

## 12. Point-in-time external findings

- Greg acquired `bowfin-lang.com`, `.org`, `.net`, `.info`, and `.online` on 2026-10-23. Registrar hardening and DNS work remain operational tasks, not evidence of cutover.
- `gregkrsak/bowfin` returned not found during the preliminary check; that is not a reservation guarantee.
- Other Bowfin-named repositories exist, but no programming language/compiler collision was identified in the preliminary search.
- The unscoped npm package `bowfin` exists at version `0.0.0`; future JavaScript tooling uses a controlled scope if separately authorized.
- PyPI returned no exact `bowfin` project during the preliminary check.
- GitHub Linguist did not list `.bow`, `.bfn`, or `.bwfn` during the preliminary check.

These findings are not legal clearance and must be rechecked at the phase that relies upon them.

## 13. Amendment 2 — Replacement M0.1 evidence architecture

### 13.1 Status, scope, and precedence

This amendment replaces the rejected bulk-artifact approach attempted in closed, unmerged PR #95. The archived PR #95 payload in gregkrsak/bowfin-migration-ledger remains an unaccepted historical snapshot. It is not a protocol, current evidence, an implementation source, or authority for later work.

The replacement architecture is a two-repository, three-layer Rule-to-Occurrence Ledger:

1. humans review compact classification sources;
2. separately reviewed deterministic tooling expands those sources into occurrence records and independently audits completeness; and
3. generated evidence is retained only in gregkrsak/bowfin-migration-ledger.

The compiler repository may contain this contract and, if separately accepted, a minimal identity or acceptance anchor. It must not acquire the generated occurrence corpus, duplicated bulk allowlists or denylists, archived tooling, or other multi-megabyte migration evidence.

This section controls wherever it is more specific than Sections 4, 5, 8, 9, 10, or 11. It does not relax any earlier restriction. M0.1 remains unaccepted until every applicable M0.1 acceptance and integration gate below is satisfied by exact identities and Greg records the corresponding owner decision. M1 remains independently blocked. M0.1 completion neither authorizes nor requires M1 and may remain complete indefinitely while M1 remains unauthorized.

### 13.2 Required order and separate authorizations

The work proceeds only in this order:

1. Amendment 2 is independently reviewed, explicitly accepted by Greg, and merged as a contract-only change into bowfin-migration-staging.
2. Greg separately authorizes preparation of the M0.1 v2 protocol specification and tooling in the ledger repository.
3. The complete protocol specification, tooling, schemas, fixtures, limits, and independent-audit design are reviewed and accepted before they may generate acceptance evidence.
4. Greg separately authorizes compact classification inputs and evidence generation against one exact compiler candidate and the accepted protocol/tooling revision.
5. Generated evidence is independently reviewed. Review recommendation, evidence validity, owner acceptance, integration, and next-phase authorization remain distinct records.
6. After the exact M0.1 candidate has been accepted and integrated and P verifies the actual post-merge correspondence, M0.1 may be declared complete. Greg may separately decide later whether to authorize M1; declining or postponing M1 does not invalidate M0.1 completion.

Contract acceptance is not tooling authorization. Tooling acceptance is not evidence acceptance. Evidence preservation is not semantic acceptance. Review is not merge authority. Merge is not next-phase authorization.

### 13.3 Compact classification sources and semantic review

The canonical compact inputs are versioned, machine-readable sources in the ledger repository. The initial expected set is:

- mappings.json
- classification-rules.json
- external-surfaces.json
- visual-inspection.json
- limitations.json

A later protocol specification may refine filenames without weakening these required meanings.

Mechanical totality proves only that every discovered occurrence received one classification. It does not prove that the classification is correct. Human semantic review is a first-class gate.

Every classification rule must expose, in a reviewable form:

- its selected population and selector meaning;
- exact namespace-qualified mapping or intentional retention;
- expected selected count;
- live, compatibility, or historical role;
- owning migration phase;
- human rationale;
- representative and boundary context sufficient to judge the population;
- producer and consumer where machine-readable;
- support or removal policy;
- explicit validation or test identity; and
- historical evidence relationship where applicable.

Every expanded occurrence must resolve all fields required by Section 4.2, including an exact occurrence location, original identity, canonical identity or retention, namespace, role, phase, producer, consumer, support or removal policy, validation or test identity, and historical relationship.

A whole file, directory, or population may not be declared homogeneous merely by its own rule or a small sample. The review must establish why the selected population is semantically uniform and must expose exceptions. Mapping injectivity is evaluated within each declared namespace; repeated occurrences of one identity are not duplicate canonical identities.

### 13.4 Independent occurrence discovery and audit

The generator and auditor must not define completeness by the same successful expansion path. The protocol specification must define an occurrence independently of the generator's accepted matches.

The independent auditor must:

- discover candidate occurrences without inheriting the generator's complete path list, token list, selector list, or exclusion list;
- reconcile exact canonical occurrence-key sets, not merely counts or summary digests;
- treat filenames and path components, raw-text spans, structured keys, structured values, embedded test or control identities, machine identifiers, and overlapping identities as explicitly defined occurrence-unit kinds;
- report unknown, ambiguous, overlapping, multiply classified, and unclassified candidates;
- reject broad wildcard historical exemptions; and
- retain enough context to permit human adjudication without trusting the generator's classification.

Common low-level Git-object reading, byte reading, hashing, and canonical serialization code may be shared only when the shared component and its resulting common-mode failure are disclosed. Generator and auditor mutation fixtures and expected results must be independently specified. The required adversarial suite includes omitted hidden paths, case variants, structured keys and values, equal-count/different-key substitutions, stale rules, overlap, exclusion drift, and unexpected embedded identities.

### 13.5 Acyclic identity and authority chain

The evidence chain is acyclic:

| Symbol | Meaning |
| --- | --- |
| C | One fixed compiler candidate commit and tree, including any accepted minimal compiler-side anchor |
| E | One immutable ledger evidence package identifying C and every exact dependency used to produce and audit it |
| D | Later preserved review and owner-decision records identifying the exact C and E dispositions |
| P | A later integration receipt identifying actual source and ledger merges and proving their correspondence to C, E, and D |

E cannot contain its own future commit identity. C cannot be modified after E is generated in order to embed E. If a compiler-side anchor is used, it must be fixed as part of C before evidence generation; otherwise the acceptance and integration records remain external.

Every E record must carry, at minimum:

- record_class equal to migration_evidence;
- phase and protocol version;
- source and ledger repository identities;
- C commit and tree;
- accepted staging base commit and tree;
- governing contract commit, tree, path, and blob;
- exact protocol/tooling commit, tree, and relevant blobs;
- exact compact-input commit, tree, paths, blobs, and aggregate identity;
- required external snapshot identities;
- generated output index and aggregate identities; and
- explicit non-authority declarations.

None of C, E, D, or P grants executable authority, approval, custody, replay admission, semantic validity, merge permission, or next-phase permission merely by existing or matching a name or digest.

### 13.6 Drift, ancestry, merge, and post-merge proof

There is no relevant-drift exception. Any change to C, the accepted staging base, governing contract, protocol/tooling, compact inputs, required external snapshot, or required visual snapshot invalidates the corresponding evidence binding. A new evidence package and renewed disposition are required even if generated chunks or aggregate counts appear unchanged.

Source and ledger integration PRs used by this protocol must use ordinary merge commits. Squash merges, rebases, and history replacement are prohibited for M0.1 v2 acceptance. Immediately before each merge, the recorded base, target, head, commit, and tree must be reverified. After each merge, P must record the actual merge commit, tree, parents, target branch, and correspondence to the reviewed head.

A pre-merge test-merge SHA is not post-merge proof. The integration check must also inspect introduced ancestry, not merely the final tree or diff, and must reject compiler history that added prohibited bulk evidence or tooling and later deleted it.

### 13.7 Durable review and owner-decision records

GitHub comments and chat messages are publication venues, not immutable storage. Every decisive review, owner decision, correction, revocation, and integration record required for M0.1 must be preserved in a content-addressed ledger record outside the immutable evidence package it describes.

Each preserved decision record must include:

- the complete recorded body;
- source URL and stable comment or record identifier where one exists;
- stated author, publisher, and publication context without claiming cryptographic identity;
- capture time and capture method;
- content digest; and
- exact C, E, D, or P identities to which the record applies.

A copied statement and digest do not independently authenticate authority; process corroboration and Greg's explicit owner role remain required. Corrections and revocations are append-only subsequent records. Prior records are not silently edited or rewritten.

Machine consumers must resolve separately:

- evidence-package structural validity;
- independent review disposition;
- Greg's acceptance or rejection;
- integration status; and
- authorization for the next phase.

Missing, conflicting, superseded, or unverifiable records fail closed. Latest comment wins, newest timestamp wins, and author-name string matching are forbidden decision rules. A creation-time review_status of proposed is permitted only if later resolution is represented by a separate bound record.

This durability requirement does not retroactively invalidate the already accepted archival treatment of ledger PR #2.

### 13.8 External, governance, and visual coverage

The compact inputs must include a complete surface register. Every relevant surface is classified as:

- inspected;
- demonstrably not applicable;
- explicitly limited; or
- unavailable.

Acquisition failure, access denial, absence of credentials, scanner failure, and incomplete visibility are not not-applicable findings.

External acquisition is separate from deterministic generation. Retained snapshots must be digest-bound and record origin, acquisition time, coverage, freshness, and known limitations. Evidence relying on the governance repository must bind its exact repository, commit, tree, path, and blob identities and retain offline-readable content sufficient for later audit.

Before tooling acceptance, the separately reviewed protocol specification must define a freshness-eligibility policy for every required external-surface class. Each policy must use a justified observation window, named invalidating events, an explicitly bounded point-in-time claim, or a reviewed combination of those mechanisms. The policy must identify its evaluation points, including before owner acceptance and immediately before integration.

Freshness evaluation fails closed when current eligibility cannot be established, an observation window has expired, or a relevant invalidating event is known, even when the retained snapshot bytes and digest are unchanged. A replacement observation requires a renewed evidence binding and disposition. An explicitly historical, unavailable, or limited observation may retain only its bounded recorded meaning and cannot silently become current clearance. Deterministic offline regeneration consumes the exact retained snapshot and must never reacquire, refresh, or substitute live external state.

Raster and vector review must bind exact blobs and record scope, method, reviewer, inspected regions, and unresolved limitations. The reviewer must inspect the complete image; a list of expected text regions is not a substitute for whole-image review.

Coverage must include, as applicable, paths and filenames, hidden files, case variants, raw text, structured keys and values, generated context inputs and outputs, tests and fixtures, workflows and settings, external facilities, and actual machine consumers. Every limitation records the affected surface, reason, consequence, and disposition.

### 13.9 Canonical retention and availability

Canonical generated occurrence data is committed as uncompressed, bounded JSONL chunks with an authenticated index and compact summary tables. The index authenticates every chunk and aggregate but does not attempt to hash itself. Derived views are reproducible conveniences and may not become independently editable duplicate truth.

Compression is permitted only as transport packaging. Expiring workflow artifacts, caches, issue attachments, or external object-store URLs may not be the sole retained evidence.

The ledger must retain the accepted protocol, inputs, evidence, and decision records through reachable Git history plus a documented backup and recovery path. The protocol must state how repository transfer or rename affects repository identity and how evidence is retired after R8 without erasing the historical record.

Ledger unavailability blocks the migration gate that depends upon it. It must never block normal compiler build, check, verify, run, failure recovery, or already accepted production operation.

### 13.10 Execution isolation and archive exclusion

The protocol specification must enumerate current source roots and exact accepted identities. archive/unaccepted/** and any equivalent unaccepted archive location are permanently ineligible as executable tooling, compact classification input, current evidence, schema authority, fixture authority, or acceptance dependency.

Current evidence must reject bindings to the archived PR #95 snapshot. Only the exact independently reviewed tooling revision may execute.

The protocol and CI design must prevent or reject:

- import or module shadowing;
- general expression, selector, template, or query evaluation;
- path escape and filesystem traversal;
- symlink traversal;
- execution of repository hooks, Git filters, external diff drivers, or mutable Git configuration;
- execution of compiler, governance, archive, or candidate code;
- unpinned direct or transitive dependencies where dependencies exist; and
- credentials or permissions capable of compiler write, merge, release, cutover, or authority changes.

Evidence jobs use read-only credentials and bounded inputs. Their checks may block a gate; they may not automatically approve, merge, accept, release, rename, or authorize later work.

### 13.11 Mandatory M0.1 v2 protocol specification

Before tooling can be accepted, a separately reviewed M0.1 v2 protocol specification must fix and test:

- exact schema dialect and validation behavior;
- UTF-8, escaping, newline, number, duplicate-key, and key-order rules;
- canonical occurrence-key construction and uniqueness;
- canonical row order and chunk-boundary algorithm;
- serialized byte, row, chunk, total-package, nesting, string, and processing limits;
- oversized-row rejection;
- exact chunk naming;
- per-chunk and aggregate digest construction;
- independent recomputation of first and last keys and row counts;
- rejection of missing, extra, duplicated, renamed, or reordered chunks;
- the rule that the index contains no self-hash;
- deterministic handling of locale, time zone, host paths, concurrency, and platform differences;
- separation of deterministic evidence bytes from runtime provenance such as timestamps, runner identities, job URLs, and acquisition logs;
- per-external-surface freshness classes, eligibility rules, observation windows or invalidating events, bounded point-in-time treatment, required evaluation checkpoints, and fail-closed outcomes; and
- pinned runtime and trusted-component identities.

The specification must include positive, negative, mutation, resource-bound, and cross-platform fixtures with independently reviewable expected results. These details need not be fully enumerated in Amendment 2, but Amendment 2 requires them to be fixed and accepted before the tooling revision can become an M0.1 dependency.

### 13.12 Bounded implementation shape

The expected ledger layout is:

    protocol/m0.1/v2/
      schema.json
      generate.py
      verify.py
      audit_scan.py
      tests/

    evidence/m0.1/<source-commit>/
      record.json
      inputs/
      generated/
        occurrences/0000.jsonl
        index.json
        summary.json
      reports/
        generation.md

The exact accepted protocol may refine this layout. Its architecture remains bounded:

- standard-library Python unless a later reviewed specification justifies and pins another dependency;
- a restricted declarative selector language;
- no plugin system, database, network service, or general query language;
- one canonical occurrence stream;
- no second generated bulk mapping table when it would duplicate occurrence truth;
- deterministic byte-for-byte regeneration from exact retained inputs; and
- generated bulk evidence only in the ledger repository.

A rules-only or on-demand-only design is not accepted because it weakens durable independent review and later reproducibility.

### 13.13 M0.1 v2 rejection and exit conditions

M0.1 v2 rejects if any required identity is absent or drifting; an occurrence is missing, ambiguous, overlapping without an explicit rule, or multiply classified; semantic review is incomplete; exact occurrence-key reconciliation fails; an external or visual surface is falsely marked not applicable; a required external observation is stale, its current eligibility is unverifiable, or a named invalidating event has occurred at a required checkpoint; a decisive record is missing or conflicting; archive content enters the current trust path; protocol limits are exceeded; deterministic regeneration differs; prohibited compiler ancestry is introduced; or post-merge correspondence is absent.

M0.1 exits only when:

- Amendment 2 and the protocol/tooling revision were accepted through separate decisions;
- compact inputs were semantically reviewed;
- generator and independent auditor agree on exact occurrence-key sets;
- canonical ledger evidence regenerates byte-for-byte within fixed limits;
- every limitation has an explicit consequence and disposition;
- independent review recommends the exact C and E pair;
- Greg explicitly accepts that exact pair;
- ordinary merge commits integrate the accepted source and ledger heads;
- P verifies the actual merge identities, parents, trees, targets, ancestry, and correspondence; and
- every required external observation remains eligible under its accepted freshness policy at the required acceptance and pre-integration checkpoints.

At that point M0.1 may be complete while M1 remains unauthorized. M1 begins only through a separate later owner authorization; no M0.1 acceptance, integration, completion, review, or receipt supplies that authorization. No earlier step implies a later one.

## 14. Immediate next move

Submit Amendment 2 Draft 2 for focused independent confirmation of BOW-M0A2-01 and BOW-M0A2-02 and regression review of BOW-M01R-01 through BOW-M01R-10. The reviewer must confirm that M0.1 may complete while M1 remains unauthorized, that M1 still requires a separate later owner decision, and that the protocol must establish and enforce per-surface freshness eligibility before tooling acceptance.

If the reviewer recommends acceptance, Greg decides whether to accept and merge this contract-only amendment. No protocol implementation, compact-input construction, evidence generation, M0.1 acceptance, M1 work, production change, or external cutover is authorized by this draft.

## 15. Amendment 3 — Phase-local migration verification

### 15.1 Adoption, precedence, and honest retirement

This amendment changes the migration process, not language semantics. It becomes
operative only after independent review, Greg's explicit acceptance, and a human
ordinary merge into `bowfin-migration-staging`. It creates no implementation,
execution, merge, release, repository-rename, or domain-cutover permission.

Upon adoption, **M0.1 is retired incomplete as a migration prerequisite**. It is
not passed, accepted, completed, or retrospectively satisfied. The owner accepts
less exhaustive up-front migration certification in exchange for direct,
reviewable phase work. Phase-local tests and review do not establish the retired
protocol's global completeness or reproducibility claims.

This section has precedence over incompatible earlier requirements as follows:

| Earlier requirement | Replacement for this migration |
| --- | --- |
| Sections 4.2 and 5: exhaustive pre-M1 manifest/inventory | The phase-local identity table and checks in 15.2–15.3 |
| Section 8 M0 exit and Section 9 R0: readiness before M1 | Adoption of this amendment, then Greg's separate M1 order |
| Section 10 and M7 references to a manifest/final ledger | Ordinary phase records and final covered-surface scans, not a generated bulk corpus |
| Section 13, including 13.2 and 13.13: formal M0.1 acceptance/integration gates | Retired as prerequisites to M1, M2–M7, production acceptance, or cutover |
| Section 14: earlier immediate next move | Focused review of Amendment 3, followed by an owner decision |

In particular, no C/E/D/P package or proof, recovery-object bundle, global
occurrence reconciliation, deterministic package regeneration, external-snapshot
freshness cycle, custom supervisor, containment qualification, or synthetic
supervisor suite is required to enter or complete a migration phase. These
requirements must not reappear indirectly through a subordinate ledger document
or an older migration instruction.

The destinations and compatibility decisions in Section 2, typed namespaces and
roles in Section 4, authority boundaries in Section 6, permitted differences in
Section 7, M1–M7 scope boundaries, staging-only/human merge rules, R1–R8 release
and cutover gates, historical integrity, and rollback obligations remain in force.
Sections 15.2–15.4 replace their verification machinery where the table specifies,
not their substantive safety or compatibility requirements. No accepted KD,
runtime approval rule, ownership rule, or deferred native-backend boundary changes.

### 15.2 Phase entry and identity accounting

Each phase still needs Greg's separate implementation order. Before code edits,
the implementer records the exact accepted staging base commit/tree and the phase
scope in its PR or preparation record. Parked branches refresh from that base;
their PRs target `bowfin-migration-staging`, never `prod`. Intervening production
changes must be incorporated and dependent work re-reviewed under Section 3.

Use one compact phase-local table, normally in the phase PR description, recording
the exact namespace-qualified old/new spelling or retention, live/compatibility/
historical role, affected paths or structured fields, producer/consumer where
applicable, support policy, and validating test or inspection. Uniform occurrences
may be grouped by exact spelling, named paths and field roles with an explained
rationale and exposed exceptions; global `K` replacement and wildcard historical
exemptions remain forbidden. Mapping injectivity applies within each namespace.

Current non-archived mappings may be working references, not presumed semantic
approval or proof of completeness. Archived historical payloads, including the
`UNACCEPTED_HISTORICAL_SNAPSHOT` class, remain inert and ineligible as current
tooling, fixtures, classifications or acceptance evidence.
A collision, ambiguous role, unknown machine identifier, unexplained difference,
or attempted historical rewrite in the affected scope blocks that phase until
resolved. Unchanged identities assigned to a later phase are deferred work, not
historical exemptions.

Inventory, implementation, tests and independent review occur within the same
phase; no separate globally approved inventory is required before M1 begins.
M1 remains machine/evidence identity compatibility only: no CLI, source-file,
documentation, branding, domain or repository rename enters through it.

### 15.3 Required checks before each phase merge

1. Review the actual phase diff and its whole affected producer/consumer paths,
   including structured-field roles. Preserve original historical bytes, hashes,
   recorded commands and verbatim claims. Renaming both an expected value and
   its observation must not erase a semantic or authority defect.
2. Exercise the changed compatibility boundaries with focused positive and
   negative tests: promised legacy inputs, canonical outputs, unknown/conflicting
   identities and mixed forms where applicable. Translation cannot manufacture,
   recover, renew or substitute approval, grants, custody, replay identity,
   schema validity or execution authority. No absent facility is invented merely
   to fill a checklist.
3. Pass the existing build, debug/release tests, formatting/lint, documentation
   and static-verification workflows on the exact reviewed phase revision.
   Reconcile predecessors and enumerate intentional one-to-one test renames.
   Record actual run identities and coverage; predecessor or test-merge success
   must not be misreported as exact-head success. GitHub Actions remains the
   authoritative Rust environment; no local Rust or custom M0.1 runner is needed.
4. Sarah independently reviews the code, table, tests, limitations and phase
   scope together. Greg separately accepts or rejects and is the sole merger.
   Only a human performs the ordinary merge into staging.
5. The phase PR records base/head/tree, the table, historical exceptions, actual
   CI/test evidence, limitations and rollback boundary. After merge, read back
   and record the actual merge commit, parents and resulting tree. Ordinary Git
   history and PR/review/run records suffice for these new phase records; no
   content-addressed ledger decision package or proof-engine run is required.

Passing CI alone is not acceptance. A substantive compatibility or safety defect,
unexplained predecessor loss, failed required check or missing affected-surface
review still blocks merge. This amendment itself authorizes no execution.
A later phase implementation order may include ordinary builds/tests and existing
CI for that phase; newly authored migration infrastructure receives no permission
merely because those ordinary checks are allowed.

### 15.4 Final and external checks remain real, but occur when relevant

At M7, use ordinary repository-wide case-variant, path/name and structured-field
searches covering Section 4's identity families, plus whole-image/manual visual
inspection and the existing full CI/Sentinel 0/clean-install checks. Account for
remaining legacy hits by exact locations, compatibility policies or truthful
historical reasons; state scan coverage and unresolved limitations plainly.
There is no claim of formal exhaustive occurrence certification.

Perform the extension gate at M3, coordinated governance work at M4, visual
replacement at M5 and actual external-surface/runbook checks at M6–M7 and the
applicable R-state transition. Relevant external facts must be current enough
for the action they justify; unavailable evidence is not "not applicable."
Unrelated external or visual observations do not block M1 or require a global
36-hour snapshot refresh before internal machine-compatibility work. Retiring
that global freshness mechanism does not clear a domain, extension, redirect,
package, governance or release surface that has not been inspected.

Production acceptance, post-merge verification and explicit external-cutover
authorization remain separate under R4–R8. No early repository rename, canonical
DNS switch, package publication or public-release claim is permitted.

### 15.5 Preserve history; stop the infrastructure critical path

Existing accepted tooling and historical records retain their original bounded
meaning; adverse or indeterminate results are not rehabilitated. Preserve rather
than rewrite or delete them. M1 does not depend on completing or merging any ledger
PR, including PR #18's supervisor and qualification work. Preservation drafts can
remain open without blocking implementation; no new ledger closeout is an entry
gate.

Retirement does not disable an installed workflow technically and does not
authorize operating it. Earlier one-run/campaign authorizations remain spent.
There is no retry, repair, replacement invocation or suite authority here, and no
requirement to repair anonymous-API acquisition before starting the rename.

Use existing review, search, diff and CI tools. **No new migration infrastructure
unless a concrete defect directly prevents the rename or makes its result
unverifiable.** Any such proposal must explain the minimum necessary change and
why existing checks are insufficient, and receive a separate owner decision.
Ordinary phase tests and small source edits are not a new assurance platform;
optional infrastructure improvements stay off the migration critical path.

### 15.6 Immediate next step

Submit this contract-only amendment for focused independent review. If Greg
accepts it, a human merges its exact reviewed revision into staging and the
actual merge is read back. Greg may then issue the separate M1 implementation
order against that accepted staging head. Neither this draft, its review, its
future merge nor M0.1's retirement starts M1 automatically.

## 16. Proposed Amendment 4 — Bounded post-cutover content alignment

**Status: Proposed, pending independent review, Greg's explicit acceptance and a
human ordinary merge into `bowfin-migration-staging`. Not operative.**

**Prepared:** 2026-11-01, as part of the M7 preparation candidate based on staging
`0f87fbc0c4001a9721561264432f625eeb6e0185`, tree
`e619fbce9f25fe25722959d4f2e5577a7b50a3ee`. The accepted pre-amendment contract
blob is `a8aa897de2c179d25ce6050bc760968f492a8707`.

### 16.1 Purpose, precedence and adoption

The complete migration still receives **one independently reviewed, fixed
production candidate through R2–R5**. The initial candidate, its exact predecessor
reconciliation, independent recommendation, Greg's separate production acceptance,
human staging-to-production merge and fresh production verification are unchanged.

An accepted, properly gated deferred alignment plan may coexist with the initial
R2/R3 disposition. Completion of the later post-rename alignment follow-up is not
a prerequisite to the original R5 production transition. The follow-up remains
separately gated after R5, the authorized and executed rename, verification of
applicable new canonical destinations, assessment of every applicable old-name
destination and Greg's separate preparation order. This sequencing distinction
waives no post-rename verification, mandatory compatibility, recovery, R8 or
external-authorization obligation.

If and only if this amendment is adopted, it creates one narrow exception to
Section 3: **one additional staging-to-production PR solely for post-cutover
content alignment**, within Sections 16.2–16.5 below. It does not turn an M0–M7
phase PR into a production PR, authorize another feature/release candidate, or
permit direct production writes. Other Section 3 and R-state requirements remain.

This amendment becomes operative only after independent review, Greg's explicit
acceptance of its exact revision, a human ordinary staging merge and readback of
that integration. Until then, Amendment 3 and the existing sole-candidate rule
remain controlling. Preparation, CI and review of this proposal grant no future
implementation, merge, repository-rename, domain, publication or cutover authority.
M6-OP07 remains pending; this M7 preparation does not establish final R2 or R3.
M0.1 remains retired incomplete.

### 16.2 Preconditions to preparing the one follow-up

Even after adoption, the follow-up may be prepared only after all of these facts
are established and recorded:

1. The actual fixed migration production candidate has been independently reviewed,
   separately accepted by Greg, human-merged into production and verified through
   R5 at its actual production commit/tree.
2. Greg separately authorized the repository rename under R6, a human executed it,
   and readback confirms the same repository identity, history and issue/PR state.
3. Individually verify each applicable new canonical repository, raw-content,
   workflow, badge and routing destination for the exact consumers the follow-up
   will change. A verified Git redirect alone is not proof for another surface.
   Test every applicable old-name destination and individually record its result
   as working compatibility or redirect behavior; failed or unavailable behavior;
   or indeterminate behavior with the limitation stated. Old-name assessment need
   not be successful to satisfy this preparation gate: an old-name failure must
   not, by itself, prohibit preparing the narrowly authorized canonical-link
   correction when its corresponding new canonical destination is verified and
   the accepted content still requires correction. Never represent a failed,
   unavailable or indeterminate old route as successful. Every other Section 16
   prerequisite must still be satisfied. Any separately mandatory legacy-
   compatibility obligation remains binding and, if unsatisfied, blocks its
   applicable completion claim. Retain old-route failures/limitations in the
   compatibility and recovery assessment; neither preparation nor the follow-up
   converts them into a passing R8 result or erases them.
4. Any domain or migration-state wording proposed for change is supported by the
   corresponding actual routing/state evidence. An unverified destination remains
   deferred; this exception cannot make an unavailable surface true by editing prose.
5. Greg separately authorizes preparing that exact bounded follow-up. Adoption of
   this amendment or successful cutover checks alone do not issue that order.

### 16.3 Exclusive permitted content boundary

The follow-up's complete tracked diff is limited to these six files and exact
existing passages/field roles. The selectors below identify the current passages
at reviewed M7 head `047e80748b45f5bf54408189b5c99780c9f80fae`; headings and field
prefixes define scope, not the whole surrounding section. Movement or line wrapping
does not enlarge a field. If a selector is missing, ambiguous or requires a wider
edit in then-verified production, stop for a separately reviewed scope decision.

| File | Sole permitted changes |
| --- | --- |
| `README.md` | Only the repository-slug component of the eight existing URL attributes in the top workflow-badge paragraph: the four `a[href]` workflow-page URLs and their four associated `img[src]` badge URLs for exactly `build.yml`, `tests.yml`, `verify.yml` and `docs.yml` under `https://github.com/gregkrsak/klyxr/actions/workflows/`. Replace that slug with the verified canonical slug; preserve every other URL component, attribute, image reference, layout, copy, command and alt text. |
| `scripts/verify-repo-scaffold.py` | Only the value of the existing module-level `repo_slug = "gregkrsak/klyxr"` assignment, coupled to those eight README fields. Its name, verifier algorithm, required checks and rejection behavior remain unchanged. |
| `REPO_STRUCTURE.md` | Only the live metadata in the opening paragraph under `# Bowfin Repository Structure`, before its first fenced tree: `Current operational GitHub home: gregkrsak/klyxr` and `The adopted destination is gregkrsak/bowfin; no repository cutover has occurred.` Update only their repository values and corresponding verified cutover-state wording. No domain field currently exists here and none is introduced by this exception. The prototype-description sentence, future-organization-transfer sentence, fenced tree (including its root label), and all later prose are outside scope. |
| `docs/ai/BOWFIN_CONTEXT.md` | Under `## Project identity`, only the existing `Adopted primary domain:` and `Operational legacy public sites:` lines and their verified operational/cutover-state qualifiers; retain the adopted domain identities and legacy-endpoint retention commitment. Under `## Brand system`, only the single bullet beginning `- adopted repository destination:` that also contains `operational repository:` and `until authorized cutover`. No other field, bullet or passage is eligible; CLI/source-extension promises, branding/artwork/hash fields and historical implementation passages remain unchanged. |
| `docs/ai/LANGUAGE_DECISIONS.md` | Under `## BD-001 — Public project name`, only the existing `**Adopted primary domain:** bowfin-lang.com (external cutover deferred)` field's verified operational/cutover-state wording. The domain identity is retained. The heading, `Status`, `Decision`, CLI, source-extension and `Operational legacy endpoints` fields, decision history and every other decision are outside scope. |
| `docs/ai/BOWFIN_AI_CONTEXT_BUNDLE.md` | Regenerate only with the unchanged canonical generator from the unchanged ordered input set, incorporating solely the specifically permitted input-field changes above. Every output difference must derive from those input changes; no hand edit or independent bundle cleanup. |

The generator and its prescribed order remain unchanged:
`BOWFIN_CONTEXT.md`, `DESIGN_PRINCIPLES.md`, `LANGUAGE_DECISIONS.md`,
`ARCHITECTURE.md`, `OPEN_QUESTIONS.md`. Only the specifically permitted input
passages may differ; generated output must reproduce byte-for-byte from those
inputs.

No compiler behavior, verifier algorithm, workflow, dependency, artwork,
compatibility promise, historical claim, recorded command, hash or decision status
may change through this exception. No historical payload or digest is rewritten.
No new reader, alias, authority, execution behavior or migration infrastructure is
created. Any necessary change outside the exact boundary stops this follow-up and
requires a separately reviewed plan and Greg's authority; it cannot be hidden in
the content-alignment PR.

### 16.4 Ancestry, review and the additional production merge

Before the additional production merge, staging must incorporate the
**then-verified production ancestry** through ordinary reviewed, human-operated
integration. Record actual commits, parents and trees; preserve history without
reset, rebase, force update, replacement cherry-pick or silent ancestry bypass.
Any intervening production change is incorporated and dependent evidence is
re-reviewed. Preparation/integration PRs continue to target staging; the one
additional staging-to-production PR is separately identified as this exception.

The exact follow-up diff must demonstrate compliance with Section 16.3, explicit
live/compatibility/historical occurrence roles, coherent URL/verifier consumers
and canonical context reproduction. Complete predecessor reconciliation, existing
required CI, independent review and **Greg's separate exact-candidate acceptance**
are required before a human performs the additional production merge. No test
identity loss, addition, substitution or migration may be attributed to this
content-only exception. A green summary, matching text or prior-candidate evidence
does not substitute for the follow-up's actual evidence.

There is no automatic ready-for-review transition, production PR creation,
acceptance, merge or authorization of the next step from this amendment. Only a
human may merge.

### 16.5 Fresh production evidence, external reconciliation and recovery

Read back the resulting production merge commit, parents and tree and obtain
fresh applicable post-merge verification. Record the new production identity;
do not reuse the original candidate's exact-head evidence for changed content.

Before further external writes or R8 completion, dependent external authorization
and checks must be reconciled or renewed against the new production state.
Unavailable or mismatched evidence stops the dependent action and completion
claim. The exception does not retrospectively close a failed route or grant
permission for an unnamed external write.

Any repair or recovery beyond this single bounded follow-up still requires the
separately reviewed plan and owner authority required by Section 11. Repository
content rollback and external-state recovery remain distinct; caches, downloads,
public statements, identity routing and issued authority are not undone by a Git
revert. Old repository-name reuse and removal of retained compatibility surfaces
remain prohibited under the governing contract.
