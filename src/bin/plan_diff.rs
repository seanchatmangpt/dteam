//! plan_diff — compare two AutomlPlan JSON files (or two plan directories).
//!
//! Answers: "did accuracy, tiers, or signals regress between runs?"
//!
//! # Exit codes
//!   0 — no regressions
//!   1 — at least one accuracy regression (plan_accuracy_vs_gt decreased)
//!   2 — parse/IO error
//!
//! # Usage
//!   cargo run --bin plan_diff -- --before=a.json --after=b.json
//!   cargo run --bin plan_diff -- --dir-a=artifacts/v1/ --dir-b=artifacts/v2/
//!   cargo run --bin plan_diff -- --before=a.json --after=b.json --json

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{json, Value};

// ── ANSI color codes ──────────────────────────────────────────────────────────
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// ── CLI args ──────────────────────────────────────────────────────────────────
struct CliArgs {
    before: Option<PathBuf>,
    after: Option<PathBuf>,
    dir_a: Option<PathBuf>,
    dir_b: Option<PathBuf>,
    json: bool,
}

fn parse_args() -> CliArgs {
    let mut args = CliArgs {
        before: None,
        after: None,
        dir_a: None,
        dir_b: None,
        json: false,
    };
    for raw in std::env::args().skip(1) {
        if raw == "--json" {
            args.json = true;
        } else if let Some(v) = raw.strip_prefix("--before=") {
            args.before = Some(PathBuf::from(v));
        } else if let Some(v) = raw.strip_prefix("--after=") {
            args.after = Some(PathBuf::from(v));
        } else if let Some(v) = raw.strip_prefix("--dir-a=") {
            args.dir_a = Some(PathBuf::from(v));
        } else if let Some(v) = raw.strip_prefix("--dir-b=") {
            args.dir_b = Some(PathBuf::from(v));
        }
    }
    args
}

// ── Plan snapshot ─────────────────────────────────────────────────────────────
#[derive(Debug)]
struct PlanSnap {
    log: String,
    accuracy_vs_gt: f64,
    oracle_gap: f64,
    total_timing_us: u64,
    selected: Vec<String>,
    accounting_balanced: bool,
    chosen_pareto_count: usize,
}

fn load_snap(path: &Path) -> Result<PlanSnap, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&content)
        .map_err(|e| format!("{}: JSON parse: {e}", path.display()))?;

    let log = v
        .get("log")
        .and_then(|x| x.as_str())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        })
        .to_string();

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

    Ok(PlanSnap {
        log,
        accuracy_vs_gt: v
            .get("plan_accuracy_vs_gt")
            .and_then(|x| x.as_f64())
            .unwrap_or(f64::NAN),
        oracle_gap: v
            .get("oracle_gap")
            .and_then(|x| x.as_f64())
            .unwrap_or(f64::NAN),
        total_timing_us: v
            .get("total_timing_us")
            .and_then(|x| x.as_u64())
            .unwrap_or(0),
        selected,
        accounting_balanced: v
            .get("accounting_balanced")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        chosen_pareto_count,
    })
}

fn load_dir(dir: &Path) -> BTreeMap<String, PlanSnap> {
    let mut map = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect();
        paths.sort();
        for p in &paths {
            match load_snap(p) {
                Ok(snap) => {
                    map.insert(snap.log.clone(), snap);
                }
                Err(e) => eprintln!("{}⚠ skip {}: {}{}", YELLOW, p.display(), e, RESET),
            }
        }
    }
    map
}

// ── Diff result ───────────────────────────────────────────────────────────────
#[derive(Debug)]
struct PlanDiff {
    log: String,
    acc_before: f64,
    acc_after: f64,
    /// positive = improvement
    acc_delta: f64,
    timing_before: u64,
    timing_after: u64,
    signals_added: Vec<String>,
    signals_removed: Vec<String>,
    accounting_before: bool,
    accounting_after: bool,
    chosen_before: usize,
    chosen_after: usize,
    oracle_gap_before: f64,
    oracle_gap_after: f64,
}

impl PlanDiff {
    fn is_regression(&self) -> bool {
        self.acc_delta < -1e-6
    }

    fn is_accounting_broken(&self) -> bool {
        self.accounting_before && !self.accounting_after
    }
}

fn diff_pair(before: &PlanSnap, after: &PlanSnap) -> PlanDiff {
    let before_set: std::collections::HashSet<&str> =
        before.selected.iter().map(|s| s.as_str()).collect();
    let after_set: std::collections::HashSet<&str> =
        after.selected.iter().map(|s| s.as_str()).collect();

    let signals_added: Vec<String> = after_set
        .difference(&before_set)
        .map(|s| s.to_string())
        .collect();
    let signals_removed: Vec<String> = before_set
        .difference(&after_set)
        .map(|s| s.to_string())
        .collect();

    PlanDiff {
        log: before.log.clone(),
        acc_before: before.accuracy_vs_gt,
        acc_after: after.accuracy_vs_gt,
        acc_delta: after.accuracy_vs_gt - before.accuracy_vs_gt,
        timing_before: before.total_timing_us,
        timing_after: after.total_timing_us,
        signals_added,
        signals_removed,
        accounting_before: before.accounting_balanced,
        accounting_after: after.accounting_balanced,
        chosen_before: before.chosen_pareto_count,
        chosen_after: after.chosen_pareto_count,
        oracle_gap_before: before.oracle_gap,
        oracle_gap_after: after.oracle_gap,
    }
}

// ── Display ───────────────────────────────────────────────────────────────────
fn fmt_us(us: u64) -> String {
    if us < 1_000 {
        format!("{}µs", us)
    } else if us < 1_000_000 {
        format!("{:.1}ms", us as f64 / 1_000.0)
    } else {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    }
}

fn fmt_acc(v: f64) -> String {
    if v.is_nan() {
        "N/A".to_string()
    } else {
        format!("{:.1}%", v * 100.0)
    }
}

fn fmt_delta(d: f64) -> String {
    if d.is_nan() {
        "N/A".to_string()
    } else if d >= 0.0 {
        format!("{}{:+.1}%{}", GREEN, d * 100.0, RESET)
    } else {
        format!("{}{:+.1}%{}", RED, d * 100.0, RESET)
    }
}

fn section(title: &str) {
    println!();
    println!(
        "{}{}── {} ─────────────────────────────────────────────────{}",
        BOLD, DIM, title, RESET
    );
}

fn print_human(diffs: &[PlanDiff], only_a: &[String], only_b: &[String]) -> bool {
    let regressions: Vec<&PlanDiff> = diffs.iter().filter(|d| d.is_regression()).collect();
    let improvements: Vec<&PlanDiff> = diffs
        .iter()
        .filter(|d| !d.is_regression() && d.acc_delta.abs() > 1e-6)
        .collect();

    // ── Summary header ────────────────────────────────────────────────────────
    println!(
        "{}{}┌─────────────────────────────────────────────────────────────┐{}",
        BOLD, DIM, RESET
    );
    println!(
        "{}{}│ dteam plan_diff — AutoML plan comparison report            │{}",
        BOLD, DIM, RESET
    );
    println!(
        "{}{}└─────────────────────────────────────────────────────────────┘{}",
        BOLD, DIM, RESET
    );

    // ── Accuracy Changes ──────────────────────────────────────────────────────
    section("Accuracy Changes");
    for d in diffs {
        let marker = if d.is_regression() {
            format!("{}↓{}", RED, RESET)
        } else if d.acc_delta > 1e-6 {
            format!("{}↑{}", GREEN, RESET)
        } else {
            format!("{}≡{}", DIM, RESET)
        };
        println!(
            "  {} {:<30} {} → {} {}",
            marker,
            d.log,
            fmt_acc(d.acc_before),
            fmt_acc(d.acc_after),
            fmt_delta(d.acc_delta)
        );
    }
    if !only_a.is_empty() {
        for log in only_a {
            println!("  {}−{} {:<30} only in before (removed)", RED, RESET, log);
        }
    }
    if !only_b.is_empty() {
        for log in only_b {
            println!("  {}+{} {:<30} only in after (new)", GREEN, RESET, log);
        }
    }

    // ── Signal Changes ────────────────────────────────────────────────────────
    let changed_signals: Vec<&PlanDiff> = diffs
        .iter()
        .filter(|d| !d.signals_added.is_empty() || !d.signals_removed.is_empty())
        .collect();
    if !changed_signals.is_empty() {
        section("Signal Changes");
        for d in &changed_signals {
            println!("  {} {}{}{}:", DIM, BOLD, d.log, RESET);
            for s in &d.signals_added {
                println!("    {}+ added:  {}{}", GREEN, s, RESET);
            }
            for s in &d.signals_removed {
                println!("    {}- removed: {}{}", RED, s, RESET);
            }
        }
    }

    // ── Tier Changes ──────────────────────────────────────────────────────────
    let tier_changes: Vec<&PlanDiff> = diffs
        .iter()
        .filter(|d| tier_label(d.timing_before) != tier_label(d.timing_after))
        .collect();
    if !tier_changes.is_empty() {
        section("Tier Changes");
        for d in &tier_changes {
            let before_tier = tier_label(d.timing_before);
            let after_tier = tier_label(d.timing_after);
            let arrow = if timing_rank(after_tier) < timing_rank(before_tier) {
                format!("{}↑ faster{}", GREEN, RESET)
            } else {
                format!("{}↓ slower{}", YELLOW, RESET)
            };
            println!(
                "  {} {:<30} {} ({}) → {} ({}) {}",
                DIM,
                d.log,
                before_tier,
                fmt_us(d.timing_before),
                after_tier,
                fmt_us(d.timing_after),
                arrow
            );
        }
    }

    // ── Accounting ───────────────────────────────────────────────────────────
    let broken_accounting: Vec<&PlanDiff> =
        diffs.iter().filter(|d| d.is_accounting_broken()).collect();
    if !broken_accounting.is_empty() {
        section("Accounting Regressions");
        for d in &broken_accounting {
            println!(
                "  {}✗{} {} — accounting_balanced flipped true → false",
                RED, RESET, d.log
            );
        }
    }

    // ── Verdict ──────────────────────────────────────────────────────────────
    section("Verdict");
    let total = diffs.len();
    println!("  Plans compared: {}", total);
    println!(
        "  Regressions:    {}{}{}",
        if regressions.is_empty() { GREEN } else { RED },
        regressions.len(),
        RESET
    );
    println!(
        "  Improvements:   {}{}{}",
        if improvements.is_empty() { DIM } else { GREEN },
        improvements.len(),
        RESET
    );
    println!(
        "  Unchanged:      {}{}{}",
        DIM,
        total - regressions.len() - improvements.len(),
        RESET
    );

    if regressions.is_empty() && broken_accounting.is_empty() {
        println!("  {}{}✓ No regressions.{}", BOLD, GREEN, RESET);
        false
    } else {
        println!(
            "  {}{}✗ Regressions detected — review before merge.{}",
            BOLD, RED, RESET
        );
        true
    }
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

fn timing_rank(label: &str) -> u8 {
    match label {
        "T0" => 0,
        "T1" => 1,
        "T2" => 2,
        _ => 3,
    }
}

fn print_json(diffs: &[PlanDiff], only_a: &[String], only_b: &[String]) {
    let regressions: usize = diffs.iter().filter(|d| d.is_regression()).count();
    let mut plan_diffs = vec![];
    for d in diffs {
        plan_diffs.push(json!({
            "log": d.log,
            "accuracy_before": d.acc_before,
            "accuracy_after": d.acc_after,
            "accuracy_delta": d.acc_delta,
            "timing_before_us": d.timing_before,
            "timing_after_us": d.timing_after,
            "tier_before": tier_label(d.timing_before),
            "tier_after": tier_label(d.timing_after),
            "signals_added": d.signals_added,
            "signals_removed": d.signals_removed,
            "accounting_before": d.accounting_before,
            "accounting_after": d.accounting_after,
            "chosen_pareto_before": d.chosen_before,
            "chosen_pareto_after": d.chosen_after,
            "oracle_gap_before": d.oracle_gap_before,
            "oracle_gap_after": d.oracle_gap_after,
            "is_regression": d.is_regression(),
        }));
    }
    let report = json!({
        "total_compared": diffs.len(),
        "regressions": regressions,
        "only_in_before": only_a,
        "only_in_after": only_b,
        "plans": plan_diffs,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

// ── Entry point ───────────────────────────────────────────────────────────────
fn main() -> ExitCode {
    let args = parse_args();

    let (before_map, after_map) = match (args.before, args.after, args.dir_a, args.dir_b) {
        (Some(b), Some(a), None, None) => {
            // Single-file comparison
            let before_snap = match load_snap(&b) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}Error loading before: {}{}", RED, e, RESET);
                    return ExitCode::from(2);
                }
            };
            let after_snap = match load_snap(&a) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}Error loading after: {}{}", RED, e, RESET);
                    return ExitCode::from(2);
                }
            };
            let mut bm: BTreeMap<String, PlanSnap> = BTreeMap::new();
            let mut am: BTreeMap<String, PlanSnap> = BTreeMap::new();
            bm.insert(before_snap.log.clone(), before_snap);
            am.insert(after_snap.log.clone(), after_snap);
            (bm, am)
        }
        (None, None, Some(da), Some(db)) => (load_dir(&da), load_dir(&db)),
        _ => {
            eprintln!(
                "Usage: plan_diff --before=a.json --after=b.json\n       plan_diff --dir-a=artifacts/v1/ --dir-b=artifacts/v2/"
            );
            return ExitCode::from(2);
        }
    };

    let mut diffs: Vec<PlanDiff> = vec![];
    let mut only_a: Vec<String> = vec![];
    let mut only_b: Vec<String> = vec![];

    for (log, before) in &before_map {
        match after_map.get(log) {
            Some(after) => diffs.push(diff_pair(before, after)),
            None => only_a.push(log.clone()),
        }
    }
    for log in after_map.keys() {
        if !before_map.contains_key(log) {
            only_b.push(log.clone());
        }
    }
    diffs.sort_by(|a, b| {
        a.acc_delta
            .partial_cmp(&b.acc_delta)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let has_regression = if args.json {
        print_json(&diffs, &only_a, &only_b);
        diffs.iter().any(|d| d.is_regression())
    } else {
        print_human(&diffs, &only_a, &only_b)
    };

    if has_regression {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
