//! # Atomic Counters for Flash Sale Stock Management
//!
//! This module teaches distributed atomic counter patterns essential for
//! managing flash sale inventory. When thousands of customers race to buy
//! limited stock, the counter must be atomic -- no two customers can
//! purchase the same last item.
//!
//! ## Module Structure
//!
//! - `p01_local_atomic_counter` - In-memory atomic counters with `AtomicI64`
//! - `p02_redis_atomic_counter` - Redis DECR-based atomic counters
//! - `p03_cas_pattern` - Compare-and-Swap with WATCH/MULTI/EXEC
//! - `p04_optimistic_locking` - Version-based optimistic concurrency control
//! - `p05_pessimistic_locking` - Distributed locks and SELECT FOR UPDATE
//! - `p06_reconciliation` - Reconciling counters across data stores
//! - `p07_counter_benchmark` - Performance comparison of counter approaches
//! - `p08_failure_analysis` - Failure modes and recovery strategies
//! - `p09_multi_instance_test` - Multi-instance consistency verification
//!
//! ## Key Invariant
//!
//! **Stock must never go negative.** Every counter implementation must
//! guarantee that concurrent decrements are serialized and the stock
//! value stays >= 0. This is the fundamental safety property of a
//! flash sale system.

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_local_atomic_counter;
#[cfg(not(feature = "solution"))]
pub mod p02_redis_atomic_counter;
#[cfg(not(feature = "solution"))]
pub mod p03_cas_pattern;
#[cfg(not(feature = "solution"))]
pub mod p04_optimistic_locking;
#[cfg(not(feature = "solution"))]
pub mod p05_pessimistic_locking;
#[cfg(not(feature = "solution"))]
pub mod p06_reconciliation;
#[cfg(not(feature = "solution"))]
pub mod p07_counter_benchmark;
#[cfg(not(feature = "solution"))]
pub mod p08_failure_analysis;
#[cfg(not(feature = "solution"))]
pub mod p09_multi_instance_test;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_local_atomic_counter.rs"]
pub mod p01_local_atomic_counter;
#[cfg(feature = "solution")]
#[path = "solution/p02_redis_atomic_counter.rs"]
pub mod p02_redis_atomic_counter;
#[cfg(feature = "solution")]
#[path = "solution/p03_cas_pattern.rs"]
pub mod p03_cas_pattern;
#[cfg(feature = "solution")]
#[path = "solution/p04_optimistic_locking.rs"]
pub mod p04_optimistic_locking;
#[cfg(feature = "solution")]
#[path = "solution/p05_pessimistic_locking.rs"]
pub mod p05_pessimistic_locking;
#[cfg(feature = "solution")]
#[path = "solution/p06_reconciliation.rs"]
pub mod p06_reconciliation;
#[cfg(feature = "solution")]
#[path = "solution/p07_counter_benchmark.rs"]
pub mod p07_counter_benchmark;
#[cfg(feature = "solution")]
#[path = "solution/p08_failure_analysis.rs"]
pub mod p08_failure_analysis;
#[cfg(feature = "solution")]
#[path = "solution/p09_multi_instance_test.rs"]
pub mod p09_multi_instance_test;
