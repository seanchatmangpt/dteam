# dteam

**dteam is a deterministic process-intelligence and capability-execution platform.**

The repository contains two related but distinct subjects:

1. the original multi-crate workspace for compiled cognition, process mining, conformance, autonomic execution, and supporting research assets;
2. a standalone dependency-free capability kernel under `capabilities/dteam-kernel` that isolates the admission, planning, actuation, receipt, replay, doctor, wizard, combinatorial, telco, event, access, and saga primitives introduced by PR #6.

Keeping those subjects separate is deliberate. A successful standalone-kernel proof does not imply that the entire multi-repository workspace is buildable, and a blocked sibling dependency does not invalidate an independently executed kernel.

## Governing model

The core manufacturing equation is:

```text
A = μ(O*)
R = receipt(A)
```

- `O*` is admitted observation: typed, bounded, policy-valid input.
- `μ` is a deterministic lawful transition.
- `A` is the resulting artifact, decision, plan, or actuation.
- `R` binds identity, authority, outcome, and replay evidence.

The runtime path is:

```text
observe
→ validate
→ route
→ admit or refuse
→ decide
→ manufacture intents
→ compose and plan
→ reserve resources
→ transition and transact
→ authorize
→ actuate
→ record provenance
→ receipt
→ replay
→ diagnose and repair
```

Actuation is permitted only through the brokered execution boundary. Hooks, planners, semantic rules, and generated projections manufacture candidates or intents; they do not receive ambient execution authority.

## Start here

### Portable local evidence crown

The fastest dependency-light proof is:

```bash
bash tools/local_finish.sh
```

This command executes twelve independently implemented semantic projections, requires byte-equivalent output, creates an object-centric execution log, performs directly-follows discovery and conformance analysis, and writes:

```text
artifacts/local-finish/ALIVE.json
```

That receipt proves the **portable local semantic subject** only. See [`docs/VALIDATION.md`](docs/VALIDATION.md) for its exact standing and exclusions.

### Standalone Rust capability kernel

With Rust available:

```bash
cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-capabilities
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-dense-demo
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- --json
```

Useful doctor commands:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- graph
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- repair
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- wizard telco
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- compose enterprise
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- telco
```

### Original workspace

The original workspace retains its existing commands:

```bash
make check
make test
make lint
make doctor
```

Some root dependencies resolve through sibling repositories such as `unibit` and `wasm4pm`. A dependency-closed checkout is required before root-workspace success can be claimed.

## Capability kernel

The standalone kernel currently exposes:

- canonical identities and deterministic hashing;
- typed observations, schemas, migrations, and transactional state;
- collect-all admission policy and explainable decision tables;
- dependency closure, critical-path scheduling, and guarded state machines;
- exclusive brokered actuation with authorization, receipts, and replay;
- object-centric process intelligence and provenance queries;
- access control, quotas, event transport, sagas, recovery, and dead letters;
- pure intent-manufacturing hooks;
- bounded combinatorial composition and Pareto selection;
- wizard profiles and telco topology/SLO analysis;
- a capability doctor, repair planner, and Vision 2030 readiness model.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for boundaries and dependency direction.

## Repository map

| Path | Purpose |
|---|---|
| `capabilities/dteam-kernel/` | Standalone deterministic capability kernel and executable demonstrations |
| `crates/insa/` | INSA primitives and execution components |
| `crates/ccog/` | Compiled-cognition facade and supporting tools |
| `crates/ccog-bridge/` | Translation and integration boundary |
| `crates/autoinstinct/` | Trace compiler and doctrine tooling |
| `tools/` | Closure verification, doctor support, local evidence, and document tooling |
| `docs/` | Architecture, validation, operations, research, and thesis assets |
| `AGENTS.md` | Detailed repository guidance for implementation agents |

## Standing vocabulary

- `UNKNOWN`: observed evidence is insufficient.
- `PARTIAL_ALIVE`: part of the admitted subject has executed successfully.
- `ALIVE`: the exact admitted subject executed successfully with matching evidence.
- `BLOCKED`: an external dependency or authority boundary prevents execution.
- `BUILD_BROKEN`: the admitted source does not currently compile or pass its required verifier.
- `UNSUPPORTED`: the requested capability is outside the admitted implementation.

Inspection is not execution. A workflow definition is not a successful run. A generated file named `receipt` is not evidence unless its manufacture and replay identities are verified.

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — system boundaries and component responsibilities
- [`docs/VALIDATION.md`](docs/VALIDATION.md) — proof ladder, standing, and release criteria
- [`docs/OPERATIONS.md`](docs/OPERATIONS.md) — operator and developer commands
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — change discipline and evidence requirements
- [`PHILOSOPHY.md`](PHILOSOPHY.md) — licensing and civilization-first motivation
- [`LICENSE`](LICENSE) — Business Source License 1.1 terms and change date

## License

dteam is licensed under the Business Source License 1.1 with the parameters in [`LICENSE`](LICENSE). The current change date is April 18, 2029. Read the license before production, hosted, managed-service, or commercial use.
