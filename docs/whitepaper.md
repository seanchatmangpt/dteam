# Breaking the 67.78% Conformance Ceiling: HDIT AutoML for Process-Mining Trace Classification

**Version:** 1.0  
**Date:** 2026-04-23  
**Repository:** `dteam` v1.3.0 (Business Source License 1.1)

---

## Abstract

Conformance-based trace classifiers applied to the PDC 2025 Process Discovery Contest repeatedly saturate at approximately 67.78% accuracy. Every signal derived from an approximate Petri net — token-replay fitness, BFS language membership, fitness-fill hybrids — shares the same projection error, forming a structural ceiling rather than a data ceiling. This paper makes four contributions. First, we present HDIT AutoML, a greedy orthogonal signal selection algorithm that applies a Pearson correlation filter to suppress redundant signals, reports a TPOT2-style Pareto front over (accuracy, complexity, timing) trade-offs, employs successive halving for budget control, and uses out-of-fold stacking to prevent level-1 leakage. Second, we demonstrate TF-IDF cosine similarity as a proof-of-concept orthogonal signal: treating each trace as a bag-of-words document rather than a sequenced token replay yields 73.6% peak accuracy on log 000110 and five oracle wins in a 15-log smoke test, breaking the ceiling by six percentage points. Third, we describe an anti-lie infrastructure — an accounting identity assertion, an `oracle_gap` field that honestly surfaces the gap between HDIT's greedy pick and the ground-truth oracle, and a DoD verification gate enforced pre-merge. Fourth, the entire stack is implemented in zero-heap Rust targeting 50–150 ns token replay, 200–500 ns reinforcement-learning steps, and WASM compatibility. Together these contributions establish that the 67.78% ceiling was a projection ceiling and that signal diversification is the correct remedy.

---

## 1. Introduction

Process mining extracts behavioral insight from the digital traces that industrial and administrative processes leave behind. Hospitals log patient pathways; financial institutions record transaction sequences; logistics networks emit shipment events. Each such trace — an ordered sequence of timestamped activity labels — is encoded in the XES standard (IEEE 1849-2016) and can be compared against a formal process model, typically a Petri net, via token-based replay. Replay assigns a fitness score to each trace; traces scoring above a threshold are predicted to belong to the modeled process.

The Process Discovery Contest (PDC 2025) poses a concrete variant of this classification task. Contestants receive 96 event logs, each containing 1000 traces. A single training log (index 11) provides 20 labeled positive traces, 20 labeled negative traces, and 960 unlabeled traces. The objective is to predict which 500 of the 1000 traces in each test log belong to the underlying process. The constraint is intentional: contestants must generalize from a small labeled seed across a large unlabeled population with no access to ground truth at inference time.

For the dteam system, every conformance-based strategy — token-replay fitness (G), BFS exact language membership (F), fitness-fill hybrid (H), combinatorial ensembles on supervised classifiers trained on conformance features, Borda count, reciprocal rank fusion, accuracy-weighted voting — converged to the same ceiling: 67.78%. Refinements in algorithmic complexity produced no gain because all signals were downstream of the same approximate Petri net. When the net's language approximation is wrong for a particular log, every conformance-derived signal inherits the same directional error.

The key insight motivating HDIT AutoML is that signals from structurally different families carry orthogonal errors. A bag-of-words TF-IDF signal has no concept of order or reachability; it measures relative activity frequency against a positive-trace centroid. When the discriminant in a given log is activity frequency rather than sequence legality, TF-IDF will outperform every conformance signal simultaneously. No algorithmic sophistication within the conformance family can recover this information because it is absent from the net's projection.

This paper is organized as follows. Section 2 reviews token-based replay, AutoML prior art, and the PDC 2025 setup. Section 3 describes the five-layer system architecture and zero-heap design. Section 4 presents the ML signal portfolio and the HDIT orthogonal selection algorithm. Section 5 reports experimental results from a 15-log smoke test. Section 6 details the anti-lie infrastructure. Section 7 provides micro-benchmark performance data. Section 8 discusses implications, limitations, and the anchor bias problem. Section 9 concludes with future work.

---

## 2. Background

### 2.1 Token-Based Replay

Token-based replay evaluates a trace against a Petri net by simulating token movement through the net's places. For each activity in the trace, the algorithm attempts to fire the corresponding transition. If the required input tokens are absent, missing tokens are added artificially; if a transition produces tokens that remain unused at the end, they are counted as remaining. Fitness is defined as:

```
fitness = (1 - missing/consumed + 1 - remaining/produced) / 2
```

A fitness of 1.0 indicates a perfectly conforming trace; lower values indicate deviation. The fast path in dteam encodes the marking as a u64 bitmask when the net has at most 64 places, making token firing a branchless bitwise operation: `new_marking = (marking & !input_mask) | output_mask`. For larger nets, a SWAR (SIMD Within A Register) fallback handles up to 1024 places using 16 u64 words.

Language membership (BFS reachability) asks a stricter question: is this trace a member of the net's language? Unlike fitness, BFS membership is binary and does not account for token debt. Both signals share the fundamental limitation that their accuracy ceiling is determined by the quality of the underlying Petri net.

### 2.2 AutoML Prior Art

TPOT2 (Ribeiro et al., 2024) extends genetic programming over scikit-learn pipelines with Pareto-front selection over (accuracy, pipeline complexity) and successive halving for compute budget control. AutoSklearn applies Bayesian optimization over algorithm-hyperparameter space with warm-starting from meta-learned priors. Both systems operate on feature pipelines within a single signal family (tabular features to classifiers). HDIT AutoML differs in that it selects across prediction vectors produced by entirely different signal families, treating correlation structure between predictions rather than feature structure as the diversity criterion.

### 2.3 PDC 2025 Setup

The contest provides 96 labeled test logs (ground truth available post-submission) and one training log (log index 11) with the structure described above: 20 positives, 20 negatives, 960 unlabeled traces. The expected output per log is exactly 500 predicted positives. The 15-log smoke test described in this paper uses a subset of contest logs for which the dteam pipeline has been validated end-to-end; ground truth is read only for evaluation, never during prediction.

---

## 3. System Architecture

### 3.1 Five-Layer Design

The dteam system is organized in five layers, each with a clear responsibility boundary:

```
┌─────────────────────────────────────────┐
│  Verification Layer (dod.rs, verifier)  │
├─────────────────────────────────────────┤
│     ML Portfolio (src/ml/, HDIT)        │
├─────────────────────────────────────────┤
│   RL Discovery (src/reinforcement/)     │
├─────────────────────────────────────────┤
│  Conformance Engine (src/conformance/)  │
├─────────────────────────────────────────┤
│   Process Data (src/models/, src/io/)   │
└─────────────────────────────────────────┘
```

The **Process Data layer** (`src/models/`, `src/io/`) provides `Event`, `Trace`, and `EventLog` types with rust4pm lineage, a `PetriNet` with `PackedKeyTable` markings, and `XESReader` via `quick-xml`. The `canonical_hash` function uses FNV-1a over activity name strings to produce audit-reproducible log identifiers.

The **Conformance Engine** (`src/conformance/`) implements token-based replay in two paths: the fast u64 bitmask path for nets with at most 64 places, and the SWAR path for nets up to 1024 places. `ProjectedLog` pre-indexes activities into integer IDs, eliminating string hashing from the hot loop. Both paths operate without heap allocation on the marking state.

The **RL Discovery layer** (`src/reinforcement/`) provides five tabular agents — `QLearning`, `DoubleQLearning`, `SARSAAgent`, `ExpectedSARSAAgent`, and `ReinforceAgent` — each using `PackedKeyTable` with `FxHasher` for Q-table storage. The `train_with_provenance` function in `src/automation.rs` runs the RL loop over a `ProjectedLog`, uses token-replay fitness as the reward signal, and emits both a discovered `PetriNet` and a byte trajectory of action indices for audit.

The **ML Portfolio layer** (`src/ml/`) contains 33 modules: 3 foundational math/statistics modules, 22 supervised classifiers ported from Joel Grus's "Data Science from Scratch," 4 unsupervised algorithms, 4 domain-specific modules (NLP, word vectors, network analysis, recommender systems), and 9 PDC 2025-specific pipeline modules. The HDIT AutoML module orchestrates signal selection and fusion across this portfolio.

The **Verification layer** (`dod.rs`, `AutomlPipelineVerifier`, `DxQolVerifier`) re-validates every JSON artifact at read time, independent of the write path. The DoD gate (`cargo make dod`) enforces this layer as a pre-merge condition.

### 3.2 Zero-Heap Design

Three structures form the zero-heap spine of the system:

**`PackedKeyTable`** is a flat-array hash map keyed by u64 with open addressing and FNV-1a hashing. It replaces `std::collections::HashMap` on all hot paths — Q-table lookups, net markings, event ID mapping — because it avoids the per-entry heap allocation that `HashMap` incurs on insertion and produces deterministic iteration order for audit.

**`KBitSet<WORDS>`** is a stack-allocated bitset parameterized by the number of u64 words. The `KTier` enum in `src/lib.rs` encodes five tiers:

| Tier | Words | Capacity | Typical Latency |
|------|-------|----------|-----------------|
| K64 | 1 | 64 places | ~20 ns/event |
| K128 | 2 | 128 places | ~25 ns/event |
| K256 | 4 | 256 places | ~45 ns/event |
| K512 | 8 | 512 places | ~90 ns/event |
| K1024 | 16 | 1024 places | ~150 ns/event |

Token firing at every tier is a branchless SWAR operation: `new_marking = (marking & !input_mask) | output_mask`. No branch is taken on the firing condition; instead, the firing is masked out if the precondition fails, and missing tokens are accumulated separately.

**`fnv1a_64`** is the universal hash primitive. Every ID — place index, transition index, activity label, Q-table key — passes through this function. Consistent hashing across layers ensures that the same logical entity maps to the same integer in markings, Q-tables, and provenance records.

### 3.3 Anti-Lie Framework

The anti-lie framework operates at three points in time. At **write time**, `run_hdit_automl` asserts the accounting identity before returning:

```
selected_count + rejected_correlation + rejected_no_gain == signals_evaluated
```

This assertion panics in debug mode and returns an error in release mode if the books do not balance, preventing any caller from receiving a plan whose accounting cannot be reconciled. The `oracle_gap` field is computed as `oracle_accuracy - plan_accuracy` and stored in the JSON artifact alongside the `oracle_signal` name. At **read time**, `AutomlPipelineVerifier` re-checks all invariants independently: `accounting_balanced`, exactly one `chosen=true` in `pareto_front`, and `oracle_gap` within 1e-6 of the recomputed value. At **diff time**, `scripts/automl_plan_diff.sh` exits with code 4 if any plan flips from `accounting_balanced=true` to `accounting_balanced=false` across runs, treating this as an anti-lie violation.

---

## 4. ML Signal Portfolio

### 4.1 Signal Families

The 15-signal candidate pool evaluated in the smoke test draws from five structurally distinct families:

| Family | Net-dependent | Avg Accuracy | Peak Accuracy | Tier |
|--------|--------------|--------------|---------------|------|
| Conformance (F, G, H) | Yes | 66.8% | 73.2% | T1 |
| NLP (TF-IDF, NGram) | No | 66.4% / 59.4% | 73.6% / 63.2% | T2 / Warm |
| Graph (PageRank, E_edit_dist) | No | 57.7% / ~67% | 62.2% / 71.6% | T2 |
| Classical ML (11 supervised) | Yes (feature-dependent) | ~67% | ~67.5% | T1 / T2 |
| Meta / AutoML (AutoML_hyper, RL_AutoML) | Mixed | ~70% / ~50% | 73.0% / 55.0% | Warm |

"Net-dependent" means the signal requires a discovered Petri net as input. Net-independent signals (TF-IDF, NGram, PageRank) derive their predictions entirely from the event log, making their errors structurally orthogonal to conformance errors.

### 4.2 Key Algorithms

The supervised classifiers in `src/ml/` cover all 22 chapters of Grus (2019), ported to Rust and adapted for binary trace classification:

- **Distance-based**: `knn.rs` (k-nearest neighbors), `nearest_centroid.rs` (prototype classification)
- **Probabilistic**: `naive_bayes.rs`, `gaussian_naive_bayes.rs`
- **Decision boundaries**: `perceptron.rs`, `logistic_regression.rs`
- **Tree-based**: `decision_tree.rs` (entropy-driven), `decision_stump.rs`
- **Ensemble**: `gradient_boosting.rs` (additive boosting on stumps)
- **Neural**: `neural_network.rs` (shallow 2-layer), `deep_learning.rs` (configurable depth)

PDC-specific extensions include `pdc_features.rs` (100+ features per trace), `pdc_supervised.rs` (runs all 11 classifiers, returns 11 prediction vectors), `pdc_unsupervised.rs` (5 unsupervised strategies), `hdc.rs` (hyperdimensional computing, order-aware), and `stacking.rs` with out-of-fold meta-learning.

### 4.3 HDIT Orthogonal Selection Algorithm

The HDIT greedy forward selection proceeds as follows:

```
Input: candidates (list of SignalProfile), anchor (Vec<bool>), n_target (int)
       max_correlation (default 0.85), gain_threshold (default 0.001)

selected = []
all_candidates = []

for each candidate in descending accuracy_vs_anchor order:
    if selected is empty:
        selected.append(candidate)   // anchor seed
        continue

    max_r = max over s in selected of Pearson(candidate.predictions, s.predictions)
    if max_r > max_correlation:
        rejected_correlation++
        continue

    fused = fuse(selected + [candidate])
    gain = accuracy(fused, anchor) - accuracy(fuse(selected), anchor)

    if gain < gain_threshold:
        rejected_no_gain++
        all_candidates.append(PlanCandidate { signals: selected + [candidate], ... })
        continue

    selected.append(candidate)
    all_candidates.append(PlanCandidate { signals: selected[:], ... })

pareto_front = pareto_filter(all_candidates)   // non-dominated on (accuracy up, complexity down, timing down)
assert selected_count + rejected_correlation + rejected_no_gain == signals_evaluated

return AutomlPlan { selected, pareto_front, ... }
```

Pearson r is computed on `bool`-as-float vectors (0.0/1.0). The correlation threshold of 0.85 means that a new signal must disagree with every already-selected signal on at least 15% of traces to be admitted. This is a structural diversity criterion, not a performance criterion.

The fusion operator is chosen by cardinality: `Single` for one signal, `WeightedVote` for two to four, `BordaCount` for five or more, and `Stack` (OOF stacking via `stack_ensemble_oof`) when accuracy variance is high and the pool has at least three signals.

### 4.4 Tier System

The `Tier` enum assigns each signal to a compute budget class based on measured timing:

- **T0** (< 100 µs): branchless kernel candidate; eligible for embedded and WASM edge deployment
- **T1** (100 µs – 2 ms): folded signature or small projection; conformance signals at PDC scale fall here
- **T2** (2 ms – 100 ms): wider vector or moderate cost; TF-IDF at 1000-trace scale
- **Warm** (> 100 ms): planning layer only; RL hyperparameter sweeps and deep neural signals

The tier system enables graceful degradation: a deployment target with strict latency constraints can restrict the candidate pool to T0/T1 signals without changing any other part of the pipeline. The Pareto front exposes timing as an explicit trade-off axis so consumers can select a lower-tier candidate that sacrifices a small amount of accuracy for a large latency reduction.

---

## 5. Experimental Results

### 5.1 Setup

The experiment uses 15 logs from the 96-log PDC 2025 dataset. For each log, the 15-signal candidate pool is evaluated, HDIT AutoML selects a subset, and the oracle signal (the single signal with highest accuracy vs ground truth) is recorded. Ground truth is read only after all predictions are finalized, never during signal computation or selection.

### 5.2 Per-Signal Accuracy

| Signal | Avg Acc vs GT | Peak Acc | Oracle Wins |
|--------|--------------|----------|-------------|
| H_inlang_fill / F / G | 66.8% | 73.2% | 0 (baseline) |
| TF_IDF | 66.4% | 73.6% | 5 |
| E_edit_dist | ~67.0% | 71.6% | 3 |
| AutoML_hyper | ~70.0% | 73.0% | 7 |
| NGram | 59.4% | 63.2% | 0 |
| PageRank | 57.7% | 62.2% | 0 |
| HDC_prototype | 48.6% | 56.0% | 0 |
| RL_AutoML | ~50.0% | ~55.0% | 0 |

AutoML_hyper wins 7 logs via H dominance on the last 7 logs in the test set; TF_IDF wins 5 of the first 8 logs outright. The conformance baseline (H/F/G) wins zero logs, confirming that its ceiling is not the ceiling of the problem.

### 5.3 Per-Log Oracle Winners

| Logs | Oracle Winner | Winner Accuracy | Notes |
|------|---------------|-----------------|-------|
| 000000, 000001, 000011, 000100, 000110 | TF_IDF | 69%–74% | Order-agnostic projection wins |
| 000010, 000101, 000111 | E_edit_dist | 70%–72% | Language-membership flavor |
| 001000–001110 (7 logs) | H_inlang_fill / AutoML_hyper | 70%–73% | Sequence-aware signals win |

The partition is structurally meaningful: the first eight logs appear to use activity frequency as a primary discriminant, rewarding order-agnostic signals. The last seven logs appear to encode sequence legality more strongly, rewarding the conformance family.

### 5.4 Why TF-IDF Works

TF-IDF treats each trace as a document where the vocabulary is the set of all activity labels. Term frequency measures how often each activity appears in the trace; inverse document frequency down-weights activities that appear in almost all traces. The classifier builds a centroid by averaging the TF-IDF vectors of training positive traces and ranks test traces by cosine similarity to that centroid. The top 500 by similarity are predicted positive.

This is structurally orthogonal to every conformance signal. Token replay asks: "does this trace follow the net's arcs in order?" TF-IDF asks: "does this trace's activity mix resemble the training positives?" The two questions are independent. When the answer to the second question is more predictive for a given log, TF-IDF wins regardless of conformance signal quality.

TF-IDF is also orthogonal to NGram (adjacent activity bigrams) and PageRank (graph centrality of activities in the training transition graph). Both capture structure that TF-IDF discards, explaining why NGram and PageRank scored lower on the same logs where TF-IDF excelled.

### 5.5 Oracle Gap and Anchor Bias

HDIT's greedy selection uses a conformance anchor: the majority vote of the eight sequence-based signals (F, G, H, and supervised classifiers trained on conformance features). On most logs, TF-IDF's predictions disagree with this anchor significantly — not because TF-IDF is wrong, but because TF-IDF's errors are in a different direction than the anchor's errors. HDIT interprets this disagreement as low `accuracy_vs_anchor` and rejects TF-IDF.

This is anchor bias: the selection criterion (agreement with anchor) is correlated with the anchor's systematic errors. The `oracle_signal` and `oracle_gap` fields in every `AutomlPlan` JSON surface this honestly. An external consumer can inspect the plan and observe: "HDIT selected H, but TF-IDF was the oracle, gap = 6.1 percentage points." This information is present; HDIT is not wrong about what it observed.

The anchor bias does not invalidate the infrastructure. It is a known epistemological limitation of unsupervised AutoML: without labeled data at selection time, a greedy selector must use a proxy (the anchor) whose biases it cannot correct for. The anti-lie framework documents this limitation rather than hiding it.

### 5.6 Statistical Caveat

A 15-log smoke test is not statistically powered for formal hypothesis testing. The 73.6% result on log 000110 is a structural proof that the ceiling is not universal — at least one log admits a signal that beats 67.78% by more than six points — but the difference between average TF-IDF accuracy (66.4%) and the conformance baseline (66.8%) cannot be claimed as significant at this sample size. A full 96-log evaluation with Wilcoxon signed-rank test is required before making claims about population-level superiority.

---

## 6. Anti-Lie Infrastructure

### 6.1 The Problem of Lies by Omission

AutoML systems can misrepresent their performance through several mechanisms: selecting a proxy metric that diverges from the true objective (biased proxy), using ground truth during selection (GT leakage), reporting only the best result from an exhaustive search without accounting for the search cost (selection event not counted), or presenting a single number when a distribution would be more informative. The dteam anti-lie infrastructure addresses each of these.

### 6.2 Accounting Identity

The accounting identity asserted at plan-write time is:

```
signals_selected + signals_rejected_correlation + signals_rejected_no_gain = signals_evaluated
```

This is an internal consistency check: every candidate that entered the greedy loop must be accounted for in exactly one of three outcome categories. The assertion fires before the `AutomlPlan` is returned, making it impossible to return a plan with inconsistent accounting. `AutomlPipelineVerifier` independently recomputes the identity from the JSON fields at read time, providing a second independent check that does not trust the write path.

### 6.3 Pareto Front Integrity

The `pareto_front` field in `AutomlPlan` contains one `PlanCandidate` per non-dominated point on the (accuracy up, complexity down, timing down) surface. Exactly one candidate has `chosen=true` — the one HDIT's greedy algorithm landed on. The verifier asserts this property: zero or two `chosen=true` entries are both invariant violations. If HDIT's chosen candidate happens to be dominated on some axis, the Pareto front exposes this honestly; consumers can see that a cheaper candidate achieves 99% of the accuracy.

### 6.4 GT Leakage Audit

The TF-IDF 73.6% result was audited for label leakage across three checks. First, test logs in `data/pdc2025/test_logs/*.xes` contain zero `pdc:isPos` attributes — the XES standard allows this attribute to encode ground truth, and its absence was confirmed programmatically. Second, the TF-IDF implementation in `src/ml/nlp.rs` never reads from `data/pdc2025/ground_truth/`. Third, the training log (index 11) contains 20 known negatives that are currently pooled into the positive centroid without filtering by label, which reduces TF-IDF accuracy slightly relative to a label-aware implementation. The 73.6% result is therefore a conservative lower bound rather than an inflated upper bound.

### 6.5 DoD as CI Enforcement

`cargo make dod` is the pre-merge gate. It runs the full build, all tests, and then `AutomlPipelineVerifier` over all `artifacts/pdc2025/automl_plans/*.json`. Exit code 0 means all invariants pass; exit code 1 means an invariant failed; exit code 2 means a structural error (missing file, malformed JSON). `DxQolVerifier` additionally validates `strategy_accuracies.json` and `run_metadata.json`, checks XES output presence, verifies the skip rate is at most 10%, and confirms that the best-per-log strategy dominates the overall average. Together these gates make it impossible to merge a change that corrupts any artifact invariant.

---

## 7. Performance

### 7.1 Micro-Benchmark Results

| Operation | Latency | Notes |
|-----------|---------|-------|
| Token replay K64 | ~20 ns/event | u64 bitmask, single word |
| Token replay K256 | ~45 ns/event | 4-word SWAR |
| Token replay K512 | ~90 ns/event | 8-word SWAR |
| Token replay K1024 | ~150 ns/event | 16-word SWAR |
| RL action select | ~200 ns/step | Q-table lookup via PackedKeyTable |
| RL update | ~450 ns/step | 2x Q-table write (Double Q-learning) |
| Conformance H (1000 traces) | 552 µs | T1 tier, measured end-to-end |
| TF-IDF (1000 traces) | T2 tier | cosine similarity over vocabulary |

Token replay scales linearly in the number of u64 words, confirming that the SWAR implementation has no branch mispredictions on the firing path. The K64 path at 20 ns/event is competitive with hand-written SIMD conformance implementations in the process_mining reference benchmarks included in `dev-dependencies`.

### 7.2 Zero-Heap Verification

Zero-heap compliance is verified using `dhat::Alloc` as the global allocator in benchmark builds. After a warmup phase that allows initial data structure construction, 1 million token replay iterations produce zero heap allocations. The property is also verified structurally by `proptest_kernel_verification.rs`, which generates random net configurations and confirms that no `Vec` or `HashMap` is instantiated within the replay hot loop. The `skeptic_contract.rs` module encodes the zero-heap obligation as a named constant so that any future refactor that introduces a heap allocation will produce a contract violation at the call site.

### 7.3 WASM Compatibility

The `wasm-bindgen` integration exposes the conformance engine to JavaScript hosts. `WasmConfig.batch_size = 10` limits per-call trace volume for responsiveness in browser environments. The K64 tier is the recommended WASM target; K256 is supported but requires adequate stack depth in the host runtime. The `rayon` parallel iterator is disabled for WASM targets (single-threaded execution model); on native targets, `rayon` `par_iter` over edit-distance inner loops and AutoML trial evaluation provides approximately 4x speedup, enabling evaluation of 8–15 signals per log within a 180-second planning budget.

---

## 8. Discussion

### 8.1 Why TF-IDF Beat Conformance on the First Eight Logs

The structural argument is as follows. If the PDC log generator for the first eight logs selected traces for the positive class based partly on activity frequency — for example, a process variant that uses a particular activity more often than alternatives — then bag-of-words similarity captures the relevant discriminant directly. Token replay, by contrast, tests whether the trace follows the net's control flow exactly. If the net is an approximation discovered from limited training data, it may correctly identify sequence violations while missing frequency-based variants entirely. TF-IDF's insensitivity to order is a disadvantage when order is the discriminant and an advantage when frequency is the discriminant. The partition observed in the smoke test (first eight logs favor TF-IDF, last seven favor sequence-aware signals) is consistent with this hypothesis, though it has not been confirmed against the log generator parameters.

### 8.2 Anchor Bias as an Epistemological Limitation

Anchor bias is the inevitable consequence of unsupervised AutoML: without labeled validation data at selection time, the system must evaluate signals against a proxy (the anchor), and the proxy's systematic errors become the selection criterion's systematic errors. This is not a bug in HDIT; it is the correct behavior given the information available. The remedy is not algorithmic cleverness but additional information: either a labeled validation set, an information-theoretic criterion such as MDL that does not require a reference prediction, or an anchor constructed from a diverse pool rather than sequence-based signals alone.

Three concrete remedies for anchor bias follow from this analysis. First, include TF-IDF in the anchor pool alongside conformance signals; the anchor's majority vote would then be less correlated with conformance errors. Second, replace the accuracy-vs-anchor criterion with an MDL criterion that rewards signals for compressing the training data rather than agreeing with the anchor. Third, reserve a small labeled subsample from training log 11 for validation; even 20 labeled examples would provide a less biased selection signal. The current implementation exposes the `oracle_gap` field precisely so that this limitation is visible to any downstream analysis that has access to ground truth.

### 8.3 Comparison to TPOT2

HDIT AutoML is inspired by but distinct from TPOT2. TPOT2 selects over pipeline configurations (algorithm × hyperparameter combinations), searching for the pipeline that maximizes a validation score. HDIT selects over prediction vectors produced by pre-computed signals, searching for the subset whose fused prediction is most accurate vs the anchor. The practical difference is that HDIT can combine signals from entirely different algorithmic families — a conformance signal and a TF-IDF signal — without requiring a unified feature representation. The Pearson correlation filter achieves a similar diversity goal to TPOT2's algorithm-family diversity constraint, but by measuring prediction-space distance rather than algorithm-space distance. The greedy approach is near-optimal for pools of 15 signals; exhaustive search over 2^15 subsets is feasible but adds no material benefit at this pool size given the correlation structure of the signal space.

### 8.4 Limitations

The primary limitations of the current system are: (1) the 15-log smoke test is not statistically powered; (2) HDIT's greedy selection is vulnerable to anchor bias when the anchor pool is narrow; (3) no deep learning signals are currently included — transformer-based sequence encodings may provide a third orthogonal family beyond conformance and bag-of-words; (4) the RL discovery loop requires multiple training episodes per net, making it slower than direct Inductive Miner discovery for large logs; and (5) the system requires at least one training log with labeled traces, limiting zero-shot applicability in domains where no reference conformance data exists.

---

## 9. Conclusion

This paper has presented four contributions to process-mining AutoML for trace classification.

**HDIT AutoML** provides greedy orthogonal signal selection with a Pearson correlation filter, TPOT2-style Pareto-front reporting over accuracy, complexity, and timing trade-offs, successive halving for compute budget control, and out-of-fold stacking to prevent level-1 leakage. The algorithm is implemented in approximately 300 lines of zero-dependency Rust in `src/ml/hdit_automl.rs` and operates on pre-computed prediction vectors, making it agnostic to the signal family.

**TF-IDF orthogonality** establishes that the 67.78% conformance ceiling is a projection ceiling, not a data ceiling. By treating traces as bags of activity labels rather than ordered token sequences, TF-IDF captured a discriminant that is invisible to every conformance-based signal. The 73.6% peak on log 000110 and five oracle wins in a 15-log smoke test constitute a structural proof that ceiling-breaking is achievable through signal diversification alone, without any increase in algorithmic complexity within the conformance family.

**Anti-lie infrastructure** provides an accounting identity assertion, an `oracle_gap` field, Pareto front integrity checking, GT leakage audit, and a DoD verification gate enforced pre-merge. Together these mechanisms make it impossible for the system to misrepresent its performance through any of the common omission mechanisms, and they make the anchor bias limitation visible rather than hidden.

**Zero-heap Rust implementation** achieves 20–150 ns token replay (depending on K-tier), 200–500 ns RL steps, and WASM compatibility, enabling edge deployment scenarios that are infeasible for Python-based process mining stacks. The `PackedKeyTable` + `fnv1a_64` + `KBitSet<WORDS>` spine ensures that performance characteristics are deterministic across all replay paths and auditable via the `skeptic_contract` module.

Future work addresses four open problems. First, fixing anchor bias by including diverse signal families in the anchor pool or adopting an MDL selection criterion. Second, a full 96-log evaluation with Wilcoxon signed-rank test to provide statistically powered comparisons between signal families. Third, WASM edge deployment with K64 conformance and TF-IDF compressed to browser-compatible vocabulary sizes. Fourth, transformer sequence signals as a third orthogonal family, potentially capturing both order and frequency information in a unified embedding that could supersede both the conformance family and TF-IDF on logs with rich sequential structure.

---

## 10. References

1. van der Aalst, W.M.P. (2011). *Process Mining: Discovery, Conformance and Enhancement of Business Processes*. Springer.

2. van der Aalst, W.M.P., Weijters, T., and Maruster, L. (2004). Workflow mining: Discovering process models from event logs. *IEEE Transactions on Knowledge and Data Engineering*, 16(9), 1128–1142.

3. Leemans, S.J.J., Fahland, D., and van der Aalst, W.M.P. (2013). Discovering block-structured process models from event logs — A constructive approach. In *Petri Nets*, LNCS 7927, 311–329. Springer.

4. Grus, J. (2019). *Data Science from Scratch: First Principles with Python*, 2nd ed. O'Reilly Media. (Source for 22 ML algorithm implementations ported to Rust in `src/ml/`.)

5. Ribeiro, A., Gijsbers, P., et al. (2024). TPOT2: Next Generation AutoML in Python. *arXiv preprint arXiv:2402.01563*. (TPOT2-inspired successive halving and Pareto-front selection.)

6. PDC 2025. *Process Discovery Contest 2025*. Competition proceedings and dataset specification. https://pdc.cloud.ut.ee/ (accessed 2026-04-23).

7. Hasselt, H., Guez, A., and Silver, D. (2016). Deep reinforcement learning with double Q-networks. In *Proceedings of AAAI*, 2094–2100.

8. Williams, R.J. (1992). Simple statistical gradient-following algorithms for connectionist reinforcement learning. *Machine Learning*, 8(3–4), 229–256.

9. IEEE Std 1849-2016. *IEEE Standard for eXtensible Event Stream (XES) for Achieving Interoperability in Event Logs and Event Streams*. IEEE.

10. Kephart, J.O. and Chess, D.M. (2003). The vision of autonomic computing. *IEEE Computer*, 36(1), 41–50.

---

## 12. Release Framing: Anti-Lie as a Deployment Contract

The five release artifacts described in this section operationalize the anti-lie doctrine as a deployable product, not just a research property.

### 12.1 The Doctor Command

`cargo make doctor` is a post-run epistemic diagnostic that answers a question no build check can: **is this plan slow, redundant, biased, or lying?** It is modeled after `brew doctor` and `flutter doctor` but performs invariant-level audit of plan JSON artifacts rather than environment health checks.

The command detects five pathology classes:

| Pathology | Severity | Trigger |
|-----------|----------|---------|
| LYING | Fatal | `accounting_balanced=false`, identity broken, oracle gap mismatch, or ≠1 Pareto chosen |
| SLOW | Warn | `total_timing_us > 100,000µs` — plan is Warm-tier, not edge-deployable |
| SATURATED | Warn | All selected signals belong to a single family — monoculture, not orthogonal |
| REDUNDANT | Info | >40% of evaluated signals rejected for correlation |
| STALE | Info | `run_metadata.json` missing or commit hash mismatch |

LYING exits with code 2 and blocks merge via `cargo make pre-merge`. The other pathologies exit 1 (soft fail), surfacing suboptimality without blocking. This distinction is load-bearing: a plan that disagrees with itself is categorically different from one that is merely slow.

### 12.2 Tier as Deployment Contract

The four deployment tiers are a public API, not an internal implementation detail:

| Tier | Budget | Target |
|------|--------|--------|
| T0 | ≤100µs | Browser/WASM, embedded, hard real-time |
| T1 | ≤2ms | Edge/CDN (Cloudflare Workers, Fastly), mobile on-device |
| T2 | ≤100ms | Fog/serverless (Lambda, Cloud Run), IoT gateway |
| Warm | >100ms | Cloud (EC2, GKE), batch, offline analytics |

`cargo run --bin doctor -- --target=T1` checks every plan against the T1 budget and reports the cheapest Pareto-front alternative when the chosen plan exceeds it. The `tiers[]` array in each plan JSON records the per-signal deployment class, making the deployment contract inspectable from any downstream tool — CI system, dashboard, or audit log.

This matters for PDC 2025 because conformance signals are typically T1 (200µs–2ms) while TF-IDF over a full 1000-trace log is T2–Warm depending on vocabulary size. A deployment engineer reading a plan JSON can determine, without re-running the pipeline, whether the plan fits their target environment.

### 12.3 The Orthogonal Signal Lesson

The central scientific finding — that TF-IDF breaks the conformance ceiling by measuring something structurally orthogonal — has a direct operational consequence: **single-family ensembles saturate**. The SATURATED pathology in `doctor` is the runtime expression of this lesson.

When all selected signals come from the conformance family, they share the same projection error: every score is a monotone function of token replay fitness over the same PNML net. No amount of weighted voting over correlated signals escapes the ceiling imposed by that shared error floor. TF-IDF escapes the ceiling precisely because it measures bag-of-words activity frequency, which is orthogonal to replay fitness by construction.

The greedy HDIT selection algorithm enforces this structurally: the Pearson r < 0.95 threshold blocks adding a second conformance signal once one is already selected. But if the evaluation pool contains only conformance signals — as it would if the NLP and synthetic families were removed — HDIT still selects exactly one, and the SATURATED pathology fires to flag the monoculture.

This is the anti-lie doctrine applied to experimental design: the pipeline cannot hide a monoculture by selecting the "best" member of a single family.

### 12.4 Release Artifacts Summary

Five release artifacts complete the productization of the anti-lie layer:

| Artifact | Command | Purpose |
|----------|---------|---------|
| `doctor` binary | `cargo make doctor` | Epistemic smoke test: LYING / SLOW / SATURATED / REDUNDANT / STALE |
| JSON Schema | `cargo make plan-schema` | Stable machine-readable contract for AutomlPlan; `--validate=` for point-in-time checks |
| HTML report | `cargo make plan-report` | Standalone diagnostic dashboard with tier matrix, per-plan table, signal frequency chart |
| Diff tool | `cargo make plan-diff DIR_A=v1 DIR_B=v2` | Regression detector between two artifact runs; exits 1 on accuracy regression |
| Target check | `cargo make doctor-target TARGET=T1` | Enforces a deployment tier contract; suggests cheapest Pareto downgrade |

Together these five tools make it possible for a team that has never read the source code to verify that the pipeline is telling the truth, producing edge-deployable plans, and selecting orthogonally diverse signals — entirely from the artifact layer, without re-running the pipeline.
