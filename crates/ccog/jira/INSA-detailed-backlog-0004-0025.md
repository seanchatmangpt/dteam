# INSA BACKLOG: DETAILED TECHNICAL SPECIFICATIONS (INSA-0004 to INSA-0025)

## INSA-0004: Implement JSON-LD Evidence Projection
- **Strategy:** Extend `unrdf` template crate to iterate over the `POWL64` evidence chain.
- **Implementation:** 
    - Add a `jsonld` module to `insa-proof`.
    - Map `RouteCell64` fields to `prov:Activity`, `prov:wasAssociatedWith`, and `prov:generated`.
    - Accept a `Truthforge` key to sign the JSON-LD `@context`.
- **Criteria:** Canonical JSON-LD output must pass validation against the public PROV-O schema.

## INSA-0005: Harden POWL8 Motion against Cyclic Graphs
- **Strategy:** Implement a stack-based loop detector inside `insa-kernel`.
- **Implementation:** 
    - Maintain a bitmask of `visited_node_ids` per closure epoch.
    - Before firing `Cog8Row`, check the current `NodeId` against the mask.
    - If hit, return `InstinctByte::ESCALATE` with `Powl8Error::CycleDetected`.
- **Criteria:** Integration test `test_powl8_cycle_rejection` must abort in < 1ms.

## INSA-0006: Cryptographic Signing of RouteCells
- **Strategy:** Integrate `ed25519-dalek` into the `insa-proof` crate.
- **Implementation:**
    - Reserve 32 bytes in `RouteCell64` for the signature.
    - Sign the first 32 bytes (data) and write signature to the final 32 bytes.
    - Add `verify_signature()` method to `RouteCell64`.
- **Criteria:** Any cell with a failed signature must return `AdmitStatus::ByzantineFault`.

## INSA-0007: Implement SIMD-Accelerated Cog8 Evaluation
- **Strategy:** Port `execute_cog8_graph` to `core::arch::x86_64::_mm256_or_si256`.
- **Implementation:**
    - Use `unsafe` blocks gated by the layout proof.
    - Mask inputs for 256-bit parallel bitwise operations.
- **Criteria:** Micro-benchmark `cog8_simd_bench` must show >= 4x throughput vs. scalar implementation.

## INSA-0008: Formalize Denial-of-Service Triage (Arena Exhaustion)
- **Strategy:** Implement branch amputation in `insa-kappa8`.
- **Implementation:** 
    - Track `branch_depth` in `ReconstructDendral`.
    - If `branch_depth > 8`, return `DendralStatus::Quarantine`.
- **Criteria:** Chaos test must prove the system continues to process valid routes while a malicious branch is quarantined.

## INSA-0009: Add Witness-Based Replay Verification
- **Strategy:** Extend `insa-replay` to validate route receipts.
- **Implementation:** 
    - Read `FusionWitnessId` from `RouteCell64`.
    - Cross-reference with the `EvidenceWitness` store.
- **Criteria:** `insa-replay` must output `ReplayVerdict::Valid` or `ReplayVerdict::Tampered` based on witness hash mismatch.

## INSA-0010: Establish Truthforge Regression Corpus
- **Strategy:** Create a test-only crate `insa-adversarial-corpus`.
- **Implementation:**
    - Store poisoned payloads as `.bin` files.
    - CI pipeline loads and runs them against `Truthforge`.
- **Criteria:** All 10+ historically known vulnerabilities must fail the current CI test suite.

## INSA-0011: Develop Admissible Doctor Snapshot Protocol
- **Strategy:** Create a `doctor::Snapshot` struct.
- **Implementation:**
    - Iterate the `Blackboard` and active `Powl64` segments.
    - Serialize to a single `Snapshot` file with a Truthforge manifest.
- **Criteria:** Snapshot must be verifiable using `insa-replay --verify snapshot.bin`.

## INSA-0012: Implement Semantic Conflict Resolution Matrix
- **Strategy:** Replace bitwise union with a lookup table (LUT) of 256 entries.
- **Implementation:**
    - Create `const CONFLICT_LUT: [[u8; 8]; 8]`.
    - Rewrite `union` to perform `LUT[self.bits()][other.bits()]`.
- **Criteria:** LUT must strictly enforce that `ESCALATE` > `SETTLE`.

## INSA-0013: Harden WireV1 against Byte-Length Invariance
- **Strategy:** Implement explicit framing.
- **Implementation:**
    - Every WireV1 packet must be prefixed with a 4-byte `len` field.
    - If `packet.len != decoded.len`, reject.
- **Criteria:** Injection test must fail to consume partial packets.

## INSA-0014: Build Truthforge Adversarial Fuzzer
- **Strategy:** Leverage `libFuzzer` via `cargo-fuzz`.
- **Implementation:** 
    - Add `fuzz_target_1.rs` to fuzz the `COG8` graph input.
- **Criteria:** Fuzzer must find 0 crashes after 10 billion cycles.

## INSA-0015: Optimize POWL64 Memory Footprint
- **Strategy:** Aggressive packing.
- **Implementation:** 
    - Audit all struct members for bit-fields (`bitflags` crate).
    - Reduce 32-byte reserved padding to absolute minimum.
- **Criteria:** Final size must remain exactly 64 bytes (cache line boundary).

## INSA-0016: Implement Telco Channel Fidelity Tests
- **Strategy:** Wire-tap simulation.
- **Implementation:** 
    - Create `TestWire` trait with `send` / `receive` methods.
    - Simulate latency, packet drops, and reorders.
- **Criteria:** `insa-telco` must detect and correct or reject for all injected channel faults.

## INSA-0017: Define Stable Domain-Driven Exit Codes
- **Strategy:** Use `thiserror` for internal errors and `clap::Parser` exit logic.
- **Implementation:**
    - Create `ExitStatus` enum.
    - Map to `i32` codes: `0=Success`, `1=Blocked`, `2=Unknown`, `10=Config`, `20=Byzantine`.
- **Criteria:** `insa-cli` documentation must explicitly list every code.

## INSA-0018: Add Policy Epoch Validation Gate
- **Strategy:** Epoch-sequencing.
- **Implementation:** 
    - Add `policy_epoch: u64` to `Truthforge` manifest.
    - Reject any rule with `rule.epoch < global.epoch`.
- **Criteria:** Replay test of stale policy must return `Rejected::StaleEpoch`.

## INSA-0019: Construct In-Memory Graph Query Engine
- **Strategy:** Index-based adjacency lists.
- **Implementation:** 
    - `nodes: Vec<Node>`, `edges: Vec<Vec<EdgeIndex>>`.
- **Criteria:** All graph traversals must be $O(V+E)$ and heap-free in the hot path.

## INSA-0020: Implement Admissible Wizard Scaffolds
- **Strategy:** Integrate `hygen` templates.
- **Implementation:**
    - Create `templates/domain_scaffold.ejs.t`.
    - Ensure `wizard init <noun>` calls this template.
- **Criteria:** `wizard init doctor` must result in a compilable `insa-doctor` domain crate.

## INSA-0021: Add Telemetry Attribution to Instincts
- **Strategy:** Use thread-local storage for attribution.
- **Implementation:** 
    - `thread_local! { static ATTRIBUTION: RefCell<Vec<InstinctByte>> }`
- **Criteria:** All emitted instincts must appear in the evidence route.

## INSA-0022: Implement Semantic Alignment Checksum
- **Strategy:** SHA-256 of the dictionary.
- **Implementation:** 
    - `dictionary_hash` in `Truthforge`.
- **Criteria:** Mismatched endpoint dictionaries must return `Rejected::AlignmentMismatch`.

## INSA-0023: Optimize KAPPA8 Dispatch Logic
- **Strategy:** Jump table implementation.
- **Implementation:** 
    - `const KAPPA_DISPATCH: [fn(..); 8] = [reflect, precondition, ..];`
- **Criteria:** Micro-benchmark must show constant time dispatch regardless of mask contents.

## INSA-0024: Harden CLI against Resource Starvation
- **Strategy:** `Tokio` timeout wrappers.
- **Implementation:** 
    - Every noun command wrapped in `tokio::time::timeout(Duration::from_secs(5), ..)`.
- **Criteria:** CLI must terminate exactly at 5s with exit code 4.

## INSA-0025: Implement Truthforge Proof-of-Closure
- **Strategy:** Encode POWL8 path as ZK-SNARK or simple hash chain.
- **Implementation:** 
    - Accumulate hashes: `H_{i+1} = Hash(H_i, Cog8Row)`.
- **Criteria:** `Replay` must be able to independently derive the final state from the `RouteCell` chain.
