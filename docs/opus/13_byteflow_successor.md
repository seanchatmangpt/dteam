# 13 — ByteFlow Successor: UBitScopePlanner

## The problem

ByteFlow was the precursor research name for the "admission/scheduling"
component. User flagged that it must not persist publicly — "ByteFlow was
just research."

## The concept survives; the name does not

Old ByteFlow was:

```
PetriNet + Marking → EnabledTransitions → ScheduleWork
```

The mature UBit version is:

```
ProcessSemantics + Capability + Law + Locality + CacheTier + CoreTopology
    → CompiledHotHandles
```

## What the thing actually is

```
ReadySet(t) = Enabled(t) ∧ LawOK(t) ∧ CapabilityOK(t)
            ∧ ResourceOK(t) ∧ LocalityOK(t) ∧ ConflictFree(t) ∧ TierOK(t)
```

Compile to denial form:

```
deny(t) = deny_enabled | deny_law | deny_capability | deny_resource
        | deny_locality | deny_conflict | deny_tier
```

Polarity: `deny = 0 ⇒ admitted`; `deny ≠ 0 ⇒ denied`.

## Naming successors considered

| Name | Meaning |
|---|---|
| UBitFrontier | Best technical name — names the scheduling object |
| UBitMotion | Broad; names whole substrate, not specifically scheduler |
| **UBitScopePlanner** | Best continuity with existing UBit primitive set |
| UFrontierPlanner | Best hybrid |

## Decision

Public subsystem: **UBitScopePlanner**.
Core scheduling artifact: **UFrontier**.
Executable result: **UMotionHandle**.

## Clean ontology

```
UniverseOS
  └── unibit
        ├── UBitField
        ├── UBitImage
        ├── UBitCapability
        ├── UBitScopePlanner
        ├── UBitSupervisor
        ├── UFrontier
        ├── UCompiledMotion
        └── UMotionHandle
```

## Formal definition

```
UBitScopePlanner: (O*, U_t, C_t, H_t) → {UMotionHandle_i}_{i=1}^n
```

Each handle is already `Enabled ∧ LawOK ∧ CapabilityOK ∧ LocalityOK ∧ TierOK`,
or represented as a branchless denial mask:

```
deny_i = deny_enabled | deny_law | deny_cap | deny_locality | deny_tier
```

## Admissible Frontier Scheduling

```
AFS(U_t) = { m_i ∈ Motions | Enabled_i(U_t) ∧ Law_i(U_t) ∧ Capability_i(U_t)
                          ∧ Resource_i(U_t) ∧ Locality_i(U_t) ∧ Scenario_i(U_t) }
```

Then:

```
Schedule(U_t) = SelectNonConflicting(AFS(U_t), HardwareTopology, ReceiptPolicy)
```

Then:

```
Compile(Schedule(U_t)) → {UWorkHandle_1, ..., UWorkHandle_n}
```

## Naming migration table

| Research name | Keep? | Successor |
|---|---|---|
| ByteFlow | ❌ | UBitScopePlanner |
| ByteActor | ❌ public | UBitKernel / UMotionKernel |
| ByteEthos | ❌ | UBitReceipt / DeltaTape / UBitSupervisor |
| ByteCore ABI | ❌ | UMotionABI / unibit-core |
| Crystal envelope | ❌ | UInstruction / UMotionHandle |
| Result crystal | ❌ | UDeltaRef / UMotionResult |
| Doctrine of 8 | maybe historical | T0/T1 timing law |

## Best final wording

ByteFlow was a research artifact. Its surviving invariant is not flow; it is
**frontier closure**. In UniverseOS, the successor is UBitScopePlanner: it
compiles the admissible frontier into UMotionHandles, then disappears.

No ByteFlow. No UBitFlow. Just:

```
O* → UFrontier → UMotionHandle → Kernel
```
