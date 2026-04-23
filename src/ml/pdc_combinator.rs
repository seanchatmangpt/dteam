//! Full combinatorial wiring for PDC 2025.
//!
//! Pools supervised, unsupervised, and synthetic classifier outputs with all
//! conformance score signals and runs exhaustive/greedy ensemble search,
//! optionally guided by the 6-bit process-parameter configuration encoded in
//! the log filename.

use crate::conformance::bitmask_replay::{classify_exact, replay_log, NetBitmask64};
use crate::ml::pdc_ensemble::{
    best_bool_score_pair, calibrate_to_target, combinatorial_ensemble, full_combinatorial,
    score, vote_fractions,
};
use crate::ml::pdc_features::{extract_log_features, pseudo_labels};
use crate::ml::pdc_supervised::{run_supervised, to_named_list as sup_named};
use crate::ml::pdc_unsupervised::{run_unsupervised, to_named_list as unsup_named};
use crate::ml::synthetic_trainer::classify_with_synthetic;
use crate::models::{AttributeValue, EventLog};

// ── Parameter config ──────────────────────────────────────────────────────────

/// 6-bit PDC 2025 process-parameter configuration decoded from a log filename
/// like `pdc2025_010101.xes`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Pdc2025Config {
    pub dependent_tasks: bool,
    pub loops: u8,            // 0 = none, 1 = simple, 2 = complex
    pub or_constructs: bool,
    pub routing: bool,
    pub optional_tasks: bool,
    pub duplicate_tasks: bool,
}

impl Pdc2025Config {
    /// Parse from filename.  Returns `None` if the 6-digit suffix is absent or
    /// contains non-digit characters.
    pub fn from_log_name(name: &str) -> Option<Self> {
        let base = name.trim_end_matches(".xes");
        let suffix = base.rsplit('_').next()?;
        if suffix.len() != 6 {
            return None;
        }
        let d: Vec<u8> = suffix
            .chars()
            .map(|c| c.to_digit(10).map(|x| x as u8))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            dependent_tasks: d[0] != 0,
            loops: d[1],
            or_constructs: d[2] != 0,
            routing: d[3] != 0,
            optional_tasks: d[4] != 0,
            duplicate_tasks: d[5] != 0,
        })
    }

    /// Composite complexity score in `[0.0, 1.0]`.
    pub fn complexity_score(&self) -> f64 {
        let mut s = 0.0_f64;
        if self.dependent_tasks { s += 0.20; }
        s += match self.loops { 0 => 0.0, 1 => 0.15, _ => 0.25 };
        if self.or_constructs  { s += 0.20; }
        if self.routing        { s += 0.10; }
        if self.optional_tasks { s += 0.15; }
        if self.duplicate_tasks{ s += 0.10; }
        s
    }

    pub fn has_complex_loops(&self) -> bool { self.loops == 2 }
}

// ── Output type ───────────────────────────────────────────────────────────────

/// One solution from the combinatorial search, ranked by anchor score.
#[derive(Debug, Clone)]
pub struct CombinatorResult {
    /// Binary predictions (exactly `n_target` positives).
    pub predictions: Vec<bool>,
    /// Score on anchor pseudo-labels (recall + precision penalty).
    pub score: f64,
    /// Human-readable label identifying which sub-strategy produced this.
    pub strategy_name: String,
    /// Number of classifier outputs pooled into this result.
    pub n_classifiers: usize,
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Run the full combinatorial wiring.
///
/// 1. Extract 7+vocab features (fitness, in_language, norm_len, norm_unique,
///    is_perfect, missing_norm, remaining_norm, BoW).
/// 2. Pool supervised (11) + unsupervised (6) + synthetic (1 ensemble) + baseline
///    classify_exact → up to 19 bool predictions.
/// 3. Build 4 score signals (fitness, is_perfect, 1-missing_norm, 1-remaining_norm).
/// 4. Run `full_combinatorial`, `best_bool_score_pair`, `combinatorial_ensemble`,
///    and optional parameter-aware routing.
/// 5. Return results sorted by score descending.
///
/// * `log_name` — optional filename used to decode the 6-bit parameter config.
pub fn run_combinator(
    log: &EventLog,
    net: &NetBitmask64,
    n_target: usize,
    log_name: Option<&str>,
) -> Vec<CombinatorResult> {
    // ── 1. Feature extraction + conformance signals ───────────────────────────
    let (features, in_lang, fitness) = extract_log_features(log, net);

    let replay = replay_log(net, log);
    let is_perfect_scores: Vec<f64> =
        replay.iter().map(|r| if r.is_perfect() { 1.0 } else { 0.0 }).collect();
    let missing_norm: Vec<f64> = replay
        .iter()
        .map(|r| {
            let d = (r.consumed + r.missing) as f64;
            if d == 0.0 { 0.0 } else { r.missing as f64 / d }
        })
        .collect();
    let remaining_norm: Vec<f64> = replay
        .iter()
        .map(|r| {
            let d = (r.produced + r.remaining) as f64;
            if d == 0.0 { 0.0 } else { r.remaining as f64 / d }
        })
        .collect();

    let anchor = &in_lang;

    // ── 2. Parameter-aware config ─────────────────────────────────────────────
    let config = log_name.and_then(Pdc2025Config::from_log_name);
    let complexity = config.map(|c| c.complexity_score()).unwrap_or(0.5);
    let n_synthetic = if complexity < 0.3 { 500 } else if complexity < 0.6 { 1_000 } else { 2_000 };

    // ── 3. Build classifier pool ──────────────────────────────────────────────
    // Pseudo-labels for supervised (bool) and unsupervised (Option<bool>)
    let pseudo_opt = pseudo_labels(&in_lang, &fitness);
    let pseudo_bool: Vec<bool> = pseudo_opt.iter().map(|p| p.unwrap_or(false)).collect();

    let sup = run_supervised(&features, &pseudo_bool);
    let unsup = run_unsupervised(&features, &pseudo_opt, &fitness, n_target);

    let act_seqs: Vec<Vec<String>> = log
        .traces
        .iter()
        .map(|t| {
            t.events
                .iter()
                .filter_map(|e| {
                    e.attributes.iter().find(|a| a.key == "concept:name").and_then(|a| {
                        if let AttributeValue::String(s) = &a.value {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .collect();
    let synth = classify_with_synthetic(net, &act_seqs, n_synthetic, n_target);

    // ── 4. Collect all bool predictions ──────────────────────────────────────
    let mut bool_preds: Vec<Vec<bool>> = Vec::new();
    bool_preds.push(classify_exact(net, log, n_target));       // baseline
    for (_, v) in sup_named(&sup)   { bool_preds.push(v); }   // +11
    for (_, v) in unsup_named(&unsup) { bool_preds.push(v); } // +6
    bool_preds.push(synth.ensemble.clone());                   // +1  → total ≤19

    // ── 5. Score signals (higher = more positive, bounded [0,1]) ─────────────
    let score_signals: Vec<Vec<f64>> = vec![
        fitness.clone(),
        is_perfect_scores.clone(),
        missing_norm.iter().map(|v| 1.0 - v).collect(),
        remaining_norm.iter().map(|v| 1.0 - v).collect(),
    ];

    // ── 6. Combinatorial search ───────────────────────────────────────────────
    let mut results: Vec<CombinatorResult> = Vec::new();

    // 6a. Full combinatorial (bool + score → exploits all 2^(k+m) combos)
    {
        let k = bool_preds.len();
        let preds = full_combinatorial(&bool_preds, &score_signals, anchor, n_target);
        let s = score(&preds, anchor, n_target);
        results.push(CombinatorResult {
            predictions: preds,
            score: s,
            strategy_name: "full_combinatorial".into(),
            n_classifiers: k,
        });
    }

    // 6b. Best bool+score pair (fastest; O(k·m·n log n))
    {
        let preds = best_bool_score_pair(&bool_preds, &score_signals, anchor, n_target);
        let s = score(&preds, anchor, n_target);
        results.push(CombinatorResult {
            predictions: preds,
            score: s,
            strategy_name: "best_pair".into(),
            n_classifiers: 2,
        });
    }

    // 6c. Combinatorial ensemble on bool predictions only
    {
        let k = bool_preds.len();
        let preds = combinatorial_ensemble(&bool_preds, anchor, n_target);
        let s = score(&preds, anchor, n_target);
        results.push(CombinatorResult {
            predictions: preds,
            score: s,
            strategy_name: "combo_bool_only".into(),
            n_classifiers: k,
        });
    }

    // 6d. Parameter-aware routing (only when filename config is available)
    if let Some(cfg) = config {
        let preds = if cfg.has_complex_loops() {
            // Complex-loop nets: synthetic classifier captures loop structure best
            let fracs = vote_fractions(&bool_preds[bool_preds.len().saturating_sub(4)..]);
            calibrate_to_target(&synth.ensemble, &fracs, n_target)
        } else {
            // Simple / no-loop nets: fitness + is_perfect are reliable
            best_bool_score_pair(&bool_preds[..3.min(bool_preds.len())], &score_signals[..2], anchor, n_target)
        };
        let s = score(&preds, anchor, n_target);
        results.push(CombinatorResult {
            predictions: preds,
            score: s,
            strategy_name: format!("param_aware(loops={},complexity={:.2})", cfg.loops, complexity),
            n_classifiers: 3,
        });
    }

    // Sort best first
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let c = Pdc2025Config::from_log_name("pdc2025_010101.xes").unwrap();
        assert!(!c.dependent_tasks);
        assert_eq!(c.loops, 1);
        assert!(c.or_constructs);
        assert!(!c.routing);
        assert!(c.optional_tasks);
        assert!(!c.duplicate_tasks);

        let c2 = Pdc2025Config::from_log_name("pdc2025_121111.xes").unwrap();
        assert!(c2.dependent_tasks);
        assert_eq!(c2.loops, 2);
        assert!(c2.has_complex_loops());
    }

    #[test]
    fn test_config_complexity_bounds() {
        let min = Pdc2025Config::from_log_name("pdc2025_000000.xes").unwrap();
        let max = Pdc2025Config::from_log_name("pdc2025_121111.xes").unwrap();
        assert!(min.complexity_score() < max.complexity_score());
        assert!(min.complexity_score() >= 0.0);
        assert!(max.complexity_score() <= 1.0);
    }

    #[test]
    fn test_config_none_on_bad_name() {
        assert!(Pdc2025Config::from_log_name("unknown.xes").is_none());
        assert!(Pdc2025Config::from_log_name("pdc2025_1234567.xes").is_none());
    }
}
