use insa_instinct::{InstinctByte, KappaByte, MycinByte};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MycinStatus {
    Fired = 0,
    Conflict = 1,
    #[default]
    NoMatch = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MycinResult {
    pub status: MycinStatus,
    pub detail: MycinByte,
    pub kappa: KappaByte,
    pub emits: InstinctByte,
}
