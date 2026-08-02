//! Deterministic knowledge hooks that manufacture intents but never actuate them.

use crate::decision::{Condition, DecisionEffect, DecisionOutcome, DecisionRule, DecisionTable};
use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{
    AdmittedObservation, AuthorityId, CapabilityId, FactValue, Intent, OperationId, SubjectId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// External or derived event presented to hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookEvent {
    topic: String,
    sequence: u64,
    subject: SubjectId,
    facts: BTreeMap<String, FactValue>,
    payload: Vec<u8>,
    digest: Digest,
}

impl HookEvent {
    /// Creates a hook event with no facts or payload.
    pub fn new(
        topic: impl Into<String>,
        sequence: u64,
        subject: SubjectId,
    ) -> Result<Self, HookError> {
        let topic = topic.into();
        if topic.trim().is_empty() {
            return Err(HookError::EmptyTopic);
        }
        let mut event = Self {
            topic,
            sequence,
            subject,
            facts: BTreeMap::new(),
            payload: Vec::new(),
            digest: Digest::ZERO,
        };
        event.digest = event.recompute_digest();
        Ok(event)
    }

    /// Adds or replaces a fact and recomputes identity.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Option<FactValue> {
        let previous = self.facts.insert(key.into(), value.into());
        self.digest = self.recompute_digest();
        previous
    }

    /// Sets the opaque event payload and recomputes identity.
    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
        self.digest = self.recompute_digest();
    }

    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn subject(&self) -> &SubjectId {
        &self.subject
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn fact(&self, key: &str) -> Option<&FactValue> {
        self.facts.get(key)
    }

    pub fn facts(&self) -> impl ExactSizeIterator<Item = (&str, &FactValue)> {
        self.facts
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "hook-event-v1")
            .text("topic", &self.topic)
            .u64("sequence", self.sequence)
            .text("subject", self.subject.as_str())
            .u64("fact-count", self.facts.len() as u64);
        for (key, value) in &self.facts {
            encoder.text("fact-key", key);
            value.encode(&mut encoder, "fact-type");
        }
        encoder.field("payload", &self.payload);
        encoder.digest()
    }
}

/// Payload construction policy for a manufactured intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayloadTemplate {
    Empty,
    EventPayload,
    Literal(Vec<u8>),
    FactText { key: String },
    CanonicalEvent,
}

impl PayloadTemplate {
    fn render(&self, event: &HookEvent) -> Result<Vec<u8>, HookError> {
        match self {
            Self::Empty => Ok(Vec::new()),
            Self::EventPayload => Ok(event.payload.clone()),
            Self::Literal(value) => Ok(value.clone()),
            Self::FactText { key } => match event.fact(key) {
                Some(FactValue::Text(value)) => Ok(value.as_bytes().to_vec()),
                Some(value) => Err(HookError::PayloadType {
                    key: key.clone(),
                    actual: format!("{value:?}"),
                }),
                None => Err(HookError::PayloadFactMissing(key.clone())),
            },
            Self::CanonicalEvent => {
                let mut encoder = CanonicalEncoder::new();
                encoder
                    .field("event", &event.digest().0)
                    .text("topic", event.topic())
                    .u64("sequence", event.sequence())
                    .text("subject", event.subject().as_str())
                    .field("payload", event.payload());
                Ok(encoder.finish())
            }
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Empty => {
                encoder.text("payload-template", "empty");
            }
            Self::EventPayload => {
                encoder.text("payload-template", "event-payload");
            }
            Self::Literal(value) => {
                encoder
                    .text("payload-template", "literal")
                    .field("literal", value);
            }
            Self::FactText { key } => {
                encoder
                    .text("payload-template", "fact-text")
                    .text("key", key);
            }
            Self::CanonicalEvent => {
                encoder.text("payload-template", "canonical-event");
            }
        }
    }
}

/// Intent template emitted by a matching hook.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentTemplate {
    capability: CapabilityId,
    operation: OperationId,
    authority: AuthorityId,
    payload: PayloadTemplate,
}

impl IntentTemplate {
    #[must_use]
    pub const fn new(
        capability: CapabilityId,
        operation: OperationId,
        authority: AuthorityId,
        payload: PayloadTemplate,
    ) -> Self {
        Self {
            capability,
            operation,
            authority,
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
    pub const fn authority(&self) -> &AuthorityId {
        &self.authority
    }

    #[must_use]
    pub const fn payload(&self) -> &PayloadTemplate {
        &self.payload
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        encoder
            .text("capability", self.capability.as_str())
            .text("operation", self.operation.as_str())
            .text("authority", self.authority.as_str());
        self.payload.encode(encoder);
    }
}

/// One declarative hook.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hook {
    id: String,
    topic: String,
    priority: i32,
    conditions: Vec<Condition>,
    intents: Vec<IntentTemplate>,
    stop_propagation: bool,
}

impl Hook {
    pub fn new(id: impl Into<String>, topic: impl Into<String>) -> Result<Self, HookError> {
        let id = id.into();
        let topic = topic.into();
        if id.trim().is_empty() {
            return Err(HookError::EmptyHookId);
        }
        if topic.trim().is_empty() {
            return Err(HookError::EmptyTopic);
        }
        Ok(Self {
            id,
            topic,
            priority: 0,
            conditions: Vec::new(),
            intents: Vec::new(),
            stop_propagation: false,
        })
    }

    #[must_use]
    pub const fn priority(mut self, value: i32) -> Self {
        self.priority = value;
        self
    }

    #[must_use]
    pub fn when(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    #[must_use]
    pub fn emits(mut self, template: IntentTemplate) -> Self {
        self.intents.push(template);
        self
    }

    #[must_use]
    pub const fn stop_propagation(mut self, value: bool) -> Self {
        self.stop_propagation = value;
        self
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    #[must_use]
    pub const fn priority_value(&self) -> i32 {
        self.priority
    }

    #[must_use]
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    #[must_use]
    pub fn intents(&self) -> &[IntentTemplate] {
        &self.intents
    }

    #[must_use]
    pub const fn stops_propagation(&self) -> bool {
        self.stop_propagation
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "hook-v1")
            .text("id", &self.id)
            .text("topic", &self.topic)
            .i64("priority", i64::from(self.priority))
            .boolean("stop-propagation", self.stop_propagation)
            .u64("condition-count", self.conditions.len() as u64);
        for condition in &self.conditions {
            let mut table = DecisionTable::new();
            table.push(
                DecisionRule::new("condition", 0, DecisionEffect::Allow)
                    .when(condition.clone()),
            );
            encoder.field("condition", &table.digest().0);
        }
        encoder.u64("intent-count", self.intents.len() as u64);
        for template in &self.intents {
            template.encode(&mut encoder);
        }
        encoder.digest()
    }
}

/// Static hook-registry defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HookLint {
    DuplicateId(String),
    NoIntents(String),
    Shadowed { hook: String, by: String },
    DuplicateEmission {
        hook: String,
        capability: CapabilityId,
        operation: OperationId,
    },
}

/// Hook evaluation evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookEvaluation {
    hook: String,
    topic_matched: bool,
    conditions_matched: bool,
    failures: Vec<String>,
    emitted: usize,
    digest: Digest,
}

impl HookEvaluation {
    #[must_use]
    pub fn hook(&self) -> &str {
        &self.hook
    }

    #[must_use]
    pub const fn topic_matched(&self) -> bool {
        self.topic_matched
    }

    #[must_use]
    pub const fn conditions_matched(&self) -> bool {
        self.conditions_matched
    }

    #[must_use]
    pub fn failures(&self) -> &[String] {
        &self.failures
    }

    #[must_use]
    pub const fn emitted(&self) -> usize {
        self.emitted
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Complete pure hook result. It contains intents, never outcomes or receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookReport {
    event_digest: Digest,
    intents: Vec<Intent>,
    evaluations: Vec<HookEvaluation>,
    stopped_by: Option<String>,
    digest: Digest,
}

impl HookReport {
    #[must_use]
    pub const fn event_digest(&self) -> Digest {
        self.event_digest
    }

    #[must_use]
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }

    #[must_use]
    pub fn evaluations(&self) -> &[HookEvaluation] {
        &self.evaluations
    }

    #[must_use]
    pub fn stopped_by(&self) -> Option<&str> {
        self.stopped_by.as_deref()
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Hook construction or payload error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HookError {
    EmptyHookId,
    EmptyTopic,
    DuplicateHook(String),
    SubjectMismatch { event: SubjectId, admitted: SubjectId },
    PayloadFactMissing(String),
    PayloadType { key: String, actual: String },
    NonceOverflow,
}

impl Display for HookError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyHookId => formatter.write_str("hook id must not be empty"),
            Self::EmptyTopic => formatter.write_str("hook topic must not be empty"),
            Self::DuplicateHook(id) => write!(formatter, "duplicate hook `{id}`"),
            Self::SubjectMismatch { event, admitted } => write!(
                formatter,
                "hook event subject `{event}` differs from admitted subject `{admitted}`"
            ),
            Self::PayloadFactMissing(key) => {
                write!(formatter, "payload fact `{key}` is absent")
            }
            Self::PayloadType { key, actual } => write!(
                formatter,
                "payload fact `{key}` is {actual}, expected text"
            ),
            Self::NonceOverflow => formatter.write_str("hook intent nonce overflow"),
        }
    }
}

impl std::error::Error for HookError {}

/// Priority-ordered registry of pure hooks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HookRegistry {
    hooks: BTreeMap<String, Hook>,
}

impl HookRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, hook: Hook) -> Result<(), HookError> {
        if self.hooks.contains_key(hook.id()) {
            return Err(HookError::DuplicateHook(hook.id().to_owned()));
        }
        self.hooks.insert(hook.id().to_owned(), hook);
        Ok(())
    }

    pub fn hooks(&self) -> impl ExactSizeIterator<Item = &Hook> {
        self.hooks.values()
    }

    /// Finds empty hooks, duplicate emissions, and unconditional propagation shadows.
    #[must_use]
    pub fn lint(&self) -> Vec<HookLint> {
        let mut findings = Vec::new();
        let mut ordered = self.hooks.values().collect::<Vec<_>>();
        ordered.sort_by_key(|hook| (std::cmp::Reverse(hook.priority_value()), hook.id()));
        for hook in &ordered {
            if hook.intents().is_empty() {
                findings.push(HookLint::NoIntents(hook.id().to_owned()));
            }
            let mut emissions = BTreeSet::new();
            for template in hook.intents() {
                let key = (template.capability().clone(), template.operation().clone());
                if !emissions.insert(key.clone()) {
                    findings.push(HookLint::DuplicateEmission {
                        hook: hook.id().to_owned(),
                        capability: key.0,
                        operation: key.1,
                    });
                }
            }
        }
        for (index, hook) in ordered.iter().enumerate() {
            if hook.stops_propagation() && hook.conditions().is_empty() {
                for shadowed in ordered.iter().skip(index + 1) {
                    if shadowed.topic() == hook.topic()
                        && shadowed.priority_value() <= hook.priority_value()
                    {
                        findings.push(HookLint::Shadowed {
                            hook: shadowed.id().to_owned(),
                            by: hook.id().to_owned(),
                        });
                    }
                }
            }
        }
        findings
    }

    /// Manufactures all matching intents in priority order and deduplicates exact identities.
    pub fn manufacture(
        &self,
        event: &HookEvent,
        admitted: &AdmittedObservation,
        nonce_base: u64,
    ) -> Result<HookReport, HookError> {
        if event.subject() != admitted.observation().subject() {
            return Err(HookError::SubjectMismatch {
                event: event.subject().clone(),
                admitted: admitted.observation().subject().clone(),
            });
        }
        let mut ordered = self.hooks.values().collect::<Vec<_>>();
        ordered.sort_by_key(|hook| (std::cmp::Reverse(hook.priority_value()), hook.id()));
        let event_observation = event_as_observation(event)?;
        let mut intents = Vec::new();
        let mut seen = BTreeSet::new();
        let mut evaluations = Vec::new();
        let mut stopped_by = None;
        let mut next_nonce = nonce_base;

        for hook in ordered {
            let topic_matched = hook.topic() == event.topic();
            let mut failures = Vec::new();
            let conditions_matched = if topic_matched {
                let mut table = DecisionTable::new();
                let mut rule = DecisionRule::new("hook", 0, DecisionEffect::Allow);
                for condition in hook.conditions() {
                    rule = rule.when(condition.clone());
                }
                table.push(rule);
                match table.evaluate(&event_observation) {
                    DecisionOutcome::Selected { trace, .. } => {
                        failures.extend(
                            trace
                                .into_iter()
                                .flat_map(|evaluation| evaluation.failures().to_vec()),
                        );
                        true
                    }
                    DecisionOutcome::NoMatch { trace, .. }
                    | DecisionOutcome::Conflict { trace, .. } => {
                        failures.extend(
                            trace
                                .into_iter()
                                .flat_map(|evaluation| evaluation.failures().to_vec()),
                        );
                        false
                    }
                }
            } else {
                failures.push(format!(
                    "topic `{}` does not match `{}`",
                    event.topic(),
                    hook.topic()
                ));
                false
            };

            let before = intents.len();
            if topic_matched && conditions_matched {
                for template in hook.intents() {
                    let payload = template.payload().render(event)?;
                    let intent = Intent::construct(
                        admitted,
                        template.capability().clone(),
                        template.operation().clone(),
                        template.authority().clone(),
                        next_nonce,
                        payload,
                    );
                    next_nonce = next_nonce.checked_add(1).ok_or(HookError::NonceOverflow)?;
                    if seen.insert(intent.digest()) {
                        intents.push(intent);
                    }
                }
            }
            let emitted = intents.len() - before;
            let mut encoder = CanonicalEncoder::new();
            encoder
                .text("type", "hook-evaluation-v1")
                .field("hook", &hook.digest().0)
                .field("event", &event.digest().0)
                .boolean("topic-matched", topic_matched)
                .boolean("conditions-matched", conditions_matched)
                .u64("failure-count", failures.len() as u64)
                .u64("emitted", emitted as u64);
            for failure in &failures {
                encoder.text("failure", failure);
            }
            evaluations.push(HookEvaluation {
                hook: hook.id().to_owned(),
                topic_matched,
                conditions_matched,
                failures,
                emitted,
                digest: encoder.digest(),
            });
            if topic_matched && conditions_matched && hook.stops_propagation() {
                stopped_by = Some(hook.id().to_owned());
                break;
            }
        }

        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "hook-report-v1")
            .field("event", &event.digest().0)
            .field("admission", &admitted.digest().0)
            .u64("intent-count", intents.len() as u64);
        for intent in &intents {
            encoder.field("intent", &intent.digest().0);
        }
        encoder.u64("evaluation-count", evaluations.len() as u64);
        for evaluation in &evaluations {
            encoder.field("evaluation", &evaluation.digest().0);
        }
        match &stopped_by {
            Some(value) => {
                encoder
                    .boolean("stopped", true)
                    .text("stopped-by", value);
            }
            None => {
                encoder.boolean("stopped", false);
            }
        }
        Ok(HookReport {
            event_digest: event.digest(),
            intents,
            evaluations,
            stopped_by,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "hook-registry-v1")
            .u64("hook-count", self.hooks.len() as u64);
        for hook in self.hooks.values() {
            encoder
                .text("hook", hook.id())
                .field("hook-digest", &hook.digest().0);
        }
        encoder.digest()
    }
}

fn event_as_observation(event: &HookEvent) -> Result<crate::model::Observation, HookError> {
    let mut observation = crate::model::Observation::new(event.subject().clone(), event.sequence());
    for (key, value) in event.facts() {
        observation
            .insert(key.to_owned(), value.clone())
            .expect("hook event fact keys are already admitted strings");
    }
    Ok(observation)
}

#[cfg(test)]
mod tests {
    use super::{
        Hook, HookEvent, HookLint, HookRegistry, IntentTemplate, PayloadTemplate,
    };
    use crate::decision::Condition;
    use crate::model::{
        AdmittedObservation, AuthorityId, CapabilityId, Observation, OperationId, PolicyId,
        SubjectId,
    };

    fn admitted(subject: &str) -> AdmittedObservation {
        AdmittedObservation::new(
            Observation::new(SubjectId::new(subject).unwrap(), 1),
            PolicyId::new("policy").unwrap(),
            1,
        )
    }

    fn template() -> IntentTemplate {
        IntentTemplate::new(
            CapabilityId::new("sync").unwrap(),
            OperationId::new("apply").unwrap(),
            AuthorityId::new("hook-authority").unwrap(),
            PayloadTemplate::EventPayload,
        )
    }

    #[test]
    fn matching_hook_manufactures_intent_without_execution() {
        let mut registry = HookRegistry::new();
        registry
            .insert(
                Hook::new("sync-on-change", "entity.changed")
                    .unwrap()
                    .when(Condition::Equals {
                        key: "ready".to_owned(),
                        value: true.into(),
                    })
                    .emits(template()),
            )
            .unwrap();
        let mut event = HookEvent::new(
            "entity.changed",
            1,
            SubjectId::new("entity-1").unwrap(),
        )
        .unwrap();
        event.insert("ready", true);
        event.set_payload(b"delta".to_vec());
        let report = registry
            .manufacture(&event, &admitted("entity-1"), 100)
            .unwrap();
        assert_eq!(report.intents().len(), 1);
        assert_eq!(report.intents()[0].nonce(), 100);
        assert_eq!(report.intents()[0].payload(), b"delta");
    }

    #[test]
    fn subject_mismatch_is_refused() {
        let registry = HookRegistry::new();
        let event = HookEvent::new(
            "entity.changed",
            1,
            SubjectId::new("entity-1").unwrap(),
        )
        .unwrap();
        assert!(registry
            .manufacture(&event, &admitted("entity-2"), 0)
            .is_err());
    }

    #[test]
    fn exact_intents_are_deduplicated() {
        let mut registry = HookRegistry::new();
        registry
            .insert(
                Hook::new("one", "event")
                    .unwrap()
                    .priority(2)
                    .emits(template()),
            )
            .unwrap();
        registry
            .insert(
                Hook::new("two", "event")
                    .unwrap()
                    .priority(1)
                    .emits(template()),
            )
            .unwrap();
        let event = HookEvent::new("event", 1, SubjectId::new("entity").unwrap()).unwrap();
        let report = registry
            .manufacture(&event, &admitted("entity"), 0)
            .unwrap();
        assert_eq!(report.intents().len(), 2);
        assert_ne!(report.intents()[0].nonce(), report.intents()[1].nonce());
    }

    #[test]
    fn linter_finds_unconditional_shadow() {
        let mut registry = HookRegistry::new();
        registry
            .insert(
                Hook::new("stop", "event")
                    .unwrap()
                    .priority(100)
                    .emits(template())
                    .stop_propagation(true),
            )
            .unwrap();
        registry
            .insert(
                Hook::new("shadowed", "event")
                    .unwrap()
                    .priority(1)
                    .emits(template()),
            )
            .unwrap();
        assert!(registry.lint().iter().any(|finding| matches!(
            finding,
            HookLint::Shadowed { hook, by } if hook == "shadowed" && by == "stop"
        )));
    }
}
