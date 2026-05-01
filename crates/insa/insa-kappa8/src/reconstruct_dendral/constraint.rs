use crate::reconstruct_dendral::fragment::FragmentId;
use insa_types::{FieldMask, PolicyEpoch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ConstraintId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintKind {
    TimeOrder {
        before: FragmentId,
        after: FragmentId,
    },
    SameObject {
        a: FragmentId,
        b: FragmentId,
    },
    RequiredMask {
        mask: FieldMask,
    },
    ForbiddenMask {
        mask: FieldMask,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconstructionConstraint {
    pub id: ConstraintId,
    pub kind: ConstraintKind,
    pub valid_time: crate::reconstruct_dendral::fragment::TimeRange,
    pub epoch: PolicyEpoch,
}
