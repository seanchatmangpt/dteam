# MDL Refinement

## Objective
Upgrade structural scoring in src/models/petri_net.rs to follow Φ(N) exactly.

## Requirements
- Conform to the 200ns T1 admissibility threshold (where applicable).
- Adhere to the Universe64 Dual-Plane L1 Architecture.
- Zero heap allocations in the hot path.
- Branchless execution logic (CC=1).

## Context
See `src/agentic/ralph/patterns/U64_ARCHITECTURE.md` for substrate laws.
