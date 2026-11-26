# Contributing to Bowfin

Bowfin is a young systems-language project whose design, compiler, evidence, and
documentation are reviewed as one assurance story. Careful reasoning matters
more than churn.

## Before contributing authored material

The Bowfin Project requires a signed **Contributor Assignment Agreement** before
it may merge any externally authored code, tests, documentation, examples,
artwork, specifications, or other copyrightable material. There is no small-patch
exception.

The current project rights-holder is **Gregory M. Krsak**, individually. Bowfin
uses assignment agreements based on the Harmony Contributor Assignment Agreement,
Version 1.0, with outbound-license Option Five:

- [Individual Contributor Assignment Agreement](legal/INDIVIDUAL_CONTRIBUTOR_ASSIGNMENT_AGREEMENT.md)
- [Entity Contributor Assignment Agreement](legal/ENTITY_CONTRIBUTOR_ASSIGNMENT_AGREEMENT.md)

Under Option Five, the rights-holder may license an accepted contribution under
any additional license, including commercial or proprietary terms, while also
making that contribution available under the license or licenses applied to the
Bowfin material on the contribution's submission date.

Until an executed agreement is on file, you may open issues containing ideas,
questions, reproduction steps, and factual bug reports, but do not submit patches,
implementation text, documentation prose, artwork, or other authored material
intended for inclusion in the repository.

## Agreement process

1. Determine whether the contribution is owned by you or by an employer or other
   legal entity.
2. Open a repository issue requesting private contributor-agreement submission
   instructions. Do not post a signed agreement, home address, or other personal
   information publicly.
3. Execute the applicable agreement and wait for written confirmation that it is
   on file.
4. Only then open a pull request containing authored material.

An individual contributor whose employer may own relevant work must obtain the
necessary employer authorization or use the Entity agreement. The person signing
an Entity agreement must have authority to bind that entity.

A pull-request checkbox, commit sign-off, or Developer Certificate of Origin is
not a substitute for the executed assignment agreement.

## Ownership and provenance

Submit only material that you have the legal authority to assign. Identify any
third-party material before submission and do not include it unless the project
owner has approved its license and provenance in writing.

Disclose material use of generative-AI or code-generation tools in the pull
request. The disclosure must identify the tool and the affected portions. Do not
submit generated material when you cannot make the ownership, authority, and
provenance representations required by the contributor agreement.

Never submit confidential information, credentials, private keys, controlled
data, export-restricted material, or material subject to an incompatible
agreement.

## Engineering process

Before substantial work:

1. read `AGENTS.md`;
2. use the repository branch and canonical inputs it identifies;
3. distinguish accepted decisions from proposals and open questions;
4. preserve explicit trust, ownership, effect, and evidence boundaries; and
5. include documentation and focused tests with behavior-changing work.

GitHub Actions is the authoritative executable environment where the repository's
current instructions say so. A green check does not replace specification review,
independent review, owner acceptance, or human merge authority.

Commit messages use the repository's Conventional-Commit-style lowercase prefix,
for example:

```text
docs: clarify native target qualification
fix: preserve ownership across conditional value joins
```

Only a human performs the final merge.

## Review expectations

Contributions should favor:

- precision over hype;
- clarity over cleverness;
- explicit authority over convention;
- useful diagnostics over compiler mysticism;
- bounded, reproducible evidence over implication; and
- honest limitations over sweeping guarantees.

The project may decline, defer, rewrite, or omit a contribution even after an
agreement is executed. Executing an agreement creates no promise of acceptance,
credit, support, or future project role.

## License and marks

Bowfin is distributed under `Apache-2.0 WITH LLVM-exception`, except where a file
or directory clearly states different terms. See [LICENSE](LICENSE),
[NOTICE](NOTICE), [TRADEMARKS.md](TRADEMARKS.md), and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

The software license does not grant permission to represent a fork, modified
distribution, or independent implementation as an official Bowfin product.
