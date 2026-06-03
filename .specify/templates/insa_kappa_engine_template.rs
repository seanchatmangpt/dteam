use insa_instinct::{InstinctOctet, KappaOctet, StripsOctet};
use insa_types::FieldMask;
use crate::schema::{ActionSchema, PolicyEpoch};
use crate::result::PreconditionResult;

/// Standard Template for INSA Kappa-8 Breeds
/// 
/// This encapsulates the unit-width logic to evaluate a cognitive schema
/// directly into InstinctOctet and StripsOctet representations without allocating
/// external memory on the hot path.
pub struct KappaBreedTemplate;

impl KappaBreedTemplate {
    #[inline(always)]
    pub fn evaluate(
        schema: &ActionSchema,
        present: FieldMask,
        current_epoch: PolicyEpoch,
    ) -> PreconditionResult {
        // Bitwise extraction of states
        let missing = (present.0 & schema.required.0) ^ schema.required.0;
        let forbidden = present.0 & schema.forbidden.0;

        let mut detail = StripsOctet::empty();
        let mut emits = InstinctOctet::empty();

        let is_stale = schema.policy_epoch.0 != current_epoch.0;

        if is_stale {
            emits = emits.union(InstinctOctet::AWAIT).union(InstinctOctet::ESCALATE);
            detail = detail.union(StripsOctet::ACTION_BLOCKED);
        } else {
            if missing != 0 {
                detail = detail.union(StripsOctet::MISSING_REQUIRED);
                emits = emits
                    .union(InstinctOctet::RETRIEVE)
                    .union(InstinctOctet::ASK)
                    .union(InstinctOctet::AWAIT);
            }
            if forbidden != 0 {
                detail = detail.union(StripsOctet::FORBIDDEN_PRESENT);
                emits = emits.union(InstinctOctet::REFUSE);
            }

            let effects_conflict = (schema.add_effects.0 & schema.clear_effects.0) != 0;
            if effects_conflict {
                detail = detail.union(StripsOctet::EFFECTS_CONFLICT);
                emits = emits.union(InstinctOctet::INSPECT);
            } else {
                detail = detail.union(StripsOctet::EFFECTS_KNOWN);
            }

            let satisfied = missing == 0 && forbidden == 0 && !effects_conflict;
            if satisfied {
                detail = detail
                    .union(StripsOctet::PRECONDITIONS_SATISFIED)
                    .union(StripsOctet::ACTION_ENABLED);
            } else {
                detail = detail
                    .union(StripsOctet::ACTION_BLOCKED)
                    .union(StripsOctet::REQUIRES_REPLAN);
                emits = emits
                    .union(InstinctOctet::REFUSE)
                    .union(InstinctOctet::ESCALATE);
            }
        }

        PreconditionResult {
            detail,
            kappa: KappaOctet::PRECONDITION,
            emits,
            missing_required: FieldMask(missing),
            present_forbidden: FieldMask(forbidden),
            add_effects: schema.add_effects,
            clear_effects: schema.clear_effects,
        }
    }
}
