# 15 — Nightly Rust as Private Compiler Surface

## The posture

Rust is used as a **private compiler and verification surface**. Source code
is not the product. The sealed binary artifact is the product.

Three Rust worlds:

1. **Code-manufacturing Rust** — generates the final fast Rust (macros, builders, const generics, codegen binaries, build.rs, proc-macros).
2. **Admissibility Rust** — unios concerns: bind authority, validate handles, strongly typed, sealed modules, auditable.
3. **Ugly-fast Rust** — unibit hot path: concrete, monomorphized, fixed-layout, branchless, allocation-free, direct, assembly-verified.

## Nightly features (the private lab)

### `generic_const_exprs`
Type-level math: derive word count from bit count.
```rust
struct UWork<const BITS: usize>
where [(); BITS / 64]: {
    words: [u64; BITS / 64],
}
```

### `adt_const_params` / `min_adt_const_params`
Enum const parameters. Encode tier/receipt/memory mode as const enums:
```rust
UCompiledMotion<{ UWorkTier::U512 }, { UReceiptMode::Fragment }>
```

### `generic_const_items`
Generic free constants / associated constants:
```rust
pub const WORK_BITS<const N: usize>: usize = pow8(N);
```

### `const_trait_impl`
`impl const Trait for T` — compile-time mask construction polymorphism without runtime dispatch.

### `strict_provenance_lints`
`#![warn(fuzzy_provenance_casts)]` / `#![warn(lossy_provenance_casts)]`.
Mandatory for raw pointer kernels.

### `portable_simd`
For U512Line, U4096Block, U32768Tile. Scalar kernels first; SIMD as private build layer.

### `#[target_feature]` + ISA-specific private kernels
Detect features at boot, compile kernel choice into table/handle, never feature-detect in hot loop.

### Inline / naked asm
`asm!` for register-level snippets, `global_asm!` for external ABI symbols, `naked_asm!` for exact ABI trampolines only. Don't write ordinary logic as naked assembly.

### `maybe_uninit_array_assume_init`
For generated tables [UKernelFn; 64], [UCompiledMask; N], [UReceiptFragment; N]. Boot-time construction without heap.

### `custom_test_frameworks`
no_std substrate test runner bypassing the normal test harness.

### Sanitizers (nightly)
AddressSanitizer, MemorySanitizer, LeakSanitizer, RealtimeSanitizer (detects non-deterministic execution time in realtime contexts — philosophically aligned). Use in proof builds, not production.

## Cargo / rustc surface

### Custom profiles
```toml
[profile.unibit-hot]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"

[profile.unibit-proof]
inherits = "release"
debug = true
strip = "none"

[profile.unibit-sealed]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
```

### `trim-paths` (nightly)
```toml
[profile.release]
trim-paths = "all"
```
Directly relevant for binary secrecy without runtime overhead.

### `-Z build-std`
For private panic/stdlib builds where needed.

### AArch64 branch-protection
`-Z branch-protection` for sealed high-assurance ARM builds.

## Binary secrecy without runtime overhead

**Good zero-overhead opacity:**
- strip symbols, remove debug info
- trim paths
- panic = abort
- LTO/internalize functions
- avoid `#[no_mangle]` except for ABI exports
- minimize exported symbol table
- static link where appropriate
- private generated names at compile time
- closed ABI surface

**Bad (adds overhead, contaminates hot path):**
- runtime control-flow obfuscation
- opaque predicate branches
- string encryption with runtime decrypt
- anti-debugging checks
- self-modifying code
- VM-based obfuscation

The law:
```
opacity = metadata removal + internalization
       ≠ runtime confusion
```

## What to avoid / quarantine

- `core_intrinsics` — compiler-internal, not for general use
- `specialization` / `min_specialization` — soundness issues pending; quarantine
- `dyn Trait`, `Box`, `Vec`, `String`, `Result` in hot path, `unwrap`, `assert`, `format`, panic paths, short-circuit `&&`/`||`, iterator chains over unknown lengths — all keep to planner/CLI/codegen, not kernels

## Three compiler worlds

**World 1: Proof build**
- Debug info retained, symbols retained, sanitizers, assembly emitted, perf counters, Miri, trybuild, asmcheck

**World 2: Benchmark build**
- target-cpu=native, LTO, codegen-units=1, panic=abort, no debug, symbols optional, perf counters

**World 3: Sealed build**
- Strip symbols, trim paths, panic abort, LTO fat, private generated names, minimal exports, no debug, no source, no public crate

## Philosophy

Use nightly Rust not as unstable developer ergonomics, but as a **private code-manufacturing lab**. Stabilize the artifact, not the source API.

Do not obfuscate the hot path. Erase metadata, internalize symbols,
specialize binaries, and distribute only sealed artifacts.
