# Complete retained X01 native boundary — source review only

Decision6619582603 retires the numerical X01 boundary stop. It does not retire
independent examination of any unsafe operation, foreign declaration, ABI call
or backing implementation. The Rust/C/IR fixture bytes remain unchanged from
the preserved cap-stop bundle; this document supplies their complete contract.
No fixture, native object, toolchain build or ABI execution has run here.

| Retained surface | Necessity and invariant |
|---|---|
| Rust `Wide`, `Callback`, foreign `b38_probe` | Explicit POD/out-parameter experiment. `repr(C)` Wide is16bytes/alignment8, offsets0/8. Signed64 inputs, unsigned32 status/context and raw pointers cross the boundary; no Rust reference, enum or aggregate-by-value layout does. One route's object supplies the symbol per binary. SystemV C calling convention and signatures must match exactly. |
| `addr_of_mut!(out/context)` and unsafe foreign invocation | Separate initialized aligned stack objects remain live through synchronous return. A valid nonnull callback code pointer is process-live. No exclusive Rust reference spans callback entry. The raw addresses do not themselves dereference memory. |
| Unsafe callback `*context` read/write | Live aligned readable/writable u32. Nonzero context returns7 without modifying it or reading Wide/guard. Otherwise the checksum expression yields1..4 without overflow. |
| Callback `(*wide).lo/hi` reads | Both words have committed before entry. Wide is aligned and readable. Callback reads it only, mutates context only, retains no pointer, performs no reentrant native call and starts no thread. C classifies stored output afterward; LLVM classifies its SSA difference, so this immutability is an essential fixture invariant. |
| `read_volatile(guard)` | Only initialized guard[0] of the live16byte native allocation may be read. The other15bytes must not be read as initialized values. This is ordinary memory, not MMIO or synchronization. |
| Callback lifetime/error boundary | Wide, context and guard are disjoint. No pointer escapes; no unwind may cross the ABI. Under the declared valid preconditions the callback has no intended panic path. |
| C `b38_probe` definition and indirect callback | Exact ABI signature; void* carries the same context address. uint64 conversions/subtraction/borrow are defined modulo2^64; no signed overflow, pointer arithmetic outside a live object, extension or hidden allocation. Both output words store before callback. Volatile guard[0]=0 remains live through return. Callback rejection takes precedence; otherwise high==0 and low<=1000 is the nominal test. |
| LLVM `b38_probe` definition and indirect call | Both i64 inputs sign-extend before i128 subtraction; every mathematical difference fits i128. Low/high extraction preserves the modulo2^128 representation. Unsigned i128<=1000 rejects negative differences as well as large positives. Output has16writable bytes/alignment8; the field GEP addresses the second live word. No nsw/nuw/inbounds/noalias/lifetime assertion is added. |
| LLVM guard and stores | alloca16/alignment16; only first byte initialized/read. Wide stores precede callback; guard remains live throughout it. Raw-pointer opaque callback invokes the matching Rust signature synchronously. |
| Hardening and code lifetime | Escaping stack guard supports a meaningful instrumentation check. GCC strong stack/full CF flags and LLVM sspstrong/module flags request mechanisms; they prove no emitted instrumentation, final-link properties or runtime enablement. Fixture code is statically linked and lives for the process lifetime; no loader/unloader is introduced. |

Status0 means nominal0..1000; status1 means outside it. Both preserve the complete
committed two-word result and callback context. Status7 rejects initial nonzero
context, preserving that context and the already committed output. Future guard
variants may propagate status8. Missing/unparseable arguments or X01_LAYOUT panic
before foreign entry; extra arguments are ignored. Printing may fail afterward. Stack-protection failure, CET
fault, signal, OOM or substrate loss is failed/incomplete execution, never an
ordinary source status, successful custody evidence or permission to retry.

Invalid pointers, layouts, signatures, alignment, lifetimes or concurrent access
violate private fixture preconditions. Controls must never execute those undefined
cases. They may mutate inert ELF header copies and reject them before invocation.
The callback is not an extensible or adversarial plugin interface.

Independent controls (all unexecuted): ordinary9-3 yields lo6/hi0/context3/status0;
minimum-minus-one distinguishes widened subtraction from signed64 wrapping.
Nonzero context37 yields unchanged37 and status7. Wrong Rust layout assertion
must report X01_LAYOUT before foreign entry. Wrong ELF class/data/e_machine must
report X01_ELF_TARGET without executing mutated bytes. C sign-from-low and LLVM
narrow-before-extend defects must fail the named minimum_minus_one CSV row.
To distinguish an actual guard read from constant0, change only the C guard[0]
initializer or LLVM volatile first-byte store to8: ordinary9-3 must yield
lo6/hi0/context3/status8; context37 remains37/status7. Original zero controls pass.
These source variants add no ABI feature or authorized execution attempt.

Retained boundary: complete Rust30nonblank lines, C23, LLVM IR28 =81. Historical
superseded work adds6Rust plus2C plus1IR lines, giving cumulative Rust36 and
inclusive90. All original versions/diffs remain in the untouched cap-stop ZIP.
These are conservative whole-fixture accounts, not unsafe-operation counts or
safety ratings. No narrower Rust metric erases C/IR obligations. No added unsafe
or FFI is allowed in X03.

Future gates still owe exact compiler/tool/subprocess/sysroot/import identities,
ABI behavior, O0/O2 and dev/release results, guard/TLS and stack-failure imports,
final ELF/GNU-property preservation, actual CET support/enablement, resource
controls and artifact/cessation evidence. These fixtures neither implement a
backend nor establish Bowfin resource custody or a production runtime ABI.
