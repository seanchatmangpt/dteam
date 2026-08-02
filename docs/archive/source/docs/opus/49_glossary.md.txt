# 49 — Glossary: Arena to Popcount

Every term used across the archive, in one reference document.
Organized by layer (top-down); alphabetical index at the end.

Legend:
- **bold** — the canonical term
- *italic* — the layer it belongs to
- `code` — types, functions, or literal syntax
- `→ doc N` — first substantial appearance

---

## 1. Doctrine and equation

| Term | Meaning |
|---|---|
| **Chatman equation** | `A = μ(O*)` — action equals morphism applied to closed corpus → doc 07 |
| **O\*** | `Cl(O_public ⊕ ΔO_private)` — closed corpus under semantic operators → doc 07 |
| **Cl()** | closure operator — fixed point of applicable derivations → doc 07 |
| **O_public** | publicly known corpus (web, literature, public code) → doc 07 |
| **ΔO_private** | private corpus delta (proprietary logs, internal code) → doc 07 |
| **μ** | deterministic morphism: corpus → action → doc 07 |
| **μ_unrdf** | RDF/OWL reasoning instance of μ → doc 08 |
| **μ_mcp** | tool-call / code-write instance of μ → doc 08 |
| **μ_llm** | token-generation instance of μ → doc 08 |
| **self-specifying loop** | `observe → infer → propose → accept → execute → adapt → receipt` until fixed point → doc 37 |
| **fixed point** | BLAKE3 hash of corpus stops changing → doc 37 |

---

## 2. Repositories and product names

| Term | Meaning |
|---|---|
| **dteam** | deterministic process-intelligence engine; this repo → doc 01 |
| **ostar** / **O\*** | generation + receipts + release substrate → doc 08 |
| **unrdf** | RDF/OWL closure substrate; 24 packages → doc 08 |
| **wasm4pm** | process mining substrate; 38/38 conformance → doc 08 |
| **UniverseOS** | umbrella name for the closed-grammar substrate → doc 11 |
| **unibit** | physical substrate; L0–L2 layers → doc 12 |
| **unios** | public release surface; the single `#[no_mangle]` entry → doc 23 |
| **AtomVM** | precursor VM → doc 11 |
| **CodeManufactory** | the product (RevOps is a test case, not a product) → global CLAUDE.md |
| **ralph** | orchestration binary in dteam → dteam/CLAUDE.md |
| **PDC 2025** | Process Discovery Contest 2025 → doc 01 |

---

## 3. The two ladders

| Term | Meaning |
|---|---|
| **8ⁿ ladder** | kinetic work ladder; instruction cost → doc 09 |
| **64ⁿ ladder** | semantic memory ladder; residence size → doc 09 |
| **8⁶ = 64³ = 262,144 bits = 32 KiB** | the identity where work meets memory → doc 09 |
| **8¹ (8 bits)** | flag / micro-role tier → doc 28 |
| **8² (64 bits)** | word-bind / atomic mask tier → doc 28 |
| **8³ (512 bits)** | mini-hypervector tier → doc 28 |
| **8⁴ (4,096 bits)** | attention hypervector tier → doc 28 |
| **8⁵ (32,768 bits)** | active tile tier → doc 28 |
| **8⁶ (262,144 bits)** | operational-depth tier; full TruthBlock → doc 28 |
| **8⁷ (2,097,152 bits)** | L2-resident tier → doc 40 |
| **8⁸ (16,777,216 bits)** | L3/SLC-resident tier → doc 40 |
| **attention cell** | one 64² cell among 4,096 → doc 31 |
| **operational depth** | the 64³ active universe → doc 31 |
| **globe cell** | `(domain, cell, place)` triple → doc 19 |

---

## 4. Tier-discriminator types

| Term | Meaning |
|---|---|
| **`WorkTier`** | enum: `U8, U64, U512, U4096, U32768, U262144` → doc 29 |
| **`ResidenceTier`** | enum: `Reg, L1, L2, L3, Dram, Nvm` → doc 40 |
| **`ReceiptMode`** | enum: `None, Fragment, Chain` → doc 29 |
| **`FieldLane`** | enum: `Prereq, Law, Capability, Scenario, RiskReward, Causality, Conformance, Attention` → doc 26 |
| **`InstrFlags`** | enum: `Hot, Planning, Projection` → doc 30 |
| **`UOp`** | enum of opcodes → doc 30 |

---

## 5. Core data structures

| Term | Meaning |
|---|---|
| **`HotRegion`** | `align(4096)` pinned L1D page (64 KiB) → doc 40 |
| **`TruthBlock`** | 4,096 × u64 words = 32 KiB current state → doc 25 |
| **`Scratchpad`** | 4,096 × u64 shadow of TruthBlock → doc 25 |
| **`PackedEightField`** | 8 × `FieldMask` = 256 B → doc 36 |
| **`FieldMask`** | `{ required: u128, forbidden: u128 }` → doc 36 |
| **`DeltaRing`** | 256 × `Delta` journal → doc 40 |
| **`Delta`** | `{ word: u32, old: u64, new: u64 }` → doc 40 |
| **`ReceiptRing`** | ring buffer of u128 fragments → doc 40 |
| **`Hv<const T: WorkTier>`** | hypervector at compile-time tier → doc 29 |
| **`HdcSig128`** | folded 128-bit signature → doc 33 |
| **`MotionPacket<OP, T, RECEIPT>`** | compiled motion; tier + receipt at type level → doc 29 |
| **`Snapshot`** | sealed final state + chain tail; recursive via `.inner` → doc 48 |
| **`SignedSnapshot`** | `Snapshot` + signer + sig → doc 48 |
| **`LanePolicy`** | per-lane mask + mode + strikes → doc 48 |
| **`LaneMode`** | enum: `Passive, Active, Escalating, Commandeering` → doc 48 |
| **`LaneOutcome`** | enum: `Admit, Warn, Deny, Promote, Override` → doc 48 |
| **`Watchdog`** | deadman-timer cache line → doc 44 |
| **`WatchdogShards`** | per-core `[Watchdog; 8]` → doc 46 |
| **`SpscRing<T, const N>`** | single-producer single-consumer ring → doc 48 |
| **`ReduceBuffer`** | L2-shared 8-slot OR-reduce buffer → doc 46 |
| **`LaneHotRegion<LANE>`** | per-core 32 KiB slice for one lane → doc 46 |

---

## 6. POWL languages

| Term | Meaning |
|---|---|
| **POWL** | Partially Ordered Workflow Language → doc 10 |
| **POWL8** | kinetic dialect; partial order over work tiers → doc 45 |
| **POWL64** | geometric dialect; partial order over residence coordinates → doc 45 |
| **`HPowl<T>`** | hyperdimensional POWL AST (Sequence/Parallel/Choice/Activity) → doc 29 |
| **`Motion<T>`** | `(POWL8, POWL64)` lockstep pair → doc 45 |
| **sequence** | POWL seq operator; emits trajectory hv → doc 29 |
| **parallel** | POWL par operator; emits bundle hv → doc 29 |
| **choice** | POWL choice operator; emits choice hv → doc 29 |
| **trajectory** | hv encoding of sequential order → doc 29 |
| **bundle** | hv encoding of concurrent presence → doc 29 |
| **Geodesic** | POWL64 node: lawful path between two cells → doc 45 |
| **Concur** | POWL64 concurrent occupation → doc 45 |
| **Fork** | POWL64 branching node → doc 45 |
| **Residence** | POWL64 cache-tier change node → doc 45 |
| **Descend** | POWL64 entry into nested `Snapshot.inner` → doc 45 |

---

## 7. ISA / ABI / MLA

| Term | Meaning |
|---|---|
| **ISA** | Instruction Set Architecture → doc 40 |
| **ABI** | Application Binary Interface → doc 40 |
| **MLA** | Memory Layout Agreement → doc 40 |
| **UHDC** | UniverseOS Hyperdimensional Computing ISA → doc 30 |
| **`UInstr<OP,TIER,FIELD,RECEIPT,FLAGS>`** | typed instruction; five const params → doc 30 |
| **AEF.t** | fused `ADMIT.t.*.*` + `COMMIT.t` + `FRAG.t` superop → doc 40 |
| **ADMIT.t.f.r** | admit at tier, field, receipt mode → doc 40 |
| **COMMIT.t** | branchless select at tier → doc 40 |
| **REDUCE.n** | OR-reduce n ∈ {2,4,8} lanes → doc 40 |
| **FRAG.t** | emit receipt fragment → doc 40 |
| **FOLD.t→t'** | fold hypervector from tier t to smaller t' → doc 40 |
| **PROMOTE.t→t+1** | tier promotion (cold path) → doc 40 |
| **DEMOTE.t→t-1** | tier demotion (hot path) → doc 40 |
| **PIN** | semantic-position validate (L0) → doc 40 |
| **SEAL** | BLAKE3 receipt close (L5) → doc 40 |
| **superinstruction** | fused sequence compiled by MuStar → doc 34 |
| **opcode encoding** | `[tier:3 \| field:3 \| class:2]` one byte → doc 40 |

---

## 8. Admission algebra

| Term | Meaning |
|---|---|
| **admission** | does the state satisfy the lane's mask? → doc 36 |
| **deny / denial** | the negation of admission → doc 36 |
| **commit** | branchless select: `(candidate & admitted_mask) \| (old & !admitted_mask)` → doc 36 |
| **required mask** | bits that must be set → doc 36 |
| **forbidden mask** | bits that must be clear → doc 36 |
| **`missing_required`** | `(state & required) XOR required` → doc 36 |
| **`forbidden_present`** | `state & forbidden` → doc 36 |
| **deny_bits** | `missing_required \| forbidden_present` → doc 36 |
| **admitted_mask** | `((deny == 0) as u64).wrapping_neg()` → doc 34 |
| **branchless** | no conditional branch; select via mask → doc 34 |
| **mask calculus** | the algebra of `AND / OR / XOR / NOT` on u64/u128 masks → doc 34 |

---

## 9. Bit and SIMD operations

| Term | Meaning |
|---|---|
| **AND / &** | bitwise AND |
| **OR / \|** | bitwise OR |
| **XOR / ^** | bitwise exclusive OR |
| **NOT / !** | bitwise complement |
| **BIC** | AND-NOT (aarch64 instruction) |
| **popcount** | count the set bits; `u64::count_ones()` → doc 32 |
| **`wrapping_neg`** | two's-complement negation of unsigned (mask trick) → doc 34 |
| **Hamming distance** | `popcount(a XOR b)` → doc 27 |
| **folded signature** | `Hv<U4096>` → `HdcSig128` via deterministic reduction → doc 33 |
| **`portable_simd`** | nightly Rust SIMD crate (`core::simd`) → doc 29 |
| **NEON** | Apple Silicon / ARM SIMD unit (128-bit) → doc 34 |
| **AVX2 / AVX-512** | x86 SIMD → doc 40 |
| **`#[target_feature]`** | opt-in compiler feature annotation → doc 34 |
| **prefetch** | `__prefetch` hint; warms next tile → doc 40 |

---

## 10. HDC primitives

| Term | Meaning |
|---|---|
| **HDC** | Hyperdimensional Computing → doc 24 |
| **Kinetic HDC** | HDC disciplined by 8ⁿ tier → doc 28 |
| **binding** | role-value combination (XOR) → doc 27 |
| **bundling** | context superposition (majority / OR) → doc 27 |
| **permutation** | sequential encoding (rotation) → doc 27 |
| **similarity** | Hamming distance compare → doc 27 |
| **associative memory** | nearest-lawful prototype lookup → doc 27 |
| **cleanup** | snap denoised vector to nearest prototype → doc 27 |
| **repair** | find lawful alternative for denied motion → doc 30 |
| **field vector** | per-lane `Hv<T>` with type-level lane → doc 29 |
| **progressive admission** | try 8² → 8³ → … escalate only on tight margin → doc 28 |
| **Hyperdimensional Workflow Geometry (HDWG)** | formal object `(X, V, F, M, Π, R)` → doc 24 |

---

## 11. Verification and receipts

| Term | Meaning |
|---|---|
| **receipt** | cryptographic proof of lawful motion → doc 07 |
| **receipt fragment** | one u128 emitted per tick → doc 40 |
| **receipt chain** | BLAKE3-sealed fragment sequence → doc 35 |
| **five verification surfaces** | Execution, Telemetry, State, Process log, Causality → doc 37 |
| **release gate** | all 5 surfaces must agree → doc 37 |
| **quarantine** | what happens to motions that fail the gate → doc 35 |
| **BLAKE3** | the chosen hash for seals → doc 37 |
| **L0 fragment** | position hash — `unibit-phys` → doc 35 |
| **L1 fragment** | kernel i/o hash — `unibit-hot` → doc 35 |
| **L2 fragment** | instruction id + source commitment — `unibit-isa` → doc 35 |
| **L3 fragment** | compile commitment — `compile` crate → doc 35 |
| **L4 fragment** | conformance score — `dteam` → doc 35 |
| **L5 fragment** | release signature — `unios` → doc 35 |
| **verify fn** | `pub fn verify(s: &Snapshot) -> Result<(), VerifyError>` → doc 48 |

---

## 12. Cache and memory

| Term | Meaning |
|---|---|
| **cache line** | 64 B; minimum coherence unit → doc 34 |
| **page** | 4,096 B (4 KiB); alignment for HotRegion → doc 40 |
| **`align(64)`** | cache-line alignment attribute → doc 34 |
| **`align(4096)`** | page alignment attribute → doc 40 |
| **`Pin<Box<T>>`** | heap-pinned ownership → doc 43 |
| **mlock** | lock page in physical memory → doc 43 |
| **position validation** | assert virtual address at pin matches boot-receipt → doc 43 |
| **L1BootReceipt** | semantic-position commitment at startup → doc 16 |
| **L1D** | per-core data cache (128 KiB on M3 Max P-core) → doc 25 |
| **L1I** | per-core instruction cache → doc 42 |
| **L2** | per-cluster shared cache (16 MiB P-cluster) → doc 40 |
| **L3 / SLC** | System-Level Cache (48 MiB on M3 Max) → doc 40 |
| **DRAM / LPDDR5** | main memory → doc 40 |
| **NVM** | non-volatile memory / disk → doc 40 |
| **false sharing** | unrelated writes to same cache line → doc 34 |
| **coherence storm** | many cores writing one line → doc 46 |
| **federated counter** | per-core shards to avoid coherence storm → doc 46 |

---

## 13. Hardware

| Term | Meaning |
|---|---|
| **M3 Max** | Apple Silicon SoC → doc 25 |
| **P-core** | performance core → doc 25 |
| **E-core** | efficiency core → doc 25 |
| **P-cluster** | 4 P-cores sharing L2 → doc 46 |
| **E-cluster** | 4 E-cores sharing L2 → doc 46 |
| **SoC** | system on chip → doc 42 |
| **IPI** | inter-processor interrupt → doc 42 |
| **register file** | ~256 × 128-bit NEON registers per core → doc 40 |

---

## 14. Process mining

| Term | Meaning |
|---|---|
| **XES** | eXtensible Event Stream; log format → doc 01 |
| **OCEL** | Object-Centric Event Log → doc 08 |
| **Petri net** | place/transition/arc process model → doc 03 |
| **place** | Petri-net state holder; token-carrier → doc 03 |
| **transition** | Petri-net event handler → doc 03 |
| **arc** | directed edge between place and transition → doc 03 |
| **marking** | distribution of tokens across places → doc 03 |
| **token-based replay** | walk trace through net, track tokens → doc 03 |
| **bitmask replay** | u64 marking (≤64 places) → doc 05 |
| **`NetBitmask64`** | bitmask Petri net representation → doc 05 |
| **`replay_trace`** | one trace through a net → doc 05 |
| **`ReplayResult`** | `{ missing, remaining, produced, consumed }` → doc 05 |
| **fitness** | fraction of trace lawfully replayable → doc 03 |
| **precision** | fraction of model behavior used by log → doc 03 |
| **simplicity** | model size penalty → doc 03 |
| **generalization** | model generalizes beyond training log → doc 03 |
| **conformance** | agreement between log and model → doc 03 |
| **trace** | one case / one execution instance → doc 01 |
| **event** | one activity occurrence → doc 01 |
| **case** | one process instance → doc 01 |
| **alpha miner** | classic discovery algorithm → — |
| **inductive miner** | structured discovery algorithm → — |
| **BaseDfg** | directly-follows graph baseline → doc 03 |
| **HPowl discovery** | POWL discovery over hypervector lane → doc 29 |
| **288 training logs** | PDC 2025 training set → doc 31 |
| **96 test logs** | PDC 2025 evaluation set → doc 31 |

---

## 15. RL / ML

| Term | Meaning |
|---|---|
| **Q-learning** | off-policy TD control → dteam/CLAUDE.md |
| **Double Q-learning** | two Q-tables to fight bias → dteam/CLAUDE.md |
| **SARSA** | on-policy TD control → dteam/CLAUDE.md |
| **Expected SARSA** | SARSA with expectation over next action → dteam/CLAUDE.md |
| **REINFORCE** | policy-gradient algorithm → dteam/CLAUDE.md |
| **LinUCB** | linear-UCB contextual bandit → dteam/CLAUDE.md |
| **ε-greedy** | exploration policy → — |
| **`PackedKeyTable`** | dense indexing + `fnv1a_64`, replaces HashMap → dteam/CLAUDE.md |
| **`DenseIndex`** | integer-key mapping for PKT → doc utils |
| **`fnv1a_64`** | 64-bit FNV-1a hash → dteam/CLAUDE.md |
| **`RlState<const WORDS>`** | stack-allocated RL state → dteam/CLAUDE.md |
| **β / λ wiring** | soundness bonus / complexity penalty in reward → doc 02 |
| **`train_with_provenance`** | RL loop over `ProjectedLog` → doc 02 |

---

## 16. Build system and Rust surface

| Term | Meaning |
|---|---|
| **cargo make** | task runner used in dteam/unibit → global CLAUDE.md |
| **mvnd** | Maven Daemon (Java projects) → global CLAUDE.md |
| **typer** | preferred Python CLI framework → global CLAUDE.md |
| **release-hot profile** | `lto=fat`, `codegen-units=1`, `panic=abort`, `strip=symbols`, `trim-paths` → doc 38 |
| **nightly toolchain** | required for const generics and portable SIMD → doc 15 |
| **`#![no_std]`** | do not link std → doc 29 |
| **`#[repr(C, align(N))]`** | explicit layout and alignment → doc 40 |
| **`#[inline(always)]`** | force-inline hot path → doc 34 |
| **`#[inline(never)]`** | keep cold path out of icache → doc 34 |

---

## 17. Rust nightly features used

| Feature | Why |
|---|---|
| `generic_const_exprs` | `[u64; WORK_WORDS::<T>]` sizing → doc 29 |
| `adt_const_params` | enums as const generics → doc 29 |
| `generic_const_items` | const items parameterized by generics → doc 29 |
| `const_trait_impl` | const impls of traits → doc 29 |
| `portable_simd` | `core::simd` → doc 29 |
| `strict_provenance_lints` | pointer provenance discipline → doc 29 |
| `naked_functions` / `naked_asm!` | last-mile inner loops → doc 34 |
| `negative_impls` | `!Sync` on Runner → doc 43 |
| `coroutines` / `coroutine_trait` | background task shape → doc 44 |
| `never_type` | `!` return → doc 44 |

---

## 18. Crate layout (Rust-core-idiomatic, doc 48)

| Crate | Role |
|---|---|
| `unibit-phys` | pinned memory, alignment, position validation |
| `unibit-hot` | admit/commit/reduce kernels, NEON |
| `unibit-isa` | typed `UInstr`, `WorkTier`, `FieldLane` |
| `compile` | HPowl → MotionPacket (was "mustar" / "Wintermute") |
| `runner` | HotRegion + `step` + `finalize` |
| `lane-policy` | `LanePolicy`, `LaneMode`, `LaneOutcome` |
| `watchdog` | `Watchdog`, `WatchdogShards` |
| `ring` | `SpscRing<T, const N>` |
| `verify` | `verify(&Snapshot) -> Result` |
| `chain-analyzer` | attribution fns (was "marly" / "chronicler") |
| `orphan-assembler` | background Snapshot assembly (was "boxmaker" / "curator") |
| `endpoint` | `Endpoint` trait + `transfer` fn |
| `dteam` | XES/OCEL ingestion, discovery, conformance, RL |
| `unios` | release surface, single `#[no_mangle]` entry |

---

## 19. Canonical fns (Rust-core-idiomatic)

| Fn | Signature | Role |
|---|---|---|
| `step` | `(&mut HotRegion, &MotionPacket) -> StepOutcome` | one admit-commit-emit |
| `finalize` | `(HotRegion) -> Snapshot` | seal and consume |
| `compile` | `(&HPowl<T>) -> MotionPacket<..>` | intent → packet |
| `verify` | `(&Snapshot) -> Result<(), VerifyError>` | independent re-seal check |
| `transfer` | `(&S: Endpoint, &D: Endpoint, [u8;32]) -> Result<()>` | cross-endpoint move |
| `assemble` | `(&OrphanPool) -> Option<Snapshot>` | E-core orphan assembly |
| `analyze` | `(&Snapshot) -> ChainReport` | attribution analysis |
| `rehearse` | `(&Snapshot, candidate, &Gate) -> LaneOutcome` | counterfactual replay |

---

## 20. Benchmark numbers

### Baselines (measured, single-core)

| Op | Latency |
|---|---|
| action selection | 6.95 ns |
| Q-learning update | 14.87 ns |
| SARSA update | 17.53 ns |
| `PackedKeyTable` lookup | 23.30 ns |
| double Q-learning | 27.67 ns |

### Targets (per tier, projected)

| Tier | Target |
|---|---|
| 8¹ | < 2 ns |
| 8² | < 10 ns |
| 8³ | < 100 ns |
| 8⁴ | < 200 ns |
| 8⁵ | < 500 ns |
| 8⁶ | < 5 µs |

### 8⁶ instruction floor (SIMD ops over 4,096 u64 words)

| Class | Ops |
|---|---|
| fused admission | ~10,240 |
| 8-field aggregate | ~81,920 |
| commit / delta | ~18,432 |
| fused admission + commit | ~28,672 |

### 8-core pantheon targets (doc 46)

| Metric | Target |
|---|---|
| 8-lane admission critical path | ~35 ns |
| motions / sec / core | ~300 M |
| motions / sec / 8 cores | ~228 M (balanced) |

---

## 21. PDC 2025 facts

| Fact | Value |
|---|---|
| Training logs | 288 |
| Test / base log pairs | 96 |
| Max places in any model | 64 |
| Place distribution | 36, 40, 44, 48, 52, 56, 60, 64 |
| Bitmask-replay accuracy (initial) | 67.29% |
| All models fit `u64` marking | yes |

---

## 22. Lexicon / naming rules

| Term | Meaning |
|---|---|
| **lexicon law** | forbidden storage-noun vocabulary in unibit docs/code → doc 18 |
| **`check-lexicon.mjs`** | build-time scanner → doc 18 |
| **`FORBIDDEN_LITERARY`** | Gibson names forbidden in source (not comments) → doc 47 |
| **`FORBIDDEN_ROLES`** | anthropomorphic role-names forbidden anywhere → doc 48 |
| **Rust API Guidelines C-CASE, C-GOOD-ERR, C-WORD-ORDER** | naming convention references → doc 48 |

---

## 23. Literary ↔ canonical map (docs 43–48)

| Literary (Gibson) | Canonical (doc 48) |
|---|---|
| Matrix / Cyberspace | — (prose only) |
| Straylight / Deck | `HotRegion` |
| Cowboy | — (becomes `step` fn) |
| Jack in | `HotRegion::pin()` |
| Ice / White / Gray / Black | `Gate { policy: GatePolicy { Deny, Escalate, Counter } }` |
| Construct / Dixie Flatline | `Snapshot` |
| Flatline | `finalize` fn |
| Wintermute | `compile` fn |
| Neuromancer | `step` fn |
| Turing Police | `verify` fn |
| Simstim | `rehearse` fn |
| Zaibatsu | `Endpoint` trait |
| Chiba / Ninsei | `Archive` |
| Count Zero Interrupt | `Watchdog` / `WatchdogShards` |
| Loa | `LanePolicy` |
| Legba / Ougou / Erzulie / Samedi / Danbala / MetKalfu / Simbi / Ayizan | `FieldLane` variants |
| Mood | `LaneMode` |
| LoaVerdict | `LaneOutcome` |
| Ridden / Mount | `LaneMode::Commandeering` / `LaneOutcome::Override` |
| Aleph | `Snapshot.inner` (recursive) |
| Finn | `SpscRing<T, const N>` |
| Biosoft | `SignedSnapshot` |
| Virek | background task (no type) |
| Turner | `transfer` fn |
| Marly | `analyze` fn / `chain-analyzer` crate |
| Boxmaker | `assemble` fn / `orphan-assembler` crate |
| Mitchell | — (human role) |
| Sniffer | `LexiconCheck` |

---

## 24. Terms used only in prose (no code)

| Term | Meaning |
|---|---|
| **Buckminster Fuller stance** | "only work with companies willing to start from scratch" → doc 06 |
| **Blue River Dam** | metaphor for controlling the bit-supply upstream → doc 21 |
| **Conway's / Little's law** | two laws inverted at N=2 operator regime → doc 23 |
| **DFLSS Pugh matrix** | O\* +87, IaC −16, docs 0 → doc 23 |
| **Big 4 memo** | synthetic board-level memo on True/False economics → doc 21 |
| **CAO transcript** | synthetic Chief AI Officer boardroom transcript → doc 22 |
| **Chatman doctrine sentence** | "The product is CodeManufactory; RevOps is merely proof" |
| **Pragmatic Programmer rules** | DRY, orthogonality, reversibility, tracer bullets → doc 34 |

---

## 25. Alphabetical index (A–Z)

Quick lookup of every term above. Reference back to the section listed.

```
A   admission ..................... §8
    admitted_mask ................. §8
    AEF.t ......................... §7
    AND ........................... §9
    AND-NOT / BIC ................. §9
    A = μ(O*) ..................... §1
    align(64) / align(4096) ....... §12
    alpha miner ................... §14
    analyze fn .................... §19
    arc ........................... §14
    Archive ....................... §23
    assemble fn ................... §19
    associative memory ............ §10
    attention cell ................ §3
    attention (lane) .............. §4, §23
    AtomVM ........................ §2

B   β / λ wiring .................. §15
    BaseDfg ....................... §14
    binding ....................... §10
    bitmask replay ................ §14
    BLAKE3 ........................ §11
    branchless .................... §8
    Broker (forbidden) ............ §22, §23
    bundle (HPowl) ................ §6
    bundling (HDC) ................ §10

C   cache line .................... §12
    Cartridge (forbidden) ......... §23
    Capsule (forbidden) ........... §23
    cargo make .................... §16
    case (PM) ..................... §14
    Chatman equation .............. §1
    choice (POWL) ................. §6
    chain-analyzer crate .......... §18
    check-lexicon.mjs ............. §22
    Cl() .......................... §1
    cleanup ....................... §10
    CodeManufactory ............... §2
    coherence storm ............... §12
    commit ........................ §8
    COMMIT.t ...................... §7
    compile fn .................... §19
    compile crate ................. §18
    conformance ................... §14
    const_trait_impl .............. §17
    Concur ........................ §6
    Construct (forbidden) ......... §23
    Conway's law .................. §24
    coroutines .................... §17
    count_ones .................... §9
    Count Zero Interrupt .......... §23
    Cowboy (forbidden) ............ §23
    Curator (forbidden) ........... §23

D   Danbala ....................... §23
    Delta ......................... §5
    DeltaRing ..................... §5
    DEMOTE ........................ §7
    DenseIndex .................... §15
    deny / denial ................. §8
    Descend ....................... §6
    dteam ......................... §2
    double Q-learning ............. §15, §20
    DRAM .......................... §12
    DX/QoL ........................ §23

E   E-cluster ..................... §13
    E-core ........................ §13
    Endpoint ...................... §18
    Envelope (forbidden) .......... §23
    Erzulie ....................... §23
    Executor (forbidden) .......... §22
    Expected SARSA ................ §15

F   false sharing ................. §12
    federated counter ............. §12
    FieldLane ..................... §4
    FieldMask ..................... §5
    finalize fn ................... §19
    Finn (forbidden) .............. §23
    fixed point ................... §1
    Flatline (forbidden) .......... §23
    fnv1a_64 ...................... §15
    FOLD.t→t' ..................... §7
    folded signature .............. §9
    forbidden mask ................ §8
    forbidden_present ............. §8
    FORBIDDEN_LITERARY ............ §22
    FORBIDDEN_ROLES ............... §22
    Fork .......................... §6
    FRAG.t ........................ §7

G   Gate .......................... §5
    GatePolicy .................... §5
    generalization ................ §14
    generic_const_exprs ........... §17
    Geodesic ...................... §6
    globe cell .................... §3
    glossary ...................... §23

H   Hamming distance .............. §9
    HdcSig128 ..................... §5
    HDWG .......................... §10
    HotRegion ..................... §5
    HPowl<T> ...................... §6
    Hv<const T: WorkTier> ......... §5
    Hyperdimensional Computing .... §10

I   Ice (forbidden) ............... §23
    #[inline(always/never)] ....... §16
    InstrFlags .................... §4
    instruction floor ............. §20
    Intensity ..................... §6
    IPI ........................... §13
    ISA ........................... §7

J   Jack in (forbidden) ........... §23

K   kinetic HDC ................... §10

L   L0..L5 fragments .............. §11
    L1BootReceipt ................. §12
    L1D / L1I ..................... §12
    L2 / L3 / SLC ................. §12
    LaneMode ...................... §5
    LaneOutcome ................... §5
    LanePolicy .................... §5
    LaneHotRegion<LANE> ........... §5
    lane-policy crate ............. §18
    Law (lane) .................... §4
    Legba ......................... §23
    LexiconCheck .................. §22
    lexicon law ................... §22
    LinUCB ........................ §15
    Little's law .................. §24
    Loa (forbidden) ............... §23
    lockstep invariant ............ §6
    LPDDR5 ........................ §12
    lto = fat ..................... §16

M   M3 Max ........................ §13
    Marly (forbidden) ............. §23
    marking ....................... §14
    mask calculus ................. §8
    mcp ........................... §1
    MetKalfu ...................... §23
    Mitchell (not a type) ......... §23
    missing_required .............. §8
    mlock ......................... §12
    Motion<T> ..................... §6
    MotionPacket<...> ............. §5
    Mount / Override .............. §23
    μ (morphism) .................. §1

N   naked_functions ............... §17
    negative_impls ................ §17
    NEON .......................... §9
    NetBitmask64 .................. §14
    Neuromancer (forbidden) ....... §23
    never_type .................... §17
    nightly ....................... §16, §17
    #[no_mangle] .................. §34 (in doc 34)
    NVM ........................... §12

O   O* / Cl / O_public / ΔO_private §1
    OCEL .......................... §14
    opcode encoding ............... §7
    operational depth ............. §3
    orphan-assembler crate ........ §18
    ostar / O* .................... §2
    Ougou Feray ................... §23
    Override (LaneOutcome) ........ §5

P   P-cluster ..................... §13
    P-core ........................ §13
    PackedEightField .............. §5
    PackedKeyTable ................ §15
    page .......................... §12
    panic = abort ................. §16
    parallel (POWL) ............... §6
    PhantomPinned ................. §17
    Pin<Box<T>> ................... §12
    PIN (instruction) ............. §7
    place (PM) .................... §14
    PDC 2025 ...................... §2, §21
    popcount ...................... §9
    portable_simd ................. §9, §17
    position validation ........... §12
    POWL / POWL8 / POWL64 / HPowl . §6
    precision ..................... §14
    prefetch ...................... §9
    Prereq (lane) ................. §4
    progressive admission ......... §10
    PROMOTE ....................... §7

Q   Q-learning .................... §15

R   rehearse fn ................... §19
    ralph ......................... §2
    receipt / receipt fragment .... §11
    receipt chain ................. §11
    ReceiptMode ................... §4
    ReceiptRing ................... §5
    REDUCE.n ...................... §7
    ReduceBuffer .................. §5
    REINFORCE ..................... §15
    release-hot profile ........... §16
    release gate .................. §11
    repair ........................ §10
    Residence ..................... §6
    ResidenceTier ................. §4
    RevOps ........................ §2
    ReplayResult .................. §14
    replay_trace .................. §14
    required mask ................. §8
    RiskReward (lane) ............. §4
    ring crate .................... §18
    RlState<const WORDS> .......... §15
    runner crate .................. §18

S   Samedi ........................ §23
    SARSA ......................... §15
    Scenario (lane) ............... §4
    Scratchpad .................... §5
    SEAL .......................... §7
    SelfSpecifyingLoop ............ §1
    self-specifying loop .......... §1
    sequence (POWL) ............... §6
    SignedSnapshot ................ §5
    SimbI ......................... §23
    Simstim (forbidden) ........... §23
    similarity .................... §10
    simplicity .................... §14
    Simulator (forbidden) ......... §22
    SLC ........................... §12
    Snapshot ...................... §5
    SpscRing<T, const N> .......... §5
    state (verification surface) .. §11
    step fn ....................... §19
    Straylight (forbidden) ........ §23
    strict_provenance_lints ....... §17
    superinstruction .............. §7

T   #[target_feature] ............. §9
    TinyML HDC .................... — (doc 27)
    token-based replay ............ §14
    trace (PM) .................... §14
    trajectory (HPowl) ............ §6
    transfer fn ................... §19
    transition (PM) ............... §14
    Truing Police (forbidden) ..... §23
    trim-paths .................... §16
    TruthBlock .................... §5
    Turner (forbidden) ............ §23

U   UHDC .......................... §7
    UInstr<...> ................... §7
    UOp ........................... §4
    unibit ........................ §2
    unibit-hot / -isa / -phys crates §18
    unibit-nightly-smoke .......... doc 16
    unrdf ......................... §2
    unios ......................... §2
    UniverseOS .................... §2

V   verify crate .................. §18
    verify fn ..................... §19
    Verifier (forbidden) .......... §22
    Virek (forbidden) ............. §23

W   Warden (forbidden) ............ §22, §23
    wasm4pm ....................... §2
    Watchdog ...................... §5
    WatchdogShards ................ §5
    Wintermute (forbidden) ........ §23
    WorkTier ...................... §4
    wrapping_neg .................. §9

X   XES ........................... §14
    XOR ........................... §9
    XOR-popcount .................. §9

Z   Zaibatsu (forbidden) .......... §23
```

---

## 26. The sentence

**Every term in the archive maps to exactly one place in this index —
from the top-level Chatman equation `A = μ(O*)` through the two 8ⁿ /
64ⁿ ladders, the typed `UInstr<OP, TIER, FIELD, RECEIPT, FLAGS>`, the
branchless `deny = missing_required | forbidden_present`, down to the
hardware-level `popcount` and `__prefetch` — and anything not in this
glossary should be renamed, documented, or removed before source
touches silicon.**
