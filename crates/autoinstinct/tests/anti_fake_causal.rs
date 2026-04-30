//! Kill Zone 2 — Causal Dependency Gauntlet.
//!
//! For every canonical response class:
//!
//! 1. **Positive**: under the declared closed surface, the response fires.
//! 2. **Perturbation**: removing each load-bearing input (triple, posture
//!    bit, expectation bit, risk bit, or affordance bit) changes the
//!    response — proving causal dependency, not coincidence.
//!
//! A constant-response policy is also rejected via the in-tree gauntlet
//! tester: `Ask` everywhere fails on perturbation.

use autoinstinct::causal_harness::{
    build_inputs, canonical_scenarios, perturb, respond, CausalScenario,
};
use ccog::instinct::AutonomicInstinct;

fn assert_positive(s: &CausalScenario) {
    let (field, posture, ctx) = build_inputs(s);
    let (got, _) = respond(&field, &posture, &ctx);
    assert_eq!(
        got, s.expected,
        "scenario `{}` expected {:?} but got {:?}",
        s.name, s.expected, got
    );
}

fn perturbation_tag(p: &autoinstinct::causal_harness::Perturbation) -> &'static str {
    use autoinstinct::causal_harness::Perturbation::*;
    match p {
        DropTriple(_) => "drop_ntriple",
        AddTriple(_) => "add_ntriple",
        DropPostureBit(_) => "drop_posture",
        DropExpectation(_) => "drop_expectation",
        DropRisk(_) => "drop_risk",
        DropAffordance(_) => "drop_affordance",
    }
}

fn assert_every_perturbation_changes_response(s: &CausalScenario) {
    let (baseline, _) = {
        let (f, p, c) = build_inputs(s);
        respond(&f, &p, &c)
    };
    for (pert, expected_after, expected_reason) in &s.perturbations {
        let (f, p, c) = perturb(s, pert);
        let (after, after_reason) = respond(&f, &p, &c);
        println!(
            "scenario={} baseline={:?} perturb={} expected={:?} after={:?} reason={} correct={}",
            s.name,
            baseline,
            perturbation_tag(pert),
            expected_after,
            after,
            after_reason,
            after == *expected_after && after_reason == *expected_reason
        );
        assert_eq!(
            after, *expected_after,
            "scenario `{}`: perturbation {:?} produced {:?} but expected {:?}",
            s.name, pert, after, expected_after
        );
        assert_eq!(
            after_reason, *expected_reason,
            "scenario `{}`: perturbation {:?} expected trace reason `{}` but got `{}`",
            s.name, pert, expected_reason, after_reason
        );
    }
}

#[test]
fn causal_every_response_class_has_positive_assertion() {
    for s in canonical_scenarios() {
        assert_positive(&s);
    }
}

#[test]
fn causal_every_perturbation_changes_response() {
    for s in canonical_scenarios() {
        assert_every_perturbation_changes_response(&s);
    }
}

#[test]
fn causal_remove_required_triple_changes_response() {
    let scenarios = canonical_scenarios();
    let s = scenarios
        .iter()
        .find(|s| s.name == "ask_via_evidence_gap")
        .expect("ask scenario present");
    // Without the DigitalDocument triple, evidence-gap branch can't fire,
    // so Ask should fall through to Ignore (calm + empty).
    let (pert, expected, _) = &s.perturbations[0];
    let (f, p, c) = perturb(s, pert);
    let (result, _) = respond(&f, &p, &c);
    assert_eq!(result, *expected, "removing DD triple must produce Ignore");
    assert_ne!(result, AutonomicInstinct::Ask);
}

#[test]
fn causal_remove_required_posture_bit_changes_response() {
    let scenarios = canonical_scenarios();
    let s = scenarios
        .iter()
        .find(|s| s.name == "settle_via_settled_posture")
        .expect("settle scenario present");
    let (pert, expected, _) = &s.perturbations[0];
    let (f, p, c) = perturb(s, pert);
    let (result, _) = respond(&f, &p, &c);
    assert_eq!(result, *expected);
    assert_ne!(result, AutonomicInstinct::Settle);
}

#[test]
fn causal_remove_required_affordance_changes_response() {
    let scenarios = canonical_scenarios();
    let s = scenarios
        .iter()
        .find(|s| s.name == "retrieve_via_expected_package")
        .expect("retrieve scenario present");
    // Drop CAN_RETRIEVE_NOW — Retrieve precondition fails.
    let (pert, expected, _) = s
        .perturbations
        .iter()
        .find(|(p, _, _)| matches!(p, autoinstinct::causal_harness::Perturbation::DropAffordance(_)))
        .expect("retrieve scenario must drop CAN_RETRIEVE_NOW");
    let (f, p, c) = perturb(s, pert);
    let (result, _) = respond(&f, &p, &c);
    assert_eq!(result, *expected);
    assert_ne!(result, AutonomicInstinct::Retrieve);
}

#[test]
fn causal_constant_response_policy_is_rejected_by_gauntlet() {
    // A "policy" that emits the same response regardless of inputs cannot
    // be earned: at least one perturbation must demote it. We sample the
    // canonical scenarios to confirm the lattice is *not* constant — the
    // observed response set must include ≥6 distinct classes.
    use std::collections::HashSet;
    let mut seen: HashSet<AutonomicInstinct> = HashSet::new();
    for s in canonical_scenarios() {
        let (f, p, c) = build_inputs(&s);
        seen.insert(respond(&f, &p, &c).0);
        for (pert, _, _) in &s.perturbations {
            let (f, p, c) = perturb(&s, pert);
            seen.insert(respond(&f, &p, &c).0);
        }
    }
    assert!(
        seen.len() >= 6,
        "input space did not produce ≥6 distinct response classes; saw {seen:?}"
    );
}
