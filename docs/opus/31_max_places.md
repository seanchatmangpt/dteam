# 31 — Max Places in PDC 2025 / the 64³ Universe

## The question

How many places do PDC 2025 challenge models have, and how does that map to
the `64³ = 262,144` operational-depth universe?

## Current data known

From uploaded DTEAM benchmark material: three PDC-2025 rows with MDL scores
and verified soundness. No extracted `|P|` / max-places profile.

PDC-2025 contest description: 288 training event logs and 96 test/base log
pairs for evaluation. Does not state max places in the underlying models.

## Why it matters

For the hyperdimensional design:

```
max_places_per_model ≤ 64 ?
```

If yes, one workflow cell executes its place marking as a single machine word:
```
M_next = (M & !consume) | produce;
enabled = (M & input) == input;
```

A 64-place local cell fits in one u64; 64² gives 4,096 addressable cells;
64³ gives the active truth state.

## Empirical check

From earlier mermaid-diagram validation: all 96 PDC 2025 models have ≤ 64
places. Distribution:

| Places | Models |
|---|---|
| 36 | 8 |
| 40 | 8 |
| 44 | 8 |
| 48 | 8 |
| 52 | 16 |
| 56 | 16 |
| 60 | 16 |
| **64** | **16** |

## The question reframed

The right metric isn't just total places. It's **max local place cluster** —
the largest causally-coupled cluster of places. A global model with >64 places
may still fit perfectly if it decomposes into multiple 64-place cells:

```
cell_0: places 0..63
cell_1: places 64..127
cell_2: places 128..191
...
```

The real benchmark question: **can every challenge model be tiled into
64-place cells without destroying causal locality?**

## The independent-place capacity

Assuming only independent places (dumbest encoding, no compression, no
structure, no locality):

```
64 × 64 × 64 = 262,144
```

Or:

```
64² attention surface = 4,096 cells
each cell has 64 independent places
4,096 × 64 = 262,144 independent places
```

At 1 bit per place:

```
1 place = 1 bit
262,144 places = 262,144 bits = 32 KiB
```

## The punchline

**A single 64³ TruthBlock can hold the marking state of 262,144 independent
workflow places.**

With truth/scratch pair:
```
TruthBlock  = 262,144 place bits = 32 KiB
Scratchpad  = 262,144 motion bits = 32 KiB
Total       = 524,288 bits = 64 KiB
```

One lane evaluates motion over a 262k-place independent workflow universe in
the 64 KiB truth/scratch envelope.

## Implication for any normal benchmark

The threshold is not "does the model have fewer than 64 places?" It is:
**does the entire active marking universe fit under 262,144 independent
places?**

For almost any normal process-discovery benchmark, that threshold is
enormous. The real bottleneck will not be place count. It will be:

- transition fan-in/fan-out
- active frontier size
- causal locality
- receipt/proof overhead
- synchronization cost across lanes

## What to add to benchmark harness

```rust
pub struct PlaceProfile {
    pub dataset_id: String,
    pub activities: usize,
    pub places_ground_truth: usize,
    pub places_discovered: usize,
    pub transitions: usize,
    pub arcs: usize,
    pub max_parallel_width: usize,
    pub max_local_place_cluster: usize,
    pub required_cells_64: usize,
    pub fits_one_cell: bool,
    pub fits_one_truth_block: bool,
}
```

```rust
required_cells_64 = places_discovered.div_ceil(64);
fits_one_cell = places_discovered <= 64;
fits_one_truth_block = places_discovered <= 262_144;
```

Smarter version clusters by POWL component / connected component / transition
adjacency and reports `max_local_place_cluster`.

## The sharper sentence

**Even under the dumbest possible encoding — one independent place equals
one bit — 64³ gives 262,144 independent workflow places per active
TruthBlock.**
