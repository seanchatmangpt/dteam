# Product Requirements Document (PRD): Utilities & Types (dteam)

## 1. Overview
The `dteam` project relies heavily on specialized utility functions and type definitions to achieve extreme performance (nanosecond scale) and strict determinism. This PRD outlines the product requirements for organizing and maintaining these core components.

## 2. Scope
This document covers the foundational abstractions of the repository:
- **Core Utilities:** `src/utils/` (e.g., bitsets, math, SCC, static PKT).
- **Foundational Types:** `crates/insa/insa-types/`.
- **Compiled Cognition Types:** `crates/ccog/src/` (ABIs, multimodal components, metrics).

## 3. Goals
- **Determinism:** Ensure all utility functions produce identical results across runs, enabling bit-perfect reproducibility.
- **Performance:** Maintain zero-allocation and branchless execution paths to preserve nanosecond latency characteristics.
- **Modularity:** Keep types and utilities decoupled from heavy business logic for maximum reusability across all workspace crates.

## 4. Core Requirements

### 4.1. Utilities (`src/utils/`)
- **Bitwise Supremacy:** Operations must rely on `u64`/`u128` bitmasks (`bitset.rs`) instead of dynamic collections.
- **Deterministic Math:** Mathematical operations (`math.rs`, `perturbation.rs`) must use stable, non-floating-point approximations where exact reproducibility is needed.
- **Dense Memory:** Graph algorithms (`scc.rs`) must operate on dense, contiguous memory structures (`dense_kernel.rs`).

### 4.2. Types (`insa-types` & `ccog`)
- **Stack-allocated First:** Types must implement `Copy` and `Clone` natively wherever possible.
- **Cache-line Alignment:** Memory layouts must be explicitly sized and aligned (`repr(C)`, `align(64)` where applicable) to prevent cache thrashing.
- **Fixed Topology:** Avoid `String` or `Vec` in hot-path types; prefer fixed-size arrays (`[u8; N]`) or region-native types.

## 5. Success Metrics
- 100% test coverage on `src/utils/` utilities.
- Zero heap allocations (`malloc`/`free`) on the main execution hot paths.
- Successful execution of branchless benchmarks (`kernel_bench`, `zero_allocation_bench`) without performance regressions.
