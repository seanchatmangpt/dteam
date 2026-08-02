#![forbid(unsafe_code)]
//! High-density deterministic capability kernel for dteam.
//!
//! The crate is deliberately standalone: it provides a runnable nucleus while the
//! larger workspace's sibling repositories are unavailable. Its execution path is:
//!
//! `observe → route → admit/refuse → construct → authorize → actuate → receipt → replay`.

pub mod broker;
pub mod graph;
pub mod hash;
pub mod ledger;
pub mod model;
pub mod policy;
pub mod process;
pub mod runtime;

pub use broker::{
    ActuationEvidence, AuthorizationLedger, AuthorizationReceipt, BatchEvidence, BatchMode, Broker,
    BrokerError, BrokerVerification, Executor, PreflightRefusal,
};
pub use graph::{Capability, CapabilityGraph, CapabilityPlan, GraphError};
pub use hash::{sha256, CanonicalEncoder, Digest};
pub use ledger::{
    LedgerError, Receipt, ReceiptKind, ReceiptLedger, ReceiptQuery, ReplayReport,
};
pub use model::{
    AdmittedObservation, AuthorityId, CapabilityId, FactValue, Intent, ModelError, Observation,
    OperationId, Outcome, PolicyId, Standing, SubjectId,
};
pub use policy::{
    AdmissionDecision, AdmissionPolicy, Predicate, Rule, Violation,
};
pub use process::{
    discover_transition_system, Activity, ConformanceReport, ConformanceViolation, EventId,
    EventRecord, ObjectEventLog, ObjectId, ObjectRecord, ObjectType, ProcessError, ProcessMetrics,
    TransitionSystem,
};
pub use runtime::{
    ProcessError as RuntimeError, ProcessResult, ProcessTrace, Route, Router, Runtime, TraceError,
    TraceEvent, TraceStage,
};

/// Common imports for applications embedding the capability kernel.
pub mod prelude {
    pub use crate::{
        discover_transition_system, Activity, AdmissionPolicy, AuthorityId, BatchMode, Broker,
        Capability, CapabilityGraph, CapabilityId, EventId, EventRecord, Executor, FactValue,
        Intent, ObjectEventLog, ObjectId, ObjectRecord, ObjectType, Observation, OperationId,
        Outcome, PolicyId, Predicate, PreflightRefusal, ReceiptQuery, Route, Router, Rule, Runtime,
        SubjectId, TransitionSystem,
    };
}
