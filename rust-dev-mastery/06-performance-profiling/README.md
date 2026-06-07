# Module 6: Performance & Profiling

Master Rust performance optimization from profiling to production. Covers criterion
benchmarks, memory profiling, SIMD intrinsics, cache optimization, allocation
strategies, and zero-cost abstractions.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_profiling_tools.rs` | perf, flamegraph, cargo-flamegraph, Instruments, profiling workflow |
| 2 | `p02_criterion_benchmarks.rs` | Benchmark groups, custom inputs, statistical analysis, comparison reports |
| 3 | `p03_memory_profiling.rs` | heaptrack, dhat, massif, allocation tracking, memory usage reporting |
| 4 | `p04_cpu_optimization.rs` | Branch prediction, cache lines, data layout, loop optimization, inlining |
| 5 | `p05_simd_intrinsics.rs` | std::arch, portable SIMD, auto-vectorization, SIMD examples |
| 6 | `p06_cache_optimization.rs` | Cache-friendly data structures, SoA vs AoS, prefetching, false sharing |
| 7 | `p07_allocation_strategies.rs` | Avoiding allocations, pre-allocation, String vs &str, Cow, SmallVec |
| 8 | `p08_zero_cost_abstractions.rs` | Iterators vs loops, monomorphization, compile-time computation, traits |
| 9 | `p09_lazy_evaluation.rs` | Lazy initialization, once_cell, LazyLock, deferred computation, caching |
| 10 | `p10_production_perf.rs` | Performance monitoring, latency percentiles, throughput metrics, regression detection |
