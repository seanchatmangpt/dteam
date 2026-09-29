//! Conformance checking of payment effect chains over OCEL-style event logs.
//!
//! Model (CASTLE/SA2A payments spec, non-authoritative operational extension):
//! OBLIGATION -> PREPARED -> SUBMITTED -> {UNKNOWN | ACCEPTED | REJECTED}
//!            -> {SETTLED | RETURNED | REVERSED}.
//! A timeout yields UNKNOWN; UNKNOWN must be reconciled, never blindly retried.
//! No ISO 20022 conformance is claimed; this is a process-level checker.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identity fields hashed into `effect_id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectIdentity {
    pub principal: String,
    pub beneficiary: String,
    pub amount: String,
    pub asset: String,
    pub purpose: String,
    pub authority: String,
    pub obligation: String,
    pub rail: String,
    pub expiry: String,
    pub reservation: String,
}

/// Canonical effect id: blake3 over length-prefixed fields in fixed order,
/// domain-separated. Any field change changes the id.
pub fn effect_id(i: &EffectIdentity) -> String {
    let mut h = blake3::Hasher::new();
    h.update(b"castle.PreparedEconomicEffect.v1\0");
    for f in [
        &i.principal,
        &i.beneficiary,
        &i.amount,
        &i.asset,
        &i.purpose,
        &i.authority,
        &i.obligation,
        &i.rail,
        &i.expiry,
        &i.reservation,
    ] {
        h.update(&(f.len() as u64).to_le_bytes());
        h.update(f.as_bytes());
    }
    h.finalize().to_hex().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub activity: String,
    pub obligation_id: String,
    #[serde(default)]
    pub effect_id: Option<String>,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Present on `effect_prepared`: the identity the effect_id must match.
    #[serde(default)]
    pub identity: Option<EffectIdentity>,
    /// True when the event was produced by event replay rather than live execution.
    #[serde(default)]
    pub replay: bool,
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum State {
    Obligation,
    Prepared,
    Submitted,
    Unknown,
    Accepted,
    Rejected,
    Settled,
    Returned,
    Reversed,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Violation {
    /// Falsifier F2: effect_id does not match hash of identity, or submitted id differs from prepared.
    RefusedEffectIdentityMismatch { event: String },
    /// F3: submit after UNKNOWN without reconciliation proving non-acceptance.
    BlindRetry { event: String },
    /// F4: event references an effect/obligation not joinable to the chain.
    ChainNotJoinable { event: String },
    /// F5: ledger marks SETTLED without an observed settlement.
    LedgerSettledWithoutFinality { event: String },
    /// F6: replayed event performed an external action (submit).
    ReplayTriggeredExternalAction { event: String },
    /// More than one valid settlement for one obligation.
    DoubleSettlement { obligation: String },
    /// Transition not permitted by the state model.
    IllegalTransition { event: String, from: State },
    /// Obligation ends in UNKNOWN with no reconciliation.
    UnreconciledUnknown { obligation: String },
}

#[derive(Debug, Default)]
pub struct Report {
    pub violations: Vec<Violation>,
    pub final_states: BTreeMap<String, State>,
}

impl Report {
    pub fn conformant(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Default)]
struct Chain {
    state: Option<State>,
    prepared_effect: Option<String>,
    reconciled_clear: bool,
    settlements: usize,
    settlement_observed: bool,
    reconciled_any: bool,
}

pub fn check(events: &[Event]) -> Report {
    let mut chains: BTreeMap<String, Chain> = BTreeMap::new();
    let mut v = Vec::new();
    for e in events {
        let c = chains.entry(e.obligation_id.clone()).or_default();
        let from = c.state;
        let bad = |v: &mut Vec<Violation>, from: Option<State>| {
            v.push(Violation::IllegalTransition {
                event: e.id.clone(),
                from: from.unwrap_or(State::Obligation),
            })
        };
        match e.activity.as_str() {
            "obligation_created" => {
                if from.is_some() {
                    bad(&mut v, from);
                } else {
                    c.state = Some(State::Obligation);
                }
            }
            "effect_prepared" => {
                if from != Some(State::Obligation) {
                    if from.is_none() {
                        v.push(Violation::ChainNotJoinable {
                            event: e.id.clone(),
                        });
                    } else {
                        bad(&mut v, from);
                    }
                    continue;
                }
                match (&e.identity, &e.effect_id) {
                    (Some(i), Some(id))
                        if effect_id(i) == *id && i.obligation == e.obligation_id =>
                    {
                        c.prepared_effect = Some(id.clone());
                        c.state = Some(State::Prepared);
                    }
                    _ => v.push(Violation::RefusedEffectIdentityMismatch {
                        event: e.id.clone(),
                    }),
                }
            }
            "submitted" => {
                if e.replay {
                    v.push(Violation::ReplayTriggeredExternalAction {
                        event: e.id.clone(),
                    });
                    continue;
                }
                if c.prepared_effect.is_none() {
                    v.push(Violation::ChainNotJoinable {
                        event: e.id.clone(),
                    });
                    continue;
                }
                if e.effect_id != c.prepared_effect {
                    v.push(Violation::RefusedEffectIdentityMismatch {
                        event: e.id.clone(),
                    });
                    continue;
                }
                match from {
                    Some(State::Prepared) => c.state = Some(State::Submitted),
                    Some(State::Unknown) if !c.reconciled_clear => v.push(Violation::BlindRetry {
                        event: e.id.clone(),
                    }),
                    Some(State::Unknown) | Some(State::Rejected)
                        if c.reconciled_clear || from == Some(State::Rejected) =>
                    {
                        // Non-acceptance proven: a new submission attempt is eligible.
                        c.reconciled_clear = false;
                        c.state = Some(State::Submitted);
                    }
                    _ => bad(&mut v, from),
                }
            }
            "timeout_observed" => match from {
                Some(State::Submitted) => c.state = Some(State::Unknown),
                _ => bad(&mut v, from),
            },
            "reconciled_accepted" => match from {
                Some(State::Unknown) => {
                    c.reconciled_any = true;
                    c.state = Some(State::Accepted)
                }
                _ => bad(&mut v, from),
            },
            "reconciled_not_accepted" => match from {
                Some(State::Unknown) => {
                    c.reconciled_any = true;
                    c.reconciled_clear = true;
                }
                _ => bad(&mut v, from),
            },
            "accepted" => match from {
                Some(State::Submitted) => c.state = Some(State::Accepted),
                _ => bad(&mut v, from),
            },
            "rejected" => match from {
                Some(State::Submitted) => c.state = Some(State::Rejected),
                _ => bad(&mut v, from),
            },
            "settled" => match from {
                Some(State::Accepted) => {
                    c.settlements += 1;
                    c.settlement_observed = true;
                    c.state = Some(State::Settled);
                    if c.settlements > 1 {
                        v.push(Violation::DoubleSettlement {
                            obligation: e.obligation_id.clone(),
                        });
                    }
                }
                Some(State::Settled) => {
                    c.settlements += 1;
                    v.push(Violation::DoubleSettlement {
                        obligation: e.obligation_id.clone(),
                    });
                }
                _ => bad(&mut v, from),
            },
            "returned" => match from {
                Some(State::Settled) | Some(State::Accepted) => c.state = Some(State::Returned),
                _ => bad(&mut v, from),
            },
            "reversed" => match from {
                Some(State::Settled) => c.state = Some(State::Reversed),
                _ => bad(&mut v, from),
            },
            "ledger_posted" => {
                if e.attrs.get("posting").map(String::as_str) == Some("SETTLED")
                    && !c.settlement_observed
                {
                    v.push(Violation::LedgerSettledWithoutFinality {
                        event: e.id.clone(),
                    });
                }
            }
            _ => v.push(Violation::IllegalTransition {
                event: e.id.clone(),
                from: from.unwrap_or(State::Obligation),
            }),
        }
    }
    let mut r = Report::default();
    for (o, c) in &chains {
        if c.state == Some(State::Unknown) && !c.reconciled_any {
            v.push(Violation::UnreconciledUnknown {
                obligation: o.clone(),
            });
        }
        if let Some(s) = c.state {
            r.final_states.insert(o.clone(), s);
        }
    }
    v.sort();
    r.violations = v;
    r
}
