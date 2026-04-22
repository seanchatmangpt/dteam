# 53 — Why Memory Movement Doesn't Appear in the Core Benchmark

## The short answer

You don't care about memory movement in the core benchmark because
**there is no memory movement in the core.** The pinned 64³ layout
and the AtomVM boundary together guarantee that every cycle inside
`step()` is spent on arithmetic and branchless selects over cache
lines that were resident at boot and have not moved since.

When memory doesn't move, the benchmark measures the kernel. When
memory moves, the benchmark measures the environment. The boundary
choice puts you on the first side of that line.

---

## The five things memory movement usually adds as noise

Standard benchmarks of "tight" code are contaminated by five classes of
noise, all of them memory-movement related:

1. **Allocation jitter** — the allocator's variable cost to return a
   block
2. **GC pauses** — managed runtimes pausing to collect
3. **Cache fill latency** — first-touch cost when data isn't resident
4. **TLB misses / page walks** — OS paging, page table traversals
5. **False sharing / coherence traffic** — cores fighting over lines

Every real benchmarker knows these. Every real benchmarker runs a
warmup loop, uses a custom allocator, pins cores, and takes the median
of N runs to filter them out. That filtering process is itself an
admission that memory movement is dominating the signal.

The architecture removes all five, *at compile time*, so no filtering
is needed at benchmark time.

---

## Why each is zero here

### 1. Allocation jitter → zero
`unibit-hot` is `#![no_std]` with no allocator linked in. There is no
`Box::new`, no `Vec`, no `String`, no `Arc` on the hot path. Compile
fails if any of them appear. **If it compiled, it didn't allocate.**

### 2. GC pauses → zero
No managed runtime inside the core. AtomVM's GC lives on the other
side of the NIF. A GC pause can delay your *next* motion being
delivered, but it cannot interrupt a motion that is already in `step()`.

### 3. Cache fill latency → zero (steady state)
The HotRegion is 64 KiB, pinned, `mlock`'d, page-aligned. After the
first warmup access, all 16 pages are resident in L1D. Every subsequent
`step()` finds the same lines at the same offsets, hot. **First-touch
is a one-time cost at boot, not a benchmark cost.**

### 4. TLB misses → zero (steady state)
16 pages × 4 KiB = 64 KiB, well under the DTLB's 64–96 entry budget.
Because each page is page-aligned and adjacent, the TLB holds all 16
entries; the prefetcher rarely needs to walk a page table. **Steady-
state page-walk cost is zero.**

### 5. False sharing / coherence traffic → zero
Every sub-structure is `#[repr(C, align(64))]`. Per-core layouts
(doc 46) give each Loa its own private Straylight slice. The
ReduceBuffer's slots are each on separate cache lines. Federated CZI
gives each core its own watchdog shard. **No line is shared on the
hot path.**

---

## What you *are* measuring

When you run `cargo bench --bench admit_eight`, the cycle count you
observe is a function of exactly two things:

```
benchmark_cycles = instruction_count × L1D_hit_latency
```

Both are constants of the kernel:
- `instruction_count` is determined by the `#[inline(always)]` expansion
  of `admit_eight` — fixed at compile time.
- `L1D_hit_latency` is determined by the silicon — fixed by the target.

Neither varies per iteration. That's why the benchmark converges on a
stable number with n=100 instead of needing n=10,000 with
warmup/median/trim filtering.

**The number on your benchmark output is the speed of the kernel,
nothing else.**

---

## The variance test

Here is how to know your benchmark is honest in this architecture:

```
Run 10,000 iterations of admit_eight.
Compute the 99.9th percentile.
Compute the minimum.
Divide p99.9 by min.

If the ratio < 1.1  → benchmark is honest; kernel is pinned cleanly.
If the ratio ≥ 2    → something is moving that shouldn't.
```

On a healthy pinned region, this ratio is ~1.02–1.05. The kernel is
deterministic; the only variance is hardware counter noise (interrupt
handlers, IPI, one-shot uops).

**Variance itself becomes a diagnostic tool.** A high-variance
benchmark in this architecture means:
- the HotRegion is not actually pinned
- a different process is contending for L1D
- the benchmark thread is being preempted mid-hot-loop
- false sharing has crept in via a non-aligned new field

None of those are "benchmark noise." All of them are bugs. You fix the
pinning; the variance drops to 1.02× automatically.

---

## What to watch for instead (Instruments counters)

Because wall-clock is a clean signal, the *interesting* benchmark data
moves up a level to hardware counters:

```
L1D hit rate                 should be > 99.9%
DTLB hit rate                should be 100% (steady state)
Branch mispredicts           should be 0 (branchless)
Retired instructions / cycle should be near IPC ceiling
NEON port utilization        tells you if FIELD_ADMIT is fused
```

These are the signals when memory movement is zero. They describe the
kernel's relationship to the core's microarchitecture, which is the
only thing worth benchmarking in a substrate whose memory layout is
fixed.

---

## Comparing to the 14.87 ns baseline is honest

The prior DTEAM benchmarks (action select 6.95 ns, Q-update 14.87 ns)
were *already* measured under the same disciplines — `PackedKeyTable`
instead of `HashMap`, stack `RlState<const WORDS>`, no heap on the
update path. So comparing a new `admit_eight` at ~10 ns to the 14.87 ns
baseline is:

- same measurement clock (instruction-count)
- same allocator stance (zero allocations)
- same cache residency assumption (warm working set)
- same pinning guarantee (implicit; the Q-table was small)

**Like-for-like.** The comparison is not measuring one cache regime
against another; it is measuring one kernel against another, with
memory movement held at zero on both sides.

---

## The AtomVM side of the same benchmark

If you benchmark the full NIF round-trip instead of just `step()`, you
get an entirely different number — typically microseconds, not
nanoseconds. That number is also honest, but it is measuring:

```
NIF round-trip = BEAM scheduling
              + BEAM heap alloc for the packet
              + marshal into stack MotionPacket
              + step() ← the deterministic part
              + unmarshal into BEAM binary
              + BEAM message send
```

Five of those six items are wall-clock. One is instruction-count.
**Mixing the two clocks gives a meaningless average.** You must report
them separately:

```
step() latency:           10 ns     (cycle-exact, variance < 1.05×)
NIF overhead:             ~2 µs     (wall-clock, variance ~5–20×)
end-to-end:               ~2 µs     (dominated by NIF)
```

The core benchmark is the 10 ns. The system benchmark is the 2 µs.
Both are real; they are different measurements.

---

## What this means for benchmark harness design

Because memory movement is zero on the core, the harness is
radically simpler than a typical Criterion setup:

```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_step(c: &mut Criterion) {
    // One-time setup: pin the HotRegion.
    let region = unibit_phys::HotRegion::pin();
    let packet = compile::fixed_test_packet();

    // No warmup loop needed — pinning is the warmup.
    // No per-iteration allocation — nothing to allocate.
    // No custom allocator — there is no allocator.
    c.bench_function("step", |b| {
        b.iter(|| {
            let outcome = unibit_hot::step(
                black_box(&mut *region.as_mut()),
                black_box(&packet),
            );
            black_box(outcome);
        });
    });
}

criterion_group!(benches, bench_step);
criterion_main!(benches);
```

That's it. No `setup` closure to re-allocate per iteration. No
`iter_batched` to avoid amortizing allocation. Just a pinned region,
a constant packet, and a black_boxed call. The benchmark code reads
like the architecture: simple, deterministic, finite.

---

## The corollary: benchmark variance is a free bug detector

On a typical system, you filter out variance. In this architecture,
**variance is the diagnostic.** Any benchmark with p99.9/min > 1.1 is
a bug report, not a performance report.

You can add a single-line variance assertion to CI:

```
assert!(bench_step.p99_9 / bench_step.min < 1.10,
        "HotRegion is not pinned cleanly — memory is moving somewhere");
```

A failed assertion doesn't mean the kernel is slow. It means the
substrate's pinning discipline has broken. **The benchmark is also a
pinning test.**

---

## The formal statement

Let `C` be the cycle count of one `step()` invocation.

```
C = sum of retired instructions × microarch cost

On a pinned, branchless, allocation-free hot path:
    no branch mispredict       →  no speculative-rollback cycles
    no cache miss              →  no stall cycles
    no TLB miss                →  no page-walk cycles
    no allocation              →  no allocator-walk cycles
    no GC                      →  no global pause
    no false sharing           →  no coherence-invalidation cycles
    no preemption              →  no scheduler-swap cycles (over ns windows)

    → C is a function of the code, not of the system state.
```

When `C` is a function of the code alone, the benchmark measures the
code alone. That is why you don't care about memory movement: it
isn't in the equation.

---

## The sentence

**Memory movement doesn't appear in the core benchmark because memory
doesn't move in the core — the HotRegion is pinned once at boot, the
hot path has no allocator linked in, the branchless select flips bits
in place, and every cache line touched by `step()` was already
resident when the benchmark started — so the cycle count you measure
is a function of the kernel's retired instructions and the silicon's
L1D latency, both of which are constants of the code and the chip,
never of the environment; and if the benchmark variance ever rises
above 1.1× it's not noise, it's a bug in the pinning discipline
asking to be fixed.**
