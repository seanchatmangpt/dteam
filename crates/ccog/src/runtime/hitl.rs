//! Human-In-The-Loop (HITL) Ecology (PRD v0.8).
//!
//! Provides the HumanRoleProfile for authority and burden tracking,
//! and the LeastCostHandler for routing tasks to operators.

use crate::ids::HumanRoleId;
use crate::powl64::ProjectionTarget;

/// Human role profile for authority and burden tracking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HumanRoleProfile {
    /// Unique identifier for this role.
    pub id: HumanRoleId,
    /// Authority level (higher = more capable/permitted).
    pub authority: u32,
    /// Current burden (cumulative task load).
    pub current_burden: u64,
    /// Last tick when burden decay was applied.
    pub last_decay_tick: u64,
    /// Historical reliability score [0.0, 1.0].
    pub reliability: f64,
}

impl HumanRoleProfile {
    /// Creates a new profile.
    pub fn new(id: HumanRoleId, authority: u32, reliability: f64) -> Self {
        Self {
            id,
            authority,
            current_burden: 0,
            last_decay_tick: 0,
            reliability,
        }
    }

    /// Applies burden decay based on elapsed time (ticks).
    ///
    /// Burden decreases over time, representing task completion or rest.
    pub fn decay_burden(&mut self, current_tick: u64, decay_rate: u64) {
        if current_tick > self.last_decay_tick {
            let elapsed = current_tick - self.last_decay_tick;
            let decay_amount = elapsed * decay_rate;
            self.current_burden = self.current_burden.saturating_sub(decay_amount);
            self.last_decay_tick = current_tick;
        }
    }

    /// Adds burden to the profile.
    pub fn add_burden(&mut self, amount: u64) {
        self.current_burden = self.current_burden.saturating_add(amount);
    }
}

/// Principle: Least-External-Burden.
/// Penalizes routes that externalize work (MCP/A2A/HITL) when a local closure route was admissible.
pub struct ExternalBurden;

impl ExternalBurden {
    /// Returns the cost penalty for a given projection target.
    pub fn cost(target: ProjectionTarget) -> u64 {
        match target {
            ProjectionTarget::NoOp => 0,
            ProjectionTarget::Mcp => 500,
            ProjectionTarget::A2a => 1000,
            ProjectionTarget::Hitl => 5000,
        }
    }

    /// Penalizes a route based on its external projection.
    pub fn penalize_externalization(target: ProjectionTarget, current_score: f64) -> f64 {
        current_score + (Self::cost(target) as f64)
    }
}

/// Selector that routes tasks to the human with the lowest burden and highest reliability.
pub struct LeastCostHandler;

impl LeastCostHandler {
    /// Selects the best role from a slice of profiles.
    ///
    /// The 'cost' is calculated as burden divided by reliability.
    /// Lower cost is better.
    pub fn select(profiles: &[HumanRoleProfile]) -> Option<&HumanRoleProfile> {
        if profiles.is_empty() {
            return None;
        }

        profiles.iter().min_by(|a, b| {
            // Lower burden and higher reliability = lower cost.
            // We use a small epsilon to avoid division by zero.
            let cost_a = a.current_burden as f64 / (a.reliability + 0.001);
            let cost_b = b.current_burden as f64 / (b.reliability + 0.001);

            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}
