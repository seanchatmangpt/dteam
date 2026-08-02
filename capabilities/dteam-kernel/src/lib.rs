#![forbid(unsafe_code)]
//! High-density deterministic capability kernel for dteam.
//!
//! The crate is deliberately standalone: it provides a runnable nucleus while the
//! larger workspace's sibling repositories are unavailable. Its execution path is:
//!
//! `observe → route → admit/refuse → decide → plan → transition → authorize → actuate → receipt → replay`.

pub mod broker;
pub mod decision;
pub mod graph;
pub mod hash;
pub mod ledger;
pub mod model;
pub mod policy;
pub mod process;
pub mod runtime;
pub mod scheduler;
pub mod state_machine;

pub use broker::{
    ActuationEvidence, AuthorizationLedger, AuthorizationReceipt, BatchEvidence, BatchMode, Broker,
    BrokerError, BrokerVerification, Executor, PreflightRefusal,
};
pub use decision::{
    Condition, DecisionEffect, DecisionLint, DecisionOutcome, DecisionRule, DecisionTable,
    RuleEvaluation,
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
pub use policy::{AdmissionDecision, AdmissionPolicy, Predicate, Rule, Violation};
pub use process::{
    discover_transition_system, Activity, ConformanceReport, ConformanceViolation, EventId,
    EventRecord, ObjectEventLog, ObjectId, ObjectRecord, ObjectType, ProcessError, ProcessMetrics,
    TransitionSystem,
};
pub use runtime::{
    ProcessError as RuntimeError, ProcessResult, ProcessTrace, Route, Router, Runtime, TraceError,
    TraceEvent, TraceStage,
};
pub use scheduler::{
    execute_schedule, ScheduleError, ScheduleExecution, SchedulePlan, Task, TaskExecutor, TaskGraph,
    TaskId, TaskOutcome, TaskReceipt,
};
pub use state_machine::{
    ApplyResult, DispatchDecision, EventKind, Guard, GuardFailure, InstanceVerificationError,
    MachineAnalysis, MachineError, MachineFinding, MachineInstance, StateId, StateMachine,
    StateReceipt, Transition, TransitionEvaluation, TransitionId,
};

/// Common imports for applications embedding the capability kernel.
pub mod prelude {
    pub use crate::{
        discover_transition_system, execute_schedule, Activity, AdmissionPolicy, ApplyResult,
        AuthorityId, BatchMode, Broker, Capability, CapabilityGraph, CapabilityId, Condition,
        DecisionEffect, DecisionOutcome, DecisionRule, DecisionTable, EventId, EventKind,
        EventRecord, Executor, FactValue, Guard, Intent, MachineInstance, ObjectEventLog, ObjectId,
        ObjectRecord, ObjectType, Observation, OperationId, Outcome, PolicyId, Predicate,
        PreflightRefusal, ReceiptQuery, Route, Router, Rule, Runtime, StateId, StateMachine,
        SubjectId, Task, TaskExecutor, TaskGraph, TaskId, TaskOutcome, Transition, TransitionId,
        TransitionSystem,
    };
}
