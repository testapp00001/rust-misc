//! # Exercise 07: Health Check
//!
//! ## Learning Objective
//!
//! Implement health check endpoints that report the status of your service and its
//! dependencies. Health checks are critical for load balancers, orchestrators, and
//! monitoring systems to know whether to route traffic to an instance.
//!
//! ## Flash Sale Context
//!
//! During a flash sale, a service might be running but degraded -- Redis is slow,
//! the database connection pool is nearly exhausted, or the queue is backing up.
//! Simple "is the process alive" checks aren't enough. You need deep health checks
//! that verify each dependency and report an overall status: Healthy (all good),
//! Degraded (some issues but still serving), or Unhealthy (should not receive traffic).
//!
//! ## Instructions
//!
//! 1. Define `ComponentHealth` with status and optional latency/details
//! 2. Define `HealthStatus` enum: Healthy, Degraded, Unhealthy
//! 3. Implement `HealthChecker` with checks for each component
//! 4. Implement `check()` that aggregates component statuses into an overall status
//! 5. Implement individual check functions for Redis, DB, and queue depth
//!
//! ## Hints
//!
//! - Each component check returns a `ComponentHealth` independently
//! - Overall status is the worst component status
//! - Use `tokio::time::timeout` for individual health check probes
//! - Consider: Healthy + Healthy = Healthy, Healthy + Degraded = Degraded, any + Unhealthy = Unhealthy

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during health checks.
#[derive(Debug, Error)]
pub enum HealthCheckError {
    #[error("health check timed out for component: {0}")]
    CheckTimeout(String),
    #[error("health check failed: {0}")]
    CheckFailed(String),
}

/// Status of a single component.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStatus {
    /// Component is functioning normally
    Healthy,
    /// Component is functioning but with degraded performance
    Degraded,
    /// Component is not functioning
    Unhealthy,
}

/// Health information for a single component.
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Name of the component (e.g., "redis", "database")
    pub name: String,
    /// Current status
    pub status: ComponentStatus,
    /// Response time of the health check probe
    pub latency: Duration,
    /// Optional human-readable details
    pub details: Option<String>,
}

/// Overall health status of the service.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// All components are healthy
    Healthy,
    /// Some components are degraded but the service can still serve traffic
    Degraded,
    /// Critical components are unhealthy, the service should not receive traffic
    Unhealthy,
}

/// Result of a complete health check.
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// Overall status
    pub status: HealthStatus,
    /// Individual component health
    pub components: Vec<ComponentHealth>,
    /// When the check was performed
    pub timestamp: std::time::Instant,
}

impl HealthReport {
    /// Check if the service is ready to receive traffic.
    pub fn is_ready(&self) -> bool {
        self.status == HealthStatus::Healthy || self.status == HealthStatus::Degraded
    }
}

/// Health checker that probes dependencies and reports overall status.
pub struct HealthChecker {
    // TODO: Add fields for component check functions or configuration
    // Each component check is an async function that returns ComponentHealth
    checks: Vec<Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ComponentHealth> + Send>> + Send + Sync>>,
}

impl HealthChecker {
    /// Create a new health checker.
    pub fn new() -> Self {
        // TODO: Initialize with an empty list of checks
        todo!("Implement HealthChecker::new")
    }

    /// Register a component health check.
    ///
    /// # Arguments
    /// * `name` - Name of the component to check
    /// * `check_fn` - Async function that probes the component and returns ComponentHealth
    pub fn register_check<F, Fut>(&mut self, name: &str, check_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ComponentHealth> + Send + 'static,
    {
        // TODO: Box and store the check function
        todo!("Implement HealthChecker::register_check")
    }

    /// Run all registered health checks and return an aggregated report.
    ///
    /// Checks are run concurrently. The overall status is determined by the
    /// worst component status.
    ///
    /// # Returns
    /// A `HealthReport` with the aggregated status and individual component results
    pub async fn check(&self) -> HealthReport {
        // TODO: Implement health check aggregation:
        //   1. Run all checks concurrently (e.g., with join_all)
        //   2. Collect results into components Vec
        //   3. Determine overall status from component statuses:
        //      - Any Unhealthy -> overall Unhealthy
        //      - Any Degraded -> overall Degraded
        //      - All Healthy -> overall Healthy
        //   4. Return HealthReport
        todo!("Implement HealthChecker::check")
    }
}

/// Simulate a Redis connectivity check.
///
/// In production, this would ping Redis and measure response time.
/// For this exercise, simulate the check based on the `is_available` parameter.
pub async fn check_redis(is_available: bool, latency: Duration) -> ComponentHealth {
    // TODO: Simulate a Redis health check:
    //   - Sleep for the specified latency (simulating network round-trip)
    //   - Return Healthy if available, Unhealthy if not
    //   - Set Degraded if latency > 20ms
    todo!("Implement check_redis")
}

/// Simulate a database connectivity check.
///
/// In production, this would execute a simple query and measure response time.
pub async fn check_database(is_available: bool, latency: Duration) -> ComponentHealth {
    // TODO: Simulate a database health check:
    //   - Sleep for the specified latency
    //   - Return Healthy if available, Unhealthy if not
    //   - Set Degraded if latency > 100ms
    todo!("Implement check_database")
}

/// Simulate a queue depth check.
///
/// In production, this would check the message queue depth.
pub async fn check_queue(depth: usize, max_depth: usize) -> ComponentHealth {
    // TODO: Simulate a queue depth check:
    //   - Return Healthy if depth < 50% of max
    //   - Return Degraded if depth >= 50% and < 90% of max
    //   - Return Unhealthy if depth >= 90% of max
    todo!("Implement check_queue")
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_all_healthy() {
        let mut checker = HealthChecker::new();
        checker.register_check("redis", || async {
            check_redis(true, Duration::from_millis(5)).await
        });
        checker.register_check("database", || async {
            check_database(true, Duration::from_millis(10)).await
        });
        checker.register_check("queue", || async { check_queue(10, 100).await });

        let report = checker.check().await;
        assert_eq!(report.status, HealthStatus::Healthy);
        assert!(report.is_ready());
        assert_eq!(report.components.len(), 3);
    }

    #[tokio::test]
    async fn test_one_component_degraded() {
        let mut checker = HealthChecker::new();
        checker.register_check("redis", || async {
            check_redis(true, Duration::from_millis(5)).await
        });
        checker.register_check("database", || async {
            check_database(true, Duration::from_millis(150)).await // slow = degraded
        });
        checker.register_check("queue", || async { check_queue(10, 100).await });

        let report = checker.check().await;
        assert_eq!(report.status, HealthStatus::Degraded);
        assert!(report.is_ready()); // degraded is still ready
    }

    #[tokio::test]
    async fn test_multiple_failures() {
        let mut checker = HealthChecker::new();
        checker.register_check("redis", || async {
            check_redis(false, Duration::from_millis(0)).await // down
        });
        checker.register_check("database", || async {
            check_database(false, Duration::from_millis(0)).await // down
        });
        checker.register_check("queue", || async { check_queue(10, 100).await });

        let report = checker.check().await;
        assert_eq!(report.status, HealthStatus::Unhealthy);
        assert!(!report.is_ready());
    }

    #[tokio::test]
    async fn test_component_health_details() {
        let health = check_redis(true, Duration::from_millis(5)).await;
        assert_eq!(health.name, "redis");
        assert_eq!(health.status, ComponentStatus::Healthy);
    }

    #[tokio::test]
    async fn test_queue_depth_healthy() {
        let health = check_queue(10, 100).await;
        assert_eq!(health.status, ComponentStatus::Healthy);
    }

    #[tokio::test]
    async fn test_queue_depth_degraded() {
        let health = check_queue(60, 100).await;
        assert_eq!(health.status, ComponentStatus::Degraded);
    }

    #[tokio::test]
    async fn test_queue_depth_unhealthy() {
        let health = check_queue(95, 100).await;
        assert_eq!(health.status, ComponentStatus::Unhealthy);
    }
}
