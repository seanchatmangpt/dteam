# U64-K-T0-Atoms

## Objective
Implement bit-level truth algebra (set, clear, test, select) as T0 primitives.

## Requirements
- System: UniverseOS (Deterministic Operating Environment).
- Hardware: Single-Core Dual-Plane L1 (Plane_D, Plane_S).
- Constraints: T1 (<200ns) for microkernels, CC=1 (Branchless), Zero-Heap.
- Output: Every state motion must emit a UDelta and update the UReceipt rolling hash.

## Context
See `src/agentic/ralph/patterns/U64_ARCHITECTURE.md and `src/agentic/ralph/patterns/U64_SEQUENCES.md`` for C4 diagrams, sequence diagrams, and substrate law.
This task is part of the **KERNEL** subsystem of UniverseOS.

## Definition of Done
- Implementation adheres to the T1 admissibility budget.
- Proptests cover both successful transitions and admissibility violations.
- Logic is strictly branchless mask algebra.
