//! # Observability for Flash Sales
//!
//! This module teaches distributed tracing, metrics, and observability patterns
//! essential for monitoring flash sale systems. When 500K requests hit your system
//! simultaneously and something goes wrong, you need deep visibility into every
//! layer of the stack.
//!
//! ## Module Structure
//!
//! - `p01_tracing_setup` - Structured logging with tracing
//! - `p02_span_propagation` - Distributed trace context propagation
//! - `p03_custom_metrics` - Prometheus metrics for flash sale operations
//! - `p04_redis_metrics` - Redis-specific performance metrics
//! - `p05_business_metrics` - Business-level KPI metrics
//! - `p06_alert_rules` - Alert rule definitions and evaluation
//! - `p07_dashboard_spec` - Dashboard specification structs
//! - `p08_log_analysis` - Log parsing and analysis utilities

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_tracing_setup;
#[cfg(not(feature = "solution"))]
pub mod p02_span_propagation;
#[cfg(not(feature = "solution"))]
pub mod p03_custom_metrics;
#[cfg(not(feature = "solution"))]
pub mod p04_redis_metrics;
#[cfg(not(feature = "solution"))]
pub mod p05_business_metrics;
#[cfg(not(feature = "solution"))]
pub mod p06_alert_rules;
#[cfg(not(feature = "solution"))]
pub mod p07_dashboard_spec;
#[cfg(not(feature = "solution"))]
pub mod p08_log_analysis;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_tracing_setup.rs"]
pub mod p01_tracing_setup;
#[cfg(feature = "solution")]
#[path = "solution/p02_span_propagation.rs"]
pub mod p02_span_propagation;
#[cfg(feature = "solution")]
#[path = "solution/p03_custom_metrics.rs"]
pub mod p03_custom_metrics;
#[cfg(feature = "solution")]
#[path = "solution/p04_redis_metrics.rs"]
pub mod p04_redis_metrics;
#[cfg(feature = "solution")]
#[path = "solution/p05_business_metrics.rs"]
pub mod p05_business_metrics;
#[cfg(feature = "solution")]
#[path = "solution/p06_alert_rules.rs"]
pub mod p06_alert_rules;
#[cfg(feature = "solution")]
#[path = "solution/p07_dashboard_spec.rs"]
pub mod p07_dashboard_spec;
#[cfg(feature = "solution")]
#[path = "solution/p08_log_analysis.rs"]
pub mod p08_log_analysis;
