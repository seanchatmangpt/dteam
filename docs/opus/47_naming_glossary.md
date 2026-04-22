# 47 — Naming Glossary: Literary Framing vs Source Code

## The rule

Gibson's vocabulary is a **reading aid**, not an API surface. It makes
docs 43 and 44 memorable and clustered around a single coherent fiction.
It must not appear in:

- crate names
- module paths
- type names
- public function names
- public constants or trait names
- error types
- feature flags
- build-script output
- binary exports (`#[no_mangle]` symbols)

Literary framing lives in comments and `docs/opus/*`. Source code uses
the canonical technical names below.

**Enforcement:** the Sniffer (doc 44's lexicon guard, canonically
`LexiconCheck`) is extended with the forbidden-character list in §4.
Any forbidden token outside a comment fails the build.

---

## 1. The glossary (literary ↔ technical)

### Matrix crate

| Literary | Canonical name | Purpose |
|---|---|---|
| The Matrix / Cyberspace | *no type* — used only in prose | the conceptual 64³ space |
| Straylight | `HotRegion` | pinned 4 KiB-aligned L1D page |
| Deck | `HotRegion` (same type; different lens) | the Runner's working page |
| Cowboy | `Runner<const T: WorkTier>` | owner of one HotRegion; executes admissions |
| Jack in | `Runner::attach()` / `HotRegion::pin()` | acquire + mlock + position-validate |
| Trode | `Observation` | input channel to the Runner |
| Ice | `Gate` | a FieldMask bundled with a denial policy |
| White Ice | `Gate { policy: GatePolicy::Soft, .. }` | deny-only |
| Gray Ice | `Gate { policy: GatePolicy::Firm, .. }` | deny + escalate |
| Black Ice | `Gate { policy: GatePolicy::Strike, .. }` | deny + counter-emit fragment |
| IceKind enum | `GatePolicy { Soft, Firm, Strike }` | denial-strength discriminator |
| Construct | `Capsule` | sealed snapshot of final state + chain |
| Dixie Flatline | *no type* — narrative only | retired Runner's archived Capsule |
| Flatline | `Runner::finalize()` | consume Runner, emit Capsule |
| Wintermute | `Compiler` | intent → MotionPacket (front half of μ) |
| Neuromancer | `Executor` | MotionPacket → Capsule (back half of μ) |
| Turing Police | `Verifier` | independent Capsule re-seal check |
| Simstim | `Simulator` | counterfactual replay against Scratchpad |
| The Wall | `ReleaseGate` | five-surface agreement gate |
| Zaibatsu | `Endpoint` | downstream consumer of sealed Capsules |
| Chiba / Ninsei | `Archive` | cold DRAM / NVM spill tier |

### Count Zero crate

| Literary | Canonical name | Purpose |
|---|---|---|
| Count Zero Interrupt (CZI) | `Watchdog` | hot-path deadman timer |
| Count Zero (the counter) | `Watchdog::counter` | atomic countdown |
| Federated CZI | `FederatedWatchdog` | per-core shard array |
| Bobby Newmark / Newbie | `Runner` (same type; first-time flag optional) | no new type; `first_run: bool` field if needed |
| Loa | `LaneWarden` | per-lane autonomous guardian |
| Pantheon (8 Loa) | `WardenArray` / `[LaneWarden; 8]` | the eight per-core wardens |
| Legba | `LaneWarden { lane: FieldLane::Prereq, .. }` | no per-name type; use `FieldLane` |
| Ougou Feray | `LaneWarden { lane: FieldLane::Law, .. }` | ditto |
| Erzulie | `LaneWarden { lane: FieldLane::Capability, .. }` | ditto |
| Samedi | `LaneWarden { lane: FieldLane::Scenario, .. }` | ditto |
| Danbala | `LaneWarden { lane: FieldLane::Causality, .. }` | ditto |
| Met-Kalfu | `LaneWarden { lane: FieldLane::RiskReward, .. }` | ditto |
| Simbi | `LaneWarden { lane: FieldLane::Conformance, .. }` | ditto |
| Ayizan | `LaneWarden { lane: FieldLane::Attention, .. }` | ditto |
| LoaName | *removed* | `FieldLane` is the canonical identifier |
| Mood | `WardenMode` | discriminator for warden state |
| Mood::Quiet | `WardenMode::Passive` | deny only on clear violation |
| Mood::Watching | `WardenMode::Active` | normal admission |
| Mood::Restless | `WardenMode::Escalating` | promote borderline motions |
| Mood::Ridden | `WardenMode::Commandeering` | warden takes over tier choice |
| LoaVerdict | `WardenVerdict` | per-warden decision |
| LoaVerdict::Mount | `WardenVerdict::Override` | warden commandeers pipeline |
| Aleph | `Envelope` | recursive wrapper around a `Capsule` |
| Finn | `Broker<T>` | lock-free SPSC channel between HotRegions |
| Biosoft | `Cartridge` | hot-pluggable verified Capsule |
| Josef Virek | `Resident` | feature-gated infinite re-admission loop |
| Virek's vat | `ResidentHost` | E-core host for a Resident |
| Turner | `Relay` | Endpoint-to-Endpoint Envelope transfer |
| Marly Krushkhova | `Chronicler` | receipt-chain attribution engine |
| The Boxmaker | `Curator` | E-core orphan-fragment assembler |
| Orphan pool | `CuratorPool` | DRAM-resident orphan fragment buffer |
| Christopher Mitchell | `Architect` | intent designer; HPowl model author |
| Sniffer / drug dogs | `LexiconCheck` | build-time forbidden-token scanner |

### Matrix concepts already canonical (no renaming)

These terms are not Gibson — they are already the technical names and
should continue to be used unchanged:

- `TruthBlock`
- `Scratchpad`
- `PackedEightField`
- `FieldMask`
- `FieldLane`
- `DeltaRing`
- `ReceiptRing`
- `WorkTier`
- `ResidenceTier`
- `ReceiptMode`
- `UInstr`
- `MotionPacket`
- `HPowl`
- `HotRegion`
- `AttentionScope`
- `OperationalTarget`

---

## 2. Mapping glossary in Rust

```rust
// crates/count-zero/src/lib.rs — PUBLIC SURFACE
//
// NOTE: This crate's public API must not contain Gibson terms.
// Literary framing lives in docs/opus/43 and docs/opus/44.

pub use self::watchdog::{Watchdog, FederatedWatchdog};
pub use self::warden::{LaneWarden, WardenArray, WardenMode, WardenVerdict};
pub use self::envelope::Envelope;
pub use self::broker::Broker;
pub use self::cartridge::Cartridge;
pub use self::relay::Relay;
pub use self::chronicler::{Chronicler, Signature, Attribution};
pub use self::curator::Curator;
pub use self::architect::Architect;
pub use self::lexicon::LexiconCheck;

#[cfg(feature = "resident")]
pub use self::resident::{Resident, ResidentHost};
```

No `Loa`, no `Finn`, no `Marly`, no `Aleph` exported. The implementation
modules can reference the glossary in their doc comments — `//! Also
known in the docs as "Finn (the broker)"` — but nothing in the
signature, name, or export.

---

## 3. Allowed usage zones

```
zone                           Gibson names allowed?
───────────────────────────────────────────────────
docs/opus/*.md                 yes (full literary framing)
//! module doc comments        yes (as "also known as …")
/// item doc comments          yes (same rule)
comments in code (//)          yes
docstring tests                no (test code is source)
type names                     no
function names                 no
module names                   no
crate names                    no
feature flags                  no
constants                      no
error variants                 no
benchmark names                no
test names                     yes in file name for story tests (e.g.
                               `tests/scene_cowboy_runs_ice.rs`) if the
                               test is a narrative test
```

**Rule of thumb:** if a stranger can compile the code without reading
Gibson, the name is fine. If they need to read *Neuromancer* to
understand the signature, the name is a leak.

---

## 4. LexiconCheck: extended forbidden tokens

The `bin/check-lexicon.mjs` script already forbids storage-noun
vocabulary (doc 18). Extend it with:

```js
// bin/check-lexicon.mjs — additions
const FORBIDDEN_LITERARY = new Set([
  // Matrix crate
  "Straylight", "Cowboy", "Ice", "WhiteIce", "BlackIce", "GrayIce",
  "Construct", "DixieFlatline", "Flatline", "Wintermute", "Neuromancer",
  "Turing", "Simstim", "Zaibatsu", "Chiba", "Ninsei", "Jack", "Trode",

  // Count Zero crate
  "CountZero", "CZI", "Bobby", "Newbie", "Loa", "Legba", "Ougou",
  "OugouFeray", "Erzulie", "Samedi", "Danbala", "MetKalfu", "Simbi",
  "Ayizan", "LoaName", "LoaVerdict", "Mood", "Ridden", "Mount",
  "Aleph", "Finn", "Biosoft", "Virek", "Vat", "Turner", "Marly",
  "Boxmaker", "Krushkhova", "Mitchell", "Sniffer",
]);

function scanSourceFile(path, text) {
  // Strip line comments, block comments, and doc strings.
  const stripped = stripComments(text);
  // Strip string literals (so "Ice-cream" in a test fixture is OK).
  const cleaned = stripStringLiterals(stripped);
  for (const token of FORBIDDEN_LITERARY) {
    if (new RegExp(`\\b${token}\\b`).test(cleaned)) {
      throw new Error(`${path}: forbidden literary term '${token}'. ` +
                      `Use canonical name from docs/opus/47.`);
    }
  }
}
```

The scan respects comments — you can still write `// the "Ice" pattern
from doc 43` above a `Gate` definition. It only fails if a forbidden
token appears in code-executing positions.

---

## 5. Example: doc-commented canonical code

```rust
//! count-zero — per-lane wardens over an 8-core pantheon.
//!
//! Literary framing for this crate lives in `docs/opus/44`. This
//! module documentation lists the glossary for readers coming from
//! that document:
//!
//! | Literary     | Canonical here                         |
//! |--------------|----------------------------------------|
//! | Loa          | `LaneWarden`                           |
//! | Legba etc.   | `LaneWarden { lane: FieldLane::X }`    |
//! | Mood         | `WardenMode`                           |
//! | Verdict      | `WardenVerdict`                        |
//! | Aleph        | `Envelope`                             |
//! | Finn         | `Broker<u128>`                         |
//! | Biosoft      | `Cartridge`                            |
//! | Virek        | `Resident` (feature = "resident")      |
//! | Turner       | `Relay`                                |
//! | Marly        | `Chronicler`                           |
//! | Boxmaker     | `Curator`                              |

/// A per-lane autonomous guardian.
///
/// Known in the docs as a "Loa" (doc 44).
pub struct LaneWarden {
    pub lane:    FieldLane,
    pub mask:    FieldMask,
    pub mode:    WardenMode,
    pub strikes: u32,
}

pub enum WardenMode {
    Passive,         // doc: "Quiet"
    Active,          // doc: "Watching"
    Escalating,      // doc: "Restless"
    Commandeering,   // doc: "Ridden"
}

pub enum WardenVerdict {
    Admit,
    Warn,
    Deny,
    Promote,
    Override,        // doc: "Mount"
}
```

The file compiles without any reader needing to know Gibson. The
comments invite the reader into the literary framing if they want it.

---

## 6. What docs 43 and 44 become

They stay exactly as written. Their purpose is narrative memory — a
way for future conversations to re-load the architecture through a
single coherent story. The glossary is how that story is translated
to source.

**Pragmatic reason to keep both:** the literary names cluster ideas
that the technical names scatter. "Finn brokers fragments between
Decks" is a single picture; "Broker\<u128\> moves u128 between
HotRegions" is accurate but forgettable. The literary framing is
working memory; the canonical names are long-term memory.

---

## 7. The rewrite plan

Docs 43 and 44 are unchanged — they are the literary layer.

Future files (and any `.rs` file that currently uses a literary name)
use the canonical names from §1. In particular:

- Doc 45's `Orchestrator` fields: rename `cowboy → runner`,
  `pantheon → wardens`, `czi → watchdog`, `finn → broker`,
  `aleph_depth → envelope_depth`.
- Doc 46's per-core struct: rename `LoaStraylight → LaneHotRegion`,
  the eight `Loa` references → `LaneWarden`, `FederatedCzi →
  FederatedWatchdog`, `Finn → Broker<u128>`.

Source code in `crates/matrix/*` and `crates/count-zero/*` is renamed
accordingly before first compile. The `crates/` directory layout
becomes:

```
crates/
├── unibit-phys/     — pinned memory
├── unibit-hot/      — hot kernels
├── unibit-isa/      — typed instructions
├── mustar/          — renamed: `compiler`
├── runtime/         — renamed: `executor` (was "Neuromancer")
├── runner/          — was "matrix" (the Runner + HotRegion lives here)
├── warden/          — was "count-zero"
├── verifier/        — was "Turing"
├── dteam/           — unchanged
└── unios/           — unchanged
```

Crate names are canonical. Doc filenames stay literary (for memory).

---

## 8. The glossary itself as a build artifact

Ship the glossary table as `target/doc/glossary.html` generated from
this file so `cargo doc --open` includes it in the crate documentation.
Anyone reading API docs gets the translation in the same place as the
types.

---

## 9. The sentence

**Gibson's vocabulary is working memory; the canonical names are the
artifact — Straylight becomes `HotRegion`, Cowboy becomes `Runner`,
Loa becomes `LaneWarden`, Finn becomes `Broker`, Aleph becomes
`Envelope`, Turner becomes `Relay`, Marly becomes `Chronicler`,
Boxmaker becomes `Curator`, and the build script fails if any of the
fictional names crosses the code/comment boundary — because the
story is a reading aid, not an API surface.**
