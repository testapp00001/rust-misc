//! # Caching Strategy for Flash Sales
//!
//! This module teaches cache warming, thundering herd prevention, and cache
//! strategies for flash sale systems. Each exercise focuses on a specific
//! caching pattern and its application in high-throughput, low-latency scenarios.
//!
//! ## Module Structure
//!
//! - `p01_cache_warming` - Pre-warming cache before flash sale starts
//! - `p02_thundering_herd` - Thundering herd problem and mutex-based solutions
//! - `p03_singleflight` - Singleflight pattern for request deduplication
//! - `p04_cache_aside` - Cache-aside pattern with TTL management
//! - `p05_primary_vs_cache` - Redis as primary vs cache based on mode
//! - `p06_invalidation` - Cache invalidation strategies
//! - `p07_transition_strategy` - Post-sale transition: drain, sync, switch
//! - `p08_cache_benchmark` - Benchmark cache hit vs miss performance

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_cache_warming;
#[cfg(not(feature = "solution"))]
pub mod p02_thundering_herd;
#[cfg(not(feature = "solution"))]
pub mod p03_singleflight;
#[cfg(not(feature = "solution"))]
pub mod p04_cache_aside;
#[cfg(not(feature = "solution"))]
pub mod p05_primary_vs_cache;
#[cfg(not(feature = "solution"))]
pub mod p06_invalidation;
#[cfg(not(feature = "solution"))]
pub mod p07_transition_strategy;
#[cfg(not(feature = "solution"))]
pub mod p08_cache_benchmark;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_cache_warming.rs"]
pub mod p01_cache_warming;
#[cfg(feature = "solution")]
#[path = "solution/p02_thundering_herd.rs"]
pub mod p02_thundering_herd;
#[cfg(feature = "solution")]
#[path = "solution/p03_singleflight.rs"]
pub mod p03_singleflight;
#[cfg(feature = "solution")]
#[path = "solution/p04_cache_aside.rs"]
pub mod p04_cache_aside;
#[cfg(feature = "solution")]
#[path = "solution/p05_primary_vs_cache.rs"]
pub mod p05_primary_vs_cache;
#[cfg(feature = "solution")]
#[path = "solution/p06_invalidation.rs"]
pub mod p06_invalidation;
#[cfg(feature = "solution")]
#[path = "solution/p07_transition_strategy.rs"]
pub mod p07_transition_strategy;
#[cfg(feature = "solution")]
#[path = "solution/p08_cache_benchmark.rs"]
pub mod p08_cache_benchmark;
