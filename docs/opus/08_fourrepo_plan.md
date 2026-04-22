# 08 — The Four-Repo Plan

## Discovery

Survey of `~/wasm4pm`, `~/chatmangpt/unrdf`, `~/chatmangpt/ostar`, and
`~/dteam` revealed not four projects but **four substrates of one
self-specifying loop**.

## The loop

```
unrdf (O*)
    │  μ_unrdf = sync: SPARQL + Nunjucks → code
    ▼
ostar (manufacturing)
    │  10-operator pipeline, BLAKE3 receipts, 14 proof gates
    ▼
wasm4pm (pm4py-as-WASM)
    │  conformance / discovery / POWL oracle (38/38 tests)
    ▼
dteam (unibit / POWL64 / kinetic)
    │  branchless bitmask execution at L₁
    ▼
receipts + OCEL + OTel
    │  observe execution, propose constraint patches via μ_llm
    ▼
back into unrdf as new triples → BLAKE3 hash of O* moves
    │  runHooksAutonomics until hash stabilizes
    ▼
FixedPoint reached ⇒ O* closed ⇒ A = μ(O*) provable
```

## Layer roles

**Layer 1 — Specification substrate: `~/chatmangpt/unrdf`.**
24-package monorepo. O* lives here.
- `core` + `oxigraph` hold the triples
- `kgc-4d` extends into 4D (observable/time/vector/git) with 176/176 tests, 99.8% coverage, BLAKE3-verified snapshots
- `governance` enforces closure
- `hooks` runs the SelfSpecifyingLoop
- `sync` is the μ_unrdf code generator
- `manufacturing` is the artifact pipeline
- `daemon` is the μ_llm continuous constraint proposer with Groq backing
- `federation` is the multi-operator future

**Layer 2 — Process-mining substrate: `~/chatmangpt/pm4py` (fork) + `~/wasm4pm`.**
pm4py-on-Rust replaced by pm4py-on-WASM. Process mining ships as a
browser/Node WASM artifact with 38/38 conformance tests (as of 2026-04-17).
Provides regex-validatable, parseable, lower-able output grammar for
everything upstream. With wasm4pm, process mining is no longer a Python
service; it's a compiled artifact that ships inside the browser runtime or
the daemon.

**Layer 3 — Manufacturing substrate: `~/chatmangpt/ostar`.**
CodeManufactory — 10-operator pipeline (Seed → Breed → Validate → Project →
Compile → Benchmark → Release + 3 supplementary), BLAKE3 receipt chain, 14
proof gates, OTel→OCEL→pm4py conformance, Rust truth-gate enforcing
no-mocks. Consumes O* from Layer 1 via `unrdf sync`, produces artifacts via
μ_unrdf, validates them via Layer 2, receipts them. ETHOS Claims Matrix
lives here as seven-state type system for claim promotion.

**Layer 4 — Kinetic substrate: `~/dteam` (with unibit, MuStar, unios, POWL64).**
Hot-path execution layer. 8ⁿ/64ⁿ kinetic-capacity ladder. Takes compiled
motion packets from the manufacturing layer and executes branchlessly at L₁
residency with BLAKE3 witness fragments. DTEAM Arena / POWL64 / PDC 2025
work sits here.

## What remains

Not design. Closure. The packages exist; conformance receipts exist;
ontology files exist; operator pipeline exists; kinetic substrate exists.
What remains is finishing the wire-up so an ontology-level commit in unrdf
produces, automatically, a regenerated artifact in ostar, validated by
wasm4pm, executed by unibit, receipted back into the same ontology — and the
BLAKE3 hash either moves (constraint was novel) or stabilizes (fixed point).

## Summary

The four repos are not four projects. They are the four substrates of one
self-specifying loop whose correctness is `A = μ(O*)` and whose termination
is a hash that stops moving.
