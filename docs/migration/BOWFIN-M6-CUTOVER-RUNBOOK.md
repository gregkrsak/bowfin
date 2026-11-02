# Bowfin M6 external-cutover runbook

**Revision 1 — preparation candidate, not an execution authorization.**

This document prepares human-operated external changes. Nothing below has renamed
a repository, changed production, DNS, redirects, hosting, settings, packages or
announcements. Greg must separately authorize each external write after the
production gates. Sarah's review and passing preparation CI do not supply that
authority.

## 1. Provenance, scope and execution gates

| Item | Preparation identity |
| --- | --- |
| Compiler repository | `gregkrsak/klyxr`, repository ID `1398259526`, node ID `R_kgDOU1e_Rg` |
| Accepted staging base | `e21ab1bd827ef805a7d4d9649337f445d27ddc77` |
| Accepted base tree | `1774b42b2d791b0da73caf41a1917a7401a37675` |
| Unchanged production at preparation | `3c0a540df7d5ad75510db4ecad80e27fb4d0903a` |
| Preparation branch / PR target | `bowfin-m6-cutover-preparation` / `bowfin-migration-staging` |
| Parked branch, untouched | `bowfin-m6-public-cutover` at `637f10a57dedf941cc7c75ad5b57d66184716458` |
| Accepted contract | [BOWFIN-NAME-MIGRATION-PLAN.md](BOWFIN-NAME-MIGRATION-PLAN.md), blob `a8aa897de2c179d25ce6050bc760968f492a8707` |
| Accepted Arnold charter | Governance commit `bc5a66892493c5c326662ada1db4259f8187eade`, `agents/arnold/CHARTER.md` |

Contract §§8–11 and 15 control. Section 15 supersedes incompatible M0.1 machinery:
M0.1 remains **retired incomplete**. The contract's retained historical proposal
header does not replace its recorded acceptance or the current assignment. No
global certification platform, new workflow, website or release system is needed
or authorized by this runbook.

The sequence is mandatory:

| Gate | Evidence required before proceeding | Next permitted activity |
| --- | --- | --- |
| M6 preparation | Exact base; this one-file candidate; ordinary CI; Sarah's review; Greg's decision and a human staging merge | M6 preparation may be accepted; no automatic M7 start |
| M7 / R2–R3 | Separate M7 order; fixed staging commit/tree; stabilization evidence and independent recommendation | Greg may consider that exact production candidate |
| R4 | Greg's separate production acceptance; a human ordinary merge of the sole staging-to-`prod` candidate | Production verification, not external cutover |
| R5 | Actual production merge commit, parents, tree and required post-merge evidence read back | Greg may consider named external actions |
| R6 | Greg's separate written authorization naming surfaces, operators, order, exact targets, permitted recovery and stop conditions | Only the named external operations |
| R7 | Authorized public changes executed in the dependency order below, with receipts | Post-cutover verification |
| R8 | Independent checks of all applicable old/new entry points and consumers | Greg may decide completion; compatibility removal still requires a future BED |

At execution time, replace the **preparation** production identity above with the
actual R5-verified production identity in the execution record. Do not substitute
this M6 base/head or a synthetic PR merge for R5. Candidate drift, a changed
production branch, broadened merge authority or automatic merging stops the plan
until the contract's review requirements are met.

Greg is the operator and final owner authority. A specifically named human
delegate may operate only within Greg's written authorization. Only a human may
merge. An independent checker should read the resulting state from another
normal client/network where practical, rather than accepting the operator's
success dialog.

## 2. What was actually inspected

These are bounded observations, not promises about execution-day state.

### 2.1 Repository and consumer observations

The authenticated connector inventory was recorded on **2026-11-01 at
17:30:10 UTC (09:30:10 PST)**, from reads earlier in the preparation session.
The exact baseline/production/parked identities passed before any tracked edit.
The full baseline tree was non-truncated: 286 non-tree entries.

| Surface | Observed result | Coverage and limitation |
| --- | --- | --- |
| Repository metadata | Public `gregkrsak/klyxr`; description `The Klyxr language. Systems programming you can reason about.`; homepage empty; default branch `prod`; topics empty | Public metadata is not an export of administrative settings |
| Repository facilities | `has_pages=false`, `has_wiki=true`, `has_discussions=false`, `has_projects=true`, `has_downloads=false` | Feature flags do not establish existing wiki pages, project boards, deployments or hidden integrations |
| README links | Lines 9–12 contain four workflow links and four badge-image URLs using `gregkrsak/klyxr` | Exact consumers listed in §7; unchanged here |
| Scaffold verifier | `scripts/verify-repo-scaffold.py:10` fixes `repo_slug = "gregkrsak/klyxr"` and checks all eight URLs | A future README URL change must update this consumer atomically |
| Tracked integration files | Exactly `build.yml`, `tests.yml`, `verify.yml`, `docs.yml` under `.github/workflows/`; no tracked `CNAME`, `.gitmodules`, `action.yml` or `action.yaml` anywhere | No Pages/deployment/publishing workflow or repository-hosted Action definition found in this tree; untracked/private/external services unavailable |
| Action consumers | Those workflows use third-party `actions/checkout@v4`; Docs also uses `actions/setup-python@v5` | These are not Actions hosted by this repository. External callers/Marketplace settings were not globally discoverable |
| Releases / tags | Releases API returned an empty page (`per_page=100`); Git tag-ref enumeration returned zero | No visible release/tag migration. Private drafts or unavailable administration views require owner inspection |
| Distribution | `compiler/Cargo.toml:6` remains `publish = false`; repository clone/source-download facility exists | No registry release, binary installer or package publication was established or authorized |
| Other refs / PRs | 21 pre-existing branch refs, zero tag refs; 59 PRs returned in the all-state page (`per_page=100`) | Preserve them. R6 must take a fresh snapshot; these counts are not execution-day expectations |
| Destination repository | Read of `gregkrsak/bowfin` returned 404 | Not proof of reservable availability; no destination repository was created |
| Governance | Separate `gregkrsak/klyxr-ai-governance` and accepted charter remain independent | Its repository name, charters and installed Project Instructions are not renamed by this assignment |

The connector did not expose the attempted Pages, workflow-list or repository
tag-list API routes. Those failures are **unavailable through this tool**, not
proof of absence. The separate Git tag-ref read succeeded. Registrar accounts,
hosting configuration, private release drafts, webhooks, installed Apps, effective
selected-repository restrictions, social-preview settings and project-board
contents were not inspected administratively.

Public HTTP GETs between **17:26:59 and 17:27:06 UTC (09:26:59–09:27:06 PST)**
observed:

| Requested surface | HTTP chain seen | Meaning |
| --- | --- | --- |
| `https://github.com/gregkrsak/klyxr/wiki` | 302 → `https://github.com/gregkrsak/klyxr` → 200 | No public wiki page was served in this check; not a privileged wiki-content inventory |
| `https://gregkrsak.github.io/klyxr/` | 404 | Consistent with Pages disabled; not proof about unrelated sites/domains |
| Fixed-base raw README at `raw.githubusercontent.com/gregkrsak/klyxr/e21ab1bd827ef805a7d4d9649337f445d27ddc77/README.md` | 200, no HTTP redirect | Only this old-name raw endpoint was observed |
| Each of the four old-name workflow pages and four badge URLs in README | 200, no HTTP redirect | Does not establish future rename behavior or exact-head test success |

The HTTP client bounded redirects to six responses and body reads to 65,536 bytes.
It did not execute browser JavaScript. A 200 alone is not a functional or semantic
pass.

### 2.2 Domain observations and limits

The contract reports acquisition of `bowfin-lang.com` and the secondary
`.org`, `.net`, `.info`, `.online` names. The table expands those suffixes
to the probed `bowfin-lang.*` spellings. Greg must confirm the exact registered
FQDN/account inventory before any operation; public answers do not prove ownership.

Between **2026-11-01 17:23:48–17:24:28 UTC (09:23:48–09:24:28 PST)**, 77
read-only Google Public DNS JSON queries covered A, AAAA, CNAME, NS, SOA, MX, TXT,
CAA at each apex and A/AAAA/CNAME at each corresponding `www` hostname.
All queries returned DNS status 0. This is one recursive resolver's view, not an
authoritative zone export or a global propagation/DNSSEC claim.

| Probed apex (and its `www` host) | Apex A answer / observed TTL | Initial HTTP apex / www | Initial HTTPS apex / www |
| --- | --- | --- | --- |
| `bowfin-lang.com` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `bowfin-lang.org` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `bowfin-lang.net` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `bowfin-lang.info` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `bowfin-lang.online` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `klyxr.com` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |
| `klyxr.online` | `91.195.240.94` / 300 s | 200 / 200 | observer 503 / 503 |

All apex NS answers contained `ns1bcp.name.com.`, `ns2lns.name.com.`,
`ns3cfp.name.com.`, `ns4sxy.name.com.` (observed TTL 300 s). Apex
AAAA/CNAME/MX/TXT/CAA queries returned no Answer records; SOA answers were present.
These observations neither identify the registrar/hosting account nor authorize
editing those nameservers. Do not invent a zone from these partial answers.

Each of the 28 initial HTTP/HTTPS GET chains ended at its first response: no
HTTP `Location` destination was observed. HTTPS responses contained
Envoy-style upstream-connection error text. Origin reachability, certificate
identity, expiry, renewal and end-to-end TLS remain **unverified**, not failed
origin certificates and not passed TLS checks. The environment's ordinary system
resolver also failed; DoH was the available DNS observation path.

A later same-client HTTP read of `bowfin-lang.com` at **17:30:13 UTC** returned
200 with a domain-resources title, `img.sedoparking.com` assets and client-side
navigation code. That is an observed noncanonical parking-like page, not a
Bowfin project preview. Client-side navigation was not executed. Additional
default-client reads of the primary/legacy apexes returned 403, and a later
legacy read returned 445; the differing observer results are disclosed, not
normalized into success. No advertised project destination or secure redirect
was established.

No provider preview URL, authenticated hosting configuration or certificate
inventory was supplied. Testing the eventual destination without changing public
behavior is therefore limited to these existing endpoints. Before execution,
Greg must supply an existing noncanonical preview route if his provider has one;
otherwise record that preview is unavailable and rely on an explicitly authorized,
reversible activation window followed by independent checks. Do not deploy a new
site or change DNS merely to manufacture preview evidence.

## 3. Unresolved prerequisites: execution stops here until supplied

| ID | Required owner-supplied information or evidence | Blocks |
| --- | --- | --- |
| M6-OP01 | Fixed M7 candidate/recommendation, R4 human merge receipt, actual R5 production commit/tree and passing post-merge evidence | Every external write |
| M6-OP02 | Repository-admin confirmation of rename availability, Pages/custom-domain state, wiki contents, webhooks/Apps/Marketplace consumers, project boards and social-preview settings | Repository rename and consumer-completeness claims |
| M6-OP03 | Current selected-repository App/connector list and effective permissions, with this repository identified by ID; post-rename inspection plan | Claim that connector authority survives |
| M6-OP04 | Exact domain FQDNs, controlled registrar/DNS/hosting accounts, renewal/lock status, existing zone/routing exports, restore capability and operator access | Any DNS/hosting/domain operation |
| M6-OP05 | Existing provider's actual forwarding/TLS capabilities, preview if available, certificate names/expiry/renewal, complete before/after record/rule tuples and approved route policy | Any domain activation or truthful secure-domain claim |
| M6-OP06 | Existing non-root paths, query handling, mail/subdomain consumers and their exact retention/mapping; explicit decision for `www` and optional secondary routes | Silent loss of existing URL/service behavior |
| M6-OP07 | Revised proposed Amendment 4 (contract §16) is pending: independent review, Greg's explicit acceptance and human staging integration/readback, then its separate post-R5/rename canonical-verification, old-route-assessment, preparation/merge and exact-content/CI gates | Still unresolved; no current URL/metadata edit, extra production PR or future merge authority. After adoption, a properly gated deferred plan may coexist with initial R2/R3; completing the later follow-up is not a prerequisite to original R5. Compatibility/recovery/R8 obligations remain binding. |
| M6-OP08 | Named announcement channels, exact approved copy, timing and publication operator; any separately authorized release/package plan | Publication or new distribution claims |

These are operational requirements, not language questions or instructions to
create missing facilities. The absence of an owner export cannot be converted
into an absence finding. If inspection discovers another real consumer, add a
specific target/verification/recovery row through reviewed runbook revision
before authorizing its change.

## 4. Execution record and read-only preflight (future, not run here)

**Operator/access:** Greg, repository admin plus his existing Git credentials;
DNS/hosting credentials only for later named operations. Keep private exports
private. Do not put tokens, cookies, private settings or credentials in the
public PR.

**Gate:** M7/R4/R5 complete; R6 must name the writes before §5 onward. The checks
in this section are read-only remotely. They are prepared commands, not results
of this M6 assignment. Run in Bash on Greg's Linux machine with existing
`git`, `curl` and Python 3; do not install a new migration tool.

Start one shell session. Supply the two exact values from the R5 record, then
retain the resulting evidence directory outside any repository:

```bash
set -euo pipefail
read -r -p 'R5 verified production commit (40 hex): ' M6_PROD_SHA
read -r -p 'R5 verified production tree (40 hex): ' M6_PROD_TREE
[[ "$M6_PROD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$M6_PROD_TREE" =~ ^[0-9a-f]{40}$ ]]
M6_EVIDENCE_DIR=$(mktemp -d "${TMPDIR:-/tmp}/bowfin-cutover.XXXXXX")
export M6_PROD_SHA M6_PROD_TREE M6_EVIDENCE_DIR
date -u --iso-8601=seconds > "$M6_EVIDENCE_DIR/started-utc.txt"
curl --fail --silent --show-error --proto '=https' \
  https://api.github.com/repos/gregkrsak/klyxr \
  > "$M6_EVIDENCE_DIR/repository-before.json"
git ls-remote --heads --tags https://github.com/gregkrsak/klyxr.git \
  | LC_ALL=C sort > "$M6_EVIDENCE_DIR/refs-before.txt"
git ls-remote --symref https://github.com/gregkrsak/klyxr.git HEAD \
  > "$M6_EVIDENCE_DIR/head-before.txt"
grep -F "refs/heads/prod" "$M6_EVIDENCE_DIR/head-before.txt"
git ls-remote https://github.com/gregkrsak/klyxr.git refs/heads/prod \
  | grep -F "$M6_PROD_SHA"
```

Independently require the exact repository ID above, default branch `prod`,
the R5 production SHA, and a matching R5 tree via the public Git-commit API or a
clean clone. Stop on a mismatch, not just a missing grep match. The M6 preparation
production SHA is deliberately not used as the execution SHA.

Before rename, Greg also captures:

- All heads/tags and key historical refs, with their exact values.
- Paginated existing issue and PR IDs/numbers/states and representative historical
  discussion URLs, including Issue #84/#89 and PR #105/#106. Use the current
  connector or GET `/repos/gregkrsak/klyxr/issues?state=all&per_page=100&page=N`
  and `/pulls?state=all&per_page=100&page=N` through the last page. Preserve
  payloads; a count alone does not prove preservation.
- Public metadata and the administrative settings/consumer inventory in
  M6-OP02–03, through owner inspection. Record existing selected-repository
  restrictions and granted permissions without changing them.
- Releases, tags, any downloadable assets and applicable wiki/Pages state.
- Provider exports of DNS records, routing rules and TLS settings, with exact
  record names/types/values/TTLs and recovery instructions, plus the named
  destination preview results or the disclosed lack of preview.
- The R6 authorization reference and the exact ordered action list, including
  which recovery operations Greg has authorized. If emergency recovery was not
  authorized, stop and obtain a named owner decision rather than improvising.

Keep a change freeze during the comparison interval. Unexpected new commits,
refs, releases, consumers or provider changes stop execution for reconciliation.
This is an ordinary owner record, not a revived M0.1 evidence protocol.

## 5. A01 — Rename this repository in place

**Current → target:** `gregkrsak/klyxr` → `gregkrsak/bowfin`; same owner,
repository ID, Git objects, issues and PRs. No organization transfer, copied
repository, archive/recreate operation or default-branch change.

**Operator/access/gate:** Greg with repository administration access; R5 and
explicit R6 authorization for this rename; fresh §4 snapshot and destination
availability confirmed in the owner UI. A 404 during preparation did not reserve
the name.

**Exact future operation:** In the existing repository, open **Settings →
General → Repository name**, enter `bowfin`, and select **Rename**. Check the
displayed owner and repository before submitting. Do not create a destination
repository first. GitHub documents this operation and its access requirements
in [S1](#14-primary-documentation).

**Expected result and independent verification:** Read
`https://api.github.com/repos/gregkrsak/bowfin`; require ID `1398259526`,
node ID `R_kgDOU1e_Rg`, full name `gregkrsak/bowfin`, same visibility and
default branch `prod`. Compare every saved head/tag and the issue/PR ID/state
snapshot; open old and new representative discussion URLs and compare their
actual contents. Do not infer preservation from a matching title.

**Failure stop:** Name unavailable, unexpected owner/ID, lost ref/discussion,
changed protection/selected access, login/error destination or any unresolved
consumer. Stop subsequent metadata/domain/publication actions.

**Recovery/consequence:** Prefer a forward repair with GitHub support and Greg's
specific authorization. Preserve the existing repository and all refs; never
create `gregkrsak/klyxr` after rename. GitHub warns that reusing the old name
breaks redirects. Reverting Git content or changing the display description
does not undo the external rename. A reverse rename is not an automatic rollback
and requires a separately reviewed owner decision.

## 6. A02 — Verify web, raw, badges and connector access independently

**Targets/operator/gate:** Greg and an independent reader; public reads plus
owner access for the connector-settings inspection; A01 completed. These reads
must pass before their dependent public claims. No write probe is authorized.

GitHub documents redirects for repository web information and Git operations,
while project-site URLs and hosted-Action calls need separate handling [S1].
Treat raw files, badges and connectors as separate tests; this runbook makes no
undocumented redirect guarantee for them.

| Facility | Exact old / new forms to exercise | Pass criterion |
| --- | --- | --- |
| Repository and discussions | `https://github.com/gregkrsak/{klyxr,bowfin}`, `/issues/84`, `/issues/89`, `/pull/105`, `/pull/106`; expand both names | Canonical pages show the same repository/items; old links reach the corresponding item, not merely the repository root |
| API identity | `https://api.github.com/repos/gregkrsak/{klyxr,bowfin}`; both `/branches/prod` and `/git/commits/$M6_PROD_SHA` | Same repository ID and exact R5 production commit/tree; retain status/Location chains |
| Raw, pinned | `https://raw.githubusercontent.com/gregkrsak/{klyxr,bowfin}/$M6_PROD_SHA/README.md` | Each successful response hashes to the README Git blob at the R5 commit |
| Raw, moving | Same two raw names with `refs/heads/prod/README.md` | At the frozen check, bytes equal that production commit; stale CDN/moved prod is investigated |
| Workflow pages | Both old/new slug expansions of the four named workflow links in §7 | Each names the intended workflow; independent exact-prod run evidence agrees |
| Badge images | Both old/new slug expansions of the four badge-image paths in §7 | Actual SVG/image returned; test each separately and compare with that workflow's actual production evidence, not the color alone |
| Connector / installed App | The existing selected repository, ID `1398259526` | Greg inspects the same selected-repository set and granted permissions after rename; intended read operations resolve the same repository |

Use GET, retain every response header block and final URL, and reject login,
HTML error or unrelated content despite a 200. For example (prepared only):

```bash
curl --silent --show-error --location --max-redirs 5 \
  --proto '=https' --proto-redir '=https' --max-time 45 \
  --dump-header "$M6_EVIDENCE_DIR/old-issue-84.headers" \
  --output "$M6_EVIDENCE_DIR/old-issue-84.html" \
  --write-out '%{http_code} %{url_effective}\n' \
  https://github.com/gregkrsak/klyxr/issues/84
```

Check pinned raw content against an independently fetched Git object:

```bash
git clone --no-checkout https://github.com/gregkrsak/bowfin.git \
  "$M6_EVIDENCE_DIR/raw-reference"
test "$(git -C "$M6_EVIDENCE_DIR/raw-reference" rev-parse "$M6_PROD_SHA^{tree}")" \
  = "$M6_PROD_TREE"
M6_README_BLOB=$(git -C "$M6_EVIDENCE_DIR/raw-reference" \
  rev-parse "$M6_PROD_SHA:README.md")
for M6_NAME in klyxr bowfin; do
  curl --fail --silent --show-error --location --max-redirs 5 \
    --proto '=https' --proto-redir '=https' --max-time 45 \
    --dump-header "$M6_EVIDENCE_DIR/raw-$M6_NAME.headers" \
    --output "$M6_EVIDENCE_DIR/raw-$M6_NAME.md" \
    "https://raw.githubusercontent.com/gregkrsak/$M6_NAME/$M6_PROD_SHA/README.md"
  test "$(git hash-object --no-filters "$M6_EVIDENCE_DIR/raw-$M6_NAME.md")" \
    = "$M6_README_BLOB"
done
```

**Failure/recovery:** Missing raw/badge redirects are not repaired by fabricating
an old-name repository. Stop the affected claim; preserve observations, use a
verified canonical URL where a separately reviewed content update permits it,
and escalate any required legacy access that GitHub does not supply. Do not
report an unimplemented compatibility route as working.

A successful public GET does **not** establish authenticated write authority.
Greg must inspect effective App permissions and the selected repository identity.
Do not select “All repositories,” add unrelated repositories, install another
App, broaden scopes or perform a test mutation to make a connector green.
If the restricted connection loses access, pause its dependent work and request
a separately authorized correction that preserves Greg's restriction. Public
read fallback is not proof that the restricted connection is healthy.

## 7. A03 — Canonical live links and their coupled verifier (future content PR)

**Operator/access/gate (future, not yet authorized):** Greg must separately
authorize the bounded URL-only repository-content change after A01 and the
canonical-verification and old-route-assessment requirements of contract
§16.2(3), with every other §16 prerequisite satisfied. It needs ordinary review,
required CI and a human merge against the then-authorized target. This M6 candidate
authorizes none of those edits or merges.

This cannot be implemented by retargeting an M0–M7 phase PR to `prod`. Contract
§3 also reserves staging-to-production integration to its sole separately
identified final candidate. Proposed Amendment 4 in contract §16 would permit
one additional staging-to-production PR solely for its exact six-file content
boundary. It is **pending and not operative**. Independent review, Greg's explicit
acceptance, human staging integration and readback are required before it can
resolve M6-OP07. This preparation does not close M6-OP07 or establish final R2/R3.

If adopted, the follow-up still cannot be prepared until the actual fixed
production candidate has passed acceptance/human merge/R5 verification, the
repository rename is separately authorized and executed, each applicable new
canonical repository/raw-content/workflow/badge/routing destination is individually
verified, every applicable old-name destination is tested and its outcome recorded,
and Greg separately authorizes preparation. Record each old outcome as working
compatibility/redirect behavior, failed or unavailable behavior, or indeterminate
behavior with its limitation stated. An old-route failure does not, by itself,
prevent the bounded canonical-link correction when the corresponding new canonical
destination is verified and the accepted content still requires correction. Never report an
old failed/unavailable/indeterminate route as successful. Any separately mandatory
legacy-compatibility obligation still blocks its applicable completion claim if
unsatisfied; failures and limitations remain in compatibility/recovery assessment
and cannot become a passing R8 result through the follow-up. All other §16 gates
remain required. The then-verified production ancestry must enter staging through
ordinary reviewed human integration.
Exact diff, predecessor reconciliation, CI, independent review and Greg's separate
acceptance precede the one additional human production merge. No second feature
candidate, verifier algorithm change or wider exception is implied.

An accepted and properly gated deferred alignment plan may coexist with the
initial R2/R3 disposition. Completion of this post-rename follow-up is not a
prerequisite to the original R5 transition; preparation remains separately gated
after R5, the authorized rename, verified canonical destinations, recorded old-route
assessments and Greg's separate order. This does not waive post-rename checks,
mandatory compatibility, recovery, R8 or external-authorization obligations.

Any resulting production commit/tree requires fresh post-merge verification and
reconciled or renewed dependent external authorization/checks before more external
writes or R8 completion. Beyond-boundary repair/recovery still requires §11's
separately reviewed plan and owner authority. Until adoption and those gates are
satisfied, keep the current links, make no final-alignment/completion claim and do
not recycle the original R5 evidence for changed content.

The exact eight current URLs are:

| Current README workflow link | Current badge image |
| --- | --- |
| `https://github.com/gregkrsak/klyxr/actions/workflows/build.yml` | `https://github.com/gregkrsak/klyxr/actions/workflows/build.yml/badge.svg` |
| `https://github.com/gregkrsak/klyxr/actions/workflows/tests.yml` | `https://github.com/gregkrsak/klyxr/actions/workflows/tests.yml/badge.svg` |
| `https://github.com/gregkrsak/klyxr/actions/workflows/verify.yml` | `https://github.com/gregkrsak/klyxr/actions/workflows/verify.yml/badge.svg` |
| `https://github.com/gregkrsak/klyxr/actions/workflows/docs.yml` | `https://github.com/gregkrsak/klyxr/actions/workflows/docs.yml/badge.svg` |

**Proposed operation:** Replace exactly the `gregkrsak/klyxr` slug in those
eight live URL attributes with `gregkrsak/bowfin`, and change the one
`repo_slug` constant in `scripts/verify-repo-scaffold.py` to the same value
in that content candidate. Keep workflow filenames, image/alt/layout attributes,
branch defaults and the existing verifier's rejection behavior. Do not change
those files during M6.

**Verification:** Individually verify the applicable new canonical URLs and test
and classify every applicable old-name URL under the rule above; preserve all
failed/unavailable/indeterminate outcomes and their compatibility/recovery and
completion consequences. The new content candidate then runs the ordinary
README/scaffold checks (including a missing mobile `srcset` negative control) and
applicable CI. Badges default to the default branch unless
qualified [S3]; because `prod` remains default, a staging badge or inherited
green image cannot stand for final production validation.

**Additional truthful metadata alignment, precisely scoped by §16.3:** Only the
opening `REPO_STRUCTURE.md` operational-home/adopted-destination/cutover-state
sentences before its first fenced tree; `BOWFIN_CONTEXT.md`'s two domain/legacy-site
lines under `Project identity` and its one adopted/operational-repository bullet
under `Brand system`; and BD-001's `Adopted primary domain` field are eligible.
The exact selectors and exclusions in §16.3 control, not the surrounding sections.
Do not alter the repository tree diagram, prototype/organization-transfer prose,
CLI/source-extension or legacy-retention promises, decision/status/history fields,
artwork/hash fields or unrelated migration prose. Use only verified current facts;
regenerate the single bundle solely from the permitted input changes with the
unchanged generator and five-input order. The existing bundle mirrors these
passages. Historical Issue #70/#72 links in
`LANGUAGE_DECISIONS.md`, their bundle copies, frozen reports, recorded commands
and hashes retain their evidence roles. They are not a bulk replacement list.

The separate governance repository and installed ChatGPT instructions need their
own owner-scoped decisions if live links need alignment. A compiler-content PR
cannot change accepted charters, installed instructions or their authority.

**Failure/recovery:** Stop preparation on an unverified applicable canonical
destination, a coupled-verifier failure, a historical rewrite or any other
unsatisfied §16 prerequisite. Assess an old-route failure under §16.2(3); it does
not by itself stop the bounded follow-up when the corresponding canonical
destination is verified and accepted content still requires correction. An
unsatisfied mandatory legacy obligation still blocks its applicable compatibility
or R8 completion claim; retain its failure and recovery assessment. Abandon an
unmerged content candidate, or obtain a separately
authorized ordinary revert/forward fix of its exact commit. This does not undo
A01 or any cached URL. No wildcard historical exemption is created.

## 8. A04 — HTTPS/SSH clone, fetch and local remote updates

**Operator/access/gate:** Greg/independent reader with their existing Git/SSH
access; A01 and R6 authorize these local updates. SSH uses the already trusted
GitHub host key and existing keys; never disable host-key checking to force a
pass. No push or access expansion is required for these checks.

Run all four combinations independently; do not infer SSH success from HTTPS:

```bash
for M6_NAME in klyxr bowfin; do
  for M6_TRANSPORT in https ssh; do
    if test "$M6_TRANSPORT" = https; then
      M6_URL="https://github.com/gregkrsak/$M6_NAME.git"
    else
      M6_URL="git@github.com:gregkrsak/$M6_NAME.git"
    fi
    M6_CLONE="$M6_EVIDENCE_DIR/$M6_NAME-$M6_TRANSPORT"
    git ls-remote --heads --tags "$M6_URL" | LC_ALL=C sort \
      > "$M6_EVIDENCE_DIR/$M6_NAME-$M6_TRANSPORT.refs"
    cmp "$M6_EVIDENCE_DIR/refs-before.txt" \
      "$M6_EVIDENCE_DIR/$M6_NAME-$M6_TRANSPORT.refs"
    git clone --no-checkout "$M6_URL" "$M6_CLONE"
    git -C "$M6_CLONE" fetch origin
    test "$(git -C "$M6_CLONE" rev-parse refs/remotes/origin/prod)" = "$M6_PROD_SHA"
    test "$(git -C "$M6_CLONE" rev-parse "$M6_PROD_SHA^{tree}")" = "$M6_PROD_TREE"
  done
done
```

**Expected:** Both names and transports expose the exact frozen heads/tags;
clone and fetch succeed, retain history, and resolve the same production tree.
These commands make only local evidence/clones, with no checkout or Rust
execution. No rehearsal clone/fetch was executed by this M6 preparation.

For each existing local clone, record its actual fetch URL and any explicit push
URLs privately before changing anything. Do not rename Greg's directories or
assume his owner-local paths changed. After confirming a single intended
`origin` URL, use the appropriate one of:

```bash
git remote set-url origin https://github.com/gregkrsak/bowfin.git
git remote get-url --all origin
git remote get-url --push --all origin
git fetch origin
```

```bash
git remote set-url origin git@github.com:gregkrsak/bowfin.git
git remote get-url --all origin
git remote get-url --push --all origin
git fetch origin
```

These are alternatives, not two sequential changes. An existing explicit
`remote.origin.pushurl` does not automatically follow the fetch URL: if there
is exactly one and it is the old GitHub location, Greg may separately update that
local value with `git remote set-url --push origin NEW_URL` under the named
local-update authorization. Multiple URLs, custom mirrors or credentials embedded
in URLs require per-clone inspection; do not overwrite or publish them blindly
[S2].

**Failure/recovery:** Stop on ref/tree mismatch, SSH/authentication error or a
different remote. Restore the saved local fetch/push configuration if needed.
Legacy URLs remain intentional compatibility probes; changing local configuration
does not change server history or prove the public redirect healthy.

## 9. A05 — Domain/routing activation, conditional on an approved provider plan

**Operator/access/gate:** Greg or a named operator with the existing registrar,
DNS and routing-service access; R5, explicit R6 action-by-action authorization,
M6-OP04–06 completed, A01/A02's canonical repository destination verified. No
provider, DNS address, certificate service or website implementation is selected
by this preparation.

The following is the **proposed smallest routing policy**, not current behavior
and not an approved deployment. It uses an existing HTTP forwarding facility only
if Greg establishes that it can satisfy the rules and TLS requirements.

| Surface | Proposed root destination after authorization | Retention / activation condition |
| --- | --- | --- |
| `https://bowfin-lang.com/` | `https://github.com/gregkrsak/bowfin` | Primary public project entry; canonical repository must first pass A01/A02 |
| `http://bowfin-lang.com/` | `https://bowfin-lang.com/` | Upgrade first, then primary root route |
| `klyxr.com`, `klyxr.online` roots, HTTP and HTTPS | `https://bowfin-lang.com/` | Retained controlled legacy entry points; never release the domains or remove compatibility merely because of rename |
| `bowfin-lang.org`, `bowfin-lang.net`, `bowfin-lang.info`, `bowfin-lang.online` roots | `https://bowfin-lang.com/` | Secondary redirect candidates only; exact registered names and each activation must be confirmed/named |
| Existing `www` variants of the seven probed names | Corresponding secure canonical root above | Public DNS currently resolves; exact support/TLS/retention decision required before activation |
| Non-root paths and query strings | **Not yet specified** | Existing route inventory and explicit per-path/query disposition required; no silent root collapse, dropping queries or invented website routes |

Initial activation should use a reversible temporary redirect (proposed 302), not
a permanent cache commitment. Greg must approve the status codes, slash/query
policy and exact Location values. A later permanent 301/308 conversion is a
separate named action after successful observation; it is not implied here.
An HTTP redirect requires an HTTP/TLS service; DNS alone cannot implement this
routing policy. The currently observed A answer is not a proposed production
record.

**Exact operation once inputs are supplied:** Attach the provider's existing
zone/routing export and an explicit before/after table for every changed record
or rule: account/service, hostname, record type or rule identifier, value/target,
TTL/status, TLS coverage and restore action. Greg reviews that table as part of
R6. In the existing provider UI, apply only those named changes, one dependency at
a time: establish/verify primary destination, then primary root routing, then
legacy entry points, then any individually authorized secondary/`www` routes.
Do not replace whole zones, nameservers, MX, TXT, CAA or unrelated subdomains from
this partial observation inventory. Do not deploy Pages or a new site as an
unreviewed alternative.

No paste-ready DNS-write command is possible without the provider and approved
record tuples. Supplying fictitious IPs, API endpoints or certificate commands
would make this runbook unsafe. The missing tuples are an explicit execution
block, not discretionary implementation work.

**Independent verification after each change:** From at least two ordinary
network/resolver views where available, query the approved DNS tuples and inspect
HTTP GET status/Location for each exact host, both schemes and approved paths.
Verify SNI certificate hostname/chain/expiry and renewal arrangement for every
HTTPS source before expecting its redirect to be delivered. A client must first
complete TLS even for a redirect. Test for loops, parking/ad pages, unrelated
destinations and mixed-content/login errors. Record times, headers, final URL,
TLS result, resolver/cache view and actual body meaning; do not use `curl -k`.

Prepared Linux read commands (not executed here; `dig` must already exist on
the operator's machine, otherwise use an existing read-only resolver facility):

```bash
dig +noall +comments +answer +authority bowfin-lang.com A
dig +noall +comments +answer +authority bowfin-lang.com AAAA
dig +noall +comments +answer +authority bowfin-lang.com NS
curl --silent --show-error --location --max-redirs 5 --max-time 45 \
  --proto '=http,https' --proto-redir '=http,https' \
  --dump-header "$M6_EVIDENCE_DIR/primary-domain.headers" \
  --output "$M6_EVIDENCE_DIR/primary-domain.body" \
  --write-out '%{http_code} %{url_effective}\n' https://bowfin-lang.com/
timeout 20s openssl s_client -connect bowfin-lang.com:443 -servername bowfin-lang.com \
  -verify_hostname bowfin-lang.com -verify_return_error </dev/null
```

Expand those checks to every authorized host, route, both HTTP/HTTPS schemes,
approved query case and a known non-root retention case. Check final URL/body
manually; a successful network command is not a route assertion. Record any
unavailable second network or certificate inspection as unavailable.

**Failure stop/recovery:** Stop downstream routes and announcements on any
mismatch, unresolved TLS, lost pre-existing service or inability to restore.
Use the saved exact provider records/rules under the authorized recovery scope,
or obtain Greg's decision if recovery was not authorized. Retain registration
and existing compatibility entry points; a restored parking route is a recovery
state, not a successful Bowfin destination. DNS TTLs/caches, certificate issuance
and permanent redirects may outlast a change. Do not promise instant global
rollback or use Git revert as DNS recovery. Loss of domain control requires
registrar/hosting escalation, not a guessed replacement.

## 10. A06 — Repository metadata, after its destinations are truthful

**Current → proposed:** description
`The Klyxr language. Systems programming you can reason about.` →
`The Bowfin language. Systems programming you can reason about.`;
homepage empty → `https://bowfin-lang.com/` **only after A05 passes**.

**Operator/access/gate:** Greg, repository administration; R6 explicitly names
these fields; A01 passed and the homepage destination verified. Other public
metadata is inventoried, not silently rewritten.

**Exact future operation:** In the renamed repository's **About → edit** UI,
change only the description and, once usable, Website/homepage fields. Leave
visibility, `prod` default, feature flags, topics, issue/PR state, merge rules,
social-preview artwork and permissions unchanged. If another existing metadata
consumer needs a change, stop and obtain a named reviewed action.

**Verification:** Independently GET the new repository metadata, require the
same ID and exact two intended values, inspect the rendered About panel and
follow the homepage in a fresh browser. Recheck unchanged default/visibility and
selected access. The repository REST documentation identifies description and
homepage as separate fields [S5].

**Failure/recovery:** Stop on an incorrect public claim or changed unrelated
setting. Restore the saved description/homepage values under Greg's recovery
authorization or forward-repair the specific value. Metadata can be restored;
cached previews and screenshots cannot be retracted by that action.

## 11. A07 — Existing/conditional Pages, wiki, hosted Actions and other consumers

| Facility | Applicability now | Future operation and independent verification | Stop / recovery |
| --- | --- | --- | --- |
| Pages/custom domain | Metadata says disabled; project URL 404; no tracked CNAME/Pages workflow. Administrative settings unavailable | Greg inspects Settings → Pages before rename. If still absent, record not applicable and do not enable it. If a real site/custom domain exists, pause for a specific site/source/domain/URL inventory and approved route/recovery table; project-site URLs do not inherit the repository redirect guarantee [S1, S6] | No invented CNAME, site or deployment. If existing service fails, restore its saved exact configuration under named authority; do not leave a dangling custom-domain route |
| Wiki | Feature enabled; public wiki request redirected to repository root | Greg inspects whether pages actually exist. If present, retain the same wiki/pages/history, test each saved old/new page URL and its separate wiki Git remote where used. Do not create a wiki or copy pages to another repository | Missing pages/history stops the claim; preserve the existing repository and use an authorized forward repair, not recreation of old repo name |
| Repository-hosted Action | No tracked Action definition or self-hosted Action consumer found | Not applicable in the inspected tree. Owner must check Marketplace/external consumers before rename. If one is discovered, stop and obtain a design compatible with retaining the old-name redirect; GitHub expressly does not redirect hosted-Action calls [S1] | Do not apply GitHub's alternative create/archive pattern where it conflicts with this contract's no-recreation rule; no Action platform is authorized |
| Webhooks, Apps, project boards, social preview | Public flags are not contents; administrative/external inventory unavailable | Greg exports existing entries, owner/consumer/URL, access and delivery expectations. Retain settings; inspect any automatic rename behavior individually. Any required change needs a specific operation and recovery record before R6 | Do not disable integrations, expand permissions or replace artwork to obtain a pass; restore only the saved changed setting under owner authority |
| Separate governance and installed instructions | Independent repository/installation surfaces | Keep `gregkrsak/klyxr-ai-governance` and current installed authority unchanged; test legacy compiler links that are actually used. Any later live-link edit needs its own authorized review boundary | No companion rename or installed-instruction update is implicit |

GitHub's Pages custom-domain documentation requires provider-specific setup and
warns about takeover risks from misconfigured/dangling domains [S6]. It is
reference guidance for a discovered existing Pages service, not permission to
create one. None of the conditional writes in this section is executable until
a reviewed row supplies the missing exact target, operator/access, R6 operation,
verification and restoration steps.

## 12. A08 — Releases, tags, source distribution and announcement preparation

### Existing distribution

**Operator/gate:** Greg and independent reader; read-only comparisons after A01;
any release/tag/package/publication write needs its own explicit authority.

The preparation inventory found zero visible releases/tags and
`publish = false`. Preserve that state unless a separately authorized release
plan says otherwise. Do not create a tag, GitHub release, registry package,
installer or binary distribution merely for rename testing. No Rust package
publication is included in this migration.

Existing source distribution is repository Git access, source files and GitHub's
generated source archives [S7]. After A01, test the old and new archive routes
independently, for example
`https://github.com/gregkrsak/klyxr/archive/$M6_PROD_SHA.tar.gz` and
`https://github.com/gregkrsak/bowfin/archive/$M6_PROD_SHA.tar.gz`.
ZIP may be checked with the analogous `.zip` route if used. Verify their
resolved status and extracted source against the exact production Git tree.
Top-level archive directory names, compression and headers may differ; do not
claim whole-archive byte identity from source equivalence. This M6 preparation
did not download/extract archives.

If owner inspection discovers a real existing release/draft/asset or external
download consumer, inventory its exact ID/tag, URLs, signature/digest role and
consumers before any change. Keep historical asset bytes and issued identifiers.
A Git revert cannot retract a downloaded artifact; a missing consumer route
requires an explicit forward repair or a truthful availability notice.

### Announcement draft — not published

Proposed copy, usable only after the corresponding facts are independently
verified and Greg authorizes the named channel and exact text:

> Klyxr is now named Bowfin. The current compiler prototype source is at
> https://github.com/gregkrsak/bowfin, with https://bowfin-lang.com/ as the project
> entry point. The retained klyxr command and explicit .klx source input remain
> supported. This is an identity migration, not a new language-feature or native
> backend release. See the repository's current documentation for implemented
> behavior and limits.

No existing external announcement channel was established by this preparation.
GitHub Discussions is disabled; do not enable it, invent a mailing list/social
account, or manufacture a release to supply a channel.

**Operator/access/gate:** Greg or his named publisher with existing access to an
explicitly authorized channel. R4/R5/R6 must be complete; A01/A02/A04/A05 and
applicable consumer/link/metadata checks must support every statement. Record
remaining limitations. Do not say “cutover complete” before the R8 checklist
passes; an alias-first preview would require a separate owner decision.

**Verification:** Independent reader follows every published link from a clean
client and compares exact text, timestamp and channel against the authorization.
Record the actual publication URL/ID. No package/download or installed-tool
availability claim is permitted without separate evidence.

**Failure/recovery:** Stop publication if any claim is premature. Correct the
same published item or issue a clearly linked correction under Greg's named
authority; do not rewrite old historical reports. Cached/shared copies may remain.
Withdrawing an announcement cannot retract an emitted package or granted authority.

## 13. Post-cutover checklist, stops and recovery boundary

The order after R6 is: A01 rename; A02 immediate identity/routing/connector
checks; A04 Git checks; A05 primary then legacy/optional domain routes; A06
truthful external metadata; A03 URL/content alignment only if proposed Amendment 4
has been adopted and all of its separate follow-up gates are satisfied;
A07 applicable-consumer checks throughout; A08 source-distribution checks and
any specifically authorized announcement. Reconcile every changed surface before
R8. This dependency order does not authorize optional writes or override missing
prerequisites. Amendment 4 remains proposed/pending in this candidate; the current
§3 rule still controls, M6-OP07 remains open and final R2/R3 disposition awaits
acceptance and verification of the resulting staging integration.
After adoption, the properly gated deferred alignment plan can coexist with the
initial R2/R3 disposition: finishing the later follow-up is not required for the
original R5 transition. Post-rename route assessment, mandatory compatibility,
recovery, R8 and reconciliation/renewal of dependent external authority remain
required; an old-route failure is never converted into a successful result.

Greg and the independent checker record actual time/result/evidence/recovery
status for every applicable item:

- Same repository ID/owner, all saved Git heads/tags, complete saved issue/PR
  identities/history, default branch and effective restricted access.
- Old/new corresponding web and discussion pages; each raw, workflow, badge,
  archive and applicable hosted-consumer route tested separately.
- Both HTTPS and SSH clone/fetch paths at the fixed production SHA/tree; intended
  local remotes updated without changing Greg's local directory identity.
- Fresh production CI evidence tied to its actual commit/tree and appropriate
  event; badges do not replace logs. Any authorized content alignment has its
  own exact revision/CI/review record.
- Every authorized domain/source scheme, final destination, certificate,
  redirect chain, path/query policy and retained old service; unavailable
  observations explicitly unresolved.
- Correct description/homepage, individually checked applicable Pages/wiki/
  Action/integration surfaces, preserved separate governance/installed authority.
- Truthful distribution and announcement status, including “none published” where
  no publication was authorized.

A missing or indeterminate mandatory check is not a pass. Stop completion and
downstream claims, retain compatibility control and existing authoritative state,
and ask Greg for the narrow operational decision. Use provider-specific recovery
for external state; use a separately authorized ordinary Git revert/forward fix
for repository content. Neither recovers the other, clears caches, retracts
downloads nor manufactures approval.

Before R4, this unmerged preparation PR may simply be abandoned. If integrated,
an ordinary authorized revert of its one document invalidates dependent
preparation and requires re-review; it does not “roll back” public state because
this preparation changed none. After external execution, prefer bounded forward
repairs and retain the old names' routing; never recreate the old repository name
or silently remove domain/command/source compatibility.

**Preparation disposition:** reviewable runbook with unresolved operational
prerequisites. No live cutover, M7 start, production acceptance, merge,
completion declaration or external publication is authorized by this document.

## 14. Primary documentation

The following primary sources were consulted on 2026-11-01. Their documented
behavior is distinct from §2's observations and §§4–13's proposed future steps.
Refresh the applicable documentation before acting; preserve the execution-day
observations rather than treating this snapshot as eternal.

- **S1:** [GitHub — Renaming a repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/renaming-a-repository).
- **S2:** [GitHub — Managing remote repositories](https://docs.github.com/en/get-started/git-basics/managing-remote-repositories), [Git — git-remote](https://git-scm.com/docs/git-remote), [Git — git-ls-remote](https://git-scm.com/docs/git-ls-remote).
- **S3:** [GitHub — Adding a workflow status badge](https://docs.github.com/en/actions/how-tos/monitor-workflows/add-a-status-badge).
- **S4:** [GitHub — Viewing and understanding files](https://docs.github.com/en/repositories/working-with-files/using-files/viewing-and-understanding-files).
- **S5:** [GitHub REST — Repositories](https://docs.github.com/en/rest/repos/repos), [GitHub — Reviewing installed Apps](https://docs.github.com/en/apps/using-github-apps/reviewing-and-modifying-installed-github-apps).
- **S6:** [GitHub Pages — Managing a custom domain](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site).
- **S7:** [GitHub — Downloading source code archives](https://docs.github.com/en/repositories/working-with-files/using-files/downloading-source-code-archives).
- **Observed DNS source:** [Google Public DNS JSON query for the primary A record](https://dns.google/resolve?name=bowfin-lang.com&type=A); the same endpoint was queried for the names/types in §2.2. A later response may differ.
