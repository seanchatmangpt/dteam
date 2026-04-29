//! POWL64 geometric routing + BLAKE3 receipt chain.
//!
//! Sparse 64×64×64 cell universe addressed by GlobeCell-packed coordinates.
//! Each [`Powl64Cell`] carries a BLAKE3 source receipt and links to its prior
//! receipt to form a cryptographic chain through `Runtime::step`.
//!
//! # Geometry
//!
//! ```text
//! 64³ = 64 × 64 × 64 = 262,144 independent places
//!       ─┬─   ─┬─   ─┬─
//!        │     │     └── 64 places per attention cell
//!        │     └──────── 64 cells per domain
//!        └────────────── 64 domains in the globe
//! ```
//!
//! Storage is sparse: only cells touched by [`Powl64::extend`] are
//! materialized. A program with two extends occupies two `HashMap` entries,
//! never the full 262,144 slot grid.
//!
//! # Chain receipts
//!
//! Genesis: `chain_hash = source_receipt = blake3(iri)`.
//! Subsequent: `chain_hash = blake3(prior_chain_hash || source_receipt)`,
//! folded through a 64-byte stack buffer (no heap allocation per step).

use std::collections::HashMap;

use crate::graph::GraphIri;

// =============================================================================
// PACKING CONSTANTS
// =============================================================================

/// Bits per coordinate component (6 bits ⇒ 0..64).
pub const COORD_BITS: u32 = 6;

// =============================================================================
// GLOBE CELL
// =============================================================================

/// Packed `(domain, cell, place)` coordinate — 6 bits per component, low 18
/// bits used.
///
/// Layout: `[63:18 unused][17:12 domain][11:6 cell][5:0 place]`. The `u64`
/// form is the canonical hash and `HashMap` key. Coordinate computation is
/// pure arithmetic: shifts and masks, no symbol table.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct GlobeCell(pub u64);

impl GlobeCell {
    /// Mask for one 6-bit coordinate component (`0x3F`).
    pub const COORD_MASK: u64 = 0x3F;

    /// Bit offset of the `place` field inside the packed `u64`.
    pub const PLACE_SHIFT: u32 = 0;

    /// Bit offset of the `cell` field inside the packed `u64`.
    pub const CELL_SHIFT: u32 = 6;

    /// Bit offset of the `domain` field inside the packed `u64`.
    pub const DOMAIN_SHIFT: u32 = 12;

    /// The origin cell `(0, 0, 0)`.
    pub const ORIGIN: Self = Self(0);

    /// Construct from `(domain, cell, place)`, each in `0..64`.
    ///
    /// Values outside `0..64` are truncated to their low 6 bits. This is a
    /// const fn with no branches; callers on the hot path should validate
    /// inputs upstream.
    #[must_use]
    #[inline(always)]
    pub const fn new(domain: u8, cell: u8, place: u8) -> Self {
        let d = (domain as u64) & Self::COORD_MASK;
        let c = (cell as u64) & Self::COORD_MASK;
        let p = (place as u64) & Self::COORD_MASK;
        Self((d << Self::DOMAIN_SHIFT) | (c << Self::CELL_SHIFT) | (p << Self::PLACE_SHIFT))
    }

    /// Extract the domain component.
    #[must_use]
    #[inline(always)]
    pub const fn domain(self) -> u8 {
        ((self.0 >> Self::DOMAIN_SHIFT) & Self::COORD_MASK) as u8
    }

    /// Extract the cell component.
    #[must_use]
    #[inline(always)]
    pub const fn cell(self) -> u8 {
        ((self.0 >> Self::CELL_SHIFT) & Self::COORD_MASK) as u8
    }

    /// Extract the place component.
    #[must_use]
    #[inline(always)]
    pub const fn place(self) -> u8 {
        ((self.0 >> Self::PLACE_SHIFT) & Self::COORD_MASK) as u8
    }

    /// Derive a coordinate from the low 18 bits of a BLAKE3 hash.
    ///
    /// Uses the first 3 bytes of the hash in little-endian order, masked to
    /// 18 bits. The result is deterministic for a given hash, so equal
    /// receipts always land in the same cell.
    #[must_use]
    #[inline]
    pub fn from_hash_low18(h: &blake3::Hash) -> Self {
        let bytes = h.as_bytes();
        let raw = (bytes[0] as u64)
            | ((bytes[1] as u64) << 8)
            | ((bytes[2] as u64) << 16);
        Self(raw & 0x3_FFFF)
    }
}

// =============================================================================
// POWL64 CELL
// =============================================================================

/// Per-cell payload: source IRI hash, prior link, polarity, derived chain
/// hash.
///
/// `chain_hash` equals `source_receipt` at genesis (no prior). For every
/// subsequent extend, `chain_hash = blake3(prior_chain_hash || source_receipt)`.
#[derive(Clone, Debug)]
pub struct Powl64Cell {
    /// Packed coordinate where this cell lives in the 64³ universe.
    pub coord: GlobeCell,
    /// Receipt polarity tag (caller-defined: e.g., `1 = required`).
    pub receipt_polarity: u8,
    /// BLAKE3 hash of the `breed_output_iri` string this cell witnesses.
    pub source_receipt: blake3::Hash,
    /// Prior chain hash if this cell extends an existing chain; `None` at
    /// genesis.
    pub prior_receipt: Option<blake3::Hash>,
    /// Derived chain hash: `blake3(prior || source)` if prior exists,
    /// else equals `source_receipt`.
    pub chain_hash: blake3::Hash,
}

// =============================================================================
// POWL64 UNIVERSE
// =============================================================================

/// Sparse Powl64 universe + chain head cursor.
///
/// Holds only the cells produced by [`Powl64::extend`]. The chain is
/// linear in the current phase; full DAG fan-out is future work.
#[derive(Debug, Default)]
pub struct Powl64 {
    cells: HashMap<GlobeCell, Powl64Cell>,
    cursor: GlobeCell,
    chain_head: Option<blake3::Hash>,
}

impl Powl64 {
    /// Build an empty universe with no cells and no chain head.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of materialized cells (NOT 262,144 — only those extended).
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Current chain head, or `None` before any extend.
    #[must_use]
    pub fn chain_head(&self) -> Option<blake3::Hash> {
        self.chain_head
    }

    /// Coordinate of the most recently inserted cell (or origin if empty).
    #[must_use]
    pub fn cursor(&self) -> GlobeCell {
        self.cursor
    }

    /// Borrow the cell at `coord`, if present.
    #[must_use]
    pub fn cell_at(&self, coord: GlobeCell) -> Option<&Powl64Cell> {
        self.cells.get(&coord)
    }

    /// Append a new cell to the chain.
    ///
    /// Hashes the IRI to produce `source_receipt`. If a chain head exists,
    /// folds it with `source_receipt` through a stack buffer to derive
    /// `chain_hash`. Coordinate is the low-18-bit projection of
    /// `chain_hash`. Updates cursor and chain head, returns the new cell.
    ///
    /// # Collision
    ///
    /// If two distinct chain hashes happen to project to the same 18-bit
    /// coordinate, the second extend overwrites the first cell at that
    /// coord. The chain hash itself remains globally distinct; only the
    /// geometric address aliases. This matches the sparse-universe contract
    /// — coords are routing labels, not identity.
    pub fn extend(&mut self, breed_output_iri: &GraphIri, polarity: u8) -> Powl64Cell {
        let source_receipt = blake3::hash(breed_output_iri.as_str().as_bytes());
        let chain_hash = match self.chain_head {
            None => source_receipt,
            Some(prior) => {
                let mut buf = [0u8; 64];
                buf[..32].copy_from_slice(prior.as_bytes());
                buf[32..].copy_from_slice(source_receipt.as_bytes());
                blake3::hash(&buf)
            }
        };
        let coord = GlobeCell::from_hash_low18(&chain_hash);
        let cell = Powl64Cell {
            coord,
            receipt_polarity: polarity,
            source_receipt,
            prior_receipt: self.chain_head,
            chain_hash,
        };
        self.cells.insert(coord, cell.clone());
        self.cursor = coord;
        self.chain_head = Some(chain_hash);
        cell
    }

    /// Stub shape-match: cell-count parity.
    ///
    /// Full DAG isomorphism is a future phase — the chain is currently
    /// linear, so cell count alone is a meaningful first invariant.
    ///
    /// # Errors
    ///
    /// Returns `Err(message)` describing the cell-count mismatch.
    pub fn shape_match(&self, other: &Powl64) -> Result<(), String> {
        if self.cells.len() == other.cells.len() {
            Ok(())
        } else {
            Err(format!(
                "cell count mismatch: {} vs {}",
                self.cells.len(),
                other.cells.len()
            ))
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globe_cell_packs_and_unpacks() {
        let g = GlobeCell::new(7, 13, 42);
        assert_eq!(g.domain(), 7);
        assert_eq!(g.cell(), 13);
        assert_eq!(g.place(), 42);
    }

    #[test]
    fn globe_cell_origin_is_zero() {
        assert_eq!(GlobeCell::ORIGIN.0, 0);
        assert_eq!(GlobeCell::ORIGIN.domain(), 0);
        assert_eq!(GlobeCell::ORIGIN.cell(), 0);
        assert_eq!(GlobeCell::ORIGIN.place(), 0);
    }

    #[test]
    fn globe_cell_truncates_oversized_inputs() {
        // 64 wraps to 0 since only low 6 bits are kept.
        let g = GlobeCell::new(64, 64, 64);
        assert_eq!(g.0, 0);
    }

    #[test]
    fn from_hash_low18_uses_first_three_bytes_le() {
        let h = blake3::hash(b"deterministic input");
        let g = GlobeCell::from_hash_low18(&h);
        let bytes = h.as_bytes();
        let expected = ((bytes[0] as u64)
            | ((bytes[1] as u64) << 8)
            | ((bytes[2] as u64) << 16))
            & 0x3_FFFF;
        assert_eq!(g.0, expected);
    }

    #[test]
    fn empty_universe_has_no_chain_head() {
        let p = Powl64::new();
        assert_eq!(p.cell_count(), 0);
        assert!(p.chain_head().is_none());
        assert_eq!(p.cursor(), GlobeCell::ORIGIN);
    }

    #[test]
    fn shape_match_succeeds_for_equal_cell_counts() {
        let a = Powl64::new();
        let b = Powl64::new();
        assert!(a.shape_match(&b).is_ok());
    }

    #[test]
    fn shape_match_fails_for_mismatched_counts() {
        let mut a = Powl64::new();
        let b = Powl64::new();
        let iri = GraphIri::from_iri("http://example.org/x").unwrap();
        a.extend(&iri, 1);
        assert!(a.shape_match(&b).is_err());
    }
}
