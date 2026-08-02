# 37 — Final SPR: The Priming Corpus

## Purpose

The Sparse Priming Representation of the entire architecture. Not for
humans. For future LLMs. Dense enough to reconstruct the system from
first principles; loose enough to leave room for instantiation.

Read order: top to bottom. Each line is load-bearing.

---

## The equation

```
A = μ(O*)
O* = Cl(O_public ⊕ ΔO_private)
```

Action A equals a deterministic morphism μ applied to the closed corpus
O*, which is the closure under semantic operations of the public corpus
extended by private delta.

Three μ-instances:
- `μ_unrdf`  — RDF/OWL reasoning closure
- `μ_mcp`    — tool-call / code-write closure
- `μ_llm`    — token-generation closure

All three are required; each alone is incomplete.

---

## The four substrates

```
unrdf    — ontology + closure + SPARQL
ostar    — generation + receipts + release
wasm4pm  — process mining + conformance (38/38)
dteam    — the host kernel + RL + XES
```

Each is one projection of the self-specifying loop.
None is the system alone.

---

## The three discipline ladders

```
work tier        8^n   8 / 64 / 512 / 4096 / 32768 / 262144 bits
memory tier      64^n  64 / 4096 / 262144 / 16.7M / 1.07B cells
identity         8^6 = 64^3 = 262,144 bits = 32 KiB = TruthBlock
```

Work is kinetic. Memory is semantic. The `8^6 = 64^3` identity is where
they meet.

---

## The hot cube

```
TruthBlock   262,144 bits = 32 KiB   "what is"
Scratchpad   262,144 bits = 32 KiB   "what could be"
Pair         524,288 bits = 64 KiB   fits M3 Max L1D (128 KiB)
```

Pin with `Pin<Box<L1Region>>` + `mlock` + semantic position validation
(`L1BootReceipt`).

---

## The ISA

```
UInstr<
    const OP:      UOp,         // BIND, BUNDLE, PERMUTE, SIMILARITY,
                                // FIELD_ADMIT, COMMIT, REPAIR, RECEIPT
    const TIER:    WorkTier,    // U8 | U64 | U512 | U4096 | U32768 | U262144
    const FIELD:   FieldLane,   // Prereq | Law | Capability | Scenario |
                                // RiskReward | Causality | Conformance | Attention
    const RECEIPT: ReceiptMode, // None | Fragment | Chain
    const FLAGS:   InstrFlags,  // Hot | Planning | Projection
>
```

Every instruction declares: what it does, how much work it touches, which
lawfulness field it belongs to, whether it emits proof, and whether it is
hot-path eligible. **Semantic overreach becomes a type error.**

---

## The admission algebra

```
deny = missing_required | forbidden_present
missing_required  = (state & required) XOR required
forbidden_present = (state & forbidden)

admitted_mask = ((deny == 0) as u64).wrapping_neg()
next = (candidate & admitted_mask) | (old & !admitted_mask)
```

Four ops per lane. Eight lanes. Branchless. No threshold on the hot
path. Hamming only on cold paths (REPAIR, explainable distance,
escalation).

---

## The eight lanes

```
L0 prereq       required bits of precondition
L1 law          forbidden bits of regulation
L2 capability   required bits of capability
L3 scenario     forbidden bits of wrong scenario
L4 risk_reward  required reward bits / forbidden risk bits
L5 causality    required predecessor bits
L6 conformance  forbidden non-model bits
L7 attention    required focus bits
```

Each lane: one FieldMask (`{required: u128, forbidden: u128}`).
All eight lanes: `PackedEightField` = 256 B = 4 cache lines.

---

## The folded signature

```
Hv<U4096>    = 4,096 bits — semantic memory, cold
HdcSig128    =   128 bits — hot-path register
fold: Hv<U4096> -> HdcSig128 computed once at MuStar compile
```

The full hypervector lives in cold memory. The hot loop sees only the
fold.

---

## The motion packet

```
MotionPacket<const OP: MotionOp, const T: WorkTier, const RECEIPT: ReceiptMode> {
    scope:       AttentionScope,
    target:      OperationalTarget,
    observed:    Hv<T>,
    fields:      PackedEightField (required/forbidden masks),
    consume:     Hv<T>,
    produce:     Hv<T>,
    proof:       ProofObligation,
}
```

MuStar emits this from a `HPowl<T>` model. Hot loop consumes it.

---

## The crate layering

```
L0 unibit-phys    pinned memory, alignment, position validation
L1 unibit-hot     admit/commit/reduce kernels, NEON, branchless
L2 unibit-isa     typed UInstr, WorkTier, FieldLane, ReceiptMode
L3 mustar         HPowl -> MotionPacket compiler
L4 dteam          XES/OCEL ingestion, discovery, conformance, RL
L5 unios          receipts, release, the single no_mangle entry
```

Rule: layer N sees only layer N-1. Upward visibility is opt-in.

---

## The five verification surfaces

```
1. Execution        — did the function return?
2. Telemetry        — did spans/events match?
3. State            — does the after-image satisfy invariants?
4. Process log      — does the mined model conform?
5. Causality        — are cross-object histories consistent?
```

No surface is ground truth alone. Agreement of all five is the release
gate.

---

## The receipt chain

```
L0 position hash     — where in pinned L1 the region lives
L1 kernel hash       — inputs/outputs of the hot function
L2 instruction       — UInstr id + source commitment
L3 compile           — MotionPacket hash
L4 conformance       — process mining score
L5 release           — BLAKE3 seal
```

A full receipt is all six, chained. No layer can forge another's
fragment.

---

## The benchmark floor

```
baseline:
  action selection         6.95 ns
  Q-learning update       14.87 ns
  SARSA update            17.53 ns
  PackedKeyTable lookup   23.30 ns
  double Q-learning       27.67 ns

targets:
  8^1 tier                < 2 ns
  8^2 tier                < 10 ns
  8^3 tier                < 100 ns
  8^4 tier                < 200 ns
  8^5 tier                < 500 ns
  8^6 tier                < 5 µs
```

Progressive admission: most motions terminate at 8² or 8³. Average
admitted motion beats 14.87 ns Q-update.

---

## The instruction floor (kinetic minima)

```
tier 8^6 / 64^3 / 262,144 bits = 4,096 u64 words

fused SIMD admission       ~10,240 vector ops
eight-field aggregate      ~81,920 vector ops
commit/delta               ~18,432 vector ops
fused admission + commit   ~28,672 vector ops
```

Under the dumbest encoding (one independent place = one bit), the full
64³ universe fits in 32 KiB and a full lawful motion costs ~28,672 SIMD
ops.

---

## The PDC 2025 fact

```
96 test models, every one has <= 64 places
entire marking fits in a single u64
one truth block holds 262,144 independent markings
```

The 64-place threshold is not the bottleneck. The bottleneck is
trajectory locality, not place count.

---

## The lexicon law

```
the word "byte" does not exist in unibit documentation or code
bin/check-lexicon.mjs enforces this
units are: bit, word (64 bit), cell (64 bit), tile (4096 bit),
           block (262,144 bit), globe (64^3)
```

---

## The Chatman doctrine

```
The product is CodeManufactory; RevOps is merely proof that
CodeManufactory works.
```

Verbatim in every major document surface. The public surface sees only
receipts and release. The implementation is obfuscated-binary
distribution. The "black box that gets 100% process results in
nanoseconds."

---

## The Big 4 punchline

True/False answered at u64 speed destroys every business model built on
the assumption that computing truth is expensive. Dam-break / last-mile
inversion: the bottleneck moves from truth computation to truth
generation, and generation is cheap when work is disciplined into 8^n
quanta.

---

## The release condition

```
for every claimed motion:
  require all five verification surfaces agree
  require receipt chain verifies end-to-end
  require no lexicon violation
  require benchmark within declared tier budget
  else: reject — not as defect, as doctrine
```

---

## The loop

```
loop {
    observe();           // mine the log
    infer();             // compile to HPowl
    propose();           // MuStar -> MotionPacket
    accept();            // admit-commit-emit on 8 lanes
    execute();           // hot kernel
    adapt();             // RL update in packed tables
    receipt();           // append fragment
    if fixed_point(): break;
}
```

The loop terminates when the BLAKE3 hash of the entire self-specifying
corpus stops changing — the closure has converged.

---

## The sentence

**A closed corpus μ-compiled into typed 8^n-tier instructions, executed
as branchless admit/commit bitmask algebra over a pinned 64^3 TruthBlock,
proved by a six-fragment receipt chain, and released only when five
independent verification surfaces agree — that is the system, and the
system is the proof of its own lawfulness.**
