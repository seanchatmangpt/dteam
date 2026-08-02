# 41 — C4 Diagrams: UniverseOS / unibit / dteam / unios

## C4 model recap

```
L1 Context    — systems and users surrounding the thing
L2 Container  — runtime deployables inside the thing
L3 Component  — modules inside a container
L4 Code       — classes/functions inside a component (omitted; lives in code)
```

All diagrams are Mermaid C4. Render with `@mermaid-js/mermaid-cli` or any
C4-aware tool.

---

## C4-L1 — System Context

```mermaid
C4Context
    title System Context — UniverseOS

    Person(analyst, "Process Analyst", "Submits XES/OCEL logs; consumes receipts")
    Person(operator, "Motion Operator", "Issues motion packets; consumes admission results")
    Person(auditor, "Auditor", "Verifies receipt chains; does not trust internals")

    System_Boundary(universeos, "UniverseOS") {
        System(unios, "unios", "Public surface: receipts, release, single C-ABI entry")
    }

    System_Ext(event_sources, "Event Sources", "XES logs, OCEL streams, OTel traces")
    System_Ext(prototypes, "Prototype Memory", "Nearest-lawful prototype library for REPAIR")
    System_Ext(release_target, "Release Target", "Downstream consumers of sealed artifacts")

    Rel(analyst, unios, "Submits logs; reads receipts")
    Rel(operator, unios, "Submits motion packets")
    Rel(auditor, unios, "Verifies receipt chain")

    Rel(event_sources, unios, "Event evidence", "XES / OCEL / OTel")
    Rel(unios, prototypes, "Nearest-lawful query", "cold path")
    Rel(unios, release_target, "Sealed artifacts", "BLAKE3 receipt")
```

**Read it as:** analysts, operators, and auditors interact with a single
system (unios). Everything below unios is invisible from outside. Event
sources feed in; prototypes are consulted only on the cold path; release
targets receive sealed artifacts.

---

## C4-L2 — Container Diagram

```mermaid
C4Container
    title Container Diagram — UniverseOS internals

    Person(operator, "Motion Operator")

    System_Boundary(universeos, "UniverseOS") {
        Container(unios, "unios", "Rust crate", "Public API; receipt chain; release gate")
        Container(dteam, "dteam", "Rust crate", "XES/OCEL ingestion; discovery; conformance; RL")
        Container(mustar, "mustar", "Rust crate", "HPowl → MotionPacket compiler; folds signatures")
        Container(unibit_isa, "unibit-isa", "Rust crate (no_std)", "Typed UInstr; tier/field/receipt encoding")
        Container(unibit_hot, "unibit-hot", "Rust crate (no_std)", "Hot kernels: admit/commit/reduce; NEON")
        Container(unibit_phys, "unibit-phys", "Rust crate (no_std)", "Pinned memory; alignment; position validation")

        ContainerDb(hotregion, "HotRegion", "64 KiB pinned L1D page", "TruthBlock + Scratchpad + Fields + ReceiptRing")
    }

    System_Ext(event_sources, "Event Sources")
    System_Ext(prototypes, "Prototype Memory")

    Rel(operator, unios, "motion_tick(packet)", "C-ABI")
    Rel(unios, dteam, "Log ingest; discovery")
    Rel(dteam, event_sources, "Read", "XES/OCEL")
    Rel(dteam, mustar, "HPowl model")
    Rel(mustar, unibit_isa, "Emit UInstr stream")
    Rel(unibit_isa, unibit_hot, "Dispatch typed instruction")
    Rel(unibit_hot, hotregion, "Read Truth / Write Scratch", "cache-line")
    Rel(unibit_hot, unibit_phys, "Pin; validate position")
    Rel(unios, prototypes, "REPAIR lookup", "cold")
    Rel(unios, operator, "Receipt fragment", "u128")
```

**Layering law:** each arrow crosses exactly one layer boundary. No
arrow skips. `unios` never reaches into `unibit-hot`; `dteam` never
reaches into `unibit-phys`.

---

## C4-L3a — Component Diagram: unibit-hot

```mermaid
C4Component
    title Component Diagram — unibit-hot (L1 hot kernels)

    Container_Boundary(hot, "unibit-hot") {
        Component(t0, "t0.rs", "8^2 kernel", "admit_eight / commit_8 / admit_commit_emit")
        Component(t1, "t1.rs", "8^4 kernel", "admit_tile / commit_tile over 64-word Tile")
        Component(t2, "t2.rs", "8^6 kernel", "admit_block_fused / commit_block over 4096-word TruthBlock")
        Component(fold, "fold.rs", "Signature folder", "Hv<U4096> → HdcSig128 (compile-time)")
        Component(prefetch, "prefetch.rs", "Prefetch helper", "__prefetch; called by caller frame")
        Component(reduce, "reduce.rs", "Reducer", "OR-reduce 2/4/8 lanes")
    }

    Container_Ext(isa, "unibit-isa", "Typed instruction entry")
    ContainerDb_Ext(hotregion, "HotRegion", "pinned L1D page")

    Rel(isa, t0, "AEF.t tier=8^2")
    Rel(isa, t1, "AEF.t tier=8^4")
    Rel(isa, t2, "AEF.t tier=8^6")

    Rel(t0, reduce, "reduce_or_8")
    Rel(t1, reduce, "reduce_or_8")
    Rel(t2, reduce, "reduce_or_8")

    Rel(t1, prefetch, "prefetch next tile")
    Rel(t2, prefetch, "prefetch next block")

    Rel(t0, hotregion, "read state u128")
    Rel(t1, hotregion, "read Tile; write Scratch")
    Rel(t2, hotregion, "read TruthBlock; write Scratch")

    Rel(t2, fold, "cold path: fold for REPAIR")
```

**Tier discipline:** t0 handles 8¹/8² (register-resident), t1 handles
8³/8⁴/8⁵ (tile-resident), t2 handles 8⁶ (block-resident). The
`fold.rs` helper is cold — only entered on escalation.

---

## C4-L3b — Component Diagram: unios (public surface)

```mermaid
C4Component
    title Component Diagram — unios (public surface)

    Person(operator, "Motion Operator")
    Person(auditor, "Auditor")

    Container_Boundary(unios_c, "unios") {
        Component(entry, "motion_tick", "#[no_mangle] C-ABI", "The one exported symbol")
        Component(gate, "release_gate", "Release controller", "Requires all 5 verification surfaces to agree")
        Component(chain, "receipt_chain", "Receipt manager", "Accumulates six fragments; BLAKE3-seals")
        Component(verifier, "receipt_verifier", "Chain verifier", "Independent of emitter; auditable")
        Component(lexicon, "lexicon_check", "Build-time", "Enforces forbidden vocabulary")
    }

    Container_Ext(dteam_ext, "dteam", "Log mining, conformance")
    Container_Ext(mustar_ext, "mustar", "Compile motion")
    Container_Ext(hot_ext, "unibit-hot", "Execute motion")

    Rel(operator, entry, "motion_tick(packet)")
    Rel(entry, dteam_ext, "mine log context")
    Rel(entry, mustar_ext, "compile HPowl → MotionPacket")
    Rel(entry, hot_ext, "dispatch UInstr")
    Rel(entry, chain, "emit fragment")
    Rel(chain, gate, "sealed chain")
    Rel(gate, operator, "result + receipt")
    Rel(auditor, verifier, "verify(receipt)")
    Rel(verifier, chain, "inspect (read-only)")
```

**Public surface rule:** every external caller touches only `motion_tick`
or `verify`. Nothing else in unios is exported.

---

## C4-L3c — Component Diagram: HotRegion Layout (data view)

```mermaid
C4Component
    title HotRegion Layout — 64 KiB pinned L1D page

    Container_Boundary(hot_region, "HotRegion (align 4096, pinned, mlocked)") {
        ComponentDb(truth, "TruthBlock", "offset 0, 32 KiB", "4,096 × u64 — what is")
        ComponentDb(scratch, "Scratchpad", "offset 32,768, 32 KiB", "4,096 × u64 — what might be")
        ComponentDb(fields, "PackedEightField", "offset 65,536, 512 B", "8 × FieldMask {required, forbidden}")
        ComponentDb(delta, "DeltaRing", "offset 66,048, 4 KiB", "256 × Delta {word_idx, old, new}")
        ComponentDb(receipts, "ReceiptRing", "offset 70,144, 4 KiB", "Ring buffer of u128 fragments")
        ComponentDb(pad, "Padding", "up to 65,536 B", "Tail padding to page boundary")
    }

    Container_Ext(hot, "unibit-hot", "Kernels")
    Container_Ext(phys, "unibit-phys", "Pin + validate")

    Rel(hot, truth, "read (hot path)")
    Rel(hot, scratch, "write (hot path)")
    Rel(hot, fields, "read (hot path)")
    Rel(hot, delta, "append (commit path, <256 words)")
    Rel(hot, receipts, "append (always)")
    Rel(phys, hot_region, "Pin<Box>, mlock, position-validate")
```

**Layout invariants:** all offsets are compile-time constants; any
change is a UHDC version bump.

---

## C4-L2' — Deployment / Cache Residence View

```mermaid
C4Deployment
    title Deployment — Cache Residence on M3 Max P-core

    Deployment_Node(cpu, "M3 Max P-core", "Apple Silicon") {
        Deployment_Node(regs, "Register File", "256 × 128-bit NEON") {
            Container(reg_tier, "Reg-tier live values", "tier 8^1 / 8^2")
        }
        Deployment_Node(l1d, "L1D — 128 KiB", "~4 cycle latency") {
            ContainerDb(hotregion_l1, "HotRegion (64 KiB)", "Truth + Scratch + Fields + Receipts")
            Container(working_set, "Working set (64 KiB)", "Hot kernel locals + prefetch window")
        }
        Deployment_Node(l2, "L2 — 16 MiB cluster", "~15 cycle latency") {
            Container(l2_tier, "Tier 8^7 residents", "Promoted blocks; escalation")
        }
        Deployment_Node(l3, "L3/SLC — 48 MiB", "~40 cycle latency") {
            Container(l3_tier, "Tier 8^8 residents", "Planning-tier large states")
        }
    }

    Deployment_Node(dram, "DRAM — 128 GiB LPDDR5", "~200 cycle latency") {
        Container(cold, "Cold storage", "Full Hv<U4096> prototypes; log archives")
    }

    Deployment_Node(nvm, "NVM — Persistent", "Disk / SSD") {
        ContainerDb(receipt_log, "Receipt log", "Append-only chain")
    }

    Rel(reg_tier, hotregion_l1, "reads state word")
    Rel(hotregion_l1, l2_tier, "promote (cold)")
    Rel(l2_tier, l3_tier, "escalate (cold)")
    Rel(l3_tier, cold, "spill (cold)")
    Rel(hotregion_l1, receipt_log, "seal on release")
```

**Pragmatic rule:** if the hot path ever reads from L2 or beyond, the
tier dispatch was wrong. Use Instruments' cache-miss counters as the
absolute truth; cache-miss count is the test.

---

## C4 — Dynamic View: One Motion Tick

```mermaid
sequenceDiagram
    autonumber
    participant Op as Motion Operator
    participant Un as unios::motion_tick
    participant Dt as dteam
    participant Mu as mustar
    participant Is as unibit-isa
    participant Ho as unibit-hot
    participant Hr as HotRegion (L1D)
    participant Ch as receipt_chain

    Op->>Un: motion_tick(MotionPacket)
    Un->>Dt: mine_context(log_ref)
    Dt-->>Un: conformance snapshot
    Un->>Mu: compile(HPowl)
    Mu->>Mu: fold Hv<U4096> → HdcSig128
    Mu-->>Un: UInstr stream (tier-tagged)
    Un->>Is: dispatch(UInstr)
    Is->>Ho: AEF.t (tier, field, class)
    Ho->>Hr: read Truth word (L1D hit, ~4 cycles)
    Ho->>Hr: check required/forbidden
    Ho->>Hr: write Scratch (branchless select)
    Ho->>Hr: append Delta (if <256 words)
    Ho-->>Is: deny_total, fragment
    Is-->>Un: result
    Un->>Ch: append fragment (L0..L5)
    alt all 5 surfaces agree
        Un->>Ch: SEAL (BLAKE3)
        Un-->>Op: admitted + sealed receipt
    else any surface disagrees
        Un-->>Op: denied + partial receipt
    end
```

**End-to-end budget (happy path, 8² tier):**
```
ingest + mine        ~ 50 ns   (dteam, cached)
compile              ~ 100 ns  (mustar, cached signatures)
dispatch             ~ 2 ns    (unibit-isa, monomorphized)
AEF.t                ~ 10 ns   (unibit-hot, L1D)
chain append         ~ 5 ns    (ring buffer)
seal                 ~ 50 ns   (BLAKE3, amortized)
────────────────────────────
total                ~ 220 ns  per motion tick
```

---

## C4 — Dynamic View: Escalation Path

```mermaid
sequenceDiagram
    autonumber
    participant Ho as unibit-hot (t0)
    participant Ho4 as unibit-hot (t1, 8^4)
    participant Ho6 as unibit-hot (t2, 8^6)
    participant Cold as prototypes (DRAM)

    Ho->>Ho: AEF.t=8^2
    Note right of Ho: deny != 0 but margin tight
    Ho->>Ho4: PROMOTE.8^2→8^4
    Ho4->>Ho4: admit_tile
    Note right of Ho4: still inconclusive
    Ho4->>Ho6: PROMOTE.8^4→8^6
    Ho6->>Ho6: admit_block_fused
    Note right of Ho6: still denied
    Ho6->>Cold: REPAIR: nearest-lawful prototype
    Cold-->>Ho6: prototype + distance
    Ho6-->>Ho4: DEMOTE.8^6→8^4 (apply repair)
    Ho4-->>Ho: DEMOTE.8^4→8^2 (resume)
```

**Escalation rule:** tier ascent is always followed by tier descent
carrying the result. The hot path returns to the warmest tier that
preserves the answer.

---

## Rendering

```bash
npx @mermaid-js/mermaid-cli \
  -i docs/opus/41_c4_diagrams.md \
  -o build/c4.svg \
  --configFile mermaidrc.json
```

Or pipe individual fenced blocks through `mmdc` for per-diagram SVG.

## The sentence

**Six C4 diagrams — context (who uses it), container (six crates
layered), three component views (hot kernels, public surface, hot region
layout), deployment (cache residence on M3 Max), and two dynamic views
(one tick, escalation) — that is the complete architectural picture at
the level of abstraction where trade-offs are still visible.**
