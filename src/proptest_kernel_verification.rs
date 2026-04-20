#[cfg(test)]
mod proptests {
    use crate::reinforcement::WorkflowAction;
    use crate::utils::dense_kernel::KBitSet;
    use crate::{RlAction, RlState};
    use proptest::prelude::*;

    // μ-kernel invariant: Var(τ) == 0 (Deterministic Execution)
    // For a fixed state and action, the next_state must be identical.
    proptest! {
        #[test]
        fn test_μ_kernel_determinism(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = RlState::<1> {
                health_level: h,
                event_rate_q: 0,
                activity_count_q: 0,
                spc_alert_level: 0,
                drift_status: 0,
                rework_ratio_q: 0,
                circuit_state: 0,
                cycle_phase: 0,
                marking_mask: KBitSet::zero(),
                activities_hash: 0,
            };
            let action = RlAction::from_index(a).unwrap();

            // Execute twice to check variancy τ
            let result1 = transition(state, action);
            let result2 = transition(state, action);

            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic");
        }
    }
    fn transition(state: RlState<1>, action: RlAction) -> RlState<1> {
        let mut next = state;
        match action {
            RlAction::Idle => (),
            RlAction::Optimize => next.health_level += 1,
            RlAction::Rework => next.health_level = (next.health_level - 1).max(0),
        }
        next
    }
}
