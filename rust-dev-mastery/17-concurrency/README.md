# Module 17: Concurrency Patterns

Master Rust's powerful concurrency primitives and patterns for building
high-performance concurrent applications.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_channel_patterns.rs` | mpsc, crossbeam channels, bounded/unbounded, select |
| 02 | `p02_shared_state.rs` | Mutex, RwLock, parking_lot, deadlock prevention |
| 03 | `p03_lock_free_structures.rs` | Atomic types, CAS operations, lock-free stacks/queues |
| 04 | `p04_actor_model.rs` | Actor pattern, message passing, supervision |
| 05 | `p05_work_stealing.rs` | Rayon, work stealing scheduler, parallel iterators |
| 06 | `p06_parallel_iterators.rs` | Rayon parallel iterators, custom parallel operations |
| 07 | `p07_synchronization.rs` | Barriers, Condvar, Semaphore, once_cell, LazyLock |
| 08 | `p08_concurrent_data_structures.rs` | DashMap, concurrent hash maps, sharding |
| 09 | `p09_concurrent_testing.rs` | Loom, race detection, stress testing |
| 10 | `p10_concurrent_architecture.rs` | Thread-per-core, shared-nothing, event loops |

## Key Concepts

- **Message Passing**: Channels for safe communication between threads
- **Shared State**: Mutex, RwLock for protected access to shared data
- **Lock-Free**: Atomic operations for maximum performance
- **Actor Model**: Isolated actors communicating via messages
- **Work Stealing**: Efficient parallel task scheduling
- **Data Parallelism**: Parallel iterators over collections
