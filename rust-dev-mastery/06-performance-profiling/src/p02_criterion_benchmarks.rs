//! # Criterion Benchmarks
//!
//! Criterion is the gold standard for Rust benchmarking. It provides statistical
//! analysis, comparison reports, and beautiful HTML output. This module covers
//! how to write effective benchmarks with Criterion.
//!
//! ## Key Concepts
//! - **Benchmark groups**: Organize related benchmarks together
//! - **Custom inputs**: Parameterize benchmarks with different data sizes
//! - **Statistical analysis**: Criterion uses bootstrap sampling for confidence intervals
//! - **Comparison reports**: Compare benchmarks across runs to detect regressions

use std::collections::HashMap;

/// Demonstrates the kinds of functions you'd benchmark with Criterion.
/// The actual Criterion benchmarks would be in a benches/ directory; these are
/// the implementations that Criterion would call.

/// A simple hash map lookup benchmark target.
pub fn hashmap_lookup(map: &HashMap<u64, u64>, key: u64) -> Option<u64> {
    map.get(&key).copied()
}

/// A binary search benchmark target.
pub fn binary_search_sorted(sorted: &[u64], target: u64) -> Option<usize> {
    sorted.binary_search(&target).ok()
}

/// Linear search for comparison with binary search.
pub fn linear_search(sorted: &[u64], target: u64) -> Option<usize> {
    sorted.iter().position(|&x| x == target)
}

/// String concatenation benchmark: format! vs push_str.
pub fn concat_with_format(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        result = format!("{result}{item}");
    }
    result
}

pub fn concat_with_push(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(item);
    }
    result
}

/// Iterator vs manual loop benchmark targets.
pub fn sum_iterator(data: &[u64]) -> u64 {
    data.iter().sum()
}

pub fn sum_loop(data: &[u64]) -> u64 {
    let mut sum = 0u64;
    for &item in data {
        sum += item;
    }
    sum
}

/// Sorting benchmark targets.
pub fn sort_vec(data: &mut Vec<u64>) {
    data.sort();
}

pub fn sort_unstable(data: &mut Vec<u64>) {
    data.sort_unstable();
}

/// Generates test data of various sizes for benchmarking.
pub fn generate_data(size: usize) -> Vec<u64> {
    (0..size as u64).collect()
}

pub fn generate_shuffled(size: usize) -> Vec<u64> {
    let mut data: Vec<u64> = (0..size as u64).collect();
    // Simple pseudo-shuffle for deterministic results
    for i in 0..size {
        let j = (i.wrapping_mul(2654435761)) % size;
        data.swap(i, j);
    }
    data
}

pub fn generate_hashmap(size: usize) -> HashMap<u64, u64> {
    (0..size as u64).map(|i| (i, i * 2)).collect()
}

/// A benchmark runner that provides structured results.
/// In practice, you'd use Criterion's built-in analysis.
pub struct BenchRunner {
    name: String,
    results: Vec<BenchIteration>,
}

#[derive(Debug, Clone)]
pub struct BenchIteration {
    pub input_size: usize,
    pub duration_ns: u64,
    pub iterations: u64,
}

impl BenchRunner {
    pub fn new(name: impl Into<String>) -> Self {
        BenchRunner {
            name: name.into(),
            results: Vec::new(),
        }
    }

    pub fn run_with_size<F>(&mut self, size: usize, iterations: u64, mut f: F)
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..100 {
            f();
        }

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            f();
        }
        let elapsed = start.elapsed();

        self.results.push(BenchIteration {
            input_size: size,
            duration_ns: elapsed.as_nanos() as u64,
            iterations,
        });
    }

    pub fn report(&self) -> String {
        let mut report = format!("Benchmark: {}\n", self.name);
        report.push_str(&format!(
            "{:<15} {:>12} {:>12} {:>15}\n",
            "Input Size", "Iterations", "Total(ms)", "Per-iter(ns)"
        ));
        report.push_str(&"-".repeat(56));
        report.push('\n');

        for r in &self.results {
            let total_ms = r.duration_ns as f64 / 1_000_000.0;
            let per_iter = r.duration_ns as f64 / r.iterations as f64;
            report.push_str(&format!(
                "{:<15} {:>12} {:>12.3} {:>15.1}\n",
                r.input_size, r.iterations, total_ms, per_iter
            ));
        }

        report
    }

    pub fn results(&self) -> &[BenchIteration] {
        &self.results
    }
}

/// Demonstrates what a Criterion benchmark group looks like.
/// This is reference code showing the Criterion API; it would be in benches/.
///
/// ```rust,ignore
/// use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
///
/// fn bench_hashmap_lookup(c: &mut Criterion) {
///     let mut group = c.benchmark_group("hashmap_lookup");
///
///     for size in [100, 1_000, 10_000, 100_000] {
///         let map = generate_hashmap(size);
///         group.bench_with_input(
///             BenchmarkId::new("lookup", size),
///             &map,
///             |b, map| {
///                 b.iter(|| hashmap_lookup(map, size as u64 / 2));
///             },
///         );
///     }
///     group.finish();
/// }
///
/// fn bench_sorting(c: &mut Criterion) {
///     let mut group = c.benchmark_group("sorting");
///
///     for size in [100, 1_000, 10_000] {
///         group.bench_with_input(
///             BenchmarkId::new("sort", size),
///             &size,
///             |b, &size| {
///                 b.iter_batched(
///                     || generate_shuffled(size),
///                     |mut data| sort_vec(&mut data),
///                     criterion::BatchSize::SmallInput,
///                 );
///             },
///         );
///     }
///     group.finish();
/// }
///
/// criterion_group!(benches, bench_hashmap_lookup, bench_sorting);
/// criterion_main!(benches);
/// ```
pub struct CriterionReference;

/// Comparing two implementations to determine which is faster.
pub fn compare_implementations<F1, F2>(
    name1: &str,
    mut f1: F1,
    name2: &str,
    mut f2: F2,
    iterations: usize,
) -> ComparisonResult
where
    F1: FnMut(),
    F2: FnMut(),
{
    // Warmup
    for _ in 0..100 {
        f1();
        f2();
    }

    // Measure f1
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f1();
    }
    let duration1 = start.elapsed();

    // Measure f2
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f2();
    }
    let duration2 = start.elapsed();

    let faster = if duration1 < duration2 {
        name1.to_string()
    } else {
        name2.to_string()
    };

    let ratio = if duration1 < duration2 {
        duration2.as_nanos() as f64 / duration1.as_nanos() as f64
    } else {
        duration1.as_nanos() as f64 / duration2.as_nanos() as f64
    };

    ComparisonResult {
        name1: name1.to_string(),
        duration1,
        name2: name2.to_string(),
        duration2,
        faster,
        ratio,
    }
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub name1: String,
    pub duration1: std::time::Duration,
    pub name2: String,
    pub duration2: std::time::Duration,
    pub faster: String,
    pub ratio: f64,
}

impl std::fmt::Display for ComparisonResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:?} vs {}: {:?} -> {} is {:.2}x faster",
            self.name1, self.duration1, self.name2, self.duration2, self.faster, self.ratio
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_data() {
        let data = generate_data(100);
        assert_eq!(data.len(), 100);
        assert_eq!(data[0], 0);
        assert_eq!(data[99], 99);
    }

    #[test]
    fn test_generate_shuffled() {
        let data = generate_shuffled(100);
        assert_eq!(data.len(), 100);
        // Should be shuffled (not sorted)
        // Just check it has all elements
        let mut sorted = data.clone();
        sorted.sort();
        let expected: Vec<u64> = (0..100).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_hashmap_lookup() {
        let map = generate_hashmap(1000);
        assert_eq!(hashmap_lookup(&map, 500), Some(1000));
        assert_eq!(hashmap_lookup(&map, 9999), None);
    }

    #[test]
    fn test_binary_search_vs_linear() {
        let data = generate_data(10000);

        // Both should find the same element
        assert_eq!(binary_search_sorted(&data, 5000), Some(5000));
        assert_eq!(linear_search(&data, 5000), Some(5000));

        // Both should return None for missing element
        assert_eq!(binary_search_sorted(&data, 99999), None);
        assert_eq!(linear_search(&data, 99999), None);
    }

    #[test]
    fn test_concat_methods() {
        let items = vec!["a", "b", "c"];
        assert_eq!(concat_with_format(&items), "abc");
        assert_eq!(concat_with_push(&items), "abc");
    }

    #[test]
    fn test_sum_methods() {
        let data = generate_data(100);
        assert_eq!(sum_iterator(&data), sum_loop(&data));
    }

    #[test]
    fn test_sort_methods() {
        let data = generate_shuffled(100);

        let mut d1 = data.clone();
        sort_vec(&mut d1);

        let mut d2 = data;
        sort_unstable(&mut d2);

        assert_eq!(d1, d2);
    }

    #[test]
    fn test_bench_runner() {
        let mut runner = BenchRunner::new("test");

        let data = generate_data(1000);
        runner.run_with_size(1000, 100, || {
            std::hint::black_box(sum_iterator(&data));
        });

        assert_eq!(runner.results().len(), 1);
        assert!(runner.results()[0].duration_ns > 0);
    }

    #[test]
    fn test_bench_runner_report() {
        let mut runner = BenchRunner::new("sort_test");
        runner.run_with_size(100, 50, || {});
        runner.run_with_size(1000, 50, || {});

        let report = runner.report();
        assert!(report.contains("sort_test"));
        assert!(report.contains("100"));
        assert!(report.contains("1000"));
    }

    #[test]
    fn test_compare_implementations() {
        let result = compare_implementations(
            "fast",
            || { std::hint::black_box(1 + 1); },
            "slow",
            || {
                for _ in 0..10 {
                    std::hint::black_box(1 + 1);
                }
            },
            10000,
        );

        assert_eq!(result.faster, "fast");
        assert!(result.ratio >= 1.0);
    }

    #[test]
    fn test_comparison_result_display() {
        let result = compare_implementations(
            "a",
            || {},
            "b",
            || {},
            100,
        );
        let display = format!("{result}");
        assert!(display.contains("a"));
        assert!(display.contains("b"));
    }
}
