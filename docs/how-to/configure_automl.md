# Enable and Configure AutoML Discovery

This guide covers turning on HDIT AutoML in `dteam.toml` and understanding what each key
controls. AutoML is disabled by default (`enabled = false`). When enabled, it runs a greedy
orthogonal signal selection across the 15 HDIT candidate pool and writes a per-log plan JSON.

---

## Configuration keys

The `[automl]` section lives at the bottom of `dteam.toml`. Config is loaded by
`AutonomicConfig::load("dteam.toml")` in `src/config.rs` line 197; a missing file silently
returns `Default`.

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `enabled` | bool | `false` | `true`, `false` | Master switch. When `false` all other keys are ignored. |
| `strategy` | string | `"random"` | `"random"`, `"grid"` | Search strategy for hyperparameter trials. See below. |
| `budget` | usize | `20` | Any positive integer | Number of RandomSearch trials per log. Must be > 0 when enabled. |
| `seed` | u64 | `42` | Any u64 | Base seed for reproducible trial ordering. |
| `successive_halving` | bool | `false` | `true`, `false` | Enable TPOT2 two-rung evaluation. Rung-0 subsamples; top fraction advances to rung-1. |
| `sh_subsample` | f64 | `0.2` | `(0.0, 1.0)` | Fraction of traces used for rung-0 scoring. Only read when `successive_halving = true`. |
| `sh_promotion_ratio` | f64 | `3.0` | `> 1.0` | Keep top `1/ratio` candidates after rung 0. `3.0` keeps the top third. |

---

## The banned `ensemble_only` strategy

Setting `strategy = "ensemble_only"` causes an immediate panic at startup — not a graceful
error. The validation runs at `src/bin/pdc2025.rs` lines 141-153 before any log is
processed:

```
AutoML config lie: strategy="ensemble_only" has been removed.
It was a structural no-op: combinatorial_ensemble + score_vs_in_lang
is a supremum operation that absorbs trial variation, so all trials
produced identical scores. Use "random" or "grid" instead.
```

The same startup guard also panics if `budget = 0` while `enabled = true`:

```
AutoML config lie: budget=0 would produce zero trials but AutoML is enabled
```

The DoD verifier (`cargo make dod`) scans `dteam.toml` for `ensemble_only` as a separate
check (Phase 3, check 1) so it will also flag this even without running the binary.

---

## How `successive_halving` changes behavior

When `successive_halving = false` (default): `run_hdit_automl(candidates, &anchor, 500)` is
called. All 15 candidates are evaluated against the full trace set.

When `successive_halving = true`: `run_hdit_automl_sh(candidates, &anchor, 500, sh_subsample,
sh_promotion_ratio)` is called instead (dispatch at `src/bin/pdc2025.rs` lines 1006-1016).

Rung-0 scores every signal on `sh_subsample` fraction of traces. Only the top
`1/sh_promotion_ratio` signals advance to rung-1 where they are scored against all traces.
With defaults (`sh_subsample=0.2`, `sh_promotion_ratio=3.0`), rung-0 uses 20% of traces and
the top third of candidates proceed. This reduces total scoring work but may demote a signal
that performs poorly on the subsample.

---

## Reading AutoML plan JSON fields

Each plan JSON is written to `artifacts/pdc2025/automl_plans/<stem>.json`. Key fields:

| Field | Meaning |
|---|---|
| `accounting_balanced` | Must be `true`. Invariant: `selected + rejected_corr + rejected_gain == evaluated`. |
| `oracle_gap` | `plan_accuracy_vs_gt - oracle_vs_gt`. Negative means the plan beat the single best signal; 0.0 means tied. |
| `pareto_front` | Array of `PlanCandidate` objects. Exactly one has `"chosen": true`. |
| `plan_accuracy_vs_anchor` | Fraction of traces where the fused plan agrees with the conformance anchor. |
| `plan_accuracy_vs_gt` | Fraction of traces where the fused plan agrees with ground truth labels. |
| `fusion` | Operator chosen: `Single`, `WeightedVote`, `BordaCount`, or `Stack`. |
| `selected` | Array of signal names included in the final fused prediction. |
| `tiers` | Per-signal tier assignments (`T0`/`T1`/`T2`/`Warm`). |

`oracle_gap` is verified by the DoD: stored value must equal `plan_accuracy_vs_gt -
oracle_vs_gt` to within 1e-6 (`src/agentic/ralph/verifier.rs` lines 260-267).

---

## Minimal config to enable AutoML

```toml
[automl]
enabled = true
strategy = "random"
budget = 20
seed = 42
```

For faster iteration with successive halving (evaluate each signal on 20% of traces first,
advance the top third):

```toml
[automl]
enabled = true
strategy = "random"
budget = 20
seed = 42
successive_halving = true
sh_subsample = 0.2
sh_promotion_ratio = 3.0
```

After updating `dteam.toml`, run the full pipeline:

```bash
cargo run --bin pdc2025 --release
```

Then verify the artifacts:

```bash
cargo make dod
```
