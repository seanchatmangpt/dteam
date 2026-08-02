# How to Run the PDC 2025 Pipeline on Custom Logs

## Prerequisites

- Release build of the `pdc2025` binary: `cargo build --bin pdc2025 --release`
- PNML models must have **64 or fewer places** for the bitmask fast path. Models with >64 places still run but conformance strategies F, G, and H fall back to fitness-only scoring.

---

## 1. Create the directory layout

Create the following structure under the repo root:

```
data/pdc2025/
  test_logs/       ← your XES test logs (one file per stem)
  training_logs/   ← optional supervised training logs (suffix-named; see section 3)
  ground_truth/    ← XES files with pdc:isPos boolean attribute per trace
  models/          ← PNML workflow nets (one file per stem)
artifacts/pdc2025/ ← created automatically by the pipeline
```

Verify the layout:

```sh
cargo make pdc-check-data
```

All four directories must be present. The `pdc-check-data` task exits with a non-zero status and prints `✗ <dir> missing` for any absent directory.

---

## 2. Name your files

The pipeline is stem-based. A stem is the shared filename prefix across all files for one log.

For a stem named `mystem`:

| File | Path |
|---|---|
| Test log | `data/pdc2025/test_logs/mystem.xes` |
| Ground truth | `data/pdc2025/ground_truth/mystem.xes` |
| PNML model | `data/pdc2025/models/mystem.pnml` |

**Training logs** use numeric suffixes appended directly to the stem (no separator):

| Suffix | Path | Meaning |
|---|---|---|
| `_00` | `data/pdc2025/training_logs/mystem_00.xes` | Labelled training log, split 0/0 |
| `_10` | `data/pdc2025/training_logs/mystem_10.xes` | Labelled training log, split 1/0 |
| `_11` | `data/pdc2025/training_logs/mystem_11.xes` | Labelled training log, split 1/1 |

Training logs are optional. When present, the pipeline uses them for supervised strategy transfer. The `pdc:isPos` boolean trace attribute must be present in both training logs and ground truth files.

---

## 3. Run the pipeline

**Full run — all logs in `test_logs/`:**

```sh
cargo make pdc
```

**Single log by stem:**

```sh
STEM=mystem cargo make pdc-stem
```

**Direct binary invocation:**

```sh
cargo run --bin pdc2025 --release -- --stem=mystem
```

**With verbose logging:**

```sh
RUST_LOG=info cargo make pdc
```

Info-level logging prints per-trace classification decisions, strategy scores, and HDIT signal selection details.

---

## 4. Read strategy_accuracies.json

After the run, `artifacts/pdc2025/strategy_accuracies.json` contains accuracy scores for each strategy evaluated against ground truth:

| Key | Strategy | Description |
|---|---|---|
| `f` | `classify_exact` | Bitmask exact-match against language enumeration |
| `g` | `fitness_rank` | Rank traces by token-replay fitness score, threshold at 0.5 |
| `h` | `in_lang_fill` | Language membership fill: positive if trace is in the enumerated language |
| `hdc` | hyperdimensional | HDC vector similarity classification |
| `automl` | HDIT fusion | Best trial from HDIT AutoML hyperparameter search (only present if `[automl] enabled = true`) |
| `best_per_log` | oracle | Per-log best strategy; theoretical ceiling, not deployable |

Scores are fractions in `[0.0, 1.0]`. A score of `1.0` means every trace in the test log was classified correctly against ground truth.

---

## 5. Inspect the HDIT plans

When `[automl] enabled = true`, each stem produces a plan file:

```
artifacts/pdc2025/automl_plans/<stem>.json
```

Key fields:

| Field | Meaning |
|---|---|
| `selected` | List of signal names chosen by HDIT for this stem's fusion |
| `fusion` | Fusion method applied (majority vote, weighted, etc.) |
| `plan_accuracy_vs_gt` | Accuracy of this plan against ground truth |
| `oracle_gap` | Difference between `best_per_log` and `plan_accuracy_vs_gt`; how far from the per-log oracle ceiling |

A small `oracle_gap` (< 0.05) indicates the AutoML plan is near-optimal for this stem. A large gap suggests the available signals do not cover the discriminative structure of the log — consider adding training logs or increasing `budget`.

---

## 6. Troubleshooting

**XES parse error on a custom log**

The `XESReader` requires UTF-8 encoded files. Convert to UTF-8 before running:

```sh
iconv -f <source-encoding> -t UTF-8 mylog.xes -o mylog_utf8.xes
```

**`strategy="ensemble_only"` panics at startup**

If `[automl]` is enabled and `strategy = "ensemble_only"`, the binary panics immediately with:

```
AutoML config lie: strategy="ensemble_only" has been removed.
```

Change `strategy` to `"random"` or `"grid"` in `dteam.toml`.

**Models with >64 places**

Conformance strategies F (`classify_exact`), G (`fitness_rank`), and H (`in_lang_fill`) rely on the u64 bitmask fast path, which requires the model to have ≤ 64 places. For larger models, these strategies fall back to fitness-only scoring and will not use language enumeration. If your accuracy for F/G/H is unexpectedly low, verify place count in your PNML.

Count places in a PNML file:

```sh
grep -c '<place ' data/pdc2025/models/mystem.pnml
```
