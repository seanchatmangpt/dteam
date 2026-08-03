# 02 — Wiring β/λ into the RL Reward

## Gap

`train_with_provenance_projected` in `src/automation.rs` accepted `beta` and
`lambda` but prefixed them `_beta`, `_lambda`. `_unsoundness_u` and
`_complexity_c` were computed but discarded. The RL agent called
`select_action()` each epoch but never called `update()`. No reward signal
ever reached the learner.

Paper claim: MDL penalty (λ) and reward shaping (β) drive structural
minimality in training. Standard path used them; projected path did not.

## Fix

1. Removed unused `warn` from `use log::{info, warn};`
2. Wired β and λ into the reward computation:
   ```
   reward = avg_f + β·(1 - unsoundness_u) − λ·complexity_c
   ```
3. Introduced `prev_state` / `prev_action` tracking so `agent.update(prev_state, pa, reward, prev_state, done)` fires each epoch.
4. Verified `RlState` and `RlAction` both derive `Copy`.
5. Verified `QLearning::update` signature: `(state: S, action: A, reward: f32, next_state: S, done: bool)`.
6. Verified `structural_unsoundness_score()` returns `f32`.

## Result

- `cargo make check` — clean.
- `cargo make test` — 82/82 pass.

## Summary

| Gap | Fix |
|---|---|
| Unused `warn` import | removed from import |
| β/λ not wired | added reward formula + `agent.update()` call per epoch |

Both gaps closed; the paper's strong-form claim (β/λ drive structural
minimality) now has the code to match it.
