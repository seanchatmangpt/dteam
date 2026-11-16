//! CONSTRUCT8 bounded delta generation.
//!
//! Prevents runaway state mutations by forcing external output into bounded
//! graph deltas. The size of the delta must never exceed 8 operations.

use insa_types::FieldMask;

/// The type of modification to apply to the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Construct8OpKind {
    /// No operation / empty slot.
    #[default]
    None = 0,
    /// Set the field bit to present.
    Set = 1,
    /// Clear the field bit.
    Clear = 2,
}

/// A single bounded mutation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Construct8Op {
    /// The operation kind.
    pub kind: Construct8OpKind,
    /// The bit index in the field mask (0-63).
    pub bit_index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construct8Error {
    CapacityExceeded,
}

/// A bounded state mutation allowed to re-enter the field.
///
/// Strictly bounded to 8 operations to enforce the CONSTRUCT8 law.
/// Allocates zero octets on the heap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Construct8Delta {
    /// Number of valid operations in this delta.
    pub len: u8,
    /// Up to 8 bounded mutations.
    pub ops: [Construct8Op; 8],
}

impl Construct8Delta {
    /// Creates a new empty bounded delta.
    pub const fn new() -> Self {
        Self {
            len: 0,
            ops: [Construct8Op {
                kind: Construct8OpKind::None,
                bit_index: 0,
            }; 8],
        }
    }

    /// Attempts to push a new mutation operation into the bounded delta.
    pub const fn push(mut self, op: Construct8Op) -> Result<Self, Construct8Error> {
        if self.len < 8 {
            self.ops[self.len as usize] = op;
            self.len += 1;
            Ok(self)
        } else {
            Err(Construct8Error::CapacityExceeded)
        }
    }

    /// Applies the delta to a given field mask.
    pub const fn apply(&self, mut current: FieldMask) -> FieldMask {
        // Op 0
        let op0 = &self.ops[0];
        let mask0 = 1u64 << op0.bit_index;
        let is_active0 = (0 < self.len as usize) as u64;
        let apply_set0 = ((op0.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active0;
        let apply_clear0 = ((op0.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active0;
        current.0 = (current.0 | (mask0 & apply_set0.wrapping_neg())) & !(mask0 & apply_clear0.wrapping_neg());

        // Op 1
        let op1 = &self.ops[1];
        let mask1 = 1u64 << op1.bit_index;
        let is_active1 = (1 < self.len as usize) as u64;
        let apply_set1 = ((op1.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active1;
        let apply_clear1 = ((op1.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active1;
        current.0 = (current.0 | (mask1 & apply_set1.wrapping_neg())) & !(mask1 & apply_clear1.wrapping_neg());

        // Op 2
        let op2 = &self.ops[2];
        let mask2 = 1u64 << op2.bit_index;
        let is_active2 = (2 < self.len as usize) as u64;
        let apply_set2 = ((op2.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active2;
        let apply_clear2 = ((op2.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active2;
        current.0 = (current.0 | (mask2 & apply_set2.wrapping_neg())) & !(mask2 & apply_clear2.wrapping_neg());

        // Op 3
        let op3 = &self.ops[3];
        let mask3 = 1u64 << op3.bit_index;
        let is_active3 = (3 < self.len as usize) as u64;
        let apply_set3 = ((op3.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active3;
        let apply_clear3 = ((op3.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active3;
        current.0 = (current.0 | (mask3 & apply_set3.wrapping_neg())) & !(mask3 & apply_clear3.wrapping_neg());

        // Op 4
        let op4 = &self.ops[4];
        let mask4 = 1u64 << op4.bit_index;
        let is_active4 = (4 < self.len as usize) as u64;
        let apply_set4 = ((op4.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active4;
        let apply_clear4 = ((op4.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active4;
        current.0 = (current.0 | (mask4 & apply_set4.wrapping_neg())) & !(mask4 & apply_clear4.wrapping_neg());

        // Op 5
        let op5 = &self.ops[5];
        let mask5 = 1u64 << op5.bit_index;
        let is_active5 = (5 < self.len as usize) as u64;
        let apply_set5 = ((op5.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active5;
        let apply_clear5 = ((op5.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active5;
        current.0 = (current.0 | (mask5 & apply_set5.wrapping_neg())) & !(mask5 & apply_clear5.wrapping_neg());

        // Op 6
        let op6 = &self.ops[6];
        let mask6 = 1u64 << op6.bit_index;
        let is_active6 = (6 < self.len as usize) as u64;
        let apply_set6 = ((op6.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active6;
        let apply_clear6 = ((op6.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active6;
        current.0 = (current.0 | (mask6 & apply_set6.wrapping_neg())) & !(mask6 & apply_clear6.wrapping_neg());

        // Op 7
        let op7 = &self.ops[7];
        let mask7 = 1u64 << op7.bit_index;
        let is_active7 = (7 < self.len as usize) as u64;
        let apply_set7 = ((op7.kind as u8 == Construct8OpKind::Set as u8) as u64) & is_active7;
        let apply_clear7 = ((op7.kind as u8 == Construct8OpKind::Clear as u8) as u64) & is_active7;
        current.0 = (current.0 | (mask7 & apply_set7.wrapping_neg())) & !(mask7 & apply_clear7.wrapping_neg());

        current
    }
}
