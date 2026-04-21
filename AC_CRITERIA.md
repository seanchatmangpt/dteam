# AC_CRITERIA: DDS-Grade LinUCB Implementation

## 1. Objective
Implement a linear reinforcement learning agent (LinUCB) that adheres to the **Deterministic Data Science (DDS)** paradigms: zero-heap, branchless execution, and deterministic transformation kernel identity ($Var(\tau) = 0$).

## 2. Acceptance Criteria

### AC 1: Zero-Heap State Matrices
- **Requirement:** All state variables for LinUCB ($A^{-1} \in \mathbb{R}^{D \times D}$ and $b \in \mathbb{R}^D$) must be stack-allocated.
- **Verification:** 
    - Use `const` generics for dimensions ($D, D^2$).
    - Zero runtime heap allocations in `select_action` and `update` paths.
    - Verified by `dhat` or manual inspection of MIR/LLVM IR.

### AC 2: Branchless Decision Kernel
- **Requirement:** Arm selection must use bitwise mask calculus to identify the optimal action.
- **Verification:**
    - Replace `if/else` or `max_by` with `bcinr`-style `select_f32` or equivalent mask-based comparisons.
    - Constant execution time (latency jitter $\approx 0$) regardless of input context.

### AC 3: Non-Allocating Feature Projection
- **Requirement:** `WorkflowState::features` (or its equivalent in the LinUCB hot path) must return features without heap allocation.
- **Verification:**
    - Refactor `WorkflowState` to provide a fixed-size array or reference to a stack-allocated buffer.
    - No `Vec<f32>` generation during feature extraction.

### AC 4: Deterministic Transformation Kernel (μ)
- **Requirement:** The state update $S_{t+1} = \mu(S_t, x_t, r_t)$ must be perfectly deterministic across all targets (x86_64, WASM).
- **Verification:**
    - $Var(\tau) = 0$ for any fixed input trajectory $\tau = \{(x_i, r_i)\}$.
    - State hash $H(A^{-1}, b)$ must be bit-identical across runs with identical inputs.

### AC 5: Admissibility and Stability
- **Requirement:** $A^{-1}$ must remain positive semi-definite to prevent matrix collapse or numerical instability.
- **Verification:**
    - Diagonal protection (e.g., `a_inv[i][i] = max(a_inv[i][i], min_eigen)`).
    - Hard bounds on all matrix/vector components to prevent overflow/underflow in long-running autonomic loops.

### AC 6: Execution Provenance (Manifest)
- **Requirement:** The LinUCB agent must emit a compliant `ExecutionManifest` for auditability.
- **Verification:**
    - Inclusion of input log hash $H(L)$, action trajectory $\pi$, and output model hash $H(N)$.
    - Reproducibility: Re-running with the manifest must yield an identical artifact.

### AC 7: Structural Minimality (MDL)
- **Requirement:** The linear model complexity must be minimized relative to the discovery accuracy.
- **Verification:**
    - Enforce $\min \Phi(N)$ where $N$ is the discovered process model guided by LinUCB.

## 3. Verification Strategy
- **Property-Based Testing:** Use `proptest` in `src/reinforcement_tests.rs` to verify $Var(\tau) = 0$ across $10^5$ iterations.
- **Zero-Heap Audit:** Use `cargo test` with a check for heap allocations to assert the hot path is truly zero-heap.
- **Manifest Playback:** Implement a reproduction test in `src/dteam/orchestration.rs` that verifies bit-identical model generation from an `ExecutionManifest`.
- **Latency Benchmark:** Profile `src/ml/linucb.rs` with `criterion` to ensure sub-microsecond latency with zero variance.
