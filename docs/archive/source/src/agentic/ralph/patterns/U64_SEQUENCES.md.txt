# UniverseOS Sequence Diagrams — Subsystem Interaction Reference

## 1. Core Law Sequence
[U_t stays resident. Instructions move. Deltas emit. Receipts remember.]

```mermaid
sequenceDiagram
    autonumber
    participant Truth as Data Plane / U_t
    participant Motion as UniverseScratch (Plane S)
    participant Law as Admission Kernel
    participant Memory as UReceipt
    participant View as UProjection

    Motion->>Truth: bring tiny instruction to resident universe
    Law->>Truth: test lawful motion ((U & I) == I)
    Truth->>Truth: transition scoped words branchlessly
    Truth->>Memory: emit UDelta and update UReceipt
    Truth->>View: update projections from UDelta
```

## 2. Cell-Level Petri64 Transition (T1)
```mermaid
sequenceDiagram
    autonumber
    participant SP as UniverseScratch (Plane S)
    participant DP as UniverseBlock (Plane D)
    participant CM as Cell Microkernel
    participant DB as UDelta Builder

    SP->>CM: cell_word_index, input_mask, output_mask
    CM->>DP: read M = U[word]
    CM->>CM: enabled_mask = select(((M & I) == I), !0, 0)
    CM->>CM: candidate = (M & !I) | O
    CM->>CM: next = select(enabled_mask, candidate, M)
    CM->>DP: write next to U[word]
    CM->>DB: emit before, after, changed_mask (XOR)
```

## 3. Sparse Multi-Word Transition (T1)
```mermaid
sequenceDiagram
    autonumber
    participant SP as UniverseScratch (Plane S)
    participant AW as ActiveWordSet
    participant DP as UniverseBlock (Plane D)
    participant SM as Sparse Microkernel
    participant DB as UDelta Builder

    SP->>AW: active word list
    AW->>SM: bounded active words
    loop for each active word
        SM->>DP: read U[word_i]
        SM->>SP: read I_i, O_i
        SM->>SM: compute enabled contribution
    end
    SM->>SM: reduce enabled into fired_mask
    loop for each active word
        SM->>SM: candidate_i = (U_i & !I_i) | O_i
        SM->>SM: next_i = select(fired, candidate_i, U_i)
        SM->>DP: commit next_i
        SM->>DB: emit changed word index + Δ
    end
    DB->>SP: store sparse UDelta
```

## 4. Deterministic RL Update from UDelta
```mermaid
sequenceDiagram
    autonumber
    participant BUS as UDelta Bus
    participant RL as RL Reward Kernel
    participant SP as UniverseScratch (Plane S)
    participant QS as Q/Policy State (PackedKeyTable)

    BUS->>RL: UDelta
    RL->>SP: read good_mask, bad_mask, target_mask refs
    RL->>RL: reward = popcount(UDelta & good) - popcount(UDelta & bad)
    RL->>QS: branchless update Q-value/policy entry
    QS-->>RL: updated policy state
```

## 5. Substrate Integrity Receipt (Hot Path)
```mermaid
sequenceDiagram
    autonumber
    participant DB as UDelta Builder
    participant RM as UReceipt Mixer
    participant RS as Receipt State (Rolling Hash)
    participant DT as DeltaTape

    DB->>RM: instruction_id, scope, fired_mask, UDelta
    RM->>RS: read current receipt R_t
    RM->>RM: mix deterministic FNV-1a material
    RM->>RS: write R_t+1
    RM->>DT: attach receipt fragment to tape entry
```

## Subsystem Index
| Subsystem | Diagrams |
| :--- | :--- |
| **Execution** | Core Law (1), Cell Transition (2), Sparse Transition (3) |
| **Reinforcement** | RL Reward Update (4) |
| **Provenance** | Receipt Hot Path (5) |
| **Orchestration** | Full-Block T2 Scan (See `U64_ARCHITECTURE.md`) |
