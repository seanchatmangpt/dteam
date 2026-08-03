# Validation and Standing

## Evidence rule

A capability is `ALIVE` only after the exact admitted subject executes successfully against its required verifier.

These are not equivalent:

- source inspection and compilation;
- a workflow definition and a completed workflow run;
- a generated receipt-shaped file and a verified receipt;
- a standalone kernel and the original multi-repository workspace;
- one successful implementation and cross-language equivalence.

Every result must name its subject and exclusions.

## Standing states

| State | Meaning |
|---|---|
| `UNKNOWN` | Evidence is missing or insufficient. |
| `PARTIAL_ALIVE` | Some admitted transitions executed successfully, but the complete acceptance boundary has not. |
| `ALIVE` | The complete admitted subject executed successfully and emitted matching evidence. |
| `BLOCKED` | Execution is prevented by an external dependency, authority, transport, or environment boundary. |
| `BUILD_BROKEN` | The admitted source or verifier fails before successful execution. |
| `UNSUPPORTED` | The requested capability is not implemented or admitted. |

Refusal is separate from standing. A lawful typed refusal can itself be an `ALIVE` outcome for a negative-control scenario.

## Verification ladder

Run the cheapest high-information gates first:

```text
source closure
→ type-check
→ focused unit tests
→ state-based module tests
→ integrated runtime demonstrations
→ negative and recovery controls
→ process evidence and replay
→ crown receipt
```

Do not rerun an unchanged failure without a new hypothesis or repair.

## Portable local crown

```bash
bash tools/local_finish.sh
```

This proof executes the portable semantic subject:

1. twelve implementations in Python, Node.js, Go, Java, Kotlin, Ruby, PHP, Swift, C/GCC, C/Clang, C++, and Bash;
2. byte-equivalent canonical output;
3. a cryptographic polyglot receipt;
4. an object-centric execution log;
5. directly-follows discovery and conformance analysis;
6. a final crown at `artifacts/local-finish/ALIVE.json`.

The observed reference receipt recorded:

```text
implementation_count = 12
byte_equivalent      = true
semantic_sha256      = 187b097d514ed1106145a88b2401533a2c88d3d12dc915fa7a250549fb97416d
event_count          = 26
fitness              = 1.0
violations           = 0
crown_sha256         = 1060cb2d1b8a60043eb0916ba8b51522e9638e4bc713bfffcd7ac7cd92a571d3
```

Those values are evidence for that executed local subject, not permanent constants. A new run must manufacture its own receipt.

## Rust capability kernel

The standalone crate acceptance sequence is:

```bash
cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-capabilities
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-dense-demo
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- --json
python3 tools/chicago_validator.py --root . --manifest capabilities/dteam-kernel/Cargo.toml
```

The Chicago validator derives the public module inventory from `src/lib.rs`. Every module must be owned by at least one real state-based scenario. Mocks, `todo!`, and unimplemented evidence are refused.

## Required negative controls

The validation matrix must include, where applicable:

- duplicate identity with changed content;
- rejected admission with complete violation reporting;
- failed transactional precondition with no partial mutation;
- failed atomic reservation with no partial resource consumption;
- authorization denial before executor invocation;
- event redelivery and dead-letter boundaries;
- saga compensation failure and resumable recovery;
- graph, role, and provenance cycle refusal;
- crown refusal while any required capability lacks observed `ALIVE` evidence.

## Process evidence

A process-evidence receipt must preserve:

- execution-run identity;
- capability or implementation identity;
- activity name and ordering;
- object relationships;
- outcome and refusal state;
- source receipt identity;
- semantic identity;
- discovered directly-follows relationships;
- conformance fitness and violations.

A fitness score is meaningful only with the admitted reference model and event projection recorded.

## Current PR boundary

PR #6 contains an independently executable local evidence crown and a standalone Rust capability kernel. The original root workspace remains a separate subject because some dependencies resolve through sibling repositories.

The PR must remain draft while any required exact-head Rust acceptance command is `BUILD_BROKEN`. Documentation must report that state rather than substituting the portable polyglot crown for the Rust acceptance boundary.

## Release checklist

A release candidate may be marked ready when:

- documentation matches the current source and commands;
- public-module structural coverage is complete;
- every required Rust command succeeds at the exact head;
- demonstrations emit expected evidence;
- negative and recovery controls pass;
- receipt replay matches;
- no unresolved review threads remain;
- the PR body states exact scope, exclusions, commands, and observed identities.
