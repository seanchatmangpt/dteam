use crate::reflect_eliza::pattern::PatternId;
use insa_instinct::{ElizaByte, InstinctByte, KappaByte};
use insa_types::FieldMask;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ReflectStatus {
    Matched = 0,
    Incomplete = 1,
    #[default]
    NoMatch = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReflectResult {
    pub status: ReflectStatus,
    pub detail: ElizaByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
    pub missing_slots: FieldMask,
    pub selected_pattern: Option<PatternId>,
}
