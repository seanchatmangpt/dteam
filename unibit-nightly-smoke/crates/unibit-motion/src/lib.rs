#![no_std]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

//! # unibit-motion
//!
//! Defines the geometric state motion structures for the `uni*` ecosystem.
//! This includes the `64^2` attention surface and the `64^3` operational depth.
//! Motion is evaluated as a lawful trajectory through `GlobeCell` coordinates.

/// Timing constitution: $8^1$ work tier
pub const TIER_8_1: usize = 8;
/// Timing constitution: $8^2$ work tier
pub const TIER_8_2: usize = 64;
/// Timing constitution: $8^3$ work tier
pub const TIER_8_3: usize = 512;
/// Timing constitution: $8^4$ work tier
pub const TIER_8_4: usize = 4096;
/// Timing constitution: $8^5$ work tier
pub const TIER_8_5: usize = 32768;

/// A bounded work envelope scaled strictly to $8^n$ geometric constraints.
pub struct Work<const BITS: usize>
where
    [(); BITS / 64]:,
{
    /// Internal 64-bit alignment array matching the tier size.
    pub words: [u64; BITS / 64],
}

/// Represents a discrete position in the $64^3$ operational geometry.
///
/// # Doctests
///
/// ```rust
/// use unibit_motion::GlobeCell;
///
/// // Create a coordinate: domain 1, cell 4095 (surface), place 63 (depth)
/// let coord = GlobeCell::new(1, 4095, 63);
/// assert_eq!(coord.domain, 1);
/// assert_eq!(coord.cell, 4095);
/// assert_eq!(coord.place, 63);
/// 
/// // Calculate bit offset
/// assert_eq!(coord.bit_offset(), (4095 * 64) + 63);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobeCell {
    /// The universe or process domain.
    pub domain: u64,
    /// The $64^2$ attention surface coordinate (0..4095).
    pub cell: u64,
    /// The $64$ operational depth state within the cell (0..63).
    pub place: u64,
}

impl GlobeCell {
    /// Establishes a new canonical coordinate.
    pub const fn new(domain: u64, cell: u64, place: u64) -> Self {
        Self { domain, cell, place }
    }

    /// Computes the exact linear bit offset for masking.
    pub const fn bit_offset(&self) -> u64 {
        (self.cell * 64) + self.place
    }
}

/// A kinetic motion packet compiled by MuStar.
/// It provides the explicit bit-level constraints for a lawful trajectory transition.
pub struct MotionPacket {
    /// Mask of required bits to proceed
    pub prereq_mask: u64,
    /// Mask of structural law bits that cannot be violated
    pub law_mask: u64,
    /// Mask of bits that are zeroed upon transition
    pub consume_mask: u64,
    /// Mask of bits that are asserted upon transition
    pub produce_mask: u64,
}

impl MotionPacket {
    /// Creates a pre-compiled packet of geometric motion.
    pub const fn new(prereq: u64, law: u64, consume: u64, produce: u64) -> Self {
        Self {
            prereq_mask: prereq,
            law_mask: law,
            consume_mask: consume,
            produce_mask: produce,
        }
    }
}
