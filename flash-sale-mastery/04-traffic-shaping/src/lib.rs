//! # Traffic Shaping and Rate Limiting for Flash Sales
//!
//! This module teaches rate limiting, admission control, and traffic shaping
//! strategies essential for protecting a flash sale system from overload.
//! When 100K+ requests per second hit your API, you need multiple layers of
//! defense to prevent cascading failures and ensure fair access.
//!
//! ## Module Structure
//!
//! - `p01_token_bucket` - Token bucket rate limiter (burst-tolerant)
//! - `p02_sliding_window_log` - Sliding window log (precise, memory-heavy)
//! - `p03_sliding_window_counter` - Sliding window counter (approximate, memory-efficient)
//! - `p04_leaky_bucket` - Leaky bucket (constant output rate)
//! - `p05_admission_control` - Admission control: admit, queue, or reject
//! - `p06_virtual_waiting_room` - Virtual waiting room with HTTP 202
//! - `p07_backpressure_channel` - Backpressure using bounded channels
//! - `p08_graceful_degradation` - Graceful degradation strategies
//! - `p09_multi_layer_rate_limit` - Multi-layer rate limiting (per-IP, per-account, global)
//! - `p10_rate_limit_middleware` - Axum/Tower middleware for rate limiting

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_token_bucket;
#[cfg(not(feature = "solution"))]
pub mod p02_sliding_window_log;
#[cfg(not(feature = "solution"))]
pub mod p03_sliding_window_counter;
#[cfg(not(feature = "solution"))]
pub mod p04_leaky_bucket;
#[cfg(not(feature = "solution"))]
pub mod p05_admission_control;
#[cfg(not(feature = "solution"))]
pub mod p06_virtual_waiting_room;
#[cfg(not(feature = "solution"))]
pub mod p07_backpressure_channel;
#[cfg(not(feature = "solution"))]
pub mod p08_graceful_degradation;
#[cfg(not(feature = "solution"))]
pub mod p09_multi_layer_rate_limit;
#[cfg(not(feature = "solution"))]
pub mod p10_rate_limit_middleware;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_token_bucket.rs"]
pub mod p01_token_bucket;
#[cfg(feature = "solution")]
#[path = "solution/p02_sliding_window_log.rs"]
pub mod p02_sliding_window_log;
#[cfg(feature = "solution")]
#[path = "solution/p03_sliding_window_counter.rs"]
pub mod p03_sliding_window_counter;
#[cfg(feature = "solution")]
#[path = "solution/p04_leaky_bucket.rs"]
pub mod p04_leaky_bucket;
#[cfg(feature = "solution")]
#[path = "solution/p05_admission_control.rs"]
pub mod p05_admission_control;
#[cfg(feature = "solution")]
#[path = "solution/p06_virtual_waiting_room.rs"]
pub mod p06_virtual_waiting_room;
#[cfg(feature = "solution")]
#[path = "solution/p07_backpressure_channel.rs"]
pub mod p07_backpressure_channel;
#[cfg(feature = "solution")]
#[path = "solution/p08_graceful_degradation.rs"]
pub mod p08_graceful_degradation;
#[cfg(feature = "solution")]
#[path = "solution/p09_multi_layer_rate_limit.rs"]
pub mod p09_multi_layer_rate_limit;
#[cfg(feature = "solution")]
#[path = "solution/p10_rate_limit_middleware.rs"]
pub mod p10_rate_limit_middleware;
