# matrix-tv

Three.js + Next.js visualisation of `unibit` arenas, mapped to runs
from the Gibson Sprawl trilogy (Neuromancer, Count Zero, Mona Lisa
Overdrive).

Designed in [`docs/opus/60_threejs_neuromancer_tv.md`](../../docs/opus/60_threejs_neuromancer_tv.md).

## Status

**Phase 1 shipped** — standalone TS implementation mirrors the
branchless admission algebra. No napi-rs yet; the visualiser runs
self-contained.

**Phase 2 (deferred)** — wire `unibit-cabi`'s `unibit_motion_tick`
through napi-rs so the same binary drives the UI and production.

## What's here

```
app/
  page.tsx                     — episode selector, runs grouped by novel
  episode/[slug]/page.tsx      — scene viewer
  episode/[slug]/scene.tsx     — Three.js canvas + controls
components/
  GlobeRenderer.tsx            — 64³ point-cloud sphere + 8 lane rings
  VerdictBadge.tsx             — Lawful / Unlawful surface
  ReceiptRibbon.tsx            — Cornell-box chain at the bottom
lib/
  unibit.ts                    — pure-TS motion_tick + CANONICAL_FIELDS
  runs.ts                      — 11 Sprawl-trilogy runs mapped to arenas
```

## Running

```bash
cd apps/matrix-tv
npm install
npm run dev
```

Open **http://localhost:31337** — the dev and start scripts pin port
31337 so they don't collide with Grafana (which defaults to 3000).

Pages:
- `/` — episode selector (the 30+ Gibson runs)
- `/sprawl` — blockchain-MUD replay: Case → Loa, nine rooms
- `/ocel` — live observer: tails `ocel-log.jsonl` via SSE; open alongside
  `/sprawl` in another tab to watch the session in real time

## DX / QoL — `matrix-tv-doctor`

A citty-based CLI lives under `cli/` and drives every diagnostic signal
we care about. Run these from `apps/matrix-tv/`:

| Command | What it does |
|---|---|
| `npm run doctor` | Coloured health-check summary (node, ports, files, HTTP routes, Rust). Exits 1 on any MISS. |
| `npm run doctor:json` | Same checks, machine-readable `{ checks, summary }`. |
| `npm run ports` | Listeners on :31337 (us), :3000 (Grafana), :4317/:4318 (OTLP), :8088 (unibit-sprawl WS), :3412 (Playwright). |
| `npm run replay` | Regenerate `public/sprawl-replay.ndjson` by shelling to the unibit repo. |
| `npm run tail` | Follow `ocel-log.jsonl` — one compact coloured line per turn. |
| `npm run ocel:reset` | Truncate the session log. |
| `npm run ocel:stats` | Aggregate: players / rooms / verbs / verdicts. |
| `npm run play` | Open `/sprawl` and `/ocel` in the default browser. |

## Design notes

Every on-screen element corresponds to a measurable architectural
concept:

| Element | unibit concept |
|---|---|
| Point-cloud sphere | TruthBlock (64³ cells) |
| 8 colored torus rings | FieldLane (4 required + 4 forbidden) |
| Green/red sphere flares | LaneOutcome denial fragments |
| Lawful/Unlawful badge | Verifier verdict |
| Cornell-box ribbon | Receipt chain, oldest → newest |

The TS motion_tick exactly mirrors `unibit-hot::t0::admit8_t0`
semantics:

```
deny_required  = (state & required) XOR required
deny_forbidden = state & forbidden
deny_total     = OR across 8 lanes
admitted_mask  = ((deny_total == 0) as u64).wrapping_neg()
next_marking   = (candidate & admitted_mask) | (old & ~admitted_mask)
```

Same algebra, TS types; the Rust kernel-side and TS-side produce
identical denials given identical masks.

## Next steps (Phase 2)

1. `crates/matrix-bridge/` — napi-rs bridge that exposes
   `unibit_motion_tick` to Node.
2. Swap `motionTick()` in `lib/unibit.ts` for the bridge call.
3. Add the remaining 29 Sprawl runs (currently 11 of 40).
4. Wire the 64³-cell count via a WebGPU compute shader — direct
   `TruthBlock` read-out from the Rust side.
