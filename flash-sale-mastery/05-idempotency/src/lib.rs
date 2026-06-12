//! # Idempotency and Exactly-Once Semantics
//!
//! This module teaches how to make flash sale purchase flows idempotent, ensuring
//! that duplicate requests (from user double-clicks, browser retries, or network
//! timeouts) never produce duplicate vouchers or charges.
//!
//! ## Module Structure
//!
//! - `p01_idempotency_key_generation` - Generating and validating idempotency keys
//! - `p02_redis_idempotency_store` - Fast deduplication with Redis SET NX
//! - `p03_db_idempotency_store` - Durable deduplication with database constraints
//! - `p04_concurrent_dedup` - Safe handling of concurrent duplicate requests
//! - `p05_idempotent_handler_middleware` - Axum middleware for transparent idempotency
//! - `p06_exactly_once_consumer` - Effectively-once message processing
//! - `p07_race_condition_test` - Race condition tests for the "never double-issue" invariant
//! - `p08_idempotency_benchmark` - Performance benchmarks for deduplication strategies

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_idempotency_key_generation;
#[cfg(not(feature = "solution"))]
pub mod p02_redis_idempotency_store;
#[cfg(not(feature = "solution"))]
pub mod p03_db_idempotency_store;
#[cfg(not(feature = "solution"))]
pub mod p04_concurrent_dedup;
#[cfg(not(feature = "solution"))]
pub mod p05_idempotent_handler_middleware;
#[cfg(not(feature = "solution"))]
pub mod p06_exactly_once_consumer;
#[cfg(not(feature = "solution"))]
pub mod p07_race_condition_test;
#[cfg(not(feature = "solution"))]
pub mod p08_idempotency_benchmark;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_idempotency_key_generation.rs"]
pub mod p01_idempotency_key_generation;
#[cfg(feature = "solution")]
#[path = "solution/p02_redis_idempotency_store.rs"]
pub mod p02_redis_idempotency_store;
#[cfg(feature = "solution")]
#[path = "solution/p03_db_idempotency_store.rs"]
pub mod p03_db_idempotency_store;
#[cfg(feature = "solution")]
#[path = "solution/p04_concurrent_dedup.rs"]
pub mod p04_concurrent_dedup;
#[cfg(feature = "solution")]
#[path = "solution/p05_idempotent_handler_middleware.rs"]
pub mod p05_idempotent_handler_middleware;
#[cfg(feature = "solution")]
#[path = "solution/p06_exactly_once_consumer.rs"]
pub mod p06_exactly_once_consumer;
#[cfg(feature = "solution")]
#[path = "solution/p07_race_condition_test.rs"]
pub mod p07_race_condition_test;
#[cfg(feature = "solution")]
#[path = "solution/p08_idempotency_benchmark.rs"]
pub mod p08_idempotency_benchmark;
