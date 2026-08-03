# 32 — Minimum CPU Instruction Floor by Tier

## The question

Ignoring wall-clock, what is the minimum CPU instruction shape for a lawful
motion over the max independent-place universe?

## The layers

Two very different layers:

```
1. Semantic/discovery layer:
   parse logs, discover/compile POWL, build masks, build receipts.

2. Kinetic execution layer:
   evaluate compiled places/transitions over a 64³ TruthBlock.
```

Minimum instruction count only becomes interesting *after* MuStar has
compiled the challenge into a Motion Packet.

## Full universe

```
64³ = 262,144 independent places
stored as 1 bit per place:
262,144 bits = 32 KiB = 4,096 u64 words
W = 4,096 words
```

## Case A — only yes/no admission

MuStar pre-fuses all requirements into one mask:
```
required = prereq | law | capability | scenario | conformance | causality | ...
```

Hot loop per word:
```
deny |= ((state & required) ^ required);
```

Per u64 word, scalar shape:
```
load state
load required
AND
XOR
OR into deny accumulator
→ ~5 instructions / word
```

Full 64³:
```
5 × 4,096 = 20,480 scalar instructions
```

With 128-bit SIMD (two u64 per op):
```
≈ 5 instructions / 2 words
≈ 10,240 SIMD-vector instructions
```

This is the minimum yes/no admission scan over 262,144 independent places.

## Case B — eight-field explainable admission

Can't fully fuse if we need field-specific denial/distance/proof fragments.
Each lane:
```
deny_i |= ((state & field_mask_i) ^ field_mask_i);
```

Per field over full 64³:
```
scalar: 5 × 4,096 = 20,480 instructions per lane
SIMD:   ≈10,240 vector instructions per lane
```

Across 8 lanes:
```
scalar aggregate: 8 × 20,480 = 163,840 instructions
SIMD aggregate:   8 × 10,240 = 81,920 vector instructions
```

If 8 lanes run in sync on 8 cores, critical-path depth is closer to one lane
plus reduction:
```
SIMD critical path ≈ 10,240 + reduction
```

Aggregate chip does ~81,920 vector instructions, but synchronized decision
depth is roughly one field scan.

## Commit cost

Per word, branchless commit with delta:
```
load old, load consume, load produce
BIC / AND-NOT
OR
mask select
XOR delta
store next, store delta
→ ~9 instructions / word
```

Full scalar commit:
```
9 × 4,096 = 36,864 instructions
```

SIMD commit:
```
≈ 9 instructions / 2 words
≈ 18,432 vector instructions
```

## Full 64³ instruction floor

**Yes/no fused admission + commit:**
```
Admission scan: ≈10,240 SIMD instructions
Commit/delta:   ≈18,432 SIMD instructions
Total:          ≈28,672 SIMD instructions

Scalar:
  Admission scan: 20,480
  Commit/delta:   36,864
  Total:          57,344 scalar instructions
```

**Eight-field explainable admission + commit (aggregate SIMD):**
```
8-field admission: ≈81,920 vector instructions
commit/delta:       ≈18,432 vector instructions
total aggregate:   ≈100,352 vector instructions

Synchronized critical path:
  one lane scan + reduction + commit
  ≈ 10,240 + tiny reduction + 18,432
  ≈ 28,700 vector instructions
```

## Instruction table by 8ⁿ tier

Let `W = bits / 64`. Approximate lower-bound shapes:
```
fused SIMD admission ≈ 2.5W instructions
8-field SIMD aggregate ≈ 20W instructions
SIMD commit + delta ≈ 4.5W instructions
```

| Tier | Bits | Words | Fused admission | 8-field aggregate | Commit/delta | Fused+commit |
|---|---|---|---|---|---|---|
| 8² | 64 | 1 | ~3 | ~20 | ~5 | ~8 |
| 8³ | 512 | 8 | ~20 | ~160 | ~36 | ~56 |
| 8⁴ | 4,096 | 64 | ~160 | ~1,280 | ~288 | ~448 |
| 8⁵ | 32,768 | 512 | ~1,280 | ~10,240 | ~2,304 | ~3,584 |
| 8⁶ = 64³ | 262,144 | 4,096 | ~10,240 | ~81,920 | ~18,432 | ~28,672 |

This table is the **8ⁿ discipline made physical**.

## The real lower bound

True theoretical minimum: must inspect enough bits to distinguish admitted
from denied. For full independent-place universe, machine must touch 4,096
words. Any honest full-universe evaluation has lower bound:

```
Ω(4,096 word inspections)
```

Everything else is instruction engineering.

If MuStar can restrict active scope, lower bound drops:
```
active scope = 64 words
→ full admission ~160 vector instructions fused
```

That's why 8ⁿ matters: **benchmark is won by proving most motions do not
require 8⁶. They terminate at 8³, 8⁴, or 8⁵.**

## Benchmark output format

Use instruction-based metrics, not just wall-clock:

```
instructions_per_motion =
  admission_instructions
  + commit_instructions
  + receipt_fragment_instructions
  + reduction_instructions
```

Report:
- active_words
- work_tier
- field_count
- instruction_floor
- measured_instruction_count
- ratio_to_floor

Example:
```json
{
  "challenge": "pdc_2025_case_001",
  "places_independent_capacity": 262144,
  "active_words": 64,
  "work_tier": "8^4",
  "field_count": 8,
  "instruction_floor": {
    "fused_admission_simd": 160,
    "eight_field_aggregate_simd": 1280,
    "commit_delta_simd": 288,
    "critical_path_simd": 448
  },
  "result": "admitted",
  "receipt": "fragment"
}
```

## The sentence

**Even under the dumbest encoding — one independent place equals one bit —
the full 64³ universe requires only 4,096 machine words. A fused yes/no
admission over all 262,144 places has a lower-bound shape of roughly 10,240
SIMD-vector instructions, and an admitted branchless commit with delta
emission brings the full motion to roughly 28,672 vector instructions. With
eight synchronized field lanes, aggregate work is richer, but critical path
remains close to one field scan plus commit.**
