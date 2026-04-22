# 60 — Three.js + Next.js Interface for the Neuromancer TV Series

## The pitch

The 33 arena tests in `unibit-e2e` are already a scripted corpus of
runs — admit, deny, tamper, route, seal, verify. The Gibson Sprawl
trilogy (*Neuromancer*, *Count Zero*, *Mona Lisa Overdrive*) contains
~40 scripted runs that map one-to-one onto our architecture's
operations.

If Apple/A24/whoever greenlights the Neuromancer TV adaptation, the
on-screen "cyberspace" visualisation is exactly our 64³ globe. The
frame is built; it just needs a camera and a shader.

This doc is the design for that interface, plus the corpus of Gibson
runs to arena-ify.

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  Next.js 15 (App Router, RSC) — show-control UI          │
│  ┌───────────────────────────────────────────────────┐   │
│  │  Three.js scene — 64³ globe renderer              │   │
│  │  - Truth/Scratch pair as two concentric spheres   │   │
│  │  - Field lanes as 8 colored axes                  │   │
│  │  - Admitted motions as green geodesics            │   │
│  │  - Denied motions as red flares                   │   │
│  │  - Receipt chain as a Cornell-box ribbon          │   │
│  └───────────────────────────────────────────────────┘   │
│  ┌───────────────────────────────────────────────────┐   │
│  │  Run selector + episode timeline                  │   │
│  └───────────────────────────────────────────────────┘   │
│  ┌───────────────────────────────────────────────────┐   │
│  │  Receipt panel — Turing-Police verdict, depth,    │   │
│  │  L0..L5 fragments                                 │   │
│  └───────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
                          │ WebSocket + JSON-RPC
                          ▼
┌──────────────────────────────────────────────────────────┐
│  Node side-car — bridges to unibit_motion_tick (C-ABI)   │
│  via napi-rs or Rust→WASM compile                        │
└──────────────────────────────────────────────────────────┘
```

**Key insight:** the same `unibit-cabi` entry point that a production
NIF uses also feeds the visualization. One binary, two frontends.

---

## Scene elements ↔ architecture

| On-screen | unibit concept | Visual |
|---|---|---|
| Outer sphere (64³) | TruthBlock | 32 KiB as 64-lat × 64-lon × 64-radius grid; each cell a glowing point |
| Inner sphere | Scratchpad | shadow of Truth; differentially highlighted |
| 8 colored great-circles | FieldLane (Prereq/Capability/Causality/Conformance/Law/Scenario/Risk/Attention) | each lane gets a distinct hue; required lanes warm, forbidden lanes cool |
| Geodesic arc | POWL64 Geodesic edge | drawn from source cell to dest cell; green = admitted, red = denied |
| Flare | LaneOutcome::Deny | burst at the offending cell, color-matched to the offending lane |
| Cornell-box ribbon | Receipt chain | each fragment a small "box" in a floating ribbon, like the Boxmaker's assemblages |
| Turing police badge | Verifier verdict | Lawful = green seal; Unlawful = red quarantine halo |
| Watchdog countdown | Watchdog counter | thin ring around the globe; decrements with each cycle |
| Ripple | ReduceBuffer fan-in | 8 concentric ripples converging on center |

---

## Gibson Sprawl runs → arena mapping

Each of these is a scene from the books that maps onto one of our 33
arenas (or new arenas to add). An episode's "hacker scene" becomes a
scripted run through the visualiser.

### Neuromancer (1984)

| Run | Gibson scene | Arena | Visual highlight |
|---|---|---|---|
| N1 | Case's first jack-in at the Chiba clinic | `arena_15_l1_region` | Pin<Box<L1Region>> materialising; position receipt flares |
| N2 | Dixie Flatline construct replay | `arena_03_snapshot_nesting` | nested Snapshot.inner glows recursively |
| N3 | First ICE encounter (Sense/Net) | `arena_17_hot_kernels::admit8_denies_on_forbidden_bit_set` | red flare on Law lane |
| N4 | Sushi bar meet; Wintermute's phone | `arena_12_powl_lockstep` | compile: semantic intent → MotionPacket |
| N5 | Villa Straylight approach | `arena_15_l1_region` | HotRegion rotates into view; layout asserts flash |
| N6 | Ratz's bar, the scorched trodes | `arena_04_watchdog_liveness::starve_trips_the_watchdog` | watchdog ring dims then fires |
| N7 | Turing police raid on the Villa | `arena_07_chain_replay::tampered_fast_receipt_reports_causal_only` | red quarantine halo |
| N8 | Wintermute + Neuromancer fusion | `arena_03` deep verify | two Snapshots merge; seal re-derives |
| N9 | Molly's knives / corporate run on Sense/Net | `arena_13_worker_pool::pool_denies_missing_bits_across_1000_dispatches` | 1000 geodesics; some red, some green |
| N10 | The flatline's last laugh | `arena_22_snapshot_seal::leaf_vs_wrapped…` | seal difference is visible |

### Count Zero (1986)

| Run | Gibson scene | Arena | Visual highlight |
|---|---|---|---|
| CZ1 | Bobby Newmark's first run, near-flatline | `arena_04_watchdog_liveness::starve_trips_the_watchdog` | counter hits 0; screen dims |
| CZ2 | The Virek contract (biomed vat) | new arena 34 (Resident long-run) | Snapshot re-seals every cycle; depth stays constant |
| CZ3 | Legba rides Bobby | `arena_31_commandeering_override::commandeering_law_lane_drives_denial` | one lane's color saturates; others fade |
| CZ4 | The Boxmaker's assemblages | new arena 35 (orphan fragments → new Snapshot) | scattered fragments converge into a new inner Snapshot |
| CZ5 | Turner's extraction | new arena 36 (Relay across endpoints) | Snapshot travels between two globes |
| CZ6 | Marly walks the Cornell boxes | `arena_27_multi_motion_replay` | receipt ribbon animates; viewer pans along |
| CZ7 | Gentleman Loser bar — fixer trades | new arena 37 (SpscRing broker) | fragments flow through a channel |
| CZ8 | Biosoft slotted into skull | new arena 38 (SignedSnapshot verify + load) | external Snapshot docks onto active region |
| CZ9 | The Finn's shop — one-of-a-kinds | `arena_05_ring_fragments::ring_supports_u128_fragments_end_to_end` | 64 fragment boxes pulse in sequence |
| CZ10 | Void past the sprawl — orbital silence | new arena 39 (Nvm cold-storage access) | region fades from L1→L2→L3→DRAM→NVM colors |

### Mona Lisa Overdrive (1988)

| Run | Gibson scene | Arena | Visual highlight |
|---|---|---|---|
| MLO1 | Mona's first simstim session | new arena 40 (Simulator rehearse) | scratchpad-only run; no truth mutation |
| MLO2 | The Aleph — Straylight recreated | `arena_03_snapshot_nesting::deep_nesting_to_max_depth_verifies` | 16-deep Snapshot cascades in |
| MLO3 | Kumiko in London | new arena 41 (Envelope traversal) | camera descends Aleph.inner repeatedly |
| MLO4 | Angie's dustings (cortical slow-drip) | new arena 42 (Watchdog long-interval tick) | counter ticks every N cycles instead of every 1 |
| MLO5 | Gentry's shape theory | new arena 43 (geometry JTBD full coverage) | all 8 JTBDs visualized as distinct scene actions |
| MLO6 | The Count's return | `arena_08_publish_readiness::publish_readiness_determinism_under_replay` | two runs play side-by-side; receipts match bit-for-bit |
| MLO7 | Tessier-Ashpool voidsmen | `arena_23_watchdog_isolation::multiple_shards_can_trip_independently` | 8-core ring where shards go dark independently |
| MLO8 | Bobby in the Aleph | `arena_32_full_workflow` | the three-activity workflow rendered as a stack of Snapshots |
| MLO9 | Slick Henry's Judge | new arena 44 (large-scale Construct assembly) | Boxmaker assembly animation |
| MLO10 | The Jammer's last run | `arena_33_publish_checklist::publish_checklist_all_green` | every indicator greens; the show closes |

---

## Arenas to add (34–44) to reach full corpus

| # | Arena | Purpose |
|---|---|---|
| 34 | `arena_34_resident_long_run` | Virek vat — repeated re-seal without depth growth |
| 35 | `arena_35_orphan_assembler` | Boxmaker — orphan fragment → assembled Snapshot |
| 36 | `arena_36_endpoint_transfer` | Turner — Snapshot move between endpoints |
| 37 | `arena_37_broker_channel` | Finn — SpscRing brokering between two motion pipelines |
| 38 | `arena_38_signed_snapshot` | Biosoft — signed external Snapshot load + verify |
| 39 | `arena_39_residence_ladder` | Voidspace — traverse Reg→L1→L2→L3→DRAM→NVM tiers |
| 40 | `arena_40_simstim_rehearse` | Simstim — scratchpad-only counterfactual |
| 41 | `arena_41_envelope_descend` | Kumiko — walk Snapshot.inner recursively with breadcrumbs |
| 42 | `arena_42_long_interval_tick` | Slow-drip — Watchdog ticks every N cycles |
| 43 | `arena_43_geometry_full_jtbd` | Gentry — all 8 JTBDs run in one scripted sequence |
| 44 | `arena_44_construct_assembly` | Judge — Boxmaker + Turner + Finn + Marly composed |

All 11 additions use existing unibit types; no new crates needed.

---

## Next.js + Three.js surface

### File layout

```
apps/matrix-tv/
├── app/
│   ├── layout.tsx
│   ├── page.tsx                         — episode selector
│   ├── episode/[slug]/
│   │   ├── page.tsx                     — scene viewer
│   │   └── controls.tsx                 — run buttons
│   └── api/
│       └── motion/
│           └── route.ts                 — /api/motion → NAPI binding
├── lib/
│   ├── unibit.ts                        — TS types for MotionPacket/Response
│   ├── scene.ts                         — Three.js setup
│   └── runs.ts                          — the 44 arena scripts
├── components/
│   ├── GlobeRenderer.tsx                — the 64³ sphere
│   ├── LaneAxes.tsx                     — the 8 colored lanes
│   ├── ReceiptRibbon.tsx                — Cornell-box chain
│   ├── WatchdogRing.tsx                 — countdown ring
│   └── VerdictBadge.tsx                 — Turing Police seal
├── public/
│   └── shaders/
│       ├── globe.vert
│       ├── globe.frag                   — signed-distance globe cells
│       ├── lane.frag                    — per-lane hue
│       └── flare.frag                   — denial flare
├── package.json
├── next.config.ts
└── tsconfig.json
```

### Dependencies

```json
{
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19",
    "react-dom": "^19",
    "three": "^0.170",
    "@react-three/fiber": "^9",
    "@react-three/drei": "^10",
    "@react-three/postprocessing": "^3"
  },
  "devDependencies": {
    "typescript": "^5.6",
    "@napi-rs/cli": "^2.18"
  }
}
```

### Node bridge

The simplest way: compile `unibit-cabi` as a N-API addon via
`napi-rs`. That gives TS a typed `motionTick(request) → response`.

```rust
// apps/matrix-tv/crates/matrix-bridge/src/lib.rs
use napi_derive::napi;
use unibit_cabi::{unibit_motion_tick, UnibitMotionRequest, UnibitMotionResponse};

#[napi]
pub fn motion_tick(req_bytes: Vec<u8>) -> Result<Vec<u8>> {
    // deserialise, call through, serialise
}
```

TS call-site:
```ts
import { motionTick } from '@matrix/bridge';
const resp = motionTick(Buffer.from(req));
```

---

## Shader strategy

### Globe cells as signed-distance sprites

Each cell is a 3D point on a lat/lon/radius grid. The fragment shader
computes admission-state-to-color:

```glsl
// globe.frag (GLSL ES 3.0)
uniform uint state[4096];     // TruthBlock, 4096 × u64 packed as u32 pairs
uniform uint mask_required[4];
uniform uint mask_forbidden[4];

void main() {
    uint word = state[gl_CellIndex / 2];
    bool bit = (word >> (gl_CellIndex & 63)) & 1u;
    vec3 color = bit ? vec3(0.6, 0.9, 1.0) : vec3(0.05, 0.05, 0.1);
    // fade by distance from camera for cell-density hinting
    fragColor = vec4(color, 1.0);
}
```

The shader doesn't know or care about the 8-lane semantics; that's the
frontend's job to overlay via great-circle lines.

### Lane axes

Eight great circles around the sphere, each in the lane's signature
color. When a motion admits, the responsible lane's arc flashes. On
denial, the violating lane's flare fires.

```glsl
// lane color palette (from doc 26)
const vec3 LANE_HUES[8] = vec3[8](
    vec3(0.3, 0.7, 1.0),  // prereq      — cold blue
    vec3(0.8, 0.2, 0.2),  // law         — crimson
    vec3(0.9, 0.5, 0.1),  // capability  — amber
    vec3(0.6, 0.1, 0.7),  // scenario    — purple
    vec3(0.2, 0.8, 0.3),  // risk/reward — green
    vec3(0.9, 0.9, 0.3),  // causality   — yellow
    vec3(0.7, 0.7, 0.7),  // conformance — silver
    vec3(1.0, 0.4, 0.8)   // attention   — rose
);
```

---

## Episode structure

An episode is a sequence of runs. The `lib/runs.ts` file defines each
run as a triple of (input state, expected output, camera choreography).

```ts
// lib/runs.ts
export const RUN_N2_DIXIE_FLATLINE_REPLAY: Run = {
  id: 'N2',
  title: "Dixie Flatline construct replay",
  arena: 'arena_03_snapshot_nesting',
  inputState: {...},
  expected: { verdict: 'Lawful', depth: 2 },
  camera: {
    path: [
      { pos: [0, 0, 10], look: [0, 0, 0], t: 0 },
      { pos: [3, 2, 4], look: [0, 0, 0], t: 2.5 },
      { pos: [0, 5, 2], look: [0, 0, 0], t: 5.0 },
    ],
    annotations: [
      { t: 1.0, text: "Pin<Box<L1Region>> materialising" },
      { t: 3.0, text: "Inner Snapshot loads" },
      { t: 4.5, text: "Seal verifies; flatline complete" },
    ],
  },
};
```

A director can wire a scene by selecting runs in order and the
interface renders them with cuts between.

---

## What this delivers on screen

1. **"Hacker scene" cliché dies.** No green-on-black text waterfall.
   Instead, a glowing 64³ globe with colored lanes, visible admissions
   and denials, receipt chains materialising as Cornell boxes.

2. **Every run is real.** The shader reads actual `unibit_motion_tick`
   output. The receipts that verify on screen are the same receipts
   that verify in production.

3. **Science-advisor-grade accuracy.** For the show's technical
   consultants, every visual element maps to a measurable architectural
   concept. Interviews can explain what the audience saw.

4. **Reusable as a dev tool.** The same UI, fed by dev-mode receipts,
   is the production observability surface. Ship one app for two
   audiences.

---

## Work estimate

| Phase | Effort | Deliverable |
|---|---|---|
| 1. napi-rs bridge crate | 2 days | `motionTick(req) → resp` from TS |
| 2. Three.js globe renderer | 1 week | 64³ sphere with 4,096 live cells |
| 3. Lane axes + flares | 3 days | 8 colored great-circles + denial burst |
| 4. Receipt ribbon | 2 days | Cornell-box chain visualisation |
| 5. Run scripting DSL | 3 days | `lib/runs.ts` format + parser |
| 6. 40 Gibson runs scripted | 1 week | all Sprawl-trilogy scenes |
| 7. Episode timeline | 2 days | scene selector + camera choreography |
| 8. Arenas 34–44 in Rust | 1 week | the 11 new unibit-e2e tests |

**Total: ~5 weeks of one engineer, or 2 weeks with two.**

---

## The one-line pitch

**Convert the "hacker scene cliché" into a scientific instrument: the
same 64³ globe, the same receipt chains, the same branchless admission
algebra — visible on screen, driven by the same binary that runs in
production, faithful to every run in the Sprawl trilogy.**

---

## Next concrete step

Before any TV network, the 80/20 deliverable is a **standalone demo**:
one Next.js page that renders one arena (say `arena_08_publish_readiness`)
live in the browser, with controls to admit / deny / tamper. That's the
3-day spike that proves the rest.

If green-lit, arenas 34–44 get written in parallel with the shader
work.
