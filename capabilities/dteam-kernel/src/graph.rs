//! Deterministic capability dependency graph and closure planner.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{AuthorityId, CapabilityId, OperationId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// Declares one composable capability surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    id: CapabilityId,
    dependencies: BTreeSet<CapabilityId>,
    operations: BTreeSet<OperationId>,
    authorities: BTreeSet<AuthorityId>,
    reversible: bool,
    cost_units: u64,
}

impl Capability {
    /// Creates a capability with no dependencies or operations.
    #[must_use]
    pub fn new(id: CapabilityId) -> Self {
        Self {
            id,
            dependencies: BTreeSet::new(),
            operations: BTreeSet::new(),
            authorities: BTreeSet::new(),
            reversible: true,
            cost_units: 1,
        }
    }

    /// Adds a required capability.
    #[must_use]
    pub fn depends_on(mut self, dependency: CapabilityId) -> Self {
        self.dependencies.insert(dependency);
        self
    }

    /// Adds a supported operation.
    #[must_use]
    pub fn supports(mut self, operation: OperationId) -> Self {
        self.operations.insert(operation);
        self
    }

    /// Adds an authority allowed to actuate this capability.
    #[must_use]
    pub fn allows(mut self, authority: AuthorityId) -> Self {
        self.authorities.insert(authority);
        self
    }

    /// Sets reversibility metadata.
    #[must_use]
    pub const fn reversible(mut self, value: bool) -> Self {
        self.reversible = value;
        self
    }

    /// Sets an abstract deterministic cost used by bounded planning.
    #[must_use]
    pub const fn cost_units(mut self, value: u64) -> Self {
        self.cost_units = value;
        self
    }

    #[must_use]
    pub const fn id(&self) -> &CapabilityId {
        &self.id
    }

    #[must_use]
    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = &CapabilityId> {
        self.dependencies.iter()
    }

    #[must_use]
    pub fn supports_operation(&self, operation: &OperationId) -> bool {
        self.operations.contains(operation)
    }

    #[must_use]
    pub fn allows_authority(&self, authority: &AuthorityId) -> bool {
        self.authorities.is_empty() || self.authorities.contains(authority)
    }

    #[must_use]
    pub const fn is_reversible(&self) -> bool {
        self.reversible
    }

    #[must_use]
    pub const fn cost(&self) -> u64 {
        self.cost_units
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "capability-v1")
            .text("id", self.id.as_str())
            .boolean("reversible", self.reversible)
            .u64("cost", self.cost_units)
            .u64("dependency-count", self.dependencies.len() as u64);
        for dependency in &self.dependencies {
            encoder.text("dependency", dependency.as_str());
        }
        encoder.u64("operation-count", self.operations.len() as u64);
        for operation in &self.operations {
            encoder.text("operation", operation.as_str());
        }
        encoder.u64("authority-count", self.authorities.len() as u64);
        for authority in &self.authorities {
            encoder.text("authority", authority.as_str());
        }
        encoder.digest()
    }
}

/// Dependency-closed deterministic execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityPlan {
    ordered: Vec<CapabilityId>,
    total_cost: u64,
    irreversible: Vec<CapabilityId>,
    digest: Digest,
}

impl CapabilityPlan {
    #[must_use]
    pub fn ordered(&self) -> &[CapabilityId] {
        &self.ordered
    }

    #[must_use]
    pub const fn total_cost(&self) -> u64 {
        self.total_cost
    }

    #[must_use]
    pub fn irreversible(&self) -> &[CapabilityId] {
        &self.irreversible
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Errors that prevent a dependency-closed plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    DuplicateCapability(CapabilityId),
    MissingCapability(CapabilityId),
    MissingDependency {
        capability: CapabilityId,
        dependency: CapabilityId,
    },
    Cycle(Vec<CapabilityId>),
    BudgetExceeded {
        required: u64,
        maximum: u64,
    },
}

impl Display for GraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCapability(id) => write!(formatter, "duplicate capability `{id}`"),
            Self::MissingCapability(id) => write!(formatter, "missing capability `{id}`"),
            Self::MissingDependency {
                capability,
                dependency,
            } => write!(
                formatter,
                "capability `{capability}` depends on missing `{dependency}`"
            ),
            Self::Cycle(path) => {
                formatter.write_str("capability cycle:")?;
                for id in path {
                    write!(formatter, " {id}")?;
                }
                Ok(())
            }
            Self::BudgetExceeded { required, maximum } => {
                write!(formatter, "plan cost {required} exceeds budget {maximum}")
            }
        }
    }
}

impl std::error::Error for GraphError {}

/// Canonical graph of every admitted capability.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilityGraph {
    capabilities: BTreeMap<CapabilityId, Capability>,
}

impl CapabilityGraph {
    /// Starts an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a capability, refusing identity replacement.
    pub fn insert(&mut self, capability: Capability) -> Result<(), GraphError> {
        let id = capability.id.clone();
        if self.capabilities.contains_key(&id) {
            return Err(GraphError::DuplicateCapability(id));
        }
        self.capabilities.insert(id, capability);
        Ok(())
    }

    /// Looks up a capability.
    #[must_use]
    pub fn get(&self, id: &CapabilityId) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// Returns every capability in identity order.
    pub fn capabilities(&self) -> impl ExactSizeIterator<Item = &Capability> {
        self.capabilities.values()
    }

    /// Validates that all referenced dependencies exist and the graph is acyclic.
    pub fn validate(&self) -> Result<(), GraphError> {
        for capability in self.capabilities.values() {
            for dependency in &capability.dependencies {
                if !self.capabilities.contains_key(dependency) {
                    return Err(GraphError::MissingDependency {
                        capability: capability.id.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }
        let requested: BTreeSet<_> = self.capabilities.keys().cloned().collect();
        self.resolve_internal(&requested, None).map(|_| ())
    }

    /// Resolves requested capabilities plus all transitive dependencies.
    pub fn resolve(
        &self,
        requested: impl IntoIterator<Item = CapabilityId>,
    ) -> Result<CapabilityPlan, GraphError> {
        let requested: BTreeSet<_> = requested.into_iter().collect();
        self.resolve_internal(&requested, None)
    }

    /// Resolves a dependency closure under a maximum abstract cost.
    pub fn resolve_bounded(
        &self,
        requested: impl IntoIterator<Item = CapabilityId>,
        maximum_cost: u64,
    ) -> Result<CapabilityPlan, GraphError> {
        let requested: BTreeSet<_> = requested.into_iter().collect();
        self.resolve_internal(&requested, Some(maximum_cost))
    }

    fn resolve_internal(
        &self,
        requested: &BTreeSet<CapabilityId>,
        maximum_cost: Option<u64>,
    ) -> Result<CapabilityPlan, GraphError> {
        for id in requested {
            if !self.capabilities.contains_key(id) {
                return Err(GraphError::MissingCapability(id.clone()));
            }
        }

        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Mark {
            Visiting,
            Complete,
        }

        fn visit(
            graph: &CapabilityGraph,
            id: &CapabilityId,
            marks: &mut BTreeMap<CapabilityId, Mark>,
            stack: &mut Vec<CapabilityId>,
            ordered: &mut Vec<CapabilityId>,
        ) -> Result<(), GraphError> {
            match marks.get(id) {
                Some(Mark::Complete) => return Ok(()),
                Some(Mark::Visiting) => {
                    let start = stack.iter().position(|entry| entry == id).unwrap_or(0);
                    let mut cycle = stack[start..].to_vec();
                    cycle.push(id.clone());
                    return Err(GraphError::Cycle(cycle));
                }
                None => {}
            }

            let capability = graph
                .capabilities
                .get(id)
                .ok_or_else(|| GraphError::MissingCapability(id.clone()))?;
            marks.insert(id.clone(), Mark::Visiting);
            stack.push(id.clone());
            for dependency in &capability.dependencies {
                if !graph.capabilities.contains_key(dependency) {
                    return Err(GraphError::MissingDependency {
                        capability: id.clone(),
                        dependency: dependency.clone(),
                    });
                }
                visit(graph, dependency, marks, stack, ordered)?;
            }
            stack.pop();
            marks.insert(id.clone(), Mark::Complete);
            ordered.push(id.clone());
            Ok(())
        }

        let mut marks = BTreeMap::new();
        let mut stack = Vec::new();
        let mut ordered = Vec::new();
        for id in requested {
            visit(self, id, &mut marks, &mut stack, &mut ordered)?;
        }

        let total_cost = ordered.iter().try_fold(0_u64, |total, id| {
            total
                .checked_add(self.capabilities[id].cost())
                .ok_or(GraphError::BudgetExceeded {
                    required: u64::MAX,
                    maximum: maximum_cost.unwrap_or(u64::MAX),
                })
        })?;
        if let Some(maximum) = maximum_cost {
            if total_cost > maximum {
                return Err(GraphError::BudgetExceeded {
                    required: total_cost,
                    maximum,
                });
            }
        }

        let irreversible = ordered
            .iter()
            .filter(|id| !self.capabilities[*id].is_reversible())
            .cloned()
            .collect::<Vec<_>>();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "capability-plan-v1")
            .u64("count", ordered.len() as u64)
            .u64("cost", total_cost);
        for id in &ordered {
            encoder.text("capability", id.as_str());
            encoder.field("capability-digest", &self.capabilities[id].digest().0);
        }
        let digest = encoder.digest();
        Ok(CapabilityPlan {
            ordered,
            total_cost,
            irreversible,
            digest,
        })
    }

    /// Computes a canonical identity for the entire graph.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "capability-graph-v1")
            .u64("count", self.capabilities.len() as u64);
        for capability in self.capabilities.values() {
            encoder
                .text("capability", capability.id.as_str())
                .field("digest", &capability.digest().0);
        }
        encoder.digest()
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability, CapabilityGraph, GraphError};
    use crate::model::CapabilityId;

    fn id(value: &str) -> CapabilityId {
        CapabilityId::new(value).unwrap()
    }

    #[test]
    fn closure_orders_dependencies_before_dependents() {
        let mut graph = CapabilityGraph::new();
        graph.insert(Capability::new(id("observe"))).unwrap();
        graph
            .insert(Capability::new(id("admit")).depends_on(id("observe")))
            .unwrap();
        graph
            .insert(Capability::new(id("actuate")).depends_on(id("admit")))
            .unwrap();
        let plan = graph.resolve([id("actuate")]).unwrap();
        assert_eq!(
            plan.ordered()
                .iter()
                .map(CapabilityId::as_str)
                .collect::<Vec<_>>(),
            ["observe", "admit", "actuate"]
        );
    }

    #[test]
    fn cycle_is_refused_with_path() {
        let mut graph = CapabilityGraph::new();
        graph
            .insert(Capability::new(id("a")).depends_on(id("b")))
            .unwrap();
        graph
            .insert(Capability::new(id("b")).depends_on(id("a")))
            .unwrap();
        let error = graph.resolve([id("a")]).unwrap_err();
        assert!(matches!(error, GraphError::Cycle(_)));
    }

    #[test]
    fn bounded_resolution_refuses_excess_cost() {
        let mut graph = CapabilityGraph::new();
        graph
            .insert(Capability::new(id("expensive")).cost_units(10))
            .unwrap();
        assert_eq!(
            graph.resolve_bounded([id("expensive")], 9).unwrap_err(),
            GraphError::BudgetExceeded {
                required: 10,
                maximum: 9
            }
        );
    }
}
