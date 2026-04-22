# 04 — Mermaid Architecture Diagrams

Four diagrams rendered to validate the planned bitmask-replay architecture.

## 1. Overall PDC 2025 pipeline

```mermaid
flowchart TD
    A[PNML Model] -->|read_pnml| B[PetriNet]
    B -->|compile_incidence + pack places| C{places ≤ 64?}
    C -->|yes| D[NetBitmask64: initial_mask u64, final_mask u64, trans Vec of in/out masks]
    C -->|no| E[PackedKeyTable fallback]
    F[XES Test Log] -->|XESReader| G[EventLog: Vec of Traces]
    D --> H[bitmask_replay_all: zero heap, stack only]
    E --> I[pkt_replay_all: no HashMap]
    G --> H
    G --> I
    H --> J[Vec of ReplayResult: missing, remaining, produced, consumed]
    I --> J
    J --> K[classify_traces: exact threshold first]
    K -->|missing=0 AND remaining=0| L[isPos=true perfect conformance]
    K -->|otherwise| M[rank by fitness, fill to N_target]
    L --> N[XES Output pdc:isPos annotations]
    M --> N
```

## 2. Bitmask token replay hot loop

```mermaid
flowchart TD
    A[marking: u64 = initial_mask] --> B[fire_invisible loop until fixpoint]
    B --> C{more events?}
    C -->|no| D[check final marking, missing += popcount of final_mask AND NOT marking, remaining += popcount of marking AND NOT final_mask]
    D --> E[return ReplayResult]
    C -->|yes activity label| F[lookup label_to_trans O1 packed table]
    F --> G{any transition enabled? marking AND in_mask == in_mask}
    G -->|yes| H[pick first enabled transition index]
    G -->|no| I[inject missing: missing += popcount in_mask AND NOT marking, marking = marking OR in_mask]
    I --> H
    H --> J[fire: new = marking AND NOT in_mask OR out_mask, consumed += popcount in_mask, produced += popcount out_mask]
    J --> K[fire_invisible fixpoint]
    K --> C
```

## 3. Two-tier net dispatch

```mermaid
flowchart LR
    A[PetriNet after read_pnml] --> B{count places}
    B -->|n ≤ 64| C[Tier-1 u64 NetBitmask64 stack resident 1 register per marking]
    B -->|64 lt n ≤ 128| D[Tier-2 u128 NetBitmask128 2 registers]
    B -->|n gt 128| E[Tier-3 PackedKeyTable bounded capacity]
    C --> F[enable check: 1 AND + 1 CMP ~1 ns]
    C --> G[fire: 1 AND + 1 OR ~1 ns]
    C --> H[missing count POPCNT ~1 ns]
```

## 4. Deterministic classification logic

```mermaid
flowchart TD
    A[Vec of ReplayResult one per trace] --> B[partition traces]
    B -->|missing=0 AND remaining=0| C[perfect: Vec of idx]
    B -->|otherwise| D[imperfect: Vec of idx + score sorted desc by fitness]
    C --> E{perfect.len vs N_target}
    E -->|== N_target| F[DONE take all perfect 100% deterministic]
    E -->|gt N_target tie break| G[secondary sort by trace length asc then lex on activities]
    G --> H[take first N_target from sorted perfect]
    E -->|lt N_target| I[take all perfect fill from top of imperfect]
    F --> J[classifications Vec of bool]
    H --> J
    I --> J
    J --> K[write_classified_log XES output]
```
