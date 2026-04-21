#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::dteam::orchestration::{Engine, EngineResult};
    use crate::models::{EventLog, Trace, Event, Ontology};

    proptest! {
        #[test]
        fn test_ontology_noise_invariance(
            mut ontology_acts in prop::collection::vec("[a-z]{1,5}", 1..5),
            noise_acts in prop::collection::vec("[A-Z]{1,5}", 1..5),
            trace_len in 1..10usize
        ) {
            // Deduplicate ontology acts to avoid DuplicateSymbol error
            ontology_acts.sort();
            ontology_acts.dedup();
            let ontology = Ontology::new(ontology_acts.clone());
            
            // 1. Create a base sequence of ontology activities
            let mut base_sequence = Vec::new();
            for _ in 0..trace_len {
                let act = &ontology_acts[fastrand::usize(..ontology_acts.len())];
                base_sequence.push(act.clone());
            }

            // 2. Create a log with only ontology activities
            let mut log_pure = EventLog::new();
            let mut trace_pure = Trace::new("pure".to_string());
            for act in &base_sequence {
                trace_pure.events.push(Event::new(act.clone()));
            }
            log_pure.add_trace(trace_pure);

            // 3. Create a log with noise (out-of-ontology activities)
            let mut log_noisy = EventLog::new();
            let mut trace_noisy = Trace::new("noisy".to_string());
            for act in &base_sequence {
                trace_noisy.events.push(Event::new(act.clone()));
                
                // Inject noise
                let noise = &noise_acts[fastrand::usize(..noise_acts.len())];
                trace_noisy.events.push(Event::new(noise.clone()));
            }
            log_noisy.add_trace(trace_noisy);

            // 4. Setup Engine with Pruning
            let engine = crate::dteam::orchestration::Engine::builder()
                .with_k_tier(64)
                .with_ontology(ontology.clone())
                .with_pruning(true)
                .build();

            // 5. Run engine on both. Results should be invariant (AC 6.2).
            let res_pure = engine.run(&log_pure);
            let res_noisy = engine.run(&log_noisy);

            if let (EngineResult::Success(net_p, man_p), EngineResult::Success(net_n, man_n)) = (res_pure, res_noisy) {
                // The models should be identical because noise was pruned
                prop_assert_eq!(net_p.canonical_hash(), net_n.canonical_hash());
                // The manifest should record the violations
                prop_assert_eq!(man_p.violation_count, 0);
                prop_assert_eq!(man_n.violation_count, trace_len);
                // Closure must be verified
                prop_assert!(man_p.closure_verified);
                prop_assert!(man_n.closure_verified);
            } else {
                panic!("Engine failed unexpectedly");
            }
        }

        #[test]
        fn test_strict_boundary_violation(
            mut ontology_acts in prop::collection::vec("[a-z]{1,5}", 1..5),
            noise_act in "[A-Z]{1,5}"
        ) {
            ontology_acts.sort();
            ontology_acts.dedup();
            let ontology = Ontology::new(ontology_acts.clone());
            
            let mut log_noisy = EventLog::new();
            let mut trace_noisy = Trace::new("noisy".to_string());
            trace_noisy.events.push(Event::new(noise_act.clone()));
            log_noisy.add_trace(trace_noisy);

            // Setup Engine without Pruning (Strict Mode)
            let engine = crate::dteam::orchestration::Engine::builder()
                .with_k_tier(64)
                .with_ontology(ontology.clone())
                .with_pruning(false)
                .build();

            let res = engine.run(&log_noisy);
            if let EngineResult::BoundaryViolation { activity } = res {
                prop_assert_eq!(activity, noise_act);
            } else {
                panic!("Engine should have failed with BoundaryViolation");
            }
        }
    }
}
