# The 67.78% Ceiling Story

## The ceiling

For months, every classifier, every fusion strategy, and every ensemble applied to the PDC 2025 dataset capped at 67.78% accuracy versus ground truth. The ceiling looked like a data limit — as if the training logs simply didn't contain enough information to classify test traces more accurately. The assumption was natural: add more algorithms, try better ensembles, tune hyperparameters. None of it helped.

67.78% is not a bad number for process conformance classification. It represents a meaningful separation between conforming and non-conforming traces. But when dozens of algorithms cluster at the same ceiling, the ceiling itself is the finding. It indicates a structural constraint in the signal space, not a limitation of any particular algorithm.

The breakthrough came from a different direction entirely: TF-IDF hit 73.6% on log 000110 — roughly 6 points above the ceiling — without any new algorithm, any new training data, or any change to the conformance infrastructure. The fix was a different projection, not a better one within the same projection family.

## Why conformance signals are correlated

The conformance signals F (fitness), G (generalization), and H (language membership via in-language check) all route through the same object: an approximate Petri net discovered from the training log. The net is approximate because α-algorithm or heuristic miner discovery is lossy — the discovered net is a model of the training log, not the true process. When the net has an error — a missing arc, an over-generalized silent transition, an imprecise initial marking — that error propagates identically to F, G, and H, because all three ask the same approximate net the same question with different scoring formulas.

E (edit distance to the nearest positive trace) escapes some of this correlation because it does not route through the net at all — it measures sequence distance directly. But E is still sequence-dependent: it treats the trace as an ordered string and penalizes deviations from the known-positive order. The same structural errors that confuse F/G/H affect E differently, but E is not orthogonal to F/G/H across all log types.

Fusing correlated signals does not recover lost information. If F=0.65, G=0.63, and H=0.67 all reflect the same net error, a weighted sum produces ~0.65, not a number closer to ground truth. The lost information is in the net's approximation errors, and no fusion of same-projection signals can access information that the shared projection destroyed.

## Signal family taxonomy

Understanding why TF-IDF escaped requires understanding the three families into which all 15 signals fall:

**Language-membership signals (F, G, H, E):** All ask "does this trace belong to the language defined by the Petri net?" or "how far is this trace from known-positive traces?" They are sequence-dependent and net-dependent. Their ceiling is the ceiling of the net approximation.

**Order-agnostic signals (TF-IDF, NGram, PageRank):** These ask "does the activity frequency distribution of this trace match the frequency distribution of positive traces?" They do not ask about ordering and do not route through the Petri net. TF-IDF: frequency-weighted bag-of-activities cosine similarity. NGram: adjacent-activity pair statistics. PageRank: graph centrality over the activity transition graph.

**Embedding signals (HDC, synthetic ML classifiers):** These learn a feature representation from training traces and classify test traces by similarity in embedding space. Order-aware hypervectors (HDC) remain sequence-dependent and inherit some of the language-membership ceiling. Synthetic ML classifiers vary by their internal projection choice.

## What makes TF-IDF structurally orthogonal

TF-IDF treats each trace as a document of activity labels, with no concept of ordering between activities. The exact algorithm steps are:

1. Collect all training and test traces. Build a vocabulary: the sorted, deduplicated set of all activity labels (sorted to ensure insertion-order independence).
2. For each trace, compute TF-IDF weights: `TF(trace, activity) = count(activity, trace) / len(trace)`, `IDF(activity) = ln((1 + N) / (1 + df(activity))) + 1` where N is the document count and df is the number of traces containing the activity. The smooth IDF formula matches scikit-learn's default.
3. Build a positive-trace centroid: average the TF-IDF vectors of all training traces (currently the full training set — filtering known negatives is a future optimization expected to improve accuracy further).
4. Score each test trace by cosine similarity to the centroid.
5. Predict the top-500 traces by similarity as positive.

Step 4 is where the orthogonality emerges. Cosine similarity to a centroid is invariant to activity order within a trace. A trace `[A, B, C, A]` and a trace `[A, A, B, C]` produce identical TF-IDF vectors and therefore identical centroid-similarity scores. The Petri net's language — which distinguishes these two orderings — is invisible to TF-IDF. This is precisely why TF-IDF captures information that F/G/H cannot: it asks a completely different question, through a completely different projection, about a completely different property of the trace.

The result is structurally independent of the net's approximation errors. If the net fails to recognize a valid ordering `[B, A, C]` as conforming (because the discovered net requires `[A, B, C]`), F/G/H all give low scores to that trace. TF-IDF scores it by whether B, A, and C appear in proportions similar to positive traces — and may correctly identify it as positive even when the net is wrong.

## The anti-lie infrastructure that made the result trustworthy

A 6-point improvement over a months-long ceiling is exactly the kind of result that demands skepticism. The infrastructure that validated the TF-IDF finding:

**GT leakage audit:** Test logs contain zero `pdc:isPos` attributes (verified by inspection). TF-IDF code never reads the `ground_truth/` directory. The positive centroid is built from training trace activities only, with no access to test labels.

**OOF stacking:** `stack_ensemble_oof` uses K-fold out-of-fold predictions to evaluate signal accuracy. No signal sees its own training labels during evaluation. This prevents the inflated accuracy that arises when a model is evaluated on data it was trained on.

**Accounting identity:** The HDIT anti-lie assertion verifies that `selected + rejected_corr + rejected_gain == evaluated` exactly. No signal can disappear from the accounting. This makes it impossible to silently exclude signals from the evaluation.

**`oracle_signal` field:** Every `AutomlPlan` JSON artifact records which signal had the highest accuracy against ground truth (`oracle_signal`) alongside which signal HDIT selected (`selected`). When these differ, the artifact honestly surfaces the discrepancy rather than concealing it.

## The anchor bias problem

HDIT's greedy selector uses the majority vote of all 8 sequence-based signals as its anchor — the pseudo-ground-truth it optimizes against. The anchor is conformance-dominated: F, G, H, E together constitute most of the vote. A signal that disagrees with the anchor scores low on `accuracy_vs_anchor` even if it is correct against the true ground truth.

TF-IDF wins against true ground truth on 5 of 15 logs. But on most of those logs, TF-IDF disagrees with the conformance-dominated anchor. HDIT's greedy selector therefore rejects TF-IDF (it loses against the anchor) and selects H-dominated signals (which agree with the anchor but are wrong against ground truth).

This is not a bug in HDIT. The anchor bias is a structural consequence of using conformance signals as pseudo-labels for a log where the true labels are not available during selection. The `oracle_signal` and `oracle_gap` fields in the `AutomlPlan` JSON exist precisely to surface this limitation honestly. A downstream DoD verifier can read the plan and understand both what HDIT selected and what a ground-truth-optimal selection would have been — without HDIT pretending the two are the same.

The ceiling is not gone. It has been correctly characterized: a projection ceiling that TF-IDF escapes for activity-frequency-separable logs, but that conformance-anchored HDIT selection partially reinstates. The next step — adjusting the anchor to reduce conformance dominance — is now a well-specified problem rather than a mystery.
