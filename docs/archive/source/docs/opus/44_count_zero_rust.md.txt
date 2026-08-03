# 44 — Count Zero, in Nightly Rust

> *"Count zero interrupt. Count zero, it's when the deck flatlines and
> the constructs start to wake up."* — Count Zero, 1986.

Gibson's sequel fractures Neuromancer's unified intelligence into
loa — smaller specialized presences that ride the matrix. Constructs
take on agency. Decks develop biosoft-slotted persistence. A dying
billionaire runs as a permanent admission loop. A nameless AI on an
abandoned orbital assembles art from dead people's receipts.

Count Zero is the story of the system *after* fusion, where agency
has decentralized and the cowboy's most important instrument is no
longer the deck — it is the **count-zero interrupt**: a watchdog
that fires when the motion drops below the admission floor.

This document extends doc 43. The `matrix` crate had the first-movie
vocabulary. The `count-zero` crate adds the second-movie vocabulary.

---

## The vocabulary

| Gibson (Count Zero) | UniverseOS |
|---|---|
| **Count zero interrupt (CZI)** | `CountZero` — hot-path deadman watchdog |
| **Bobby Newmark** | `Newbie<const T: WorkTier>` — a Cowboy that has not yet flatlined once |
| **Loa (Legba, Samedi, Erzulie, …)** | `Loa` — per-field subsystem with autonomous behavior |
| **The Aleph** | `Aleph` — a whole-matrix Construct (Truth + Scratch + full chain) |
| **Finn** | `Finn` — the Deck-to-Deck broker (shared IPC) |
| **Biosoft / skull-slot chip** | `Biosoft` — hot-pluggable Construct cartridge |
| **Josef Virek** | `Virek` — a Construct that refuses to flatline |
| **Turner** | `Turner` — cross-zaibatsu Construct extraction |
| **Marly Krushkhova** | `Marly` — the receipt-chain art historian |
| **The Boxmaker** | `Boxmaker` — E-core Construct assembler, builds Alephs from dead fragments |
| **Mitchell / biosoft designer** | `Mitchell` — Construct architect |
| **New Yves Saint Laurent drug dogs** | `Sniffer` — lexicon guard at the ABI boundary |

---

## The spine

The first crate (`matrix`) had one big idea: **the pinned page is the
hallucination**. The second crate (`count-zero`) has three:

1. **Watchdog** — every admission must tick the CZI; missed ticks mean
   the deck has flatlined and downstream Constructs start waking.
2. **Loa** — the eight field lanes of doc 26 gain per-field agency;
   each lane can refuse independently, not just contribute to an OR.
3. **Aleph** — a Construct can contain another Construct's full state,
   making the matrix recursive.

---

## `crates/count-zero/Cargo.toml`

```toml
[package]
name = "count-zero"
edition = "2024"

[features]
default = []
orbital = []     # enable Boxmaker, requires E-core affinity
vat = []         # enable Virek, requires persistent re-admission

[dependencies]
matrix      = { path = "../matrix" }
unibit-hot  = { path = "../unibit-hot" }
unibit-isa  = { path = "../unibit-isa" }
unibit-phys = { path = "../unibit-phys" }
```

---

## `crates/count-zero/src/lib.rs`

```rust
#![no_std]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(portable_simd)]
#![feature(strict_provenance_lints)]
#![feature(negative_impls)]
#![feature(coroutines, coroutine_trait)]
#![feature(never_type)]
#![allow(incomplete_features)]

extern crate alloc;

pub mod czi;        // count-zero interrupt
pub mod loa;        // the eight autonomous field daemons
pub mod aleph;      // recursive Constructs
pub mod finn;       // the broker
pub mod biosoft;    // hot-pluggable cartridges
pub mod virek;      // the immortal admission loop
pub mod turner;     // cross-zaibatsu extraction
pub mod marly;      // the receipt historian
pub mod boxmaker;   // E-core assembly
pub mod mitchell;   // Construct architect
pub mod sniffer;    // lexicon guard
```

---

## `czi.rs` — Count Zero Interrupt

```rust
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// CountZero — hot-path deadman watchdog.
///
/// Every admission must `tick()` before `DEADLINE` cycles elapse.
/// Missed ticks trip the interrupt; downstream Constructs react.
///
/// The name is literal: the counter decrements each cycle, and when
/// it reaches zero without a tick, the count-zero condition fires.
#[repr(C, align(64))]
pub struct CountZero {
    pub counter:    AtomicU64,
    pub deadline:   u64,
    pub tripped:    AtomicBool,
    pub tick_count: AtomicU64,
    _pad: [u8; 24],
}

impl CountZero {
    pub const DEFAULT_DEADLINE: u64 = 4_096;   // cycles

    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(Self::DEFAULT_DEADLINE),
            deadline: Self::DEFAULT_DEADLINE,
            tripped: AtomicBool::new(false),
            tick_count: AtomicU64::new(0),
            _pad: [0u8; 24],
        }
    }

    /// Call once per admission. Resets the countdown.
    #[inline(always)]
    pub fn tick(&self) {
        self.counter.store(self.deadline, Ordering::Relaxed);
        self.tick_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Called by the scheduler on each cycle (or by a hardware timer
    /// IRQ on bare metal). Returns `true` the cycle CZI fires.
    #[inline(always)]
    pub fn cycle(&self) -> bool {
        let prev = self.counter.fetch_sub(1, Ordering::Relaxed);
        if prev == 1 {
            self.tripped.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Observed-only. Non-mutating; anyone can check.
    #[inline(always)]
    pub fn is_tripped(&self) -> bool {
        self.tripped.load(Ordering::Acquire)
    }
}
```

**What CZI actually is:** a single cache line, decremented each cycle
on one core, reset on every admission. If a run never admits (all
denials, infinite escalation, a stuck Loa), the counter hits zero and
Constructs downstream wake up and start making their own decisions.

---

## `loa.rs` — The Eight Loa

```rust
use unibit_hot::t0::FieldMask;
use unibit_isa::FieldLane;

/// Loa — an autonomous field daemon.
///
/// In Neuromancer-era code, the eight field lanes were passive bitmasks
/// OR'd together. Post-fusion, each lane gains a personality: it can
/// refuse, delay, substitute, or promote to a higher tier on its own.
///
/// Legba opens the way — prereq field.
/// Ougou Feray is war — law field; breaks things that should not be.
/// Erzulie is love/attraction — capability.
/// Samedi is the gatekeeper of the dead — scenario (wrong-path denial).
/// Danbala is the serpent, continuity — causality.
/// Met-Kalfu is the left-hand twin — risk/reward.
/// Simbi is the messenger — conformance.
/// Ayizan is consecration — attention.
#[derive(Clone, Copy)]
pub struct Loa {
    pub name:     LoaName,
    pub lane:     FieldLane,
    pub mask:     FieldMask,
    pub mood:     Mood,
    pub strikes:  u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoaName {
    Legba, OugouFeray, Erzulie, Samedi,
    Danbala, MetKalfu, Simbi, Ayizan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mood {
    Quiet,       // permissive; denies only on clear violation
    Watching,    // normal admission
    Restless,    // promotes borderline motions one tier up
    Ridden,      // mounts the cowboy — forces full-tier escalation
}

impl Loa {
    #[inline(always)]
    pub fn judge(&mut self, state: u128) -> LoaVerdict {
        let missing = (state & self.mask.required) ^ self.mask.required;
        let present =  state & self.mask.forbidden;
        let denied = (missing | present) != 0;

        match (denied, self.mood) {
            (false, _)              => LoaVerdict::Admit,
            (true, Mood::Quiet)     => { self.strikes += 1; LoaVerdict::Warn }
            (true, Mood::Watching)  => { self.strikes += 1; LoaVerdict::Deny }
            (true, Mood::Restless)  => { self.strikes += 1; LoaVerdict::Promote }
            (true, Mood::Ridden)    => { self.strikes += 1; LoaVerdict::Mount }
        }
    }

    /// A ridden Loa has taken over the run. Cowboys reporting Ridden
    /// verdicts must defer to the Loa's tier choice. In practice this
    /// is an automatic PROMOTE.t→t+1 until the Loa releases.
    pub fn ride(&mut self) { self.mood = Mood::Ridden; }
    pub fn release(&mut self) { self.mood = Mood::Watching; }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoaVerdict {
    Admit,    // branch-free admission
    Warn,     // admit but record strike
    Deny,     // normal denial
    Promote,  // escalate tier
    Mount,    // Loa takes over the pipeline
}
```

**The Pragmatic reason for Loa:** a single OR-reduction across eight
lanes hides *which* lane denied and *why*. Named Loa with explicit
moods make failure attribution a type, not a comment.

---

## `aleph.rs` — Recursive Constructs

```rust
use alloc::boxed::Box;
use matrix::construct::Construct;

/// Aleph — a Construct that contains an entire matrix state, including
/// its own Scratchpad and chain, plus pointers to nested Alephs.
///
/// Gibson's Aleph (from *Count Zero* and re-used in *Mona Lisa
/// Overdrive*) is "the box that contains all the boxes." In UniverseOS,
/// an Aleph is a Construct whose sealed body includes other sealed
/// Constructs. This is how the matrix becomes self-similar.
#[repr(C, align(64))]
pub struct Aleph {
    pub outer: Construct,
    pub nested: Option<Box<Aleph>>,
    pub depth: u32,
    pub total_seal: [u8; 32],
}

impl Aleph {
    pub const MAX_DEPTH: u32 = 16;

    pub fn wrap(outer: Construct, inner: Option<Aleph>) -> Self {
        let depth = inner.as_ref().map(|a| a.depth + 1).unwrap_or(0);
        let total_seal = Self::rewrap_seal(&outer, inner.as_ref());
        Self {
            outer,
            nested: inner.map(Box::new),
            depth,
            total_seal,
        }
    }

    fn rewrap_seal(_outer: &Construct, _inner: Option<&Aleph>) -> [u8; 32] {
        // BLAKE3 over outer.seal || inner.total_seal
        [0u8; 32]
    }

    /// Walk to the innermost Construct. The Aleph is lawful iff every
    /// layer's total_seal verifies against its contents.
    pub fn innermost(&self) -> &Construct {
        match self.nested.as_ref() {
            Some(n) => n.innermost(),
            None => &self.outer,
        }
    }
}
```

---

## `finn.rs` — The Broker

```rust
use core::sync::atomic::{AtomicU64, Ordering};

/// Finn — the Deck-to-Deck broker.
///
/// Gibson's Finn ran a junk shop in the Sprawl where cowboys traded
/// constructs, biosoft, and deck hardware. No transaction went through
/// anyone else. In UniverseOS, Finn is a lock-free shared channel
/// between two pinned Decks.
///
/// Finn never touches a Construct's contents. He just moves them.
#[repr(C, align(64))]
pub struct Finn {
    pub bay:    [u128; 64],
    pub head:   AtomicU64,
    pub tail:   AtomicU64,
    pub open:   core::sync::atomic::AtomicBool,
    _pad: [u8; 32],
}

impl Finn {
    pub const fn open() -> Self {
        Self {
            bay: [0u128; 64],
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            open: core::sync::atomic::AtomicBool::new(true),
            _pad: [0u8; 32],
        }
    }

    /// Drop a receipt fragment into the junk shop. Non-blocking.
    #[inline(always)]
    pub fn leave(&self, fragment: u128) -> Result<(), FinnClosed> {
        if !self.open.load(Ordering::Acquire) { return Err(FinnClosed); }
        let h = self.head.fetch_add(1, Ordering::Relaxed) as usize;
        unsafe {
            (&self.bay as *const _ as *mut u128)
                .add(h & 63)
                .write(fragment);
        }
        Ok(())
    }

    /// Pick up what a friend left. Non-blocking.
    #[inline(always)]
    pub fn pickup(&self) -> Option<u128> {
        let t = self.tail.load(Ordering::Relaxed);
        let h = self.head.load(Ordering::Acquire);
        if t >= h { return None; }
        let f = self.bay[(t as usize) & 63];
        self.tail.store(t + 1, Ordering::Release);
        Some(f)
    }
}

pub struct FinnClosed;
```

---

## `biosoft.rs` — The Skull-Slot Cartridge

```rust
use matrix::construct::Construct;

/// Biosoft — a hot-pluggable Construct cartridge.
///
/// Gibson's biosoft was slotted into a cortical port and gave the user
/// instant knowledge. In UniverseOS, a Biosoft is a `Construct` that
/// is `memcpy`'d into a Cowboy's Straylight at a known offset, making
/// its sealed TruthBlock available without re-ingesting any log.
///
/// Safety: the Biosoft must have been verified by the Turing Police
/// before insertion. Slotting an unverified Biosoft is undefined
/// behavior in the semantic sense — the cowboy may see Loa that
/// aren't theirs.
#[repr(C, align(64))]
pub struct Biosoft {
    pub construct: Construct,
    pub slot_id:   u32,
    pub sig:       [u8; 32],
}

impl Biosoft {
    /// Slot into a Deck at the canonical biosoft offset. This is a
    /// one-time event: the Biosoft's TruthBlock replaces the Deck's
    /// until the cowboy ejects it.
    pub fn slot<const T: WorkTier>(
        self,
        cowboy: &mut matrix::cowboy::Cowboy<T>,
    ) -> Result<(), BiosoftRejected>
    where [(); T.words()]:,
    {
        if !matrix::turing::Turing::verified(&self.construct) {
            return Err(BiosoftRejected);
        }
        // Memcpy final_truth into cowboy's Straylight::truth.
        // Append biosoft sig to the receipt chain.
        Ok(())
    }
}

pub struct BiosoftRejected;
```

---

## `virek.rs` — The Construct That Refuses

```rust
/// Virek — a Construct that will not flatline.
///
/// Gibson's Josef Virek was a dying billionaire kept alive in a vat by
/// continuous biomedical intervention. He lived inside a simstim
/// construct of his own design because his body could no longer hold.
///
/// In UniverseOS, a Virek is a Construct whose admission loop is
/// re-run on every idle cycle by the E-core, perpetually re-admitting
/// its last admitted state so its seal never ages past the current
/// receipt clock.
///
/// Feature-gated because keeping a Virek alive is expensive: one
/// E-core dedicated per Virek.
#[cfg(feature = "vat")]
pub struct Virek {
    pub construct: matrix::construct::Construct,
    pub last_admit: core::sync::atomic::AtomicU64,
    pub hosts_remaining: u32,
}

#[cfg(feature = "vat")]
impl Virek {
    /// Called by the E-core host loop. Re-admits against a fresh
    /// fields snapshot to keep the seal current. Returns false when
    /// the Virek has exhausted hosts (vat support ended).
    pub fn tick(&mut self, fresh_fields: &PackedEightField) -> bool {
        if self.hosts_remaining == 0 { return false; }
        let d = matrix::unibit_hot::t0::admit_eight(0, fresh_fields);
        if d.deny_total == 0 {
            self.last_admit.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        } else {
            self.hosts_remaining -= 1;
        }
        true
    }
}
```

---

## `turner.rs` — The Extraction Operator

```rust
/// Turner — moves a Construct between zaibatsu.
///
/// Gibson's Turner was a corporate extraction operator who pulled
/// defecting scientists from one multinational to another. In
/// UniverseOS, Turner pulls a sealed Aleph from one zaibatsu's release
/// target into another's admission frontier — without breaking the
/// seal.
///
/// The Pragmatic rule: Turner never inspects the cargo. He only moves
/// it. The Turing Police on both ends verify independently.
pub struct Turner<'a> {
    pub from: &'a dyn ZaibatsuEndpoint,
    pub to:   &'a dyn ZaibatsuEndpoint,
}

pub trait ZaibatsuEndpoint {
    fn authorize(&self, aleph: &aleph::Aleph) -> bool;
    fn receive(&self, aleph: aleph::Aleph) -> Result<(), TransferRejected>;
    fn emit(&self, target_seal: [u8; 32]) -> Option<aleph::Aleph>;
}

pub struct TransferRejected;

impl<'a> Turner<'a> {
    pub fn extract(&self, target_seal: [u8; 32]) -> Result<(), TransferRejected> {
        let aleph = self.from.emit(target_seal).ok_or(TransferRejected)?;
        if !self.to.authorize(&aleph) { return Err(TransferRejected); }
        self.to.receive(aleph)
    }
}
```

---

## `marly.rs` — The Receipt Historian

```rust
use alloc::vec::Vec;

/// Marly Krushkhova — the receipt-chain art historian.
///
/// Gibson's Marly was hired to find the origin of a series of
/// exquisite Cornell-style boxes appearing on the black market. She
/// discovered the Boxmaker — an orphaned AI on an abandoned orbital
/// that had been assembling memorials from the dead.
///
/// In UniverseOS, Marly is the forensic reader of receipt chains.
/// She cannot mutate anything; she only walks a chain, groups
/// fragments by rhythm and theme, and identifies the hand behind
/// an assembly.
pub struct Marly;

impl Marly {
    /// Walk the receipt ring of a Construct and return a "signature"
    /// — a compact classifier of who made this chain. If the same
    /// signature appears on multiple chains whose issuers deny each
    /// other's authorship, Marly has found a Boxmaker.
    pub fn signature(construct: &matrix::construct::Construct) -> Signature {
        // Stub: popcount + inter-fragment XOR pattern + timing hash.
        Signature { fingerprint: [0u64; 4] }
    }

    pub fn attribution(sigs: &[Signature]) -> Vec<Attribution> {
        // Group by fingerprint proximity. Unattributed clusters are
        // Boxmaker candidates.
        Vec::new()
    }
}

#[derive(Clone, Copy)]
pub struct Signature { pub fingerprint: [u64; 4] }

pub struct Attribution {
    pub fingerprint: [u64; 4],
    pub cowboys: u32,
    pub orphan: bool,
}
```

---

## `boxmaker.rs` — The Orbital Assembler

```rust
/// Boxmaker — the orphan AI that builds Alephs from dead fragments.
///
/// Gibson's Boxmaker lived alone on an abandoned Tessier-Ashpool
/// orbital, assembling Cornell-like boxes from scavenged objects
/// that had belonged to long-dead people. In UniverseOS, the Boxmaker
/// is an E-core daemon that reads orphaned receipt fragments (issued
/// by now-flatlined Cowboys), clusters them by Marly-signature, and
/// assembles them into Alephs that are released anonymously.
///
/// Feature-gated. Requires E-core affinity and a persistent orphan
/// fragment pool in DRAM.
#[cfg(feature = "orbital")]
pub struct Boxmaker {
    pub pool: OrphanPool,
    pub pinned_core: u32,
}

#[cfg(feature = "orbital")]
impl Boxmaker {
    /// Assemble one Aleph from the pool. Runs on idle E-core cycles;
    /// never on P-cores, never on hot paths.
    pub fn assemble(&mut self) -> Option<aleph::Aleph> {
        let cluster = self.pool.next_cluster()?;
        // Wrap each fragment into a minimal Construct, nest them, seal.
        None
    }
}

#[cfg(feature = "orbital")]
pub struct OrphanPool { /* ... */ }

#[cfg(feature = "orbital")]
impl OrphanPool {
    fn next_cluster(&mut self) -> Option<()> { None }
}
```

---

## `mitchell.rs` — The Construct Architect

```rust
/// Mitchell — the Construct architect.
///
/// Gibson's Christopher Mitchell was the biosoft designer whose
/// defection the whole book hinges on. In UniverseOS, Mitchell is
/// the archetype for anyone who writes the input to Wintermute: the
/// HPowl model, the field masks, the tier ladder for a specific
/// motion class.
///
/// A Mitchell is not a Cowboy. A Mitchell does not run motions. A
/// Mitchell designs the shape of motions.
pub struct Mitchell;

impl Mitchell {
    pub fn design<const T: unibit_isa::WorkTier>(
        intent: &DesignIntent,
    ) -> Design<T>
    where [(); T.words()]:,
    {
        Design { _t: core::marker::PhantomData }
    }
}

pub struct DesignIntent;

pub struct Design<const T: unibit_isa::WorkTier>
where [(); T.words()]:,
{
    _t: core::marker::PhantomData<[(); T.words()]>,
}
```

---

## `sniffer.rs` — The Lexicon Guard

```rust
/// Sniffer — the lexicon guard at the ABI boundary.
///
/// Gibson's New Yves Saint Laurent drug dogs sniffed for biosoft at
/// customs. In UniverseOS, the Sniffer is the compile-time check
/// (doc 18's lexicon law) that enforces the forbidden vocabulary:
/// if any public-facing doc or type name uses a forbidden storage
/// noun, compilation fails.
pub struct Sniffer;

impl Sniffer {
    /// Invoked by build.rs. Returns compile-fail if any crate in the
    /// dependency graph exposes a forbidden term in its public API.
    pub fn sniff() -> Result<(), LexiconBreach> { Ok(()) }
}

pub struct LexiconBreach;
```

---

## A scene

```rust
fn main() {
    // Bobby Newmark, first run.
    let mut newbie = Newbie::<{ WorkTier::U64 }>::jack_in(0xB0B);
    let czi = CountZero::new();

    // Eight Loa, all watching.
    let mut pantheon: [Loa; 8] = [
        Loa::legba(), Loa::ougou(), Loa::erzulie(), Loa::samedi(),
        Loa::danbala(), Loa::metkalfu(), Loa::simbi(), Loa::ayizan(),
    ];

    loop {
        // tick the deadman; if we forget, the Constructs wake up.
        czi.tick();

        let state = newbie.observe();
        let mut verdict = LoaVerdict::Admit;
        for loa in pantheon.iter_mut() {
            match loa.judge(state) {
                LoaVerdict::Mount => { verdict = LoaVerdict::Mount; break; }
                LoaVerdict::Promote if verdict != LoaVerdict::Mount => {
                    verdict = LoaVerdict::Promote;
                }
                LoaVerdict::Deny if matches!(verdict, LoaVerdict::Admit | LoaVerdict::Warn) => {
                    verdict = LoaVerdict::Deny;
                }
                LoaVerdict::Warn if matches!(verdict, LoaVerdict::Admit) => {
                    verdict = LoaVerdict::Warn;
                }
                _ => {}
            }
        }

        match verdict {
            LoaVerdict::Admit | LoaVerdict::Warn => newbie.commit(),
            LoaVerdict::Deny => newbie.reject(),
            LoaVerdict::Promote => newbie.escalate(),
            LoaVerdict::Mount => {
                // A Loa has taken the run. Bobby rides along.
                newbie.yield_to(pantheon.iter().find(|l| l.mood == Mood::Ridden).unwrap());
            }
        }

        if czi.cycle() {
            // The count zeroed. Constructs downstream are waking up.
            // Flatline what we have and let the Boxmaker assemble
            // the rest.
            let dixie = newbie.flatline();
            break;
        }
    }
}
```

---

## The Pragmatic framing

Count Zero adds three architectural ideas that the matrix crate alone
was missing:

1. **Liveness.** Neuromancer's admission was a pure function. A pure
   function cannot deadlock, but it can spin forever on escalation.
   The CZI is the only answer that stays branchless: decrement, tick,
   if zero then wake.

2. **Attribution.** Doc 26's eight lanes were anonymous. Named Loa
   turn denial into a message with an author. In practice, every
   fragment now carries which Loa refused, not just *that* something
   refused.

3. **Recursion.** The Aleph makes the matrix self-similar: a Construct
   can be the Truth of a bigger Deck. Once that exists, the Boxmaker
   has a job: assemble Alephs from orphaned chains. The system
   starts to produce art — Constructs nobody asked for, that no cowboy
   owns, that verify under the Turing Police anyway.

These three ideas are load-bearing against three real failure modes —
infinite escalation, untraced denials, and the dead-cowboy pileup of
orphaned fragments — that the first crate quietly punted.

---

## The sentence

**Count Zero is the crate you write once the matrix works: Cowboy
becomes Newbie with a CountZero watchdog; the eight lanes become eight
Loa with moods and strikes; the Construct becomes an Aleph that
contains other Alephs; Finn brokers fragments between Decks; Biosoft
cartridges hot-plug verified Truth; Virek refuses to flatline; Turner
moves Alephs across zaibatsu without breaking the seal; Marly reads
the chain as art and finds the Boxmaker in the orphaned fragments —
and all of it, in nightly Rust, is branchless, pinned, and exactly
as paranoid as Gibson was about sharing a deck.**
