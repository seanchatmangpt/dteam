//! Causal trace artifact for bark dispatch (Phase 5 Track E stub).
//!
//! Stub module — `Track E` writer fills in `BarkNodeTrace` semantics and
//! `decide_with_trace()` integration. Intentionally minimal so other tracks
//! can land without depending on Track E's final shape.

use crate::verdict::PackPosture;

/// Per-node entry in a [`CcogTrace`]. Records why a slot fired or skipped.
#[derive(Clone, Debug, Default)]
pub struct BarkNodeTrace {
    /// Index of the slot in the compiled bark kernel.
    pub slot_idx: u16,
    /// Hook identifier (static name).
    pub hook_id: &'static str,
    /// AND-mask of canonical predicate bits required to fire.
    pub require_mask: u64,
    /// Bitmask of plan predecessors that must be advanced.
    pub predecessor_mask: u64,
    /// True iff the trigger condition was satisfied.
    pub trigger_fired: bool,
    /// True iff the check passed.
    pub check_passed: bool,
    /// Number of triples emitted by this slot's act (0 if it did not fire).
    pub act_emitted_triples: u8,
    /// Deterministic receipt URN if the slot emitted one.
    pub receipt_urn: Option<String>,
    /// Reason the slot was skipped, if applicable.
    pub skip_reason: Option<&'static str>,
}

/// Causal trace of a single bark dispatch — present mask, posture, per-slot detail.
#[derive(Clone, Debug, Default)]
pub struct CcogTrace {
    /// Bitmask of canonical predicates present in the snapshot.
    pub present_mask: u64,
    /// Pack posture observed for this fire.
    pub posture: PackPosture,
    /// Per-slot entries in plan-order.
    pub nodes: Vec<BarkNodeTrace>,
}

impl Default for PackPosture {
    fn default() -> Self {
        PackPosture::Calm
    }
}

/// Tier annotation for benchmarks — declares what the bench actually measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchmarkTier {
    /// `decide()` only — no allocation, no act.
    KernelFloor,
    /// `decide()` + `materialize()` — allocates `Construct8`.
    CompiledBark,
    /// Just the act fns over the snapshot.
    Materialization,
    /// `seal()` — receipt construction.
    ReceiptPath,
    /// `process_with_hooks` — full warm path through HookRegistry.
    FullProcess,
    /// Replay against a prior trace for semantic conformance.
    ConformanceReplay,
}
