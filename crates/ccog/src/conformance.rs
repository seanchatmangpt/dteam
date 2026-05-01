//! Conformance replay (Phase 7).
//!
//! Re-runs [`crate::trace::decide_with_trace_table`] against a snapshot and
//! a table, then diffs the produced [`crate::trace::CcogTrace`] against a
//! prior trace. Emits a [`ReplayReport`] that names the first divergent
//! slot and its skip reasons. Replay is the contract that `decide_with_trace`
//! is reproducible — same snapshot, same table, same decision, same
//! per-slot reasoning.

use crate::bark_artifact::BarkSlot;
use crate::ids::CollapseFn;
use crate::runtime::ClosedFieldContext;
use crate::trace::{decide_with_trace_table, BarkSkipReason, CcogTrace};

/// Result of [`replay_trace`] — pinpoints the first divergence, if any.
///
/// `decision_eq` is `true` iff every slot's `(trigger_fired, check_passed,
/// skip)` triple matches between the original trace and the replay. On the
/// first divergence, `diverged_slot` is set and `original_skip` /
/// `replay_skip` capture the typed reasons.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplayReport {
    /// True iff every per-slot record is identical.
    pub decision_eq: bool,
    /// True iff the POWL64 route proof is identical.
    pub route_conformance: bool,
    /// Slot index where the first divergence occurred, if any.
    pub diverged_slot: Option<u16>,
    /// Skip reason in the original trace at the divergence point.
    pub original_skip: Option<BarkSkipReason>,
    /// Skip reason in the replay trace at the divergence point.
    pub replay_skip: Option<BarkSkipReason>,
    /// Collapse function in the original trace at divergence.
    pub original_collapse: Option<CollapseFn>,
    /// Collapse function in the replay trace at divergence.
    pub replay_collapse: Option<CollapseFn>,
}

/// Replay a trace against a snapshot and table.
///
/// Re-invokes [`decide_with_trace_table`] and walks both traces in
/// lock-step. The first slot whose `(trigger_fired, check_passed, skip)`
/// differs is reported. If both traces agree on every slot,
/// `decision_eq = true` and `diverged_slot = None`.
pub fn replay_trace(
    trace: &CcogTrace,
    context: &ClosedFieldContext,
    table: &'static [BarkSlot],
) -> ReplayReport {
    let (_decision, replay) = decide_with_trace_table(context, table);

    let route_conformance = trace.route_proof == replay.route_proof;

    if trace.nodes.len() != replay.nodes.len() {
        let idx = trace.nodes.len().min(replay.nodes.len()) as u16;
        return ReplayReport {
            decision_eq: false,
            route_conformance,
            diverged_slot: Some(idx),
            original_skip: trace.nodes.get(idx as usize).and_then(|n| n.skip),
            replay_skip: replay.nodes.get(idx as usize).and_then(|n| n.skip),
            original_collapse: trace.nodes.get(idx as usize).map(|n| n.collapse_fn),
            replay_collapse: replay.nodes.get(idx as usize).map(|n| n.collapse_fn),
        };
    }

    for (i, (a, b)) in trace.nodes.iter().zip(replay.nodes.iter()).enumerate() {
        if a.trigger_fired != b.trigger_fired
            || a.check_passed != b.check_passed
            || a.skip != b.skip
            || a.collapse_fn != b.collapse_fn
            || a.selected_node != b.selected_node
        {
            return ReplayReport {
                decision_eq: false,
                route_conformance,
                diverged_slot: Some(i as u16),
                original_skip: a.skip,
                replay_skip: b.skip,
                original_collapse: Some(a.collapse_fn),
                replay_collapse: Some(b.collapse_fn),
            };
        }
    }

    ReplayReport {
        decision_eq: true,
        route_conformance,
        diverged_slot: None,
        original_skip: None,
        replay_skip: None,
        original_collapse: None,
        replay_collapse: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bark_artifact::BUILTINS;
    use crate::compiled::CompiledFieldSnapshot;
    use crate::field::FieldContext;
    use crate::multimodal::{ContextBundle, PostureBundle};
    use crate::packs::TierMasks;
    use crate::trace::decide_with_trace;

    fn empty_context(snap: std::sync::Arc<CompiledFieldSnapshot>) -> ClosedFieldContext {
        ClosedFieldContext {
            snapshot: snap,
            posture: PostureBundle::default(),
            context: ContextBundle::default(),
            tiers: TierMasks::ZERO,
            human_burden: 0,
        }
    }

    #[test]
    fn replay_matches_self_on_empty_field() {
        let field = FieldContext::new("test");
        let snap = std::sync::Arc::new(CompiledFieldSnapshot::from_field(&field).unwrap());
        let context = empty_context(snap);
        let (_d, trace) = decide_with_trace(&context);
        let report = replay_trace(&trace, &context, BUILTINS);
        assert!(report.decision_eq);
        assert!(report.diverged_slot.is_none());
    }

    #[test]
    fn replay_matches_self_on_loaded_field() {
        let mut field = FieldContext::new("test");
        field
            .load_field_state(
                "<http://example.org/d1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://schema.org/DigitalDocument> .\n",
            )
            .unwrap();
        let snap = std::sync::Arc::new(CompiledFieldSnapshot::from_field(&field).unwrap());
        let context = empty_context(snap);
        let (_d, trace) = decide_with_trace(&context);
        let report = replay_trace(&trace, &context, BUILTINS);
        assert!(report.decision_eq, "{:?}", report);
    }
}
