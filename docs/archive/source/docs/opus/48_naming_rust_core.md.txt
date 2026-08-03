# 48 — Naming Revisited: What the Rust Core Team Would Actually Sign Off On

## Self-critique

Doc 47 replaced Gibson names with "canonical" names. Re-read with Rust
API Guidelines in hand, half of those are still wrong:

- `Runner`, `Cowboy` → **`Runner`** is almost as anthropomorphic as
  `Cowboy`. Rust idiom: types are *things*, not *actors*.
- `Capsule`, `Envelope` → metaphor in a suit.
- `Broker<T>` → product-marketing, not std-style.
- `LaneWarden`, `Chronicler`, `Curator`, `Architect` → anthropomorphism
  with extra steps.
- `Executor` → collides with the entire async ecosystem
  (`tokio::runtime::Executor`, `futures::executor::Executor`).
- `Resident`, `Relay` → dress-up words.

The Rust core team's actual preference, derived from `std`, `tokio`,
`hyper`, `serde`, and the API Guidelines:

1. **Describe the data, not the role.** `HashMap`, not `Dictionary`.
   `Mutex`, not `Guard`. `Vec`, not `Container`.
2. **Pick the least surprising word.** If a competent Rust engineer
   who has never seen the crate can guess what the type does from the
   name alone, it's right.
3. **If it has no state or behavior, don't make it a type.** Roles
   that don't carry state become free functions, not structs.
4. **Respect existing namespaces.** `Executor`, `Runtime`, `Task`,
   `Worker`, `Handle` are all already spoken for — avoid.
5. **Acronyms are fine** when standard. `Csv`, `Url`, `Spsc`, `Mpsc`,
   `Ewma`, `Crc`. Not `LaneGuard` when `Gate` works.
6. **`Error` suffix for errors.** `LexiconError`, not `LexiconBreach`.

---

## The revised glossary

### Types that survive

| Old (doc 47)   | Rust-core-idiomatic       | Why |
|----------------|---------------------------|-----|
| `HotRegion`    | `HotRegion` ✓             | describes layout, not role; survives |
| `Gate`         | `Gate` ✓                  | standard pattern noun; survives |
| `GatePolicy`   | `GatePolicy` ✓            | fine |
| `FieldMask`    | `FieldMask` ✓             | already canonical; survives |
| `FieldLane`    | `FieldLane` ✓             | already canonical; survives |
| `DeltaRing`    | `DeltaRing` ✓             | describes structure; survives |
| `ReceiptRing`  | `ReceiptRing` ✓           | survives |
| `Watchdog`     | `Watchdog` ✓              | std-linux-kernel precedent; survives |

### Types that need renaming

| Old (doc 47)   | Revised                    | Rationale |
|----------------|----------------------------|-----------|
| `Runner<T>`    | **removed** — it was a role, not a type | execution becomes a free fn: `pub fn step(region: &mut HotRegion, packet: &MotionPacket) -> StepOutcome` |
| `Capsule`      | **`Snapshot`**             | "snapshot of sealed state" — zero metaphor, direct |
| `Envelope`     | **removed** — make `Snapshot` recursive | `Snapshot { inner: Option<Box<Snapshot>> }` |
| `Broker<T>`    | **`SpscRing<T>`**          | it's a single-producer single-consumer ring; call it that |
| `LaneWarden`   | **`LanePolicy`**           | it holds per-lane mask + mode; policy is the standard word |
| `WardenMode`   | **`LaneMode`**             | consistency; drops anthropomorphism |
| `WardenVerdict`| **`LaneOutcome`**          | `Outcome` is already used in stdlib-adjacent code |
| `Cartridge`    | **`SignedSnapshot`**       | what it literally is: a `Snapshot` + signature |
| `Resident`     | **removed** — becomes a task | start as `std::thread::spawn(move || loop { … })`; no type needed |
| `Relay`        | **removed** — becomes a free fn | `pub fn transfer(src: &Endpoint, dst: &Endpoint, seal: &[u8; 32])` |
| `Chronicler`   | **`ChainAnalyzer`**        | describes behavior; no metaphor |
| `Curator`      | **`OrphanAssembler`**      | describes behavior |
| `Architect`    | **removed** — human role | the human writes HPowl; not a code type |
| `FederatedWatchdog` | **`WatchdogShards`**  | standard sharding terminology |
| `LexiconBreach`| **`LexiconError`**         | `Error` suffix per C-GOOD-ERR |
| `BiosoftRejected` | **`SignatureError`**    | `Error` suffix; name the failure, not the plot |
| `TransferRejected` | **`TransferError`**    | same |

### Roles that were types but shouldn't be

| Doc 47 type     | Becomes in source                   |
|-----------------|-------------------------------------|
| `Compiler`      | module `compile` with `pub fn compile(...)` — not a struct |
| `Executor`      | **removed** — free fn `step` or `run` on the region |
| `Verifier`      | module `verify` with `pub fn verify(s: &Snapshot) -> Result<(), VerifyError>` |
| `Simulator`     | module `sim`; `pub fn rehearse(...)` |
| `Endpoint`      | trait `Endpoint` (not a struct); users implement |

---

## The revised surface

```rust
// crates/runner/src/lib.rs
//
// Literary framing in docs/opus/43–46 is glossaried in docs/opus/48.

pub struct HotRegion { /* ... */ }

pub struct Gate {
    pub mask:   FieldMask,
    pub policy: GatePolicy,
}

pub enum GatePolicy { Deny, Escalate, Counter }

pub struct Snapshot {
    pub seal:        [u8; 32],
    pub final_state: TruthBlock,
    pub chain_tail:  [u128; 16],
    pub chain_len:   u64,
    pub inner:       Option<Box<Snapshot>>,   // formerly "Envelope" / "Aleph"
}

pub struct SignedSnapshot {
    pub snapshot: Snapshot,
    pub signer:   [u8; 32],
    pub sig:      [u8; 64],
}

pub enum StepOutcome {
    Admitted(u128),
    Denied(u128, GatePolicy),
    Mismatch,
    WatchdogTripped,
    Override(FieldLane),
}

pub fn step(
    region: &mut HotRegion,
    packet: &MotionPacket,
) -> StepOutcome { /* ... */ }

pub fn finalize(region: HotRegion) -> Snapshot { /* ... */ }
```

```rust
// crates/warden/src/lib.rs  →  renamed to crates/lane-policy/

pub struct LanePolicy {
    pub lane:    FieldLane,
    pub mask:    FieldMask,
    pub mode:    LaneMode,
    pub strikes: u32,
}

pub enum LaneMode {
    Passive,
    Active,
    Escalating,
    Commandeering,
}

pub enum LaneOutcome {
    Admit,
    Warn,
    Deny,
    Promote,
    Override,
}

impl LanePolicy {
    pub fn judge(&mut self, state: u128) -> LaneOutcome { /* ... */ }
}

pub struct WatchdogShards {
    pub shards: [Watchdog; 8],
}
```

```rust
// crates/ring/src/lib.rs  (formerly "broker")

pub struct SpscRing<T, const N: usize> { /* ... */ }

impl<T, const N: usize> SpscRing<T, N> {
    pub const fn new() -> Self { /* ... */ }
    pub fn push(&self, item: T) -> Result<(), T> { /* ... */ }
    pub fn pop(&self) -> Option<T> { /* ... */ }
}
```

```rust
// crates/chain-analyzer/src/lib.rs  (formerly "marly" / "chronicler")

pub fn signature(s: &Snapshot) -> Signature { /* ... */ }
pub fn attribute(sigs: &[Signature]) -> Vec<Attribution> { /* ... */ }
```

```rust
// crates/orphan-assembler/src/lib.rs  (formerly "boxmaker" / "curator")

pub fn assemble(pool: &OrphanPool) -> Option<Snapshot> { /* ... */ }
```

```rust
// crates/endpoint/src/lib.rs

pub trait Endpoint {
    fn authorize(&self, snap: &Snapshot) -> bool;
    fn receive(&self, snap: Snapshot) -> Result<(), TransferError>;
    fn emit(&self, seal: [u8; 32]) -> Option<Snapshot>;
}

pub fn transfer<S: Endpoint, D: Endpoint>(
    src: &S,
    dst: &D,
    seal: [u8; 32],
) -> Result<(), TransferError> { /* ... */ }
```

---

## Crate layout, Rust-core-idiomatic

```
crates/
├── unibit-phys/       — pinned memory, alignment
├── unibit-hot/        — hot kernels (admit/commit/reduce)
├── unibit-isa/        — typed UInstr, WorkTier, FieldLane
├── compile/           — intent → MotionPacket   (was "mustar")
├── runner/            — HotRegion + step + finalize   (was "matrix")
├── lane-policy/       — LanePolicy, LaneMode, LaneOutcome   (was "count-zero")
├── watchdog/          — Watchdog, WatchdogShards
├── ring/              — SpscRing<T, const N>
├── verify/            — verify(Snapshot) → Result
├── chain-analyzer/    — attribution helpers   (was "marly")
├── orphan-assembler/  — background snapshot assembly   (was "boxmaker")
├── endpoint/          — Endpoint trait + transfer fn
├── dteam/             — unchanged
└── unios/             — unchanged
```

Crate names are `kebab-case` (Cargo convention). Types inside are
`UpperCamelCase`. No anthropomorphism. No metaphors that a newcomer
has to decode.

---

## The test: read a signature cold

A Rust engineer reading these for the first time, with no context,
should immediately understand:

```rust
pub fn step(region: &mut HotRegion, packet: &MotionPacket) -> StepOutcome;
pub fn finalize(region: HotRegion) -> Snapshot;
pub fn verify(snap: &Snapshot) -> Result<(), VerifyError>;
pub fn transfer<S: Endpoint, D: Endpoint>(src: &S, dst: &D, seal: [u8; 32])
    -> Result<(), TransferError>;

pub struct LanePolicy { lane: FieldLane, mask: FieldMask, mode: LaneMode, strikes: u32 }
pub enum LaneOutcome { Admit, Warn, Deny, Promote, Override }
pub struct SpscRing<T, const N: usize>;
```

Every type and function signature reads as "data, verb, data." No
reader needs to know Gibson. No reader needs to learn internal jargon.
That is the standard the core team actually enforces.

---

## What gets left out

Doc 47 had types for:
- `Runner` (an actor)
- `Compiler` / `Executor` / `Verifier` / `Simulator` (all verbs)
- `Architect` (a human)
- `Relay` (a verb)
- `Curator` / `Chronicler` (verbs + metaphor)
- `Resident` (a background state)

**Seven of these don't need types at all.** They become free functions
(`compile`, `verify`, `transfer`, `assemble`, `analyze`) or modules
(`compile::`, `verify::`, `sim::`). Deleting them shrinks the API
surface, removes vocabulary the reader has to learn, and matches what
std does — `serde::to_string` is a function, not a `Serializer` actor
struct.

---

## The rule, sharper than doc 47

```
if it has state and behavior           → struct (UpperCamelCase)
if it is a discriminator                → enum (UpperCamelCase)
if it is a verb over data               → fn (snake_case, module-level)
if it is a role a human plays           → not a type at all
if it has a human name in the story     → never a type
```

---

## Updated LexiconCheck

The forbidden list from doc 47 stands. Additions:

```js
// Anthropomorphic roles that sneaked back in.
const FORBIDDEN_ROLES = new Set([
  "Runner",      // fn step() instead
  "Executor",    // namespace collision + role word
  "Compiler",    // fn compile() instead
  "Verifier",    // fn verify() instead
  "Simulator",   // fn rehearse() in mod sim
  "Warden",      // anthropomorphic; LanePolicy is correct
  "Chronicler",  // ChainAnalyzer fns
  "Curator",     // OrphanAssembler fns
  "Architect",   // not a code entity
  "Broker",      // SpscRing
  "Envelope",    // make Snapshot recursive
  "Capsule",     // Snapshot
  "Cartridge",   // SignedSnapshot
  "Relay",       // fn transfer()
  "Resident",    // background task
]);
```

Either the word is a standard Rust idiom for that shape (`HashMap`,
`Mutex`, `Vec`, `Ring`, `Shard`) or it is not allowed.

---

## Docs 43 and 44 — unchanged rationale

Literary framing still lives in the Gibson docs. The story is the
reading aid. The glossary of doc 47, tightened in doc 48, is how that
story translates. The narrative does what prose does well — cluster
meanings. The canonical names do what code does well — carry no
hidden metaphor.

---

## The sentence

**The Rust core team would sign off on `HotRegion`, `Gate`, `Snapshot`,
`LanePolicy`, `Watchdog`, `SpscRing`, and a handful of free functions
like `step`, `finalize`, `verify`, `transfer`, `compile`, `assemble`,
`analyze` — and would delete every `Runner`, `Executor`, `Broker`,
`Cartridge`, `Envelope`, `Relay`, `Chronicler`, `Curator`, `Architect`,
`Resident`, and `Warden` as anthropomorphism or metaphor; because in
Rust the type names describe the data, the function names describe the
verb, and the story stays in the docs where it belongs.**
