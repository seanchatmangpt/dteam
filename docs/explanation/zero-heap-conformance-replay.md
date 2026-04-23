# Why Zero-Heap Conformance Matters: Branchless Token Replay

*Diátaxis — Understanding-oriented explanation.*

---

## 1. The Allocation Tax: What Conventional Replay Costs

Token-based replay, the bedrock of process conformance checking, answers a deceptively simple question: given an event log and a Petri net model, did the observed traces follow the model? The canonical implementation treats a Petri net marking as a `HashMap<PlaceId, usize>` — a dictionary that maps each place to the number of tokens sitting in it. For a process with forty places and a thousand traces, that seems reasonable. The reality is more expensive than it looks.

Every trace requires allocating, populating, and eventually deallocating that HashMap. In Rust's allocator this is not free: `jemalloc` and `mimalloc` are fast, but they still touch the heap, which means cache-line evictions, potential allocator lock contention under parallelism, and variable-time behavior. The time to allocate a 40-entry HashMap is not a constant — it depends on the allocator's free-list state, the OS's memory pressure, and whether a prior allocation left the right-sized arena warm. That variable-time behavior is the first problem: it breaks the guarantee that replay latency is proportional only to trace length.

The second problem is more subtle. Conformance replay is frequently used inside tight optimization loops — RL discovery epochs, ensemble evaluation passes, or PDC contest scoring. If each inner iteration allocates and frees memory, the garbage in those free-lists grows with iteration count. What was fast at epoch 1 may be measurably slower at epoch 1000, not because the algorithm changed, but because the allocator's state did. This is the allocation tax: the hidden per-call cost that accumulates invisibly across the run.

The third problem is audit-grade determinism. In process mining for regulatory or forensic contexts, two replay runs on the same log and net must produce identical timing profiles, not just identical results. Heap allocation introduces non-determinism through allocator state. Zero-heap replay eliminates that non-determinism entirely — the only input is the net masks and the event sequence.

The zero-heap path in `dteam` exists precisely to pay none of this tax.

---

## 2. The Bitmask Insight: Marking as a Register

The central insight behind zero-heap replay is that a 1-safe Petri net marking is not a counter — it is a membership set. In a 1-safe net, each place holds either zero or one token: a binary proposition. The marking is therefore a set of places currently holding a token. Sets have a natural representation: bitvectors.

If the net has at most 64 places, the entire marking fits in a single `u64`. Place 0 corresponds to bit 0, place 1 to bit 1, and so on. A token in place *k* is represented as the *k*-th bit being set. The marking `{p0, p2, p5}` is the integer `0b100101`. No heap, no indirection, no allocator — one CPU register holds the complete state of the net.

The `NetBitmask64` struct captures this encoding for the entire net at construction time:

```rust
pub struct NetBitmask64 {
    pub initial_mask: u64,
    pub final_mask: u64,
    pub n_places: usize,
    pub(crate) transitions: Vec<TransMask>,
    pub(crate) label_index: Vec<(String, Vec<usize>)>,
    pub(crate) invisible_indices: Vec<usize>,
}
```

`initial_mask` encodes the initial marking. `final_mask` encodes the acceptance condition. Each `TransMask` stores an `in_mask` (the set of places that must hold tokens for the transition to be enabled) and an `out_mask` (the set of places that receive tokens after firing). These masks are computed once when converting from a `PetriNet`, then reused across every trace replay without further allocation.

This is the filing cabinet analogy: a conventional marking is a filing cabinet full of folders, one per place, each holding a count. The bitmask marking is a single light switch panel with one switch per place — on means token present, off means absent. You can read or flip the entire panel in a single CPU instruction.

---

## 3. The State Equation

Once the marking is a register, transition firing becomes arithmetic. In classical Petri net theory, the state equation is written M' = M + C·x, where C is the incidence matrix (the net's structure encoded as a matrix of +1/−1 entries) and x is a firing vector with a 1 in the position of the fired transition. For the 1-safe bitmask case, this reduces to something far more efficient.

Let I be the `in_mask` — the bits the transition consumes — and O be the `out_mask` — the bits the transition produces. The new marking is:

```
M' = (M ∧ ¬I) ∨ O
```

Read this in English: clear the bits the transition takes away (subtract the input places), then set the bits the transition produces (add the output places). In code, from `bitmask_replay.rs`:

```rust
marking = (marking & !t.in_mask) | t.out_mask;
```

This two-line formula is the complete semantic content of a transition firing. It appears three times in the codebase, each time in a different context but with the same mathematical structure. In `bitmask_replay.rs` it drives the trace replay loop. In `lib.rs` inside `apply_branchless_update`, it is the core of the branchless kernel:

```rust
(marking_mask & !input_mask) | output_mask
```

In `simd/swar.rs`, inside `SwarMarking::try_fire_branchless`, it appears again:

```rust
let next = (self.words[i] & !req[i]) | next_val;
```

This is not a coincidence of implementation; it is the same mathematical identity expressed at three levels of abstraction — single-net, branchless-kernel, and SWAR multi-word. The formula M' = (M ∧ ¬I) ∨ O is the irreducible core of what token-based replay *is*. Every optimization in the codebase is a refinement of the data representation around this identity, never a change to the identity itself.

---

## 4. Counting Without Branching: POPCNT as Silicon-Native Measurement

Replay is not just about updating the marking — it also needs to *measure* how well the trace fits the model. The fitness score requires counting missing tokens: tokens the transition needed but the current marking did not provide.

In a conventional implementation, this might be an explicit loop with an if/else per place:

```
for each input place p of transition t:
    if marking[p] == 0:
        missing_count += 1
```

The bitmask representation makes this dramatically more elegant. A token is missing from input place *p* if bit *p* is set in `in_mask` but not set in `marking`. The set of missing tokens is therefore:

```rust
let need = t.in_mask & !marking;
```

The *count* of missing tokens is then the number of set bits in `need`:

```rust
missing += need.count_ones();
```

`count_ones()` in Rust compiles to the POPCNT instruction — a single CPU instruction that counts set bits in a 64-bit word in one cycle. There is no loop, no branch, no iteration over places. The hardware is doing the counting.

The `TransMask` struct pre-stores `in_popcount: u32` — the count of bits in `in_mask` — computed once during net construction. This means the total consumed token count for a transition is always available without recomputation. During replay, `consumed += t.in_popcount` adds the contribution of each fired transition to the denominator of the fitness formula.

This is measurement without decision-making: the silicon counts bits, not logic. The program never asks "is this place missing?" It computes a word representing all the missing places simultaneously, then asks the hardware "how many bits are set?" That is a fundamentally different interaction with the machine.

---

## 5. Timing Sidechannels and Branchless Logic

Consider what happens when the transition-firing code contains a branch: `if (marking & t.in_mask) != t.in_mask { handle_missing_token(); }`. The CPU's branch predictor begins to build a model of which path is taken more often. If token availability correlates with the business significance of the activity — as it does in process mining, where conformant activities are far more common — then the execution time of the branch-containing code leaks statistical information about the conformance of the trace.

This is not a theoretical concern. In high-assurance process compliance systems, the conformance checker is part of the audit trail. If a trace's measured replay time correlates with how many exceptions it triggered, a timing-based side channel exists. An adversary with access to replay timing could infer the conformance properties of traces they should not be able to examine.

The `select_u64` function in `src/utils/bitset.rs` addresses this:

```rust
pub const fn select_u64(cond: u64, true_val: u64, false_val: u64) -> u64 {
    (cond.wrapping_neg()) & true_val | (!cond.wrapping_neg()) & false_val
}
```

This is a branchless conditional move expressed in integer arithmetic. The trick is that `0u64.wrapping_neg()` is `0`, and `1u64.wrapping_neg()` is `u64::MAX` (all ones). When `cond` is 1 (fired), `cond.wrapping_neg()` is all ones, masking in `true_val` and masking out `false_val`. When `cond` is 0 (not fired), the opposite occurs. No branch is taken; both paths compute, but only one result is selected. The CPU cannot predict which value will matter.

`SwarMarking::try_fire_branchless` uses this directly:

```rust
next_words[i] = select_u64(cond, next, self.words[i]);
```

The word is either the fired-transition result or the unchanged marking, with no data-dependent branch. Execution time is constant regardless of whether the transition was enabled or not. This is not a performance optimization in the traditional sense — it is a correctness property for systems where timing must not reveal state.

---

## 6. KTier Design: Fitting State into Cache Lines

Not all processes fit in 64 places. The `KTier` enum in `src/lib.rs` (module `dteam::core`) expresses the graduated scale:

```rust
pub enum KTier {
    K64,    // 1 word  = 8 bytes
    K128,   // 2 words = 16 bytes
    K256,   // 4 words = 32 bytes
    K512,   // 8 words = 64 bytes
    K1024,  // 16 words = 128 bytes
}
```

The capacity of each tier is `words() * 64` places. The arithmetic is intentional.

K64 (one 64-bit word, 8 bytes) fits a net of up to 64 places entirely within a single CPU register. There is no memory access at all: the marking lives in a register, and firing a transition is two bitwise instructions.

K512 (eight words, 64 bytes) is the most architecturally significant tier. 64 bytes is exactly one L1 cache line on x86-64, ARM64, and virtually every modern architecture. A K512 marking fits entirely within a single cache line, meaning the processor can load the complete state of a 512-place net in a single cache transaction. No partial loads, no cache-line straddling, no false sharing under parallelism. This is not an accident — the tier boundary is *defined* by the cache line size.

K1024 (16 words, 128 bytes) spans two cache lines. At this scale, the processor may need two cache transactions to load the marking, but the data remains compact enough that both lines are likely in L1 if the replay is hot.

The `RlState<const WORDS: usize>` struct uses the same WORDS parameter to encode the marking. A `RlState<1>` is a K64 marking embedded in the RL state. A `RlState<8>` is K512. The generic parameter makes the tier selection visible at compile time, not hidden in a runtime branch.

The practical implication: for the PDC 2025 contest traces, most real-world process nets have well under 64 activities, meaning K64 is the operative tier. The entire replay of a trace is a sequence of register operations with no heap access and no cache misses.

---

## 7. Trade-offs and the PartitionRequired Principle

The bitmask representation has one non-negotiable prerequisite: the net must have at most `WORDS × 64` places. For `NetBitmask64`, this is 64. The constructor enforces this:

```rust
assert!(
    n_places <= 64,
    "NetBitmask64 requires ≤64 places, got {}",
    n_places
);
```

There is also the 1-safe assumption. The bitmask encoding represents token presence as a single bit. If a place can hold more than one token, the representation is lossy — it cannot distinguish "two tokens in p3" from "one token in p3." The standard replay path falls back to a general implementation (`replay_trace_standard`) when 1-safety cannot be assumed.

These constraints might seem like limitations. They are better understood as a surface that makes the constraint explicit. Consider the alternative: a dynamic representation that silently degrades to a slow path when the net grows large. The silent degradation makes performance non-predictable — the same code that runs in nanoseconds for a 60-place net silently switches to microseconds for a 65-place net, with no indication to the caller.

The explicit assertion and the fallback function make the constraint a first-class design decision. If a net exceeds 64 places, the caller knows it. They can choose to partition the net, use the standard replay, or redesign the net. The constraint surfaces the trade-off rather than hiding it.

The 1-safe assumption is similarly principled. Most workflow net models encountered in process mining — BPEL-derived, WF-net, α-algorithm output — are 1-safe by construction. The assumption is correct for the dominant use case. The fallback exists for edge cases without silently compromising the fast path.

---

## 8. The Fitness Formula: Connecting Performance to Meaning

All this engineering serves one purpose: producing a fitness score that correctly measures how well the event log fits the model. The `ReplayResult::fitness()` method:

```rust
pub fn fitness(&self) -> f64 {
    let total = self.consumed + self.missing;
    let denom = (total + self.produced) as f64;
    1.0 - (self.missing as f64 + self.remaining as f64) / denom
}
```

This is the standard token-based fitness formula from van der Aalst's process mining literature. The denominator `consumed + missing + produced` counts every token movement that *happened* or *should have happened*. The numerator `missing + remaining` counts the tokens that were artificially injected (missing) or left unconsumed (remaining) — the penalties for deviation from the model.

A perfect trace has `missing = 0` and `remaining = 0`, yielding fitness 1.0. `is_perfect()` checks exactly this at zero cost: `self.missing == 0 && self.remaining == 0`. Because `missing` and `remaining` are `u32` values accumulated by POPCNT during replay, this check is a single OR-and-comparison on two integers — no floating point, no division.

The fitness score connects to the RL reward in `automation.rs`: the outer training loop uses `token_replay_projected` to obtain the average fitness across the log, which feeds directly into the reward term `avg_f as f32`. The zero-heap replay is not isolated engineering — it is the measurement instrument for a larger optimization. When the RL agent runs ten thousand discovery epochs, zero-heap replay means those ten thousand measurements cost register operations, not allocator calls. The constraint that seemed like a limitation (64 places) turns out to be the enabler of the scale at which process discovery becomes practical.

The fitness formula is also the connection back to meaning: a score of 0.85 says that 85% of the token movements the model expected actually happened without artificial intervention. The bit manipulation and POPCNT arithmetic are not abstract optimization tricks — they are the fastest path from an observed event sequence to a number that means something about process conformance.
