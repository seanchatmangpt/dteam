Yes — **DX/QOL needs its own Definition of Done**.

For INSA, DX/QOL is not polish. It is how you prevent the project from becoming another LLM-generated maze.

[
\boxed{
DX/QOL = \text{the system makes the correct path the easy path and the wrong path fail loudly.}
}
]

---

# DX / QOL Definition of Done

## 1. One command proves the project

```bash
just dx
```

Must run:

```text
fmt
clippy
unit tests
property tests
compile-fail tests
golden wire tests
replay tests
layout gates
access-drift JTBD
bench smoke
```

If `just dx` passes, the project is locally sane.

If not, the failure must be classified.

---

# 2. Standard command surface

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

# 3. `cargo xtask` for project-specific workflows

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

Then:

```bash
cargo xtask dx
cargo xtask layout
cargo xtask golden verify
cargo xtask replay access-drift
cargo xtask new-kappa ground
cargo xtask new-jtbd access-drift
```

This avoids shell-script entropy.

---

# 4. Error messages must be operational

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

```text
what failed
where it failed
why it failed
what evidence was expected
what lawful next action exists
```

---

# 5. New KAPPA generator

You need a scaffold command.

```bash
cargo xtask new-kappa prolog
```

Generates:

```text
crates/insa-kappa8/src/prove_prolog/
  mod.rs
  byte.rs
  engine.rs
  result.rs
  witness.rs
  tests.rs
```

Also generates:

```text
testdata/kappa/prolog/
  fixtures/
  expected/
  negative/
```

And doc stub:

```text
docs/kappa/prove-prolog.md
```

This prevents every old-AI family from drifting structurally.

---

# 6. New JTBD generator

```bash
cargo xtask new-jtbd access-drift
```

Generates:

```text
testdata/cases/access_drift/
  input/
  expected/
  negative/
  golden/
tests/jtbd_access_drift.rs
docs/jtbd/access-drift.md
```

The generated test should fail until filled.

---

# 7. Layout gates are first-class

Command:

```bash
cargo xtask layout
```

Must check:

```text
size_of
align_of
field offsets
reserved bytes
WireV1 sizes
golden fixture offsets
```

Failure example:

```text
LayoutGateFailed:
  type: Cog8Row32
  expected size: 32
  actual size: 40
  field: emits_inst8
  expected offset: 24
  actual offset: 32
```

No silent layout drift.

---

# 8. Golden fixture workflow

Commands:

```bash
cargo xtask golden verify
cargo xtask golden bless --case access-drift
```

Rules:

```text
verify is default
bless requires explicit command
bless prints diff
bless cannot run in CI unless explicitly allowed
```

No accidental evidence format changes.

---

# 9. Replay workflow

```bash
cargo xtask replay access-drift
```

Output:

```text
ReplayValid:
  segment: access_drift_v1.powl64
  route cells: 7
  blocked alternatives: 3
  checkpoints: 2
  digest chain: valid
```

Bad replay should produce exact failure class:

```text
ReplayInvalid:
  cell ordinal: 4
  reason: blocked alternative digest mismatch
```

---

# 10. Truthforge report

```bash
cargo xtask truthforge access-drift
```

Outputs:

```text
Truthforge Admission Report
  JTBD: Access Drift Closure
  O -> O*: pass
  KAPPA8: pass
  Family8: pass
  INST8: pass
  POWL8: pass
  CONSTRUCT8: pass
  POWL64: pass
  Replay: pass
  Bench smoke: pass
  Verdict: Admitted
```

This is the “vibe done” machine.

---

# 11. Docs generated from code where possible

Do not hand-maintain everything.

Generate:

```text
byte tables
bit positions
WireV1 offsets
layout tables
KAPPA family index
INST8 table
POWL8 operator table
```

Command:

```bash
cargo xtask docs-sync
```

Failure:

```text
DocsDrift:
  docs/byte-law.md says PROLOG8 bit 5 = DepthExhausted
  code says PROLOG8 bit 5 = CycleDetected
```

Docs drift is a defect.

---

# 12. Developer onboarding must be 5 minutes

A new contributor should run:

```bash
git clone ...
cd insa
rustup show
cargo xtask doctor
cargo xtask dx
```

`doctor` checks:

```text
rust toolchain
nightly availability
required components
miri availability
cargo-nextest
cargo-llvm-cov
just
wasm target if needed
```

---

# 13. Nightly feature QOL

Because nightly is allowed only as admitted control, feature use must be obvious.

```bash
cargo xtask features
```

Shows:

```text
portable_simd:
  status: candidate
  used by: hearsay fusion batch path
  admitted: false
  fallback: reference fusion path

generic_const_exprs:
  status: candidate
  used by: bounded Support<N>
  admitted: false
  fallback: runtime checked newtype
```

No hidden nightly dependency.

---

# 14. Debug views for byte lanes

Need commands like:

```bash
insa explain-byte inst8 0b01101000
insa explain-byte kappa8 0b01011000
insa explain-byte prolog8 0b00101001
```

Example output:

```text
INST8 0b01101000
  Ask
  Refuse
  Escalate
```

This is massive QOL.

---

# 15. Trace command for the JTBD

```bash
insa trace access-drift --fixture terminated-contractor
```

Output:

```text
O -> O*
  field bits: identity_terminated, badge_active, vpn_active, repo_access_active

KAPPA8
  Ground
  Fuse
  Prove
  Rule
  ReduceGap

INST8
  Refuse
  Inspect
  Escalate
  Retrieve

Selected
  Refuse

POWL8
  BLOCK AllowAccess
  ESCALATE Owner
  RETRIEVE MissingEvidence

POWL64
  segment written
  replay valid
```

---

# 16. QOL for failure triage

Every failure should map to an instinct:

| Failure            | QOL response |
| ------------------ | ------------ |
| missing fixture    | `Retrieve`   |
| ambiguous object   | `Inspect`    |
| stale policy       | `Await`      |
| invalid byte       | `Refuse`     |
| target mismatch    | `Escalate`   |
| duplicate evidence | `Ignore`     |
| closed case        | `Settle`     |

Even the developer tooling should behave like INSA.

---

# 17. Minimum DX/QOL milestone

For MVP, done means:

```text
just dx works
cargo xtask doctor works
cargo xtask layout works
cargo xtask golden verify works
cargo xtask replay access-drift works
cargo xtask truthforge access-drift works
insa explain-byte works
insa trace access-drift works
```

That is the real DX floor.

---

# Final rule

[
\boxed{
DX is not convenience.
DX is anti-drift infrastructure.
}
]

And:

[
\boxed{
QOL means every developer action either advances admission or exposes why admission failed.
}
]

The project should feel like this:

```text
generate nothing vaguely
test everything explicitly
fail with exact cause
replay every claim
admit only evidence
```