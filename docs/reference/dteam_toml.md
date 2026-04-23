# dteam.toml Configuration Reference

Complete reference for all configuration sections and keys. Config is loaded by
`AutonomicConfig::load("dteam.toml")` (`src/config.rs` line 197). A missing file silently
returns `Default` values — no error is raised.

---

## [meta]

Rust struct: `MetaConfig` (`src/config.rs` lines 19-24)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `version` | string | `"2026.04.18"` | Any string | Schema/config version label. Used for audit and provenance. |
| `environment` | string | `"autonomous"` | Any string | Runtime environment label (e.g. `"autonomous"`, `"dev"`, `"ci"`). |
| `identity` | string | `"dteam-alpha-01"` | Any string | Instance identity string written to run metadata artifacts. |

---

## [kernel]

Rust struct: `KernelConfig` (`src/config.rs` lines 26-32)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `tier` | string | `"K256"` | `"K64"`, `"K128"`, `"K256"`, `"K512"`, `"K1024"` | K-Tier selection controlling the packed table word width used by `PackedKeyTable`. |
| `alignment` | usize | `8` | `8` (mandatory) | Memory alignment in bytes. Must be 8 for branchless mask calculus. |
| `determinism` | string | `"strict"` | `"strict"`, `"relaxed"`, `"non_deterministic"` | Determines replay ordering guarantees. `"strict"` enforces canonical ordering. |
| `allocation_policy` | string | `"zero_heap"` | `"zero_heap"`, `"stack_only"`, `"bounded_heap"` | Heap allocation policy. `"zero_heap"` enforces no hot-path heap allocation. |

---

## [autonomic]

Rust struct: `AutonomicSystemConfig` (`src/config.rs` lines 34-41)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `mode` | string | `"guarded"` | `"recommend"`, `"guarded"`, `"autonomous"` | Operational mode. `"guarded"` requires human sign-off above risk threshold. `"autonomous"` acts without approval. |
| `sampling_rate` | u64 | `100` | Any positive integer | Online ingestion sampling interval in milliseconds. |
| `integrity_hash` | string | `"fnv1a_64"` | `"sha256"`, `"blake3"`, `"fnv1a_64"` | Hash algorithm used for manifest integrity verification. |

---

## [autonomic.guards]

Rust struct: `GuardConfig` (`src/config.rs` lines 43-50)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `risk_threshold` | string | `"Low"` | `"Low"`, `"Medium"`, `"High"` | Maximum risk level for automatic action execution in autonomous mode. |
| `min_health_threshold` | f32 | `0.7` | `[0.0, 1.0]` | Minimum system health score required before autonomous intervention. |
| `max_cycle_latency_ms` | u64 | `50` | Any positive integer | Maximum allowed latency for a single autonomic cycle in milliseconds. |
| `repair_authority` | string | `"senior_engineer"` | Any string | Authority level label required for structural repair actions. |

---

## [autonomic.policy]

Rust struct: `PolicyConfig` (`src/config.rs` lines 52-57)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `profile` | string | `"strict_conformance"` | `"strict_conformance"`, `"permissive"`, `"audit_only"` | Compliance profile controlling how violations are handled. |
| `mdl_penalty` | f32 | `0.05` | `[0.0, 1.0]` | Penalty applied to the reward surface for actions that increase model complexity (MDL). |
| `human_weight` | f32 | `0.8` | `[0.0, 1.0]` | Weight given to human feedback in the adapt phase of the autonomic loop. |

---

## [rl]

Rust struct: `RlConfig` (`src/config.rs` lines 59-66)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `algorithm` | string | `"DoubleQLearning"` | `"QLearning"`, `"DoubleQLearning"`, `"SARSA"`, `"ExpectedSARSA"`, `"REINFORCE"` | RL algorithm used in `train_with_provenance`. |
| `learning_rate` | f32 | `0.08` | `(0.0, 1.0]` | Step size for Q-value updates (α). |
| `discount_factor` | f32 | `0.95` | `[0.0, 1.0]` | Future reward discount (γ). |
| `exploration_rate` | f32 | `0.2` | `[0.0, 1.0]` | Initial ε for ε-greedy exploration. |
| `exploration_decay` | f32 | `0.999` | `(0.0, 1.0]` | Multiplicative decay applied to `exploration_rate` each episode. |
| `reward_weights` | table | `{fitness=0.6, soundness=0.2, simplicity=0.1, latency=0.1}` | Keys must sum to 1.0 | Per-component weights for the composite reward surface. |

---

## [discovery]

Rust struct: `DiscoveryConfig` (`src/config.rs` lines 68-74)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `max_training_epochs` | usize | `100` | Any positive integer | Maximum RL episodes per discovery run. |
| `fitness_stopping_threshold` | f64 | `0.995` | `[0.0, 1.0]` | Early-stop threshold: if fitness exceeds this value, training halts. |
| `strategy` | string | `"incremental"` | `"incremental"`, `"batch"`, `"sliding_window"` | How event log data is presented to the RL loop. |
| `drift_window` | usize | `1000` | Any positive integer | Window size (in events) for concept drift detection. |

---

## [paths]

Rust struct: `PathConfig` (`src/config.rs` lines 76-83)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `training_logs_dir` | string | `"data/pdc2025/training_logs"` | Any valid path | Directory containing training XES event logs. |
| `test_logs_dir` | string | `"data/pdc2025/test_logs"` | Any valid path | Directory containing test XES event logs for classification. |
| `ground_truth_dir` | string | `"data/pdc2025/ground_truth"` | Any valid path | Directory containing ground truth label files. |
| `artifacts_dir` | string | `"artifacts"` | Any valid path | Root directory for all pipeline output artifacts. |
| `manifest_bus_path` | string | `"tmp/dmanifest_bus"` | Any valid path | Path for the manifest bus used by the autonomic kernel. |

---

## [wasm]

Rust struct: `WasmConfig` (`src/config.rs` lines 85-89)

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `batch_size` | usize | `10` | Any positive integer | Amortization batch size for WASM host calls. |
| `max_pages` | usize | `16` | Any positive integer | Maximum WASM memory pages (1 page = 64 KiB). |

---

## [automl]

Rust struct: `AutomlConfig` (`src/config.rs` lines 91-107). All keys have `#[serde(default)]`;
section may be omitted entirely.

| Key | Type | Default | Valid Values | Description |
|---|---|---|---|---|
| `enabled` | bool | `false` | `true`, `false` | Master switch for HDIT AutoML. When `false` all other keys are ignored. |
| `strategy` | string | `"random"` | `"random"`, `"grid"` | Search strategy. `"ensemble_only"` is banned and causes a startup panic. |
| `budget` | usize | `20` | Any positive integer (> 0 when enabled) | Number of RandomSearch trials per log. `0` causes a startup panic when enabled. |
| `seed` | u64 | `42` | Any u64 | Base seed for reproducible trial ordering. |
| `successive_halving` | bool | `false` | `true`, `false` | Enable TPOT2 two-rung evaluation. Rung-0 subsamples; top fraction advances. |
| `sh_subsample` | f64 | `0.2` | `(0.0, 1.0)` | Fraction of traces used for rung-0 scoring. Only read when `successive_halving = true`. |
| `sh_promotion_ratio` | f64 | `3.0` | `> 1.0` | Keep top `1/ratio` candidates after rung 0. Default `3.0` keeps the top third. |
