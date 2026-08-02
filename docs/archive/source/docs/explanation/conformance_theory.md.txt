# Token Replay Theory

## Petri net marking

A Petri net consists of places P, transitions T, arcs A ⊆ (P × T) ∪ (T × P), a weight function W: A → ℕ₊, an initial marking M₀: P → ℕ, and a set of final markings F ⊆ {M: P → ℕ}.

A **marking** M assigns a non-negative integer token count to each place. The initial marking M₀ defines the starting state. A final marking (from the set F) defines an accepting state — the net has "run to completion" when a final marking is reached.

In dteam's `PetriNet`, places are `Vec<Place>` structs with string IDs. Markings are `PackedKeyTable<String, usize>` — a flat array mapping place IDs to token counts. The `initial_marking` field on `PetriNet` is this table.

## Token firing rule

A transition t is **enabled** in marking M if every input place p of t has at least W(p,t) tokens: for all (p, t) ∈ A, M(p) ≥ W(p, t).

When enabled transition t **fires**, it produces a new marking M': for each input place p, M'(p) = M(p) - W(p,t); for each output place p, M'(p) = M(p) + W(t,p). In matrix notation: M' = M + C·f, where C is the incidence matrix and f is the firing vector (1 at position t, 0 elsewhere).

The bitmask encoding makes this concrete for binary markings (one token per place, or zero). Each place is assigned a bit position: `place_bit[place_id] = 1u64 << i`. The in-mask for a transition is the OR of all input place bits: `in_mask = place_bit[p1] | place_bit[p2] | ...`. The out-mask is the OR of all output place bits.

The enabled condition becomes: `(marking & in_mask) == in_mask` — all input place bits are set.

The firing step is the branchless expression: `marking = (marking & !in_mask) | out_mask`. This atomically clears all input place bits and sets all output place bits. There are no branches, no loops over place lists, and no intermediate allocations. The transition happens in a single bitwise expression.

For invisible transitions (silent steps, labeled `$` or empty), the fixpoint loop `fire_invisible` repeatedly applies this expression until no more invisible transitions are enabled.

## Fitness formula

Two distinct fitness variants appear in the codebase, reflecting two different scoring philosophies.

**`bitmask_replay.rs` version** (`ReplayResult::fitness`):

```
fitness = 1.0 - (missing + remaining) / (consumed + missing + produced)
```

This is a global balance formula: it measures the total token deficit (missing) and surplus (remaining) as a fraction of total token activity (consumed + produced + missing). A trace with zero missing and zero remaining tokens scores 1.0.

**`case_centric/token_based_replay.rs` version** (`TokenBasedReplayResult::compute_fitness`):

```
fitness = 0.5 × (1 - missing / consumed) + 0.5 × (1 - remaining / produced)
```

This is a two-component formula:

- The first term, `1 - missing/consumed`, is a **recall** measure: what fraction of consumed tokens were actually available? Missing tokens indicate the trace required transitions the net could not enable — the trace deviates from the net's language.
- The second term, `1 - remaining/produced`, is a **precision** measure: what fraction of produced tokens were eventually consumed? Remaining tokens indicate the net produced tokens the trace never used — the net is over-permissive relative to the trace.

The 0.5/0.5 weighting treats recall and precision symmetrically. A trace that misses many tokens but leaves none remaining scores the same as a trace that leaves many tokens remaining but misses none. This symmetry is intentional: both kinds of deviation are conformance failures.

The `case_centric` path is the feature-gated `token-based-replay` implementation, used when detailed deviation accounting per event is needed. The `bitmask_replay` path is the fast path used in `token_replay_projected` during RL training loops.

## Why bitmask is exact

A common source of confusion: bitmask token replay is sometimes described as an "approximation" because it restricts markings to binary (one token per place). This is not an approximation for workflow nets — it is an exact representation of their semantics.

Workflow nets (WF-nets) have a structural property: under normal execution, each place holds at most one token at a time. This is the soundness condition. A sound WF-net with binary place occupancy can be exactly encoded as a `u64` bitmask: bit i is set iff place i holds a token.

The bitmask transition firing `(marking & !in_mask) | out_mask` is not a greedy approximation — it is the exact state equation M' = M + C·f, restricted to binary markings. There is no loss of information for nets that satisfy the one-token-per-place invariant.

The `in_language` function makes this explicit: it uses BFS over all reachable markings, not greedy firing. At each event, it computes the full set of markings reachable by firing any enabled transition with the required label, then takes the ε-closure (all markings reachable by invisible transitions). The trace is in the language if and only if at least one reachable marking after the final event contains the final marking bits. This is the exact language membership check — not an approximation.

## When to use the fallback

`NetBitmask64::from_petri_net` contains an explicit panic:

```rust
assert!(n_places <= 64, "NetBitmask64 requires ≤64 places, got {}", n_places);
```

For nets with more than 64 places, the `u64` bitmask cannot encode the marking (64 bits = 64 place bits maximum). The system falls back to `replay_trace_standard` with `PackedKeyTable` markings — the BCINR path, which uses the `bcinr` crate's bitset algebra to handle larger nets.

The fallback is also invoked when the net has transitions that produce multiple tokens per firing (non-binary arc weights), since the binary marking assumption breaks down. In practice, the PDC 2025 benchmark nets fit well within 64 places, making the K64 bitmask path the common execution path and the BCINR fallback a correctness safeguard rather than a frequent code path.

## The four deviation patterns

Token replay deviation analysis classifies trace-net mismatches into four cases based on the missing and remaining token counts after replay:

| missing | remaining | Pattern | Interpretation |
|---------|-----------|---------|----------------|
| > 0 | = 0 | **Under-produced** | The net's transitions could not be enabled for some events — the trace required activities the net does not permit in this order. The net's language is too restrictive. |
| = 0 | > 0 | **Over-produced** | The net produced tokens that the trace never consumed — the trace ended before reaching the final marking. The trace is a prefix of a conforming trace, or the net has dead-end places. |
| > 0 | > 0 | **Both** | The trace both required tokens the net couldn't provide and left tokens unconsumed. This is the most complex deviation: the net's routing diverges from the trace's activity sequence in both directions. |
| = 0 | = 0 | **Perfect** | The trace is exactly in the language of the net. `is_perfect()` returns true. Fitness = 1.0 for both formula variants. |

The missing-token count drives the first fitness term (recall): high missing means the net failed to enable transitions the trace needed. The remaining-token count drives the second fitness term (precision): high remaining means the net reached states the trace did not require. A well-discovered net minimizes both simultaneously on the training log — which is the optimization objective of the RL discovery loop.
