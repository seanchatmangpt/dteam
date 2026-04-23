# dteam.toml Configuration Reference

## Overview

`AutonomicConfig::load("dteam.toml")` reads this file at startup. If the file is absent, all fields fall back to `Default`. All sections are optional; unset fields use the defaults listed below.

---

## [meta]

Identity and environment metadata. Informational only — no runtime behaviour depends on these values.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `version` | String | `"2026.04.18"` | Any string | Configuration schema version. Used in manifests and audit output. |
| `environment` | String | `"autonomous"` | Any string | Deployment environment label (`"production"`, `"staging"`, etc.). |
| `identity` | String | `"dteam-alpha-01"` | Any string | Instance identity. Appears in manifest output. |

```toml
[meta]
version     = "2026.04.18"
environment = "production"
identity    = "dteam-prod-01"
```

---

## [kernel]

Controls the PackedKeyTable tier, memory alignment, and allocation policy for the hot-path engine.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `tier` | String | `"K256"` | `"K64"`, `"K128"`, `"K256"`, `"K512"`, `"K1024"` | K-tier for PackedKeyTable sizing. K64 enables the u64 bitmask fast path for nets with ≤ 64 places. |
| `alignment` | usize | `8` | `8`, `16`, `32`, `64` | Memory alignment in bytes. 8-byte (64-bit) alignment is required for branchless mask operations. Do not lower. |
| `determinism` | String | `"strict"` | `"strict"`, `"relaxed"`, `"non_deterministic"` | Replay determinism guarantee. `"strict"` enforces canonical ordering and reproducible hashing. |
| `allocation_policy` | String | `"zero_heap"` | `"zero_heap"`, `"stack_only"`, `"bounded_heap"` | Heap allocation policy for hot paths. `"zero_heap"` enforces the no-heap constraint on conformance replay and RL update loops. |

```toml
[kernel]
tier              = "K256"
alignment         = 8
determinism       = "strict"
allocation_policy = "zero_heap"
```

---

## [autonomic]

Top-level operational mode and integrity settings for the autonomic cycle.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `mode` | String | `"guarded"` | `"recommend"`, `"guarded"`, `"autonomous"` | Operational mode. `"guarded"` enforces risk and policy gates. `"recommend"` proposes only, never executes. `"autonomous"` executes without policy gates. |
| `sampling_rate` | u64 | `100` | Any positive integer | Online ingestion interval in milliseconds. |
| `integrity_hash` | String | `"fnv1a_64"` | `"fnv1a_64"`, `"sha256"`, `"blake3"` | Hash algorithm used for manifest integrity. `"fnv1a_64"` matches the internal `fnv1a_64` hashing used throughout the hot-path engine. |

```toml
[autonomic]
mode           = "guarded"
sampling_rate  = 100
integrity_hash = "fnv1a_64"
```

---

## [autonomic.guards]

Runtime safety thresholds evaluated at the start of each `run_cycle()` call.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `risk_threshold` | String | `"Low"` | `"Low"`, `"Medium"`, `"High"` | Maximum `ActionRisk` level that `accept()` will pass to `execute()`. Actions with a higher risk level are rejected. |
| `min_health_threshold` | f32 | `0.7` | `0.0`–`1.0` | Minimum `process_health` score required to proceed with the propose/accept/execute cycle. |
| `max_cycle_latency_ms` | u64 | `50` | Any positive integer | Informational ceiling for single-cycle latency in milliseconds. Logged but not enforced in code. |
| `repair_authority` | String | `"senior_engineer"` | Any string | Annotation written into manifests for structural repair actions. Not evaluated by code. |

> **NOTE — hardcoded conformance floor:** Regardless of `min_health_threshold`, `run_cycle()` also checks `state.conformance_score < 0.75`. This floor is hardcoded in `src/autonomic/kernel.rs` and **cannot be changed via `dteam.toml`**. If conformance falls below 0.75, the kernel returns an empty result set and no actions execute.

```toml
[autonomic.guards]
risk_threshold       = "Low"
min_health_threshold = 0.85
max_cycle_latency_ms = 50
repair_authority     = "senior_engineer"
```

---

## [autonomic.policy]

Controls `accept()` filtering logic and adapt-phase weighting.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `profile` | String | `"strict_conformance"` | `"strict_conformance"`, `"permissive"`, `"audit_only"` | Compliance profile. `"strict_conformance"` unconditionally rejects `ActionRisk::High` actions. `"permissive"` allows High-risk actions if they pass the `risk_threshold` guard. `"audit_only"` is intended for use with `mode = "recommend"` for read-only observation. |
| `mdl_penalty` | f32 | `0.05` | `0.0`–`1.0` | Minimum Description Length penalty applied to actions that increase model complexity. Higher values favour simpler models. |
| `human_weight` | f32 | `0.8` | `0.0`–`1.0` | Weight for human feedback signals in the `adapt()` phase. `1.0` makes human feedback the sole driver of health updates. |

```toml
[autonomic.policy]
profile      = "strict_conformance"
mdl_penalty  = 0.05
human_weight = 0.8
```

---

## [rl]

Reinforcement learning algorithm and hyperparameters for the discovery loop.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `algorithm` | String | `"DoubleQLearning"` | `"QLearning"`, `"DoubleQLearning"`, `"SARSA"`, `"ExpectedSARSA"`, `"REINFORCE"`, `"LinUCB"` | RL algorithm used in `train_with_provenance`. |
| `learning_rate` | f32 | `0.08` | `0.0`–`1.0` | Step size for Q-table updates (α). |
| `discount_factor` | f32 | `0.95` | `0.0`–`1.0` | Future reward discount (γ). |
| `exploration_rate` | f32 | `0.2` | `0.0`–`1.0` | Initial ε for ε-greedy exploration. |
| `exploration_decay` | f32 | `0.999` | `0.0`–`1.0` | Multiplicative decay applied to `exploration_rate` each epoch. |
| `reward_weights` | HashMap\<String, f32\> | `{fitness=0.6, soundness=0.2, simplicity=0.1, latency=0.1}` | Map of signal names to weights | Weights for the composite reward surface. Weights need not sum to 1.0 but the ratio between them determines relative importance. |

```toml
[rl]
algorithm         = "DoubleQLearning"
learning_rate     = 0.08
discount_factor   = 0.95
exploration_rate  = 0.2
exploration_decay = 0.999
reward_weights    = { fitness = 0.6, soundness = 0.2, simplicity = 0.1, latency = 0.1 }
```

---

## [discovery]

Process discovery loop configuration.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `max_training_epochs` | usize | `100` | Any positive integer | Maximum number of epochs before the discovery loop halts regardless of fitness. |
| `fitness_stopping_threshold` | f64 | `0.995` | `0.0`–`1.0` | Early stopping: loop terminates when fitness meets or exceeds this value. |
| `strategy` | String | `"incremental"` | `"incremental"`, `"batch"`, `"sliding_window"` | Discovery strategy. `"incremental"` processes one event at a time. `"batch"` processes a full log per epoch. `"sliding_window"` uses a moving window of `drift_window` events. |
| `drift_window` | usize | `1000` | Any positive integer | Number of events in the sliding window for concept drift detection. Only used when `strategy = "sliding_window"`. |

```toml
[discovery]
max_training_epochs       = 100
fitness_stopping_threshold = 0.995
strategy                  = "incremental"
drift_window              = 1000
```

---

## [paths]

File system paths for logs, models, and output artifacts.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `training_logs_dir` | String | `"data/pdc2025/training_logs"` | Any path | Directory containing labelled training XES logs. |
| `test_logs_dir` | String | `"data/pdc2025/test_logs"` | Any path | Directory containing unlabelled test XES logs. |
| `ground_truth_dir` | String | `"data/pdc2025/ground_truth"` | Any path | Directory containing ground-truth XES logs with `pdc:isPos` attributes. |
| `artifacts_dir` | String | `"artifacts"` | Any path | Root output directory for pipeline artifacts, plans, and classified logs. |
| `manifest_bus_path` | String | `"tmp/dmanifest_bus"` | Any path | Path for the manifest integrity bus file. |

```toml
[paths]
training_logs_dir = "data/pdc2025/training_logs"
test_logs_dir     = "data/pdc2025/test_logs"
ground_truth_dir  = "data/pdc2025/ground_truth"
artifacts_dir     = "artifacts"
manifest_bus_path = "tmp/dmanifest_bus"
```

---

## [wasm]

WebAssembly host call amortization settings.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `batch_size` | usize | `10` | Any positive integer | Number of traces batched per host call to amortize crossing overhead. |
| `max_pages` | usize | `16` | Any positive integer | Maximum WebAssembly memory pages (1 page = 64 KiB). Total addressable WASM memory = `max_pages * 65536` bytes. |

```toml
[wasm]
batch_size = 10
max_pages  = 16
```

---

## [automl]

HDIT AutoML hyperparameter search for the PDC 2025 pipeline. Disabled by default.

| Key | Type | Default | Valid values | Description |
|---|---|---|---|---|
| `enabled` | bool | `false` | `true`, `false` | Enable AutoML hyperparameter search. When `false`, the HDIT fusion block is skipped entirely. |
| `strategy` | String | `"random"` | `"random"`, `"grid"` | Search strategy. `"random"` samples `budget` trials at random. `"grid"` exhaustively enumerates the space. |
| `budget` | usize | `20` | Any positive integer (> 0) | Number of trials per log for `"random"` strategy. Must be > 0 when `enabled = true`. |
| `seed` | u64 | `42` | Any u64 | Base random seed for reproducible search. |
| `successive_halving` | bool | `false` | `true`, `false` | Enable TPOT2-style 2-rung successive halving. Rung-0 scores all signals on a subsample; only the top fraction advance to rung-1 on full data. |
| `sh_subsample` | f64 | `0.2` | `0.0`–`1.0` | Fraction of traces used for rung-0 scoring. Only applies when `successive_halving = true`. |
| `sh_promotion_ratio` | f64 | `3.0` | Any positive f64 | Keep top `1/ratio` candidates after rung-0. `3.0` retains the top third. Only applies when `successive_halving = true`. |

> **BANNED VALUE**
>
> `strategy = "ensemble_only"` — **panics at startup** in the `pdc2025` binary with the message:
> ```
> AutoML config lie: strategy="ensemble_only" has been removed.
> ```
> This value was removed because it was a structural no-op: `combinatorial_ensemble` + `score_vs_in_lang` is a supremum operation that absorbs all trial variation, making every trial produce identical scores. Use `"random"` or `"grid"` instead.

```toml
[automl]
enabled             = false
strategy            = "random"
budget              = 20
seed                = 42
successive_halving  = false
sh_subsample        = 0.2
sh_promotion_ratio  = 3.0
```

---

## Complete minimal production config

A safe baseline combining all sections with conservative production values:

```toml
[meta]
version     = "2026.04.18"
environment = "production"
identity    = "dteam-prod-01"

[kernel]
tier              = "K256"
alignment         = 8
determinism       = "strict"
allocation_policy = "zero_heap"

[autonomic]
mode           = "guarded"
sampling_rate  = 100
integrity_hash = "fnv1a_64"

[autonomic.guards]
risk_threshold       = "Low"
min_health_threshold = 0.85
max_cycle_latency_ms = 50
repair_authority     = "senior_engineer"

[autonomic.policy]
profile      = "strict_conformance"
mdl_penalty  = 0.05
human_weight = 0.8

[rl]
algorithm         = "DoubleQLearning"
learning_rate     = 0.08
discount_factor   = 0.95
exploration_rate  = 0.2
exploration_decay = 0.999
reward_weights    = { fitness = 0.6, soundness = 0.2, simplicity = 0.1, latency = 0.1 }

[discovery]
max_training_epochs        = 100
fitness_stopping_threshold = 0.995
strategy                   = "incremental"
drift_window               = 1000

[paths]
training_logs_dir = "data/pdc2025/training_logs"
test_logs_dir     = "data/pdc2025/test_logs"
ground_truth_dir  = "data/pdc2025/ground_truth"
artifacts_dir     = "artifacts"
manifest_bus_path = "tmp/dmanifest_bus"

[wasm]
batch_size = 10
max_pages  = 16

[automl]
enabled             = false
strategy            = "random"
budget              = 20
seed                = 42
successive_halving  = false
sh_subsample        = 0.2
sh_promotion_ratio  = 3.0
```
