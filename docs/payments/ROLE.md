# Payments Role: dteam

dteam is the process/evidence input for CASTLE/SA2A payment effect chains. Deliverable:
`crates/payment-conformance` (standalone crate, own `[workspace]`, no change to existing crates).

## What it checks

An OCEL-style event list (one chain per `obligation_id`) against the settlement state model
OBLIGATION, PREPARED, SUBMITTED, then UNKNOWN or ACCEPTED or REJECTED, then SETTLED or RETURNED
or REVERSED. Timeout yields UNKNOWN; only `reconciled_not_accepted` makes resubmission eligible.

## Falsifier coverage

| Falsifier | Violation | Test |
|---|---|---|
| F2 changed beneficiary/amount reuses authorization | RefusedEffectIdentityMismatch | changed_beneficiary_or_amount_... |
| F3 timeout permits blind retry | BlindRetry | accept_then_drop_ack_blind_retry_refused |
| F4 ids not joinable | ChainNotJoinable | unjoinable_effect_refused |
| F5 ledger SETTLED without finality | LedgerSettledWithoutFinality | ledger_settled_without_finality_refused |
| F6 replay triggers external DO | ReplayTriggeredExternalAction | replay_submit_refused |
| single settlement per obligation | DoubleSettlement | double_settlement_refused |

Also: `effect_id` canonical hash (blake3, domain-separated, length-prefixed) with a pinned
vector; JSON Schema for the identity subset. Not covered here: F1, F7-F10.

No ISO 20022 conformance is claimed; example.org identifiers are non-authoritative.

## Gate

`cd crates/payment-conformance && cargo test`
