//! Explainable deterministic decision tables with linting and conflict detection.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{FactValue, Observation, OperationId};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// One side-effect-free predicate over an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Condition {
    Present { key: String },
    Absent { key: String },
    Equals { key: String, value: FactValue },
    NotEquals { key: String, value: FactValue },
    I64Range {
        key: String,
        minimum: Option<i64>,
        maximum: Option<i64>,
    },
    U64Range {
        key: String,
        minimum: Option<u64>,
        maximum: Option<u64>,
    },
    TextPrefix { key: String, prefix: String },
    TextContains { key: String, fragment: String },
    TextSetContains { key: String, member: String },
}

impl Condition {
    fn evaluate(&self, observation: &Observation) -> Result<(), String> {
        match self {
            Self::Present { key } => observation
                .fact(key)
                .map(|_| ())
                .ok_or_else(|| format!("`{key}` is absent")),
            Self::Absent { key } => {
                if observation.fact(key).is_none() {
                    Ok(())
                } else {
                    Err(format!("`{key}` is present"))
                }
            }
            Self::Equals { key, value } => match observation.fact(key) {
                Some(actual) if actual == value => Ok(()),
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected {value:?}")),
                None => Err(format!("`{key}` is absent")),
            },
            Self::NotEquals { key, value } => match observation.fact(key) {
                Some(actual) if actual == value => {
                    Err(format!("`{key}` has forbidden value {value:?}"))
                }
                _ => Ok(()),
            },
            Self::I64Range {
                key,
                minimum,
                maximum,
            } => match observation.fact(key) {
                Some(FactValue::I64(actual)) => {
                    if minimum.is_some_and(|value| actual < &value) {
                        return Err(format!("`{key}` was {actual}, below {minimum:?}"));
                    }
                    if maximum.is_some_and(|value| actual > &value) {
                        return Err(format!("`{key}` was {actual}, above {maximum:?}"));
                    }
                    Ok(())
                }
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected i64")),
                None => Err(format!("`{key}` is absent")),
            },
            Self::U64Range {
                key,
                minimum,
                maximum,
            } => match observation.fact(key) {
                Some(FactValue::U64(actual)) => {
                    if minimum.is_some_and(|value| actual < &value) {
                        return Err(format!("`{key}` was {actual}, below {minimum:?}"));
                    }
                    if maximum.is_some_and(|value| actual > &value) {
                        return Err(format!("`{key}` was {actual}, above {maximum:?}"));
                    }
                    Ok(())
                }
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected u64")),
                None => Err(format!("`{key}` is absent")),
            },
            Self::TextPrefix { key, prefix } => match observation.fact(key) {
                Some(FactValue::Text(actual)) if actual.starts_with(prefix) => Ok(()),
                Some(FactValue::Text(actual)) => {
                    Err(format!("`{key}` value `{actual}` does not start with `{prefix}`"))
                }
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected text")),
                None => Err(format!("`{key}` is absent")),
            },
            Self::TextContains { key, fragment } => match observation.fact(key) {
                Some(FactValue::Text(actual)) if actual.contains(fragment) => Ok(()),
                Some(FactValue::Text(actual)) => Err(format!(
                    "`{key}` value `{actual}` does not contain `{fragment}`"
                )),
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected text")),
                None => Err(format!("`{key}` is absent")),
            },
            Self::TextSetContains { key, member } => match observation.fact(key) {
                Some(FactValue::TextSet(actual)) if actual.contains(member) => Ok(()),
                Some(FactValue::TextSet(_)) => {
                    Err(format!("`{key}` does not contain `{member}`"))
                }
                Some(actual) => Err(format!("`{key}` was {actual:?}, expected text set")),
                None => Err(format!("`{key}` is absent")),
            },
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Present { key } => {
                encoder.text("condition", "present").text("key", key);
            }
            Self::Absent { key } => {
                encoder.text("condition", "absent").text("key", key);
            }
            Self::Equals { key, value } => {
                encoder.text("condition", "equals").text("key", key);
                value.encode(encoder, "value-type");
            }
            Self::NotEquals { key, value } => {
                encoder
                    .text("condition", "not-equals")
                    .text("key", key);
                value.encode(encoder, "value-type");
            }
            Self::I64Range {
                key,
                minimum,
                maximum,
            } => {
                encoder.text("condition", "i64-range").text("key", key);
                if let Some(value) = minimum {
                    encoder.i64("minimum", *value);
                }
                if let Some(value) = maximum {
                    encoder.i64("maximum", *value);
                }
            }
            Self::U64Range {
                key,
                minimum,
                maximum,
            } => {
                encoder.text("condition", "u64-range").text("key", key);
                if let Some(value) = minimum {
                    encoder.u64("minimum", *value);
                }
                if let Some(value) = maximum {
                    encoder.u64("maximum", *value);
                }
            }
            Self::TextPrefix { key, prefix } => {
                encoder
                    .text("condition", "text-prefix")
                    .text("key", key)
                    .text("prefix", prefix);
            }
            Self::TextContains { key, fragment } => {
                encoder
                    .text("condition", "text-contains")
                    .text("key", key)
                    .text("fragment", fragment);
            }
            Self::TextSetContains { key, member } => {
                encoder
                    .text("condition", "text-set-contains")
                    .text("key", key)
                    .text("member", member);
            }
        }
    }
}

/// Pure decision effect. `Emit` constructs an operation request but cannot execute it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionEffect {
    Allow,
    Deny { code: String },
    Escalate { queue: String },
    Emit { operation: OperationId, payload: Vec<u8> },
}

impl DecisionEffect {
    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Allow => {
                encoder.text("effect", "allow");
            }
            Self::Deny { code } => {
                encoder.text("effect", "deny").text("code", code);
            }
            Self::Escalate { queue } => {
                encoder
                    .text("effect", "escalate")
                    .text("queue", queue);
            }
            Self::Emit { operation, payload } => {
                encoder
                    .text("effect", "emit")
                    .text("operation", operation.as_str())
                    .field("payload", payload);
            }
        }
    }
}

/// One deterministic decision rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRule {
    id: String,
    priority: i32,
    conditions: Vec<Condition>,
    effect: DecisionEffect,
    terminal: bool,
}

impl DecisionRule {
    #[must_use]
    pub fn new(id: impl Into<String>, priority: i32, effect: DecisionEffect) -> Self {
        Self {
            id: id.into(),
            priority,
            conditions: Vec::new(),
            effect,
            terminal: true,
        }
    }

    #[must_use]
    pub fn when(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    #[must_use]
    pub const fn terminal(mut self, terminal: bool) -> Self {
        self.terminal = terminal;
        self
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }

    #[must_use]
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    #[must_use]
    pub const fn effect(&self) -> &DecisionEffect {
        &self.effect
    }

    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.terminal
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "decision-rule-v1")
            .text("id", &self.id)
            .i64("priority", i64::from(self.priority))
            .boolean("terminal", self.terminal)
            .u64("condition-count", self.conditions.len() as u64);
        for condition in &self.conditions {
            condition.encode(&mut encoder);
        }
        self.effect.encode(&mut encoder);
        encoder.digest()
    }
}

/// Evaluation evidence for one rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleEvaluation {
    rule: String,
    matched: bool,
    failures: Vec<String>,
    digest: Digest,
}

impl RuleEvaluation {
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    #[must_use]
    pub const fn matched(&self) -> bool {
        self.matched
    }

    #[must_use]
    pub fn failures(&self) -> &[String] {
        &self.failures
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Complete decision result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionOutcome {
    Selected {
        rule: String,
        effect: DecisionEffect,
        trace: Vec<RuleEvaluation>,
        digest: Digest,
    },
    NoMatch {
        trace: Vec<RuleEvaluation>,
        digest: Digest,
    },
    Conflict {
        priority: i32,
        rules: Vec<String>,
        effects: Vec<DecisionEffect>,
        trace: Vec<RuleEvaluation>,
        digest: Digest,
    },
}

impl DecisionOutcome {
    #[must_use]
    pub fn trace(&self) -> &[RuleEvaluation] {
        match self {
            Self::Selected { trace, .. }
            | Self::NoMatch { trace, .. }
            | Self::Conflict { trace, .. } => trace,
        }
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        match self {
            Self::Selected { digest, .. }
            | Self::NoMatch { digest, .. }
            | Self::Conflict { digest, .. } => *digest,
        }
    }
}

/// Static table defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionLint {
    EmptyRuleId { index: usize },
    DuplicateRuleId { id: String },
    ShadowedRule { rule: String, by: String },
    ConflictingIdenticalRules { left: String, right: String },
}

impl Display for DecisionLint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRuleId { index } => write!(formatter, "rule {index} has an empty id"),
            Self::DuplicateRuleId { id } => write!(formatter, "duplicate rule id `{id}`"),
            Self::ShadowedRule { rule, by } => {
                write!(formatter, "rule `{rule}` is shadowed by terminal `{by}`")
            }
            Self::ConflictingIdenticalRules { left, right } => write!(
                formatter,
                "rules `{left}` and `{right}` have identical conditions and conflicting effects"
            ),
        }
    }
}

/// Deterministically sorted and explainable decision table.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DecisionTable {
    rules: Vec<DecisionRule>,
}

impl DecisionTable {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, rule: DecisionRule) {
        self.rules.push(rule);
        self.rules
            .sort_by_key(|entry| (Reverse(entry.priority()), entry.id.clone()));
    }

    #[must_use]
    pub fn rules(&self) -> &[DecisionRule] {
        &self.rules
    }

    /// Finds duplicate identities, static conflicts, and unconditional shadowing.
    #[must_use]
    pub fn lint(&self) -> Vec<DecisionLint> {
        let mut findings = Vec::new();
        let mut identities = BTreeSet::new();
        for (index, rule) in self.rules.iter().enumerate() {
            if rule.id.trim().is_empty() {
                findings.push(DecisionLint::EmptyRuleId { index });
            } else if !identities.insert(rule.id.clone()) {
                findings.push(DecisionLint::DuplicateRuleId {
                    id: rule.id.clone(),
                });
            }
        }

        for (index, left) in self.rules.iter().enumerate() {
            for right in self.rules.iter().skip(index + 1) {
                if left.conditions == right.conditions && left.effect != right.effect {
                    findings.push(DecisionLint::ConflictingIdenticalRules {
                        left: left.id.clone(),
                        right: right.id.clone(),
                    });
                }
                if left.terminal
                    && left.conditions.is_empty()
                    && left.priority >= right.priority
                {
                    findings.push(DecisionLint::ShadowedRule {
                        rule: right.id.clone(),
                        by: left.id.clone(),
                    });
                }
            }
        }
        findings
    }

    /// Evaluates every rule for evidence, then selects or reports a top-priority conflict.
    #[must_use]
    pub fn evaluate(&self, observation: &Observation) -> DecisionOutcome {
        let mut trace = Vec::with_capacity(self.rules.len());
        let mut matches = Vec::new();
        for rule in &self.rules {
            let failures = rule
                .conditions
                .iter()
                .filter_map(|condition| condition.evaluate(observation).err())
                .collect::<Vec<_>>();
            let matched = failures.is_empty();
            let mut encoder = CanonicalEncoder::new();
            encoder
                .text("type", "rule-evaluation-v1")
                .field("rule", &rule.digest().0)
                .field("observation", &observation.digest().0)
                .boolean("matched", matched)
                .u64("failure-count", failures.len() as u64);
            for failure in &failures {
                encoder.text("failure", failure);
            }
            trace.push(RuleEvaluation {
                rule: rule.id.clone(),
                matched,
                failures,
                digest: encoder.digest(),
            });
            if matched {
                matches.push(rule);
            }
        }

        if matches.is_empty() {
            let digest = outcome_digest("no-match", &trace, &[]);
            return DecisionOutcome::NoMatch { trace, digest };
        }

        let top_priority = matches[0].priority;
        let top = matches
            .into_iter()
            .take_while(|rule| rule.priority == top_priority)
            .collect::<Vec<_>>();
        let distinct_effects = top
            .iter()
            .map(|rule| &rule.effect)
            .collect::<BTreeSet<_>>();
        if distinct_effects.len() > 1 {
            let rules = top.iter().map(|rule| rule.id.clone()).collect::<Vec<_>>();
            let effects = top.iter().map(|rule| rule.effect.clone()).collect::<Vec<_>>();
            let digest = outcome_digest("conflict", &trace, &top);
            return DecisionOutcome::Conflict {
                priority: top_priority,
                rules,
                effects,
                trace,
                digest,
            };
        }

        let selected = top[0];
        let digest = outcome_digest("selected", &trace, &[selected]);
        DecisionOutcome::Selected {
            rule: selected.id.clone(),
            effect: selected.effect.clone(),
            trace,
            digest,
        }
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "decision-table-v1")
            .u64("rule-count", self.rules.len() as u64);
        for rule in &self.rules {
            encoder
                .text("rule", rule.id())
                .field("rule-digest", &rule.digest().0);
        }
        encoder.digest()
    }
}

fn outcome_digest(kind: &str, trace: &[RuleEvaluation], rules: &[&DecisionRule]) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "decision-outcome-v1")
        .text("kind", kind)
        .u64("trace-count", trace.len() as u64);
    for evaluation in trace {
        encoder.field("evaluation", &evaluation.digest().0);
    }
    encoder.u64("selected-count", rules.len() as u64);
    for rule in rules {
        encoder.field("selected", &rule.digest().0);
    }
    encoder.digest()
}

impl Ord for DecisionEffect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        effect_sort_key(self).cmp(&effect_sort_key(other))
    }
}

impl PartialOrd for DecisionEffect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn effect_sort_key(effect: &DecisionEffect) -> (u8, String, Vec<u8>) {
    match effect {
        DecisionEffect::Allow => (0, String::new(), Vec::new()),
        DecisionEffect::Deny { code } => (1, code.clone(), Vec::new()),
        DecisionEffect::Escalate { queue } => (2, queue.clone(), Vec::new()),
        DecisionEffect::Emit { operation, payload } => {
            (3, operation.as_str().to_owned(), payload.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Condition, DecisionEffect, DecisionLint, DecisionOutcome, DecisionRule, DecisionTable,
    };
    use crate::model::{Observation, SubjectId};

    fn observation(risk: u64) -> Observation {
        let mut observation = Observation::new(SubjectId::new("case").unwrap(), 1);
        observation.insert("risk", risk).unwrap();
        observation.insert("region", "us-west").unwrap();
        observation
    }

    #[test]
    fn highest_priority_match_is_selected_with_full_trace() {
        let mut table = DecisionTable::new();
        table.push(
            DecisionRule::new(
                "deny-high-risk",
                100,
                DecisionEffect::Deny {
                    code: "HIGH_RISK".to_owned(),
                },
            )
            .when(Condition::U64Range {
                key: "risk".to_owned(),
                minimum: Some(8),
                maximum: None,
            }),
        );
        table.push(DecisionRule::new("allow-default", 0, DecisionEffect::Allow));
        let DecisionOutcome::Selected { rule, trace, .. } = table.evaluate(&observation(9)) else {
            panic!("expected selection");
        };
        assert_eq!(rule, "deny-high-risk");
        assert_eq!(trace.len(), 2);
        assert!(trace[0].matched());
    }

    #[test]
    fn equal_priority_distinct_effects_are_conflict() {
        let mut table = DecisionTable::new();
        table.push(DecisionRule::new("allow", 10, DecisionEffect::Allow));
        table.push(DecisionRule::new(
            "deny",
            10,
            DecisionEffect::Deny {
                code: "NO".to_owned(),
            },
        ));
        assert!(matches!(
            table.evaluate(&observation(1)),
            DecisionOutcome::Conflict { .. }
        ));
    }

    #[test]
    fn linter_finds_unconditional_shadow() {
        let mut table = DecisionTable::new();
        table.push(DecisionRule::new("catch-all", 100, DecisionEffect::Allow));
        table.push(
            DecisionRule::new(
                "specific",
                1,
                DecisionEffect::Deny {
                    code: "DENY".to_owned(),
                },
            )
            .when(Condition::Present {
                key: "risk".to_owned(),
            }),
        );
        assert!(table.lint().iter().any(|finding| matches!(
            finding,
            DecisionLint::ShadowedRule { rule, by }
                if rule == "specific" && by == "catch-all"
        )));
    }
}
