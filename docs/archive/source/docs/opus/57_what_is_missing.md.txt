# 57 — What Is Missing

## Framing

Honest inventory after 56 docs, 20 unibit crates, and three bench suites.
What the archive claims vs. what is actually on disk, compiling, and
measured. Grouped by priority, not by topic.

The rule: a thing is *missing* if (a) a doc promises it by name, (b)
the code can run without it, and (c) the manifesto or a JTBD depends on
it. If any one of those is false, it's not missing — it's either done
or out of scope.

---

## P1 — Blocks manifesto completion

These are the items where the docs name an artifact, code won't be
complete without it, and a manifesto word (*pinned, branchless, typed,
receipted, narrow*) or a JTBD depends on it.

### 1. POWL8 / POWL64 / Motion crates — the orchestration types
**Where:** docs 45, 46.
**Status:** defined in prose, zero lines of code.
**Why it matters:** without these, the `Orchestrator::step` on which
docs 45/46/56 depend has no AST to walk. Lane admission benches
simulate it sequentially, but the lockstep invariant (Aef ↔ Place,
Seq ↔ Geodesic, Par ↔ Concur, Choice ↔ Fork) is unenforced.
**Cost:** one crate, ~200 lines of enums + `shape_match` validator.

### 2. Orchestrator crate — the scheduler
**Where:** docs 45, 46.
**Status:** prose only. No `MainOrchestrator`, no `LaneOrchestrator`,
no `ReduceBuffer`.
**Why it matters:** without an orchestrator, the 8-core fan-out claim
(35 ns critical path, 228 M motions/sec) is unmeasured — it stays
extrapolated from single-core timings. Doc 46's central claim is
currently a projection.
**Cost:** one crate, `ReduceBuffer` + thread-pinned dispatcher + one
integration bench. Needs std for thread pinning; can be std-feature-
gated.

### 3. `LanePolicy<const MODE: LaneMode>` monomorphization
**Where:** flagged in the 8ⁿ bench and geometry-JTBD bench (doc 56
Job 6).
**Status:** `lane::judge` lands at **20 ns** when the budget is < 2 ns.
**Why it matters:** JTBD 6 (attribute a denial inline) is the one JTBD
that misses budget by an order of magnitude. Doc 48 prescribed the
fix: make `LaneMode` a const generic so the four-variant match
collapses at monomorphization.
**Cost:** one API refactor in `unibit-lane`, ~50 lines. Follow with
a bench to confirm sub-2 ns.

### 4. 8-core reduce, actually run on 8 cores
**Where:** doc 46.
**Status:** `WatchdogShards`, `ReduceBuffer`, `LaneHotRegion` don't
yet run against real thread pinning.
**Why it matters:** 8⁵ misses budget 2×, 8⁶ misses 2× — both projected
to land on budget under real parallelism. The parallel claim is
unverified.
**Cost:** one integration bench with `std::thread::spawn` + `pthread`
affinity; ~150 lines including the thread-to-core mapping.

---

## P2 — Quality gates

Things every shipping substrate has that we don't yet.

### 5. Variance CI assertion
**Where:** doc 53 — `assert!(p99_9 / min < 1.10)` as a bug detector.
**Status:** idea only; Criterion `--quick` doesn't compute p99.9.
**Why it matters:** without this, a silent pinning regression passes
CI.
**Cost:** swap `--quick` for a full sample run on one hot bench + a
JSON-parsing CI step.

### 6. `cargo deny check` in CI
**Where:** `deny.toml` was written.
**Status:** never run.
**Why it matters:** the license/source/banned-version guarantees in
`deny.toml` aren't enforced if no job runs the tool.
**Cost:** one CI line.

### 7. Asm verification of new crates
**Where:** `unibit-asmcheck` + `cargo xtask asm` exist.
**Status:** never run against `unibit-lane`, `unibit-watchdog`,
`unibit-globe`, `unibit-residence`, `unibit-cabi`, `unibit-ring`,
`unibit-verify`.
**Why it matters:** the branchless claim is measured at the clock but
not verified at the asm. If the compiler chooses a branching
instruction mix, the variance test would eventually catch it, but asm
catches it earlier.
**Cost:** six xtask asm entries.

### 8. `cargo doc --workspace` clean
**Where:** every crate has `#![deny(missing_docs)]`.
**Status:** unverified across the workspace.
**Why it matters:** the `deny(missing_docs)` lint should already catch
gaps, but cross-crate link resolution is only verified by `cargo doc`.
**Cost:** one workflow step.

### 9. Exported `#[no_mangle]` symbol verification
**Where:** `unibit-cabi::unibit_motion_tick`.
**Status:** compiles; not confirmed it lands in `nm`/`objdump` output.
**Why it matters:** the single C-ABI contract of doc 52 is moot if the
symbol isn't exported when the crate is linked as a staticlib.
**Cost:** one integration test that links `unibit-cabi` into a cdylib
and greps `nm`.

---

## P3 — Docs vs code drift

### 10. Retroactive rename of docs 45, 46
**Where:** docs 47/48 renamed `Cowboy → Runner → step()`,
`Loa → LanePolicy`, `Finn → SpscRing`, etc.
**Status:** docs 45 and 46 still contain the Gibson names.
**Why it matters:** a new reader following the doc chain hits the
renames mid-stream. Not load-bearing, but noisy.
**Cost:** two file edits.

### 11. The three closing measurements from doc 39
**Where:** doc 39 listed `40_smoke_build.md`,
`41_benchmark_run.md`, `42_receipt_roundtrip.md` as "the next three
commits, not the next three documents."
**Status:** we've done the equivalent commits (compile-clean workspace,
bench output reported) but never folded the measurement into a
closing doc.
**Why it matters:** the archive's closing promise is "measurement
folded back into the priming corpus." Without the closing docs, doc
39's promise is partially unkept.
**Cost:** three short docs summarising what was measured.

### 12. AtomVM NIF shim
**Where:** doc 52 named AtomVM as the external boundary.
**Status:** `unibit-cabi` has the Rust-side C-ABI; no Erlang-side
NIF glue. No example downstream linker pulling staticlib + providing
panic_handler.
**Why it matters:** the two-clock boundary claim is structurally
correct but not wired. A reader can't see the boundary in code.
**Cost:** one small Erlang NIF example + a `cdylib` consumer crate
with a panic handler.

---

## P4 — Conceptual unfinished

Items that docs 43–48 named for completeness but that are not yet
load-bearing.

### 13. Recursive `Snapshot` with `.inner`
**Where:** docs 44, 48 (Envelope / Aleph).
**Status:** `UCausalReceipt` is flat; recursion not implemented.
**Why it matters:** the Aleph recursion enables nested Constructs
inside Constructs. Useful for prototype libraries that contain
prototype libraries.
**Cost:** one type + one seal-rewrap step.

### 14. `SignedSnapshot` and `Endpoint` trait
**Where:** doc 48 (Cartridge, Turner/Relay).
**Status:** not implemented.
**Why it matters:** required for cross-zaibatsu transfer. Until someone
wants to ship a Construct between endpoints, these are speculative.
**Cost:** two traits + a `transfer` fn; ~100 lines.

### 15. Chain-analyzer, Orphan-assembler crates
**Where:** doc 48 (Marly → `analyze`, Boxmaker → `assemble`).
**Status:** not implemented.
**Why it matters:** the forensic and repair surfaces. Not needed for
first release; needed for operations.
**Cost:** two crates, each ~100 lines; each needs a real orphan
fragment pool to be meaningfully measured.

---

## P5 — Performance ceiling (past the manifesto targets)

Things that would push numbers *below* the manifesto budgets.

### 16. NEON-specific kernels
**Where:** doc 34 item 4.
**Status:** `portable_simd` is enabled; NEON-specific intrinsics are
not emitted deliberately.
**Why it matters:** the autovectorizer is good; it is not always
optimal. For 8⁵/8⁶ sweeps, hand-written NEON could close the 2×
gap on single-core.
**Cost:** `#[target_feature(enable = "neon")]` on 3–4 kernels; measure.

### 17. Superinstruction fusion measurement
**Where:** doc 34 item 3 (the fused `admit_commit_emit_fragment`).
**Status:** `super_admit_commit_fragment_t0` exists in
`unibit-hot::t0` and lands at 2.7 ns through the FFI. Not measured
against the non-fused three-step alternative.
**Why it matters:** the "one fused superop vs. three sequential ops"
claim is currently only asserted, not shown.
**Cost:** one bench delta.

### 18. Instruments cache-counter capture
**Where:** doc 53.
**Status:** wall-clock measured; hardware counters (`L1D.miss`,
`DTLB.miss`, `BR_MIS_PRED`) not captured.
**Why it matters:** at sub-1 ns per op, wall-clock loses resolution;
cache counters are the honest signal. Also required for the "L1D hit
rate > 99.9%" claim.
**Cost:** one `xctrace` / `Instruments` run with the `time-profiler`
and `cpu counter` templates. macOS-specific.

---

## What is NOT missing

Honest distinction — items doc readers might expect to be gaps but
aren't:

- **Kernel primitives** — `admit3`, `admit4`, `bool_mask`,
  `commit_masked`, `UDelta`, `UReceipt`, `verify_receipt_chain`,
  `execute_step` all real and tested.
- **L1Region pinning** — `Pin<Box<L1Region>>` + `mlock` + size
  assertion + position receipt all in place.
- **Typed UInstr / UMotion** — `unibit-unios::UMotion<Pending|Hot|Spent>`
  typestate, `admit_motion`, `execute_motion` with full tests.
- **8-field branchless admission** — `admit8_t0` at 1.43 ns, working
  against the canonical `PackedEightField`.
- **BLAKE3 receipt** — `unibit-causality::UCausalReceipt` with
  verifier, genesis, and chain helper.
- **Two receipt chains, two verifiers** — `UReceipt` (FNV-1a) and
  `UCausalReceipt` (BLAKE3) both verifiable.
- **Workspace compile clean** — 20 crates, 18 warnings in pre-existing
  code, 0 in the 7 crates added this round.

---

## Priority ship list

If tomorrow is the ship day and only four items fit:

```
1. #3 LanePolicy<const MODE>          — closes the one budget miss
2. #2 Orchestrator + real 8-core run  — proves the multi-core claim
3. #5 Variance CI assertion           — catches regressions
4. #10 Rename docs 45, 46             — closes the naming drift
```

Everything below the line is legitimate future work, not pre-ship work.

---

## The sentence

**What is missing, honestly, is the POWL8/POWL64 AST crates that would
let the Orchestrator actually run lockstep on 8 cores and turn the
single-core 10.3 µs 8⁶ measurement into the 35 ns parallel critical
path doc 46 predicts; the `LanePolicy<const MODE>` monomorphization
that would pull `lane::judge` from 20 ns back under the 2 ns budget;
a CI variance assertion that would catch pinning regressions; and
two documentation renames — everything else is either shipped, not
blocking, or belongs to a future release.**
