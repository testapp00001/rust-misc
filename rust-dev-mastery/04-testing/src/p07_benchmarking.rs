//! # Lesson 7: Benchmarking
//!
//! Benchmarks measure performance and detect regressions.
//! This lesson covers criterion, benchmark groups, statistical analysis,
//! and how to set up benchmarks in a Cargo project.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Code to benchmark
// ---------------------------------------------------------------------------

/// A hash map wrapper for benchmarking different operations.
pub struct BenchStore {
    data: HashMap<String, String>,
}

impl BenchStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Get a value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Delete a key.
    pub fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys (unsorted).
    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }

    /// Get all keys, sorted.
    pub fn sorted_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.data.keys().map(|s| s.as_str()).collect();
        keys.sort();
        keys
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Create a store pre-populated with n entries.
    pub fn with_entries(n: usize) -> Self {
        let mut store = Self::new();
        for i in 0..n {
            store.insert(&format!("key-{}", i), &format!("value-{}", i));
        }
        store
    }
}

impl Default for BenchStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// String operations for benchmarking
// ---------------------------------------------------------------------------

/// Concatenate strings using format! macro.
pub fn concat_format(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result.push_str(&format!("item-{} ", i));
    }
    result
}

/// Concatenate strings using push_str.
pub fn concat_push(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result.push_str("item-");
        result.push_str(&i.to_string());
        result.push(' ');
    }
    result
}

/// Concatenate strings using collect.
pub fn concat_collect(n: usize) -> String {
    (0..n)
        .map(|i| format!("item-{} ", i))
        .collect::<String>()
}

// ---------------------------------------------------------------------------
// Sorting benchmark helpers
// ---------------------------------------------------------------------------

/// Generate a vector of random-ish integers.
pub fn generate_random_vec(size: usize) -> Vec<i32> {
    (0..size as i32).rev().collect()
}

/// Bubble sort (O(n^2)).
pub fn bubble_sort(data: &mut [i32]) {
    let len = data.len();
    for i in 0..len {
        for j in 0..len - 1 - i {
            if data[j] > data[j + 1] {
                data.swap(j, j + 1);
            }
        }
    }
}

/// Standard library sort.
pub fn std_sort(data: &mut [i32]) {
    data.sort();
}

// ---------------------------------------------------------------------------
// Benchmark result analysis
// ---------------------------------------------------------------------------

/// Statistics from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub name: String,
    pub iterations: usize,
    pub times_ns: Vec<u64>,
}

impl BenchmarkStats {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            iterations: 0,
            times_ns: Vec::new(),
        }
    }

    pub fn record(&mut self, time_ns: u64) {
        self.times_ns.push(time_ns);
        self.iterations += 1;
    }

    pub fn mean_ns(&self) -> f64 {
        if self.times_ns.is_empty() {
            return 0.0;
        }
        self.times_ns.iter().sum::<u64>() as f64 / self.times_ns.len() as f64
    }

    pub fn median_ns(&self) -> f64 {
        if self.times_ns.is_empty() {
            return 0.0;
        }
        let mut sorted = self.times_ns.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
        } else {
            sorted[mid] as f64
        }
    }

    pub fn min_ns(&self) -> u64 {
        self.times_ns.iter().copied().min().unwrap_or(0)
    }

    pub fn max_ns(&self) -> u64 {
        self.times_ns.iter().copied().max().unwrap_or(0)
    }

    pub fn stddev_ns(&self) -> f64 {
        if self.times_ns.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_ns();
        let variance: f64 = self
            .times_ns
            .iter()
            .map(|&t| (t as f64 - mean).powi(2))
            .sum::<f64>()
            / (self.times_ns.len() - 1) as f64;
        variance.sqrt()
    }
}

/// Compare two benchmark results.
pub fn compare_benchmarks(a: &BenchmarkStats, b: &BenchmarkStats) -> BenchmarkComparison {
    let ratio = b.mean_ns() / a.mean_ns();
    BenchmarkComparison {
        baseline: a.name.clone(),
        candidate: b.name.clone(),
        mean_ratio: ratio,
        faster: if ratio > 1.0 {
            a.name.clone()
        } else {
            b.name.clone()
        },
        speedup: if ratio > 1.0 { ratio } else { 1.0 / ratio },
    }
}

#[derive(Debug)]
pub struct BenchmarkComparison {
    pub baseline: String,
    pub candidate: String,
    pub mean_ratio: f64,
    pub faster: String,
    pub speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_store_basic() {
        let mut store = BenchStore::new();
        store.insert("key", "value");
        assert_eq!(store.get("key"), Some("value"));
        assert!(store.contains("key"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_bench_store_with_entries() {
        let store = BenchStore::with_entries(100);
        assert_eq!(store.len(), 100);
        assert!(store.get("key-0").is_some());
        assert!(store.get("key-99").is_some());
    }

    #[test]
    fn test_bench_store_sorted_keys() {
        let mut store = BenchStore::new();
        store.insert("c", "3");
        store.insert("a", "1");
        store.insert("b", "2");

        let sorted = store.sorted_keys();
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_bench_store_delete() {
        let mut store = BenchStore::with_entries(10);
        assert!(store.delete("key-5"));
        assert!(!store.contains("key-5"));
        assert_eq!(store.len(), 9);
    }

    #[test]
    fn test_concat_format() {
        let result = concat_format(3);
        assert!(result.contains("item-0"));
        assert!(result.contains("item-2"));
    }

    #[test]
    fn test_concat_push() {
        let result = concat_push(3);
        assert!(result.contains("item-0"));
        assert!(result.contains("item-2"));
    }

    #[test]
    fn test_concat_collect() {
        let result = concat_collect(3);
        assert!(result.contains("item-0"));
        assert!(result.contains("item-2"));
    }

    #[test]
    fn test_concat_equivalence() {
        let n = 10;
        let a = concat_format(n);
        let b = concat_push(n);
        let c = concat_collect(n);
        // All should produce the same result
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn test_generate_random_vec() {
        let v = generate_random_vec(100);
        assert_eq!(v.len(), 100);
    }

    #[test]
    fn test_bubble_sort() {
        let mut data = generate_random_vec(100);
        bubble_sort(&mut data);
        for i in 1..data.len() {
            assert!(data[i - 1] <= data[i]);
        }
    }

    #[test]
    fn test_std_sort() {
        let mut data = generate_random_vec(100);
        std_sort(&mut data);
        for i in 1..data.len() {
            assert!(data[i - 1] <= data[i]);
        }
    }

    #[test]
    fn test_benchmark_stats() {
        let mut stats = BenchmarkStats::new("test");
        stats.record(100);
        stats.record(200);
        stats.record(300);

        assert_eq!(stats.iterations, 3);
        assert_eq!(stats.mean_ns(), 200.0);
        assert_eq!(stats.median_ns(), 200.0);
        assert_eq!(stats.min_ns(), 100);
        assert_eq!(stats.max_ns(), 300);
    }

    #[test]
    fn test_benchmark_stats_empty() {
        let stats = BenchmarkStats::new("empty");
        assert_eq!(stats.mean_ns(), 0.0);
        assert_eq!(stats.median_ns(), 0.0);
        assert_eq!(stats.min_ns(), 0);
        assert_eq!(stats.max_ns(), 0);
    }

    #[test]
    fn test_benchmark_stats_stddev() {
        let mut stats = BenchmarkStats::new("test");
        stats.record(100);
        stats.record(200);
        stats.record(300);

        let stddev = stats.stddev_ns();
        assert!(stddev > 0.0);
    }

    #[test]
    fn test_benchmark_stats_stddev_single() {
        let mut stats = BenchmarkStats::new("test");
        stats.record(100);
        assert_eq!(stats.stddev_ns(), 0.0);
    }

    #[test]
    fn test_compare_benchmarks() {
        let mut fast = BenchmarkStats::new("fast");
        fast.record(100);
        fast.record(100);

        let mut slow = BenchmarkStats::new("slow");
        slow.record(200);
        slow.record(200);

        let comparison = compare_benchmarks(&fast, &slow);
        assert_eq!(comparison.faster, "fast");
        assert!(comparison.speedup > 1.0);
    }

    #[test]
    fn test_compare_benchmarks_equal() {
        let mut a = BenchmarkStats::new("a");
        a.record(100);

        let mut b = BenchmarkStats::new("b");
        b.record(100);

        let comparison = compare_benchmarks(&a, &b);
        assert!(comparison.speedup >= 1.0);
    }

    #[test]
    fn test_benchmark_stats_clone() {
        let mut stats = BenchmarkStats::new("test");
        stats.record(100);
        let cloned = stats.clone();
        assert_eq!(cloned.iterations, stats.iterations);
    }
}
