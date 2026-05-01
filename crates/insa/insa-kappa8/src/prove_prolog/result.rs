use insa_instinct::{InstinctByte, KappaByte, PrologByte};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ProofStatus {
    Proved = 0,
    #[default]
    Failed = 1,
    DepthExhausted = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProofResult {
    pub status: ProofStatus,
    pub detail: PrologByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
}
