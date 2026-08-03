# 29 — Hyperdimensional POWL + MuStar + MotionPacket in Nightly Rust

## Crate: `hdwfn-nightly`

Nightly features used:
```
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(portable_simd)]
#![feature(strict_provenance_lints)]
```

## Module layout

```
tier.rs          — WorkTier enum, ResidenceTier, WORK_BITS<T>, WORK_WORDS<T>
hypervector.rs   — Hv<const T: WorkTier>
coordinate.rs    — DomainId, AttentionCell, LocalPlace, GlobeCell
hpowl.rs         — ActivityId, Activity<T>, HPowlKind, HPowl<T>
fields.rs        — FieldLane enum, FieldVector<LANE, T>, LaneResult
motion.rs        — MotionOp, ReceiptMode, MotionPacket<OP, T, RECEIPT>
compiler.rs      — MuStarCompiler
execute.rs       — EightLaneResult, evaluate_8lane, reduce_8
```

## WorkTier as const enum

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum WorkTier {
    U8,        // 8^1 = 8
    U64,       // 8^2 = 64
    U512,      // 8^3 = 512
    U4096,     // 8^4 = 4,096
    U32768,    // 8^5 = 32,768
    U262144,   // 8^6 = 262,144 = 64^3
}

pub const WORK_BITS<const T: WorkTier>: usize = T.bits();
pub const WORK_WORDS<const T: WorkTier>: usize = T.words();
```

## Fixed-tier hypervector

```rust
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct Hv<const T: WorkTier>
where [(); WORK_WORDS::<T>]:,
{
    words: [u64; WORK_WORDS::<T>],
}

impl<const T: WorkTier> Hv<T>
where [(); WORK_WORDS::<T>]:,
{
    pub const fn zeroed() -> Self { ... }
    pub fn seeded(seed: u64) -> Self { ... }

    #[inline(always)]
    pub fn bind(&self, other: &Self) -> Self {
        // XOR-based binding
    }

    #[inline(always)]
    pub fn permute<const ROT: usize>(&self) -> Self {
        // word-position permutation encoding POWL order
    }

    #[inline(always)]
    pub fn hamming(&self, other: &Self) -> u32 {
        // XOR + popcount
    }

    #[inline(always)]
    pub fn similar_mask(&self, other: &Self, threshold: u32) -> u64 {
        let d = self.hamming(other);
        ((d <= threshold) as u64).wrapping_neg()
    }
}
```

## HPowl AST

```rust
pub enum HPowl<const T: WorkTier>
where [(); WORK_WORDS::<T>]:,
{
    Activity(Activity<T>),
    Sequence { left: Activity<T>, right: Activity<T>,
               trajectory: Hv<T> },
    Parallel { left: Activity<T>, right: Activity<T>,
               bundle: Hv<T> },
    Choice   { left: Activity<T>, right: Activity<T>,
               choice: Hv<T> },
}

impl<const T: WorkTier> HPowl<T>
where [(); WORK_WORDS::<T>]:,
{
    pub fn sequence(left: Activity<T>, right: Activity<T>) -> Self {
        let a = left.hv.permute::<0>();
        let b = right.hv.permute::<1>();
        Self::Sequence { left, right, trajectory: a.bundle_xor(&b) }
    }
    pub fn parallel(...) -> Self { ... }
    pub fn choice(...)   -> Self { ... }
}
```

## MotionPacket with type-level constraints

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum MotionOp { ValidateTrajectory, Similarity, Repair, Commit }

#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum ReceiptMode { None, Fragment, Chain }

#[repr(C, align(64))]
pub struct MotionPacket<
    const OP: MotionOp,
    const T: WorkTier,
    const RECEIPT: ReceiptMode,
>
where [(); WORK_WORDS::<T>]:,
{
    pub scope: AttentionScope,
    pub target: OperationalTarget,
    pub observed: Hv<T>,
    pub prerequisite:  FieldVector<{ FieldLane::Prerequisite }, T>,
    pub law:           FieldVector<{ FieldLane::Law }, T>,
    pub capability:    FieldVector<{ FieldLane::Capability }, T>,
    pub scenario:      FieldVector<{ FieldLane::Scenario }, T>,
    pub risk_reward:   FieldVector<{ FieldLane::RiskReward }, T>,
    pub causality:     FieldVector<{ FieldLane::Causality }, T>,
    pub conformance:   FieldVector<{ FieldLane::Conformance }, T>,
    pub attention:     FieldVector<{ FieldLane::Attention }, T>,
    pub consume: Hv<T>,
    pub produce: Hv<T>,
    pub proof: ProofObligation,
}
```

## MuStar compiler

```rust
pub struct MuStarCompiler;

impl MuStarCompiler {
    pub fn compile<const T: WorkTier>(
        model: &HPowl<T>,
    ) -> MotionPacket<
        { MotionOp::ValidateTrajectory },
        T,
        { ReceiptMode::Fragment },
    >
    where [(); WORK_WORDS::<T>]:,
    {
        let observed = model.model_hv();
        // Build field vectors, scope, proof obligation...
        MotionPacket { ... }
    }
}
```

## Eight-lane evaluation

```rust
pub struct EightLaneResult {
    pub deny_total: u64,
    pub distance_vector: [u32; 8],
    pub fragments: [u64; 8],
}

impl EightLaneResult {
    pub const fn admitted(&self) -> bool {
        self.deny_total == 0
    }
}

#[inline(always)]
pub fn evaluate_8lane<OP, T, RECEIPT>(
    packet: &MotionPacket<OP, T, RECEIPT>,
) -> EightLaneResult
where [(); WORK_WORDS::<T>]:,
{
    // Evaluate each field (sequentially in reference impl; production
    // maps to synchronized cores for large tiers)
    // Reduce via branchless OR
}

#[inline(always)]
pub fn reduce_8(results: &[LaneResult; 8]) -> u64 {
    results[0].deny | results[1].deny | results[2].deny | results[3].deny
        | results[4].deny | results[5].deny | results[6].deny | results[7].deny
}
```

## Usage example

```rust
let receive  = Activity::<{ WorkTier::U4096 }>::seeded(ActivityId(1), 0x1001)
    .at(GlobeCell::new(1, 42, 1));
let validate = Activity::<{ WorkTier::U4096 }>::seeded(ActivityId(2), 0x1002)
    .at(GlobeCell::new(1, 42, 2));

let seq = HPowl::<{ WorkTier::U4096 }>::sequence(receive, validate);
let packet = MuStarCompiler::compile::<{ WorkTier::U4096 }>(&seq);
let result = evaluate_8lane(&packet);
```

## Key design properties

- `HyperVector<const T: WorkTier>` — tier is part of the type. No arbitrary vectors, no dynamic tier selection inside hot path.
- `MotionPacket<const OP, const T, const RECEIPT>` — op, tier, receipt mode all compile-time.
- `FieldVector<const LANE: FieldLane, const T: WorkTier>` — lane is part of the type.

## The two most important types

```rust
pub struct HyperVector<const T: WorkTier>
where [(); WORK_WORDS::<T>]:,
{
    words: [u64; WORK_WORDS::<T>],
}
```

That one line makes HDC lawful.

```rust
pub struct MotionPacket<
    const OP: MotionOp,
    const T: WorkTier,
    const RECEIPT: ReceiptMode,
>
```

That makes MuStar's output typed, tiered, and admissible.

## The final move

```rust
pub struct EightLaneResult {
    pub deny_total: u64,
    pub distance_vector: [u32; 8],
    pub fragments: [u64; 8],
}
```

Hyperdimensional POWL becoming executable geometry.
