//! plan_report — generate a standalone HTML diagnostic report from AutoML plan artifacts.
//!
//! Produces a single self-contained HTML file with:
//!   - Deployment tier matrix (T0/T1/T2/Warm distribution across logs)
//!   - Per-plan summary table (accuracy, timing, signals, tier)
//!   - Anti-lie audit table (accounting_balanced, oracle_gap, pareto invariant)
//!   - Signal frequency chart (text-based bar using Unicode)
//!
//! Usage:
//!   cargo run --bin plan_report                                   # reads artifacts/pdc2025/automl_plans/
//!   cargo run --bin plan_report -- --plans-dir=path/to/plans      # custom dir
//!   cargo run --bin plan_report -- --out=report.html              # custom output
//!
//! Exit codes:
//!   0 — report written successfully
//!   1 — no plans found
//!   2 — I/O error

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

// ── CLI args ──────────────────────────────────────────────────────────────────
struct CliArgs {
    plans_dir: PathBuf,
    out: PathBuf,
}

fn parse_args() -> CliArgs {
    let mut plans_dir = PathBuf::from("artifacts/pdc2025/automl_plans");
    let mut out = PathBuf::from("artifacts/pdc2025/report.html");
    for raw in std::env::args().skip(1) {
        if let Some(v) = raw.strip_prefix("--plans-dir=") {
            plans_dir = PathBuf::from(v);
        } else if let Some(v) = raw.strip_prefix("--out=") {
            out = PathBuf::from(v);
        }
    }
    CliArgs { plans_dir, out }
}

// ── Plan data ─────────────────────────────────────────────────────────────────
#[derive(Debug)]
struct Plan {
    log: String,
    #[allow(dead_code)]
    log_idx: usize,
    #[allow(dead_code)]
    fusion: String,
    selected: Vec<String>,
    total_timing_us: u64,
    plan_accuracy_vs_gt: f64,
    #[allow(dead_code)]
    plan_accuracy_vs_anchor: f64,
    #[allow(dead_code)]
    anchor_vs_gt: f64,
    #[allow(dead_code)]
    oracle_signal: String,
    oracle_vs_gt: f64,
    oracle_gap: f64,
    signals_evaluated: usize,
    signals_rejected_correlation: usize,
    signals_rejected_no_gain: usize,
    accounting_balanced: bool,
    chosen_pareto_count: usize,
}

impl Plan {
    fn tier(&self) -> &'static str {
        tier_label(self.total_timing_us)
    }

    fn accounting_ok(&self) -> bool {
        let sum =
            self.selected.len() + self.signals_rejected_correlation + self.signals_rejected_no_gain;
        self.accounting_balanced && sum == self.signals_evaluated
    }

    fn pareto_ok(&self) -> bool {
        self.chosen_pareto_count == 1
    }

    fn oracle_gap_ok(&self) -> bool {
        if self.oracle_gap.is_nan() {
            return false;
        }
        let expected = self.plan_accuracy_vs_gt - self.oracle_vs_gt;
        (self.oracle_gap - expected).abs() <= 1e-6
    }
}

fn load_plan(path: &Path) -> Option<Plan> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;

    let selected: Vec<String> = v
        .get("selected")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let chosen_pareto_count = v
        .get("pareto_front")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter(|c| c.get("chosen").and_then(|x| x.as_bool()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    Some(Plan {
        log: v
            .get("log")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        log_idx: v.get("log_idx").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
        fusion: v
            .get("fusion")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        selected,
        total_timing_us: v
            .get("total_timing_us")
            .and_then(|x| x.as_u64())
            .unwrap_or(0),
        plan_accuracy_vs_gt: v
            .get("plan_accuracy_vs_gt")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0),
        plan_accuracy_vs_anchor: v
            .get("plan_accuracy_vs_anchor")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0),
        anchor_vs_gt: v
            .get("anchor_vs_gt")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0),
        oracle_signal: v
            .get("oracle_signal")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        oracle_vs_gt: v
            .get("oracle_vs_gt")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0),
        oracle_gap: v
            .get("oracle_gap")
            .and_then(|x| x.as_f64())
            .unwrap_or(f64::NAN),
        signals_evaluated: v
            .get("signals_evaluated")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as usize,
        signals_rejected_correlation: v
            .get("signals_rejected_correlation")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as usize,
        signals_rejected_no_gain: v
            .get("signals_rejected_no_gain")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as usize,
        accounting_balanced: v
            .get("accounting_balanced")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        chosen_pareto_count,
    })
}

fn load_plans(dir: &Path) -> Vec<Plan> {
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect(),
        Err(_) => return vec![],
    };
    paths.sort();
    paths.iter().filter_map(|p| load_plan(p)).collect()
}

fn tier_label(us: u64) -> &'static str {
    if us <= 100 {
        "T0"
    } else if us <= 2_000 {
        "T1"
    } else if us <= 100_000 {
        "T2"
    } else {
        "Warm"
    }
}

fn fmt_us(us: u64) -> String {
    if us < 1_000 {
        format!("{}µs", us)
    } else if us < 1_000_000 {
        format!("{:.1}ms", us as f64 / 1_000.0)
    } else {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    }
}

fn pct(v: f64) -> String {
    if v.is_nan() {
        "N/A".to_string()
    } else {
        format!("{:.1}%", v * 100.0)
    }
}

// ── HTML generation ───────────────────────────────────────────────────────────

fn tier_color(tier: &str) -> &'static str {
    match tier {
        "T0" => "#2ecc71",
        "T1" => "#27ae60",
        "T2" => "#f39c12",
        _ => "#e74c3c",
    }
}

fn bool_badge(ok: bool) -> String {
    if ok {
        r#"<span class="badge pass">✓</span>"#.to_string()
    } else {
        r#"<span class="badge fail">✗</span>"#.to_string()
    }
}

fn signal_frequency_chart(plans: &[Plan]) -> String {
    let mut freq: BTreeMap<String, usize> = BTreeMap::new();
    for p in plans {
        for s in &p.selected {
            *freq.entry(s.clone()).or_insert(0) += 1;
        }
    }
    if freq.is_empty() {
        return "<p>No signals selected.</p>".to_string();
    }
    let max_count = *freq.values().max().unwrap_or(&1);
    let mut rows: Vec<(String, usize)> = freq.into_iter().collect();
    rows.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

    let mut html = String::from(
        r#"<table class="freq-table"><thead><tr><th>Signal</th><th>Plans</th><th>Frequency</th></tr></thead><tbody>"#,
    );
    for (sig, count) in &rows {
        let bar_width = (count * 200 / max_count).max(2);
        html.push_str(&format!(
            r#"<tr><td class="mono">{}</td><td>{}</td><td><div class="bar" style="width:{}px"></div></td></tr>"#,
            html_escape(sig), count, bar_width
        ));
    }
    html.push_str("</tbody></table>");
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn generate_report(plans: &[Plan]) -> String {
    let t0 = plans.iter().filter(|p| p.tier() == "T0").count();
    let t1 = plans.iter().filter(|p| p.tier() == "T1").count();
    let t2 = plans.iter().filter(|p| p.tier() == "T2").count();
    let warm = plans.iter().filter(|p| p.tier() == "Warm").count();
    let total = plans.len();

    let avg_acc = if total > 0 {
        plans.iter().map(|p| p.plan_accuracy_vs_gt).sum::<f64>() / total as f64
    } else {
        0.0
    };

    let all_accounting_ok = plans.iter().all(|p| p.accounting_ok());
    let all_pareto_ok = plans.iter().all(|p| p.pareto_ok());

    // Per-plan rows
    let mut plan_rows = String::new();
    for p in plans {
        let tier = p.tier();
        plan_rows.push_str(&format!(
            r#"<tr>
              <td class="mono">{}</td>
              <td><span class="tier-badge" style="background:{}">{}</span></td>
              <td class="mono">{}</td>
              <td>{}</td>
              <td>{}</td>
              <td>{}</td>
              <td class="mono">{}</td>
            </tr>"#,
            html_escape(&p.log),
            tier_color(tier),
            tier,
            fmt_us(p.total_timing_us),
            pct(p.plan_accuracy_vs_gt),
            pct(p.oracle_vs_gt),
            pct(p.oracle_gap.abs()),
            html_escape(&p.selected.join(", "))
        ));
    }

    // Anti-lie audit rows
    let mut audit_rows = String::new();
    for p in plans {
        let oracle_gap_str = if p.oracle_gap.is_nan() {
            "N/A".to_string()
        } else {
            format!("{:.4}", p.oracle_gap)
        };
        audit_rows.push_str(&format!(
            r#"<tr>
              <td class="mono">{}</td>
              <td>{}</td>
              <td>{}</td>
              <td>{}</td>
              <td class="mono">{}</td>
            </tr>"#,
            html_escape(&p.log),
            bool_badge(p.accounting_ok()),
            bool_badge(p.pareto_ok()),
            bool_badge(p.oracle_gap_ok()),
            oracle_gap_str
        ));
    }

    let signal_chart = signal_frequency_chart(plans);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>dteam AutoML Plan Report</title>
  <style>
    :root {{
      --t0: #2ecc71; --t1: #27ae60; --t2: #f39c12; --warm: #e74c3c;
      --bg: #0f1117; --surface: #1a1d27; --border: #2d3047;
      --text: #e8e8f0; --dim: #7a7d9c; --pass: #2ecc71; --fail: #e74c3c;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{ font-family: -apple-system, 'Segoe UI', sans-serif; background: var(--bg); color: var(--text); padding: 2rem; line-height: 1.6; }}
    h1 {{ font-size: 1.6rem; font-weight: 700; margin-bottom: 0.25rem; }}
    h2 {{ font-size: 1.1rem; font-weight: 600; margin: 2rem 0 0.75rem; color: var(--dim); text-transform: uppercase; letter-spacing: 0.08em; border-bottom: 1px solid var(--border); padding-bottom: 0.4rem; }}
    p {{ color: var(--dim); font-size: 0.9rem; margin-bottom: 1rem; }}
    .meta {{ color: var(--dim); font-size: 0.85rem; margin-bottom: 2rem; }}
    .stat-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-bottom: 2rem; }}
    .stat {{ background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 1rem; text-align: center; }}
    .stat .value {{ font-size: 2rem; font-weight: 700; }}
    .stat .label {{ font-size: 0.8rem; color: var(--dim); text-transform: uppercase; letter-spacing: 0.06em; }}
    .tier-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.75rem; margin-bottom: 2rem; }}
    .tier-card {{ background: var(--surface); border-radius: 8px; padding: 1rem; border-top: 3px solid; text-align: center; }}
    .tier-card .count {{ font-size: 1.8rem; font-weight: 700; }}
    .tier-card .name {{ font-weight: 600; margin-bottom: 0.25rem; }}
    .tier-card .deploy {{ font-size: 0.75rem; color: var(--dim); }}
    table {{ width: 100%; border-collapse: collapse; font-size: 0.87rem; background: var(--surface); border-radius: 8px; overflow: hidden; }}
    th {{ background: #12151f; padding: 0.6rem 0.8rem; text-align: left; font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--dim); border-bottom: 1px solid var(--border); }}
    td {{ padding: 0.55rem 0.8rem; border-bottom: 1px solid var(--border); vertical-align: middle; }}
    tr:last-child td {{ border-bottom: none; }}
    tr:hover td {{ background: rgba(255,255,255,0.03); }}
    .mono {{ font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.82rem; }}
    .tier-badge {{ display: inline-block; padding: 0.15rem 0.5rem; border-radius: 4px; font-size: 0.75rem; font-weight: 700; color: #fff; }}
    .badge {{ display: inline-flex; align-items: center; justify-content: center; width: 1.4rem; height: 1.4rem; border-radius: 50%; font-size: 0.8rem; font-weight: 700; }}
    .badge.pass {{ background: rgba(46,204,113,0.2); color: var(--pass); }}
    .badge.fail {{ background: rgba(231,76,60,0.2); color: var(--fail); }}
    .summary-badges {{ display: flex; gap: 1rem; margin-bottom: 1rem; }}
    .summary-badge {{ padding: 0.4rem 1rem; border-radius: 20px; font-size: 0.82rem; font-weight: 600; }}
    .summary-badge.ok {{ background: rgba(46,204,113,0.15); color: var(--pass); border: 1px solid rgba(46,204,113,0.3); }}
    .summary-badge.fail {{ background: rgba(231,76,60,0.15); color: var(--fail); border: 1px solid rgba(231,76,60,0.3); }}
    .freq-table {{ margin-top: 0.5rem; }}
    .bar {{ height: 14px; background: linear-gradient(90deg, #4a9eff, #2d6db5); border-radius: 3px; min-width: 2px; }}
    .avg-acc {{ font-size: 1.1rem; font-weight: 600; color: #4a9eff; }}
  </style>
</head>
<body>
  <h1>dteam AutoML Plan Report</h1>
  <p class="meta">Generated {timestamp} · {total} plans</p>

  <div class="stat-grid">
    <div class="stat"><div class="value avg-acc">{avg_acc_pct}</div><div class="label">Avg Accuracy vs GT</div></div>
    <div class="stat"><div class="value" style="color:var(--pass)">{total}</div><div class="label">Plans Loaded</div></div>
    <div class="stat"><div class="value" style="color:{acc_color}">{acc_count}</div><div class="label">Accounting OK</div></div>
    <div class="stat"><div class="value" style="color:{pareto_color}">{pareto_count}</div><div class="label">Pareto OK</div></div>
  </div>

  <h2>Deployment Tier Distribution</h2>
  <div class="tier-grid">
    <div class="tier-card" style="border-color: var(--t0)">
      <div class="name" style="color:var(--t0)">T0</div>
      <div class="count" style="color:var(--t0)">{t0}</div>
      <div class="deploy">≤100µs · browser/WASM</div>
    </div>
    <div class="tier-card" style="border-color: var(--t1)">
      <div class="name" style="color:var(--t1)">T1</div>
      <div class="count" style="color:var(--t1)">{t1}</div>
      <div class="deploy">≤2ms · edge/CDN</div>
    </div>
    <div class="tier-card" style="border-color: var(--t2)">
      <div class="name" style="color:var(--t2)">T2</div>
      <div class="count" style="color:var(--t2)">{t2}</div>
      <div class="deploy">≤100ms · fog/serverless</div>
    </div>
    <div class="tier-card" style="border-color: var(--warm)">
      <div class="name" style="color:var(--warm)">Warm</div>
      <div class="count" style="color:var(--warm)">{warm}</div>
      <div class="deploy">&gt;100ms · cloud/batch</div>
    </div>
  </div>

  <h2>Per-Plan Summary</h2>
  <table>
    <thead>
      <tr>
        <th>Log</th><th>Tier</th><th>Latency</th><th>Plan Acc (GT)</th><th>Oracle Acc</th><th>Oracle Gap</th><th>Selected Signals</th>
      </tr>
    </thead>
    <tbody>{plan_rows}</tbody>
  </table>

  <h2>Anti-Lie Audit</h2>
  <div class="summary-badges">
    <span class="summary-badge {acc_cls}">Accounting: {acc_count}/{total}</span>
    <span class="summary-badge {pareto_cls}">Pareto: {pareto_count}/{total}</span>
  </div>
  <table>
    <thead>
      <tr><th>Log</th><th>Accounting</th><th>Pareto (1 chosen)</th><th>Oracle Gap</th><th>Gap Value</th></tr>
    </thead>
    <tbody>{audit_rows}</tbody>
  </table>

  <h2>Signal Selection Frequency</h2>
  <p>How often each signal appears in a chosen plan across all logs.</p>
  {signal_chart}
</body>
</html>"#,
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
        total = total,
        avg_acc_pct = pct(avg_acc),
        acc_count = plans.iter().filter(|p| p.accounting_ok()).count(),
        pareto_count = plans.iter().filter(|p| p.pareto_ok()).count(),
        acc_color = if all_accounting_ok {
            "#2ecc71"
        } else {
            "#e74c3c"
        },
        pareto_color = if all_pareto_ok { "#2ecc71" } else { "#e74c3c" },
        t0 = t0,
        t1 = t1,
        t2 = t2,
        warm = warm,
        plan_rows = plan_rows,
        audit_rows = audit_rows,
        signal_chart = signal_chart,
        acc_cls = if all_accounting_ok { "ok" } else { "fail" },
        pareto_cls = if all_pareto_ok { "ok" } else { "fail" },
    )
}

fn main() -> ExitCode {
    let args = parse_args();
    let plans = load_plans(&args.plans_dir);

    if plans.is_empty() {
        eprintln!(
            "No plan JSON files found in {}. Run `cargo make pdc` first.",
            args.plans_dir.display()
        );
        return ExitCode::from(1);
    }

    println!(
        "Loaded {} plans from {}",
        plans.len(),
        args.plans_dir.display()
    );

    let html = generate_report(&plans);

    // Ensure output directory exists
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create output directory: {e}");
                return ExitCode::from(2);
            }
        }
    }

    match std::fs::write(&args.out, &html) {
        Ok(()) => {
            println!("Report written to {}", args.out.display());
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("Failed to write report: {e}");
            ExitCode::from(2)
        }
    }
}
