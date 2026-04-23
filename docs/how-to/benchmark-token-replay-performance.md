# How to Benchmark Token Replay Performance

## Prerequisites

- Rust stable toolchain installed
- `cargo-make` installed (`cargo install cargo-make`)

---

## 1. Run the benchmarks

Run individual benchmark suites directly:

```sh
# Divan benchmarks — kernel and token replay hot paths
cargo bench --bench kernel_bench

# K-tier scalability across K64–K1024
cargo bench --bench ktier_scalability_bench

# Combined hot-path regression suite
cargo bench --bench hot_path_performance_bench
```

To run all benchmark suites in one command:

```sh
cargo make bench
```

---

## 2. Read the Divan output (kernel_bench)

`kernel_bench` uses the Divan harness. Output looks like:

```
Timer precision: 41 ns
kernel_bench              fastest       │ slowest       │ median        │ mean
├─ petri_net_incidence_matrix           │               │               │
│  ├─ 10_places            2.1 µs       │ 3.4 µs        │ 2.3 µs        │ 2.4 µs
│  └─ 64_places            4.8 µs       │ 6.1 µs        │ 5.0 µs        │ 5.1 µs
├─ replay_trace                         │               │               │
│  ├─ 10_places_cached     1.1 µs       │ 1.9 µs        │ 1.2 µs        │ 1.3 µs
│  └─ 64_places_cached     2.2 µs       │ 3.5 µs        │ 2.4 µs        │ 2.5 µs
```

**Column meanings:**

| Column | Meaning |
|---|---|
| `fastest` | Best observed sample across all iterations |
| `slowest` | Worst observed sample (watch for outliers indicating GC or scheduling jitter) |
| `median` | Primary number to compare across runs — resistant to outliers |
| `mean` | Use alongside median; a large gap between median and mean indicates tail latency |

**`_cached` suffix** — Benchmarks with this suffix call `compile_incidence()` once during setup, then replay. This isolates pure replay cost from model compilation cost. Compare cached vs. uncached numbers to measure how much `compile_incidence()` contributes to total latency.

**`petri_net_incidence_matrix`** — Measures construction of the incidence matrix. This is the one-time compilation cost paid before any replay can begin.

---

## 3. Read the Criterion output (ktier_scalability_bench)

`ktier_scalability_bench` uses the Criterion harness. Output looks like:

```
ktier/K64/epoch_update    time:   [2.134 µs 2.241 µs 2.358 µs]
ktier/K128/epoch_update   time:   [5.210 µs 5.388 µs 5.571 µs]
ktier/K256/epoch_update   time:   [9.012 µs 9.187 µs 9.401 µs]
```

The `time: [lower, estimate, upper]` triplet is a 95% confidence interval:

- **`lower`** — Lower bound of the CI. Your best-case latency under stable conditions.
- **`estimate`**** — Point estimate. Use this number for comparisons and regression tracking.
- **`upper`** — Upper bound. If this grows between runs, the environment has scheduling noise.

**Reference latency table by K-tier:**

| K-tier | Typical latency per epoch |
|---|---|
| K64 | 2–5 µs |
| K128 | 5–10 µs |
| K256 | 8–12 µs |
| K512 | 14–20 µs |
| K1024 | 30–50 µs |

If your measured estimates fall significantly above these ranges, check for heap allocation in hot paths — the performance constraints in `CLAUDE.md` prohibit `Vec`/`HashMap` on replay and RL update paths.

---

## 4. Select the right tier

**Programmatic selection** — Call `Engine::budget(&log)` with your `EventLog`. It inspects the number of distinct places in the projected net and returns the minimum sufficient `KTier`.

**Manual selection** — Count places in your PNML model. Then:

- Nets with ≤ 64 places: use `K64` for maximum speed (bitmask fast path)
- 65–128 places: use `K128`
- 129–256 places: use `K256`
- Larger: step up to `K512` or `K1024`

Once you have chosen a tier, set it in `dteam.toml`:

```toml
[kernel]
tier = "K256"
```

Nets with ≤ 64 places automatically use the u64 bitmask fast path in conformance replay (`src/conformance/bitmask_replay.rs`). Nets above 64 places fall back to `replay_trace_standard`. The bitmask path is the primary performance target; measure against it first.
