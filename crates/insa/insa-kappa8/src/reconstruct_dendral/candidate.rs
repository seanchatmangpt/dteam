use insa_types::FieldMask;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct CandidateId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ReconstructionCandidate {
    pub id: CandidateId,
    pub support: FieldMask,
    pub inferred: FieldMask,
    pub satisfied_constraints: u64,
    pub violated_constraints: u64,
    pub fragments_used: u64,
    pub score: i32,
}

pub const MAX_CANDIDATES: usize = 16;
pub const MAX_FRAGMENTS: usize = 64;

pub struct CandidateArena {
    pub candidates: [ReconstructionCandidate; MAX_CANDIDATES],
    pub len: usize,
}

impl CandidateArena {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            candidates: [ReconstructionCandidate {
                id: CandidateId(0),
                support: FieldMask(0),
                inferred: FieldMask(0),
                satisfied_constraints: 0,
                violated_constraints: 0,
                fragments_used: 0,
                score: 0,
            }; MAX_CANDIDATES],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, candidate: ReconstructionCandidate) -> Result<(), &'static str> {
        if self.len < MAX_CANDIDATES {
            self.candidates[self.len] = candidate;
            self.len += 1;
            Ok(())
        } else {
            Err("Candidate explosion: budget exhausted")
        }
    }
}

impl Default for CandidateArena {
    fn default() -> Self {
        Self::new()
    }
}
