# INSA: DX/QOL Instructions

## Primary DX Tool: `just`
When working in this repository, always use `just` as the primary entry point for Developer Experience (DX) and Quality of Life (QOL) commands. 

- **Do not** write raw shell scripts for workflows.
- **Do not** run raw `cargo` commands for complex tasks if a `just` command exists (e.g., use `just test-jtbd`, `just layout`, `just dx`).
- **Use `just dx`** as the ultimate admission gate to verify the project's structural and semantic integrity.
- If a new workflow or anti-drift gate is required, add it to `cargo xtask` and expose it via the `justfile`.

## Anti-Drift Infrastructure
DX/QOL is not polish. It is anti-drift infrastructure. The system must make the correct path the easy path and the wrong path fail loudly.
