use crate::reflect_eliza::pattern::{AskKind, ReflectPattern, PatternId};
use crate::reflect_eliza::result::{ReflectResult, ReflectStatus};
use insa_instinct::{ElizaByte, InstinctByte, KappaByte};
use insa_types::FieldMask;

pub struct ReflectEliza;

impl ReflectEliza {
    pub fn evaluate(
        patterns: &[ReflectPattern],
        present: FieldMask,
        expected_slots: FieldMask,
    ) -> ReflectResult {
        let missing_slots = (present.0 & expected_slots.0) ^ expected_slots.0;

        let mut best_pattern_id = 0u16;
        let mut best_pattern_emits = InstinctByte::empty();
        let mut best_pattern_detail = ElizaByte::empty();
        let mut best_pattern_ask_kind = AskKind::None;
        let mut already_matched = 0u8;

        for pat in patterns {
            let matched = ((present.0 & pat.required_context.0) == pat.required_context.0) as u8;
            let select_pattern = matched & (!already_matched);
            already_matched |= matched;

            let select_mask = select_pattern.wrapping_neg();
            best_pattern_id = (pat.id.0 & (select_mask as u16)) | (best_pattern_id & !(select_mask as u16));
            best_pattern_emits = InstinctByte((pat.emits.bits() & select_mask) | (best_pattern_emits.bits() & !select_mask));
            best_pattern_detail = ElizaByte((pat.eliza_detail.0 & select_mask) | (best_pattern_detail.0 & !select_mask));
            best_pattern_ask_kind = [best_pattern_ask_kind, pat.ask_kind][select_pattern as usize];
        }

        let is_incomplete = missing_slots != 0;
        let is_matched = !is_incomplete && already_matched != 0;
        let is_nomatch = !is_incomplete && already_matched == 0;

        let select_incomplete_mask = (is_incomplete as u8).wrapping_neg();
        let select_matched_mask = (is_matched as u8).wrapping_neg();
        let select_nomatch_mask = (is_nomatch as u8).wrapping_neg();

        // Outcome 0: Incomplete
        let detail_0 = ElizaByte::empty()
            .union(ElizaByte::DETECT_MISSING_SLOT)
            .union(ElizaByte::ASK_CLARIFYING);
        let emits_0 = InstinctByte::empty().union(InstinctByte::ASK).union(InstinctByte::INSPECT);
        let missing_slots_0 = FieldMask(missing_slots);

        // Outcome 1: Matched
        let is_ask = best_pattern_ask_kind as u8 != AskKind::None as u8;
        let detail_ask_bit = [0, ElizaByte::ASK_CLARIFYING.0][is_ask as usize];
        let emits_ask_bit = [0, InstinctByte::ASK.bits()][is_ask as usize];
        let detail_1 = ElizaByte(best_pattern_detail.0 | detail_ask_bit);
        let emits_1 = InstinctByte(best_pattern_emits.bits() | emits_ask_bit);
        let missing_slots_1 = FieldMask(0);

        // Outcome 2: NoMatch
        let detail_2 = ElizaByte::empty().union(ElizaByte::DEFER_TO_CLOSURE);
        let emits_2 = InstinctByte::empty().union(InstinctByte::SETTLE);
        let missing_slots_2 = FieldMask(0);

        let status = [
            [ReflectStatus::NoMatch, ReflectStatus::Matched][is_matched as usize],
            ReflectStatus::Incomplete
        ][is_incomplete as usize];

        let final_detail = ElizaByte(
            (detail_0.0 & select_incomplete_mask) |
            (detail_1.0 & select_matched_mask) |
            (detail_2.0 & select_nomatch_mask)
        );

        let final_emits = InstinctByte(
            (emits_0.bits() & select_incomplete_mask) |
            (emits_1.bits() & select_matched_mask) |
            (emits_2.bits() & select_nomatch_mask)
        );

        let final_missing_slots = FieldMask(
            (missing_slots_0.0 & (select_incomplete_mask as u64)) |
            (missing_slots_1.0 & (select_matched_mask as u64)) |
            (missing_slots_2.0 & (select_nomatch_mask as u64))
        );

        let pattern_present = is_matched;
        let selected_pattern = [None, Some(PatternId(best_pattern_id))][pattern_present as usize];

        ReflectResult {
            status,
            detail: final_detail,
            kappa: KappaByte::REFLECT,
            emits: final_emits,
            missing_slots: final_missing_slots,
            selected_pattern,
        }
    }
}
