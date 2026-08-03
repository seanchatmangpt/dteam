# Glossary

This document owns the current technical vocabulary used by dteam documentation. Historical documents may use older terms; when they conflict, this glossary controls current prose.

## Admitted observation (`O*`)

An input that has passed the applicable type, scope, policy, and boundary checks. Raw observation is not automatically admitted.

## Actuation

A state-changing effect outside pure construction: writing durable state, invoking an executor, publishing an event, reserving a resource, or performing another externally observable transition.

## `ALIVE`

Standing granted only when the exact admitted subject executed successfully and the required evidence identities match. `ALIVE` does not transfer automatically to a larger subject.

## Archive

Historical content preserved for provenance but removed from current authority. During the Markdown migration, exact prior bytes are stored under `docs/archive/source/` with the suffix `.md.txt`.

## Artifact

A manufactured output such as a plan, executable, model, report, receipt, event log, or generated source projection.

## Authority

The explicit permission and identity required to perform an operation. Authority is checked before actuation.

## Blocked

Standing indicating that an external dependency, unavailable authority, missing fixture, or environmental boundary prevents the admitted subject from executing.

## Broker

The exclusive execution boundary responsible for admitting authorized effects, invoking executors, and recording evidence. Planning and hooks do not bypass the broker.

## Build broken

Standing indicating that the admitted source fails its required compile, test, validation, or structural verification boundary.

## Capability

A named, bounded behavior with declared dependencies, proof requirements, standing, and repair guidance.

## Canonical identity

A deterministic digest produced from a stable encoding of semantically relevant fields.

## Chicago-style test

A state-based test that exercises real collaborators and observes externally meaningful state rather than replacing the subject with mocks.

## Combinatorial maximalism

The bounded preservation of all lawful alternatives before selection. Bounds, incompatibilities, dependencies, and objectives must be explicit.

## Conformance

Comparison between observed behavior and an expected model or invariant set. A conformance result should identify deviations rather than silently normalize them away.

## Construct

The phase that manufactures candidate artifacts, plans, or alternatives. Construction does not imply selection or authority to actuate.

## Crown

A terminal verifier that grants standing only after all required subordinate evidence has been admitted. A crown is a gate, not a filename convention.

## Dependency closure

The property that every required component, input, toolchain, and authority for an admitted subject is available and identified.

## Deterministic

Given the same admitted inputs and implementation identity, the relevant output and evidence identity are reproducible. Environmental inputs must be captured or bounded.

## Directly-follows relation

An observed process relation indicating that one activity immediately follows another within the selected event-ordering scope.

## Doctor

A diagnostic interface that reports standing, blockers, proof commands, critical paths, and repair order without receiving ambient actuation authority.

## Evidence

Observed output and associated identities sufficient to support a bounded claim. Source inspection and workflow definitions are not execution evidence.

## Hook

A pure or bounded component that converts an event or condition into candidate intents. A hook does not directly perform effects.

## Intent

A typed request describing a proposed operation before authorization and execution.

## Object-centric event log

An event representation in which events may relate to multiple typed objects rather than being forced into one case identifier.

## Partial alive

Standing indicating that a proper subset of the admitted subject executed successfully but the full acceptance boundary has not been satisfied.

## Pareto frontier

The set of lawful alternatives for which no other alternative is at least as good on every admitted objective and strictly better on one.

## Plan

A dependency-ordered or otherwise constrained description of intended work. A plan is not an execution receipt.

## Process evidence

Events, object relationships, ordering, metrics, and conformance results derived from an observed execution.

## Provenance

Recorded relationships describing how entities, activities, agents, inputs, and outputs are connected.

## Receipt

A canonical evidence record binding an operation or artifact to relevant source, input, authority, outcome, and predecessor identities.

## Replay

Re-execution or verification that checks whether an admitted subject reproduces the expected evidence identity or behavior.

## Select

The phase that chooses among lawful alternatives. Selection does not itself construct or actuate the chosen option.

## Standing

A typed assessment attached to an explicitly named subject. Current values include `UNKNOWN`, `PARTIAL_ALIVE`, `ALIVE`, `BLOCKED`, `BUILD_BROKEN`, and `UNSUPPORTED`.

## Superseded document

A stable Markdown path whose former content has been archived and whose current content redirects readers to the archive and canonical authority map.

## Unsupported

Standing indicating that the requested behavior is outside the admitted implementation or execution environment.

## Unknown

Standing indicating that available evidence is insufficient to classify the subject more specifically.

## Wizard

A guided intent compiler that converts explicit operator answers into a bounded composition or proof request.

## Zero unreceipted actuation

The invariant that no admitted effect occurs without evidence linking it to authority, operation identity, and outcome.

## Related authorities

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — boundaries and dependency direction
- [`VALIDATION.md`](VALIDATION.md) — standing and evidence rules
- [`RESEARCH.md`](RESEARCH.md) — research questions and methodology
- [`DOCUMENTATION_MAP.md`](DOCUMENTATION_MAP.md) — documentation ownership
