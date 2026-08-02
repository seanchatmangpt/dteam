# From Zero to First Conformance Check

**Learning goal:** Run a Rust example that prints `Fitness: 1.00 | Traces: 12 | Perfect: 12`.

By the end of this tutorial you will have:
- A working dteam build
- A hand-written Petri net and event log
- Your first token-based conformance replay result

---

## Prerequisites

| Requirement | Minimum version | Check |
|---|---|---|
| Rust toolchain | 1.80 (stable) | `rustc --version` |
| cargo-make | any recent | `cargo make --version` |

Install cargo-make if needed:

```sh
cargo install cargo-make
```

---

## Step 1 — Clone and verify the build

```sh
git clone <repo-url> dteam
cd dteam
cargo make check
```

A clean check prints something like:

```
[cargo-make] INFO - Running Task: check
    Checking dteam v0.1.0 ...
    Finished `dev` profile
```

If you see errors, confirm your Rust toolchain matches the `rust-toolchain.toml` at the repo root.

---

## Step 2 — Understand the model you will build

The example uses a minimal sequential workflow net:

```
source --[t_A]--> p1 --[t_B]--> sink
```

Three places, two transitions, four arcs. A valid trace must fire `t_A` then `t_B`. The initial marking places one token on `source`.

---

## Step 3 — Create `examples/my_first_check.rs`

Create the file at the path shown and paste the code below exactly.

**File:** `examples/my_first_check.rs`

```rust
use dteam::conformance::case_centric::token_based_replay::apply_token_based_replay;
use dteam::models::petri_net::{Arc, PetriNet, Place, Transition};
use dteam::models::{AttributeValue, Attribute, Event, EventLog, Trace};
use dteam::utils::dense_kernel::{fnv1a_64, PackedKeyTable};

fn make_net() -> PetriNet {
    // Three places: source, p1, sink
    let source = Place { id: "source".to_string() };
    let p1     = Place { id: "p1".to_string() };
    let sink   = Place { id: "sink".to_string() };

    // Two transitions labelled A and B
    let t_a = Transition {
        id: "t_A".to_string(),
        label: "A".to_string(),
        is_invisible: Some(false),
    };
    let t_b = Transition {
        id: "t_B".to_string(),
        label: "B".to_string(),
        is_invisible: Some(false),
    };

    // Arcs: source->t_A, t_A->p1, p1->t_B, t_B->sink
    let arcs = vec![
        Arc { from: "source".to_string(), to: "t_A".to_string(), weight: Some(1) },
        Arc { from: "t_A".to_string(),   to: "p1".to_string(),   weight: Some(1) },
        Arc { from: "p1".to_string(),    to: "t_B".to_string(),  weight: Some(1) },
        Arc { from: "t_B".to_string(),   to: "sink".to_string(), weight: Some(1) },
    ];

    // Initial marking: one token on source
    let mut initial_marking: PackedKeyTable<String, usize> = PackedKeyTable::default();
    let h = fnv1a_64(b"source");
    initial_marking.insert(h, "source".to_string(), 1);

    PetriNet {
        places: vec![source, p1, sink],
        transitions: vec![t_a, t_b],
        arcs,
        initial_marking,
        final_markings: vec![],
        cached_incidence: None,
        cached_index: None,
    }
}

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

    // 12 identical traces: each fires A then B
    for i in 0..12 {
        let mut trace = Trace::new(format!("case-{i}"));
        trace.events.push(make_event("A"));
        trace.events.push(make_event("B"));
        log.add_trace(trace);
    }

    log
}

fn main() {
    let net = make_net();
    let log = make_log();

    let result = apply_token_based_replay(&net, &log);

    let fitness     = result.compute_fitness();
    let trace_count = log.traces.len();
    // A trace is "perfect" when it produced no missing and no remaining tokens.
    // With the standard replay the aggregate result is returned; count perfect
    // traces by checking missing == 0 && remaining == 0 on the aggregate when
    // all traces are identical and conforming.
    let perfect = if result.missing == 0 && result.remaining == 0 {
        trace_count
    } else {
        0
    };

    println!(
        "Fitness: {:.2} | Traces: {} | Perfect: {}",
        fitness, trace_count, perfect
    );
}
```

---

## Step 4 — Run the example

```sh
cargo run --example my_first_check
```

Expected output:

```
Fitness: 1.00 | Traces: 12 | Perfect: 12
```

If fitness is lower than 1.00, confirm that the arc `from`/`to` strings match the place and transition IDs exactly — the replay uses string equality for lookups.

---

## What just happened?

`apply_token_based_replay` iterates every event in every trace. For each event it:

1. Finds the transition whose `label` matches the activity name (`"A"` → `t_A`).
2. Checks that every input arc has enough tokens. If not, it increments `result.missing`.
3. Fires the transition: consumes tokens from input places, produces tokens on output places.
4. After the trace ends, any tokens still on the net add to `result.remaining`.

`compute_fitness` then applies the standard fitness formula:

```
fitness = 0.5 × (1 − missing/consumed) + 0.5 × (1 − remaining/produced)
```

All 12 traces fired perfectly, so `missing = 0` and `remaining = 0`, giving fitness `1.00`.

---

## Next steps

- **Tutorial 01** — Watch exactly how the bitmask marking changes at each step.
- **Tutorial 02** — Use the RL discovery loop to learn a net from traces.
- Explore `dteam::conformance::bitmask_replay::NetBitmask64` for the fast u64-bitmask path when your net has 64 or fewer places.
