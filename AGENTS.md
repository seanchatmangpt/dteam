# AGENTS.md — implementation guidance

This file governs automated and human implementation work in the repository root. A nested `AGENTS.md`, when present, governs its subtree.

## Mission

dteam manufactures deterministic process-intelligence capabilities whose decisions and effects can be explained, receipted, and replayed.

Use this execution sequence:

```text
parse
→ orient
→ resolve exact subject and base
→ inspect doctrine and manifests
→ admit a bounded plan
→ implement the smallest coherent change
→ verify the failed boundary
→ expand validation
→ record evidence
→ publish intentionally
```

## Repository subjects

Do not collapse these into one claim.

### Original workspace

The root workspace contains INSA, compiled cognition, conformance, reinforcement learning, autonomic kernels, process models, benchmarks, and research assets. Some dependencies resolve through sibling repositories. Root-workspace validation requires a dependency-closed checkout.

### Standalone capability kernel

`capabilities/dteam-kernel` is a dependency-free Rust crate that isolates deterministic admission, planning, brokered execution, receipts, replay, process evidence, access control, event transport, sagas, combinatorial composition, telco assessment, and doctor capabilities.

A standalone-kernel success does not prove the root workspace. A blocked root dependency does not invalidate an executed standalone subject.

## Foundational invariants

### Governing equation

```text
A = μ(O*)
R = receipt(A)
```

`O*` is admitted and bounded observation. `μ` is deterministic lawful manufacture. `R` binds identity, authority, outcome, and replay.

### Exclusive actuation

Separate:

- `SELECT`: choose among admitted alternatives;
- `CONSTRUCT`: manufacture plans, intents, schemas, projections, and evidence;
- `DO`: mutate machine or domain state.

Only the broker may perform `DO`. Hooks manufacture intents. Planners manufacture plans. Semantic derivations and generated outputs have no ambient execution authority.

### Standing

Use only:

- `UNKNOWN`
- `PARTIAL_ALIVE`
- `ALIVE`
- `BLOCKED`
- `BUILD_BROKEN`
- `UNSUPPORTED`

A typed lawful refusal is an outcome, not a failure of standing. Inspection is not execution. A workflow is not a successful run. A receipt-shaped file is not verified evidence.

## Change discipline

1. Resolve repository, branch, base SHA, and subject.
2. Read the nearest doctrine, manifests, task runners, tests, and generated-source policy.
3. Inspect the complete transition path affected by the change.
4. Write or identify an executable acceptance boundary.
5. Make the smallest coherent bounded diff.
6. Preserve authority, receipts, replay, portability, and failure transparency.
7. Run the narrowest high-information verifier first.
8. On failure, classify the failed transition and apply a new hypothesis before rerunning.
9. Update public documentation in the same change.

Avoid unrelated refactors, fabricated evidence, weakened tests, unnecessary dependencies, workstation-specific paths, unchecked subprocesses, and hand-edited generated outputs.

## Standalone kernel commands

```bash
cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-capabilities
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-dense-demo
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- --json
python3 tools/chicago_validator.py --root . --manifest capabilities/dteam-kernel/Cargo.toml
```

Portable evidence crown:

```bash
bash tools/local_finish.sh
```

## Root workspace commands

With all sibling dependencies present:

```bash
make check
make test
make lint
make fmt
make doctor
```

Do not substitute standalone proof when a user explicitly requests root integration behavior.

## Testing standard

Prefer Chicago-style state-based tests against real collaborators. Every public capability module must own an executable scenario. Required negative controls include:

- duplicate identity with changed content;
- complete admission violations;
- no partial transaction or quota mutation;
- no executor call before authorization;
- hook intent manufacture without actuation;
- event lease, acknowledgement, redelivery, and dead-letter boundaries;
- saga compensation and resumable recovery;
- cycle refusal in dependency, role, and provenance graphs;
- replay mismatch detection;
- crown refusal before complete observed evidence.

## Generated and archival files

Edit canonical sources and generators, not projections. Historical one-shot patch scripts may remain as lineage evidence but are not supported production APIs unless admitted by `closure-policy.json`.

## Documentation contract

Public claims must name:

- exact subject;
- observed command and outcome;
- evidence identity;
- blockers and exclusions;
- generated status.

Keep these documents aligned:

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/VALIDATION.md`
- `docs/OPERATIONS.md`
- `CONTRIBUTING.md`

## Publication

Use a purpose branch and intentional conventional commits. Do not force-push. Keep the PR draft until required exact-head validation is observed. Never merge unless explicitly requested.
