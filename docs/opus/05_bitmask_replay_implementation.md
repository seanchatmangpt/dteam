# 05 — Bitmask Replay Implementation

## Files created / modified

- `src/conformance/bitmask_replay.rs` — new module
- `src/conformance/mod.rs` — added `pub mod bitmask_replay;`
- `src/bin/pdc2025.rs` — replaced `NetIndex`/HashMap scoring with `NetBitmask64` + `classify_exact`

## Core types

```rust
pub struct ReplayResult {
    pub missing: u32,
    pub remaining: u32,
    pub produced: u32,
    pub consumed: u32,
}

impl ReplayResult {
    pub fn fitness(&self) -> f64 { /* 4-counter formula */ }
    pub fn is_perfect(&self) -> bool {
        self.missing == 0 && self.remaining == 0
    }
}

struct TransMask {
    in_mask: u64,
    out_mask: u64,
    is_invisible: bool,
    in_popcount: u32,
    out_popcount: u32,
}

pub struct NetBitmask64 {
    pub initial_mask: u64,
    pub final_mask: u64,
    pub n_places: usize,
    transitions: Vec<TransMask>,
    label_index: Vec<(String, Vec<usize>)>,   // sorted for binary search
    invisible_indices: Vec<usize>,
}
```

## Hot loop (`replay_trace`)

```rust
let mut marking = net.initial_mask;
let mut missing = 0u32;
let mut consumed = 0u32;
let mut produced = net.initial_mask.count_ones();

fire_invisible(net, &mut marking);

for event in &trace.events {
    let activity = /* extract concept:name */;
    let t_indices = match net.label_index.binary_search_by(...) { ... };
    let t_idx = t_indices.iter().copied()
        .find(|&i| (marking & net.transitions[i].in_mask) == net.transitions[i].in_mask)
        .unwrap_or(t_indices[0]);
    let t = &net.transitions[t_idx];
    let need = t.in_mask & !marking;
    if need != 0 {
        missing += need.count_ones();
        marking |= need;
    }
    marking = (marking & !t.in_mask) | t.out_mask;
    consumed += t.in_popcount;
    produced += t.out_popcount;
    fire_invisible(net, &mut marking);
}

// Final marking
let need = net.final_mask.count_ones();
let have = (marking & net.final_mask).count_ones();
if need > have {
    missing += need - have;
    marking |= net.final_mask & !marking;
}
consumed += need;
marking &= !net.final_mask;
let remaining = marking.count_ones();
```

## Classify

```rust
pub fn classify(results: &[ReplayResult], n_target: usize) -> Vec<bool> {
    // Perfect first; fill remaining from top-fitness imperfect.
    // Tie-break by ascending index (deterministic).
}
```

## Bench results

86/86 tests pass (82 old + 4 new). Lint clean. Binary runs all 96 PDC 2025
logs in ~1 second.

**Accuracy: 67.29% avg across 96 logs.**

Typical per-log output:

```
pdc2025_000000: places=18 perfect=7/1000 accuracy=66.6%
pdc2025_000001: places=18 perfect=16/1000 accuracy=69.4%
...
pdc2025_121111: places=64 perfect=0/1000 accuracy=66.2%
=== PDC 2025 Results: 96/96 logs, avg accuracy=67.29% ===
```

## Interpretation

Infrastructure correct. Very few traces are "perfect" (0–19 of 1000 per log).
Models are imprecise — many negative traces also replay with high fitness.
Top-500-by-fitness doesn't cleanly separate positives from negatives.

67% baseline says fitness is a useful signal but not sufficient alone.
Possible improvements (not implemented):
- Re-add BaseDfg differential (multiply, not subtract)
- Per-trace length normalization
- Alignment-based replay (A* search)
- Training log feature extraction
