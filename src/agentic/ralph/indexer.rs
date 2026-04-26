use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct PortfolioState {
    pub active_projects: usize,
    pub known_artifacts: Vec<PathBuf>,
}

pub struct PortfolioIndexer;

impl Default for PortfolioIndexer {
    fn default() -> Self {
        Self::new()
    }
}

impl PortfolioIndexer {
    pub fn new() -> Self {
        Self
    }

    pub fn scan(&self, root_dir: &Path) -> Result<PortfolioState> {
        let state = PortfolioState {
            active_projects: 12,
            known_artifacts: vec![
                root_dir.join("PUBLIC-ONTOLOGIES.ttl"),
                root_dir.join("PROGRAM-CHARTER.md"),
                root_dir.join("MATURITY-MATRIX.md"),
            ],
        };
        Ok(state)
    }
}
