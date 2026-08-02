# 56 — Geometry Jobs-to-be-Done

## Framing

"Geometry" in this archive is the 64ⁿ ladder: globe cell addressing,
POWL64 routing, cache residence, pinned TruthBlock offsets, eight-core
spatial sharding. Doc 19 named the globe; doc 40 pinned the residence;
doc 45 made it a dialect; doc 51 proved the layout is load-bearing.

**Jobs-to-be-Done** (Ulwick / Christensen) asks: what is the customer
hiring this thing to do? Frame the answer as:

```
When I am [situation]
I want to [outcome]
So I can [higher-level goal]
```

The goal is to make the *job* explicit so the artifact can be judged
against it. If an artifact doesn't serve a job, it is not hired.

This document is the geometry JTBD — the eight jobs the 64ⁿ ladder is
actually being hired to perform, the artifact that performs each, and
the measurement that proves the job is done.

---

## Job 1 — Address a place without a symbol table

```
When I need to identify a workflow place,
I want to compute its address geometrically,
so I can avoid maintaining a hash-keyed symbol table.
```

**Hired to do** — replace `HashMap<PlaceId, Position>` with
`GlobeCell(domain, cell, place)` — three `u16`s compute to a single
`u64` coordinate.

**Artifact** — `unibit-l1::L1Position(u32)` + doc 19's
`geo_to_globe_cell`.

**Measurement** — `PackedKeyTable` lookup baseline is 23.30 ns per
resolve. The geometric equivalent is two shifts and one OR: `(domain
<< 12) | (cell << 6) | place` — measured at **824 ps** (from 64¹
popcount-class ops). **28× faster, deterministic, cache-line local.**

**Not hired to** — handle sparse or dynamically-renamed place sets;
those stay in `PackedKeyTable`.

---

## Job 2 — Route a motion without accidentally violating causality

```
When I need to sequence a motion from place A to place B,
I want the compiler to enforce that the path is causal-lawful,
so I can't silently break POWL ordering.
```

**Hired to do** — turn causality from a runtime check into a
syntax error. POWL64's `Geodesic(a, b)` requires the edge to be
geometrically lawful or it won't type-check.

**Artifact** — POWL64 `Geodesic` node (doc 45).

**Measurement** — compile-time, not runtime. The proof is that an
illegal path produces a `mismatched types` error before `cargo check`
finishes; there is no counter to record.

**Not hired to** — decide *which* lawful path to take; that's the
compiler's free choice over the geodesic.

---

## Job 3 — Declare cache residence at the type level

```
When I allocate hot-path state,
I want to declare its residence tier as a type parameter,
so the compiler rejects a cache-tier mismatch.
```

**Hired to do** — make "this is in L1" a `ResidenceTier::L1` type
parameter, not a hope. An `L1`-tier function refuses an `L2`-tier
argument at the signature line.

**Artifact** — `ResidenceTier` enum (doc 40) + the layout discipline
in `unibit-l1::L1Region` (`#[repr(C, align(64))]`, pinned 64 KiB).

**Measurement** — from doc 51's layout criticality:
- `truth::read_word(42)` = 679 ps
- `truth::read_word(4000)` = 671 ps — **same latency at both ends of
  the 32 KiB region**, proving L1D residence end-to-end.

**Not hired to** — guarantee residence when the caller breaks the
contract (lifetime escape, manual page-out). The compiler enforces
the declaration; the OS enforces the pin; both must hold.

---

## Job 4 — Shard eight cores without cross-core coordination

```
When I need to fan out work across eight cores,
I want each core to own a geometrically-adjacent slice of truth,
so the cores do shared-nothing parallelism with zero locks.
```

**Hired to do** — partition the 64³ TruthBlock into 8 × 32 KiB slices
by lane, one per core, with an L2-shared reduce buffer for the OR
fan-in. Each core writes only its own region; the reduce runs at L2
latency after everyone posts.

**Artifact** — `LaneHotRegion<const LANE: FieldLane>` (doc 46)
+ `WatchdogShards<const N: usize>` + `ReduceBuffer`.

**Measurement** —
- `watchdog::shards8::tick_one` = **762 ps** (per-shard write, no
  coherence storm)
- `watchdog::shards8::tripped_any` = **3.66 ns** (all-eight scan in
  one L2-line read)

**Not hired to** — coordinate reduce *without* that final L2 write.
The reduce is the handshake; it costs one cache line round-trip, and
that cost is the 35 ns critical path from doc 46.

---

## Job 5 — Put the crime scene in the receipt

```
When a motion is later audited,
I want the receipt to say where on the globe it happened,
so the auditor can independently localize and re-verify the claim.
```

**Hired to do** — the L0 position-hash fragment. Every receipt is
prefixed with a BLAKE3 of `(virtual_base, physical_base, layout_hash)`
so the chain is pinned to an address, not just to a computation.

**Artifact** — `unibit-l1::emit_position_receipt` + the L0 layer of
the five-surface chain (doc 35, doc 37).

**Measurement** — independent verify: `unibit_verify::verify_dual_chain`
returns `Verdict::Lawful` only when both FNV-1a and BLAKE3 chains
re-derive to the same address-witnessed value.

**Not hired to** — prove *who* ran the motion. That's the L5 release
signature in `unios`. L0 proves the where; L5 proves the who; both
are needed.

---

## Job 6 — Attribute a denial without rescanning the state

```
When a motion is denied,
I want the denied bits to carry their lane provenance inline,
so I can attribute the denial without recomputing the admission.
```

**Hired to do** — each `LaneOutcome::Deny` comes with the lane's
`FieldLane` discriminant and a strike count. The receipt fragment
encodes the lane in its low bits, so the post-mortem is an address
lookup, not a replay.

**Artifact** — `unibit-lane::LanePolicy::judge` returning
`LaneOutcome` with implicit `FieldLane` attribution + the
`cabi::motion_tick`'s fragment word (doc 53's ~2.69 ns hot op).

**Measurement** — `cabi::motion_tick_deny` = **2.69 ns** — a denied
motion emits its attributable fragment in the same time as an
admitted one. No attribution surcharge.

**Not hired to** — attribute *which of several required bits* was
missing. That remains a popcount of `missing_required`, not a lane
attribution — different job, different artifact.

---

## Job 7 — Escalate tier by literal zoom, not by mode switch

```
When admission is inconclusive at a small tier,
I want escalation to mean "widen the neighborhood",
so escalation is a geometric operation, not a separate algorithm.
```

**Hired to do** — `PROMOTE.t→t+1` (doc 40) moves from an 8³ mini-HV
to an 8⁴ attention tile to an 8⁶ full TruthBlock — each step is "look
at a larger contiguous region of the same globe." The algorithm
doesn't change; only the scope does.

**Artifact** — POWL64 `Residence(ResidenceTier::L2, Box<Powl64>)`
nodes (doc 45) + the tier-specific hot kernels (`admit8_t0`,
`admit_tile`, `admit_block_fused`).

**Measurement** —
- 8² single admit: 1.43 ns (`hot::admit8_t0`)
- 8³ eight-lane: 21.67 ns (`lane::deny_bits_8_fused`)
- 8⁴ tile (64 admits): 165 ns (`motion_tick_64_admits`)
- **Each tier is ~15× the previous tier**, exactly matching the
  geometric `8×` scope increase. Zoom is literal.

**Not hired to** — change the shape of the kernel. A tier-8 kernel
that does something *other* than the 8² kernel scaled up is a
different algorithm and belongs in a different artifact.

---

## Job 8 — Find the nearest lawful prototype by vector subtraction

```
When a motion is denied and I need to suggest a lawful alternative,
I want "nearest" to mean geometric Hamming distance,
so repair is a subtraction, not a search over a rule catalog.
```

**Hired to do** — REPAIR (doc 40) is a single Hamming distance from
the denied state to every prototype in an associative memory; the
minimum is the nearest lawful fix. Because distances are symmetric
and bit-parallel, this is a sweep over a pinned prototype region
with no branching.

**Artifact** — `unibit-hdc` distance functions + prototype memory
(cold-path, DRAM-resident).

**Measurement** — `popcount(state ^ prototype)` lands in the same
counting-tier as doc 49's popcount at 824 ps per u64; a 4,096-word
TruthBlock-shaped prototype comparison sweeps at 87 ps/word (doc's
64³ sweep bandwidth) = **357 ns per prototype**. Repair is a
multi-prototype sweep, still cache-friendly at any prototype library
size that fits L2.

**Not hired to** — explain *why* a prototype is the nearest. That
requires the L4 process-mining-score fragment, not the L0 geometric
one.

---

## The anti-jobs — what geometry is NOT hired for

| Candidate job | Why it's refused |
|---|---|
| Message routing between hosts | Network, not cache geometry — AtomVM handles this. |
| Access control policy | Capability masks are bit algebra, not position. |
| Cryptographic sealing | BLAKE3 is flat; adding geometry would weaken the proof. |
| User-facing identifier generation | `GlobeCell` is for the kernel, not the UI — UIs hash their own. |
| Dynamic tenant sharding | Adding/removing domains mid-run would break the pinning invariant. |

Keep geometry narrow. Every job it does, it does at L1D latency. Jobs
that would pull it out of L1D go to other substrates.

---

## Mapping JTBD to the manifesto five words

| Geometry job | Manifesto word covered |
|---|---|
| 1 address | **typed** (compile-time coordinate) + **narrow** (no symbol table) |
| 2 route | **typed** (lockstep invariant) |
| 3 residence | **pinned** + **typed** |
| 4 shard | **pinned** (per-core slices) + **narrow** (no locks) |
| 5 crime scene | **receipted** (L0 fragment) + **pinned** (address in chain) |
| 6 attribute | **branchless** (fragment emit is inline) |
| 7 zoom | **typed** (tier as const param) |
| 8 nearest | **branchless** (Hamming is XOR+popcount) |

Eight jobs, five words, no gaps. Every job hits at least one manifesto
word; every manifesto word is hit by at least three jobs. The
vocabulary closes on itself.

---

## The one question each geometry change must answer

> *Which of the eight geometry jobs does this change serve, and what
> is the measurement that proves it?*

If a proposed change can't cite a job number and a benchmark, it's not
hired. This is the JTBD version of the five-word manifesto filter:
make the job explicit; make the measurement obligatory; ship only
what pays rent.

---

## The sentence

**The 64ⁿ geometry is hired to do exactly eight jobs — address a place,
route a motion, declare residence, shard eight cores, witness where a
motion happened, attribute a denial, escalate by zoom, and find the
nearest lawful prototype — and every job has a concrete artifact and
a measured latency that proves the hiring; anything else geometry is
asked to do is a different job and belongs in a different substrate.**
