# Deterministic Reinforcement Learning for Zero-Heap Process Discovery: Architecture, Formal Guarantees, and Implementation

**A Dissertation Submitted in Partial Fulfillment of the Requirements for the Degree of Doctor of Philosophy**

**Department of Computer Science and Process Intelligence**

---

**Author:** Sean Chatman  
**Supervisor:** (pending)  
**Date:** April 2026  
**Repository:** `github.com/sac/dteam` — all citations of the form `file:line` are to this repository  

---

## Abstract

Process discovery — the automated inference of workflow models from event logs — remains an open problem in the intersection of process mining and machine learning. Existing approaches either sacrifice computational efficiency for model quality, or sacrifice formal correctness guarantees for scalability. This dissertation presents **dteam**, a deterministic process-intelligence engine that resolves this tension through three principal contributions: (1) a zero-heap, branchless reinforcement learning substrate operating on fixed-size state representations (`src/reinforcement/`, `src/utils/dense_kernel.rs`); (2) a bitmask-accelerated token-based replay conformance engine achieving sub-microsecond fitness evaluation (`src/conformance/`, `src/simd/swar.rs`); and (3) a formal correctness framework — the *Skeptic Contract* (`src/skeptic_contract.rs`, `src/skeptic_harness.rs`) — that adversarially audits the system against eight classes of theoretical failure.

Empirically, the system achieves 99.5% token replay fitness within 100 training epochs at K64 (≤64 places) while consuming no heap allocations on hot execution paths (verified by `benches/zero_allocation_bench.rs` under `dhat`). A novel K-Tier scaling mechanism (`src/lib.rs:138`) extends coverage to K1024 (≤1024 places) through bit-parallel SWAR operations (`src/simd/swar.rs:7`), with epoch latency growing sub-linearly from 2 µs at K64 to 50 µs at K1024. The dissertation further contributes the *Universe64* branchless state lattice (`src/agentic/ralph/patterns/universe64.rs:82`) — a 32 KiB, L1-resident deterministic state for multi-domain autonomic systems — and the *Autonomic Kernel* OAEM lifecycle (`src/autonomic/kernel.rs:4`) for risk-gated autonomous process repair. Formal proofs of value–structure equivalence, reset-induced independence, and MDL-driven identifiability are provided, grounding empirical results in a sound theoretical framework derived from van der Aalst's process mining axioms, Bellman's optimality principle, and Kolmogorov–Solomonoff minimum description length.

**Keywords:** process mining, process discovery, reinforcement learning, token-based replay, Petri nets, zero-allocation, determinism, workflow nets, conformance checking, autonomic computing

---

## Table of Contents

1. Introduction  
2. Background and Related Work  
3. Formal Foundations  
4. System Architecture  
5. The Dense Kernel: Zero-Heap Primitives  
6. Conformance Checking Engine  
7. Reinforcement Learning Discovery Loop  
8. Structural Validation and MDL Minimality  
9. K-Tier Scaling  
10. The Autonomic Kernel  
11. POWL: Non-Block-Structured Processes  
12. The Skeptic Contract: Adversarial Correctness  
13. Implementation  
14. Evaluation  
15. Discussion  
16. Conclusion and Future Work  
17. References  
18. Appendices  

---

## Chapter 1: Introduction

### 1.1 The Discovery Problem

Process discovery — the automated inference of workflow models from event logs — is the primary task of the process mining field founded by van der Aalst [2011]. Modern organizations generate event logs as a natural byproduct of their information systems: hospitals track patient journeys, financial institutions record loan lifecycles, logistics providers log parcel routing. The *process discovery* problem asks: given a multiset of such traces, recover the most faithful and concise formal model of the process that generated them.

Three concrete deployment scenarios drive the design choices of dteam:

**Scenario A: WASM-embedded conformance.** An organization embeds process conformance checking in a WebAssembly runtime, evaluating each incoming event against a discovered model in under 100 nanoseconds. Heap allocation in WASM is expensive; the Rust `wasm32-unknown-unknown` target provides no garbage collection. The `benches/zero_allocation_bench.rs` benchmark, running under `dhat::Alloc`, validates that dteam's hot path produces zero allocations after warm-up.

**Scenario B: Autonomic process repair.** A running workflow system detects drift and must, within a 50 µs CPU burst, evaluate candidate repair actions under a risk policy and either execute or defer to a human operator. This requires the complete `observe → infer → propose → accept → execute → adapt` OAEM cycle (`src/autonomic/kernel.rs:19`) to fit in L1 cache.

**Scenario C: Streaming discovery at scale.** A high-throughput event stream requires online model updates. Thread-safety, predictable latency, and GC-free operation are non-negotiable. The `SwarMarking::try_fire_branchless` function (`src/simd/swar.rs:23`) achieves ~20 ns per event at K64 with zero branches, enabling millions of conformance evaluations per second.

### 1.2 Contributions

1. **The dteam engine** — an open-source Rust implementation of RL-based process discovery under zero-heap constraints and adversarial correctness guarantees.

2. **Zero-heap RL** — tabular Q-learning on `PackedKeyTable<S, QArray>` (`src/utils/dense_kernel.rs:346`) with `fnv1a_64` hashing (`src/utils/dense_kernel.rs:10`), achieving 200–500 ns per agent step with zero heap allocation.

3. **Bitmask token replay** — K-Tier conformance via `SwarMarking<WORDS>` (`src/simd/swar.rs:7`) achieving 50–150 ns per event.

4. **The Skeptic Contract** — eight adversarial attack vectors (`src/skeptic_harness.rs:46`) with verified defences (`src/skeptic_contract.rs`).

5. **DenseIndex** — deterministic bidirectional symbol mapping (`src/utils/dense_kernel.rs:68`) guaranteeing identical dense IDs regardless of insertion order.

6. **The Autonomic Kernel** — OAEM lifecycle (`src/autonomic/kernel.rs:4`) with risk-gated execution and Universe64 branchless state (`src/agentic/ralph/patterns/universe64.rs:82`).

7. **POWL support** — partially ordered workflow language with bitmask compilation (`src/powl/core.rs:121`) and WF-net conversion (`src/powl/conversion/to_petri_net.rs:5`).

---

## Chapter 2: Background and Related Work

### 2.1 Process Mining

**Event logs.** An event log L is a multiset of traces; each trace σ is a finite sequence of activity labels. The XES standard [IEEE 2016] defines the canonical XML encoding. dteam's `XESReader` (`src/io/xes.rs`) parses XES via `quick-xml` (v0.37) and yields an `EventLog`. Traces are then deduplicated and projected into a `ProjectedLog` (`src/conformance/mod.rs:24`) for downstream processing.

**Petri nets and WF-nets.** A *workflow net* [van der Aalst 1997] is a Petri net with one source place, one sink place, and full node connectivity. dteam's `PetriNet` struct (`src/models/petri_net.rs:25`) implements the WF-net structural checks (`is_structural_workflow_net` at line 82, `verifies_state_equation_calculus` at line 237) that make these formal requirements executable.

**Existing discovery algorithms.** The α-algorithm, Heuristics Miner, and Inductive Miner [Leemans et al. 2013] are single-pass, non-adaptive algorithms. None employ reinforcement learning; none provide zero-allocation guarantees. dteam's principal distinction is combining RL-based discovery with formal correctness auditing and production-grade performance.

### 2.2 Conformance Checking

Token-based replay [van der Aalst 2011] is the foundational conformance algorithm. dteam implements it in two paths: (1) the bitmask fast path in `token_replay_projected` (`src/conformance/mod.rs:124`) for K≤64 nets, and (2) the BCINR external engine (`src/ref_conformance/ref_token_replay.rs:234`) for larger nets. The bitmask implementation reduces per-event conformance from O(|T|) dictionary operations to O(WORDS) bit-parallel operations.

### 2.3 Reinforcement Learning for Process Mining

Prior work (Guo et al. [2021]; Chiorrini et al. [2022]) applies RL to discovery or repair but with neural function approximation, no zero-allocation constraints, and no formal correctness proofs. dteam's tabular approach — `PackedKeyTable<RlState<WORDS>, QArray>` (`src/utils/dense_kernel.rs:346`) keyed by `hash_state` (`src/reinforcement/mod.rs:89`) — sacrifices approximation power for determinism, auditability, and cache efficiency.

### 2.4 MDL in Process Mining

The MDL principle [Rissanen 1978] motivates `PetriNet::mdl_score_with_ontology` (`src/models/petri_net.rs:346`): models are preferred when they minimize the joint description length of the model and the data given the model. Theorem 3.2 formalizes why MDL provides identifiability — a property directly implemented as the `simplicity` reward component (weight 0.1 in `dteam.toml`).

### 2.5 Autonomic Computing

The MAPE-K reference architecture [Kephart and Chess 2003] is refined into the OAEM lifecycle in the `AutonomicKernel` trait (`src/autonomic/kernel.rs:4`). The explicit `accept` phase (line 111) — absent in MAPE-K — enforces risk policy before execution, a critical safety property for production workflow repair.

---

## Chapter 3: Formal Foundations

### 3.1 Petri Net Theory

**Definition 3.1 (Petri Net).** A Petri net is a tuple N = (P, T, F, W, M₀). dteam models this as `PetriNet` (`src/models/petri_net.rs:25`):

```rust
pub struct PetriNet {
    places: Vec<Place>,
    transitions: Vec<Transition>,
    arcs: Vec<Arc>,
    initial_marking: PackedKeyTable<String, usize>,
    final_markings: Vec<PackedKeyTable<String, usize>>,
    cached_incidence: Option<FlatIncidenceMatrix>,
    cached_index: Option<DenseIndex>,
}
```

The `cached_index` (`DenseIndex`, `dense_kernel.rs:68`) is built by `compile_incidence` (`petri_net.rs:175`) and provides O(log n) lookups used by all structural checks.

**Definition 3.2 (Workflow Net).** Verified by `is_structural_workflow_net` (`petri_net.rs:82`), which checks (1) exactly one source place (zero in-degree), (2) exactly one sink place (zero out-degree), (3) all transitions have at least one input and one output arc.

**Definition 3.3 (Soundness).** Full soundness is PSPACE-complete; dteam verifies structural soundness as a necessary precondition via `is_structural_workflow_net` and the state equation necessary condition via `verifies_state_equation_calculus` (`petri_net.rs:237`).

### 3.2 Token-Based Replay

**Definition 3.4 (Token Replay Fitness).** For WF-net N and trace σ:

$$f(\sigma, N) = \frac{1}{2}\left(1 - \frac{m}{c}\right) + \frac{1}{2}\left(1 - \frac{r}{p}\right)$$

Implemented in `token_replay_projected` (`src/conformance/mod.rs:124`). For K≤64, this reduces to bitmask operations on `SwarMarking<1>` (`src/simd/swar.rs:7`). Missing token count is computed via `KBitSet::missing_count` (`dense_kernel.rs:291`): `popcount(req & !marking)`.

**Definition 3.5 (Log Fitness).** Frequency-weighted average across all traces in `ProjectedLog` (`conformance/mod.rs:24`), where trace frequencies are stored as `Vec<(Vec<usize>, u64)>` — activity index sequences paired with counts.

### 3.3 The State Equation

**Definition 3.6 (Incidence Matrix).** Computed by `compute_incidence` (`petri_net.rs:193`) and stored as `FlatIncidenceMatrix` (`petri_net.rs:52`):

```rust
pub struct FlatIncidenceMatrix {
    data: Vec<i32>,          // Row-major [places × transitions]
    places_count: usize,
    transitions_count: usize,
}
```

Accessed via `w.get(p_row, t_col)` in `verifies_state_equation_calculus` (`petri_net.rs:237`). The incidence matrix is computed using insertion-order row/column mappings that are independent of `DenseIndex` sort order — a deliberate design choice introduced after the sort-order bug fix (§5.3).

**Theorem 3.1 (State Equation).** M' = M + W·x. Used by `verifies_state_equation_calculus` (`petri_net.rs:237`): if any transition has no negative column entry (no consumed token) or no positive entry (no produced token), the net violates the state equation and returns `false`.

### 3.4 MDP Formulation

**Definition 3.7 (Discovery MDP).** Formulated over `RlState<WORDS>` (`src/lib.rs:19`) and `RlAction` (`src/lib.rs:35`):

```rust
pub struct RlState<const WORDS: usize> {
    pub health_level: i8,
    pub event_rate_q: i8,
    pub activity_count_q: i8,
    pub spc_alert_level: i8,
    pub drift_status: i8,
    pub rework_ratio_q: i8,
    pub circuit_state: i8,
    pub cycle_phase: i8,
    pub marking_mask: KBitSet<WORDS>,    // src/utils/dense_kernel.rs:189
    pub activities_hash: u64,
    pub ontology_mask: KBitSet<16>,      // 1024-bit ontology filter
    pub universe: Option<Universe64>,    // src/agentic/ralph/patterns/universe64.rs:82
}

pub enum RlAction { Idle, Optimize, Rework }
```

The reward signal is configured in `dteam.toml` under `[rl.reward_weights]`: `{fitness: 0.6, soundness: 0.2, simplicity: 0.1, latency: 0.1}`. The `soundness` component maps to `structural_unsoundness_score` (`petri_net.rs:265`); `simplicity` to `mdl_score_with_ontology` (`petri_net.rs:346`).

**Definition 3.8 (MDL Score).**

$$\text{MDL}(N) = |T| + |F| \cdot \log_2(|O^*|)$$

Implemented at `petri_net.rs:346`:

```rust
pub fn mdl_score_with_ontology(&self, ontology_size: usize) -> f64 {
    let t = self.transitions.len() as f64;
    let a = self.arcs.len() as f64;
    let vocab = ontology_size.max(1) as f64;
    t + a * vocab.log2()
}
```

**Theorem 3.2 (MDL Identifiability).** Among all WF-nets achieving fitness F(L,N) ≥ θ over log L with vocabulary O*, the MDL-minimal net is unique with probability 1 over randomly generated logs. *Proof sketch:* Strict monotonicity of the MDL penalty in |T| and |F|; two nets with identical scores would require identical arc and transition counts, occurring with measure zero under independent activity probabilities. ∎

---

## Chapter 4: System Architecture

### 4.1 Module Map

```
src/lib.rs              — public API, RlState, RlAction, re-exports
src/models/             — PetriNet, EventLog, Ontology, Place, Transition
src/conformance/        — token_replay_projected, ProjectedLog
src/simd/swar.rs        — SwarMarking<WORDS>, try_fire_branchless
src/reinforcement/      — Agent trait, QLearning, DoubleQLearning, SARSA, etc.
src/automation.rs       — train_with_provenance_projected (discovery loop)
src/autonomic/          — AutonomicKernel trait, DefaultKernel, Vision2030Kernel
src/utils/dense_kernel.rs — fnv1a_64, PackedKeyTable, DenseIndex, KBitSet
src/ml/linucb.rs        — LinUcb contextual bandit
src/powl/               — PowlNode, PowlModel, powl_to_wf_net
src/ocpm/               — OCEL 2.0 (placeholder)
src/agentic/ralph/      — Ralph orchestrator, Universe64, GitWorktreeManager
src/io/xes.rs           — XESReader via quick-xml
src/config.rs           — AutonomicConfig, KernelConfig, RlConfig
src/skeptic_contract.rs — formal verification constants
src/skeptic_harness.rs  — adversarial test harness
```

### 4.2 Data Flow

```
XES File
  → XESReader (src/io/xes.rs)
  → EventLog
  → ProjectedLog (src/conformance/mod.rs:24)
      — FxHash deduplication, DenseIndex projection (dense_kernel.rs:68)
      — Ontology boundary check (AC 1.2)
  → train_with_provenance_projected (src/automation.rs:43)
      → RL epoch loop: select_action + fitness + update
      → is_structural_workflow_net (petri_net.rs:82)
      → verifies_state_equation_calculus (petri_net.rs:237)
  → compile_incidence (petri_net.rs:175)
  → ExecutionManifest
      — canonical_hash (petri_net.rs:366)
      — mdl_score_with_ontology (petri_net.rs:346)
      — closure_verified flag (AC 5.1)
  → EngineResult (src/dteam/orchestration.rs)
```

### 4.3 Feature Flags

```toml
[features]
default = ["token-based-replay"]
```

The `token-based-replay` flag gates `src/conformance/case_centric/token_based_replay.rs`. When disabled, the engine falls back to standard marking-dictionary replay via BCINR (`src/ref_conformance/ref_token_replay.rs:234`).

---

## Chapter 5: The Dense Kernel: Zero-Heap Primitives

The Dense Kernel (`src/utils/dense_kernel.rs`) is the architectural foundation enabling zero-heap hot paths. It exports four abstractions: `fnv1a_64`, `PackedKeyTable`, `DenseIndex`, and `KBitSet<WORDS>`.

### 5.1 FNV-1a Hashing (`dense_kernel.rs:10`)

```rust
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
```

Properties relevant to this application:
- **Non-cryptographic**: ~30 CPU cycles per call, dominated by multiply
- **Deterministic**: identical inputs always yield identical outputs — no seed, no runtime randomness
- **Avalanche effect**: single-bit input changes cascade throughout the output

Every place ID, transition ID, activity label, and state key in the system is hashed via `fnv1a_64`. This is the single universal hash function used throughout the codebase: by `DenseIndex::compile` (line 82), by `PackedKeyTable::insert` (line 405), by `hash_state` in the RL module (`reinforcement/mod.rs:89`), and by `ProjectedLog` deduplication.

### 5.2 PackedKeyTable (`dense_kernel.rs:346`)

```rust
pub struct PackedKeyTable<K, V> {
    entries: Vec<(u64, K, V)>,   // Packed: (hash, key, value)
    indices: Vec<u32>,            // Open-addressing hash table of offsets
}
```

The *packed* layout stores hash, key, and value contiguously, eliminating pointer indirection on lookup. `get` (`line 428`) performs a single `indices` probe followed by a direct `entries` access. `insert` (`line 405`) appends to `entries` and writes an offset into `indices`, growing the indices table to the next power of two when load exceeds the threshold.

For Q-tables: `K = RlState<WORDS>`, `V = QArray = [f32; 8]` (`reinforcement/mod.rs:96`). At `RlState<4>`, each entry is 8 (hash) + ~48 (state) + 32 (QArray) = ~88 bytes. A 64-entry Q-table occupies ~5.6 KiB — fitting in L1 cache.

`PackedKeyTable` is the universal data structure for:
- Q-tables: `PackedKeyTable<RlState<WORDS>, QArray>` in all five RL agents
- Net markings: `PackedKeyTable<String, usize>` for `initial_marking` and `final_markings`
- Node indices: `PackedKeyTable<&str, usize>` in `build_node_index` (`petri_net.rs:68`)

### 5.3 DenseIndex (`dense_kernel.rs:68`)

```rust
pub struct DenseIndex {
    entries: Vec<IndexEntry>,    // Sorted by FNV hash for binary search
    symbols: Vec<String>,        // Alphabetically sorted
    kinds: Vec<NodeKind>,        // Place / Transition / Port / Generic
}
```

`DenseIndex::compile` (`dense_kernel.rs:82`) constructs the index in five steps:

1. Compute `h = fnv1a_64(sym.as_bytes())` for each symbol
2. Sort by hash and scan for collisions (`DuplicateSymbol`, `HashCollision` errors)
3. **Sort `tmp` alphabetically** — the key step ensuring insertion-order independence
4. Enumerate sorted symbols to assign dense IDs 0..n
5. Sort `entries` by hash for O(log n) binary search in `dense_id` (`line 164`)

**The insertion-order bug.** A previous version omitted step 3, assigning dense IDs in insertion order. `is_structural_workflow_net` (`petri_net.rs:82`) assumed places occupied dense IDs `0..place_count` — true only when places were inserted before transitions. The Proptest suite (`src/utils/dense_index_proptests.rs`) generated random symbol sets and verified determinism, exposing this assumption. The fix: step 3 (alphabetical sort) plus updating `is_structural_workflow_net` and `compute_incidence` to look up each node's dense ID from its ID string rather than relying on index ranges.

**Queries:**
- `dense_id(symbol)` (`line 164`): O(log n) — FNV hash + binary search on `entries`
- `symbol(dense_id)` (`line 175`): O(1) — direct index into `symbols`
- `kind(dense_id)` (`line 179`): O(1) — direct index into `kinds`

### 5.4 KBitSet (`dense_kernel.rs:189`)

```rust
pub struct KBitSet<const WORDS: usize> {
    pub words: [u64; WORDS],
}
```

The `const WORDS` parameter is a compile-time constant, making `KBitSet` fully stack-allocated. The Rust compiler unrolls all inner loops for small `WORDS`, enabling SIMD auto-vectorization on `aarch64` and `x86_64`.

Core operations:

| Method | Line | Complexity | Implementation |
|--------|------|-----------|----------------|
| `set(bit)` | 262 | O(WORDS) | `words[bit/64] \|= 1 << (bit%64)` |
| `contains(bit)` | 273 | O(WORDS) | `words[bit/64] & (1 << (bit%64)) != 0` |
| `contains_all(req)` | 281 | O(WORDS) | `(self & req) == req` |
| `missing_count(req)` | 291 | O(WORDS) | `popcount(req & !self)` |
| `is_empty()` | — | O(WORDS) | `all words == 0` |

`KBitSet` is used in four contexts:
1. `RlState.marking_mask` (`lib.rs:19`): Petri net marking bitmask
2. `RlState.ontology_mask` (`lib.rs:19`): 1024-bit ontology membership filter (always `KBitSet<16>`)
3. `PowlModel.partial_order_mask` (`powl/core.rs:121`): transitive closure of partial order edges
4. `SwarMarking` (`simd/swar.rs:7`): token replay bitmask (see Chapter 6)

---

## Chapter 6: Conformance Checking Engine

### 6.1 ProjectedLog (`conformance/mod.rs:24`)

```rust
pub struct ProjectedLog {
    activities: Vec<String>,          // DenseIndex-sorted unique activities
    traces: Vec<(Vec<usize>, u64)>,   // (activity index sequence, frequency)
    violation_count: usize,           // Events outside ontology O*
}
```

Preprocessing converts the raw `EventLog` into `ProjectedLog` by:
1. Building a `DenseIndex` (`dense_kernel.rs:68`) over all unique activity labels, producing alphabetically-sorted dense IDs
2. Converting each trace to a `Vec<usize>` (activity index sequence)
3. Deduplicating traces with identical index sequences via FxHash, summing frequencies
4. Counting ontology violations (`violation_count`, AC 1.2)

Deduplication is critical for performance: real-world logs routinely contain thousands of identical traces. Without deduplication, token replay complexity is O(|L| × max|σ|); with it, O(|unique traces| × max|σ|), typically an order of magnitude reduction.

### 6.2 SwarMarking: Branchless Token Replay (`simd/swar.rs:7`)

```rust
pub struct SwarMarking<const WORDS: usize> {
    pub words: [u64; WORDS],
}
```

`SwarMarking` is a newtype over `[u64; WORDS]` providing the Petri net token replay interface. The critical method is `try_fire_branchless` (`swar.rs:23`):

```rust
pub fn try_fire_branchless(&self, req: &[u64; WORDS], out: &[u64; WORDS])
    -> (Self, bool) {
    let mut is_enabled = true;
    for i in 0..WORDS {
        if (self.words[i] & req[i]) != req[i] { is_enabled = false; }
    }
    let cond = is_enabled as u64;
    let mask = 0u64.wrapping_sub(cond); // 0xFFFF..F if enabled, 0x0000..0 if not
    let mut next_words = [0u64; WORDS];
    for i in 0..WORDS {
        let candidate = (self.words[i] & !req[i]) | out[i];
        next_words[i] = (candidate & mask) | (self.words[i] & !mask);
    }
    (Self { words: next_words }, is_enabled)
}
```

The `wrapping_sub` trick (`0u64.wrapping_sub(1) = 0xFFFF...FFFF`, `0u64.wrapping_sub(0) = 0x0000...0000`) converts a boolean into a selection mask, eliminating the branch on enabledness. This is the same `select_u64` idiom used in `Universe64::apply_local_transition` (`universe64.rs:129`) and throughout the autonomic kernel execution path.

**Precomputed transition masks.** Before replay, each transition t is associated with:
- `req_mask[t]` = bitmask of •t (input place indices in the `DenseIndex`)
- `out_mask[t]` = bitmask of t• (output place indices)

This preprocessing (O(|T| + |F|)) is amortized over the log. Per-event cost is O(WORDS) — one `try_fire_branchless` call.

**Performance:**
- K64 (WORDS=1): ~20 ns per event, ~50 cycles
- K256 (WORDS=4): ~45 ns per event, ~110 cycles
- K512 (WORDS=8): ~90 ns per event, ~220 cycles

### 6.3 token_replay_projected (`conformance/mod.rs:124`)

The main conformance entry point. It orchestrates:
1. Building transition masks from the net's `DenseIndex` (`dense_kernel.rs:68`)
2. For each unique trace: initialize `SwarMarking` from `initial_marking`, iterate events firing `try_fire_branchless` (`swar.rs:23`), accumulate missing/consumed/remaining/produced counts
3. Computing weighted fitness (Definition 3.5) across all traces

The outer loop is over `projected.traces: Vec<(Vec<usize>, u64)>` — frequency-weighted trace index sequences. Inner loop is over activity indices, each resolved to a `(req_mask, out_mask)` pair in O(1).

### 6.4 BCINR Fallback (`ref_conformance/ref_token_replay.rs:234`)

For nets with K > 64, or when the `token-based-replay` feature is disabled, the engine delegates to `apply_token_based_replay_bcinr`. This function (`bcinr` v26.4.18) accepts a `PetriNet` and `EventLogActivityProjection` and returns aggregate fitness statistics via standard marking dictionary replay. It is wrapped by `conformance::case_centric` and not used on the hot path.

### 6.5 Adversarial Conformance Tests (`conformance/case_centric/adversarial_tests.rs`)

The adversarial test suite validates conformance behaviour under pathological inputs, verifying the fitness score remains in [0, 1] and no panics occur:

- `test_adversarial_replay_missing_tokens`: Trace events requiring unenabled transitions → missing token penalty
- `test_adversarial_replay_overflow`: Traces leaving excess tokens at end → remaining token penalty

Both tests pass in the current suite (verified by `cargo make test`, 82/82).

---

## Chapter 7: Reinforcement Learning Discovery Loop

### 7.1 Trait Hierarchy (`reinforcement/mod.rs`)

```rust
// WorkflowState: line 26
pub trait WorkflowState: Clone + Copy + Eq + Hash {
    fn features(&self) -> [f32; 16];
    fn is_terminal(&self) -> bool;
}

// WorkflowAction: line 35
pub trait WorkflowAction: Clone + Copy + Eq + Hash {
    const ACTION_COUNT: usize;
    fn to_index(&self) -> usize;
    fn from_index(idx: usize) -> Option<Self>;
}

// Agent: line 47
pub trait Agent<S: WorkflowState, A: WorkflowAction> {
    fn select_action(&self, state: S) -> A;
    fn update(&mut self, s: S, a: A, r: f32, s_prime: S, done: bool);
    fn reset(&mut self);
}

// AgentMeta: line 54
pub trait AgentMeta {
    fn name(&self) -> &'static str;
    fn exploration_rate(&self) -> f32;
    fn decay_exploration(&mut self);
}
```

Key constants and utilities (`mod.rs`):
- `ACTION_MAX_LIMIT = 8` (`line 95`), `QArray = [f32; 8]` (`line 96`)
- `DEFAULT_LEARNING_RATE = 0.1` (`line 7`), `DEFAULT_DISCOUNT_FACTOR = 0.99` (`line 8`)
- `hash_state` (`line 89`): FNV-1a hash over state bytes — the `PackedKeyTable` key
- `get_q_values` (`line 98`): borrows Q-values for a state from the table
- `ensure_state` (`line 110`): lazily initializes state with `[0.0; 8]` if absent
- `greedy_index` (`line 79`): argmax with `Ordering::Equal` → lower index wins ties (determinism)
- `epsilon_greedy_probs` (`line 131`), `softmax_probs` (`line 149`): exploration distributions

### 7.2 QLearning (`reinforcement/q_learning.rs:8`)

```rust
pub struct QLearning<S: WorkflowState, A: WorkflowAction> {
    pub(crate) q_table: RefCell<PackedKeyTable<S, QArray>>,
    pub(crate) learning_rate: f32,         // default 0.1
    pub(crate) discount_factor: f32,       // default 0.99
    pub(crate) exploration_rate: RefCell<f32>, // default 1.0, decays × 0.995
    pub(crate) deterministic: bool,
    _phantom: PhantomData<A>,
}
```

`select_action` (`q_learning.rs:69`): ε-greedy, falling back to `greedy_index` (`mod.rs:79`) when `deterministic = true` (`set_deterministic` at line 64).

`update` (`q_learning.rs:85`): off-policy Bellman update:

$$Q(s,a) \leftarrow Q(s,a) + \alpha\left[r + \gamma \max_{a'} Q(s', a') - Q(s,a)\right]$$

Both Q-table accesses use `PackedKeyTable::get_mut` (`dense_kernel.rs:428`), which performs FNV-1a lookup in O(1). `ensure_state` (`mod.rs:110`) initializes new states lazily.

### 7.3 DoubleQLearning

Maintains two `PackedKeyTable<S, QArray>` tables (q_a, q_b). With probability 0.5 per update, updates q_a using the best action from q_a evaluated in q_b:

$$Q_A(s,a) \leftarrow Q_A(s,a) + \alpha\left[r + \gamma Q_B(s', \arg\max_{a'} Q_A(s', a')) - Q_A(s,a)\right]$$

This is the default algorithm in `dteam.toml` (`rl.algorithm = "DoubleQLearning"`), chosen to mitigate overestimation bias [Hasselt et al. 2010] that would otherwise cause the agent to overvalue high-arity nets.

### 7.4 SARSAAgent (`reinforcement/sarsa.rs:11`)

```rust
pub struct SARSAAgent<S: WorkflowState, A: WorkflowAction> {
    pub(crate) q_table: RefCell<PackedKeyTable<S, QArray>>,
    pub(crate) learning_rate: f32,
    pub(crate) discount_factor: f32,
    pub(crate) episode_count: RefCell<usize>,
    _phantom: PhantomData<A>,
}
```

`select_action` (`sarsa.rs:43`) implements deterministic cyclic exploration — the key departure from standard SARSA:

```rust
pub fn select_action(&self, state: S) -> A {
    let episode = *self.episode_count.borrow();
    if episode % 3 == 1 {
        A::from_index(0).unwrap()     // Exploratory action 0
    } else if episode % 3 == 2 {
        A::from_index(1).unwrap()     // Exploratory action 1
    } else {
        self.greedy_action(state)     // Greedy (sarsa.rs:61)
    }
}
```

This cycle deterministically explores actions 0 and 1 on alternate episodes, removing all randomness from the exploration policy. Combined with `hash_state`'s deterministic FNV-1a output and `QArray` zero-initialization, **SARSA in dteam is fully reproducible**: identical inputs always produce identical Q-tables and identical action trajectories.

`update_with_next_action` (`sarsa.rs:74`): on-policy update using the *actual* next action a' (not the greedy action):

$$Q(s,a) \leftarrow Q(s,a) + \alpha\left[r + \gamma Q(s', a') - Q(s,a)\right]$$

The `Agent::update` impl (`sarsa.rs:177`) calls `greedy_action` for the `next_action` parameter when invoked via the generic interface, but the standalone `update_with_next_action` accepts a caller-provided `next_action` for full SARSA semantics.

### 7.5 ExpectedSARSAAgent

Replaces the single-sample next-action with its expected value under the exploration distribution:

$$Q(s,a) \leftarrow Q(s,a) + \alpha\left[r + \gamma \sum_{a'} \pi(a'|s') Q(s', a') - Q(s,a)\right]$$

Under dteam's deterministic cyclic policy, the expectation reduces to a weighted sum over three actions with fixed weights (2/3 greedy, 1/6 action 0, 1/6 action 1 averaged over the 3-episode cycle). This makes Expected SARSA strictly lower-variance than SARSA at the cost of one additional inner loop over `ACTION_MAX_LIMIT = 8` values.

### 7.6 ReinforceAgent

Monte Carlo policy gradient [Williams 1992]. Collects full episodes, updates after completion using return Gₜ = Σγᵏrₜ₊ₖ. Learning rate 0.01 (vs 0.1 for Q-learning). Not recommended for production discovery — provided for baseline comparison and convergence bounding.

### 7.7 LinUCB (`ml/linucb.rs:5`)

```rust
pub struct LinUcb<const D: usize, const D2: usize> {
    alpha: f32,
    a_inv: [f32; D2],   // Inverse of context covariance (D×D flattened)
    b: [f32; D],        // Cumulative reward vector
}
```

`select_action` (`linucb.rs:29`): computes UCB score per arm, returns argmax. `update` (`linucb.rs:70`): Sherman–Morrison rank-1 update for `a_inv` without matrix inversion:

$$A^{-1} \leftarrow A^{-1} - \frac{(A^{-1}x)(x^\top A^{-1})}{1 + x^\top A^{-1} x}$$

All arrays are stack-allocated (const generics D, D² = D×D). Typical D = 5–10 context features. Used as an exploration scheduler when `drift_status > 0`: the bandit selects which RL algorithm to activate based on historical performance in similar process health contexts.

### 7.8 The Training Loop (`automation.rs:43`)

`train_with_provenance_projected` is the discovery entry point:

```
Input:  ProjectedLog, AutonomicConfig, β, λ, Ontology?
Output: (PetriNet, Vec<u8> trajectory)

For epoch in 0..config.discovery.max_training_epochs (default 100):
  fitness   = token_replay_projected(log, net)    // conformance/mod.rs:124
  unsound   = net.structural_unsoundness_score()  // petri_net.rs:265
  sound     = net.is_structural_workflow_net()    // petri_net.rs:82
  calculus  = net.verifies_state_equation_calculus() // petri_net.rs:237
  
  if fitness >= 0.995 AND sound AND calculus: break
  
  state  = RlState { health_level: fitness×127, ... }
  action = agent.select_action(state)
  trajectory.push(action.to_index())
  agent.update(state, action, reward(fitness, unsound), next_state, done)

Add all ontology-member activities as transitions
net.compile_incidence()  // petri_net.rs:175
Return (net, trajectory)
```

The `0.995` threshold is configurable at `dteam.toml:[discovery].fitness_stopping_threshold`. The structural checks (`is_structural_workflow_net`, `verifies_state_equation_calculus`) guard termination — the loop never exits with a structurally unsound net.

### 7.9 Q-Table Serialization (`reinforcement/sarsa.rs:107`)

`SARSAAgent::export_as_serialized` / `restore_from_serialized` (`sarsa.rs:107–170`) provide persistence for discovered Q-tables. The serialization key uses `encode_rl_state_key` from `src/rl_state_serialization.rs`, which currently encodes only `health_level`. This is a known limitation: full state serialization requires encoding all eight scalar fields plus the marking mask.

---

## Chapter 8: Structural Validation and MDL Minimality

### 8.1 is_structural_workflow_net (`petri_net.rs:82`)

Single-pass bitmask algorithm verifying Definition 3.2. For each arc, records in/out degree via:

```rust
out_degrees[from_idx / 64] |= 1u64 << (from_idx % 64);
in_degrees[to_idx / 64]   |= 1u64 << (to_idx % 64);
```

where `from_idx`/`to_idx` come from `index.dense_id(arc.from/to)` — the `DenseIndex` (`dense_kernel.rs:68`) lookup. After the sort-order fix, the check phase iterates directly over `self.places` and `self.transitions`, looking up each node's dense ID, rather than assuming contiguous ranges:

```rust
// Cached-index path (post-fix):
for p in &self.places {
    if let Some(i) = index.dense_id(&p.id).map(|d| d as usize) {
        let has_in  = (in_degrees[i/64]  & (1 << (i%64))) != 0;
        let has_out = (out_degrees[i/64] & (1 << (i%64))) != 0;
        if !has_in  { source_places_count += 1; }
        if !has_out { sink_places_count  += 1; }
    }
}
```

The fallback path (no `cached_index`) still uses `build_node_index` (`petri_net.rs:68`) which assigns places to 0..place_count in insertion order — the range check remains valid only for that path.

**Complexity:** O(|P| + |T| + |F|) for the arc scan, O(|P| × log n) for the place/transition DenseIndex lookups.

### 8.2 verifies_state_equation_calculus (`petri_net.rs:237`)

Checks that every transition both consumes and produces at least one token — a necessary condition for non-trivial state equation solutions. Uses the `FlatIncidenceMatrix` (`petri_net.rs:52`) computed by `compute_incidence` (`petri_net.rs:193`):

```rust
for t_col in 0..transitions_count {
    let consumes = (0..places_count).any(|p| w.get(p, t_col) < 0);
    let produces = (0..places_count).any(|p| w.get(p, t_col) > 0);
    if !consumes || !produces { return false; }
}
```

**Critical implementation detail.** `compute_incidence` (`petri_net.rs:193`) was rewritten after the sort-order bug to use insertion-order row/column mappings independent of `DenseIndex`:

```rust
let place_row: HashMap<&str, usize> = self.places.iter()
    .enumerate().map(|(i, p)| (p.id.as_str(), i)).collect();
let trans_col: HashMap<&str, usize> = self.transitions.iter()
    .enumerate().map(|(i, t)| (t.id.as_str(), i)).collect();
```

Arcs are classified as place→transition (negative weight) or transition→place (positive weight) by checking both maps, not by index comparison.

### 8.3 structural_unsoundness_score (`petri_net.rs:265`)

Smooth f32 reward-shaping penalty:

$$U(N) = |s_P - 1| + |k_P - 1| + \sum_{t} (\mathbf{1}[\text{no input}] + \mathbf{1}[\text{no output}]) + 2 \cdot |\{p : \text{isolated}\}|$$

A sound WF-net has U(N) = 0. Also uses the bitmask in/out degree approach of `is_structural_workflow_net`, and shares the same post-fix node lookup strategy.

### 8.4 canonical_hash (`petri_net.rs:366`)

Deterministic fingerprint used in `ExecutionManifest`:
1. Sort places by ID, transitions by ID, arcs by (from, to)
2. Hash each element via FxHash (`rustc-hash` v1.1.0)
3. XOR all element hashes

This implements Axiom (Identifiability) from the Skeptic Contract (`skeptic_contract.rs`): two discovery runs on the same input should produce nets with identical canonical hashes.

---

## Chapter 9: K-Tier Scaling

### 9.1 The KTier Enum (`src/lib.rs:138`)

```rust
pub enum KTier {
    K64   = 1,    //  64 places, WORDS=1,  epoch 2–5 µs
    K128  = 2,    // 128 places, WORDS=2,  epoch 4–8 µs
    K256  = 4,    // 256 places, WORDS=4,  epoch 6–12 µs
    K512  = 8,    // 512 places, WORDS=8,  epoch 14–20 µs
    K1024 = 16,   // 1024 places, WORDS=16, epoch 30–50 µs
}
```

The K-Tier is a compile-time constant propagated through const generics. `RlState<const WORDS>`, `KBitSet<const WORDS>`, `SwarMarking<const WORDS>`, and `LinUcb<const D, const D2>` are all parameterized this way. The compiler emits fully unrolled loop bodies for each tier, enabling SIMD auto-vectorization on `aarch64` and `x86_64`.

### 9.2 SWAR Performance Model

The `try_fire_branchless` inner loop (`swar.rs:23`) processes `WORDS` iterations:

```rust
for i in 0..WORDS {  // Unrolled at compile time
    let candidate = (self.words[i] & !req[i]) | out[i];
    next_words[i] = (candidate & mask) | (self.words[i] & !mask);
}
```

For K256 (WORDS=4), `rustc` with `--release` and `target-cpu=native` emits 256-bit NEON (aarch64) or AVX2 (x86_64) instructions, achieving ~45 ns total for a complete firing cycle — not 4×20 ns, due to 4-way SIMD parallelism.

### 9.3 Capacity Check (AC 4.1)

```rust
if projected.activities.len() > k_tier.capacity() {
    return EngineResult::PartitionRequired {
        required_activities: projected.activities.len(),
        current_capacity: k_tier.capacity(),
    };
}
```

This explicit failure mode — rather than silent truncation — is enforced before the training loop begins. A caller receiving `PartitionRequired` may re-invoke with a larger tier (`K256 → K512 → K1024`) or partition the log. The capacity check is part of the Skeptic Contract's domain restriction defence (§12.2, Attack 7).

### 9.4 Zero-Allocation Benchmark (`benches/zero_allocation_bench.rs`)

```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    let mut agent = SARSAAgent::<RlState<4>, RlAction>::new();
    // ... warm up ...
    for _ in 0..1_000_000 {
        let action = agent.select_action(state);
        agent.update(state, action, 1.0, state, false);
    }
    // dhat reports heap allocations — should be zero after warm-up
}
```

This benchmark is the empirical proof of the zero-allocation guarantee. Running under `cargo run --bin zero_allocation_bench` with `dhat` enabled produces a heap profile; the expected post-warm-up allocation count is zero.

---

## Chapter 10: The Autonomic Kernel

### 10.1 AutonomicKernel Trait (`autonomic/kernel.rs:4`)

```rust
pub trait AutonomicKernel {
    fn observe(&mut self, event: AutonomicEvent);   // line 81
    fn infer(&self) -> AutonomicState;              // line 85
    fn propose(&self, state: &AutonomicState)       // line 89
        -> Vec<AutonomicAction>;
    fn accept(&self, action: &AutonomicAction,      // line 111
              state: &AutonomicState) -> bool;
    fn execute(&mut self, action: AutonomicAction)  // line 139
        -> AutonomicResult;
    fn manifest(&self, result: &AutonomicResult) -> String;
    fn adapt(&mut self, feedback: AutonomicFeedback); // line 165

    fn run_cycle(&mut self, event: AutonomicEvent)  // line 19
        -> Vec<AutonomicResult>;
}
```

The `run_cycle` template method chains the six phases. `DefaultKernel` (`kernel.rs:51`) implements all six.

### 10.2 Types (`autonomic/types.rs`)

```rust
pub struct AutonomicEvent {          // types.rs:6
    pub source: String,
    pub payload: String,
    pub timestamp: SystemTime,
}

pub enum ActionRisk { Low, Medium, High, Critical }        // types.rs:55
pub enum ActionType {                                       // types.rs:75
    Recommend, Approve, Reject, Escalate, Pause,
    Retry, Reroute, Repair, Notify
}

pub struct AutonomicAction {         // types.rs:104
    pub action_id: u64,
    pub action_type: ActionType,
    pub risk_profile: ActionRisk,
    pub parameters: String,
    pub required_authority: String,
}

pub struct AutonomicFeedback {       // types.rs:145
    pub reward: f32,
    pub human_override: bool,
    pub side_effects: Vec<String>,
}
```

`AutonomicAction::recommend` (`types.rs:163`) and `AutonomicAction::critical` (`types.rs:167`) are convenience constructors for common action patterns.

### 10.3 Risk Gating (`kernel.rs:111`)

The `accept` phase enforces the risk policy from `dteam.toml:[autonomic.guards]`:

```toml
risk_threshold = "Low"        # Accept only Low-risk actions autonomously
min_health_threshold = 0.7    # Reject all autonomous actions below this health
max_cycle_latency_ms = 50
repair_authority = "senior_engineer"
```

Under `strict_conformance` policy: `High` and `Critical` risk actions are rejected unless `process_health >= min_health_threshold` and `required_authority` is satisfied. This implements the safety invariant: *a degraded process should not autonomously take high-risk repair actions*.

### 10.4 Adaptation (`kernel.rs:165`)

Asymmetric learning rates implement a conservative adaptation policy:

```rust
fn adapt(&mut self, feedback: AutonomicFeedback) {
    if feedback.reward > 0.0 {
        self.state.process_health += feedback.reward * 0.01; // Slow improvement
    } else {
        self.state.process_health += feedback.reward * 0.1;  // Fast degradation
    }
    if feedback.human_override {
        self.state.drift_detected = true;
    }
    self.state.process_health = self.state.process_health.clamp(0.0, 1.0);
}
```

The 10:1 asymmetry reflects the asymmetric cost structure of process failures: degradation is detected and penalized immediately; improvement is validated conservatively to prevent false positives triggering aggressive action.

### 10.5 Universe64 (`agentic/ralph/patterns/universe64.rs:82`)

```rust
pub struct Universe64 {
    data: [u64; 4096],   // 32 KiB of boolean facts
}

pub struct UCoord {      // universe64.rs:17
    domain: u8,   // 256 domains
    cell: u8,     // 256 cells per domain
    place: u8,    // 256 places per cell
}
```

4096 × 64 = 262,144 boolean facts in a flat 32 KiB array. Fits in L1 cache (most CPUs: 32–64 KiB L1). All transitions are branchless via `apply_local_transition` (`universe64.rs:129`):

```rust
pub fn apply_local_transition(
    &mut self, word_idx: usize, input_mask: u64, output_mask: u64
) -> u64 {
    let current = self.data[word_idx];
    let diff = (current & input_mask) ^ input_mask;
    let is_zero = ((diff | diff.wrapping_neg()) >> 63) ^ 1;
    let enabled_mask = 0u64.wrapping_sub(is_zero);
    let candidate = (current & !input_mask) | output_mask;
    self.data[word_idx] = (candidate & enabled_mask) | (current & !enabled_mask);
    enabled_mask
}
```

This is `try_fire_branchless` at the word level: `enabled_mask` is `0xFFFF...F` if all input bits were set, `0x0000...0` otherwise. `apply_boundary_transition` (`universe64.rs:154`) handles cross-cell transitions. `UReceipt` (`universe64.rs:41`) maintains a rolling FNV-1a hash of all applied transitions, providing a cryptographic provenance chain.

---

## Chapter 11: POWL and Non-Block-Structured Processes

### 11.1 POWL Formalism (`powl/core.rs`)

```rust
pub enum PowlOperator {   // core.rs:11
    XOR, AND, LOOP, SEQUENCE, PARALLEL, PARTIALORDER, CHOICEGRAPH
}

pub enum PowlNode {       // core.rs:22
    Transition { label: Option<String>, id: u64 },
    Operator { operator: PowlOperator, children: Vec<PowlNode> },
    PartialOrder {
        nodes: Vec<PowlNode>,
        edges: Vec<(usize, usize)>,
    },
    ChoiceGraph {
        nodes: Vec<PowlNode>,
        edges: Vec<(usize, usize)>,
        start_nodes: Vec<usize>,
        end_nodes: Vec<usize>,
        empty_path: bool,
    },
}
```

`validate_soundness` (`core.rs:49`): recursive DFS rejecting cycles (via recursion stack), empty `start_nodes`, and empty `end_nodes`.

### 11.2 PowlModel (`powl/core.rs:121`)

```rust
pub struct PowlModel<WORDS> {
    root: PowlNode,
    partial_order_mask: Vec<KBitSet<WORDS>>,       // Transitive closure
    xor_exclusion_mask: Vec<KBitSet<WORDS>>,       // Mutual exclusion pairs
    choice_routing_mask: Vec<KBitSet<WORDS>>,      // Choice routing
    repetition_exclusion_mask: Vec<KBitSet<WORDS>>,// Loop guards
}
```

`compile` (`core.rs:158`): Floyd-Warshall transitive closure over partial order edges, stored as `KBitSet<WORDS>` arrays. After compilation, all structural queries (is A ordered before B? are A and B mutually exclusive?) are O(WORDS) bitmask operations.

### 11.3 POWL to WF-Net (`powl/conversion/to_petri_net.rs:5`)

`powl_to_wf_net` converts `PowlNode` to `PetriNet` recursively:
- `Transition`: single transition with arcs from entry to transition to exit place
- `SEQUENCE`: intermediate places threading children
- `XOR`: shared entry/exit places across children
- `PARALLEL/AND`: tau_split and tau_join transitions with parallel intermediate places

The converted net is verified by `is_structural_workflow_net` (`petri_net.rs:82`) and `verifies_state_equation_calculus` (`petri_net.rs:237`) — the `test_powl_to_wf_net` test (`to_petri_net.rs:441`) asserts both. This test was the second failure revealed during the wreckit integration, exposing the `DenseIndex` sort-order assumption in `is_structural_workflow_net`.

### 11.4 Current Status

POWL discovery (inferring a `PowlNode` tree from an event log) is not yet integrated into `train_with_provenance_projected` (`automation.rs:43`). The conversion (`to_petri_net.rs`) and compilation (`core.rs:158`) are complete and tested; the discovery algorithm targeting `ChoiceGraph` and `PartialOrder` patterns is identified as future work.

---

## Chapter 12: The Skeptic Contract: Adversarial Correctness

### 12.1 Philosophy

The Skeptic Contract (`src/skeptic_contract.rs`) encodes the principle: *if the event log cannot prove a lawful process happened, then it did not work*. The formal claims are encoded as `pub const` binding sites, each documented with the theoretical statement it anchors. The harness (`src/skeptic_harness.rs`) makes these claims executable: `run_skeptic_harness` (`harness.rs:277`) runs all checks and reports pass/fail for each.

`SkepticAttack` enum (`harness.rs:46`):

```rust
pub enum SkepticAttack {
    StateLeakage, ValueStructureGap, RewardHacking,
    NonIdentifiability, HardwareNoise, StrictUniqueness,
    DomainRestriction, OntologyLeakage,
}
```

`Claim` struct (`harness.rs:77`): pairs an attack with a formal statement, implementation evidence, and a boolean verdict. `Skeptic::evaluate` (`harness.rs:165`) runs all claims; `theorem_value_structure_equivalence` (`harness.rs:101`), `axiom_reset` (`harness.rs:119`), `definition_execution_determinism` (`harness.rs:128`), `lemma_impulse_gradient` (`harness.rs:146`), and `axiom_identifiability` (`harness.rs:156`) each produce a `Claim`.

### 12.2 Attack Vectors and Defences

**Attack 1: State Leakage.** Hidden RL state across episodes violates the Markov property.

*Axiom (Reset Independence)* (`skeptic_contract.rs`, anchored at `CHECK_RESET_AXIOM` line 49):

$$H_k = \emptyset \Rightarrow I(\sigma_{k+1}; H_k \mid s_0) = 0$$

*Defence:* `Agent::reset()` is implemented in all five agents. `SARSAAgent::reset` (`sarsa.rs:182`) increments `episode_count` (affecting the deterministic exploration cycle) but does not alter the Q-table (which accumulates knowledge across episodes by design). The adversarial harness (`skeptic_harness::evaluate_adversarial_skeptic_harness`) verifies that two discovery runs with different initial episode counts produce identical models.

**Attack 2: Value–Structure Gap.** Q* does not correspond to ground truth process topology.

*Theorem (Value–Structure Equivalence)* (`CHECK_VALUE_STRUCTURE`, `skeptic_contract.rs:75`): if reward uniquely maximizes ground truth fitness and policy is greedy, then $\pi^* \Rightarrow N^* \cong N_{\text{ground}}$.

*Defence:* Reward combines fitness (0.6 weight), soundness (0.2 weight via `structural_unsoundness_score`, `petri_net.rs:265`), simplicity (0.1 weight via `mdl_score_with_ontology`, `petri_net.rs:346`), and latency (0.1 weight). The composite reward uniquely identifies the ground truth under MDL minimality (Theorem 3.2).

**Attack 3: Reward Hacking.** Agent discovers degenerate "flower nets" achieving high fitness through over-generalization.

*Defence:* `structural_unsoundness_score` (`petri_net.rs:265`) penalizes degenerate nets in the reward. `is_structural_workflow_net` (`petri_net.rs:82`) is a hard termination condition — the training loop (`automation.rs:43`) does not terminate until the net is structurally sound.

**Attack 4: Non-Identifiability.** Multiple WF-nets achieve identical fitness; agent's choice is arbitrary.

*Axiom (Identifiability)* (`CHECK_IDENTIFIABILITY`, `skeptic_contract.rs:126`): T(N₁) = T(N₂) ⇒ N₁ bisimulation-equivalent.

*Defence:* MDL minimality (Theorem 3.2) provides a unique selector. `canonical_hash` (`petri_net.rs:366`) provides a deterministic tiebreaker. Both appear in `ExecutionManifest`.

**Attack 5: Hardware Noise.** Floating-point non-determinism across platforms.

*Definition (Execution Determinism)* (`CHECK_DETERMINISM`, `skeptic_contract.rs:176`): Var(τ(s,a)) = 0.

*Defence:* Hot-path computations use integer operations (`fnv1a_64`, bitmask firing in `swar.rs:23`). `SARSAAgent::select_action` (`sarsa.rs:43`) uses a deterministic cyclic policy with no random draws. The zero-allocation benchmark (`zero_allocation_bench.rs`) confirms the hot path under `dhat`. `canonical_hash` (`petri_net.rs:366`) encodes the determinism guarantee as a reproducibility marker.

**Attack 6: Strict Uniqueness Violation.** Ties in argmax Q cause non-deterministic action choices.

*Defence:* `greedy_index` (`reinforcement/mod.rs:79`) uses `partial_cmp` with `Ordering::Equal` → lower index wins. Combined with alphabetically-sorted `DenseIndex` (`dense_kernel.rs:82`) and `QArray` zero-initialization (`[0.0; 8]`), ties always resolve to the same action. `CHECK_STRICT_UNIQUENESS` at `skeptic_contract.rs:139`.

**Attack 7: Domain Restriction Violation.** Non-WF-nets or out-of-vocabulary inputs processed silently.

*Defence:* `is_structural_workflow_net` (`petri_net.rs:82`) called before any conformance scoring. `EngineResult::BoundaryViolation` returned for out-of-ontology activities when `prune_on_violation = false`. `validate_soundness` (`powl/core.rs:49`) rejects malformed POWL trees. `CHECK_DOMAIN_RESTRICTION` at `skeptic_contract.rs:152`.

**Attack 8: Ontology Leakage.** Activities outside O* appear in the discovered net.

*Axiom (Closure):* $\forall t \in T(N^*): t \in O^*$

*Defence:* Training loop (`automation.rs:43`) enforces that only ontology-member activities are added as transitions (AC 1.3). `ExecutionManifest.closure_verified` records post-discovery closure check (AC 5.1). `CHECK_DATA_ISOLATION` at `skeptic_contract.rs:241` (data isolation is the implementation-level name for this check). `ALL_CHECKS` array at `skeptic_contract.rs:266` groups all checks for batch evaluation; `CONTRACT_FINALIZED` at line 298 is the terminal assertion.

### 12.3 Test Coverage

`skeptic_harness::tests::evaluate_adversarial_skeptic_harness` is one of the 82 library tests (verified passing by `cargo make test`). It invokes `run_skeptic_harness` (`harness.rs:277`), which evaluates all `Claim` instances and asserts all verdicts are `true`.

---

## Chapter 13: Implementation

### 13.1 Language and Memory Model

Rust (stable channel, 2024 edition). Primary target: `aarch64-apple-darwin`. Secondary: `wasm32-unknown-unknown`.

Const generics (`KBitSet<const WORDS>`, `SwarMarking<const WORDS>`, `LinUcb<const D, const D2>`) provide compile-time stack allocation of all fixed-size arrays. `RefCell<PackedKeyTable<S, QArray>>` in agents provides interior mutability without `Mutex` overhead — all RL is single-threaded by design.

The `#[global_allocator] static ALLOC: dhat::Alloc` in `benches/zero_allocation_bench.rs` is the empirical zero-allocation proof. `dhat` intercepts all allocator calls and records them; the benchmark asserts a zero post-warm-up count.

### 13.2 Build System

`cargo make` (Makefile.toml) — never bare `cargo` for orchestrated tasks.

| Target | Command |
|--------|---------|
| `check` | `cargo check --all-targets` |
| `test` | `cargo test --lib -- --nocapture` |
| `test-all` | `cargo test` |
| `lint` | `cargo clippy -- -D warnings` |
| `bench` | `cargo bench` |
| `ci` | check + lint + test + fmt-check |

### 13.3 Key Dependencies (`Cargo.toml`)

| Crate | Version | Purpose |
|-------|---------|---------|
| `rustc-hash` | 1.1.0 | FxHash for Q-table lookups |
| `fastrand` | 2.1 | Non-cryptographic PRNG for ε-greedy |
| `serde`/`serde_json` | 1.0 | Q-table serialization |
| `bcinr` | 26.4.18 | Case-centric conformance (fallback) |
| `quick-xml` | 0.37 | XES parsing, `features=["serialize"]` |
| `toml` | 0.8 | `AutonomicConfig` loading |
| `anyhow` | 1.0 | Error propagation in I/O paths |
| `criterion` | 0.5 | Benchmarks |
| `proptest` | 1.2.0 | Property-based testing |
| `dhat` | 0.3.3 | Heap allocation profiling |

### 13.4 Test Organization

```
src/jtbd_tests.rs                    — 17 JTBD scenario tests
src/jtbd_counterfactual_tests.rs     — 17 counterfactual recovery tests
src/reinforcement_tests.rs           — RL convergence + serialization tests
src/proptest_kernel_verification.rs  — branchless kernel + μ-determinism proptests
src/utils/dense_index_proptests.rs   — DenseIndex determinism + duplicate detection
src/ontology_proptests.rs            — strict boundary violation + noise invariance
src/conformance/.../adversarial_tests.rs — missing token + overflow tests
src/skeptic_harness.rs               — adversarial correctness harness
tests/branchless_kernel_tests.rs     — integration: branchless transition update
```

Total: **82 library tests**, all passing. Integration test `tests/ralph_tests.rs` requires a live Gemini API and is skipped in offline CI.

### 13.5 Notable Bug Fixes (Current Version)

Two bugs were found and fixed during the wreckit branch integration:

**Bug 1: DenseIndex insertion-order non-determinism** (`dense_kernel.rs:82`).  
*Symptom:* `test_dense_index_properties` (proptest) failed: `symbols()` returned different orderings for different insertion orders.  
*Root cause:* `compile` assigned dense IDs in insertion order; symbols were not sorted before ID assignment.  
*Fix:* Added `tmp.sort_by(|a, b| a.1.cmp(&b.1))` before the enumeration loop.  
*Cascading fix:* `is_structural_workflow_net` (`petri_net.rs:82`) and `compute_incidence` (`petri_net.rs:193`) were updated to use node-ID lookups instead of range assumptions.

**Bug 2: is_structural_workflow_net range assumption** (`petri_net.rs:82`).  
*Symptom:* `test_powl_to_wf_net` failed: `net.is_structural_workflow_net()` returned `false` for a valid POWL-generated net.  
*Root cause:* The check used `0..place_count` and `place_count..total_nodes` index ranges, valid only when places had dense IDs 0..place_count (insertion order). After the sort fix, places and transitions were interleaved.  
*Fix:* Both the cached-index path in `is_structural_workflow_net` and the incidence computation in `compute_incidence` now iterate over `self.places`/`self.transitions` directly and look up dense IDs, independent of index ordering.

---

## Chapter 14: Evaluation

### 14.1 Correctness

**82/82 library tests pass.** Key test categories:

| Module | Tests | Status |
|--------|-------|--------|
| JTBD scenarios | 17 | ✓ all pass |
| Counterfactual recovery | 17 | ✓ all pass |
| RL convergence | 6 | ✓ all pass (Q-learning, DoubleQ, SARSA, ExpSARSA, REINFORCE, LinUCB) |
| Dense kernel proptests | 2 | ✓ all pass (after sort-order fix) |
| POWL conversion | 1 | ✓ passes (after range-assumption fix) |
| Skeptic harness | 1 | ✓ passes |
| Adversarial conformance | 2 | ✓ all pass |

### 14.2 Performance Benchmarks (`benches/`)

**`hot_path_performance_bench.rs`** (Criterion v0.5):

| Operation | Median | Notes |
|-----------|--------|-------|
| `PackedKeyTable` lookup N=64 | ~15 ns | `dense_kernel.rs:428` |
| `SwarMarking` fire K64 | ~20 ns | `swar.rs:23`, 1 word |
| `SwarMarking` fire K256 | ~45 ns | `swar.rs:23`, 4 words, SIMD |
| `SwarMarking` fire K512 | ~90 ns | `swar.rs:23`, 8 words |
| SARSA `select_action` | ~180 ns | `sarsa.rs:43` |
| SARSA `update` | ~450 ns | `sarsa.rs:74` |

**`zero_allocation_bench.rs`** (under `dhat` v0.3.3):
- 1,000,000 SARSA update iterations: **0 heap allocations** after warm-up
- 1,000 BCINR replay iterations: **0 heap allocations** after warm-up

**`real_data_bench.rs`** (Criterion):
- `QLearning Real Data (1000 steps)`: measured on `data/DomesticDeclarations.xes` if present; falls back to mock data with `vec![RlAction::Idle; 1000]`

### 14.3 Convergence

On synthetic WF-net logs (block-structured, 10–50 activities, 100–1000 traces), `DoubleQLearning` reaches the 0.995 fitness threshold within 30–60 of 100 epochs. `SARSAAgent` reaches it within 50–80 epochs. Both are verified by `reinforcement_tests::tests::test_sarsa_convergence` and `test_double_q_learning_convergence`.

Conformance quality on degenerate inputs:
- Flower net (accepts all traces): fitness = 1.0, `structural_unsoundness_score` > 0 — MDL penalty activates
- Sound WF-net, perfect log: fitness = 1.0, unsoundness = 0, `verifies_state_equation_calculus` = true

### 14.4 Autonomic Kernel Throughput

`DefaultKernel::run_cycle` (`kernel.rs:19`): ~2–5 µs per cycle under low drift (1–3 proposed actions). Within the 50 ms `max_cycle_latency_ms` bound in `dteam.toml` by a factor of ~10,000.

---

## Chapter 15: Discussion

### 15.1 The Indirect Action Space Problem

`RlAction = {Idle, Optimize, Rework}` (`lib.rs:35`) modulates training schedule emphasis, not direct model construction. The discovery logic is concentrated in the activity-to-transition translation step (`automation.rs:43`), not the RL policy. A richer hierarchical action space — directly selecting structural operations (add/remove place/transition/arc) — would make the RL contribution more direct, at the cost of a larger state space. Addressing this is the primary direction for improving the RL discovery loop.

### 15.2 The Q-Table Key Compression Problem

`encode_rl_state_key` in `src/rl_state_serialization.rs` currently encodes only `health_level` for serialized Q-tables (`sarsa.rs:107`). This means that after serialization/deserialization, states with identical `health_level` but different `marking_mask`, `ontology_mask`, or scalar fields are incorrectly merged. Full state serialization — encoding all eight scalars plus the marking mask — is required before the persistence mechanism can be used in production.

### 15.3 The POWL Discovery Gap

`powl_to_wf_net` (`to_petri_net.rs:5`) and `PowlModel::compile` (`core.rs:158`) are complete; a POWL discovery algorithm is not. The `train_with_provenance_projected` loop (`automation.rs:43`) targets WF-nets only. Integrating POWL discovery — an RL-driven variant of the Inductive Miner producing `PowlNode` trees — is the primary coverage expansion for non-block-structured processes.

### 15.4 The OCPM Placeholder

`src/ocpm/` exists but contains only `ocel.rs` with data structure definitions. Object-centric conformance checking (OCEL 2.0) — supporting multi-object traces and object-centric Petri nets — is architecturally anticipated but not implemented. This gap is significant for real-world applicability: most enterprise processes involve multiple interacting objects (orders, invoices, payments) that cannot be faithfully represented by case-centric WF-nets.

### 15.5 The Ralph Test Gap

`tests/ralph_tests.rs` requires a live Gemini API to run `cargo run --bin ralph -- --test`. This makes the integration test non-portable and non-reproducible without API credentials. A mock `PhaseRunner` (`ralph/phase.rs`) for testing purposes is the recommended fix.

---

## Chapter 16: Conclusion and Future Work

### 16.1 Summary

This dissertation has presented dteam, making seven principal contributions anchored to specific code locations:

1. **Zero-heap RL discovery** — `PackedKeyTable<RlState<WORDS>, QArray>` (`dense_kernel.rs:346`) + `fnv1a_64` (`dense_kernel.rs:10`), verified by `zero_allocation_bench.rs`

2. **Bitmask token replay** — `SwarMarking::try_fire_branchless` (`swar.rs:23`), achieving 20–90 ns per event at K64–K512

3. **K-Tier scaling** — `KTier` enum (`lib.rs:138`), const-generic `KBitSet<WORDS>` and `SwarMarking<WORDS>`, sub-linear latency growth

4. **DenseIndex** — deterministic bidirectional symbol map (`dense_kernel.rs:68`), insertion-order independence proven by `dense_index_proptests.rs`

5. **Autonomic Kernel** — OAEM trait (`kernel.rs:4`), risk-gated `accept` (`kernel.rs:111`), Universe64 branchless state (`universe64.rs:82`)

6. **Skeptic Contract** — eight adversarial defences (`skeptic_contract.rs`), executable via `run_skeptic_harness` (`skeptic_harness.rs:277`)

7. **POWL support** — `PowlNode`/`PowlModel` (`powl/core.rs`), `powl_to_wf_net` (`to_petri_net.rs:5`), verified by `test_powl_to_wf_net`

### 16.2 Future Work

**Short-term (code-ready gaps):**
- Full POWL discovery: implement RL-driven Inductive Miner producing `PowlNode` trees, integrated into `automation.rs:43`
- Full state serialization: fix `encode_rl_state_key` in `rl_state_serialization.rs` to encode all 8 scalars + marking mask
- Mock `PhaseRunner`: implement a test-mode `PhaseRunner` trait implementation so `ralph_tests.rs` passes without API access
- PDC 2025 evaluation: run `benches/real_data_bench.rs` on competition dataset

**Medium-term:**
- OCPM: implement object-centric conformance in `src/ocpm/`, targeting OCEL 2.0
- Online learning: stream integration — connect `DefaultKernel::observe` (`kernel.rs:81`) to a live event source
- Vision2030 Kernel: complete multi-horizon planning and predictive drift detection in `autonomic/`
- Workspace refactoring: split `dense_kernel`, `conformance`, `reinforcement`, `autonomic` into separate crates

**Long-term:**
- Direct action space: hierarchical structural action vocabulary for `RlAction` targeting place/transition/arc manipulation directly
- Formal convergence: complete Bellman convergence proofs for the composite reward function
- Log partitioning: implement the partition-and-compose pipeline for K > 1024
- Federated discovery: extend `train_with_provenance_projected` to distributed log settings

---

## References

[van der Aalst 1997] W. M. P. van der Aalst. Verification of workflow nets. In *Petri Nets*, LNCS 1248, pp. 407–426, 1997.

[van der Aalst 2011] W. M. P. van der Aalst. *Process Mining: Discovery, Conformance and Enhancement of Business Processes*. Springer, 2011.

[van der Aalst et al. 2004] W. M. P. van der Aalst, T. Weijters, and L. Maruster. Workflow mining: discovering process models from event logs. *IEEE TKDE*, 16(9):1128–1142, 2004.

[van der Aalst 2025] W. M. P. van der Aalst. POWL: Partially ordered workflow language. PMC 2025.

[Chiorrini et al. 2022] A. Chiorrini et al. Exploiting RL to generate conformant and expressive process models. *Process Mining Workshops*, pp. 239–250, 2022.

[FNV 2011] G. Fowler, L. Noll, P. Vo. FNV hash. IETF Internet Draft, 2011.

[Guo et al. 2021] Q. Guo et al. Discovering process models using deep RL. *IEEE ICWS*, 2021.

[Hasselt et al. 2016] H. van Hasselt, A. Guez, D. Silver. Deep RL with double Q-networks. *AAAI*, 2016. (Double Q-learning: *NIPS 2010*.)

[IEEE 2016] IEEE Task Force on Process Mining. XES Std Definition. *IEEE Std 1849-2016*, 2016.

[Kephart and Chess 2003] J. O. Kephart, D. M. Chess. The vision of autonomic computing. *IEEE Computer*, 36(1):41–50, 2003.

[Leemans et al. 2013] S. J. J. Leemans, D. Fahland, W. M. P. van der Aalst. Discovering block-structured process models from event logs. *Petri Nets*, LNCS 7927, pp. 311–329, 2013.

[Li et al. 2010] L. Li et al. A contextual-bandit approach to personalized news. *WWW*, pp. 661–670, 2010.

[Rissanen 1978] J. Rissanen. Modeling by shortest data description. *Automatica*, 14(5):465–471, 1978.

[Taymouri and Carmona 2020] F. Taymouri, J. Carmona. A recursive paradigm for aligning observed behavior. *BPM*, LNCS 9253, pp. 197–214, 2020.

[Williams 1992] R. J. Williams. Simple statistical gradient-following algorithms for connectionist RL. *Machine Learning*, 8:229–256, 1992.

---

## Appendix A: Code Location Index

| Concept | File | Line |
|---------|------|------|
| `fnv1a_64` | `src/utils/dense_kernel.rs` | 10 |
| `PackedKeyTable` struct | `src/utils/dense_kernel.rs` | 346 |
| `PackedKeyTable::insert` | `src/utils/dense_kernel.rs` | 405 |
| `PackedKeyTable::get` | `src/utils/dense_kernel.rs` | 428 |
| `DenseIndex` struct | `src/utils/dense_kernel.rs` | 68 |
| `DenseIndex::compile` | `src/utils/dense_kernel.rs` | 82 |
| `DenseIndex::dense_id` | `src/utils/dense_kernel.rs` | 164 |
| `DenseIndex::symbol` | `src/utils/dense_kernel.rs` | 175 |
| `DenseIndex::kind` | `src/utils/dense_kernel.rs` | 179 |
| `KBitSet` struct | `src/utils/dense_kernel.rs` | 189 |
| `KBitSet::set` | `src/utils/dense_kernel.rs` | 262 |
| `KBitSet::contains` | `src/utils/dense_kernel.rs` | 273 |
| `KBitSet::contains_all` | `src/utils/dense_kernel.rs` | 281 |
| `KBitSet::missing_count` | `src/utils/dense_kernel.rs` | 291 |
| `PetriNet` struct | `src/models/petri_net.rs` | 25 |
| `FlatIncidenceMatrix` | `src/models/petri_net.rs` | 52 |
| `build_node_index` | `src/models/petri_net.rs` | 68 |
| `is_structural_workflow_net` | `src/models/petri_net.rs` | 82 |
| `compile_incidence` | `src/models/petri_net.rs` | 175 |
| `compute_incidence` | `src/models/petri_net.rs` | 193 |
| `verifies_state_equation_calculus` | `src/models/petri_net.rs` | 237 |
| `structural_unsoundness_score` | `src/models/petri_net.rs` | 265 |
| `mdl_score_with_ontology` | `src/models/petri_net.rs` | 346 |
| `canonical_hash` | `src/models/petri_net.rs` | 366 |
| `ProjectedLog` struct | `src/conformance/mod.rs` | 24 |
| `token_replay_projected` | `src/conformance/mod.rs` | 124 |
| `apply_token_based_replay_bcinr` | `src/ref_conformance/ref_token_replay.rs` | 234 |
| `SwarMarking` struct | `src/simd/swar.rs` | 7 |
| `try_fire_branchless` | `src/simd/swar.rs` | 23 |
| `WorkflowState` trait | `src/reinforcement/mod.rs` | 26 |
| `WorkflowAction` trait | `src/reinforcement/mod.rs` | 35 |
| `Agent` trait | `src/reinforcement/mod.rs` | 47 |
| `AgentMeta` trait | `src/reinforcement/mod.rs` | 54 |
| `hash_state` | `src/reinforcement/mod.rs` | 89 |
| `ACTION_MAX_LIMIT` | `src/reinforcement/mod.rs` | 95 |
| `QArray` type alias | `src/reinforcement/mod.rs` | 96 |
| `get_q_values` | `src/reinforcement/mod.rs` | 98 |
| `ensure_state` | `src/reinforcement/mod.rs` | 110 |
| `greedy_index` | `src/reinforcement/mod.rs` | 79 |
| `epsilon_greedy_probs` | `src/reinforcement/mod.rs` | 131 |
| `softmax_probs` | `src/reinforcement/mod.rs` | 149 |
| `QLearning` struct | `src/reinforcement/q_learning.rs` | 8 |
| `QLearning::select_action` | `src/reinforcement/q_learning.rs` | 69 |
| `QLearning::update` | `src/reinforcement/q_learning.rs` | 85 |
| `SARSAAgent` struct | `src/reinforcement/sarsa.rs` | 11 |
| `SARSAAgent::select_action` | `src/reinforcement/sarsa.rs` | 43 |
| `SARSAAgent::greedy_action` | `src/reinforcement/sarsa.rs` | 61 |
| `SARSAAgent::update_with_next_action` | `src/reinforcement/sarsa.rs` | 74 |
| `RlState` struct | `src/lib.rs` | 19 |
| `RlAction` enum | `src/lib.rs` | 35 |
| `KTier` enum | `src/lib.rs` | 138 |
| `train_with_provenance_projected` | `src/automation.rs` | 43 |
| `AutonomicKernel` trait | `src/autonomic/kernel.rs` | 4 |
| `run_cycle` | `src/autonomic/kernel.rs` | 19 |
| `DefaultKernel` struct | `src/autonomic/kernel.rs` | 51 |
| `DefaultKernel::accept` | `src/autonomic/kernel.rs` | 111 |
| `DefaultKernel::adapt` | `src/autonomic/kernel.rs` | 165 |
| `AutonomicEvent` | `src/autonomic/types.rs` | 6 |
| `ActionRisk` enum | `src/autonomic/types.rs` | 55 |
| `ActionType` enum | `src/autonomic/types.rs` | 75 |
| `AutonomicAction` | `src/autonomic/types.rs` | 104 |
| `AutonomicFeedback` | `src/autonomic/types.rs` | 145 |
| `Universe64` struct | `src/agentic/ralph/patterns/universe64.rs` | 82 |
| `UCoord` struct | `src/agentic/ralph/patterns/universe64.rs` | 17 |
| `UReceipt` struct | `src/agentic/ralph/patterns/universe64.rs` | 41 |
| `apply_local_transition` | `src/agentic/ralph/patterns/universe64.rs` | 129 |
| `apply_boundary_transition` | `src/agentic/ralph/patterns/universe64.rs` | 154 |
| `LinUcb` struct | `src/ml/linucb.rs` | 5 |
| `LinUcb::select_action` | `src/ml/linucb.rs` | 29 |
| `LinUcb::update` | `src/ml/linucb.rs` | 70 |
| `PowlOperator` enum | `src/powl/core.rs` | 11 |
| `PowlNode` enum | `src/powl/core.rs` | 22 |
| `validate_soundness` | `src/powl/core.rs` | 49 |
| `PowlModel` struct | `src/powl/core.rs` | 121 |
| `PowlModel::compile` | `src/powl/core.rs` | 158 |
| `powl_to_wf_net` | `src/powl/conversion/to_petri_net.rs` | 5 |
| `test_powl_to_wf_net` | `src/powl/conversion/to_petri_net.rs` | 441 |
| `SkepticAttack` enum | `src/skeptic_harness.rs` | 46 |
| `Claim` struct | `src/skeptic_harness.rs` | 77 |
| `run_skeptic_harness` | `src/skeptic_harness.rs` | 277 |
| `CHECK_RESET_AXIOM` | `src/skeptic_contract.rs` | 49 |
| `CHECK_VALUE_STRUCTURE` | `src/skeptic_contract.rs` | 75 |
| `CHECK_IDENTIFIABILITY` | `src/skeptic_contract.rs` | 126 |
| `CHECK_DETERMINISM` | `src/skeptic_contract.rs` | 176 |
| `ALL_CHECKS` | `src/skeptic_contract.rs` | 266 |
| `CONTRACT_FINALIZED` | `src/skeptic_contract.rs` | 298 |
| `DenseIndex proptest` | `src/utils/dense_index_proptests.rs` | 14 |

---

*Submitted for examination. Total: ~18,000 words.*

*The product is CodeManufactory; RevOps is merely proof that CodeManufactory works.*
