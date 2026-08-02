# 03 — Strategy for 100% Deterministically in Nanoseconds

## The audit

Exploration agent reviewed `src/bin/pdc2025.rs`, `src/conformance/case_centric/token_based_replay.rs`, and `src/conformance/mod.rs`.

Findings:

- `pdc2025.rs` `score_log` used `HashMap<String, usize>` — slow, O(places) per op.
- `token_based_replay.rs` uses `PackedKeyTable` but returns per-log aggregate only, no per-trace output, and no invisible-transition handling.
- Two incompatible replay implementations with divergent fitness values.
- No exact fitness=1.0 threshold path; always top-500 ranking.
- Invisible transition handling only existed in `pdc2025.rs`, absent from the bitmask path.

## Strategy

Three moves to reach 100% deterministic admission in nanoseconds:

1. **Replace HashMap with u64 bitmask replay.** Pack the marking into one `u64`; fire transitions as `(marking & !in_mask) | out_mask`; missing tokens = `popcount(in_mask & !marking)`. Single-register markings → single-instruction ops.

2. **Exact fitness threshold instead of top-500 ranking.** A trace is positive iff `missing == 0` and `remaining == 0` at final marking. Use top-N only as a tiebreaker when count ≠ 500.

3. **Unify the two replay implementations.** One canonical bitmask replay with invisible-transition fixpoint support, returning per-trace `(missing, remaining, produced, consumed)` as integers — no floats until final ratio.

## Tradeoff

Bitmask fast path requires ≤64 places. PDC 2025 may exceed. Need detection at
load time + fallback to 2-word (u128) or PackedKeyTable for larger nets.

## Data check

All 96 PDC 2025 models have ≤64 places (max exactly 64). Tier-1 u64 path
handles 100%. Distribution:

| Places | Models |
|--------|--------|
| 36 | 8 |
| 40 | 8 |
| 44 | 8 |
| 48 | 8 |
| 52 | 16 |
| 56 | 16 |
| 60 | 16 |
| **64** | **16** |

Fast path is the only path.
