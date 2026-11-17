# B38-X01 evidence and cost report — 2026-11-03

**STOPPED: required all-task disk-confinement verification is incomplete.** This is not a compatibility pass. Resource-only run [41679554939](https://github.com/gregkrsak/bowfin/actions/runs/41679554939) returned success for its committed checks; subsequent inspection of its mount record found writable mount surfaces outside bounded scratch, including /run/lock and /dev/mqueue, absent from those checks. No overrun or use of those paths is demonstrated. Do not treat that workflow conclusion as certification of the 10 GiB boundary.

Authorization: [6609874959](https://github.com/gregkrsak/bowfin/issues/117#issuecomment-6609874959); controlling assignment: [6609510272](https://github.com/gregkrsak/bowfin/issues/117#issuecomment-6609510272). Both and Draft 2, Sarah's complete review, the preserved pressure report, current charter, baseline AGENTS.md and canonical context were read before edits/CI. Branch absence and input identities were verified before creating this branch directly from the compiler baseline. No identity drift was found; the Draft 2 body remained unchanged at closeout.

Compiler commit/tree: `fb67a7608ab75d9b6721e91b58dbf61e8edaace1` / `c9e5047eaec9430698cd7a82c1d05205d729994f`. Separate design commit/tree: `efb636de48c7b7b6c412aa30520e112329baed80` / `5e2376650d6306e127cfe107c98e7f286aa652e8`. Draft 2: 59,340 UTF-8 bytes, SHA-256 `31abb5a224a8c877d64749cc548588a3910d7df400bf99c7411b58e60c53dd85`.

Launch manifest, independent question/expectation plan, source pins and hardening assessment preceded metadata collection at commit/tree `30bb7c86122cdeef52a53883aa206b5b49ec04e2` / `0e77b00e1e496c9dd6ce1209c9e6216ee275ed6e`. Workflow SHA-256: `da28238ff1987173b7bf718faeefce89fca5d3b33aa83337f746fa38bd1d301f`. This report/index is a later append; the launch inputs and earlier evidence remain unchanged.

[Input resolution](../../experiments/bed-038/x01/input-resolution.json) records primary identities for LLVM 18.1.8 (commit 3b5b5c1ec4a3095ab096dd780e84d7ab81f3d7ff), GCC 14.4.0 (98859340e0916f004ac78d5fe20549fb100d41aa), binutils 2.43.1, glibc 2.39 (ef321e23c20eebc6d6fb4044425c00e6df27b05f), Rust 1.80.0 and the permitted object/memchr candidates. LLVM/glibc archives were streamed and hashed (150,588,248 bytes total); GCC/binutils checksums were retrieved but archive bodies were not verified. No signature or archive/Git-tree equivalence is claimed.

The selected Ubuntu amd64 manifest is `sha256:78d4744ead1f5b19b04d0df558ab182a5300903900755e515b2ca4d10eb2044d`; index and manifest bodies were verified. Layer/config bodies, installed image package closure, patched libc, build options, sanitized environment and complete producer argv remain unestablished. No candidate image was pulled, built or run.

Observed experimental runner: ubuntu24 image 20261101.360.1, kernel 7.0.0-1017-azure, four allocated vCPUs, MemTotal 16,366,512 kB, MemAvailable 15,434,088 kB, free disk 92,123,287,552 bytes. Effective group cpuset 0-1, cpu.max 200000/100000, memory.max 6,442,450,944 bytes, memory.swap.max 0; runtime ceiling 240 seconds. Fixed ext4 backing 10,603,200,512 bytes plus 128 MiB reserved for checkout/evidence/upload was intended to cover 10 GiB. The mount audit prevents claiming that aggregate ceiling was enforced.

Read-only rejection was actually checked at /tmp, home, checkout and /dev/shm. The mounted root was read-only; /run and other API mounts remained writable. No image build/daemon storage was introduced. Group log reports 871 ms runtime and 141 ms CPU; its 256 KiB memory-peak output is not accepted as a reliable process-tree peak.

Observed bootstrap executables, absolute/resolved paths, SHA-256, version statuses and dpkg inventory are retained in the artifact. GCC/G++ were Ubuntu 13.3.0-6ubuntu2~24.04.1; as/ld were Ubuntu binutils 2.42; CMake 3.31.6, Ninja 1.13.2, Make 4.3, Python 3.12.3. These are metadata observations, not substitutions for the selected candidate tuple. Rustup 1.29.2 reported Rust 1.80.0 absent; no Rust installation or compilation occurred in the X01 experimental group.

| Required probe | Executed result |
| --- | --- |
| Actual selected producers; Clang/wrapper/fallback rejection | Not run; bootstrap paths/hashes only |
| Restricted Cargo closure on Rust 1.80 debug/release | Not run; zero new dependencies |
| Widened arithmetic, ordinary/boundary controls | Not run; independent mathematical predictions retained |
| Enumerated POD layout, wide out-parameter, callback/borrow controls | Not run; concrete fixtures and exact offsets still owed |
| LLVM/C unoptimized/optimized layouts, ELF/relocations/imports/features | Not run |
| Same-input product repetition and whole-image reproduction | Not run; no product/image digest exists |

[Hardening assessment](../../experiments/bed-038/x01/HARDENING.md) distinguishes primary-source-supported GCC strong stack protection/full CET and LLVM strong stack/branch mechanisms from unverified equivalence, host features, defaults and support imports. Neither forbidden disabling flag was used. Possible stack-guard TLS/__stack_chk_fail implications are unresolved, not an observed integration conflict. Preserve protections and return any demonstrated conflict for review.

Artifact 13252985712: 17,414-byte ZIP, SHA-256 `74c509ba7bff2a810a878be06ac89c29277d21ba6976c68031f9682b9355c82f`, expires 2026-12-03T16:58:27Z. [Committed index](bed-038-x01-evidence.json) contains member hashes, runner/tool metadata, all pre-closeout run/job timings and source-file hashes. Restore the artifact from the run's Artifacts panel or authenticated Actions artifact API, verify the ZIP hash, extract without executing contents, then verify sha256.json. Expiry/missing bytes require re-establishment; this artifact is not an X02 toolchain input.

Pre-closeout cost: 13 workflows / 14 jobs, including branch creation, automatic push/PR CI and metadata run; 642 job-seconds × four vCPUs / 60 = **42.800 proxy CPU-minutes**. Four-vCPU allocations were observed on both experimental and baseline runner labels; allocation for each ordinary job was not independently instrumented, so the same-class proxy is a limitation. No failures/cancellations in X01. Reserve 36 further proxy minutes for this report/index commit's automatic CI; append actual settled timings on the draft PR. No budget is reset.

Conservative handwritten account before closeout: 421 live nonblank lines INCLUDING comments/headings, plus 32 charged for discarded/revised child lines and 3 root edits = at most 456. Report/index construction is capped at another 75, cumulative at most **531/600**; generated evidence rows/hashes/timings are not handwritten source. Unsafe/FFI 0/30; new Cargo packages 0/2; image attempts 0/2; native/executable probe attempts 0. Human engineer-time was not supplied or inferred from AI wall time; no human-effort measurement is claimed.

All existing workflows, manifests, locks, production compiler/runtime code and predecessor evidence are unchanged. Existing candidate debug/release CI reconciled all 2,262 exact identities across 40 targets, not merely a historical count; see compiler-tests job 126069282599. This says nothing about unexecuted X01 probes. No local Rust/native input execution, Clang, loader, emitter, registry publication or merge occurred.

| Actual component / F6 | Expected successor cost; uncertainty |
| --- | --- |
| Source/input manifest — extend | 2–6 hours to finish bootstrap/support identities; actual archive and package verification still required |
| Resource launcher — extend, discard incomplete confinement claim | 2–4 hours and about 20–35 changed lines for a complete writable-mount policy and negative checks; unexecuted estimate |
| Metadata collector — reuse within this hosted scope | 1–2 hours review per runner-image change; banners alone do not authenticate a candidate producer |
| Independent probe plan — extend | 4–8 hours for exact C/IR/Rust fixtures, layouts and discriminating controls; no executable fixture yet |
| Hardening assessment — extend | 2–4 hours selected-object testing/review; return-CET and imports remain unresolved |
| Workflow/artifact ledger — extend | 1–2 hours per revised launch for accounting/restore verification; no automatic adoption |

Smallest next proposal: preserve current evidence, make /run plus subordinate storage mounts (including /run/lock and /dev/mqueue) read-only or explicitly quota-accounted, audit every remaining writable storage surface, and verify actual write rejection/quotas before further execution. Keep all numerical ceilings and protection requirements. A reviewed resumption must also complete the existing exact build/probe launch prerequisites; if that exceeds the remaining cumulative line/CI allowance, request a quantified scope amendment rather than compressing checks.

Disposition: incomplete compatibility investigation, stopped at required enforcement verification. This leaves X01's dependency/ABI/toolchain/reproducibility gates open, closes no finding, and supplies no freeze or production-integration evidence. PR #120 remains draft and unmerged; X02/X04 remain unauthorized.

