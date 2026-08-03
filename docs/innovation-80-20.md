# dteam 80/20 Innovation Closure

This change closes the four controls that dominate developer trust and iteration speed.

| Gap | Prior behavior | Closed behavior |
|---|---|---|
| Fail-open doctor | stderr was discarded and a failed diagnostic was replaced by an informational echo | `make doctor` propagates typed failure or blockage |
| Configuration authority | a missing file silently became defaults with no provenance | callers can retain `ConfigSource` and use `load_required` when defaults are unlawful |
| Configuration coherence | numeric and policy contradictions were accepted until downstream behavior failed | `AutonomicConfig::validate` rejects incoherent ranges, weights, paths, and AutoML controls |
| Reproducibility/evidence | floating nightly and no repository receipt | exact nightly plus source audit, SHA-256 receipt, mutation controls, and two-pass replay |

## Commands

```bash
make audit       # source-only; no Rust dependency closure required
make doctor      # source audit plus local toolchain and sibling-path observations
make acceptance  # mutation suite and exact two-pass replay
make doctor-artifacts  # existing AutoML artifact doctor
```

## Standing model

- `ALIVE`: source controls pass and evidence is deterministic.
- `BLOCKED_DEPENDENCY`: source controls pass, but local Rust/toolchain/sibling paths are unavailable.
- `BUILD_BROKEN`: a source invariant or replay obligation fails.

Environment absence is never rewritten as success, and generated evidence never grants standing to unexecuted Rust code.
