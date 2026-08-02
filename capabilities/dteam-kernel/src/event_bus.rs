//! Durable deterministic event transport with partitions, offsets, leases, and dead letters.

use crate::hash::{sha256, CanonicalEncoder, Digest};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

macro_rules! bus_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EventBusError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(EventBusError::EmptyIdentifier(stringify!($name)));
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

bus_id!(TopicId);
bus_id!(EventId);
bus_id!(SubscriptionId);
bus_id!(ConsumerId);
bus_id!(DeliveryId);

/// Topic transport policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopicConfig {
    partitions: u16,
    retention_events_per_partition: usize,
    visibility_timeout: u64,
    maximum_delivery_attempts: u32,
}

impl TopicConfig {
    pub fn new(
        partitions: u16,
        retention_events_per_partition: usize,
        visibility_timeout: u64,
        maximum_delivery_attempts: u32,
    ) -> Result<Self, EventBusError> {
        if partitions == 0 {
            return Err(EventBusError::ZeroPartitions);
        }
        if retention_events_per_partition == 0 {
            return Err(EventBusError::ZeroRetention);
        }
        if visibility_timeout == 0 {
            return Err(EventBusError::ZeroVisibilityTimeout);
        }
        if maximum_delivery_attempts == 0 {
            return Err(EventBusError::ZeroDeliveryAttempts);
        }
        Ok(Self {
            partitions,
            retention_events_per_partition,
            visibility_timeout,
            maximum_delivery_attempts,
        })
    }

    #[must_use]
    pub const fn partitions(&self) -> u16 {
        self.partitions
    }

    #[must_use]
    pub const fn retention_events_per_partition(&self) -> usize {
        self.retention_events_per_partition
    }

    #[must_use]
    pub const fn visibility_timeout(&self) -> u64 {
        self.visibility_timeout
    }

    #[must_use]
    pub const fn maximum_delivery_attempts(&self) -> u32 {
        self.maximum_delivery_attempts
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "topic-config-v1")
            .u64("partitions", u64::from(self.partitions))
            .u64("retention", self.retention_events_per_partition as u64)
            .u64("visibility-timeout", self.visibility_timeout)
            .u64(
                "maximum-delivery-attempts",
                u64::from(self.maximum_delivery_attempts),
            );
        encoder.digest()
    }
}

/// Immutable published event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEnvelope {
    id: EventId,
    topic: TopicId,
    partition: u16,
    offset: u64,
    logical_time: u64,
    key: Vec<u8>,
    payload: Vec<u8>,
    headers: BTreeMap<String, Vec<u8>>,
    digest: Digest,
}

impl EventEnvelope {
    #[allow(clippy::too_many_arguments)]
    fn manufacture(
        id: EventId,
        topic: TopicId,
        partition: u16,
        offset: u64,
        logical_time: u64,
        key: Vec<u8>,
        payload: Vec<u8>,
        headers: BTreeMap<String, Vec<u8>>,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "event-envelope-v1")
            .text("id", id.as_str())
            .text("topic", topic.as_str())
            .u64("partition", u64::from(partition))
            .u64("offset", offset)
            .u64("logical-time", logical_time)
            .field("key", &key)
            .field("payload", &payload)
            .u64("header-count", headers.len() as u64);
        for (name, value) in &headers {
            encoder.text("header", name).field("header-value", value);
        }
        Self {
            id,
            topic,
            partition,
            offset,
            logical_time,
            key,
            payload,
            headers,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &EventId {
        &self.id
    }

    #[must_use]
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    #[must_use]
    pub const fn partition(&self) -> u16 {
        self.partition
    }

    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn headers(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.headers
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Publication request before partition and offset assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishRequest {
    id: EventId,
    topic: TopicId,
    logical_time: u64,
    key: Vec<u8>,
    payload: Vec<u8>,
    headers: BTreeMap<String, Vec<u8>>,
    digest: Digest,
}

impl PublishRequest {
    pub fn new(
        id: EventId,
        topic: TopicId,
        logical_time: u64,
        key: Vec<u8>,
        payload: Vec<u8>,
    ) -> Self {
        let mut request = Self {
            id,
            topic,
            logical_time,
            key,
            payload,
            headers: BTreeMap::new(),
            digest: Digest::ZERO,
        };
        request.digest = request.recompute_digest();
        request
    }

    pub fn insert_header(
        &mut self,
        name: impl Into<String>,
        value: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let previous = self.headers.insert(name.into(), value);
        self.digest = self.recompute_digest();
        previous
    }

    #[must_use]
    pub const fn id(&self) -> &EventId {
        &self.id
    }

    #[must_use]
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn headers(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.headers
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "publish-request-v1")
            .text("id", self.id.as_str())
            .text("topic", self.topic.as_str())
            .u64("logical-time", self.logical_time)
            .field("key", &self.key)
            .field("payload", &self.payload)
            .u64("header-count", self.headers.len() as u64);
        for (name, value) in &self.headers {
            encoder.text("header", name).field("header-value", value);
        }
        encoder.digest()
    }
}

/// Consumer subscription to a topic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    id: SubscriptionId,
    topic: TopicId,
    consumer: ConsumerId,
    start_offsets: BTreeMap<u16, u64>,
}

impl Subscription {
    #[must_use]
    pub fn new(id: SubscriptionId, topic: TopicId, consumer: ConsumerId) -> Self {
        Self {
            id,
            topic,
            consumer,
            start_offsets: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn start_at(mut self, partition: u16, offset: u64) -> Self {
        self.start_offsets.insert(partition, offset);
        self
    }

    #[must_use]
    pub const fn id(&self) -> &SubscriptionId {
        &self.id
    }

    #[must_use]
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    #[must_use]
    pub const fn consumer(&self) -> &ConsumerId {
        &self.consumer
    }
}

/// Leased event delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivery {
    id: DeliveryId,
    subscription: SubscriptionId,
    consumer: ConsumerId,
    event: EventEnvelope,
    attempt: u32,
    leased_until: u64,
    digest: Digest,
}

impl Delivery {
    fn manufacture(
        id: DeliveryId,
        subscription: SubscriptionId,
        consumer: ConsumerId,
        event: EventEnvelope,
        attempt: u32,
        leased_until: u64,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "event-delivery-v1")
            .text("id", id.as_str())
            .text("subscription", subscription.as_str())
            .text("consumer", consumer.as_str())
            .field("event", &event.digest().0)
            .u64("attempt", u64::from(attempt))
            .u64("leased-until", leased_until);
        Self {
            id,
            subscription,
            consumer,
            event,
            attempt,
            leased_until,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &DeliveryId {
        &self.id
    }

    #[must_use]
    pub const fn subscription(&self) -> &SubscriptionId {
        &self.subscription
    }

    #[must_use]
    pub const fn consumer(&self) -> &ConsumerId {
        &self.consumer
    }

    #[must_use]
    pub const fn event(&self) -> &EventEnvelope {
        &self.event
    }

    #[must_use]
    pub const fn attempt(&self) -> u32 {
        self.attempt
    }

    #[must_use]
    pub const fn leased_until(&self) -> u64 {
        self.leased_until
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InFlight {
    delivery: Delivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SubscriptionState {
    subscription: Subscription,
    committed_offsets: BTreeMap<u16, u64>,
    attempts: BTreeMap<(u16, u64), u32>,
    in_flight: BTreeMap<DeliveryId, InFlight>,
    leased_events: BTreeSet<(u16, u64)>,
    dead_letters: Vec<EventEnvelope>,
    next_delivery: u64,
}

impl SubscriptionState {
    fn new(subscription: Subscription, partitions: u16) -> Self {
        let committed_offsets = (0..partitions)
            .map(|partition| {
                let start = subscription
                    .start_offsets
                    .get(&partition)
                    .copied()
                    .unwrap_or(0);
                (partition, start)
            })
            .collect();
        Self {
            subscription,
            committed_offsets,
            attempts: BTreeMap::new(),
            in_flight: BTreeMap::new(),
            leased_events: BTreeSet::new(),
            dead_letters: Vec::new(),
            next_delivery: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopicState {
    config: TopicConfig,
    partitions: BTreeMap<u16, Vec<EventEnvelope>>,
    base_offsets: BTreeMap<u16, u64>,
    next_offsets: BTreeMap<u16, u64>,
}

impl TopicState {
    fn new(config: TopicConfig) -> Self {
        let partitions = (0..config.partitions())
            .map(|partition| (partition, Vec::new()))
            .collect();
        let base_offsets = (0..config.partitions())
            .map(|partition| (partition, 0))
            .collect();
        let next_offsets = (0..config.partitions())
            .map(|partition| (partition, 0))
            .collect();
        Self {
            config,
            partitions,
            base_offsets,
            next_offsets,
        }
    }

    fn event(&self, partition: u16, offset: u64) -> Option<&EventEnvelope> {
        let base = *self.base_offsets.get(&partition)?;
        let index = offset.checked_sub(base)? as usize;
        self.partitions.get(&partition)?.get(index)
    }

    fn append(&mut self, request: PublishRequest) -> EventEnvelope {
        let partition = partition_for(request.key(), self.config.partitions());
        let offset = self.next_offsets[&partition];
        let envelope = EventEnvelope::manufacture(
            request.id,
            request.topic,
            partition,
            offset,
            request.logical_time,
            request.key,
            request.payload,
            request.headers,
        );
        self.partitions
            .get_mut(&partition)
            .expect("configured partition")
            .push(envelope.clone());
        *self
            .next_offsets
            .get_mut(&partition)
            .expect("configured partition") += 1;
        let partition_log = self
            .partitions
            .get_mut(&partition)
            .expect("configured partition");
        while partition_log.len() > self.config.retention_events_per_partition() {
            partition_log.remove(0);
            *self
                .base_offsets
                .get_mut(&partition)
                .expect("configured partition") += 1;
        }
        envelope
    }
}

/// Audited event-bus action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventBusAction {
    TopicCreated { topic: TopicId, config: TopicConfig },
    Published { event: EventEnvelope },
    SubscriptionCreated { subscription: Subscription },
    Delivered { delivery: Delivery },
    Acknowledged { delivery: DeliveryId, logical_time: u64 },
    Rejected {
        delivery: DeliveryId,
        logical_time: u64,
        reason: String,
    },
    DeadLettered {
        subscription: SubscriptionId,
        event: EventEnvelope,
        attempts: u32,
        logical_time: u64,
    },
}

impl EventBusAction {
    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::TopicCreated { topic, config } => {
                encoder
                    .text("action", "topic-created")
                    .text("topic", topic.as_str())
                    .field("config", &config.digest().0);
            }
            Self::Published { event } => {
                encoder
                    .text("action", "published")
                    .field("event", &event.digest().0);
            }
            Self::SubscriptionCreated { subscription } => {
                encoder
                    .text("action", "subscription-created")
                    .text("subscription", subscription.id().as_str())
                    .text("topic", subscription.topic().as_str())
                    .text("consumer", subscription.consumer().as_str());
            }
            Self::Delivered { delivery } => {
                encoder
                    .text("action", "delivered")
                    .field("delivery", &delivery.digest().0);
            }
            Self::Acknowledged {
                delivery,
                logical_time,
            } => {
                encoder
                    .text("action", "acknowledged")
                    .text("delivery", delivery.as_str())
                    .u64("logical-time", *logical_time);
            }
            Self::Rejected {
                delivery,
                logical_time,
                reason,
            } => {
                encoder
                    .text("action", "rejected")
                    .text("delivery", delivery.as_str())
                    .u64("logical-time", *logical_time)
                    .text("reason", reason);
            }
            Self::DeadLettered {
                subscription,
                event,
                attempts,
                logical_time,
            } => {
                encoder
                    .text("action", "dead-lettered")
                    .text("subscription", subscription.as_str())
                    .field("event", &event.digest().0)
                    .u64("attempts", u64::from(*attempts))
                    .u64("logical-time", *logical_time);
            }
        }
    }
}

/// Immutable event-bus state transition receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventBusReceipt {
    index: u64,
    previous: Digest,
    action: EventBusAction,
    state_digest: Digest,
    digest: Digest,
}

impl EventBusReceipt {
    fn manufacture(
        index: u64,
        previous: Digest,
        action: EventBusAction,
        state_digest: Digest,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "event-bus-receipt-v1")
            .u64("index", index)
            .field("previous", &previous.0);
        action.encode(&mut encoder);
        encoder.field("state", &state_digest.0);
        Self {
            index,
            previous,
            action,
            state_digest,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub const fn index(&self) -> u64 {
        self.index
    }

    #[must_use]
    pub const fn previous(&self) -> Digest {
        self.previous
    }

    #[must_use]
    pub const fn action(&self) -> &EventBusAction {
        &self.action
    }

    #[must_use]
    pub const fn state_digest(&self) -> Digest {
        self.state_digest
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Event transport failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventBusError {
    EmptyIdentifier(&'static str),
    ZeroPartitions,
    ZeroRetention,
    ZeroVisibilityTimeout,
    ZeroDeliveryAttempts,
    DuplicateTopic(TopicId),
    UnknownTopic(TopicId),
    DuplicateSubscription(SubscriptionId),
    UnknownSubscription(SubscriptionId),
    EventIdentityConflict(EventId),
    LogicalTimeRegression { previous: u64, actual: u64 },
    DeliveryMissing(DeliveryId),
    DeliveryConsumerMismatch {
        delivery: DeliveryId,
        expected: ConsumerId,
        actual: ConsumerId,
    },
    DeliveryExpired {
        delivery: DeliveryId,
        leased_until: u64,
        logical_time: u64,
    },
}

impl Display for EventBusError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::ZeroPartitions => formatter.write_str("topic requires at least one partition"),
            Self::ZeroRetention => formatter.write_str("topic retention must be positive"),
            Self::ZeroVisibilityTimeout => {
                formatter.write_str("visibility timeout must be positive")
            }
            Self::ZeroDeliveryAttempts => {
                formatter.write_str("maximum delivery attempts must be positive")
            }
            Self::DuplicateTopic(topic) => write!(formatter, "topic `{topic}` already exists"),
            Self::UnknownTopic(topic) => write!(formatter, "topic `{topic}` is unknown"),
            Self::DuplicateSubscription(id) => {
                write!(formatter, "subscription `{id}` already exists")
            }
            Self::UnknownSubscription(id) => write!(formatter, "subscription `{id}` is unknown"),
            Self::EventIdentityConflict(id) => {
                write!(formatter, "event id `{id}` was reused with different content")
            }
            Self::LogicalTimeRegression { previous, actual } => write!(
                formatter,
                "event-bus logical time regressed from {previous} to {actual}"
            ),
            Self::DeliveryMissing(id) => write!(formatter, "delivery `{id}` is missing"),
            Self::DeliveryConsumerMismatch {
                delivery,
                expected,
                actual,
            } => write!(
                formatter,
                "delivery `{delivery}` belongs to `{expected}`, not `{actual}`"
            ),
            Self::DeliveryExpired {
                delivery,
                leased_until,
                logical_time,
            } => write!(
                formatter,
                "delivery `{delivery}` expired at {leased_until}, time is {logical_time}"
            ),
        }
    }
}

impl std::error::Error for EventBusError {}

/// Durable in-memory event bus with deterministic state and evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventBus {
    topics: BTreeMap<TopicId, TopicState>,
    events: BTreeMap<EventId, (Digest, EventEnvelope)>,
    subscriptions: BTreeMap<SubscriptionId, SubscriptionState>,
    receipts: Vec<EventBusReceipt>,
    logical_time: u64,
}

impl EventBus {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_topic(
        &mut self,
        topic: TopicId,
        config: TopicConfig,
    ) -> Result<&EventBusReceipt, EventBusError> {
        if self.topics.contains_key(&topic) {
            return Err(EventBusError::DuplicateTopic(topic));
        }
        self.topics
            .insert(topic.clone(), TopicState::new(config.clone()));
        self.append_action(EventBusAction::TopicCreated { topic, config })
    }

    /// Publishes exactly once by event id and request digest.
    pub fn publish(
        &mut self,
        request: PublishRequest,
    ) -> Result<&EventEnvelope, EventBusError> {
        self.advance_time(request.logical_time())?;
        if let Some(existing_digest) = self.events.get(request.id()).map(|(digest, _)| *digest) {
            if existing_digest == request.digest() {
                return Ok(&self.events[request.id()].1);
            }
            return Err(EventBusError::EventIdentityConflict(request.id().clone()));
        }
        let topic = self
            .topics
            .get_mut(request.topic())
            .ok_or_else(|| EventBusError::UnknownTopic(request.topic().clone()))?;
        let request_digest = request.digest();
        let envelope = topic.append(request);
        let id = envelope.id().clone();
        self.events
            .insert(id.clone(), (request_digest, envelope.clone()));
        self.append_action(EventBusAction::Published {
            event: envelope.clone(),
        })?;
        Ok(&self.events[&id].1)
    }

    pub fn subscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<&EventBusReceipt, EventBusError> {
        if self.subscriptions.contains_key(subscription.id()) {
            return Err(EventBusError::DuplicateSubscription(
                subscription.id().clone(),
            ));
        }
        let topic = self
            .topics
            .get(subscription.topic())
            .ok_or_else(|| EventBusError::UnknownTopic(subscription.topic().clone()))?;
        let id = subscription.id().clone();
        self.subscriptions.insert(
            id,
            SubscriptionState::new(subscription.clone(), topic.config.partitions()),
        );
        self.append_action(EventBusAction::SubscriptionCreated { subscription })
    }

    /// Leases available events in partition/offset order.
    pub fn poll(
        &mut self,
        subscription: &SubscriptionId,
        consumer: &ConsumerId,
        logical_time: u64,
        maximum: usize,
    ) -> Result<Vec<Delivery>, EventBusError> {
        self.advance_time(logical_time)?;
        self.expire_leases(subscription, logical_time)?;
        let topic_id = self
            .subscriptions
            .get(subscription)
            .ok_or_else(|| EventBusError::UnknownSubscription(subscription.clone()))?
            .subscription
            .topic()
            .clone();
        let topic = self
            .topics
            .get(&topic_id)
            .ok_or_else(|| EventBusError::UnknownTopic(topic_id.clone()))?
            .clone();
        let config = topic.config.clone();
        let mut candidates = Vec::new();
        {
            let state = self
                .subscriptions
                .get(subscription)
                .expect("validated subscription");
            for partition in 0..config.partitions() {
                let base = topic.base_offsets[&partition];
                let committed = state.committed_offsets[&partition].max(base);
                let next = topic.next_offsets[&partition];
                for offset in committed..next {
                    if !state.leased_events.contains(&(partition, offset)) {
                        if let Some(event) = topic.event(partition, offset) {
                            candidates.push(event.clone());
                        }
                    }
                }
            }
        }
        candidates.sort_by_key(|event| (event.logical_time(), event.partition(), event.offset()));
        candidates.truncate(maximum);

        let mut deliveries = Vec::with_capacity(candidates.len());
        for event in candidates {
            let key = (event.partition(), event.offset());
            let state = self
                .subscriptions
                .get_mut(subscription)
                .expect("validated subscription");
            let attempt = state.attempts.get(&key).copied().unwrap_or(0) + 1;
            state.attempts.insert(key, attempt);
            if attempt > config.maximum_delivery_attempts() {
                state.dead_letters.push(event.clone());
                advance_committed_offset(state, event.partition(), event.offset());
                self.append_action(EventBusAction::DeadLettered {
                    subscription: subscription.clone(),
                    event,
                    attempts: attempt - 1,
                    logical_time,
                })?;
                continue;
            }
            let id = DeliveryId::new(format!(
                "{}:{}",
                subscription.as_str(),
                state.next_delivery
            ))?;
            state.next_delivery += 1;
            let delivery = Delivery::manufacture(
                id.clone(),
                subscription.clone(),
                consumer.clone(),
                event,
                attempt,
                logical_time.saturating_add(config.visibility_timeout()),
            );
            state.leased_events.insert(key);
            state.in_flight.insert(
                id,
                InFlight {
                    delivery: delivery.clone(),
                },
            );
            self.append_action(EventBusAction::Delivered {
                delivery: delivery.clone(),
            })?;
            deliveries.push(delivery);
        }
        Ok(deliveries)
    }

    pub fn acknowledge(
        &mut self,
        subscription: &SubscriptionId,
        delivery: &DeliveryId,
        consumer: &ConsumerId,
        logical_time: u64,
    ) -> Result<&EventBusReceipt, EventBusError> {
        self.advance_time(logical_time)?;
        let state = self
            .subscriptions
            .get_mut(subscription)
            .ok_or_else(|| EventBusError::UnknownSubscription(subscription.clone()))?;
        let in_flight = state
            .in_flight
            .remove(delivery)
            .ok_or_else(|| EventBusError::DeliveryMissing(delivery.clone()))?;
        if in_flight.delivery.consumer() != consumer {
            state
                .in_flight
                .insert(delivery.clone(), in_flight.clone());
            return Err(EventBusError::DeliveryConsumerMismatch {
                delivery: delivery.clone(),
                expected: in_flight.delivery.consumer().clone(),
                actual: consumer.clone(),
            });
        }
        if logical_time > in_flight.delivery.leased_until() {
            state.in_flight.insert(delivery.clone(), in_flight.clone());
            return Err(EventBusError::DeliveryExpired {
                delivery: delivery.clone(),
                leased_until: in_flight.delivery.leased_until(),
                logical_time,
            });
        }
        let event = in_flight.delivery.event();
        state
            .leased_events
            .remove(&(event.partition(), event.offset()));
        advance_committed_offset(state, event.partition(), event.offset());
        self.append_action(EventBusAction::Acknowledged {
            delivery: delivery.clone(),
            logical_time,
        })
    }

    pub fn reject(
        &mut self,
        subscription: &SubscriptionId,
        delivery: &DeliveryId,
        consumer: &ConsumerId,
        logical_time: u64,
        reason: impl Into<String>,
    ) -> Result<&EventBusReceipt, EventBusError> {
        self.advance_time(logical_time)?;
        let state = self
            .subscriptions
            .get_mut(subscription)
            .ok_or_else(|| EventBusError::UnknownSubscription(subscription.clone()))?;
        let in_flight = state
            .in_flight
            .remove(delivery)
            .ok_or_else(|| EventBusError::DeliveryMissing(delivery.clone()))?;
        if in_flight.delivery.consumer() != consumer {
            state
                .in_flight
                .insert(delivery.clone(), in_flight.clone());
            return Err(EventBusError::DeliveryConsumerMismatch {
                delivery: delivery.clone(),
                expected: in_flight.delivery.consumer().clone(),
                actual: consumer.clone(),
            });
        }
        let event = in_flight.delivery.event();
        state
            .leased_events
            .remove(&(event.partition(), event.offset()));
        self.append_action(EventBusAction::Rejected {
            delivery: delivery.clone(),
            logical_time,
            reason: reason.into(),
        })
    }

    #[must_use]
    pub fn dead_letters(
        &self,
        subscription: &SubscriptionId,
    ) -> Result<&[EventEnvelope], EventBusError> {
        self.subscriptions
            .get(subscription)
            .map(|state| state.dead_letters.as_slice())
            .ok_or_else(|| EventBusError::UnknownSubscription(subscription.clone()))
    }

    #[must_use]
    pub fn receipts(&self) -> &[EventBusReceipt] {
        &self.receipts
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts.last().map_or(Digest::ZERO, EventBusReceipt::digest)
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn state_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "event-bus-state-v1")
            .u64("topic-count", self.topics.len() as u64);
        for (topic_id, topic) in &self.topics {
            encoder
                .text("topic", topic_id.as_str())
                .field("config", &topic.config.digest().0);
            for partition in 0..topic.config.partitions() {
                encoder
                    .u64("partition", u64::from(partition))
                    .u64("base-offset", topic.base_offsets[&partition])
                    .u64("next-offset", topic.next_offsets[&partition]);
                for event in &topic.partitions[&partition] {
                    encoder.field("event", &event.digest().0);
                }
            }
        }
        encoder.u64("subscription-count", self.subscriptions.len() as u64);
        for (id, state) in &self.subscriptions {
            encoder
                .text("subscription", id.as_str())
                .text("topic", state.subscription.topic().as_str())
                .text("consumer", state.subscription.consumer().as_str());
            for (partition, offset) in &state.committed_offsets {
                encoder
                    .u64("partition", u64::from(*partition))
                    .u64("committed-offset", *offset);
            }
            encoder.u64("in-flight-count", state.in_flight.len() as u64);
            for delivery in state.in_flight.values() {
                encoder.field("delivery", &delivery.delivery.digest().0);
            }
            encoder.u64("dead-letter-count", state.dead_letters.len() as u64);
            for event in &state.dead_letters {
                encoder.field("dead-letter", &event.digest().0);
            }
        }
        encoder.digest()
    }

    fn advance_time(&mut self, logical_time: u64) -> Result<(), EventBusError> {
        if logical_time < self.logical_time {
            return Err(EventBusError::LogicalTimeRegression {
                previous: self.logical_time,
                actual: logical_time,
            });
        }
        self.logical_time = logical_time;
        Ok(())
    }

    fn expire_leases(
        &mut self,
        subscription: &SubscriptionId,
        logical_time: u64,
    ) -> Result<(), EventBusError> {
        let state = self
            .subscriptions
            .get_mut(subscription)
            .ok_or_else(|| EventBusError::UnknownSubscription(subscription.clone()))?;
        let expired = state
            .in_flight
            .iter()
            .filter(|(_, in_flight)| in_flight.delivery.leased_until() <= logical_time)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in expired {
            if let Some(in_flight) = state.in_flight.remove(&id) {
                let event = in_flight.delivery.event();
                state
                    .leased_events
                    .remove(&(event.partition(), event.offset()));
            }
        }
        Ok(())
    }

    fn append_action(
        &mut self,
        action: EventBusAction,
    ) -> Result<&EventBusReceipt, EventBusError> {
        let receipt = EventBusReceipt::manufacture(
            self.receipts.len() as u64,
            self.head(),
            action,
            self.state_digest(),
        );
        self.receipts.push(receipt);
        Ok(self.receipts.last().expect("just appended receipt"))
    }
}

fn partition_for(key: &[u8], partitions: u16) -> u16 {
    let digest = sha256(key);
    let number = u64::from_be_bytes(digest.0[..8].try_into().expect("eight-byte digest prefix"));
    (number % u64::from(partitions)) as u16
}

fn advance_committed_offset(state: &mut SubscriptionState, partition: u16, offset: u64) {
    let committed = state
        .committed_offsets
        .get_mut(&partition)
        .expect("configured partition");
    if offset == *committed {
        *committed += 1;
        while state.attempts.remove(&(partition, *committed)).is_some()
            && !state.leased_events.contains(&(partition, *committed))
        {
            *committed += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConsumerId, EventBus, EventBusError, EventId, PublishRequest, Subscription,
        SubscriptionId, TopicConfig, TopicId,
    };

    fn bus(maximum_attempts: u32) -> EventBus {
        let mut bus = EventBus::new();
        bus.create_topic(
            TopicId::new("events").unwrap(),
            TopicConfig::new(2, 100, 5, maximum_attempts).unwrap(),
        )
        .unwrap();
        bus.subscribe(Subscription::new(
            SubscriptionId::new("worker").unwrap(),
            TopicId::new("events").unwrap(),
            ConsumerId::new("consumer").unwrap(),
        ))
        .unwrap();
        bus
    }

    #[test]
    fn publish_is_exactly_once_by_identity_and_content() {
        let mut bus = bus(3);
        let request = PublishRequest::new(
            EventId::new("event-1").unwrap(),
            TopicId::new("events").unwrap(),
            1,
            b"case-1".to_vec(),
            b"payload".to_vec(),
        );
        let first = bus.publish(request.clone()).unwrap().digest();
        let duplicate = bus.publish(request).unwrap().digest();
        assert_eq!(first, duplicate);
        assert_eq!(
            bus.receipts()
                .iter()
                .filter(|receipt| matches!(receipt.action(), super::EventBusAction::Published { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn event_id_content_conflict_is_refused() {
        let mut bus = bus(3);
        bus.publish(PublishRequest::new(
            EventId::new("event-1").unwrap(),
            TopicId::new("events").unwrap(),
            1,
            b"case-1".to_vec(),
            b"one".to_vec(),
        ))
        .unwrap();
        assert!(matches!(
            bus.publish(PublishRequest::new(
                EventId::new("event-1").unwrap(),
                TopicId::new("events").unwrap(),
                2,
                b"case-1".to_vec(),
                b"two".to_vec(),
            )),
            Err(EventBusError::EventIdentityConflict(_))
        ));
    }

    #[test]
    fn acknowledgement_advances_offset() {
        let mut bus = bus(3);
        bus.publish(PublishRequest::new(
            EventId::new("event-1").unwrap(),
            TopicId::new("events").unwrap(),
            1,
            b"case-1".to_vec(),
            b"payload".to_vec(),
        ))
        .unwrap();
        let subscription = SubscriptionId::new("worker").unwrap();
        let consumer = ConsumerId::new("consumer").unwrap();
        let delivery = bus.poll(&subscription, &consumer, 2, 10).unwrap().remove(0);
        bus.acknowledge(&subscription, delivery.id(), &consumer, 3)
            .unwrap();
        assert!(bus.poll(&subscription, &consumer, 4, 10).unwrap().is_empty());
    }

    #[test]
    fn lease_expiry_redelivers_with_incremented_attempt() {
        let mut bus = bus(3);
        bus.publish(PublishRequest::new(
            EventId::new("event-1").unwrap(),
            TopicId::new("events").unwrap(),
            1,
            b"case-1".to_vec(),
            b"payload".to_vec(),
        ))
        .unwrap();
        let subscription = SubscriptionId::new("worker").unwrap();
        let consumer = ConsumerId::new("consumer").unwrap();
        let first = bus.poll(&subscription, &consumer, 2, 10).unwrap().remove(0);
        let second = bus.poll(&subscription, &consumer, 7, 10).unwrap().remove(0);
        assert_eq!(first.event().digest(), second.event().digest());
        assert_eq!(second.attempt(), 2);
    }

    #[test]
    fn repeated_rejection_dead_letters_event() {
        let mut bus = bus(2);
        bus.publish(PublishRequest::new(
            EventId::new("event-1").unwrap(),
            TopicId::new("events").unwrap(),
            1,
            b"case-1".to_vec(),
            b"payload".to_vec(),
        ))
        .unwrap();
        let subscription = SubscriptionId::new("worker").unwrap();
        let consumer = ConsumerId::new("consumer").unwrap();
        for logical_time in [2, 3] {
            let delivery = bus
                .poll(&subscription, &consumer, logical_time, 10)
                .unwrap()
                .remove(0);
            bus.reject(
                &subscription,
                delivery.id(),
                &consumer,
                logical_time,
                "failed",
            )
            .unwrap();
        }
        assert!(bus.poll(&subscription, &consumer, 4, 10).unwrap().is_empty());
        assert_eq!(bus.dead_letters(&subscription).unwrap().len(), 1);
    }
}
