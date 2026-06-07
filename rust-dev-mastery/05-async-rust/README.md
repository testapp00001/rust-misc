# Module 5: Async Rust

Deep dive into Rust's asynchronous programming ecosystem. Covers the Tokio runtime,
the Future trait internals, async/await patterns, streams, cancellation safety,
and production-grade async architecture.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_tokio_deep_dive.rs` | Runtime configuration, multi-thread vs current-thread, spawning, JoinHandle |
| 2 | `p02_future_internals.rs` | Future trait, poll, Waker, Pin, async state machines, hand-rolled futures |
| 3 | `p03_async_patterns.rs` | select!, join!, try_join!, timeout, async blocks, async closures |
| 4 | `p04_async_error_handling.rs` | Error propagation in async, Result in futures, panic handling, JoinError |
| 5 | `p05_async_concurrency.rs` | Semaphore, Mutex, RwLock, broadcast/mpsc/oneshot channels, Notify |
| 6 | `p06_streams.rs` | Stream trait, async-stream crate, stream combinators, buffering, backpressure |
| 7 | `p07_cancellation_safety.rs` | Cancellation-safe patterns, tokio::select! pitfalls, drop safety, cleanup |
| 8 | `p08_async_testing.rs` | tokio::test, async test patterns, time simulation, test utilities |
| 9 | `p09_async_performance.rs` | Avoiding allocations, boxing futures, spawn_local, work stealing, batching |
| 10 | `p10_async_architecture.rs` | Structuring async apps, graceful shutdown, connection pools, middleware |
