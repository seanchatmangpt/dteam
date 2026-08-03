# 80/20 Innovation Audit

## Selection rule

The audit ranks gaps by the amount of trust, iteration speed, and downstream correctness unlocked by a small bounded change. Four gaps dominated the repository's operational risk:

1. fail-open diagnostics;
2. configuration without source provenance;
3. configuration without admission invariants;
4. floating toolchain and non-replayable repository evidence.

## Closed gaps

- `make doctor` now preserves the producing exit code and reports dependency blockage explicitly.
- `AutonomicConfig` now exposes `ConfigSource`, `load_with_source`, `load_required`, and `validate`.
- Validation rejects incoherent reward weights, numeric ranges, alignment, paths, WASM bounds, and AutoML successive-halving parameters.
- The Rust nightly is date-pinned and the WASM target is explicit.
- A dependency-free Python audit emits canonical JSON, a SHA-256 receipt, and an exact two-pass replay result.
- Four targeted source mutants plus the replay crown are locally exercised by six tests.

## Explicit boundary

The local source audit is `ALIVE`. Native Rust execution is not claimed: this container has no Rust toolchain and lacks the sibling `unibit` and `wasm4pm` path dependencies. `make doctor` classifies that state as `BLOCKED_DEPENDENCY` instead of rewriting it as success.
