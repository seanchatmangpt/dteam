# HDIT and the Anti-Lie Doctrine: Orthogonal Signal Selection

*Diátaxis — Understanding-oriented explanation.*

---

## 1. The Core Problem: Correlated Signals Add No Information

Suppose you are trying to determine which traces in an event log belong to the "normal" process variant. You have three classifiers: a conformance-replay-based score, a structural variant detector, and a timing anomaly detector. Each one alone achieves 78% accuracy. You combine all three by majority vote and get — 78% accuracy, identical to any single one. You have added two classifiers and gained nothing.

This failure mode is not rare; it is the default outcome when signals are selected by individual accuracy alone. If the three classifiers make the same mistakes on the same traces, their errors are perfectly correlated. Voting on three copies of the same opinion does not reduce uncertainty; it amplifies overconfidence. The combined classifier is precisely as wrong as each individual one, on exactly the same cases, but now it is also more expensive to compute.

The inverse is equally instructive. Suppose you have two signals, each with 62% accuracy — worse than random-plus-domain-knowledge alone. If their errors are perfectly anti-correlated — one is wrong exactly where the other is right — majority vote on the two achieves 100% accuracy. Two individually poor signals can produce a perfect ensemble if they are orthogonal.

This is why signal selection by accuracy rank is wrong, and why HDIT (Hyperdimensional Information Theory) automl exists. The question is not "which signals are most accurate?" but "which combination of signals covers the error space most completely, with minimum redundancy and minimum cost?" That is a combinatorial optimization problem that requires explicitly measuring the overlap between signals.

The surveyor analogy captures it well. A surveyor triangulating a point takes two bearings from different positions. A third bearing from a position on the same line as the first two adds nothing — the three lines of sight are coplanar and share all the same error sources. The third bearing only adds information if taken from a position that is genuinely independent: off the line, in a different direction, with different atmospheric conditions. The value of a measurement is its independence, not its precision in isolation.

---

## 2. Hyperdimensional Computing Intuition

Each signal in HDIT is a binary code vector: one bit per trace in the evaluation pool. A signal that correctly classifies trace *i* has bit *i* set; one that misclassifies it has bit *i* clear. The set of traces is the "trace space," and each signal is a point in that space, represented as a binary vector of length N (where N is the number of traces).

Hyperdimensional computing (HDC) is a computational paradigm built on the observation that random high-dimensional binary vectors are nearly orthogonal with high probability. In a space of 1000 traces, two randomly generated classifiers will agree on approximately 500 traces (50%) — neither helping nor hurting each other. Useful classifiers do better than 50% (they agree with the anchor more than a coin flip), and useful *combinations* of classifiers maintain mutual near-orthogonality while each exceeding baseline individually.

The HDC classifier in `src/ml/hdc.rs` is the direct implementation of this intuition: it encodes traces as high-dimensional binary vectors and classifies by Hamming distance to prototype vectors. The prototype for each class is the majority vote of training examples — the "center" of the class in the binary trace space. This is identical in structure to the majority-vote fusion used in HDIT when combining signal predictions.

The practical translation: Hamming distance between two signal prediction vectors is the number of traces where they disagree. A Hamming distance of 0 means perfect correlation (they always agree). A Hamming distance of N/2 means orthogonality (they agree on exactly half the traces, like two coin flips). A Hamming distance of N means perfect anti-correlation (they always disagree). Pearson correlation is the continuous analog of Hamming distance normalized to [−1, +1].

HDIT operates in this binary code vector space. It searches for a small set of signal vectors that collectively cover as much of the trace space as possible, without redundantly covering the same regions.

---

## 3. Pearson Correlation as Information Overlap

To measure how much information two signals share, HDIT uses Pearson correlation on the boolean prediction vectors treated as float values (false = 0.0, true = 1.0). The implementation in `hdit_automl.rs`:

```rust
fn correlation(a: &[bool], b: &[bool]) -> f64 {
    let a_f: Vec<f64> = a[..n].iter().map(|&x| x as u8 as f64).collect();
    let b_f: Vec<f64> = b[..n].iter().map(|&x| x as u8 as f64).collect();

    let a_mean = a_f.iter().sum::<f64>() / n as f64;
    let b_mean = b_f.iter().sum::<f64>() / n as f64;

    // ... compute variances and covariance ...

    if a_std < 1e-10 || b_std < 1e-10 {
        return 0.0;
    }

    cov / (a_std * b_std)
}
```

Pearson correlation r lies in [−1, +1]. When r = 1.0, the two signals are perfectly linearly related — knowing one tells you everything about the other. When r = 0.0, they are uncorrelated — knowing one tells you nothing about the other. In information-theoretic terms, r² is the fraction of variance in signal B that is explained by signal A. An r of 0.95 means that 90.25% of the variance in B is already captured by A; adding B to a pool that contains A recovers only the remaining 9.75%.

The threshold used by HDIT is r = 0.95. Signals with |r| ≥ 0.95 are rejected as "too correlated" with any already-selected signal. This threshold means we tolerate up to 90% shared variance before calling two signals redundant — a conservative threshold that ensures genuine independent information is required for admission.

The constant-signal special case is equally important: `if a_std < 1e-10 || b_std < 1e-10 { return 0.0; }`. A signal that predicts the same value for every trace (all-true or all-false) has zero variance. It carries zero information — it is equivalent to a constant function. Pearson correlation is undefined for constant signals (division by zero in the denominator), so the code returns 0.0. This does not mean the signal is orthogonal to everything; it means it is informationless. The `gain_threshold` of 0.001 in `greedy_orthogonal_select` will separately reject such signals because their marginal gain over any non-trivial baseline is negligible.

---

## 4. The Greedy Orthogonal Algorithm

The `greedy_orthogonal_select` function implements a specific variant of the classic greedy subset selection algorithm, adapted for the information-theoretic structure of signal selection:

```rust
fn greedy_orthogonal_select(
    candidates: &[SignalProfile],
    anchor: &[bool],
    n_target: usize,
    gain_threshold: f64,   // 0.001
    max_correlation: f64,  // 0.95
) -> (Vec<SignalProfile>, usize, usize)
```

The algorithm sorts candidates by accuracy descending and seeds the selection with the highest-accuracy signal unconditionally. Then it iterates: for each remaining candidate, it computes the maximum absolute Pearson correlation with any already-selected signal (`max_corr`), immediately rejects any candidate with `max_corr >= 0.95`, computes the marginal gain of adding this candidate, and scores the candidate as:

```
score = gain / (max_corr + 0.01)
```

This score formula is the critical design decision. It penalizes correlation even when marginal gain is positive. A signal that adds 5% gain but has 0.90 correlation with an existing signal scores `0.05 / (0.90 + 0.01) = 0.055`. A signal that adds 3% gain but has 0.10 correlation scores `0.03 / (0.10 + 0.01) = 0.273`. The low-correlation signal wins decisively despite offering less raw gain, because its gain is genuinely new information rather than repackaged existing information.

The algorithm stops when no candidate offers `gain > gain_threshold = 0.001`. This 0.1% minimum gain threshold prevents adding signals that offer negligible improvement while still contributing to ensemble computational cost.

Why greedy, without backtracking? Because the problem is NP-hard in the general case (set cover), and greedy provides a provable approximation guarantee: the greedy solution achieves at least `(1 − 1/e)` ≈ 63% of the optimal submodular objective. In practice, on real signal pools, greedy performs close to optimal because the signal space is well-structured — a domain signal and a timing signal and a conformance signal cover genuinely different error regions.

---

## 5. The Pareto Front: Three Objectives, No Single Winner

A single number cannot capture the trade-off between accuracy, complexity, and computational cost. The `pareto_filter` function in `hdit_automl.rs` implements the classic multi-objective Pareto filter:

```rust
fn pareto_filter(mut cands: Vec<PlanCandidate>) -> Vec<PlanCandidate> {
    // A candidate is dominated if another has
    // >= accuracy, <= complexity, <= timing,
    // with at least one strict inequality.
}
```

Candidate A dominates candidate B if A is at least as good as B on every objective and strictly better on at least one. The Pareto front is the set of non-dominated candidates — the candidates for which no other candidate is better on all three dimensions simultaneously.

Consider a concrete example. Signal ensemble E1 achieves 92% accuracy in 5ms with 3 signals. Signal ensemble E2 achieves 92% accuracy in 200ms with 5 signals. E1 dominates E2 (same accuracy, lower timing, lower complexity), so E2 is removed from the Pareto front. Signal ensemble E3 achieves 95% accuracy in 150ms with 6 signals. Neither E1 nor E3 dominates the other (E1 is faster and simpler; E3 is more accurate), so both remain on the front.

The Pareto front does not make the final choice for you — it surfaces the trade-off. A deployment context with tight latency requirements chooses E1. A context where accuracy is paramount chooses E3. The HDIT algorithm labels one candidate as `chosen: true` — the greedy selection result — but the full Pareto front is returned so callers can make context-appropriate choices.

The `chosen: true` invariant is enforced by an assertion:

```rust
let chosen_count = pareto_front.iter().filter(|c| c.chosen).count();
assert_eq!(chosen_count, 1, "Pareto front lie: {} candidates marked chosen", chosen_count);
```

If the greedy selection is not on the Pareto front (it was dominated), it is added to the front with `chosen: true` regardless, ensuring the caller always has access to the greedy result as a baseline. This is the `chosen` invariant: the algorithm's recommendation is always surfaced, even if the Pareto filter would have excluded it.

---

## 6. The Anti-Lie Accounting Invariant

The most unusual feature of the HDIT implementation is an `assert_eq!` in production code that verifies a bookkeeping identity:

```rust
assert_eq!(
    selected.len() + n_rejected_corr + n_rejected_gain,
    n_evaluated,
    "HDIT accounting lie: selected({}) + rejected_corr({}) + rejected_gain({}) != evaluated({})",
    selected.len(), n_rejected_corr, n_rejected_gain, n_evaluated,
);
```

Every candidate that entered the evaluation is accounted for: either selected, rejected for excessive correlation, or rejected for insufficient marginal gain. There is no fourth category. There is no "slipped through the cracks" or "evaluated but not counted." The three buckets must partition the evaluated set exactly.

This is double-entry bookkeeping applied to algorithm evaluation. In financial accounting, every transaction appears in exactly two places: a debit and a credit. The balance sheet must balance precisely — any discrepancy reveals an error or a lie. The HDIT accounting assert plays the same role: it is a balance sheet that must close at zero. If signal search code were to silently skip candidates (a common optimization shortcut that introduces subtle correctness bugs), the assert would catch it immediately.

The anti-lie framework extends to downstream reporting. When you report that HDIT evaluated N signals and selected K, rejected M for correlation, and rejected N−K−M for gain, a reader can verify: K + M + (N−K−M) = N. The numbers add up. The search space is fully accounted for, not implicitly cropped.

This matters for process mining in particular because the event log is the audit record. A process mining system that silently excludes signal candidates from its evaluation is lying about its search scope. A system that cannot explain why each candidate was included or excluded cannot be trusted to surface deviations between the declared process and the real one.

---

## 7. Successive Halving: Budget Management Without Lying

Successive halving (SH) is a multi-fidelity optimization technique: evaluate all candidates cheaply on a small sample, promote only the top fraction to full evaluation, achieving order-of-magnitude speedup with acceptable false-negative risk.

The `run_hdit_automl_sh` implementation uses a deterministic stride for subsampling, not random sampling:

```rust
let stride = anchor.len() / n_sub.max(1);
let sub_anchor: Vec<bool> = (0..n_sub)
    .map(|i| anchor[(i * stride).min(anchor.len() - 1)])
    .collect();
```

Stride-based sampling takes every `stride`-th trace: traces 0, stride, 2·stride, etc. This is reproducible — given the same input, the same subsample is always produced, with no random seed required. Random subsampling introduces variance in which signals are promoted, making the overall result non-deterministic even if the HDIT algorithm itself is deterministic.

The false-negative risk of SH is that a signal which would be excellent on the full dataset appears mediocre on the subsample (perhaps the traces where it excels are not represented in the stride). Deterministic stride sampling minimizes this risk because it spaces the sampled traces evenly across the trace sequence, capturing the distribution of trace types more faithfully than a random sample of equal size.

After SH promotes the top 1/3 of candidates and runs full HDIT on the promoted pool, a critical correction is applied:

```rust
// Anti-lie: signals_evaluated MUST reflect the ORIGINAL pool size
let n_sh_rejected = n_total - n_promoted;
plan.signals_evaluated = n_total;
plan.signals_rejected_no_gain += n_sh_rejected;
```

The candidates dropped by SH at rung 0 were evaluated — on a smaller sample, but evaluated nonetheless. They must appear in the accounting as `rejected_no_gain`. Without this correction, the plan would report evaluating only the promoted candidates, hiding the fact that the majority of the search space was handled at rung 0. The anti-lie invariant is then re-verified after the correction:

```rust
assert_eq!(
    plan.selected.len() + plan.signals_rejected_correlation + plan.signals_rejected_no_gain,
    plan.signals_evaluated,
    "SH accounting lie: ...",
);
```

The invariant holds across both single-stage and multi-stage (SH) evaluation. The caller cannot distinguish SH from full HDIT by examining the plan's accounting fields — both produce plans where every evaluated candidate is accounted for.

---

## 8. Contest Design and the Self-Aware System

The HDIT evaluation framework includes two meta-metrics that measure the quality of the evaluation itself, not just the quality of the selected ensemble: `anchor_vs_gt` and `oracle_gap`.

The anchor is the pseudo-ground-truth used internally by HDIT: the majority vote of all eight input signals. It is not the true ground truth (which may not be available) but a proxy derived from the signal pool itself. `anchor_vs_gt` measures how close this proxy is to the actual ground truth when the true labels are available for comparison. A high `anchor_vs_gt` means the anchor is a reliable proxy; a low value means the signal pool is systematically biased in the same direction, making the anchor misleading.

`oracle_gap` is the difference between HDIT's achieved accuracy and the best single signal's accuracy. If HDIT achieves 87% and the best single signal achieves 84%, oracle_gap = +3%. HDIT's fusion exceeded the best individual. If HDIT achieves 82% and the best individual achieves 84%, oracle_gap = −2%. Negative oracle_gap means HDIT's fusion was counterproductive — the selected signals, despite being individually accurate and pairwise orthogonal, combined to perform worse than simply using the best one.

Negative oracle_gap is a diagnostic signal. It typically means the signal pool lacked genuine orthogonality (the signals that appeared orthogonal by Pearson correlation shared higher-order dependencies), or that the fusion operator (weighted vote, Borda count) was mismatched to the signal error structure. It is not a failure of the algorithm — the algorithm reported the truth. It is a prompt to re-examine the signal design.

This transparency is the anti-lie principle applied to self-evaluation. A system that reports only its successes and hides its suboptimal runs is lying by omission. A system that reports oracle_gap honestly, even when it is negative, is giving the engineer the information needed to improve the signal pool rather than the impression that the current approach is unimprovable.

The self-aware system knows how much it left on the table. That knowledge is what separates a trustworthy process intelligence engine from a black box that produces confident-sounding numbers.
