# Porting Plan: Achieving Independence for wasm4pm

## Objective
Migrate core data models, conformance algorithms, and utility functions from `pm4py-rust` to `wasm4pm` to achieve independent execution.

## Scope
- Port core data models (`EventLog`, `PetriNet`).
- Implement the Conformance Engine (`TokenReplay`).
- Migrate necessary IO/Utilities for XES support and JS interop.

## Implementation Steps

### Phase 1: Core Models
- **File:** `src/models/mod.rs`
- **Action:** Define `EventLog`, `Trace`, `Event`. Implement standard getters/setters and trait derivations (Serialize/Deserialize).
- **File:** `src/models/petri_net.rs`
- **Action:** Define `PetriNet`, `Place`, `Transition`, `Arc`. Include marking state management.

### Phase 2: Conformance Kernel
- **File:** `src/conformance/mod.rs`
- **Action:** Implement token replay logic (as previously discussed), using the local `models` definitions for input.
- **Action:** Ensure WASM-native state management (using `RefCell` where necessary).

### Phase 3: Utilities & IO
- **File:** `src/io/xes.rs`
- **Action:** Port XES parsing logic to handle external data ingestion without `process_mining` crate dependencies.
- **File:** `src/utils/mod.rs`
- **Action:** Port attribute handlers and WASM JS-interop helper functions.

### Phase 4: Verification & Integration
- **Action:** Re-run reinforcement learning tests to ensure compatibility with newly ported models.
- **Action:** Validate conformance output against existing `lab/reports/nodejs-conformance.json` results to ensure zero-regression.

## Verification
- Unit tests for `EventLog` and `PetriNet` instantiation.
- Test suite verifying token replay consistency on `test_simple.xes`.
- Audit verify the final implementation against reference outputs from `pm4py-rust`.
