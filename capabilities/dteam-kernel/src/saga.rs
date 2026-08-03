//! Deterministic saga orchestration with retries, compensation, checkpoints, and replay.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{CapabilityId, OperationId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

macro_rules! saga_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, SagaError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(SagaError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

saga_id!(SagaId);
saga_id!(SagaDefinitionId);
saga_id!(SagaStepId);

/// One executable saga operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaOperation {
    capability: CapabilityId,
    operation: OperationId,
    payload: Vec<u8>,
}

impl SagaOperation {
    #[must_use]
    pub const fn new(capability: CapabilityId, operation: OperationId, payload: Vec<u8>) -> Self {
        Self {
            capability,
            operation,
            payload,
        }
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
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "saga-operation-v1")
            .text("capability", self.capability.as_str())
            .text("operation", self.operation.as_str())
            .field("payload", &self.payload);
        encoder.digest()
    }
}

/// One forward step and optional compensating operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaStep {
    id: SagaStepId,
    forward: SagaOperation,
    compensation: Option<SagaOperation>,
    maximum_attempts: u32,
    retry_delay: u64,
    timeout: u64,
}

impl SagaStep {
    #[must_use]
    pub fn new(id: SagaStepId, forward: SagaOperation) -> Self {
        Self {
            id,
            forward,
            compensation: None,
            maximum_attempts: 1,
            retry_delay: 0,
            timeout: u64::MAX,
        }
    }

    #[must_use]
    pub fn compensate_with(mut self, operation: SagaOperation) -> Self {
        self.compensation = Some(operation);
        self
    }

    #[must_use]
    pub fn retry(mut self, maximum_attempts: u32, delay: u64) -> Self {
        self.maximum_attempts = maximum_attempts.max(1);
        self.retry_delay = delay;
        self
    }

    #[must_use]
    pub const fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub const fn id(&self) -> &SagaStepId {
        &self.id
    }

    #[must_use]
    pub const fn forward(&self) -> &SagaOperation {
        &self.forward
    }

    #[must_use]
    pub const fn compensation(&self) -> Option<&SagaOperation> {
        self.compensation.as_ref()
    }

    #[must_use]
    pub const fn maximum_attempts(&self) -> u32 {
        self.maximum_attempts
    }

    #[must_use]
    pub const fn retry_delay(&self) -> u64 {
        self.retry_delay
    }

    #[must_use]
    pub const fn timeout_value(&self) -> u64 {
        self.timeout
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "saga-step-v1")
            .text("id", self.id.as_str())
            .field("forward", &self.forward.digest().0)
            .u64("maximum-attempts", u64::from(self.maximum_attempts))
            .u64("retry-delay", self.retry_delay)
            .u64("timeout", self.timeout);
        match &self.compensation {
            Some(operation) => {
                encoder
                    .boolean("has-compensation", true)
                    .field("compensation", &operation.digest().0);
            }
            None => {
                encoder.boolean("has-compensation", false);
            }
        }
        encoder.digest()
    }
}

/// Ordered immutable saga definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaDefinition {
    id: SagaDefinitionId,
    version: u64,
    steps: Vec<SagaStep>,
    digest: Digest,
}

impl SagaDefinition {
    pub fn new(
        id: SagaDefinitionId,
        version: u64,
        steps: Vec<SagaStep>,
    ) -> Result<Self, SagaError> {
        if steps.is_empty() {
            return Err(SagaError::EmptyDefinition);
        }
        let mut ids = BTreeSet::new();
        for step in &steps {
            if !ids.insert(step.id().clone()) {
                return Err(SagaError::DuplicateStep(step.id().clone()));
            }
            if step.timeout_value() == 0 {
                return Err(SagaError::ZeroStepTimeout(step.id().clone()));
            }
        }
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "saga-definition-v1")
            .text("id", id.as_str())
            .u64("version", version)
            .u64("step-count", steps.len() as u64);
        for step in &steps {
            encoder
                .text("step", step.id().as_str())
                .field("step-digest", &step.digest().0);
        }
        Ok(Self {
            id,
            version,
            steps,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> &SagaDefinitionId {
        &self.id
    }

    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub fn steps(&self) -> &[SagaStep] {
        &self.steps
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Executor result for one forward or compensation operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SagaOperationResult {
    Succeeded {
        output: Vec<u8>,
        elapsed: u64,
    },
    Retryable {
        code: String,
        detail: String,
        elapsed: u64,
    },
    Failed {
        code: String,
        detail: String,
        elapsed: u64,
    },
    Refused {
        code: String,
        detail: String,
        elapsed: u64,
    },
}

impl SagaOperationResult {
    #[must_use]
    pub const fn elapsed(&self) -> u64 {
        match self {
            Self::Succeeded { elapsed, .. }
            | Self::Retryable { elapsed, .. }
            | Self::Failed { elapsed, .. }
            | Self::Refused { elapsed, .. } => *elapsed,
        }
    }

    #[must_use]
    pub const fn succeeded(&self) -> bool {
        matches!(self, Self::Succeeded { .. })
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Succeeded { output, elapsed } => {
                encoder
                    .text("result", "succeeded")
                    .field("output", output)
                    .u64("elapsed", *elapsed);
            }
            Self::Retryable {
                code,
                detail,
                elapsed,
            } => {
                encoder
                    .text("result", "retryable")
                    .text("code", code)
                    .text("detail", detail)
                    .u64("elapsed", *elapsed);
            }
            Self::Failed {
                code,
                detail,
                elapsed,
            } => {
                encoder
                    .text("result", "failed")
                    .text("code", code)
                    .text("detail", detail)
                    .u64("elapsed", *elapsed);
            }
            Self::Refused {
                code,
                detail,
                elapsed,
            } => {
                encoder
                    .text("result", "refused")
                    .text("code", code)
                    .text("detail", detail)
                    .u64("elapsed", *elapsed);
            }
        }
    }
}

/// Execution adapter. It receives stable saga and step identities for idempotency.
pub trait SagaExecutor {
    fn execute(
        &mut self,
        saga: &SagaId,
        step: &SagaStep,
        attempt: u32,
        input: &[u8],
    ) -> SagaOperationResult;

    fn compensate(
        &mut self,
        saga: &SagaId,
        step: &SagaStep,
        attempt: u32,
        forward_output: &[u8],
    ) -> SagaOperationResult;
}

/// Lifecycle state of a saga instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SagaState {
    Running,
    Compensating,
    Completed,
    Compensated,
    Failed,
    CompensationFailed,
}

impl SagaState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Compensating => "compensating",
            Self::Completed => "completed",
            Self::Compensated => "compensated",
            Self::Failed => "failed",
            Self::CompensationFailed => "compensation-failed",
        }
    }
}

/// Per-step lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SagaStepState {
    Pending,
    Succeeded,
    Failed,
    Compensated,
    CompensationFailed,
}

impl SagaStepState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Compensated => "compensated",
            Self::CompensationFailed => "compensation-failed",
        }
    }
}

/// One immutable saga transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SagaAction {
    Started {
        input: Vec<u8>,
    },
    StepAttempted {
        step: SagaStepId,
        attempt: u32,
        result: SagaOperationResult,
    },
    CompensationAttempted {
        step: SagaStepId,
        attempt: u32,
        result: SagaOperationResult,
    },
    StateChanged {
        from: SagaState,
        to: SagaState,
    },
}

impl SagaAction {
    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Started { input } => {
                encoder.text("action", "started").field("input", input);
            }
            Self::StepAttempted {
                step,
                attempt,
                result,
            } => {
                encoder
                    .text("action", "step-attempted")
                    .text("step", step.as_str())
                    .u64("attempt", u64::from(*attempt));
                result.encode(encoder);
            }
            Self::CompensationAttempted {
                step,
                attempt,
                result,
            } => {
                encoder
                    .text("action", "compensation-attempted")
                    .text("step", step.as_str())
                    .u64("attempt", u64::from(*attempt));
                result.encode(encoder);
            }
            Self::StateChanged { from, to } => {
                encoder
                    .text("action", "state-changed")
                    .text("from", from.as_str())
                    .text("to", to.as_str());
            }
        }
    }
}

/// Immutable saga receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaReceipt {
    index: u64,
    previous: Digest,
    saga: SagaId,
    definition: Digest,
    action: SagaAction,
    state_digest: Digest,
    digest: Digest,
}

impl SagaReceipt {
    fn manufacture(
        index: u64,
        previous: Digest,
        saga: SagaId,
        definition: Digest,
        action: SagaAction,
        state_digest: Digest,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "saga-receipt-v1")
            .u64("index", index)
            .field("previous", &previous.0)
            .text("saga", saga.as_str())
            .field("definition", &definition.0);
        action.encode(&mut encoder);
        encoder.field("state", &state_digest.0);
        Self {
            index,
            previous,
            saga,
            definition,
            action,
            state_digest,
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
    pub const fn saga(&self) -> &SagaId {
        &self.saga
    }

    #[must_use]
    pub const fn definition(&self) -> Digest {
        self.definition
    }

    #[must_use]
    pub const fn action(&self) -> &SagaAction {
        &self.action
    }

    #[must_use]
    pub const fn state_digest(&self) -> Digest {
        self.state_digest
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Mutable resumable saga instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaInstance {
    id: SagaId,
    definition: Digest,
    state: SagaState,
    input: Vec<u8>,
    step_states: BTreeMap<SagaStepId, SagaStepState>,
    outputs: BTreeMap<SagaStepId, Vec<u8>>,
    next_step: usize,
    receipts: Vec<SagaReceipt>,
}

impl SagaInstance {
    fn new(id: SagaId, definition: &SagaDefinition, input: Vec<u8>) -> Self {
        let step_states = definition
            .steps()
            .iter()
            .map(|step| (step.id().clone(), SagaStepState::Pending))
            .collect();
        let mut instance = Self {
            id,
            definition: definition.digest(),
            state: SagaState::Running,
            input: input.clone(),
            step_states,
            outputs: BTreeMap::new(),
            next_step: 0,
            receipts: Vec::new(),
        };
        instance.append(SagaAction::Started { input });
        instance
    }

    #[must_use]
    pub const fn id(&self) -> &SagaId {
        &self.id
    }

    #[must_use]
    pub const fn state(&self) -> SagaState {
        self.state
    }

    #[must_use]
    pub fn step_state(&self, step: &SagaStepId) -> Option<SagaStepState> {
        self.step_states.get(step).copied()
    }

    #[must_use]
    pub fn output(&self, step: &SagaStepId) -> Option<&[u8]> {
        self.outputs.get(step).map(Vec::as_slice)
    }

    #[must_use]
    pub fn receipts(&self) -> &[SagaReceipt] {
        &self.receipts
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts
            .last()
            .map_or(Digest::ZERO, SagaReceipt::digest)
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        saga_state_digest(
            &self.id,
            self.definition,
            self.state,
            &self.input,
            &self.step_states,
            &self.outputs,
            self.next_step,
        )
    }

    fn append(&mut self, action: SagaAction) {
        let receipt = SagaReceipt::manufacture(
            self.receipts.len() as u64,
            self.head(),
            self.id.clone(),
            self.definition,
            action,
            self.digest(),
        );
        self.receipts.push(receipt);
    }

    fn transition(&mut self, state: SagaState) {
        let previous = self.state;
        self.state = state;
        self.append(SagaAction::StateChanged {
            from: previous,
            to: state,
        });
    }
}

/// Saga start, execution, or replay failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SagaError {
    EmptyIdentifier(&'static str),
    EmptyDefinition,
    DuplicateStep(SagaStepId),
    ZeroStepTimeout(SagaStepId),
    DuplicateSaga(SagaId),
    SagaMissing(SagaId),
    DefinitionMismatch {
        saga: SagaId,
    },
    SagaNotRunnable {
        saga: SagaId,
        state: SagaState,
    },
    ReceiptIndex {
        expected: u64,
        actual: u64,
    },
    ReceiptPrevious {
        expected: Digest,
        actual: Digest,
    },
    ReceiptState {
        index: u64,
        expected: Digest,
        actual: Digest,
    },
    ReceiptDigest {
        index: u64,
    },
}

impl Display for SagaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::EmptyDefinition => formatter.write_str("saga definition must contain steps"),
            Self::DuplicateStep(step) => write!(formatter, "duplicate saga step `{step}`"),
            Self::ZeroStepTimeout(step) => write!(formatter, "saga step `{step}` timeout is zero"),
            Self::DuplicateSaga(saga) => write!(formatter, "saga `{saga}` already exists"),
            Self::SagaMissing(saga) => write!(formatter, "saga `{saga}` is missing"),
            Self::DefinitionMismatch { saga } => {
                write!(formatter, "saga `{saga}` definition digest changed")
            }
            Self::SagaNotRunnable { saga, state } => write!(
                formatter,
                "saga `{saga}` is {}, not runnable",
                state.as_str()
            ),
            Self::ReceiptIndex { expected, actual } => {
                write!(
                    formatter,
                    "saga receipt index {actual}, expected {expected}"
                )
            }
            Self::ReceiptPrevious { expected, actual } => {
                write!(formatter, "saga predecessor {actual}, expected {expected}")
            }
            Self::ReceiptState {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "saga receipt {index} state {actual}, expected {expected}"
            ),
            Self::ReceiptDigest { index } => {
                write!(formatter, "saga receipt {index} digest mismatch")
            }
        }
    }
}

impl std::error::Error for SagaError {}

/// Synchronous deterministic saga coordinator with resumable instances.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SagaCoordinator {
    instances: BTreeMap<SagaId, SagaInstance>,
}

impl SagaCoordinator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts exactly once. Reusing the id with the same definition returns the existing instance.
    pub fn start(
        &mut self,
        id: SagaId,
        definition: &SagaDefinition,
        input: Vec<u8>,
    ) -> Result<&SagaInstance, SagaError> {
        if self.instances.contains_key(&id) {
            let matches_existing = {
                let existing = &self.instances[&id];
                existing.definition == definition.digest() && existing.input == input
            };
            if matches_existing {
                return Ok(&self.instances[&id]);
            }
            return Err(SagaError::DuplicateSaga(id));
        }
        self.instances
            .insert(id.clone(), SagaInstance::new(id.clone(), definition, input));
        Ok(&self.instances[&id])
    }

    #[must_use]
    pub fn instance(&self, id: &SagaId) -> Option<&SagaInstance> {
        self.instances.get(id)
    }

    /// Executes or resumes a saga to a terminal state.
    pub fn run<E: SagaExecutor>(
        &mut self,
        id: &SagaId,
        definition: &SagaDefinition,
        executor: &mut E,
    ) -> Result<&SagaInstance, SagaError> {
        let instance = self
            .instances
            .get_mut(id)
            .ok_or_else(|| SagaError::SagaMissing(id.clone()))?;
        if instance.definition != definition.digest() {
            return Err(SagaError::DefinitionMismatch { saga: id.clone() });
        }
        if !matches!(instance.state, SagaState::Running | SagaState::Compensating) {
            return Err(SagaError::SagaNotRunnable {
                saga: id.clone(),
                state: instance.state,
            });
        }

        if instance.state == SagaState::Compensating {
            compensate(instance, definition, executor);
            return Ok(self.instances.get(id).expect("instance remains present"));
        }

        while instance.next_step < definition.steps().len() {
            let step = &definition.steps()[instance.next_step];
            let input = if instance.next_step == 0 {
                instance.input.clone()
            } else {
                let previous = definition.steps()[instance.next_step - 1].id();
                instance.outputs.get(previous).cloned().unwrap_or_default()
            };
            let mut attempt = 1_u32;
            let result = loop {
                let candidate = executor.execute(id, step, attempt, &input);
                let timed_out = candidate.elapsed() > step.timeout_value();
                let normalized = if timed_out {
                    SagaOperationResult::Failed {
                        code: "STEP_TIMEOUT".to_owned(),
                        detail: format!(
                            "elapsed {} exceeds timeout {}",
                            candidate.elapsed(),
                            step.timeout_value()
                        ),
                        elapsed: candidate.elapsed(),
                    }
                } else {
                    candidate
                };
                instance.append(SagaAction::StepAttempted {
                    step: step.id().clone(),
                    attempt,
                    result: normalized.clone(),
                });
                match normalized {
                    SagaOperationResult::Retryable { .. } if attempt < step.maximum_attempts() => {
                        attempt += 1;
                    }
                    final_result => break final_result,
                }
            };
            match result {
                SagaOperationResult::Succeeded { output, .. } => {
                    instance
                        .step_states
                        .insert(step.id().clone(), SagaStepState::Succeeded);
                    instance.outputs.insert(step.id().clone(), output);
                    instance.next_step += 1;
                }
                _ => {
                    instance
                        .step_states
                        .insert(step.id().clone(), SagaStepState::Failed);
                    instance.transition(SagaState::Compensating);
                    compensate(instance, definition, executor);
                    return Ok(self.instances.get(id).expect("instance remains present"));
                }
            }
        }
        instance.transition(SagaState::Completed);
        Ok(self.instances.get(id).expect("instance remains present"))
    }

    /// Verifies every receipt chain and final state snapshot.
    pub fn verify(
        &self,
        id: &SagaId,
        definition: &SagaDefinition,
    ) -> Result<SagaVerification, SagaError> {
        let instance = self
            .instances
            .get(id)
            .ok_or_else(|| SagaError::SagaMissing(id.clone()))?;
        if instance.definition != definition.digest() {
            return Err(SagaError::DefinitionMismatch { saga: id.clone() });
        }
        let mut previous = Digest::ZERO;
        for (index, receipt) in instance.receipts().iter().enumerate() {
            if receipt.index() != index as u64 {
                return Err(SagaError::ReceiptIndex {
                    expected: index as u64,
                    actual: receipt.index(),
                });
            }
            if receipt.previous() != previous {
                return Err(SagaError::ReceiptPrevious {
                    expected: previous,
                    actual: receipt.previous(),
                });
            }
            let expected = SagaReceipt::manufacture(
                receipt.index(),
                receipt.previous(),
                receipt.saga().clone(),
                receipt.definition(),
                receipt.action().clone(),
                receipt.state_digest(),
            );
            if expected.digest() != receipt.digest() {
                return Err(SagaError::ReceiptDigest {
                    index: receipt.index(),
                });
            }
            previous = receipt.digest();
        }
        if instance
            .receipts()
            .last()
            .is_some_and(|receipt| receipt.state_digest() != instance.digest())
        {
            return Err(SagaError::ReceiptState {
                index: instance.receipts().len() as u64,
                expected: instance.digest(),
                actual: instance
                    .receipts()
                    .last()
                    .expect("checked receipt")
                    .state_digest(),
            });
        }
        Ok(SagaVerification {
            state: instance.state(),
            steps: instance.step_states.len(),
            receipts: instance.receipts().len(),
            state_digest: instance.digest(),
            head: instance.head(),
        })
    }
}

fn compensate<E: SagaExecutor>(
    instance: &mut SagaInstance,
    definition: &SagaDefinition,
    executor: &mut E,
) {
    let completed = definition.steps()[..instance.next_step]
        .iter()
        .rev()
        .collect::<Vec<_>>();
    for step in completed {
        let Some(_compensation) = step.compensation() else {
            continue;
        };
        let forward_output = instance.outputs.get(step.id()).cloned().unwrap_or_default();
        let mut attempt = 1_u32;
        let result = loop {
            let candidate = executor.compensate(instance.id(), step, attempt, &forward_output);
            instance.append(SagaAction::CompensationAttempted {
                step: step.id().clone(),
                attempt,
                result: candidate.clone(),
            });
            match candidate {
                SagaOperationResult::Retryable { .. } if attempt < step.maximum_attempts() => {
                    attempt += 1;
                }
                final_result => break final_result,
            }
        };
        if result.succeeded() {
            instance
                .step_states
                .insert(step.id().clone(), SagaStepState::Compensated);
        } else {
            instance
                .step_states
                .insert(step.id().clone(), SagaStepState::CompensationFailed);
            instance.transition(SagaState::CompensationFailed);
            return;
        }
    }
    instance.transition(SagaState::Compensated);
}

fn saga_state_digest(
    id: &SagaId,
    definition: Digest,
    state: SagaState,
    input: &[u8],
    step_states: &BTreeMap<SagaStepId, SagaStepState>,
    outputs: &BTreeMap<SagaStepId, Vec<u8>>,
    next_step: usize,
) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "saga-state-v1")
        .text("id", id.as_str())
        .field("definition", &definition.0)
        .text("state", state.as_str())
        .field("input", input)
        .u64("next-step", next_step as u64)
        .u64("step-count", step_states.len() as u64);
    for (step, step_state) in step_states {
        encoder
            .text("step", step.as_str())
            .text("step-state", step_state.as_str());
        match outputs.get(step) {
            Some(output) => {
                encoder.boolean("has-output", true).field("output", output);
            }
            None => {
                encoder.boolean("has-output", false);
            }
        }
    }
    encoder.digest()
}

/// Aggregate saga verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaVerification {
    state: SagaState,
    steps: usize,
    receipts: usize,
    state_digest: Digest,
    head: Digest,
}

impl SagaVerification {
    #[must_use]
    pub const fn state(&self) -> SagaState {
        self.state
    }

    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }

    #[must_use]
    pub const fn receipts(&self) -> usize {
        self.receipts
    }

    #[must_use]
    pub const fn state_digest(&self) -> Digest {
        self.state_digest
    }

    #[must_use]
    pub const fn head(&self) -> Digest {
        self.head
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SagaCoordinator, SagaDefinition, SagaDefinitionId, SagaExecutor, SagaId, SagaOperation,
        SagaOperationResult, SagaState, SagaStep, SagaStepId, SagaStepState,
    };
    use crate::model::{CapabilityId, OperationId};

    fn operation(name: &str) -> SagaOperation {
        SagaOperation::new(
            CapabilityId::new("orders").unwrap(),
            OperationId::new(name).unwrap(),
            name.as_bytes().to_vec(),
        )
    }

    fn definition() -> SagaDefinition {
        SagaDefinition::new(
            SagaDefinitionId::new("checkout").unwrap(),
            1,
            vec![
                SagaStep::new(SagaStepId::new("reserve").unwrap(), operation("reserve"))
                    .compensate_with(operation("release"))
                    .retry(2, 0),
                SagaStep::new(SagaStepId::new("charge").unwrap(), operation("charge"))
                    .compensate_with(operation("refund"))
                    .retry(2, 0),
                SagaStep::new(SagaStepId::new("ship").unwrap(), operation("ship")).retry(2, 0),
            ],
        )
        .unwrap()
    }

    struct Successful;

    impl SagaExecutor for Successful {
        fn execute(
            &mut self,
            _saga: &SagaId,
            step: &SagaStep,
            _attempt: u32,
            _input: &[u8],
        ) -> SagaOperationResult {
            SagaOperationResult::Succeeded {
                output: step.id().as_str().as_bytes().to_vec(),
                elapsed: 1,
            }
        }

        fn compensate(
            &mut self,
            _saga: &SagaId,
            _step: &SagaStep,
            _attempt: u32,
            _forward_output: &[u8],
        ) -> SagaOperationResult {
            SagaOperationResult::Succeeded {
                output: Vec::new(),
                elapsed: 1,
            }
        }
    }

    #[test]
    fn successful_saga_completes_and_verifies() {
        let definition = definition();
        let id = SagaId::new("order-1").unwrap();
        let mut coordinator = SagaCoordinator::new();
        coordinator
            .start(id.clone(), &definition, b"order".to_vec())
            .unwrap();
        coordinator.run(&id, &definition, &mut Successful).unwrap();
        assert_eq!(
            coordinator.instance(&id).unwrap().state(),
            SagaState::Completed
        );
        assert_eq!(coordinator.verify(&id, &definition).unwrap().steps(), 3);
    }

    struct FailShipping;

    impl SagaExecutor for FailShipping {
        fn execute(
            &mut self,
            _saga: &SagaId,
            step: &SagaStep,
            _attempt: u32,
            _input: &[u8],
        ) -> SagaOperationResult {
            if step.id().as_str() == "ship" {
                SagaOperationResult::Failed {
                    code: "SHIP_FAILED".to_owned(),
                    detail: "carrier unavailable".to_owned(),
                    elapsed: 1,
                }
            } else {
                SagaOperationResult::Succeeded {
                    output: step.id().as_str().as_bytes().to_vec(),
                    elapsed: 1,
                }
            }
        }

        fn compensate(
            &mut self,
            _saga: &SagaId,
            _step: &SagaStep,
            _attempt: u32,
            _forward_output: &[u8],
        ) -> SagaOperationResult {
            SagaOperationResult::Succeeded {
                output: Vec::new(),
                elapsed: 1,
            }
        }
    }

    #[test]
    fn failed_forward_path_compensates_reverse_order() {
        let definition = definition();
        let id = SagaId::new("order-1").unwrap();
        let mut coordinator = SagaCoordinator::new();
        coordinator
            .start(id.clone(), &definition, b"order".to_vec())
            .unwrap();
        coordinator
            .run(&id, &definition, &mut FailShipping)
            .unwrap();
        let instance = coordinator.instance(&id).unwrap();
        assert_eq!(instance.state(), SagaState::Compensated);
        assert_eq!(
            instance.step_state(&SagaStepId::new("charge").unwrap()),
            Some(SagaStepState::Compensated)
        );
        assert_eq!(
            instance.step_state(&SagaStepId::new("reserve").unwrap()),
            Some(SagaStepState::Compensated)
        );
    }

    struct Flaky {
        calls: u32,
    }

    impl SagaExecutor for Flaky {
        fn execute(
            &mut self,
            _saga: &SagaId,
            step: &SagaStep,
            attempt: u32,
            _input: &[u8],
        ) -> SagaOperationResult {
            self.calls += 1;
            if step.id().as_str() == "reserve" && attempt == 1 {
                SagaOperationResult::Retryable {
                    code: "TRANSIENT".to_owned(),
                    detail: "retry".to_owned(),
                    elapsed: 1,
                }
            } else {
                SagaOperationResult::Succeeded {
                    output: Vec::new(),
                    elapsed: 1,
                }
            }
        }

        fn compensate(
            &mut self,
            _saga: &SagaId,
            _step: &SagaStep,
            _attempt: u32,
            _forward_output: &[u8],
        ) -> SagaOperationResult {
            SagaOperationResult::Succeeded {
                output: Vec::new(),
                elapsed: 1,
            }
        }
    }

    #[test]
    fn retryable_step_retries_with_receipts() {
        let definition = definition();
        let id = SagaId::new("order-1").unwrap();
        let mut coordinator = SagaCoordinator::new();
        coordinator
            .start(id.clone(), &definition, Vec::new())
            .unwrap();
        let mut executor = Flaky { calls: 0 };
        coordinator.run(&id, &definition, &mut executor).unwrap();
        assert_eq!(executor.calls, 4);
        assert!(coordinator
            .instance(&id)
            .unwrap()
            .receipts()
            .iter()
            .filter(|receipt| matches!(receipt.action(), super::SagaAction::StepAttempted { step, .. } if step.as_str() == "reserve"))
            .count()
            >= 2);
    }
}
