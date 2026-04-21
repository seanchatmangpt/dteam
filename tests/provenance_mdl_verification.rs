use dteam::dteam::orchestration::{Engine, EngineBuilder, ExecutionManifest};
use dteam::models::{Event, EventLog, Trace};
use dteam::models::petri_net::{PetriNet, Place, Transition, Arc};

#[test]
fn test_provenance_manifest_compliance() {
    let engine = EngineBuilder::new().with_k_tier(256).build();
    let mut log = EventLog::default();
    let mut trace = Trace::default();
    trace.events.push(Event::new("A".to_string()));
    trace.events.push(Event::new("B".to_string()));
    log.add_trace(trace);

    let result = engine.run(&log);
    
    if let dteam::dteam::orchestration::EngineResult::Success(_, manifest) = result {
        // Assert manifest compliance M = {H(L), π, H(N)}
        assert!(manifest.input_log_hash != 0, "Input log hash missing");
        assert!(!manifest.action_sequence.is_empty(), "Action sequence (π) missing");
        assert!(manifest.model_canonical_hash != 0, "Model canonical hash missing");
        assert!(manifest.mdl_score > 0.0, "MDL score missing or zero");
        assert_eq!(manifest.k_tier, "K256", "K-tier mismatch in manifest");
    } else {
        panic!("Engine run failed");
    }
}

#[test]
fn test_mdl_minimality_formula() {
    let mut net = PetriNet::default();
    // T = 2 transitions
    net.transitions.push(Transition { id: "t1".into(), label: "A".into(), is_invisible: None });
    net.transitions.push(Transition { id: "t2".into(), label: "B".into(), is_invisible: None });
    // A = 1 arc
    net.arcs.push(Arc { from: "t1".into(), to: "p1".into(), weight: None });
    
    // Φ(N) = |T| + (|A| * log2 |T|)
    // Φ(N) = 2 + (1 * log2 2) = 2 + 1 = 3
    let score = net.mdl_score();
    assert_eq!(score, 3.0, "MDL score calculation mismatch");
    
    // T = 4, A = 2
    // Φ(N) = 4 + (2 * log2 4) = 4 + (2 * 2) = 8
    net.transitions.push(Transition { id: "t3".into(), label: "C".into(), is_invisible: None });
    net.transitions.push(Transition { id: "t4".into(), label: "D".into(), is_invisible: None });
    net.arcs.push(Arc { from: "t2".into(), to: "p2".into(), weight: None });
    
    let score = net.mdl_score();
    assert_eq!(score, 8.0, "MDL score calculation mismatch (extended)");
}
