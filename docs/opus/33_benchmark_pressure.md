# 33 — Benchmark Pressure: Why HDC Must Be Folded, Not Full

## The existing DTEAM baselines

These are real, measured, single-core, single-thread numbers on the current
DTEAM substrate:

```
action selection        6.95 ns
Q-learning update      14.87 ns
SARSA update           17.53 ns
PackedKeyTable lookup  23.30 ns
double Q-learning      27.67 ns
```

These are the numbers any successor must *beat*, not match.

## The initial mistake

The first instinct when adding HDC was to say:

```
evaluate eight lawfulness fields
each field as a 4,096-bit hypervector
Hamming distance per field
branchless OR across lanes
```

That is the right architecture. It is not the right micro-benchmark.

```
4,096 bits = 64 u64 words
hamming distance = 64 × (XOR + popcount) = ~128 scalar ops
eight fields × 128 ops = 1,024 scalar ops
```

At ~3 GHz that is ~340 ns *just for admission*, before commit, before
receipt, before reduction. Slower than the thing it replaces.

The prior-generation DTEAM's Q-learning update is 14.87 ns. If HDC
admission costs 340 ns we have made the system 20× slower. That would be
a correct architecture with a wrong implementation — semantic victory,
kinetic defeat.

## The folded-signature repair

Full 4,096-bit vectors are semantic memory. **Hot-path admission must run
on folded signatures.**

```
full field vector:    Hv<U4096>    = 4,096 bits (memory, cold)
folded admission:     HdcSig128    = 128  bits (register-width, hot)
```

Build rule:
```
HdcSig128(field) = fold(Hv<U4096>(field))
```

where `fold` is a deterministic reduction — 64 × u64 XOR-chain or
majority-threshold collapse — computed once, when the `MotionPacket` is
compiled, and stored inside the packet.

Hot path becomes:
```
per lane:
  sig_observed = packet.observed_sig_128          // 2 u64
  sig_field    = packet.field[i].sig_128          // 2 u64
  d            = popcount(sig_observed XOR sig_field)
  deny_i       = (d > threshold) as u64
```

Per lane: ~4 scalar ops.
Eight lanes: ~32 scalar ops.
At 3 GHz: ~11 ns.

**That fits under the 14.87 ns Q-learning baseline. Barely.**

## Corrected per-tier targets

```
tier         action class                         wall-clock
8¹  (8b)     local flag decision                  < 2 ns
8²  (64b)    word-bind / atomic mask              < 10 ns
8³  (512b)   mini-HV admission (folded)           < 100 ns
8⁴  (4,096)  attention HV admission (folded)      < 200 ns
8⁵  (32,768) active tile                          < 500 ns
8⁶  (262,144) full operational depth              < 5 µs
```

Each tier is a 10× kinetic budget step. The system is **not** a single hot
path — it is a progressive admission ladder where most motions terminate
at 8² or 8³.

Benchmark metric that matters:

```
average admission tier × frequency
```

If the mean admitted motion terminates at 8² or 8³ with folded
signatures, the system beats the 14.87 ns Q-update baseline on average
even though the 8⁶ case takes microseconds.

## What "folded" does not cost

Folded signatures are produced at `MuStar` compile time, not in the hot
loop. The full 4,096-bit vector still exists — it just lives in cold
memory and is only accessed for:

- `REPAIR` (nearest-lawful lookup)
- `RECEIPT` when chain mode is on
- escalation from 8⁴ tier to 8⁶ tier on inconclusive admission

The hot path sees only 128-bit signatures. The semantics are preserved
because `fold` is deterministic and collision probability is bounded by
the signature width vs. active vocabulary size.

## The rule

```
semantic memory lives in 64^n
kinetic work runs on folded signatures
full vectors are touched only on escalation or receipt
```

That line is how eight-lane hyperdimensional admission stays under the
existing DTEAM nanosecond envelope.

## The sentence

**HDC supplies the semantics but folded signatures supply the clock —
the full 4,096-bit field vector is memory, the 128-bit fold is the
register, and every hot admission touches only the fold.**
