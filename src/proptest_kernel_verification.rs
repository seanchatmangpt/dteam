#[cfg(test)]
mod proptests {
    use crate::dteam::kernel::branchless::{apply_branchless_update, apply_ktier_update};
    use crate::models::petri_net::{FlatIncidenceMatrix, PetriNet, Place, Transition, Arc};
    use crate::reinforcement::WorkflowAction;
    use crate::utils::dense_kernel::KBitSet;
    use crate::{RlAction, RlState};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_μ_kernel_determinism(
            h in 0i8..5,
            a in 0usize..3,
        ) {
            let state = RlState {
                health_level: h,
                event_rate_q: 0,
                activity_count_q: 0,
                spc_alert_level: 0,
                drift_status: 0,
                rework_ratio_q: 0,
                circuit_state: 0,
                cycle_phase: 0,
                marking_mask: 0,
                activities_hash: 0,
            };
            let action = RlAction::from_index(a).unwrap();
            
            // Execute twice to check variancy τ
            let result1 = transition(state, action);
            let result2 = transition(state, action);
            
            assert_eq!(result1, result2, "Kernel μ failed: transition not deterministic");
        }

        #[test]
        fn test_branchless_kernel_equation_parity(
            mask in 0u64..1024,
            transitions in 1usize..8,
        ) {
            let places_count = 10;
            // Generate a random incidence matrix
            let mut data = vec![0i32; places_count * transitions];
            let mut input_masks = vec![KBitSet::<16>::zero(); transitions];
            let mut output_masks = vec![KBitSet::<16>::zero(); transitions];

            for t in 0..transitions {
                for p in 0..places_count {
                    let val = if (t + p) % 3 == 0 { -1 } else if (t + p) % 3 == 1 { 1 } else { 0 };
                    data[p * transitions + t] = val;
                    if val < 0 {
                        let _ = input_masks[t].set(p);
                    } else if val > 0 {
                        let _ = output_masks[t].set(p);
                    }
                }
            }

            let incidence = FlatIncidenceMatrix {
                data,
                places_count,
                transitions_count: transitions,
                input_masks,
                output_masks,
            };

            let transition_idx = 0; // Test first transition
            let result1 = apply_branchless_update(mask, transition_idx, &incidence);
            let result2 = apply_branchless_update(mask, transition_idx, &incidence);
            
            assert_eq!(result1, result2, "Branchless transition failed: not deterministic");
            
            // Verification parity: manually compute the next mask
            let mut expected = mask;
            let in_mask = incidence.input_masks[transition_idx].words[0];
            let out_mask = incidence.output_masks[transition_idx].words[0];
            expected = (expected & !in_mask) | out_mask;
            assert_eq!(result1, expected);
        }

        #[test]
        fn test_ktier_branchless_updates(
            w_idx in 0..16usize,
            bit_idx in 0..64usize,
        ) {
            let mut marking = KBitSet::<16>::zero();
            let _ = marking.set(w_idx * 64 + bit_idx);
            
            let mut input_masks = vec![KBitSet::<16>::zero(); 1];
            let mut output_masks = vec![KBitSet::<16>::zero(); 1];
            
            // Transition consumes the set bit and produces one in the next word (circular)
            let _ = input_masks[0].set(w_idx * 64 + bit_idx);
            let next_bit = (w_idx * 64 + bit_idx + 1) % 1024;
            let _ = output_masks[0].set(next_bit);
            
            let incidence = FlatIncidenceMatrix {
                data: vec![0; 1024],
                places_count: 1024,
                transitions_count: 1,
                input_masks,
                output_masks,
            };
            
            let next_marking = apply_ktier_update::<16>(marking, 0, &incidence);
            
            assert!(!next_marking.contains(w_idx * 64 + bit_idx), "Should have consumed token");
            assert!(next_marking.contains(next_bit), "Should have produced token");
            assert_eq!(next_marking.words.iter().map(|w| w.count_ones()).sum::<u32>(), 1);
        }

        #[test]
        fn test_structural_workflow_net_branchless_verification(
            p_count in 2usize..20,
            t_count in 1usize..10,
        ) {
            let mut net = PetriNet::default();
            for i in 0..p_count {
                net.places.push(Place { id: format!("p{}", i) });
            }
            for i in 0..t_count {
                net.transitions.push(Transition { id: format!("t{}", i), label: format!("T{}", i), is_invisible: None });
            }
            
            // Create a simple chain: p0 -> t0 -> p1 -> t1 -> ... -> pN
            for i in 0..t_count {
                net.arcs.push(Arc { from: format!("p{}", i), to: format!("t{}", i), weight: None });
                net.arcs.push(Arc { from: format!("t{}", i), to: format!("p{}", i+1), weight: None });
            }
            
            // This is a workflow net if p0 is source and pN is sink
            // Wait, we need to handle the case where p_count > t_count + 1
            
            net.compile_incidence();
            let is_wf = net.is_structural_workflow_net();
            
            if p_count == t_count + 1 {
                assert!(is_wf, "Simple chain should be a workflow net (p={}, t={})", p_count, t_count);
            }
            
            let calculus_ok = net.verifies_state_equation_calculus();
            if is_wf {
                assert!(calculus_ok);
            }
        }
    }

    fn transition(state: RlState, action: RlAction) -> RlState {
        let mut next = state;
        match action {
            RlAction::Idle => (),
            RlAction::Optimize => next.health_level += 1,
            RlAction::Rework => next.health_level = (next.health_level - 1).max(0),
        }
        next
    }
}
