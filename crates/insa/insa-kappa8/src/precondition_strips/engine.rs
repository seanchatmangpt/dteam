use crate::precondition_strips::result::PreconditionResult;
use crate::precondition_strips::schema::{ActionSchema, PolicyEpoch};
use insa_instinct::{InstinctByte, KappaByte, StripsByte};
use insa_types::FieldMask;

pub struct PreconditionStrips;

impl PreconditionStrips {
    pub fn evaluate(
        schema: &ActionSchema,
        present: FieldMask,
        current_epoch: PolicyEpoch,
    ) -> PreconditionResult {
        let missing = (present.0 & schema.required.0 .0) ^ schema.required.0 .0;
        let forbidden = present.0 & schema.forbidden.0 .0;

        let is_stale = schema.policy_epoch.0 != current_epoch.0;
        let missing_not_zero = missing != 0;
        let forbidden_not_zero = forbidden != 0;
        let effects_conflict = (schema.add_effects.0 & schema.clear_effects.0) != 0;
        let satisfied = !missing_not_zero && !forbidden_not_zero && !effects_conflict;

        // Path A: is_stale is true
        let strips_stale = StripsByte::ACTION_BLOCKED;
        let emits_stale = InstinctByte::AWAIT.union(InstinctByte::ESCALATE);

        // Path B: is_stale is false
        let strips_missing = StripsByte([0, StripsByte::MISSING_REQUIRED.0][missing_not_zero as usize]);
        let emits_missing = InstinctByte([0, InstinctByte::RETRIEVE.bits() | InstinctByte::ASK.bits() | InstinctByte::AWAIT.bits()][missing_not_zero as usize]);

        let strips_forbidden = StripsByte([0, StripsByte::FORBIDDEN_PRESENT.0][forbidden_not_zero as usize]);
        let emits_forbidden = InstinctByte([0, InstinctByte::REFUSE.bits()][forbidden_not_zero as usize]);

        let strips_conflict = StripsByte([StripsByte::EFFECTS_KNOWN.0, StripsByte::EFFECTS_CONFLICT.0][effects_conflict as usize]);
        let emits_conflict = InstinctByte([0, InstinctByte::INSPECT.bits()][effects_conflict as usize]);

        let strips_sat = StripsByte([
            StripsByte::ACTION_BLOCKED.0 | StripsByte::REQUIRES_REPLAN.0,
            StripsByte::PRECONDITIONS_SATISFIED.0 | StripsByte::ACTION_ENABLED.0
        ][satisfied as usize]);
        let emits_sat = InstinctByte([InstinctByte::REFUSE.bits() | InstinctByte::ESCALATE.bits(), 0][satisfied as usize]);

        let strips_not_stale = StripsByte(strips_missing.0 | strips_forbidden.0 | strips_conflict.0 | strips_sat.0);
        let emits_not_stale = InstinctByte(emits_missing.bits() | emits_forbidden.bits() | emits_conflict.bits() | emits_sat.bits());

        // Select final results based on is_stale
        let strips = StripsByte( (strips_not_stale.0 & (!is_stale as u8).wrapping_neg()) | (strips_stale.0 & (is_stale as u8).wrapping_neg()) );
        let emits = InstinctByte( (emits_not_stale.bits() & (!is_stale as u8).wrapping_neg()) | (emits_stale.bits() & (is_stale as u8).wrapping_neg()) );

        PreconditionResult {
            detail: strips,
            kappa: KappaByte::PRECONDITION,
            emits,
            missing_required: FieldMask(missing),
            present_forbidden: FieldMask(forbidden),
            add_effects: schema.add_effects,
            clear_effects: schema.clear_effects,
        }
    }
}
