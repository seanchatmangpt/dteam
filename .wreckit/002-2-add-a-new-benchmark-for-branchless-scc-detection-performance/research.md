I will search for any existing SCC (Strongly Connected Components) detection code or references in the codebase to understand the current state of graph algorithms.
I will read the current SCC implementation in `src/utils/scc.rs` to assess its design and performance characteristics.
I will search the `src/utils` directory for any additional SCC-related implementations or graph algorithms that might be relevant.
I will examine the `benches/` directory to see if there are any existing graph or SCC benchmarks that I can use as a reference.
I will read `benches/algorithm_bench.rs` to see how algorithms are currently benchmarked and if any SCC tests are already present.
I will check `benches/bcinr_primitives_bench.rs` to see if there are any low-level benchmarks that could serve as a model for SCC performance testing.
I will examine `benches/instruction_stability_bench.rs` to see if it includes any benchmarks related to branchless execution or instruction-level performance.
I will re-examine `src/utils/scc.rs` and search for any other SCC-related functions or primitives in `src/utils/dense_kernel.rs` that could support branchless implementation.
I will search for "branchless" across the entire codebase to identify existing patterns and implementations that might serve as a reference for a branchless SCC algorithm.
I will read `src/utils/bitset.rs` to find branchless bitset primitives that could be used to optimize the SCC implementation.
I will read `benches/ktier_scalability_bench.rs` to see if there are any graph scaling benchmarks that might be relevant for my SCC performance research.
I will search for "Warshall" or "Floyd" in the codebase to see if there are other transitive closure or graph algorithm implementations.
I will examine `src/powl/discovery.rs` to understand the context and graph characteristics of SCC usage in the POWL discovery process.
I will search for "tarjan" in the `src` directory to see if there is an alternative SCC implementation that might be relevant for my benchmark.
I will check `Cargo.toml` to see the project's dependencies and understand its relationship with the `bcinr` crate.
I will search for `select_u64` usage in the `src` directory to see how this branchless primitive is applied in the codebase.
I've completed my research on the SCC detection logic and its relationship to the "branchless" design philosophy of the project. I've identified the current implementation, its limitations regarding the "branchless" goal, and the necessary steps to implement and benchmark a truly branchless version.

I will now generate the `research.md` report.

# Research Report: Branchless SCC Detection Performance Benchmark

## 1. Overview
The `dteam` project emphasizes nanosecond-scale, branchless execution for determinism and performance. SCC (Strongly Connected Components) detection is a critical component of the "Branchless Partitioning" strategy used in the Inductive Miner (`mine_powl`). This research evaluates the existing SCC implementation and outlines a strategy for adding a performance benchmark that compares current and future (branchless) versions.

## 2. Existing SCC Implementation
The current SCC detection logic is located in `src/utils/scc.rs`:
- **Algorithm**: Uses a transitive closure approach based on Warshall's algorithm.
- **Complexity**: $O(N^3)$ where $N$ is the number of nodes (constrained by `K-Tier` size).
- **Branchless Status**: Labeled `// Transitive Closure (Branchless)`, but contains data-dependent branches:
    ```rust
    if r[i].contains(k) {
        let k_mask = r[k];
        r[i] = r[i].bitwise_or(k_mask);
    }
    ```
- **Usage**: Primary consumer is `src/powl/discovery.rs` for finding SEQUENCE cuts.

## 3. Benchmarking Architecture
The project utilizes multiple benchmarking tools:
- **Criterion**: Used for general algorithm benchmarks (e.g., `benches/algorithm_bench.rs`).
- **Divan**: Used for low-level primitive benchmarks (e.g., `benches/bcinr_primitives_bench.rs`).
- **iai-callgrind**: Used for instruction-level stability and branch count verification (e.g., `benches/instruction_stability_bench.rs`).

## 4. Proposed Strategy for SCC Benchmark
A new benchmark `benches/scc_bench.rs` should be created with the following goals:
1.  **Synthetic Graph Generation**: Functions to generate random directed graphs represented as `KBitSet` adjacency matrices with varying densities (10%, 30%, 50%, 90%).
2.  **K-Tier Scalability**: Benchmark across different `WORDS` sizes (e.g., 1, 2, 4, 8) to reflect $K \in \{64, 128, 256, 512\}$.
3.  **Baseline vs. Branchless**: Compare the current `compute_sccs_generic` against a newly implemented `compute_sccs_branchless` that uses mask calculus to eliminate branches.
4.  **Instruction Stability**: Integrate with `iai-callgrind` to prove that the branchless version has constant branch counts regardless of graph density.

## 5. Branchless SCC Optimization Path
The transitive closure inner loop can be transformed into a pure branchless operation:
```rust
for k in 0..max_nodes {
    let k_mask = r[k];
    for i in 0..max_nodes {
        // Compute mask based on whether r[i] contains k without 'if'
        let bit = (r[i].words[k >> 6] >> (k & 63)) & 1;
        let mask = bit.wrapping_neg(); // 0 -> 0, 1 -> u64::MAX
        
        // Branchlessly OR the k_mask into r[i]
        for w in 0..WORDS {
            r[i].words[w] |= k_mask.words[w] & mask;
        }
    }
}
```

## 6. Actionable Integration Points
- **Files to Create**: `benches/scc_bench.rs`.
- **Files to Modify**: 
    - `Cargo.toml`: Register the new benchmark.
    - `src/utils/scc.rs`: Add the optimized branchless variant.
- **Reference Symbols**: `select_u64` in `src/utils/bitset.rs`, `KBitSet` in `src/utils/dense_kernel.rs`.

## 7. Next Steps
1.  Implement a mock `scc_bench` to confirm the graph generation logic.
2.  Port the optimized branchless loop to `scc.rs`.
3.  Run `cargo bench --bench scc_bench` and `cargo bench --bench instruction_stability_bench` to verify the performance gains and branchless purity.

---
**Prepared by**: Gemini CLI (Autonomous Agent)
**Date**: April 19, 2026

I will write the `research.md` file now.
I will now compile my findings into a detailed `research.md` plan document.
I will now exit plan mode and present the generated `research.md` report for approval.
I have completed the research for adding a new benchmark for branchless SCC detection performance. My findings are summarized in the detailed `research.md` report now available in the project's plans directory.

### Key Research Findings:
- **Current Implementation:** SCC detection in `src/utils/scc.rs` uses a transitive closure (Warshall's) algorithm. Although labeled "branchless," it contains data-dependent `if` statements that introduce stochasticity.
- **Branchless Strategy:** I identified a path to a 100% branchless implementation using bitwise mask calculus (e.g., `bit.wrapping_neg()`) to eliminate conditional branching in the inner loop.
- **Benchmark Plan:** I've proposed a new benchmark, `benches/scc_bench.rs`, that will utilize `Criterion` and `iai-callgrind` to compare the baseline against the branchless version across various `K-Tier` sizes and graph densities.
- **Integration:** The benchmark will generate synthetic graphs using `KBitSet` and measure both execution time and instruction stability (branch count invariance).

The full report can be found at: `.gemini/tmp/dteam/f7a00af4-34c0-4a56-a227-0197e118622d/plans/research.md` (or simply `research.md` in the plans folder).
