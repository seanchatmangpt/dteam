use insa_instinct::{InstinctByte, KappaByte, ShrdluByte};
use insa_types::ObjectRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GroundingStatus {
    Resolved = 0,
    Ambiguous = 1,
    #[default]
    Missing = 2,
    Failed = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GroundingResult {
    pub status: GroundingStatus,
    pub detail: ShrdluByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
    pub resolved_object: Option<ObjectRef>,
}
