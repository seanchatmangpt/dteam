pub mod controller;
pub mod execution;
pub mod indexer;
pub mod ontology;
pub mod patterns;
pub mod phase;
pub mod receipt;
pub mod router;
pub mod scorer;
pub mod selector;
pub mod verifier;
pub mod workspace;

pub use controller::AutonomicController;
pub use execution::ExecutionEngine;
pub use indexer::{PortfolioIndexer, PortfolioState};
pub use ontology::OntologyClosureEngine;
pub use phase::{
    AgentKind, PhaseReceipt, PhaseRunner, RalphMode, SpecKitInvocation, SpecKitPhase,
    SpecKitRunner, SpecifyPhaseRunner, SpeckitController,
};
pub use receipt::ReceiptEmitter;
pub use router::{AgentRouter, KeywordRouter};
pub use scorer::MaturityScorer;
pub use selector::WorkSelector;
pub use verifier::{CargoVerifier, DoDVerifier};
pub use workspace::{GitWorktreeManager, WorkspaceManager};
