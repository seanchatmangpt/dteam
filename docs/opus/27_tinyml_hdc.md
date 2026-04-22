# 27 — TinyML + HDC: Patterns to Lift

## Key insight

TinyML + HDC already solved the problem of useful learning under brutal
resource constraints. The lesson is not "copy TinyML" — it is:

**Lift those patterns from MCU scale to chip-geometry scale: cache-local
hypervectors, branchless binding/bundling/permutation, associative memory,
adaptive dimensionality, and robust noisy evidence.**

## HDC core operations mapped

| HDC op | Usual meaning | UniverseOS meaning |
|---|---|---|
| Binding | combine key + value | bind role/fact to coordinate |
| Bundling | combine many facts | build local process context |
| Permutation | encode order / sequence | encode trajectory / POWL order |
| Similarity | compare hypervectors | distance from lawful manifold |
| Associative memory | retrieve closest class / pattern | nearest lawful route / repair |

## Binary HDC on constrained hardware

TinyML HDC implementations often use binary / low-precision hypervectors:
- cheap, robust, hardware-friendly
- packed into machine integers
- bitwise operations for binding/permutation
- 4096-bit hypervectors in experiments, packed 32 dimensions per integer

UniverseOS direct match:
```
U_{1,4096}   = one HDC attention hypervector
U_{1,32768}  = active tile / multi-field bundle
U_{1,262144} = field memory / associative memory page
```

`64² = 4,096` attention geometry is literally representable as a 4,096-bit
hypervector. 512-unit packed representation → fast Hamming / XOR / popcount.

And `64³ = 262,144` becomes:
```
64 hypervectors × 4,096 bits = 64 stacked attention surfaces
= hyperdimensional operational geometry
```

## HDC binding for GlobeCell

Instead of `GlobeCell(domain, cell, place)` as a tuple, encode as hypervector:

```
HV_cell = bind(DOMAIN_role, HV_domain)
        ⊕ bind(CELL_role, HV_cell)
        ⊕ bind(PLACE_role, HV_place)
```

For binary HDC:
- binding = XOR
- bundling = majority / threshold / saturating counter
- permutation = rotate / lane shuffle
- similarity = Hamming distance / `popcount(XOR)`

The coordinate becomes a **semantic vector**, not just an index. Bridge
between geometry, process mining, hyperdimensional information theory, and
chip-local bit motion.

## Permutation for POWL order

```
sequence(A, B)     = bundle(permute(A, 0), permute(B, 1))
parallel(A, B)     = bundle(A, B)
choice(A, B)       = alternate lawful submanifolds
loop(A)            = bounded cyclic permutation
partial_order(A,B) = causal cone constraint
```

A trajectory:
```
HV_trace = bundle(
  permute(HV_step_0, 0),
  permute(HV_step_1, 1),
  permute(HV_step_2, 2), ...
)
```

Conformance = distance:
```
distance = hamming(HV_observed_trace, HV_lawful_trace)
```

## Associative memory for nearest lawful repair

HDC classifiers: class hypervectors as model weights, updated with single
bundling operation; classification checks which class hypervector is most
similar to sample.

UniverseOS translation:
```
class hypervector      → lawful trajectory prototype
sample hypervector     → proposed route/process motion
nearest class          → nearest lawful repair
update class prototype → learn new lawful operating pattern
```

MuStar Lawful Trajectory Associative Memory:
```
LawfulRouteMemory {
  prototype_hv[]
  route_id[]
  proof_obligation_template[]
  motion_packet_template[]
}
```

Denied route? `encode denied → compare to prototypes → retrieve repair → compile packet`.

## Two-tier memory (LifeHD pattern)

LifeHD uses short-term working memory + long-term associative memory with
adaptive dimension reduction.

UniverseOS equivalent:
```
short-term working memory  = L1 Truth / Scratch + active scope
long-term memory           = L2 / unified memory lawful prototype bank
```

Chip geometry:
```
L1D            = current motion / context hypervectors
L2             = local route / prototype neighborhood
unified memory = global lawful trajectory associative memory
```

## Adaptive dimensionality (MicroHD pattern)

MicroHD co-optimizes HDC hyperparameters for TinyML constraints — up to
200× compression with accuracy degradation under 1%.

**MuStar Dimension Planner:** chooses smallest HDC dimensionality that
preserves required accuracy/proof margin. Not every motion needs 64³. Not
every route needs 262,144 dimensions. Not every repair search needs full
field geometry.

## On-device self-debugging (DEBUG-HD pattern)

HDC can detect input corruptions and debug whether input / projection /
route is corrupted.

Add a corruption / anomaly field (or fold into causality/conformance).
MuStar diagnostic:
```
HV_expected_motion
HV_observed_delta
HV_telemetry_span
HV_ocel_event
HV_receipt_chain

corruption_score = distance(bundle(all surfaces), expected_claim_hv)
```

Multi-surface corroboration becomes HDC-native. Instead of checking five
surfaces as only structured records, encode each surface as a hypervector
and compare:
```
ClaimHV ≈ ExecutionHV ≈ TelemetryHV ≈ StateHV ≈ OCELHV ≈ CausalityHV
```

## The architecture addition

### A. Hypervector layer inside MuStar

```
POWL AST
→ HDC semantic encoding
→ GlobeCell hypervectors
→ trajectory hypervector
→ field masks
→ Motion Packet
```

Compile to hypervectors first, then lower to masks.

### B. Lawful Trajectory Associative Memory

```rust
struct LawfulTrajectoryMemory {
    prototypes: [HyperVector4096; N],
    route_templates: [RouteTemplate; N],
    proof_templates: [ProofObligation; N],
}
```

### C. HDC surfaces for verification

Add HDC encoding to the five corroboration surfaces:
```
ExecutionHV, TelemetryHV, StateHV, ProcessLogHV, CausalityHV, ClaimHV
```

Continuous trust score plus hard cryptographic proof.

## Architecture proposal

```
64² = one 4,096-bit attention hypervector
64³ = 64 stacked attention hypervectors = operational depth
64⁴ = 64 operational-depth slabs = meaning field
```

GlobeCell(domain, cell, place):
- domain selects slab
- cell selects bit/region in 4096-HV
- place selects one of 64 stacked HDC planes

Both geometric indexability and HDC similarity semantics.

## External validation

TinyML/HDC gives external validation that:

1. Binary high-dimensional vectors are useful on constrained hardware.
2. Binding/bundling/permutation are enough for meaningful learning.
3. Associative memory supports lightweight training/inference.
4. HDC is robust to noise and bit-level errors.
5. Adaptive dimensionality can trade accuracy and resource cost.
6. On-device debugging/corroboration can be HDC-native.

UniverseOS extends from tiny sensor inference to lawful enterprise/process
geometry.

## The phrase

**MuStar treats 64² as an HDC attention hypervector, 64³ as stacked
operational hypervectors, and 64⁴ as a meaning field. POWL supplies the
process grammar; HDC supplies the semantic geometry; unios supplies
admission; unibit supplies lawful motion.**
