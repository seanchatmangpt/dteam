# System Architecture Overview

## Why layers matter

dteam is built as five horizontal layers, each narrowing variance so the layer above it is fast, deterministic, and auditable. The fundamental insight is that process intelligence requires sequential trust: you cannot produce a meaningful fitness score from malformed event data, you cannot reward an RL agent from a meaningless fitness score, and you cannot make a trustworthy selection decision from an unreliable reward signal. Each layer makes a specific narrowing promise to the layer above it:

- I/O disorder → clean, canonical event data
- Clean data → a deterministic fitness score
- Fitness score → an RL reward signal
- RL reward → signal predictions
- Predictions → auditable selection with provenance

Breaking any link invalidates everything downstream. The architecture enforces these dependencies structurally — the ML layer does not import conformance internals; the conformance layer does not import RL state. This is not a style choice. It is what makes the system testable, replaceable, and honest.

## The five layers

### Layer 1: Data model (`src/models/`, `src/io/`)

The data model layer translates external, disordered XES event logs into canonical Rust types: `Event`, `Trace`, `EventLog`, and `PetriNet`. This is where ordering is imposed. `XESReader` uses `quick-xml` for streaming parse; `canonical_hash` (FNV-1a over `concept:name` activity strings) assigns a deterministic fingerprint to every event log before it enters the pipeline. Petri nets use `PackedKeyTable` markings to avoid per-marking heap allocation.

The layer's promise upward: by the time data leaves this layer, it has a canonical identity, a well-formed structure, and no parsing surprises.

### Layer 2: Conformance (`src/conformance/`)

Conformance takes a `(Trace, PetriNet)` pair and produces a fitness score in [0, 1]. There are two execution paths:

- **K64 bitmask path**: for nets with ≤ 64 places, token marking is encoded as a `u64`. Transition firing is the branchless expression `(M & !in_mask) | out_mask`. This path is exact — not an approximation — and runs in 2–5 µs per epoch.
- **BCINR fallback** (`replay_trace_standard`): for larger nets or when bitmask construction fails, the crate falls back to `bcinr`-based token replay with `PackedKeyTable` markings.

`ProjectedLog` indexes activities by `DenseIndex` IDs and aggregates traces for batch replay without per-trace allocation.

The layer's promise upward: every fitness score is reproducible — the same `(Trace, PetriNet)` always produces the same number, with no floating-point variance or OS-dependent randomness.

### Layer 3: RL discovery (`src/reinforcement/`, `src/automation.rs`)

The RL layer treats conformance fitness as the reward signal and drives discovery of better Petri nets. `train_with_provenance` runs an RL loop over `ProjectedLog`, calling `token_replay_projected` to score each proposed net, then updating the Q-table. Available agents: `QLearning`, `DoubleQLearning`, `SARSAAgent`, `ExpectedSARSAAgent`, `ReinforceAgent`. All Q-tables use `PackedKeyTable` and `FxHasher` — no `std::HashMap`.

`RlState<const WORDS>` is stack-allocated: an 8-field quantized state vector plus a `KBitSet<WORDS>` marking mask. State transitions happen in 200–500 ns. The layer emits a `PetriNet` artifact plus a byte `trajectory` encoding the action sequence.

The layer's promise upward: the discovery loop produces a net that maximizes replay fitness on the training log, with a tamper-evident trajectory as evidence.

### Layer 4: ML portfolio (`src/ml/`)

The ML layer operates on signal predictions, not on event logs directly. It receives a pool of pre-computed `SignalProfile` predictions — each one a `Vec<bool>` over the test traces — and selects the best orthogonal subset using HDIT AutoML. The 40 available algorithms span conformance signals (F, G, H, E), language-projection signals (TF-IDF, NGram, PageRank), and embedding signals (HDC, synthetic ML classifiers).

HDIT is not a hyperparameter tuner. It does not adjust learning rates or tree depths. It selects among already-evaluated predictions, using a greedy orthogonal selector that rejects signals correlated above 0.95 with already-selected signals and requires a minimum 0.1% marginal gain. The output is an `AutomlPlan` artifact — a JSON document describing the selected signals, their compute tiers, the fusion operator, and Pareto front.

The layer's promise upward: the selected prediction set is non-redundant, auditable, and accompanied by honest accounting of every candidate evaluated.

### Layer 5: Verification (`src/skeptic_harness.rs`, `src/bin/dod.rs`)

The verification layer asserts that the promises made by layers 1–4 are actually kept. `skeptic_harness` enumerates adversarial attacks and a claim registry — tests that tie narrative claims to runnable code. `skeptic_contract` encodes non-negotiable correctness obligations as constants (`CHECK_RESET_AXIOM`, etc.). Property tests in `proptest_kernel_verification` verify µ-kernel determinism, bitset logic, KTier alignment, and MDL minimality.

The anti-lie infrastructure is most visible in the HDIT layer: four `assert!` calls (not `Result` — panics) verify the accounting identity, accuracy consistency, prediction length, and Pareto front integrity. These are not defensive checks. They are structural guarantees that make dishonest results impossible to construct silently.

## Four cross-cutting primitives

These four primitives appear throughout every layer and were chosen for specific reasons — not familiarity or convention.

### `fnv1a_64`

FNV-1a (Fowler–Noll–Vo 1a) hashes 8 bytes per CPU cycle with no initialization seed. The absence of a seed is the point: `std::hash` uses OS-supplied randomness via `SipHash`, meaning the same string maps to different `u64` values across process restarts. For a system that must produce audit-reproducible fingerprints (`canonical_hash`, `activities_hash`, Q-table state keys), OS-seeded hashing is a correctness defect, not just a performance issue. FNV-1a's 30-cycle constant-time hash eliminates the problem.

### `PackedKeyTable`

`PackedKeyTable` stores `(hash, key, value)` triples in a flat array. Lookup is O(1) by linear scan on hash. The design keeps all entries cache-local — a single 64-byte cache line covers several entries. This is the universal pattern for net markings, Q-tables, and hashed IDs throughout the codebase. The alternative — `std::HashMap` — uses a randomized hasher (back to the OS seed problem), performs heap allocation per insertion, and scatters entries across the heap, breaking L1 cache locality on hot paths.

### `KBitSet<WORDS>`

`KBitSet<WORDS>` is a const-generic bitset stored as `[u64; WORDS]`. The const generic gives the compiler the array size at monomorphization time, enabling loop unrolling and SIMD auto-vectorization. For `WORDS=16` (the K1024 tier), union and intersection operations over 1024 bits are compiler-unrolled to 16 u64 operations — no loop overhead, no branch misprediction, no heap. `KBitSet` is `Copy`, meaning state transitions clone without allocation.

### `DenseIndex`

`DenseIndex::compile` takes an iterator of `(symbol, NodeKind)` pairs, sorts them by hash, detects collisions (returning `DenseError::HashCollision` rather than silently corrupting data), and assigns monotonically increasing dense IDs. The alphabetical sort before ID assignment is the crucial property: two `DenseIndex` instances compiled from the same symbol set in different insertion orders produce identical ID assignments. This makes place indices and activity indices insertion-order-independent — a prerequisite for deterministic bitmask encoding.

## The "does not know" rule

Layer independence is enforced by module boundaries. The ML layer (`src/ml/`) does not import from `src/conformance/` — it receives conformance fitness as a numeric value in a `SignalProfile`. The conformance layer does not import RL state — it receives a `(Trace, PetriNet)` pair and returns a `ConformanceResult`. This is not enforced by the type system alone; it is a design invariant documented in `AGENTS.md` and auditable via `cargo check --all-targets`.

The value of this rule is testability. Conformance can be tested without an RL agent. RL can be tested with a mock fitness function. ML selection can be tested with synthetic signals. The system is five independently testable, independently deployable components that happen to compose into a process intelligence engine.
