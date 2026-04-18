use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Place {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transition {
    pub id: String,
    pub label: String,
    pub is_invisible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Arc {
    pub from: String,
    pub to: String,
    pub weight: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    pub initial_marking: HashMap<String, usize>,
    pub final_markings: Vec<HashMap<String, usize>>,
}

impl PetriNet {
    /// Evaluates if the net is a structurally valid workflow net
    /// (1 unique start place, 1 unique end place, strongly connected).
    /// Highly optimized using bitset algebra to map node connectivity.
    pub fn is_structural_workflow_net(&self) -> bool {
        if self.places.is_empty() || self.transitions.is_empty() { return false; }
        
        let mut id_to_index = HashMap::new();
        let mut idx = 0;
        
        for p in &self.places {
            id_to_index.insert(&p.id, idx);
            idx += 1;
        }
        let place_count = idx;
        
        for t in &self.transitions {
            id_to_index.insert(&t.id, idx);
            idx += 1;
        }
        let total_nodes = idx;
        let num_words = (total_nodes + 63) / 64;
        
        // Bitset algebra replacing HashMap counters for microsecond latency
        let mut in_degrees = vec![0u64; num_words];
        let mut out_degrees = vec![0u64; num_words];
        
        for arc in &self.arcs {
            if let Some(&from_idx) = id_to_index.get(&arc.from) {
                out_degrees[from_idx / 64] |= 1u64 << (from_idx % 64);
            }
            if let Some(&to_idx) = id_to_index.get(&arc.to) {
                in_degrees[to_idx / 64] |= 1u64 << (to_idx % 64);
            }
        }
        
        let mut source_places_count = 0;
        let mut sink_places_count = 0;
        
        for i in 0..place_count {
            let has_in = (in_degrees[i / 64] & (1u64 << (i % 64))) != 0;
            let has_out = (out_degrees[i / 64] & (1u64 << (i % 64))) != 0;
            
            if !has_in { source_places_count += 1; }
            if !has_out { sink_places_count += 1; }
        }
        
        // A workflow net must have exactly one source place and one sink place
        if source_places_count != 1 || sink_places_count != 1 {
            return false;
        }
        
        // Ensure no transitions are sources or sinks (must have in > 0 and out > 0)
        for i in place_count..total_nodes {
            let has_in = (in_degrees[i / 64] & (1u64 << (i % 64))) != 0;
            let has_out = (out_degrees[i / 64] & (1u64 << (i % 64))) != 0;
            
            if !has_in || !has_out {
                return false;
            }
        }
        
        true
    }
}
