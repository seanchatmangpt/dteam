use crate::agentic::ralph::indexer::PortfolioState;
use anyhow::Result;

pub struct MaturityScorer;

impl Default for MaturityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl MaturityScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, state: &PortfolioState) -> Result<u8> {
        let level = if state.active_projects > 10 { 3 } else { 1 };
        Ok(level)
    }
}
