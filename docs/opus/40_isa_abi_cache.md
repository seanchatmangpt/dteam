# 40 — The ISA/ABI for 8ⁿ × 64ⁿ: Cache Layout, Action Motion, Scratchpad Pattern

## Pragmatic framing

> *You don't write programs, you write motions. The CPU doesn't hold data,
> it holds residents. The cache doesn't cache, it admits.*

This document is the instruction-set contract (what the compiler emits)
and the application-binary interface (what two crates agree on across a
function boundary). Both are tier-parameterized. Neither is optional.

---

## 1. Three levels of contract

```
ISA   — instructions the hot kernel executes
ABI   — how MotionPackets, TruthBlocks, and receipts cross a boundary
MLA   — Memory Layout Agreement: where bytes live across cache tiers
```

The Pragmatic rule: **changes to any one break the other two.** Treat
them as a single versioned artifact.

---

## 2. The cache ladder and the tier ladder

### Intel/Apple Silicon reality (M3 Max P-core)

```
register file     ~ 256 × 128-bit NEON        256-cycle reuse
L1D               128 KiB per P-core           ~4 cycles
L2 (shared pair)    16 MiB (cluster)          ~15 cycles
L3 / SLC            32–48 MiB                 ~40 cycles
DRAM (LPDDR5)      128 GiB                    ~200 cycles
```

### The 8ⁿ × 64ⁿ mapping

```
tier     bits         bytes    residence              hot latency
8¹           8           1      register               1 cycle
8²          64           8      register               1 cycle
8³         512          64      L1D (1 cache line)     ~4 cycles
8⁴       4,096         512      L1D (8 lines)          ~4 cycles
8⁵      32,768       4,096      L1D (64 lines)         ~4 cycles
8⁶     262,144      32,768      L1D (exact TruthBlock) ~4 cycles
8⁷   2,097,152     262,144      L2 residence           ~15 cycles
8⁸  16,777,216   2,097,152      L3/SLC                 ~40 cycles
```

**Identity: 8⁶ = 64³ = 32 KiB, which is exactly 1/4 of the 128 KiB L1D
on M3 Max.** The TruthBlock/Scratchpad pair (64 KiB) fits in half of
L1D, leaving the other half for the hot kernel's working set. This is
not a coincidence; it is the budget that defines the tier ladder.

---

## 3. ResidenceTier — the layout type

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, ConstParamTy)]
pub enum ResidenceTier {
    Reg,     // fits in NEON/x86 vector registers
    L1,      // resident in L1D of the executing core
    L2,      // resident in L2 cluster, core-local
    L3,      // resident in shared last-level cache
    Dram,    // spilled to main memory
    Nvm,     // persistent; cold path only
}
```

**ABI rule:** every struct that crosses a function boundary declares its
residence tier at the type level. A function that consumes `Reg`-tier
input cannot be called with `Dram`-tier input — it is a compile error,
not a performance regression.

```rust
pub fn admit_reg(fields: PackedEightField<{Reg}>) -> u64;
pub fn admit_l1(block: &TruthBlock<{L1}>) -> u64;
```

---

## 4. The ISA layout

### Opcode space — 8 bits, fixed

```
bits [7:5]   tier      WorkTier (8 values: 8^1 ... 8^8)
bits [4:2]   field     FieldLane (8 values)
bits [1:0]  class     {Admit, Commit, Reduce, Receipt}
```

Every instruction fits in one byte. Tier, field, and class are encoded
in the opcode itself, so the decoder does not need a tier table. This
is the **hard Pragmatic DRY rule**: the opcode is the tier is the
field.

### Micro-ops (what the hot kernel actually emits)

```
ADMIT.t.f.r      admit at tier t, field f, receipt mode r
COMMIT.t         branchless select at tier t
REDUCE.n         OR-reduce n lanes, n ∈ {2, 4, 8}
FRAG.t           emit receipt fragment
FOLD.t→t'        fold hypervector from tier t to tier t' (t > t')
PROMOTE.t→t+1    tier promotion (cold path)
DEMOTE.t→t-1    tier demotion (e.g. 8⁴ → 8²)
PIN              semantic-position validate (L0 only)
SEAL             BLAKE3 receipt close (L5 only)
```

### The fused superop

```
AEF.t  =  ADMIT.t.*.* || COMMIT.t || FRAG.t
```

One opcode, one inline function, one branchless select. This is the hot
path. Everything else is escalation.

---

## 5. The ABI

### Register-tier calling convention

For tier-8² and tier-8³ admission, the full call is register-resident:

```
x0    state pointer (or state value if 8²)
x1    PackedEightField pointer
x2    receipt-chain pointer (or null for ReceiptMode::None)
v0-v7 reserved for NEON fused admission
x0    return:  deny_total (u64)
v0    return:  receipt fragment (u128, if emitted)
```

No caller-save of v0–v7 across an `AEF.t` call. The ABI says they are
scratch within the fused superop and live on return.

### L1-tier calling convention (8⁴, 8⁵, 8⁶)

```
x0    &TruthBlock<{L1}>       — immutable state
x1    &PackedEightField<{L1}> — required/forbidden masks
x2    &mut Scratchpad<{L1}>   — destination for next-state
x3    &mut ReceiptFragment    — destination for fragment
x0    return:  deny_total (u64)
```

The contract: **the callee must not cause a cache miss.** If it does,
the measurement will tell us, and the ABI must either reshape or we
violated the tier promise.

### L2/L3 calling convention (8⁷, 8⁸)

Escalation. Uses the standard Rust ABI. No performance guarantee — these
are the "it's ok to take microseconds" paths.

---

## 6. Memory Layout Agreement (MLA)

### The pinned hot region

```rust
#[repr(C, align(4096))]   // page-aligned
pub struct HotRegion {
    pub truth:     TruthBlock,     // 32 KiB
    pub scratch:   Scratchpad,     // 32 KiB
    pub fields:    PackedEightTile, // 512 B
    pub receipts:  ReceiptRing,    // 4 KiB
    _pad:          [u8; 28_160],   // pad to 64 KiB + L1D working set
}

impl HotRegion {
    pub fn pin() -> Pin<Box<Self>> { ... }   // mlock + madvise(MADV_DONTNEED off)
}
```

**MLA rule 1:** Truth and Scratch are always at fixed offsets within
`HotRegion`. The compiler knows `truth` is at offset 0, `scratch` at
offset 32,768. This enables:

```rust
// Page offset from &HotRegion determines tier residence.
const TRUTH_OFFSET:   usize = 0;
const SCRATCH_OFFSET: usize = 32_768;
const FIELDS_OFFSET:  usize = 65_536;
```

**MLA rule 2:** Semantic position has meaning. A byte at offset 12,345
is not the same as a byte at offset 12,346 — the offset is part of the
admission calculus. This is why pinning cannot drift.

**MLA rule 3:** Cache-line alignment at `align(64)` for every struct;
page alignment at `align(4096)` for every `HotRegion`.

---

## 7. The scratchpad pattern

The scratchpad is **not** a buffer. It is the shadow of Truth.

```
Truth       = "what is"
Scratchpad  = "what might become, if admitted"
```

### The triple-step

```
1. COPY    scratch ← truth            (hot: memcpy 32 KiB = ~1 µs cold / ~10 µs L2)
2. MUTATE  apply candidate to scratch
3. ADMIT   check scratch against masks
4. COMMIT  branchless select: truth ← (scratch if admitted else truth)
```

### The optimization — delta journaling

Step 1 (full copy) is 32 KiB — expensive at 8⁶ scale. Instead, journal
the deltas:

```rust
#[repr(C, align(64))]
pub struct Delta {
    pub word_index:  u32,
    pub old_value:   u64,
    pub new_value:   u64,
}

#[repr(C, align(64))]
pub struct DeltaRing {
    pub entries:  [Delta; 256],
    pub head:     u32,
    pub tail:     u32,
}
```

If a motion touches ≤ 256 words, the scratchpad is the delta ring, not a
copy of Truth. On commit, apply the deltas; on reject, drop the ring.
On 8² and 8³ admissions, the ring is always shorter than a copy.

### The Pragmatic rule: "prefer deltas to copies"

```
motions < 256 words touched   → delta ring
motions 256–4,096 words       → tile-scratch (8⁴)
motions 4,096+ words          → full scratchpad (8⁶)
```

Tier dispatch at the ABI level: `COMMIT.t` has three variants keyed by
the size of the mutation, chosen by MuStar at compile time.

---

## 8. Action motion: bringing ops to data, not data to ops

The classical mistake: load Truth, operate, store Truth. That streams
32 KiB through registers every tick.

### The kinetic inversion

Truth stays pinned. **The instruction stream visits Truth in place.**
Each instruction is a folded 128-bit signature that admits or denies
relative to the resident TruthBlock.

```
old world:  data flows through a pipeline of instructions
new world:  instructions flow through a pinned data region
```

Two consequences:

1. **Instruction bandwidth** becomes the metric, not data bandwidth.
   Emitting 16-byte `UInstr` values at 10 GB/s is trivial; streaming
   32 KiB TruthBlocks at 10 GB/s is 300,000 admissions/sec, tops.

2. **The CPU is the router, not the cruncher.** The hot kernel moves a
   small instruction packet to the correct resident tile and fires. The
   L1D-resident TruthBlock is not touched until the fused superop
   dispatches.

### Prefetch discipline

```rust
#[inline(always)]
pub fn prefetch_next_tile(region: &HotRegion, next_offset: usize) {
    unsafe {
        core::arch::aarch64::__prefetch(
            region.as_ptr().add(next_offset) as *const i8,
            core::arch::aarch64::_PREFETCH_READ,
            core::arch::aarch64::_PREFETCH_LOCALITY3,
        );
    }
}
```

The Pragmatic DRY: **prefetch is an ABI responsibility, not a kernel
responsibility.** MuStar's codegen emits the prefetch in the caller's
frame, one tile ahead, so by the time the hot kernel is called the next
tile is already being fetched.

---

## 9. The bidirectional ABI invariant

Every ABI-boundary function obeys:

```
pre:   all inputs declared at their residence tier
post:  all outputs at the same or warmer tier
       no input ever promoted by the callee without an explicit PROMOTE
       receipt fragment always emitted if ReceiptMode != None
```

This is the *caller-sees-residence* contract. It means:

- A function declared to consume `L1` input can fail-fast if its inputs
  are cold (L2 or worse).
- A function declared to produce `Reg` output guarantees its return
  value is register-resident; the caller can chain it into the next
  instruction without a load.

---

## 10. Versioning

The (ISA, ABI, MLA) triple is versioned together:

```
UHDC-v1.0 = {
    opcodes:    0x00 .. 0xFF (as specified above),
    abi:        aarch64-neon + x86-64-avx2,
    mla:        HotRegion = 64 KiB pinned, align(4096),
    tier_map:   8^n → ResidenceTier as defined in §2,
}
```

A change to any field of any layout is a UHDC-v1.1, not a patch. The
priming corpus (doc 37) and the Rust skeleton (doc 38) are both stamped
with the UHDC version they target.

---

## 11. The Pragmatic Programmer's four checks

From the book, rephrased for 8ⁿ × 64ⁿ:

| Check | Applied here |
|---|---|
| **DRY** | Opcode = tier = field; one encoding, one table |
| **Orthogonality** | Tier, field, class are independent axes; any combo is legal |
| **Reversibility** | `DEMOTE.t→t-1` is always defined; no one-way promotions |
| **Tracer bullets** | t0.rs compiles and benches *before* t1.rs is written |

---

## 12. The sentence

**The ISA encodes tier in the opcode, the ABI encodes residence in the
type, and the MLA pins Truth + Scratch at fixed cache-line offsets
within a single 64 KiB page — so instructions travel to pinned data,
deltas replace copies below tier 8⁴, and the fused admit-commit-emit
superop is a single byte that names the tier, the field, and the class
it touches.**
