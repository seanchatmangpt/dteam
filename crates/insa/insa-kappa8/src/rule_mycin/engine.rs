use crate::rule_mycin::result::{MycinResult, MycinStatus};
use crate::rule_mycin::rule::{ExpertRule, PolicyEpoch};
use insa_instinct::{InstinctByte, KappaByte, MycinByte};
use insa_types::FieldMask;

pub struct RuleMycin;

impl RuleMycin {
    pub fn evaluate(
        rules: &[ExpertRule],
        present: FieldMask,
        current_epoch: PolicyEpoch,
    ) -> MycinResult {
        let mut detail = MycinByte::empty();
        let emits = InstinctByte::empty();

        let mut highest_confidence = 0u8;
        let mut selected_emits = InstinctByte::empty();
        let mut matched_rules = 0;

        for rule in rules {
            let is_stale = rule.epoch.0 != current_epoch.0;
            let is_valid = !is_stale;

            let detail_stale_bit = [0, MycinByte::POLICY_EPOCH_STALE.0][is_stale as usize];
            let detail_valid_bit = [0, MycinByte::POLICY_EPOCH_VALID.0][is_valid as usize];
            detail = MycinByte(detail.0 | detail_stale_bit | detail_valid_bit);

            let missing = (present.0 & rule.required.0) ^ rule.required.0;
            let forbidden = present.0 & rule.forbidden.0;

            let matched = missing == 0 && forbidden == 0 && is_valid;
            matched_rules += matched as i32;

            let detail_match_bit = [0, MycinByte::RULE_MATCHED.0][matched as usize];
            detail = MycinByte(detail.0 | detail_match_bit);

            let conf_greater = rule.confidence.0 > highest_confidence && matched;
            let select_mask_conf = (conf_greater as u8).wrapping_neg();
            highest_confidence = (rule.confidence.0 & select_mask_conf) | (highest_confidence & !select_mask_conf);
            selected_emits = InstinctByte((rule.emits.bits() & select_mask_conf) | (selected_emits.bits() & !select_mask_conf));
        }

        let is_conflict = matched_rules > 1;
        let is_fired = matched_rules == 1;
        let is_nomatch = matched_rules == 0;

        let select_mask_conflict = (is_conflict as u8).wrapping_neg();
        let select_mask_fired = (is_fired as u8).wrapping_neg();
        let select_mask_nomatch = (is_nomatch as u8).wrapping_neg();

        // Case A: Conflict
        let detail_a = MycinByte(detail.0 | MycinByte::RULE_CONFLICT.0 | MycinByte::EXPERT_REVIEW_REQUIRED.0);
        let emits_a = InstinctByte(emits.bits() | InstinctByte::INSPECT.bits() | InstinctByte::ESCALATE.bits());

        // Case B: Fired
        let is_high = highest_confidence >= 80;
        let detail_high_bit = [MycinByte::CONFIDENCE_LOW.0 | MycinByte::EXPERT_REVIEW_REQUIRED.0, MycinByte::CONFIDENCE_HIGH.0][is_high as usize];
        let detail_b = MycinByte(detail.0 | MycinByte::RULE_FIRED.0 | detail_high_bit);
        let emits_b = InstinctByte([selected_emits.bits() | InstinctByte::INSPECT.bits(), selected_emits.bits()][is_high as usize]);

        // Case C: NoMatch
        let detail_c = detail;
        let emits_c = InstinctByte(emits.bits() | InstinctByte::ASK.bits() | InstinctByte::RETRIEVE.bits());

        let final_status = [
            [MycinStatus::NoMatch, MycinStatus::Fired][is_fired as usize],
            MycinStatus::Conflict,
        ][is_conflict as usize];

        let final_detail = MycinByte(
            (detail_a.0 & select_mask_conflict) |
            (detail_b.0 & select_mask_fired) |
            (detail_c.0 & select_mask_nomatch)
        );

        let final_emits = InstinctByte(
            (emits_a.bits() & select_mask_conflict) |
            (emits_b.bits() & select_mask_fired) |
            (emits_c.bits() & select_mask_nomatch)
        );

        MycinResult {
            status: final_status,
            detail: final_detail,
            kappa: KappaByte::RULE,
            emits: final_emits,
        }
    }
}
