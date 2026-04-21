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
        fn test_μ_kernel_determinism_k64(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<1>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K64)");
        }

        #[test]
        fn test_μ_kernel_determinism_k128(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<2>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K128)");
        }

        #[test]
        fn test_μ_kernel_determinism_k256(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<4>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K256)");
        }

        #[test]
        fn test_μ_kernel_determinism_k512(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<8>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K512)");
        }

        #[test]
        fn test_μ_kernel_determinism_k1024(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<16>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K1024)");
        }

        #[test]
        fn test_μ_kernel_determinism_k2048(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<32>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K2048)");
        }

        #[test]
        fn test_μ_kernel_determinism_k4096(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = create_test_state::<64>(h);
            let action = RlAction::from_index(a).unwrap();
            let result1 = state.step(action);
            let result2 = state.step(action);
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic (K4096)");
        }
    }

    fn create_test_state<const W: usize>(h: i8) -> RlState<W> {
        RlState::<W> {
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
        }
    }
}
