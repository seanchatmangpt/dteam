use crate::reconstruct_dendral::fragment::ReconstructionRule;
use crate::reconstruct_dendral::result::{DendralResult, DendralStatus};
use insa_instinct::{DendralByte, InstinctByte, KappaByte};
use insa_types::FieldMask;

pub struct ReconstructDendral;

impl ReconstructDendral {
    pub fn evaluate(rules: &[ReconstructionRule], fragments_present: FieldMask) -> DendralResult {
        let mut detail = DendralByte::empty();
        let mut emits = InstinctByte::empty();

        let mut valid_hypotheses = 0;

        for rule in rules {
            let missing =
                (fragments_present.0 & rule.fragments_required.0) ^ rule.fragments_required.0;
            let forbidden = fragments_present.0 & rule.constraints_forbidden.0;

            let has_missing = missing != 0;
            let has_forbidden = forbidden != 0;
            let is_valid = !has_missing && !has_forbidden;

            let detail_missing_bit = [0, DendralByte::MISSING_FRAGMENT.0][has_missing as usize];
            let detail_forbidden_bit = [0, DendralByte::CONSTRAINT_VIOLATION.0 | DendralByte::CANDIDATE_PRUNED.0][has_forbidden as usize];
            let detail_valid_bit = [0, DendralByte::FRAGMENTS_SUFFICIENT.0 | DendralByte::CANDIDATE_GENERATED.0][is_valid as usize];
            let emits_valid_bit = [0, rule.emits.bits()][is_valid as usize];

            detail = DendralByte(detail.0 | detail_missing_bit | detail_forbidden_bit | detail_valid_bit);
            emits = InstinctByte(emits.bits() | emits_valid_bit);
            valid_hypotheses += is_valid as i32;
        }

        let is_unique = valid_hypotheses == 1;
        let is_ambiguous = valid_hypotheses > 1;
        let is_failed = valid_hypotheses == 0;

        let select_mask_unique = (is_unique as u8).wrapping_neg();
        let select_mask_ambiguous = (is_ambiguous as u8).wrapping_neg();
        let select_mask_failed = (is_failed as u8).wrapping_neg();

        // Case A: Unique
        let detail_a = DendralByte(detail.0 | DendralByte::UNIQUE_RECONSTRUCTION.0);
        let emits_a = InstinctByte(emits.bits() | InstinctByte::SETTLE.bits());

        // Case B: Ambiguous
        let detail_b = DendralByte(detail.0 | DendralByte::MULTIPLE_RECONSTRUCTIONS.0 | DendralByte::RECONSTRUCTION_UNSTABLE.0);
        let emits_b = InstinctByte(emits.bits() | InstinctByte::INSPECT.bits() | InstinctByte::ASK.bits());

        // Case C: Failed
        let detail_c = detail;
        let emits_c = InstinctByte(emits.bits() | InstinctByte::RETRIEVE.bits() | InstinctByte::ASK.bits());

        let final_status = [
            [DendralStatus::Failed, DendralStatus::Ambiguous][is_ambiguous as usize],
            DendralStatus::Unique,
        ][is_unique as usize];

        let final_detail = DendralByte(
            (detail_a.0 & select_mask_unique) |
            (detail_b.0 & select_mask_ambiguous) |
            (detail_c.0 & select_mask_failed)
        );

        let final_emits = InstinctByte(
            (emits_a.bits() & select_mask_unique) |
            (emits_b.bits() & select_mask_ambiguous) |
            (emits_c.bits() & select_mask_failed)
        );

        DendralResult {
            status: final_status,
            detail: final_detail,
            kappa: KappaByte::RECONSTRUCT,
            emits: final_emits,
        }
    }
}
