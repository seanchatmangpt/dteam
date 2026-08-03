# 50 — The Power in Naming Everything

## The short answer

Naming is not labeling. Naming is **gating**. Every name in the glossary
is a gate that admits one thing and refuses everything else. A system
whose vocabulary is closed is a system whose behavior is bounded.

The power is that by the time you finish naming, you can no longer be
imprecise — because imprecision has no word to live in.

---

## The specific powers

### 1. Naming makes concepts compile

A term that isn't in the glossary cannot be in the type system. A term
that isn't in the type system cannot be statically checked. A concept
with no name is a runtime problem; a concept with a name is a
compile-time problem.

Before the glossary: "we should probably make sure this is in L1."
After: `ResidenceTier::L1` — and the compiler refuses the other five
options.

Naming pulls concepts upward: from comment → variable → type parameter
→ const generic. Each promotion removes a class of bugs.

### 2. Naming makes failures addressable

A bug is a diff between what happened and what was supposed to happen.
If neither side has a name, "what was supposed to happen" is an
argument. If both sides have names, the diff is a fact.

"The admission was slow" → unprovable.
"`admit_eight` at `FieldLane::Causality` exceeded its `8²` budget on
E-core 4 at UInstr offset 0x148" → fixable.

The glossary is the shape of every legitimate bug report.

### 3. Naming makes audit possible

Receipts reference names. A receipt that says "L2: UInstr(0x42,
U4096, Prereq, Fragment, Hot)" is auditable; a receipt that says
"some instruction passed" is not. Every L0..L5 fragment in the chain
quotes the glossary. Without the glossary, the receipts are opaque
hashes that verify nothing meaningful.

**Audit is the power to disagree with a system's own claim about
itself.** That power requires shared vocabulary between the issuer and
the auditor. The glossary is that vocabulary.

### 4. Naming makes refactor a diff

Rename-in-place is cheap when names are canonical. It is expensive
when they are metaphorical because metaphor carries connotations that
break under renaming. `Cowboy → Runner` is a metaphor-swap with
connotations to preserve; `Runner → step` is a type-to-function
demotion with clear mechanics. The first is a paragraph of
justification; the second is a diff.

Canonical names reduce future work to mechanical work.

### 5. Naming makes teaching scale

Onboarding a new engineer into a system without a glossary is a
weeks-long narrative transfer. With a glossary, it is a read-and-grep
exercise. The glossary is the reason a future Claude can be primed on
this architecture in a single context load instead of re-deriving it
from scratch.

The glossary compresses the architecture's learning curve from "read
the whole repo" to "read these 49 docs."

### 6. Naming makes search exact

`grep "LaneOutcome"` finds every use site. "The thing the warden
returns" finds nothing. Every function, every benchmark, every log
line, every docstring routes through names. A system whose vocabulary
is exact is a system whose tools cooperate.

### 7. Naming makes benchmarks comparable

"Faster" means nothing. "`admit_eight` 9.8 ns vs baseline 14.87 ns on
M3 Max P-core" means something because every noun and unit in that
sentence has a pinned definition. The glossary is the dimensional
analysis of the benchmark suite.

### 8. Naming makes generativity compositional

Once you have `POWL8`, `POWL64`, `ResidenceTier::L2`, and `LaneMode`,
you can ask new questions that compose: *what is the POWL8 dialect
running at ResidenceTier::L2 under LaneMode::Escalating?* The answer
is derivable because every term in the question is a handle on
existing machinery. **Named systems generate new valid questions
without new vocabulary.**

---

## The power of *not* naming

Half the glossary's power is in the forbidden list.

```
FORBIDDEN_LITERARY   Gibson names — cannot cross code/comment line
FORBIDDEN_ROLES      Anthropomorphic roles — cannot be types at all
FORBIDDEN_LEXICON    Storage nouns — cannot appear in unibit docs
```

Naming what is banned is as powerful as naming what is allowed. It
prevents:

- fiction leaking into infrastructure
- roles masquerading as data
- metaphor creating hidden contracts

A forbidden list is a vaccine against convergence on the wrong
abstraction. It freezes the cost of "we'll clean this up later" at
zero, because "later" means now or never.

---

## The deeper answer: closure

`A = μ(O*)` requires `O*` be closed. The glossary is what makes it
closed. Every term that appears in the system is derivable from the
glossary, and nothing in the system appears that isn't in it.

```
Open system:   you can always add one more concept
Closed system: any new concept must be expressed in the existing ones
```

A closed system is a system whose behavior is bounded. A bounded
system can be proved, sealed, and released. An open system cannot —
because the proof's vocabulary is already obsolete by the time you
write it down.

Naming everything is how you close the vocabulary. Closing the
vocabulary is how you close the system. Closing the system is how you
release it.

**The glossary is the closure operator `Cl` applied to the vocabulary
of the architecture itself.**

---

## The commitment property

Once a name is published, the cost of changing it is social, not
technical. Every receipt that references the name, every benchmark
that cites it, every doc that quotes it, every engineer who learned
it, is a stakeholder in its stability.

This is both the power and the cost:

- **Power:** the name is stable; reasoning built on it is durable.
- **Cost:** you have to get the name right, because getting it wrong
  compounds.

The two rounds of naming (doc 47 metaphor-to-role, doc 48
role-to-idiom) were exactly this cost being paid before the names
entered source. Paying it late is the expensive version. Paying it
early — at the glossary level, before any `.rs` file uses the term —
is the cheap version.

---

## The asymmetry of naming

The person who names, owns. Not legally — cognitively. The name frames
every conversation about the thing. *CodeManufactory vs ostar vs
framework vs system* — the choice of word precedes the choice of
strategy. Whoever names the thing first controls which strategies are
obvious.

The Chatman doctrine — "The product is CodeManufactory; RevOps is
merely proof that CodeManufactory works" — is a naming move disguised
as a sentence. Once you adopt those two nouns in that relationship,
every roadmap decision tilts toward making CodeManufactory the
product. Try to argue for the opposite with those words; it feels
wrong because the names already took the position.

**Naming is positioning before strategy.** The glossary is the
position-space.

---

## The test

A good glossary passes this test:

> Can a stranger, with no knowledge of the fiction layer, the
> conversation history, or the authors, read the glossary and
> reproduce the architecture?

If yes, naming has done its work — it has compressed the whole
archive into something small enough to load into a fresh head.

If no, naming is still metaphor in the foreground, and the glossary
is narrative in table form.

Doc 49 sits between those states. Section 23 (literary ↔ canonical)
is the bridge that lets the fiction coexist with the technical
without leaking.

---

## What naming everything produces

| Output | Cause |
|---|---|
| compile-time safety | every concept is a type |
| receipt auditability | every fragment names its origin |
| cheap refactor | renames are diffs, not rewrites |
| fast onboarding | glossary compresses the learning curve |
| exact search | grep replaces narrative archaeology |
| comparable benchmarks | units are pinned by definition |
| composable questions | new queries combine existing terms |
| bounded vocabulary | the forbidden list vaccinates |
| closure | `Cl(vocabulary) = vocabulary` |
| release | bounded systems can be sealed |

These are not ten different benefits. They are ten projections of one
benefit: **the system has become finitely describable**, and finite
descriptions can be proved, sealed, released, and handed to the next
reader without information loss.

---

## The one-line answer

**Naming everything turns an open design into a closed system — the
vocabulary becomes a type system, the type system becomes a compiler,
the compiler becomes a gate, the gate becomes a receipt, and the
receipt becomes a release — and nothing outside the glossary can
cross any of those thresholds.**

---

## The sentence

**The power in naming everything is that after you finish, imprecision
has nowhere to live: every concept is a type, every type is a gate,
every gate is a receipt, and every receipt verifies against a closed
glossary — so the system becomes provable in the same breath that it
becomes legible, and the line between documentation and specification
disappears because they are the same document.**
