//! Canonical domain objects for observations, intents, outcomes, and standing.

use crate::hash::{CanonicalEncoder, Digest};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Constructs a validated non-empty identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ModelError::EmptyIdentifier(stringify!($name)));
                }
                if value.bytes().any(|byte| byte.is_ascii_control()) {
                    return Err(ModelError::ControlCharacter(stringify!($name)));
                }
                Ok(Self(value))
            }

            /// Returns the identifier as a string slice.
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

id_type!(SubjectId);
id_type!(CapabilityId);
id_type!(AuthorityId);
id_type!(OperationId);
id_type!(PolicyId);

/// Errors constructing canonical model values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelError {
    EmptyIdentifier(&'static str),
    ControlCharacter(&'static str),
    EmptyFactKey,
}

impl Display for ModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::ControlCharacter(kind) => {
                write!(formatter, "{kind} must not contain control characters")
            }
            Self::EmptyFactKey => formatter.write_str("fact key must not be empty"),
        }
    }
}

impl std::error::Error for ModelError {}

/// A deterministic fact value admitted into an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    Text(String),
    Bytes(Vec<u8>),
    TextSet(BTreeSet<String>),
}

impl FactValue {
    pub(crate) fn encode(&self, encoder: &mut CanonicalEncoder, tag: &str) {
        match self {
            Self::Bool(value) => {
                encoder.text(tag, "bool").boolean("value", *value);
            }
            Self::I64(value) => {
                encoder.text(tag, "i64").i64("value", *value);
            }
            Self::U64(value) => {
                encoder.text(tag, "u64").u64("value", *value);
            }
            Self::Text(value) => {
                encoder.text(tag, "text").text("value", value);
            }
            Self::Bytes(value) => {
                encoder.text(tag, "bytes").field("value", value);
            }
            Self::TextSet(values) => {
                encoder.text(tag, "text-set").u64("len", values.len() as u64);
                for value in values {
                    encoder.text("item", value);
                }
            }
        }
    }

    /// Returns this value as a boolean when types match.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns this value as a signed integer when types match.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        if let Self::I64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns this value as an unsigned integer when types match.
    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        if let Self::U64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns this value as text when types match.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns whether this value contains a member when it is a text set.
    #[must_use]
    pub fn contains_text(&self, member: &str) -> bool {
        matches!(self, Self::TextSet(values) if values.contains(member))
    }
}

impl From<bool> for FactValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for FactValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for FactValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<String> for FactValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for FactValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// An immutable, canonically hashable admitted-observation candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    subject: SubjectId,
    sequence: u64,
    facts: BTreeMap<String, FactValue>,
    authorities: BTreeSet<AuthorityId>,
}

impl Observation {
    /// Starts an observation for a subject at a monotonic sequence.
    #[must_use]
    pub fn new(subject: SubjectId, sequence: u64) -> Self {
        Self {
            subject,
            sequence,
            facts: BTreeMap::new(),
            authorities: BTreeSet::new(),
        }
    }

    /// Adds or replaces a fact.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Result<Option<FactValue>, ModelError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(ModelError::EmptyFactKey);
        }
        Ok(self.facts.insert(key, value.into()))
    }

    /// Adds an authority attestation.
    pub fn attest(&mut self, authority: AuthorityId) -> bool {
        self.authorities.insert(authority)
    }

    /// Returns the observation subject.
    #[must_use]
    pub const fn subject(&self) -> &SubjectId {
        &self.subject
    }

    /// Returns the monotonic source sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns a fact by key.
    #[must_use]
    pub fn fact(&self, key: &str) -> Option<&FactValue> {
        self.facts.get(key)
    }

    /// Returns all facts in canonical key order.
    pub fn facts(&self) -> impl ExactSizeIterator<Item = (&str, &FactValue)> {
        self.facts.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Returns whether an authority attested this observation.
    #[must_use]
    pub fn has_authority(&self, authority: &AuthorityId) -> bool {
        self.authorities.contains(authority)
    }

    /// Computes the canonical observation identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "observation-v1")
            .text("subject", self.subject.as_str())
            .u64("sequence", self.sequence)
            .u64("fact-count", self.facts.len() as u64);
        for (key, value) in &self.facts {
            encoder.text("fact-key", key);
            value.encode(&mut encoder, "fact-type");
        }
        encoder.u64("authority-count", self.authorities.len() as u64);
        for authority in &self.authorities {
            encoder.text("authority", authority.as_str());
        }
        encoder.digest()
    }
}

/// An admitted observation bound to the exact policy that admitted it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedObservation {
    observation: Observation,
    policy: PolicyId,
    policy_epoch: u64,
    admission_digest: Digest,
}

impl AdmittedObservation {
    pub(crate) fn new(observation: Observation, policy: PolicyId, policy_epoch: u64) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "admission-v1")
            .field("observation", &observation.digest().0)
            .text("policy", policy.as_str())
            .u64("policy-epoch", policy_epoch);
        Self {
            observation,
            policy,
            policy_epoch,
            admission_digest: encoder.digest(),
        }
    }

    /// Returns the admitted observation.
    #[must_use]
    pub const fn observation(&self) -> &Observation {
        &self.observation
    }

    /// Returns the admitting policy identifier.
    #[must_use]
    pub const fn policy(&self) -> &PolicyId {
        &self.policy
    }

    /// Returns the admitting policy epoch.
    #[must_use]
    pub const fn policy_epoch(&self) -> u64 {
        self.policy_epoch
    }

    /// Returns the admission identity.
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.admission_digest
    }
}

/// A constructed request that has no ambient execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    capability: CapabilityId,
    operation: OperationId,
    subject: SubjectId,
    authority: AuthorityId,
    admission_digest: Digest,
    nonce: u64,
    payload: Vec<u8>,
    digest: Digest,
}

impl Intent {
    /// Constructs a deterministic intent from admitted state.
    #[must_use]
    pub fn construct(
        admitted: &AdmittedObservation,
        capability: CapabilityId,
        operation: OperationId,
        authority: AuthorityId,
        nonce: u64,
        payload: Vec<u8>,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "intent-v1")
            .text("capability", capability.as_str())
            .text("operation", operation.as_str())
            .text("subject", admitted.observation().subject().as_str())
            .text("authority", authority.as_str())
            .field("admission", &admitted.digest().0)
            .u64("nonce", nonce)
            .field("payload", &payload);
        let digest = encoder.digest();
        Self {
            capability,
            operation,
            subject: admitted.observation().subject().clone(),
            authority,
            admission_digest: admitted.digest(),
            nonce,
            payload,
            digest,
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
    pub const fn nonce(&self) -> u64 {
        self.nonce
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Normalized result of an external actuation attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Applied { code: u16, output: Vec<u8> },
    Refused { code: String, detail: String },
    Failed { code: String, detail: String },
}

impl Outcome {
    /// Computes the canonical outcome identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder.text("type", "outcome-v1");
        match self {
            Self::Applied { code, output } => {
                encoder
                    .text("kind", "applied")
                    .u64("code", u64::from(*code))
                    .field("output", output);
            }
            Self::Refused { code, detail } => {
                encoder
                    .text("kind", "refused")
                    .text("code", code)
                    .text("detail", detail);
            }
            Self::Failed { code, detail } => {
                encoder
                    .text("kind", "failed")
                    .text("code", code)
                    .text("detail", detail);
            }
        }
        encoder.digest()
    }

    /// Returns true only for a completed applied outcome.
    #[must_use]
    pub const fn is_applied(&self) -> bool {
        matches!(self, Self::Applied { .. })
    }
}

/// Scoped standing derived from a verified receipt chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Standing {
    Unknown,
    PartialAlive,
    Alive,
    Blocked,
    BuildBroken,
    Unsupported,
    Refused,
}
