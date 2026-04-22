# 51 — Why 64³ Memory + 64³ Scratchpad Layout in the Core is Critical

## The missing piece

Docs 25, 40, and 42 mentioned the TruthBlock/Scratchpad pair. None of
them explained **why this specific pair, at this specific size, laid
out in this specific position inside the core's private cache, is
load-bearing.**

If the 64³ pair is misplaced, misaligned, or mis-sized by a single
factor, the entire architecture degrades from "branchless admission at
L1D latency" to "just another pipeline." This document says what
breaks and why.

---

## The number

```
64³ = 262,144 bits = 32 KiB = 4,096 u64 words
pair = truth + scratch = 64 KiB total
M3 Max P-core L1D = 128 KiB
→ pair occupies exactly 1/2 of L1D
```

The remaining half of L1D holds the working set: field masks, delta
ring, receipt ring, per-cycle instruction window, stack frames for
the hot kernel. **The pair and the working set are co-resident by
design, not by luck.**

---

## What "correctly laid out" means

Five constraints, in order of cost-of-violation:

1. **Contiguous** — truth and scratch are adjacent in virtual memory.
2. **Page-aligned** — base address is a multiple of 4,096.
3. **Cache-line aligned** — every sub-structure starts at a 64-byte
   boundary.
4. **Pinned** — the physical page never moves; `mlock` + `madvise`.
5. **Position-validated** — the virtual address is sealed into the
   L1 boot receipt; any drift invalidates every downstream fragment.

All five must hold. Dropping any one breaks a different property of
the architecture.

---

## Why contiguity matters: the branchless commit depends on it

The commit step is:

```rust
next = (candidate & admitted_mask) | (old & !admitted_mask);
```

Vectorized over 4,096 words, this is:

```rust
for i in 0..4096 {
    scratch[i] = (candidate[i] & mask) | (truth[i] & !mask);
}
```

If `truth` and `scratch` live in the same cache lines, **the loop
streams linearly** — one cache-line load carries words for both
regions. The prefetcher recognizes the stride and brings the next line
in on the same cycle.

If they live in different pages (say, scratch allocated separately via
`Box`), the loop issues **two independent streams**. The prefetcher
now has to split its budget. L1D hit rate drops. Instead of ~4,096
cycles for the commit, you pay ~8,000. The claimed `<5 µs` target
becomes `~10 µs`. The system has quietly halved.

**Contiguity is not a nicety. It is what makes the linear scan free.**

---

## Why page alignment matters: TLB pressure

The HotRegion is 64 KiB = sixteen 4 KiB pages. At 16 pages, the L1
DTLB (typically 64–96 entries on Apple Silicon) easily holds every
page with room to spare — but only if the base address is page-aligned
so each page gets exactly one DTLB entry.

Non-page-aligned:
```
page 0:  [......truth header......][truth bytes 0..3968 ]
page 1:  [truth bytes 3968..8064  ][...               ]
...
page 16: [...overflow into page 17 ]
```

Now the block spans 17 pages instead of 16. DTLB pressure climbs. On
context switch, the scheduler may evict one of your pages. Next access
is a TLB miss → page walk → 100+ cycle stall. The hot path's
deterministic latency is gone.

**Page alignment is the difference between "in the core" and "close
to the core."**

---

## Why cache-line alignment matters: one motion = one line, not two

Every sub-structure — `TruthBlock`, `Scratchpad`, `PackedEightField`,
`DeltaRing`, `ReceiptRing` — is `#[repr(C, align(64))]`. The reason is
**false sharing**.

Consider a `FieldMask` at offset 65,500 and a `DeltaRing::head` at
offset 65,528. Both fit in a single 64-byte cache line (65,472 to
65,535). If one core writes the mask and another reads the delta head,
the line bounces through the cache coherence protocol on every access.
What should be two independent operations becomes one serialized
operation.

`align(64)` on every field-guard sub-structure eliminates this. Every
hot cache line belongs to one logical quantity. No bouncing.

**Cache-line alignment is the only protection against phantom
serialization.**

---

## Why pinning matters: position is semantic

This is the subtle one. The physical address of the truth block
**matters to the receipt**. The L1 boot receipt is computed over
`(virtual_base, physical_base, layout_hash)`. If the OS remaps the
page between boot and first motion, the boot receipt no longer
matches — every downstream L0 fragment references an address that no
longer exists.

Without `mlock`:
- the OS may page the region out to swap
- the physical address may change on page-fault resolve
- the boot receipt becomes stale
- the receipt chain rejects every motion that cites it

**`mlock` is not performance tuning. It is what makes receipts
truthful over time.**

---

## Why 32 KiB exactly? The arithmetic

You can't pick an arbitrary size. The size has to simultaneously
satisfy:

```
fit in L1D                    ≤ 128 KiB / 2 (leave room for working set)
hold 64³ independent places   ≥ 262,144 bits
fold cleanly to tiered views  size = N × 8^k for multiple k
align to L2 sub-bank          typically 32 KiB granularity
fit DTLB coverage             ≤ 16 pages
be a round word count         4,096 words / 64 = fits SIMD cleanly
```

The one number that satisfies all six is **32 KiB**. Smaller loses
expressivity; larger spills out of L1D and defeats the entire design.

Nothing else fits the slot. The architecture is not free to pick a
different size; the L1D on the target silicon picks it.

---

## Why the truth/scratch dual specifically?

One region is not enough. Three is wasteful. Two is exactly right
because branchless commit requires **read-from-A, write-to-B, swap**:

```
cycle n:    admit reads truth, writes scratch
cycle n+1:  if admitted, swap truth ↔ scratch (O(1) pointer swap)
cycle n+1': if denied,  discard scratch (O(1) reset)
```

With one region you cannot roll back without another copy. With three
you waste half your L1D budget. Two is minimal and sufficient. The
dual is the substrate's expression of *reversibility* — it is how the
system can reject a motion at cost zero.

**The scratchpad is not a buffer. It is the physical expression of the
Pragmatic reversibility rule.**

---

## Why "in the core" and not "near the core"

L1D latency: ~4 cycles. L2 latency: ~15 cycles. L3: ~40 cycles. DRAM:
~200 cycles.

A fused admit-commit-emit is ~10 scalar ops per lane. If every word
access is 4 cycles, the admission runs at speed. If every word access
is 40 cycles, the same admission runs ten times slower — and the 14.87
ns Q-update baseline beats it comfortably. The whole architecture
loses its reason to exist.

This is the threshold:

```
L1D-resident:   admission beats the prior benchmark
L2-resident:    admission is competitive with the prior benchmark
L3 or worse:    admission is slower than the prior benchmark
```

**The 64³ pair is in L1D or the architecture fails.** Everything else
in the stack — typed ISA, POWL8/POWL64, CountZero, ReduceBuffer — is
scaffolding for keeping the pair in L1D. Lose that residence and none
of the scaffolding matters.

---

## What breaks if you get each wrong

| Violation | Consequence | Observable |
|---|---|---|
| pair not contiguous | prefetch can't stride | L1D miss counter climbs |
| pair not page-aligned | TLB pressure, eviction | DTLB miss counter climbs |
| sub-structure not line-aligned | false sharing | coherence traffic per admission |
| not pinned | swap, remap, eviction | receipts fail verification |
| position not validated | receipt chain still matches, but to wrong memory | Turing Police accepts forgeries |
| size != 32 KiB | spills L1D or underfills expressivity | benchmark misses tier budget |

Each row is a distinct failure mode with a distinct symptom. The
architecture's discipline is naming them all and refusing to ship if
any counter breaches.

---

## The layout, precisely

```
┌─────────────────────────────────────────┐  0       ──┐
│  TruthBlock    32 KiB   4,096 u64      │            │
│                                         │            │
├─────────────────────────────────────────┤ 32,768    │ 64 KiB
│  Scratchpad    32 KiB   4,096 u64      │            │  in L1D
│                                         │            │
├─────────────────────────────────────────┤ 65,536    │
│  PackedEightField   512 B   8 masks    │            │
├─────────────────────────────────────────┤ 66,048    │
│  DeltaRing    4 KiB   256 × Delta      │            │
├─────────────────────────────────────────┤ 70,144    │
│  ReceiptRing  4 KiB   256 × u128       │            │
├─────────────────────────────────────────┤ 74,240    │
│  BootReceipt  64 B                     │            │
├─────────────────────────────────────────┤ 74,304    │
│  Padding to 128 KiB                    │            │
└─────────────────────────────────────────┘ 131,072  ──┘
```

Every offset is a compile-time constant. Every offset is cache-line
aligned. Every offset is referenced by receipt fragments. **A single
offset shift is a UHDC version bump.**

---

## The critical path depends on all five at once

The happy path — one admit-commit-emit at tier 8² — touches:

```
1. Load u128 of truth from L1D              (must be pinned, must be warm)
2. Load FieldMask from L1D                  (must be cache-line aligned)
3. Compute deny bits                        (pure compute, no memory)
4. Branchless select to scratch             (must be contiguous with truth)
5. Append fragment to ReceiptRing           (must be in same HotRegion)
6. Decrement Watchdog counter               (must not cause coherence storm)
```

Every step is gated by a layout invariant. If any invariant fails, the
step stalls. If any step stalls, the whole pipeline stalls. If the
pipeline stalls, the 8-core reduce is waiting on a missing lane. If
the reduce is waiting, the MainOrchestrator's 228 M motions/sec target
collapses.

**A single layout violation propagates through seven layers of the
architecture.**

---

## Why the core must be the authority

The layout cannot be "suggested" or "recommended." It must be
**enforced in the physical substrate crate** (`unibit-phys`) so that
any upper layer using the `HotRegion` type inherits the guarantees
automatically:

```rust
#[repr(C, align(4096))]
pub struct HotRegion {
    pub truth:         TruthBlock,        // offset 0
    pub scratch:       Scratchpad,        // offset 32,768
    pub fields:        PackedEightField,  // offset 65,536
    pub delta:         DeltaRing,         // offset 66,048
    pub receipts:      ReceiptRing,       // offset 70,144
    pub boot_receipt:  [u8; 32],          // offset 74,240
    _pad:              [u8; 56_832],      // pad to 128 KiB
    _pin:              PhantomPinned,
}

const _: () = assert!(core::mem::size_of::<HotRegion>() == 131_072);
const _: () = assert!(core::mem::align_of::<HotRegion>() == 4_096);
```

The `const _: () = assert!(...)` block is the core's authority.
Compilation fails if the layout doesn't match. The architecture
literally cannot be built wrong.

---

## The philosophical part

The core is where the architecture becomes physical. Everywhere else
in the stack, "memory" is an abstraction. Inside the core, it is a
location with a latency and a coherence protocol. The 64³ pair is the
single point where abstract closure `A = μ(O*)` becomes **something
happening at a physical address with a measurable clock**.

Get the layout right and the closure has a body. Get it wrong and the
closure is still true — it is just a piece of math with no machine to
run it.

**The layout is the arithmetic of "this works on real silicon."**

---

## The sentence

**The 64³ truth/scratch pair must sit contiguous, page-aligned,
cache-line-aligned, pinned, and position-validated inside a single
core's L1D because contiguity gives the branchless commit linear
stride, page alignment gives DTLB residence, line alignment prevents
false sharing, pinning keeps the boot receipt truthful, and
position-validation seals the entire receipt chain against address
drift — and a single violation of any one of those five collapses
seven layers of the architecture above it from L1D-hit speed to
cache-miss speed, which is where the design stops outperforming the
baseline it was built to replace.**
