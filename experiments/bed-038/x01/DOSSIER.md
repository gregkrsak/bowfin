# B38-X01 prepared source/build/probe dossier

PREPARATION ONLY; UNEXECUTED; NOT BUILD-READY. No command below is activated by the experimental workflow. Amendment 1 as conditionally adopted and Amendment 2 authorize source preparation and a later reviewed inert preflight; image acquisition/build/run, experimental Rust installation/compilation and ABI/native/dependency probes remain excluded.
Start: compiler `fb67a7608ab75d9b6721e91b58dbf61e8edaace1` / tree `c9e5047eaec9430698cd7a82c1d05205d729994f`; preserved X01 head `3c6ed30188d732c4a5eb18cc6f6848a73a15e2f8` / tree `974daa488c70f0d2ec1b8bd08483abe20bf8210b`. Existing `input-resolution.json`, `HARDENING.md`, original reports and failed evidence remain immutable inputs. Documentation is a separate design input, never the compiler baseline.
Known source/archive/base descriptors are exactly those in `input-resolution.json`; verification qualifications remain unchanged. Required unresolved acquisition records: OCI config/layer verified bytes; GCC/binutils archive bodies; exact Ubuntu patched glibc 2.39 headers/DSOs/crt/kernel headers; every bootstrap package URL/version/architecture/size/checksum and extracted binary/DSO hash. No mutable apt selection, image pull or unbounded acquisition is supplied here.
Bootstrap closure must bind `/opt/bootstrap/bin/{gcc,g++,as,ld,make,cmake,ninja,python3,sh,install,tar,cmp}` and every configure/build utility and subprocess, libstdc++/libgcc, GMP/MPFR/MPC, libc/loader and all detected optional support libraries to reviewed identities. The old runner snapshot is evidence about that runner, not this image. Source extraction, sysroot assembly and final image/archive construction remain unresolved exact-input gates; do not improvise them from version labels.
Future fixed paths: verified sources `/work/src/{gcc,binutils,llvm}`, build directories `/work/build/{gcc,binutils,llvm}`, copied prepared fixtures `/work/probes`, target sysroot `/opt/b38/sysroot`, isolated Cargo home `/work/cargo`. Populate these only inside a separately authorized, fully audited task envelope. Missing directories/inputs stop; no fallback or unrelated host cleanup.
Future sanitized environment: clear inherited variables; permit only `HOME=/work/home`, `TMPDIR=/work/tmp`, `PATH=/opt/bootstrap/bin`, `LC_ALL=C`, `TZ=UTC`, `SOURCE_DATE_EPOCH=1704067200`, `CC=/opt/bootstrap/bin/gcc`, `CXX=/opt/bootstrap/bin/g++`, `CFLAGS=-O2 -fstack-protector-strong -fcf-protection=full`, identical `CXXFLAGS`, `LDFLAGS=-Wl,-z,relro,-z,now`, `CARGO_HOME=/work/cargo`, `CARGO_NET_OFFLINE=true`, `RUSTC=/opt/rust-1.80.0/bin/rustc`, and the explicitly enumerated per-command variables below. Record full argv/environment and hashes; reject undeclared `CPATH`, library paths, compiler wrappers and Cargo configuration/build hooks.
Cargo host build scripts require `/opt/bootstrap/bin/cc` to resolve to the reviewed bootstrap GCC bytes; record this alias, the configure shebang `/bin/sh` and every host assembler/linker, never substitute a wrapper. Rust's bundled LLVM, standard libraries and hosted dynamic/runtime closure remain selected Rust-distribution dependencies, with exact identities still owed.
The following are exact proposed build-command templates, not an image recipe or executed configuration. Resolve all support inputs and source-review every detected dependency before a later build authorization; capture configure outputs, CMake cache, link maps and tool/subprocess hashes. GCC uses a disclosed single-stage native build, not a validated three-stage bootstrap. No Clang, plugin or LTO route is proposed.

```sh
cd /work/build/binutils
/work/src/binutils/configure --prefix=/opt/b38/gnu --build=x86_64-pc-linux-gnu \
  --host=x86_64-pc-linux-gnu --target=x86_64-pc-linux-gnu --disable-nls --disable-plugins
/opt/bootstrap/bin/make -j2 all-binutils all-gas all-ld
/opt/bootstrap/bin/make install-binutils install-gas install-ld
cd /work/build/gcc
/work/src/gcc/configure --prefix=/opt/b38/gcc --build=x86_64-pc-linux-gnu \
  --host=x86_64-pc-linux-gnu --target=x86_64-pc-linux-gnu --enable-languages=c \
  --disable-bootstrap --disable-multilib --disable-nls --disable-plugin --without-isl \
  --with-gmp=/opt/bootstrap --with-mpfr=/opt/bootstrap --with-mpc=/opt/bootstrap \
  --with-as=/opt/b38/gnu/bin/as --with-ld=/opt/b38/gnu/bin/ld --with-sysroot=/opt/b38/sysroot
/opt/bootstrap/bin/make -j2 all-gcc all-target-libgcc
/opt/bootstrap/bin/make install-gcc install-target-libgcc
/opt/bootstrap/bin/cmake -S /work/src/llvm/llvm -B /work/build/llvm -G Ninja \
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_C_COMPILER=/opt/b38/gcc/bin/gcc -DCMAKE_CXX_COMPILER=/opt/bootstrap/bin/g++ \
  '-DCMAKE_C_FLAGS=-O2 -fstack-protector-strong -fcf-protection=full -B/opt/b38/gnu/bin/' \
  '-DCMAKE_CXX_FLAGS=-O2 -fstack-protector-strong -fcf-protection=full -B/opt/b38/gnu/bin/' \
  '-DCMAKE_EXE_LINKER_FLAGS=-Wl,-z,relro,-z,now -B/opt/b38/gnu/bin/' -DLLVM_TARGETS_TO_BUILD=X86 \
  -DLLVM_ENABLE_PROJECTS= -DLLVM_ENABLE_RUNTIMES= -DLLVM_ENABLE_LTO=OFF -DBUILD_SHARED_LIBS=OFF \
  -DLLVM_INCLUDE_TESTS=OFF -DLLVM_INCLUDE_EXAMPLES=OFF -DLLVM_ENABLE_ZLIB=OFF -DLLVM_ENABLE_ZSTD=OFF \
  -DLLVM_ENABLE_TERMINFO=OFF -DLLVM_ENABLE_LIBXML2=OFF -DLLVM_ENABLE_LIBEDIT=OFF -DLLVM_ENABLE_LIBPFM=OFF
/opt/bootstrap/bin/cmake --build /work/build/llvm --target opt llc --parallel 2
```

Proposed product matrix: `route=c,llvm`, `mode=0,2`, then Rust `profile=dev,release`. This source is not a runnable image recipe: missing acquisition/bootstrap/import/protection identities are hard future launch prerequisites. For each route/mode, run its exact producer argv below twice in the identical fixed directory with identical input/output names. After each producer and all its descendants cease under the future outer resource controller, copy the object into a bounded evidence name `repeat-1.o` or `repeat-2.o`, hash the copy, remove only the fixed output before the second producer, and leave its second output available for linking. Compare the two retained copies byte for byte; keep every discrepancy. Do not vary producer paths/argv by repeat or normalize differences. Whole-image independent rebuild remains unresolved and receives no extra successful image-build attempt.

```sh
# C route, once per repeat, same paths and argv:
/opt/b38/gcc/bin/gcc -std=c17 -pedantic-errors -Wall -Wextra -Werror -m64 -march=x86-64 -mtune=generic \
  -fPIC -fstack-protector-strong -fcf-protection=full -O"$mode" -c /work/probes/probes/probe.c -o /work/out/product.o
# LLVM route, once per repeat, same paths and argv:
/work/build/llvm/bin/opt "-passes=default<O$mode>" -verify-each -S /work/probes/probes/probe.ll -o /work/out/product.ll
/work/build/llvm/bin/llc -O="$mode" -mtriple=x86_64-pc-linux-gnu -mcpu=x86-64 -relocation-model=pic \
  -code-model=small -filetype=obj /work/out/product.ll -o /work/out/product.o
/opt/b38/gnu/bin/readelf --wide --file-header --sections --symbols --relocs --notes /work/out/product.o
/opt/b38/gnu/bin/objdump -drw /work/out/product.o
/opt/b38/gcc/bin/gcc -dumpmachine
/opt/b38/gcc/bin/gcc -dumpspecs
/opt/b38/gcc/bin/gcc -dM -E -std=c17 -pedantic-errors -Wall -Wextra -Werror -m64 -march=x86-64 -mtune=generic -fPIC -fstack-protector-strong -fcf-protection=full -O"$mode" -x c /dev/null
/opt/b38/gcc/bin/gcc -print-search-dirs
/opt/b38/gcc/bin/gcc -Q --help=target -m64 -march=x86-64 -mtune=generic -fPIC -fstack-protector-strong -fcf-protection=full -O"$mode"
# The producer dry-run is exactly its C argv with -v -### prepended:
/opt/b38/gcc/bin/gcc -v -### -std=c17 -pedantic-errors -Wall -Wextra -Werror -m64 -march=x86-64 -mtune=generic \
  -fPIC -fstack-protector-strong -fcf-protection=full -O"$mode" -c /work/probes/probes/probe.c -o /work/out/product.o
/opt/bootstrap/bin/cmp /work/out/repeat-1.o /work/out/repeat-2.o
RUSTFLAGS="-C linker=/opt/b38/gcc/bin/gcc -C link-arg=/work/out/product.o -C lto=off -C target-cpu=x86-64" \
  /opt/rust-1.80.0/bin/cargo build --manifest-path /work/probes/probes/cargo/Cargo.toml --locked --offline \
  --target x86_64-unknown-linux-gnu --profile "$profile" --target-dir /work/out/"$route-$mode-$profile"
# Before execution, inspect the final Rust-linked ELF as well as its native object:
/opt/b38/gnu/bin/readelf --wide --file-header --program-headers --sections --symbols --dynamic --relocs --notes "$binary"
/opt/b38/gnu/bin/objdump -drw "$binary"
# Only after separate authorization and the full outer identity/import/protection gates:
B38_FUTURE_PROBE_AUTH=separate-reviewed-authorization-required /opt/bootstrap/bin/python3 \
  /work/probes/probes/verify.py "$binary" /work/out/product.o
```

All future subprocesses require pre-write output caps, finite deadlines, actual descendant cessation and retention on failure in the separately reviewed outer controller. `probes/verify.py` supplies bounded scenario output/timeout source, not that outer controller or executable authority. The old unbounded `capture_output` example is superseded and retained in the original bundle. Its replacement records bounded nonzero output/status before a mismatch. Negative controls match their named expected diagnostic and an otherwise-identical valid control; a generic nonzero exit is insufficient.

`binary` is exactly `/work/out/$route-$mode-$profile/x86_64-unknown-linux-gnu/{debug,release}/b38-x01-probe` according to profile. Rust distribution bytes, executable/std hashes and offline registry/cache closure are unresolved; the manually prepared lock is not resolver/compiler evidence. Require Cargo metadata/feature and lock readback to contain only the root, object 0.36.7 (`read_core,elf,unaligned`, default off) and memchr 2.7.4 (default off); missing offline content or lock changes stop. No root workspace attachment or build script is added.
Independent fixture contract: `Wide` size/alignment 16/8, offsets 0/8; status 0 means result in 0..1000, 1 means outside, callback rejection 7 takes precedence. Full signed mathematical subtraction is represented modulo 2^128 by low/high words before callback/classification. Callback arguments are context, borrowed Wide and borrowed stack-array base (only initialized guard[0] may be read); a valid callback reads both words, changes context from 0 to `1+((lo XOR hi) AND 3)` and returns a volatile read of initialized guard[0]=0. CSV values derive from integer mathematics and these declared fixture rules. C uses defined uint64 conversion/borrow arithmetic without C extensions; LLVM widens first and uses no `nsw`, `nuw`, `inbounds`, `noalias` or lifetime claims. Private live nonnull aligned objects are disjoint; Rust passes raw addresses, holds no exclusive reference across callback, and retains no borrowed pointer. The complete retained Rust, C and LLVM IR boundary is inventoried in NATIVE-BOUNDARY.md. Retained and superseded counts are informational under6619582603; every applicable operation still requires independent review.
Discriminators, future only: change C's `out->hi` assignment to `out->hi = (out->lo >> 63) ? UINT64_MAX : 0;`; replace LLVM `%difference = sub i128 %wa, %wb` with `%narrow = sub i64 %a, %b` followed by `%difference = sext i64 %narrow to i128`. Ordinary 9−3 must still pass; minimum-minus-one must report `X01_VALUE:minimum_minus_one`. Change expected Rust size 16 to 24: `X01_LAYOUT` must fail before foreign entry. Context 37 must produce the CSV's rejection status/context; extra/omitted callbacks fail ordinary rows. Change object e_machine offsets 18/19 to 183/0: `X01_ELF_TARGET` must reject before invoking the untouched binary; original EM_X86_64=62 passes. Class/data mutations similarly reject; never execute mutated bytes. Undefined-symbol acceptance remains a separate unresolved import-policy gate requiring exact enumeration, not a header-check claim.
Identity policy: future gate must verify regular ELF executable bytes against the source-built output manifest before invocation, fixed absolute subordinate paths/hashes, GCC target/specs/macros and verbose invocation closure, and resolved CPU/features/layout. A wrong path/hash/wrapper or `__clang__` identity is rejection before producer work; never invoke Clang as a control. That checker and exact acquisition/image-assembly recipes remain unimplemented launch gates, not capabilities supplied by this dossier.
Hardening remains open: volatile local arrays escape to the opaque callback, preserving the allocation needed for strong stack-protection checks; preserve GCC strong stack/full CET and LLVM `sspstrong`. Exact LLVM 18 source `X86AsmPrinter.cpp` reads both CF flags and emits IBT/SHSTK notes, `Module.h` defines Min=8, and `X86TargetMachine.cpp` includes `i128:128` in this target layout; these newly inspected sources support the IR proposal, not executed equivalence. Require O0/O2 disassembly/notes, actual defaults/host support, guard/TLS and `__stack_chk_fail` imports, Rust-link property preservation and runtime enablement checks before compatibility claims. A real protection/import conflict returns for review; no replacement failure helper is supplied.
Future primary source context is the exact LLVM/GCC commits already pinned, including LLVM `llvm/{CMakeLists.txt,include/llvm/IR/Module.h,lib/Target/X86/X86AsmPrinter.cpp,lib/Target/X86/X86TargetMachine.cpp}` and GCC `gcc/doc/install.texi`. Proposed flags are review inputs; any unrecognized option, new dependency, missing mechanism or cap pressure stops. Later execution must independently enforce image 2 CPU/6 GiB/no swap/10 GiB and probe 2 GiB/4 GiB, producer 10 seconds/1 GiB, product 2 MiB text/1 MiB object/32 MiB outputs; no such execution is authorized here. Record full binary/image/source/build manifests, all failed attempts, artifact IDs/hashes/30-day expiry/restore checks and every remaining gate; missing bytes never become trusted X02 input by name.

Additional source-review closure: object0.36.7's existing `build.rs` at commit5b66b69298f6198f9d5b5bb2a0b284b4d920a92c is an explicitly declared hook, not a new dependency or an undeclared hook exception. It runs exact `$RUSTC --version`, parses the minor version and on1.80 emits only `cargo:rustc-check-cfg=cfg(core_error)`. Future evidence must inventory hook source/hash/build executable/host compiler-linker/env/output, positively confirm the pinned Rust identity and require the expected output. Its fallback minor0 on version failure is not compatibility evidence. The snapshot and hash accompany the handoff. Standard Cargo/rustc hosted LLVM/runtime components remain unremoved.

Current preparation under6619582603 completes the source package, not missing future build identity. No unsafe/FFI numeric threshold decides readiness. See NATIVE-BOUNDARY.md for every retained operation, necessary invariants, lifetimes and error behavior; superseded source remains in the preserved original bundle. No protection-disabling flag or fallback was introduced.

The inert verify.py implements baseline CSV validation only. It separately retains bounded stdout/stderr/status or partial failure, requires empty stderr on baseline success, and reads at most803 expectation bytes. Matching layout/guard8/lowering/ELF-mutant rejections to their named controls remains an explicit future outer-controller prerequisite; the declared predictions are not implemented-control or passing-test claims. Its subprocess-group cleanup does not establish final cgroup cessation.
