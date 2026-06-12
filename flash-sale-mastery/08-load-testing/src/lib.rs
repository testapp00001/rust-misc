//! # Module 08: Load Testing and Capacity Planning
//!
//! This module teaches how to load test flash sale systems and use the results
//! to build capacity models. You will implement load generators, scenario
//! orchestrators, metrics collectors, latency analyzers, capacity models, and
//! bottleneck identifiers.
//!
//! ## Module Structure
//!
//! - `p01_load_generator` - Custom HTTP load generator using tokio + reqwest
//! - `p02_ramp_up_scenario` - Gradual ramp-up from 0 to N concurrent users
//! - `p03_spike_scenario` - Instant spike to N concurrent users
//! - `p04_sustained_load` - Steady-state sustained load scenario
//! - `p05_metrics_collector` - Collect and aggregate load test metrics
//! - `p06_latency_analysis` - Percentile calculation and outlier detection
//! - `p07_capacity_model` - Capacity planning using Little's Law
//! - `p08_bottleneck_identifier` - Identify system bottlenecks from metrics

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_load_generator;
#[cfg(not(feature = "solution"))]
pub mod p02_ramp_up_scenario;
#[cfg(not(feature = "solution"))]
pub mod p03_spike_scenario;
#[cfg(not(feature = "solution"))]
pub mod p04_sustained_load;
#[cfg(not(feature = "solution"))]
pub mod p05_metrics_collector;
#[cfg(not(feature = "solution"))]
pub mod p06_latency_analysis;
#[cfg(not(feature = "solution"))]
pub mod p07_capacity_model;
#[cfg(not(feature = "solution"))]
pub mod p08_bottleneck_identifier;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_load_generator.rs"]
pub mod p01_load_generator;
#[cfg(feature = "solution")]
#[path = "solution/p02_ramp_up_scenario.rs"]
pub mod p02_ramp_up_scenario;
#[cfg(feature = "solution")]
#[path = "solution/p03_spike_scenario.rs"]
pub mod p03_spike_scenario;
#[cfg(feature = "solution")]
#[path = "solution/p04_sustained_load.rs"]
pub mod p04_sustained_load;
#[cfg(feature = "solution")]
#[path = "solution/p05_metrics_collector.rs"]
pub mod p05_metrics_collector;
#[cfg(feature = "solution")]
#[path = "solution/p06_latency_analysis.rs"]
pub mod p06_latency_analysis;
#[cfg(feature = "solution")]
#[path = "solution/p07_capacity_model.rs"]
pub mod p07_capacity_model;
#[cfg(feature = "solution")]
#[path = "solution/p08_bottleneck_identifier.rs"]
pub mod p08_bottleneck_identifier;
