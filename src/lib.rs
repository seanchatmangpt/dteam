pub mod models;
pub mod conformance;
pub mod io;
pub mod utils;
pub mod reinforcement;
pub mod reinforcement_tests;

// Re-export models for easier access
pub use models::*;
pub use conformance::*;

// Zero-heap, stack-allocated RL state for nanosecond-scale updates.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct RlState {
    pub health_level: i8,
    pub event_rate_q: i8,
    pub activity_count_q: i8,
    pub spc_alert_level: i8,
    pub drift_status: i8,
    pub rework_ratio_q: i8,
    pub circuit_state: i8,
    pub cycle_phase: i8,
    pub marking_mask: u64,      // BCINR bitset mask for Petri net marking
    pub activities_hash: u64,   // Rolling FNV-1a hash of recent activities
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum RlAction {
    Idle,
    Optimize,
    Rework,
}

impl reinforcement::WorkflowAction for RlAction {
    const ACTION_COUNT: usize = 3;
    fn to_index(&self) -> usize {
        match self {
            RlAction::Idle => 0,
            RlAction::Optimize => 1,
            RlAction::Rework => 2,
        }
    }
    fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(RlAction::Idle),
            1 => Some(RlAction::Optimize),
            2 => Some(RlAction::Rework),
            _ => None,
        }
    }
}

// Minimal RlState impls for reinforcement trait
impl reinforcement::WorkflowState for RlState {
    fn features(&self) -> Vec<f32> { 
        // Optimized feature vector: only allocate if necessary for function approx.
        // For Q-Table, this is rarely called in the hot path.
        vec![self.health_level as f32, self.marking_mask as f32] 
    }
    fn is_terminal(&self) -> bool { self.health_level < 0 || self.health_level >= 5 }
}

pub mod rl_state_serialization {
    use std::collections::HashMap;
    pub struct SerializedAgentQTable {
        pub agent_type: u8,
        pub state_values: HashMap<i64, Vec<f32>>,
    }
    pub fn encode_rl_state_key(h: i8, _e: i8, _a: i8, _s: i8, _d: i8, _r: i8, _c: i8, _p: i8) -> i64 { h as i64 }
    pub fn decode_rl_state_key(key: i64) -> (i8, i8, i8, i8, i8, i8, i8, i8) { (key as i8,0,0,0,0,0,0,0) }
}
pub mod automation;
pub mod benchmark;
pub mod config;
pub mod skeptic_harness;
pub mod skeptic_contract;
pub mod ref_models {
    pub mod ref_petri_net;
    pub mod ref_event_log;
}
pub mod ref_conformance {
    pub mod ref_token_replay;
}
