//! Autonomic Instinct response classes (Phase 5 Track F stub).
//!
//! Maps closed `O*` (compiled snapshot + posture + context) to a single
//! right-sized response class. Track F writer fleshes out `select_instinct()`
//! against the multimodal bundles.

use crate::compiled::CompiledFieldSnapshot;
use crate::multimodal::{ContextBit, ContextBundle, PostureBit, PostureBundle};

/// Right-sized response class — the action the cognition surface admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutonomicInstinct {
    /// Known harmless event — return to baseline.
    Settle,
    /// Expected package/delivery — retrieve now.
    Retrieve,
    /// Unknown but low-threat — inspect.
    Inspect,
    /// Missing evidence — request clarification.
    Ask,
    /// Action does not belong — refuse the transition.
    Refuse,
    /// Persistent unresolved disturbance — escalate.
    Escalate,
    /// No-op.
    Ignore,
}

/// Select a single response class from the closed cognition surface.
///
/// This is a stub mapping using only posture + context bits. The full
/// implementation in Track F+ uses the snapshot's predicate masks too.
pub fn select_instinct(
    _snap: &CompiledFieldSnapshot,
    posture: &PostureBundle,
    ctx: &ContextBundle,
) -> AutonomicInstinct {
    if posture.has(PostureBit::SETTLED) {
        return AutonomicInstinct::Settle;
    }
    if (ctx.expectation_mask & (1u64 << ContextBit::PACKAGE_EXPECTED)) != 0
        && (ctx.affordance_mask & (1u64 << ContextBit::CAN_RETRIEVE_NOW)) != 0
    {
        return AutonomicInstinct::Retrieve;
    }
    if (ctx.risk_mask & (1u64 << ContextBit::MUST_ESCALATE)) != 0 {
        return AutonomicInstinct::Escalate;
    }
    if (ctx.affordance_mask & (1u64 << ContextBit::CAN_INSPECT)) != 0
        && posture.has(PostureBit::ALERT)
    {
        return AutonomicInstinct::Inspect;
    }
    if posture.has(PostureBit::CALM) && ctx.expectation_mask == 0 && ctx.risk_mask == 0 {
        return AutonomicInstinct::Ignore;
    }
    AutonomicInstinct::Ask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::FieldContext;

    #[test]
    fn settled_yields_settle() {
        let f = FieldContext::new("t");
        let snap = CompiledFieldSnapshot::from_field(&f).unwrap();
        let posture = PostureBundle {
            posture_mask: 1u64 << PostureBit::SETTLED,
            confidence: 200,
        };
        let ctx = ContextBundle::default();
        assert_eq!(select_instinct(&snap, &posture, &ctx), AutonomicInstinct::Settle);
    }

    #[test]
    fn package_expected_plus_affordance_yields_retrieve() {
        let f = FieldContext::new("t");
        let snap = CompiledFieldSnapshot::from_field(&f).unwrap();
        let posture = PostureBundle {
            posture_mask: 1u64 << PostureBit::ALERT,
            confidence: 200,
        };
        let ctx = ContextBundle {
            expectation_mask: 1u64 << ContextBit::PACKAGE_EXPECTED,
            risk_mask: 0,
            affordance_mask: 1u64 << ContextBit::CAN_RETRIEVE_NOW,
        };
        assert_eq!(select_instinct(&snap, &posture, &ctx), AutonomicInstinct::Retrieve);
    }
}
