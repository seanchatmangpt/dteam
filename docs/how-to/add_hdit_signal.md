# Add a New Signal to the HDIT Candidate Pool

This guide covers adding a new prediction signal to the 15-candidate HDIT pool in
`src/bin/pdc2025.rs`. HDIT selects a minimal orthogonal subset of signals; adding a signal
that is strongly correlated with an existing one will result in it being rejected for
correlation.

---

## What a signal is

A signal is a `Vec<bool>` with one entry per trace in the test log, plus a `u64` timing
measurement in microseconds. Timing determines which compute tier the signal is assigned to,
which affects how HDIT weights cost vs. accuracy in the Pareto front.

`SignalProfile` is constructed at `src/ml/hdit_automl.rs` lines 70-88:

```rust
pub struct SignalProfile {
    pub name: String,
    pub predictions: Vec<bool>,       // one bool per trace
    pub accuracy_vs_anchor: f64,      // computed automatically
    pub timing_us: u64,               // measured wall time
    pub tier: Tier,                   // derived from timing_us
}
```

The `accuracy_vs_anchor` field is computed automatically from the conformance anchor
(`hdit_anchor`) — you do not set it manually.

---

## Step 1 — Compute predictions

Write code that produces a `Vec<bool>` with `n_traces` elements. Your algorithm can live
anywhere; add a new `src/ml/my_signal.rs` module or extend an existing one.

```rust
// Returns one bool per trace: true = predicted in-language, false = out-of-language
let cls_my_signal: Vec<bool> = my_signal::classify(&traces, &model);
assert_eq!(cls_my_signal.len(), n_traces, "signal length must match trace count");
```

Measure wall time with `std::time::Instant`:

```rust
let t0 = std::time::Instant::now();
let cls_my_signal: Vec<bool> = my_signal::classify(&traces, &model);
let timing_my_signal = t0.elapsed().as_micros() as u64;
```

---

## Step 2 — Add to `hdit_candidate_names_preds`

The tuple list is at `src/bin/pdc2025.rs` lines 981-998. Each entry is
`(&str name, &Vec<bool> predictions, u64 timing_us)`.

```rust
let hdit_candidate_names_preds: Vec<(&str, &Vec<bool>, u64)> = vec![
    ("H_inlang_fill",   &cls_h,            552),
    ("G_fitness_rank",  &cls_g,            690),
    ("F_classify_exact",&cls_f,          1_300),
    ("HDC_prototype",   &cls_hdc,         2_500),
    ("S_synthetic",     &cls_s_ensemble,  8_000),
    ("E_edit_dist",     &cls_e,          60_000),
    ("Combo_ensemble",  &cls_combo,     541_000),
    ("Vote500",         &cls_vote500,   541_000),
    ("Combinator",      &cls_combinator, 15_000),
    ("SupTrained_vote", &cls_sup_trained,20_000),
    ("AutoML_hyper",    &cls_automl_hyper,45_000),
    ("RL_AutoML",       &cls_rl_automl,  25_000),
    ("TF_IDF",          &cls_tfidf,     100_000),
    ("NGram",           &cls_ngram,      30_000),
    ("PageRank",        &cls_pagerank,   50_000),
    ("My_Signal",       &cls_my_signal, timing_my_signal),  // <-- add here
];
```

Use a descriptive, unique name. Names appear verbatim in plan JSON and DoD output.

---

## Step 3 — How timing determines tier

Tier boundaries are defined in `src/ml/hdit_automl.rs` lines 35-40:

| Timing range | Tier | Implication |
|---|---|---|
| 0 – 100 µs | `T0` | Branchless kernel candidate — cheapest path |
| 101 – 2,000 µs | `T1` | Folded signature / small projection |
| 2,001 – 100,000 µs | `T2` | Wider vector / moderate cost |
| > 100,000 µs | `Warm` | Planning layer only — expensive, used sparingly |

`Tier::from_timing_us(us)` is called inside `SignalProfile::new` automatically.
Tier is informational for the Pareto front; T0/T1 signals are preferred when accuracy is
equal to a T2/Warm alternative.

---

## Step 4 — Verify the accounting invariant

After HDIT selection, three asserts run at `src/ml/hdit_automl.rs` lines 258-284:

1. `selected.len() + n_rejected_corr + n_rejected_gain == n_evaluated`
   Every evaluated candidate must be accounted for: selected, rejected for high correlation
   with an already-selected signal, or rejected because it added no marginal accuracy gain.

2. `(plan_accuracy - verify_accuracy).abs() < 1e-9`
   Stored accuracy must exactly match the recomputed value.

3. `predictions.len() == anchor.len()`
   The fused prediction length must match the anchor (= trace count).

Adding a signal increases `n_evaluated` by 1. The accounting identity will still hold as
long as the new signal ends up in exactly one of: selected, rejected for correlation, or
rejected for no gain. No special action needed unless your signal causes `n_evaluated` to
drift from the tuple list length — that would indicate a bug in your insertion.

If an assert fires, check:
- Your `Vec<bool>` has exactly `n_traces` elements (not padded or truncated).
- The timing is a real measurement, not a placeholder `0` that would misclassify to T0.

---

## Step 5 — Smoke test

```bash
cargo test --lib
```

For a full end-to-end run with plan JSON output:

```bash
cargo run --bin pdc2025 --release -- --stem=<one_log_stem>
```

Then inspect `artifacts/pdc2025/automl_plans/<stem>.json` and confirm:
- `accounting_balanced: true`
- Your signal name appears in either `selected` or the per-signal accuracy table
  (`per_signal_gt_accuracy`)
- `pareto_front` has exactly one `"chosen": true` entry

```bash
cargo make dod
```

All Phase 3 checks must pass (green).
