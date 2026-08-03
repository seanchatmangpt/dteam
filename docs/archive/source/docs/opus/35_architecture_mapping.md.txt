# 35 — Architecture Mapping: Benchmark Techniques → Layers

## Purpose

Map each optimization technique named in doc 34 to the exact
architectural layer where it lives. This is the "which crate owns this
cycle" chart.

## The layers

```
┌──────────────────────────────────────────────┐
│ L5 — unios        receipts, release, proof   │
├──────────────────────────────────────────────┤
│ L4 — dteam        discovery, conformance, RL │
├──────────────────────────────────────────────┤
│ L3 — mustar       semantic → motion compile  │
├──────────────────────────────────────────────┤
│ L2 — unibit-isa   typed instructions, tiers  │
├──────────────────────────────────────────────┤
│ L1 — unibit-hot   hot-path kernels, SIMD     │
├──────────────────────────────────────────────┤
│ L0 — unibit-phys  pinned memory, cache lines │
└──────────────────────────────────────────────┘
```

## Technique → Layer table

| # | Technique | Layer | Why there |
|---|---|---|---|
| 1 | branchless select via mask | L1 | one-off scalar fusion, owned by hot kernel |
| 2 | pre-computed popcount | L3 | cache values at MuStar compile time |
| 3 | superinstruction fusion | L1 + L2 | fused kernel + typed instruction entry |
| 4 | `target_feature(neon)` | L1 | platform-specific, isolate under cfg |
| 5 | `repr(C, align(64))` | L0 + L1 | physical invariant, enforced by types |
| 6 | `strip` + `trim-paths` | build profile | release posture, not a runtime layer |
| 7 | `inline(never)` cold paths | L1 | icache hygiene, hot kernel decision |
| 8 | forbidden-mask discipline | L2 + L3 | admission algebra, typed at ISA level |
| 9 | `naked_asm!` last-mile | L1 | only when measurement demands it |
| 10 | single `no_mangle` entry | L5 | distribution boundary, the one exported symbol |

## Vertical responsibilities

### L0 — `unibit-phys`
- `Pin<Box<L1Region>>` with `mlock`
- semantic position validation (`L1BootReceipt`)
- cache-line alignment enforcement
- no ops, only layout

### L1 — `unibit-hot`
- `admit_hot<W>`, `commit_hot<W>`, `reduce_or_8`
- NEON-feature-gated kernels
- branchless mask calculus
- fold / unfold primitives
- one single inlined superinstruction: `admit_commit_emit`

### L2 — `unibit-isa`
- `UInstr<OP, TIER, FIELD, RECEIPT, FLAGS>`
- `WorkTier`, `FieldLane`, `ReceiptMode`
- tier-width type math (`WORK_WORDS<T>`)
- forbidden/required mask duality at the type level
- compile-time rejection of tier mismatch

### L3 — `mustar`
- semantic → `MotionPacket` compiler
- folded signature generation (once, at compile)
- popcount caching
- progressive tier selection (8² → 8⁶)
- proof obligation emission

### L4 — `dteam`
- XES/OCEL ingestion
- POWL discovery
- conformance measurement
- RL training (Q / SARSA / DoubleQ / Expected SARSA / REINFORCE)
- `PackedKeyTable` + `fnv1a_64` everywhere

### L5 — `unios`
- black-box public surface
- receipt chains
- release gating
- the single `universeos_motion_tick` C-ABI entry

## Crossing rules

```
upward:   layer N may depend only on layer N-1 or below
downward: layer N may expose types to layer N+1 but not to N+2
```

This keeps:
- `unios` ignorant of `unibit-hot`'s SIMD shape
- `dteam` ignorant of `unibit-phys` pinning details
- `unibit-isa` ignorant of `mustar`'s compile strategy

## The ownership rule for each optimization

| Optimization | Owner crate | Verification |
|---|---|---|
| folded signatures | `mustar` (emit) + `unibit-hot` (consume) | signature round-trip test |
| superinstruction fusion | `unibit-hot` | benchmark vs sequential |
| branchless commit | `unibit-hot` | assembly inspection |
| tier dispatch | `unibit-isa` | compile-time monomorphization |
| cache alignment | `unibit-phys` | layout assertion in tests |
| no-mangle entry | `unios` | symbol visibility test |

## What crosses no boundary

The **receipt** crosses every boundary. It is the one object that every
layer can read and no layer can forge. Each layer contributes its
fragment:

```
L0: physical position hash
L1: hot-kernel output hash
L2: instruction-id + source-commitment
L3: compile commitment (motion-packet hash)
L4: process-mining conformance score
L5: release signature
```

A full receipt is all six concatenated and BLAKE3-sealed.

## The sentence

**Each optimization has exactly one owner crate; the layer chart is the
contract that keeps eight-lane HDC admission fast without any layer
reaching into the internals of another.**
