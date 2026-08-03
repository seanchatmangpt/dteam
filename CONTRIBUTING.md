# Contributing to dteam

Contributions must preserve deterministic behavior, explicit authority, and replayable evidence. The project values small coherent changes over broad speculative refactors.

## Before changing code

Read:

1. [`README.md`](README.md)
2. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
3. [`docs/VALIDATION.md`](docs/VALIDATION.md)
4. the nearest `AGENTS.md` governing the files you will change
5. the relevant manifest, tests, and task runner

Open an issue before changing public APIs, authority boundaries, receipt formats, license terms, generated-source policy, or cross-repository ownership.

## Choose the correct subject

The repository has two distinct validation surfaces:

- **standalone kernel:** `capabilities/dteam-kernel`
- **root workspace:** the original multi-crate and multi-repository system

Do not claim root-workspace success from standalone-kernel evidence. Do not block standalone work merely because sibling root dependencies are absent.

## Development loop

Use the narrowest executable loop that proves the change:

```text
inspect
→ write a failing behavioral or negative test
→ implement the smallest coherent repair
→ run the focused verifier
→ run module tests
→ expand to all targets
→ execute demonstrations
→ verify receipts and replay
→ update documentation
```

Never rerun an unchanged failure without a new hypothesis.

## Standalone kernel commands

```bash
cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-capabilities
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-dense-demo
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- --json
python3 tools/chicago_validator.py --root . --manifest capabilities/dteam-kernel/Cargo.toml
```

Portable semantic and process-evidence crown:

```bash
bash tools/local_finish.sh
```

## Root-workspace commands

With a dependency-closed checkout:

```bash
make check
make test
make lint
make fmt
make doctor
```

Document missing sibling repositories as `BLOCKED`; do not replace requested root integration proof with a narrower unit result.

## Definition of done

A capability change is complete when it has:

- a stable semantic definition;
- typed input, error, and refusal boundaries;
- no ambient execution authority outside the broker;
- deterministic ordering and identity;
- state-based tests using real collaborators;
- relevant negative and recovery controls;
- an evidence or receipt path;
- replay verification where state changes occur;
- registration in the capability graph if public;
- ownership in the Chicago scenario matrix if a public module is added;
- updated operator and architecture documentation.

## Testing rules

Prefer Chicago-style state-based tests. Exercise real stores, ledgers, planners, brokers, event buses, and coordinators. Test doubles may isolate genuinely external systems, but mock output cannot establish capability standing.

Required invariants include, where applicable:

- failed admission does not actuate;
- failed preconditions do not partially mutate state;
- failed resource reservations do not partially consume quota;
- duplicate identities with changed content are refused;
- broker authorization precedes executor invocation;
- hooks manufacture intents but never actuate;
- event acknowledgement cannot skip rejected earlier offsets;
- saga compensation is reverse ordered, idempotent, and resumable;
- cycles in capability, role, or provenance graphs are refused;
- replay identity matches the admitted source and configuration.

## Code style

- Rust edition: 2021
- unsafe code: forbidden in the standalone kernel
- formatting: `cargo fmt --all`
- linting: `cargo clippy --all-targets -- -D warnings` when the relevant workspace is available
- public APIs: module and item documentation required
- dependencies: avoid adding one unless the capability cannot be expressed with the existing platform or standard library
- generated outputs: edit the canonical source or generator, not the generated projection

## Scope discipline

Keep a change bounded. Avoid:

- unrelated refactors;
- hand-edited generated artifacts;
- weakened tests or deleted negative controls;
- acceptance tests that only assert mocked success;
- hard-coded developer workstation paths;
- unchecked subprocess execution;
- silent fallback from a requested integration proof to a narrower test;
- claims that exceed observed evidence.

## Commits and pull requests

Use conventional commits:

```text
feat(scope): add capability
fix(scope): repair observed defect
test(scope): add behavioral guard
docs(scope): align documentation with evidence
refactor(scope): change structure without changing behavior
```

PR descriptions must state:

- exact base and subject;
- what changed and why;
- commands executed and exit status;
- negative controls;
- receipt or artifact identities;
- known blockers and exclusions;
- whether generated outputs changed.

Keep PRs draft while required exact-head validation is `BUILD_BROKEN`, `BLOCKED`, or unobserved.

## DCO and license

Commits must include a Developer Certificate of Origin sign-off:

```bash
git commit -s -m "fix(event-bus): release immutable borrow before mutation"
```

Contributions are licensed under the repository's Business Source License 1.1 parameters. The current change date is April 18, 2029. Read [`LICENSE`](LICENSE) before contributing code intended for hosted, managed-service, production, or commercial use.

## Review standard

Reviewers evaluate:

1. semantic correctness;
2. authority and actuation boundaries;
3. deterministic behavior;
4. failure transparency;
5. state-based test quality;
6. evidence and replay integrity;
7. operational clarity;
8. scope and maintainability.

A green status without matching behavioral evidence is insufficient. A typed lawful refusal with the expected evidence is a valid successful negative-control outcome.
