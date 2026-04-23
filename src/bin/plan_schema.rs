//! plan_schema — emit the JSON Schema 2020-12 for AutomlPlan artifacts.
//!
//! Usage:
//!   cargo run --bin plan_schema                       # print schema to stdout
//!   cargo run --bin plan_schema -- --out=schema.json  # write to file
//!   cargo run --bin plan_schema -- --validate=plan.json  # validate a plan against the schema
//!
//! Exit codes:
//!   0 — schema printed / file written / plan valid
//!   1 — validation failed (plan does not conform)
//!   2 — I/O or parse error

use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::{json, Value};

fn parse_args() -> (Option<PathBuf>, Option<PathBuf>) {
    let mut out: Option<PathBuf> = None;
    let mut validate: Option<PathBuf> = None;
    for raw in std::env::args().skip(1) {
        if let Some(v) = raw.strip_prefix("--out=") {
            out = Some(PathBuf::from(v));
        } else if let Some(v) = raw.strip_prefix("--validate=") {
            validate = Some(PathBuf::from(v));
        }
    }
    (out, validate)
}

/// The canonical JSON Schema 2020-12 for an AutomlPlan artifact.
/// Derived from the 17-field invariant specification and the anti-lie doctrine.
fn automl_plan_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://dteam.dev/schemas/automl-plan.json",
        "title": "AutomlPlan",
        "description": "Artifact produced by dteam pdc2025 binary for one event log. All accounting fields must satisfy the anti-lie identity: selected.len() + signals_rejected_correlation + signals_rejected_no_gain == signals_evaluated.",
        "type": "object",
        "required": [
            "log",
            "log_idx",
            "n_traces",
            "fusion",
            "selected",
            "tiers",
            "plan_accuracy_vs_anchor",
            "plan_accuracy_vs_gt",
            "anchor_vs_gt",
            "oracle_signal",
            "oracle_vs_gt",
            "oracle_gap",
            "per_signal_gt_accuracy",
            "total_timing_us",
            "signals_evaluated",
            "signals_rejected_correlation",
            "signals_rejected_no_gain",
            "accounting_balanced",
            "pareto_front"
        ],
        "properties": {
            "log": {
                "type": "string",
                "description": "Stem identifier of the event log (e.g. 'pdc2025_000000')."
            },
            "log_idx": {
                "type": "integer",
                "minimum": 0,
                "description": "Zero-based index of the log in the evaluation run."
            },
            "n_traces": {
                "type": "integer",
                "minimum": 1,
                "description": "Number of traces in the log."
            },
            "fusion": {
                "type": "string",
                "description": "Fusion strategy name (e.g. 'tf_idf', 'h_inlang_fill', 'weighted_vote')."
            },
            "selected": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Signal names selected by the HDIT greedy orthogonal algorithm."
            },
            "tiers": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["signal", "tier", "timing_us"],
                    "properties": {
                        "signal": { "type": "string" },
                        "tier": {
                            "type": "string",
                            "enum": ["T0", "T1", "T2", "Warm"],
                            "description": "Deployment tier: T0≤100µs, T1≤2ms, T2≤100ms, Warm>100ms."
                        },
                        "timing_us": { "type": "integer", "minimum": 0 }
                    }
                },
                "description": "Per-signal tier classification for the selected signals."
            },
            "plan_accuracy_vs_anchor": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Accuracy of the chosen plan measured against the anchor labels."
            },
            "plan_accuracy_vs_gt": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Accuracy of the chosen plan measured against ground truth."
            },
            "anchor_vs_gt": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Accuracy of the anchor signal itself against ground truth."
            },
            "oracle_signal": {
                "type": "string",
                "description": "The signal with highest accuracy vs GT across all evaluated candidates."
            },
            "oracle_vs_gt": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Oracle signal accuracy against ground truth."
            },
            "oracle_gap": {
                "type": "number",
                "description": "plan_accuracy_vs_gt - oracle_vs_gt. Negative means plan underperforms oracle. Must equal plan_accuracy_vs_gt - oracle_vs_gt within 1e-6."
            },
            "per_signal_gt_accuracy": {
                "type": "object",
                "additionalProperties": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0
                },
                "description": "Map of signal_name → accuracy_vs_gt for all evaluated signals."
            },
            "total_timing_us": {
                "type": "integer",
                "minimum": 0,
                "description": "Total inference latency in microseconds for the chosen plan."
            },
            "signals_evaluated": {
                "type": "integer",
                "minimum": 0,
                "description": "Count of signals passed through the full evaluation pipeline."
            },
            "signals_rejected_correlation": {
                "type": "integer",
                "minimum": 0,
                "description": "Signals rejected because Pearson r ≥ 0.95 with an already-selected signal."
            },
            "signals_rejected_no_gain": {
                "type": "integer",
                "minimum": 0,
                "description": "Signals rejected because marginal accuracy gain < 0.001."
            },
            "accounting_balanced": {
                "type": "boolean",
                "description": "Anti-lie invariant: true iff selected.len() + rejected_corr + rejected_gain == signals_evaluated."
            },
            "pareto_front": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["signals", "total_timing_us", "accuracy_vs_anchor", "chosen"],
                    "properties": {
                        "signals": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "total_timing_us": { "type": "integer", "minimum": 0 },
                        "accuracy_vs_anchor": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "chosen": {
                            "type": "boolean",
                            "description": "Exactly one candidate in the front must have chosen=true."
                        }
                    }
                },
                "description": "Non-dominated signal combinations on the accuracy/timing Pareto front. Invariant: exactly one candidate has chosen=true."
            }
        },
        "additionalProperties": true
    })
}

// ── Minimal structural validator (no external crate required) ─────────────────
// Checks required fields are present and the accounting identity holds.

#[derive(Debug)]
struct ValidationError(String);

fn validate_plan(v: &Value) -> Vec<ValidationError> {
    let mut errors: Vec<ValidationError> = vec![];

    // Required fields
    let required = [
        "log",
        "log_idx",
        "n_traces",
        "fusion",
        "selected",
        "tiers",
        "plan_accuracy_vs_anchor",
        "plan_accuracy_vs_gt",
        "anchor_vs_gt",
        "oracle_signal",
        "oracle_vs_gt",
        "oracle_gap",
        "per_signal_gt_accuracy",
        "total_timing_us",
        "signals_evaluated",
        "signals_rejected_correlation",
        "signals_rejected_no_gain",
        "accounting_balanced",
        "pareto_front",
    ];
    for field in &required {
        if v.get(field).is_none() {
            errors.push(ValidationError(format!(
                "missing required field: '{}'",
                field
            )));
        }
    }

    // Accounting identity
    if let (Some(eval), Some(sel_arr), Some(rej_corr), Some(rej_gain)) = (
        v.get("signals_evaluated").and_then(|x| x.as_u64()),
        v.get("selected").and_then(|x| x.as_array()),
        v.get("signals_rejected_correlation")
            .and_then(|x| x.as_u64()),
        v.get("signals_rejected_no_gain").and_then(|x| x.as_u64()),
    ) {
        let sum = sel_arr.len() as u64 + rej_corr + rej_gain;
        if sum != eval {
            errors.push(ValidationError(format!(
                "accounting identity broken: selected({}) + rej_corr({}) + rej_gain({}) = {} ≠ signals_evaluated({})",
                sel_arr.len(), rej_corr, rej_gain, sum, eval
            )));
        }
    }

    // accounting_balanced must be true
    if let Some(bal) = v.get("accounting_balanced").and_then(|x| x.as_bool()) {
        if !bal {
            errors.push(ValidationError(
                "accounting_balanced is false — anti-lie invariant violated".to_string(),
            ));
        }
    }

    // oracle_gap must equal plan_accuracy_vs_gt - oracle_vs_gt
    if let (Some(gap), Some(plan_acc), Some(oracle_acc)) = (
        v.get("oracle_gap").and_then(|x| x.as_f64()),
        v.get("plan_accuracy_vs_gt").and_then(|x| x.as_f64()),
        v.get("oracle_vs_gt").and_then(|x| x.as_f64()),
    ) {
        let expected = plan_acc - oracle_acc;
        if (gap - expected).abs() > 1e-6 {
            errors.push(ValidationError(format!(
                "oracle_gap {:.6} ≠ plan_accuracy_vs_gt({:.6}) - oracle_vs_gt({:.6}) = {:.6}",
                gap, plan_acc, oracle_acc, expected
            )));
        }
    }

    // Pareto front: exactly one chosen
    if let Some(front) = v.get("pareto_front").and_then(|x| x.as_array()) {
        let chosen_count = front
            .iter()
            .filter(|c| c.get("chosen").and_then(|x| x.as_bool()).unwrap_or(false))
            .count();
        if chosen_count != 1 {
            errors.push(ValidationError(format!(
                "pareto_front has {} chosen candidates; exactly 1 required",
                chosen_count
            )));
        }
    }

    // Range checks
    for field in &[
        "plan_accuracy_vs_gt",
        "plan_accuracy_vs_anchor",
        "anchor_vs_gt",
        "oracle_vs_gt",
    ] {
        if let Some(v) = v.get(field).and_then(|x| x.as_f64()) {
            if !(0.0..=1.0).contains(&v) {
                errors.push(ValidationError(format!(
                    "{} = {:.4} is outside [0, 1]",
                    field, v
                )));
            }
        }
    }

    errors
}

fn main() -> ExitCode {
    let (out_path, validate_path) = parse_args();
    let schema = automl_plan_schema();
    let schema_str = serde_json::to_string_pretty(&schema).expect("schema serialization failed");

    if let Some(path) = validate_path {
        // Validate mode
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                return ExitCode::from(2);
            }
        };
        let v: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("JSON parse error in {}: {}", path.display(), e);
                return ExitCode::from(2);
            }
        };
        let errors = validate_plan(&v);
        if errors.is_empty() {
            println!(
                "\x1b[32m✓\x1b[0m {} conforms to AutomlPlan schema",
                path.display()
            );
            ExitCode::from(0)
        } else {
            eprintln!(
                "\x1b[31m✗\x1b[0m {} has {} schema violation(s):",
                path.display(),
                errors.len()
            );
            for e in &errors {
                eprintln!("  • {}", e.0);
            }
            ExitCode::from(1)
        }
    } else if let Some(path) = out_path {
        // Write mode
        match std::fs::write(&path, &schema_str) {
            Ok(()) => {
                println!("Schema written to {}", path.display());
                ExitCode::from(0)
            }
            Err(e) => {
                eprintln!("Error writing {}: {}", path.display(), e);
                ExitCode::from(2)
            }
        }
    } else {
        // Print to stdout
        println!("{}", schema_str);
        ExitCode::from(0)
    }
}
