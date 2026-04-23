# Hello dteam — Load an Event Log and Run Conformance Replay

## Overview

This tutorial teaches you how to load a XES event log from disk, construct a Petri net in Rust code, and run token-based conformance replay to produce per-trace fitness scores. By the end you will have a working `examples/hello_conformance.rs` binary that prints average fitness and the five worst-fitting traces, and you will understand when dteam's bitmask fast path engages versus the standard fallback.

## Prerequisites

- Rust 1.78 or later (`rustup show`)
- Cargo (ships with Rust)
- dteam added as a dependency in your `Cargo.toml`:

```toml
[dependencies]
dteam = { path = "." }   # or a version/git reference
```

If you are working inside the dteam repository itself, all examples in `examples/` are already part of the crate — no dependency change is needed.

## Step 1 — Create the example file

Create `examples/hello_conformance.rs` (the file does not exist yet):

```
examples/hello_conformance.rs
```

All code from Steps 2–8 lives in this single file.

## Step 2 — Import the necessary types

```rust
use dteam::io::xes::XESReader;
use dteam::conformance::{token_replay, token_replay_projected, ConformanceResult, ProjectedLog};
use dteam::models::petri_net::{Arc, PetriNet, Place, Transition};
use dteam::utils::dense_kernel::{fnv1a_64, PackedKeyTable};
use std::path::Path;
```

`XESReader` lives in `dteam::io::xes`. The conformance functions and types are re-exported from `dteam::conformance` (which is also re-exported at the crate root via `pub use conformance::*`, so `dteam::token_replay` also works). The Petri net node types live in `dteam::models::petri_net`. `fnv1a_64` and `PackedKeyTable` are the two primitives that power all markings and fast-path lookups.

## Step 3 — Load the event log

```rust
fn load_log(path: &str) -> dteam::models::EventLog {
    let reader = XESReader::new();
    reader
        .read(Path::new(path))
        .expect("Failed to read XES file")
}
```

`XESReader::read` takes a `&Path` and returns `Result<EventLog, XesError>`. The returned `EventLog` contains:

- `log.traces` — a `Vec<Trace>`, one entry per `<trace>` element in the XES file
- Each `Trace` has `trace.id` (the value of the `concept:name` string attribute on the trace element) and `trace.events` — a `Vec<Event>`
- Each `Event` has `event.attributes` — a `Vec<Attribute>` where each `Attribute` carries a `key: String` and a `value: AttributeValue`
- The activity label of an event is the `Attribute` whose `key == "concept:name"` and whose value is `AttributeValue::String(s)`

```rust
// Inspect the first trace
let log = load_log("my_process.xes");
if let Some(trace) = log.traces.first() {
    println!("trace id: {}", trace.id);
    for event in &trace.events {
        if let Some(attr) = event.attributes.iter().find(|a| a.key == "concept:name") {
            if let dteam::models::AttributeValue::String(name) = &attr.value {
                println!("  event: {}", name);
            }
        }
    }
}
```

## Step 4 — Construct a Petri net in code

This step builds a simple four-activity sequential net by hand — the same technique you will use to inject a reference model in conformance checking.

```rust
fn build_purchase_net() -> PetriNet {
    // Places
    let p_start   = Place { id: "p_start".to_string() };
    let p_after_a = Place { id: "p_after_a".to_string() };
    let p_after_b = Place { id: "p_after_b".to_string() };
    let p_after_c = Place { id: "p_after_c".to_string() };
    let p_end     = Place { id: "p_end".to_string() };

    // Transitions — id is the internal identifier; label is matched against
    // the "concept:name" attribute in each event.
    let t_a = Transition { id: "t_a".to_string(), label: "Create PO".to_string(),   is_invisible: None };
    let t_b = Transition { id: "t_b".to_string(), label: "Approve PO".to_string(),  is_invisible: None };
    let t_c = Transition { id: "t_c".to_string(), label: "Send Invoice".to_string(), is_invisible: None };
    let t_d = Transition { id: "t_d".to_string(), label: "Close PO".to_string(),    is_invisible: None };

    // Arcs — each Arc connects a place to a transition (input arc) or a
    // transition to a place (output arc) by their string IDs.
    let arcs = vec![
        Arc { from: "p_start".to_string(),   to: "t_a".to_string(), weight: Some(1) },
        Arc { from: "t_a".to_string(),   to: "p_after_a".to_string(), weight: Some(1) },
        Arc { from: "p_after_a".to_string(), to: "t_b".to_string(), weight: Some(1) },
        Arc { from: "t_b".to_string(),   to: "p_after_b".to_string(), weight: Some(1) },
        Arc { from: "p_after_b".to_string(), to: "t_c".to_string(), weight: Some(1) },
        Arc { from: "t_c".to_string(),   to: "p_after_c".to_string(), weight: Some(1) },
        Arc { from: "p_after_c".to_string(), to: "t_d".to_string(), weight: Some(1) },
        Arc { from: "t_d".to_string(),   to: "p_end".to_string(),   weight: Some(1) },
    ];

    // Initial marking: one token in p_start.
    // PackedKeyTable<String, usize> stores (fnv1a_64(place_id), place_id, token_count).
    let mut initial_marking: PackedKeyTable<String, usize> = PackedKeyTable::new();
    initial_marking.insert(
        fnv1a_64(b"p_start"),
        "p_start".to_string(),
        1,
    );

    // Final marking: one token in p_end.
    let mut final_marking: PackedKeyTable<String, usize> = PackedKeyTable::new();
    final_marking.insert(
        fnv1a_64(b"p_end"),
        "p_end".to_string(),
        1,
    );

    let mut net = PetriNet {
        places:          vec![p_start, p_after_a, p_after_b, p_after_c, p_end],
        transitions:     vec![t_a, t_b, t_c, t_d],
        arcs,
        initial_marking,
        final_markings:  vec![final_marking],
        cached_incidence: None,
        cached_index:     None,
    };

    // compile_incidence() caches the flat incidence matrix and dense node index.
    // This is optional for correctness but enables the branchless kernel and
    // is_structural_workflow_net() fast path.
    net.compile_incidence();
    net
}
```

The `PackedKeyTable<String, usize>` marking maps `fnv1a_64(place_id_bytes)` to a token count. You must use `fnv1a_64` as the hash key — this is the universal spine for all dteam identity operations.

## Step 5 — Run token replay

```rust
fn run_conformance(log: &dteam::models::EventLog, net: &PetriNet) -> Vec<ConformanceResult> {
    // token_replay dispatches automatically:
    //   net.places.len() <= 64  ->  u64 bitmask fast path (nanosecond-scale per trace)
    //   net.places.len()  > 64  ->  replay_trace_standard fallback (general, heap-based)
    token_replay(log, net)
}
```

`token_replay` returns `Vec<ConformanceResult>` with one entry per trace in the log. Each `ConformanceResult` has:

- `case_id: String` — the trace ID
- `fitness: f64` — a value in `[0.0, 1.0]` where 1.0 means perfectly conforming
- `deviations: Vec<TokenReplayDeviation>` — currently empty in the bitmask path; populated by the standard fallback for nets with more than 64 places

The bitmask fast path engages automatically when `net.places.len() <= 64`. This covers the overwhelming majority of process models and runs at nanosecond latency per trace with zero heap allocation on the hot path. No user configuration is needed to activate it.

## Step 6 — Print and interpret results

```rust
fn report(results: &[ConformanceResult]) {
    if results.is_empty() {
        println!("No traces in log.");
        return;
    }

    let avg_fitness: f64 = results.iter().map(|r| r.fitness).sum::<f64>()
        / results.len() as f64;

    println!("Traces replayed : {}", results.len());
    println!("Average fitness : {:.4}", avg_fitness);
    println!();

    // Sort descending by fitness — worst-fitting traces are at the tail.
    let mut sorted = results.to_vec();
    sorted.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());

    println!("5 worst-fitting traces:");
    for r in sorted.iter().take(5) {
        println!("  case={:<20}  fitness={:.4}", r.case_id, r.fitness);
    }
}
```

Interpreting fitness values:

| Fitness | Meaning |
|---------|---------|
| 1.0 | Every event in the trace corresponds to a firable transition; the final marking is reached. |
| 0.8–0.99 | Minor deviations — a few events did not match or the final place was not reached. |
| 0.5–0.79 | Significant deviations — the model and log diverge frequently. |
| < 0.5 | The trace is structurally inconsistent with the model. Consider whether the log or model is wrong. |

A fitness of 0.0 does not mean the trace is empty; it means every token that was needed was missing. This often indicates a model that does not contain any of the activities in the trace.

## Step 7 — The projected fast path

For large logs with many repeated trace variants, `ProjectedLog` deduplicates traces and assigns a frequency weight. `token_replay_projected` then produces a single aggregate fitness weighted by frequency:

```rust
fn run_projected_conformance(log: &dteam::models::EventLog, net: &PetriNet) -> f64 {
    // ProjectedLog::from(&log) calls ProjectedLog::generate internally.
    // It deduplicates traces and stores (activity_index_vec, frequency) pairs.
    let projected = ProjectedLog::from(log);

    println!("Distinct trace variants : {}", projected.traces.len());
    println!("Distinct activities     : {}", projected.activities.len());

    // token_replay_projected returns a single f64 in [0.0, 1.0].
    // Internally it uses the same u64 bitmask kernel as token_replay.
    let aggregate_fitness = token_replay_projected(&projected, net);
    aggregate_fitness
}
```

Use the projected path when:
- The log has more than a few thousand traces and you need a single population-level fitness number quickly.
- You are running a discovery loop where per-trace detail is not required.

Use `token_replay` (per-trace) when:
- You need the `case_id` for individual trace diagnostics.
- You are building a dashboard that surfaces per-case compliance status.

## Step 8 — Run the example

Assemble the complete `examples/hello_conformance.rs`:

```rust
use dteam::conformance::{token_replay, token_replay_projected, ConformanceResult, ProjectedLog};
use dteam::io::xes::XESReader;
use dteam::models::petri_net::{Arc, PetriNet, Place, Transition};
use dteam::utils::dense_kernel::{fnv1a_64, PackedKeyTable};
use std::path::Path;

fn build_purchase_net() -> PetriNet {
    let p_start   = Place { id: "p_start".to_string() };
    let p_after_a = Place { id: "p_after_a".to_string() };
    let p_after_b = Place { id: "p_after_b".to_string() };
    let p_after_c = Place { id: "p_after_c".to_string() };
    let p_end     = Place { id: "p_end".to_string() };

    let t_a = Transition { id: "t_a".to_string(), label: "Create PO".to_string(),    is_invisible: None };
    let t_b = Transition { id: "t_b".to_string(), label: "Approve PO".to_string(),   is_invisible: None };
    let t_c = Transition { id: "t_c".to_string(), label: "Send Invoice".to_string(), is_invisible: None };
    let t_d = Transition { id: "t_d".to_string(), label: "Close PO".to_string(),     is_invisible: None };

    let arcs = vec![
        Arc { from: "p_start".to_string(),   to: "t_a".to_string(),       weight: Some(1) },
        Arc { from: "t_a".to_string(),        to: "p_after_a".to_string(), weight: Some(1) },
        Arc { from: "p_after_a".to_string(), to: "t_b".to_string(),       weight: Some(1) },
        Arc { from: "t_b".to_string(),        to: "p_after_b".to_string(), weight: Some(1) },
        Arc { from: "p_after_b".to_string(), to: "t_c".to_string(),       weight: Some(1) },
        Arc { from: "t_c".to_string(),        to: "p_after_c".to_string(), weight: Some(1) },
        Arc { from: "p_after_c".to_string(), to: "t_d".to_string(),       weight: Some(1) },
        Arc { from: "t_d".to_string(),        to: "p_end".to_string(),     weight: Some(1) },
    ];

    let mut initial_marking: PackedKeyTable<String, usize> = PackedKeyTable::new();
    initial_marking.insert(fnv1a_64(b"p_start"), "p_start".to_string(), 1);

    let mut final_marking: PackedKeyTable<String, usize> = PackedKeyTable::new();
    final_marking.insert(fnv1a_64(b"p_end"), "p_end".to_string(), 1);

    let mut net = PetriNet {
        places:          vec![p_start, p_after_a, p_after_b, p_after_c, p_end],
        transitions:     vec![t_a, t_b, t_c, t_d],
        arcs,
        initial_marking,
        final_markings:  vec![final_marking],
        cached_incidence: None,
        cached_index:     None,
    };
    net.compile_incidence();
    net
}

fn main() {
    // --- Build a small synthetic log for demonstration ---
    // Replace this block with XESReader::new().read(Path::new("my.xes")).unwrap()
    // to load a real file.
    use dteam::models::{AttributeValue, Attribute, Event, EventLog, Trace};

    let mut log = EventLog::new();

    // Conforming trace
    let mut t1 = Trace::new("case-001".to_string());
    for label in &["Create PO", "Approve PO", "Send Invoice", "Close PO"] {
        t1.events.push(Event::new(label.to_string()));
    }

    // Non-conforming trace — skips "Approve PO"
    let mut t2 = Trace::new("case-002".to_string());
    for label in &["Create PO", "Send Invoice", "Close PO"] {
        t2.events.push(Event::new(label.to_string()));
    }

    // Duplicate conforming trace (different case id)
    let mut t3 = Trace::new("case-003".to_string());
    for label in &["Create PO", "Approve PO", "Send Invoice", "Close PO"] {
        t3.events.push(Event::new(label.to_string()));
    }

    log.add_trace(t1);
    log.add_trace(t2);
    log.add_trace(t3);

    // To load from XES instead:
    // let log = XESReader::new().read(Path::new("my_process.xes")).unwrap();

    let net = build_purchase_net();

    // --- Per-trace conformance ---
    let results: Vec<ConformanceResult> = token_replay(&log, &net);

    let avg: f64 = results.iter().map(|r| r.fitness).sum::<f64>() / results.len() as f64;
    println!("Traces replayed : {}", results.len());
    println!("Average fitness : {:.4}", avg);
    println!();

    let mut sorted = results.clone();
    sorted.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
    println!("5 worst-fitting traces:");
    for r in sorted.iter().take(5) {
        println!("  case={:<20}  fitness={:.4}", r.case_id, r.fitness);
    }

    // --- Projected (aggregate) conformance ---
    println!();
    let projected = ProjectedLog::from(&log);
    println!("Distinct variants   : {}", projected.traces.len());
    println!("Distinct activities : {}", projected.activities.len());
    let agg = token_replay_projected(&projected, &net);
    println!("Aggregate fitness   : {:.4}", agg);
}
```

Run it:

```
cargo run --example hello_conformance
```

Expected output (synthetic log):

```
Traces replayed : 3
Average fitness : 0.8667

5 worst-fitting traces:
  case=case-002              fitness=0.6000
  case=case-001              fitness=1.0000
  case=case-003              fitness=1.0000

Distinct variants   : 2
Distinct activities : 4
Aggregate fitness   : 0.8333
```

The aggregate fitness from the projected path differs from the arithmetic mean because it is frequency-weighted: case-001 and case-003 share the same variant, so that variant receives weight 2 while the non-conforming variant receives weight 1.

## What you learned

- `XESReader::new().read(path)` parses a `.xes` file into an `EventLog` containing `Vec<Trace>`, each `Trace` holding `Vec<Event>`, each `Event` carrying `Vec<Attribute>`.
- Activity labels are stored as `Attribute { key: "concept:name", value: AttributeValue::String(s) }`.
- A `PetriNet` is built from `Vec<Place>`, `Vec<Transition>`, `Vec<Arc>`, plus `PackedKeyTable<String, usize>` markings keyed by `fnv1a_64(place_id_bytes)`.
- `compile_incidence()` caches the flat incidence matrix and dense node index — call it once after construction.
- `token_replay(&log, &net)` returns `Vec<ConformanceResult>` with per-trace `case_id` and `fitness`.
- The bitmask fast path (u64 marking, zero heap) activates automatically for nets with 64 or fewer places.
- `ProjectedLog::from(&log)` deduplicates trace variants; `token_replay_projected` produces a single frequency-weighted aggregate fitness.

## Next steps

Tutorial 2 shows how to use the `Engine` API to discover a Petri net from an event log using the RL-based discovery loop, and how to read the `ExecutionManifest` to verify and reproduce results.
