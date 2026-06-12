//! # Solution 07: Health Check
//!
//! Complete implementation of health check endpoints that probe dependencies
//! and report aggregated service status.

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
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health information for a single component.
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ComponentStatus,
    pub latency: Duration,
    pub details: Option<String>,
}

/// Overall health status of the service.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Result of a complete health check.
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub timestamp: std::time::Instant,
}

impl HealthReport {
    /// Check if the service is ready to receive traffic.
    pub fn is_ready(&self) -> bool {
        self.status == HealthStatus::Healthy || self.status == HealthStatus::Degraded
    }
}

/// Type-erased check function stored in the HealthChecker.
type CheckFn = Box<
    dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ComponentHealth> + Send>>
        + Send
        + Sync,
>;

/// Health checker that probes dependencies and reports overall status.
pub struct HealthChecker {
    checks: Vec<CheckFn>,
}

impl HealthChecker {
    /// Create a new health checker.
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Register a component health check.
    pub fn register_check<F, Fut>(&mut self, _name: &str, check_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ComponentHealth> + Send + 'static,
    {
        self.checks
            .push(Box::new(move || Box::pin(check_fn())));
    }

    /// Run all registered health checks and return an aggregated report.
    ///
    /// Checks are run concurrently. The overall status is the worst component status.
    pub async fn check(&self) -> HealthReport {
        let futures: Vec<_> = self.checks.iter().map(|check| check()).collect();
        let components = futures::future::join_all(futures).await;

        // Determine overall status from component results
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        for component in &components {
            match component.status {
                ComponentStatus::Unhealthy => has_unhealthy = true,
                ComponentStatus::Degraded => has_degraded = true,
                ComponentStatus::Healthy => {}
            }
        }

        let status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthReport {
            status,
            components,
            timestamp: std::time::Instant::now(),
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulate a Redis connectivity check.
pub async fn check_redis(is_available: bool, latency: Duration) -> ComponentHealth {
    tokio::time::sleep(latency).await;

    let status = if !is_available {
        ComponentStatus::Unhealthy
    } else if latency > Duration::from_millis(20) {
        ComponentStatus::Degraded
    } else {
        ComponentStatus::Healthy
    };

    ComponentHealth {
        name: "redis".to_string(),
        status,
        latency,
        details: if is_available {
            Some("connection pool healthy".to_string())
        } else {
            Some("connection refused".to_string())
        },
    }
}

/// Simulate a database connectivity check.
pub async fn check_database(is_available: bool, latency: Duration) -> ComponentHealth {
    tokio::time::sleep(latency).await;

    let status = if !is_available {
        ComponentStatus::Unhealthy
    } else if latency > Duration::from_millis(100) {
        ComponentStatus::Degraded
    } else {
        ComponentStatus::Healthy
    };

    ComponentHealth {
        name: "database".to_string(),
        status,
        latency,
        details: if is_available {
            Some("connection pool healthy".to_string())
        } else {
            Some("connection refused".to_string())
        },
    }
}

/// Simulate a queue depth check.
pub async fn check_queue(depth: usize, max_depth: usize) -> ComponentHealth {
    let ratio = depth as f64 / max_depth as f64;

    let status = if ratio >= 0.9 {
        ComponentStatus::Unhealthy
    } else if ratio >= 0.5 {
        ComponentStatus::Degraded
    } else {
        ComponentStatus::Healthy
    };

    ComponentHealth {
        name: "queue".to_string(),
        status,
        latency: Duration::ZERO,
        details: Some(format!("{depth}/{max_depth} messages pending")),
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
            check_database(true, Duration::from_millis(150)).await
        });
        checker.register_check("queue", || async { check_queue(10, 100).await });

        let report = checker.check().await;
        assert_eq!(report.status, HealthStatus::Degraded);
        assert!(report.is_ready());
    }

    #[tokio::test]
    async fn test_multiple_failures() {
        let mut checker = HealthChecker::new();
        checker.register_check("redis", || async {
            check_redis(false, Duration::from_millis(0)).await
        });
        checker.register_check("database", || async {
            check_database(false, Duration::from_millis(0)).await
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
