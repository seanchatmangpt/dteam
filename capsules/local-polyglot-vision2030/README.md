# Local Polyglot Vision 2030 Capsule

Dependency-free local acceptance capsule for the DTeam combinatorial capability graph.

## Local execution

```bash
./run-local.sh
```

The runner compiles or executes every runtime observed in the working container: Python, Node.js, C, C++, Go, Java, Ruby, Perl, PHP, and Bash orchestration. It validates the same canonical subject in each language, exhaustively checks the 8,640-profile product, kills two negative controls, and emits `evidence/receipt.json`.

Rust is intentionally not claimed by this capsule because `rustc` and `cargo` were absent from the local container. That boundary is recorded as `BLOCKED_TOOLCHAIN_ABSENT`; no CI evidence substitutes for local execution.

Observed local receipt root: `a0ce0c2c84f58ef1219f34d11178ca9b851a957d4f4d7737f428b8ce27a138a8`.
