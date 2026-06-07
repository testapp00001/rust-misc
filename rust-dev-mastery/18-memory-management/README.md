# Module 18: Memory Management

Deep dive into Rust's memory model, custom allocators, arena/pool allocation,
memory mapping, and profiling techniques.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_allocator_basics.rs` | Global allocator, System allocator, Allocator trait |
| 02 | `p02_arena_allocation.rs` | Bump allocation, typed arenas, arena lifetimes |
| 03 | `p03_pool_allocation.rs` | Object pools, slab allocation, recycling |
| 04 | `p04_memory_mapping.rs` | mmap, memmap2, memory-mapped files |
| 05 | `p05_leak_detection.rs` | Leak detection, tracking allocations, Drop patterns |
| 06 | `p06_stack_vs_heap.rs` | Stack vs heap, stack overflow, heap fragmentation |
| 07 | `p07_memory_profiling.rs` | dhat, heaptrack, massif, allocation flamegraphs |
| 08 | `p08_small_vec_optimization.rs` | SmallVec, inline storage, compact representations |
| 09 | `p09_zero_copy.rs` | Bytes crate, zero-copy parsing, buffer management |
| 10 | `p10_memory_safe_patterns.rs` | RAII, self-referential types, Pin usage |

## Key Concepts

- **Ownership**: Rust's primary memory safety mechanism
- **RAII**: Resource Acquisition Is Initialization
- **Arena Allocation**: Bump-pointer allocation for fast deallocation
- **Object Pools**: Pre-allocated pools for reuse
- **Zero-Copy**: Avoiding unnecessary memory copies
