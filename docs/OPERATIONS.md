# Operations

## Operator model

The supported operator surface is the standalone capability kernel and its doctor. The root workspace retains separate legacy and research commands.

## Fast local diagnosis

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- --json
```

The report includes standing, weighted score, capability counts, critical path, quick wins, and a canonical report digest.

Human-readable alternatives:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- status
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- graph
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- repair
```

## QoL profiles

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol check
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol doctor
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol prove
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol repair
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol ship
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- qol explain
```

A QoL profile expands to commands. It does not execute them implicitly.

## Wizard and composition

Wizard profiles compile operational intent into bounded construction requests:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- wizard developer
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- wizard edge
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- wizard telco
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- wizard enterprise
```

Search the bounded lawful composition space:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- compose developer
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- compose edge
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- compose telco
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- compose enterprise
```

Composition reports explored, refused, lawful, and Pareto-optimal alternatives. Construction does not imply deployment or actuation.

## Telco assessment

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- telco
```

The assessment reports compliant paths, transport-disjoint paths, single points of failure, latency, capacity, reliability, failure-domain coverage, and a digest.

Endpoint sharing is not automatically a transport single point of failure. Redundancy is evaluated over the admitted failure-domain model.

## Runtime demonstrations

Base capability path:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-capabilities
```

Dense integrated path:

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-dense-demo
```

The dense demonstration exercises access decisions, quota reservations, event publication and acknowledgement, state projection, saga execution, quota commit, and replay.

## Portable local crown

```bash
bash tools/local_finish.sh
```

Optional explicit output directory:

```bash
bash tools/local_finish.sh "$(pwd)" /tmp/dteam-local-finish
```

Expected outputs:

```text
artifacts/local-finish/polyglot/receipt.json
artifacts/local-finish/process-evidence.json
artifacts/local-finish/ALIVE.json
```

Delete the output directory before replay when testing clean manufacture. The script already removes its selected output directory at startup.

## Failure handling

Classify the failed transition before repair:

| Failure | First inspection |
|---|---|
| Parse or syntax error | Exact file and compiler location |
| Type or borrow error | Returned ownership and mutation boundary |
| Admission refusal | Complete violation set and input identity |
| Runtime error | Route, authorization, reservation, executor, and receipt sequence |
| Replay mismatch | Source, configuration, toolchain, and receipt identities |
| Process nonconformance | Event projection, reference model, and violating trace |
| Missing dependency | Whether the dependency belongs to the standalone kernel or root workspace |

Do not rerun an unchanged failure. Record a new hypothesis, apply the narrowest coherent repair, then repeat the failed boundary before expanding validation.

## Crown

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- crown
```

The crown is expected to refuse while any required capability remains below `ALIVE`. A nonzero exit is therefore a valid negative-control result until the complete exact subject has executed successfully.
