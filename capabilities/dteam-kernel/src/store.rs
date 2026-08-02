//! Deterministic transactional state store with optimistic concurrency and replay receipts.

use crate::hash::{CanonicalEncoder, Digest};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// Stable record key.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordKey(String);

impl RecordKey {
    /// Creates a non-empty key without control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, StoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(StoreError::EmptyKey);
        }
        if value.bytes().any(|byte| byte.is_ascii_control()) {
            return Err(StoreError::ControlCharacter);
        }
        Ok(Self(value))
    }

    /// Returns the key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RecordKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Optimistic version precondition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectedVersion {
    Any,
    Missing,
    Exact(u64),
}

impl ExpectedVersion {
    fn encode(self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Any => {
                encoder.text("expected", "any");
            }
            Self::Missing => {
                encoder.text("expected", "missing");
            }
            Self::Exact(version) => {
                encoder.text("expected", "exact").u64("version", version);
            }
        }
    }
}

/// One mutation in an atomic transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mutation {
    Put {
        key: RecordKey,
        value: Vec<u8>,
        expected: ExpectedVersion,
        expires_at: Option<u64>,
    },
    Delete {
        key: RecordKey,
        expected: ExpectedVersion,
    },
}

impl Mutation {
    #[must_use]
    pub const fn key(&self) -> &RecordKey {
        match self {
            Self::Put { key, .. } | Self::Delete { key, .. } => key,
        }
    }

    #[must_use]
    pub const fn expected(&self) -> ExpectedVersion {
        match self {
            Self::Put { expected, .. } | Self::Delete { expected, .. } => *expected,
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Put {
                key,
                value,
                expected,
                expires_at,
            } => {
                encoder
                    .text("mutation", "put")
                    .text("key", key.as_str())
                    .field("value", value);
                expected.encode(encoder);
                match expires_at {
                    Some(value) => {
                        encoder.boolean("has-expiry", true).u64("expires-at", *value);
                    }
                    None => {
                        encoder.boolean("has-expiry", false);
                    }
                }
            }
            Self::Delete { key, expected } => {
                encoder
                    .text("mutation", "delete")
                    .text("key", key.as_str());
                expected.encode(encoder);
            }
        }
    }
}

/// Immutable transaction request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    id: String,
    logical_time: u64,
    mutations: Vec<Mutation>,
    digest: Digest,
}

impl Transaction {
    /// Builds a transaction and rejects duplicate keys.
    pub fn new(
        id: impl Into<String>,
        logical_time: u64,
        mutations: Vec<Mutation>,
    ) -> Result<Self, StoreError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(StoreError::EmptyTransactionId);
        }
        if mutations.is_empty() {
            return Err(StoreError::EmptyTransaction);
        }
        let mut keys = BTreeSet::new();
        for mutation in &mutations {
            if !keys.insert(mutation.key().clone()) {
                return Err(StoreError::DuplicateMutationKey(mutation.key().clone()));
            }
        }
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "transaction-v1")
            .text("id", &id)
            .u64("logical-time", logical_time)
            .u64("mutation-count", mutations.len() as u64);
        for mutation in &mutations {
            mutation.encode(&mut encoder);
        }
        Ok(Self {
            id,
            logical_time,
            mutations,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn mutations(&self) -> &[Mutation] {
        &self.mutations
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Versioned internal record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    key: RecordKey,
    version: u64,
    value: Vec<u8>,
    tombstone: bool,
    expires_at: Option<u64>,
    digest: Digest,
}

impl Record {
    fn new(
        key: RecordKey,
        version: u64,
        value: Vec<u8>,
        tombstone: bool,
        expires_at: Option<u64>,
    ) -> Self {
        let mut record = Self {
            key,
            version,
            value,
            tombstone,
            expires_at,
            digest: Digest::ZERO,
        };
        record.digest = record.recompute_digest();
        record
    }

    #[must_use]
    pub const fn key(&self) -> &RecordKey {
        &self.key
    }

    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    #[must_use]
    pub const fn tombstone(&self) -> bool {
        self.tombstone
    }

    #[must_use]
    pub const fn expires_at(&self) -> Option<u64> {
        self.expires_at
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn visible_at(&self, logical_time: u64) -> bool {
        !self.tombstone && self.expires_at.is_none_or(|expiry| logical_time < expiry)
    }

    #[must_use]
    pub fn recompute_digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "record-v1")
            .text("key", self.key.as_str())
            .u64("version", self.version)
            .field("value", &self.value)
            .boolean("tombstone", self.tombstone);
        match self.expires_at {
            Some(value) => {
                encoder.boolean("has-expiry", true).u64("expires-at", value);
            }
            None => {
                encoder.boolean("has-expiry", false);
            }
        }
        encoder.digest()
    }
}

/// Per-key mutation evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    key: RecordKey,
    before: Option<Digest>,
    after: Digest,
    version: u64,
}

impl Change {
    #[must_use]
    pub const fn key(&self) -> &RecordKey {
        &self.key
    }

    #[must_use]
    pub const fn before(&self) -> Option<Digest> {
        self.before
    }

    #[must_use]
    pub const fn after(&self) -> Digest {
        self.after
    }

    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
}

/// Immutable atomic commit evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitReceipt {
    index: u64,
    previous: Digest,
    transaction: Transaction,
    changes: Vec<Change>,
    state_digest: Digest,
    digest: Digest,
}

impl CommitReceipt {
    fn manufacture(
        index: u64,
        previous: Digest,
        transaction: Transaction,
        changes: Vec<Change>,
        state_digest: Digest,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "commit-receipt-v1")
            .u64("index", index)
            .field("previous", &previous.0)
            .field("transaction", &transaction.digest().0)
            .u64("change-count", changes.len() as u64);
        for change in &changes {
            encoder.text("key", change.key().as_str());
            match change.before() {
                Some(value) => {
                    encoder.boolean("has-before", true).field("before", &value.0);
                }
                None => {
                    encoder.boolean("has-before", false);
                }
            }
            encoder
                .field("after", &change.after().0)
                .u64("version", change.version());
        }
        encoder.field("state", &state_digest.0);
        Self {
            index,
            previous,
            transaction,
            changes,
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
    pub const fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    #[must_use]
    pub fn changes(&self) -> &[Change] {
        &self.changes
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

/// Point-in-time deterministic snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreSnapshot {
    logical_time: u64,
    records: BTreeMap<RecordKey, Record>,
    state_digest: Digest,
    commit_head: Digest,
}

impl StoreSnapshot {
    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn records(&self) -> &BTreeMap<RecordKey, Record> {
        &self.records
    }

    #[must_use]
    pub const fn state_digest(&self) -> Digest {
        self.state_digest
    }

    #[must_use]
    pub const fn commit_head(&self) -> Digest {
        self.commit_head
    }
}

/// Transactional store failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreError {
    EmptyKey,
    ControlCharacter,
    EmptyTransactionId,
    EmptyTransaction,
    DuplicateMutationKey(RecordKey),
    VersionConflict {
        key: RecordKey,
        expected: ExpectedVersion,
        actual: Option<u64>,
    },
    ExpiryNotInFuture {
        key: RecordKey,
        logical_time: u64,
        expires_at: u64,
    },
    LogicalTimeRegression { previous: u64, actual: u64 },
    ReceiptIndex { expected: u64, actual: u64 },
    ReceiptPrevious { expected: Digest, actual: Digest },
    ReceiptState { index: u64, expected: Digest, actual: Digest },
    ReceiptDigest { index: u64 },
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("record key must not be empty"),
            Self::ControlCharacter => formatter.write_str("record key contains control characters"),
            Self::EmptyTransactionId => formatter.write_str("transaction id must not be empty"),
            Self::EmptyTransaction => formatter.write_str("transaction must contain mutations"),
            Self::DuplicateMutationKey(key) => {
                write!(formatter, "transaction mutates `{key}` more than once")
            }
            Self::VersionConflict {
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "version conflict for `{key}`: expected {expected:?}, actual {actual:?}"
            ),
            Self::ExpiryNotInFuture {
                key,
                logical_time,
                expires_at,
            } => write!(
                formatter,
                "expiry {expires_at} for `{key}` is not after logical time {logical_time}"
            ),
            Self::LogicalTimeRegression { previous, actual } => write!(
                formatter,
                "transaction logical time regressed from {previous} to {actual}"
            ),
            Self::ReceiptIndex { expected, actual } => {
                write!(formatter, "commit index {actual}, expected {expected}")
            }
            Self::ReceiptPrevious { expected, actual } => {
                write!(formatter, "commit predecessor {actual}, expected {expected}")
            }
            Self::ReceiptState {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "commit {index} state digest {actual}, expected {expected}"
            ),
            Self::ReceiptDigest { index } => {
                write!(formatter, "commit {index} receipt digest mismatch")
            }
        }
    }
}

impl std::error::Error for StoreError {}

/// Append-only transactional store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionalStore {
    records: BTreeMap<RecordKey, Record>,
    commits: Vec<CommitReceipt>,
    transactions: BTreeMap<Digest, usize>,
    logical_time: u64,
}

impl TransactionalStore {
    /// Starts an empty store at logical time zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the latest physical record including tombstones and expired values.
    #[must_use]
    pub fn raw(&self, key: &RecordKey) -> Option<&Record> {
        self.records.get(key)
    }

    /// Returns a visible record at a caller-supplied logical time.
    #[must_use]
    pub fn get_at(&self, key: &RecordKey, logical_time: u64) -> Option<&Record> {
        self.records
            .get(key)
            .filter(|record| record.visible_at(logical_time))
    }

    /// Returns the record visible at the store's current logical time.
    #[must_use]
    pub fn get(&self, key: &RecordKey) -> Option<&Record> {
        self.get_at(key, self.logical_time)
    }

    /// Queries visible records by key prefix in canonical key order.
    pub fn prefix<'store>(
        &'store self,
        prefix: &'store str,
    ) -> impl Iterator<Item = &'store Record> + 'store {
        self.records
            .values()
            .filter(move |record| record.key().as_str().starts_with(prefix))
            .filter(|record| record.visible_at(self.logical_time))
    }

    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    #[must_use]
    pub fn commits(&self) -> &[CommitReceipt] {
        &self.commits
    }

    #[must_use]
    pub fn head(&self) -> Digest {
        self.commits.last().map_or(Digest::ZERO, CommitReceipt::digest)
    }

    /// Computes canonical physical state identity, including tombstones and expiry metadata.
    #[must_use]
    pub fn state_digest(&self) -> Digest {
        state_digest(&self.records)
    }

    /// Atomically applies all transaction mutations after validating every precondition.
    pub fn commit(&mut self, transaction: Transaction) -> Result<&CommitReceipt, StoreError> {
        if let Some(index) = self.transactions.get(&transaction.digest()).copied() {
            return Ok(&self.commits[index]);
        }
        if transaction.logical_time() < self.logical_time {
            return Err(StoreError::LogicalTimeRegression {
                previous: self.logical_time,
                actual: transaction.logical_time(),
            });
        }
        validate_mutations(&self.records, &transaction)?;
        let (next, changes) = apply_mutations(&self.records, &transaction);
        let next_state = state_digest(&next);
        let receipt = CommitReceipt::manufacture(
            self.commits.len() as u64,
            self.head(),
            transaction,
            changes,
            next_state,
        );
        self.records = next;
        self.logical_time = receipt.transaction().logical_time();
        let index = self.commits.len();
        self.transactions
            .insert(receipt.transaction().digest(), index);
        self.commits.push(receipt);
        Ok(&self.commits[index])
    }

    /// Advances logical time without changing physical records.
    pub fn advance_time(&mut self, logical_time: u64) -> Result<(), StoreError> {
        if logical_time < self.logical_time {
            return Err(StoreError::LogicalTimeRegression {
                previous: self.logical_time,
                actual: logical_time,
            });
        }
        self.logical_time = logical_time;
        Ok(())
    }

    /// Captures visible state at a logical time while preserving commit identity.
    #[must_use]
    pub fn snapshot_at(&self, logical_time: u64) -> StoreSnapshot {
        let records = self
            .records
            .iter()
            .filter(|(_, record)| record.visible_at(logical_time))
            .map(|(key, record)| (key.clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        StoreSnapshot {
            logical_time,
            state_digest: state_digest(&records),
            records,
            commit_head: self.head(),
        }
    }

    /// Replays every transaction and verifies indexes, links, state identities, and receipts.
    pub fn verify(&self) -> Result<StoreVerification, StoreError> {
        let mut records = BTreeMap::new();
        let mut previous = Digest::ZERO;
        let mut logical_time = 0_u64;
        for (index, receipt) in self.commits.iter().enumerate() {
            let expected_index = index as u64;
            if receipt.index() != expected_index {
                return Err(StoreError::ReceiptIndex {
                    expected: expected_index,
                    actual: receipt.index(),
                });
            }
            if receipt.previous() != previous {
                return Err(StoreError::ReceiptPrevious {
                    expected: previous,
                    actual: receipt.previous(),
                });
            }
            if receipt.transaction().logical_time() < logical_time {
                return Err(StoreError::LogicalTimeRegression {
                    previous: logical_time,
                    actual: receipt.transaction().logical_time(),
                });
            }
            validate_mutations(&records, receipt.transaction())?;
            let (next, changes) = apply_mutations(&records, receipt.transaction());
            let digest = state_digest(&next);
            if digest != receipt.state_digest() {
                return Err(StoreError::ReceiptState {
                    index: receipt.index(),
                    expected: digest,
                    actual: receipt.state_digest(),
                });
            }
            let expected_receipt = CommitReceipt::manufacture(
                receipt.index(),
                receipt.previous(),
                receipt.transaction().clone(),
                changes,
                digest,
            );
            if expected_receipt.digest() != receipt.digest() {
                return Err(StoreError::ReceiptDigest {
                    index: receipt.index(),
                });
            }
            records = next;
            logical_time = receipt.transaction().logical_time();
            previous = receipt.digest();
        }
        let actual = state_digest(&records);
        let expected = self.state_digest();
        if actual != expected {
            return Err(StoreError::ReceiptState {
                index: self.commits.len() as u64,
                expected,
                actual,
            });
        }
        Ok(StoreVerification {
            commits: self.commits.len(),
            records: records.len(),
            logical_time,
            state_digest: actual,
            commit_head: previous,
        })
    }
}

/// Aggregate replay verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreVerification {
    commits: usize,
    records: usize,
    logical_time: u64,
    state_digest: Digest,
    commit_head: Digest,
}

impl StoreVerification {
    #[must_use]
    pub const fn commits(&self) -> usize {
        self.commits
    }

    #[must_use]
    pub const fn records(&self) -> usize {
        self.records
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
    pub const fn commit_head(&self) -> Digest {
        self.commit_head
    }
}

fn validate_mutations(
    records: &BTreeMap<RecordKey, Record>,
    transaction: &Transaction,
) -> Result<(), StoreError> {
    for mutation in transaction.mutations() {
        let actual = records.get(mutation.key()).map(Record::version);
        let valid = match mutation.expected() {
            ExpectedVersion::Any => true,
            ExpectedVersion::Missing => actual.is_none(),
            ExpectedVersion::Exact(expected) => actual == Some(expected),
        };
        if !valid {
            return Err(StoreError::VersionConflict {
                key: mutation.key().clone(),
                expected: mutation.expected(),
                actual,
            });
        }
        if let Mutation::Put {
            key,
            expires_at: Some(expires_at),
            ..
        } = mutation
        {
            if *expires_at <= transaction.logical_time() {
                return Err(StoreError::ExpiryNotInFuture {
                    key: key.clone(),
                    logical_time: transaction.logical_time(),
                    expires_at: *expires_at,
                });
            }
        }
    }
    Ok(())
}

fn apply_mutations(
    records: &BTreeMap<RecordKey, Record>,
    transaction: &Transaction,
) -> (BTreeMap<RecordKey, Record>, Vec<Change>) {
    let mut next = records.clone();
    let mut mutations = transaction.mutations().iter().collect::<Vec<_>>();
    mutations.sort_by_key(|mutation| mutation.key().clone());
    let mut changes = Vec::with_capacity(mutations.len());
    for mutation in mutations {
        let before = next.get(mutation.key()).map(Record::digest);
        let version = next
            .get(mutation.key())
            .map_or(1, |record| record.version().saturating_add(1));
        let record = match mutation {
            Mutation::Put {
                key,
                value,
                expires_at,
                ..
            } => Record::new(key.clone(), version, value.clone(), false, *expires_at),
            Mutation::Delete { key, .. } => {
                Record::new(key.clone(), version, Vec::new(), true, None)
            }
        };
        changes.push(Change {
            key: mutation.key().clone(),
            before,
            after: record.digest(),
            version,
        });
        next.insert(mutation.key().clone(), record);
    }
    (next, changes)
}

fn state_digest(records: &BTreeMap<RecordKey, Record>) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "transactional-store-state-v1")
        .u64("record-count", records.len() as u64);
    for record in records.values() {
        encoder
            .text("key", record.key().as_str())
            .field("record", &record.digest().0);
    }
    encoder.digest()
}

#[cfg(test)]
mod tests {
    use super::{
        ExpectedVersion, Mutation, RecordKey, StoreError, Transaction, TransactionalStore,
    };

    fn key(value: &str) -> RecordKey {
        RecordKey::new(value).unwrap()
    }

    #[test]
    fn multi_key_commit_is_atomic_and_replayable() {
        let mut store = TransactionalStore::new();
        let transaction = Transaction::new(
            "create",
            1,
            vec![
                Mutation::Put {
                    key: key("case/1/status"),
                    value: b"open".to_vec(),
                    expected: ExpectedVersion::Missing,
                    expires_at: None,
                },
                Mutation::Put {
                    key: key("case/1/owner"),
                    value: b"alice".to_vec(),
                    expected: ExpectedVersion::Missing,
                    expires_at: None,
                },
            ],
        )
        .unwrap();
        store.commit(transaction).unwrap();
        assert_eq!(store.prefix("case/1/").count(), 2);
        let verification = store.verify().unwrap();
        assert_eq!(verification.commits(), 1);
        assert_eq!(verification.state_digest(), store.state_digest());
    }

    #[test]
    fn failed_precondition_changes_nothing() {
        let mut store = TransactionalStore::new();
        let create = Transaction::new(
            "create",
            1,
            vec![Mutation::Put {
                key: key("value"),
                value: b"one".to_vec(),
                expected: ExpectedVersion::Missing,
                expires_at: None,
            }],
        )
        .unwrap();
        store.commit(create).unwrap();
        let before = store.state_digest();
        let conflict = Transaction::new(
            "conflict",
            2,
            vec![Mutation::Put {
                key: key("value"),
                value: b"two".to_vec(),
                expected: ExpectedVersion::Exact(9),
                expires_at: None,
            }],
        )
        .unwrap();
        assert!(matches!(
            store.commit(conflict),
            Err(StoreError::VersionConflict { .. })
        ));
        assert_eq!(store.state_digest(), before);
        assert_eq!(store.commits().len(), 1);
    }

    #[test]
    fn duplicate_transaction_is_exactly_once() {
        let mut store = TransactionalStore::new();
        let transaction = Transaction::new(
            "create",
            1,
            vec![Mutation::Put {
                key: key("value"),
                value: b"one".to_vec(),
                expected: ExpectedVersion::Missing,
                expires_at: None,
            }],
        )
        .unwrap();
        let first = store.commit(transaction.clone()).unwrap().digest();
        let duplicate = store.commit(transaction).unwrap().digest();
        assert_eq!(first, duplicate);
        assert_eq!(store.commits().len(), 1);
    }

    #[test]
    fn logical_expiry_hides_record_without_mutation() {
        let mut store = TransactionalStore::new();
        store
            .commit(
                Transaction::new(
                    "lease",
                    10,
                    vec![Mutation::Put {
                        key: key("lease"),
                        value: b"holder".to_vec(),
                        expected: ExpectedVersion::Missing,
                        expires_at: Some(20),
                    }],
                )
                .unwrap(),
            )
            .unwrap();
        assert!(store.get(&key("lease")).is_some());
        store.advance_time(20).unwrap();
        assert!(store.get(&key("lease")).is_none());
        assert!(store.raw(&key("lease")).is_some());
    }

    #[test]
    fn tombstone_preserves_monotonic_version() {
        let mut store = TransactionalStore::new();
        store
            .commit(
                Transaction::new(
                    "create",
                    1,
                    vec![Mutation::Put {
                        key: key("value"),
                        value: b"one".to_vec(),
                        expected: ExpectedVersion::Missing,
                        expires_at: None,
                    }],
                )
                .unwrap(),
            )
            .unwrap();
        store
            .commit(
                Transaction::new(
                    "delete",
                    2,
                    vec![Mutation::Delete {
                        key: key("value"),
                        expected: ExpectedVersion::Exact(1),
                    }],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(store.raw(&key("value")).unwrap().version(), 2);
        assert!(store.raw(&key("value")).unwrap().tombstone());
        assert!(store.get(&key("value")).is_none());
    }
}
