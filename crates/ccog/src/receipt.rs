//! PROV-O receipt with BLAKE3 proof-of-origin hashing.

use chrono::{DateTime, Utc};
use crate::graph::GraphIri;

/// PROV receipt with cryptographic proof and deterministic hash.
#[derive(Clone, Debug)]
pub struct Receipt {
    /// IRI of the prov:Activity that generated the outcome.
    pub activity_iri: GraphIri,

    /// BLAKE3 hash of the activity's inputs, rules, and outputs.
    /// Hex-encoded, 64 characters (256 bits).
    pub hash: String,

    /// Timestamp when the receipt was generated.
    pub timestamp: DateTime<Utc>,
}

impl Receipt {
    /// Create a new receipt.
    pub fn new(activity_iri: GraphIri, hash: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            activity_iri,
            hash,
            timestamp,
        }
    }

    /// Generate a BLAKE3 hash from input data.
    pub fn blake3_hex(data: &[u8]) -> String {
        let hash = blake3::hash(data);
        hash.to_hex().to_string()
    }

    /// Parse this receipt's hex hash back into a typed `blake3::Hash`.
    ///
    /// # Errors
    ///
    /// Returns `Err(message)` if `self.hash` is not a valid 64-character
    /// BLAKE3 hex digest.
    pub fn chain_hash(&self) -> Result<blake3::Hash, String> {
        blake3::Hash::from_hex(&self.hash)
            .map_err(|e| format!("Receipt::chain_hash: invalid hex '{}': {}", self.hash, e))
    }
}
