//! POWL64 proof/data ABI (PRD v0.4, v0.8).
//!
//! Records the nonlinear route proof through COG8 graphs, capturing collapse
//! attribution, path traversal, and cryptographic chain heads.

use serde::{Deserialize, Serialize};

use crate::runtime::cog8::{AgentId, CollapseFn, EdgeId, EdgeKind, HumanRoleId, NodeId, ToolId};

/// Path polarity for a route segment.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Polarity {
    /// Action admitted and performed.
    #[default]
    Positive = 1,
    /// Action blocked or denied.
    Negative = 2,
    /// Informational or intermediate node.
    Neutral = 3,
    /// Explicitly blocked path.
    Blocking = 4,
    /// High-priority override.
    Override = 5,
}

/// Target for the cognitive projection.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum ProjectionTarget {
    /// Model Context Protocol tool call.
    Mcp = 1,
    /// Agent-to-Agent protocol.
    A2a = 2,
    /// Human-in-the-loop validation.
    Hitl = 3,
    /// No projection performed.
    #[default]
    NoOp = 4,
}

/// Collaborative partner identifier.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PartnerId {
    /// Discriminant for the partner type (0=None, 1=Tool, 2=Agent, 3=Human).
    pub tag: u8,
    /// Numeric identifier.
    pub id: u16,
}

impl PartnerId {
    /// No partner.
    pub const NONE: Self = Self { tag: 0, id: 0 };
    /// MCP tool partner.
    pub fn tool(id: ToolId) -> Self {
        Self { tag: 1, id: id.0 }
    }
    /// Autonomous agent partner.
    pub fn agent(id: AgentId) -> Self {
        Self { tag: 2, id: id.0 }
    }
    /// Human role partner.
    pub fn human(id: HumanRoleId) -> Self {
        Self { tag: 3, id: id.0 }
    }
}

/// A single cell in the POWL64 route proof.
///
/// Records the traversal of an edge and the collapse function that admitted it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Powl64RouteCell {
    /// Stable identifier for the COG8 graph.
    pub graph_id: u32,
    /// Source node index.
    pub from_node: NodeId,
    /// Target node index.
    pub to_node: NodeId,
    /// Edge identifier.
    pub edge_id: EdgeId,
    /// Kind of topological relation traversed.
    pub edge_kind: EdgeKind,
    /// Cognitive function attributed to this segment.
    pub collapse_fn: CollapseFn,
    /// Logical polarity of the traversal.
    pub polarity: Polarity,

    /// Projection target.
    pub projection_target: ProjectionTarget,
    /// Collaborative partner identifier.
    pub partner_id: PartnerId,

    /// BLAKE3 digest of the input field snapshot at this point.
    pub input_digest: u64,
    /// BLAKE3 digest of external call arguments (0 if none).
    pub args_digest: u64,
    /// BLAKE3 digest of external call result (0 if none).
    pub result_digest: u64,

    /// Previous chain head hash (truncated to u64 for ABI).
    pub prior_chain: u64,
    /// New chain head hash after this extension.
    pub chain_head: u64,
}

/// POWL64 route proof accumulator.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Powl64 {
    /// Sequence of route cells.
    pub cells: Vec<Powl64RouteCell>,
}

impl Powl64 {
    /// Create an empty route proof.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of cells in the route proof.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Alias for `cell_count()` for backward compatibility with the test suite.
    pub fn chain_len(&self) -> usize {
        self.cell_count()
    }

    /// Extend the route proof with a new segment.
    pub fn extend(&mut self, cell: Powl64RouteCell) {
        self.cells.push(cell);
    }

    /// Return the current chain head.
    pub fn chain_head(&self) -> Option<u64> {
        self.cells.last().map(|c| c.chain_head)
    }

    /// Return the sequence of chain head hashes.
    pub fn path(&self) -> Vec<blake3::Hash> {
        self.cells
            .iter()
            .map(|c| {
                let mut bytes = [0u8; 32];
                bytes[..8].copy_from_slice(&c.chain_head.to_le_bytes());
                blake3::Hash::from_bytes(bytes)
            })
            .collect()
    }

    /// Compare two route proofs by cell count only (v0).
    pub fn shape_match_v0_cell_count(&self, other: &Self) -> Result<(), &'static str> {
        if self.cell_count() == other.cell_count() {
            Ok(())
        } else {
            Err("cell count mismatch. Hint: the two POWL64 proofs represent different execution paths or lengths.")
        }
    }

    /// Compare two route proofs by full path hashes (v1).
    pub fn shape_match_v1_path(&self, other: &Self) -> Result<(), &'static str> {
        if self.path() == other.path() {
            Ok(())
        } else {
            Err("path hash mismatch. Hint: the two POWL64 proofs have identical lengths but different cryptographic state transitions.")
        }
    }
}
