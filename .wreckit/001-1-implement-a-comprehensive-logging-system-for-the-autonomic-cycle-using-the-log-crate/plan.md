# Implementation Plan: Comprehensive Logging System for Autonomic Cycle

## 1. Objective
Integrate the `log` crate to provide a comprehensive and robust logging system across the autonomic cycle in the `dteam` project. This replaces `println!` and provides granular observability into state transitions, inference, and execution logic.

## 2. Key Files & Context
- `Cargo.toml`: Dependency management.
- `src/autonomic/kernel.rs`: Core trait (`AutonomicKernel`), cycle orchestration (`run_cycle`), and `DefaultKernel` implementation.
- `src/autonomic/vision_2030_kernel.rs`: Advanced context-bandit and MCTS implementation.
- `examples/autonomic_runner.rs`: Example runner needing logger initialization.

## 3. Implementation Steps

### Step 1: Add Dependencies
Modify `Cargo.toml` to include the required crates.
- Add `log = "0.4"` to `[dependencies]`.
- Add `env_logger = "0.11"` to `[dev-dependencies]` for testing and examples.

### Step 2: Instrument `AutonomicKernel::run_cycle`
Update `src/autonomic/kernel.rs` to include logging at high-level phase transitions inside `run_cycle`.
- Add `log` imports: `use log::{info, debug, warn, error};`.
- Wrap the entire cycle flow with logging statements:
    - **INFO**: "Starting autonomic cycle..."
    - **DEBUG**: Log the inferred state.
    - **INFO**: Log the number of proposed actions.
    - **WARN**: Log when an action is rejected.
    - **INFO**: Log when an action is accepted and executed, along with the result.
    - **INFO**: "Cycle complete. Manifest hash: {hash}"

### Step 3: Instrument Core Kernels
Add contextual logs within specific methods of both `DefaultKernel` (`src/autonomic/kernel.rs`) and `Vision2030Kernel` (`src/autonomic/vision_2030_kernel.rs`).
- `observe`: **DEBUG** level logs for incoming payloads, specific metrics updates (e.g., OCPM binding frequencies).
- `infer`: **DEBUG** level logs for calculated health, throughput, and conformance metrics.
- `propose`: **DEBUG** and **INFO** level logs describing logic for deciding on an action, highlighting MCTS/Bandit context in `Vision2030Kernel`.
- `accept`: **INFO** and **WARN** level logs detailing why actions are accepted or rejected based on safety/soundness guards.
- `execute`: **DEBUG** and **INFO** level logs capturing state mutation, execution latency, and conformance delta.
- `adapt`: **DEBUG** logs representing feedback integration, reward adaptation, and health progression.

### Step 4: Refactor Examples and Testing
Update `examples/autonomic_runner.rs` to initialize the logging framework.
- Import `log` macros.
- Add `env_logger::init();` at the beginning of `main()`.
- Replace instances of `println!` with `info!` or `debug!` as appropriate.

## 4. Verification & Testing
1. **Compilation**: Run `cargo check` to ensure `log` usage is correct and no typing issues occur.
2. **Tests**: Run `cargo test` to ensure tests compile. Tests will remain quiet by default.
3. **Execution**: Run the `autonomic_runner` example using the `RUST_LOG` environment variable:
   `RUST_LOG=info cargo run --example autonomic_runner`
   `RUST_LOG=debug cargo run --example autonomic_runner`
   Verify that logs are correctly formatted and printed without crashing.
4. **Performance Validation**: (Optional but recommended) Run `cargo bench` to confirm that the `log` crate macros (especially `debug!` and `trace!`) do not introduce measurable latency overhead on the zero-heap hot paths. Ensure any expensive formatting parameters inside logs are conditionally checked via `log_enabled!` if strictly necessary.

## 5. Migration & Rollback Strategies
- **Rollback**: To rollback, simply remove the `log` and `env_logger` dependencies from `Cargo.toml` and run `git revert` or perform a hard reset on `src/autonomic` and `examples/`.
- **WASM Compatibility**: The `log` crate is naturally compatible with WASM. Applications interacting with WASM will need to hook a web-compatible backend (like `wasm-logger` or `console_error_panic_hook`) on the client side, but the Rust codebase itself only relies on the abstract `log` macros.