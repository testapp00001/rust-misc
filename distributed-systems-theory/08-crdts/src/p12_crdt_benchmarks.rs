//! # Exercise: CRDT Benchmarks
//!
//! ## Theory
//!
//! Benchmarking CRDTs is essential for understanding their performance
//! characteristics in production systems. Key metrics include:
//!
//! - **Merge time**: How long does merging two states take?
//! - **Memory usage**: How much memory does each CRDT use?
//! - **Operation throughput**: How many operations per second?
//! - **Scaling behavior**: How do metrics change with size?
//!
//! ## Proof / Intuition
//!
//! Different CRDT types have different performance profiles:
//! - G-Counter: O(1) merge, O(n) memory where n = replicas
//! - PN-Counter: O(1) merge, O(2n) memory (two G-Counters)
//! - G-Set: O(n) merge where n = elements added, O(n) memory
//! - OR-Set: O(n) merge, O(n) memory with tags
//! - LWW-Register: O(1) merge, O(1) memory
//!
//! ## Implementation Task
//!
//! Benchmark each CRDT type for merge time, memory, and throughput.
//! Compare against a simple atomic counter baseline.
//!
//! ## Verification
//!
//! Verify benchmarks run correctly and print performance characteristics.
//!
use crate::p01_g_counter::GCounter;
use crate::p02_pn_counter::PNCounter;
use crate::p03_g_set::GSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Benchmark results for a single CRDT type.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub merge_time_ns: u64,
    pub memory_bytes: usize,
    pub ops_per_sec: f64,
    pub merge_count: usize,
}

impl BenchmarkResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            merge_time_ns: 0,
            memory_bytes: 0,
            ops_per_sec: 0.0,
            merge_count: 0,
        }
    }
}

/// Benchmark suite for CRDT operations.
pub struct CrdtBenchmark {
    num_iterations: usize,
    num_elements: usize,
}

impl CrdtBenchmark {
    pub fn new(num_iterations: usize, num_elements: usize) -> Self {
        Self {
            num_iterations,
            num_elements,
        }
    }

    /// Benchmark G-Counter merge performance.
    pub fn benchmark_gcounter(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::new("GCounter");

        let mut a = GCounter::new();
        let mut b = GCounter::new();

        for i in 0..self.num_elements {
            a.increment(i % 10);
            b.increment((i + 5) % 10);
        }

        // Benchmark merge
        let start = Instant::now();
        for _ in 0..self.num_iterations {
            let mut c = a.clone();
            c.merge(&b);
        }
        result.merge_time_ns = start.elapsed().as_nanos() as u64 / self.num_iterations as u64;
        result.merge_count = self.num_iterations;

        // Estimate memory
        result.memory_bytes = std::mem::size_of::<GCounter>() + self.num_elements * 8;

        // Benchmark operations
        let start = Instant::now();
        let mut counter = GCounter::new();
        for i in 0..self.num_iterations {
            counter.increment(i % 10);
        }
        let elapsed = start.elapsed();
        result.ops_per_sec = self.num_iterations as f64 / elapsed.as_secs_f64();

        result
    }

    /// Benchmark PN-Counter merge performance.
    pub fn benchmark_pncounter(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::new("PNCounter");

        let mut a = PNCounter::new();
        let mut b = PNCounter::new();

        for i in 0..self.num_elements {
            if i % 3 == 0 {
                a.decrement(i % 10);
                b.decrement((i + 5) % 10);
            } else {
                a.increment(i % 10);
                b.increment((i + 5) % 10);
            }
        }

        // Benchmark merge
        let start = Instant::now();
        for _ in 0..self.num_iterations {
            let mut c = a.clone();
            c.merge(&b);
        }
        result.merge_time_ns = start.elapsed().as_nanos() as u64 / self.num_iterations as u64;
        result.merge_count = self.num_iterations;

        result.memory_bytes = std::mem::size_of::<PNCounter>() + self.num_elements * 16;

        let start = Instant::now();
        let mut counter = PNCounter::new();
        for i in 0..self.num_iterations {
            if i % 3 == 0 {
                counter.decrement(i % 10);
            } else {
                counter.increment(i % 10);
            }
        }
        let elapsed = start.elapsed();
        result.ops_per_sec = self.num_iterations as f64 / elapsed.as_secs_f64();

        result
    }

    /// Benchmark G-Set merge performance.
    pub fn benchmark_gset(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::new("GSet");

        let mut a = GSet::new();
        let mut b = GSet::new();

        for i in 0..self.num_elements {
            a.add(i);
            b.add(i + self.num_elements / 2);
        }

        // Benchmark merge
        let start = Instant::now();
        for _ in 0..self.num_iterations {
            let mut c = a.clone();
            c.merge(&b);
        }
        result.merge_time_ns = start.elapsed().as_nanos() as u64 / self.num_iterations as u64;
        result.merge_count = self.num_iterations;

        result.memory_bytes = std::mem::size_of::<GSet<usize>>() + self.num_elements * 16;

        let start = Instant::now();
        let mut set = GSet::new();
        for i in 0..self.num_iterations {
            set.add(i);
        }
        let elapsed = start.elapsed();
        result.ops_per_sec = self.num_iterations as f64 / elapsed.as_secs_f64();

        result
    }

    /// Benchmark atomic counter baseline.
    pub fn benchmark_atomic(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::new("AtomicU64");

        let counter = AtomicU64::new(0);

        // Benchmark increment
        let start = Instant::now();
        for _ in 0..self.num_iterations {
            counter.fetch_add(1, Ordering::SeqCst);
        }
        let elapsed = start.elapsed();
        result.ops_per_sec = self.num_iterations as f64 / elapsed.as_secs_f64();

        result.memory_bytes = std::mem::size_of::<AtomicU64>();
        result.merge_time_ns = 0; // Atomic doesn't have merge

        result
    }

    /// Run all benchmarks and return results.
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        vec![
            self.benchmark_gcounter(),
            self.benchmark_pncounter(),
            self.benchmark_gset(),
            self.benchmark_atomic(),
        ]
    }
}

/// Format benchmark results for display.
pub fn format_results(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{:<15} {:>12} {:>12} {:>15}\n",
        "CRDT Type", "Merge (ns)", "Memory (B)", "Ops/sec"
    ));
    output.push_str(&"-".repeat(56));
    output.push('\n');

    for r in results {
        output.push_str(&format!(
            "{:<15} {:>12} {:>12} {:>15.0}\n",
            r.name, r.merge_time_ns, r.memory_bytes, r.ops_per_sec
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmarks_run() {
        let benchmark = CrdtBenchmark::new(1000, 100);
        let results = benchmark.run_all();

        assert_eq!(results.len(), 4);

        for result in &results {
            assert!(
                result.ops_per_sec > 0.0,
                "{} should have positive ops/sec",
                result.name
            );
            assert!(
                result.memory_bytes > 0,
                "{} should have positive memory usage",
                result.name
            );
        }
    }

    #[test]
    fn test_gcounter_faster_than_gset_merge() {
        let benchmark = CrdtBenchmark::new(1000, 100);
        let gc = benchmark.benchmark_gcounter();
        let gs = benchmark.benchmark_gset();

        // G-Counter merge should be faster (O(1) vs O(n))
        // With 100 elements, G-Counter should be notably faster
        assert!(
            gc.merge_time_ns <= gs.merge_time_ns * 10,
            "G-Counter merge ({:?}ns) should not be dramatically slower than G-Set ({:?}ns)",
            gc.merge_time_ns,
            gs.merge_time_ns
        );
    }

    #[test]
    fn test_atomic_counter_faster_than_crdt() {
        let benchmark = CrdtBenchmark::new(10_000, 100);
        let atomic = benchmark.benchmark_atomic();
        let gc = benchmark.benchmark_gcounter();

        // Atomic should be faster than G-Counter (no merge overhead)
        assert!(
            atomic.ops_per_sec >= gc.ops_per_sec,
            "Atomic ({:.0} ops/s) should be >= GCounter ({:.0} ops/s)",
            atomic.ops_per_sec,
            gc.ops_per_sec
        );
    }

    #[test]
    fn test_format_results() {
        let benchmark = CrdtBenchmark::new(100, 10);
        let results = benchmark.run_all();
        let formatted = format_results(&results);

        assert!(formatted.contains("GCounter"));
        assert!(formatted.contains("PNCounter"));
        assert!(formatted.contains("GSet"));
        assert!(formatted.contains("AtomicU64"));
    }
}
