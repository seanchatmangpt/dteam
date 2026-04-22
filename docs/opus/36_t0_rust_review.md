# 36 — t0.rs Review: By-Value PackedEightField and Required/Forbidden Split

## The starting sketch

An earlier draft proposed:

```rust
#[repr(C, align(64))]
pub struct PackedEightField {
    pub prereq:      Hv<U4096>,
    pub law:         Hv<U4096>,
    pub capability:  Hv<U4096>,
    pub scenario:    Hv<U4096>,
    pub risk_reward: Hv<U4096>,
    pub causality:   Hv<U4096>,
    pub conformance: Hv<U4096>,
    pub attention:   Hv<U4096>,
}

pub fn admit(fields: PackedEightField, state: &Hv<U4096>) -> u64 { ... }
```

Two problems.

## Problem 1 — by-value passing of a 4 KiB struct

`PackedEightField` is `8 × 4,096 bits = 32,768 bits = 4,096 bytes = 4 KiB`.

Passing it by value:
```rust
pub fn admit(fields: PackedEightField, ...)
```

forces a 4 KiB memcpy onto every admission. At ~32 B/cycle that is ~128
cycles just for the move. Completely defeats the hot-path budget.

### Fix — by-reference

```rust
pub fn admit(fields: &PackedEightField, state: &Hv<U4096>) -> u64 { ... }
```

Zero copy. Eight pointer dereferences. All cache-line aligned.

### Better — by folded signature

Because we don't scan the full 4 KiB in the hot path (see doc 33), the
admission actually takes folded 128-bit signatures:

```rust
#[repr(C, align(64))]
pub struct PackedEightSig {
    pub prereq:      HdcSig128,
    pub law:         HdcSig128,
    pub capability:  HdcSig128,
    pub scenario:    HdcSig128,
    pub risk_reward: HdcSig128,
    pub causality:   HdcSig128,
    pub conformance: HdcSig128,
    pub attention:   HdcSig128,
}
// size: 8 * 16 = 128 bytes = exactly 2 cache lines
```

By value now costs ~4 cycles. `PackedEightSig` is sized to pass through
NEON registers with room to spare.

## Problem 2 — distance threshold instead of required/forbidden split

The draft computed:
```rust
let d = hamming(state, field);
let deny = (d > threshold) as u64;
```

This is semantically "does the observed state look similar enough to the
field prototype?" It works but it is coarse and requires a tunable
threshold per field.

The sharper admission algebra is *two masks per field*:

```rust
#[repr(C, align(64))]
pub struct FieldMask {
    pub required:  u128,   // bits that must be present
    pub forbidden: u128,   // bits that must NOT be present
}
```

Admission becomes pure bitmask algebra:

```rust
#[inline(always)]
pub fn deny_bits(state: u128, mask: &FieldMask) -> u64 {
    let missing  = (state & mask.required) ^ mask.required;
    let present  =  state & mask.forbidden;
    (missing | present) as u64
}
```

Four ops. No threshold. No Hamming. No branching. And it is *exact*
rather than *similar-enough* — either the required bits are present and
the forbidden bits are absent, or they are not.

## Combined fix — the t0.rs hot admission

```rust
#[repr(C, align(64))]
pub struct PackedEightField {
    pub masks: [FieldMask; 8],
}
// size: 8 * 32 = 256 bytes = 4 cache lines

#[inline(always)]
#[target_feature(enable = "neon")]
pub unsafe fn admit_eight(
    state: u128,
    fields: &PackedEightField,
) -> EightLaneResult {
    let mut deny = [0u64; 8];
    let mut i = 0;
    while i < 8 {
        deny[i] = deny_bits(state, &fields.masks[i]);
        i += 1;
    }
    EightLaneResult {
        deny_total: deny[0] | deny[1] | deny[2] | deny[3]
                  | deny[4] | deny[5] | deny[6] | deny[7],
        per_lane: deny,
    }
}
```

Eight lanes × four ops = ~32 scalar ops, fully branchless. On NEON with
two u128 per vector register, ~16 vector ops. At 3 GHz that is ~5–10 ns.

**Under the 14.87 ns Q-update baseline.**

## Why required/forbidden is the right duality

```
required:  "if the state claims X, X must include ...".
forbidden: "if the state claims X, X must NOT include ...".
```

This is literally how lawfulness is expressed in process mining:

- *prerequisite*: required bits must be set
- *law*: forbidden bits must be cleared
- *capability*: required capability bits must be present
- *scenario*: forbidden scenario bits must be absent
- *risk_reward*: required reward bits / forbidden risk bits
- *causality*: required predecessor bits
- *conformance*: forbidden non-model bits
- *attention*: required focus bits

The admission algebra is the law itself, rendered as bits.

## Distance is not dead

Hamming distance is still needed for:

- **REPAIR**: find the prototype nearest to a denied state
- **explainable denial**: "you were 17 bits away from lawful"
- **escalation**: move from 8⁴ to 8⁶ only when the distance margin is
  tight

Hamming runs on the cold path. Required/forbidden runs on the hot path.

## The t0.rs skeleton

```rust
// crates/unibit-hot/src/t0.rs

#![no_std]

use core::arch::aarch64::*;

#[repr(C, align(64))]
pub struct FieldMask { pub required: u128, pub forbidden: u128 }

#[repr(C, align(64))]
pub struct PackedEightField { pub masks: [FieldMask; 8] }

#[repr(C, align(64))]
pub struct EightLaneResult { pub deny_total: u64, pub per_lane: [u64; 8] }

#[inline(always)]
pub const fn deny_bits(state: u128, mask: &FieldMask) -> u64 {
    let missing = (state & mask.required) ^ mask.required;
    let present =  state & mask.forbidden;
    (missing | present) as u64
}

#[inline(always)]
#[target_feature(enable = "neon")]
pub unsafe fn admit_eight(
    state: u128,
    fields: &PackedEightField,
) -> EightLaneResult {
    let mut per_lane = [0u64; 8];
    let mut i = 0;
    while i < 8 {
        per_lane[i] = deny_bits(state, &fields.masks[i]);
        i += 1;
    }
    let deny_total = per_lane[0] | per_lane[1] | per_lane[2] | per_lane[3]
                   | per_lane[4] | per_lane[5] | per_lane[6] | per_lane[7];
    EightLaneResult { deny_total, per_lane }
}
```

That is the entire hot admission, in 50 lines, zero allocations, all
branchless, all typed at tier 8², natural fit for NEON.

## The sentence

**Pass by reference, fold signatures before the hot path, and replace
threshold-distance with required/forbidden masks — that single refactor
turns 340 ns of vector scan into under 10 ns of pure bitmask algebra.**
