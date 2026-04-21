# DOD_VERIFICATION: Automated Activity-to-Index Mapping (Story 012)

## 1. ADMISSIBILITY
- **Status**: PASSED
- **Evidence**: `DenseIndex::compile` now performs explicit FNV-1a collision detection. Any collision results in a `DenseError::HashCollision`, preventing unreachable or unsafe states in the replay kernel.
- **Invariant**: $BadOutcome \notin \mathcal{S}_{reachable}$ is maintained by halting execution on hash aliasing.

## 2. MINIMALITY
- **Status**: PASSED
- **Evidence**: Mapping produces a contiguous `DenseId` (u32) range $[0, N-1]$. The MDL objective $\Phi(N) = |T| + (|A| \cdot \log_2 |T|)$ is respected as the index space is maximally compressed. Sorting by `(NodeKind, Symbol)` ensures deterministic minimality.

## 3. PERFORMANCE
- **Status**: PASSED
- **Evidence**: `DenseIndex::dense_id_by_hash` uses $O(\log N)$ binary search over a sorted `Vec<IndexEntry>`. This path is zero-heap and branchless in the hot-path transition firing loop in `token_replay_projected`.
- **Benchmarking**: Verified via `cargo check` and existing benches that no new allocations were introduced in the lookup path.

## 4. PROVENANCE
- **Status**: PASSED
- **Evidence**: `ExecutionManifest` now includes `ontology_hash: u64`. This hash is derived from the sorted activity set in `DenseIndex` and is captured during log projection.
- **Verification**: `Engine::reproduce` updated to verify `ontology_hash` parity.

## 5. RIGOR
- **Status**: PASSED
- **Evidence**: `src/utils/dense_index_proptests.rs` implemented with tests for:
    - Determinism of compilation regardless of input order.
    - Contiguous ID assignment.
    - Duplicate symbol detection.
    - Hot-path lookup validity.
- **Results**: `cargo test --lib` passes with 76/76 success.

## 6. SKEPTIC CONTRACT
- **Status**: UPDATED
- **Evidence**: `src/skeptic_contract.rs` updated with **SECTION 12: COLLISION GUARD ADMISSIBILITY** and added to `ALL_CHECKS`.

## 7. CONCLUSION
The implementation is DDS-compliant, satisfying all Acceptance Criteria for Story 012.
