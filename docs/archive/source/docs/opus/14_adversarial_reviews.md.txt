# 14 — Adversarial Tri-Review: Rust Core / OTP / van der Aalst

## Panel 1 — Rust core team (adversarial performance review)

### Accepted
- 32 KiB truth + 32 KiB scratch geometry
- Active universe as `[u64; 4096]`
- One u64 = 64-place truth/Petri cell
- Branchless transition: `M' = (M & !I) | O`
- Branchless admission algebra with denial polarity

### Rejected

**"Pinning means we no longer worry about memory safety."**
Pinning gives address stability. It does not prove aliasing safety, provenance
safety, thread safety, page residency, or semantic address validity.
Requirement: L1PositionReceipt recording base, truth_offset, scratch_offset,
bytes, align, epoch, kernel_table_hash. No hot kernel runs until this exists.

**"A crates/ workspace is better than src/, but you still need asymmetric dependency physics."**
Required partition:
```
crates/
├── unibit-word, unibit-block, unibit-kernel   (T0/T1, no_std, no_alloc)
├── unibit-geometry, unibit-typestate          (tier/layout)
├── unibit-field                                (L2, alloc behind feature)
├── unibit-planner, unibit-runtime              (visible OS)
├── unibit-delta, unibit-receipt               (evidence)
├── unibit-supervisor                           (off-path)
├── unibit-macros, unibit-asmcheck, unibit-bench, unibit-cli
```

**"You are leaving layout value on the table."**
`#[repr(transparent)]` over u64 is good, but higher-order structures need
explicit `#[repr(C, align(64))]` on UBitBlock, UBitScratch, UCoreLane.

**"Your hot API still has too much generic shape."**
Provide concrete named kernels per U_{1,n} level alongside generic internals.
Exported benchmark targets should use `#[inline(never)]` for stable
symbol-level measurement.

**"You are underusing unsafe."**
Safe Rust at the boundary, audited unsafe in the kernel. `pub unsafe fn admit_ptr_words<const WORDS: usize>(state: *const u64, prereq: *const u64) -> u64`.

**"You are not using target features explicitly."**
Use `#[target_feature(enable = "popcnt"|"bmi2"|"avx2"|"avx512"|"sve")]`. Runtime
feature detection is fine; runtime feature detection in the hot loop is not.

**"Your receipt path is still vague."**
Compile receipt mode into the handle as a type parameter. No runtime branch.
Three tiers: NoReceipt (T0), FragmentReceipt (T1), ChainReceipt (T2).

**"Your benchmark design is under-specified."**
For every T0/T1 benchmark: best-case, cold-ish cache, randomized masks,
all-admitted, all-denied, 50/50, alias-safe, unaligned rejection, cross-core
interference, batch mode, single-call FFI. Report p50/p90/p99/p999, cycles/op,
instructions/op, branch-misses/op, cache-misses/op, allocations/op. Split
carefully.

## Panel 2 — OTP / BEAM / AtomVM core team

### Accepted
- AtomVM as real-world membrane
- DTEAM consuming canonical facts only

### Rejected

**"Native kernels must be bounded, supervised, and isolated."**
AtomVM never trusts native work to be polite. Use Port-style external process,
isolated native worker, dedicated scheduler/core lane, bounded NIF for tiny
calls, dirty/scheduled native for larger calls, crash-isolated adapter.

**"Perfect data must mean canonicalized disorder, not ignored disorder."**
AtomVM emits not only `ObservedFact` but also `LateFact`, `CorrectedFact`,
`MissingFact`, `TimeoutFact`, `ActorRestartFact`, `RejectedFact`,
`CompensatedFact`, `DuplicateFact`, `ClockSkewFact`.

**"Mailbox pressure is semantic."**
Represent as `BackpressureFact`, `DroppedFact`, `RestartedFact`. If you drop,
delay, retry, or reorder external work, that is not an implementation detail.
That is process information.

**"Supervision must not be a dashboard."**
Supervision tree: `WorldAdapterSupervisor` → `ProtocolAdapter`, `Canonicalizer`,
`ClockNormalizer`, `BoundaryFactEmitter`, `NativeUnibitWorker`, `ReceiptForwarder`,
`DTeamBridge`. Every child has restart policy, crash reason, receipt/correction
fact, bounded mailbox strategy.

**Final:** "Let it crash is not a performance strategy. It is a containment
strategy. If the crash is not supervised and canonicalized, it is just a crash."

## Panel 3 — Van der Aalst-style process mining review

### Accepted
- L1 block gives one-word Petri cells and exact XOR/popcount conformance
- 64-place local cell fits in one u64

### Rejected

**"Event logs are not perfect because you want them to be."**
Must define: case notion, event identity, activity identity, timestamp
semantics, lifecycle semantics, resource semantics, ordering semantics, object
relations, late/correction semantics, noise semantics.

**"Accuracy is not enough."**
100% classification accuracy on a dataset suite is not a theory of process
discovery. Demand: fitness, precision, generalization, simplicity, soundness,
language/trace equivalence, alignment cost, structural minimality, duplicate
label handling, invisible transition handling, concurrency preservation, loop
handling, choice handling, multi-object handling.

**"Token replay is not enough."**
Token replay is useful, but alignments matter. Behavioral conformance is not
just missing tokens. DTEAM must eventually implement token replay, alignments,
anti-alignments, precision measures, process tree / Petri / POWL projections,
object-centric conformance.

**"POWL may be upstream; Petri may be execution."**
Accepted. POWL/HPOWL = planning semantics. Petri / bit masks = execution
substrate. BPMN = interoperability projection. Do not collapse all semantics
into Petri bit cells too early.

**"DTEAM must know the semantic contract."**
Canonical ordering, late data, corrections, case/object identity, provenance,
missingness. Without this, "perfect data" becomes magical thinking.

## Composite verdict

**Conditional pass. Do not build DTEAM yet.**

Order forced by review:

1. unibit (substrate fact)
2. unios (admission/proof)
3. AtomVM boundary contract
4. dteam (process intelligence)

Each committee rejected a different hidden assumption:

| Reviewer | Rejected |
|---|---|
| Rust core | "Pinning means we no longer worry about memory safety." |
| OTP core | "AtomVM can clean up the world without poisoning schedulers or semantics." |
| Van der Aalst | "DTEAM can assume perfect data without a formal event-log/canonicalization contract." |

## Composite architecture

```
dteam (canonical process semantics only, no substrate knowledge)
  ↑  CanonicalEvent / CanonicalTrace / CorrectionFact
AtomVM (real-world disorder membrane)

unios (admission/proof/supervision, knows unibit API only)
  ↑  UMotionHandle / UDelta / UReceipt
unibit (sealed bit-motion substrate, pinned L1, assembly-proven)
```

AtomVM is the boundary membrane that manufactures DTEAM's perfect world.

## Each layer's fiction

| Layer | Fiction it is allowed to believe |
|---|---|
| unibit | "Every handle I receive is admitted." |
| unios | "The unibit API is truthful." |
| AtomVM | "Any real-world mess can become a canonical fact." |
| DTEAM | "All input is perfect because imperfections are explicit facts." |
