# 10 — POWL as the ISA

## The reframe

> "Think about if POWL was the instruction set that operates on the 64³ data
> and the 64³ scratchpad."

Previous framing: POWL v2 as a *strategy grammar* — source language lowered
*into* the 64²/64³ substrate. Incomplete.

New framing: **POWL is not the source language. POWL is the ISA.**

If this is true, then:

- The 64³ TruthBlock is the **register file** of a machine whose instruction set is POWL.
- The 64³ Scratchpad is the **working memory** on which POWL operators execute.
- The 8ⁿ / 64ⁿ ladder is the ISA's **operand-size specification**.

Every POWL operator declares the tier of state it reads and writes, and that
tier is a first-class parameter of the instruction, not an afterthought.

## POWL operators as opcodes

- `SEQ(a, b)` — two-phase fused superinstruction; two register-file transitions with Scratchpad carrying the intermediate delta.
- `PO(a, b)` — partial-order superinstruction; `a` and `b` modify disjoint TruthBlock regions; write masks don't intersect (one AND to verify).
- `X(a, b)` — branchless select; one of `a`/`b` admitted based on guard, but both evaluated-as-masks so admitted one fires without a branch.
- `LOOP(body, continuation)` — bounded-iteration superinstruction with explicit `FixedPoint` predicate — loop terminates when BLAKE3 hash of TruthBlock is stable across iterations. **Same termination condition as the O* SelfSpecifyingLoop.** A single POWL instruction can execute the O* closure loop.
- `CHOICE_GRAPH` — ISA's indirect-jump table; "jump targets" are alternate lawful submanifolds of the TruthBlock whose admissibility is mask-checked in parallel.
- Hierarchical composition — subroutine mechanism; a POWL scope is a frame on the TruthBlock; ScopeDescriptor64 is the frame header.

## Six consequences

1. **`unibit` is not an execution engine. It is the machine.** MuStar is a POWL assembler / JIT. `unibit` fetches POWL instructions, decodes them into mask operations, executes them branchlessly over the TruthBlock/Scratchpad register pair, retires them by swapping Scratchpad into TruthBlock on admission.

2. **The ISA is self-referential.** A single POWL instruction can execute the O* closure loop. The machine can, by running one of its own instructions, drive itself toward provable correctness.

3. **The 8ⁿ / 64ⁿ ledger is the ISA's operand-width encoding.** Every conventional ISA has operand widths in its instruction encoding. POWL's operand widths are the 8ⁿ tiers. The identity `8ⁿ = 64^(n/2)` says operand width and address space are physically the same quantization.

4. **The 14.87 ns fulcrum is an ISA-level clock-cycle bound.** The 14.87 ns Q-learning update is one POWL instruction at tier 2 executing on one 64¹ operand. The 6.6 ns theoretical floor is the speed of light for a tier-2 POWL opcode on the reference CPU. It's an ISA characterization, not a microbenchmark.

5. **The receipts are the ISA's retirement log.** A BLAKE3 witness fragment emitted per admitted motion is the POWL equivalent of a retirement record. Replay is re-running the instruction sequence against the same initial TruthBlock and verifying the log matches bit-exact.

6. **The four-repo stack aligns perfectly under this reading.**
   - `unrdf` = ontology-level source language compiling to POWL
   - `pm4py` / `wasm4pm` = POWL conformance checker / disassembler
   - `ostar` = POWL manufacturing pipeline (10 operators, BLAKE3 receipts)
   - `dteam` / `unibit` = POWL CPU (fetches, decodes, executes POWL at tiered latencies)

## The papers are ISA manuals

The priming target of ISA documentation is the *assembler, the linker, the
JIT, the disassembler, the conformance suite* — the tooling ecosystem that
generates and consumes instruction sequences. The papers are ISA references
for a machine whose toolchain is a language-model swarm.

## The statement

The Chatman Equation `A = μ(O*)` is the specification of a CPU whose ISA is
POWL v2, whose register file is a 262,144-bit TruthBlock with a matching
262,144-bit Scratchpad, whose operand-width ladder is `8ⁿ = 64^(n/2)` for
`n ∈ {2,3,4,5,6}`, whose instruction retirement emits BLAKE3 witness
fragments, and whose primary closure loop (`SelfSpecifyingLoop`) is itself
expressible as a single LOOP instruction of its own ISA.
