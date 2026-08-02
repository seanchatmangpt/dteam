//! Object-centric event log, process discovery, metrics, and conformance.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::FactValue;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

macro_rules! process_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ProcessError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ProcessError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

process_id!(ObjectId);
process_id!(EventId);
process_id!(ObjectType);
process_id!(Activity);

/// Object participating in one or more process events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectRecord {
    id: ObjectId,
    object_type: ObjectType,
    attributes: BTreeMap<String, FactValue>,
}

impl ObjectRecord {
    #[must_use]
    pub fn new(id: ObjectId, object_type: ObjectType) -> Self {
        Self {
            id,
            object_type,
            attributes: BTreeMap::new(),
        }
    }

    pub fn insert_attribute(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Option<FactValue> {
        self.attributes.insert(key.into(), value.into())
    }

    #[must_use]
    pub const fn id(&self) -> &ObjectId {
        &self.id
    }

    #[must_use]
    pub const fn object_type(&self) -> &ObjectType {
        &self.object_type
    }

    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&str, &FactValue)> {
        self.attributes
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "ocel-object-v1")
            .text("id", self.id.as_str())
            .text("object-type", self.object_type.as_str())
            .u64("attribute-count", self.attributes.len() as u64);
        for (key, value) in &self.attributes {
            encoder.text("attribute-key", key);
            value.encode(&mut encoder, "attribute-type");
        }
        encoder.digest()
    }
}

/// Event connected to one or more objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    id: EventId,
    activity: Activity,
    logical_time: u64,
    objects: BTreeSet<ObjectId>,
    attributes: BTreeMap<String, FactValue>,
}

impl EventRecord {
    #[must_use]
    pub fn new(id: EventId, activity: Activity, logical_time: u64) -> Self {
        Self {
            id,
            activity,
            logical_time,
            objects: BTreeSet::new(),
            attributes: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn relating(mut self, object: ObjectId) -> Self {
        self.objects.insert(object);
        self
    }

    pub fn relate(&mut self, object: ObjectId) -> bool {
        self.objects.insert(object)
    }

    pub fn insert_attribute(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Option<FactValue> {
        self.attributes.insert(key.into(), value.into())
    }

    #[must_use]
    pub const fn id(&self) -> &EventId {
        &self.id
    }

    #[must_use]
    pub const fn activity(&self) -> &Activity {
        &self.activity
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    pub fn objects(&self) -> impl ExactSizeIterator<Item = &ObjectId> {
        self.objects.iter()
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "ocel-event-v1")
            .text("id", self.id.as_str())
            .text("activity", self.activity.as_str())
            .u64("logical-time", self.logical_time)
            .u64("object-count", self.objects.len() as u64);
        for object in &self.objects {
            encoder.text("object", object.as_str());
        }
        encoder.u64("attribute-count", self.attributes.len() as u64);
        for (key, value) in &self.attributes {
            encoder.text("attribute-key", key);
            value.encode(&mut encoder, "attribute-type");
        }
        encoder.digest()
    }
}

/// Event-log construction or query failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessError {
    EmptyIdentifier(&'static str),
    DuplicateObject(ObjectId),
    DuplicateEvent(EventId),
    UnknownObject(ObjectId),
    EventWithoutObjects(EventId),
    LogicalTimeRegression { previous: u64, actual: u64 },
}

impl Display for ProcessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::DuplicateObject(id) => write!(formatter, "duplicate object `{id}`"),
            Self::DuplicateEvent(id) => write!(formatter, "duplicate event `{id}`"),
            Self::UnknownObject(id) => write!(formatter, "event references unknown object `{id}`"),
            Self::EventWithoutObjects(id) => write!(formatter, "event `{id}` has no related objects"),
            Self::LogicalTimeRegression { previous, actual } => write!(
                formatter,
                "logical time regressed from {previous} to {actual}"
            ),
        }
    }
}

impl std::error::Error for ProcessError {}

/// Object-centric event log with referential and temporal integrity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectEventLog {
    objects: BTreeMap<ObjectId, ObjectRecord>,
    events: Vec<EventRecord>,
    event_ids: BTreeSet<EventId>,
}

impl ObjectEventLog {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&mut self, object: ObjectRecord) -> Result<(), ProcessError> {
        if self.objects.contains_key(object.id()) {
            return Err(ProcessError::DuplicateObject(object.id().clone()));
        }
        self.objects.insert(object.id().clone(), object);
        Ok(())
    }

    pub fn append_event(&mut self, event: EventRecord) -> Result<(), ProcessError> {
        if self.event_ids.contains(event.id()) {
            return Err(ProcessError::DuplicateEvent(event.id().clone()));
        }
        if event.objects.is_empty() {
            return Err(ProcessError::EventWithoutObjects(event.id().clone()));
        }
        for object in event.objects() {
            if !self.objects.contains_key(object) {
                return Err(ProcessError::UnknownObject(object.clone()));
            }
        }
        if let Some(previous) = self.events.last().map(EventRecord::logical_time) {
            if event.logical_time() < previous {
                return Err(ProcessError::LogicalTimeRegression {
                    previous,
                    actual: event.logical_time(),
                });
            }
        }
        self.event_ids.insert(event.id().clone());
        self.events.push(event);
        Ok(())
    }

    #[must_use]
    pub fn object(&self, id: &ObjectId) -> Option<&ObjectRecord> {
        self.objects.get(id)
    }

    #[must_use]
    pub fn events(&self) -> &[EventRecord] {
        &self.events
    }

    pub fn objects(&self) -> impl ExactSizeIterator<Item = &ObjectRecord> {
        self.objects.values()
    }

    /// Returns events related to an object in append order.
    pub fn trace<'log>(
        &'log self,
        object: &'log ObjectId,
    ) -> impl Iterator<Item = &'log EventRecord> + 'log {
        self.events
            .iter()
            .filter(move |event| event.objects.contains(object))
    }

    /// Computes deterministic event-log identity.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "object-event-log-v1")
            .u64("object-count", self.objects.len() as u64);
        for object in self.objects.values() {
            encoder
                .text("object", object.id().as_str())
                .field("object-digest", &object.digest().0);
        }
        encoder.u64("event-count", self.events.len() as u64);
        for event in &self.events {
            encoder
                .text("event", event.id().as_str())
                .field("event-digest", &event.digest().0);
        }
        encoder.digest()
    }

    /// Computes activity, object, variant, and directly-follows metrics.
    #[must_use]
    pub fn metrics(&self) -> ProcessMetrics {
        let mut activity_frequency = BTreeMap::new();
        for event in &self.events {
            *activity_frequency
                .entry(event.activity().clone())
                .or_insert(0_usize) += 1;
        }

        let mut variants: BTreeMap<Vec<Activity>, usize> = BTreeMap::new();
        let mut directly_follows = BTreeMap::new();
        let mut total_trace_events = 0_usize;
        for object in self.objects.keys() {
            let trace = self
                .trace(object)
                .map(|event| event.activity().clone())
                .collect::<Vec<_>>();
            total_trace_events += trace.len();
            *variants.entry(trace.clone()).or_insert(0) += 1;
            for pair in trace.windows(2) {
                *directly_follows
                    .entry((pair[0].clone(), pair[1].clone()))
                    .or_insert(0_usize) += 1;
            }
        }

        ProcessMetrics {
            object_count: self.objects.len(),
            event_count: self.events.len(),
            activity_frequency,
            variant_frequency: variants,
            directly_follows,
            mean_events_per_object: if self.objects.is_empty() {
                0.0
            } else {
                total_trace_events as f64 / self.objects.len() as f64
            },
        }
    }
}

/// Aggregate object-centric process statistics.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessMetrics {
    object_count: usize,
    event_count: usize,
    activity_frequency: BTreeMap<Activity, usize>,
    variant_frequency: BTreeMap<Vec<Activity>, usize>,
    directly_follows: BTreeMap<(Activity, Activity), usize>,
    mean_events_per_object: f64,
}

impl ProcessMetrics {
    #[must_use]
    pub const fn object_count(&self) -> usize {
        self.object_count
    }

    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    #[must_use]
    pub fn activity_frequency(&self) -> &BTreeMap<Activity, usize> {
        &self.activity_frequency
    }

    #[must_use]
    pub fn variant_frequency(&self) -> &BTreeMap<Vec<Activity>, usize> {
        &self.variant_frequency
    }

    #[must_use]
    pub fn directly_follows(&self) -> &BTreeMap<(Activity, Activity), usize> {
        &self.directly_follows
    }

    #[must_use]
    pub const fn mean_events_per_object(&self) -> f64 {
        self.mean_events_per_object
    }
}

/// Discovered or prescribed directly-follows process model.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransitionSystem {
    starts: BTreeSet<Activity>,
    ends: BTreeSet<Activity>,
    edges: BTreeSet<(Activity, Activity)>,
}

impl TransitionSystem {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_start(&mut self, activity: Activity) -> bool {
        self.starts.insert(activity)
    }

    pub fn allow_end(&mut self, activity: Activity) -> bool {
        self.ends.insert(activity)
    }

    pub fn allow_transition(&mut self, from: Activity, to: Activity) -> bool {
        self.edges.insert((from, to))
    }

    #[must_use]
    pub fn starts(&self) -> &BTreeSet<Activity> {
        &self.starts
    }

    #[must_use]
    pub fn ends(&self) -> &BTreeSet<Activity> {
        &self.ends
    }

    #[must_use]
    pub fn edges(&self) -> &BTreeSet<(Activity, Activity)> {
        &self.edges
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "transition-system-v1")
            .u64("start-count", self.starts.len() as u64);
        for activity in &self.starts {
            encoder.text("start", activity.as_str());
        }
        encoder.u64("end-count", self.ends.len() as u64);
        for activity in &self.ends {
            encoder.text("end", activity.as_str());
        }
        encoder.u64("edge-count", self.edges.len() as u64);
        for (from, to) in &self.edges {
            encoder
                .text("from", from.as_str())
                .text("to", to.as_str());
        }
        encoder.digest()
    }

    /// Checks every object trace and reports all model deviations.
    #[must_use]
    pub fn conform(&self, log: &ObjectEventLog) -> ConformanceReport {
        let mut violations = Vec::new();
        let mut inspected_moves = 0_usize;
        for object in log.objects.keys() {
            let trace = log
                .trace(object)
                .map(|event| event.activity().clone())
                .collect::<Vec<_>>();
            if trace.is_empty() {
                continue;
            }
            inspected_moves += trace.len() + 1;
            if !self.starts.is_empty() && !self.starts.contains(&trace[0]) {
                violations.push(ConformanceViolation {
                    object: object.clone(),
                    position: 0,
                    code: "INVALID_START",
                    from: None,
                    to: Some(trace[0].clone()),
                });
            }
            for (position, pair) in trace.windows(2).enumerate() {
                if !self.edges.contains(&(pair[0].clone(), pair[1].clone())) {
                    violations.push(ConformanceViolation {
                        object: object.clone(),
                        position: position + 1,
                        code: "INVALID_TRANSITION",
                        from: Some(pair[0].clone()),
                        to: Some(pair[1].clone()),
                    });
                }
            }
            let last = trace.last().expect("non-empty trace");
            if !self.ends.is_empty() && !self.ends.contains(last) {
                violations.push(ConformanceViolation {
                    object: object.clone(),
                    position: trace.len(),
                    code: "INVALID_END",
                    from: Some(last.clone()),
                    to: None,
                });
            }
        }

        let fitness = if inspected_moves == 0 {
            1.0
        } else {
            (1.0 - violations.len() as f64 / inspected_moves as f64).max(0.0)
        };
        ConformanceReport {
            inspected_moves,
            violations,
            fitness,
            model_digest: self.digest(),
            log_digest: log.digest(),
        }
    }
}

/// One precise conformance deviation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceViolation {
    object: ObjectId,
    position: usize,
    code: &'static str,
    from: Option<Activity>,
    to: Option<Activity>,
}

impl ConformanceViolation {
    #[must_use]
    pub const fn object(&self) -> &ObjectId {
        &self.object
    }

    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn from(&self) -> Option<&Activity> {
        self.from.as_ref()
    }

    #[must_use]
    pub const fn to(&self) -> Option<&Activity> {
        self.to.as_ref()
    }
}

/// Complete conformance result.
#[derive(Clone, Debug, PartialEq)]
pub struct ConformanceReport {
    inspected_moves: usize,
    violations: Vec<ConformanceViolation>,
    fitness: f64,
    model_digest: Digest,
    log_digest: Digest,
}

impl ConformanceReport {
    #[must_use]
    pub const fn inspected_moves(&self) -> usize {
        self.inspected_moves
    }

    #[must_use]
    pub fn violations(&self) -> &[ConformanceViolation] {
        &self.violations
    }

    #[must_use]
    pub const fn fitness(&self) -> f64 {
        self.fitness
    }

    #[must_use]
    pub const fn model_digest(&self) -> Digest {
        self.model_digest
    }

    #[must_use]
    pub const fn log_digest(&self) -> Digest {
        self.log_digest
    }

    #[must_use]
    pub fn is_conformant(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Discovers a directly-follows model using a minimum edge frequency.
#[must_use]
pub fn discover_transition_system(
    log: &ObjectEventLog,
    minimum_frequency: usize,
) -> TransitionSystem {
    let threshold = minimum_frequency.max(1);
    let mut starts = BTreeMap::<Activity, usize>::new();
    let mut ends = BTreeMap::<Activity, usize>::new();
    for object in log.objects.keys() {
        let trace = log
            .trace(object)
            .map(|event| event.activity().clone())
            .collect::<Vec<_>>();
        if let Some(first) = trace.first() {
            *starts.entry(first.clone()).or_insert(0) += 1;
        }
        if let Some(last) = trace.last() {
            *ends.entry(last.clone()).or_insert(0) += 1;
        }
    }

    let metrics = log.metrics();
    let mut system = TransitionSystem::new();
    for (activity, count) in starts {
        if count >= threshold {
            system.allow_start(activity);
        }
    }
    for (activity, count) in ends {
        if count >= threshold {
            system.allow_end(activity);
        }
    }
    for ((from, to), count) in metrics.directly_follows {
        if count >= threshold {
            system.allow_transition(from, to);
        }
    }
    system
}

#[cfg(test)]
mod tests {
    use super::{
        discover_transition_system, Activity, EventId, EventRecord, ObjectEventLog, ObjectId,
        ObjectRecord, ObjectType,
    };

    fn sample_log() -> ObjectEventLog {
        let mut log = ObjectEventLog::new();
        for case in ["case-1", "case-2"] {
            log.add_object(ObjectRecord::new(
                ObjectId::new(case).unwrap(),
                ObjectType::new("case").unwrap(),
            ))
            .unwrap();
        }
        let mut sequence = 0;
        for (case, activities) in [
            ("case-1", ["open", "review", "close"]),
            ("case-2", ["open", "review", "close"]),
        ] {
            for activity in activities {
                let event = EventRecord::new(
                    EventId::new(format!("event-{sequence}")).unwrap(),
                    Activity::new(activity).unwrap(),
                    sequence,
                )
                .relating(ObjectId::new(case).unwrap());
                log.append_event(event).unwrap();
                sequence += 1;
            }
        }
        log
    }

    #[test]
    fn metrics_capture_variants_and_edges() {
        let log = sample_log();
        let metrics = log.metrics();
        assert_eq!(metrics.object_count(), 2);
        assert_eq!(metrics.event_count(), 6);
        assert_eq!(metrics.variant_frequency().values().sum::<usize>(), 2);
        assert_eq!(metrics.directly_follows().values().sum::<usize>(), 4);
    }

    #[test]
    fn discovered_model_replays_source_log() {
        let log = sample_log();
        let model = discover_transition_system(&log, 2);
        let report = model.conform(&log);
        assert!(report.is_conformant());
        assert_eq!(report.fitness(), 1.0);
    }

    #[test]
    fn unknown_transition_is_reported() {
        let mut log = sample_log();
        log.add_object(ObjectRecord::new(
            ObjectId::new("case-3").unwrap(),
            ObjectType::new("case").unwrap(),
        ))
        .unwrap();
        let model = discover_transition_system(&log, 2);
        log.append_event(
            EventRecord::new(
                EventId::new("event-extra-1").unwrap(),
                Activity::new("open").unwrap(),
                100,
            )
            .relating(ObjectId::new("case-3").unwrap()),
        )
        .unwrap();
        log.append_event(
            EventRecord::new(
                EventId::new("event-extra-2").unwrap(),
                Activity::new("cancel").unwrap(),
                101,
            )
            .relating(ObjectId::new("case-3").unwrap()),
        )
        .unwrap();
        let report = model.conform(&log);
        assert!(!report.is_conformant());
        assert!(report
            .violations()
            .iter()
            .any(|violation| violation.code() == "INVALID_TRANSITION"));
    }
}
