//! Typed provenance graph with validated relations, lineage, impact, and witnesses.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::FactValue;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Display, Formatter};

/// Stable provenance node identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(String);

impl NodeId {
    /// Creates a non-empty provenance identity.
    pub fn new(value: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ProvenanceError::EmptyNodeId);
        }
        Ok(Self(value))
    }

    /// Returns the identity text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NodeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Typed provenance node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceNode {
    Entity {
        id: NodeId,
        attributes: BTreeMap<String, FactValue>,
    },
    Activity {
        id: NodeId,
        started_at: u64,
        ended_at: Option<u64>,
        attributes: BTreeMap<String, FactValue>,
    },
    Agent {
        id: NodeId,
        attributes: BTreeMap<String, FactValue>,
    },
}

impl ProvenanceNode {
    /// Creates an entity.
    #[must_use]
    pub fn entity(id: NodeId) -> Self {
        Self::Entity {
            id,
            attributes: BTreeMap::new(),
        }
    }

    /// Creates an activity.
    #[must_use]
    pub fn activity(id: NodeId, started_at: u64) -> Self {
        Self::Activity {
            id,
            started_at,
            ended_at: None,
            attributes: BTreeMap::new(),
        }
    }

    /// Creates an agent.
    #[must_use]
    pub fn agent(id: NodeId) -> Self {
        Self::Agent {
            id,
            attributes: BTreeMap::new(),
        }
    }

    /// Returns the node identity.
    #[must_use]
    pub const fn id(&self) -> &NodeId {
        match self {
            Self::Entity { id, .. } | Self::Activity { id, .. } | Self::Agent { id, .. } => id,
        }
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match self {
            Self::Entity { .. } => NodeKind::Entity,
            Self::Activity { .. } => NodeKind::Activity,
            Self::Agent { .. } => NodeKind::Agent,
        }
    }

    /// Inserts or replaces an attribute.
    pub fn insert_attribute(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Option<FactValue> {
        match self {
            Self::Entity { attributes, .. }
            | Self::Activity { attributes, .. }
            | Self::Agent { attributes, .. } => attributes.insert(key.into(), value.into()),
        }
    }

    /// Ends an activity at a logical time.
    pub fn end(&mut self, ended_at: u64) -> Result<(), ProvenanceError> {
        match self {
            Self::Activity {
                started_at,
                ended_at: end,
                ..
            } => {
                if ended_at < *started_at {
                    return Err(ProvenanceError::ActivityTimeRegression {
                        started_at: *started_at,
                        ended_at,
                    });
                }
                *end = Some(ended_at);
                Ok(())
            }
            _ => Err(ProvenanceError::NotActivity(self.id().clone())),
        }
    }

    /// Returns all attributes in key order.
    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&str, &FactValue)> {
        let attributes = match self {
            Self::Entity { attributes, .. }
            | Self::Activity { attributes, .. }
            | Self::Agent { attributes, .. } => attributes,
        };
        attributes
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    /// Computes canonical node identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "provenance-node-v1")
            .text("id", self.id().as_str())
            .text("kind", self.kind().as_str());
        if let Self::Activity {
            started_at,
            ended_at,
            ..
        } = self
        {
            encoder.u64("started-at", *started_at);
            match ended_at {
                Some(value) => {
                    encoder.boolean("has-ended-at", true).u64("ended-at", *value);
                }
                None => {
                    encoder.boolean("has-ended-at", false);
                }
            }
        }
        let attributes = self.attributes().collect::<Vec<_>>();
        encoder.u64("attribute-count", attributes.len() as u64);
        for (key, value) in attributes {
            encoder.text("attribute-key", key);
            value.encode(&mut encoder, "attribute-type");
        }
        encoder.digest()
    }
}

/// Provenance node category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NodeKind {
    Entity,
    Activity,
    Agent,
}

impl NodeKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Activity => "activity",
            Self::Agent => "agent",
        }
    }
}

/// Typed provenance relation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Relation {
    Used { activity: NodeId, entity: NodeId },
    GeneratedBy { entity: NodeId, activity: NodeId },
    AssociatedWith { activity: NodeId, agent: NodeId },
    AttributedTo { entity: NodeId, agent: NodeId },
    DerivedFrom { entity: NodeId, source: NodeId },
    DelegatedTo { delegate: NodeId, responsible: NodeId },
    InformedBy { activity: NodeId, source: NodeId },
}

impl Relation {
    /// Returns relation source and target in canonical directed order.
    #[must_use]
    pub const fn endpoints(&self) -> (&NodeId, &NodeId) {
        match self {
            Self::Used { activity, entity } => (activity, entity),
            Self::GeneratedBy { entity, activity } => (entity, activity),
            Self::AssociatedWith { activity, agent } => (activity, agent),
            Self::AttributedTo { entity, agent } => (entity, agent),
            Self::DerivedFrom { entity, source } => (entity, source),
            Self::DelegatedTo {
                delegate,
                responsible,
            } => (delegate, responsible),
            Self::InformedBy { activity, source } => (activity, source),
        }
    }

    /// Returns the relation name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Used { .. } => "used",
            Self::GeneratedBy { .. } => "generated-by",
            Self::AssociatedWith { .. } => "associated-with",
            Self::AttributedTo { .. } => "attributed-to",
            Self::DerivedFrom { .. } => "derived-from",
            Self::DelegatedTo { .. } => "delegated-to",
            Self::InformedBy { .. } => "informed-by",
        }
    }

    /// Computes canonical relation identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let (source, target) = self.endpoints();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "provenance-relation-v1")
            .text("kind", self.kind())
            .text("source", source.as_str())
            .text("target", target.as_str());
        encoder.digest()
    }
}

/// Provenance graph construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    EmptyNodeId,
    DuplicateNode(NodeId),
    MissingNode(NodeId),
    NotActivity(NodeId),
    ActivityTimeRegression { started_at: u64, ended_at: u64 },
    RelationType {
        relation: &'static str,
        node: NodeId,
        expected: NodeKind,
        actual: NodeKind,
    },
    DerivationCycle(Vec<NodeId>),
}

impl Display for ProvenanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNodeId => formatter.write_str("provenance node id must not be empty"),
            Self::DuplicateNode(id) => write!(formatter, "duplicate provenance node `{id}`"),
            Self::MissingNode(id) => write!(formatter, "missing provenance node `{id}`"),
            Self::NotActivity(id) => write!(formatter, "provenance node `{id}` is not an activity"),
            Self::ActivityTimeRegression {
                started_at,
                ended_at,
            } => write!(
                formatter,
                "activity ended at {ended_at} before it started at {started_at}"
            ),
            Self::RelationType {
                relation,
                node,
                expected,
                actual,
            } => write!(
                formatter,
                "relation `{relation}` requires `{node}` to be {expected:?}, found {actual:?}"
            ),
            Self::DerivationCycle(path) => {
                formatter.write_str("entity derivation cycle:")?;
                for node in path {
                    write!(formatter, " {node}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ProvenanceError {}

/// Shortest relation witness between two nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenancePath {
    nodes: Vec<NodeId>,
    relations: Vec<Relation>,
    digest: Digest,
}

impl ProvenancePath {
    #[must_use]
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }

    #[must_use]
    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Typed, cycle-checked provenance graph.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProvenanceGraph {
    nodes: BTreeMap<NodeId, ProvenanceNode>,
    relations: BTreeSet<Relation>,
}

impl ProvenanceGraph {
    /// Starts an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node without replacing existing identity.
    pub fn add_node(&mut self, node: ProvenanceNode) -> Result<(), ProvenanceError> {
        if self.nodes.contains_key(node.id()) {
            return Err(ProvenanceError::DuplicateNode(node.id().clone()));
        }
        self.nodes.insert(node.id().clone(), node);
        Ok(())
    }

    /// Returns a node.
    #[must_use]
    pub fn node(&self, id: &NodeId) -> Option<&ProvenanceNode> {
        self.nodes.get(id)
    }

    /// Returns a mutable node for attribute or activity completion updates.
    #[must_use]
    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut ProvenanceNode> {
        self.nodes.get_mut(id)
    }

    /// Adds a relation after validating endpoint categories and derivation acyclicity.
    pub fn add_relation(&mut self, relation: Relation) -> Result<bool, ProvenanceError> {
        self.validate_relation(&relation)?;
        if self.relations.contains(&relation) {
            return Ok(false);
        }
        self.relations.insert(relation.clone());
        if let Err(error) = self.validate_derivation_cycles() {
            self.relations.remove(&relation);
            return Err(error);
        }
        Ok(true)
    }

    /// Returns all nodes in identity order.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &ProvenanceNode> {
        self.nodes.values()
    }

    /// Returns all relations in canonical order.
    pub fn relations(&self) -> impl ExactSizeIterator<Item = &Relation> {
        self.relations.iter()
    }

    /// Returns direct outgoing relations from a node.
    pub fn outgoing<'graph>(
        &'graph self,
        id: &'graph NodeId,
    ) -> impl Iterator<Item = &'graph Relation> + 'graph {
        self.relations
            .iter()
            .filter(move |relation| relation.endpoints().0 == id)
    }

    /// Returns direct incoming relations to a node.
    pub fn incoming<'graph>(
        &'graph self,
        id: &'graph NodeId,
    ) -> impl Iterator<Item = &'graph Relation> + 'graph {
        self.relations
            .iter()
            .filter(move |relation| relation.endpoints().1 == id)
    }

    /// Computes upstream entity lineage through direct derivation and activity usage.
    pub fn lineage(&self, entity: &NodeId) -> Result<BTreeSet<NodeId>, ProvenanceError> {
        self.require_kind(entity, NodeKind::Entity, "lineage")?;
        let mut found = BTreeSet::new();
        let mut queue = VecDeque::from([entity.clone()]);
        while let Some(current) = queue.pop_front() {
            for relation in &self.relations {
                match relation {
                    Relation::DerivedFrom {
                        entity: derived,
                        source,
                    } if derived == &current => {
                        if found.insert(source.clone()) {
                            queue.push_back(source.clone());
                        }
                    }
                    Relation::GeneratedBy {
                        entity: generated,
                        activity,
                    } if generated == &current => {
                        for used in &self.relations {
                            if let Relation::Used {
                                activity: used_activity,
                                entity: input,
                            } = used
                            {
                                if used_activity == activity && found.insert(input.clone()) {
                                    queue.push_back(input.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        found.remove(entity);
        Ok(found)
    }

    /// Computes all downstream entities affected by an entity.
    pub fn impact(&self, entity: &NodeId) -> Result<BTreeSet<NodeId>, ProvenanceError> {
        self.require_kind(entity, NodeKind::Entity, "impact")?;
        let mut found = BTreeSet::new();
        let mut queue = VecDeque::from([entity.clone()]);
        while let Some(current) = queue.pop_front() {
            for relation in &self.relations {
                match relation {
                    Relation::DerivedFrom {
                        entity: derived,
                        source,
                    } if source == &current => {
                        if found.insert(derived.clone()) {
                            queue.push_back(derived.clone());
                        }
                    }
                    Relation::Used {
                        activity,
                        entity: input,
                    } if input == &current => {
                        for generated in &self.relations {
                            if let Relation::GeneratedBy {
                                entity: output,
                                activity: generating_activity,
                            } = generated
                            {
                                if generating_activity == activity && found.insert(output.clone()) {
                                    queue.push_back(output.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        found.remove(entity);
        Ok(found)
    }

    /// Finds the shortest directed relation witness from `source` to `target`.
    pub fn shortest_path(
        &self,
        source: &NodeId,
        target: &NodeId,
    ) -> Result<Option<ProvenancePath>, ProvenanceError> {
        if !self.nodes.contains_key(source) {
            return Err(ProvenanceError::MissingNode(source.clone()));
        }
        if !self.nodes.contains_key(target) {
            return Err(ProvenanceError::MissingNode(target.clone()));
        }
        let mut queue = VecDeque::from([source.clone()]);
        let mut seen = BTreeSet::from([source.clone()]);
        let mut parent = BTreeMap::<NodeId, (NodeId, Relation)>::new();
        while let Some(current) = queue.pop_front() {
            if &current == target {
                let mut nodes = vec![current.clone()];
                let mut relations = Vec::new();
                let mut cursor = current;
                while let Some((previous, relation)) = parent.get(&cursor).cloned() {
                    nodes.push(previous.clone());
                    relations.push(relation);
                    cursor = previous;
                }
                nodes.reverse();
                relations.reverse();
                let mut encoder = CanonicalEncoder::new();
                encoder
                    .text("type", "provenance-path-v1")
                    .u64("node-count", nodes.len() as u64);
                for node in &nodes {
                    encoder.text("node", node.as_str());
                }
                encoder.u64("relation-count", relations.len() as u64);
                for relation in &relations {
                    encoder.field("relation", &relation.digest().0);
                }
                return Ok(Some(ProvenancePath {
                    nodes,
                    relations,
                    digest: encoder.digest(),
                }));
            }
            let mut outgoing = self.outgoing(&current).cloned().collect::<Vec<_>>();
            outgoing.sort();
            for relation in outgoing {
                let next = relation.endpoints().1.clone();
                if seen.insert(next.clone()) {
                    parent.insert(next.clone(), (current.clone(), relation));
                    queue.push_back(next);
                }
            }
        }
        Ok(None)
    }

    /// Validates every relation and all derivation cycles.
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        for relation in &self.relations {
            self.validate_relation(relation)?;
        }
        self.validate_derivation_cycles()
    }

    /// Computes canonical graph identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "provenance-graph-v1")
            .u64("node-count", self.nodes.len() as u64);
        for node in self.nodes.values() {
            encoder
                .text("node", node.id().as_str())
                .field("node-digest", &node.digest().0);
        }
        encoder.u64("relation-count", self.relations.len() as u64);
        for relation in &self.relations {
            encoder.field("relation", &relation.digest().0);
        }
        encoder.digest()
    }

    fn node_kind(&self, id: &NodeId) -> Result<NodeKind, ProvenanceError> {
        self.nodes
            .get(id)
            .map(ProvenanceNode::kind)
            .ok_or_else(|| ProvenanceError::MissingNode(id.clone()))
    }

    fn require_kind(
        &self,
        id: &NodeId,
        expected: NodeKind,
        relation: &'static str,
    ) -> Result<(), ProvenanceError> {
        let actual = self.node_kind(id)?;
        if actual == expected {
            Ok(())
        } else {
            Err(ProvenanceError::RelationType {
                relation,
                node: id.clone(),
                expected,
                actual,
            })
        }
    }

    fn validate_relation(&self, relation: &Relation) -> Result<(), ProvenanceError> {
        match relation {
            Relation::Used { activity, entity } => {
                self.require_kind(activity, NodeKind::Activity, relation.kind())?;
                self.require_kind(entity, NodeKind::Entity, relation.kind())
            }
            Relation::GeneratedBy { entity, activity } => {
                self.require_kind(entity, NodeKind::Entity, relation.kind())?;
                self.require_kind(activity, NodeKind::Activity, relation.kind())
            }
            Relation::AssociatedWith { activity, agent } => {
                self.require_kind(activity, NodeKind::Activity, relation.kind())?;
                self.require_kind(agent, NodeKind::Agent, relation.kind())
            }
            Relation::AttributedTo { entity, agent } => {
                self.require_kind(entity, NodeKind::Entity, relation.kind())?;
                self.require_kind(agent, NodeKind::Agent, relation.kind())
            }
            Relation::DerivedFrom { entity, source } => {
                self.require_kind(entity, NodeKind::Entity, relation.kind())?;
                self.require_kind(source, NodeKind::Entity, relation.kind())
            }
            Relation::DelegatedTo {
                delegate,
                responsible,
            } => {
                self.require_kind(delegate, NodeKind::Agent, relation.kind())?;
                self.require_kind(responsible, NodeKind::Agent, relation.kind())
            }
            Relation::InformedBy { activity, source } => {
                self.require_kind(activity, NodeKind::Activity, relation.kind())?;
                self.require_kind(source, NodeKind::Activity, relation.kind())
            }
        }
    }

    fn validate_derivation_cycles(&self) -> Result<(), ProvenanceError> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Mark {
            Visiting,
            Complete,
        }

        fn visit(
            graph: &ProvenanceGraph,
            id: &NodeId,
            marks: &mut BTreeMap<NodeId, Mark>,
            stack: &mut Vec<NodeId>,
        ) -> Result<(), ProvenanceError> {
            match marks.get(id) {
                Some(Mark::Complete) => return Ok(()),
                Some(Mark::Visiting) => {
                    let start = stack.iter().position(|node| node == id).unwrap_or(0);
                    let mut cycle = stack[start..].to_vec();
                    cycle.push(id.clone());
                    return Err(ProvenanceError::DerivationCycle(cycle));
                }
                None => {}
            }
            marks.insert(id.clone(), Mark::Visiting);
            stack.push(id.clone());
            for relation in &graph.relations {
                if let Relation::DerivedFrom { entity, source } = relation {
                    if entity == id {
                        visit(graph, source, marks, stack)?;
                    }
                }
            }
            stack.pop();
            marks.insert(id.clone(), Mark::Complete);
            Ok(())
        }

        let mut marks = BTreeMap::new();
        let mut stack = Vec::new();
        for node in self
            .nodes
            .values()
            .filter(|node| node.kind() == NodeKind::Entity)
        {
            visit(self, node.id(), &mut marks, &mut stack)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeId, ProvenanceError, ProvenanceGraph, ProvenanceNode, Relation};

    fn id(value: &str) -> NodeId {
        NodeId::new(value).unwrap()
    }

    fn graph() -> ProvenanceGraph {
        let mut graph = ProvenanceGraph::new();
        graph.add_node(ProvenanceNode::entity(id("raw"))).unwrap();
        graph
            .add_node(ProvenanceNode::activity(id("normalize"), 1))
            .unwrap();
        graph
            .add_node(ProvenanceNode::entity(id("normalized")))
            .unwrap();
        graph.add_node(ProvenanceNode::agent(id("worker"))).unwrap();
        graph
            .add_relation(Relation::Used {
                activity: id("normalize"),
                entity: id("raw"),
            })
            .unwrap();
        graph
            .add_relation(Relation::GeneratedBy {
                entity: id("normalized"),
                activity: id("normalize"),
            })
            .unwrap();
        graph
            .add_relation(Relation::AssociatedWith {
                activity: id("normalize"),
                agent: id("worker"),
            })
            .unwrap();
        graph
    }

    #[test]
    fn lineage_and_impact_cross_activity_boundaries() {
        let graph = graph();
        assert!(graph.lineage(&id("normalized")).unwrap().contains(&id("raw")));
        assert!(graph.impact(&id("raw")).unwrap().contains(&id("normalized")));
    }

    #[test]
    fn shortest_path_returns_relation_witness() {
        let graph = graph();
        let path = graph
            .shortest_path(&id("normalized"), &id("normalize"))
            .unwrap()
            .unwrap();
        assert_eq!(path.nodes().len(), 2);
        assert_eq!(path.relations().len(), 1);
    }

    #[test]
    fn derivation_cycle_is_refused_and_rolled_back() {
        let mut graph = graph();
        graph
            .add_relation(Relation::DerivedFrom {
                entity: id("normalized"),
                source: id("raw"),
            })
            .unwrap();
        let error = graph
            .add_relation(Relation::DerivedFrom {
                entity: id("raw"),
                source: id("normalized"),
            })
            .unwrap_err();
        assert!(matches!(error, ProvenanceError::DerivationCycle(_)));
        assert_eq!(
            graph
                .relations()
                .filter(|relation| matches!(relation, Relation::DerivedFrom { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn relation_endpoint_types_are_enforced() {
        let mut graph = graph();
        assert!(matches!(
            graph.add_relation(Relation::Used {
                activity: id("worker"),
                entity: id("raw")
            }),
            Err(ProvenanceError::RelationType { .. })
        ));
    }
}
