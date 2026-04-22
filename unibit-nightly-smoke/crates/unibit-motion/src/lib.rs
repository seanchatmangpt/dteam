#![no_std]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

// Timing constitution: 8^1 through 8^5 bits
pub const TIER_8_1: usize = 8;
pub const TIER_8_2: usize = 64;
pub const TIER_8_3: usize = 512;
pub const TIER_8_4: usize = 4096;
pub const TIER_8_5: usize = 32768;

pub struct Work<const BITS: usize>
where
    [(); BITS / 64]:,
{
    pub words: [u64; BITS / 64],
}

pub struct GlobeCell {
    pub domain: u64,
    pub cell: u64,
    pub place: u64,
}
