#![no_std]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

//! # unibit-l1
//!
//! The foundational physical geometry of the `unibit` execution substrate.
//! This crate defines the 512-bit aligned, L1-resident `TruthBlock` and `Scratchpad`.
//! It strictly adheres to the Lexicon Law: scaling by `8^n` and `64^n`, tracking
//! exact bit counts without reference to conventional storage paradigms.

pub const WORDS: usize = 4096;
pub const BLOCK_BITS: usize = 262144;
pub const REGION_BITS: usize = 524288;
pub const ALIGN_BITS: usize = 512;

/// The `TruthBlock` represents the verified, pinned state of the operational globe.
/// It spans exactly 262144 bits.
#[repr(C, align(64))]
pub struct TruthBlock {
    pub words: [u64; WORDS],
}

/// The `Scratchpad` is the isolated workspace for geometric motion evaluation.
/// It spans exactly 262144 bits.
#[repr(C, align(64))]
pub struct Scratchpad {
    pub words: [u64; WORDS],
}

/// The complete L1-resident execution region.
///
/// # Doctests
///
/// ```rust
/// use unibit_l1::{L1Region, BLOCK_BITS, REGION_BITS, ALIGN_BITS};
/// use core::mem::{size_of, align_of};
///
/// let region = L1Region::zeroed();
/// // Ensure exact bit sizes corresponding to 8^n / 64^n geometry
/// assert_eq!(size_of::<L1Region>() * 8, REGION_BITS);
/// assert_eq!(align_of::<L1Region>() * 8, ALIGN_BITS);
/// ```
#[repr(C, align(64))]
pub struct L1Region {
    pub truth: TruthBlock,
    pub scratch: Scratchpad,
}

impl L1Region {
    /// Initializes a strictly zeroed L1-resident region.
    pub const fn zeroed() -> Self {
        Self {
            truth: TruthBlock { words: [0; WORDS] },
            scratch: Scratchpad { words: [0; WORDS] },
        }
    }

    /// Computes the base address of the region in memory.
    pub fn base_addr(&self) -> usize {
        self as *const L1Region as usize
    }

    /// Computes the address of the resident truth block.
    pub fn truth_addr(&self) -> usize {
        &self.truth as *const TruthBlock as usize
    }

    /// Computes the address of the scratch memory region.
    pub fn scratch_addr(&self) -> usize {
        &self.scratch as *const Scratchpad as usize
    }
}
