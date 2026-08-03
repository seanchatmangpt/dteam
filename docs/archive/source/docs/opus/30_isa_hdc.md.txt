# 30 — UHDC ISA: Typed Instruction Set

## Principle

The ISA should make illegal work impossible to express without declaring
tier, geometry, field lane, and receipt mode.

```
semantic operation → typed instruction → declared 8^n tier
  → executable packet → receipt fragment
```

## Crate: `uhdc-isa-nightly`

```rust
#![no_std]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(portable_simd)]
```

## Work tier enum

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum WorkTier {
    U8, U64, U512, U4096, U32768, U262144,
}
```

## Receipt mode enum

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum ReceiptMode { None, Fragment, Chain }
```

## Field lanes enum

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum FieldLane {
    Prereq, Law, Capability, Scenario,
    RiskReward, Causality, Conformance, Attention,
}
```

## Operation codes

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum UOp {
    Bind,           // out = a XOR b
    Bundle,         // out = bundle(a, b)
    Permute,        // out = permute(a)
    Similarity,     // distance = hamming(a, b)
    FieldAdmit,     // deny = field_distance > threshold
    Commit,         // next = admitted ? candidate : old
    Repair,         // find nearest lawful prototype
    Receipt,        // emit bounded receipt fragment
}
```

## Instruction flags

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum InstrFlags { Hot, Planning, Projection }
```

## The UInstr type

```rust
#[repr(C, align(64))]
pub struct UInstr<
    const OP: UOp,
    const TIER: WorkTier,
    const FIELD: FieldLane,
    const RECEIPT: ReceiptMode,
    const FLAGS: InstrFlags,
>
where [(); WORK_WORDS::<TIER>]:,
{
    pub a: Hv<TIER>,
    pub b: Hv<TIER>,
    pub threshold: u32,
    pub instruction_id: u64,
    pub source_commitment: u64,
}
```

**An instruction cannot exist without declaring its operation, tier, field,
receipt mode, and phase.**

## Instruction result

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UResult {
    pub deny: u64,       // 0 = admitted
    pub distance: u32,   // Hamming distance or score
    pub fragment: u64,   // receipt fragment
}
```

## Execution functions

```rust
#[inline(always)]
pub fn exec_bind<const T, const FIELD, const RECEIPT>(
    instr: &UInstr<{ UOp::Bind }, T, FIELD, RECEIPT, { InstrFlags::Hot }>,
) -> Hv<T> { ... }

#[inline(always)]
pub fn exec_permute<const T, const FIELD, const RECEIPT, const ROT>(
    instr: &UInstr<{ UOp::Permute }, T, FIELD, RECEIPT, { InstrFlags::Hot }>,
) -> Hv<T> { ... }

#[inline(always)]
pub fn exec_similarity<const T, const FIELD, const RECEIPT>(
    instr: &UInstr<{ UOp::Similarity }, T, FIELD, RECEIPT, { InstrFlags::Hot }>,
) -> UResult { ... }

#[inline(always)]
pub fn exec_field_admit<const T, const FIELD, const RECEIPT>(
    instr: &UInstr<{ UOp::FieldAdmit }, T, FIELD, RECEIPT, { InstrFlags::Hot }>,
) -> UResult { ... }
```

## Eight-lane packet

```rust
#[repr(C, align(64))]
pub struct EightLanePacket<const T: WorkTier, const RECEIPT: ReceiptMode>
where [(); WORK_WORDS::<T>]:,
{
    pub prereq:      UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Prereq },      RECEIPT, { InstrFlags::Hot }>,
    pub law:         UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Law },         RECEIPT, { InstrFlags::Hot }>,
    pub capability:  UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Capability },  RECEIPT, { InstrFlags::Hot }>,
    pub scenario:    UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Scenario },    RECEIPT, { InstrFlags::Hot }>,
    pub risk_reward: UInstr<{ UOp::FieldAdmit }, T, { FieldLane::RiskReward },  RECEIPT, { InstrFlags::Hot }>,
    pub causality:   UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Causality },   RECEIPT, { InstrFlags::Hot }>,
    pub conformance: UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Conformance }, RECEIPT, { InstrFlags::Hot }>,
    pub attention:   UInstr<{ UOp::FieldAdmit }, T, { FieldLane::Attention },   RECEIPT, { InstrFlags::Hot }>,
}
```

## Example

```rust
let instr: UInstr<
    { UOp::FieldAdmit },
    { WorkTier::U4096 },
    { FieldLane::Law },
    { ReceiptMode::Fragment },
    { InstrFlags::Hot },
> = UInstr {
    a: Hv::<{ WorkTier::U4096 }>::seeded(0xA11CE),
    b: Hv::<{ WorkTier::U4096 }>::seeded(0xB0B),
    threshold: 512,
    instruction_id: 42,
    source_commitment: 0xDEAD_BEEF,
};

let result = exec_field_admit::<
    { WorkTier::U4096 },
    { FieldLane::Law },
    { ReceiptMode::Fragment },
>(&instr);
```

The type says:
- operation = field admission
- tier = 8⁴ / 4,096 bits
- field = law
- receipt = emit fragment
- phase = hot eligible

## ISA table

| Instruction | Tiered? | Hot? | Meaning |
|---|---|---|---|
| `BIND<T>` | yes | yes | role/value binding |
| `BUNDLE<T>` | yes | yes | context superposition |
| `PERMUTE<T, ROT>` | yes | yes | sequence / POWL order |
| `SIMILARITY<T>` | yes | yes | Hamming distance |
| `FIELD_ADMIT<T, FIELD>` | yes | yes | field-specific denial |
| `COMMIT<T>` | yes | yes | branchless state update |
| `REPAIR<T>` | yes | planning | nearest lawful prototype |
| `RECEIPT<T>` | yes | mixed | proof fragment |

## The design law

```
Every HDC instruction must declare:
  what it does
  how much work it is allowed to touch
  which lawfulness field it belongs to
  whether it emits proof
  whether it is hot-path eligible
```

In Rust:
```rust
UInstr<
    const OP: UOp,
    const TIER: WorkTier,
    const FIELD: FieldLane,
    const RECEIPT: ReceiptMode,
    const FLAGS: InstrFlags,
>
```

## The killer property

**The instruction set makes semantic overreach a type error.**
