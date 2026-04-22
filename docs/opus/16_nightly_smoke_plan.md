# 16 — unibit-nightly-smoke: First Milestone

## Principle

First step is not ontology. First step is build determinism.

```
nightly toolchain → hello world → one unstable feature → one aligned L1 pair → one pinned/locked allocation smoke test
```

## Target green bar

```
nightly compiles
generic_const_exprs compiles
L1 region validates (64 KiB = 2 × 32 KiB)
truth region position validates (offset = 0)
scratch region position validates (offset = 32 KiB)
pinning validates (Pin<Box<L1Region>>, stable address)
OS lock attempt exists (mlock)
inline assembly smoke passes
assembly emission works
lexicon check passes
```

## Minimum first repo layout

```
unibit-nightly-smoke/
├── Cargo.toml
├── rust-toolchain.toml
└── src/
    └── main.rs
```

## `rust-toolchain.toml`

```toml
[toolchain]
channel = "nightly"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

## `Cargo.toml`

```toml
[package]
name = "unibit-nightly-smoke"
version = "0.1.0"
edition = "2024"

[dependencies]
libc = "0.2"
```

## `src/main.rs` smoke

```rust
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::arch::asm;
use std::mem::{align_of, size_of};
use std::pin::Pin;

const WORDS: usize = 4096;

struct Work<const BITS: usize> where [(); BITS / 64]: {
    words: [u64; BITS / 64],
}

#[repr(C, align(64))]
struct TruthBlock { words: [u64; WORDS] }

#[repr(C, align(64))]
struct Scratchpad { words: [u64; WORDS] }

#[repr(C, align(64))]
struct L1Region {
    truth: TruthBlock,
    scratch: Scratchpad,
}

impl L1Region {
    fn zeroed() -> Self {
        Self { truth: TruthBlock { words: [0; WORDS] },
               scratch: Scratchpad { words: [0; WORDS] } }
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn asm_add_one(x: u64) -> u64 {
    let out: u64;
    unsafe {
        asm!("lea {out}, [{x} + 1]",
             x = in(reg) x, out = lateout(reg) out,
             options(nomem, nostack, preserves_flags));
    }
    out
}

fn main() {
    let _w: Work<512> = Work { words: [0; 8] };
    let pair: Pin<Box<L1Region>> = Box::pin(L1Region::zeroed());
    assert_eq!(size_of::<L1Region>(), 64 * 1024);
    assert_eq!(align_of::<L1Region>(), 64);
    let addr = (&*pair as *const L1Region) as usize;
    assert_eq!(addr % 64, 0);
    let r = unsafe { asm_add_one(41) };
    assert_eq!(r, 42);
    println!("nightly hello world passed");
    println!("generic_const_exprs passed");
    println!("pinned L1 position validated");
    println!("inline asm smoke passed");
}
```

## Two layers of pinning

| Layer | Mechanism | Meaning |
|---|---|---|
| Rust address stability | `Pin<Box<T>>` / custom arena | value does not move in Rust |
| Physical residency | `mlock` / platform equivalent | pages stay resident, not swapped |
| Semantic validation | base/offset/alignment check | address/offset becomes admitted coordinate |
| Hot-path contract | raw pointer kernel | no lookup, no relocation, no reinterpretation |

Rust `Pin` gives address stability. `mlock` attempts OS residency. Neither
gives atomicity, cache residency, or NUMA locality.

## Semantic pinning (position as contract)

If memory position has meaning, the primitive is not just
`TruthBlock + Scratchpad` but `PinnedRegion(BaseAddr, Alignment,
TruthOffset, ScratchOffset, Epoch)`.

Emit a boot receipt:

```rust
#[repr(C)]
struct L1BootReceipt {
    base: u64,
    truth_offset: u32,
    scratch_offset: u32,
    bytes: u32,
    align: u32,
    epoch: u64,
}
```

Later this is signed; address validation becomes part of O*, not a debug log.

## Assembly capability checklist

1. `asm!` inline — prove register-level snippets work.
2. `global_asm!` — prove external assembly symbol definition works.
3. `naked_asm!` — exact prologue/epilogue control. ABI trampolines only.
4. `#[target_feature]` — CPU-specialized kernels.

## Why 2 ns isn't a security proof

The real security property is that the hot primitive has **no interpreter surface**:
- no parser
- no allocator
- no dynamic dispatch
- no policy lookup
- no graph mutation API
- no runtime plug-in
- no late binding

Red-team insertion must attack earlier: planner, mask generator, capability
binder, kernel table, pinned region, binary replacement, OS/hypervisor/hardware.

Required response: signed kernel table, sealed binary, L1 position receipt,
capability-mask receipts, assembly admission, startup self-check.
