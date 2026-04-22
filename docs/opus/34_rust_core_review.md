# 34 — Rust Core Review: What We Missed

## The review posture

Assume a hostile Rust core team read the plan. They ignore the semantics.
They ask exactly one question:

> Are you leaving cycles on the table?

## Techniques we named but did not wire

### 1. Branchless admission via SUBS/CSEL (aarch64) or CMOV (x86)

We wrote:
```rust
let deny = (d > threshold) as u64;
```

That compiles to a compare + setcc. It is already branchless. But we then
did:
```rust
if deny == 0 { commit } else { reject }
```

That branch collapses the benefit. Fix:
```rust
let admitted_mask = ((deny == 0) as u64).wrapping_neg();
next = (candidate & admitted_mask) | (old & !admitted_mask);
```

One select, no branch. We had the pattern — we did not apply it
everywhere.

### 2. Pre-computed popcount on compile

Hamming distance on hot path:
```rust
let d = popcount(a XOR b);
```

When `b` is the `field_mask`, `popcount(b)` is known at compile time.
The distance check:
```rust
(a XOR b).count_ones() > threshold
```

rewrites to:
```rust
(a & !b).count_ones() + (!a & b).count_ones() > threshold
```

If `b`'s popcount is stored, one half of the XOR-popcount already
collapses to subtraction of cached values. Half the work for admission
scan.

### 3. Superinstructions (MuStar fusion)

Current proposed hot path:
```
FIELD_ADMIT (8 lanes)
REDUCE
COMMIT
RECEIPT
```

Four dispatches. Each is register-local but each crosses an instruction
boundary.

Rust-core recommended fusion — write *one* superinstruction for the
most-common eight-lane admission + commit + fragment emit:

```rust
#[inline(always)]
pub fn admit_commit_emit_fragment<const T: WorkTier>(
    packet: &MotionPacket<...>,
    state: &mut Hv<T>,
) -> u64 /* fragment */
```

One function. One register file. One branchless select. One u64
fragment. No per-stage dispatch overhead.

This is the *single most important* micro-optimization we named but did
not write.

### 4. `#[target_feature(enable = "neon")]` + `portable_simd`

Portable SIMD is good for correctness, but on M3 Max NEON has specific
fused instructions (BIC, EOR, ORR, CNT) that the scalar autovectorizer
sometimes emits and sometimes does not. Annotate the hot loop:

```rust
#[target_feature(enable = "neon")]
#[inline(always)]
unsafe fn admit_hot<const W: usize>(state: &[u64; W], req: &[u64; W]) -> u64 {
    use core::simd::{Simd, SimdPartialEq};
    let mut deny = Simd::<u64, 2>::splat(0);
    for i in (0..W).step_by(2) {
        let s = Simd::<u64, 2>::from_slice(&state[i..]);
        let r = Simd::<u64, 2>::from_slice(&req[i..]);
        deny |= (s & r) ^ r;
    }
    deny.reduce_or()
}
```

The `target_feature` attribute removes the conditional emission.

### 5. `#[repr(C, align(64))]` — one cache line per field

We said `align(64)`. We did not enforce it for *every* hot-path struct.
Anything touched in the hot loop that is not cache-line aligned is a
false-sharing defect. Checklist:

```
Hv<T>            — aligned ✓
MotionPacket     — aligned ✓
FieldVector      — aligned ✓
EightLaneResult  — aligned (add)
UInstr           — aligned ✓
TruthBlock       — aligned (add)
Scratchpad       — aligned (add)
```

### 6. `strip = "symbols"` + `trim-paths`

In release profile, these are the difference between a 20 MB binary and
a 2 MB binary. Irrelevant to execution speed, highly relevant to
distribution and to the "complete black box" posture. We named but did
not configure:

```toml
[profile.release-hot]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
trim-paths = ["all"]
```

### 7. `#[inline(never)]` on cold paths

Aggressive `#[inline(always)]` on hot paths is correct. But the escalation
paths (`REPAIR`, full 8⁶ similarity, receipt-chain assembly) should be
`#[inline(never)]` so the icache stays clean for the hot path.

```rust
#[inline(always)] fn admit_8_2(...) { ... }
#[inline(always)] fn admit_8_3(...) { ... }
#[inline(always)] fn admit_8_4(...) { ... }

#[inline(never)] fn repair_8_6(...) { ... }
#[inline(never)] fn chain_receipt(...) { ... }
```

### 8. Forbidden-mask discipline (separation of deny semantics)

We wrote admission as *required-field distance*. We did not enforce the
dual:

```rust
deny = missing_required | forbidden_present
missing_required  = (state & required)  XOR required
forbidden_present = (state & forbidden)
```

One AND, one XOR, one AND, one OR — four ops. The prior proposed code
computed distance, compared to threshold, then branched. The separation
of *required* vs *forbidden* masks removes the threshold compare for the
majority of admission decisions — you only compute distance when
required/forbidden don't fully decide.

### 9. `naked_asm!` for the innermost loop — *if measured*

Not for aesthetics. Only if the autovectorizer fails to fuse.

```rust
#[naked]
unsafe extern "C" fn admit_64_words_neon() -> u64 { ... }
```

Reserve this for the one instruction that the benchmark proves is the
binding constraint.

### 10. `#[no_mangle]` on the single published hot entry point

For obfuscated-binary distribution, the single C-ABI entry point is
`universeos_motion_tick`. Everything else is static-linked and stripped.

## The real gap

The items above are not exotic — they are Rust-core table stakes. The
real gap between the design and the benchmark is:

```
we wrote the semantics
we did not write the fusion
we did not write the folded signatures
we did not measure
```

## The rule

```
measure first — autovectorizer output on the actual hot function —
before writing NEON intrinsics or inline assembly.
```

Every one of the ten items above should be gated on a profiler run.
Speculative optimization is how the eight-lane architecture slips back
under a HashMap lookup.

## The sentence

**The design is right; the fusion is missing — superinstruction, folded
signature, forbidden mask, and cache-line alignment are the four levers
that make eight-lane HDC admission land under the existing 14.87 ns
baseline instead of 340 ns above it.**
