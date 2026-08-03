# Research program

This document owns the current research questions, evaluation boundaries, and evidence expectations for dteam.

## Research objective

dteam studies whether decision and process infrastructure can be manufactured as bounded, deterministic, receipt-bearing systems rather than opaque, continuously interpreted services.

The central hypothesis is that useful operational intelligence can be decomposed into:

```text
admitted observation
→ lawful transformation
→ explicit authority
→ observable effect
→ replayable evidence
```

The repository contains implementations and experiments across process mining, conformance, policy admission, deterministic scheduling, state machines, brokered actuation, object-centric event data, provenance, and bounded capability composition.

## Primary questions

### Deterministic execution

Can consequential decisions and transitions be reproduced byte-for-byte from admitted inputs, policy, and versioned implementation state?

Evaluation requires repeated execution, canonical output identities, and explicit treatment of environmental inputs.

### Evidence-bearing actuation

Can an execution system prove that authorization preceded effect and that every effect belongs to an admitted operation?

Evaluation requires negative controls for missing authority, tampered receipts, replay mismatch, and direct actuation outside the broker boundary.

### Process intelligence

Can object-centric event logs and discovered process structures explain runtime behavior without collapsing multiple interacting objects into a single case identifier?

Evaluation includes directly-follows discovery, conformance violations, object relationships, transition frequencies, and reproducible fixtures.

### Combinatorial construction

Can a system preserve all lawful alternatives inside explicit bounds, then derive Pareto-optimal choices without conflating construction with selection?

Evaluation includes exhaustive bounded enumeration, dependency closure, incompatibility refusal, objective calculation, and deterministic frontier identity.

### Operational diagnosis

Can a doctor explain current standing, shortest blocker paths, repair order, and proof commands without granting itself ambient actuation authority?

Evaluation requires machine-readable reports, stable digests, negative crown behavior, and repair plans that respect dependencies.

### Portability and independent verification

Can the same admitted semantics be implemented and checked independently across runtimes and architectures?

The portable crown is one experiment in this direction. Its result applies only to the bounded model it executes; it does not substitute for compiling or testing the Rust kernel.

## Evaluation ladder

Research claims should advance through distinct levels:

1. **Specified** — the subject, inputs, invariants, and refusal conditions are written down.
2. **Implemented** — executable source exists.
3. **Unit observed** — isolated state transitions execute successfully.
4. **Integrated observed** — real collaborators execute together.
5. **Adversarial observed** — negative controls refuse invalid or tampered cases.
6. **Replay observed** — rerun evidence matches the original admitted identity.
7. **Portable observed** — an independent implementation or architecture reproduces the bounded result.

A higher level does not erase lower-level evidence. Each receipt should identify the exact subject and evaluation level.

## Methodological rules

- Define the admitted subject before running an experiment.
- Record exact source and fixture identities.
- Separate expected failures from accidental failures.
- Preserve raw evidence or cryptographic identities sufficient to detect alteration.
- Report exclusions and environmental dependencies.
- Do not infer whole-repository standing from a component proof.
- Prefer fixtures that can be replayed without proprietary services.
- Archive superseded experiments instead of rewriting their historical conclusions.

## Process-science alignment

The process-intelligence work follows several established process-mining concerns:

- event and object identity;
- ordering and lifecycle semantics;
- discovery of observed control flow;
- conformance between observed and expected behavior;
- performance and frequency analysis;
- reproducible transformations from logs to models and reports.

The implementation may use simplified or bounded models where required for determinism. Such bounds must be explicit in the corresponding architecture and validation documentation.

## Current experimental subjects

### Standalone capability kernel

Located at `capabilities/dteam-kernel`, this subject isolates deterministic capability primitives from root-workspace sibling dependencies.

### Chicago-style behavioral validation

`tools/chicago_validator.py` derives the public module inventory and assigns executable state-based scenarios using real collaborators rather than mock evidence.

### Portable polyglot crown

`tools/local_finish.sh` executes a bounded semantic model through multiple local language runtimes, compares canonical output, manufactures object-centric process evidence, and emits a crown receipt.

### Original process-intelligence workspace

The root workspace contains broader process-mining, conformance, autonomic, learned, and research surfaces. Its standing must be evaluated in a dependency-closed checkout.

## Open questions

- How should receipt schemas evolve without weakening replay compatibility?
- Which process models provide the best balance between expressive power and mechanically bounded verification?
- How should uncertainty be represented without allowing probabilistic output to bypass deterministic authority checks?
- What minimum event semantics are required for meaningful object-centric conformance?
- How should distributed failure domains be represented in capability and telco composition?
- Which properties should be proven formally, property-tested, model-checked, or observed operationally?
- How should repair automation remain useful while preserving zero unreceipted actuation?

## Publication discipline

A publication or benchmark derived from this repository should state:

- repository and exact revision;
- admitted component or workspace subject;
- toolchain and platform;
- fixture or dataset identity;
- commands executed;
- successful and negative-control results;
- receipt or artifact identities;
- known exclusions and threats to validity.

## Related authorities

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — implementation boundaries
- [`VALIDATION.md`](VALIDATION.md) — standing and release evidence
- [`OPERATIONS.md`](OPERATIONS.md) — executable procedures
- [`GLOSSARY.md`](GLOSSARY.md) — terminology
- [`../PHILOSOPHY.md`](../PHILOSOPHY.md) — project motivation
