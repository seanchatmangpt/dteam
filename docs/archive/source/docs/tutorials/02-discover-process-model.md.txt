# Discover Your First Process Model — RL-Based Petri Net Discovery

## Overview

This tutorial teaches you how to use the `Engine` API to discover a Petri net from an event log using dteam's reinforcement-learning discovery loop. You will learn how to configure the engine with the right K-tier for your log's activity footprint, how to handle all three possible `EngineResult` variants, and how to read every field of the `ExecutionManifest` to verify and reproduce results. By the end you will have a working `examples/discover_process.rs` binary that discovers a model, prints the manifest, and validates the discovered net with token replay.

## Prerequisites

- Completed Tutorial 1 (`docs/tutorials/01-hello-conformance.md`)
- Rust 1.78+ and Cargo
- dteam in your `Cargo.toml` (same as Tutorial 1)

## Step 1 — Create the example file

Create `examples/discover_process.rs`. All code from Steps 2–9 lives in this file.

## Step 2 — Import types

```rust
use dteam::dteam::orchestration::{Engine, EngineResult, ExecutionManifest};
use dteam::dteam::core::KTier;
use dteam::conformance::{token_replay_projected, ProjectedLog};
use dteam::io::xes::XESReader;
use dteam::models::{Event, EventLog, Trace};
use std::path::Path;
```

Note the nested module path: `dteam::dteam::orchestration`. The outer `dteam` is the crate name; the inner `dteam` is the `pub mod dteam` declared in `src/lib.rs`. `Engine`, `EngineResult`, and `ExecutionManifest` all live inside `dteam::dteam::orchestration`. `KTier` lives in `dteam::dteam::core`.

## Step 3 — Build an engine

```rust
fn build_engine() -> Engine {
    Engine::builder()
        .with_k_tier(64)       // select bitmask tier based on expected activity footprint
        .with_reward(0.6, 0.01) // beta = fitness weight; lambda = soundness penalty weight
        .with_deterministic(true) // deterministic RL updates (reproducible across runs)
        .build()               // loads dteam.toml or falls back to Default
}
```

Builder method reference:

| Method | Parameter | Effect |
|--------|-----------|--------|
| `with_k_tier(k: usize)` | number of distinct activities | Maps to a `KTier` variant (see table below). If the log's footprint exceeds the tier's capacity, `run()` returns `PartitionRequired`. |
| `with_reward(beta, lambda)` | `beta: f32`, `lambda: f32` | `beta` weights the fitness term in the Bellman update; `lambda` weights the structural-soundness penalty. `beta=0.6, lambda=0.01` is a good starting point for most event logs. |
| `with_deterministic(det: bool)` | `true` or `false` | When `true`, the RL policy is deterministic across runs for the same log and config. Set to `false` to enable stochastic exploration. |
| `with_ontology(ontology)` | `dteam::models::Ontology` | Restricts discovery to a declared activity vocabulary. Events outside the ontology are counted in `manifest.violation_count`. |
| `with_pruning(prune: bool)` | `true` or `false` | When `true`, out-of-ontology events are silently pruned before discovery instead of triggering `BoundaryViolation`. |

K-tier mapping applied by `with_k_tier(k)`:

| `k` value | `KTier` selected | Capacity | Approximate epoch latency |
|-----------|-----------------|----------|--------------------------|
| 0 – 64 | `KTier::K64` | 64 places | 2–5 µs |
| 65 – 128 | `KTier::K128` | 128 places | 5–8 µs |
| 129 – 256 | `KTier::K256` | 256 places | 8–12 µs |
| 257 – 512 | `KTier::K512` | 512 places | 14–20 µs |
| 513+ | `KTier::K1024` | 1024 places | 30–50 µs |

`build()` attempts to load `dteam.toml` from the current working directory via `AutonomicConfig::load("dteam.toml")`. If the file is absent or malformed, it silently falls back to `Default` — no error is returned and discovery still runs.

## Step 4 — Prepare a log

You can either load a real XES file or construct a synthetic log programmatically.

### Loading from a XES file

```rust
fn load_from_xes(path: &str) -> EventLog {
    XESReader::new()
        .read(Path::new(path))
        .expect("Failed to read XES file")
}
```

### Building a synthetic log

```rust
fn build_synthetic_log() -> EventLog {
    let mut log = EventLog::new();

    // Event::new(activity) creates an Event with a single
    // Attribute { key: "concept:name", value: AttributeValue::String(activity) }.
    for i in 0..10u32 {
        let mut trace = Trace::new(format!("case-{:03}", i));
        trace.events.push(Event::new("Register".to_string()));
        trace.events.push(Event::new("Validate".to_string()));
        trace.events.push(Event::new("Approve".to_string()));
        trace.events.push(Event::new("Close".to_string()));
        log.add_trace(trace);
    }

    // Add one variant with a rework loop
    let mut rework = Trace::new("case-011".to_string());
    rework.events.push(Event::new("Register".to_string()));
    rework.events.push(Event::new("Validate".to_string()));
    rework.events.push(Event::new("Validate".to_string())); // repeated
    rework.events.push(Event::new("Approve".to_string()));
    rework.events.push(Event::new("Close".to_string()));
    log.add_trace(rework);

    log
}
```

`EventLog::activity_footprint()` counts distinct activities and is the programmatic way to pick a K-tier (see Step 8).

## Step 5 — Run the engine and match results

```rust
fn run_discovery(engine: &Engine, log: &EventLog) {
    match engine.run(log) {
        EngineResult::Success(net, manifest) => {
            println!("Discovery succeeded.");
            println!("Places      : {}", net.places.len());
            println!("Transitions : {}", net.transitions.len());
            println!("Arcs        : {}", net.arcs.len());
            print_manifest(&manifest);
        }

        EngineResult::PartitionRequired { required, configured } => {
            // The log's activity footprint exceeds the configured tier capacity.
            // Remedy: rebuild the engine with with_k_tier(required) or partition
            // the log into sub-logs whose footprints fit within the current tier.
            eprintln!(
                "Partition required: log needs {} activity slots, engine configured for {}.",
                required, configured
            );
            eprintln!("Rebuild with Engine::builder().with_k_tier({}).build()", required);
        }

        EngineResult::BoundaryViolation { activity } => {
            // An event in the log references an activity that is not in the
            // ontology attached via with_ontology(). This variant only fires
            // when with_pruning(false) (the default).
            eprintln!(
                "Boundary violation: activity '{}' is outside the declared ontology.",
                activity
            );
            eprintln!("Either add the activity to the Ontology or call with_pruning(true).");
        }
    }
}
```

`EngineResult` variants:

| Variant | When it occurs | What to do |
|---------|---------------|------------|
| `Success(Box<PetriNet>, ExecutionManifest)` | Discovery completed within the configured tier and ontology bounds. | Read the manifest; run token replay to verify. |
| `PartitionRequired { required, configured }` | `log.activity_footprint() > engine.k_tier.capacity()`. | Rebuild with `with_k_tier(required)`, or split the log. |
| `BoundaryViolation { activity }` | An event's activity label is absent from the attached `Ontology`, and `prune_on_violation` is `false`. | Add the activity to the `Ontology` or enable pruning. |

## Step 6 — Read the ExecutionManifest

`ExecutionManifest` is a serializable audit record that captures a complete, reproducible description of the discovery run.

```rust
fn print_manifest(m: &ExecutionManifest) {
    println!();
    println!("--- ExecutionManifest ---");
    // Canonical FNV-1a hash of the input log's activity sequence.
    // Reproducing a run requires presenting a log with this exact hash.
    println!("input_log_hash      : {:#018x}", m.input_log_hash);

    // Canonical hash of the discovered PetriNet (sorted places, transitions, arcs).
    // Two manifests with identical model_canonical_hash produced the same net.
    println!("model_canonical_hash: {:#018x}", m.model_canonical_hash);

    // The KTier variant used (e.g. "K64", "K256").
    println!("k_tier              : {}", m.k_tier);

    // Minimum Description Length score: transitions + arcs * log2(vocab_size).
    // Lower is more parsimonious. Compare across candidate models.
    println!("mdl_score           : {:.4}", m.mdl_score);

    // Wall-clock latency of the entire engine.run() call in nanoseconds.
    println!("latency_ns          : {} ns  ({:.2} ms)",
        m.latency_ns,
        m.latency_ns as f64 / 1_000_000.0);

    // The RL policy trajectory as a Vec<u8>.
    // Each byte encodes one action step: 0 = Idle, 1 = Optimize, 2 = Rework.
    // Replaying this sequence on the same log and config reproduces the model.
    println!("action_sequence     : {:?}", m.action_sequence);
    println!("  (0=Idle, 1=Optimize, 2=Rework)");

    // Number of events that were outside the ontology (0 when no ontology is set).
    println!("violation_count     : {}", m.violation_count);

    // True if every transition label in the discovered net is in the ontology.
    // Always true when no ontology is set.
    println!("closure_verified    : {}", m.closure_verified);

    // Present only if an Ontology was passed to the engine.
    if let Some(h) = m.ontology_hash {
        println!("ontology_hash       : {:#018x}", h);
    }
}
```

Field-by-field reference:

| Field | Type | Meaning |
|-------|------|---------|
| `input_log_hash` | `u64` | FNV-1a hash of the log's activity sequences in trace order. Deterministic for the same log. |
| `model_canonical_hash` | `u64` | Hash of the discovered net's sorted place IDs, transition IDs, and arcs. Two runs with identical hashes produced structurally equal nets. |
| `k_tier` | `String` | Debug representation of the `KTier` variant selected (`"K64"`, `"K128"`, …). |
| `mdl_score` | `f64` | Minimum Description Length score — lower means the model is more parsimonious relative to the log vocabulary. |
| `latency_ns` | `u64` | Total wall-clock latency of `engine.run()` in nanoseconds. |
| `action_sequence` | `Vec<u8>` | RL policy trajectory: sequence of `0` (Idle), `1` (Optimize), or `2` (Rework) actions. |
| `violation_count` | `usize` | Events pruned because they were outside the ontology. Zero when no ontology is attached. |
| `closure_verified` | `bool` | `true` when every transition in the net is within the ontology (or no ontology is set). |
| `ontology_hash` | `Option<u64>` | Hash of the `Ontology`'s bitset, present when an ontology was provided. |

## Step 7 — Verify the discovered model

After a successful discovery run, verify the net with token replay and structural checks:

```rust
use dteam::models::petri_net::PetriNet;

fn verify_net(net: &PetriNet, log: &EventLog) {
    // --- Projected token replay fitness ---
    let projected = ProjectedLog::from(log);
    let fitness = token_replay_projected(&projected, net);
    println!();
    println!("Post-discovery verification");
    println!("  token replay fitness      : {:.4}", fitness);
    // For a synthetic log with a single activity sequence, expect 1.0.

    // --- Structural workflow net check ---
    // is_structural_workflow_net() verifies:
    //   - exactly one source place (no incoming arcs)
    //   - exactly one sink place   (no outgoing arcs)
    //   - every transition has at least one input arc and one output arc
    let is_wf = net.is_structural_workflow_net();
    println!("  is structural workflow net: {}", is_wf);

    // --- State equation calculus ---
    // verifies_state_equation_calculus() additionally checks that every
    // transition both consumes and produces tokens in the incidence matrix.
    let state_eq = net.verifies_state_equation_calculus();
    println!("  state equation verified   : {}", state_eq);

    // --- MDL score ---
    // Lower MDL = more parsimonious model. Useful for comparing multiple candidates.
    println!("  mdl_score                 : {:.4}", net.mdl_score());

    // --- Model self-explanation ---
    println!();
    println!("{}", net.explain());
}
```

A well-discovered model should show:
- `fitness == 1.0` for a purely synthetic single-variant log
- `is_structural_workflow_net == true`
- `state_equation_verified == true`
- `mdl_score` proportional to the number of transitions and arcs

## Step 8 — Choosing the right K-tier

The K-tier is the single most important configuration choice. It determines which bitmask width is used for marking representation. Undersizing it forces `PartitionRequired`; oversizing it wastes memory.

The programmatic best practice is to measure the log's footprint first:

```rust
fn recommended_k_tier(log: &EventLog) -> usize {
    // activity_footprint() counts distinct "concept:name" values across all traces.
    log.activity_footprint()
}

fn build_engine_for_log(log: &EventLog) -> Engine {
    let footprint = recommended_k_tier(log);
    println!("Activity footprint: {} — using with_k_tier({})", footprint, footprint);
    Engine::builder()
        .with_k_tier(footprint)
        .with_reward(0.6, 0.01)
        .with_deterministic(true)
        .build()
}
```

Quick reference by process domain:

| Process domain | Typical footprint | Recommended call | KTier |
|----------------|-------------------|-----------------|-------|
| Simple sequential (tutorial log) | 4–10 | `with_k_tier(10)` | K64 |
| Purchase-to-pay, order-to-cash | 15–40 | `with_k_tier(40)` | K64 |
| Healthcare patient pathways | 50–120 | `with_k_tier(120)` | K128 |
| IT incident management | 100–200 | `with_k_tier(200)` | K256 |
| PDC 2025 competition logs | up to 400 | `with_k_tier(400)` | K512 |

When in doubt, call `Engine::builder().with_k_tier(log.activity_footprint()).build()`. If the footprint later grows (e.g., new activities appear in production), the engine will return `PartitionRequired` rather than silently producing wrong results.

## Step 9 — Run the example

Assemble the complete `examples/discover_process.rs`:

```rust
use dteam::conformance::{token_replay_projected, ProjectedLog};
use dteam::dteam::orchestration::{Engine, EngineResult, ExecutionManifest};
use dteam::models::{Event, EventLog, Trace};

fn build_synthetic_log() -> EventLog {
    let mut log = EventLog::new();
    for i in 0..10u32 {
        let mut trace = Trace::new(format!("case-{:03}", i));
        trace.events.push(Event::new("Register".to_string()));
        trace.events.push(Event::new("Validate".to_string()));
        trace.events.push(Event::new("Approve".to_string()));
        trace.events.push(Event::new("Close".to_string()));
        log.add_trace(trace);
    }
    let mut rework = Trace::new("case-011".to_string());
    rework.events.push(Event::new("Register".to_string()));
    rework.events.push(Event::new("Validate".to_string()));
    rework.events.push(Event::new("Validate".to_string()));
    rework.events.push(Event::new("Approve".to_string()));
    rework.events.push(Event::new("Close".to_string()));
    log.add_trace(rework);
    log
}

fn print_manifest(m: &ExecutionManifest) {
    println!();
    println!("--- ExecutionManifest ---");
    println!("input_log_hash      : {:#018x}", m.input_log_hash);
    println!("model_canonical_hash: {:#018x}", m.model_canonical_hash);
    println!("k_tier              : {}", m.k_tier);
    println!("mdl_score           : {:.4}", m.mdl_score);
    println!("latency_ns          : {} ns  ({:.2} ms)",
        m.latency_ns, m.latency_ns as f64 / 1_000_000.0);
    println!("action_sequence len : {} steps", m.action_sequence.len());
    println!("violation_count     : {}", m.violation_count);
    println!("closure_verified    : {}", m.closure_verified);
}

fn main() {
    let log = build_synthetic_log();

    let footprint = log.activity_footprint();
    println!("Log summary");
    println!("  traces           : {}", log.traces.len());
    println!("  activity footprint: {}", footprint);

    let engine = Engine::builder()
        .with_k_tier(footprint)
        .with_reward(0.6, 0.01)
        .with_deterministic(true)
        .build();

    match engine.run(&log) {
        EngineResult::Success(net, manifest) => {
            println!();
            println!("Discovery succeeded.");
            println!("  places      : {}", net.places.len());
            println!("  transitions : {}", net.transitions.len());
            println!("  arcs        : {}", net.arcs.len());

            print_manifest(&manifest);

            // Post-discovery verification
            let projected = ProjectedLog::from(&log);
            let fitness = token_replay_projected(&projected, &net);
            println!();
            println!("Post-discovery verification");
            println!("  token replay fitness      : {:.4}", fitness);
            println!("  is structural workflow net: {}", net.is_structural_workflow_net());
            println!("  state equation verified   : {}", net.verifies_state_equation_calculus());
            println!("  mdl_score                 : {:.4}", net.mdl_score());
        }

        EngineResult::PartitionRequired { required, configured } => {
            eprintln!(
                "Partition required: log needs {} slots, engine has {}.",
                required, configured
            );
        }

        EngineResult::BoundaryViolation { activity } => {
            eprintln!("Boundary violation: '{}'", activity);
        }
    }
}
```

Run it:

```
cargo run --example discover_process
```

Expected output shape (exact hash values will vary across Rust versions and config):

```
Log summary
  traces            : 11
  activity footprint: 4

Discovery succeeded.
  places      : <N>
  transitions : 4
  arcs        : <M>

--- ExecutionManifest ---
input_log_hash      : 0x...
model_canonical_hash: 0x...
k_tier              : K64
mdl_score           : <score>
latency_ns          : <ns>  (<ms> ms)
action_sequence len : <steps> steps
violation_count     : 0
closure_verified    : true

Post-discovery verification
  token replay fitness      : 1.0000
  is structural workflow net: true
  state equation verified   : true
  mdl_score                 : <score>
```

For a purely synthetic single-variant log (the 10 identical traces), token replay fitness will be 1.0. The rework trace may reduce this slightly depending on whether the discovered net includes a self-loop on Validate.

## What you learned

- `Engine::builder()` produces an `EngineBuilder`; each `.with_*` method returns `Self` for chaining; `.build()` loads `dteam.toml` or falls back to `Default`.
- `with_k_tier(k)` maps an integer footprint to a `KTier` variant: 0–64 → K64, 65–128 → K128, 129–256 → K256, 257–512 → K512, else K1024.
- `engine.run(&log)` returns `EngineResult`: `Success`, `PartitionRequired`, or `BoundaryViolation` — all three must be handled.
- `ExecutionManifest` is a fully serializable audit record; `model_canonical_hash` and `action_sequence` together allow exact reproduction.
- `action_sequence` bytes encode the RL policy: `0` = Idle, `1` = Optimize, `2` = Rework.
- Post-discovery, verify with `token_replay_projected` for fitness and `is_structural_workflow_net` + `verifies_state_equation_calculus` for structural soundness.
- The programmatic best practice for tier selection is `Engine::builder().with_k_tier(log.activity_footprint()).build()`.

## Next steps

- Read `src/automation.rs` to understand `train_with_provenance_projected` — the function `Engine::run` calls internally.
- Explore `src/autonomic/` to use the `AutonomicKernel` trait for adaptive, feedback-driven discovery loops.
- Read `src/skeptic_contract.rs` to understand the non-negotiable correctness obligations that govern every conformance and discovery operation.
