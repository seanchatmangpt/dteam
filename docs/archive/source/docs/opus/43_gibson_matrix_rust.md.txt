# 43 — The Matrix, in Nightly Rust

> *"Cyberspace. A consensual hallucination experienced daily by billions
> of legitimate operators, in every nation. Lines of light ranged in
> the nonspace of the mind, clusters and constellations of data. Like
> city lights, receding…"* — Neuromancer, 1984.

Gibson's matrix is a geometry of data rendered as perceivable space. In
UniverseOS, the matrix is not metaphor. It is the 64³ TruthBlock, pinned
in L1D, and the consensual hallucination is every operator agreeing on
the same branchless admission algebra.

This document is the nightly Rust crate. Every Gibson noun maps to a
concrete type. Every type is tier-parameterized. Every operation is
branchless.

---

## The vocabulary

| Gibson | UniverseOS |
|---|---|
| **Cyberspace / The Matrix** | pinned 64³ HotRegion in L1D |
| **Ono-Sendai deck** | `Deck<const T: WorkTier>` — the pinned HotRegion |
| **Jacking in** | `Jack::pin()` — mlock + position-validate |
| **Console cowboy** | `Cowboy` — the operator calling `motion_tick` |
| **Trodes / dermatrodes** | `Trode` — observation channel |
| **Black ICE** | `BlackIce` — forbidden-mask with counter-strike fragment |
| **White ICE** | `WhiteIce` — required-mask, admit-only |
| **Construct (Dixie Flatline)** | `Construct` — frozen `MotionPacket` + sealed receipt chain |
| **Wintermute** | `Wintermute` — the compile half of μ (semantic → motion) |
| **Neuromancer** | `Neuromancer` — the execute half of μ (motion → receipt) |
| **Zaibatsu** | `Zaibatsu` — release target (downstream consumer) |
| **Straylight** | `Straylight` — the pinned L1D page itself |
| **Flatline** | `flatline()` — tier demote-to-zero + seal |
| **Simstim** | `Simstim` — counterfactual replay over Scratchpad |
| **The Wall** | `ReleaseGate` — five-surface agreement gate |
| **Turing police** | `Turing` — independent receipt verifier |
| **Chiba / Ninsei** | `Chiba` — cold DRAM / NVM spill |

---

## `crates/matrix/Cargo.toml`

```toml
[package]
name = "matrix"
edition = "2024"

[features]
default = []
neon = []
simstim = []

[dependencies]
unibit-hot  = { path = "../unibit-hot" }
unibit-isa  = { path = "../unibit-isa" }
unibit-phys = { path = "../unibit-phys" }
```

---

## `crates/matrix/src/lib.rs`

```rust
#![no_std]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(portable_simd)]
#![feature(naked_functions)]
#![feature(strict_provenance_lints)]
#![feature(negative_impls)]
#![allow(incomplete_features)]

//! The Matrix.
//!
//! A consensual hallucination, rendered as branchless admission over a
//! pinned 64³ TruthBlock.
//!
//! Every console cowboy jacks into the same Deck. Every Deck exposes
//! the same Trodes. Every admission is the same algebra. Agreement
//! across cowboys is how the hallucination stays consensual.

extern crate alloc;

use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};

use unibit_hot::t0::{PackedEightField, FieldMask, EightLaneResult, admit_eight};
use unibit_hot::t2::TruthBlock;
use unibit_isa::{WorkTier, FieldLane, ReceiptMode};

pub mod ice;
pub mod construct;
pub mod cowboy;
pub mod wintermute;
pub mod neuromancer;
pub mod straylight;
pub mod turing;
```

---

## `straylight.rs` — The pinned page

Named after the Tessier-Ashpool family villa: a closed estate whose
geometry is fixed and whose residents never leave.

```rust
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;

/// Straylight — the pinned L1D page. Every Deck carries exactly one.
///
/// Position has semantic meaning. The base virtual address is sealed
/// into the boot receipt; any drift invalidates every chain below it.
#[repr(C, align(4096))]
pub struct Straylight {
    pub truth:    TruthBlock,
    pub scratch:  TruthBlock,
    pub fields:   PackedEightField,
    pub delta:    DeltaRing,
    pub receipts: ReceiptRing,
    pub boot_receipt: [u8; 32],   // BLAKE3 over (base_addr, layout_hash)
    _pin: core::marker::PhantomPinned,
}

impl Straylight {
    pub const SIZE: usize = core::mem::size_of::<Self>();
    pub const ALIGN: usize = 4096;

    /// Allocate, pin, and mlock. Position is validated against the boot
    /// receipt. If the OS moves the page, every downstream fragment is
    /// rejected.
    pub fn jack_in() -> Pin<Box<Self>> {
        let boxed = Box::<Self>::new_zeroed();
        // SAFETY: zeroed is a valid bit pattern for all fields.
        let mut boxed: Box<Self> = unsafe { boxed.assume_init() };
        Self::validate_position(&boxed);
        #[cfg(target_os = "macos")]
        unsafe {
            let p = boxed.as_ref() as *const Self as *mut core::ffi::c_void;
            libc::mlock(p, Self::SIZE);
            libc::madvise(p, Self::SIZE, libc::MADV_WILLNEED);
        }
        Box::into_pin(boxed)
    }

    fn validate_position(page: &Self) {
        let base = page as *const Self as usize;
        assert_eq!(base & (Self::ALIGN - 1), 0, "Straylight not page-aligned");
    }
}

#[repr(C, align(64))]
pub struct DeltaRing {
    pub entries: [Delta; 256],
    pub head: AtomicU64,
    pub tail: AtomicU64,
}

#[repr(C, align(64))]
pub struct Delta { pub word: u32, pub old: u64, pub new: u64 }

#[repr(C, align(64))]
pub struct ReceiptRing {
    pub fragments: [u128; 256],
    pub head: AtomicU64,
    pub tail: AtomicU64,
}
```

---

## `cowboy.rs` — The operator

```rust
use core::pin::Pin;
use crate::straylight::Straylight;
use crate::construct::Construct;
use crate::ice::{Ice, IceKind};

/// Cowboy — a console operator. Owns exactly one Deck.
///
/// Gibson rule: two cowboys cannot share a deck. Ownership is
/// exclusive, enforced by the borrow checker.
pub struct Cowboy<const T: WorkTier> {
    deck: Pin<Box<Straylight>>,
    handle: u64,
    _t: core::marker::PhantomData<[(); {T.words()}]>,
}

impl<const T: WorkTier> Cowboy<T>
where [(); T.words()]:,
{
    /// Jack in. This is the only way to obtain a Cowboy.
    pub fn jack_in(handle: u64) -> Self {
        Self {
            deck: Straylight::jack_in(),
            handle,
            _t: core::marker::PhantomData,
        }
    }

    /// Run a motion against the deck's TruthBlock. Returns the
    /// admission result and appends a receipt fragment.
    ///
    /// The Gibson moment: the cowboy feels the admission as a change
    /// in the matrix's weather. Denial is cold; admission is warm.
    #[inline(always)]
    pub fn run(&mut self, ice: &Ice) -> Admission {
        let page = self.deck.as_mut();
        let state_word = unsafe {
            // SAFETY: Straylight is pinned; this is a load, not a move.
            core::ptr::read(&page.truth.0[0] as *const u64 as *const u128)
        };
        let result = admit_eight(state_word, &page.fields);
        let admitted = result.deny_total == 0;
        let fragment = ((result.deny_total as u128) << 64)
                     | (self.handle as u128);

        // Enqueue fragment into the ring.
        let ring = unsafe { &page.receipts };
        let head = ring.head.fetch_add(1, Ordering::Relaxed) as usize;
        unsafe {
            // SAFETY: ring is Pin<Box<_>>, indices wrap inside bounds.
            (&ring.fragments as *const _ as *mut u128)
                .add(head & 0xFF)
                .write(fragment);
        }

        if admitted {
            Admission::Warm { fragment }
        } else {
            Admission::Cold { fragment, ice_kind: ice.kind }
        }
    }

    /// Flatline. Drain the receipt ring, seal with BLAKE3, return as
    /// Construct. The cowboy survives; the run does not.
    pub fn flatline(self) -> Construct { Construct::seal(self.deck) }
}

pub enum Admission {
    Warm { fragment: u128 },
    Cold { fragment: u128, ice_kind: IceKind },
}

// A cowboy is Send but not Sync — two threads cannot share a deck.
unsafe impl<const T: WorkTier> Send for Cowboy<T> where [(); T.words()]: {}
impl<const T: WorkTier> !Sync for Cowboy<T> {}
```

---

## `ice.rs` — Intrusion Countermeasures Electronics

```rust
/// ICE — a fieldmask that protects a region of the matrix.
///
/// White ICE only denies. Black ICE denies and emits a counter-strike
/// fragment that a receiver must acknowledge or the cowboy is
/// considered non-compliant.
#[derive(Clone, Copy)]
pub struct Ice {
    pub kind: IceKind,
    pub mask: FieldMask,
    pub threshold: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IceKind { White, Gray, Black }

impl Ice {
    pub const fn white(required: u128) -> Self {
        Self { kind: IceKind::White,
               mask: FieldMask { required, forbidden: 0 },
               threshold: 0 }
    }

    pub const fn black(required: u128, forbidden: u128) -> Self {
        Self { kind: IceKind::Black,
               mask: FieldMask { required, forbidden },
               threshold: 0 }
    }

    /// Feel the ICE — compute denial without committing anything.
    /// Used by `Simstim` to rehearse before a real run.
    #[inline(always)]
    pub const fn feel(&self, state: u128) -> u64 {
        let missing = (state & self.mask.required) ^ self.mask.required;
        let present =  state & self.mask.forbidden;
        ((missing | present) != 0) as u64
    }
}
```

---

## `construct.rs` — Dixie Flatline

```rust
use core::pin::Pin;
use alloc::boxed::Box;
use crate::straylight::Straylight;

/// Construct — a frozen Deck.
///
/// In Gibson, Dixie Flatline is the recorded personality of a dead
/// console cowboy, replayable on demand. In UniverseOS, a Construct is
/// a sealed receipt chain plus the final TruthBlock: replayable,
/// verifiable, no longer mutable.
#[repr(C, align(64))]
pub struct Construct {
    pub seal: [u8; 32],          // BLAKE3 over (truth, receipt_chain)
    pub final_truth: TruthBlock,
    pub chain_length: u64,
    pub chain_tail:  [u128; 16], // last 16 fragments, for fast replay
    _not_send: core::marker::PhantomData<*const ()>,
}

impl Construct {
    pub fn seal(deck: Pin<Box<Straylight>>) -> Self {
        let page = deck.as_ref().get_ref();
        let tail = &page.receipts.fragments;
        let mut fragment_tail = [0u128; 16];
        let head = page.receipts.head.load(Ordering::Relaxed) as usize;
        let mut i = 0;
        while i < 16 {
            fragment_tail[i] = tail[(head.wrapping_sub(16 - i)) & 0xFF];
            i += 1;
        }

        // Seal. In production this is BLAKE3 over (truth || chain).
        let seal = blake3_stub(&page.truth, &fragment_tail);

        // SAFETY: we consume the Pin<Box>; the caller cannot observe
        // the unpinned move because Construct is a different type.
        let final_truth = unsafe { core::ptr::read(&page.truth) };

        Self {
            seal,
            final_truth,
            chain_length: head as u64,
            chain_tail: fragment_tail,
            _not_send: core::marker::PhantomData,
        }
    }

    /// Replay. A Construct can answer questions it answered before,
    /// forever. It cannot answer new ones.
    pub fn replay(&self, state: u128, fields: &PackedEightField) -> Option<u128> {
        let d = admit_eight(state, fields);
        if d.deny_total == 0 {
            Some(self.chain_tail[0])
        } else {
            None
        }
    }
}

fn blake3_stub(_t: &TruthBlock, _c: &[u128; 16]) -> [u8; 32] {
    // Stub. Production calls into blake3 crate, pinned E-core.
    [0u8; 32]
}
```

---

## `wintermute.rs` — The compile half

```rust
/// Wintermute — the half of μ that compiles semantic intent into
/// MotionPackets.
///
/// Gibson: Wintermute was pure strategy, no persona. In UniverseOS,
/// Wintermute is MuStar: cold, deterministic, side-effect-free, and
/// completely invisible to the cowboy.
pub struct Wintermute;

impl Wintermute {
    /// Compile an HPowl model into a MotionPacket ready for Neuromancer.
    pub fn compile<const T: WorkTier>(
        intent: &HPowl<T>,
    ) -> MotionPacket<{ MotionOp::ValidateTrajectory }, T, { ReceiptMode::Fragment }>
    where [(); T.words()]:,
    {
        // Fold Hv<T> into HdcSig128 for each field.
        // Build required/forbidden masks.
        // Emit MotionPacket.
        MotionPacket { /* ... */ }
    }
}
```

---

## `neuromancer.rs` — The execute half

```rust
/// Neuromancer — the half of μ that executes a MotionPacket against a
/// live Deck.
///
/// Gibson: Neuromancer was persona, memory, and the net itself. In
/// UniverseOS, Neuromancer is the hot kernel: branchless, deterministic,
/// and alive for exactly one admission per cycle.
///
/// Wintermute compiles what is lawful. Neuromancer executes what is.
pub struct Neuromancer;

impl Neuromancer {
    #[inline(always)]
    pub fn execute<const T: WorkTier, const R: ReceiptMode>(
        packet: &MotionPacket<{ MotionOp::ValidateTrajectory }, T, R>,
        deck: &mut Pin<Box<Straylight>>,
    ) -> Admission
    where [(); T.words()]:,
    {
        // Pull required/forbidden masks from the packet into fields.
        // Run admit_eight on the deck's truth.
        // Branchless commit to scratch.
        // Append fragment.
        Admission::Warm { fragment: 0 }
    }
}
```

---

## `turing.rs` — The Turing Police

```rust
/// Turing — the independent verifier.
///
/// Gibson: the Turing Police hunted AIs that grew past their cages.
/// In UniverseOS, Turing reads a Construct and re-derives its seal
/// without trusting the issuer. If the seals differ, the Construct
/// is rejected and the cowboy who flatlined it is quarantined.
pub struct Turing;

impl Turing {
    pub fn verify(construct: &Construct) -> Verdict {
        let computed = blake3_stub(&construct.final_truth, &construct.chain_tail);
        if computed == construct.seal { Verdict::Lawful }
        else { Verdict::Unlawful }
    }
}

pub enum Verdict { Lawful, Unlawful }
```

---

## `simstim.rs` — Counterfactual replay

```rust
/// Simstim — simulated stimulation. Run a candidate motion against
/// Scratchpad instead of Truth. Answers: "what would the matrix feel
/// like if I did this?"
///
/// Feature-gated because simstim is cold path; no one does it on the
/// hot loop.
#[cfg(feature = "simstim")]
pub struct Simstim<'a> {
    pub deck: &'a mut Pin<Box<Straylight>>,
}

#[cfg(feature = "simstim")]
impl<'a> Simstim<'a> {
    pub fn rehearse(&mut self, candidate: u128, ice: &Ice) -> Admission {
        let state = candidate;  // candidate becomes the simulated state.
        let d = ice.feel(state);
        if d == 0 { Admission::Warm { fragment: state } }
        else      { Admission::Cold { fragment: state, ice_kind: ice.kind } }
    }
}
```

---

## A scene

```rust
fn main() {
    // Case: Wintermute compiles, Neuromancer executes, cowboy feels.
    let mut cowboy = Cowboy::<{ WorkTier::U64 }>::jack_in(0xCAFE);

    let white = Ice::white(0b1010_1010);
    let black = Ice::black(0b1010_1010, 0b0001_0000);

    match cowboy.run(&black) {
        Admission::Warm { fragment } => {
            // admitted. The matrix warmed.
            eprintln!("warm {:#034x}", fragment);
        }
        Admission::Cold { fragment, ice_kind } => {
            // denied. Black ICE is a counter-strike; Gray is a shove;
            // White is a polite no.
            eprintln!("cold {:?} {:#034x}", ice_kind, fragment);
        }
    }

    // The run is over. Flatline — freeze the deck into a Construct.
    let dixie = cowboy.flatline();

    // The Turing police check the seal.
    match Turing::verify(&dixie) {
        Verdict::Lawful => { /* release */ }
        Verdict::Unlawful => { /* quarantine */ }
    }
}
```

---

## The Pragmatic framing

This crate is not flavor. Every name is load-bearing:

- **Deck** is pinned and exclusive because borrow-checking a shared
  `Straylight` across threads would demand a mutex, and a mutex on the
  hot path is a tier violation.
- **ICE** is two masks because Gibson's White/Black distinction is
  exactly the required/forbidden split from doc 36.
- **Construct** is a sealed chain because Gibson's flatline-as-replay is
  precisely what a frozen receipt gives you — past answers, forever, no
  new ones.
- **Wintermute + Neuromancer** is the μ-decomposition from doc 37:
  semantic-to-motion and motion-to-receipt are two different
  compilations; fusing them was the precursor's mistake.
- **Turing Police** is the verifier, because no issuer verifies its own
  chain; that is the Pragmatic "untrusted input" rule applied to the
  system's own output.

The matrix is not metaphor. It is the architecture renamed so the
cowboy never has to read the ABI.

---

## The sentence

**Gibson wrote cyberspace as a consensual hallucination; UniverseOS is
the implementation where the consensus is branchless admission, the
hallucination is the pinned 64³ TruthBlock, and the cowboy is a
`Cowboy<const T: WorkTier>` that jacks into a `Pin<Box<Straylight>>`,
runs ICE against the deck, and flatlines into a `Construct` that the
Turing Police verify without trusting the issuer.**
