//! Guarded deterministic state machines with structural analysis and transition receipts.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{FactValue, Observation};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Display, Formatter};

macro_rules! state_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a non-empty stable identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, MachineError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(MachineError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            /// Returns the identifier text.
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

state_id!(StateId);
state_id!(EventKind);
state_id!(TransitionId);

/// Side-effect-free guard evaluated against an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Guard {
    Present { key: String },
    Absent { key: String },
    Equals { key: String, value: FactValue },
    NotEquals { key: String, value: FactValue },
    Bool { key: String, expected: bool },
    I64AtLeast { key: String, minimum: i64 },
    I64AtMost { key: String, maximum: i64 },
    U64AtLeast { key: String, minimum: u64 },
    U64AtMost { key: String, maximum: u64 },
    TextOneOf { key: String, allowed: BTreeSet<String> },
    TextSetContains { key: String, member: String },
}

impl Guard {
    fn evaluate(&self, observation: &Observation) -> Result<(), GuardFailure> {
        let failure = |code, detail| Err(GuardFailure { code, detail });
        match self {
            Self::Present { key } => observation
                .fact(key)
                .map(|_| ())
                .ok_or_else(|| GuardFailure {
                    code: "MISSING_FACT",
                    detail: format!("fact `{key}` is absent"),
                }),
            Self::Absent { key } => {
                if observation.fact(key).is_none() {
                    Ok(())
                } else {
                    failure("FORBIDDEN_FACT", format!("fact `{key}` is present"))
                }
            }
            Self::Equals { key, value } => match observation.fact(key) {
                Some(actual) if actual == value => Ok(()),
                Some(actual) => failure(
                    "FACT_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected {value:?}"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::NotEquals { key, value } => match observation.fact(key) {
                Some(actual) if actual == value => failure(
                    "FORBIDDEN_VALUE",
                    format!("fact `{key}` has forbidden value {value:?}"),
                ),
                _ => Ok(()),
            },
            Self::Bool { key, expected } => match observation.fact(key) {
                Some(FactValue::Bool(actual)) if actual == expected => Ok(()),
                Some(FactValue::Bool(actual)) => failure(
                    "BOOLEAN_MISMATCH",
                    format!("fact `{key}` was {actual}, expected {expected}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected bool"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::I64AtLeast { key, minimum } => match observation.fact(key) {
                Some(FactValue::I64(actual)) if actual >= minimum => Ok(()),
                Some(FactValue::I64(actual)) => failure(
                    "BELOW_MINIMUM",
                    format!("fact `{key}` was {actual}, minimum is {minimum}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected i64"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::I64AtMost { key, maximum } => match observation.fact(key) {
                Some(FactValue::I64(actual)) if actual <= maximum => Ok(()),
                Some(FactValue::I64(actual)) => failure(
                    "ABOVE_MAXIMUM",
                    format!("fact `{key}` was {actual}, maximum is {maximum}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected i64"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::U64AtLeast { key, minimum } => match observation.fact(key) {
                Some(FactValue::U64(actual)) if actual >= minimum => Ok(()),
                Some(FactValue::U64(actual)) => failure(
                    "BELOW_MINIMUM",
                    format!("fact `{key}` was {actual}, minimum is {minimum}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected u64"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::U64AtMost { key, maximum } => match observation.fact(key) {
                Some(FactValue::U64(actual)) if actual <= maximum => Ok(()),
                Some(FactValue::U64(actual)) => failure(
                    "ABOVE_MAXIMUM",
                    format!("fact `{key}` was {actual}, maximum is {maximum}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected u64"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::TextOneOf { key, allowed } => match observation.fact(key) {
                Some(FactValue::Text(actual)) if allowed.contains(actual) => Ok(()),
                Some(FactValue::Text(actual)) => failure(
                    "VALUE_NOT_ALLOWED",
                    format!("fact `{key}` value `{actual}` is not in {allowed:?}"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected text"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
            Self::TextSetContains { key, member } => match observation.fact(key) {
                Some(FactValue::TextSet(values)) if values.contains(member) => Ok(()),
                Some(FactValue::TextSet(_)) => failure(
                    "MISSING_MEMBER",
                    format!("fact `{key}` does not contain `{member}`"),
                ),
                Some(actual) => failure(
                    "TYPE_MISMATCH",
                    format!("fact `{key}` was {actual:?}, expected text set"),
                ),
                None => failure("MISSING_FACT", format!("fact `{key}` is absent")),
            },
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Present { key } => {
                encoder.text("guard", "present").text("key", key);
            }
            Self::Absent { key } => {
                encoder.text("guard", "absent").text("key", key);
            }
            Self::Equals { key, value } => {
                encoder.text("guard", "equals").text("key", key);
                value.encode(encoder, "value-type");
            }
            Self::NotEquals { key, value } => {
                encoder.text("guard", "not-equals").text("key", key);
                value.encode(encoder, "value-type");
            }
            Self::Bool { key, expected } => {
                encoder
                    .text("guard", "bool")
                    .text("key", key)
                    .boolean("expected", *expected);
            }
            Self::I64AtLeast { key, minimum } => {
                encoder
                    .text("guard", "i64-at-least")
                    .text("key", key)
                    .i64("minimum", *minimum);
            }
            Self::I64AtMost { key, maximum } => {
                encoder
                    .text("guard", "i64-at-most")
                    .text("key", key)
                    .i64("maximum", *maximum);
            }
            Self::U64AtLeast { key, minimum } => {
                encoder
                    .text("guard", "u64-at-least")
                    .text("key", key)
                    .u64("minimum", *minimum);
            }
            Self::U64AtMost { key, maximum } => {
                encoder
                    .text("guard", "u64-at-most")
                    .text("key", key)
                    .u64("maximum", *maximum);
            }
            Self::TextOneOf { key, allowed } => {
                encoder
                    .text("guard", "text-one-of")
                    .text("key", key)
                    .u64("allowed-count", allowed.len() as u64);
                for value in allowed {
                    encoder.text("allowed", value);
                }
            }
            Self::TextSetContains { key, member } => {
                encoder
                    .text("guard", "text-set-contains")
                    .text("key", key)
                    .text("member", member);
            }
        }
    }
}

/// One guard failure in a candidate transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardFailure {
    code: &'static str,
    detail: String,
}

impl GuardFailure {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Directed guarded transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition {
    id: TransitionId,
    from: StateId,
    event: EventKind,
    to: StateId,
    priority: i32,
    guards: Vec<Guard>,
}

impl Transition {
    #[must_use]
    pub fn new(
        id: TransitionId,
        from: StateId,
        event: EventKind,
        to: StateId,
    ) -> Self {
        Self {
            id,
            from,
            event,
            to,
            priority: 0,
            guards: Vec::new(),
        }
    }

    #[must_use]
    pub const fn priority(mut self, value: i32) -> Self {
        self.priority = value;
        self
    }

    #[must_use]
    pub fn guarded_by(mut self, guard: Guard) -> Self {
        self.guards.push(guard);
        self
    }

    #[must_use]
    pub const fn id(&self) -> &TransitionId {
        &self.id
    }

    #[must_use]
    pub const fn from(&self) -> &StateId {
        &self.from
    }

    #[must_use]
    pub const fn event(&self) -> &EventKind {
        &self.event
    }

    #[must_use]
    pub const fn to(&self) -> &StateId {
        &self.to
    }

    #[must_use]
    pub const fn priority_value(&self) -> i32 {
        self.priority
    }

    #[must_use]
    pub fn guards(&self) -> &[Guard] {
        &self.guards
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "state-transition-v1")
            .text("id", self.id.as_str())
            .text("from", self.from.as_str())
            .text("event", self.event.as_str())
            .text("to", self.to.as_str())
            .i64("priority", i64::from(self.priority))
            .u64("guard-count", self.guards.len() as u64);
        for guard in &self.guards {
            guard.encode(&mut encoder);
        }
        encoder.digest()
    }
}

/// Static machine construction or validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineError {
    EmptyIdentifier(&'static str),
    DuplicateState(StateId),
    DuplicateTransition(TransitionId),
    UnknownState(StateId),
    InitialStateMissing(StateId),
    RevisionConflict { expected: u64, actual: u64 },
}

impl Display for MachineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::DuplicateState(state) => write!(formatter, "duplicate state `{state}`"),
            Self::DuplicateTransition(transition) => {
                write!(formatter, "duplicate transition `{transition}`")
            }
            Self::UnknownState(state) => write!(formatter, "unknown state `{state}`"),
            Self::InitialStateMissing(state) => {
                write!(formatter, "initial state `{state}` is not declared")
            }
            Self::RevisionConflict { expected, actual } => {
                write!(formatter, "revision {actual}, expected {expected}")
            }
        }
    }
}

impl std::error::Error for MachineError {}

/// Structural machine defect or noteworthy topology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineFinding {
    UnreachableState(StateId),
    DeadEndState(StateId),
    MissingTerminalPath(StateId),
    StructurallyAmbiguous {
        state: StateId,
        event: EventKind,
        priority: i32,
        transitions: Vec<TransitionId>,
    },
}

/// Complete structural analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineAnalysis {
    reachable: BTreeSet<StateId>,
    findings: Vec<MachineFinding>,
    digest: Digest,
}

impl MachineAnalysis {
    #[must_use]
    pub fn reachable(&self) -> &BTreeSet<StateId> {
        &self.reachable
    }

    #[must_use]
    pub fn findings(&self) -> &[MachineFinding] {
        &self.findings
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Candidate evaluation for dispatch evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionEvaluation {
    transition: TransitionId,
    matched: bool,
    failures: Vec<GuardFailure>,
    digest: Digest,
}

impl TransitionEvaluation {
    #[must_use]
    pub const fn transition(&self) -> &TransitionId {
        &self.transition
    }

    #[must_use]
    pub const fn matched(&self) -> bool {
        self.matched
    }

    #[must_use]
    pub fn failures(&self) -> &[GuardFailure] {
        &self.failures
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Pure dispatch result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchDecision {
    Selected {
        transition: Transition,
        evaluations: Vec<TransitionEvaluation>,
        digest: Digest,
    },
    NoTransition {
        state: StateId,
        event: EventKind,
        evaluations: Vec<TransitionEvaluation>,
        digest: Digest,
    },
    Ambiguous {
        state: StateId,
        event: EventKind,
        transitions: Vec<TransitionId>,
        evaluations: Vec<TransitionEvaluation>,
        digest: Digest,
    },
}

impl DispatchDecision {
    #[must_use]
    pub fn evaluations(&self) -> &[TransitionEvaluation] {
        match self {
            Self::Selected { evaluations, .. }
            | Self::NoTransition { evaluations, .. }
            | Self::Ambiguous { evaluations, .. } => evaluations,
        }
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        match self {
            Self::Selected { digest, .. }
            | Self::NoTransition { digest, .. }
            | Self::Ambiguous { digest, .. } => *digest,
        }
    }
}

/// Canonical finite-state machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachine {
    initial: StateId,
    states: BTreeSet<StateId>,
    terminal: BTreeSet<StateId>,
    transitions: BTreeMap<TransitionId, Transition>,
}

impl StateMachine {
    #[must_use]
    pub fn new(initial: StateId) -> Self {
        Self {
            initial,
            states: BTreeSet::new(),
            terminal: BTreeSet::new(),
            transitions: BTreeMap::new(),
        }
    }

    pub fn add_state(&mut self, state: StateId, terminal: bool) -> Result<(), MachineError> {
        if !self.states.insert(state.clone()) {
            return Err(MachineError::DuplicateState(state));
        }
        if terminal {
            self.terminal.insert(state);
        }
        Ok(())
    }

    pub fn add_transition(&mut self, transition: Transition) -> Result<(), MachineError> {
        if self.transitions.contains_key(transition.id()) {
            return Err(MachineError::DuplicateTransition(transition.id().clone()));
        }
        if !self.states.contains(transition.from()) {
            return Err(MachineError::UnknownState(transition.from().clone()));
        }
        if !self.states.contains(transition.to()) {
            return Err(MachineError::UnknownState(transition.to().clone()));
        }
        self.transitions.insert(transition.id().clone(), transition);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), MachineError> {
        if !self.states.contains(&self.initial) {
            return Err(MachineError::InitialStateMissing(self.initial.clone()));
        }
        Ok(())
    }

    #[must_use]
    pub const fn initial(&self) -> &StateId {
        &self.initial
    }

    #[must_use]
    pub fn states(&self) -> &BTreeSet<StateId> {
        &self.states
    }

    #[must_use]
    pub fn terminal_states(&self) -> &BTreeSet<StateId> {
        &self.terminal
    }

    pub fn transitions(&self) -> impl ExactSizeIterator<Item = &Transition> {
        self.transitions.values()
    }

    #[must_use]
    pub fn is_terminal(&self, state: &StateId) -> bool {
        self.terminal.contains(state)
    }

    /// Evaluates all candidates for `(state, event)` and detects top-priority ambiguity.
    #[must_use]
    pub fn dispatch(
        &self,
        state: &StateId,
        event: &EventKind,
        observation: &Observation,
    ) -> DispatchDecision {
        let mut candidates = self
            .transitions
            .values()
            .filter(|transition| transition.from() == state && transition.event() == event)
            .collect::<Vec<_>>();
        candidates.sort_by_key(|transition| {
            (Reverse(transition.priority_value()), transition.id().clone())
        });

        let mut evaluations = Vec::with_capacity(candidates.len());
        let mut matched = Vec::new();
        for transition in candidates {
            let failures = transition
                .guards()
                .iter()
                .filter_map(|guard| guard.evaluate(observation).err())
                .collect::<Vec<_>>();
            let is_match = failures.is_empty();
            let mut encoder = CanonicalEncoder::new();
            encoder
                .text("type", "transition-evaluation-v1")
                .field("transition", &transition.digest().0)
                .field("observation", &observation.digest().0)
                .boolean("matched", is_match)
                .u64("failure-count", failures.len() as u64);
            for failure in &failures {
                encoder
                    .text("code", failure.code())
                    .text("detail", failure.detail());
            }
            evaluations.push(TransitionEvaluation {
                transition: transition.id().clone(),
                matched: is_match,
                failures,
                digest: encoder.digest(),
            });
            if is_match {
                matched.push(transition);
            }
        }

        if matched.is_empty() {
            let digest = dispatch_digest("no-transition", state, event, &evaluations, &[]);
            return DispatchDecision::NoTransition {
                state: state.clone(),
                event: event.clone(),
                evaluations,
                digest,
            };
        }

        let priority = matched[0].priority_value();
        let top = matched
            .into_iter()
            .take_while(|transition| transition.priority_value() == priority)
            .collect::<Vec<_>>();
        if top.len() > 1 {
            let transitions = top
                .iter()
                .map(|transition| transition.id().clone())
                .collect::<Vec<_>>();
            let digest = dispatch_digest("ambiguous", state, event, &evaluations, &top);
            return DispatchDecision::Ambiguous {
                state: state.clone(),
                event: event.clone(),
                transitions,
                evaluations,
                digest,
            };
        }

        let selected = top[0].clone();
        let digest = dispatch_digest("selected", state, event, &evaluations, &[top[0]]);
        DispatchDecision::Selected {
            transition: selected,
            evaluations,
            digest,
        }
    }

    /// Returns every structurally reachable state from the initial state.
    #[must_use]
    pub fn reachable_states(&self) -> BTreeSet<StateId> {
        let mut reachable = BTreeSet::new();
        let mut queue = VecDeque::from([self.initial.clone()]);
        while let Some(state) = queue.pop_front() {
            if !reachable.insert(state.clone()) {
                continue;
            }
            for transition in self
                .transitions
                .values()
                .filter(|transition| transition.from() == &state)
            {
                queue.push_back(transition.to().clone());
            }
        }
        reachable
    }

    /// Returns a shortest structural state path, including both endpoints.
    #[must_use]
    pub fn shortest_path(&self, from: &StateId, to: &StateId) -> Option<Vec<StateId>> {
        if !self.states.contains(from) || !self.states.contains(to) {
            return None;
        }
        let mut queue = VecDeque::from([from.clone()]);
        let mut seen = BTreeSet::from([from.clone()]);
        let mut parent = BTreeMap::<StateId, StateId>::new();
        while let Some(state) = queue.pop_front() {
            if &state == to {
                let mut path = vec![state.clone()];
                let mut cursor = state;
                while let Some(previous) = parent.get(&cursor).cloned() {
                    path.push(previous.clone());
                    cursor = previous;
                }
                path.reverse();
                return Some(path);
            }
            let mut next = self
                .transitions
                .values()
                .filter(|transition| transition.from() == &state)
                .map(|transition| transition.to().clone())
                .collect::<Vec<_>>();
            next.sort();
            next.dedup();
            for candidate in next {
                if seen.insert(candidate.clone()) {
                    parent.insert(candidate.clone(), state.clone());
                    queue.push_back(candidate);
                }
            }
        }
        None
    }

    /// Reports unreachable states, dead ends, terminal reachability, and static ambiguity.
    #[must_use]
    pub fn analyze(&self) -> MachineAnalysis {
        let reachable = self.reachable_states();
        let mut findings = Vec::new();
        for state in self.states.difference(&reachable) {
            findings.push(MachineFinding::UnreachableState(state.clone()));
        }
        for state in &self.states {
            let outgoing = self
                .transitions
                .values()
                .any(|transition| transition.from() == state);
            if !outgoing && !self.terminal.contains(state) {
                findings.push(MachineFinding::DeadEndState(state.clone()));
            }
            if !self.terminal.contains(state)
                && !self
                    .terminal
                    .iter()
                    .any(|terminal| self.shortest_path(state, terminal).is_some())
            {
                findings.push(MachineFinding::MissingTerminalPath(state.clone()));
            }
        }

        let mut groups = BTreeMap::<(StateId, EventKind, i32), Vec<TransitionId>>::new();
        for transition in self.transitions.values() {
            groups
                .entry((
                    transition.from().clone(),
                    transition.event().clone(),
                    transition.priority_value(),
                ))
                .or_default()
                .push(transition.id().clone());
        }
        for ((state, event, priority), mut transitions) in groups {
            if transitions.len() > 1 {
                transitions.sort();
                findings.push(MachineFinding::StructurallyAmbiguous {
                    state,
                    event,
                    priority,
                    transitions,
                });
            }
        }

        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "machine-analysis-v1")
            .field("machine", &self.digest().0)
            .u64("reachable-count", reachable.len() as u64)
            .u64("finding-count", findings.len() as u64);
        for state in &reachable {
            encoder.text("reachable", state.as_str());
        }
        for finding in &findings {
            encode_finding(finding, &mut encoder);
        }
        MachineAnalysis {
            reachable,
            findings,
            digest: encoder.digest(),
        }
    }

    /// Computes canonical machine identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "state-machine-v1")
            .text("initial", self.initial.as_str())
            .u64("state-count", self.states.len() as u64);
        for state in &self.states {
            encoder
                .text("state", state.as_str())
                .boolean("terminal", self.terminal.contains(state));
        }
        encoder.u64("transition-count", self.transitions.len() as u64);
        for transition in self.transitions.values() {
            encoder
                .text("transition", transition.id().as_str())
                .field("transition-digest", &transition.digest().0);
        }
        encoder.digest()
    }
}

fn dispatch_digest(
    kind: &str,
    state: &StateId,
    event: &EventKind,
    evaluations: &[TransitionEvaluation],
    selected: &[&Transition],
) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "dispatch-decision-v1")
        .text("kind", kind)
        .text("state", state.as_str())
        .text("event", event.as_str())
        .u64("evaluation-count", evaluations.len() as u64);
    for evaluation in evaluations {
        encoder.field("evaluation", &evaluation.digest().0);
    }
    encoder.u64("selected-count", selected.len() as u64);
    for transition in selected {
        encoder.field("selected", &transition.digest().0);
    }
    encoder.digest()
}

fn encode_finding(finding: &MachineFinding, encoder: &mut CanonicalEncoder) {
    match finding {
        MachineFinding::UnreachableState(state) => {
            encoder
                .text("finding", "unreachable")
                .text("state", state.as_str());
        }
        MachineFinding::DeadEndState(state) => {
            encoder
                .text("finding", "dead-end")
                .text("state", state.as_str());
        }
        MachineFinding::MissingTerminalPath(state) => {
            encoder
                .text("finding", "missing-terminal-path")
                .text("state", state.as_str());
        }
        MachineFinding::StructurallyAmbiguous {
            state,
            event,
            priority,
            transitions,
        } => {
            encoder
                .text("finding", "structural-ambiguity")
                .text("state", state.as_str())
                .text("event", event.as_str())
                .i64("priority", i64::from(*priority))
                .u64("transition-count", transitions.len() as u64);
            for transition in transitions {
                encoder.text("transition", transition.as_str());
            }
        }
    }
}

/// Immutable receipt for one state transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateReceipt {
    revision_before: u64,
    revision_after: u64,
    state_before: StateId,
    state_after: StateId,
    event: EventKind,
    transition: TransitionId,
    observation: Digest,
    previous: Digest,
    digest: Digest,
}

impl StateReceipt {
    #[must_use]
    pub const fn revision_before(&self) -> u64 {
        self.revision_before
    }

    #[must_use]
    pub const fn revision_after(&self) -> u64 {
        self.revision_after
    }

    #[must_use]
    pub const fn state_before(&self) -> &StateId {
        &self.state_before
    }

    #[must_use]
    pub const fn state_after(&self) -> &StateId {
        &self.state_after
    }

    #[must_use]
    pub const fn event(&self) -> &EventKind {
        &self.event
    }

    #[must_use]
    pub const fn transition(&self) -> &TransitionId {
        &self.transition
    }

    #[must_use]
    pub const fn observation(&self) -> Digest {
        self.observation
    }

    #[must_use]
    pub const fn previous(&self) -> Digest {
        self.previous
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    fn manufacture(
        revision_before: u64,
        state_before: StateId,
        state_after: StateId,
        event: EventKind,
        transition: TransitionId,
        observation: Digest,
        previous: Digest,
    ) -> Self {
        let revision_after = revision_before.saturating_add(1);
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "state-receipt-v1")
            .u64("revision-before", revision_before)
            .u64("revision-after", revision_after)
            .text("state-before", state_before.as_str())
            .text("state-after", state_after.as_str())
            .text("event", event.as_str())
            .text("transition", transition.as_str())
            .field("observation", &observation.0)
            .field("previous", &previous.0);
        Self {
            revision_before,
            revision_after,
            state_before,
            state_after,
            event,
            transition,
            observation,
            previous,
            digest: encoder.digest(),
        }
    }
}

/// Result of applying an event to an instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyResult {
    Applied(StateReceipt),
    NoTransition(DispatchDecision),
    Ambiguous(DispatchDecision),
}

/// Mutable machine instance with optimistic concurrency and receipt history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineInstance {
    state: StateId,
    revision: u64,
    receipts: Vec<StateReceipt>,
}

impl MachineInstance {
    pub fn new(machine: &StateMachine) -> Result<Self, MachineError> {
        machine.validate()?;
        Ok(Self {
            state: machine.initial().clone(),
            revision: 0,
            receipts: Vec::new(),
        })
    }

    #[must_use]
    pub const fn state(&self) -> &StateId {
        &self.state
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn receipts(&self) -> &[StateReceipt] {
        &self.receipts
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts.last().map_or(Digest::ZERO, StateReceipt::digest)
    }

    /// Applies one event only if the caller observed the current revision.
    pub fn apply(
        &mut self,
        machine: &StateMachine,
        expected_revision: u64,
        event: &EventKind,
        observation: &Observation,
    ) -> Result<ApplyResult, MachineError> {
        if expected_revision != self.revision {
            return Err(MachineError::RevisionConflict {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        match machine.dispatch(&self.state, event, observation) {
            DispatchDecision::Selected { transition, .. } => {
                let receipt = StateReceipt::manufacture(
                    self.revision,
                    self.state.clone(),
                    transition.to().clone(),
                    event.clone(),
                    transition.id().clone(),
                    observation.digest(),
                    self.head(),
                );
                self.state = transition.to().clone();
                self.revision = receipt.revision_after();
                self.receipts.push(receipt.clone());
                Ok(ApplyResult::Applied(receipt))
            }
            decision @ DispatchDecision::NoTransition { .. } => {
                Ok(ApplyResult::NoTransition(decision))
            }
            decision @ DispatchDecision::Ambiguous { .. } => {
                Ok(ApplyResult::Ambiguous(decision))
            }
        }
    }

    /// Replays and validates the state receipt chain from the machine initial state.
    pub fn verify(&self, machine: &StateMachine) -> Result<Digest, InstanceVerificationError> {
        let mut state = machine.initial().clone();
        let mut revision = 0_u64;
        let mut previous = Digest::ZERO;
        for receipt in &self.receipts {
            if receipt.revision_before() != revision
                || receipt.revision_after() != revision.saturating_add(1)
            {
                return Err(InstanceVerificationError::Revision {
                    expected: revision,
                    before: receipt.revision_before(),
                    after: receipt.revision_after(),
                });
            }
            if receipt.state_before() != &state {
                return Err(InstanceVerificationError::State {
                    expected: state,
                    actual: receipt.state_before().clone(),
                });
            }
            if receipt.previous() != previous {
                return Err(InstanceVerificationError::Previous {
                    expected: previous,
                    actual: receipt.previous(),
                });
            }
            let transition = machine
                .transitions
                .get(receipt.transition())
                .ok_or_else(|| InstanceVerificationError::TransitionMissing(
                    receipt.transition().clone(),
                ))?;
            if transition.from() != receipt.state_before()
                || transition.to() != receipt.state_after()
                || transition.event() != receipt.event()
            {
                return Err(InstanceVerificationError::TransitionMismatch(
                    receipt.transition().clone(),
                ));
            }
            let expected = StateReceipt::manufacture(
                receipt.revision_before(),
                receipt.state_before().clone(),
                receipt.state_after().clone(),
                receipt.event().clone(),
                receipt.transition().clone(),
                receipt.observation(),
                receipt.previous(),
            );
            if expected.digest() != receipt.digest() {
                return Err(InstanceVerificationError::Digest {
                    revision: receipt.revision_after(),
                });
            }
            state = receipt.state_after().clone();
            revision = receipt.revision_after();
            previous = receipt.digest();
        }
        if state != self.state || revision != self.revision {
            return Err(InstanceVerificationError::SnapshotMismatch {
                expected_state: state,
                actual_state: self.state.clone(),
                expected_revision: revision,
                actual_revision: self.revision,
            });
        }
        Ok(previous)
    }
}

/// Receipt replay failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstanceVerificationError {
    Revision { expected: u64, before: u64, after: u64 },
    State { expected: StateId, actual: StateId },
    Previous { expected: Digest, actual: Digest },
    TransitionMissing(TransitionId),
    TransitionMismatch(TransitionId),
    Digest { revision: u64 },
    SnapshotMismatch {
        expected_state: StateId,
        actual_state: StateId,
        expected_revision: u64,
        actual_revision: u64,
    },
}

impl Display for InstanceVerificationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Revision { expected, before, after } => write!(
                formatter,
                "receipt revision before={before}, after={after}, expected before={expected}"
            ),
            Self::State { expected, actual } => {
                write!(formatter, "receipt state `{actual}`, expected `{expected}`")
            }
            Self::Previous { expected, actual } => {
                write!(formatter, "receipt predecessor {actual}, expected {expected}")
            }
            Self::TransitionMissing(id) => write!(formatter, "transition `{id}` is missing"),
            Self::TransitionMismatch(id) => {
                write!(formatter, "transition `{id}` does not match receipt")
            }
            Self::Digest { revision } => {
                write!(formatter, "receipt digest mismatch at revision {revision}")
            }
            Self::SnapshotMismatch {
                expected_state,
                actual_state,
                expected_revision,
                actual_revision,
            } => write!(
                formatter,
                "snapshot state/revision `{actual_state}`/{actual_revision}, expected `{expected_state}`/{expected_revision}"
            ),
        }
    }
}

impl std::error::Error for InstanceVerificationError {}

#[cfg(test)]
mod tests {
    use super::{
        ApplyResult, DispatchDecision, EventKind, Guard, MachineFinding, MachineInstance,
        StateId, StateMachine, Transition, TransitionId,
    };
    use crate::model::{Observation, SubjectId};

    fn id(value: &str) -> StateId {
        StateId::new(value).unwrap()
    }

    fn event(value: &str) -> EventKind {
        EventKind::new(value).unwrap()
    }

    fn machine() -> StateMachine {
        let mut machine = StateMachine::new(id("draft"));
        machine.add_state(id("draft"), false).unwrap();
        machine.add_state(id("approved"), false).unwrap();
        machine.add_state(id("released"), true).unwrap();
        machine
            .add_transition(
                Transition::new(
                    TransitionId::new("approve").unwrap(),
                    id("draft"),
                    event("approve"),
                    id("approved"),
                )
                .guarded_by(Guard::Bool {
                    key: "reviewed".to_owned(),
                    expected: true,
                }),
            )
            .unwrap();
        machine
            .add_transition(Transition::new(
                TransitionId::new("release").unwrap(),
                id("approved"),
                event("release"),
                id("released"),
            ))
            .unwrap();
        machine
    }

    #[test]
    fn guarded_dispatch_records_failures() {
        let machine = machine();
        let observation = Observation::new(SubjectId::new("change").unwrap(), 1);
        let decision = machine.dispatch(&id("draft"), &event("approve"), &observation);
        let DispatchDecision::NoTransition { evaluations, .. } = decision else {
            panic!("guard must refuse transition");
        };
        assert_eq!(evaluations.len(), 1);
        assert_eq!(evaluations[0].failures()[0].code(), "MISSING_FACT");
    }

    #[test]
    fn instance_applies_and_replays_receipts() {
        let machine = machine();
        let mut instance = MachineInstance::new(&machine).unwrap();
        let mut observation = Observation::new(SubjectId::new("change").unwrap(), 1);
        observation.insert("reviewed", true).unwrap();
        assert!(matches!(
            instance
                .apply(&machine, 0, &event("approve"), &observation)
                .unwrap(),
            ApplyResult::Applied(_)
        ));
        assert!(matches!(
            instance
                .apply(&machine, 1, &event("release"), &observation)
                .unwrap(),
            ApplyResult::Applied(_)
        ));
        assert_eq!(instance.state().as_str(), "released");
        assert_eq!(instance.verify(&machine).unwrap(), instance.head());
    }

    #[test]
    fn analysis_finds_unreachable_state() {
        let mut machine = machine();
        machine.add_state(id("orphan"), false).unwrap();
        let analysis = machine.analyze();
        assert!(analysis.findings().iter().any(|finding| matches!(
            finding,
            MachineFinding::UnreachableState(state) if state.as_str() == "orphan"
        )));
    }

    #[test]
    fn shortest_path_is_dependency_ordered() {
        let machine = machine();
        assert_eq!(
            machine
                .shortest_path(&id("draft"), &id("released"))
                .unwrap()
                .iter()
                .map(StateId::as_str)
                .collect::<Vec<_>>(),
            ["draft", "approved", "released"]
        );
    }
}
