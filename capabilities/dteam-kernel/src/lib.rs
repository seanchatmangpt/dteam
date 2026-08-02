#![forbid(unsafe_code)]
//! High-density deterministic capability kernel for dteam.
//!
//! The crate is deliberately standalone: it provides a runnable nucleus while the
//! larger workspace's sibling repositories are unavailable. Its execution path is:
//!
//! `observe → validate → route → admit/refuse → decide → hook → plan → reserve → transition → transact → authorize → actuate → provenance → receipt → replay`.

pub mod broker;
pub mod decision;
pub mod graph;
pub mod hash;
pub mod hook;
pub mod ledger;
pub mod model;
pub mod policy;
pub mod process;
pub mod provenance;
pub mod quota;
pub mod runtime;
pub mod scheduler;
pub mod schema;
pub mod state_machine;
pub mod store;

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
pub use hook::{
    Hook, HookError, HookEvaluation, HookEvent, HookLint, HookRegistry, HookReport, IntentTemplate,
    PayloadTemplate,
};
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
pub use provenance::{
    NodeId, NodeKind, ProvenanceError, ProvenanceGraph, ProvenanceNode, ProvenancePath, Relation,
};
pub use quota::{
    PrincipalId, QuotaAction, QuotaClaim, QuotaError, QuotaManager, QuotaPolicy, QuotaReceipt,
    QuotaVerification, Reservation, ReservationId, ReservationRequest, ReservationState, ResourceId,
};
pub use runtime::{
    ProcessError as RuntimeError, ProcessResult, ProcessTrace, Route, Router, Runtime, TraceError,
    TraceEvent, TraceStage,
};
pub use scheduler::{
    execute_schedule, ScheduleError, ScheduleExecution, SchedulePlan, Task, TaskExecutor, TaskGraph,
    TaskId, TaskOutcome, TaskReceipt,
};
pub use schema::{
    CompatibilityChange, CompatibilityReport, Constraint, Document, DocumentSchema, FieldSchema,
    MigrationPlan, MigrationResult, MigrationStep, SchemaError, UnknownFieldPolicy, ValidationIssue,
    ValidationReport, ValueType,
};
pub use state_machine::{
    ApplyResult, DispatchDecision, EventKind, Guard, GuardFailure, InstanceVerificationError,
    MachineAnalysis, MachineError, MachineFinding, MachineInstance, StateId, StateMachine,
    StateReceipt, Transition, TransitionEvaluation, TransitionId,
};
pub use store::{
    Change, CommitReceipt, ExpectedVersion, Mutation, Record, RecordKey, StoreError, StoreSnapshot,
    StoreVerification, Transaction, TransactionalStore,
};

/// Common imports for applications embedding the capability kernel.
pub mod prelude {
    pub use crate::{
        discover_transition_system, execute_schedule, Activity, AdmissionPolicy, ApplyResult,
        AuthorityId, BatchMode, Broker, Capability, CapabilityGraph, CapabilityId, Condition,
        Constraint, DecisionEffect, DecisionOutcome, DecisionRule, DecisionTable, Document,
        DocumentSchema, EventId, EventKind, EventRecord, Executor, ExpectedVersion, FactValue,
        FieldSchema, Guard, Hook, HookEvent, HookRegistry, Intent, IntentTemplate, MachineInstance,
        MigrationPlan, MigrationStep, Mutation, NodeId, ObjectEventLog, ObjectId, ObjectRecord,
        ObjectType, Observation, OperationId, Outcome, PayloadTemplate, PolicyId, Predicate,
        PreflightRefusal, PrincipalId, ProvenanceGraph, ProvenanceNode, QuotaClaim, QuotaManager,
        QuotaPolicy, ReceiptQuery, RecordKey, Relation, ReservationId, ReservationRequest, ResourceId,
        Route, Router, Rule, Runtime, StateId, StateMachine, SubjectId, Task, TaskExecutor, TaskGraph,
        TaskId, TaskOutcome, Transaction, TransactionalStore, Transition, TransitionId,
        TransitionSystem, UnknownFieldPolicy, ValueType,
    };
}
