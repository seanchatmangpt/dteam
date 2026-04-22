#![no_std]

//! # unibit-kernel
//!
//! The strictly branchless, `no_alloc`, `no_std` execution kernel.
//! Applies mask calculus to evaluate semantic motion through the `unibit-l1`
//! resident structures based on `unibit-motion` trajectory constraints.

#[cfg(target_arch = "x86_64")]
use core::arch::asm;

use unibit_l1::L1Region;
use unibit_motion::MotionPacket;

/// Direct inline assembly verification test function.
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

/// Executes a purely branchless mask algebra update.
///
/// The geometric state motion rule:
/// `deny = (S & prereq) ^ prereq | (S & law) ^ law`
/// `admitted_mask = mask(deny == 0)`
/// `next = ( (S & !consume) | produce ) & admitted_mask | S & !admitted_mask`
///
/// # Doctests
///
/// ```rust
/// use unibit_kernel::execute_hot_path_word;
/// use unibit_motion::MotionPacket;
///
/// let state = 0b1011;
/// let packet = MotionPacket::new(
///     0b1000, // prereq: must have high bit
///     0b0000, // law: no structural limits applied here
///     0b0010, // consume: removes the second bit
///     0b0100  // produce: adds the third bit
/// );
/// 
/// // State 1011 consumes 0010 (leaving 1001) and produces 0100 -> 1101
/// let next_state = execute_hot_path_word(state, &packet);
/// assert_eq!(next_state, 0b1101);
/// 
/// // Fail the prerequisite check (requires bit 10000 which is 0)
/// let bad_packet = MotionPacket::new(0b10000, 0, 0b10, 0b100);
/// let unchanged = execute_hot_path_word(state, &bad_packet);
/// assert_eq!(unchanged, state);
/// ```
#[inline(always)]
pub fn execute_hot_path_word(state: u64, packet: &MotionPacket) -> u64 {
    let deny_prereq = (state & packet.prereq_mask) ^ packet.prereq_mask;
    let deny_law = (state & packet.law_mask) ^ packet.law_mask;
    
    let deny = deny_prereq | deny_law;
    
    // Broadcast denial to a 64-bit mask branchlessly.
    // If deny == 0, `is_zero` becomes 1. Else 0.
    let is_zero = ((deny | deny.wrapping_neg()) >> 63) ^ 1;
    // 1.wrapping_neg() is 0xFFFF_FFFF_FFFF_FFFF. 0 is 0.
    let admitted_mask = is_zero.wrapping_neg();

    let candidate = (state & !packet.consume_mask) | packet.produce_mask;

    // Apply candidate if admitted, else preserve state.
    (candidate & admitted_mask) | (state & !admitted_mask)
}

/// Executes the motion packet against the first word of the TruthBlock and writes delta to Scratchpad.
/// This demonstrates the absolute minimum branchless interaction with the L1 resident envelope.
pub fn execute_hot_path(region: &mut L1Region, packet: &MotionPacket) {
    let current = region.truth.words[0];
    let next = execute_hot_path_word(current, packet);
    
    // Compute sparse delta in the scratch workspace
    let delta = current ^ next;
    region.scratch.words[0] = delta;
    
    // Apply admitted motion back to the resident truth block
    region.truth.words[0] = next;
}
