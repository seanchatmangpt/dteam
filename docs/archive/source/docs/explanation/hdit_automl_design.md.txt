# HDIT AutoML Design Philosophy

## What HDIT is not

HDIT (Hyperdimensional Information Theory AutoML) is frequently mischaracterized as a hyperparameter tuner. It is not. Understanding what it is requires first being precise about what it is not.

**TPOT2** searches a space of pipeline configurations — which algorithms to combine, in which order, with which hyperparameters. It mutates and evaluates pipelines, spending its budget finding configurations that score better than their predecessors. The output is a fitted pipeline.

**Auto-sklearn** uses Bayesian optimization over a joint space of algorithm identity and hyperparameter values. It builds an ensemble from validated configurations. The output is a meta-learner.

Both approaches share a core assumption: the signal space is fixed, and the job is to find the best configuration within it. HDIT rejects this framing entirely.

HDIT's input is a pool of pre-computed signal predictions — each prediction is a `Vec<bool>` over N traces, produced by an already-evaluated algorithm. The predictions are already computed; the algorithms have already run. HDIT does not retrain anything, adjust any hyperparameter, or evaluate any new configuration. Its job is to select the minimal orthogonal subset of the pre-computed predictions that collectively maximize accuracy without redundancy.

This distinction matters because orthogonality is not a hyperparameter. You cannot tune your way from correlated signals to orthogonal ones. You need structurally different projections — different algorithms that project the trace space through different lenses. HDIT identifies which of the available projections are genuinely independent, and which are just correlated copies of each other dressed in different algorithm clothing.

## Orthogonal signal selection

The core algorithm is `greedy_orthogonal_select`. It works as follows:

1. Sort all candidate signals by `accuracy_vs_anchor` descending. The signal most aligned with the anchor (the pseudo-ground-truth) goes first.
2. Add the top signal unconditionally to the selected set.
3. For each remaining candidate (in sorted order):
   a. Compute its Pearson correlation with each already-selected signal's predictions.
   b. Let `max_corr` = the maximum correlation found.
   c. If `max_corr >= 0.95`: the candidate is redundant. Reject, increment `signals_rejected_correlation`.
   d. Compute `marginal_gain`: the accuracy improvement the candidate adds over the current selected set's fused predictions.
   e. If `marginal_gain <= 0.001` (0.1% threshold): the candidate adds nothing. Reject, increment `signals_rejected_no_gain`.
   f. Otherwise: compute `signal_score = gain / (max_corr + 0.01)`. The `+0.01` prevents division by zero when a candidate has zero correlation with all selected signals.
   g. If this score is the best seen in this round: tentatively select the candidate.
4. Stop when no candidate passes both thresholds.

The scoring formula `gain / (max_corr + 0.01)` encodes the orthogonality objective directly: a signal with high marginal gain but low correlation scores much higher than a signal with the same gain but high correlation. The selector prefers accuracy contributions from new directions over accuracy contributions that duplicate existing coverage.

The 0.95 correlation threshold and 0.001 gain threshold are not tunable parameters in the usual sense. They define what "orthogonal" and "useful" mean for this system. Raising the correlation threshold to 0.99 would allow near-duplicate signals; lowering it to 0.5 would reject most signals. The current values were chosen to select the smallest set that captures genuine diversity.

## The Pareto front

After greedy selection, HDIT constructs a TPOT2-style Pareto front over three axes:

- **Accuracy vs anchor** (maximize)
- **Complexity** (minimize) — signal count plus fusion operator cost: Stack adds 2, Borda/Weighted add 1, Single adds 0
- **Total timing** in microseconds (minimize) — sum of selected signal timing measurements

A candidate is **non-dominated** if no other candidate is simultaneously at least as good on all three axes and strictly better on at least one. The Pareto front is the set of all non-dominated candidates.

Every individual signal is a Pareto candidate (Single fusion). Every prefix of the selected set is a Pareto candidate (the greedy prefix with appropriate fusion). The full selected set is the chosen candidate. After Pareto filtering, candidates are sorted by accuracy descending, then complexity ascending.

The chosen candidate — what HDIT's greedy selection produced — is always included in the Pareto front. If the greedy result was dominated and filtered out during Pareto computation, it is added back explicitly. This is guaranteed by the code: `if !found_chosen { pareto_front.push(...) }`. The reason is that the consumer must always be able to see what HDIT actually selected, even if a lower-complexity or lower-latency alternative dominates it on some axes. The `chosen: true` field marks this entry.

The Pareto front is not a recommendation engine. It is honest disclosure: here are all the non-dominated tradeoffs the system found. The deployer, not the algorithm, makes the deployment tradeoff.

## Successive halving

When `successive_halving = true` in `dteam.toml`, HDIT uses a two-rung evaluation protocol:

- **Rung 0**: Score all candidates on a subsampled anchor (default: 20% of traces). Cost is proportional to subsample size.
- **Promotion**: Keep the top 1/`promotion_ratio` fraction by rung-0 accuracy. Default: top 1/3, meaning 2/3 of candidates are eliminated after seeing only 20% of the data.
- **Rung 1**: Run full HDIT selection on the promoted candidates only.

The expected speedup is approximately `promotion_ratio × (1 - rung0_subsample)` — about 2.4× with defaults. The accuracy cost is the risk that the rung-0 ordering on a 20% sample differs from the full ordering. In practice, the top-accuracy signals tend to rank consistently across sample sizes.

The critical accounting rule: `signals_evaluated` in the returned `AutomlPlan` MUST reflect the original candidate count — the full pool before rung-0 screening — not the promoted count. If 15 signals were considered at rung 0 and 5 were promoted to rung 1, `signals_evaluated = 15`. The plan must honestly represent the true search space, even though selection only operated on the promoted subset.

## The anchor bias problem

The HDIT anchor is the majority vote of the input signal pool. For PDC 2025, the pool is dominated by conformance-derived signals (F, G, H, E, several synthetic classifiers). The anchor is therefore a conformance-weighted pseudo-label: a trace the conformance signals agree is positive gets anchor = true.

A signal that systematically disagrees with conformance signals — like TF-IDF, which uses a completely different projection — scores low on `accuracy_vs_anchor` even when it is correct against the true ground truth. The greedy selector sorts by `accuracy_vs_anchor` and starts with the highest-scoring signal. If TF-IDF is 15th in that sorted order and the first few signals are H-dominated with high mutual correlation, the selector may exhaust its budget before reaching TF-IDF.

The `oracle_gap` field in the `AutomlPlan` JSON is the honest disclosure of this limitation: `oracle_gap = oracle_accuracy - plan_accuracy`. When `oracle_gap > 0`, HDIT's selected signal was not the best one available, and the plan says so explicitly. The `oracle_signal` field names which signal was actually best against ground truth.

This is not a bug that should be fixed by changing the greedy algorithm. The anchor bias is inherent to any selection system that must operate without ground truth labels. The honest response is to surface it clearly rather than conceal it behind a number that looks better than it is.

## Anti-lie accounting

The four panics in `run_hdit_automl` are not defensive programming. They are structural guarantees that make dishonest results impossible to construct silently.

**Accounting identity:** `assert_eq!(selected.len() + n_rejected_corr + n_rejected_gain, n_evaluated)`. Every candidate is accounted for. A bug in `greedy_orthogonal_select` that loses or double-counts a candidate will panic immediately rather than silently producing incorrect selection statistics.

**Accuracy consistency:** `assert!((plan_accuracy - verify_accuracy).abs() < 1e-9)`. The stored accuracy is recomputed from the stored predictions and verified to match. This makes it impossible to store an accuracy figure that does not correspond to the actual predictions in the plan.

**Length consistency:** `assert_eq!(predictions.len(), anchor.len())`. The output predictions cover every trace in the input. No trace is silently dropped or duplicated.

**Pareto integrity:** `assert_eq!(chosen_count, 1)`. Exactly one candidate in the Pareto front is marked `chosen: true`. This prevents the ambiguity of a plan where the deployer cannot identify which candidate HDIT selected.

All four are `assert!` calls — panics, not `Result::Err`. The distinction is intentional. A `Result::Err` can be silently ignored by a caller that discards the result. A panic cannot. The anti-lie infrastructure is designed to make honest results the only structurally possible outcome.
