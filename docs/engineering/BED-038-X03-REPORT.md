# B38-X03 evidence and cost report — 2026-11-03

**STOPPED at unchanged Sentinel-companion admission failure:** `ConfiguredLimit("Result outcome storage reservation")`, origin START, expression None, empty trace. Computational admission/inventory succeeded first. No fixture/checker/limit was changed after the substantive failure; no interpreter comparison or source execution ran in the X03 experimental group.

Authorization [6609874959](https://github.com/gregkrsak/bowfin/issues/117#issuecomment-6609874959), assignment [6609513223](https://github.com/gregkrsak/bowfin/issues/117#issuecomment-6609513223), Draft 2, Sarah's complete review, preserved pressure report, current charter, baseline AGENTS.md and canonical context were read before editing/CI. Branch absence and identities were verified before creating this branch directly from baseline; no material identity drift was found. PR #118 was not incorporated.

Compiler commit/tree: `fb67a7608ab75d9b6721e91b58dbf61e8edaace1` / `c9e5047eaec9430698cd7a82c1d05205d729994f`. Separate design head/tree: `efb636de48c7b7b6c412aa30520e112329baed80` / `5e2376650d6306e127cfe107c98e7f286aa652e8`. Draft 2 remained 59,340 UTF-8 bytes, SHA-256 `31abb5a224a8c877d64749cc548588a3910d7df400bf99c7411b58e60c53dd85`.

Static expectations and admission-only launch files were committed at `ee7b4a17ef95d78bf560f86af706694360da9718`, tree `1b5649dcca4a66be32e863b6b58fc19594117997`, before collecting results. One diagnosed infrastructure repair produced `7c01f0b9a998fe97ad665fef86b60f1118a6b2e5`, tree `cd992c6bb96062089d53d03b4651993b9c9627a4`; its workflow SHA-256 is `526ab098b65b3c26bc52c77f3d2b9ab8d6ac32b7cb1d2356167c7525597dfe73`. This report/index is a later append.

| Exact source | Bytes / identity |
| --- | --- |
| computational.bow | 947; SHA-256 1c9a863981ec28467de4f1850ba797a6ad19652fb6613897d94ceae402d4fe42 |
| Unchanged examples/sentinel.bow | Git blob 7fbd56a828e3110c60f31c1d879644befe085526 |
| companion-suffix.bow | 713; SHA-256 ba74eceee3661196f85e89d21aebc41c52a009fe6bc876822cb353abfbfaa055 |
| Sentinel + one LF + suffix | 7,866; SHA-256 c0c3725889590b60a1df713f4d25a252f0774b1eab4373a7e8256cbb4ca02934 |

[EXPECTATIONS.md](../../experiments/bed-038/x03/EXPECTATIONS.md) preceded admission and records independently source/protocol-derived application results, structural closures, scenarios, threshold neighbors and fuel hypotheses 1675/18/4/8. These fuel predictions are not executed or frozen schedules. It explicitly forbids comparison until exact independent schedules are committed; that later stage was never reached.

First run [41678693408](https://github.com/gregkrsak/bowfin/actions/runs/41678693408) stopped before Rust compilation: rustup attempted self-update under read-only /home/runner/.cargo. Repair added --no-self-update without weakening confinement, changing sources or changing expectations. Two ordinary build jobs also rejected initial Rust formatting; the exact formatting was repaired. Both failed experiment artifact and automatic CI failures remain in the ledger.

Second run [41679108551](https://github.com/gregkrsak/bowfin/actions/runs/41679108551) installed/hashed Rust 1.80.0 (051478957371ee0084a7c0913941d2a8c4757bb9), host x86_64-unknown-linux-gnu, bundled LLVM 18.1.7. Restricted admission-only test ran in debug with the unchanged frontend and sealed-contract validator; all four computational structural-closure assertions passed:

| Root / FunctionId | Complete ordinary structural closure, IDs in extracted order |
| --- | --- |
| compute / 2 | compute, seed, step / [2,0,1] |
| trap / 3 | trap, seed, step / [3,0,1] |
| short / 4 | short, step / [4,1], including the unreached RHS callee |
| wide_trap / 6 | wide_trap, wide_seed / [6,5] |

Partial inventory contains HIR/MIR, original origins, trusted facts, outcome/cleanup/Result plans, sites/operands and four validated contracts. It is bounded labeled Debug text, not the completed machine-readable workload manifest or a complete phase/custody oracle. Original IDs remain in the artifact; no new IDs, variants or fields were added to accounted types.

| Extracted version-one schedule family | Quantities in label order from inventory, bytes except fuel unit |
| --- | --- |
| machine | 88,136,48,64,56,24,40,48 |
| cleanup | 360,16,64,40,24,96,16,56,176 |
| results | 72,176,16,72,48 |
| temporaries | 56,136,64,48 |
| trusted | 528,24,80,24,16,56,144,24 |
| observation | 112,128 |
| comparison | 240,56,224,56,24,16,224,16 |
| extraction | 72,8,16,544 |

Semantic/accounting/cuts/observation/replay versions are 1; compiler=0.0.1 and fuel_unit=1 logical action. Inventory records exact index labels, units and original contracts. No schema/schedule migration was made.

Admission configuration: source_bytes=65536, tokens=4096, identifier_bytes=128, frontend_work=10000000000000000, traversal_depth=24, alias_depth=48, alias_work=1000000, working_storage=1073741824, diagnostic_storage=4194304, rendered_output=65536. Sentinel selection was sensor-demo/Sample, Grants::ALL, roots monitor and b38_probe, live-Sensor limit1; relay helpers were not public live-resource roots. These were admission inputs, not executed transcripts.

Unchanged combined source failed during Result reservation before extracting either custody-root contract. The test returns immediately on that error. It contains no session construction/advancement; the trailing MACHINE_STARTS assertion was not reached and is not claimed as executed evidence.

Static diagnosis, not another execution: execution/mod.rs passes actual token count to bounded.rs; reserve_outcomes uses n=max(envelope_nodes,holders,1). Lexer-equivalent source counting gives n=689 including EOF, cleanup holder bound K=10, outcome holders H=34. With B=7866, I=128, D=24, base=(16D+32)n(I+512)+8B=183502288; cleanup=256nK(16D+16)=705536000; outcomes=512n²+128DH=243161600. Their sum alone exceeds 1 GiB, before positive catalog storage. Reducing only the token ceiling cannot eliminate that lower bound.

Using the existing 1320-byte completion descriptor quantity and catalog reservation 5488896 gives an unexecuted initial total 1137688784 at D24; D23 predicts 1102407632; D22 predicts 1067126480 (6615344 below 1 GiB). Depth22 is a one-field candidate, not demonstrated admission success: source traversal depth and later reservations still require justification and checking. No depth/limit change was applied.

Unanswered: complete custody inventories, phase/invocation/ancestry/occurrence mappings, exact positive-fuel/depth/expression/storage neighbors, partial transfers, completion/release/publication cuts, response-consumption controls, relay discrimination, final token/holder/protected-Result censuses, and repeated terminal advances. None of the normal/error/threshold/release scenarios was interpreter-compared in either build mode. No release workload extraction or cross-check ran. Baseline regression execution is separate from these missing fixture comparisons.

Observed second runner: ubuntu24 20261026.354.1, kernel7.0.0-1017-azure, four allocated CPUs, MemTotal16367520kB, MemAvailable15454660kB, disk91881533440 bytes. Effective group CPU quota200000/100000, cpuset0-1, memory.max4294967296, swap.max0; fixed4GiB scratch and720-second group ceiling,20-minute job ceiling. Group runtime27.147s and reported CPU36.194s are retained; 256KiB reported peak is not reliable compiler-tree memory evidence.

**Enforcement discrepancy:** source audit found the same incomplete mount policy as X01, whose actual mount record leaves /run/lock and /dev/mqueue outside scratch. X03 captured no mount tree and tested write rejection only at /tmp. Therefore full all-task 4GiB storage enforcement was not established before compilation, despite successful committed CPU/memory checks. Preserve the observations and this launch-verification failure; do not certify compliance, silently rerun or weaken the ceiling. The missing audit should have blocked compilation.

| Retained artifact | ZIP SHA-256 / expiry |
| --- | --- |
| Attempt1, 13252635601, 2236 bytes | 39c0660cd4128c16d66d8f88104fedde1bbc842425573bda2a4c213cfbce94b9 / 2026-12-03T16:53:20Z |
| Attempt2, 13251298236, 18813 bytes | d407dd7c22ac1a84e735bc872620ce9bd926a20de406cb6dcdb445b723279539 / 2026-12-03T16:56:09Z |

[Committed index](bed-038-x03-evidence.json) records member/source hashes, all pre-closeout runs/jobs, resources and artifact restore information. inventory.txt is157999 bytes, SHA-256 22dafec3df2df1db2bf9dbe889db29f102e4344385cd759bcb6bec49186ab25a. Restore each ZIP from its run/Actions API, verify ZIP digest, extract without executing contents, then verify sha256.json; expired bytes require re-establishment.

Pre-closeout cost:22 workflows/24 jobs,1031 job-seconds×four vCPUs/60=**68.733333 proxy CPU-minutes**, including automatic branch/push/PR CI, two failed experiment attempts and two formatting failures. Allocation4 was observed on experimental and baseline runner labels, not independently inside each ordinary job; same-class proxy is a limitation. Reserve36 more minutes for report/index automatic CI; settle exact closeout costs append-only on the PR. Zero cancellations, new dependencies or unsafe/FFI; no local Rust/native execution.

Conservative pre-closeout handwritten account≤520:470 live ALL nonblank lines including parent declaration, replace module+parent component155 by cumulative allowance200 (covers discarded/reformatted drafts), plus5 for other discarded/repair uncertainty. Report/index construction≤75 further lines, cumulative≤**595/600**; generated evidence rows/hashes/timings are excluded. No measured human engineer-time was supplied; AI wall time is separate and does not certify that account.

Existing workflows/semantic code/manifests/locks remain unchanged; the only existing compiler-file change is the permitted test-module declaration. The experimental cfg adds no tests to ordinary predecessor runs. Candidate debug/release logs in job126067681247 reconciled all2262 exact identities/40targets against unchanged identity inventory SHA-256 d19fea5f852d74c6c3552e0c3e52d8c195da5a39419c0a77df04b4878bcde36a. The selected admission-only experimental test is separately qualified; it is not predecessor or native evidence.

F6: exact fixtures are reusable identity inputs (0.5–1h review after any approved configuration revision); private extraction seam is extensible (2–4h to machine-readable completeness and finite access review); static expectation tables are extensible (4–8h for complete independent phase/storage/custody schedules); CI/confinement/index plumbing is extensible but its incomplete disk-compliance claim is discarded (2–4h correction/audit). All are successor estimates, not measured savings or adoption. No general lowering/runtime/native machinery exists here.

Smallest next proposal: preserve failure and exact source hashes; review a bounded admission-configuration investigation, with D22 justified as a candidate while keeping 1GiB storage and accepted semantics unchanged, then require a new committed launch/configuration and complete disk audit before any rerun. If later reservations or remaining line/CI allowance cannot fit, return that result for a quantified amendment. Never repair the fixture/checker to obtain admission.

Disposition: X03 incomplete, stopped; no complete independent schedule/cross-check gate is discharged. PR #119 remains draft/unmerged, no finding is automatically closed, BED-038 is not frozen, and X02/X04 remain unauthorized.

