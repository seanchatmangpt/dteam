pub mod petri_net;
use crate::utils::dense_kernel::{DenseIndex, KBitSet, NodeKind};
/// Data structures derived from `rust4pm` (MIT/Apache-2.0).
/// See ATTRIBUTION.md for details.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

pub type Attributes = Vec<Attribute>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Event {
    pub attributes: Attributes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Trace {
    pub id: String,
    pub events: Vec<Event>,
    pub attributes: Attributes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EventLog {
    pub traces: Vec<Trace>,
    pub attributes: Attributes,
}

/// A formal ontology defining authorized activities for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ontology {
    pub bitset: KBitSet<16>, // K1024 capacity for universal alignment
    pub index: DenseIndex,
}

impl Ontology {
    pub fn new(activities: Vec<String>) -> Self {
        let mut symbols = Vec::with_capacity(activities.len());
        for act in activities {
            symbols.push((act, NodeKind::Transition));
        }
        let index = DenseIndex::compile(symbols).expect("Failed to compile ontology index");
        let mut bitset = KBitSet::<16>::zero();
        for i in 0..index.len() {
            bitset.set(i).unwrap();
        }
        Self { bitset, index }
    }

    pub fn contains(&self, activity: &str) -> bool {
        self.index
            .dense_id(activity)
            .is_some_and(|id| self.bitset.contains(id as usize))
    }

    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for w in &self.bitset.words {
            h ^= *w;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

impl Event {
    pub fn new(activity: String) -> Self {
        Self {
            attributes: vec![Attribute {
                key: "concept:name".to_string(),
                value: AttributeValue::String(activity),
            }],
        }
    }
}

impl Trace {
    pub fn new(id: String) -> Self {
        Self {
            id,
            events: Vec::new(),
            attributes: Attributes::new(),
        }
    }
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trace(&mut self, trace: Trace) {
        self.traces.push(trace);
    }

    /// Pre-pass sizing: returns the number of distinct activities in the log.
    pub fn activity_footprint(&self) -> usize {
        let mut activities = std::collections::HashSet::new();
        for trace in &self.traces {
            for event in &trace.events {
                if let Some(attr) = event.attributes.iter().find(|a| a.key == "concept:name") {
                    if let AttributeValue::String(s) = &attr.value {
                        activities.insert(s);
                    }
                }
            }
        }
        activities.len()
    }

    pub fn canonical_hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for trace in &self.traces {
            for event in &trace.events {
                if let Some(attr) = event.attributes.iter().find(|a| a.key == "concept:name") {
                    if let AttributeValue::String(s) = &attr.value {
                        for b in s.as_bytes() {
                            h ^= *b as u64;
                            h = h.wrapping_mul(0x100000001b3);
                        }
                    }
                }
            }
        }
        h
    }
}
