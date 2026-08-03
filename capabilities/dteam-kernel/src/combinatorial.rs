//! Combinatorial-maximalist composition, guided wizards, and telco-grade topology analysis.
//!
//! The engine preserves every bounded lawful option until an explicit objective selects
//! a Pareto-optimal composition. It never actuates; it manufactures plans and evidence.

use crate::hash::{CanonicalEncoder, Digest};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Display, Formatter};

/// Stable identifier for a capability offered by a component.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FeatureId(String);

impl FeatureId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(CompositionError::InvalidIdentifier("feature"))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FeatureId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One reversible implementation option in the composition space.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentOption {
    id: String,
    provides: BTreeSet<FeatureId>,
    requires: BTreeSet<FeatureId>,
    excludes: BTreeSet<String>,
    cost: u64,
    latency_micros: u64,
    reliability_ppm: u32,
    complexity: u32,
    reversible: bool,
    tags: BTreeSet<String>,
}

impl ComponentOption {
    pub fn new(id: impl Into<String>) -> Result<Self, CompositionError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(CompositionError::InvalidIdentifier("component"));
        }
        Ok(Self {
            id,
            provides: BTreeSet::new(),
            requires: BTreeSet::new(),
            excludes: BTreeSet::new(),
            cost: 0,
            latency_micros: 0,
            reliability_ppm: 1_000_000,
            complexity: 0,
            reversible: true,
            tags: BTreeSet::new(),
        })
    }

    #[must_use]
    pub fn provides(mut self, feature: FeatureId) -> Self {
        self.provides.insert(feature);
        self
    }

    #[must_use]
    pub fn requires(mut self, feature: FeatureId) -> Self {
        self.requires.insert(feature);
        self
    }

    #[must_use]
    pub fn excludes(mut self, component: impl Into<String>) -> Self {
        self.excludes.insert(component.into());
        self
    }

    #[must_use]
    pub const fn economics(mut self, cost: u64, latency_micros: u64, complexity: u32) -> Self {
        self.cost = cost;
        self.latency_micros = latency_micros;
        self.complexity = complexity;
        self
    }

    #[must_use]
    pub const fn reliability(mut self, ppm: u32) -> Self {
        self.reliability_ppm = if ppm > 1_000_000 { 1_000_000 } else { ppm };
        self
    }

    #[must_use]
    pub const fn reversible(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }

    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn provides_features(&self) -> &BTreeSet<FeatureId> {
        &self.provides
    }
    #[must_use]
    pub fn required_features(&self) -> &BTreeSet<FeatureId> {
        &self.requires
    }
    #[must_use]
    pub fn exclusions(&self) -> &BTreeSet<String> {
        &self.excludes
    }
    #[must_use]
    pub const fn cost(&self) -> u64 {
        self.cost
    }
    #[must_use]
    pub const fn latency_micros(&self) -> u64 {
        self.latency_micros
    }
    #[must_use]
    pub const fn reliability_ppm(&self) -> u32 {
        self.reliability_ppm
    }
    #[must_use]
    pub const fn complexity(&self) -> u32 {
        self.complexity
    }
    #[must_use]
    pub const fn is_reversible(&self) -> bool {
        self.reversible
    }
    #[must_use]
    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }
}

/// Explicit bounds for lawful search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRequest {
    required: BTreeSet<FeatureId>,
    forbidden_tags: BTreeSet<String>,
    maximum_components: usize,
    maximum_cost: u64,
    maximum_latency_micros: u64,
    minimum_reliability_ppm: u32,
    require_reversible: bool,
    maximum_results: usize,
}

impl CompositionRequest {
    #[must_use]
    pub fn new(required: impl IntoIterator<Item = FeatureId>) -> Self {
        Self {
            required: required.into_iter().collect(),
            forbidden_tags: BTreeSet::new(),
            maximum_components: 12,
            maximum_cost: u64::MAX,
            maximum_latency_micros: u64::MAX,
            minimum_reliability_ppm: 0,
            require_reversible: false,
            maximum_results: 256,
        }
    }

    #[must_use]
    pub const fn bounds(mut self, components: usize, cost: u64, latency_micros: u64) -> Self {
        self.maximum_components = components;
        self.maximum_cost = cost;
        self.maximum_latency_micros = latency_micros;
        self
    }

    #[must_use]
    pub const fn minimum_reliability(mut self, ppm: u32) -> Self {
        self.minimum_reliability_ppm = ppm;
        self
    }

    #[must_use]
    pub const fn require_reversible(mut self, value: bool) -> Self {
        self.require_reversible = value;
        self
    }

    #[must_use]
    pub const fn maximum_results(mut self, value: usize) -> Self {
        self.maximum_results = value;
        self
    }

    #[must_use]
    pub fn forbid_tag(mut self, tag: impl Into<String>) -> Self {
        self.forbidden_tags.insert(tag.into());
        self
    }
}

/// One admitted composition with canonical identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Composition {
    components: Vec<String>,
    provides: BTreeSet<FeatureId>,
    cost: u64,
    latency_micros: u64,
    reliability_ppm: u32,
    complexity: u32,
    reversible: bool,
    digest: Digest,
}

impl Composition {
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
    #[must_use]
    pub fn provides(&self) -> &BTreeSet<FeatureId> {
        &self.provides
    }
    #[must_use]
    pub const fn cost(&self) -> u64 {
        self.cost
    }
    #[must_use]
    pub const fn latency_micros(&self) -> u64 {
        self.latency_micros
    }
    #[must_use]
    pub const fn reliability_ppm(&self) -> u32 {
        self.reliability_ppm
    }
    #[must_use]
    pub const fn complexity(&self) -> u32 {
        self.complexity
    }
    #[must_use]
    pub const fn is_reversible(&self) -> bool {
        self.reversible
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn dominates(&self, other: &Self) -> bool {
        let no_worse = self.cost <= other.cost
            && self.latency_micros <= other.latency_micros
            && self.complexity <= other.complexity
            && self.reliability_ppm >= other.reliability_ppm;
        let strictly_better = self.cost < other.cost
            || self.latency_micros < other.latency_micros
            || self.complexity < other.complexity
            || self.reliability_ppm > other.reliability_ppm;
        no_worse && strictly_better
    }
}

/// Deterministic composition search result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionSpace {
    lawful: Vec<Composition>,
    pareto: Vec<Composition>,
    explored: usize,
    refused: usize,
    digest: Digest,
}

impl CompositionSpace {
    #[must_use]
    pub fn lawful(&self) -> &[Composition] {
        &self.lawful
    }
    #[must_use]
    pub fn pareto(&self) -> &[Composition] {
        &self.pareto
    }
    #[must_use]
    pub const fn explored(&self) -> usize {
        self.explored
    }
    #[must_use]
    pub const fn refused(&self) -> usize {
        self.refused
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompositionError {
    InvalidIdentifier(&'static str),
    DuplicateComponent(String),
    EmptyRequirement,
    SearchBoundTooLarge(usize),
}

impl Display for CompositionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier(kind) => write!(formatter, "invalid {kind} identifier"),
            Self::DuplicateComponent(id) => write!(formatter, "duplicate component `{id}`"),
            Self::EmptyRequirement => {
                formatter.write_str("composition requires at least one feature")
            }
            Self::SearchBoundTooLarge(size) => write!(
                formatter,
                "search contains {size} components; maximum is 24"
            ),
        }
    }
}

/// Bounded option graph. Search is exhaustive inside the admitted bound.
#[derive(Clone, Debug, Default)]
pub struct CombinatorialEngine {
    components: BTreeMap<String, ComponentOption>,
}

impl CombinatorialEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, component: ComponentOption) -> Result<(), CompositionError> {
        if self.components.contains_key(component.id()) {
            return Err(CompositionError::DuplicateComponent(
                component.id().to_owned(),
            ));
        }
        self.components.insert(component.id().to_owned(), component);
        Ok(())
    }

    #[must_use]
    pub fn components(&self) -> &BTreeMap<String, ComponentOption> {
        &self.components
    }

    pub fn explore(
        &self,
        request: &CompositionRequest,
    ) -> Result<CompositionSpace, CompositionError> {
        if request.required.is_empty() {
            return Err(CompositionError::EmptyRequirement);
        }
        if self.components.len() > 24 {
            return Err(CompositionError::SearchBoundTooLarge(self.components.len()));
        }
        let options = self.components.values().collect::<Vec<_>>();
        let combinations = 1_u64 << options.len();
        let mut explored = 0_usize;
        let mut refused = 0_usize;
        let mut lawful = Vec::new();
        for mask in 1..combinations {
            explored += 1;
            if mask.count_ones() as usize > request.maximum_components {
                refused += 1;
                continue;
            }
            let selected = options
                .iter()
                .enumerate()
                .filter_map(|(index, option)| ((mask & (1_u64 << index)) != 0).then_some(*option))
                .collect::<Vec<_>>();
            match admit_composition(&selected, request) {
                Some(composition) => lawful.push(composition),
                None => refused += 1,
            }
        }
        lawful.sort_by(composition_order);
        lawful.truncate(request.maximum_results);
        let pareto = lawful
            .iter()
            .filter(|candidate| {
                !lawful
                    .iter()
                    .any(|other| other.digest != candidate.digest && other.dominates(candidate))
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "combinatorial-space-v1")
            .u64("explored", explored as u64)
            .u64("refused", refused as u64);
        for composition in &lawful {
            encoder.field("composition", &composition.digest().0);
        }
        Ok(CompositionSpace {
            lawful,
            pareto,
            explored,
            refused,
            digest: encoder.digest(),
        })
    }
}

fn admit_composition(
    selected: &[&ComponentOption],
    request: &CompositionRequest,
) -> Option<Composition> {
    let ids = selected
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    if selected
        .iter()
        .any(|item| item.excludes.iter().any(|excluded| ids.contains(excluded)))
    {
        return None;
    }
    if selected.iter().any(|item| {
        item.tags
            .iter()
            .any(|tag| request.forbidden_tags.contains(tag))
    }) {
        return None;
    }
    let provides = selected
        .iter()
        .flat_map(|item| item.provides.iter().cloned())
        .collect::<BTreeSet<_>>();
    let requires = selected
        .iter()
        .flat_map(|item| item.requires.iter().cloned())
        .collect::<BTreeSet<_>>();
    if !request.required.is_subset(&provides) || !requires.is_subset(&provides) {
        return None;
    }
    let cost = selected.iter().map(|item| item.cost).sum::<u64>();
    let latency_micros = selected.iter().map(|item| item.latency_micros).sum::<u64>();
    let complexity = selected.iter().map(|item| item.complexity).sum::<u32>();
    let reliability_ppm = selected
        .iter()
        .map(|item| item.reliability_ppm)
        .min()
        .unwrap_or(0);
    let reversible = selected.iter().all(|item| item.reversible);
    if cost > request.maximum_cost
        || latency_micros > request.maximum_latency_micros
        || reliability_ppm < request.minimum_reliability_ppm
        || (request.require_reversible && !reversible)
    {
        return None;
    }
    let components = ids.into_iter().collect::<Vec<_>>();
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "composition-v1")
        .u64("cost", cost)
        .u64("latency", latency_micros)
        .u64("reliability", u64::from(reliability_ppm))
        .u64("complexity", u64::from(complexity));
    for component in &components {
        encoder.text("component", component);
    }
    Some(Composition {
        components,
        provides,
        cost,
        latency_micros,
        reliability_ppm,
        complexity,
        reversible,
        digest: encoder.digest(),
    })
}

fn composition_order(left: &Composition, right: &Composition) -> Ordering {
    left.cost
        .cmp(&right.cost)
        .then_with(|| left.latency_micros.cmp(&right.latency_micros))
        .then_with(|| right.reliability_ppm.cmp(&left.reliability_ppm))
        .then_with(|| left.complexity.cmp(&right.complexity))
        .then_with(|| left.components.cmp(&right.components))
}

/// Wizard answer values are deliberately closed and serializable without dependencies.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WizardValue {
    Choice(String),
    Boolean(bool),
    Number(u64),
    Choices(BTreeSet<String>),
}

/// One guided question with explicit admissible responses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WizardQuestion {
    id: String,
    prompt: String,
    choices: Vec<String>,
    required: bool,
    rationale: String,
}

impl WizardQuestion {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
    #[must_use]
    pub fn choices(&self) -> &[String] {
        &self.choices
    }
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
    #[must_use]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }
}

/// Wizard session that compiles user intent into a composition request.
#[derive(Clone, Debug, Default)]
pub struct VisionWizard {
    questions: BTreeMap<String, WizardQuestion>,
    answers: BTreeMap<String, WizardValue>,
}

impl VisionWizard {
    #[must_use]
    pub fn standard() -> Self {
        let mut wizard = Self::default();
        let entries = [
            (
                "mode",
                "What operating mode is primary?",
                vec!["developer", "edge", "telco", "enterprise"],
                "Sets the topology and governance envelope.",
            ),
            (
                "availability",
                "What availability tier is required?",
                vec!["standard", "ha", "carrier"],
                "Determines redundancy and reliability constraints.",
            ),
            (
                "authority",
                "Must every side effect pass through BRCE?",
                vec!["yes", "no"],
                "Controls whether actuation can receive standing.",
            ),
            (
                "offline",
                "Must the system operate offline?",
                vec!["yes", "no"],
                "Selects local-first and deterministic transports.",
            ),
            (
                "reversible",
                "Must every selected option remain reversible?",
                vec!["yes", "no"],
                "Preserves lawful alternatives before irreversible selection.",
            ),
        ];
        for (id, prompt, choices, rationale) in entries {
            wizard.questions.insert(
                id.to_owned(),
                WizardQuestion {
                    id: id.to_owned(),
                    prompt: prompt.to_owned(),
                    choices: choices.into_iter().map(str::to_owned).collect(),
                    required: true,
                    rationale: rationale.to_owned(),
                },
            );
        }
        wizard
    }

    pub fn answer(&mut self, id: &str, value: WizardValue) -> Result<(), WizardError> {
        let question = self
            .questions
            .get(id)
            .ok_or_else(|| WizardError::UnknownQuestion(id.to_owned()))?;
        if let WizardValue::Choice(choice) = &value {
            if !question.choices.is_empty() && !question.choices.contains(choice) {
                return Err(WizardError::InvalidChoice {
                    question: id.to_owned(),
                    choice: choice.clone(),
                });
            }
        }
        self.answers.insert(id.to_owned(), value);
        Ok(())
    }

    #[must_use]
    pub fn unanswered(&self) -> Vec<&WizardQuestion> {
        self.questions
            .values()
            .filter(|question| question.required && !self.answers.contains_key(question.id()))
            .collect()
    }

    pub fn compile(&self) -> Result<WizardPlan, WizardError> {
        let missing = self
            .unanswered()
            .into_iter()
            .map(|question| question.id.clone())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(WizardError::MissingAnswers(missing));
        }
        let choice = |id: &str| match self.answers.get(id) {
            Some(WizardValue::Choice(value)) => value.as_str(),
            _ => "",
        };
        let mut required = BTreeSet::new();
        for feature in ["identity", "receipts", "doctor"] {
            required.insert(FeatureId::new(feature).expect("static feature"));
        }
        if choice("authority") == "yes" {
            required.insert(FeatureId::new("brce").expect("static feature"));
        }
        if choice("offline") == "yes" {
            required.insert(FeatureId::new("offline").expect("static feature"));
        }
        if choice("mode") == "telco" {
            required.insert(FeatureId::new("telco").expect("static feature"));
        }
        let minimum_reliability = match choice("availability") {
            "carrier" => 999_999,
            "ha" => 999_000,
            _ => 990_000,
        };
        let request = CompositionRequest::new(required)
            .bounds(
                12,
                10_000,
                if choice("mode") == "edge" {
                    5_000
                } else {
                    100_000
                },
            )
            .minimum_reliability(minimum_reliability)
            .require_reversible(choice("reversible") == "yes")
            .forbid_tag(if choice("offline") == "yes" {
                "network-required"
            } else {
                "never"
            });
        let commands = vec![
            "dteam-doctor graph".to_owned(),
            "dteam-doctor compose".to_owned(),
            "dteam-doctor telco".to_owned(),
            "dteam-doctor crown".to_owned(),
        ];
        let mut encoder = CanonicalEncoder::new();
        encoder.text("type", "vision-wizard-plan-v1");
        for (id, answer) in &self.answers {
            encoder.text("answer", &format!("{id}:{answer:?}"));
        }
        Ok(WizardPlan {
            request,
            commands,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub fn questions(&self) -> &BTreeMap<String, WizardQuestion> {
        &self.questions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WizardPlan {
    request: CompositionRequest,
    commands: Vec<String>,
    digest: Digest,
}

impl WizardPlan {
    #[must_use]
    pub fn request(&self) -> &CompositionRequest {
        &self.request
    }
    #[must_use]
    pub fn commands(&self) -> &[String] {
        &self.commands
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WizardError {
    UnknownQuestion(String),
    InvalidChoice { question: String, choice: String },
    MissingAnswers(Vec<String>),
}

impl Display for WizardError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownQuestion(id) => write!(formatter, "unknown wizard question `{id}`"),
            Self::InvalidChoice { question, choice } => {
                write!(formatter, "invalid choice `{choice}` for `{question}`")
            }
            Self::MissingAnswers(ids) => write!(formatter, "missing answers: {}", ids.join(",")),
        }
    }
}

/// Telco topology node role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TelcoRole {
    Edge,
    Access,
    Transport,
    Core,
    Control,
    Data,
    Observability,
}

/// Failure domain identity used for redundancy analysis.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FailureDomain(String);

impl FailureDomain {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelcoNode {
    id: String,
    role: TelcoRole,
    domain: FailureDomain,
    capacity: u64,
}

impl TelcoNode {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        role: TelcoRole,
        domain: FailureDomain,
        capacity: u64,
    ) -> Self {
        Self {
            id: id.into(),
            role,
            domain,
            capacity,
        }
    }
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub const fn role(&self) -> TelcoRole {
        self.role
    }
    #[must_use]
    pub fn domain(&self) -> &FailureDomain {
        &self.domain
    }
    #[must_use]
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelcoLink {
    from: String,
    to: String,
    latency_micros: u64,
    capacity: u64,
    reliability_ppm: u32,
}

impl TelcoLink {
    #[must_use]
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        latency_micros: u64,
        capacity: u64,
        reliability_ppm: u32,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            latency_micros,
            capacity,
            reliability_ppm: reliability_ppm.min(1_000_000),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceObjective {
    pub maximum_latency_micros: u64,
    pub minimum_capacity: u64,
    pub minimum_reliability_ppm: u32,
    pub minimum_failure_domains: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelcoPath {
    nodes: Vec<String>,
    latency_micros: u64,
    capacity: u64,
    reliability_ppm: u32,
    failure_domains: BTreeSet<FailureDomain>,
    digest: Digest,
}

impl TelcoPath {
    #[must_use]
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }
    #[must_use]
    pub const fn latency_micros(&self) -> u64 {
        self.latency_micros
    }
    #[must_use]
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }
    #[must_use]
    pub const fn reliability_ppm(&self) -> u32 {
        self.reliability_ppm
    }
    #[must_use]
    pub fn failure_domains(&self) -> &BTreeSet<FailureDomain> {
        &self.failure_domains
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelcoAssessment {
    compliant_paths: Vec<TelcoPath>,
    disjoint_path_count: usize,
    single_points_of_failure: Vec<String>,
    standing: &'static str,
    digest: Digest,
}

impl TelcoAssessment {
    #[must_use]
    pub fn compliant_paths(&self) -> &[TelcoPath] {
        &self.compliant_paths
    }
    #[must_use]
    pub const fn disjoint_path_count(&self) -> usize {
        self.disjoint_path_count
    }
    #[must_use]
    pub fn single_points_of_failure(&self) -> &[String] {
        &self.single_points_of_failure
    }
    #[must_use]
    pub const fn standing(&self) -> &'static str {
        self.standing
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Default)]
pub struct TelcoTopology {
    nodes: BTreeMap<String, TelcoNode>,
    links: Vec<TelcoLink>,
}

impl TelcoTopology {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: TelcoNode) {
        self.nodes.insert(node.id.clone(), node);
    }
    pub fn add_link(&mut self, link: TelcoLink) {
        self.links.push(link);
    }

    #[must_use]
    pub fn assess(&self, from: &str, to: &str, objective: &ServiceObjective) -> TelcoAssessment {
        let all = self.paths(from, to, self.nodes.len().max(1));
        let mut compliant_paths = all
            .into_iter()
            .filter(|path| {
                path.latency_micros <= objective.maximum_latency_micros
                    && path.capacity >= objective.minimum_capacity
                    && path.reliability_ppm >= objective.minimum_reliability_ppm
                    && path.failure_domains.len() >= objective.minimum_failure_domains
            })
            .collect::<Vec<_>>();
        compliant_paths.sort_by(|a, b| {
            a.latency_micros
                .cmp(&b.latency_micros)
                .then_with(|| b.reliability_ppm.cmp(&a.reliability_ppm))
                .then_with(|| a.nodes.cmp(&b.nodes))
        });
        let disjoint_path_count = maximum_domain_disjoint(&compliant_paths);
        let mut counts = BTreeMap::<String, usize>::new();
        for path in &compliant_paths {
            for node in path
                .nodes
                .iter()
                .skip(1)
                .take(path.nodes.len().saturating_sub(2))
            {
                *counts.entry(node.clone()).or_default() += 1;
            }
        }
        let single_points_of_failure = counts
            .into_iter()
            .filter_map(|(node, count)| {
                (count == compliant_paths.len() && count > 0).then_some(node)
            })
            .collect::<Vec<_>>();
        let standing = if compliant_paths.is_empty() {
            "BLOCKED"
        } else if disjoint_path_count >= 2 && single_points_of_failure.is_empty() {
            "ALIVE"
        } else {
            "PARTIAL_ALIVE"
        };
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "telco-assessment-v1")
            .text("standing", standing)
            .u64("disjoint", disjoint_path_count as u64);
        for path in &compliant_paths {
            encoder.field("path", &path.digest.0);
        }
        TelcoAssessment {
            compliant_paths,
            disjoint_path_count,
            single_points_of_failure,
            standing,
            digest: encoder.digest(),
        }
    }

    fn paths(&self, from: &str, to: &str, maximum_hops: usize) -> Vec<TelcoPath> {
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return Vec::new();
        }
        let mut queue = VecDeque::from([(vec![from.to_owned()], 0_u64, u64::MAX, 1_000_000_u32)]);
        let mut results = Vec::new();
        while let Some((nodes, latency, capacity, reliability)) = queue.pop_front() {
            let current = nodes.last().expect("path is non-empty");
            if current == to {
                let domains = nodes
                    .iter()
                    .filter_map(|id| self.nodes.get(id).map(|node| node.domain.clone()))
                    .collect::<BTreeSet<_>>();
                let mut encoder = CanonicalEncoder::new();
                encoder
                    .text("type", "telco-path-v1")
                    .u64("latency", latency)
                    .u64("capacity", capacity)
                    .u64("reliability", u64::from(reliability));
                for node in &nodes {
                    encoder.text("node", node);
                }
                results.push(TelcoPath {
                    nodes,
                    latency_micros: latency,
                    capacity,
                    reliability_ppm: reliability,
                    failure_domains: domains,
                    digest: encoder.digest(),
                });
                continue;
            }
            if nodes.len() > maximum_hops {
                continue;
            }
            for link in self.links.iter().filter(|link| &link.from == current) {
                if nodes.contains(&link.to) {
                    continue;
                }
                let mut next = nodes.clone();
                next.push(link.to.clone());
                queue.push_back((
                    next,
                    latency.saturating_add(link.latency_micros),
                    capacity.min(link.capacity),
                    reliability.min(link.reliability_ppm),
                ));
            }
        }
        results
    }

    #[must_use]
    pub fn standard() -> Self {
        let mut topology = Self::new();
        for node in [
            TelcoNode::new(
                "edge-a",
                TelcoRole::Edge,
                FailureDomain::new("zone-a"),
                10_000,
            ),
            TelcoNode::new(
                "edge-b",
                TelcoRole::Edge,
                FailureDomain::new("zone-b"),
                10_000,
            ),
            TelcoNode::new(
                "transport-a",
                TelcoRole::Transport,
                FailureDomain::new("metro-a"),
                20_000,
            ),
            TelcoNode::new(
                "transport-b",
                TelcoRole::Transport,
                FailureDomain::new("metro-b"),
                20_000,
            ),
            TelcoNode::new(
                "core-a",
                TelcoRole::Core,
                FailureDomain::new("region-a"),
                50_000,
            ),
            TelcoNode::new(
                "core-b",
                TelcoRole::Core,
                FailureDomain::new("region-b"),
                50_000,
            ),
        ] {
            topology.add_node(node);
        }
        for link in [
            TelcoLink::new("edge-a", "transport-a", 500, 10_000, 999_999),
            TelcoLink::new("transport-a", "core-a", 800, 10_000, 999_999),
            TelcoLink::new("edge-a", "transport-b", 650, 8_000, 999_999),
            TelcoLink::new("transport-b", "core-a", 900, 8_000, 999_999),
            TelcoLink::new("edge-b", "transport-b", 500, 10_000, 999_999),
            TelcoLink::new("transport-b", "core-b", 800, 10_000, 999_999),
            TelcoLink::new("edge-b", "transport-a", 650, 8_000, 999_999),
            TelcoLink::new("transport-a", "core-b", 900, 8_000, 999_999),
        ] {
            topology.add_link(link);
        }
        topology
    }
}

fn maximum_domain_disjoint(paths: &[TelcoPath]) -> usize {
    let mut selected: Vec<&TelcoPath> = Vec::new();
    'candidate: for path in paths {
        let internal = path
            .failure_domains
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for existing in &selected {
            let existing_domains = existing
                .failure_domains
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if !internal.is_disjoint(&existing_domains) {
                continue 'candidate;
            }
        }
        selected.push(path);
    }
    selected.len()
}

/// Standard blue-ocean component catalog used by the doctor and wizard.
#[must_use]
pub fn standard_combinatorial_engine() -> CombinatorialEngine {
    let feature = |value| FeatureId::new(value).expect("static feature");
    let mut engine = CombinatorialEngine::new();
    let options = [
        ComponentOption::new("local-kernel")
            .unwrap()
            .provides(feature("identity"))
            .provides(feature("receipts"))
            .provides(feature("offline"))
            .economics(10, 200, 2)
            .reliability(999_999)
            .tag("local-first"),
        ComponentOption::new("brce-runtime")
            .unwrap()
            .provides(feature("brce"))
            .requires(feature("receipts"))
            .economics(20, 400, 3)
            .reliability(999_999),
        ComponentOption::new("doctor-plane")
            .unwrap()
            .provides(feature("doctor"))
            .requires(feature("identity"))
            .economics(5, 50, 1)
            .reliability(1_000_000),
        ComponentOption::new("telco-fabric")
            .unwrap()
            .provides(feature("telco"))
            .requires(feature("receipts"))
            .economics(100, 2_000, 5)
            .reliability(999_999)
            .tag("carrier"),
        ComponentOption::new("cloud-control")
            .unwrap()
            .provides(feature("doctor"))
            .provides(feature("telco"))
            .economics(60, 5_000, 4)
            .reliability(999_000)
            .tag("network-required")
            .excludes("local-only"),
        ComponentOption::new("local-only")
            .unwrap()
            .provides(feature("offline"))
            .economics(1, 10, 1)
            .reliability(999_999)
            .excludes("cloud-control"),
        ComponentOption::new("dual-region-replay")
            .unwrap()
            .provides(feature("telco"))
            .requires(feature("receipts"))
            .economics(80, 1_500, 4)
            .reliability(1_000_000)
            .tag("carrier"),
    ];
    for option in options {
        engine.insert(option).expect("unique standard option");
    }
    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_search_preserves_multiple_lawful_options() {
        let engine = standard_combinatorial_engine();
        let request = CompositionRequest::new([
            FeatureId::new("identity").unwrap(),
            FeatureId::new("doctor").unwrap(),
        ])
        .bounds(5, 200, 10_000);
        let space = engine.explore(&request).unwrap();
        assert!(space.lawful().len() >= 2);
        assert!(!space.pareto().is_empty());
        assert!(space.explored() > space.lawful().len());
    }

    #[test]
    fn wizard_compiles_telco_carrier_constraints() {
        let mut wizard = VisionWizard::standard();
        for (id, choice) in [
            ("mode", "telco"),
            ("availability", "carrier"),
            ("authority", "yes"),
            ("offline", "yes"),
            ("reversible", "yes"),
        ] {
            wizard
                .answer(id, WizardValue::Choice(choice.to_owned()))
                .unwrap();
        }
        let plan = wizard.compile().unwrap();
        let space = standard_combinatorial_engine()
            .explore(plan.request())
            .unwrap();
        assert!(!space.lawful().is_empty());
        assert_eq!(plan.commands().len(), 4);
    }

    #[test]
    fn telco_assessment_requires_redundant_compliant_paths() {
        let topology = TelcoTopology::standard();
        let objective = ServiceObjective {
            maximum_latency_micros: 2_000,
            minimum_capacity: 8_000,
            minimum_reliability_ppm: 999_999,
            minimum_failure_domains: 3,
        };
        let assessment = topology.assess("edge-a", "core-a", &objective);
        assert!(assessment.compliant_paths().len() >= 2);
        assert_eq!(assessment.standing(), "ALIVE");
        assert!(assessment.single_points_of_failure().is_empty());
    }

    #[test]
    fn impossible_slo_is_blocked() {
        let topology = TelcoTopology::standard();
        let objective = ServiceObjective {
            maximum_latency_micros: 100,
            minimum_capacity: 100_000,
            minimum_reliability_ppm: 1_000_000,
            minimum_failure_domains: 6,
        };
        assert_eq!(
            topology.assess("edge-a", "core-a", &objective).standing(),
            "BLOCKED"
        );
    }
}
