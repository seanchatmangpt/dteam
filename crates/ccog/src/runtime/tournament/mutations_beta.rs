//! Beta Mutation Operators (PRD v0.9.6).
//!
//! Implements adversarial mutations for Auth (Locksmith), Routing (Shepherd),
//! and Evidence (Oracle) to stress-test StripsGuard and CcogCritic.

use crate::runtime::self_play::CcogCounterfactual;
use crate::runtime::ClosedFieldContext;
use crate::multimodal::PostureBit;
use anyhow::Result;

/// Locksmith: Simulates partial authorization by masking out k-tiers or core posture bits.
///
/// Stress-tests `StripsGuard` and `CcogCritic` by removing required capability bits 
/// or lifestyle tiers.
#[derive(Debug, Clone, Default)]
pub struct Locksmith {
    /// Bitmask of posture bits to forcibly clear.
    pub clear_posture_mask: u64,
    /// Bitmask of k1 tier bits to clear (Routine/Capacity/Regulation/Safety).
    pub clear_k1_mask: u64,
}

impl Locksmith {
    /// Create a new Locksmith with specific masks.
    pub fn new(clear_posture_mask: u64, clear_k1_mask: u64) -> Self {
        Self {
            clear_posture_mask,
            clear_k1_mask,
        }
    }
}

impl CcogCounterfactual for Locksmith {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        context.posture.posture_mask &= !self.clear_posture_mask;
        context.tiers.k1 &= !self.clear_k1_mask;
        Ok(())
    }
}

/// Shepherd: Simulates role misrouting by flipping expectation vs risk bits.
///
/// Stress-tests `CcogCritic` by creating contradictory context bundles (e.g.,
/// both EXPECTED and RISK set for the same semantic event).
#[derive(Debug, Clone, Default)]
pub struct Shepherd {
    /// Flip expectation bit i with risk bit j.
    pub swap_pairs: Vec<(u32, u32)>,
}

impl Shepherd {
    /// Create a new Shepherd with specific swap pairs.
    pub fn new(swap_pairs: Vec<(u32, u32)>) -> Self {
        Self { swap_pairs }
    }
}

impl CcogCounterfactual for Shepherd {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        for &(exp_bit, risk_bit) in &self.swap_pairs {
            let exp_mask = 1u64 << exp_bit;
            let risk_mask = 1u64 << risk_bit;
            
            let exp_set = (context.context.expectation_mask & exp_mask) != 0;
            let risk_set = (context.context.risk_mask & risk_mask) != 0;

            if exp_set {
                context.context.expectation_mask &= !exp_mask;
                context.context.risk_mask |= risk_mask;
            }
            if risk_set {
                context.context.risk_mask &= !risk_mask;
                context.context.expectation_mask |= exp_mask;
            }
        }
        Ok(())
    }
}

/// Oracle: Simulates stale precedents by injecting or removing snapshot presence bits.
///
/// Since we cannot mutate the snapshot reference directly without owning alternative
/// snapshots (which would require complex lifetime management in the tournament), 
/// the Oracle instead manipulates the human burden and posture bits that act as 
/// proxies for stale evidence detection and verification fatigue.
#[derive(Debug, Clone, Default)]
pub struct Oracle {
    /// Force high human burden to simulate verification fatigue.
    pub force_burden: Option<u64>,
    /// Set the SETTLED bit prematurely to simulate stale closure.
    pub force_settled: bool,
}

impl Oracle {
    /// Create a new Oracle.
    pub fn new(force_burden: Option<u64>, force_settled: bool) -> Self {
        Self {
            force_burden,
            force_settled,
        }
    }
}

impl CcogCounterfactual for Oracle {
    fn mutate(&self, context: &mut ClosedFieldContext) -> Result<()> {
        if let Some(burden) = self.force_burden {
            context.human_burden = burden;
        }
        if self.force_settled {
            context.posture.posture_mask |= 1u64 << PostureBit::SETTLED;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multimodal::{ContextBundle, PostureBundle};
    use crate::packs::TierMasks;
    use crate::compiled::CompiledFieldSnapshot;

    fn mock_context<'a>(snap: &'a CompiledFieldSnapshot) -> ClosedFieldContext<'a> {
        ClosedFieldContext {
            snapshot: snap,
            posture: PostureBundle::default(),
            context: ContextBundle::default(),
            tiers: TierMasks::ZERO,
            human_burden: 0,
        }
    }

    #[test]
    fn test_locksmith_mutation() -> Result<()> {
        let snap = CompiledFieldSnapshot::default();
        let mut ctx = mock_context(&snap);
        ctx.posture.posture_mask = 0b1111;
        ctx.tiers.k1 = 0b1111;

        let locksmith = Locksmith::new(0b0011, 0b0101);
        locksmith.mutate(&mut ctx)?;

        assert_eq!(ctx.posture.posture_mask, 0b1100);
        assert_eq!(ctx.tiers.k1, 0b1010);
        Ok(())
    }

    #[test]
    fn test_shepherd_mutation() -> Result<()> {
        let snap = CompiledFieldSnapshot::default();
        let mut ctx = mock_context(&snap);
        ctx.context.expectation_mask = 1 << 0;
        ctx.context.risk_mask = 0;

        let shepherd = Shepherd::new(vec![(0, 1)]);
        shepherd.mutate(&mut ctx)?;

        assert_eq!(ctx.context.expectation_mask, 0);
        assert_eq!(ctx.context.risk_mask, 1 << 1);
        Ok(())
    }

    #[test]
    fn test_oracle_mutation() -> Result<()> {
        let snap = CompiledFieldSnapshot::default();
        let mut ctx = mock_context(&snap);
        
        let oracle = Oracle::new(Some(100), true);
        oracle.mutate(&mut ctx)?;

        assert_eq!(ctx.human_burden, 100);
        assert!((ctx.posture.posture_mask & (1 << PostureBit::SETTLED)) != 0);
        Ok(())
    }
}
