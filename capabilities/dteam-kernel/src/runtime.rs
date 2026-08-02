//! End-to-end parse, route, admit, construct, actuate, receipt, and trace runtime.

use crate::broker::{ActuationEvidence, Broker, BrokerError, Executor};
use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{AuthorityId, CapabilityId, Intent, Observation, OperationId, Outcome};
use crate::policy::{AdmissionDecision, AdmissionPolicy, Violation};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

/// Stable external route bound to a capability operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    name: String,
    capability: CapabilityId,
    operation: OperationId,
}

impl Route {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        capability: CapabilityId,
        operation: OperationId,
    ) -> Self {
        Self {
            name: name.into(),
            capability,
            operation,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn capability(&self) -> &CapabilityId {
        &self.capability
    }

    #[must_use]
    pub const fn operation(&self) -> &OperationId {
        &self.operation
    }
}

/// Deterministic route registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Router {
    routes: BTreeMap<String, Route>,
}

impl Router {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a route, returning the displaced route when replacing a name.
    pub fn insert(&mut self, route: Route) -> Option<Route> {
        self.routes.insert(route.name.clone(), route)
    }

    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<&Route> {
        self.routes.get(name)
    }

    pub fn routes(&self) -> impl ExactSizeIterator<Item = &Route> {
        self.routes.values()
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "router-v1")
            .u64("count", self.routes.len() as u64);
        for route in self.routes.values() {
            encoder
                .text("route", route.name())
                .text("capability", route.capability().as_str())
                .text("operation", route.operation().as_str());
        }
        encoder.digest()
    }
}

/// Process transition captured in an immutable trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceStage {
    Parsed,
    Routed,
    Admitted,
    Refused,
    Constructed,
    Authorized,
    Actuated,
    Receipted,
}

impl TraceStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parsed => "parsed",
            Self::Routed => "routed",
            Self::Admitted => "admitted",
            Self::Refused => "refused",
            Self::Constructed => "constructed",
            Self::Authorized => "authorized",
            Self::Actuated => "actuated",
            Self::Receipted => "receipted",
        }
    }
}

/// Hash-chained process trace event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
    index: u64,
    previous: Digest,
    stage: TraceStage,
    evidence: Digest,
    digest: Digest,
}

impl TraceEvent {
    fn new(index: u64, previous: Digest, stage: TraceStage, evidence: Digest) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "process-trace-event-v1")
            .u64("index", index)
            .field("previous", &previous.0)
            .text("stage", stage.as_str())
            .field("evidence", &evidence.0);
        Self {
            index,
            previous,
            stage,
            evidence,
            digest: encoder.digest(),
        }
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
    pub const fn stage(&self) -> TraceStage {
        self.stage
    }

    #[must_use]
    pub const fn evidence(&self) -> Digest {
        self.evidence
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn recompute_digest(&self) -> Digest {
        Self::new(self.index, self.previous, self.stage, self.evidence).digest
    }
}

/// Immutable trace for one runtime request.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessTrace {
    events: Vec<TraceEvent>,
}

impl ProcessTrace {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn append(&mut self, stage: TraceStage, evidence: Digest) {
        let previous = self.events.last().map_or(Digest::ZERO, TraceEvent::digest);
        self.events.push(TraceEvent::new(
            self.events.len() as u64,
            previous,
            stage,
            evidence,
        ));
    }

    #[must_use]
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.events.last().map_or(Digest::ZERO, TraceEvent::digest)
    }

    /// Verifies event indexes, predecessor links, and digests.
    pub fn verify(&self) -> Result<Digest, TraceError> {
        let mut previous = Digest::ZERO;
        for (position, event) in self.events.iter().enumerate() {
            if event.index() != position as u64 {
                return Err(TraceError::IndexMismatch {
                    expected: position as u64,
                    actual: event.index(),
                });
            }
            if event.previous() != previous {
                return Err(TraceError::PreviousMismatch {
                    expected: previous,
                    actual: event.previous(),
                });
            }
            if event.digest() != event.recompute_digest() {
                return Err(TraceError::DigestMismatch {
                    index: event.index(),
                });
            }
            previous = event.digest();
        }
        Ok(previous)
    }
}

/// Process trace corruption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceError {
    IndexMismatch { expected: u64, actual: u64 },
    PreviousMismatch { expected: Digest, actual: Digest },
    DigestMismatch { index: u64 },
}

impl Display for TraceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexMismatch { expected, actual } => {
                write!(formatter, "trace index {actual}, expected {expected}")
            }
            Self::PreviousMismatch { expected, actual } => {
                write!(formatter, "trace predecessor {actual}, expected {expected}")
            }
            Self::DigestMismatch { index } => write!(formatter, "trace digest mismatch at {index}"),
        }
    }
}

impl std::error::Error for TraceError {}

/// Successful end-to-end runtime result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessResult {
    intent: Intent,
    outcome: Outcome,
    evidence: ActuationEvidence,
    trace: ProcessTrace,
}

impl ProcessResult {
    #[must_use]
    pub const fn intent(&self) -> &Intent {
        &self.intent
    }

    #[must_use]
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    #[must_use]
    pub const fn evidence(&self) -> &ActuationEvidence {
        &self.evidence
    }

    #[must_use]
    pub const fn trace(&self) -> &ProcessTrace {
        &self.trace
    }
}

/// Typed refusal or infrastructure failure from the runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessError {
    RouteNotFound { route: String, trace: ProcessTrace },
    AdmissionRefused {
        violations: Vec<Violation>,
        trace: ProcessTrace,
    },
    Broker(BrokerError),
}

impl Display for ProcessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RouteNotFound { route, .. } => write!(formatter, "route `{route}` not found"),
            Self::AdmissionRefused { violations, .. } => {
                write!(formatter, "admission refused by {} rule(s)", violations.len())
            }
            Self::Broker(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<BrokerError> for ProcessError {
    fn from(value: BrokerError) -> Self {
        Self::Broker(value)
    }
}

/// Full deterministic process runtime.
#[derive(Clone, Debug)]
pub struct Runtime {
    router: Router,
    policy: AdmissionPolicy,
    broker: Broker,
}

impl Runtime {
    #[must_use]
    pub const fn new(router: Router, policy: AdmissionPolicy, broker: Broker) -> Self {
        Self {
            router,
            policy,
            broker,
        }
    }

    #[must_use]
    pub const fn router(&self) -> &Router {
        &self.router
    }

    #[must_use]
    pub const fn policy(&self) -> &AdmissionPolicy {
        &self.policy
    }

    #[must_use]
    pub const fn broker(&self) -> &Broker {
        &self.broker
    }

    #[must_use]
    pub const fn broker_mut(&mut self) -> &mut Broker {
        &mut self.broker
    }

    /// Runs one complete request through every transition.
    #[allow(clippy::too_many_arguments)]
    pub fn process<E: Executor>(
        &mut self,
        executor: &mut E,
        route_name: &str,
        observation: Observation,
        authority: AuthorityId,
        nonce: u64,
        payload: Vec<u8>,
    ) -> Result<ProcessResult, ProcessError> {
        let mut trace = ProcessTrace::new();
        trace.append(TraceStage::Parsed, observation.digest());

        let Some(route) = self.router.resolve(route_name).cloned() else {
            trace.append(TraceStage::Refused, self.router.digest());
            return Err(ProcessError::RouteNotFound {
                route: route_name.to_owned(),
                trace,
            });
        };
        let mut route_encoder = CanonicalEncoder::new();
        route_encoder
            .text("route", route.name())
            .text("capability", route.capability().as_str())
            .text("operation", route.operation().as_str());
        trace.append(TraceStage::Routed, route_encoder.digest());

        let admitted = match self.policy.evaluate(observation) {
            AdmissionDecision::Admitted(admitted) => admitted,
            AdmissionDecision::Refused(violations) => {
                let mut refusal = CanonicalEncoder::new();
                refusal.u64("violations", violations.len() as u64);
                for violation in &violations {
                    refusal
                        .text("rule", violation.rule())
                        .text("code", violation.code())
                        .text("detail", violation.detail());
                }
                trace.append(TraceStage::Refused, refusal.digest());
                return Err(ProcessError::AdmissionRefused { violations, trace });
            }
        };
        trace.append(TraceStage::Admitted, admitted.digest());

        let intent = Intent::construct(
            &admitted,
            route.capability().clone(),
            route.operation().clone(),
            authority,
            nonce,
            payload,
        );
        trace.append(TraceStage::Constructed, intent.digest());

        let evidence = self.broker.actuate(executor, &intent)?;
        if let Some(authorization) = evidence.authorization() {
            trace.append(TraceStage::Authorized, authorization.digest());
        }
        let outcome = self
            .broker
            .outcome(intent.digest())
            .cloned()
            .unwrap_or_else(|| Outcome::Failed {
                code: "OUTCOME_MISSING".to_owned(),
                detail: "completion receipt exists without retained outcome".to_owned(),
            });
        trace.append(TraceStage::Actuated, outcome.digest());
        trace.append(TraceStage::Receipted, evidence.completion().digest());
        Ok(ProcessResult {
            intent,
            outcome,
            evidence,
            trace,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessError, Route, Router, Runtime, TraceStage};
    use crate::broker::{Broker, Executor, PreflightRefusal};
    use crate::graph::{Capability, CapabilityGraph};
    use crate::model::{
        AuthorityId, CapabilityId, Intent, Observation, OperationId, Outcome, PolicyId, SubjectId,
    };
    use crate::policy::{AdmissionPolicy, Predicate, Rule};

    struct Echo;

    impl Executor for Echo {
        fn id(&self) -> &str {
            "echo"
        }

        fn preflight(&self, _intent: &Intent) -> Result<(), PreflightRefusal> {
            Ok(())
        }

        fn execute(&mut self, intent: &Intent) -> Outcome {
            Outcome::Applied {
                code: 200,
                output: intent.payload().to_vec(),
            }
        }
    }

    fn runtime() -> Runtime {
        let capability = CapabilityId::new("notify").unwrap();
        let operation = OperationId::new("send").unwrap();
        let authority = AuthorityId::new("operator").unwrap();
        let mut graph = CapabilityGraph::new();
        graph
            .insert(
                Capability::new(capability.clone())
                    .supports(operation.clone())
                    .allows(authority),
            )
            .unwrap();
        let broker = Broker::new("broker", graph, 10).unwrap();
        let mut router = Router::new();
        router.insert(Route::new("notifications.send", capability, operation));
        let policy = AdmissionPolicy::new(PolicyId::new("policy").unwrap(), 1).with_rule(
            Rule::new(
                "ready",
                Predicate::Equals {
                    key: "ready".to_owned(),
                    expected: true.into(),
                },
            ),
        );
        Runtime::new(router, policy, broker)
    }

    #[test]
    fn process_executes_complete_transition_chain() {
        let mut runtime = runtime();
        let mut executor = Echo;
        let mut observation = Observation::new(SubjectId::new("case-1").unwrap(), 1);
        observation.insert("ready", true).unwrap();
        let result = runtime
            .process(
                &mut executor,
                "notifications.send",
                observation,
                AuthorityId::new("operator").unwrap(),
                1,
                b"hello".to_vec(),
            )
            .unwrap();
        assert!(result.outcome().is_applied());
        assert_eq!(result.trace().verify().unwrap(), result.trace().head());
        assert_eq!(
            result
                .trace()
                .events()
                .iter()
                .map(|event| event.stage())
                .collect::<Vec<_>>(),
            [
                TraceStage::Parsed,
                TraceStage::Routed,
                TraceStage::Admitted,
                TraceStage::Constructed,
                TraceStage::Authorized,
                TraceStage::Actuated,
                TraceStage::Receipted,
            ]
        );
    }

    #[test]
    fn admission_refusal_has_no_actuation_receipt() {
        let mut runtime = runtime();
        let mut executor = Echo;
        let observation = Observation::new(SubjectId::new("case-1").unwrap(), 1);
        let error = runtime
            .process(
                &mut executor,
                "notifications.send",
                observation,
                AuthorityId::new("operator").unwrap(),
                1,
                Vec::new(),
            )
            .unwrap_err();
        assert!(matches!(error, ProcessError::AdmissionRefused { .. }));
        assert!(runtime.broker().completions().receipts().is_empty());
    }
}
