//! # Redis Fundamentals for Flash Sales
//!
//! This module teaches core Redis data types and operations through the lens
//! of a flash sale system. Each exercise focuses on a specific Redis feature
//! and its application in high-throughput, low-latency scenarios.
//!
//! ## Module Structure
//!
//! - `p01_connection_pool` - Connection pooling with deadpool-redis
//! - `p02_string_operations` - STRING ops for stock counters
//! - `p03_hash_operations` - HASH ops for product details
//! - `p04_set_operations` - SET ops for claim deduplication
//! - `p05_sorted_set_operations` - SORTED SET for rate limiting
//! - `p06_pipeline_operations` - Pipelining for batch efficiency
//! - `p07_transaction_basic` - WATCH/MULTI/EXEC for consistency
//! - `p08_key_expiration` - TTL management for ephemeral data
//! - `p09_error_handling` - Error types and recovery patterns
//! - `p10_connection_resilience` - Retry logic and reconnection

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_connection_pool;
#[cfg(not(feature = "solution"))]
pub mod p02_string_operations;
#[cfg(not(feature = "solution"))]
pub mod p03_hash_operations;
#[cfg(not(feature = "solution"))]
pub mod p04_set_operations;
#[cfg(not(feature = "solution"))]
pub mod p05_sorted_set_operations;
#[cfg(not(feature = "solution"))]
pub mod p06_pipeline_operations;
#[cfg(not(feature = "solution"))]
pub mod p07_transaction_basic;
#[cfg(not(feature = "solution"))]
pub mod p08_key_expiration;
#[cfg(not(feature = "solution"))]
pub mod p09_error_handling;
#[cfg(not(feature = "solution"))]
pub mod p10_connection_resilience;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_connection_pool.rs"]
pub mod p01_connection_pool;
#[cfg(feature = "solution")]
#[path = "solution/p02_string_operations.rs"]
pub mod p02_string_operations;
#[cfg(feature = "solution")]
#[path = "solution/p03_hash_operations.rs"]
pub mod p03_hash_operations;
#[cfg(feature = "solution")]
#[path = "solution/p04_set_operations.rs"]
pub mod p04_set_operations;
#[cfg(feature = "solution")]
#[path = "solution/p05_sorted_set_operations.rs"]
pub mod p05_sorted_set_operations;
#[cfg(feature = "solution")]
#[path = "solution/p06_pipeline_operations.rs"]
pub mod p06_pipeline_operations;
#[cfg(feature = "solution")]
#[path = "solution/p07_transaction_basic.rs"]
pub mod p07_transaction_basic;
#[cfg(feature = "solution")]
#[path = "solution/p08_key_expiration.rs"]
pub mod p08_key_expiration;
#[cfg(feature = "solution")]
#[path = "solution/p09_error_handling.rs"]
pub mod p09_error_handling;
#[cfg(feature = "solution")]
#[path = "solution/p10_connection_resilience.rs"]
pub mod p10_connection_resilience;
