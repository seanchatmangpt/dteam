# U64-Receipt-Mixer

## Objective
Implement rolling FNV-1a state mixing for Delta-Receipt provenance.

## Requirements
- Conform to the 200ns T1 admissibility threshold (where applicable).
- Adhere to the Universe64 Dual-Plane L1 Architecture.
- Zero heap allocations in the hot path.
- Branchless execution logic (CC=1).

## Context
See `src/agentic/ralph/patterns/U64_ARCHITECTURE.md` for substrate laws.
