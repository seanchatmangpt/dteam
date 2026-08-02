# Project philosophy

dteam exists to make computational decisions inspectable, bounded, reproducible, and locally operable.

This document explains the project’s design and licensing motivation. It is not legal advice and does not replace [`LICENSE`](LICENSE).

## Engineering position

The governing model is:

```text
A = μ(O*)
R = receipt(A)
```

- `O*` is an admitted observation: typed, bounded, and policy-valid.
- `μ` is a deterministic lawful transition.
- `A` is the resulting artifact, decision, plan, or actuation.
- `R` records identity, authority, outcome, and replay evidence.

The model produces several practical rules:

1. **Observation is not authority.** Reading state does not grant permission to mutate it.
2. **Selection is not actuation.** Planners and semantic systems may manufacture alternatives, but only an admitted execution boundary may perform effects.
3. **Claims require evidence.** Source inspection, generated files, and workflow definitions do not establish runtime standing.
4. **Local operation matters.** A user should be able to inspect, execute, verify, and retain the system without mandatory dependence on a hosted inference or control service.
5. **History should remain visible.** Failed experiments and superseded designs are archived rather than silently rewritten as though they never existed.

## Civilization-first objective

The project is intended to reduce dependency rents around decision and process infrastructure. Source availability serves three purposes:

- people can inspect and challenge the implementation;
- researchers and practitioners can reproduce the ideas;
- organizations can develop internal competence rather than outsourcing every decision to an opaque service.

At the same time, immediate unrestricted platform capture could recreate the same dependency structure under a new provider. The licensing model therefore separates learning and bounded use from unrestricted commercial enclosure during the source-available period.

## Release model

The project uses three layers.

### Public theory

Concepts, equations, architectural arguments, and research results are documented for study and critique.

### Source-available implementation

The implementation can be inspected and used under the terms and additional grant in [`LICENSE`](LICENSE). The license text, not this document, controls.

### Delayed commons

The Business Source License parameters specify a change date of **April 18, 2029**, after which the licensed work converts to the stated change license. Consult [`LICENSE`](LICENSE) for the exact legal terms.

## Bounded intelligence

dteam favors systems that are:

- **bounded** — explicit inputs, limits, and refusal conditions;
- **lawful** — policy and authority are checked before effect;
- **non-sovereign** — no component receives unlimited ambient control;
- **replayable** — decisions and effects can be reconstructed from evidence;
- **replaceable** — components are connected through explicit contracts rather than institutional dependence.

This is a technical discipline, not a claim that software can eliminate judgment, politics, or organizational responsibility.

## Evidence discipline

The repository uses typed standing:

- `UNKNOWN` when evidence is insufficient;
- `PARTIAL_ALIVE` when only part of the admitted subject executed;
- `ALIVE` when the exact subject executed successfully with matching evidence;
- `BLOCKED` when an external dependency or authority boundary prevents execution;
- `BUILD_BROKEN` when the admitted source fails its required build or verifier;
- `UNSUPPORTED` when the requested capability is outside the implementation.

The standing belongs to a named subject and receipt. It must not be generalized beyond that boundary.

## Design principles

1. Preserve the distinction between `SELECT`, `CONSTRUCT`, and `DO`.
2. Permit actuation only through an explicit broker or equivalent authority boundary.
3. Prefer deterministic representations and canonical identities.
4. Record provenance and receipts for consequential transitions.
5. Keep local execution and independent verification viable.
6. Make failure states explicit rather than converting them into success-shaped output.
7. Preserve historical evidence while maintaining one current authority per concept.
8. Treat documentation, tests, and operational receipts as parts of the system contract.

## Related authorities

- [`README.md`](README.md) — project identity and entry point
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — component and authority boundaries
- [`docs/VALIDATION.md`](docs/VALIDATION.md) — evidence and release criteria
- [`docs/RESEARCH.md`](docs/RESEARCH.md) — research questions and evaluation discipline
- [`docs/DOCUMENTATION_MAP.md`](docs/DOCUMENTATION_MAP.md) — documentation authorities
- [`LICENSE`](LICENSE) — controlling license text
