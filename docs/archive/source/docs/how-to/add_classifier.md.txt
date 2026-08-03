# Add a New Supervised Classifier to the ML Pool

This guide walks through adding a new supervised classifier to the transductive pipeline in
`src/ml/pdc_supervised.rs`. The pipeline trains on pseudo-labeled traces and predicts every
trace in the same pass.

## Prerequisites

- Classifier signature: `pub fn classify(train: &[Vec<f64>], labels: &[bool], test: &[Vec<f64>]) -> Vec<bool>`
- Edge cases to handle: empty training set returns `vec![false; test.len()]`
- Follow the pattern in `src/ml/knn.rs` lines 12-44

---

## Step 1 — Create `src/ml/my_classifier.rs`

The minimum viable implementation:

```rust
/// Classify test points using your algorithm.
///
/// - Empty training set → every test point returns `false`
pub fn classify(train: &[Vec<f64>], labels: &[bool], test: &[Vec<f64>]) -> Vec<bool> {
    if train.is_empty() {
        return vec![false; test.len()];
    }
    // your algorithm here
    test.iter().map(|_point| false).collect()
}
```

Ensure the returned `Vec<bool>` always has exactly `test.len()` elements.

---

## Step 2 — Register the module in `src/ml/mod.rs`

Add one line to the alphabetically sorted pub-mod list. The existing list is at
`src/ml/mod.rs` lines 1-36. Insert after the neighboring entry:

```rust
pub mod my_classifier;
```

Example insertion point (after `logistic_regression`, before `naive_bayes`):

```
// src/ml/mod.rs line 17 area
pub mod logistic_regression;
pub mod my_classifier;       // <-- add here
pub mod naive_bayes;
```

---

## Step 3 — Add the import to `src/ml/pdc_supervised.rs`

The import block is at `src/ml/pdc_supervised.rs` lines 9-12. Add your module:

```rust
use crate::ml::{
    decision_stump, decision_tree, gaussian_naive_bayes, gradient_boosting, knn, linear_regression,
    logistic_regression, my_classifier, naive_bayes, nearest_centroid, neural_network, perceptron,
};
```

---

## Step 4 — Add a field to `SupervisedPredictions`

The struct is at `src/ml/pdc_supervised.rs` lines 23-35. Add your field:

```rust
pub struct SupervisedPredictions {
    pub knn: Vec<bool>,
    pub naive_bayes: Vec<bool>,
    pub decision_tree: Vec<bool>,
    pub logistic_regression: Vec<bool>,
    pub gaussian_nb: Vec<bool>,
    pub nearest_centroid: Vec<bool>,
    pub perceptron: Vec<bool>,
    pub neural_net: Vec<bool>,
    pub gradient_boosting: Vec<bool>,
    pub decision_stump: Vec<bool>,
    pub linear_classify: Vec<bool>,
    pub my_classifier: Vec<bool>,   // <-- add here
}
```

Also add `my_classifier: vec![false; n]` to both early-return blocks at lines 63-75 and
the `n_test == 0` block at lines 142-155 in `run_supervised_transfer`.

---

## Step 5 — Call the classifier in `run_supervised`

The classifier chain runs from line 79 to line 113 in `src/ml/pdc_supervised.rs`. Add your
call after the last existing classifier (`linear_classify`):

```rust
// ── My Classifier ─────────────────────────────────────────────────────────
let my_classifier = my_classifier::classify(features, labels, features);
```

Then add it to the `SupervisedPredictions { ... }` struct literal at lines 114-127:

```rust
SupervisedPredictions {
    knn,
    naive_bayes,
    decision_tree,
    logistic_regression,
    gaussian_nb,
    nearest_centroid,
    perceptron,
    neural_net,
    gradient_boosting,
    decision_stump,
    linear_classify,
    my_classifier,   // <-- add here
}
```

Repeat the same call pattern inside `run_supervised_transfer` (lines 156-180), using
`train_features` and `train_labels` instead of `features` and `labels`.

---

## Step 6 — Register in `to_named_list`

The helper at `src/ml/pdc_supervised.rs` lines 191-205 returns a fixed-length list.
Add your entry at the end:

```rust
pub fn to_named_list(preds: &SupervisedPredictions) -> Vec<(&'static str, Vec<bool>)> {
    vec![
        ("knn", preds.knn.clone()),
        // ... existing entries ...
        ("linear_classify", preds.linear_classify.clone()),
        ("my_classifier", preds.my_classifier.clone()),  // <-- add here
    ]
}
```

The docstring says "exactly 11 entries" — update it to reflect the new count.

---

## Step 7 — Test

```bash
cargo test --lib
```

Target just the supervised module if iterating quickly:

```bash
cargo test --lib pdc_supervised
```

Confirm no regressions in the `test_empty_features_returns_default` and
`test_all_false_features` tests. If your classifier adds a field to `SupervisedPredictions`,
add a corresponding assertion to the default-check test.
