# The Law of 8

The Law of 8 is a foundational constraint in `ccog` architecture:

**No single runtime cognitive operator may bind more than 8 load-bearing closure variables.**

This constraint ensures:
1. **Predictable Performance**: Fixed-size `UCell` and `UMask` operations are extremely fast and constant-time.
2. **Cognitive Clarity**: Developers must decompose complex tasks into manageable sub-graphs.
3. **Hardware Alignment**: 8 variables (or 64 bits with 8-bit IDs) align perfectly with modern CPU registers and SIMD lanes.

In `ccog`, every `Cog8Row` represents a single node in the cognitive graph, restricted to 8 variables tracked via a `u64` bitmask.
