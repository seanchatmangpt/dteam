# DoD Invariant Reference

All invariant checks performed by `cargo make dod` / `cargo make dod-dev`.

Sources:
- `src/bin/dod.rs` — orchestration, exit codes, output formatting
- `src/agentic/ralph/verifier.rs` — `AutomlPipelineVerifier` and `DxQolVerifier`

---

## Phase 1: Compilation

| Check | Command | Failure trigger | Exit code on failure |
|---|---|---|---|
| `cargo check` | `cargo check --quiet` | Non-zero exit from `cargo check` | `2` (hard) |

Phase 1 failure immediately sets `hard_fail = true`. All subsequent phases still run and
print output, but the final exit code is `2` regardless of their results.

---

## Phase 2: Tests

| Check | Command | Failure trigger | Exit code on failure |
|---|---|---|---|
| `cargo test --lib` | `cargo test --lib` | Non-zero exit from `cargo test --lib` | `2` (hard) |

Test count is parsed from stdout lines matching `"test result: ok"`. On failure, the first
line containing `"FAILED"` or `"failures"` is printed.

---

## Phase 3: AutoML Pipeline Artifacts

Verifier: `AutomlPipelineVerifier` (`src/agentic/ralph/verifier.rs` lines 77-156).
Artifact directory: `artifacts/pdc2025/automl_plans/`.
Summary file: `artifacts/pdc2025/automl_summary.json`.
Config file: `dteam.toml`.

All Phase 3 failures are **soft** (contribute to exit code `1`, not `2`).

| # | Check | Type | Failure trigger |
|---|---|---|---|
| 1 | Config has no banned `ensemble_only` strategy | Soft | `dteam.toml` line contains `strategy` and `ensemble_only` simultaneously |
| 2 | Plan JSON invariants — all required fields present | Soft | Any of the 17 required JSON fields is absent from a plan file |
| 3 | Plan JSON invariants — `accounting_balanced == true` | Soft | `accounting_balanced` field is not JSON boolean `true` |
| 4 | Plan JSON invariants — accounting identity holds | Soft | `selected.len() + rejected_corr + rejected_gain != signals_evaluated` |
| 5 | Plan JSON invariants — Pareto front has exactly 1 chosen | Soft | `pareto_front` array has 0 or 2+ entries with `"chosen": true` |
| 6 | Plan JSON invariants — `oracle_gap` is not a lie | Soft | `|oracle_gap - (plan_accuracy_vs_gt - oracle_vs_gt)| > 1e-6` |
| 7 | `automl_summary.json` exists | Soft | File `artifacts/pdc2025/automl_summary.json` does not exist |
| 8 | `summary.accounting_balanced_global == true` | Soft | `accounting_balanced_global` field in summary is not `true` |

The 17 required fields validated per plan file (check 2):
`log`, `log_idx`, `fusion`, `selected`, `tiers`, `plan_accuracy_vs_anchor`,
`plan_accuracy_vs_gt`, `anchor_vs_gt`, `oracle_signal`, `oracle_vs_gt`, `oracle_gap`,
`per_signal_gt_accuracy`, `total_timing_us`, `signals_evaluated`,
`signals_rejected_correlation`, `signals_rejected_no_gain`, `accounting_balanced`,
`pareto_front`.

If no plan JSON files are found, a warning (`⚠`) is printed and the phase does not fail
(the binary simply has not been run with `[automl].enabled = true`).

---

## Phase 4: DX / QoL Artifacts

Verifier: `DxQolVerifier` (`src/agentic/ralph/verifier.rs` lines 335-490).
Artifact directory: `artifacts/pdc2025/`.
Maximum skip rate: `0.10` (10%).

All Phase 4 failures are **soft** (contribute to exit code `1`, not `2`).

| # | Check | Type | Failure trigger |
|---|---|---|---|
| 1 | `strategy_accuracies.json` exists | Soft | File absent from `artifacts/pdc2025/` |
| 2 | All 20 strategy keys present | Soft | Any of the 20 expected strategy keys missing from `strategies` object |
| 3 | All strategy values in `[0, 1]` | Soft | Any strategy accuracy value outside `[0.0, 1.0]` |
| 4 | `best_per_log` dominates all individual strategies | Soft | `best_per_log < max(all other strategies)` by more than 1e-9 |
| 5 | `run_metadata.json` exists | Soft | File absent from `artifacts/pdc2025/` |
| 6 | `run_metadata.json` has required fields | Soft | Any of `git_commit`, `timestamp`, `n_logs_total`, `n_logs_processed`, `n_failed_xes`, `n_failed_gt` is absent |
| 7 | Skip rate <= 10% | Soft | `(n_failed_xes + n_failed_gt) / n_logs_total > 0.10` |
| 8 | `git_commit` is not `"unknown"` | Soft | `git_commit` field equals the string `"unknown"` |
| 9 | At least one classified `.xes` output | Soft | No file with `.xes` extension found in `artifacts/pdc2025/` |
| 10 | `strategy_accuracies.json` has `n_logs > 0` | Soft | `n_logs` field is 0 or absent |
| 11 | `run_metadata.json` parse succeeds | Soft | File exists but is not valid JSON |

The 20 expected strategy keys (from `EXPECTED_STRATEGIES` at `src/agentic/ralph/verifier.rs`
lines 281-302):
`f`, `g`, `h`, `hdc`, `automl`, `rl_automl`, `combinator`, `sup_trained`, `automl_hyper`,
`borda`, `rrf`, `weighted`, `prec_weighted`, `stacked`, `full_combo`, `best_pair`, `combo`,
`vote500`, `s_ensemble`, `best_per_log`.

---

## Hard vs Soft Fail

| Category | Phases | Exit code | Meaning |
|---|---|---|---|
| Hard fail | Phase 1, Phase 2 | `2` | Compilation or tests broken. Pipeline cannot produce valid artifacts. Fix before anything else. |
| Soft fail | Phase 3, Phase 4 | `1` | Build and tests are healthy but pipeline artifacts are missing, stale, or contain invariant violations. Run `cargo run --bin pdc2025 --release` then re-check. |
| Pass | All phases | `0` | All checks green. |

Hard fail takes priority: if Phase 1 or Phase 2 fails, the exit code is `2` even if Phase 3
or Phase 4 would also fail independently. The hard/soft distinction reflects the principle
that broken compilation and broken tests are the most severe defects — no artifact check is
meaningful when the binary itself cannot be produced.

Phase 3 and Phase 4 failures are soft because they reflect the state of pipeline run
artifacts, not the state of the code. Artifacts are regenerated by running the pipeline;
code defects require fixes and commits.
