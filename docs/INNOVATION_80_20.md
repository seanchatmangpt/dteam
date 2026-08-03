# 80/20 innovation closure

## Scope

This audit targets the standalone capability kernel introduced on `agent/ggen-alive-closure`. It does not treat line count, subsystem count, or speculative roadmap breadth as innovation.

The selection criterion is compounding leverage:

```text
leverage = impact × confidence ÷ effort
```

The selected surface must account for at least 80 percent of the weighted innovation impact while preserving deterministic execution, broker-only actuation, receipts, and replay.

## Repository findings

### 1. Runtime standing was not derived from runtime execution

`Vision2030::standard()` classified the runtime and crown as `PARTIAL_ALIVE`, while the doctor itself only reported static capability constants. The repository already contained the full route → admission → broker → actuation → receipt → trace path, but no operator command executed that path and used the result as innovation standing.

**Closure:** `InnovationAudit::run()` executes a real in-process tracer bullet through the exclusive broker path and verifies the seven-stage hash-chained trace plus the retained completion receipt.

### 2. Repair output implied certainty

The repair command printed `projected_score=100` without stating that the projection was conditional on every action succeeding.

**Closure:** the operator output now names the field `projected_score_if_all_actions_succeed` and prints each action's reason and reversibility.

### 3. Preset usability was not proven as a matrix

Developer, edge, telco, and enterprise presets could be invoked individually, but no single audit proved that every preset still produced lawful bounded compositions and a nonempty Pareto frontier.

**Closure:** the scenario probe executes all four profiles and binds each plan and composition-space digest into the audit evidence.

### 4. Telco disjointness rejected shared service endpoints

The redundancy algorithm compared every failure domain in each path. Two valid routes from the same source to the same destination therefore appeared non-disjoint because the endpoint domains necessarily overlap.

**Closure:** disjointness is now evaluated over internal transit nodes only. Shared service endpoints remain legal; shared transit infrastructure does not.

### 5. No deterministic regression or support handoff surface existed

The doctor could print current state, but it could not compare immutable snapshots or package its evidence into one self-verifying support payload.

**Closure:** `AuditSnapshot::diff` classifies improvements, regressions, and unchanged findings. `SupportBundle` packages the audit, doctor output, reproduction commands, and a canonical digest.

## Operator surface

```bash
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- innovation
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- innovation-json
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- snapshot
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- support
```

## Exact acceptance

```bash
python3 tools/innovation_80_20_patch.py
cargo fmt --manifest-path capabilities/dteam-kernel/Cargo.toml --all -- --check
cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1
cargo clippy --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- innovation-json
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- support
cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- telco
```

The workflow stores exact outputs beneath `artifacts/innovation-80-20/`. `ALIVE` is admitted only when the audit and telco outputs both report `ALIVE` on the same exact head.
