//! # Benchmarking
//!
//! Benchmarking measures code performance. In Rust, you can use:
//! - `#[bench]` (nightly only)
//! - `criterion` crate (recommended)
//! - Manual timing with `std::time::Instant`
//!
//! This lesson demonstrates benchmarking patterns using standard library timing.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A simple benchmark runner.
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
}

impl BenchmarkResult {
    pub fn ops_per_second(&self) -> f64 {
        self.iterations as f64 / self.total_duration.as_secs_f64()
    }

    pub fn ns_per_op(&self) -> f64 {
        self.avg_duration.as_nanos() as f64
    }
}

pub fn benchmark<F: FnMut()>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult {
    // Warmup
    for _ in 0..iterations.min(100) {
        f();
    }

    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let total_duration: Duration = durations.iter().sum();
    let min_duration = durations.iter().min().copied().unwrap_or_default();
    let max_duration = durations.iter().max().copied().unwrap_or_default();
    let avg_duration = total_duration / iterations as u32;

    BenchmarkResult {
        name: name.to_string(),
        iterations,
        total_duration,
        min_duration,
        max_duration,
        avg_duration,
    }
}

/// Demonstrates comparing two implementations.
pub fn compare_benchmarks<F1, F2>(
    name1: &str,
    mut f1: F1,
    name2: &str,
    mut f2: F2,
    iterations: usize,
) -> (BenchmarkResult, BenchmarkResult)
where
    F1: FnMut(),
    F2: FnMut(),
{
    let r1 = benchmark(name1, iterations, || f1());
    let r2 = benchmark(name2, iterations, || f2());
    (r1, r2)
}

/// Data structures for benchmarking.
pub fn linear_search(data: &[i32], target: i32) -> Option<usize> {
    data.iter().position(|&x| x == target)
}

pub fn binary_search(data: &[i32], target: i32) -> Option<usize> {
    let mut left = 0;
    let mut right = data.len();
    while left < right {
        let mid = left + (right - left) / 2;
        match data[mid].cmp(&target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }
    None
}

/// String operations for benchmarking.
pub fn string_concat_loop(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&i.to_string());
    }
    s
}

pub fn string_concat_collect(n: usize) -> String {
    (0..n).map(|i| i.to_string()).collect()
}

/// HashMap operations for benchmarking.
pub fn hashmap_insert_sequential(n: usize) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    for i in 0..n {
        map.insert(i, i * 2);
    }
    map
}

pub fn hashmap_insert_with_capacity(n: usize) -> HashMap<usize, usize> {
    let mut map = HashMap::with_capacity(n);
    for i in 0..n {
        map.insert(i, i * 2);
    }
    map
}

/// Vec operations for benchmarking.
pub fn vec_push_loop(n: usize) -> Vec<usize> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }
    v
}

pub fn vec_push_with_capacity(n: usize) -> Vec<usize> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        v.push(i);
    }
    v
}

/// Sorting algorithms for benchmarking.
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

pub fn quicksort(data: &mut [i32]) {
    if data.len() <= 1 {
        return;
    }
    let pivot = data[data.len() / 2];
    let mut left = Vec::new();
    let mut middle = Vec::new();
    let mut right = Vec::new();
    for &x in data.iter() {
        if x < pivot {
            left.push(x);
        } else if x == pivot {
            middle.push(x);
        } else {
            right.push(x);
        }
    }
    quicksort(&mut left);
    quicksort(&mut right);
    let mut i = 0;
    for x in left.into_iter().chain(middle).chain(right) {
        data[i] = x;
        i += 1;
    }
}

/// Measures throughput (items per second).
pub fn measure_throughput<F: FnMut()>(name: &str, items_per_op: usize, iterations: usize, mut f: F) -> ThroughputResult {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let total_items = items_per_op * iterations;
    let items_per_sec = total_items as f64 / elapsed.as_secs_f64();

    ThroughputResult {
        name: name.to_string(),
        total_items,
        elapsed,
        items_per_second: items_per_sec,
    }
}

pub struct ThroughputResult {
    pub name: String,
    pub total_items: usize,
    pub elapsed: Duration,
    pub items_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_basic() {
        let result = benchmark("empty_loop", 1000, || {
            let mut sum = 0u64;
            for i in 0..100 {
                sum = sum.wrapping_add(i);
            }
            let _ = sum;
        });
        assert_eq!(result.iterations, 1000);
        assert!(result.avg_duration.as_nanos() > 0);
        assert!(result.ops_per_second() > 0.0);
    }

    #[test]
    fn test_linear_vs_binary_search() {
        let data: Vec<i32> = (0..10000).collect();

        let r1 = benchmark("linear_search", 1000, || {
            let _ = linear_search(&data, 9999);
        });

        let r2 = benchmark("binary_search", 1000, || {
            let _ = binary_search(&data, 9999);
        });

        // Binary search should be faster
        assert!(
            r2.avg_duration <= r1.avg_duration,
            "binary_search ({:?}) should be faster than linear_search ({:?})",
            r2.avg_duration,
            r1.avg_duration
        );
    }

    #[test]
    fn test_string_concat_comparison() {
        let n = 1000;
        let r1 = benchmark("push_str_loop", 100, || {
            let _ = string_concat_loop(n);
        });
        let r2 = benchmark("collect", 100, || {
            let _ = string_concat_collect(n);
        });
        // Both should complete successfully
        assert!(r1.iterations == 100);
        assert!(r2.iterations == 100);
    }

    #[test]
    fn test_hashmap_capacity() {
        let n = 10000;
        let r1 = benchmark("hashmap_no_capacity", 100, || {
            let _ = hashmap_insert_sequential(n);
        });
        let r2 = benchmark("hashmap_with_capacity", 100, || {
            let _ = hashmap_insert_with_capacity(n);
        });
        // With capacity should be faster or equal
        assert!(r2.avg_duration <= r1.avg_duration * 2);
    }

    #[test]
    fn test_vec_capacity() {
        let n = 10000;
        let r1 = benchmark("vec_no_capacity", 100, || {
            let _ = vec_push_loop(n);
        });
        let r2 = benchmark("vec_with_capacity", 100, || {
            let _ = vec_push_with_capacity(n);
        });
        assert!(r2.avg_duration <= r1.avg_duration * 2);
    }

    #[test]
    fn test_throughput() {
        let result = measure_throughput("vec_push", 10000, 100, || {
            let _ = vec_push_with_capacity(10000);
        });
        assert!(result.items_per_second > 0.0);
        assert_eq!(result.total_items, 1_000_000);
    }

    #[test]
    fn test_benchmark_result_ops_per_second() {
        let result = benchmark("simple", 1000, || {});
        assert!(result.ops_per_second() > 0.0);
        assert!(result.ns_per_op() > 0.0);
    }
}
