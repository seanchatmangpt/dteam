# Run and Interpret the Definition of Done Report

The Definition of Done (DoD) binary verifies that the dteam codebase compiles, passes tests,
and has valid pipeline artifacts. Run it before any commit that touches the AutoML pipeline,
HDIT signal pool, or verifier code.

Source: `src/bin/dod.rs`. Registered tasks: `Makefile.toml` lines 158-166.

---

## Commands

```bash
cargo make dod        # release build — use for pre-commit and pre-merge
cargo make dod-dev    # debug build — faster compilation, use during development
```

`dod` runs `cargo run --bin dod --release --quiet`.
`dod-dev` runs `cargo run --bin dod --quiet` (no `--release`).

Use `dod-dev` when iterating on verifier logic or artifact formats. Use `dod` for the final
check before merging.

---

## Exit codes

| Code | Meaning | When triggered |
|---|---|---|
| `0` | PASS | All 4 phases passed with no failures. |
| `1` | SOFT FAIL | Build and tests OK but pipeline artifacts have issues. |
| `2` | HARD FAIL | Compilation failed or tests broken. |

Hard fail always takes priority: if Phase 1 or Phase 2 fails, the exit code is `2` even if
Phase 3 or Phase 4 would also fail.

---

## Phase-by-phase output

### Phase 1: Compilation

Runs `cargo check --quiet`. A failure here is always a hard fail.

| Symbol | Meaning |
|---|---|
| `✓ cargo check` | Crate compiles cleanly. |
| `✗ cargo check` | Compilation error. The first `error[...]` line is printed. Fix before anything else. |

### Phase 2: Tests

Runs `cargo test --lib`. A failure here is always a hard fail.

| Symbol | Meaning |
|---|---|
| `✓ cargo test --lib` | All library unit tests passed. Count of passed tests shown. |
| `✗ cargo test --lib` | One or more tests failed. The first `FAILED` or `failures` line is shown. |

### Phase 3: Pipeline artifacts (AutoML plans)

Scans `artifacts/pdc2025/automl_plans/*.json` and `artifacts/pdc2025/automl_summary.json`.
Uses `AutomlPipelineVerifier` from `src/agentic/ralph/verifier.rs`. Failures here are soft.

| Symbol | Check | Meaning |
|---|---|---|
| `✓` | config has no banned ensemble_only strategy | `dteam.toml` does not set `strategy = "ensemble_only"`. |
| `✗` | config has no banned ensemble_only strategy | Change to `"random"` or `"grid"`. |
| `⚠` | no plan JSON files found | AutoML has not been run. Run the pipeline with `[automl].enabled = true`. |
| `✓` | plan JSON invariants (N plans) | All plan files have balanced accounting, one chosen Pareto candidate, all required fields. |
| `✗` | plan JSON invariants | Per-plan failures are listed beneath with bullet points. |
| `✓` | automl_summary.json exists | Summary file was written by the binary. |
| `✗` | automl_summary.json exists | Binary did not complete. Check for panics in the pipeline run. |
| `✓` | summary accounting_balanced_global=true | Aggregate accounting identity holds. |
| `✗` | summary accounting_balanced_global=true | Invariant violated — aggregate lie detected. |

### Phase 4: DX / QoL artifacts

Scans `artifacts/pdc2025/` for quality-of-life artifacts. Uses `DxQolVerifier`. Failures
here are soft.

| Symbol | Check | Meaning |
|---|---|---|
| `✓` | strategy_accuracies.json | All 20 strategy keys present, all values in `[0,1]`. |
| `✗` | strategy_accuracies.json | Missing keys or out-of-range values. Run `cargo make pdc`. |
| `✓` | run_metadata.json | Has `git_commit`, `timestamp`, `n_logs_total`, `n_logs_processed`, `n_failed_xes`, `n_failed_gt`. |
| `✗` | run_metadata.json | Missing or incomplete. Run `cargo make pdc`. |
| `✓` | classified output XES files | At least one `.xes` written to `artifacts/pdc2025/`. |
| `✗` | classified output XES files | No output XES found. Run `cargo make pdc`. |
| `✓` | skip rate ≤ 10% | `(n_failed_xes + n_failed_gt) / n_logs_total` is within threshold. |
| `✗` | skip rate ≤ 10% | Too many logs skipped. Check XES parsing and ground truth availability. |
| `✓` | best_per_log dominates all individual strategies | `best_per_log >= max(all other strategies)`. |
| `✗` | best_per_log dominates all individual strategies | Invariant violated — lie detected in strategy comparison. |

---

## How to fix each failure type

### Hard fail — Phase 1 (compilation)

```bash
cargo make check    # see full error detail
```

Fix the compile error, then re-run `cargo make dod`.

### Hard fail — Phase 2 (tests)

```bash
cargo test --lib -- --nocapture 2>&1 | grep -A 10 FAILED
```

Fix the failing test. Do not use `#[ignore]` to suppress a test that was previously passing.

### Soft fail — Phase 3 (pipeline artifacts missing)

The pipeline has not been run or ran with `[automl].enabled = false`.

```toml
# dteam.toml
[automl]
enabled = true
strategy = "random"
budget = 20
```

Then:

```bash
cargo run --bin pdc2025 --release
cargo make dod
```

### Soft fail — Phase 3 (plan JSON invariants)

Inspect the failing plan file:

```bash
cat artifacts/pdc2025/automl_plans/<stem>.json | python3 -m json.tool
```

Common causes:
- `accounting_balanced = false`: a signal was neither selected nor rejected, indicating a
  bug in `run_hdit_automl`. Check that the candidate list length equals `n_evaluated`.
- `pareto_front` has 0 or 2+ `chosen=true`: Pareto selection logic returned an ambiguous
  result. Check `src/ml/hdit_automl.rs`.
- `oracle_gap` lie: the stored gap was cached from a stale run. Delete
  `artifacts/pdc2025/automl_plans/` and re-run.

### Soft fail — Phase 4 (DX/QoL)

```bash
cargo make pdc    # regenerates all DX/QoL artifacts
cargo make dod    # re-check
```

---

## Daily workflow

1. Make changes.
2. `cargo make dod-dev` — fast check during iteration.
3. Before committing: `cargo make dod` — release build, full verification.
4. If `dod` exits 0, commit. If it exits non-zero, fix before committing.

The DoD binary is not a substitute for `cargo make ci`. Run `cargo make ci` for the full
lint + format + test suite. `cargo make dod` is a focused check on pipeline artifact health.
