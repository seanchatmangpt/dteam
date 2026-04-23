# How to Run the Doctor Diagnostic

The `doctor` command performs an epistemic diagnostic on your AutoML pipeline artifacts. It answers a sharper question than a build check: **is your plan slow, redundant, biased, or lying?**

---

## Prerequisites

- A working `cargo make pdc` run that produced plan JSON files in `artifacts/pdc2025/automl_plans/`
- `cargo make ci` passing (doctor is a runtime artifact check, not a compile check)

---

## 1. Basic run

```bash
cargo make doctor
```

Reads all `.json` files in `artifacts/pdc2025/automl_plans/`, runs all five pathology checks, and exits:

- `0` — healthy
- `1` — soft fail (SLOW / REDUNDANT / SATURATED / STALE — suboptimal, not lying)
- `2` — fatal (LYING — an invariant is violated)

---

## 2. Target-tier check

```bash
cargo make doctor-target TARGET=T1
```

Or directly:

```bash
cargo run --bin doctor --release -- --target=T1
```

Reports which plans exceed the T1 budget (>2ms) and, where possible, suggests the cheapest Pareto-front candidate that would bring the plan under budget. Valid tier names: `T0` (≤100µs), `T1` (≤2ms), `T2` (≤100ms), `Warm` (>100ms).

---

## 3. JSON output (for CI integration)

```bash
cargo make doctor-json
```

Emits a structured JSON report to stdout. Useful in CI pipelines that need machine-readable output:

```json
{
  "healthy": false,
  "exit_code": 1,
  "pathologies": [
    { "kind": "SATURATED", "severity": "Warn", "count": 28 },
    { "kind": "STALE",     "severity": "Info", "count": 1  }
  ],
  "tier_distribution": { "T0": 0, "T1": 28, "T2": 0, "Warm": 0 }
}
```

---

## 4. Single-plan inspection

```bash
cargo run --bin doctor --release -- --plan=artifacts/pdc2025/automl_plans/pdc2025_000000.json
```

Runs all checks on a single plan file and shows full detail for that log. Useful for debugging a specific log after a regression.

---

## 5. Custom plans directory

```bash
cargo run --bin doctor --release -- --plans-dir=my/custom/plans/
```

Useful when comparing multiple experiment runs stored in different directories.

---

## Pathology Reference

| Pathology | Severity | Meaning | Remediation |
|-----------|----------|---------|-------------|
| **LYING** | Fatal (exit 2) | Anti-lie invariant violated: `accounting_balanced=false`, accounting identity broken, `oracle_gap ≠ plan_acc - oracle_acc`, or Pareto front has ≠1 chosen candidate | Fix the pdc2025 binary; never suppress this |
| **SLOW** | Warn (exit 1) | Selected plan's `total_timing_us` exceeds Warm threshold (>100ms) | Use `--target=T2` check; consider cheaper Pareto-front alternative |
| **SATURATED** | Warn (exit 1) | All selected signals belong to a single family (e.g., all `conformance`) — monoculture, not orthogonal selection | Ensure diverse signal families are in the evaluation pool; check anchor choice |
| **REDUNDANT** | Info (exit 1) | >40% of evaluated signals were rejected for high correlation with an already-selected signal | Expected for conformance family; Info only if it's the dominant rejection reason |
| **STALE** | Info (exit 1) | `run_metadata.json` is missing or the artifact commit hash doesn't match the current HEAD | Run `cargo make pdc` to regenerate artifacts |
| **UNDEPLOYABLE** | Warn (exit 1) | Plan exceeds the requested `--target` tier | Shown only when `--target` flag is set |

---

## Deployment Tier Contract

| Tier | Max Latency | Deployment Target |
|------|-------------|-------------------|
| T0 | ≤100µs | Browser/WASM, embedded, hard real-time |
| T1 | ≤2ms | Edge/CDN (Cloudflare Workers, Fastly), mobile on-device |
| T2 | ≤100ms | Fog/serverless (Lambda, Cloud Run), IoT gateway |
| Warm | >100ms | Cloud (EC2, GKE), batch, offline analytics |

Use `--target=T1` in CI to enforce edge-deployability. The tier of a plan is determined by its `total_timing_us` field in the plan JSON.

---

## Related Commands

| Command | Purpose |
|---------|---------|
| `cargo make dod` | Definition-of-Done: compile + tests + artifact invariants |
| `cargo make plan-diff DIR_A=v1 DIR_B=v2` | Compare accuracy/tiers between two runs |
| `cargo make plan-schema` | Print the JSON Schema for AutomlPlan artifacts |
| `cargo make plan-report` | Generate a standalone HTML diagnostic report |
