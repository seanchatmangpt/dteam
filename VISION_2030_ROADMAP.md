# dteam Vision 2030: Autonomic Enterprise Roadmap

## 1. Executive Summary
The `dteam` engine, currently optimized for bounded WASM and deterministic K-Tier execution, requires a structural evolution to handle petabyte-scale, messy enterprise event streams. This roadmap builds on the **`bcinr`** crate (dependency), prior research trajectories (POWL, streaming mining, GPU offload), and **this repository’s** implementations under `src/` to move from a fast, auditable kernel toward a hardware-accelerated, autonomic enterprise intelligence platform.

## 2. Phase 1: Hardware-Sympathetic Kernel (Q1 2027)
**Goal:** Saturate modern CPU and GPU architectures by eliminating branching and utilizing SIMD/SWAR instructions for petabyte-scale token replay and marking updates.

*   **SIMD & SWAR Integration**: Replace scalar bitset operations with vectorized equivalents.
    *   *In-repo*: [`src/simd/swar.rs`](src/simd/swar.rs), [`src/simd/mod.rs`](src/simd/mod.rs); conformance hot path [`src/conformance/case_centric/token_based_replay.rs`](src/conformance/case_centric/token_based_replay.rs), [`src/conformance/mod.rs`](src/conformance/mod.rs).
    *   *External crate*: `bcinr` (see `Cargo.toml`) supplies bitset primitives used across the engine.
*   **GPU Offloading**: Utilize WebGPU bindings for massively parallel matrix operations (e.g., incidence matrix calculations, ILP constraints).
    *   *Status*: **Not in this repository yet** (backlog; no `wgpu` module under `src/`).
*   **Constant Latency Loops**: Ensure WCET (Worst-Case Execution Time) guarantees for real-time edge processing.
    *   *In-repo benches*: [`benches/instruction_stability_bench.rs`](benches/instruction_stability_bench.rs), [`benches/zero_allocation_bench.rs`](benches/zero_allocation_bench.rs).

## 3. Phase 2: Infinite Stream Mining (Q3 2027)
**Goal:** Break free from static `.xes` batch processing. Enable real-time process discovery over unbounded event streams using probabilistic data structures.

*   **Streaming DFG & Heuristics**: Incrementally update process models without holding the entire log in memory.
    *   *Status*: **Not in this repository yet** (backlog; XES batch path lives under [`src/io/xes.rs`](src/io/xes.rs)).
*   **Probabilistic Bounding**: Use sketches to estimate footprint matrices and activity frequencies within the strict K-Tier memory limits.
    *   *In-repo*: [`src/probabilistic/count_min.rs`](src/probabilistic/count_min.rs), [`src/probabilistic/mod.rs`](src/probabilistic/mod.rs).
    *   *External crate*: additional sketch logic may live in `bcinr` (dependency); extend here as needed.

## 4. Phase 3: Advanced Formalisms (POWL) (Q1 2028)
**Goal:** Solve the "Spaghetti Process" problem. Move beyond strict Petri nets to Partially Ordered Workflow Models (POWL) to handle concurrency, complex choices, and unstructured enterprise reality without deadlocks.

*   **POWL Core & Discovery**: Implement the POWL data structures and discover them from event logs.
    *   *In-repo*: [`src/powl/core.rs`](src/powl/core.rs), [`src/powl/discovery.rs`](src/powl/discovery.rs), [`src/powl/mod.rs`](src/powl/mod.rs).
*   **POWL to Petri Net Conversion**: Maintain backward compatibility with the high-speed token replayer by compiling POWL back to WF-nets.
    *   *Status*: **Not in this repository yet** (backlog; Petri net model: [`src/models/petri_net.rs`](src/models/petri_net.rs)).
*   **Genetic & ILP Metaheuristics**: For complex structural repairs when greedy RL fails.
    *   *Status*: **Not in this repository yet** (backlog).

## 5. Phase 4: Predictive & Agentic Autonomy (Q3 2028)
**Goal:** Upgrade the RL agent from a reactive model-builder to a proactive, contextual AI that anticipates bottlenecks and simulates interventions.

*   **Contextual Bandits (LinUCB)**: Replace basic Q-Learning with contextual bandits that adapt to streaming drift.
    *   *In-repo*: [`src/ml/linucb.rs`](src/ml/linucb.rs), [`src/ml/mod.rs`](src/ml/mod.rs).
    *   *Dedicated drift module*: **Not in tree yet** — use [`src/config.rs`](src/config.rs) / discovery `drift_window` and [`src/autonomic/vision_2030_kernel.rs`](src/autonomic/vision_2030_kernel.rs) until a standalone predictor lands.
*   **Counterfactual Simulation**: Allow the "Digital Team" to simulate "what-if" scenarios (e.g., "If I reroute this invoice, what happens to throughput?") before executing an action.
    *   *In-repo*: [`src/agentic/counterfactual.rs`](src/agentic/counterfactual.rs), [`src/agentic/mod.rs`](src/agentic/mod.rs).
    *   *Full simulation harness*: **Not in tree yet** (extend agentic module as needed).
*   **Agentic Handoff & Escalation**: Formalize when the engine must defer to a human operator.
    *   *Status*: **Not in this repository yet** (backlog; related: autonomic guards in [`dteam.toml`](dteam.toml), [`src/autonomic/kernel.rs`](src/autonomic/kernel.rs)).

## 6. Phase 5: Object-Centric Process Mining (OCPM) (Q1 2029)
**Goal:** Handle real-world 1:N and N:M object relationships (e.g., Sales Orders to Deliveries) natively, without flattening data.

*   **OCEL Ingestion & Flattening**: Read Object-Centric Event Logs.
    *   *In-repo*: [`src/ocpm/ocel.rs`](src/ocpm/ocel.rs), [`src/ocpm/mod.rs`](src/ocpm/mod.rs).
*   **Object-Centric Petri Nets**: Discover and replay over multi-object models.
    *   *Status*: **Not in this repository yet** (backlog; classic net + replay: [`src/models/petri_net.rs`](src/models/petri_net.rs), [`src/conformance/mod.rs`](src/conformance/mod.rs)).

## 7. Quality Assurance: Autonomic Acceptance Suite (dteam-jtbd-suite)
**Goal:** Verify that all hyper-optimized algorithms (SWAR, CMS, LinUCB, Simulator, OCPM) combine successfully into a coherent "Digital Team" closure. We move beyond isolated micro-benches to combinatorial end-to-end Jobs-To-Be-Done (JTBD) scenarios.

### 7.1. Suite Architecture & Execution Flow
The following sequence demonstrates how the 5 phases interoperate during a live operational event (e.g., Scenario 1: Offshore Maintenance Drift):

```mermaid
sequenceDiagram
    participant Log as Event Stream (Phase 2)
    participant State as CMS + SWAR (Phase 1/2)
    participant POWL as Formalism Engine (Phase 3)
    participant Bandit as LinUCB (Phase 4)
    participant Sim as Counterfactual Sim (Phase 4)
    participant Policy as Acceptance Calculus
    participant Manifest as Manifest Bus

    Log->>State: Ingest Event (e.g., Temp rising)
    State->>POWL: Check Partial Order / State Bounds
    POWL-->>State: Violation Detected (Valve skip)
    State->>Bandit: Request Action Candidate (Zero-Heap)
    Bandit-->>Sim: Propose 'Reroute/Pause'
    Sim-->>Policy: Expected Reward & Projected State
    Policy-->>Manifest: Accept/Handoff based on Risk Guard
    Manifest-->>Log: Emit Reproducible Receipt
```

### 7.2. Feature Interaction Matrix
The suite validates combinatorial interactions across 16 critical scenarios.

### 7.3. Definition of Done (DoD)
I must personally validate these criteria before reporting "done":
1. **Zero-Heap Verification**: No allocations (`vec!`, `Box`, `HashMap::insert`) occur inside the operational `observe`, `propose`, `accept`, or `update` loops during steady-state.
2. **Branchless Purity**: Performance benchmarks must show 1-15ns latency for state updates and `<2µs` for bandit selections.
3. **Scenario Implementation**: All 16 JTBD scenarios exist and explicitly run through the `Vision2030Kernel`.
4. **Test Pass**: `cargo test jtbd_tests` executes and passes cleanly.
5. **System Health**: `make doctor` returns NOMINAL with no structural warnings.
