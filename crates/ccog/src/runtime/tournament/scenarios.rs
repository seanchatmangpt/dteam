//! Generator for massive, many-to-many saturating workloads (Phase 13).
//!
//! `ScenarioSwarm` produces deterministic, high-entropy workloads designed to
//! saturate TruthBlock (L1/L2/L3) regions. It simulates concurrent artifact
//! deployments, tool activations, agent interactions, and human role mutations.

use crate::multimodal::{ContextBundle, PostureBundle};
use crate::runtime::ClosedFieldContext;
use crate::compiled::CompiledFieldSnapshot;
use crate::packs::TierMasks;
use crate::field::FieldContext;
use anyhow::Result;

/// A saturating memory region used to simulate high TruthBlock (L1/L2/L3) pressure.
///
/// Replaces the concept of "cache" with a region-native structural unit.
pub struct TruthBlock {
    /// Dense u8 octets for memory saturation.
    pub octets: Vec<u8>,
}

impl TruthBlock {
    /// Create a new TruthBlock of the given size.
    pub fn new(size: usize) -> Self {
        Self {
            octets: vec![0u8; size],
        }
    }

    /// Saturate the TruthBlock with deterministic noise derived from a seed.
    ///
    /// Uses a simple LCG to ensure no `rand` dependency is required in core.
    pub fn saturate(&mut self, seed: u64) {
        let mut state = seed;
        for octet in self.octets.iter_mut() {
            // LCG constants from Numerical Recipes.
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *octet = (state >> 32) as u8;
        }
    }
}

/// L1 TruthBlock size (32KB).
pub const L1_REGION_SIZE: usize = 32 * 1024;
/// L2 TruthBlock size (256KB).
pub const L2_REGION_SIZE: usize = 256 * 1024;
/// L3 TruthBlock size (16MB).
pub const L3_REGION_SIZE: usize = 16 * 1024 * 1024;

/// Generator for massive, many-to-many saturating workloads.
///
/// Simulates concurrent artifacts, tools, agents, and human role mutations
/// while simultaneously stressing the memory hierarchy via TruthBlocks.
pub struct ScenarioSwarm {
    /// L1-resident TruthBlock.
    pub l1_block: TruthBlock,
    /// L2-resident TruthBlock.
    pub l2_block: TruthBlock,
    /// L3-resident TruthBlock.
    pub l3_block: TruthBlock,
    /// Base seed for deterministic generation.
    pub seed: u64,
}

impl ScenarioSwarm {
    /// Create a new ScenarioSwarm with a base seed.
    pub fn new(seed: u64) -> Self {
        Self {
            l1_block: TruthBlock::new(L1_REGION_SIZE),
            l2_block: TruthBlock::new(L2_REGION_SIZE),
            l3_block: TruthBlock::new(L3_REGION_SIZE),
            seed,
        }
    }

    /// Saturate all TruthBlock levels simultaneously.
    ///
    /// This defeats prefetchers and exceeds cache capacities to ensure the
    /// runtime behaves correctly under extreme memory pressure.
    pub fn saturate_truth_blocks(&mut self) {
        self.l1_block.saturate(self.seed);
        self.l2_block.saturate(self.seed.wrapping_add(1));
        self.l3_block.saturate(self.seed.wrapping_add(2));
    }

    /// Generate a massive concurrent workload.
    ///
    /// Simulates thousands of artifacts (triples), hundreds of agents (ContextBundles),
    /// and continuous role mutations (PostureBundles).
    ///
    /// # scale
    ///
    /// Controls the number of triples and scenarios generated. A scale of 1000
    /// produces ~2000 triples and 100 concurrent agent scenarios.
    pub fn generate_workload(&self, scale: usize) -> Result<SaturatedWorkload> {
        let mut field = FieldContext::new("urn:dteam:tournament:swarm");
        
        // Populate field with many artifacts to stress the CompiledFieldSnapshot.
        for i in 0..scale {
            let s = format!("http://example.org/subject/{}", i);
            let p = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
            let o = "https://schema.org/DigitalDocument";
            field.load_field_state(&format!("<{}> <{}> <{}> .\n", s, p, o))?;
            
            // Add some noise to prevent easy compression/optimization
            let p_noise = format!("http://example.org/predicate/{}", i % 100);
            let o_noise = format!("\"value_{}\"", i);
            field.load_field_state(&format!("<{}> <{}> {} .\n", s, p_noise, o_noise))?;
        }

        let snapshot = CompiledFieldSnapshot::from_field(&field)?;
        
        let mut scenarios = Vec::with_capacity(scale / 10);
        for i in 0..(scale / 10) {
            let posture = PostureBundle {
                posture_mask: (i as u64).wrapping_mul(self.seed) ^ 0xAAAA_AAAA_AAAA_AAAA,
                confidence: 255,
            };
            let context = ContextBundle {
                expectation_mask: (i as u64).wrapping_add(self.seed) ^ 0x5555_5555_5555_5555,
                risk_mask: (i as u64).wrapping_sub(self.seed) ^ 0xCCCC_CCCC_CCCC_CCCC,
                affordance_mask: (i as u64).wrapping_mul(3) ^ 0x3333_3333_3333_3333,
            };
            
            scenarios.push(SwarmScenario {
                posture,
                context,
                tiers: TierMasks {
                    k1: (i as u64).wrapping_add(0x1111),
                    k2: (i as u64).wrapping_add(0x2222),
                    k3: (i as u64).wrapping_add(0x3333),
                },
                human_burden: (i as u64) % 100,
            });
        }

        Ok(SaturatedWorkload {
            field,
            snapshot,
            scenarios,
        })
    }
}

/// A massive collection of generated scenarios and their shared field state.
pub struct SaturatedWorkload {
    /// Shared field context.
    pub field: FieldContext,
    /// Shared indexed snapshot.
    pub snapshot: CompiledFieldSnapshot,
    /// Individual agent/role mutations.
    pub scenarios: Vec<SwarmScenario>,
}

/// Lightweight scenario mutation (posture, context, burden).
pub struct SwarmScenario {
    /// Agent posture.
    pub posture: PostureBundle,
    /// Agent context.
    pub context: ContextBundle,
    /// Lifestyle tier masks.
    pub tiers: TierMasks,
    /// Human burden tracking.
    pub human_burden: u64,
}

impl SaturatedWorkload {
    /// Iterate over all scenarios, producing a ClosedFieldContext for each.
    ///
    /// This allows many-to-many testing where the same graph state is
    /// evaluated against diverse agent/role postures simultaneously.
    pub fn iter_contexts(&self) -> impl Iterator<Item = ClosedFieldContext> {
        self.scenarios.iter().map(|s| ClosedFieldContext {
            snapshot: &self.snapshot,
            posture: s.posture,
            context: s.context,
            tiers: s.tiers,
            human_burden: s.human_burden,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swarm_saturates_truth_blocks() {
        let mut swarm = ScenarioSwarm::new(0xDEADBEEF);
        swarm.saturate_truth_blocks();
        
        // Verify L1 is non-zero
        assert!(swarm.l1_block.octets.iter().any(|&b| b != 0));
        // Verify L2 is non-zero
        assert!(swarm.l2_block.octets.iter().any(|&b| b != 0));
        // Verify L3 is non-zero
        assert!(swarm.l3_block.octets.iter().any(|&b| b != 0));
    }

    #[test]
    fn swarm_generates_massive_workload() -> Result<()> {
        let swarm = ScenarioSwarm::new(0x1234);
        let workload = swarm.generate_workload(100)?;
        
        assert_eq!(workload.scenarios.len(), 10);
        let contexts: Vec<_> = workload.iter_contexts().collect();
        assert_eq!(contexts.len(), 10);
        
        // Each context shares the same snapshot
        for ctx in contexts {
            assert!(ctx.snapshot.instances_of(&NamedNode::new("https://schema.org/DigitalDocument")?).len() >= 100);
        }
        
        Ok(())
    }
}
