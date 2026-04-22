#![no_std]
#[cfg(target_arch = "x86_64")]
use core::arch::asm;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn asm_add_one(x: u64) -> u64 {
    let out: u64;
    unsafe {
        asm!(
            "lea {out}, [{x} + 1]",
            x = in(reg) x,
            out = lateout(reg) out,
            options(nomem, nostack, preserves_flags)
        );
    }
    out
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn asm_add_one(x: u64) -> u64 {
    x + 1
}

pub fn execute_hot_path(_region: &mut unibit_l1::L1Region) {
    // Branchless execution placeholder
}
