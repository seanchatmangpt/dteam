use crate::fuse_hearsay::blackboard::Blackboard;
use crate::fuse_hearsay::fusion_rule::FusionRule;
use crate::fuse_hearsay::result::{FusionResult, FusionStatus};
use crate::fuse_hearsay::witness::FusionWitnessId;
use insa_instinct::{HearsayByte, InstinctByte, KappaByte};
use insa_types::FieldMask;

pub struct FuseHearsay;

impl FuseHearsay {
    pub fn fuse(board: &Blackboard, rule: &FusionRule) -> FusionResult {
        let missing = (board.present.0 & rule.required_sources.0 .0) ^ rule.required_sources.0 .0;
        let conflicts = board.conflicted.0 & rule.conflict_mask.0 .0;

        let has_missing = missing != 0;
        let has_conflicts = conflicts != 0;
        let has_stale = board.stale.0 != 0;
        let is_complete = (!has_missing) && (!has_conflicts) && (!has_stale);

        let detail_missing = [0, HearsayByte::SOURCE_MISSING.0][has_missing as usize];
        let emits_missing = [0, InstinctByte::RETRIEVE.bits() | InstinctByte::ASK.bits()][has_missing as usize];

        let detail_conflict = [0, HearsayByte::SOURCE_CONFLICTS.0][has_conflicts as usize];
        let emits_conflict = [0, InstinctByte::INSPECT.bits() | InstinctByte::ESCALATE.bits()][has_conflicts as usize];

        let detail_stale = [0, HearsayByte::SOURCE_STALE.0][has_stale as usize];
        let emits_stale = [0, InstinctByte::AWAIT.bits() | InstinctByte::RETRIEVE.bits()][has_stale as usize];

        let detail_base = HearsayByte(detail_missing | detail_conflict | detail_stale);
        let emits_base = InstinctByte(emits_missing | emits_conflict | emits_stale);

        // Path A: is_complete is true
        let detail_a = HearsayByte(detail_base.0 | HearsayByte::FUSION_COMPLETE.0 | HearsayByte::SOURCE_AGREES.0);
        let emits_a = InstinctByte(emits_base.bits() | InstinctByte::SETTLE.bits());

        // Path B: is_complete is false
        let detail_conflict_inspect = [0, HearsayByte::FUSION_REQUIRES_INSPECTION.0][has_conflicts as usize];
        let detail_b = HearsayByte(detail_base.0 | detail_conflict_inspect);
        let emits_b = emits_base;

        let status = [FusionStatus::Incomplete, FusionStatus::Complete][is_complete as usize];
        let detail = HearsayByte( (detail_b.0 & (!is_complete as u8).wrapping_neg()) | (detail_a.0 & (is_complete as u8).wrapping_neg()) );
        let emits = InstinctByte( (emits_b.bits() & (!is_complete as u8).wrapping_neg()) | (emits_a.bits() & (is_complete as u8).wrapping_neg()) );

        FusionResult {
            status,
            detail,
            kappa: KappaByte::FUSE,
            emits,
            agreed: FieldMask(board.present.0 & !board.conflicted.0),
            conflicted: FieldMask(conflicts),
            missing: FieldMask(missing),
            stale: board.stale,
            witness_index: FusionWitnessId(0), // Mapped dynamically in POWL64 layer
        }
    }
}
