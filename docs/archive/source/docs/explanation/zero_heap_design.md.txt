# Zero-Heap Execution Philosophy

## The problem

Three deployment scenarios make heap allocation structurally incompatible with correct execution:

**WASM (no GC).** WebAssembly runtimes expose a linear memory model without a garbage collector. A Rust binary compiled to WASM can use `Vec` and `HashMap` — these allocate from the WASM linear heap — but the allocator has no GC to reclaim memory, no OS to swap pages, and no way to handle allocation failure except by trapping. For a conformance replay kernel that must process thousands of traces in a browser session, the cumulative allocation pressure from per-trace `Vec` creation would exhaust the 1MB WASM linear memory budget within minutes. The only safe path is to never allocate during replay.

**Real-time autonomic loop (50 µs budget).** The `AutonomicKernel::run_cycle` must complete observe → infer → propose → accept → execute → adapt within a single epoch. Heap allocation introduces two unpredictable costs: the `malloc` call itself (tens to hundreds of nanoseconds, depending on allocator state and fragmentation), and the potential for the allocator to request more virtual memory from the OS (a system call that can take tens of microseconds). For a 50 µs budget, a single system-call allocator request consumes the entire epoch. Zero-heap eliminates both costs.

**Streaming replay (millions of replays/sec).** The PDC 2025 benchmark requires scoring thousands of test traces, potentially across 15 logs in a timed evaluation. Per-trace allocation means millions of `malloc`/`free` round-trips. At heap allocation rates of ~100 ns each, 1M allocations consume 100 ms — a meaningful fraction of the total pipeline budget. Zero-heap on the hot path makes allocation cost proportional to the number of nets loaded, not the number of traces replayed.

## `PackedKeyTable` vs `HashMap`

`std::collections::HashMap` uses SipHash by default, initialized with a seed drawn from OS randomness at construction time. This creates two problems. First, the same key maps to different `u64` hash values across process restarts, making results non-reproducible across runs. For a system whose correctness depends on audit-reproducible fingerprints (`canonical_hash`, Q-table state keys, net markings), non-deterministic hashing is a correctness defect. Second, `HashMap` scatters its buckets across the heap, breaking cache locality on lookup. A hot Q-table lookup with 64+ entries may touch 4–8 cache lines, each a potential L1 miss.

`PackedKeyTable` is a flat array of `(u64, K, V)` triples with a separate open-addressed index array. The layout stores all entries contiguously in a `Vec<(u64, K, V)>`. The index array is a separate `Vec<u32>` used for O(1) hash-based lookup, rebuilt lazily when the load factor exceeds 50%. For small tables (fewer than 16 entries, typical for net markings), the index array fits in a single cache line. For Q-tables, entries are sequential in memory, making linear scans (the fallback when the index is stale) cache-friendly.

The key property: `PackedKeyTable` uses the pre-computed `fnv1a_64` hash passed by the caller — no hashing happens inside the table. The caller controls the hash function. This means the determinism guarantee flows from `fnv1a_64` properties, not from the table implementation.

## `KBitSet` vs `Vec`

A `Vec<bool>` for Petri net markings would require a heap allocation per marking, a length field, and a capacity field. For a 64-place net, this is 3 words of overhead for 64 bits of actual data. More importantly, `Vec::clone()` allocates a new buffer — meaning every state transition in the RL loop that clones the current marking triggers a `malloc`.

`KBitSet<const WORDS: usize>` stores marking state as `[u64; WORDS]`. The const generic `WORDS` is resolved at monomorphization time, giving the compiler the array size as a compile-time constant. This enables:

- **Loop unrolling.** `for i in 0..WORDS` over a compile-time constant `WORDS` is unrolled by the compiler with no loop overhead.
- **SIMD auto-vectorization.** For `WORDS=16` (K1024 tier, 1024 bits), bitwise operations over 16 x `u64` are vectorized to 2 x 256-bit AVX2 operations or 4 x 128-bit SSE2 operations on compatible hardware.
- **`Copy` semantics.** `[u64; WORDS]` implements `Copy`. `KBitSet<WORDS>` is `Copy`. State transitions that need to explore alternatives can copy the current `KBitSet` with no allocation — it is a stack copy, not a heap allocation.

The capacity of each tier flows directly from the word count: K64 = 1 word = 64 bits = 64 places; K256 = 4 words = 256 places; K512 = 8 words = 512 places; K1024 = 16 words = 1024 places. Each tier's capacity is the bit width of its `KBitSet<WORDS>`, and the compiler's monomorphization ensures the right word count is used throughout the hot path.

## `fnv1a_64` vs `std` hash

The FNV-1a hash function is defined by two constants: offset basis `0xcbf29ce484222325` and prime `0x100000001b3`. The algorithm XORs each input byte with the current hash value, then multiplies by the prime. For 8 bytes of input (a `u64` key), this is 8 XOR-multiply cycles — roughly 30 CPU cycles total on modern hardware.

The properties that make FNV-1a the right choice here:

**No seed.** FNV-1a's output is fully determined by its input and the two constants. The same byte sequence always produces the same `u64`. `std::hash` uses `SipHash` with an OS-randomized seed per `HashMap` instance, producing different hashes for the same key in different process runs.

**Non-cryptographic by design.** FNV-1a is not safe against adversarial inputs — a sufficiently motivated attacker can construct inputs that produce collisions, causing hash table degradation. This is acceptable for dteam's use cases (event log activity names, place IDs, Q-table state keys) because there is no adversarial user providing activity names. The payoff for accepting this limitation is speed: SipHash runs at ~1–2 ns/byte; FNV-1a runs at ~0.3 ns/byte.

**Deterministic collision detection.** `DenseIndex::compile` detects collisions explicitly (returns `DenseError::HashCollision`) because FNV-1a's non-cryptographic strength means collisions are possible on unlucky inputs. This is a better design than silently allowing collisions to corrupt the index.

## Performance numbers

These figures come from the benchmark suite and the whitepaper analysis:

| Configuration | Memory footprint | Epoch latency |
|---|---|---|
| K64 (`KBitSet<1>`) | 16 KB table | 2–5 µs |
| K256 (`KBitSet<4>`) | 16 KB table | 2–5 µs |
| K512 (`KBitSet<8>`) | 64 KB table | 14–20 µs |
| K1024 (`KBitSet<16>`) | 128 KB table | 30–50 µs |
| RL step (Q-table update) | stack only | 200–500 ns |

The RL step figure reflects the full `observe → update Q-table → select action` cycle in `DefaultKernel::run_cycle` with a `PackedKeyTable` Q-table and a `KBitSet`-based `RlState`. No heap allocation occurs on this path. The 200–500 ns range reflects the difference between Q-table lookup on a warm L1 cache (when the same state is revisited frequently) and a cold lookup (first visit to a new state region).

The 50 µs autonomic loop budget accommodates K512 epoch timing (14–20 µs) with headroom for the infer and propose phases. K1024 (30–50 µs) consumes most of the budget and is appropriate only when the net genuinely requires 512–1024 places. The tier system exists precisely to match net complexity to deployment constraints.
