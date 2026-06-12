//! # Resilience Patterns for Flash Sales
//!
//! This module teaches resilience patterns essential for surviving the chaos of
//! a flash sale. When thousands of users hit your system simultaneously, downstream
//! services (Redis, databases, payment processors, notification services) will fail.
//! These patterns ensure your system degrades gracefully instead of collapsing.
//!
//! ## Module Structure
//!
//! - `p01_circuit_breaker` - Stop calling failing services
//! - `p02_bulkhead` - Isolate failures to prevent cascade
//! - `p03_retry_with_backoff` - Retry transient failures with exponential backoff
//! - `p04_timeout_management` - Never wait forever for a response
//! - `p05_fallback_strategies` - Graceful degradation when primary fails
//! - `p06_graceful_shutdown` - Drain in-flight requests before exit
//! - `p07_health_check` - Know when your system is unhealthy
//! - `p08_failure_injection` - Test resilience by injecting failures

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_circuit_breaker;
#[cfg(not(feature = "solution"))]
pub mod p02_bulkhead;
#[cfg(not(feature = "solution"))]
pub mod p03_retry_with_backoff;
#[cfg(not(feature = "solution"))]
pub mod p04_timeout_management;
#[cfg(not(feature = "solution"))]
pub mod p05_fallback_strategies;
#[cfg(not(feature = "solution"))]
pub mod p06_graceful_shutdown;
#[cfg(not(feature = "solution"))]
pub mod p07_health_check;
#[cfg(not(feature = "solution"))]
pub mod p08_failure_injection;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_circuit_breaker.rs"]
pub mod p01_circuit_breaker;
#[cfg(feature = "solution")]
#[path = "solution/p02_bulkhead.rs"]
pub mod p02_bulkhead;
#[cfg(feature = "solution")]
#[path = "solution/p03_retry_with_backoff.rs"]
pub mod p03_retry_with_backoff;
#[cfg(feature = "solution")]
#[path = "solution/p04_timeout_management.rs"]
pub mod p04_timeout_management;
#[cfg(feature = "solution")]
#[path = "solution/p05_fallback_strategies.rs"]
pub mod p05_fallback_strategies;
#[cfg(feature = "solution")]
#[path = "solution/p06_graceful_shutdown.rs"]
pub mod p06_graceful_shutdown;
#[cfg(feature = "solution")]
#[path = "solution/p07_health_check.rs"]
pub mod p07_health_check;
#[cfg(feature = "solution")]
#[path = "solution/p08_failure_injection.rs"]
pub mod p08_failure_injection;
