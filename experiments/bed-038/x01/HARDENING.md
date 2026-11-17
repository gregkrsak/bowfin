# B38-X01 hardening input and unresolved gates

Status: static source assessment only. No compiler defaults, object imports,
CPU capability, runtime protection or executable result has been measured.
This file is not a complete build/probe launch manifest.

The owner forbids `-fno-stack-protector` and `-fcf-protection=none`.
Preserve applicable protections; do not remove one to satisfy the proposed
zero-import policy. No emitted-product or loader compatibility is asserted.

## Candidate protection mechanisms

GCC 14.4.0's exact `gcc/doc/invoke.texi` documents
`-fstack-protector-strong` for local arrays and referenced local frame addresses.
It documents `-fcf-protection=full` as branch plus return instrumentation on
the supported x86 GNU/Linux implementation. Those two positive options are
candidate probe inputs, not substitutes for inspecting actual compiler defaults,
effective features, emitted code, GNU properties and support imports in both modes.
Do not claim protection from a flag alone or use a probe whose protected object
is optimized away as the only positive control. No out-of-bounds or other UB
is needed to check instrumented object structure.

LLVM 18.1.8's `Attributes.td` names `StackProtectStrong` as `sspstrong`;
`StackProtector.cpp` handles that function attribute with the strong heuristic.
`X86IndirectBranchTracking.cpp` checks module flag `cf-protection-branch` and
adds ENDBR at eligible indirect targets. These are source-supported candidates.
Full return-CET/GNU-property generation, exact module-flag values, tool options
and target/host compatibility have not been established. Do not label the branch
mechanism a verified equivalent of GCC's full branch-and-return protection.
No Clang frontend invocation, Rust rebaseline or unstable Rust option is proposed.

Stack guards can require TLS guard access and a failure support symbol such as
`__stack_chk_fail`; inspect actual objects rather than assuming an empty import
set. A conflict between required protection and the zero-import candidate is a
result to return for a reviewed amendment. It is not permission to invent a
replacement failure implementation, omit instrumentation or proceed with a loader.
X01 authorizes object inspection and tiny ABI probes, not installation or custody.

## Launch remains blocked by missing inputs

`input-resolution.json` pins known primary source/archive/registry identities.
It does not establish the actual Ubuntu package closure, patched libc build,
bootstrap compiler binaries, build tools, complete exact argv/environment,
finite committed probes/expectations or enforced Actions resources.
Do not start the image or executable probes before those conditions are complete.
Keep a resource-only preflight distinct from an image-build attempt.

For enforcement, verify a two-CPU cpuset plus quota, the effective memory ceiling
and zero swap, and a task-owned disk quota or fixed filesystem covering all inputs,
scratch, caches, image state and outputs. Docker daemon layers/BuildKit state count;
`TMPDIR` and container CPU/memory flags alone do not enforce the disk cap.
Do not reclaim unrelated runner storage or reset shared Docker state.
One initial image build plus one diagnosed repair does not authorize a second
successful whole-image build merely to claim independent reproducibility.

## Primary source evidence

- GCC source: https://github.com/gcc-mirror/gcc/blob/98859340e0916f004ac78d5fe20549fb100d41aa/gcc/doc/invoke.texi
- Retrieved GCC document SHA-256: `0143ce9c93096ae1cfe92de934d5e0797b038afcfe335cdc6fa55ff3e8cf7c78`.
- LLVM attributes: https://github.com/llvm/llvm-project/blob/3b5b5c1ec4a3095ab096dd780e84d7ab81f3d7ff/llvm/include/llvm/IR/Attributes.td
- LLVM stack pass: https://github.com/llvm/llvm-project/blob/3b5b5c1ec4a3095ab096dd780e84d7ab81f3d7ff/llvm/lib/CodeGen/StackProtector.cpp
- Retrieved stack-pass SHA-256: `d4c76cbf0fa7106976907234ea31ea7945cafa60a56c9bc8d75a3a5b8595cc15`.
- LLVM branch pass: https://github.com/llvm/llvm-project/blob/3b5b5c1ec4a3095ab096dd780e84d7ab81f3d7ff/llvm/lib/Target/X86/X86IndirectBranchTracking.cpp
- Retrieved branch-pass SHA-256: `20c454c397d70494e710bcd91a1c0f61359a6e9efe021c1706553645b645801b`.
- Docker limits: https://docs.docker.com/engine/containers/resource_constraints/
- Runner baseline documentation: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
