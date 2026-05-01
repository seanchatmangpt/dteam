# INSA DX / QOL Definition of Done

**DX/QOL is not polish. It is anti-drift infrastructure.**
**The system makes the correct path the easy path and the wrong path fail loudly.**

---

## 1. One command proves the project

```bash
just dx
```

Must run:
* fmt
* clippy
* unit tests
* property tests
* compile-fail tests
* golden wire tests
* replay tests
* layout gates
* access-drift JTBD
* bench smoke

If `just dx` passes, the project is locally sane. If not, the failure must be classified.

---

## 2. Standard command surface

Use `just` or `cargo xtask`.

Recommended:
```bash
just fmt
just lint
just test
just test-unit
just test-prop
just test-compile-fail
just test-golden
just test-replay
just test-jtbd
just bench-smoke
just layout
just dx
just clean
```

Optional deep gates:
```bash
just fuzz
just miri
just sanitizer
just bench-full
just truthforge
```

---

## 3. `cargo xtask` for project-specific workflows

Recommended structure:
```text
crates/
  xtask/
    src/
      main.rs
      commands/
        dx.rs
        layout.rs
        golden.rs
        replay.rs
        truthforge.rs
        new_kappa.rs
        new_jtbd.rs
```

This avoids shell-script entropy.

---

## 4. Error messages must be operational

Bad:
```text
test failed
```

Good:
```text
TruthforgeAdmissionFailed:
  fixture: access_drift_terminated_contractor
  stage: HEARSAY8 fusion
  expected: FusionComplete + SourceAuthoritative
  actual: SourceConflicts + FusionRequiresInspection
  missing source: badge_export_epoch
  next lawful motion: Retrieve
```

Every failure should answer:
* what failed
* where it failed
* why it failed
* what evidence was expected
* what lawful next action exists

---

## 5. Generators

### New KAPPA generator
```bash
cargo xtask new-kappa prolog
```
Generates standard module structure, testdata fixtures, and doc stubs to prevent structural drift.

### New JTBD generator
```bash
cargo xtask new-jtbd access-drift
```
Generates standard test and fixture inputs, failing until filled.

---

## 6. Layout & Golden workflow

Layout gates are first-class:
```bash
cargo xtask layout
```
Must check `size_of`, `align_of`, field offsets, reserved bytes, WireV1 sizes, and golden fixture offsets. No silent layout drift.

Golden fixture workflow:
```bash
cargo xtask golden verify
cargo xtask golden bless --case access-drift
```
Verify is default. Bless requires explicit command and prints diffs. No accidental evidence format changes.

---

## 7. Replay workflow & Truthforge report

```bash
cargo xtask replay access-drift
```
Output validates segment ordinals, route cells, blocked alternatives, checkpoints, and digest chains.

```bash
cargo xtask truthforge access-drift
```
Outputs the formal Truthforge Admission Report (the "vibe done" machine).

---

## 8. Docs generated from code where possible

```bash
cargo xtask docs-sync
```
Generate byte tables, bit positions, WireV1 offsets, layout tables, and operator tables. Docs drift is a defect.

---

## 9. Developer onboarding

```bash
cargo xtask doctor
```
Checks rust toolchain, nightly availability, required components, miri, cargo-nextest, just, etc. Must be 5 minutes.

---

## 10. Nightly feature QOL

```bash
cargo xtask features
```
Shows status, users, admitted state, and fallbacks for `portable_simd`, `generic_const_exprs`, etc. No hidden nightly dependency.

---

## 11. Debug views for byte lanes

```bash
insa explain-byte inst8 0b01101000
```
Example output:
```text
INST8 0b01101000
  Ask
  Refuse
  Escalate
```
Massive QOL.

---

## 12. Trace command for the JTBD

```bash
insa trace access-drift --fixture terminated-contractor
```
Provides explicit trace of O -> O*, KAPPA8, INST8, Selected, POWL8, and POWL64.

---

## 13. QOL for failure triage

Every failure should map to an instinct:
| Failure | QOL response |
|---|---|
| missing fixture | Retrieve |
| ambiguous object | Inspect |
| stale policy | Await |
| invalid byte | Refuse |
| target mismatch | Escalate |
| duplicate evidence | Ignore |
| closed case | Settle |

Even the developer tooling should behave like INSA.

---

## Final Rule
**QOL means every developer action either advances admission or exposes why admission failed.**
