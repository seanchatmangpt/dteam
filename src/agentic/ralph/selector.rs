use crate::agentic::ralph::indexer::PortfolioState;
use anyhow::Result;

pub struct WorkSelector;

impl Default for WorkSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkSelector {
    pub fn new() -> Self {
        Self
    }

    pub fn select_next(&self, _state: &PortfolioState) -> Result<String> {
        Ok("Generate the ChatmanGPT portfolio production topology.".to_string())
    }
}
