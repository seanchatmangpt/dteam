# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

Use `cargo make` (never bare `cargo` for orchestrated tasks — see global CLAUDE.md).

| Goal | Command |
|------|---------|
| Fast check | `cargo make check` |
| Library tests | `cargo make test` |
| All tests (integration + doc) | `cargo make test-all` |
| Single test by name | `cargo test --lib <test_name> -- --nocapture` |
| Lint (warnings as errors) | `cargo make lint` |
| Format | `cargo make fmt` |
| Format check | `cargo make fmt-check` |
| CI pipeline | `cargo make ci` |
| Pre-merge (CI + PDF) | `cargo make pre-merge` |
| Diagnostics | `cargo make doctor` |
| Autonomic simulation | `cargo make run` |
| Benchmarks | `cargo make bench <filter>` |

## Architecture

**dteam** is a deterministic process-intelligence engine. The core problem it solves: given XES event logs and Petri net models, perform token-based conformance replay and drive reinforcement-learning discovery loops — all with zero-heap allocation on hot paths.

### Key Modules

- **`src/models/`** — `Event`/`Trace`/`EventLog` (rust4pm lineage), `PetriNet` with `PackedKeyTable` markings
- **`src/conformance/`** — Token replay: u64 bitmask path (≤64 places, fast) or `replay_trace_standard` fallback
- **`src/reinforcement/`** — `QLearning`, `DoubleQLearning`, `SARSAAgent`, `ExpectedSARSAAgent`, `ReinforceAgent`; all Q-tables use `PackedKeyTable` + `FxHasher`
- **`src/automation.rs`** — `train_with_provenance`: RL loop over `ProjectedLog` → `PetriNet` + action `trajectory`
- **`src/autonomic/`** — `AutonomicKernel` trait (observe → infer → propose → accept → execute → adapt), `DefaultKernel`, `Vision2030Kernel`
- **`src/utils/dense_kernel.rs`** — `PackedKeyTable`, `DenseIndex`, `fnv1a_64` — the universal ID/hash spine
- **`src/io/xes.rs`** — `XESReader` via `quick-xml`
- **`src/agentic/ralph/`** — Orchestration engine with pluggable `PhaseRunner`, `WorkspaceManager`, `AgentRouter`, `DoDVerifier`

### Extended Stacks (Vision 2030 roadmap)

- **`src/powl/`** — Partially ordered workflow language (POWL)
- **`src/ocpm/`** — Object-centric event logs (OCEL)
- **`src/ml/linucb.rs`** — Contextual bandits
- **`src/agentic/counterfactual.rs`** — Scenario simulation
- **`src/probabilistic/`** — Count-min sketch

### Binaries

- **`ralph`** — Orchestration engine (`src/bin/ralph.rs`)
- **`ostar_bridge`** — JSON RPC kernel bridge stub

## Critical Performance Constraints

- **Zero-heap hot paths**: Never introduce `Vec`/`HashMap` in conformance replay or RL update loops. Use `PackedKeyTable` and stack-allocated `RlState<const WORDS>`.
- **u64 bitmask conformance**: The fast replay path depends on `places.len() ≤ 64`. Preserve bitmask semantics and `KTier` alignment when extending.
- **`PackedKeyTable` + `fnv1a_64`**: The universal pattern for net markings, Q-tables, and hashed IDs. Do not substitute with `HashMap`.

## Configuration

`dteam.toml` at repo root drives runtime behavior. `AutonomicConfig::load` returns `Default` if missing — no error. Sections: `[meta]`, `[kernel]`, `[autonomic]` (guards + policy), `[rl]`, `[discovery]`, `[paths]`, `[wasm]`. See `src/config.rs` for Rust types.

## Test Organization

- **Library unit tests**: `src/jtbd_tests.rs`, `src/reinforcement_tests.rs`, `src/proptest_kernel_verification.rs`, plus `#[cfg(test)]` modules in most source files
- **Integration tests**: `tests/branchless_kernel_tests.rs`, `tests/ralph_tests.rs`
- **Property tests**: `src/utils/dense_index_proptests.rs`, `src/ontology_proptests.rs`
- **Adversarial conformance**: `src/conformance/case_centric/adversarial_tests.rs`

Run `cargo test --lib <module_path>` to target a specific test module (e.g. `cargo test --lib jtbd_tests`).

## Code Conventions

- **Feature gate**: `token-based-replay` (default) gates `src/conformance/case_centric/token_based_replay.rs`. Use `#[cfg(feature = "token-based-replay")]` for gated additions.
- **Error handling**: `anyhow::Result` in I/O paths; `Result` types throughout (no unwrap/expect on hot paths).
- **Hashing**: Always `fnv1a_64` + `PackedKeyTable` for IDs and state keys. Never introduce `std::collections::HashMap` for process data.
- **Determinism**: `canonical_hash` and token replay are audit-reproducible — preserve ordering and hashing semantics when modifying event or net serialization.
- **No hidden state across traces**: `skeptic_contract` codifies this. RL evaluation loops must not share caches between traces.

## See Also

- **`AGENTS.md`** — Full contributor guide; authoritative on layout, commands, and conventions
- **`dteam.toml`** — Runtime configuration reference
- **`src/skeptic_contract.rs`** — Non-negotiable correctness obligations
