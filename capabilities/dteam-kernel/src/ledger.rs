//! Immutable receipt chain, replay verification, standing, and queries.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{AuthorityId, CapabilityId, Intent, OperationId, Outcome, Standing, SubjectId};
use std::fmt::{Display, Formatter};

/// Normalized receipt classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptKind {
    Applied,
    Refused,
    Failed,
}

impl ReceiptKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Refused => "refused",
            Self::Failed => "failed",
        }
    }
}

impl From<&Outcome> for ReceiptKind {
    fn from(outcome: &Outcome) -> Self {
        match outcome {
            Outcome::Applied { .. } => Self::Applied,
            Outcome::Refused { .. } => Self::Refused,
            Outcome::Failed { .. } => Self::Failed,
        }
    }
}

/// Immutable evidence binding one intent to its observed outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    index: u64,
    previous: Digest,
    broker: String,
    executor: String,
    capability: CapabilityId,
    operation: OperationId,
    subject: SubjectId,
    authority: AuthorityId,
    admission_digest: Digest,
    intent_digest: Digest,
    outcome_digest: Digest,
    kind: ReceiptKind,
    logical_start: u64,
    logical_end: u64,
    digest: Digest,
}

impl Receipt {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn manufacture(
        index: u64,
        previous: Digest,
        broker: &str,
        executor: &str,
        intent: &Intent,
        outcome: &Outcome,
        logical_start: u64,
        logical_end: u64,
    ) -> Self {
        let kind = ReceiptKind::from(outcome);
        let mut receipt = Self {
            index,
            previous,
            broker: broker.to_owned(),
            executor: executor.to_owned(),
            capability: intent.capability().clone(),
            operation: intent.operation().clone(),
            subject: intent.subject().clone(),
            authority: intent.authority().clone(),
            admission_digest: intent.admission_digest(),
            intent_digest: intent.digest(),
            outcome_digest: outcome.digest(),
            kind,
            logical_start,
            logical_end,
            digest: Digest::ZERO,
        };
        receipt.digest = receipt.recompute_digest();
        receipt
    }

    #[must_use]
    pub const fn index(&self) -> u64 {
        self.index
    }

    #[must_use]
    pub const fn previous(&self) -> Digest {
        self.previous
    }

    #[must_use]
    pub fn broker(&self) -> &str {
        &self.broker
    }

    #[must_use]
    pub fn executor(&self) -> &str {
        &self.executor
    }

    #[must_use]
    pub const fn capability(&self) -> &CapabilityId {
        &self.capability
    }

    #[must_use]
    pub const fn operation(&self) -> &OperationId {
        &self.operation
    }

    #[must_use]
    pub const fn subject(&self) -> &SubjectId {
        &self.subject
    }

    #[must_use]
    pub const fn authority(&self) -> &AuthorityId {
        &self.authority
    }

    #[must_use]
    pub const fn admission_digest(&self) -> Digest {
        self.admission_digest
    }

    #[must_use]
    pub const fn intent_digest(&self) -> Digest {
        self.intent_digest
    }

    #[must_use]
    pub const fn outcome_digest(&self) -> Digest {
        self.outcome_digest
    }

    #[must_use]
    pub const fn kind(&self) -> ReceiptKind {
        self.kind
    }

    #[must_use]
    pub const fn logical_start(&self) -> u64 {
        self.logical_start
    }

    #[must_use]
    pub const fn logical_end(&self) -> u64 {
        self.logical_end
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "receipt-v1")
            .u64("index", self.index)
            .field("previous", &self.previous.0)
            .text("broker", &self.broker)
            .text("executor", &self.executor)
            .text("capability", self.capability.as_str())
            .text("operation", self.operation.as_str())
            .text("subject", self.subject.as_str())
            .text("authority", self.authority.as_str())
            .field("admission", &self.admission_digest.0)
            .field("intent", &self.intent_digest.0)
            .field("outcome", &self.outcome_digest.0)
            .text("kind", self.kind.as_str())
            .u64("logical-start", self.logical_start)
            .u64("logical-end", self.logical_end);
        encoder.digest()
    }
}

/// Errors appending or replaying a receipt chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerError {
    IndexMismatch { expected: u64, actual: u64 },
    PreviousMismatch { expected: Digest, actual: Digest },
    DigestMismatch { index: u64 },
    LogicalClockRegression { index: u64, start: u64, end: u64 },
}

impl Display for LedgerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexMismatch { expected, actual } => {
                write!(formatter, "receipt index {actual}, expected {expected}")
            }
            Self::PreviousMismatch { expected, actual } => {
                write!(formatter, "receipt predecessor {actual}, expected {expected}")
            }
            Self::DigestMismatch { index } => {
                write!(formatter, "receipt {index} digest does not match content")
            }
            Self::LogicalClockRegression { index, start, end } => write!(
                formatter,
                "receipt {index} logical clock regressed: start={start}, end={end}"
            ),
        }
    }
}

impl std::error::Error for LedgerError {}

/// Verified aggregate replay evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReport {
    entries: usize,
    head: Digest,
    applied: usize,
    refused: usize,
    failed: usize,
    standing: Standing,
}

impl ReplayReport {
    #[must_use]
    pub const fn entries(&self) -> usize {
        self.entries
    }

    #[must_use]
    pub const fn head(&self) -> Digest {
        self.head
    }

    #[must_use]
    pub const fn applied(&self) -> usize {
        self.applied
    }

    #[must_use]
    pub const fn refused(&self) -> usize {
        self.refused
    }

    #[must_use]
    pub const fn failed(&self) -> usize {
        self.failed
    }

    #[must_use]
    pub const fn standing(&self) -> Standing {
        self.standing
    }
}

/// Query over immutable receipts. Unset fields are wildcards.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReceiptQuery {
    capability: Option<CapabilityId>,
    operation: Option<OperationId>,
    subject: Option<SubjectId>,
    authority: Option<AuthorityId>,
    kind: Option<ReceiptKind>,
    min_index: Option<u64>,
    max_index: Option<u64>,
}

impl ReceiptQuery {
    #[must_use]
    pub fn capability(mut self, value: CapabilityId) -> Self {
        self.capability = Some(value);
        self
    }

    #[must_use]
    pub fn operation(mut self, value: OperationId) -> Self {
        self.operation = Some(value);
        self
    }

    #[must_use]
    pub fn subject(mut self, value: SubjectId) -> Self {
        self.subject = Some(value);
        self
    }

    #[must_use]
    pub fn authority(mut self, value: AuthorityId) -> Self {
        self.authority = Some(value);
        self
    }

    #[must_use]
    pub const fn kind(mut self, value: ReceiptKind) -> Self {
        self.kind = Some(value);
        self
    }

    #[must_use]
    pub const fn from_index(mut self, value: u64) -> Self {
        self.min_index = Some(value);
        self
    }

    #[must_use]
    pub const fn through_index(mut self, value: u64) -> Self {
        self.max_index = Some(value);
        self
    }

    fn matches(&self, receipt: &Receipt) -> bool {
        self.capability
            .as_ref()
            .is_none_or(|value| value == receipt.capability())
            && self
                .operation
                .as_ref()
                .is_none_or(|value| value == receipt.operation())
            && self
                .subject
                .as_ref()
                .is_none_or(|value| value == receipt.subject())
            && self
                .authority
                .as_ref()
                .is_none_or(|value| value == receipt.authority())
            && self.kind.is_none_or(|value| value == receipt.kind())
            && self
                .min_index
                .is_none_or(|minimum| receipt.index() >= minimum)
            && self
                .max_index
                .is_none_or(|maximum| receipt.index() <= maximum)
    }
}

/// Append-only in-memory ledger with deterministic replay.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReceiptLedger {
    receipts: Vec<Receipt>,
}

impl ReceiptLedger {
    /// Starts an empty chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the next receipt index.
    #[must_use]
    pub fn next_index(&self) -> u64 {
        self.receipts.len() as u64
    }

    /// Returns the current chain head.
    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts.last().map_or(Digest::ZERO, Receipt::digest)
    }

    /// Returns all receipts.
    #[must_use]
    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    /// Appends one pre-manufactured receipt after checking chain identity.
    pub fn append(&mut self, receipt: Receipt) -> Result<(), LedgerError> {
        let expected_index = self.next_index();
        if receipt.index() != expected_index {
            return Err(LedgerError::IndexMismatch {
                expected: expected_index,
                actual: receipt.index(),
            });
        }
        let expected_previous = self.head();
        if receipt.previous() != expected_previous {
            return Err(LedgerError::PreviousMismatch {
                expected: expected_previous,
                actual: receipt.previous(),
            });
        }
        if receipt.logical_end() < receipt.logical_start() {
            return Err(LedgerError::LogicalClockRegression {
                index: receipt.index(),
                start: receipt.logical_start(),
                end: receipt.logical_end(),
            });
        }
        if receipt.digest() != receipt.recompute_digest() {
            return Err(LedgerError::DigestMismatch {
                index: receipt.index(),
            });
        }
        self.receipts.push(receipt);
        Ok(())
    }

    /// Returns the receipt already bound to an intent, enabling exactly-once semantics.
    #[must_use]
    pub fn by_intent(&self, digest: Digest) -> Option<&Receipt> {
        self.receipts
            .iter()
            .find(|receipt| receipt.intent_digest() == digest)
    }

    /// Queries immutable evidence without changing its order.
    pub fn query<'ledger>(
        &'ledger self,
        query: &'ledger ReceiptQuery,
    ) -> impl Iterator<Item = &'ledger Receipt> + 'ledger {
        self.receipts
            .iter()
            .filter(move |receipt| query.matches(receipt))
    }

    /// Replays and verifies the complete chain.
    pub fn verify(&self) -> Result<ReplayReport, LedgerError> {
        let mut previous = Digest::ZERO;
        let mut applied = 0;
        let mut refused = 0;
        let mut failed = 0;
        for (position, receipt) in self.receipts.iter().enumerate() {
            let expected_index = position as u64;
            if receipt.index() != expected_index {
                return Err(LedgerError::IndexMismatch {
                    expected: expected_index,
                    actual: receipt.index(),
                });
            }
            if receipt.previous() != previous {
                return Err(LedgerError::PreviousMismatch {
                    expected: previous,
                    actual: receipt.previous(),
                });
            }
            if receipt.logical_end() < receipt.logical_start() {
                return Err(LedgerError::LogicalClockRegression {
                    index: receipt.index(),
                    start: receipt.logical_start(),
                    end: receipt.logical_end(),
                });
            }
            if receipt.digest() != receipt.recompute_digest() {
                return Err(LedgerError::DigestMismatch {
                    index: receipt.index(),
                });
            }
            match receipt.kind() {
                ReceiptKind::Applied => applied += 1,
                ReceiptKind::Refused => refused += 1,
                ReceiptKind::Failed => failed += 1,
            }
            previous = receipt.digest();
        }

        let standing = if self.receipts.is_empty() {
            Standing::Unknown
        } else if failed > 0 {
            Standing::PartialAlive
        } else if applied > 0 {
            Standing::Alive
        } else if refused > 0 {
            Standing::Refused
        } else {
            Standing::Unknown
        };
        Ok(ReplayReport {
            entries: self.receipts.len(),
            head: previous,
            applied,
            refused,
            failed,
            standing,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Receipt, ReceiptKind, ReceiptLedger, ReceiptQuery};
    use crate::model::{
        AdmittedObservation, AuthorityId, CapabilityId, Intent, Observation, OperationId, Outcome,
        PolicyId, SubjectId,
    };

    fn intent(nonce: u64) -> Intent {
        let observation = Observation::new(SubjectId::new("subject").unwrap(), 1);
        let admitted = AdmittedObservation::new(observation, PolicyId::new("policy").unwrap(), 1);
        Intent::construct(
            &admitted,
            CapabilityId::new("capability").unwrap(),
            OperationId::new("apply").unwrap(),
            AuthorityId::new("operator").unwrap(),
            nonce,
            vec![nonce as u8],
        )
    }

    #[test]
    fn chain_replays_and_queries() {
        let mut ledger = ReceiptLedger::new();
        for nonce in 0..2 {
            let intent = intent(nonce);
            let outcome = Outcome::Applied {
                code: 200,
                output: vec![nonce as u8],
            };
            let receipt = Receipt::manufacture(
                ledger.next_index(),
                ledger.head(),
                "broker",
                "executor",
                &intent,
                &outcome,
                nonce,
                nonce + 1,
            );
            ledger.append(receipt).unwrap();
        }
        let report = ledger.verify().unwrap();
        assert_eq!(report.applied(), 2);
        assert_eq!(
            ledger
                .query(&ReceiptQuery::default().kind(ReceiptKind::Applied))
                .count(),
            2
        );
    }

    #[test]
    fn intent_lookup_supports_idempotency() {
        let mut ledger = ReceiptLedger::new();
        let intent = intent(7);
        let receipt = Receipt::manufacture(
            0,
            ledger.head(),
            "broker",
            "executor",
            &intent,
            &Outcome::Applied {
                code: 200,
                output: Vec::new(),
            },
            0,
            1,
        );
        ledger.append(receipt).unwrap();
        assert_eq!(ledger.by_intent(intent.digest()).unwrap().index(), 0);
    }
}
