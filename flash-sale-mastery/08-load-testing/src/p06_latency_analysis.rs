//! # Exercise 06: Latency Analysis
//!
//! ## Learning Objective
//! Implement latency analysis functions that compute percentiles, detect
//! outliers, and build histograms from raw latency data.
//!
//! ## Flash Sale Context
//! Raw latency numbers from a load test are meaningless without analysis. A
//! system with "average 50ms latency" might have 1% of requests taking 5
//! seconds. Percentile analysis, outlier detection, and histograms reveal the
//! full picture of user experience.
//!
//! ## Instructions
//! 1. Implement `calculate_percentiles` to compute p50, p95, p99, p99.9, max,
//!    min, mean, and standard deviation
//! 2. Implement `detect_outliers` to find latencies beyond 3 standard deviations
//! 3. Implement `latency_histogram` to bucket latencies into a histogram
//!
//! ## Hints
//! - Sort the latencies first for percentile calculation
//! - Use the nearest-rank method: index = ceil(percentile/100 * n) - 1
//! - For standard deviation: sqrt(sum((x - mean)^2) / n)
//! - Histogram buckets should cover the range from min to max

use std::time::Duration;

/// Latency percentiles and summary statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct Percentiles {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
    pub max: f64,
    pub min: f64,
    pub mean: f64,
    pub stddev: f64,
}

/// A single histogram bucket.
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    /// Lower bound of the bucket (inclusive), in milliseconds.
    pub lower_ms: f64,
    /// Upper bound of the bucket (exclusive), in milliseconds.
    pub upper_ms: f64,
    /// Number of values in this bucket.
    pub count: usize,
}

/// Custom error type for latency analysis operations.
#[derive(Debug, thiserror::Error)]
pub enum LatencyError {
    #[error("No latency data provided")]
    NoData,

    #[error("Invalid bucket count: {0}")]
    InvalidBucketCount(usize),
}

/// Calculate percentile statistics from a slice of latencies.
///
/// # Arguments
/// * `latencies` - Slice of latency durations
///
/// # Returns
/// A `Percentiles` struct with p50, p95, p99, p99.9, max, min, mean, stddev
/// all in milliseconds.
///
/// # Errors
/// Returns `LatencyError::NoData` if the slice is empty.
pub fn calculate_percentiles(latencies: &[Duration]) -> Result<Percentiles, LatencyError> {
    // TODO: Convert durations to milliseconds (f64)
    // TODO: Sort the values
    // TODO: Compute p50, p95, p99, p99.9 using nearest-rank method
    // TODO: Compute min, max, mean, stddev
    todo!("Implement percentile calculation")
}

/// Detect outliers in latency data (values beyond 3 standard deviations from mean).
///
/// # Arguments
/// * `latencies` - Slice of latency durations
///
/// # Returns
/// A vector of indices into the original slice where outliers were found.
///
/// # Errors
/// Returns `LatencyError::NoData` if the slice is empty.
pub fn detect_outliers(latencies: &[Duration]) -> Result<Vec<usize>, LatencyError> {
    // TODO: Compute mean and stddev
    // TODO: Find all values where |x - mean| > 3 * stddev
    // TODO: Return their original indices
    todo!("Implement outlier detection")
}

/// Build a latency histogram with the specified number of buckets.
///
/// # Arguments
/// * `latencies` - Slice of latency durations
/// * `bucket_count` - Number of buckets to create
///
/// # Returns
/// A vector of `Bucket` structs covering the range from min to max.
///
/// # Errors
/// Returns `LatencyError::NoData` if the slice is empty.
/// Returns `LatencyError::InvalidBucketCount` if bucket_count is 0.
pub fn latency_histogram(
    latencies: &[Duration],
    bucket_count: usize,
) -> Result<Vec<Bucket>, LatencyError> {
    // TODO: Find min and max values
    // TODO: Compute bucket width: (max - min) / bucket_count
    // TODO: Count values falling into each bucket
    todo!("Implement histogram generation")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_latencies_ms(values: &[u64]) -> Vec<Duration> {
        values.iter().map(|&ms| Duration::from_millis(ms)).collect()
    }

    #[test]
    fn test_percentiles_basic() {
        let latencies = make_latencies_ms(&[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
        let p = calculate_percentiles(&latencies).expect("Should compute percentiles");
        assert!((p.min - 10.0).abs() < 0.01, "min should be 10ms");
        assert!((p.max - 100.0).abs() < 0.01, "max should be 100ms");
        assert!((p.p50 - 50.0).abs() < 10.0, "p50 should be near 50ms");
        assert!(p.p95 >= p.p50, "p95 should be >= p50");
        assert!(p.p99 >= p.p95, "p99 should be >= p95");
    }

    #[test]
    fn test_percentiles_empty() {
        let result = calculate_percentiles(&[]);
        assert!(result.is_err(), "Should error on empty data");
    }

    #[test]
    fn test_outlier_detection() {
        // 10 normal values + 1 outlier at 1000ms
        let mut values = make_latencies_ms(&[10, 12, 11, 13, 10, 14, 12, 11, 13, 10]);
        values.push(Duration::from_millis(1000));
        let outliers = detect_outliers(&values).expect("Should detect outliers");
        assert!(!outliers.is_empty(), "Should find at least one outlier");
        // The outlier at index 10 (1000ms) should be detected
        assert!(
            outliers.contains(&10),
            "Should detect the 1000ms outlier at index 10"
        );
    }

    #[test]
    fn test_histogram_basic() {
        let latencies = make_latencies_ms(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let buckets = latency_histogram(&latencies, 5).expect("Should create histogram");
        assert_eq!(buckets.len(), 5, "Should have 5 buckets");
        let total: usize = buckets.iter().map(|b| b.count).sum();
        assert_eq!(total, 10, "All values should be in some bucket");
    }

    #[test]
    fn test_histogram_invalid_bucket_count() {
        let latencies = make_latencies_ms(&[1, 2, 3]);
        let result = latency_histogram(&latencies, 0);
        assert!(result.is_err(), "Should error on zero buckets");
    }
}
