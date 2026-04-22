# 52 — Two Clocks, One Boundary: AtomVM as the Memory-Movement Firewall

## The question, sharpened

If we run AtomVM at the boundary, and the inside uses pinned 64³
TruthBlocks with branchless admission, what have we actually decided
about memory movement?

The answer is more radical than it looks: **we have decided that
memory does not move inside the core at all.** Every copy, every
marshal, every allocation, every garbage collection is pushed into
AtomVM. The core handles *state transitions*, not *memory management*.

These are two different disciplines. The boundary choice names which
discipline lives where.

---

## Two clocks

The architecture runs on two distinct time models, not one:

```
┌──────────────────────────────────────────────────────┐
│  Outside / AtomVM                                     │
│  ─────────────────────                                │
│  wall-clock time                                      │
│  GC pauses are real                                   │
│  OS preemption happens                                │
│  network jitter is real                               │
│  message passing, per-process heaps                   │
│  "soft real-time" (BEAM heritage)                     │
│  latency bound: best-effort                           │
└──────────────────────────────────────────────────────┘
                       ↕ AtomVM mailbox
┌──────────────────────────────────────────────────────┐
│  Inside / unibit-hot                                  │
│  ─────────────────────                                │
│  instruction-count time                               │
│  no GC, no allocator, no preemption                   │
│  no messages, no shared mutable state                 │
│  pinned memory, deterministic cycles                  │
│  "hard deterministic" (closure heritage)              │
│  latency bound: exact cycle count                     │
└──────────────────────────────────────────────────────┘
```

A benchmark of "10 ns per admission" is true on the inside clock and
meaningless on the outside clock, because the outside clock includes
GC pauses that have nothing to do with admission. A benchmark of
"end-to-end request 2 ms" is true on the outside clock and hides the
fact that the core admission cost 10 ns and AtomVM's mailbox
round-trip cost 1.999 ms.

Both are true. Neither is the whole truth. **AtomVM is the translator
between the two clocks.**

---

## What AtomVM is doing at the boundary

AtomVM inherits BEAM's model: per-process heaps, preemptive scheduling,
message passing, copy-on-send, per-process GC. All of this is **exactly
what we refuse inside.**

AtomVM sits at the boundary because its model matches the external
world's model:

- the outside world has processes, mailboxes, retries, timeouts
- the outside world has variable message sizes that must be allocated
- the outside world has crashes that must not corrupt other processes
- the outside world has wall-clock deadlines, not cycle budgets

AtomVM speaks all of that natively. It is the *impedance match*
between a chaotic external reality and an internally deterministic
admission engine.

---

## The boundary discipline

Every bit entering the core is marshalled by AtomVM first:

```
external message
   │
   ▼
AtomVM process: receive, parse, GC, allocate, copy into BEAM heap
   │
   ▼
AtomVM NIF: serialize BEAM term into a stack-allocated MotionPacket
   │
   ▼  (this is the memory-movement boundary)
   ▼
unibit-hot: step(&mut HotRegion, &MotionPacket)  ← zero allocations
   │
   ▼  (same boundary, outbound)
   ▼
AtomVM NIF: serialize Snapshot fragment into BEAM term
   │
   ▼
AtomVM process: forward to caller, schedule, GC
   │
   ▼
external reply
```

**Memory moves at three places only:**

1. AtomVM receive path (external bytes → BEAM heap)
2. AtomVM NIF marshal (BEAM heap → stack-allocated MotionPacket)
3. AtomVM NIF unmarshal (stack Snapshot → BEAM heap)

Inside `unibit-hot::step`, zero bytes move. The only operation is a
branchless select that flips bits in place on the pinned HotRegion.

---

## What "memory movement" means on each side

### Outside AtomVM

```
alloc      — ask the runtime for a new heap chunk
copy       — memcpy between heaps or mailboxes
free / GC  — release heap back to the runtime
move       — transfer ownership; may or may not copy
```

The outside is a world of **quantities**. Memory is a resource:
scarce, acquired, released, fragmented.

### Inside unibit-hot

```
no alloc      — HotRegion is pinned once at boot, never again
no copy       — instructions visit the region; data does not travel
no free       — the pin lasts the process lifetime
no move       — state transitions are bit flips in situ
```

The inside is a world of **positions**. Memory is a location:
pre-allocated, pre-aligned, never moved, its virtual address sealed
into the boot receipt.

These two worlds use the same word ("memory") for entirely different
things. The boundary choice — AtomVM — makes the translation explicit
and local.

---

## Why this matters: the "no GC inside" guarantee

If any part of the hot path could trigger an allocator, the
deterministic cycle budget is a lie. A single GC pause is millions of
cycles. A single heap expansion is hundreds of thousands.

By pushing all allocation to AtomVM, we get a compile-time guarantee:
**no instruction inside `unibit-hot` touches a runtime.** The hot
path's worst-case latency equals its best-case latency, because the
operations are closed over a fixed budget of pinned L1D operations.

In Rust terms:

```rust
// unibit-hot: denied
extern crate alloc;     // forbidden
use std::collections::*;  // forbidden
Box::new(...)            // forbidden (except at init)
Vec::new()               // forbidden
String::from(...)        // forbidden
Arc::new(...)            // forbidden

// unibit-hot: allowed
#![no_std]
[T; N] on stack
&'static references to pinned data
core::mem::* stack ops only
```

The `#![no_std]` declaration on `unibit-hot/src/lib.rs` is not a
fashion choice. It is the compile-time expression of the two-clock
architecture.

---

## What this says about our internal consideration of memory movement

**We don't have one.** We replaced the concept with *layout*.

Where other systems ask "where should this value be moved?" — which
presupposes that values move — we ask:

> "At what pinned offset does this bit pattern live, and which
> instruction will visit it next?"

The answers are:
- offset is determined by the MLA (Memory Layout Agreement, doc 40)
- instruction visit is determined by POWL8 (kinetic dialect, doc 45)

No answer is ever "the value is moved to X." Values are not objects
that travel. Values are **bit patterns at named offsets**, and
motions are **branchless transformations of those patterns**.

This is the deep payoff of the `64³` layout discipline: once you
commit to pinned memory, you stop thinking about memory movement and
start thinking about instruction motion. **The instruction is the
thing that travels; the data stays still.** (Doc 42 called this the
kinetic inversion.)

---

## The boundary does the dirty work

AtomVM exists because the outside world insists on memory movement.
External callers send messages of variable size. Their state has to
be allocated, parsed, GC'd. Our interior refuses all of that — but
the outside doesn't care what we refuse. Somebody has to translate.

AtomVM is that somebody.

```
AtomVM soaks up:
    - arbitrary message sizes
    - variable-length BEAM terms
    - per-request allocations
    - GC for all of the above
    - preemptive scheduling across thousands of BEAM processes
    - fault isolation ("let it crash")

AtomVM hands us:
    - a single MotionPacket of fixed size
    - at a stack address we control
    - with a known hash for receipt sealing
    - during a cycle window we specified
```

The NIF is where wall-clock time ends and instruction-count time
begins. One function call stands on both clocks. Before the call,
anything could have happened (GC, scheduling). After the call, nothing
can happen except the deterministic admission.

---

## What this forbids on the inside

Because of the boundary choice, the following are **forbidden inside
`unibit-hot`, `unibit-isa`, `unibit-phys`, `compile`, and `verify`:**

```
• any allocator
• any GC
• any message passing
• any lifetime that outlives the current motion
• any copy of a TruthBlock
• any move of a TruthBlock
• any reference that crosses the NIF boundary
• any code that could trigger a page fault after boot
• any lock, any channel, any condvar
• any call into libc, std::io, std::fs
• any panic that could unwind
```

The discipline is severe on purpose. Anything that could touch
wall-clock time taints the instruction-count contract. Wall-clock
concerns live only on the AtomVM side of the NIF.

---

## The NIF as the "memory barrier"

The AtomVM NIF is not just a Rust-to-Erlang call. It is the point
where memory transitions from "a quantity the runtime manages" to
"a location the ISA specifies."

```rust
#[nif]
pub fn motion_tick(
    env: NifEnv,
    packet_binary: NifBinary,     // owned by BEAM
) -> NifResult<NifBinary> {
    // (1) marshal boundary: BEAM heap → stack MotionPacket
    let packet: MotionPacket = parse_stack(packet_binary.as_slice())?;

    // (2) instruction-count zone: no allocation, no GC possible
    let outcome = unibit_hot::step(&mut HOT_REGION.lock(), &packet);

    // (3) unmarshal boundary: stack Snapshot → BEAM heap
    let snapshot: Vec<u8> = serialize_snapshot(&outcome);  // BEAM alloc
    NifBinary::from_slice(env, &snapshot)
}
```

Three zones. Zone (2) is the deterministic one. Zones (1) and (3) are
where memory moves. The compiler, the AtomVM scheduler, and the
linker conspire to make zone (2) untouchable by the allocator.

---

## What the boundary choice tells us

1. **We believe the outside world is not deterministic and cannot be
   made so.** AtomVM exists because we accept that.

2. **We believe the inside can be made deterministic if we refuse to
   do memory management there.** `unibit-hot` exists because we act
   on that belief.

3. **We believe the translation cost is worth paying once per
   request, at the NIF boundary, in exchange for cycle-exact interior
   behavior.** The NIF is that receipt.

4. **We believe wall-clock concerns are a category of requirement
   that must not leak inward.** The forbidden list enforces this.

These four beliefs are not separable. Pick any three and the fourth
follows mechanically.

---

## The receipt implication

Every AtomVM inbound message crosses the NIF and becomes part of an
`L0` fragment (the position hash of the HotRegion at motion start).
Every outbound Snapshot's `L5` release fragment is signed by
`unios` and then marshalled back by AtomVM into a BEAM term.

AtomVM never sees the L1..L4 fragments. It only sees L0 and L5 — the
envelope. This is deliberate: **the outside world audits the ends of
the chain; the interior proves the middle.** The interior's proof
(L1..L4) is never exported in raw form because it references pinned
addresses and instruction offsets that the outside has no context for.

The boundary is not just where memory stops moving. It is also where
receipts stop including interior-only fields.

---

## The sentence

**AtomVM at the boundary is the decision that memory movement,
garbage collection, message passing, and wall-clock scheduling all
live outside the core — so inside the core we do not have a
memory-management model, we have a layout; instructions travel to
pinned regions, bits flip branchlessly in place, no byte ever moves,
the only values that cross the NIF are fixed-size MotionPackets in
and fixed-size Snapshots out, and the cycle-exact determinism of the
interior is guaranteed by the single architectural rule that the
outside world's clock stops at the NIF and does not come in.**
