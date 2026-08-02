# 01 — PDC 2025 Repair

## Starting task

User asked the assistant to find all `.tex` papers with "RL" in them (13 unique
files across docs/thesis, docs/thesis_v2, and root-level whitepapers), then to
verify whether the code referenced in those papers actually works.

## Findings

All 10 paper-claimed implementations existed:

| Claim | File | Status |
|---|---|---|
| `ExecutionManifest` | `src/lib.rs:444` | ✓ |
| `SkepticHarness` (named `Skeptic` in code) | `src/skeptic_harness.rs:161` | ✓ |
| `train_with_provenance()` | `src/automation.rs:35` | ✓ |
| `token_replay()` | `src/conformance/mod.rs:263` | ✓ |
| `read_pnml()` | `src/io/pnml.rs:9` | ✓ new |
| `write_classified_log()` | `src/io/xes_writer.rs:7` | ✓ new |
| `XESReader::new()` / `read()` | `src/io/xes.rs:77,228` | ✓ |
| 5 RL agents | `src/reinforcement/*.rs` | ✓ all present |
| `PackedKeyTable` | `src/utils/dense_kernel.rs:348` | ✓ |
| `AutonomicKernel` / `DefaultKernel` | `src/autonomic/kernel.rs:4,51` | ✓ |

## Bug found

`cargo make test-all` failed:

```
error[E0425]: cannot find function `score_log` in this scope
  --> src/bin/pdc2025.rs:72:22
```

`score_log` was called on line 72 but never defined. Initial attempt to add it
failed because the file was actually 344 lines long (truncated Read), and a
complete `score_log` already existed at line 214 using `NetIndex`.

## Repair

Restored original imports (`HashMap`, `AttributeValue`, `Trace`) and removed
the duplicate `score_log` insertion. Build passed clean. All 82 tests passed.

## Separate gaps identified

1. Unused `warn` import in `src/automation.rs:7` — lint warning.
2. `_beta` and `_lambda` parameters in `train_with_provenance_projected` were
   declared but unused. The standard path uses them; the projected (fast) path
   silently ignored them. The RL agent selected actions but `agent.update()`
   was never called — no reward signal ever reached the learner.
3. Naming: papers say `SkepticHarness`; code has `struct Skeptic` in
   `src/skeptic_harness.rs`. Cosmetic.

## Files touched

- `src/bin/pdc2025.rs` — verified intact after accidental edit
- (Next document: `src/automation.rs` repair)
