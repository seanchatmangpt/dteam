# 55 — Code Audit: ~/unibit and ~/chatmangpt/bcinr Against Our Five Criteria

## The five criteria (from doc 54)

```
pinned, branchless, typed, receipted, narrow.
```

This document grades the two candidate substrates against those five
and names which files to keep, which to promote, and which to backfill.

---

## Scoreboard

| Criterion | `~/unibit` | `~/chatmangpt/bcinr` |
|---|---|---|
| **pinned** | strong | absent |
| **branchless** | very strong | strong |
| **typed** | moderate–strong | absent |
| **receipted** | strong (BLAKE3) | moderate (FNV-1a only) |
| **narrow** | moderate | strong |

**unibit: 4.5 / 5 — closest to manifesto; broad foundations present.**
**bcinr: 2.5 / 5 — strong branchless + narrow; missing the crypto,
type-level, and pinning infrastructure.**

---

## ~/unibit — what meets the criteria

### PINNED (strong)

```
crates/unibit-smoke/src/main.rs:20
  let pinned: Pin<Box<L1Region>> = Box::pin(L1Region::default());

crates/unibit-l1/src/lib.rs:229
  const _: () = assert!(core::mem::size_of::<L1Region>() == 65536);

crates/unibit-l1/src/lib.rs:171
  pub fn attempt_mlock(r: &L1Region) -> bool   // mlock(2) wrapper

crates/unibit-hot/src/t0.rs:67
  #[repr(C, align(64))] on PackedEightField

crates/unibit-l1/src/lib.rs:60
  #[repr(C, align(64))] on TruthBlock (32 KiB)
```

**Hit rate:** `Pin<Box<L1Region>>` is already the canonical pattern.
Size-asserted to 65,536 bytes. mlock wrapper exists. `align(64)` on
every hot-path struct. Matches doc 51's layout criticality checklist.

### BRANCHLESS (very strong)

```
crates/unibit-hot/src/t0.rs:245
  ((hdc_distance_256(x, y) > threshold) as u64).wrapping_neg()

crates/unibit-hot/src/t0.rs:268
  let admitted = ((deny == 0) as u64).wrapping_neg();

crates/unibit-kernel/src/lib.rs:140
  0u64.wrapping_sub(flag as u64)   // bool_mask

crates/unibit-hdc/src/lib.rs:393-394
  popcount(state ^ proto_lo) + popcount(query_hi ^ proto_hi);
  deny = (dist > threshold).wrapping_neg()

crates/unibit-hot/src/t0.rs  (7 functions)
  #[inline(always)] on admit/commit/fragment paths
```

**Hit rate:** the exact pattern from doc 36 and doc 34. `wrapping_neg`
as mask construction. XOR + popcount as Hamming. `#[inline(always)]`
pervasive. This is the language of the manifesto, in place.

### TYPED (moderate–strong)

```
crates/unibit-nightly/src/lib.rs:2-3
  #![feature(generic_const_exprs)]
  #![feature(adt_const_params)]

crates/unibit-nightly/src/tiers.rs:11
  pub enum WorkTier { U8, U64, U512, ... }   // ConstParamTy

crates/unibit-nightly/src/tiers.rs:80
  pub const WORK_BITS<const T: WorkTier>: usize = T.bits();

crates/unibit-kernel/src/lib.rs:41
  Work<BITS>   // tier-typed word array
```

**Hit rate:** `unibit-nightly` already has the tier-as-const pattern.
The other crates lag — tier dispatch is implicit, not type-parameterized.
Doc 29's `Hv<const T: WorkTier>` exists in name; not every hot path is
monomorphized yet.

### RECEIPTED (strong)

```
crates/unibit-causality/src/lib.rs:67
  pub struct UCausalReceipt(pub blake3::Hash);

crates/unibit-causality/src/lib.rs:75
  pub fn genesis() -> Self { Self(blake3::hash(&[0u8; 32])) }

crates/unibit-kernel/src/lib.rs:29-30
  pub struct UReceipt   (rolling FNV-1a chain)
  pub fn verify_receipt_chain(...)

crates/unibit-hot/src/t0.rs:291
  pub const fn fragment_t0(...)   // receipt fragment ctor
```

**Hit rate:** BLAKE3 is already wired via `unibit-causality`. Both a
cryptographic chain (BLAKE3) and a fast rolling chain (FNV-1a) exist.
The `L0..L5` layering from doc 35 isn't explicit yet; each crate emits
its own receipt shape.

### NARROW (moderate)

```
10 crates with #![no_std]
crates/unibit-hot/src/lib.rs:27-34
  pub use { t0, t2 }   // two re-exports, minimal surface
```

**Hit rate:** `no_std` is pervasive. The public surface is small.
**What's missing:** no single `#[no_mangle]` C-ABI entry, no `cargo
deny` or `cargo vet` config at repo root.

---

## ~/unibit — top 5 hot-path files

1. **`crates/unibit-hot/src/t0.rs`** (631 lines) — PINNED + BRANCHLESS +
   RECEIPTED + TYPED; the canonical admit/commit/fragment file.
2. **`crates/unibit-kernel/src/lib.rs`** — pinned truth region,
   `bool_mask`, `verify_receipt_chain`, `Work<BITS>`.
3. **`crates/unibit-causality/src/lib.rs`** — BLAKE3 receipt chain.
4. **`crates/unibit-l1/src/lib.rs`** — `align(64)`, layout assertion,
   mlock wrapper.
5. **`crates/unibit-nightly/src/tiers.rs`** — `WorkTier` with
   `adt_const_params` + `generic_const_exprs`.

---

## ~/unibit — gaps to close

```
1. Single #[no_mangle] C-ABI entry           — add to a new `unios` crate
2. cargo deny / cargo vet                    — add deny.toml at root
3. Tier-parameterize every hot path          — propagate
                                                const T: WorkTier into
                                                unibit-hot/t0.rs types
4. L0..L5 receipt layering explicit          — rename rolling FNV-1a
                                                chain to L1 fragment
                                                emit; add L0 position,
                                                L5 seal layers
5. Federated Watchdog                        — add per-core shard
                                                array (doc 46)
```

These are backfills, not rewrites. The foundation passes.

---

## ~/chatmangpt/bcinr — what meets the criteria

### BRANCHLESS (strong)

```
unibit_math.rs:27-30
  #[inline(always)] fn isolate_lowest_set_mask(val: u64) -> u64 {
      val ^ val.wrapping_sub(1)
  }

unibit_math.rs:94-98
  fn sub_sat_u64(a: u64, b: u64) -> u64 {
      res & !0u64.wrapping_sub((a < b) as u64)
  }

crates/bcinr-logic/src/int.rs:21-22
  pub const fn popcount_u64(x: u64) -> u64 { x.count_ones() as u64 }

crates/bcinr-logic/src/bitset.rs:80-81
  count_ones() on XOR distance (Hamming)
```

**Hit rate:** the bit-level discipline is clean. `wrapping_sub` and
`wrapping_neg` used as mask constructors. `count_ones` for Hamming.
This is the vocabulary of doc 9.

### RECEIPTED (moderate)

```
crates/bcinr-logic/src/patterns/integrity_receipt.rs:24
  pub struct DeterministicSubstrateReceipt { current_hash: u64, steps: u64 }

crates/bcinr-logic/src/patterns/integrity_receipt.rs:36-37
  FNV_OFFSET = 0xcbf29ce484222325
  FNV_PRIME  = 0x100000001b3

crates/bcinr-logic/src/patterns/integrity_receipt.rs:43,52,66
  #[inline(always)] mix / record / finalize
```

**Hit rate:** rolling FNV-1a deterministic receipt. Clean code.
**What's missing:** no BLAKE3 or other cryptographic primitive. Suitable
for interior chaining (fast FNV-1a like unibit's `UReceipt`); not
suitable alone for L5 release seal.

### NARROW (strong)

```
crates/bcinr-logic/src/lib.rs:1
  #![no_std]

crates/bcinr-logic/src/mask.rs:1
  #![forbid(unsafe_code)]

bcinr-core/src/lib.rs:6-9
  #![deny(missing_docs)]
  #![deny(rustdoc::broken_intra_doc_links)]
  #![deny(unsafe_op_in_unsafe_fn)]
```

**Hit rate:** stronger than unibit on `forbid(unsafe_code)` and deny
lints. The no_std discipline is absolute. No public surface sprawl.

---

## ~/chatmangpt/bcinr — top 5 hot-path files

1. **`unibit_math.rs`** (138 lines) — branchless arithmetic primitives
   (6× `wrapping_*`, 6× `#[inline(always)]`, `const fn`).
2. **`crates/bcinr-logic/src/lib.rs`** — `no_std` + `Branchless`
   marker trait + modular patterns.
3. **`crates/bcinr-logic/src/patterns/integrity_receipt.rs`** —
   `DeterministicSubstrateReceipt` + FNV-1a chain.
4. **`crates/bcinr-core/src/lib.rs`** — `no_std` + three deny lints +
   pattern integration.
5. **`crates/bcinr-logic/src/bitset.rs`** — branchless Hamming distance.

---

## ~/chatmangpt/bcinr — gaps to close

```
1. PINNED (absent)        — no Pin<Box<_>>, no mlock, no align(4096),
                            no layout asserts. Needs a pinned-region
                            crate before it can host hot admission.

2. TYPED (absent)          — no generic_const_exprs, no adt_const_params,
                            no tier/lane enum dispatch. Needs const-
                            generic surface before it can monomorphize
                            any of unibit's admit/commit patterns.

3. RECEIPTED (partial)     — FNV-1a rolling chain present. BLAKE3 seal
                            absent; L0/L5 layering absent.

4. NO HOT MOTION PATH      — the code is bit-level infrastructure; it
                            has no admit_eight, no PackedEightField,
                            no TruthBlock-shaped region. Missing the
                            admission algebra itself.
```

---

## The combined read

### unibit is the host
Of the five criteria, unibit hits four and a half. It already has:

- `L1Region` of exactly 64 KiB, pinned with `Pin<Box<>>`, mlockable
- `#[repr(C, align(64))]` on every hot struct
- `wrapping_neg` admission-mask pattern in `t0.rs`
- BLAKE3 receipt chain in `unibit-causality`
- `WorkTier` enum as `ConstParamTy` with `adt_const_params`
- `no_std` on ten crates

**Map from docs 43–54 to unibit:**
```
HotRegion        → L1Region
TruthBlock       → TruthBlock  (already named canonically)
PackedEightField → PackedEightField  (already canonical, t0.rs)
Gate             → fragment_t0 admission  (reshape to FieldMask pair)
Snapshot         → UCausalReceipt + truth snapshot
Watchdog         → not yet present; add unibit-watchdog crate
LanePolicy       → not yet present; add unibit-lane crate
SpscRing<T,N>    → not yet present; add unibit-ring crate
```

**Recommended next step:** rather than start a new workspace, graft
the missing crates (`unibit-watchdog`, `unibit-lane`, `unibit-ring`,
`unibit-compile`, `unibit-verify`, `unios`) onto the existing
`~/unibit/crates/` tree. The manifesto-compliant skeleton is already
there.

### bcinr is the branchless library, not the hot path
bcinr is high-discipline bit-level math. It can contribute:

- branchless primitives (`isolate_lowest_set_mask`, `sub_sat_u64`,
  `log2_floor`) to a new `unibit-bits` crate
- `forbid(unsafe_code)` and `deny` lints as a template for every new
  hot-path crate
- the `Branchless` marker trait as a compile-time audit shape

But bcinr on its own does not host a 64³ pinned admission. It would
need to import unibit's `L1Region`, `PackedEightField`, and the
`WorkTier` dispatch before any of its branchless primitives can reach
a hot motion path.

**Recommended next step:** extract the bit-level math from bcinr into
`~/unibit/crates/unibit-bits/` under `#![no_std]` + `#![forbid(unsafe_code)]`.
Leave bcinr as a reference lab; promote its primitives.

---

## What the audit proves

Doc 54's five-word manifesto — *pinned, branchless, typed, receipted,
narrow* — is not aspirational. It is a description of code that
already exists across these two repositories. The exercise ahead is
**consolidation, not invention:**

```
pinned     ← unibit-l1, unibit-smoke (already present)
branchless ← unibit-hot/t0.rs + bcinr unibit_math.rs (merge)
typed      ← unibit-nightly tiers.rs (propagate outward)
receipted  ← unibit-causality (BLAKE3) + bcinr integrity_receipt.rs
             (promote FNV-1a to L1 fragment, BLAKE3 to L5 seal)
narrow     ← both repos' no_std + bcinr's forbid(unsafe_code)
             (apply bcinr's stricter lints to unibit)
```

The substrate the manifesto describes is ~60% shipped. The remaining
work is adding:
1. a single `#[no_mangle]` `unios` entry point
2. const-generic tier-parameterization of `t0.rs`
3. L0..L5 receipt layering
4. federated `Watchdog` for multi-core
5. `cargo deny` config at the unibit root

Five items. Each is additive, none breaks existing code.

---

## Go / No-Go

**unibit: GO.** Four and a half of five criteria are already met. The
architecture is consistent with the manifesto. The gaps are additive.

**bcinr: PROMOTE AND MERGE.** The branchless primitives are valuable.
The crate structure is not load-bearing. Move the good math into
unibit; leave the lab intact for reference.

**The manifesto is compilable. The compile target is ~/unibit.**

---

## The sentence

**Of the five manifesto words, unibit already embodies pinned,
branchless, typed, and receipted at strong levels, with narrow as a
moderate pass; bcinr embodies branchless and narrow strongly but
lacks pinning and typing entirely — so the next step is not to start
a new workspace but to graft five additive crates (watchdog, lane,
ring, compile, verify, unios) onto ~/unibit, import bcinr's bit-level
math and its `forbid(unsafe_code)` discipline, and apply the L0..L5
receipt layering from doc 35 — because the manifesto is not an
aspiration, it is a description of code that already exists across
these two repositories and needs only to be consolidated.**
