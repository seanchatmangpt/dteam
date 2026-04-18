# Plan: License-Compliant Porting of Core Data Structures

## Objective
Migrate high-performance event data models from the `rust4pm` vendor directory to the `wasm4pm` source while ensuring full compliance with the MIT/Apache-2.0 licenses.

## Scope
- Port `Attribute`, `AttributeValue`, `Event`, `Trace`, and `EventLog` structures from `vendors/rust4pm/process_mining/src/core/event_data/case_centric/event_log_struct.rs`.
- Ensure all ported code includes proper attribution.
- Maintain existing WASM-compatible trait derivations (`Serialize`, `Deserialize`, etc.).

## Implementation Steps

### Phase 1: Licensing & Attribution Setup
- Create a `LICENSES` file or add an `ATTRIBUTION.md` in the project root.
- Document that the ported data models are derived from `rust4pm` (MIT/Apache-2.0) and include the required license notices.

### Phase 2: Core Data Models Porting
- **Module:** `src/models/mod.rs`
- **Action:** Define the `AttributeValue` enum, `Attribute` struct, `Event` struct, `Trace` struct, and `EventLog` struct.
- **Action:** Retain functionality while pruning unnecessary `polars` or `schemars` dependencies if they are not required for WASM.
- **Action:** Add helper traits (e.g., `XESEditableAttribute`) to maintain feature parity.

### Phase 3: Integration
- Update `src/lib.rs` to expose the new model structures.
- Update `src/conformance/mod.rs` to use the fully defined `Attribute` types instead of the current stubbed versions.

## Verification
- Ensure `cargo check` passes.
- Validate that the data structures retain their `serde` compatibility.
- Perform a license scan to verify that all files with attributed code have appropriate headers.
