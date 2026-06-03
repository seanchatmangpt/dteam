# Architecture Requirements Document (ARD): Utilities & Types (dteam)

## 1. Architectural Principles
The utility and type architecture in `dteam` is driven by the **Compiled Cognition (CCOG)** model and the **INSA** (Instruction Set Architecture) constraints. The primary programming paradigm is strict **Data-Oriented Design (DOD)**.

## 2. Directory Structure & Boundaries
- **`crates/insa/insa-types/`**: The absolute bottom layer. Contains vocabulary types. Must have no dependencies on other workspace crates.
- **`src/utils/`**: High-performance algorithmic building blocks (math, subsets, dense structures).
- **`crates/ccog/`**: Cognitive modeling types (breeds, ABIs, masks, multimodal endpoints).

## 3. Implementation Patterns

### 3.1. Branchless & Dense Kernels
Files like `src/utils/dense_kernel.rs` and `static_pkt.rs` must utilize branchless bitwise operations to prevent CPU pipeline stalls.
```rust
// Expected Pattern: Branchless Merge
#[inline(always)]
pub fn merge_masks(a: u64, b: u64) -> u64 {
    a | b // No branching, single instruction
}
```

### 3.2. Lexicon Doctrine Compliance
To maintain substrate uniformity with the wider architecture:
- Use `UCell`, `UMask`, `TruthBlock` instead of "buffer" or "cache".
- Use `octet`, `u8`, or structural units instead of "byte".
- **No mocks, fakes, or stubs.** Real implementations only.

### 3.3. Cognitive Types (CCOG)
Types in `ccog/src/abi/` must serialize and deserialize deterministically. Submodules like `ktier.rs` and `simd.rs` are responsible for parallel processing masks using explicit AVX-512/NEON intrinsics where applicable, falling back to scalar bitwise ops cleanly.

## 4. Constraints
- **Zero Heap Allocation:** Utility functions must not allocate on the heap.
- **Panic Bounds:** Extensive use of `assert!` on initialization boundaries, but unreachable states during execution must be logically constrained by types (Typestate pattern) to avoid panics.
