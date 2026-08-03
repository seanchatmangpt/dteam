# Debug a Conformance Replay Failure

This guide explains how to diagnose a failing token-based conformance replay, interpret
`ReplayResult` fields, and check Petri net structural soundness.

---

## The two replay paths

dteam uses two replay implementations selected at trace time:

| Path | Condition | Location |
|---|---|---|
| Bitmask (fast) | `net.places.len() <= 64` | `src/conformance/bitmask_replay.rs` |
| Standard (fallback) | `places.len() > 64` | `src/conformance/case_centric/token_based_replay.rs` (feature-gated) |

The bitmask path represents the marking as a single `u64` where bit `i` corresponds to
place index `i`. It uses bitwise AND/OR for firing checks and is the primary path for all
real PDC 2025 nets which have far fewer than 64 places.

The standard path is gated behind the `token-based-replay` feature (enabled by default).
It handles arbitrary-sized nets using `PackedKeyTable` markings.

---

## `ReplayResult` fields

Defined at `src/conformance/bitmask_replay.rs` lines 6-11:

```rust
pub struct ReplayResult {
    pub missing: u32,    // tokens artificially added to fire a transition
    pub remaining: u32,  // tokens left in the net after the trace ends
    pub produced: u32,   // total tokens produced (fired + initial)
    pub consumed: u32,   // total tokens consumed by firings
}
```

| Field | What it counts |
|---|---|
| `missing` | Each time a transition could not fire because a required input place had no token, one token is added artificially. High `missing` = the trace contains activities not enabled by the net. |
| `remaining` | Tokens left in non-final places after the trace ends. High `remaining` = the trace ended without reaching the final marking. |
| `produced` | Total tokens placed on places, including initial marking. |
| `consumed` | Total tokens removed from places by transition firings. |

---

## Interpreting the 4 patterns

| `missing` | `remaining` | Interpretation |
|---|---|---|
| `> 0` | `= 0` | Trace contains activities that the net cannot enable — event log is ahead of the model. The net is missing transitions or arcs. |
| `= 0` | `> 0` | Trace completes but the net was not fully consumed — some branches were never activated. The trace is a proper prefix of the model or the net has dead branches. |
| `> 0` | `> 0` | Both: the trace forced tokens in AND left tokens behind. Combined model-log mismatch. |
| `= 0` | `= 0` | Perfect replay. The trace is fully conformant. `fitness() == 1.0`. |

---

## Fitness formula

Both paths compute fitness as a value in `[0.0, 1.0]`:

### Bitmask path (`src/conformance/bitmask_replay.rs` lines 14-24)

```
fitness = 1.0 - (missing + remaining) / (consumed + missing + produced)
```

Special case: if `consumed + missing == 0` and `produced == 0`, return `1.0` (empty trace
against empty net).

### Standard path

Uses the same conceptual formula but operates over `PackedKeyTable` markings. The exact
implementation is in `src/conformance/case_centric/token_based_replay.rs`. The
`is_perfect()` helper (`missing == 0 && remaining == 0`) is common to both.

A fitness of `1.0` is a necessary but not sufficient condition for full conformance: it
means the trace could be replayed with zero artificial tokens, but the net may still have
structural issues.

---

## Checking structural soundness

Call `is_structural_workflow_net()` on your `PetriNet` before replay if you suspect the
net itself is malformed. Defined at `src/models/petri_net.rs` line 82:

```rust
pub fn is_structural_workflow_net(&self) -> bool
```

Returns `false` (unsound) if any of these conditions hold:
- No places or no transitions.
- More than one source place (place with no incoming arcs).
- More than one sink place (place with no outgoing arcs).
- Any transition has no incoming arc or no outgoing arc (dead transition).

Use it in a debug session:

```rust
if !net.is_structural_workflow_net() {
    eprintln!("Net is structurally unsound — replay results unreliable");
}
let result = replay_trace_bitmask(&net_bitmask, &trace);
```

The `structural_unsoundness_score` method returns a float penalty used by the RL reward
surface; it does not need to be called for debugging, only `is_structural_workflow_net()`
is required for a boolean pass/fail.

---

## Single-log debug mode

Run the pipeline on a single log with full trace output:

```bash
RUST_LOG=debug cargo run --bin pdc2025 -- --stem=<log_stem>
```

`--stem=<log_stem>` filters to a single XES file (without `.xes` extension).
`RUST_LOG=debug` enables the `log` crate debug output across all modules including the
conformance replay loop.

Look for lines containing `replay` or `fitness` to see per-trace replay results. The
bitmask path logs transition firing decisions at `log::trace!` level; upgrade to
`RUST_LOG=trace` if you need transition-level detail.

---

## Common root causes

| Symptom | Likely cause | Fix |
|---|---|---|
| All traces: `missing > 0`, `remaining = 0` | Net lacks a sink transition or has wrong final marking | Check `net.final_markings`; ensure sink place index matches |
| All traces: `missing = 0`, `remaining > 0` | Net never reaches final place | Verify initial marking includes the source place |
| Bitmask path panics with "requires ≤64 places" | Net has >64 places | Use the standard replay path; check feature flag `token-based-replay` |
| Fitness oscillates between traces | Non-deterministic net with silent transitions | Inspect invisible transition fixpoint loop in `NetBitmask64` |
| `is_structural_workflow_net()` returns `false` | Mined net has unconnected transitions | Apply structural repair or use a manually crafted net |
