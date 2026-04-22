#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::mem::{align_of, size_of};
use std::pin::Pin;

use unibit_l1::{L1Region, TruthBlock, Scratchpad, ALIGN_BITS, BLOCK_BITS, REGION_BITS, WORDS};
use unibit_kernel::{asm_add_one, execute_hot_path};
use unibit_motion::{Work, GlobeCell, TIER_8_1, TIER_8_2, TIER_8_3, TIER_8_4, TIER_8_5};

#[derive(Debug, Clone, Copy)]
struct L1Position {
    base: usize,
    truth: usize,
    scratch: usize,
    truth_offset_bits: usize,
    scratch_offset_bits: usize,
}

fn validate_l1_position(region: &L1Region) -> L1Position {
    assert_eq!(size_of::<TruthBlock>() * 8, BLOCK_BITS);
    assert_eq!(size_of::<Scratchpad>() * 8, BLOCK_BITS);
    assert_eq!(size_of::<L1Region>() * 8, REGION_BITS);

    assert_eq!(align_of::<TruthBlock>() * 8, ALIGN_BITS);
    assert_eq!(align_of::<Scratchpad>() * 8, ALIGN_BITS);
    assert_eq!(align_of::<L1Region>() * 8, ALIGN_BITS);

    let base = region.base_addr();
    let truth = region.truth_addr();
    let scratch = region.scratch_addr();

    assert_eq!((base * 8) % ALIGN_BITS, 0);
    assert_eq!((truth * 8) % ALIGN_BITS, 0);
    assert_eq!((scratch * 8) % ALIGN_BITS, 0);

    let truth_offset_raw = truth - base;
    let scratch_offset_raw = scratch - base;

    assert_eq!(truth_offset_raw * 8, 0);
    assert_eq!(scratch_offset_raw * 8, BLOCK_BITS);

    L1Position {
        base,
        truth,
        scratch,
        truth_offset_bits: truth_offset_raw * 8,
        scratch_offset_bits: scratch_offset_raw * 8,
    }
}

#[cfg(unix)]
unsafe fn lock_region(region: &L1Region) -> std::io::Result<()> {
    let ptr = (region as *const L1Region).cast::<libc::c_void>();
    let len = size_of::<L1Region>();

    let rc = unsafe { libc::mlock(ptr, len) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn main() {
    let _work_tier3: Work<TIER_8_3> = Work { words: [0; TIER_8_3 / 64] };
    let _cell = GlobeCell { domain: 0, cell: 42, place: 5 };

    let mut region: Pin<Box<L1Region>> = Box::pin(L1Region::zeroed());

    let pos1 = validate_l1_position(&region);
    let pos2 = validate_l1_position(&region);

    assert_eq!(pos1.base, pos2.base);
    assert_eq!(pos1.truth, pos2.truth);
    assert_eq!(pos1.scratch, pos2.scratch);

    #[cfg(unix)]
    unsafe {
        match lock_region(&region) {
            Ok(()) => println!("mlock: ok"),
            Err(e) => println!("mlock: skipped/failed: {e}"),
        }
    }

    let asm_result = unsafe { asm_add_one(41) };
    assert_eq!(asm_result, 42);
    
    // Simulate hot path execution
    execute_hot_path(&mut region);

    println!("nightly compiler passed");
    println!("generic_const_exprs passed");
    println!("pinned L1 position validated");
    println!("inline asm smoke passed");
    println!("base    = 0x{:x}", pos1.base);
    println!("truth   = 0x{:x} offset_bits={}", pos1.truth, pos1.truth_offset_bits);
    println!("scratch = 0x{:x} offset_bits={}", pos1.scratch, pos1.scratch_offset_bits);
}