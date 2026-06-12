//! # Solution 07: Capacity Model
//!
//! Complete implementation of capacity planning using Little's Law and queuing theory.

/// Custom error type for capacity model operations.
#[derive(Debug, thiserror::Error)]
pub enum CapacityError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Utilization exceeds 100%")]
    OverCapacity,
}

/// Calculate the expected concurrency using Little's Law.
///
/// Little's Law: L = lambda * W
pub fn calculate_concurrency(arrival_rate: f64, latency: f64) -> f64 {
    arrival_rate * latency
}

/// Calculate the maximum throughput given concurrency and latency.
///
/// Rearranging Little's Law: lambda = L / W
pub fn calculate_max_throughput(concurrency: f64, latency: f64) -> f64 {
    if latency <= 0.0 {
        return 0.0;
    }
    concurrency / latency
}

/// Calculate the number of server instances needed for a target RPS.
///
/// Adds 20% headroom for safety margin.
pub fn calculate_required_instances(target_rps: f64, per_instance_capacity: f64) -> usize {
    if per_instance_capacity <= 0.0 {
        return 1;
    }
    let raw = (target_rps / per_instance_capacity) * 1.2;
    let instances = raw.ceil() as usize;
    instances.max(1)
}

/// Model how latency increases as utilization approaches 100%.
///
/// Uses the M/M/1 queuing model:
///   latency = service_time / (1 - utilization)
pub fn utilization_latency_curve(
    base_latency_ms: f64,
    utilization_levels: &[f64],
) -> Vec<(f64, f64)> {
    utilization_levels
        .iter()
        .map(|&u| {
            if u >= 1.0 {
                (u, f64::INFINITY)
            } else {
                (u, base_latency_ms / (1.0 - u))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_littles_law() {
        let concurrency = calculate_concurrency(100.0, 0.1);
        assert!(
            (concurrency - 10.0).abs() < 0.001,
            "Expected 10.0, got {}",
            concurrency
        );
    }

    #[test]
    fn test_littles_law_zero_latency() {
        let concurrency = calculate_concurrency(100.0, 0.0);
        assert_eq!(concurrency, 0.0, "Zero latency means zero concurrency");
    }

    #[test]
    fn test_max_throughput() {
        let throughput = calculate_max_throughput(10.0, 0.1);
        assert!(
            (throughput - 100.0).abs() < 0.001,
            "Expected 100.0, got {}",
            throughput
        );
    }

    #[test]
    fn test_instance_sizing() {
        let instances = calculate_required_instances(50000.0, 5000.0);
        assert_eq!(instances, 12, "Should need 12 instances with 20% headroom");
    }

    #[test]
    fn test_instance_sizing_small() {
        let instances = calculate_required_instances(10.0, 5000.0);
        assert_eq!(instances, 1, "Should need at least 1 instance");
    }

    #[test]
    fn test_utilization_curve() {
        let curve = utilization_latency_curve(10.0, &[0.0, 0.5, 0.9, 0.95, 0.99]);
        assert_eq!(curve.len(), 5);
        assert!(
            (curve[0].1 - 10.0).abs() < 0.001,
            "At 0% util, latency = base"
        );
        assert!(
            (curve[1].1 - 20.0).abs() < 0.001,
            "At 50% util, latency = 20ms"
        );
        assert!(
            (curve[2].1 - 100.0).abs() < 0.001,
            "At 90% util, latency = 100ms"
        );
        assert!(
            (curve[4].1 - 1000.0).abs() < 0.001,
            "At 99% util, latency = 1000ms"
        );
    }
}
