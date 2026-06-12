//! # Solution 06: Latency Analysis
//!
//! Complete implementation of latency analysis functions.

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
pub fn calculate_percentiles(latencies: &[Duration]) -> Result<Percentiles, LatencyError> {
    if latencies.is_empty() {
        return Err(LatencyError::NoData);
    }

    let mut values: Vec<f64> = latencies
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = values.len() as f64;
    let sum: f64 = values.iter().sum();
    let mean = sum / n;

    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    let p50 = nearest_rank(&values, 50.0);
    let p95 = nearest_rank(&values, 95.0);
    let p99 = nearest_rank(&values, 99.0);
    let p999 = nearest_rank(&values, 99.9);

    Ok(Percentiles {
        p50,
        p95,
        p99,
        p999,
        max: *values.last().unwrap(),
        min: values[0],
        mean,
        stddev,
    })
}

fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    let n = sorted.len() as f64;
    let rank = (percentile / 100.0 * n).ceil().max(1.0) as usize;
    let index = rank.min(sorted.len()) - 1;
    sorted[index]
}

/// Detect outliers in latency data (values beyond 3 standard deviations from mean).
pub fn detect_outliers(latencies: &[Duration]) -> Result<Vec<usize>, LatencyError> {
    if latencies.is_empty() {
        return Err(LatencyError::NoData);
    }

    let values: Vec<f64> = latencies
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .collect();

    let n = values.len() as f64;
    let mean: f64 = values.iter().sum::<f64>() / n;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    let threshold = 3.0 * stddev;
    let outliers: Vec<usize> = values
        .iter()
        .enumerate()
        .filter(|(_, v)| (*v - mean).abs() > threshold)
        .map(|(i, _)| i)
        .collect();

    Ok(outliers)
}

/// Build a latency histogram with the specified number of buckets.
pub fn latency_histogram(
    latencies: &[Duration],
    bucket_count: usize,
) -> Result<Vec<Bucket>, LatencyError> {
    if latencies.is_empty() {
        return Err(LatencyError::NoData);
    }
    if bucket_count == 0 {
        return Err(LatencyError::InvalidBucketCount(0));
    }

    let values: Vec<f64> = latencies
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .collect();

    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Handle the case where all values are the same.
    let range = max_val - min_val;
    let bucket_width = if range > 0.0 {
        range / bucket_count as f64
    } else {
        1.0 // arbitrary width when all values are identical
    };

    let mut buckets: Vec<Bucket> = (0..bucket_count)
        .map(|i| Bucket {
            lower_ms: min_val + i as f64 * bucket_width,
            upper_ms: min_val + (i + 1) as f64 * bucket_width,
            count: 0,
        })
        .collect();

    for &v in &values {
        let mut idx = ((v - min_val) / bucket_width) as usize;
        if idx >= bucket_count {
            idx = bucket_count - 1; // put max value in last bucket
        }
        buckets[idx].count += 1;
    }

    Ok(buckets)
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
        let mut values = make_latencies_ms(&[10, 12, 11, 13, 10, 14, 12, 11, 13, 10]);
        values.push(Duration::from_millis(1000));
        let outliers = detect_outliers(&values).expect("Should detect outliers");
        assert!(!outliers.is_empty(), "Should find at least one outlier");
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
