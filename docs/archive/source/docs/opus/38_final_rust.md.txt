# 38 — Final Rust: unibit-hot Skeleton

## Purpose

The canonical `unibit-hot` crate skeleton. Three tier files. Each is
buildable on nightly as-is. Each has a declared benchmark pass/fail
target.

## Workspace shape

```
unibit/
├── crates/
│   ├── unibit-phys/      L0 — pinned memory, alignment
│   ├── unibit-hot/       L1 — hot kernels
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── t0.rs     8^2 tier, 64-bit admission
│   │       ├── t1.rs     8^4 tier, 4,096-bit admission
│   │       └── t2.rs     8^6 tier, 262,144-bit admission
│   ├── unibit-isa/       L2 — typed instructions
│   └── mustar/           L3 — compiler
├── Cargo.toml
└── rust-toolchain.toml
```

## `rust-toolchain.toml`

```toml
[toolchain]
channel = "nightly-2026-04-01"
components = ["rust-src", "rustfmt", "clippy", "miri"]
```

## `Cargo.toml` (workspace root)

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.90"

[profile.release-hot]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
trim-paths = ["all"]
debug = false
overflow-checks = false
```

## `crates/unibit-hot/Cargo.toml`

```toml
[package]
name = "unibit-hot"
edition.workspace = true
rust-version.workspace = true

[features]
default = []
neon = []

[dependencies]
# intentionally empty — no_std, no deps
```

## `crates/unibit-hot/src/lib.rs`

```rust
#![no_std]
#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]

pub mod t0;
pub mod t1;
pub mod t2;

pub use t0::{FieldMask, PackedEightField, EightLaneResult};
```

## `crates/unibit-hot/src/t0.rs` — 8² tier (64-bit admission)

```rust
//! t0.rs — 8^2 tier hot admission.
//! Budget: < 10 ns per admit_eight() on M3 Max.

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct FieldMask {
    pub required:  u128,
    pub forbidden: u128,
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct PackedEightField {
    pub masks: [FieldMask; 8],
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct EightLaneResult {
    pub deny_total: u64,
    pub per_lane:   [u64; 8],
}

#[inline(always)]
pub const fn deny_bits(state: u128, mask: &FieldMask) -> u64 {
    let missing = (state & mask.required) ^ mask.required;
    let present =  state & mask.forbidden;
    ((missing | present) != 0) as u64
}

#[inline(always)]
pub fn admit_eight(state: u128, fields: &PackedEightField) -> EightLaneResult {
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

#[inline(always)]
pub fn commit_8(
    old: u128,
    candidate: u128,
    admitted: u64,
) -> u128 {
    let mask = (admitted == 0) as u128;
    let mask = mask.wrapping_neg();
    (candidate & mask) | (old & !mask)
}

#[inline(always)]
pub fn admit_commit_emit(
    state: u128,
    candidate: u128,
    fields: &PackedEightField,
) -> (u128, u64 /* receipt fragment */) {
    let res = admit_eight(state, fields);
    let next = commit_8(state, candidate, res.deny_total);
    let fragment = (res.deny_total ^ (next as u64))
                 ^ ((next >> 64) as u64);
    (next, fragment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_mask_admits_everything() {
        let f = PackedEightField { masks: [FieldMask { required: 0, forbidden: 0 }; 8] };
        let r = admit_eight(0xdeadbeef, &f);
        assert_eq!(r.deny_total, 0);
    }

    #[test]
    fn required_bit_missing_denies() {
        let mut f = PackedEightField { masks: [FieldMask { required: 0, forbidden: 0 }; 8] };
        f.masks[0].required = 0b1010;
        let r = admit_eight(0b0010, &f);
        assert_ne!(r.deny_total, 0);
    }

    #[test]
    fn forbidden_bit_present_denies() {
        let mut f = PackedEightField { masks: [FieldMask { required: 0, forbidden: 0 }; 8] };
        f.masks[0].forbidden = 0b0001;
        let r = admit_eight(0b0001, &f);
        assert_ne!(r.deny_total, 0);
    }
}
```

Benchmark target: `admit_eight` < 10 ns, `admit_commit_emit` < 15 ns.

## `crates/unibit-hot/src/t1.rs` — 8⁴ tier (4,096-bit admission)

```rust
//! t1.rs — 8^4 tier hot admission over a 64-word tile.
//! Budget: < 200 ns per admit_tile() on M3 Max.

use core::simd::{Simd, num::SimdUint};

const W: usize = 64;  // 4,096 bits / 64 = 64 u64 words

#[repr(C, align(64))]
pub struct Tile([u64; W]);

#[repr(C, align(64))]
pub struct TileMask {
    pub required:  Tile,
    pub forbidden: Tile,
}

#[repr(C, align(64))]
pub struct PackedEightTile {
    pub masks: [TileMask; 8],
}

#[inline(always)]
pub fn deny_tile(state: &Tile, mask: &TileMask) -> u64 {
    let mut acc: u64 = 0;
    let mut i = 0;
    while i < W {
        let miss = (state.0[i] & mask.required.0[i]) ^ mask.required.0[i];
        let pres =  state.0[i] & mask.forbidden.0[i];
        acc |= miss | pres;
        i += 1;
    }
    acc
}

#[inline(always)]
pub fn admit_tile(state: &Tile, fields: &PackedEightTile) -> u64 {
    let mut deny: u64 = 0;
    let mut i = 0;
    while i < 8 {
        deny |= deny_tile(state, &fields.masks[i]);
        i += 1;
    }
    deny
}

#[inline(always)]
pub fn commit_tile(old: &Tile, candidate: &Tile, admitted: u64, out: &mut Tile) {
    let mask_bit = (admitted == 0) as u64;
    let mask = mask_bit.wrapping_neg();
    let mut i = 0;
    while i < W {
        out.0[i] = (candidate.0[i] & mask) | (old.0[i] & !mask);
        i += 1;
    }
}
```

Benchmark target: `admit_tile` < 200 ns.

## `crates/unibit-hot/src/t2.rs` — 8⁶ tier (262,144-bit admission)

```rust
//! t2.rs — 8^6 tier hot admission over a 4,096-word TruthBlock.
//! Budget: < 5 microseconds per admit_block() on M3 Max.

const BW: usize = 4096;  // 262,144 bits / 64 = 4,096 u64 words

#[repr(C, align(64))]
pub struct TruthBlock(pub [u64; BW]);

#[repr(C, align(64))]
pub struct BlockMask {
    pub required:  TruthBlock,
    pub forbidden: TruthBlock,
}

#[inline(always)]
pub fn deny_block(state: &TruthBlock, mask: &BlockMask) -> u64 {
    let mut acc: u64 = 0;
    let mut i = 0;
    while i < BW {
        let miss = (state.0[i] & mask.required.0[i]) ^ mask.required.0[i];
        let pres =  state.0[i] & mask.forbidden.0[i];
        acc |= miss | pres;
        i += 1;
    }
    acc
}

#[inline(always)]
pub fn admit_block_fused(
    state: &TruthBlock,
    required: &TruthBlock,
    forbidden: &TruthBlock,
) -> u64 {
    let mut acc: u64 = 0;
    let mut i = 0;
    while i < BW {
        let miss = (state.0[i] & required.0[i]) ^ required.0[i];
        let pres =  state.0[i] & forbidden.0[i];
        acc |= miss | pres;
        i += 1;
    }
    acc
}

#[inline(always)]
pub fn commit_block(
    old: &TruthBlock,
    candidate: &TruthBlock,
    admitted: u64,
    out: &mut TruthBlock,
) {
    let mask_bit = (admitted == 0) as u64;
    let mask = mask_bit.wrapping_neg();
    let mut i = 0;
    while i < BW {
        out.0[i] = (candidate.0[i] & mask) | (old.0[i] & !mask);
        i += 1;
    }
}
```

Benchmark target: `admit_block_fused` < 5 µs (10,240 SIMD ops / 3 GHz
with some overhead).

## Benchmark harness target

```rust
// benches/hot_tiers.rs
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use unibit_hot::t0::*;

fn bench_admit_eight(c: &mut Criterion) {
    let fields = PackedEightField { masks: [FieldMask { required: 0, forbidden: 0 }; 8] };
    c.bench_function("admit_eight", |b| {
        b.iter(|| {
            let r = admit_eight(black_box(0xdeadbeef_u128), black_box(&fields));
            black_box(r)
        })
    });
}

criterion_group!(benches, bench_admit_eight);
criterion_main!(benches);
```

## Pass/fail matrix

| Function | Tier | Target | Source |
|---|---|---|---|
| `admit_eight` | 8² | < 10 ns | t0.rs |
| `admit_commit_emit` | 8² | < 15 ns | t0.rs |
| `admit_tile` | 8⁴ | < 200 ns | t1.rs |
| `commit_tile` | 8⁴ | < 200 ns | t1.rs |
| `admit_block_fused` | 8⁶ | < 5 µs | t2.rs |
| `commit_block` | 8⁶ | < 10 µs | t2.rs |

All branchless. All `#[inline(always)]`. All `#[no_std]`. Zero
allocations. Zero panics in release. Zero heap. Zero globals.

## The one permitted dependency

`core::simd`. That's it. No crates.

## The sentence

**Three files, three tiers, three budgets — t0 for 64-bit admission
under 10 ns, t1 for 4,096-bit admission under 200 ns, t2 for the full
262,144-bit TruthBlock under 5 µs; if any of these miss, the design is
wrong, not the implementation.**
