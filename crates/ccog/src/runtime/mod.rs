//! Closed-loop autonomic runtime: ΔO detection, hook scheduling, posture, step orchestration.

pub mod delta;
pub mod event;
pub mod scheduler;
pub mod posture;
pub mod step;
pub mod cog8;
pub mod soundness;
pub mod mcp;
pub mod mcp_guard;
pub mod mcp_result;
pub mod mcp_transport;
pub mod a2a;
pub mod a2a_guard;
pub mod reentry;
pub mod self_play;
pub mod self_play_advanced;
pub mod self_play_adversarial;
pub mod tournament;
pub mod hitl;
pub mod conformance;
pub mod alignment;
pub mod mining;
pub mod ocel;
pub mod xes;
pub mod coevo;
/// Runtime error types and diagnostic reporting.
pub mod error;

use crate::compiled::CompiledFieldSnapshot;
use crate::multimodal::{ContextBundle, PostureBundle};
use crate::packs::TierMasks;
pub use cog8::{Cog8Edge, Cog8Row, LoadedFieldPack};
pub use event::{CaseId, Event, Lifecycle};
pub use xes::XesLog;
pub use mcp::MCPProjectionTable;
pub use error::{RuntimeError, Result};
use std::sync::Arc;

/// Formalized L2 Field Snapshot (Closed Field Context).
///
/// Replaces the fragmented (CompiledFieldSnapshot, PostureBundle, ContextBundle, TierMasks)
/// tuples used in the decision loop.
#[derive(Debug, Clone)]
pub struct ClosedFieldContext {
    /// Hot-path indexed graph snapshot.
    pub snapshot: Arc<CompiledFieldSnapshot>,
    /// Multimodal posture from the interpreter.
    pub posture: PostureBundle,
    /// Canonical context (expectation, risk, affordance).
    pub context: ContextBundle,
    /// Phase 7 K-tier masks (Lifestyle fields).
    pub tiers: TierMasks,
    /// Human burden tracking state (PRD v0.8).
    pub human_burden: u64,
}

/// Formalized L3 Config Loader target (Compiled Ccog Config).
///
/// Contains the admitted compiled cognition configuration as a nonlinear
/// graph of COG8 closures.
#[derive(Debug, Clone)]
pub struct CompiledCcogConfig<const N: usize, const E: usize> {
    /// Admitted field pack logic.
    pub pack: LoadedFieldPack,
    /// COG8 closure nodes (L1 decide target).
    pub nodes: [Cog8Row; N],
    /// POWL topology edges (L1 routing target).
    pub edges: [Cog8Edge; E],
    /// MCP projection table.
    pub mcp_projections: MCPProjectionTable,
}
