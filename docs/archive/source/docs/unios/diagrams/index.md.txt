# unios / unibit / nightly Rust Diagrams

PlantUML diagrams for the layered architecture:
**dteam → unios → unibit → Nightly Rust → pinned resident region**.

Rendering: plain PlantUML, no `!include` of C4-PlantUML. Render with
`plantuml *.puml` or any PlantUML-aware viewer.

Terminology alignment: [`src/agentic/ralph/patterns/U64_ARCHITECTURE.md`](../../../src/agentic/ralph/patterns/U64_ARCHITECTURE.md)
is canonical for `UniverseBlock`, `UniverseScratch`, `UInstruction`, `UTransition`,
`UDelta`, `UReceipt`, `UProjection`, and the timing constitution
(T0 ≤ 2ns, T1 ≤ 200ns, T2 ≤ 5µs, T3 ≤ 100µs).

## Diagrams

### Architecture & boundaries

| # | File | Purpose |
|---|------|---------|
| 01 | [`01-context-stack.puml`](01-context-stack.puml) | C4 context: human / auditor / builder actors vs the dteam / unios / unibit / nightly stack. |
| 02 | [`02-container-layers.puml`](02-container-layers.puml) | Container view: dteam public API → adapter → unios → unibit → kernels → pinned region. |
| 03 | [`03-component-unios.puml`](03-component-unios.puml) | unios components: admission, planner, frontier, binders, handle, receipt policy, supervisor, projection. |
| 04 | [`04-component-unibit-nightly.puml`](04-component-unibit-nightly.puml) | unibit components: toolchain, const geometry, pinned region, branchless kernel, SIMD, asm check, sparse ΔU, receipt fragment, sealed artifact. |
| 05 | [`05-dependency-law.puml`](05-dependency-law.puml) | Dependency law — each layer's allowed ignorance (dteam does not import unibit, etc.). |

### Hot-path mechanics

| # | File | Purpose |
|---|------|---------|
| 06 | [`06-branchless-kernel.puml`](06-branchless-kernel.puml) | Core mask algebra: deny_prereq ∪ deny_law ∪ deny_cap ∪ deny_scenario → next → ΔU. |
| 07 | [`07-motion-lowering.puml`](07-motion-lowering.puml) | Semantic predicates → compiled mask packet → admit → execute. |
| 08 | [`08-no-hidden-interpretation.puml`](08-no-hidden-interpretation.puml) | Allowed vs forbidden constructs on the hot path. |
| 09 | [`09-branch-policy.puml`](09-branch-policy.puml) | Branches allowed only before the hot handle; runtime branches rejected unless proven faster. |
| 10 | [`10-hot-execution-dynamic.puml`](10-hot-execution-dynamic.puml) | Sequence: unios → unibit API → kernel → pinned region → ΔU + receipt fragment. |

### Geometry

| # | File | Purpose |
|---|------|---------|
| 11 | [`11-work-tier-selection.puml`](11-work-tier-selection.puml) | 8^n work tier selection (atom / word / line / block / tile) → monomorphized kernel. |
| 12 | [`12-memory-projection.puml`](12-memory-projection.puml) | 64^n memory projection: attention / active universe / meaning field → UProjection. |
| 13 | [`13-pinned-position.puml`](13-pinned-position.puml) | Pinning as semantic contract: allocate → validate → mlock → emit L1 position receipt. |

### Nightly Rust mapping

| # | File | Purpose |
|---|------|---------|
| 14 | [`14-nightly-capabilities.puml`](14-nightly-capabilities.puml) | Nightly features → substrate guarantees → handle / kernel / region. |

### End-to-end

| # | File | Purpose |
|---|------|---------|
| 15 | [`15-proof-loop.puml`](15-proof-loop.puml) | Full proof loop: human input → semantic motion → admission → kernel → ΔU → receipt → independent verification. |

### Process mining / conformance

| # | File | Purpose |
|---|------|---------|
| 16 | [`16-token-replay-flow.puml`](16-token-replay-flow.puml) | XES → `XESReader` → `EventLog` → trace → token firing → fitness formula. |
| 17 | [`17-conformance-path-selection.puml`](17-conformance-path-selection.puml) | u64 bitmask fast path (≤64 places) vs `replay_trace_standard` fallback — fork in `src/conformance/`. |
| 18 | [`18-petri-net-marking.puml`](18-petri-net-marking.puml) | `PetriNet` + `PackedKeyTable` markings — how places/transitions map to u64 bitmasks. |

### Reinforcement learning

| # | File | Purpose |
|---|------|---------|
| 19 | [`19-rl-agent-comparison.puml`](19-rl-agent-comparison.puml) | `QLearning`, `DoubleQLearning`, `SARSAAgent`, `ExpectedSARSAAgent`, `ReinforceAgent` — shared interface vs distinct update rules. |
| 20 | [`20-train-with-provenance.puml`](20-train-with-provenance.puml) | `train_with_provenance`: `ProjectedLog` → `PetriNet` + action `trajectory` with receipt chain. |
| 21 | [`21-udelta-to-reward.puml`](21-udelta-to-reward.puml) | `UDelta` bus → reward kernel (R = F + S) → `PackedKeyTable` Q-update. |

### Autonomic cycle

| # | File | Purpose |
|---|------|---------|
| 22 | [`22-autonomic-cycle.puml`](22-autonomic-cycle.puml) | `AutonomicKernel` trait: observe → infer → propose → accept → execute → adapt; `DefaultKernel` vs `Vision2030Kernel`. |

### Infrastructure spine

| # | File | Purpose |
|---|------|---------|
| 23 | [`23-packed-key-table.puml`](23-packed-key-table.puml) | `PackedKeyTable` + `DenseIndex` + `fnv1a_64` — universal ID/hash spine shared by net markings, Q-tables, and event IDs. |
| 24 | [`24-receipt-mixing-detail.puml`](24-receipt-mixing-detail.puml) | `R_{t+1} = mix(R_t, I, ΔU)` at instruction level — `DeltaTape` and FNV-1a material detail. |
| 25 | [`25-timing-constitution.puml`](25-timing-constitution.puml) | T0 ≤ 2ns / T1 ≤ 200ns / T2 ≤ 5µs / T3 ≤ 100µs mapped to actual modules. |

### Correctness obligations

| # | File | Purpose |
|---|------|---------|
| 26 | [`26-skeptic-contract.puml`](26-skeptic-contract.puml) | Skeptic contract: 9 checks that must all hold for "100% without overfitting" to be defensible. |

### Cell-level mechanics (PlantUML versions of Mermaid seqs 1 & 3)

| # | File | Purpose |
|---|------|---------|
| 27 | [`27-core-law-sequence.puml`](27-core-law-sequence.puml) | UInstruction moves to the universe; universe does not move to instructions; ΔU emits; UReceipt remembers. |
| 28 | [`28-sparse-multiword-transition.puml`](28-sparse-multiword-transition.puml) | `ActiveWordSet` loop, per-word enabled contribution, atomic fired_mask reduction, sparse ΔU commit. |

## Scope & exclusions

Out of scope (covered elsewhere or not yet implemented in this repo):

- Three.js / EHT black hole visualization demo.
- Supply-chain globe UI / storm reroute scenario (not implemented here).
- AtomVM boundary canonicalization (not implemented here).
- Marketing framings (Public Story vs Private Reality, board memo).
- Future `unrdf` / `ostar` ontology-driven code manufacturing.

## Relationship to existing Mermaid diagrams

The Mermaid sequence diagrams in
[`src/agentic/ralph/patterns/U64_SEQUENCES.md`](../../../src/agentic/ralph/patterns/U64_SEQUENCES.md)
cover the same core law at the cell / sparse-transition level. The PlantUML
diagrams here add the **vertical layering** (dteam / unios / unibit / Rust) and
the **work/memory tier split** (8^n / 64^n) which the Mermaid set does not yet
represent.
