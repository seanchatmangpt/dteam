# Using the Definition of Done Verifier

**Learning goal:** Run `cargo make dod`, understand what each of the four phases checks, and know which exit code means what.

By the end of this tutorial you will:
- Know the difference between a HARD FAIL and a SOFT FAIL
- Understand what Phase 3 (pipeline artifacts) and Phase 4 (DX/QoL) are checking
- Know how to deliberately trigger a soft fail and how to recover
- Have a clear workflow for using DoD as a pre-push gate

---

## What is the Definition of Done?

The `dod` binary (`src/bin/dod.rs`) is a four-phase invariant verifier. It answers one question: is the project in a state that can honestly be called "done"?

It is not a test runner. It runs `cargo check` and `cargo test --lib` as sub-commands, then inspects the artifact directory for pipeline outputs. The entire check typically completes in under two minutes.

---

## Exit codes

| Code | Meaning | When it happens |
|---|---|---|
| `0` | PASS | All four phases green |
| `1` | SOFT FAIL | Build and tests pass, but pipeline artifacts are missing or stale |
| `2` | HARD FAIL | Compilation broken, tests broken, or a critical invariant violated |

Exit code 1 is expected after a fresh clone or before you have run the main pipeline. It is not an emergency — it means the code is correct but the pipeline has not been run yet.

Exit code 2 must be resolved before merging.

---

## Step 1 — Run in dev mode (faster)

`cargo make dod-dev` compiles the `dod` binary in debug mode, which is faster to build but slower to execute. Use it during iteration.

```sh
cargo make dod-dev
```

`cargo make dod` uses `--release` — slower to compile, faster at runtime. Use it for final pre-merge checks.

---

## Step 2 — Phase 1: Compilation

The first section of the output looks like:

```
── Phase 1: Compilation ──────────────────────────────────────────
  ✓ cargo check                            clean
```

This runs `cargo check --quiet` internally. If it fails:

```
  ✗ cargo check                            error[E0308]: mismatched types
```

The first error line is displayed inline. Fix the compile error and re-run — this is a **HARD FAIL** (exit 2). Nothing else in the report is meaningful when the build is broken.

---

## Step 3 — Phase 2: Tests

```
── Phase 2: Tests ────────────────────────────────────────────────
  ✓ cargo test --lib                       142 tests passed
```

This runs `cargo test --lib` (library unit tests only, not integration tests). The reported count is the sum of `N passed` lines across all test result blocks.

A test failure triggers **HARD FAIL** (exit 2):

```
  ✗ cargo test --lib                       test conformance::tests::replay_basic ... FAILED
```

Run `cargo test --lib` directly to see the full failure output with assertion details.

---

## Step 4 — Phase 3: Pipeline artifacts (AutoML plans)

```
── Phase 3: Pipeline artifacts (AutoML plans) ────────────────────
  ⚠ no plan JSON files found              run `cargo run --bin pdc2025 --release` ...
```

This phase scans the artifact directory for `.json` plan files produced by the main pipeline binary. If none exist (for example on a fresh clone), it emits a warning rather than a hard failure.

When plan files are present the verifier checks each one for invariants:

- `accounting_balanced = true` — every evaluated signal is accounted for (selected + rejected_correlation + rejected_no_gain = evaluated)
- Exactly one `"chosen": true` row in the Pareto front
- All required fields present and in range

A failed plan invariant is a **HARD FAIL** (exit 2), because it means the pipeline produced a dishonest artifact — the accounting identity does not hold.

**When no plans are present (exit 1):**

This is normal immediately after a fresh clone or after a `cargo clean`. Run the main pipeline first:

```sh
cargo run --bin pdc2025 --release
```

Then re-run DoD. The warning will become a green check.

---

## Step 5 — Phase 4: DX / QoL artifacts

```
── Phase 4: DX / QoL artifacts ──────────────────────────────────
  ✓ strategy_accuracies.json               all 20 strategies present, values in [0,1]
  ✓ run_metadata.json                      git_commit + timestamp + counts present
  ✓ classified output XES files            at least one .xes written
  ✓ skip rate ≤ 10%                        actual 2.3%
  ✓ best_per_log dominates all strategies  invariant holds
```

This phase verifies developer-experience artifacts:

**`strategy_accuracies.json`** — produced by the pipeline to record how each of the 20 classification strategies performed. All 20 keys must be present; all values must be in [0, 1]. Missing strategies indicate the pipeline exited early.

**`run_metadata.json`** — records `git_commit`, timestamp, and trace/log counts. If this file is missing the pipeline did not complete a full run.

**Classified output XES files** — at least one output `.xes` file must exist. If none are found, the pipeline classified nothing.

**Skip rate** — the fraction of traces where the classifier emitted no prediction (skipped). Must be at most 10%. A high skip rate means the classifier is silently refusing to classify large portions of the log.

**`best_per_log` dominates** — an arithmetic invariant: the best-per-log accuracy must be at least as high as the accuracy of any individual strategy on every log. A violation means the aggregation is incorrectly computed.

Phase 4 failures are treated as **SOFT FAIL** (exit 1) when Phase 1 and 2 pass, because they indicate the pipeline has not been run, not that the code is broken.

---

## Step 6 — Deliberately trigger a soft fail

On a fresh clone, before running the pipeline:

```sh
cargo make dod-dev
```

Expected output ends with:

```
── Verdict ──────────────────────────────────────────────────────
  ⚠ Definition of Done: SOFT FAIL
  build+tests OK but pipeline artifacts have issues
```

Exit code will be `1`. This is expected and not a defect. It tells you the code compiles and passes tests, but the pipeline artifacts do not yet exist.

To recover from soft fail, run the pipeline:

```sh
cargo run --bin pdc2025 --release
cargo make dod-dev
```

The second DoD run should show exit 0 if everything is in order.

---

## Step 7 — DoD in your workflow

### Before `git push`

```sh
cargo make dod-dev
echo "Exit code: $?"
```

- Exit 0 → safe to push
- Exit 1 → push is acceptable if you are pushing work-in-progress; document that pipeline artifacts are not yet present
- Exit 2 → do not push; fix the compile or test failure first

### Pre-merge gate

The `pre-merge` task runs DoD as part of a full CI pipeline:

```sh
cargo make pre-merge
```

This runs: `ci` (check + test + lint) → `doc` → `dod` (release mode). All three must pass before a merge is approved.

### Interpreting the full verdict block

A clean PASS looks like:

```
── Verdict ──────────────────────────────────────────────────────
  ✓ Definition of Done: PASS
  compile ✓  tests ✓  12 plans ✓  DX/QoL artifacts ✓
```

The plan count in the summary comes from Phase 3's scan. A zero there while showing PASS would be a contradiction — the DoD design prevents this by treating zero plans as a warning rather than a silent pass.

---

## Quick reference

| Command | Speed | Use for |
|---|---|---|
| `cargo make dod-dev` | Fast build, slower binary | Iteration during development |
| `cargo make dod` | Slow build (--release), fast binary | Pre-merge, CI |
| `cargo make pre-merge` | Full CI + dod | Final gate before merge |

| Phase | Hard fail? | Soft fail? |
|---|---|---|
| 1 — Compilation | Yes (exit 2) | — |
| 2 — Tests | Yes (exit 2) | — |
| 3 — Plan invariant violated | Yes (exit 2) | — |
| 3 — No plan files found | — | Yes (exit 1) |
| 4 — DX/QoL artifacts missing | — | Yes (exit 1) |

---

## Next steps

- Explore `src/agentic/ralph/verifier.rs` for `AutomlPipelineVerifier` and `DxQolVerifier` — the types that implement the Phase 3 and 4 checks.
- See `src/bin/dod.rs` for the full four-phase implementation.
- Read `AGENTS.md` for the complete contributor guide and authoritative command reference.
