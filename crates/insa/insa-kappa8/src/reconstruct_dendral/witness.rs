#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ReconstructionWitnessId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ReconstructionWitness {
    pub id: ReconstructionWitnessId,
    pub fragments_evaluated: u64,
    pub candidates_pruned: u32,
    pub final_candidate_count: u16,
}
