# Watch How a Trace Fires Through a Petri Net

**Learning goal:** Understand bitmask encoding. See marking printed as both a decimal number and its binary representation at each step of token replay.

By the end of this tutorial you will be able to:
- Explain why dteam encodes Petri net markings as `u64` bitmasks
- Trace through good and bad token replay by hand
- Run an example that prints the marking at every firing step

---

## Background: why bitmasks?

A Petri net marking is a snapshot of how many tokens sit on each place. When each place holds at most one token (safe nets), the entire marking fits in a single `u64` where bit `i` is set if and only if place `i` has a token.

Checking whether a transition can fire becomes a bitwise AND:

```
can_fire = (marking & in_mask) == in_mask
```

Firing the transition is two more operations:

```
marking = (marking & !in_mask) | out_mask
```

No hash lookups, no allocations. The hot replay path in `dteam::conformance::bitmask_replay` uses exactly these operations via `NetBitmask64`.

---

## The net used in this tutorial

```
source (bit 0) --[t_A]--> p_a (bit 1) --[t_B]--> sink (bit 2)
```

| Place | Bit position | Bit value (decimal) | Binary |
|---|---|---|---|
| `source` | 0 | 1 | `0b001` |
| `p_a` | 1 | 2 | `0b010` |
| `sink` | 2 | 4 | `0b100` |

Transition masks:

| Transition | Label | in_mask | out_mask |
|---|---|---|---|
| `t_A` | `"A"` | `0b001` (source) | `0b010` (p_a) |
| `t_B` | `"B"` | `0b010` (p_a) | `0b100` (sink) |

---

## Good trace: [A, B]

Starting marking: token on `source` → `marking = 0b001 = 1`.

**Step 1 — fire t_A (label "A"):**

```
can_fire = (0b001 & 0b001) == 0b001  → true
marking  = (0b001 & !0b001) | 0b010
         = (0b001 &  0b110) | 0b010
         = 0b000 | 0b010
         = 0b010  (= 2)
```

Token moved from `source` to `p_a`.

**Step 2 — fire t_B (label "B"):**

```
can_fire = (0b010 & 0b010) == 0b010  → true
marking  = (0b010 & !0b010) | 0b100
         = (0b010 &  0b101) | 0b100
         = 0b000 | 0b100
         = 0b100  (= 4)
```

Token moved from `p_a` to `sink`. The trace is complete. Remaining tokens on non-sink places: 0. Missing tokens: 0. This trace is **perfect**.

---

## Bad trace: [B]

Starting marking: token on `source` → `marking = 0b001 = 1`.

**Step 1 — try to fire t_B (label "B"):**

```
can_fire = (0b001 & 0b010) == 0b010
         = 0b000 == 0b010  → false
```

`p_a` has no token. The transition cannot fire. The replay records one missing token and moves on. The marking stays at `0b001`.

After the trace: one token still on `source` — that is one remaining token. The trace is **not perfect** (missing > 0, remaining > 0).

---

## Step 1 — Create `examples/replay_walk.rs`

**File:** `examples/replay_walk.rs`

```rust
/// Demonstrates bitmask token replay step-by-step.
///
/// TransMask fields are pub(crate) and not accessible from examples/.
/// This example therefore builds the bitmask arithmetic by hand to
/// show exactly what NetBitmask64 does internally.

fn print_marking(label: &str, marking: u64, n_places: usize) {
    println!(
        "  {:<30} decimal={:>3}  binary={:0>width$b}",
        label,
        marking,
        marking,
        width = n_places,
    );
}

fn try_fire(
    marking: u64,
    activity: &str,
    transition_label: &str,
    in_mask: u64,
    out_mask: u64,
    missing: &mut u32,
) -> u64 {
    if activity != transition_label {
        return marking;
    }
    let can_fire = (marking & in_mask) == in_mask;
    if can_fire {
        let new_marking = (marking & !in_mask) | out_mask;
        println!(
            "  fire {:>4}  in_mask={:03b}  out_mask={:03b}  can_fire=true",
            transition_label, in_mask, out_mask,
        );
        new_marking
    } else {
        println!(
            "  fire {:>4}  in_mask={:03b}  out_mask={:03b}  can_fire=false  → missing token",
            transition_label, in_mask, out_mask,
        );
        *missing += 1;
        marking // marking unchanged
    }
}

fn replay_trace(name: &str, activities: &[&str]) {
    // Bitmask encoding for 3-place net: source=bit0, p_a=bit1, sink=bit2
    let source_bit: u64 = 0b001; // place 0
    let p_a_bit: u64    = 0b010; // place 1
    let sink_bit: u64   = 0b100; // place 2

    // Transition masks
    let t_a_in  = source_bit;
    let t_a_out = p_a_bit;
    let t_b_in  = p_a_bit;
    let t_b_out = sink_bit;

    let final_mask = sink_bit; // expected final marking

    let n_places = 3;
    let mut marking = source_bit; // initial marking: token on source
    let mut missing: u32 = 0;

    println!();
    println!("=== trace: {} ===", name);
    print_marking("initial", marking, n_places);

    for (step, &activity) in activities.iter().enumerate() {
        println!("  -- step {} activity='{}' --", step + 1, activity);

        // Try t_A
        let after_a = try_fire(marking, activity, "t_A", t_a_in, t_a_out, &mut missing);
        // Try t_B (on whichever marking resulted from t_A attempt)
        let after_b = try_fire(after_a, activity, "t_B", t_b_in, t_b_out, &mut missing);

        marking = after_b;
        print_marking("marking after step", marking, n_places);
    }

    // Remaining: tokens on non-final places
    let remaining_bits = marking & !final_mask;
    let remaining = remaining_bits.count_ones();

    println!("  -- end of trace --");
    println!("  missing={}  remaining={}  perfect={}",
        missing, remaining, missing == 0 && remaining == 0);
}

fn main() {
    println!("Token replay walk — 3-place net");
    println!("  source=bit0  p_a=bit1  sink=bit2");
    println!("  t_A: source→p_a  |  t_B: p_a→sink");

    replay_trace("[A, B] (good trace)", &["A", "B"]);
    replay_trace("[B]    (bad trace) ", &["B"]);
}
```

---

## Step 2 — Run the example

```sh
cargo run --example replay_walk
```

Expected output (abridged):

```
Token replay walk — 3-place net
  source=bit0  p_a=bit1  sink=bit2
  t_A: source→p_a  |  t_B: p_a→sink

=== trace: [A, B] (good trace) ===
  initial                        decimal=  1  binary=001
  -- step 1 activity='A' --
  fire  t_A  in_mask=001  out_mask=010  can_fire=true
  marking after step             decimal=  2  binary=010
  -- step 2 activity='B' --
  fire  t_B  in_mask=010  out_mask=100  can_fire=true
  marking after step             decimal=  4  binary=100
  -- end of trace --
  missing=0  remaining=0  perfect=true

=== trace: [B]    (bad trace)  ===
  initial                        decimal=  1  binary=001
  -- step 1 activity='B' --
  fire  t_A  in_mask=001  out_mask=010  can_fire=false  → missing token
  fire  t_B  in_mask=010  out_mask=100  can_fire=false  → missing token
  marking after step             decimal=  1  binary=001
  -- end of trace --
  missing=2  remaining=1  perfect=false
```

---

## Connection to `NetBitmask64`

`dteam::conformance::bitmask_replay::NetBitmask64` does exactly what the example does manually, but at scale:

- `NetBitmask64::from_petri_net` assigns bit positions to places and precomputes `TransMask { in_mask, out_mask }` for every transition.
- The `label_index` field sorts transitions by label for O(log n) binary search during replay.
- `invisible_indices` enables a fixpoint loop that fires invisible (tau) transitions before each visible step, keeping the marking as advanced as possible without consuming input activities.

The `ReplayResult` struct returned by bitmask replay carries the same four counters (`produced`, `consumed`, `missing`, `remaining`) used to compute fitness.

---

## Next steps

- **Tutorial 02** — Use the RL agent to discover a net from traces automatically.
- Read `src/conformance/bitmask_replay.rs` for the complete `NetBitmask64` implementation including the invisible-transition fixpoint.
