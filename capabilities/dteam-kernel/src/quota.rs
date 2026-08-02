//! Atomic multi-resource quota reservations with refill, leases, and replay receipts.

use crate::hash::{CanonicalEncoder, Digest};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

macro_rules! quota_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, QuotaError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(QuotaError::EmptyIdentifier(stringify!($name)));
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

quota_id!(PrincipalId);
quota_id!(ResourceId);
quota_id!(ReservationId);

/// Token-bucket policy for one principal and resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaPolicy {
    capacity: u64,
    refill_amount: u64,
    refill_interval: u64,
    maximum_reservation: u64,
    lease_duration: u64,
}

impl QuotaPolicy {
    pub fn new(
        capacity: u64,
        refill_amount: u64,
        refill_interval: u64,
        maximum_reservation: u64,
        lease_duration: u64,
    ) -> Result<Self, QuotaError> {
        if capacity == 0 {
            return Err(QuotaError::ZeroCapacity);
        }
        if refill_amount > 0 && refill_interval == 0 {
            return Err(QuotaError::ZeroRefillInterval);
        }
        if maximum_reservation == 0 || maximum_reservation > capacity {
            return Err(QuotaError::InvalidMaximumReservation {
                maximum: maximum_reservation,
                capacity,
            });
        }
        if lease_duration == 0 {
            return Err(QuotaError::ZeroLeaseDuration);
        }
        Ok(Self {
            capacity,
            refill_amount,
            refill_interval,
            maximum_reservation,
            lease_duration,
        })
    }

    #[must_use]
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    #[must_use]
    pub const fn refill_amount(&self) -> u64 {
        self.refill_amount
    }

    #[must_use]
    pub const fn refill_interval(&self) -> u64 {
        self.refill_interval
    }

    #[must_use]
    pub const fn maximum_reservation(&self) -> u64 {
        self.maximum_reservation
    }

    #[must_use]
    pub const fn lease_duration(&self) -> u64 {
        self.lease_duration
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "quota-policy-v1")
            .u64("capacity", self.capacity)
            .u64("refill-amount", self.refill_amount)
            .u64("refill-interval", self.refill_interval)
            .u64("maximum-reservation", self.maximum_reservation)
            .u64("lease-duration", self.lease_duration);
        encoder.digest()
    }
}

/// One atomic resource claim.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QuotaClaim {
    principal: PrincipalId,
    resource: ResourceId,
    amount: u64,
}

impl QuotaClaim {
    pub fn new(
        principal: PrincipalId,
        resource: ResourceId,
        amount: u64,
    ) -> Result<Self, QuotaError> {
        if amount == 0 {
            return Err(QuotaError::ZeroClaim);
        }
        Ok(Self {
            principal,
            resource,
            amount,
        })
    }

    #[must_use]
    pub const fn principal(&self) -> &PrincipalId {
        &self.principal
    }

    #[must_use]
    pub const fn resource(&self) -> &ResourceId {
        &self.resource
    }

    #[must_use]
    pub const fn amount(&self) -> u64 {
        self.amount
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        encoder
            .text("principal", self.principal.as_str())
            .text("resource", self.resource.as_str())
            .u64("amount", self.amount);
    }
}

/// Immutable reservation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationRequest {
    id: ReservationId,
    logical_time: u64,
    claims: Vec<QuotaClaim>,
    digest: Digest,
}

impl ReservationRequest {
    pub fn new(
        id: ReservationId,
        logical_time: u64,
        mut claims: Vec<QuotaClaim>,
    ) -> Result<Self, QuotaError> {
        if claims.is_empty() {
            return Err(QuotaError::EmptyClaims);
        }
        claims.sort();
        let mut pairs = BTreeSet::new();
        for claim in &claims {
            let pair = (claim.principal().clone(), claim.resource().clone());
            if !pairs.insert(pair.clone()) {
                return Err(QuotaError::DuplicateClaim {
                    principal: pair.0,
                    resource: pair.1,
                });
            }
        }
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "reservation-request-v1")
            .text("id", id.as_str())
            .u64("logical-time", logical_time)
            .u64("claim-count", claims.len() as u64);
        for claim in &claims {
            claim.encode(&mut encoder);
        }
        Ok(Self {
            id,
            logical_time,
            claims,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> &ReservationId {
        &self.id
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn claims(&self) -> &[QuotaClaim] {
        &self.claims
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Reservation lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationState {
    Pending,
    Committed,
    Released,
    Expired,
}

impl ReservationState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Committed => "committed",
            Self::Released => "released",
            Self::Expired => "expired",
        }
    }
}

/// Active or completed reservation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reservation {
    request: ReservationRequest,
    expires_at: u64,
    state: ReservationState,
    digest: Digest,
}

impl Reservation {
    fn new(request: ReservationRequest, expires_at: u64) -> Self {
        let mut reservation = Self {
            request,
            expires_at,
            state: ReservationState::Pending,
            digest: Digest::ZERO,
        };
        reservation.digest = reservation.recompute_digest();
        reservation
    }

    #[must_use]
    pub const fn id(&self) -> &ReservationId {
        self.request.id()
    }

    #[must_use]
    pub const fn request(&self) -> &ReservationRequest {
        &self.request
    }

    #[must_use]
    pub const fn expires_at(&self) -> u64 {
        self.expires_at
    }

    #[must_use]
    pub const fn state(&self) -> ReservationState {
        self.state
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    fn set_state(&mut self, state: ReservationState) {
        self.state = state;
        self.digest = self.recompute_digest();
    }

    fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "reservation-v1")
            .field("request", &self.request.digest().0)
            .u64("expires-at", self.expires_at)
            .text("state", self.state.as_str());
        encoder.digest()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Bucket {
    available: u64,
    last_refill: u64,
}

impl Bucket {
    fn full(policy: &QuotaPolicy, logical_time: u64) -> Self {
        Self {
            available: policy.capacity(),
            last_refill: logical_time,
        }
    }

    fn refill(&mut self, policy: &QuotaPolicy, logical_time: u64) {
        if logical_time <= self.last_refill
            || policy.refill_amount() == 0
            || policy.refill_interval() == 0
        {
            return;
        }
        let intervals = (logical_time - self.last_refill) / policy.refill_interval();
        if intervals == 0 {
            return;
        }
        let refill = intervals.saturating_mul(policy.refill_amount());
        self.available = self.available.saturating_add(refill).min(policy.capacity());
        self.last_refill = self
            .last_refill
            .saturating_add(intervals.saturating_mul(policy.refill_interval()));
    }

    fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "quota-bucket-v1")
            .u64("available", self.available)
            .u64("last-refill", self.last_refill);
        encoder.digest()
    }
}

/// Quota operation recorded in the audit chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaAction {
    PolicySet {
        principal: PrincipalId,
        resource: ResourceId,
        policy: QuotaPolicy,
        logical_time: u64,
    },
    Reserved { request: ReservationRequest },
    Committed {
        reservation: ReservationId,
        logical_time: u64,
    },
    Released {
        reservation: ReservationId,
        logical_time: u64,
    },
    Expired {
        reservation: ReservationId,
        logical_time: u64,
    },
}

impl QuotaAction {
    fn logical_time(&self) -> u64 {
        match self {
            Self::PolicySet { logical_time, .. }
            | Self::Committed { logical_time, .. }
            | Self::Released { logical_time, .. }
            | Self::Expired { logical_time, .. } => *logical_time,
            Self::Reserved { request } => request.logical_time(),
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::PolicySet {
                principal,
                resource,
                policy,
                logical_time,
            } => {
                encoder
                    .text("action", "policy-set")
                    .text("principal", principal.as_str())
                    .text("resource", resource.as_str())
                    .field("policy", &policy.digest().0)
                    .u64("logical-time", *logical_time);
            }
            Self::Reserved { request } => {
                encoder
                    .text("action", "reserved")
                    .field("request", &request.digest().0);
            }
            Self::Committed {
                reservation,
                logical_time,
            } => {
                encoder
                    .text("action", "committed")
                    .text("reservation", reservation.as_str())
                    .u64("logical-time", *logical_time);
            }
            Self::Released {
                reservation,
                logical_time,
            } => {
                encoder
                    .text("action", "released")
                    .text("reservation", reservation.as_str())
                    .u64("logical-time", *logical_time);
            }
            Self::Expired {
                reservation,
                logical_time,
            } => {
                encoder
                    .text("action", "expired")
                    .text("reservation", reservation.as_str())
                    .u64("logical-time", *logical_time);
            }
        }
    }
}

/// Immutable quota-state transition receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaReceipt {
    index: u64,
    previous: Digest,
    action: QuotaAction,
    state_digest: Digest,
    digest: Digest,
}

impl QuotaReceipt {
    fn manufacture(
        index: u64,
        previous: Digest,
        action: QuotaAction,
        state_digest: Digest,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "quota-receipt-v1")
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
    pub const fn action(&self) -> &QuotaAction {
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

/// Quota operation refusal or replay failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaError {
    EmptyIdentifier(&'static str),
    ZeroCapacity,
    ZeroRefillInterval,
    InvalidMaximumReservation { maximum: u64, capacity: u64 },
    ZeroLeaseDuration,
    ZeroClaim,
    EmptyClaims,
    DuplicateClaim {
        principal: PrincipalId,
        resource: ResourceId,
    },
    PolicyMissing {
        principal: PrincipalId,
        resource: ResourceId,
    },
    ClaimTooLarge {
        principal: PrincipalId,
        resource: ResourceId,
        requested: u64,
        maximum: u64,
    },
    InsufficientQuota {
        principal: PrincipalId,
        resource: ResourceId,
        requested: u64,
        available: u64,
    },
    DuplicateReservation(ReservationId),
    ReservationMissing(ReservationId),
    ReservationNotPending {
        reservation: ReservationId,
        state: ReservationState,
    },
    ReservationExpired {
        reservation: ReservationId,
        expires_at: u64,
        logical_time: u64,
    },
    LogicalTimeRegression { previous: u64, actual: u64 },
    ReceiptIndex { expected: u64, actual: u64 },
    ReceiptPrevious { expected: Digest, actual: Digest },
    ReceiptState { index: u64, expected: Digest, actual: Digest },
    ReceiptDigest { index: u64 },
}

impl Display for QuotaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::ZeroCapacity => formatter.write_str("quota capacity must be positive"),
            Self::ZeroRefillInterval => {
                formatter.write_str("positive refill requires a positive interval")
            }
            Self::InvalidMaximumReservation { maximum, capacity } => write!(
                formatter,
                "maximum reservation {maximum} must be between 1 and capacity {capacity}"
            ),
            Self::ZeroLeaseDuration => formatter.write_str("lease duration must be positive"),
            Self::ZeroClaim => formatter.write_str("quota claim amount must be positive"),
            Self::EmptyClaims => formatter.write_str("reservation must contain claims"),
            Self::DuplicateClaim {
                principal,
                resource,
            } => write!(
                formatter,
                "reservation contains duplicate `{principal}`/`{resource}` claim"
            ),
            Self::PolicyMissing {
                principal,
                resource,
            } => write!(formatter, "quota policy missing for `{principal}`/`{resource}`"),
            Self::ClaimTooLarge {
                principal,
                resource,
                requested,
                maximum,
            } => write!(
                formatter,
                "claim {requested} for `{principal}`/`{resource}` exceeds maximum {maximum}"
            ),
            Self::InsufficientQuota {
                principal,
                resource,
                requested,
                available,
            } => write!(
                formatter,
                "claim {requested} for `{principal}`/`{resource}` exceeds available {available}"
            ),
            Self::DuplicateReservation(id) => write!(formatter, "reservation `{id}` already exists"),
            Self::ReservationMissing(id) => write!(formatter, "reservation `{id}` is missing"),
            Self::ReservationNotPending { reservation, state } => write!(
                formatter,
                "reservation `{reservation}` is {}, expected pending",
                state.as_str()
            ),
            Self::ReservationExpired {
                reservation,
                expires_at,
                logical_time,
            } => write!(
                formatter,
                "reservation `{reservation}` expired at {expires_at}, logical time is {logical_time}"
            ),
            Self::LogicalTimeRegression { previous, actual } => write!(
                formatter,
                "quota logical time regressed from {previous} to {actual}"
            ),
            Self::ReceiptIndex { expected, actual } => {
                write!(formatter, "quota receipt index {actual}, expected {expected}")
            }
            Self::ReceiptPrevious { expected, actual } => {
                write!(formatter, "quota predecessor {actual}, expected {expected}")
            }
            Self::ReceiptState {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "quota receipt {index} state {actual}, expected {expected}"
            ),
            Self::ReceiptDigest { index } => {
                write!(formatter, "quota receipt {index} digest mismatch")
            }
        }
    }
}

impl std::error::Error for QuotaError {}

/// Atomic quota manager.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuotaManager {
    policies: BTreeMap<(PrincipalId, ResourceId), QuotaPolicy>,
    buckets: BTreeMap<(PrincipalId, ResourceId), Bucket>,
    reservations: BTreeMap<ReservationId, Reservation>,
    receipts: Vec<QuotaReceipt>,
    logical_time: u64,
}

impl QuotaManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs or replaces a policy and caps existing availability to new capacity.
    pub fn set_policy(
        &mut self,
        principal: PrincipalId,
        resource: ResourceId,
        policy: QuotaPolicy,
        logical_time: u64,
    ) -> Result<&QuotaReceipt, QuotaError> {
        self.advance_checked(logical_time)?;
        let key = (principal.clone(), resource.clone());
        self.policies.insert(key.clone(), policy.clone());
        self.buckets
            .entry(key)
            .and_modify(|bucket| bucket.available = bucket.available.min(policy.capacity()))
            .or_insert_with(|| Bucket::full(&policy, logical_time));
        self.append_action(QuotaAction::PolicySet {
            principal,
            resource,
            policy,
            logical_time,
        })
    }

    /// Returns current available quota after logical refill.
    pub fn available(
        &mut self,
        principal: &PrincipalId,
        resource: &ResourceId,
        logical_time: u64,
    ) -> Result<u64, QuotaError> {
        self.advance_checked(logical_time)?;
        let key = (principal.clone(), resource.clone());
        self.refill_key(&key, logical_time)?;
        Ok(self.buckets[&key].available)
    }

    /// Atomically reserves every claim or changes nothing.
    pub fn reserve(
        &mut self,
        request: ReservationRequest,
    ) -> Result<&QuotaReceipt, QuotaError> {
        if self.reservations.contains_key(request.id()) {
            let existing = &self.reservations[request.id()];
            if existing.request().digest() == request.digest() {
                return self
                    .receipts
                    .iter()
                    .rev()
                    .find(|receipt| matches!(
                        receipt.action(),
                        QuotaAction::Reserved { request: recorded }
                            if recorded.digest() == request.digest()
                    ))
                    .ok_or_else(|| QuotaError::ReservationMissing(request.id().clone()));
            }
            return Err(QuotaError::DuplicateReservation(request.id().clone()));
        }
        self.advance_checked(request.logical_time())?;
        let mut candidate_buckets = self.buckets.clone();
        let mut lease_duration = u64::MAX;
        for claim in request.claims() {
            let key = (claim.principal().clone(), claim.resource().clone());
            let policy = self
                .policies
                .get(&key)
                .ok_or_else(|| QuotaError::PolicyMissing {
                    principal: claim.principal().clone(),
                    resource: claim.resource().clone(),
                })?;
            lease_duration = lease_duration.min(policy.lease_duration());
            if claim.amount() > policy.maximum_reservation() {
                return Err(QuotaError::ClaimTooLarge {
                    principal: claim.principal().clone(),
                    resource: claim.resource().clone(),
                    requested: claim.amount(),
                    maximum: policy.maximum_reservation(),
                });
            }
            let bucket = candidate_buckets
                .entry(key.clone())
                .or_insert_with(|| Bucket::full(policy, request.logical_time()));
            bucket.refill(policy, request.logical_time());
            if bucket.available < claim.amount() {
                return Err(QuotaError::InsufficientQuota {
                    principal: claim.principal().clone(),
                    resource: claim.resource().clone(),
                    requested: claim.amount(),
                    available: bucket.available,
                });
            }
            bucket.available -= claim.amount();
        }
        let expires_at = request.logical_time().saturating_add(lease_duration);
        let reservation = Reservation::new(request.clone(), expires_at);
        self.buckets = candidate_buckets;
        self.reservations.insert(request.id().clone(), reservation);
        self.append_action(QuotaAction::Reserved { request })
    }

    /// Commits a pending reservation, permanently consuming its quota.
    pub fn commit(
        &mut self,
        reservation: &ReservationId,
        logical_time: u64,
    ) -> Result<&QuotaReceipt, QuotaError> {
        self.advance_checked(logical_time)?;
        self.ensure_pending_not_expired(reservation, logical_time)?;
        self.reservations
            .get_mut(reservation)
            .expect("validated reservation")
            .set_state(ReservationState::Committed);
        self.append_action(QuotaAction::Committed {
            reservation: reservation.clone(),
            logical_time,
        })
    }

    /// Releases a pending reservation and restores quota up to policy capacity.
    pub fn release(
        &mut self,
        reservation: &ReservationId,
        logical_time: u64,
    ) -> Result<&QuotaReceipt, QuotaError> {
        self.advance_checked(logical_time)?;
        self.ensure_pending(reservation)?;
        self.restore_claims(reservation, logical_time)?;
        self.reservations
            .get_mut(reservation)
            .expect("validated reservation")
            .set_state(ReservationState::Released);
        self.append_action(QuotaAction::Released {
            reservation: reservation.clone(),
            logical_time,
        })
    }

    /// Expires every pending lease at or before logical time and restores quota.
    pub fn expire(&mut self, logical_time: u64) -> Result<Vec<Digest>, QuotaError> {
        self.advance_checked(logical_time)?;
        let expired = self
            .reservations
            .values()
            .filter(|reservation| {
                reservation.state() == ReservationState::Pending
                    && reservation.expires_at() <= logical_time
            })
            .map(|reservation| reservation.id().clone())
            .collect::<Vec<_>>();
        let mut receipts = Vec::with_capacity(expired.len());
        for reservation in expired {
            self.restore_claims(&reservation, logical_time)?;
            self.reservations
                .get_mut(&reservation)
                .expect("selected reservation")
                .set_state(ReservationState::Expired);
            let digest = self
                .append_action(QuotaAction::Expired {
                    reservation,
                    logical_time,
                })?
                .digest();
            receipts.push(digest);
        }
        Ok(receipts)
    }

    #[must_use]
    pub fn reservation(&self, id: &ReservationId) -> Option<&Reservation> {
        self.reservations.get(id)
    }

    #[must_use]
    pub fn receipts(&self) -> &[QuotaReceipt] {
        &self.receipts
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.receipts.last().map_or(Digest::ZERO, QuotaReceipt::digest)
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn state_digest(&self) -> Digest {
        quota_state_digest(&self.policies, &self.buckets, &self.reservations)
    }

    /// Replays every action from empty state and verifies the receipt chain.
    pub fn verify(&self) -> Result<QuotaVerification, QuotaError> {
        let mut replay = QuotaManager::new();
        for (index, receipt) in self.receipts.iter().enumerate() {
            let expected_index = index as u64;
            if receipt.index() != expected_index {
                return Err(QuotaError::ReceiptIndex {
                    expected: expected_index,
                    actual: receipt.index(),
                });
            }
            if receipt.previous() != replay.head() {
                return Err(QuotaError::ReceiptPrevious {
                    expected: replay.head(),
                    actual: receipt.previous(),
                });
            }
            replay.apply_action_without_receipt(receipt.action().clone())?;
            let expected_state = replay.state_digest();
            if expected_state != receipt.state_digest() {
                return Err(QuotaError::ReceiptState {
                    index: receipt.index(),
                    expected: expected_state,
                    actual: receipt.state_digest(),
                });
            }
            let expected = QuotaReceipt::manufacture(
                receipt.index(),
                receipt.previous(),
                receipt.action().clone(),
                expected_state,
            );
            if expected.digest() != receipt.digest() {
                return Err(QuotaError::ReceiptDigest {
                    index: receipt.index(),
                });
            }
            replay.receipts.push(receipt.clone());
        }
        if replay.state_digest() != self.state_digest() {
            return Err(QuotaError::ReceiptState {
                index: self.receipts.len() as u64,
                expected: self.state_digest(),
                actual: replay.state_digest(),
            });
        }
        Ok(QuotaVerification {
            policies: self.policies.len(),
            reservations: self.reservations.len(),
            receipts: self.receipts.len(),
            logical_time: self.logical_time,
            state_digest: self.state_digest(),
            head: self.head(),
        })
    }

    fn append_action(&mut self, action: QuotaAction) -> Result<&QuotaReceipt, QuotaError> {
        let receipt = QuotaReceipt::manufacture(
            self.receipts.len() as u64,
            self.head(),
            action,
            self.state_digest(),
        );
        self.receipts.push(receipt);
        Ok(self.receipts.last().expect("just appended receipt"))
    }

    fn advance_checked(&mut self, logical_time: u64) -> Result<(), QuotaError> {
        if logical_time < self.logical_time {
            return Err(QuotaError::LogicalTimeRegression {
                previous: self.logical_time,
                actual: logical_time,
            });
        }
        self.logical_time = logical_time;
        Ok(())
    }

    fn refill_key(
        &mut self,
        key: &(PrincipalId, ResourceId),
        logical_time: u64,
    ) -> Result<(), QuotaError> {
        let policy = self
            .policies
            .get(key)
            .ok_or_else(|| QuotaError::PolicyMissing {
                principal: key.0.clone(),
                resource: key.1.clone(),
            })?;
        self.buckets
            .entry(key.clone())
            .or_insert_with(|| Bucket::full(policy, logical_time))
            .refill(policy, logical_time);
        Ok(())
    }

    fn ensure_pending(&self, reservation: &ReservationId) -> Result<(), QuotaError> {
        let value = self
            .reservations
            .get(reservation)
            .ok_or_else(|| QuotaError::ReservationMissing(reservation.clone()))?;
        if value.state() == ReservationState::Pending {
            Ok(())
        } else {
            Err(QuotaError::ReservationNotPending {
                reservation: reservation.clone(),
                state: value.state(),
            })
        }
    }

    fn ensure_pending_not_expired(
        &self,
        reservation: &ReservationId,
        logical_time: u64,
    ) -> Result<(), QuotaError> {
        self.ensure_pending(reservation)?;
        let value = &self.reservations[reservation];
        if logical_time < value.expires_at() {
            Ok(())
        } else {
            Err(QuotaError::ReservationExpired {
                reservation: reservation.clone(),
                expires_at: value.expires_at(),
                logical_time,
            })
        }
    }

    fn restore_claims(
        &mut self,
        reservation: &ReservationId,
        logical_time: u64,
    ) -> Result<(), QuotaError> {
        let claims = self
            .reservations
            .get(reservation)
            .ok_or_else(|| QuotaError::ReservationMissing(reservation.clone()))?
            .request()
            .claims()
            .to_vec();
        for claim in claims {
            let key = (claim.principal().clone(), claim.resource().clone());
            self.refill_key(&key, logical_time)?;
            let capacity = self.policies[&key].capacity();
            let bucket = self.buckets.get_mut(&key).expect("policy created bucket");
            bucket.available = bucket.available.saturating_add(claim.amount()).min(capacity);
        }
        Ok(())
    }

    fn apply_action_without_receipt(&mut self, action: QuotaAction) -> Result<(), QuotaError> {
        match action {
            QuotaAction::PolicySet {
                principal,
                resource,
                policy,
                logical_time,
            } => {
                self.advance_checked(logical_time)?;
                let key = (principal, resource);
                self.policies.insert(key.clone(), policy.clone());
                self.buckets
                    .entry(key)
                    .and_modify(|bucket| bucket.available = bucket.available.min(policy.capacity()))
                    .or_insert_with(|| Bucket::full(&policy, logical_time));
            }
            QuotaAction::Reserved { request } => {
                self.advance_checked(request.logical_time())?;
                let mut candidate = self.buckets.clone();
                let mut lease_duration = u64::MAX;
                for claim in request.claims() {
                    let key = (claim.principal().clone(), claim.resource().clone());
                    let policy = self
                        .policies
                        .get(&key)
                        .ok_or_else(|| QuotaError::PolicyMissing {
                            principal: claim.principal().clone(),
                            resource: claim.resource().clone(),
                        })?;
                    lease_duration = lease_duration.min(policy.lease_duration());
                    let bucket = candidate
                        .entry(key)
                        .or_insert_with(|| Bucket::full(policy, request.logical_time()));
                    bucket.refill(policy, request.logical_time());
                    bucket.available -= claim.amount();
                }
                let expires_at = request.logical_time().saturating_add(lease_duration);
                candidate.shrink_to_fit();
                self.buckets = candidate;
                self.reservations
                    .insert(request.id().clone(), Reservation::new(request, expires_at));
            }
            QuotaAction::Committed {
                reservation,
                logical_time,
            } => {
                self.advance_checked(logical_time)?;
                self.reservations
                    .get_mut(&reservation)
                    .ok_or_else(|| QuotaError::ReservationMissing(reservation.clone()))?
                    .set_state(ReservationState::Committed);
            }
            QuotaAction::Released {
                reservation,
                logical_time,
            } => {
                self.advance_checked(logical_time)?;
                self.restore_claims(&reservation, logical_time)?;
                self.reservations
                    .get_mut(&reservation)
                    .ok_or_else(|| QuotaError::ReservationMissing(reservation.clone()))?
                    .set_state(ReservationState::Released);
            }
            QuotaAction::Expired {
                reservation,
                logical_time,
            } => {
                self.advance_checked(logical_time)?;
                self.restore_claims(&reservation, logical_time)?;
                self.reservations
                    .get_mut(&reservation)
                    .ok_or_else(|| QuotaError::ReservationMissing(reservation.clone()))?
                    .set_state(ReservationState::Expired);
            }
        }
        Ok(())
    }
}

/// Aggregate quota replay evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaVerification {
    policies: usize,
    reservations: usize,
    receipts: usize,
    logical_time: u64,
    state_digest: Digest,
    head: Digest,
}

impl QuotaVerification {
    #[must_use]
    pub const fn policies(&self) -> usize {
        self.policies
    }

    #[must_use]
    pub const fn reservations(&self) -> usize {
        self.reservations
    }

    #[must_use]
    pub const fn receipts(&self) -> usize {
        self.receipts
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub const fn state_digest(&self) -> Digest {
        self.state_digest
    }

    #[must_use]
    pub const fn head(&self) -> Digest {
        self.head
    }
}

fn quota_state_digest(
    policies: &BTreeMap<(PrincipalId, ResourceId), QuotaPolicy>,
    buckets: &BTreeMap<(PrincipalId, ResourceId), Bucket>,
    reservations: &BTreeMap<ReservationId, Reservation>,
) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "quota-state-v1")
        .u64("policy-count", policies.len() as u64);
    for ((principal, resource), policy) in policies {
        encoder
            .text("principal", principal.as_str())
            .text("resource", resource.as_str())
            .field("policy", &policy.digest().0)
            .field("bucket", &buckets[&(principal.clone(), resource.clone())].digest().0);
    }
    encoder.u64("reservation-count", reservations.len() as u64);
    for reservation in reservations.values() {
        encoder
            .text("reservation", reservation.id().as_str())
            .field("reservation-digest", &reservation.digest().0);
    }
    encoder.digest()
}

#[cfg(test)]
mod tests {
    use super::{
        PrincipalId, QuotaClaim, QuotaError, QuotaManager, QuotaPolicy, ReservationId,
        ReservationRequest, ReservationState, ResourceId,
    };

    fn principal(value: &str) -> PrincipalId {
        PrincipalId::new(value).unwrap()
    }

    fn resource(value: &str) -> ResourceId {
        ResourceId::new(value).unwrap()
    }

    fn manager() -> QuotaManager {
        let mut manager = QuotaManager::new();
        manager
            .set_policy(
                principal("team"),
                resource("cpu"),
                QuotaPolicy::new(100, 10, 5, 60, 20).unwrap(),
                0,
            )
            .unwrap();
        manager
            .set_policy(
                principal("team"),
                resource("memory"),
                QuotaPolicy::new(200, 0, 1, 100, 20).unwrap(),
                0,
            )
            .unwrap();
        manager
    }

    #[test]
    fn multi_resource_reservation_is_atomic() {
        let mut manager = manager();
        let request = ReservationRequest::new(
            ReservationId::new("job-1").unwrap(),
            1,
            vec![
                QuotaClaim::new(principal("team"), resource("cpu"), 50).unwrap(),
                QuotaClaim::new(principal("team"), resource("memory"), 80).unwrap(),
            ],
        )
        .unwrap();
        manager.reserve(request).unwrap();
        assert_eq!(manager.available(&principal("team"), &resource("cpu"), 1).unwrap(), 50);
        assert_eq!(
            manager
                .available(&principal("team"), &resource("memory"), 1)
                .unwrap(),
            120
        );
        assert_eq!(manager.verify().unwrap().reservations(), 1);
    }

    #[test]
    fn failed_claim_leaves_all_buckets_unchanged() {
        let mut manager = manager();
        let request = ReservationRequest::new(
            ReservationId::new("job-1").unwrap(),
            1,
            vec![
                QuotaClaim::new(principal("team"), resource("cpu"), 50).unwrap(),
                QuotaClaim::new(principal("team"), resource("memory"), 101).unwrap(),
            ],
        )
        .unwrap();
        assert!(matches!(
            manager.reserve(request),
            Err(QuotaError::ClaimTooLarge { .. })
        ));
        assert_eq!(manager.available(&principal("team"), &resource("cpu"), 1).unwrap(), 100);
        assert_eq!(
            manager
                .available(&principal("team"), &resource("memory"), 1)
                .unwrap(),
            200
        );
    }

    #[test]
    fn release_restores_reserved_tokens() {
        let mut manager = manager();
        let id = ReservationId::new("job-1").unwrap();
        manager
            .reserve(
                ReservationRequest::new(
                    id.clone(),
                    1,
                    vec![QuotaClaim::new(principal("team"), resource("cpu"), 50).unwrap()],
                )
                .unwrap(),
            )
            .unwrap();
        manager.release(&id, 2).unwrap();
        assert_eq!(manager.available(&principal("team"), &resource("cpu"), 2).unwrap(), 100);
        assert_eq!(manager.reservation(&id).unwrap().state(), ReservationState::Released);
    }

    #[test]
    fn expiry_restores_pending_lease() {
        let mut manager = manager();
        let id = ReservationId::new("job-1").unwrap();
        manager
            .reserve(
                ReservationRequest::new(
                    id.clone(),
                    1,
                    vec![QuotaClaim::new(principal("team"), resource("cpu"), 50).unwrap()],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(manager.expire(21).unwrap().len(), 1);
        assert_eq!(manager.reservation(&id).unwrap().state(), ReservationState::Expired);
        assert_eq!(manager.available(&principal("team"), &resource("cpu"), 21).unwrap(), 100);
    }

    #[test]
    fn committed_quota_is_not_restored() {
        let mut manager = manager();
        let id = ReservationId::new("job-1").unwrap();
        manager
            .reserve(
                ReservationRequest::new(
                    id.clone(),
                    1,
                    vec![QuotaClaim::new(principal("team"), resource("cpu"), 50).unwrap()],
                )
                .unwrap(),
            )
            .unwrap();
        manager.commit(&id, 2).unwrap();
        assert_eq!(manager.expire(100).unwrap().len(), 0);
        assert_eq!(manager.reservation(&id).unwrap().state(), ReservationState::Committed);
    }
}
