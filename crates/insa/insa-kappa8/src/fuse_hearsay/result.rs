use crate::fuse_hearsay::witness::FusionWitnessId;
use insa_instinct::{HearsayByte, InstinctByte, KappaByte};
use insa_types::FieldMask;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FusionStatus {
    Complete = 0,
    #[default]
    Incomplete = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FusionResult {
    pub status: FusionStatus,
    pub detail: HearsayByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
    pub agreed: FieldMask,
    pub conflicted: FieldMask,
    pub missing: FieldMask,
    pub stale: FieldMask,
    pub witness_index: FusionWitnessId,
}
