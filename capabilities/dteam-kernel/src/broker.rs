//! Broker-only DO path with pre-actuation authorization evidence and completion receipts.

use crate::graph::{CapabilityGraph, CapabilityPlan, GraphError};
use crate::hash::{CanonicalEncoder, Digest};
use crate::ledger::{LedgerError, Receipt, ReceiptLedger};
use crate::model::{Intent, Outcome};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

/// Executor refusal before any side effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreflightRefusal {
    pub code: String,
    pub detail: String,
}

/// A side-effect adapter. Only `Broker` calls `execute`.
pub trait Executor {
    /// Stable executor identity included in receipts.
    fn id(&self) -> &str;

    /// Validates executor-local constraints without side effects.
    fn preflight(&self, intent: &Intent) -> Result<(), PreflightRefusal>;

    /// Performs the admitted side effect and returns its observed outcome.
    fn execute(&mut self, intent: &Intent) -> Outcome;
}

/// Immutable evidence emitted before calling an executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationReceipt {
    index: u64,
    previous: Digest,
    broker: String,
    executor: String,
    intent_digest: Digest,
    plan_digest: Digest,
    plan_cost: u64,
    logical_time: u64,
    digest: Digest,
}

impl AuthorizationReceipt {
    fn manufacture(
        index: u64,
        previous: Digest,
        broker: &str,
        executor: &str,
        intent: &Intent,
        plan: &CapabilityPlan,
        logical_time: u64,
    ) -> Self {
        let mut receipt = Self {
            index,
            previous,
            broker: broker.to_owned(),
            executor: executor.to_owned(),
            intent_digest: intent.digest(),
            plan_digest: plan.digest(),
            plan_cost: plan.total_cost(),
            logical_time,
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
    pub const fn intent_digest(&self) -> Digest {
        self.intent_digest
    }

    #[must_use]
    pub const fn plan_digest(&self) -> Digest {
        self.plan_digest
    }

    #[must_use]
    pub const fn plan_cost(&self) -> u64 {
        self.plan_cost
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "authorization-receipt-v1")
            .u64("index", self.index)
            .field("previous", &self.previous.0)
            .text("broker", &self.broker)
            .text("executor", &self.executor)
            .field("intent", &self.intent_digest.0)
            .field("plan", &self.plan_digest.0)
            .u64("plan-cost", self.plan_cost)
            .u64("logical-time", self.logical_time);
        encoder.digest()
    }
}

/// Errors that invalidate authorization evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    IndexMismatch { expected: u64, actual: u64 },
    PreviousMismatch { expected: Digest, actual: Digest },
    DigestMismatch { index: u64 },
}

impl Display for AuthorizationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexMismatch { expected, actual } => {
                write!(
                    formatter,
                    "authorization index {actual}, expected {expected}"
                )
            }
            Self::PreviousMismatch { expected, actual } => write!(
                formatter,
                "authorization predecessor {actual}, expected {expected}"
            ),
            Self::DigestMismatch { index } => {
                write!(formatter, "authorization {index} digest mismatch")
            }
        }
    }
}

impl std::error::Error for AuthorizationError {}

/// Append-only authorization chain that must advance before execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthorizationLedger {
    receipts: Vec<AuthorizationReceipt>,
}

impl AuthorizationLedger {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn receipts(&self) -> &[AuthorizationReceipt] {
        &self.receipts
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts
            .last()
            .map_or(Digest::ZERO, AuthorizationReceipt::digest)
    }

    pub fn append(&mut self, receipt: AuthorizationReceipt) -> Result<(), AuthorizationError> {
        let expected_index = self.receipts.len() as u64;
        if receipt.index() != expected_index {
            return Err(AuthorizationError::IndexMismatch {
                expected: expected_index,
                actual: receipt.index(),
            });
        }
        let expected_previous = self.head();
        if receipt.previous() != expected_previous {
            return Err(AuthorizationError::PreviousMismatch {
                expected: expected_previous,
                actual: receipt.previous(),
            });
        }
        if receipt.digest() != receipt.recompute_digest() {
            return Err(AuthorizationError::DigestMismatch {
                index: receipt.index(),
            });
        }
        self.receipts.push(receipt);
        Ok(())
    }

    pub fn verify(&self) -> Result<Digest, AuthorizationError> {
        let mut previous = Digest::ZERO;
        for (position, receipt) in self.receipts.iter().enumerate() {
            let expected_index = position as u64;
            if receipt.index() != expected_index {
                return Err(AuthorizationError::IndexMismatch {
                    expected: expected_index,
                    actual: receipt.index(),
                });
            }
            if receipt.previous() != previous {
                return Err(AuthorizationError::PreviousMismatch {
                    expected: previous,
                    actual: receipt.previous(),
                });
            }
            if receipt.digest() != receipt.recompute_digest() {
                return Err(AuthorizationError::DigestMismatch {
                    index: receipt.index(),
                });
            }
            previous = receipt.digest();
        }
        Ok(previous)
    }

    #[must_use]
    pub fn by_intent(&self, digest: Digest) -> Option<&AuthorizationReceipt> {
        self.receipts
            .iter()
            .find(|receipt| receipt.intent_digest() == digest)
    }
}

/// Evidence returned for every broker decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActuationEvidence {
    authorization: Option<AuthorizationReceipt>,
    completion: Receipt,
    duplicate: bool,
}

impl ActuationEvidence {
    #[must_use]
    pub const fn authorization(&self) -> Option<&AuthorizationReceipt> {
        self.authorization.as_ref()
    }

    #[must_use]
    pub const fn completion(&self) -> &Receipt {
        &self.completion
    }

    #[must_use]
    pub const fn duplicate(&self) -> bool {
        self.duplicate
    }
}

/// Batch execution policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchMode {
    Continue,
    StopOnNonApplied,
}

/// Complete evidence from a batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchEvidence {
    items: Vec<ActuationEvidence>,
    stopped_at: Option<usize>,
}

impl BatchEvidence {
    #[must_use]
    pub fn items(&self) -> &[ActuationEvidence] {
        &self.items
    }

    #[must_use]
    pub const fn stopped_at(&self) -> Option<usize> {
        self.stopped_at
    }
}

/// Broker configuration or internal invariant failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrokerError {
    UnknownCapability(String),
    UnsupportedOperation {
        capability: String,
        operation: String,
    },
    AuthorityDenied {
        capability: String,
        authority: String,
    },
    Plan(GraphError),
    Authorization(AuthorizationError),
    Ledger(LedgerError),
}

impl Display for BrokerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCapability(value) => write!(formatter, "unknown capability `{value}`"),
            Self::UnsupportedOperation {
                capability,
                operation,
            } => write!(
                formatter,
                "capability `{capability}` does not support operation `{operation}`"
            ),
            Self::AuthorityDenied {
                capability,
                authority,
            } => write!(
                formatter,
                "authority `{authority}` cannot actuate capability `{capability}`"
            ),
            Self::Plan(error) => Display::fmt(error, formatter),
            Self::Authorization(error) => Display::fmt(error, formatter),
            Self::Ledger(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for BrokerError {}

impl From<GraphError> for BrokerError {
    fn from(value: GraphError) -> Self {
        Self::Plan(value)
    }
}

impl From<AuthorizationError> for BrokerError {
    fn from(value: AuthorizationError) -> Self {
        Self::Authorization(value)
    }
}

impl From<LedgerError> for BrokerError {
    fn from(value: LedgerError) -> Self {
        Self::Ledger(value)
    }
}

/// Exclusive actuation broker. Intents cannot execute themselves.
#[derive(Clone, Debug)]
pub struct Broker {
    id: String,
    graph: CapabilityGraph,
    maximum_plan_cost: u64,
    logical_clock: u64,
    authorizations: AuthorizationLedger,
    completions: ReceiptLedger,
    outcomes: BTreeMap<Digest, Outcome>,
}

impl Broker {
    /// Creates a broker over a validated graph.
    pub fn new(
        id: impl Into<String>,
        graph: CapabilityGraph,
        maximum_plan_cost: u64,
    ) -> Result<Self, BrokerError> {
        graph.validate()?;
        Ok(Self {
            id: id.into(),
            graph,
            maximum_plan_cost,
            logical_clock: 0,
            authorizations: AuthorizationLedger::new(),
            completions: ReceiptLedger::new(),
            outcomes: BTreeMap::new(),
        })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn graph(&self) -> &CapabilityGraph {
        &self.graph
    }

    #[must_use]
    pub const fn authorizations(&self) -> &AuthorizationLedger {
        &self.authorizations
    }

    #[must_use]
    pub const fn completions(&self) -> &ReceiptLedger {
        &self.completions
    }

    #[must_use]
    pub fn outcome(&self, intent: Digest) -> Option<&Outcome> {
        self.outcomes.get(&intent)
    }

    fn tick(&mut self) -> u64 {
        let current = self.logical_clock;
        self.logical_clock = self.logical_clock.saturating_add(1);
        current
    }

    fn complete(
        &mut self,
        executor_id: &str,
        intent: &Intent,
        outcome: Outcome,
        logical_start: u64,
        authorization: Option<AuthorizationReceipt>,
    ) -> Result<ActuationEvidence, BrokerError> {
        let logical_end = self.tick();
        let completion = Receipt::manufacture(
            self.completions.next_index(),
            self.completions.head(),
            &self.id,
            executor_id,
            intent,
            &outcome,
            logical_start,
            logical_end,
        );
        self.completions.append(completion.clone())?;
        self.outcomes.insert(intent.digest(), outcome);
        Ok(ActuationEvidence {
            authorization,
            completion,
            duplicate: false,
        })
    }

    /// Validates, authorizes, executes, and receipts exactly one intent.
    pub fn actuate<E: Executor>(
        &mut self,
        executor: &mut E,
        intent: &Intent,
    ) -> Result<ActuationEvidence, BrokerError> {
        if let Some(existing) = self.completions.by_intent(intent.digest()).cloned() {
            return Ok(ActuationEvidence {
                authorization: self.authorizations.by_intent(intent.digest()).cloned(),
                completion: existing,
                duplicate: true,
            });
        }

        let logical_start = self.tick();
        let Some(capability) = self.graph.get(intent.capability()) else {
            return Err(BrokerError::UnknownCapability(
                intent.capability().as_str().to_owned(),
            ));
        };
        if !capability.supports_operation(intent.operation()) {
            let outcome = Outcome::Refused {
                code: "UNSUPPORTED_OPERATION".to_owned(),
                detail: format!(
                    "capability `{}` does not support `{}`",
                    intent.capability(),
                    intent.operation()
                ),
            };
            return self.complete(executor.id(), intent, outcome, logical_start, None);
        }
        if !capability.allows_authority(intent.authority()) {
            let outcome = Outcome::Refused {
                code: "AUTHORITY_DENIED".to_owned(),
                detail: format!(
                    "authority `{}` cannot actuate `{}`",
                    intent.authority(),
                    intent.capability()
                ),
            };
            return self.complete(executor.id(), intent, outcome, logical_start, None);
        }

        let plan = self
            .graph
            .resolve_bounded([intent.capability().clone()], self.maximum_plan_cost)?;
        if let Err(refusal) = executor.preflight(intent) {
            let outcome = Outcome::Refused {
                code: refusal.code,
                detail: refusal.detail,
            };
            return self.complete(executor.id(), intent, outcome, logical_start, None);
        }

        let authorization_time = self.tick();
        let authorization = AuthorizationReceipt::manufacture(
            self.authorizations.receipts().len() as u64,
            self.authorizations.head(),
            &self.id,
            executor.id(),
            intent,
            &plan,
            authorization_time,
        );
        self.authorizations.append(authorization.clone())?;

        // This is the only call site with execution authority. Authorization is durable first.
        let outcome = executor.execute(intent);
        self.complete(
            executor.id(),
            intent,
            outcome,
            logical_start,
            Some(authorization),
        )
    }

    /// Executes a batch while preserving per-intent authorization and completion receipts.
    pub fn actuate_batch<E: Executor>(
        &mut self,
        executor: &mut E,
        intents: &[Intent],
        mode: BatchMode,
    ) -> Result<BatchEvidence, BrokerError> {
        let mut items = Vec::with_capacity(intents.len());
        let mut stopped_at = None;
        for (index, intent) in intents.iter().enumerate() {
            let evidence = self.actuate(executor, intent)?;
            let applied = self
                .outcome(intent.digest())
                .is_some_and(Outcome::is_applied);
            items.push(evidence);
            if mode == BatchMode::StopOnNonApplied && !applied {
                stopped_at = Some(index);
                break;
            }
        }
        Ok(BatchEvidence { items, stopped_at })
    }

    /// Verifies both the pre-actuation and completion chains.
    pub fn verify(&self) -> Result<BrokerVerification, BrokerError> {
        let authorization_head = self.authorizations.verify()?;
        let completion = self.completions.verify()?;
        for receipt in self.authorizations.receipts() {
            let completion_receipt = self
                .completions
                .by_intent(receipt.intent_digest())
                .ok_or_else(|| {
                    BrokerError::Ledger(LedgerError::IndexMismatch {
                        expected: receipt.index(),
                        actual: u64::MAX,
                    })
                })?;
            if completion_receipt.intent_digest() != receipt.intent_digest() {
                return Err(BrokerError::Ledger(LedgerError::DigestMismatch {
                    index: completion_receipt.index(),
                }));
            }
        }
        Ok(BrokerVerification {
            authorization_head,
            completion_head: completion.head(),
            authorized: self.authorizations.receipts().len(),
            completed: completion.entries(),
            standing: completion.standing(),
        })
    }
}

/// Aggregate verification of the exclusive DO path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerVerification {
    authorization_head: Digest,
    completion_head: Digest,
    authorized: usize,
    completed: usize,
    standing: crate::model::Standing,
}

impl BrokerVerification {
    #[must_use]
    pub const fn authorization_head(&self) -> Digest {
        self.authorization_head
    }

    #[must_use]
    pub const fn completion_head(&self) -> Digest {
        self.completion_head
    }

    #[must_use]
    pub const fn authorized(&self) -> usize {
        self.authorized
    }

    #[must_use]
    pub const fn completed(&self) -> usize {
        self.completed
    }

    #[must_use]
    pub const fn standing(&self) -> crate::model::Standing {
        self.standing
    }
}

#[cfg(test)]
mod tests {
    use super::{BatchMode, Broker, Executor, PreflightRefusal};
    use crate::graph::{Capability, CapabilityGraph};
    use crate::model::{
        AdmittedObservation, AuthorityId, CapabilityId, Intent, Observation, OperationId, Outcome,
        PolicyId, SubjectId,
    };

    #[derive(Default)]
    struct CountingExecutor {
        calls: usize,
        refuse_preflight: bool,
    }

    impl Executor for CountingExecutor {
        fn id(&self) -> &str {
            "counting"
        }

        fn preflight(&self, _intent: &Intent) -> Result<(), PreflightRefusal> {
            if self.refuse_preflight {
                Err(PreflightRefusal {
                    code: "LOCAL_REFUSAL".to_owned(),
                    detail: "executor unavailable".to_owned(),
                })
            } else {
                Ok(())
            }
        }

        fn execute(&mut self, intent: &Intent) -> Outcome {
            self.calls += 1;
            Outcome::Applied {
                code: 200,
                output: intent.payload().to_vec(),
            }
        }
    }

    fn intent(authority: &str, nonce: u64) -> Intent {
        let observation = Observation::new(SubjectId::new("subject").unwrap(), 1);
        let admitted = AdmittedObservation::new(observation, PolicyId::new("policy").unwrap(), 1);
        Intent::construct(
            &admitted,
            CapabilityId::new("deploy").unwrap(),
            OperationId::new("apply").unwrap(),
            AuthorityId::new(authority).unwrap(),
            nonce,
            vec![nonce as u8],
        )
    }

    fn broker() -> Broker {
        let mut graph = CapabilityGraph::new();
        graph
            .insert(
                Capability::new(CapabilityId::new("deploy").unwrap())
                    .supports(OperationId::new("apply").unwrap())
                    .allows(AuthorityId::new("release").unwrap())
                    .reversible(false),
            )
            .unwrap();
        Broker::new("broker", graph, 100).unwrap()
    }

    #[test]
    fn authorization_exists_before_success_completion() {
        let mut broker = broker();
        let mut executor = CountingExecutor::default();
        let evidence = broker
            .actuate(&mut executor, &intent("release", 1))
            .unwrap();
        assert!(evidence.authorization().is_some());
        assert_eq!(executor.calls, 1);
        assert_eq!(broker.verify().unwrap().authorized(), 1);
    }

    #[test]
    fn duplicate_intent_executes_once() {
        let mut broker = broker();
        let mut executor = CountingExecutor::default();
        let intent = intent("release", 1);
        broker.actuate(&mut executor, &intent).unwrap();
        let duplicate = broker.actuate(&mut executor, &intent).unwrap();
        assert!(duplicate.duplicate());
        assert_eq!(executor.calls, 1);
        assert_eq!(broker.completions().receipts().len(), 1);
    }

    #[test]
    fn denied_authority_is_receipted_without_execution() {
        let mut broker = broker();
        let mut executor = CountingExecutor::default();
        let evidence = broker
            .actuate(&mut executor, &intent("intruder", 1))
            .unwrap();
        assert!(evidence.authorization().is_none());
        assert_eq!(executor.calls, 0);
        assert_eq!(broker.verify().unwrap().completed(), 1);
    }

    #[test]
    fn batch_can_stop_on_refusal() {
        let mut broker = broker();
        let mut executor = CountingExecutor::default();
        let intents = [
            intent("release", 1),
            intent("intruder", 2),
            intent("release", 3),
        ];
        let evidence = broker
            .actuate_batch(&mut executor, &intents, BatchMode::StopOnNonApplied)
            .unwrap();
        assert_eq!(evidence.items().len(), 2);
        assert_eq!(evidence.stopped_at(), Some(1));
        assert_eq!(executor.calls, 1);
    }
}
