use insa_instinct::{DendralByte, InstinctByte, KappaByte};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DendralStatus {
    Unique = 0,
    Ambiguous = 1,
    #[default]
    Failed = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DendralResult {
    pub status: DendralStatus,
    pub detail: DendralByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
}
