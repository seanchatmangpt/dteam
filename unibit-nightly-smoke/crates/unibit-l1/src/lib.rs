#![no_std]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub const WORDS: usize = 4096;
pub const BLOCK_BITS: usize = 262144;
pub const REGION_BITS: usize = 524288;
pub const ALIGN_BITS: usize = 512;

#[repr(C, align(64))]
pub struct TruthBlock {
    pub words: [u64; WORDS],
}

#[repr(C, align(64))]
pub struct Scratchpad {
    pub words: [u64; WORDS],
}

#[repr(C, align(64))]
pub struct L1Region {
    pub truth: TruthBlock,
    pub scratch: Scratchpad,
}

impl L1Region {
    pub const fn zeroed() -> Self {
        Self {
            truth: TruthBlock { words: [0; WORDS] },
            scratch: Scratchpad { words: [0; WORDS] },
        }
    }

    pub fn base_addr(&self) -> usize {
        self as *const L1Region as usize
    }

    pub fn truth_addr(&self) -> usize {
        &self.truth as *const TruthBlock as usize
    }

    pub fn scratch_addr(&self) -> usize {
        &self.scratch as *const Scratchpad as usize
    }
}
