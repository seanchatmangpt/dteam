# Architecture

## Scope

dteam contains two execution subjects:

- the original process-intelligence workspace;
- the standalone capability kernel in `capabilities/dteam-kernel`.

This document describes the capability kernel introduced by PR #6. It does not redefine the original workspace or claim that sibling path dependencies are present.

## Architectural rule

The system separates three kinds of work:

1. **SELECT** chooses among admitted alternatives.
2. **CONSTRUCT** manufactures plans, projections, intents, schemas, or evidence.
3. **DO** changes machine or domain state.

Only the brokered execution boundary may perform `DO`. All other components are construction or selection surfaces.

```text
raw observation
  ↓
validation and schema admission
  ↓
policy admission or typed refusal
  ↓
decision, hooks, composition, and planning
  ↓
resource reservation and authorization
  ↓
BRCE broker
  ↓
actuation
  ↓
receipt, provenance, replay, and standing
```

## Core modules

### Identity and model

`hash` and `model` define canonical identities and typed observations. Every evidence-bearing artifact must derive its identity from canonical content rather than process-local addresses or timestamps.

### Admission and decisions

`schema`, `policy`, and `decision` define the admissible input set and explain why a request is accepted or refused. Validation is collect-all where useful so one execution can expose the complete defect set.

### Construction

`graph`, `combinatorial`, `scheduler`, `state_machine`, and `hook` manufacture dependency-closed plans, bounded alternatives, schedules, transitions, and intents. They do not actuate external effects.

The combinatorial engine preserves all lawful bounded alternatives until an explicit objective selects from the Pareto frontier. Limits on cost, latency, reliability, reversibility, and search size are part of admission.

### State and resources

`store` provides atomic multi-key state transitions with versions and receipts. `quota` reserves and commits bounded resources. Failed preconditions must not partially mutate state or consume resources.

### Actuation

`broker` is the exclusive actuation path. It verifies preflight conditions, authorization, dependency closure, and idempotency before invoking an executor. Completion emits immutable evidence.

### Distributed coordination

`event_bus` provides partitioned event transport, offsets, visibility leases, acknowledgement, rejection, redelivery, retention, and dead letters.

`saga` coordinates multi-step operations with retries, compensation, checkpoints, and resumable recovery.

`access` supplies role and authority decisions used by higher-level execution paths.

### Evidence and process intelligence

`ledger` records receipts and verifies replay. `provenance` models entities, activities, agents, and derivation relationships. `process` derives object-centric traces, variants, directly-follows behavior, conformance diagnostics, and fitness.

### Runtime and control plane

`runtime` composes the lawful transition path. `phase_change` provides the Vision 2030 capability graph, readiness scoring, repair planning, and QoL command catalog. `dteam-doctor` exposes these functions as an operator surface.

## Dependency direction

Domain modules may depend on canonical identity and typed model primitives. Construction modules may depend on admission results. The broker may depend on plans, authorization, resources, and receipts.

The inverse dependencies are prohibited:

- domain rules must not depend on CLI formatting;
- hooks must not depend on executors;
- planners must not mutate stores or external systems;
- evidence verification must not infer successful execution from configuration alone;
- generated projections must not become independent sources of truth.

## Determinism

Determinism requires:

- canonical ordering for maps, sets, and encoded fields;
- logical time supplied as admitted input rather than ambient wall-clock time;
- explicit seeds for any stochastic procedure;
- stable failure and refusal types;
- content-derived identities;
- replay checks against the same source, validator, configuration, and toolchain identities.

## Extension contract

A new capability is complete only when it has:

1. a stable semantic definition;
2. typed input and refusal boundaries;
3. deterministic construction or execution behavior;
4. state-based tests using real collaborators;
5. negative controls;
6. evidence and replay identity;
7. operator documentation;
8. registration in the capability graph and validation matrix.

Adding a public module without adding an owned behavioral scenario must fail structural validation.
