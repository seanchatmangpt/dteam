I will begin by reading the current implementation of `incidence_matrix` in `src/models/petri_net.rs` and examining the `PetriNet` struct to understand how to best integrate the cached flat representation.
I have completed the research phase and analyzed the current `incidence_matrix` implementation in `src/models/petri_net.rs`. The current implementation is inefficient because it rebuilds indices on every call and returns a jagged `Vec<Vec<i32>>`, which causes excessive heap allocations and poor cache locality.

I have drafted a detailed implementation plan to introduce a `FlatIncidenceMatrix` structure and cache it within `PetriNet` to achieve "nanosecond-scale" performance and prepare for GPU offloading.

```markdown:plans/003-3-optimize-petri-net-incidence-matrix.md
# Implementation Plan: Optimized Petri Net Incidence Matrix

## 1. Objective
Replace the inefficient, heap-heavy `Vec<Vec<i32>>` incidence matrix construction with a flattened 1D representation cached directly within the `PetriNet` struct. This aligns with the "Vision 2030 Roadmap" for zero-allocation hot paths and GPU readiness.

## 2. Proposed Changes

### A. Data Structure: `FlatIncidenceMatrix`
Introduce a new struct in `src/models/petri_net.rs` (or `src/utils/dense_kernel.rs`) to hold the flattened matrix.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatIncidenceMatrix {
    /// Contiguous 1D buffer of incidence values [row-major: places x transitions]
    pub data: Vec<i32>,
    pub places_count: usize,
    pub transitions_count: usize,
}
```

### B. Update `PetriNet` Struct
Add a cached field to `PetriNet` to avoid redundant calculations.

```rust
pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    // ... existing fields ...
    
    /// Cached flat incidence matrix and indices
    #[serde(skip)]
    pub cached_incidence: Option<FlatIncidenceMatrix>,
}
```

### C. Logic Implementation: `compile_incidence()`
Implement a compilation step that:
1. Uses `DenseIndex::compile` to map Place/Transition IDs to stable integer indices.
2. Allocates a single `Vec<i32>` of size `places * transitions`.
3. Populates the matrix by iterating over `arcs` once.
4. Stores the result in `cached_incidence`.

### D. Refactor Callers
- **`incidence_matrix()`**: Update to return a reference to the cached flat matrix or a wrapper that provides the old API if backward compatibility is strictly required (though research suggests limited usage).
- **`verifies_state_equation_calculus()`**: Refactor to iterate over the flat buffer, improving cache locality.

## 3. Execution Steps

1.  **Data Structure Definition**: Add `FlatIncidenceMatrix` and update `PetriNet` in `src/models/petri_net.rs`.
2.  **Indexing Optimization**: Replace the temporary `build_node_index` (which uses `PackedKeyTable`) with a persistent `DenseIndex` during the compilation phase.
3.  **Matrix Construction**: Implement the `compile()` method on `PetriNet`.
4.  **Verification Update**: Refactor `verifies_state_equation_calculus` to use `FlatIncidenceMatrix::get(row, col)`.
5.  **Benchmark & Validation**:
    - Add a new benchmark in `benches/kernel_bench.rs` comparing jagged vs. flat matrix construction.
    - Verify correctness using existing structural workflow-net tests.

## 4. Verification Plan

### Automated Tests
- `cargo test --lib models::petri_net`: Ensure all structural validations still pass.
- Add a parity test to ensure `FlatIncidenceMatrix` produces the same values as the legacy `Vec<Vec<i32>>` implementation.

### Performance Verification
- `cargo bench --bench kernel_bench`: Expect >50% reduction in allocation count and >2x speedup for large Petri nets (>100 nodes).
```

I have fulfilled the directive to provide a detailed implementation plan.
