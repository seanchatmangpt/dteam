# 17 — crates/ Workspace Physics

## Principle

src/-only makes tier boundaries social; crates/ makes them enforceable by
Cargo, features, CI, and dependency direction.

```
crate boundary = admission boundary
```

## Workspace layout

```
unibit/
├── Cargo.toml
├── README.md
├── crates/
│   ├── unibit/                 # public facade crate
│   ├── unibit-core/            # T0/T1 hot primitives, no_std
│   ├── unibit-geometry/        # U_{1,n}, coords, planes, constants
│   ├── unibit-macros/          # ontology DSL / code generation
│   ├── unibit-derive/          # proc macros, optional
│   ├── unibit-field/           # UBitField, L2 context, alloc allowed
│   ├── unibit-planner/         # UBitScopePlanner, UFrontier
│   ├── unibit-runtime/         # executor, rings, scratch, hot handle dispatch
│   ├── unibit-delta/           # UDelta, DeltaTape, image diffing
│   ├── unibit-receipt/         # receipts, hash roots, manifests
│   ├── unibit-supervisor/      # observation/recovery, off-hot-path
│   ├── unibit-process/         # Petri/POWL/BPMN compilation adapters
│   ├── unibit-wasm/            # wasm-bindgen boundary, batch APIs
│   ├── unibit-bench/           # microbench harnesses
│   └── unibit-cli/             # CLI, normal Rust allowed
├── benches/
│   ├── t0_word.rs
│   ├── t1_active.rs
│   ├── t2_batch.rs
│   └── t3_projection.rs
├── xtask/
│   ├── Cargo.toml
│   └── src/main.rs
└── examples/
    ├── order_lifecycle.rs
    └── petri_frontier.rs
```

## Dependency direction (one-way, no cycles)

```
T0/T1:
  unibit-core, unibit-geometry

T2:
  unibit-delta, unibit-receipt, unibit-runtime

L2 / visible OS:
  unibit-field, unibit-planner, unibit-process, unibit-supervisor

Human / boundary:
  unibit-cli, unibit-wasm, unibit (facade)
```

## Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/*", "xtask"]

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
rust-version = "1.82"

[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
alloc_instead_of_core = "warn"
std_instead_of_core = "warn"
```

For hot-path crates, lints are stricter.

## Per-crate responsibilities

**unibit-core:** irreducible substrate — `no_std`, no alloc, no Vec/Box/String, no dynamic dispatch, no panic in hot functions. Types: `UBitWord`, `UMask64`, `UDenial`, `UStatus`, primitives `cell_admit()`, `fire_word()`, `select()`, `condition_to_mask()`, `UBitBlock<const WORDS>`, `UBitScratch<const WORDS>`.

**unibit-geometry:** formal U_{1,n} structure. Coordinates, planes, domains, cells. Aliases:
```rust
pub const U_WORD_WORDS: usize = 1;
pub const U_ATTENTION_WORDS: usize = 64;
pub const U_ACTIVE_WORDS: usize = 4096;
pub const U_FIELD_WORDS: usize = 262_144;
```

**unibit-macros:** declarative ontology DSL. `ubit_ontology! { ... }` generating constants, masks, compiled transition templates.

**unibit-field:** 2 MiB meaning/context field (U_{1,64⁴}). `UBitField`, `UBitPlaneRole`, `ULawPlane`, `URewardPlane`, `UScenarioPlane`, `UConformancePlane`. `alloc` for boot/L2 init only.

**unibit-planner:** UBitScopePlanner, UFrontier, UFrontierPlan, UCompiledMotion, UMotionHandle. `UBitScopePlanner: (O*, U_t, C_t, H_t) → {UMotionHandle_i}_{i=1}^n`.

**unibit-runtime:** executes hot handles. UBitExecutor, UHotHandle, URing, UCoreLane, UScratchLane. Consumes precompiled handles. No re-planning.

**unibit-delta / unibit-receipt:** `UDelta`, `DeltaTape`, `UDeltaRef`. `UBitReceipt`, `UReceiptFragment`, `UReceiptChain`, `UManifest`. BLAKE3 gated behind feature.

**unibit-supervisor:** off-hot-path observation/recovery.

**unibit-process:** adapters for Petri nets, POWL, BPMN, workflow patterns, token replay. Not hot substrate.

**unibit-wasm:** WASM boundary with batch APIs to avoid per-transition boundary tax.

**unibit-cli:** normal Rust allowed — Vec, String, Result, serde, clap, anyhow, tracing, filesystem, network.

## Tier boundaries

**T0/T1 (unibit-core, unibit-geometry):** no_std, no alloc, no serde, no tracing, no dyn Trait, no Vec/String/Box, no unwrap/expect/panic in hot functions.

**T2 (unibit-runtime, unibit-delta, unibit-receipt):** alloc only behind features, batch-oriented APIs, receipt generation tiered, no per-step heap allocation.

**L2 / visible OS (unibit-field, unibit-planner, unibit-process, unibit-supervisor):** may allocate at boot/planning, richer Rust allowed, must emit compiled hot handles, must not be in T0/T1 loop.

**Human (unibit-cli, unibit-wasm):** developer ergonomics, boundary batching required, no substrate laws.

## Microbench crate layout

```
crates/unibit-bench/
├── Cargo.toml
└── benches/
    ├── t0_word.rs
    ├── t1_attention.rs
    ├── t1_active.rs
    └── t2_batch.rs
```

Map to UBit timing constitution: T0 ≤ 2 ns, T1 ≤ 200 ns, T2 ≤ 5 μs, T3 ≤ 100 μs.

## The principle

src/ = implementation layout.
crates/ = ontology layout.
For unibit, want ontology layout.
