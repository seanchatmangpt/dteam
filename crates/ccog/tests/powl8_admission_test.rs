//! POWL8 plan admission via STRIPS.

use ccog::breeds::strips::{admit_powl8, admit_powl8_with_advanced};
use ccog::powl::{BinaryRelation, Powl8, Powl8Node, MAX_NODES};
use ccog::verdict::{Breed, PlanAdmission};
use ccog::FieldContext;

// Layout-dependent indices: tests below construct PartialOrder containers
// with `start=0`, so breed indices follow their listing order. Diamond uses
// 4 breeds (Strips at 3), linear uses 3 (Strips at 2).
const ELIZA_IDX: usize = 0;
const MYCIN_IDX: usize = 1;
const SHRDLU_IDX: usize = 2;
// Diamond layout: 0=Eliza, 1=Mycin, 2=Shrdlu, 3=Strips.
const STRIPS_IDX_DIAMOND: usize = 3;
// Linear layout: 0=Eliza, 1=Mycin, 2=Strips.
const STRIPS_IDX_LINEAR: usize = 2;

/// Build a partial-order plan whose first `count` nodes are the listed
/// breeds, followed by a single `PartialOrder` container that owns them via
/// `start = 0, count = count`.
fn build_breed_plan(breeds: &[Breed], rel: BinaryRelation) -> Powl8 {
    let mut p = Powl8::new();
    for b in breeds {
        p.push(Powl8Node::Activity(*b)).unwrap();
    }
    let _po = p
        .push(Powl8Node::PartialOrder {
            start: 0,
            count: breeds.len() as u16,
            rel,
        })
        .unwrap();
    p.root = 0;
    p
}

#[test]
fn linear_plan_ready_advances_in_order() {
    // Eliza -> Mycin -> Strips
    let mut rel = BinaryRelation::new();
    rel.add_edge(ELIZA_IDX, MYCIN_IDX);
    rel.add_edge(MYCIN_IDX, STRIPS_IDX_LINEAR);

    let plan = build_breed_plan(
        &[Breed::Eliza, Breed::Mycin, Breed::Strips],
        rel,
    );
    assert!(plan.shape_match().is_ok(), "linear plan should be Sound");

    // Step 1 — nothing advanced.
    let mut advanced = [false; MAX_NODES];
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.admission, PlanAdmission::Sound);
    assert!(v.admissible);
    assert_eq!(v.ready, vec![ELIZA_IDX]);
    assert!(v.blocked.contains(&MYCIN_IDX));
    assert!(v.blocked.contains(&STRIPS_IDX_LINEAR));

    // Step 2 — Eliza advanced.
    advanced[ELIZA_IDX] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.ready, vec![MYCIN_IDX]);
    assert!(v.blocked.contains(&STRIPS_IDX_LINEAR));

    // Step 3 — Mycin advanced.
    advanced[MYCIN_IDX] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.ready, vec![STRIPS_IDX_LINEAR]);
    assert!(v.blocked.is_empty());

    // Step 4 — all advanced. No work remains, plan still admissible.
    advanced[STRIPS_IDX_LINEAR] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.admission, PlanAdmission::Sound);
    assert!(v.admissible);
    assert!(v.ready.is_empty());
    assert!(v.blocked.is_empty());
}

#[test]
fn diamond_plan_joins_after_both_branches() {
    // Eliza -> {Mycin, Shrdlu} -> Strips
    // Layout: 0=Eliza, 1=Mycin, 2=Shrdlu, 3=Strips.
    let mut rel = BinaryRelation::new();
    rel.add_edge(ELIZA_IDX, MYCIN_IDX);
    rel.add_edge(ELIZA_IDX, SHRDLU_IDX);
    rel.add_edge(MYCIN_IDX, STRIPS_IDX_DIAMOND);
    rel.add_edge(SHRDLU_IDX, STRIPS_IDX_DIAMOND);

    let plan = build_breed_plan(
        &[Breed::Eliza, Breed::Mycin, Breed::Shrdlu, Breed::Strips],
        rel,
    );
    assert!(plan.shape_match().is_ok(), "diamond plan should be Sound");

    // Pre-advance Eliza only — both branches become ready, Strips blocked.
    let mut advanced = [false; MAX_NODES];
    advanced[ELIZA_IDX] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.admission, PlanAdmission::Sound);
    assert!(v.ready.contains(&MYCIN_IDX));
    assert!(v.ready.contains(&SHRDLU_IDX));
    assert!(v.blocked.contains(&STRIPS_IDX_DIAMOND));
    assert!(!v.ready.contains(&STRIPS_IDX_DIAMOND));

    // Mark Mycin only — Strips still blocked (waiting on Shrdlu).
    advanced[MYCIN_IDX] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert!(v.ready.contains(&SHRDLU_IDX));
    assert!(v.blocked.contains(&STRIPS_IDX_DIAMOND));
    assert!(!v.ready.contains(&STRIPS_IDX_DIAMOND));

    // Mark Shrdlu too — Strips is now ready.
    advanced[SHRDLU_IDX] = true;
    let v = admit_powl8_with_advanced(&plan, &advanced).unwrap();
    assert_eq!(v.ready, vec![STRIPS_IDX_DIAMOND]);
    assert!(v.blocked.is_empty());
}

#[test]
fn cyclic_plan_rejected_with_admission_cyclic() {
    // 0 -> 1 -> 2 -> 0 (cycle).
    let mut rel = BinaryRelation::new();
    rel.add_edge(0, 1);
    rel.add_edge(1, 2);
    rel.add_edge(2, 0);

    let plan = build_breed_plan(
        &[Breed::Eliza, Breed::Mycin, Breed::Strips],
        rel,
    );
    assert!(plan.shape_match().is_err(), "cyclic plan must fail shape_match");
    assert_eq!(
        plan.shape_match().unwrap_err(),
        PlanAdmission::Cyclic,
        "cycle should classify as Cyclic, not Malformed"
    );

    let field = FieldContext::new("powl-cyclic");
    let v = admit_powl8(&plan, &field).unwrap();
    assert_eq!(v.admission, PlanAdmission::Cyclic);
    assert!(!v.admissible);
    assert!(v.ready.is_empty());
    assert!(v.blocked.is_empty());
}
