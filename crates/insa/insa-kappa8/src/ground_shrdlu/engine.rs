use crate::ground_shrdlu::result::{GroundingResult, GroundingStatus};
use crate::ground_shrdlu::symbol::GroundingRule;
use insa_instinct::{InstinctByte, KappaByte, ShrdluByte};
use insa_types::{FieldMask, ObjectRef};

pub struct GroundShrdlu;

impl GroundShrdlu {
    pub fn evaluate(
        rules: &[GroundingRule],
        symbol_detected: bool,
        present: FieldMask,
    ) -> GroundingResult {
        let mut matched_rules = 0u32;
        let mut last_object_present = 0u64;
        let mut last_object_val = 0u64;

        for rule in rules {
            let matched = ((present.0 & rule.required_context.0) == rule.required_context.0) as u64;
            matched_rules += matched as u32;

            let select_mask = matched.wrapping_neg();
            last_object_present |= matched;
            last_object_val = (rule.expected_object.0 & select_mask) | (last_object_val & !select_mask);
        }

        let is_missing = !symbol_detected;
        let is_resolved = symbol_detected && (matched_rules == 1);
        let is_ambiguous = symbol_detected && (matched_rules > 1);
        let is_failed = symbol_detected && (matched_rules == 0);

        let select_missing_mask = (is_missing as u8).wrapping_neg();
        let select_resolved_mask = (is_resolved as u8).wrapping_neg();
        let select_ambiguous_mask = (is_ambiguous as u8).wrapping_neg();
        let select_failed_mask = (is_failed as u8).wrapping_neg();

        // Outcome 0: Missing
        let detail_0 = ShrdluByte::MISSING_OBJECT;
        let emits_0 = InstinctByte::ASK.union(InstinctByte::RETRIEVE);

        // Outcome 1: Resolved
        let detail_1 = ShrdluByte::SYMBOL_RESOLVED.union(ShrdluByte::OBJECT_UNIQUE);
        let emits_1 = InstinctByte::SETTLE;

        // Outcome 2: Ambiguous
        let detail_2 = ShrdluByte::AMBIGUOUS_REFERENCE.union(ShrdluByte::GROUNDING_FAILED);
        let emits_2 = InstinctByte::INSPECT.union(InstinctByte::ASK);

        // Outcome 3: Failed
        let detail_3 = ShrdluByte::GROUNDING_FAILED;
        let emits_3 = InstinctByte::ASK.union(InstinctByte::RETRIEVE);

        let status = [
            [
                [GroundingStatus::Failed, GroundingStatus::Ambiguous][is_ambiguous as usize],
                GroundingStatus::Resolved
            ][is_resolved as usize],
            GroundingStatus::Missing
        ][is_missing as usize];

        let final_detail = ShrdluByte(
            (detail_0.0 & select_missing_mask) |
            (detail_1.0 & select_resolved_mask) |
            (detail_2.0 & select_ambiguous_mask) |
            (detail_3.0 & select_failed_mask)
        );

        let final_emits = InstinctByte(
            (emits_0.bits() & select_missing_mask) |
            (emits_1.bits() & select_resolved_mask) |
            (emits_2.bits() & select_ambiguous_mask) |
            (emits_3.bits() & select_failed_mask)
        );

        let resolved_present = is_resolved && (last_object_present != 0);
        let resolved_object = [None, Some(ObjectRef(last_object_val))][resolved_present as usize];

        GroundingResult {
            status,
            detail: final_detail,
            kappa: KappaByte::GROUND,
            emits: final_emits,
            resolved_object,
        }
    }
}
