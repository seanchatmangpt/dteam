//! MCP Result Projection: Bounded transformation of MCP responses into COG8 constructs.

use crate::construct8::{Construct8, Triple};
use thiserror::Error;

/// Error returned when an MCP result projection exceeds the 8-triple limit.
#[derive(Debug, Error)]
pub enum OverflowError {
    /// Result contains {0} triples, exceeding the maximum of 8.
    #[error("MCP result overflow: {0} triples exceeds 8-triple limit")]
    LimitExceeded(usize),
}

/// Result from an MCP operation.
///
/// Contains the raw triples harvested from an MCP server response.
#[derive(Debug, Clone)]
pub struct MCPResult {
    /// Raw triples produced by the MCP operation.
    pub triples: Vec<Triple>,
}

/// Projector that transforms an [`MCPResult`] into a bounded [`Construct8`] delta.
pub trait ResultProjector {
    /// Projects the MCP result into a [`Construct8`].
    ///
    /// # Errors
    ///
    /// Returns [`OverflowError`] if the result violates the 8-triple limit.
    fn project(&self, result: &MCPResult) -> Result<Construct8, OverflowError>;
}

/// Default projector that strictly enforces the 8-triple limit.
///
/// Rejects any result containing more than 8 triples to maintain
/// deterministic bounded execution.
pub struct DefaultProjector;

impl ResultProjector for DefaultProjector {
    fn project(&self, result: &MCPResult) -> Result<Construct8, OverflowError> {
        let count = result.triples.len();
        if count > 8 {
            return Err(OverflowError::LimitExceeded(count));
        }

        let mut construct = Construct8::empty();
        for triple in &result.triples {
            if !construct.push(*triple) {
                // Defensive: should be unreachable due to check above.
                return Err(OverflowError::LimitExceeded(count));
            }
        }

        Ok(construct)
    }
}
