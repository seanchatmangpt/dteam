# C4 Documents — Universe64 Single-Core Dual-Plane L1 Architecture

## Scope
This C4 model describes **Universe64 running inside one CPU core** using the **Dual-Plane L1 execution model**:
- **Plane_D (Data Plane):** 32 KiB resident state (`UniverseBlock = [u64; 4096]`).
- **Plane_S (Scratch Plane):** 32 KiB workspace (masks, active-word sets, deltas, staging).

## The Core Law
> **The data plane stays resident. The scratch plane receives operators, masks, and temporary motion state.**

## C1 — System Context
```mermaid
C4Context
    title Universe64 Dual-Plane Single-Core Context

    Person(instr, "Instruction Producer", "Provides small admitted/candidate instruction values.")

    System_Boundary(core, "Single CPU Core / Dual-Plane L1 Envelope") {
        System(u64, "Universe64 Kernel", "Resident 32 KiB universe + 32 KiB scratchpad execution plane.")
    }

    Rel(instr, u64, "Submits tiny instructions/operators")
```

## C2 — Container Diagram
```mermaid
C4Container
    title Universe64 Dual-Plane Container Diagram

    System_Boundary(core, "Universe64 Single-Core Kernel") {
        Container(data, "Data Plane", "32 KiB [u64;4096]", "Resident canonical UniverseBlock U_t.")
        Container(scratch, "Scratch Plane", "32 KiB bounded workspace", "Masks, deltas, active words, temporary operands, reductions.")

        Container(instr, "Instruction Stream", "Copy-sized instructions", "Transition ids, mask ids, coordinates, projection ids, receipt tags.")
        Container(admit, "Admission Kernel", "Truth predicate", "Checks admissibility against Data Plane using Scratch operands.")
        Container(trans, "Transition Kernel", "Branchless mask algebra", "Applies scoped transition to Data Plane.")
        Container(delta, "Delta Kernel", "Sparse motion algebra", "Builds Delta U in Scratch Plane.")
        Container(receipt, "Receipt Kernel", "Rolling proof state", "Mixes instruction, scope, delta, and fired mask.")
        Container(ready, "Ready Mask Kernel", "Dependency masks", "Updates affected transition masks from Delta U.")
        Container(proj, "Projection Kernel", "Projection algebra", "Updates views/projections from Delta U.")
    }

    Rel(instr, scratch, "stages operands in")
    Rel(scratch, admit, "provides masks/scope")
    Rel(admit, data, "reads scoped words from")
    Rel(admit, trans, "passes fired mask")
    Rel(trans, data, "mutates scoped words in")
    Rel(trans, delta, "emits before/after words to")
    Rel(delta, scratch, "stores sparse delta in")
    Rel(delta, receipt, "feeds")
    Rel(delta, ready, "feeds")
    Rel(delta, proj, "feeds")
```

## C3 — Component Diagram
```mermaid
C4Component
    title Universe64 Dual-Plane Component Diagram

    Container_Boundary(kernel, "Universe64 Kernel") {
        Component(dataOwner, "DataPlane Owner", "Resident state", "Owns the canonical 32 KiB UniverseBlock.")
        Component(scratchOwner, "ScratchPlane Owner", "Bounded workspace", "Owns the 32 KiB scratchpad for masks, deltas, active words, and reductions.")

        Component(coord, "UCoord Mapper", "Coordinate algebra", "Maps (domain, cell, place) to word index and bit offset.")
        Component(instr, "Instruction Decoder", "Instruction algebra", "Decodes instruction kind, scope, transition id, mask id, receipt tag.")
        Component(mask, "Mask Stager", "Scratch staging", "Loads or references admitted masks into scratch scope.")
        Component(active, "Active Word Set", "Scratch index", "Tracks bounded word indexes touched by current operation.")
        Component(admit, "Admission Evaluator", "Truth predicate", "Computes enablement/fired masks from Data Plane and Scratch masks.")
        Component(cell, "Cell Microkernel", "Single-word algebra", "Executes Petri64 local transition.")
        Component(sparse, "Sparse Microkernel", "Bounded-word algebra", "Executes active-word transition using Scratch active set.")
        Component(full, "Full Block Kernel", "4096-word algebra", "Executes T2 full-block operations.")
        Component(delta, "Delta Builder", "Motion algebra", "Writes changed-word delta into Scratch Plane.")
        Component(receipt, "Receipt Mixer", "Proof algebra", "Mixes delta, instruction id, fired mask, and scope.")
        Component(ready, "Ready Mask Maintainer", "Dependency algebra", "Updates affected transition masks from delta.")
        Component(proj, "Projection Updater", "View algebra", "Updates projections from delta.")
    }

    Rel(instr, coord, "uses")
    Rel(instr, mask, "requests mask/scope")
    Rel(mask, scratchOwner, "stages operands")
    Rel(active, scratchOwner, "stores active indexes")
    Rel(admit, dataOwner, "reads scoped data")
    Rel(admit, scratchOwner, "reads staged masks")
    Rel(admit, cell, "fires local path")
    Rel(admit, sparse, "fires sparse path")
    Rel(admit, full, "fires full-block path")
    Rel(cell, dataOwner, "commits word")
    Rel(sparse, dataOwner, "commits active words")
    Rel(full, dataOwner, "commits/scans full block")
    Rel(cell, delta, "before/after")
    Rel(sparse, delta, "before/after")
    Rel(full, delta, "block delta")
    Rel(delta, scratchOwner, "writes delta")
    Rel(delta, receipt, "feeds")
    Rel(delta, ready, "feeds")
    Rel(delta, proj, "feeds")
```

## Timing Constitution
- **T0 (Primitive):** $\le 2\text{ns}$ (Masks, Selects).
- **T1 (Microkernel):** $\le 200\text{ns}$ (Cell/Sparse Transitions, Delta building).
- **T2 (Orchestration):** $\le 5\mu\text{s}$ (Full-block scan, Hamming distance).
- **T3 (Epoch):** $\le 100\mu\text{s}$ (System synthesis, cryptographic hashing).

## Implementation Rules
1. **Instructions move, not data:** The kernel receives tiny operator IDs, not 32 KiB payloads.
2. **Delta is the event:** Output is $\Delta U = U_t \oplus U_{t+1}$ and a rolling `UReceipt`.
3. **No branches (CC=1):** Transition admissibility must be computed via bitwise mask logic.
4. **No heap:** All staging must fit inside the 32 KiB `Plane_S`.
