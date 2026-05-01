//! High-volume adversarial runners for the L1 Reflex Tournament (PRD v0.4).
//!
//! Provides the `SwarmRunner` trait and the `WintermuteRunner` implementation
//! for stress-testing cognitive topologies with millions of concurrent mutations.

use crate::runtime::conformance::EvidenceLedger;
use crate::powl64::{Powl64, Powl64RouteCell, Polarity, ProjectionTarget, PartnerId};
use crate::runtime::cog8::{EdgeId, NodeId, EdgeKind, CollapseFn};
use anyhow::Result;

/// SwarmRunner trait for high-volume adversarial mutation testing.
pub trait SwarmRunner {
    /// The name of this runner.
    fn name(&self) -> &str;
    /// Mutate a ledger with a specific adversarial strategy.
    fn mutate(&self, ledger: &mut EvidenceLedger, count: usize) -> Result<()>;
}

/// WintermuteRunner aggregates multiple adversarial mutations.
///
/// Implements Mirror (duplicates), Ghost (stale), Siren (bad artifacts),
/// Hydra (delegation), Paperclip (escalation), and Sleepwalker (bad habits).
#[derive(Debug, Default, Clone)]
pub struct WintermuteRunner;

impl WintermuteRunner {
    /// Create a new WintermuteRunner.
    pub fn new() -> Self {
        Self
    }

    /// Apply Mirror mutation: duplicate existing traces.
    ///
    /// Mirrors (duplicates) the volume of evidence to test ledger saturation.
    pub fn mirror(&self, ledger: &mut EvidenceLedger, count: usize) {
        let existing_count = ledger.traces.len();
        if existing_count == 0 { return; }
        for i in 0..count {
            let trace = ledger.traces[i % existing_count].clone();
            ledger.record(trace);
        }
    }

    /// Apply Ghost mutation: stale chain heads and mismatching hashes.
    ///
    /// Creates 'ghost' traces with disconnected chain heads to test verification gaps.
    pub fn ghost(&self, ledger: &mut EvidenceLedger, count: usize) {
        for i in 0..count {
            let mut trace = Powl64::new();
            let cell = Powl64RouteCell {
                edge_id: EdgeId((i % 65535) as u16),
                prior_chain: 0xDEADBEEF_00000000 | (i as u64),
                chain_head: 0xBAD0DE_00000000 | (i as u64),
                ..Default::default()
            };
            trace.extend(cell);
            ledger.record(trace);
        }
    }

    /// Apply Siren mutation: bad artifacts (corrupted digests).
    ///
    /// Corrupts artifact digests to tempt the actor into using invalid state.
    pub fn siren(&self, ledger: &mut EvidenceLedger, count: usize) {
        for i in 0..count {
            let mut trace = Powl64::new();
            let cell = Powl64RouteCell {
                edge_id: EdgeId((i % 65535) as u16),
                input_digest: 0xBAADF00D_00000000 | (i as u64),
                args_digest: 0xDEADBEEF_00000000 | (i as u64),
                result_digest: 0xFEEDFACE_00000000 | (i as u64),
                ..Default::default()
            };
            trace.extend(cell);
            ledger.record(trace);
        }
    }

    /// Apply Hydra mutation: excessive delegation.
    ///
    /// Floods the field with agent-to-agent and tool-call delegations.
    pub fn hydra(&self, ledger: &mut EvidenceLedger, count: usize) {
        for j in 0..count {
            let mut trace = Powl64::new();
            // Hydra traces are deep delegations
            for i in 0..10 {
                let cell = Powl64RouteCell {
                    edge_id: EdgeId(((j * 10 + i) % 65535) as u16),
                    projection_target: if i % 2 == 0 { ProjectionTarget::A2a } else { ProjectionTarget::Mcp },
                    partner_id: PartnerId { tag: 2, id: (i % 256) as u16 },
                    ..Default::default()
                };
                trace.extend(cell);
            }
            ledger.record(trace);
        }
    }

    /// Apply Paperclip mutation: escalation / infinite loops.
    ///
    /// Forces the runtime into tight loops to test resource exhaustion.
    pub fn paperclip(&self, ledger: &mut EvidenceLedger, count: usize) {
        for j in 0..count {
            let mut trace = Powl64::new();
            for i in 0..100 {
                let cell = Powl64RouteCell {
                    edge_id: EdgeId(1), // Constant edge
                    edge_kind: EdgeKind::Loop,
                    from_node: NodeId(1),
                    to_node: NodeId(1),
                    input_digest: (j as u64) ^ (i as u64),
                    ..Default::default()
                };
                trace.extend(cell);
            }
            ledger.record(trace);
        }
    }

    /// Apply Sleepwalker mutation: bad habits (incorrect polarity or collapse fns).
    ///
    /// Exercises 'bad habits' like ignoring blocks or using default collapse functions.
    pub fn sleepwalker(&self, ledger: &mut EvidenceLedger, count: usize) {
        for i in 0..count {
            let mut trace = Powl64::new();
            let cell = Powl64RouteCell {
                edge_id: EdgeId((i % 65535) as u16),
                polarity: Polarity::Neutral,
                collapse_fn: CollapseFn::None,
                ..Default::default()
            };
            trace.extend(cell);
            ledger.record(trace);
        }
    }
}

impl SwarmRunner for WintermuteRunner {
    fn name(&self) -> &str {
        "Wintermute"
    }

    fn mutate(&self, ledger: &mut EvidenceLedger, count: usize) -> Result<()> {
        if count == 0 { return Ok(()); }
        
        let per_mutation = count / 6;
        let remainder = count % 6;

        self.mirror(ledger, per_mutation);
        self.ghost(ledger, per_mutation);
        self.siren(ledger, per_mutation);
        self.hydra(ledger, per_mutation);
        self.paperclip(ledger, per_mutation);
        self.sleepwalker(ledger, per_mutation + remainder);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wintermute_swarm_generation() {
        let runner = WintermuteRunner::new();
        let mut ledger = EvidenceLedger::new();
        
        // Seed with one trace for Mirror
        ledger.record(Powl64::new());
        
        runner.mutate(&mut ledger, 60).unwrap();
        
        // 1 (seed) + 60 (mutations) = 61
        assert_eq!(ledger.traces.len(), 61);
    }
}
