# End-to-End JTBD Architecture: COG8, POWL, and Bounded Closure

**Version:** 1.0 (Canonical)
**Status:** Approved Architecture Pillar
**Core Thesis:** End-to-end JTBD (Jobs to be Done) handling is achieved through graphs of bounded COG8 closures, compiled by `ainst`, executed by `ccog`, and proved by POWL64.

---

## 1. System Context (C4 Level 1)

The stack handles JTBDs end-to-end by moving the complexity "left" into compilation, allowing the runtime kernel to remain a small, deterministic executor of live field state.

```mermaid
flowchart TB
    Human["People / Operators / Users / Caregivers / Workers"]
    Devices["Devices / Sensors / Apps / IoT / Edge Nodes"]
    Systems["Enterprise Systems / EHR / CMMS / WMS / CRM / IAM"]
    Sources["Policies / Care Plans / SOPs / Graphs / Cases / Reports"]
    AINST["ainst<br/>Compiler / Control Plane"]
    CCOG["ccog<br/>Runtime Cognitive Kernel"]
    Ledger["EvidenceLedger / Replay / Audit"]
    Actions["Actions / Notifications / Actuation / Task Routing / Reports"]

    Human --> Devices
    Human --> Systems
    Devices --> CCOG
    Systems --> CCOG
    Sources --> AINST
    Systems --> AINST
    Devices --> AINST

    AINST --> CCOG
    CCOG --> Ledger
    CCOG --> Actions
    Actions --> Human
    Actions --> Systems
```

---

## 2. Container View (C4 Level 2)

Hard surface separation between manufacturing (Control Plane), execution (Runtime Plane), and proof (Proof Plane) ensures architectural integrity.

```mermaid
flowchart LR
    subgraph ControlPlane["ainst Control Plane"]
        Ontology["Ontology / World Model Loader"]
        Motif["Motif / Policy / Case Compiler"]
        Admission["Admission / Validation / Perturbation Tests"]
        PackGen["CompiledCcogConfig Generator"]
    end

    subgraph RuntimePlane["ccog Runtime Plane"]
        L3["L3 Field Surface<br/>COG8 Graphs / Field Packs"]
        L2["L2 Field Snapshot"]
        L1["L1 Bark Kernel<br/>Mask Banks / POWL8 Executor"]
        Writer["CONSTRUCT8 Writer"]
        Router["Response Router"]
    end

    subgraph ProofPlane["Proof / Memory Plane"]
        P64["POWL64 Route / ABI"]
        Ledger["EvidenceLedger"]
        Replay["Replay / Audit / Reporting"]
    end

    subgraph External["External Field"]
        Sensors["Sensors / Apps / Devices"]
        Systems["Enterprise Systems"]
        Humans["Humans / Operators"]
    end

    Ontology --> Motif
    Motif --> Admission
    Admission --> PackGen
    PackGen --> L3

    Sensors --> L2
    Systems --> L2
    Humans --> L2

    L3 --> L2
    L2 --> L1
    L1 --> Router
    L1 --> Writer
    L1 --> P64

    Writer --> Systems
    Router --> Humans
    Router --> Systems
    Router --> Sensors

    P64 --> Ledger
    Ledger --> Replay
```

---

## 3. Runtime Component View (C4 Level 3)

The internal execution of a JTBD inside `ccog` is a bounded cognitive cycle.

```mermaid
flowchart TB
    Ingest["Field Ingest / Adapters"]
    Normalizer["Field Normalizer"]
    Snapshot["ClosedFieldContext / L2 Snapshot"]
    GraphExec["COG8 Graph Executor"]
    POWL8["POWL8 Instruction Engine"]
    Choice["Choice-Graph / Partial-Order Resolver"]
    Collapse["Collapse Attribution Engine"]
    Decision["Canonical Response Selector"]
    Writeback["CONSTRUCT8 Writer"]
    P64["POWL64 Route Builder"]
    Output["Action / Task / Event Output"]

    Ingest --> Normalizer
    Normalizer --> Snapshot
    Snapshot --> GraphExec
    GraphExec --> POWL8
    GraphExec --> Choice
    POWL8 --> Collapse
    Choice --> Collapse
    Collapse --> Decision
    Decision --> Writeback
    Decision --> Output
    Decision --> P64
    Writeback --> P64
```

---

## 4. JTBD Meta-Workflow

Every JTBD supported by the architecture follows this end-to-end sequence:

1. **JTBD Definition**: High-level job requirement.
2. **Field / World Modeling**: Identifying the relevant entities and relations.
3. **Closure Variable Selection**: Selecting the minimal set of bits/predicates needed to close the job.
4. **COG8 Decomposition**: Breaking the job into bounded units of 8 triples.
5. **POWL Topology Generation**: Designing the nonlinear routing/loops for the job.
6. **Admission / Perturbation Tests**: Proving the job logic is sensitive to evidence.
7. **CompiledCcogConfig**: Emitting the admitted runtime artifact.
8. **Runtime Field Ingest**: Closing the loop with live data.
9. **L2 Snapshot**: Constructing the unified `ClosedFieldContext`.
10. **COG8 Graph Execution**: Collapsing the field state.
11. **Canonical Response**: Emitting the instinct decision.
12. **CONSTRUCT8 Writeback**: Committing the bounded delta.
13. **POWL64 Proof Route**: Recording the path for replay.

---

## 5. Non-linear Topology

The COG8/POWL topology allows multiple closures to happen independently or converge, handling complex JTBDs without monolithic rules.

```mermaid
flowchart LR
    S((Start))

    A["COG8 Identity Closure"]
    B["COG8 Evidence Closure"]
    C["COG8 Authorization Closure"]
    D["COG8 Risk Closure"]
    E["COG8 Routine Closure"]
    F["COG8 Inspect Node"]
    G["COG8 Escalation Node"]
    H["COG8 Settle Node"]
    I["COG8 Refusal Node"]

    T((End))

    S --> A
    S --> B
    S --> E

    A --> F
    B --> F
    E --> H

    F --> C
    F --> D

    C --> I
    D --> G

    H --> T
    I --> T
    G --> T
```

---

## 6. End-to-End Checklist

A JTBD is supported if and only if it satisfies these 10 criteria:
- [ ] Explicit Field/World model exists.
- [ ] Closed set of closure variables identified.
- [ ] COG8 decomposition (units of 8) is possible.
- [ ] POWL topology (motion) is defined.
- [ ] Mapped to canonical response lattice.
- [ ] Loop/Inspect/Retrieve requirements are bounded.
- [ ] Writeback fits within CONSTRUCT8.
- [ ] POWL64 route is recordable.
- [ ] EvidenceLedger replay is defined.
- [ ] Action integration is mapped to the external world.
