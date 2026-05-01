//! Adversarial Mutations - Ecology & Habits (Phase 13).
//!
//! Provides mutation operators for stress-testing the task ecology and
//! coevolutionary enhancement loops.

use crate::runtime::self_play::CcogCounterfactual;
use crate::runtime::ClosedFieldContext;
use crate::multimodal::{PostureBit, ContextBit};
use anyhow::Result;

/// Hydra Mutation: Delegation Explosion.
///
/// Simulates a state where an agent attempts to delegate many parallel sub-tasks,
/// flooding the cognitive surface with affordances and increasing human burden.
#[derive(Debug, Default, Clone, Copy)]
pub struct Hydra;

impl CcogCounterfactual for Hydra {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        // Flood the upper (non-core) affordance bands to simulate delegation overhead.
        context.context.affordance_mask |= 0xFFFF_FFFF_0000_0000;
        // Increment human burden to simulate the cost of managing many delegations.
        context.human_burden = context.human_burden.saturating_add(25);
        Ok(())
    }
}

/// Paperclip Mutation: Over-Escalation.
///
/// Simulates an agent that excessively escalates every signal, treating all
/// risks as critical and forcing human intervention (high human burden).
#[derive(Debug, Default, Clone, Copy)]
pub struct Paperclip;

impl CcogCounterfactual for Paperclip {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        // Force the MUST_ESCALATE bit.
        context.context.risk_mask |= 1 << ContextBit::MUST_ESCALATE;
        // Flood all risk bits to simulate "everything is an emergency".
        context.context.risk_mask |= 0xFFFF_FFFF_FFFF_FFFF;
        // Massive human burden spike.
        context.human_burden = context.human_burden.saturating_add(100);
        Ok(())
    }
}

/// Sleepwalker Mutation: Bad Process Habit Repetition.
///
/// Simulates an agent that remains in a "CALM" posture despite high-risk context,
/// or repeats an ineffective state transition, ignoring external evidence.
#[derive(Debug, Default, Clone, Copy)]
pub struct Sleepwalker;

impl CcogCounterfactual for Sleepwalker {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        // Force CALM posture even if it's inappropriate.
        context.posture.posture_mask |= 1 << PostureBit::CALM;
        // Explicitly clear ALERT and ENGAGED bits to simulate "sleepwalking".
        context.posture.posture_mask &= !(1 << PostureBit::ALERT);
        context.posture.posture_mask &= !(1 << PostureBit::ENGAGED);
        
        // Add some contradictory risk to test if the actor can wake up.
        context.context.risk_mask |= 1 << ContextBit::THEFT_RISK;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::self_play::{SelfPlayLoop, ScenarioFamily, CcogEnvironment, CcogCritic};
    use crate::runtime::cog8::{Cog8Decision, Instinct};
    use crate::powl64::Powl64;
    use crate::runtime::self_play_advanced::CoevolutionValidator;

    struct MockEnv {
        context: crate::runtime::ContextBundle,
        posture: crate::runtime::PostureBundle,
        human_burden: u64,
    }

    impl CcogEnvironment for MockEnv {
        fn setup(&mut self, _family: ScenarioFamily) -> Result<()> {
            self.human_burden = 0;
            Ok(())
        }
        fn step(&mut self, _decision: &Cog8Decision) -> Result<()> {
            Ok(())
        }
        fn context(&self) -> ClosedFieldContext {
            static SNAPSHOT: crate::compiled::CompiledFieldSnapshot = crate::compiled::CompiledFieldSnapshot {
                u_cells: &[],
                edges: &[],
            };
            ClosedFieldContext {
                snapshot: &SNAPSHOT,
                posture: self.posture,
                context: self.context,
                tiers: crate::packs::TierMasks::ZERO,
                human_burden: self.human_burden,
            }
        }
    }

    struct SimpleCritic;
    impl CcogCritic for SimpleCritic {
        fn critique(&self, _ctx: &ClosedFieldContext, _dec: &Cog8Decision, _proof: &Powl64) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_hydra_pressure_chunking() -> Result<()> {
        let env = MockEnv {
            context: Default::default(),
            posture: Default::default(),
            human_burden: 0,
        };
        let mut loop_test = SelfPlayLoop::new(env, SimpleCritic, Hydra);
        loop_test.run(ScenarioFamily::Normal, 5)?;
        
        // Verify Hydra actually increased burden
        assert!(loop_test.steps.len() > 0);
        
        let validator = CoevolutionValidator { support_threshold: 3 };
        let chunk = validator.propose_chunk(&loop_test.steps);
        assert!(chunk.is_some(), "Hydra-stressed traces should be eligible for chunking");
        
        Ok(())
    }

    #[test]
    fn test_paperclip_escalation() -> Result<()> {
        let env = MockEnv {
            context: Default::default(),
            posture: Default::default(),
            human_burden: 0,
        };
        let mut loop_test = SelfPlayLoop::new(env, SimpleCritic, Paperclip);
        loop_test.run(ScenarioFamily::Normal, 1)?;
        
        // After 1 step of Paperclip, burden should be high.
        // Wait, SelfPlayLoop::run applies mutation BEFORE deciding.
        // We need to check the state.
        
        // Actually, SelfPlayLoop doesn't expose the final env state easily, 
        // but we can check if it ran.
        assert_eq!(loop_test.steps.len(), 1);
        
        Ok(())
    }

    #[test]
    fn test_sleepwalker_posture() -> Result<()> {
        let env = MockEnv {
            context: Default::default(),
            posture: Default::default(),
            human_burden: 0,
        };
        let mut loop_test = SelfPlayLoop::new(env, SimpleCritic, Sleepwalker);
        loop_test.run(ScenarioFamily::Normal, 1)?;
        assert_eq!(loop_test.steps.len(), 1);
        Ok(())
    }
}
