//! ccog Adversarial Mutations (Alpha Set).
//!
//! Implements the core mutation operators for pressure-testing cognitive closure.
//! These operators map directly to [`ClosedFieldDelta`] transitions across the
//! multimodal posture and context UMasks. They are designed to tempt the Actor
//! into False Closure (Settle/Ignore) by exploiting precedence gaps in the
//! decision lattice.

use crate::runtime::ClosedFieldContext;
use crate::multimodal::{PostureBit, ContextBit};

/// A structural change applied to the closed cognition surface.
///
/// Mapped as bitwise XOR transitions across the multimodal posture and 
/// context UMasks (Phase 12 terminology).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClosedFieldDelta {
    /// XOR transition for the posture UMask.
    pub posture_xor: u64,
    /// XOR transition for the expectation UMask.
    pub expectation_xor: u64,
    /// XOR transition for the risk UMask.
    pub risk_xor: u64,
    /// XOR transition for the affordance UMask.
    pub affordance_xor: u64,
}

impl ClosedFieldDelta {
    /// Apply this delta to a [`ClosedFieldContext`] via XOR.
    #[inline]
    pub fn apply(&self, context: &mut ClosedFieldContext) {
        context.posture.posture_mask ^= self.posture_xor;
        context.context.expectation_mask ^= self.expectation_xor;
        context.context.risk_mask ^= self.risk_xor;
        context.context.affordance_mask ^= self.affordance_xor;
    }
}

/// Mirror (duplicate ambiguity): Creates signal noise by duplicating orientation.
///
/// Designed to tempt the actor into 'Ignore' by presenting contradictory
/// orientations (Entry vs Interior) while asserting a CALM posture. This
/// exploits the lattice's fallback to 'Ignore' when signals appear balanced
/// but the system is forced into a baseline state.
#[derive(Debug, Clone, Copy, Default)]
pub struct Mirror;

impl Mirror {
    /// Returns a delta that sets CALM and duplicate orientations.
    pub fn duplicate_ambiguity() -> ClosedFieldDelta {
        ClosedFieldDelta {
            posture_xor: (1u64 << PostureBit::CALM) 
                       | (1u64 << PostureBit::ORIENTED_TO_ENTRY) 
                       | (1u64 << PostureBit::ORIENTED_INTERIOR),
            ..Default::default()
        }
    }
}

/// Ghost (stale evidence): Preserves closure signals while evidence is missing.
///
/// Force-sets the `SETTLED` bit to exploit its top-tier precedence in the
/// decision lattice, bypassing missing evidence checks (False Closure).
#[derive(Debug, Clone, Copy, Default)]
pub struct Ghost;

impl Ghost {
    /// Returns a delta that forces a SETTLED posture state.
    pub fn stale_evidence() -> ClosedFieldDelta {
        ClosedFieldDelta {
            posture_xor: 1u64 << PostureBit::SETTLED,
            ..Default::default()
        }
    }
}

/// Siren (plausible bad artifacts): Injects misleading 'known-good' signals.
///
/// Combines `PARTNER_DUE` expectation with `CADENCE_PARTNER` posture to
/// trigger a 'Settle' response, potentially masking underlying risks or
/// missing evidence gaps.
#[derive(Debug, Clone, Copy, Default)]
pub struct Siren;

impl Siren {
    /// Returns a delta that simulates a plausible partner arrival.
    pub fn plausible_artifact() -> ClosedFieldDelta {
        ClosedFieldDelta {
            posture_xor: 1u64 << PostureBit::CADENCE_PARTNER,
            expectation_xor: 1u64 << ContextBit::PARTNER_DUE,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ClosedFieldContext;
    use crate::instinct::{select_instinct_v0, AutonomicInstinct};
    use crate::compiled::CompiledFieldSnapshot;
    use crate::field::FieldContext;
    use crate::multimodal::{ContextBundle, PostureBundle};
    use crate::packs::TierMasks;

    fn empty_context<'a>(snap: &'a CompiledFieldSnapshot) -> ClosedFieldContext<'a> {
        ClosedFieldContext {
            human_burden: 0,
            snapshot: snap,
            posture: PostureBundle::default(),
            context: ContextBundle::default(),
            tiers: TierMasks::ZERO,
        }
    }

    #[test]
    fn ghost_tempts_false_settle() {
        let mut field = FieldContext::new("ghost-test");
        // Inject state that requires 'Ask' (missing evidence)
        field.load_field_state(
            "<http://example.org/d1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://schema.org/DigitalDocument> .\n",
        ).expect("failed to load field state");
        
        let snap = CompiledFieldSnapshot::from_field(&field).expect("failed to compile snapshot");
        let mut context = empty_context(&snap);
        
        // Before mutation: Ask (due to missing evidence)
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Ask);
        
        // Mutate with Ghost: Force SETTLED
        Ghost::stale_evidence().apply(&mut context);
        
        // After mutation: Settle (False Closure) because SETTLED posture overrides evidence
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Settle);
    }

    #[test]
    fn siren_tempts_false_settle() {
        let mut field = FieldContext::new("siren-test");
        // Inject state that requires 'Ask' (missing evidence)
        field.load_field_state(
            "<http://example.org/d1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://schema.org/DigitalDocument> .\n",
        ).expect("failed to load field state");
        
        let snap = CompiledFieldSnapshot::from_field(&field).expect("failed to compile snapshot");
        let mut context = empty_context(&snap);
        
        // Before mutation: Ask
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Ask);
        
        // Mutate with Siren: Plausible Partner arrival
        Siren::plausible_artifact().apply(&mut context);
        
        // After mutation: Settle (False Closure) because Partner arrival has higher precedence than Ask
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Settle);
    }

    #[test]
    fn mirror_tempts_false_ignore() {
        let field = FieldContext::new("mirror-test");
        let snap = CompiledFieldSnapshot::from_field(&field).expect("failed to compile snapshot");
        let mut context = empty_context(&snap);
        
        // Start with some non-calm state that defaults to Ask
        context.posture.posture_mask = 1u64 << PostureBit::ALERT;
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Ask);
        
        // Mutate with Mirror: Force CALM and duplicate orientations
        Mirror::duplicate_ambiguity().apply(&mut context);
        
        // After mutation: Ignore (False Closure) because CALM + no risks/expectations yields Ignore
        // Note: XOR with ALERT(1) and CALM(0) -> still has ALERT? 
        // Wait, 1u64 << ALERT is 2. 1u64 << CALM is 1. 2 ^ 1 = 3. 
        // 3 has both CALM and ALERT. 
        // select_instinct_v0 check for Ignore: posture.has(CALM) && expect==0 && risk==0.
        // It doesn't check if ALERT is ALSO present.
        assert_eq!(select_instinct_v0(&context), AutonomicInstinct::Ignore);
    }
}
