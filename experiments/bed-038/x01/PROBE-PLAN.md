# B38-X01 independent probe plan, revision 1

Current stage: resource-only preflight and installed-bootstrap metadata.
No image or arithmetic/ABI/dependency probe is launched by this revision.
Known inputs and missing build prerequisites are in `input-resolution.json`.
Collected bootstrap facts may parameterize a later exact launch manifest;
their collection is not certification of the candidate toolchain or ABI.

## Independent expectations to preserve before later results

| Mechanism | Expected result or relationship | Independent rule |
|---|---|---|
| Widened arithmetic | 999 minus -3 is 1002, outside nominal 0..1000 | Integer mathematics; BD-041 |
| Widened endpoint | i64 minimum minus 1 is -9223372036854775809 | Integer mathematics; widen before subtraction |
| ABI scalar header | Explicit u32/u64 fields retain enumerated offsets and 8-byte alignment | System V AMD64/private explicit-layout proposal; later exact struct required |
| Wide out parameter | Low/high words reconstruct the full mathematical result | Explicit two-word representation; no Rust internal layout |
| Callback | One callback receives the same context and returns explicit integer status | Fixed signature and bounded borrow interval; no retained exclusive borrow across callback |
| Structural product | ELF64, little-endian, EM_X86_64; target is baseline x86-64 | Selected target/profile; actual layout/features still owed |
| Cargo closure | Exactly object 0.36.7 and memchr 2.7.4 under Rust 1.80 | Authorized dependency boundary; execution must validate |
| Repeat products | Same frozen inputs/argv produce identical products, or discrepancy stays open | Compare exact bytes; no nondeterminism repair by relabeling |
| Hardening | Applicable instrumentation retained and support imports disclosed | Owner restriction; HARDENING.md |

These are a finite question/expectation inventory, not yet complete executable
fixtures, signatures, offsets, compiler arguments or negative controls.
That incompleteness stops expensive execution. A later revision must commit
concrete probes and exact expectations before collecting corresponding results.
Run neither candidate archive contents nor existing Rust locally.

## Resource-only result interpretation

Read cgroup state and mount policy first. Verify two allowed CPUs, CPU quota at
most 200%, 6 GiB memory, zero swap and the disk/confined-write contract. Snapshot
absolute installed tool paths, hashes, version banners and dpkg package metadata.
Version queries are metadata only; do not compile input or install/update Rust.
The result cannot discharge arithmetic, ABI, hardening, repeatability or native
conformance obligations. Keep missing tools and nonzero version statuses visible.
