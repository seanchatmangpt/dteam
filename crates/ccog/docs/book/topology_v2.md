# Topology v2

Topology v2 defines the nonlinear structure of cognitive flows. Instead of a simple sequence, `ccog` uses POWL (Partial Order, Weighted Loops) topology.

Key topological elements:
- **Choice Graphs**: Admissible branches where the runtime selects the best fit based on priority and evidence.
- **Partial Orders**: Tasks that can be executed independently until a synchronization point (`Join`).
- **Bounded Loops**: Iterative refinement of field state until a stable closure is reached.
- **Silent Transitions**: Internal state changes that do not emit external responses but advance the completion mask.

The `cog8_graph!` macro provides a declarative DSL for constructing these topologies at compile-time or load-time.
