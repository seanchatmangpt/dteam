//! Declarative admission policies with complete violation evidence.

use crate::model::{AdmittedObservation, AuthorityId, FactValue, Observation, PolicyId};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

/// A named admission rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    name: String,
    predicate: Predicate,
}

impl Rule {
    /// Creates a rule with a stable name and predicate.
    #[must_use]
    pub fn new(name: impl Into<String>, predicate: Predicate) -> Self {
        Self {
            name: name.into(),
            predicate,
        }
    }

    /// Returns the rule name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the predicate.
    #[must_use]
    pub const fn predicate(&self) -> &Predicate {
        &self.predicate
    }
}

/// Closed set of deterministic admission predicates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Predicate {
    Present {
        key: String,
    },
    Absent {
        key: String,
    },
    Equals {
        key: String,
        expected: FactValue,
    },
    NotEquals {
        key: String,
        forbidden: FactValue,
    },
    MinI64 {
        key: String,
        minimum: i64,
    },
    MaxI64 {
        key: String,
        maximum: i64,
    },
    MinU64 {
        key: String,
        minimum: u64,
    },
    MaxU64 {
        key: String,
        maximum: u64,
    },
    TextOneOf {
        key: String,
        allowed: BTreeSet<String>,
    },
    TextSetContains {
        key: String,
        member: String,
    },
    Authority {
        authority: AuthorityId,
    },
    SequenceAtLeast {
        minimum: u64,
    },
    SequenceAtMost {
        maximum: u64,
    },
}

/// One falsified rule with machine-readable code and evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    rule: String,
    code: &'static str,
    detail: String,
}

impl Violation {
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl Display for Violation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} [{}]: {}", self.rule, self.code, self.detail)
    }
}

/// Complete admission decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionDecision {
    Admitted(AdmittedObservation),
    Refused(Vec<Violation>),
}

impl AdmissionDecision {
    /// Returns the admitted observation or the complete violations.
    pub fn into_result(self) -> Result<AdmittedObservation, Vec<Violation>> {
        match self {
            Self::Admitted(admitted) => Ok(admitted),
            Self::Refused(violations) => Err(violations),
        }
    }

    /// Returns true when all policy rules admitted the observation.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self, Self::Admitted(_))
    }
}

/// Versioned policy that turns `Observation` into `AdmittedObservation` or refusal evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionPolicy {
    id: PolicyId,
    epoch: u64,
    rules: Vec<Rule>,
}

impl AdmissionPolicy {
    /// Creates an empty policy. Empty policies intentionally admit all observations.
    #[must_use]
    pub fn new(id: PolicyId, epoch: u64) -> Self {
        Self {
            id,
            epoch,
            rules: Vec::new(),
        }
    }

    /// Adds a rule in evaluation order.
    #[must_use]
    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Adds a rule in place.
    pub fn push_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Returns the policy identifier.
    #[must_use]
    pub const fn id(&self) -> &PolicyId {
        &self.id
    }

    /// Returns the policy epoch.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns all policy rules.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Evaluates every rule and returns all violations, never only the first one.
    #[must_use]
    pub fn evaluate(&self, observation: Observation) -> AdmissionDecision {
        let violations: Vec<Violation> = self
            .rules
            .iter()
            .filter_map(|rule| evaluate_rule(rule, &observation))
            .collect();
        if violations.is_empty() {
            AdmissionDecision::Admitted(AdmittedObservation::new(
                observation,
                self.id.clone(),
                self.epoch,
            ))
        } else {
            AdmissionDecision::Refused(violations)
        }
    }
}

fn evaluate_rule(rule: &Rule, observation: &Observation) -> Option<Violation> {
    let violation = |code, detail| {
        Some(Violation {
            rule: rule.name.clone(),
            code,
            detail,
        })
    };

    match &rule.predicate {
        Predicate::Present { key } => {
            if observation.fact(key).is_some() {
                None
            } else {
                violation("MISSING_FACT", format!("required fact `{key}` is absent"))
            }
        }
        Predicate::Absent { key } => {
            if observation.fact(key).is_none() {
                None
            } else {
                violation(
                    "FORBIDDEN_FACT",
                    format!("forbidden fact `{key}` is present"),
                )
            }
        }
        Predicate::Equals { key, expected } => match observation.fact(key) {
            Some(actual) if actual == expected => None,
            Some(actual) => violation(
                "FACT_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected {expected:?}"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::NotEquals { key, forbidden } => match observation.fact(key) {
            Some(actual) if actual == forbidden => violation(
                "FORBIDDEN_VALUE",
                format!("fact `{key}` has forbidden value {forbidden:?}"),
            ),
            _ => None,
        },
        Predicate::MinI64 { key, minimum } => match observation.fact(key) {
            Some(FactValue::I64(actual)) if *actual >= *minimum => None,
            Some(FactValue::I64(actual)) => violation(
                "BELOW_MINIMUM",
                format!("fact `{key}` was {actual}, minimum is {minimum}"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected i64"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::MaxI64 { key, maximum } => match observation.fact(key) {
            Some(FactValue::I64(actual)) if *actual <= *maximum => None,
            Some(FactValue::I64(actual)) => violation(
                "ABOVE_MAXIMUM",
                format!("fact `{key}` was {actual}, maximum is {maximum}"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected i64"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::MinU64 { key, minimum } => match observation.fact(key) {
            Some(FactValue::U64(actual)) if *actual >= *minimum => None,
            Some(FactValue::U64(actual)) => violation(
                "BELOW_MINIMUM",
                format!("fact `{key}` was {actual}, minimum is {minimum}"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected u64"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::MaxU64 { key, maximum } => match observation.fact(key) {
            Some(FactValue::U64(actual)) if *actual <= *maximum => None,
            Some(FactValue::U64(actual)) => violation(
                "ABOVE_MAXIMUM",
                format!("fact `{key}` was {actual}, maximum is {maximum}"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected u64"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::TextOneOf { key, allowed } => match observation.fact(key) {
            Some(FactValue::Text(actual)) if allowed.contains(actual) => None,
            Some(FactValue::Text(actual)) => violation(
                "VALUE_NOT_ALLOWED",
                format!("fact `{key}` value `{actual}` is not in {allowed:?}"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected text"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::TextSetContains { key, member } => match observation.fact(key) {
            Some(FactValue::TextSet(values)) if values.contains(member) => None,
            Some(FactValue::TextSet(_)) => violation(
                "MISSING_MEMBER",
                format!("fact `{key}` does not contain `{member}`"),
            ),
            Some(actual) => violation(
                "TYPE_MISMATCH",
                format!("fact `{key}` was {actual:?}, expected text set"),
            ),
            None => violation("MISSING_FACT", format!("fact `{key}` is absent")),
        },
        Predicate::Authority { authority } => {
            if observation.has_authority(authority) {
                None
            } else {
                violation(
                    "MISSING_AUTHORITY",
                    format!("authority `{authority}` did not attest the observation"),
                )
            }
        }
        Predicate::SequenceAtLeast { minimum } => {
            if observation.sequence() >= *minimum {
                None
            } else {
                violation(
                    "STALE_SEQUENCE",
                    format!(
                        "observation sequence {} is below required {minimum}",
                        observation.sequence()
                    ),
                )
            }
        }
        Predicate::SequenceAtMost { maximum } => {
            if observation.sequence() <= *maximum {
                None
            } else {
                violation(
                    "FUTURE_SEQUENCE",
                    format!(
                        "observation sequence {} exceeds allowed {maximum}",
                        observation.sequence()
                    ),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdmissionDecision, AdmissionPolicy, Predicate, Rule};
    use crate::model::{AuthorityId, Observation, PolicyId, SubjectId};

    #[test]
    fn admission_collects_every_violation() {
        let policy = AdmissionPolicy::new(PolicyId::new("policy").unwrap(), 7)
            .with_rule(Rule::new(
                "needs-health",
                Predicate::Present {
                    key: "health".to_owned(),
                },
            ))
            .with_rule(Rule::new(
                "needs-board",
                Predicate::Authority {
                    authority: AuthorityId::new("board").unwrap(),
                },
            ));
        let observation = Observation::new(SubjectId::new("subject").unwrap(), 1);
        let AdmissionDecision::Refused(violations) = policy.evaluate(observation) else {
            panic!("observation must be refused");
        };
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].code(), "MISSING_FACT");
        assert_eq!(violations[1].code(), "MISSING_AUTHORITY");
    }

    #[test]
    fn admission_binds_policy_epoch() {
        let policy = AdmissionPolicy::new(PolicyId::new("policy").unwrap(), 9);
        let observation = Observation::new(SubjectId::new("subject").unwrap(), 3);
        let admitted = policy.evaluate(observation).into_result().unwrap();
        assert_eq!(admitted.policy_epoch(), 9);
        assert_eq!(admitted.policy().as_str(), "policy");
    }
}
