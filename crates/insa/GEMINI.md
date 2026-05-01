# INSA: AI Developer Instructions

## 1. Primary DX Tool: `just`
When working in this repository, always use `just` as the primary entry point for Developer Experience (DX) and Quality of Life (QOL) commands. 

- **Do not** write raw shell scripts for workflows.
- **Do not** run raw `cargo` commands for complex tasks if a `just` command exists (e.g., use `just test-jtbd`, `just layout`, `just dx`).
- **Use `just dx`** as the ultimate admission gate to verify the project's structural and semantic integrity.
- If a new workflow or anti-drift gate is required, add it to `cargo xtask` and expose it via the `justfile`.

## 2. Anti-Drift Infrastructure
DX/QOL is not polish. It is anti-drift infrastructure. The system must make the correct path the easy path and the wrong path fail loudly.

## 3. The Byte Law (Semantic Multiplexing)
INSA uses 8-bit lanes (INST8, KAPPA8, Family8, POWL8, CONSTRUCT8) as its core semantic surfaces.
- **Never expand an 8-bit lane to a `u16` or `u32` simply to add more fields.**
- **Need9 means Decompose:** If a closure needs more than 8 fields or states, it must be refactored into sequenced, composed, or hierarchical bytes.
- **Activation can be many; Selection must be one.** (e.g., INST8 can activate `Refuse + Escalate + Inspect`, but `SelectedInstinctByte` must be exactly one-hot or empty).

## 4. ReferenceLawPath Equivalence
The codebase prioritizes the **fastest lawful Rust implementation**. 
- The scalar path defines the semantic law (`ReferenceLawPath`).
- Accelerated paths (SIMD, table lookups, intrinsics, unsafe) are only admitted if they prove strict output equivalence to the ReferenceLawPath.
- Do not introduce `unsafe` unless it is isolated, benchmark-proven, safe-wrapped, and passes Truthforge layout/fuzzing gates.

## 5. In-Memory vs. Wire Layout
Layout authority is not serialization authority.
- `repr(C)` structures like `Cog8Row32` (32-bytes) and `RouteCell64` (64-bytes) define in-memory execution boundaries.
- The `WireV1` format (e.g., `.powl64`) defines the canonical byte encoding for persistence and replay.
- **Never use a raw memory transmute to write or read a file.** All wire encoding must explicitly handle endianness and reserved bytes.

## 6. Vibe Done is Not Done
An AI artifact is not complete because it compiles or generates text.
**Done = closed field + lawful motion + bounded delta + canonical evidence + deterministic replay.**
All implementations must be structurally sound and pass Truthforge adversarial tests (`cargo test -p insa-truthforge`). Mocked or stubbed success logic is considered a defect.
