//! # Exercise 07: Capacity Model
//!
//! ## Learning Objective
//! Use Little's Law and queuing theory to build a capacity planning model that
//! determines how many server instances are needed for a target throughput.
//!
//! ## Flash Sale Context
//! "We expect 50,000 requests per second during the sale. Each instance handles
//! 5,000 rps at acceptable latency. How many instances do we need?" This is
//! the fundamental question of capacity planning. But the answer is not simply
//! 50,000 / 5,000 = 10 -- you need headroom for spikes, failures, and the
//! non-linear latency increase near saturation.
//!
//! ## Instructions
//! 1. Implement `calculate_concurrency` using Little's Law: L = lambda * W
//! 2. Implement `calculate_max_throughput` from concurrency and latency
//! 3. Implement `calculate_required_instances` for target RPS with headroom
//! 4. Implement `utilization_latency_curve` to model latency at various utilization levels
//!
//! ## Hints
//! - Little's Law: concurrency = arrival_rate * latency
//! - Max throughput = concurrency / latency
//! - For instance sizing, add 20% headroom: instances = ceil(target_rps / capacity * 1.2)
//! - The M/M/1 queuing model: latency = service_time / (1 - utilization)

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
/// - L = average number of requests in the system (concurrency)
/// - lambda = arrival rate (requests per second)
/// - W = average time a request spends in the system (seconds)
///
/// # Arguments
/// * `arrival_rate` - Requests per second
/// * `latency` - Average request latency in seconds
///
/// # Returns
/// The expected number of concurrent requests.
pub fn calculate_concurrency(arrival_rate: f64, latency: f64) -> f64 {
    // TODO: Apply Little's Law: L = lambda * W
    todo!("Implement Little's Law calculation")
}

/// Calculate the maximum throughput given concurrency and latency.
///
/// Rearranging Little's Law: lambda = L / W
///
/// # Arguments
/// * `concurrency` - Number of concurrent requests the system can handle
/// * `latency` - Average request latency in seconds
///
/// # Returns
/// Maximum throughput in requests per second.
pub fn calculate_max_throughput(concurrency: f64, latency: f64) -> f64 {
    // TODO: Rearrange Little's Law: lambda = L / W
    todo!("Implement max throughput calculation")
}

/// Calculate the number of server instances needed for a target RPS.
///
/// Adds 20% headroom for safety margin.
///
/// # Arguments
/// * `target_rps` - Desired requests per second
/// * `per_instance_capacity` - RPS each instance can handle
///
/// # Returns
/// Number of instances needed (minimum 1).
pub fn calculate_required_instances(target_rps: f64, per_instance_capacity: f64) -> usize {
    // TODO: Compute instances = ceil(target_rps / per_instance_capacity * 1.2)
    // TODO: Return at least 1
    todo!("Implement instance sizing")
}

/// Model how latency increases as utilization approaches 100%.
///
/// Uses the M/M/1 queuing model:
///   latency = service_time / (1 - utilization)
///
/// # Arguments
/// * `base_latency_ms` - Latency at 0% utilization in milliseconds
/// * `utilization_levels` - Slice of utilization values (0.0 to 1.0)
///
/// # Returns
/// Vector of (utilization, latency_ms) pairs showing the latency curve.
pub fn utilization_latency_curve(
    base_latency_ms: f64,
    utilization_levels: &[f64],
) -> Vec<(f64, f64)> {
    // TODO: For each utilization level, compute latency = base / (1 - utilization)
    // TODO: Return (utilization, latency) pairs
    todo!("Implement utilization-latency curve")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_littles_law() {
        // 100 rps with 0.1s latency => 10 concurrent requests
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
        // 10 concurrent requests with 0.1s latency => 100 rps max
        let throughput = calculate_max_throughput(10.0, 0.1);
        assert!(
            (throughput - 100.0).abs() < 0.001,
            "Expected 100.0, got {}",
            throughput
        );
    }

    #[test]
    fn test_instance_sizing() {
        // 50000 rps target, 5000 per instance, with 20% headroom
        // = ceil(50000 / 5000 * 1.2) = ceil(12) = 12
        let instances = calculate_required_instances(50000.0, 5000.0);
        assert_eq!(instances, 12, "Should need 12 instances with 20% headroom");
    }

    #[test]
    fn test_instance_sizing_small() {
        // Even tiny targets need at least 1 instance
        let instances = calculate_required_instances(10.0, 5000.0);
        assert_eq!(instances, 1, "Should need at least 1 instance");
    }

    #[test]
    fn test_utilization_curve() {
        let curve = utilization_latency_curve(10.0, &[0.0, 0.5, 0.9, 0.95, 0.99]);
        assert_eq!(curve.len(), 5);
        // At 0% utilization, latency = base latency
        assert!(
            (curve[0].1 - 10.0).abs() < 0.001,
            "At 0% util, latency = base"
        );
        // At 50% utilization, latency = 2x base
        assert!(
            (curve[1].1 - 20.0).abs() < 0.001,
            "At 50% util, latency = 20ms"
        );
        // At 90% utilization, latency = 10x base
        assert!(
            (curve[2].1 - 100.0).abs() < 0.001,
            "At 90% util, latency = 100ms"
        );
        // At 99% utilization, latency = 100x base
        assert!(
            (curve[4].1 - 1000.0).abs() < 0.001,
            "At 99% util, latency = 1000ms"
        );
    }
}
