# Classify Process Traces with AutoML

**Learning goal:** Build two `SignalProfile` objects, run HDIT greedy signal selection, and read an `AutomlPlan` that tells you which signals to fuse and how accurately they classify traces.

By the end of this tutorial you will:
- Understand what a signal is in HDIT terms
- Build synthetic boolean signals without loading any XES files
- Interpret the `AutomlPlan` fields: `selected`, `fusion`, `plan_accuracy`, Pareto front

---

## Background: HDIT AutoML

HDIT (Hyperdimensional Information Theory) AutoML is not traditional hyperparameter tuning. It takes a pool of pre-computed _signal predictions_ — one `bool` per trace, saying "this signal classifies this trace as positive" — and selects the minimal orthogonal subset that maximises accuracy against a reference anchor.

The key insight: redundant signals add noise and latency without improving accuracy. HDIT greedily rejects any new signal whose correlation with an already-selected signal exceeds a threshold (0.95), and any signal whose marginal accuracy gain is below 0.1%.

`run_hdit_automl` then picks the cheapest fusion operator for the selected set:

| Set size | Fusion |
|---|---|
| 1 signal | `Single` (no fusion needed) |
| 2–4 signals | `WeightedVote` |
| 5+ signals | `BordaCount` |
| ≥3 with high accuracy variance | `Stack` (meta-learner) |

---

## Understanding `SignalProfile`

```rust
pub struct SignalProfile {
    pub name: String,
    pub predictions: Vec<bool>,   // one bool per trace
    pub accuracy_vs_anchor: f64,  // computed automatically by ::new()
    pub timing_us: u64,           // measured latency in microseconds
    pub tier: Tier,               // T0/T1/T2/Warm — derived from timing_us
}
```

The constructor `SignalProfile::new(name, predictions, anchor, timing_us)` computes `accuracy_vs_anchor` and `tier` for you.

**Compute tiers:**

| Tier | Timing | Meaning |
|---|---|---|
| `T0` | < 100 µs | Branchless kernel candidate |
| `T1` | 100 µs – 2 ms | Folded signature / small projection |
| `T2` | 2 ms – 100 ms | Wider vector |
| `Warm` | > 100 ms | Planning layer only |

---

## Step 1 — Create `examples/automl_classify.rs`

**File:** `examples/automl_classify.rs`

```rust
use dteam::ml::hdit_automl::{run_hdit_automl, FusionOp, SignalProfile};

fn main() {
    // 20 synthetic traces.
    // The anchor represents pseudo-ground-truth:
    // first 10 positive (conforming), last 10 negative (non-conforming).
    let n_traces = 20;
    let anchor: Vec<bool> = (0..n_traces).map(|i| i < 10).collect();

    // ── Signal 1: "baseline" — all-true classifier ───────────────────────────
    // Predicts every trace as positive.
    // Accuracy vs anchor = 10/20 = 0.50 (it gets the first 10 right, last 10 wrong).
    let baseline_preds: Vec<bool> = vec![true; n_traces];
    let signal_baseline = SignalProfile::new(
        "baseline_all_true",
        baseline_preds,
        &anchor,
        50, // 50 µs → T0 tier
    );

    // ── Signal 2: "alternating" — alternates true/false ─────────────────────
    // Predictions: true, false, true, false, ...
    // Correct on odd-indexed traces; accuracy depends on overlap with anchor.
    let alternating_preds: Vec<bool> = (0..n_traces).map(|i| i % 2 == 0).collect();
    let signal_alternating = SignalProfile::new(
        "alternating",
        alternating_preds,
        &anchor,
        800, // 800 µs → T1 tier
    );

    // ── Signal 3: "near-perfect" — matches anchor almost exactly ────────────
    // First 10 true, next 10 false — same as anchor.
    // Accuracy vs anchor = 1.00.
    let near_perfect_preds: Vec<bool> = (0..n_traces).map(|i| i < 10).collect();
    let signal_near_perfect = SignalProfile::new(
        "near_perfect",
        near_perfect_preds,
        &anchor,
        3_500, // 3.5 ms → T2 tier
    );

    println!("Signals before selection:");
    for sig in &[&signal_baseline, &signal_alternating, &signal_near_perfect] {
        println!(
            "  {:<25}  acc={:.3}  timing={:>6} µs  tier={:?}",
            sig.name, sig.accuracy_vs_anchor, sig.timing_us, sig.tier
        );
    }
    println!();

    let candidates = vec![signal_baseline, signal_alternating, signal_near_perfect];
    let n_target = 10; // calibrate output to 10 positives (matches anchor)

    let plan = run_hdit_automl(candidates, &anchor, n_target);

    // ── Print the plan ───────────────────────────────────────────────────────
    println!("AutoML plan:");
    println!("  Selected signals:     {:?}", plan.selected);
    println!("  Fusion operator:      {:?}", plan.fusion);
    println!("  Plan accuracy:        {:.3}", plan.plan_accuracy);
    println!("  Total timing:         {} µs", plan.total_timing_us);
    println!("  Signals evaluated:    {}", plan.signals_evaluated);
    println!("  Rejected (corr):      {}", plan.signals_rejected_correlation);
    println!("  Rejected (no gain):   {}", plan.signals_rejected_no_gain);

    println!();
    println!("Selected signal tiers:");
    for (name, tier) in &plan.tiers {
        println!("  {:<25}  tier={}", name, tier.label());
    }

    println!();
    println!("Pareto front ({} candidates):", plan.pareto_front.len());
    for cand in &plan.pareto_front {
        println!(
            "  signals={:<40}  acc={:.3}  complexity={}  timing={} µs  chosen={}",
            format!("{:?}", cand.signals),
            cand.accuracy_vs_anchor,
            cand.complexity,
            cand.total_timing_us,
            cand.chosen,
        );
    }

    // ── Interpret fusion ─────────────────────────────────────────────────────
    println!();
    match plan.fusion {
        FusionOp::Single => println!("Interpretation: single signal selected — no fusion needed."),
        FusionOp::WeightedVote => println!("Interpretation: 2–4 signals — accuracy-weighted vote applied."),
        FusionOp::BordaCount => println!("Interpretation: 5+ signals — Borda rank fusion applied."),
        FusionOp::Stack => println!("Interpretation: stacking ensemble — meta-learner applied."),
    }
}
```

---

## Step 2 — Run the example

```sh
cargo run --example automl_classify
```

Expected output (approximate — exact values depend on HDIT selection logic):

```
Signals before selection:
  baseline_all_true          acc=0.500  timing=    50 µs  tier=T0
  alternating                acc=0.500  timing=   800 µs  tier=T1
  near_perfect               acc=1.000  timing= 3500 µs  tier=T2

AutoML plan:
  Selected signals:     ["near_perfect"]
  Fusion operator:      Single
  Plan accuracy:        1.000
  Total timing:         3500 µs
  Signals evaluated:    3
  Rejected (corr):      1
  Rejected (no gain):   1

Selected signal tiers:
  near_perfect               tier=T2

Pareto front (1 candidates):
  signals=["near_perfect"]   acc=1.000  complexity=1  timing=3500 µs  chosen=true

Interpretation: single signal selected — no fusion needed.
```

HDIT selected `near_perfect` first (highest accuracy). `baseline_all_true` was then rejected for insufficient marginal gain (accuracy was already 1.00), and `alternating` was also rejected. Because only one signal was needed, the fusion operator is `Single`.

---

## Interpreting the fields

**`plan.selected`** — the ordered list of signal names chosen. Earlier entries in the list contributed more marginal gain.

**`plan.fusion`** — the cheapest fusion operator that handles the selected set size. With a single signal this is always `Single`.

**`plan.plan_accuracy`** — fraction of traces where the fused prediction agrees with the anchor. 1.000 means perfect agreement.

**`plan.total_timing_us`** — sum of `timing_us` across all selected signals. Use this to estimate runtime cost of the full decision stack.

**`plan.pareto_front`** — every non-dominated `(accuracy, complexity, timing)` candidate. A candidate is dominated if another has at least the same accuracy, no higher complexity, and no higher timing — with at least one strict improvement. The `chosen=true` row is HDIT's greedy pick, but you may choose a different Pareto-optimal point to trade off accuracy for lower latency.

**`signals_rejected_correlation`** — signals dropped because their Pearson correlation with an already-selected signal exceeded 0.95. Correlated signals add no diversity.

**`signals_rejected_no_gain`** — signals dropped because they added less than 0.1% marginal accuracy.

---

## Using real data

In production, replace the synthetic `Vec<bool>` signals with predictions from real classifiers run over XES traces. Each classifier runs independently over the traces and writes its `Vec<bool>` result. Pass all classifier outputs as `SignalProfile`s to `run_hdit_automl`.

The anchor is typically the result of token-based replay: traces where replay was perfect get `true`; imperfect traces get `false`.

---

## Next steps

- **Tutorial 04** — Run the Definition of Done verifier to check that your pipeline artifacts are present and correct.
- Explore `src/ml/hdit_automl.rs` for the `greedy_orthogonal_select` function and the full Pareto front construction logic.
- See `src/ml/weighted_vote.rs` and `src/ml/rank_fusion.rs` for the fusion operator implementations.
