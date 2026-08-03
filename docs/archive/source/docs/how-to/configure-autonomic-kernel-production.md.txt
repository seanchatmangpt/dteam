# How to Configure the Autonomic Kernel for Production

## Prerequisites

`dteam.toml` exists at the repo root and `cargo make check` passes cleanly.

---

## 1. Set the operational mode

Open `dteam.toml` and locate the `[autonomic]` section. Set `mode` to `"guarded"` for production:

```toml
[autonomic]
mode = "guarded"
sampling_rate = 100        # online ingestion interval, milliseconds
integrity_hash = "fnv1a_64"
```

**Mode comparison:**

| Mode | Behaviour |
|---|---|
| `"guarded"` | Proposes actions; `accept()` enforces risk threshold and policy guards. Correct for production. |
| `"autonomous"` | Executes without human confirmation. Use only in fully-supervised, low-risk pipelines. |
| `"recommend"` | Proposes only. No execution. Suitable for read-only monitoring or staging reviews. |

---

## 2. Tune the guards

Set `[autonomic.guards]` to match your environment's risk tolerance. For a strict production environment:

```toml
[autonomic.guards]
risk_threshold        = "Low"
min_health_threshold  = 0.85
max_cycle_latency_ms  = 50
repair_authority      = "senior_engineer"
```

**Fields:**

- `risk_threshold` — Maximum `ActionRisk` level the kernel will execute automatically. Values: `"Low"`, `"Medium"`, `"High"`. Setting `"Low"` means only `ActionRisk::Low` actions execute; Medium and High are blocked.
- `min_health_threshold` — `process_health` must be at or above this value before any actions are proposed. Set to `0.85` for strict environments (the default is `0.7`).
- `max_cycle_latency_ms` — Informational ceiling. Cycles that exceed this value should be logged and investigated.
- `repair_authority` — Annotation only; does not gate code execution. Used in manifests and audit trails.

> **NOTE — hardcoded conformance floor:** `run_cycle()` in `src/autonomic/kernel.rs` contains this check:
> ```rust
> if state.process_health < config.autonomic.guards.min_health_threshold
>     || state.conformance_score < 0.75
> {
>     return Vec::new();
> }
> ```
> The `conformance_score < 0.75` threshold is **hardcoded** and cannot be changed via `dteam.toml`. If conformance drops below 0.75, the kernel halts and returns no results regardless of any configuration value. Run Deep Diagnostics (`RUST_LOG=info cargo make run`) to investigate the root cause.

---

## 3. Choose a policy profile

Set `[autonomic.policy]` to control how `accept()` filters proposed actions:

```toml
[autonomic.policy]
profile       = "strict_conformance"
mdl_penalty   = 0.05
human_weight  = 0.8
```

**Profile behaviour:**

- `"strict_conformance"` — `accept()` unconditionally rejects any action where `action.risk_profile >= ActionRisk::High`. Use this in production. Also imposes MDL complexity penalty via `mdl_penalty`.
- `"permissive"` — High-risk actions are not automatically rejected by policy. Appropriate for staging environments only.
- `"audit_only"` — Combine with `mode = "recommend"` for a fully read-only kernel that records decisions without executing them.

**`mdl_penalty`** — Adds a Minimum Description Length penalty to actions that increase model complexity. Raise toward `0.1` to favour simpler models.

**`human_weight`** — Weight applied to human feedback during the `adapt()` phase. Range: `0.0`–`1.0`.

---

## 4. Verify the configuration

Run the diagnostic suite:

```sh
cargo make doctor
```

What to look for in the output:

- `cargo check --all-targets` must complete with zero errors and zero warnings.
- If a `doctor` example binary is present, its output will appear first; any `FAIL` lines indicate a broken invariant.

To observe `accept()` decisions in real time, run the autonomic loop with info logging:

```sh
RUST_LOG=info cargo make run
```

Each cycle logs which actions were proposed, whether they passed the risk threshold, and whether `accept()` returned true or false. Lines containing `MANIFEST:` confirm successful execution with the integrity hash.

---

## 5. Quick reference: risk_threshold × profile → ActionRisk::High accepted?

| `risk_threshold` | `profile` | `ActionRisk::High` accepted? |
|---|---|---|
| `"Low"` | `"strict_conformance"` | No — blocked by both policy and threshold |
| `"Low"` | `"permissive"` | No — blocked by threshold |
| `"High"` | `"permissive"` | Yes — both policy and threshold allow it |
