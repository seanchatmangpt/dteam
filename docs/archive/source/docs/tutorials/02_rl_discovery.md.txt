# Teach an Agent to Discover a Workflow

**Learning goal:** Run RL training and watch fitness improve epoch by epoch — from ~0.12 at epoch 1 to convergence near 1.00.

By the end of this tutorial you will:
- Understand how `train_with_provenance` wraps QL-learning over token replay
- Construct a synthetic event log with two variants
- Inspect the discovered `PetriNet` and action trajectory

---

## Background: how discovery works

`train_with_provenance` runs a Q-learning loop. Each epoch:

1. The current `PetriNet` model is replayed against the training log using `token_replay_projected`.
2. A reward is computed: `fitness + β·soundness − λ·complexity`.
3. The Q-agent picks a `WorkflowAction` (add place, add transition, add arc, etc.) and updates the net.
4. The agent's Q-table is updated using the reward.

After `max_training_epochs` epochs (or earlier if the fitness stopping threshold is reached), the function returns the discovered net and a `Vec<u8>` action trajectory that encodes every action taken — a compact provenance receipt.

---

## Step 1 — Understand the config

`AutonomicConfig::default()` already has sensible defaults. For this tutorial you will override two fields:

| Field | Default | Tutorial value |
|---|---|---|
| `discovery.max_training_epochs` | 100 | 50 |
| `discovery.fitness_stopping_threshold` | 0.995 | 0.95 |

Lower epochs and threshold make the tutorial run faster while still demonstrating convergence.

---

## Step 2 — Create `examples/rl_discovery.rs`

**File:** `examples/rl_discovery.rs`

```rust
use dteam::automation::train_with_provenance;
use dteam::config::{AutonomicConfig, DiscoveryConfig};
use dteam::models::{AttributeValue, Attribute, Event, EventLog, Trace};

fn make_event(activity: &str) -> Event {
    Event {
        attributes: vec![Attribute {
            key: "concept:name".to_string(),
            value: AttributeValue::String(activity.to_string()),
        }],
    }
}

fn make_log() -> EventLog {
    let mut log = EventLog::new();

    // Variant 1: 10 traces [A, B]
    for i in 0..10 {
        let mut trace = Trace::new(format!("ab-{i}"));
        trace.events.push(make_event("A"));
        trace.events.push(make_event("B"));
        log.add_trace(trace);
    }

    // Variant 2: 10 traces [B, A]
    for i in 0..10 {
        let mut trace = Trace::new(format!("ba-{i}"));
        trace.events.push(make_event("B"));
        trace.events.push(make_event("A"));
        log.add_trace(trace);
    }

    log
}

fn main() {
    // Build config with reduced epochs so the tutorial runs quickly
    let mut config = AutonomicConfig::default();
    config.discovery = DiscoveryConfig {
        max_training_epochs: 50,
        fitness_stopping_threshold: 0.95,
        strategy: "incremental".to_string(),
        drift_window: 1000,
    };

    let log = make_log();

    println!("Starting RL discovery — 50 epochs max, stopping at fitness 0.95");
    println!("Log: 10 traces [A,B] + 10 traces [B,A]");
    println!();

    // beta=0.1 rewards soundness; lambda=0.01 penalises complexity mildly
    let (net, trajectory) = train_with_provenance(
        &log,
        &config,
        0.1,  // beta: soundness weight
        0.01, // lambda: complexity penalty
        None, // no ontology constraint
        Some(42), // fixed seed for reproducibility
    );

    println!("Discovery complete.");
    println!("  Places:      {}", net.places.len());
    println!("  Transitions: {}", net.transitions.len());
    println!("  Arcs:        {}", net.arcs.len());
    println!("  Trajectory length (bytes): {}", trajectory.len());

    // Show discovered place and transition names
    if !net.places.is_empty() {
        println!();
        println!("Discovered places:");
        for p in &net.places {
            println!("  - {}", p.id);
        }
    }
    if !net.transitions.is_empty() {
        println!();
        println!("Discovered transitions:");
        for t in &net.transitions {
            let invisible = t.is_invisible.unwrap_or(false);
            println!("  - {} (label='{}', invisible={})", t.id, t.label, invisible);
        }
    }
}
```

---

## Step 3 — Run with INFO logging

The `log` crate INFO messages from `train_with_provenance` show per-epoch fitness. Enable them with `RUST_LOG`:

```sh
RUST_LOG=info cargo run --example rl_discovery
```

You will see lines like:

```
[INFO  dteam::automation] epoch=0  fitness=0.0000 ...
[INFO  dteam::automation] epoch=10 fitness=0.3120 ...
[INFO  dteam::automation] epoch=20 fitness=0.5800 ...
[INFO  dteam::automation] epoch=49 fitness=0.9200 ...
Discovery complete.
  Places:      4
  Transitions: 3
  Arcs:        6
  Trajectory length (bytes): 50
```

The exact values depend on the Q-learning exploration path. Because you passed `seed=Some(42)` the result is deterministic across runs.

Without `RUST_LOG`:

```sh
cargo run --example rl_discovery
```

Only the summary block prints — useful for CI where you want clean output.

---

## Understanding the output

**Places** — each place corresponds to a state in the discovered net. A net that correctly models both `[A,B]` and `[B,A]` typically needs a parallel split: one branch where A fires before B, one where B fires before A.

**Trajectory** — the `Vec<u8>` encodes every `WorkflowAction` the agent took as a compact byte sequence. This is the provenance receipt: it can be persisted, hashed, or replayed to reproduce the exact net without rerunning RL.

**Fitness stopping threshold** — set to `0.95` in this tutorial. If the agent reaches 0.95 fitness before epoch 50, it stops early. The convergence log line reads:

```
[INFO  dteam::automation]   epoch=N fitness=0.9500 sound=true calculus=true → converged
```

---

## Tuning tips

| Parameter | Effect |
|---|---|
| `beta` (soundness weight) | Higher → agent prefers structurally sound nets even at lower fitness |
| `lambda` (complexity penalty) | Higher → agent prefers simpler nets with fewer places/arcs |
| `seed` | Change to explore different RL paths; remove `Some(...)` to use a random seed |
| `max_training_epochs` | Increase for harder logs; 500 is typical for real PDC logs |
| `fitness_stopping_threshold` | 0.995 is the production default; 0.95 converges faster |

---

## Next steps

- **Tutorial 03** — Classify traces with the HDIT AutoML module.
- **Tutorial 04** — Run the Definition of Done verifier to check your pipeline end-to-end.
- Explore `src/automation.rs` for `train_with_provenance_and_vote`, which accepts an ensemble vote bonus from an external classifier pool.
