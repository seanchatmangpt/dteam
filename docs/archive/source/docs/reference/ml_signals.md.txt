# HDIT Candidate Signal Reference

All 15 candidate signals in the HDIT pool. Source: `src/bin/pdc2025.rs` lines 981-998.
Timing values are measured wall-clock microseconds as declared at insertion point; actual
runtime timing varies by hardware and log size.

---

## Signal table

| Name | timing_us | Tier | Source File | Signal Family | Orthogonality Notes |
|---|---|---|---|---|---|
| `H_inlang_fill` | 552 | T1 | `src/bin/pdc2025.rs` | Conformance | Anchor-aligned; primary reference signal. High correlation risk with `G_fitness_rank`. |
| `G_fitness_rank` | 690 | T1 | `src/bin/pdc2025.rs` | Conformance | BFS fitness ranking; similar information to `H_inlang_fill`, often rejected for correlation. |
| `F_classify_exact` | 1,300 | T1 | `src/bin/pdc2025.rs` | Conformance | Exact token replay classification; slower than H/G but distinct decision boundary. |
| `HDC_prototype` | 2,500 | T2 | `src/ml/hdc.rs` | ML / Hyperdimensional | Hyperdimensional computing prototype; structurally orthogonal to token-replay signals. |
| `S_synthetic` | 8,000 | T2 | `src/ml/synthetic_trainer.rs` | ML | Synthetic trace ensemble; trained on augmented data; orthogonal to conformance family. |
| `E_edit_dist` | 60,000 | T2 | `src/bin/pdc2025.rs` | NLP / String | Edit distance from representative positive traces; string-family signal. |
| `Combo_ensemble` | 541,000 | Warm | `src/ml/pdc_combinator.rs` | Meta | Full combinatorial ensemble; absorbs most other signal information. |
| `Vote500` | 541,000 | Warm | `src/bin/pdc2025.rs` | Meta | 500-trial majority vote; correlated with `Combo_ensemble`. |
| `Combinator` | 15,000 | T2 | `src/ml/pdc_combinator.rs` | Meta | Lightweight combinator; faster than `Combo_ensemble`; partially correlated. |
| `SupTrained_vote` | 20,000 | T2 | `src/ml/pdc_supervised.rs` | ML | Supervised classifier vote over pseudo-labeled features; ML family. |
| `AutoML_hyper` | 45,000 | T2 | `src/ml/automl.rs` | ML | RandomSearch hyperparameter-tuned ensemble; most distinct in ML family. |
| `RL_AutoML` | 25,000 | T2 | `src/automation.rs` | ML | RL-discovered configuration applied to classifier pool; orthogonal to static ML signals. |
| `TF_IDF` | 100,000 | T2 | `src/ml/nlp.rs` | NLP | TF-IDF activity sequence similarity; NLP family, orthogonal to conformance family. |
| `NGram` | 30,000 | T2 | `src/ml/nlp.rs` | NLP | N-gram frequency matching; partially correlated with TF-IDF. |
| `PageRank` | 50,000 | T2 | `src/ml/network_analysis.rs` | Graph | PageRank of activity transition graph; structurally orthogonal to all other families. |

---

## Tier boundary mapping

Tier boundaries are defined in `src/ml/hdit_automl.rs` lines 35-40 via `Tier::from_timing_us`:

| Timing range | Tier | Label | Typical use |
|---|---|---|---|
| 0 – 100 µs | `T0` | `"T0"` | Branchless kernel candidate; bit-manipulation only |
| 101 – 2,000 µs | `T1` | `"T1"` | Folded signature or small projection |
| 2,001 – 100,000 µs | `T2` | `"T2"` | Wider vector or moderate ML cost |
| > 100,000 µs | `Warm` | `"Warm"` | Planning layer only; used when accuracy gain justifies cost |

No current signal occupies T0. All three conformance signals (`H`, `G`, `F`) land in T1.
The two Warm-tier signals (`Combo_ensemble`, `Vote500`) carry identical timing because
they both run the same 500-trial combinatorial loop.

---

## Signal families

| Family | Members | Shared information source |
|---|---|---|
| Conformance | `H_inlang_fill`, `G_fitness_rank`, `F_classify_exact` | Petri net token replay fitness |
| NLP | `TF_IDF`, `NGram`, `E_edit_dist` | Activity label sequences as text |
| Graph | `PageRank` | Activity transition graph topology |
| ML | `HDC_prototype`, `S_synthetic`, `SupTrained_vote`, `AutoML_hyper`, `RL_AutoML` | Feature-matrix classifiers |
| Meta | `Combo_ensemble`, `Vote500`, `Combinator` | Ensemble aggregation of other signals |

---

## How HDIT uses correlation to reject redundant signals

HDIT applies greedy orthogonal selection (`run_hdit_automl` in `src/ml/hdit_automl.rs`):

1. Sort all candidates by `accuracy_vs_anchor` descending.
2. Seed the selected set with the highest-accuracy candidate.
3. For each remaining candidate in order:
   - Compute Pearson r against every already-selected signal (bool-as-float).
   - If `|r| > max_correlation` (default 0.9): reject with `n_rejected_corr += 1`.
   - Else if marginal accuracy gain < `gain_threshold`: reject with `n_rejected_gain += 1`.
   - Else: add to selected set.
4. Choose fusion operator by selected count: `Single` (1), `WeightedVote` (2-4), `BordaCount`
   (5+), or `Stack` (>=3 with high accuracy variance).

Consequence: signals within the same family are often rejected for correlation. In practice
HDIT typically selects 2-4 signals spanning conformance, ML, and graph families. The
`Combo_ensemble` and `Vote500` signals are Warm-tier and are frequently dominated on the
Pareto front by T2 alternatives with similar accuracy.

The accounting invariant enforced at `src/ml/hdit_automl.rs` lines 261-265 guarantees
`selected + rejected_corr + rejected_gain == evaluated` with no silent disposal of any
candidate.
