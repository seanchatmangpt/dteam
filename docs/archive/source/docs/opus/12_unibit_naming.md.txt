# 12 — unibit Naming and Ancestral Lineage

## The naming decision

| Layer | Name |
|---|---|
| Package / ecosystem | **unibit** |
| API type prefix | **UBit*** |
| Formal math | **U_{1,n}** |
| Public OS environment | **UniverseOS** |
| Runtime substrate | **UniverseBit** |

## Precursor research

User provided precursor material on ByteStar / ByteActor / BitActor / ByteCore.
The core discovery: **ByteFlow validates before the hot path; ByteActor runs
as a C-fast execution engine with zero runtime guards, L1-resident kernels,
SPSC rings, MPHF dispatch, no dynamic allocation, and a strict "Doctrine of 8"
≤8 CPU-tick target.**

That is exactly the ancestor of:

```
UniverseOS_visible = Plan + Bind + Admit + Compile
UniverseOS_hot     = Words + Masks + Kernel + Scratch
```

## The lineage

```
ByteCore ABI
    ↓
ByteActor / BitActor
    ↓
DTEAM
    ↓
Universe64
    ↓
UBit / UniverseOS
```

## Stage-by-stage invariant

| Stage | What it discovered |
|---|---|
| ByteCore ABI | Fixed envelope/result ABI, zero-copy kernel contract, content-addressed outputs |
| ByteActor | Hot path must be guardless, L1-resident, branch-free, constant-time |
| BitActor | Per-core isolation beats shared scheduling; rings + NUMA locality matter |
| DTEAM | Branchless mask calculus + PKT + zero heap produces real process-intelligence kernels |
| Universe64 | The 32 KiB 64³ state lattice is the natural active-world geometry |
| UBit | The bit itself is the truth atom; the OS is lawful state motion over U_{1,n} |

## Mapping ByteStar → UBit

| ByteStar precursor | UBit / UniverseOS successor |
|---|---|
| ByteFlow admission | UBitScopePlanner + UBitCapability + law mask binding |
| ByteActor hot path | fused UBit hot kernel |
| ByteEthos ledger | UBitReceipt / DeltaTape / manifest root |
| env64_t crystal | UInstruction / compact instruction envelope |
| res64_t crystal | UBitDeltaRef / result fragment |
| MPHF dispatch | precompiled kernel class dispatch |
| SPSC rings | per-core UBit instruction/delta queues |
| BLAKE3 content addressing | receipt fragments / image ids / tape roots |
| Doctrine of 8 | T0/T1 microbench law |
| 64 kernels | canonical UBit kernel family |
| L1 hot set | U_{1,64³} active universe hot slice |

## Historical retained names (until breaking migration)

- `UniverseBlock`
- `UniverseExecutor`
- `UniverseScratch`
- `UCoord`
- `UDelta`
- `UInstruction`
- `TransitionReceipt`
- `ActiveWordSet`
- `DeltaTape`

## The upgrade from ByteActor to UBit

**ByteActor:** byte/crystal/envelope-native.
**UBit:** bit/truth/field-native.

```
crystal ABI           → truth geometry
kernel result         → lawful state motion
receipt after execution → receipt as proof of admitted transition
```

## BitActor's multicore answer

BitActor: exclusively multicore execution with no single-core fallback, per-core
engines, private queues, local scratch memory, independent dispatch tables, no
locks/mutexes.

UBit equivalent: per-core active universes with a globally lawful field;
UBitActive262144 + UBitScratch + UBitDeltaRing + UBitKernelTable per core;
UBitField16777216 + ReceiptTape + Supervisor + ImageRegistry shared off-hot-path.

## ByteCore ABI as UBit microbench ABI

Old ABI: envelope / result.

New ABI: instruction / state / scratch / result.

Same spirit: fixed layout, zero-copy, cache-aligned, no heap, constant-time,
receipt-capable.

## The key synthesis

Old: three separated entities (ByteFlow admission, ByteActor execution,
ByteEthos governance).

UBit fuses their semantics but preserves their timing separation:

```
UBitField / ScopePlanner / Capability = admission before hot work
UBitKernel                            = execution during hot work
DeltaTape / Receipt / Supervisor      = governance after or beside hot work
```

**Line:** UniverseOS manufactures a hot path, then gets out of the way.

## Doctrine of 8 becomes Doctrine of U_{1,n}

| UBit level | Role | Microbench target |
|---|---|---|
| U_{1,64} | word atom | T0, ≤2 ns |
| U_{1,64²} | attention | T0/T1 |
| U_{1,64³} | active universe, 32 KiB | T1/T2 |
| U_{1,64⁴} | meaning field, 2 MiB | L2 prepare/compile, not hot mutation |
| DeltaTape / receipts | proof | amortized/tiered |
