//! Deterministic dependency scheduler, critical-path analysis, retries, and execution evidence.

use crate::hash::{CanonicalEncoder, Digest};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// Stable task identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(value: impl Into<String>) -> Result<Self, ScheduleError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ScheduleError::EmptyTaskId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for TaskId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Dependency-bound unit of scheduled capability work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Task {
    id: TaskId,
    dependencies: BTreeSet<TaskId>,
    cost_units: u64,
    priority: i32,
    max_attempts: u32,
    reversible: bool,
}

impl Task {
    #[must_use]
    pub fn new(id: TaskId) -> Self {
        Self {
            id,
            dependencies: BTreeSet::new(),
            cost_units: 1,
            priority: 0,
            max_attempts: 1,
            reversible: true,
        }
    }

    #[must_use]
    pub fn depends_on(mut self, dependency: TaskId) -> Self {
        self.dependencies.insert(dependency);
        self
    }

    #[must_use]
    pub const fn cost_units(mut self, value: u64) -> Self {
        self.cost_units = value;
        self
    }

    #[must_use]
    pub const fn priority(mut self, value: i32) -> Self {
        self.priority = value;
        self
    }

    #[must_use]
    pub const fn max_attempts(mut self, value: u32) -> Self {
        self.max_attempts = value.max(1);
        self
    }

    #[must_use]
    pub const fn reversible(mut self, value: bool) -> Self {
        self.reversible = value;
        self
    }

    #[must_use]
    pub const fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = &TaskId> {
        self.dependencies.iter()
    }

    #[must_use]
    pub const fn cost(&self) -> u64 {
        self.cost_units
    }

    #[must_use]
    pub const fn priority_value(&self) -> i32 {
        self.priority
    }

    #[must_use]
    pub const fn attempts(&self) -> u32 {
        self.max_attempts
    }

    #[must_use]
    pub const fn is_reversible(&self) -> bool {
        self.reversible
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "task-v1")
            .text("id", self.id.as_str())
            .u64("cost", self.cost_units)
            .i64("priority", i64::from(self.priority))
            .u64("max-attempts", u64::from(self.max_attempts))
            .boolean("reversible", self.reversible)
            .u64("dependency-count", self.dependencies.len() as u64);
        for dependency in &self.dependencies {
            encoder.text("dependency", dependency.as_str());
        }
        encoder.digest()
    }
}

/// Static scheduling failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    EmptyTaskId,
    DuplicateTask(TaskId),
    MissingDependency { task: TaskId, dependency: TaskId },
    Cycle(Vec<TaskId>),
    ZeroCapacity,
    TaskExceedsCapacity {
        task: TaskId,
        cost: u64,
        capacity: u64,
    },
    CostOverflow,
}

impl Display for ScheduleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTaskId => formatter.write_str("task id must not be empty"),
            Self::DuplicateTask(id) => write!(formatter, "duplicate task `{id}`"),
            Self::MissingDependency { task, dependency } => {
                write!(formatter, "task `{task}` depends on missing `{dependency}`")
            }
            Self::Cycle(path) => {
                formatter.write_str("task cycle:")?;
                for task in path {
                    write!(formatter, " {task}")?;
                }
                Ok(())
            }
            Self::ZeroCapacity => formatter.write_str("scheduler capacity must be positive"),
            Self::TaskExceedsCapacity {
                task,
                cost,
                capacity,
            } => write!(
                formatter,
                "task `{task}` cost {cost} exceeds wave capacity {capacity}"
            ),
            Self::CostOverflow => formatter.write_str("task cost overflow"),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// Canonical graph of schedulable tasks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskGraph {
    tasks: BTreeMap<TaskId, Task>,
}

impl TaskGraph {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, task: Task) -> Result<(), ScheduleError> {
        if self.tasks.contains_key(task.id()) {
            return Err(ScheduleError::DuplicateTask(task.id().clone()));
        }
        self.tasks.insert(task.id().clone(), task);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, id: &TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn tasks(&self) -> impl ExactSizeIterator<Item = &Task> {
        self.tasks.values()
    }

    pub fn validate(&self) -> Result<(), ScheduleError> {
        for task in self.tasks.values() {
            for dependency in task.dependencies() {
                if !self.tasks.contains_key(dependency) {
                    return Err(ScheduleError::MissingDependency {
                        task: task.id().clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }
        topological_order(&self.tasks).map(|_| ())
    }

    /// Produces bounded parallel waves and critical-path evidence.
    pub fn plan(&self, capacity: u64) -> Result<SchedulePlan, ScheduleError> {
        if capacity == 0 {
            return Err(ScheduleError::ZeroCapacity);
        }
        self.validate()?;
        for task in self.tasks.values() {
            if task.cost() > capacity {
                return Err(ScheduleError::TaskExceedsCapacity {
                    task: task.id().clone(),
                    cost: task.cost(),
                    capacity,
                });
            }
        }

        let ordered = topological_order(&self.tasks)?;
        let waves = bounded_waves(&self.tasks, capacity)?;
        let total_cost = self.tasks.values().try_fold(0_u64, |sum, task| {
            sum.checked_add(task.cost()).ok_or(ScheduleError::CostOverflow)
        })?;
        let critical_path = critical_path(&self.tasks, &ordered)?;
        let critical_cost = critical_path.iter().try_fold(0_u64, |sum, id| {
            sum.checked_add(self.tasks[id].cost())
                .ok_or(ScheduleError::CostOverflow)
        })?;
        let irreversible = ordered
            .iter()
            .filter(|id| !self.tasks[*id].is_reversible())
            .cloned()
            .collect::<Vec<_>>();

        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "schedule-plan-v1")
            .u64("capacity", capacity)
            .u64("total-cost", total_cost)
            .u64("critical-cost", critical_cost)
            .u64("wave-count", waves.len() as u64);
        for wave in &waves {
            encoder.u64("wave-size", wave.len() as u64);
            for id in wave {
                encoder
                    .text("task", id.as_str())
                    .field("task-digest", &self.tasks[id].digest().0);
            }
        }
        let digest = encoder.digest();
        Ok(SchedulePlan {
            ordered,
            waves,
            total_cost,
            critical_path,
            critical_cost,
            irreversible,
            capacity,
            digest,
        })
    }
}

/// Dependency-correct bounded execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePlan {
    ordered: Vec<TaskId>,
    waves: Vec<Vec<TaskId>>,
    total_cost: u64,
    critical_path: Vec<TaskId>,
    critical_cost: u64,
    irreversible: Vec<TaskId>,
    capacity: u64,
    digest: Digest,
}

impl SchedulePlan {
    #[must_use]
    pub fn ordered(&self) -> &[TaskId] {
        &self.ordered
    }

    #[must_use]
    pub fn waves(&self) -> &[Vec<TaskId>] {
        &self.waves
    }

    #[must_use]
    pub const fn total_cost(&self) -> u64 {
        self.total_cost
    }

    #[must_use]
    pub fn critical_path(&self) -> &[TaskId] {
        &self.critical_path
    }

    #[must_use]
    pub const fn critical_cost(&self) -> u64 {
        self.critical_cost
    }

    #[must_use]
    pub fn irreversible(&self) -> &[TaskId] {
        &self.irreversible
    }

    #[must_use]
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

fn topological_order(tasks: &BTreeMap<TaskId, Task>) -> Result<Vec<TaskId>, ScheduleError> {
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum Mark {
        Visiting,
        Complete,
    }

    fn visit(
        tasks: &BTreeMap<TaskId, Task>,
        id: &TaskId,
        marks: &mut BTreeMap<TaskId, Mark>,
        stack: &mut Vec<TaskId>,
        ordered: &mut Vec<TaskId>,
    ) -> Result<(), ScheduleError> {
        match marks.get(id) {
            Some(Mark::Complete) => return Ok(()),
            Some(Mark::Visiting) => {
                let start = stack.iter().position(|task| task == id).unwrap_or(0);
                let mut cycle = stack[start..].to_vec();
                cycle.push(id.clone());
                return Err(ScheduleError::Cycle(cycle));
            }
            None => {}
        }
        let task = &tasks[id];
        marks.insert(id.clone(), Mark::Visiting);
        stack.push(id.clone());
        for dependency in task.dependencies() {
            if !tasks.contains_key(dependency) {
                return Err(ScheduleError::MissingDependency {
                    task: id.clone(),
                    dependency: dependency.clone(),
                });
            }
            visit(tasks, dependency, marks, stack, ordered)?;
        }
        stack.pop();
        marks.insert(id.clone(), Mark::Complete);
        ordered.push(id.clone());
        Ok(())
    }

    let mut marks = BTreeMap::new();
    let mut stack = Vec::new();
    let mut ordered = Vec::with_capacity(tasks.len());
    for id in tasks.keys() {
        visit(tasks, id, &mut marks, &mut stack, &mut ordered)?;
    }
    Ok(ordered)
}

fn bounded_waves(
    tasks: &BTreeMap<TaskId, Task>,
    capacity: u64,
) -> Result<Vec<Vec<TaskId>>, ScheduleError> {
    let mut remaining: BTreeSet<TaskId> = tasks.keys().cloned().collect();
    let mut completed = BTreeSet::new();
    let mut waves = Vec::new();

    while !remaining.is_empty() {
        let mut ready = remaining
            .iter()
            .filter(|id| tasks[*id].dependencies().all(|dep| completed.contains(dep)))
            .cloned()
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(ScheduleError::Cycle(remaining.into_iter().collect()));
        }
        ready.sort_by_key(|id| (Reverse(tasks[id].priority_value()), id.clone()));

        let mut wave = Vec::new();
        let mut used = 0_u64;
        for id in ready {
            let cost = tasks[&id].cost();
            if used + cost <= capacity {
                used += cost;
                wave.push(id);
            }
        }
        if wave.is_empty() {
            let id = remaining.iter().next().expect("non-empty remaining").clone();
            return Err(ScheduleError::TaskExceedsCapacity {
                task: id.clone(),
                cost: tasks[&id].cost(),
                capacity,
            });
        }
        for id in &wave {
            remaining.remove(id);
            completed.insert(id.clone());
        }
        waves.push(wave);
    }
    Ok(waves)
}

fn critical_path(
    tasks: &BTreeMap<TaskId, Task>,
    ordered: &[TaskId],
) -> Result<Vec<TaskId>, ScheduleError> {
    let mut distance = BTreeMap::<TaskId, u64>::new();
    let mut predecessor = BTreeMap::<TaskId, TaskId>::new();
    for id in ordered {
        let task = &tasks[id];
        let (parent, parent_distance) = task
            .dependencies()
            .map(|dependency| (dependency, distance[dependency]))
            .max_by_key(|(dependency, value)| (*value, Reverse((*dependency).clone())))
            .map_or((None, 0), |(id, value)| (Some((*id).clone()), value));
        let current = parent_distance
            .checked_add(task.cost())
            .ok_or(ScheduleError::CostOverflow)?;
        distance.insert(id.clone(), current);
        if let Some(parent) = parent {
            predecessor.insert(id.clone(), parent);
        }
    }

    let Some(mut cursor) = ordered
        .iter()
        .max_by_key(|id| (distance[*id], Reverse((*id).clone())))
        .cloned()
    else {
        return Ok(Vec::new());
    };
    let mut path = vec![cursor.clone()];
    while let Some(parent) = predecessor.get(&cursor).cloned() {
        path.push(parent.clone());
        cursor = parent;
    }
    path.reverse();
    Ok(path)
}

/// Task execution adapter.
pub trait TaskExecutor {
    fn execute(&mut self, task: &Task, attempt: u32) -> TaskOutcome;
}

/// Final or retryable task outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskOutcome {
    Succeeded { output: Vec<u8> },
    Retryable { code: String, detail: String },
    Failed { code: String, detail: String },
    Refused { code: String, detail: String },
    Skipped { dependency: TaskId },
}

impl TaskOutcome {
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        matches!(self, Self::Succeeded { .. })
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Succeeded { output } => {
                encoder.text("outcome", "succeeded").field("output", output);
            }
            Self::Retryable { code, detail } => {
                encoder
                    .text("outcome", "retryable")
                    .text("code", code)
                    .text("detail", detail);
            }
            Self::Failed { code, detail } => {
                encoder
                    .text("outcome", "failed")
                    .text("code", code)
                    .text("detail", detail);
            }
            Self::Refused { code, detail } => {
                encoder
                    .text("outcome", "refused")
                    .text("code", code)
                    .text("detail", detail);
            }
            Self::Skipped { dependency } => {
                encoder
                    .text("outcome", "skipped")
                    .text("dependency", dependency.as_str());
            }
        }
    }
}

/// Immutable task execution evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskReceipt {
    task: TaskId,
    attempts: u32,
    outcome: TaskOutcome,
    previous: Digest,
    digest: Digest,
}

impl TaskReceipt {
    fn new(task: &Task, attempts: u32, outcome: TaskOutcome, previous: Digest) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "task-receipt-v1")
            .text("task", task.id().as_str())
            .field("task-digest", &task.digest().0)
            .u64("attempts", u64::from(attempts))
            .field("previous", &previous.0);
        outcome.encode(&mut encoder);
        Self {
            task: task.id().clone(),
            attempts,
            outcome,
            previous,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub const fn task(&self) -> &TaskId {
        &self.task
    }

    #[must_use]
    pub const fn attempts(&self) -> u32 {
        self.attempts
    }

    #[must_use]
    pub const fn outcome(&self) -> &TaskOutcome {
        &self.outcome
    }

    #[must_use]
    pub const fn previous(&self) -> Digest {
        self.previous
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Complete dependency-aware execution report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleExecution {
    plan_digest: Digest,
    receipts: Vec<TaskReceipt>,
    head: Digest,
}

impl ScheduleExecution {
    #[must_use]
    pub const fn plan_digest(&self) -> Digest {
        self.plan_digest
    }

    #[must_use]
    pub fn receipts(&self) -> &[TaskReceipt] {
        &self.receipts
    }

    #[must_use]
    pub const fn head(&self) -> Digest {
        self.head
    }

    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.receipts
            .iter()
            .all(|receipt| receipt.outcome().succeeded())
    }
}

/// Plans and executes all tasks, retrying only typed retryable failures.
pub fn execute_schedule<E: TaskExecutor>(
    graph: &TaskGraph,
    capacity: u64,
    executor: &mut E,
) -> Result<ScheduleExecution, ScheduleError> {
    let plan = graph.plan(capacity)?;
    let mut outcomes = BTreeMap::<TaskId, TaskOutcome>::new();
    let mut receipts = Vec::with_capacity(plan.ordered().len());
    let mut head = Digest::ZERO;

    for id in plan.ordered() {
        let task = &graph.tasks[id];
        if let Some(dependency) = task
            .dependencies()
            .find(|dependency| !outcomes.get(*dependency).is_some_and(TaskOutcome::succeeded))
            .cloned()
        {
            let outcome = TaskOutcome::Skipped { dependency };
            let receipt = TaskReceipt::new(task, 0, outcome.clone(), head);
            head = receipt.digest();
            receipts.push(receipt);
            outcomes.insert(id.clone(), outcome);
            continue;
        }

        let mut attempt = 1_u32;
        let outcome = loop {
            let candidate = executor.execute(task, attempt);
            match candidate {
                TaskOutcome::Retryable { .. } if attempt < task.attempts() => {
                    attempt += 1;
                }
                TaskOutcome::Retryable { code, detail } => {
                    break TaskOutcome::Failed {
                        code: format!("RETRIES_EXHAUSTED:{code}"),
                        detail,
                    };
                }
                final_outcome => break final_outcome,
            }
        };
        let receipt = TaskReceipt::new(task, attempt, outcome.clone(), head);
        head = receipt.digest();
        receipts.push(receipt);
        outcomes.insert(id.clone(), outcome);
    }

    Ok(ScheduleExecution {
        plan_digest: plan.digest(),
        receipts,
        head,
    })
}

#[cfg(test)]
mod tests {
    use super::{execute_schedule, ScheduleError, Task, TaskExecutor, TaskGraph, TaskId, TaskOutcome};

    fn id(value: &str) -> TaskId {
        TaskId::new(value).unwrap()
    }

    #[test]
    fn bounded_waves_and_critical_path_are_deterministic() {
        let mut graph = TaskGraph::new();
        graph.insert(Task::new(id("a")).cost_units(2)).unwrap();
        graph
            .insert(Task::new(id("b")).depends_on(id("a")).cost_units(3))
            .unwrap();
        graph
            .insert(Task::new(id("c")).depends_on(id("a")).cost_units(1))
            .unwrap();
        graph
            .insert(
                Task::new(id("d"))
                    .depends_on(id("b"))
                    .depends_on(id("c"))
                    .cost_units(5),
            )
            .unwrap();
        let plan = graph.plan(5).unwrap();
        assert_eq!(plan.critical_cost(), 10);
        assert_eq!(
            plan.critical_path()
                .iter()
                .map(TaskId::as_str)
                .collect::<Vec<_>>(),
            ["a", "b", "d"]
        );
        assert_eq!(plan.waves().len(), 3);
    }

    #[test]
    fn capacity_refuses_oversized_task() {
        let mut graph = TaskGraph::new();
        graph.insert(Task::new(id("huge")).cost_units(9)).unwrap();
        assert_eq!(
            graph.plan(8).unwrap_err(),
            ScheduleError::TaskExceedsCapacity {
                task: id("huge"),
                cost: 9,
                capacity: 8,
            }
        );
    }

    struct Flaky {
        calls: usize,
    }

    impl TaskExecutor for Flaky {
        fn execute(&mut self, task: &Task, attempt: u32) -> TaskOutcome {
            self.calls += 1;
            if task.id().as_str() == "retry" && attempt == 1 {
                TaskOutcome::Retryable {
                    code: "TRANSIENT".to_owned(),
                    detail: "retry".to_owned(),
                }
            } else {
                TaskOutcome::Succeeded {
                    output: task.id().as_str().as_bytes().to_vec(),
                }
            }
        }
    }

    #[test]
    fn retryable_task_retries_and_dependents_execute() {
        let mut graph = TaskGraph::new();
        graph
            .insert(Task::new(id("retry")).max_attempts(2))
            .unwrap();
        graph
            .insert(Task::new(id("after")).depends_on(id("retry")))
            .unwrap();
        let mut executor = Flaky { calls: 0 };
        let report = execute_schedule(&graph, 2, &mut executor).unwrap();
        assert!(report.succeeded());
        assert_eq!(executor.calls, 3);
        assert_eq!(report.receipts()[0].attempts(), 2);
    }

    struct Failing;

    impl TaskExecutor for Failing {
        fn execute(&mut self, task: &Task, _attempt: u32) -> TaskOutcome {
            if task.id().as_str() == "root" {
                TaskOutcome::Failed {
                    code: "BROKEN".to_owned(),
                    detail: "root failed".to_owned(),
                }
            } else {
                TaskOutcome::Succeeded { output: Vec::new() }
            }
        }
    }

    #[test]
    fn failed_dependency_skips_downstream_work() {
        let mut graph = TaskGraph::new();
        graph.insert(Task::new(id("root"))).unwrap();
        graph
            .insert(Task::new(id("dependent")).depends_on(id("root")))
            .unwrap();
        let report = execute_schedule(&graph, 2, &mut Failing).unwrap();
        assert!(matches!(
            report.receipts()[1].outcome(),
            TaskOutcome::Skipped { dependency } if dependency.as_str() == "root"
        ));
    }
}
