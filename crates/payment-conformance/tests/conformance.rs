use payment_conformance::*;

fn ident(beneficiary: &str, amount: &str) -> EffectIdentity {
    EffectIdentity {
        principal: "p1".into(),
        beneficiary: beneficiary.into(),
        amount: amount.into(),
        asset: "USD".into(),
        purpose: "inv-1".into(),
        authority: "grant-1".into(),
        obligation: "o1".into(),
        rail: "sim-rail".into(),
        expiry: "2026-12-31".into(),
        reservation: "res-1".into(),
    }
}

fn ev(id: &str, act: &str) -> Event {
    Event {
        id: id.into(),
        activity: act.into(),
        obligation_id: "o1".into(),
        effect_id: None,
        idempotency_key: Some("k1".into()),
        identity: None,
        replay: false,
        attrs: Default::default(),
    }
}

fn prepared() -> (Event, String) {
    let i = ident("b1", "10.00");
    let id = effect_id(&i);
    let mut e = ev("e2", "effect_prepared");
    e.identity = Some(i);
    e.effect_id = Some(id.clone());
    (e, id)
}

fn submit(id: &str, eid: &str) -> Event {
    let mut e = ev(id, "submitted");
    e.effect_id = Some(eid.into());
    e
}

fn head() -> (Vec<Event>, String) {
    let (p, id) = prepared();
    (vec![ev("e1", "obligation_created"), p], id)
}

#[test]
fn happy_path_conformant() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "accepted"));
    l.push(ev("e5", "settled"));
    let mut post = ev("e6", "ledger_posted");
    post.attrs.insert("posting".into(), "SETTLED".into());
    l.push(post);
    let r = check(&l);
    assert!(r.conformant(), "{:?}", r.violations);
    assert_eq!(r.final_states["o1"], State::Settled);
}

#[test]
fn accept_then_drop_ack_blind_retry_refused() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "timeout_observed"));
    l.push(submit("e5", &id));
    let r = check(&l);
    assert!(r
        .violations
        .contains(&Violation::BlindRetry { event: "e5".into() }));
}

#[test]
fn accept_then_drop_ack_reconcile_accepted_then_settle() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "timeout_observed"));
    l.push(ev("e5", "reconciled_accepted"));
    l.push(ev("e6", "settled"));
    let r = check(&l);
    assert!(r.conformant(), "{:?}", r.violations);
}

#[test]
fn reconcile_not_accepted_allows_resubmit() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "timeout_observed"));
    l.push(ev("e5", "reconciled_not_accepted"));
    l.push(submit("e6", &id));
    l.push(ev("e7", "accepted"));
    assert!(check(&l).conformant());
}

#[test]
fn trailing_unknown_is_flagged() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "timeout_observed"));
    let r = check(&l);
    assert!(r.violations.contains(&Violation::UnreconciledUnknown {
        obligation: "o1".into()
    }));
}

#[test]
fn replay_submit_refused() {
    let (mut l, id) = head();
    let mut s = submit("e3", &id);
    s.replay = true;
    l.push(s);
    let r = check(&l);
    assert!(r
        .violations
        .contains(&Violation::ReplayTriggeredExternalAction { event: "e3".into() }));
}

#[test]
fn ledger_settled_without_finality_refused() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "accepted"));
    let mut post = ev("e5", "ledger_posted");
    post.attrs.insert("posting".into(), "SETTLED".into());
    l.push(post);
    let r = check(&l);
    assert!(r
        .violations
        .contains(&Violation::LedgerSettledWithoutFinality { event: "e5".into() }));
}

#[test]
fn double_settlement_refused() {
    let (mut l, id) = head();
    l.push(submit("e3", &id));
    l.push(ev("e4", "accepted"));
    l.push(ev("e5", "settled"));
    l.push(ev("e6", "settled"));
    let r = check(&l);
    assert!(r.violations.contains(&Violation::DoubleSettlement {
        obligation: "o1".into()
    }));
}

#[test]
fn changed_beneficiary_or_amount_changes_effect_id_and_is_refused() {
    let base = effect_id(&ident("b1", "10.00"));
    assert_ne!(base, effect_id(&ident("b2", "10.00")));
    assert_ne!(base, effect_id(&ident("b1", "10.01")));
    // Submit under a different effect id than the prepared/authorized one.
    let (mut l, _id) = head();
    l.push(submit("e3", &effect_id(&ident("b2", "10.00"))));
    let r = check(&l);
    assert!(r
        .violations
        .contains(&Violation::RefusedEffectIdentityMismatch { event: "e3".into() }));
    // Prepared event whose id was not derived from its identity.
    let (mut p, _) = prepared();
    p.effect_id = Some("00".repeat(32));
    let r = check(&[ev("e1", "obligation_created"), p]);
    assert!(r
        .violations
        .contains(&Violation::RefusedEffectIdentityMismatch { event: "e2".into() }));
}

#[test]
fn effect_id_is_length_prefixed_not_concatenation_ambiguous() {
    let mut a = ident("ab", "c");
    let mut b = ident("a", "bc");
    a.purpose = "x".into();
    b.purpose = "x".into();
    assert_ne!(effect_id(&a), effect_id(&b));
}

#[test]
fn canonical_hash_vector_is_stable() {
    // Pinned vector: changing the canonicalisation is a breaking change.
    let got = effect_id(&ident("b1", "10.00"));
    let want = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/vectors/effect_id_v1.txt"
    ))
    .unwrap();
    assert_eq!(got, want.trim());
}

#[test]
fn unjoinable_effect_refused() {
    let r = check(&[submit("e1", "deadbeef")]);
    assert!(r
        .violations
        .contains(&Violation::ChainNotJoinable { event: "e1".into() }));
}

#[test]
fn events_roundtrip_json_and_schema_lists_identity_fields() {
    let (l, _) = head();
    let s = serde_json::to_string(&l).unwrap();
    let back: Vec<Event> = serde_json::from_str(&s).unwrap();
    assert_eq!(check(&back).violations, check(&l).violations);
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/schema/prepared_economic_effect.v1.schema.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let ident_json = serde_json::to_value(ident("b1", "10.00")).unwrap();
    let mut req: Vec<&str> = schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    let mut have: Vec<&str> = ident_json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    req.sort();
    have.sort();
    assert_eq!(req, have);
}
