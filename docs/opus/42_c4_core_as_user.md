# 42 — C4 From the Chip Core's Perspective

## The inversion

Doc 41 drew C4 with humans as users. That is the conventional view.

The kinetic view: **the chip core is the user**. Humans, logs, and
motion packets are all upstream suppliers feeding the core. The core
consumes instructions and emits receipts. UniverseOS is the service the
core calls; everything else is the core's vendor chain.

The Pragmatic question this answers: *what does the core need, in what
order, at what rate, from what residence?*

---

## C4-L1 — Context (core as user)

```mermaid
C4Context
    title Context — Chip Core as User

    Person(core, "P-core", "M3 Max execution pipeline; consumes UInstr; emits deny/fragment")
    Person_Ext(e_core, "E-core", "Background; runs cold paths; handles receipts seal")
    Person_Ext(neon, "NEON SIMD unit", "Co-user; consumes vector lanes in fused AEF.t")

    System_Boundary(universeos, "UniverseOS (as consumed by the core)") {
        System(dispatch, "Instruction Service", "Supplies UInstr stream at ≥ pipeline rate")
        System(data_service, "Data Service", "Keeps TruthBlock+Scratchpad pinned in L1D")
        System(receipt_sink, "Receipt Sink", "Accepts fragments; never blocks the core")
    }

    System_Ext(human, "Humans / Upstream", "Analysts, operators — slow, rate-limited")
    System_Ext(log_feed, "Event Feed", "XES/OCEL — bursty, buffered")
    System_Ext(prototype_db, "Prototype Library", "Cold; DRAM-resident")

    Rel(human, dispatch, "enqueue motion", "µs scale")
    Rel(log_feed, dispatch, "stream events", "ms scale")

    Rel(dispatch, core, "UInstr byte stream", "≥ 1 / cycle")
    Rel(data_service, core, "Truth/Scratch residence", "L1D ~4 cy")
    Rel(core, receipt_sink, "fragment u128", "≤ 1 / cycle")
    Rel(core, neon, "fused vector ops", "register rename")
    Rel(core, e_core, "offload cold work", "IPI / shared queue")
    Rel(prototype_db, core, "REPAIR result", "DRAM ~200 cy (cold only)")
```

**Read it as:** the P-core is the paying customer. Humans are suppliers
three tiers removed. The contract UniverseOS signs with the core is:
deliver one typed instruction per cycle, keep Truth/Scratch resident,
accept fragments without backpressure.

---

## C4-L2 — Container (the core's supply chain)

```mermaid
C4Container
    title Container — Core's Supply Chain

    Person(core, "P-core", "Consumer")
    Person_Ext(neon, "NEON", "Co-consumer")

    System_Boundary(supply, "UniverseOS Supply Chain") {
        Container(iqueue, "Instruction Queue", "Lock-free ring", "Head-tail SPSC; UInstr bytes")
        Container(prefetcher, "Prefetch Engine", "Software prefetch", "Warms next tile/block in L1D")
        ContainerDb(hotregion, "HotRegion", "64 KiB pinned L1D page", "Truth + Scratch + Fields + Receipts")
        Container(fragment_ring, "Fragment Ring", "Lock-free SPSC", "u128 fragments written by core")
        Container(escalator, "Tier Escalator", "PROMOTE/DEMOTE manager", "Moves residence across cache tiers")
        Container(spill_yard, "Spill Yard", "L2/L3 staging", "Holds promoted blocks; decays to DRAM")
    }

    System_Ext(feeder, "Upstream Feeder", "MuStar compile; emits UInstr stream")
    System_Ext(seal_thread, "Seal Thread (E-core)", "Drains fragment ring; BLAKE3 on E-core")

    Rel(feeder, iqueue, "push UInstr")
    Rel(iqueue, core, "pop UInstr", "1 / cycle")
    Rel(prefetcher, hotregion, "warm next tile")
    Rel(core, hotregion, "read Truth; write Scratch", "L1D hit")
    Rel(core, neon, "dispatch vector op")
    Rel(neon, hotregion, "vector load/store")
    Rel(core, fragment_ring, "push fragment", "1 / cycle")
    Rel(fragment_ring, seal_thread, "pop + seal")
    Rel(escalator, hotregion, "evict to L2")
    Rel(escalator, spill_yard, "store promoted")
    Rel(spill_yard, hotregion, "refill on DEMOTE")
```

**The contract:** every container on the core's side of the bus is
lock-free, bounded, and non-blocking. If the instruction queue empties,
the core stalls — that is the producer's fault (MuStar compile is too
slow). If the fragment ring fills, the core drops or blocks — that is
the sink's fault (seal thread is too slow). Both are visible as counters.

---

## C4-L3a — Component: The Instruction Queue

```mermaid
C4Component
    title Component — Instruction Queue (core's input port)

    Person(core, "P-core", "Consumer")

    Container_Boundary(iq, "Instruction Queue") {
        Component(head, "Head Counter", "atomic u64", "Producer-owned")
        Component(tail, "Tail Counter", "atomic u64", "Consumer-owned")
        ComponentDb(ring, "Ring Buffer", "4 KiB aligned", "[UInstr; 512]")
        Component(watermark, "Low Watermark", "Refill trigger", "If head-tail < 64, signal producer")
        Component(refiller, "Refiller", "Background", "Requests next compile window from MuStar")
    }

    Container_Ext(mustar_ext, "MuStar")

    Rel(core, tail, "advance after pop")
    Rel(core, ring, "pop UInstr", "cache-line read")
    Rel(head, ring, "push UInstr")
    Rel(mustar_ext, head, "compile; push")
    Rel(watermark, refiller, "signal")
    Rel(refiller, mustar_ext, "request batch")
```

**Pragmatic invariant:** the core never waits on a lock. The ring's size
(512 entries = 4 KiB, one cache line of entries per 64 bytes) is sized
so the head/tail separation survives one producer compile pass.

---

## C4-L3b — Component: The HotRegion (core's working memory)

```mermaid
C4Component
    title Component — HotRegion (core's working memory)

    Person(core, "P-core")
    Person_Ext(neon, "NEON")

    Container_Boundary(hr, "HotRegion (64 KiB, pinned, L1D-resident)") {
        ComponentDb(truth, "TruthBlock", "offset 0, 32 KiB", "the state the core reads")
        ComponentDb(scratch, "Scratchpad", "offset 32768, 32 KiB", "the state the core writes")
        ComponentDb(fields, "PackedEightField", "offset 65536, 512 B", "the masks the core checks against")
        ComponentDb(delta, "DeltaRing", "offset 66048, 4 KiB", "the deltas the core appends")
        ComponentDb(receipts, "ReceiptRing", "offset 70144, 4 KiB", "the fragments the core emits")
    }

    Container_Ext(prefetcher, "Prefetcher")
    Container_Ext(escalator, "Escalator")

    Rel(core, truth, "read word", "1 cycle from L1D")
    Rel(core, fields, "read mask", "1 cycle from L1D")
    Rel(core, scratch, "write word", "1 cycle to L1D")
    Rel(neon, truth, "vector load")
    Rel(neon, scratch, "vector store")
    Rel(core, delta, "append delta")
    Rel(core, receipts, "append fragment")
    Rel(prefetcher, truth, "warm next tile")
    Rel(escalator, truth, "evict / refill")
```

**The geometry:** the core sees exactly one page, always at the same
virtual address, always pinned, always warm. Every offset within that
page has semantic meaning; position validation runs at pin time. The
core never reads cold memory on the hot path.

---

## C4-L3c — Component: The Core's Own Pipeline

```mermaid
C4Component
    title Component — P-core Pipeline (what the core does internally)

    Container_Boundary(pcore, "P-core pipeline") {
        Component(fetch, "Fetch", "Front-end", "Pulls UInstr from iqueue")
        Component(decode, "Decode", "Opcode parse", "tier / field / class from one byte")
        Component(dispatch, "Dispatch", "Port select", "ALU / NEON / load / store")
        Component(alu, "ALU", "Integer ops", "AND / OR / XOR / popcount")
        Component(neon_unit, "NEON Unit", "Vector ops", "fused u128 admit")
        Component(ls, "Load/Store", "Memory ops", "L1D hit required")
        Component(retire, "Retire", "Commit ROB", "Append fragment, advance tail")
    }

    Container_Ext(iq_ext, "Instruction Queue")
    Container_Ext(hr_ext, "HotRegion")
    Container_Ext(fr_ext, "Fragment Ring")

    Rel(iq_ext, fetch, "UInstr byte")
    Rel(fetch, decode, "")
    Rel(decode, dispatch, "")
    Rel(dispatch, alu, "scalar class")
    Rel(dispatch, neon_unit, "vector class")
    Rel(dispatch, ls, "memory class")
    Rel(ls, hr_ext, "L1D read/write")
    Rel(alu, retire, "")
    Rel(neon_unit, retire, "")
    Rel(ls, retire, "")
    Rel(retire, fr_ext, "emit fragment")
```

**Pragmatic lens:** the core is a 7-stage pipeline that wants one fused
admit-commit-emit superop per cycle. Anything that causes a dispatch
stall (cache miss, branch misprediction, NEON port contention) is
visible in Instruments. Those are the metrics that matter — not wall
clock.

---

## C4 — Deployment (core's physical world)

```mermaid
C4Deployment
    title Deployment — Core's Physical Neighborhood

    Deployment_Node(p_core, "P-core (consumer)", "M3 Max performance core") {
        Deployment_Node(pipe, "Pipeline", "7-stage OoO") {
            Container(front_end, "Front-end", "fetch + decode")
            Container(back_end, "Back-end", "ALU + NEON + LS")
        }
        Deployment_Node(priv_l1, "Private L1", "128 KiB D + 192 KiB I") {
            ContainerDb(hotregion_dep, "HotRegion", "pinned 64 KiB")
            ContainerDb(icache, "I-cache", "hot UInstr lines")
        }
    }

    Deployment_Node(cluster, "P-cluster", "4 P-cores shared") {
        Deployment_Node(l2_dep, "Shared L2", "16 MiB") {
            Container(spill_dep, "Spill Yard", "tier 8^7 residents")
        }
    }

    Deployment_Node(soc, "SoC-wide", "M3 Max") {
        Deployment_Node(slc, "SLC / System LLC", "48 MiB") {
            Container(planning_tier, "Tier 8^8", "planning-size blocks")
        }
        Deployment_Node(e_cluster, "E-cluster", "efficiency cores") {
            Container(seal_worker, "Seal Worker", "BLAKE3 on E-core")
            Container(log_ingest_worker, "Log Ingest", "XES/OCEL parse")
        }
    }

    Deployment_Node(dram_dep, "DRAM", "128 GiB LPDDR5") {
        Container(cold_dep, "Cold Storage", "Hv<U4096> prototypes")
    }

    Rel(front_end, icache, "fetch", "1 cycle")
    Rel(back_end, hotregion_dep, "L1D", "4 cycles")
    Rel(hotregion_dep, spill_dep, "promote/demote")
    Rel(spill_dep, planning_tier, "escalate")
    Rel(planning_tier, cold_dep, "spill")
    Rel(back_end, seal_worker, "IPI to E-core")
    Rel(log_ingest_worker, back_end, "prefilled iqueue")
```

**The budget:**
```
front-end -> L1D hit      5 cycles round-trip
fused AEF.t              10 cycles (pipelined, 1 retire/cycle amortized)
fragment write            1 cycle
```

If the core achieves ~1 admitted motion per 10 cycles at 3 GHz, that is
**300 M admissions/sec per P-core**. Four P-cores synchronized = 1.2 B/s
peak. This is the target.

---

## C4 — Dynamic: One Cycle (the core's perspective)

```mermaid
sequenceDiagram
    autonumber
    participant Core as P-core
    participant IQ as Instruction Queue
    participant HR as HotRegion (L1D)
    participant NEON as NEON Unit
    participant FR as Fragment Ring

    Note over Core,FR: cycle t
    Core->>IQ: pop UInstr (load from ring)
    IQ-->>Core: AEF.t=8^2, field=law, receipt=Fragment

    Note over Core,FR: cycle t+1
    Core->>HR: load Truth[word_idx]
    Core->>HR: load Fields.law.required
    Core->>HR: load Fields.law.forbidden

    Note over Core,FR: cycle t+2
    Core->>NEON: vector admit (missing|present)
    NEON-->>Core: deny_total

    Note over Core,FR: cycle t+3
    Core->>Core: admitted_mask = (deny==0).neg()
    Core->>HR: store Scratch[word_idx] = select(candidate, old, mask)

    Note over Core,FR: cycle t+4
    Core->>FR: push fragment (deny XOR next)
    Core->>IQ: advance tail
```

**Five cycles end-to-end.** Fully pipelined: five instructions all in
flight, one retiring each cycle, so throughput is **one admission per
cycle** at steady state.

---

## C4 — Dynamic: Core's Complaint Path

```mermaid
sequenceDiagram
    autonumber
    participant Core as P-core
    participant PM as Performance Monitor
    participant Esc as Escalator
    participant Feeder as MuStar Feeder

    Core->>PM: L1D miss counter += 1
    PM->>PM: moving window breach (>1% of admits)
    PM->>Esc: "HotRegion evicted — repin"
    Esc->>Core: pause, repin HotRegion, resume

    Core->>PM: IQ empty stall counter += 1
    PM->>PM: breach (>0.1% of cycles)
    PM->>Feeder: "compile faster — batch size 2x"
    Feeder-->>Core: deeper iqueue refill

    Core->>PM: FR full stall counter += 1
    PM->>PM: breach (>0.1% of cycles)
    PM->>Esc: "seal worker behind — add E-core"
```

**The core complains in counters, not logs.** The Performance Monitor
reads counters every ~1 M cycles, applies thresholds, and issues
repairs. This is the feedback loop that keeps the core fed, warm, and
unblocked.

---

## The Pragmatic reframing

Doc 41 was the architect's view: *how do humans use this?*
Doc 42 is the core's view: *how does the core get what it needs?*

Both must be true. They are the same system from two ends of the
supply chain.

| View | Upstream | Downstream |
|---|---|---|
| Human-as-user (doc 41) | human intent | sealed artifacts |
| Core-as-user (doc 42)  | UInstr stream | retired fragments |

When the two views agree — when humans' intent arrives as UInstr at the
core's rate, and the core's fragments roll up to humans' receipts — the
supply chain is balanced.

---

## The sentence

**From the core's perspective, UniverseOS is a supply chain that must
deliver one typed instruction per cycle, keep the 64 KiB HotRegion
pinned in L1D, and accept a 128-bit fragment per cycle without
backpressure — and every architectural decision is a clause in that
contract, not a feature for humans.**
