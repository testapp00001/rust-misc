//! # Memory Profiling
//!
//! Profiling memory usage helps identify allocation hotspots, leaks, and
//! excessive memory consumption. This module covers profiling strategies
//! and tools for Rust applications.
//!
//! ## Tools:
//!
//! | Tool | Type | Notes |
//! |------|------|-------|
//! | `dhat` | Heap profiling | Rust crate, easy integration |
//! | `heaptrack` | Heap profiling | Linux, detailed call graphs |
//! | `massif` (Valgrind) | Heap profiling | Cross-platform |
//! | `jemalloc` | Allocator | Built-in profiling |
//! | Custom tracking | Allocation counting | Zero overhead |
//!
//! ## Profiling Workflow:
//!
//! 1. Identify the workload to profile
//! 2. Enable profiling (dhat, jemalloc, etc.)
//! 3. Run the workload
//! 4. Analyze results (allocation count, sizes, call stacks)
//! 5. Optimize hotspots
//! 6. Re-profile to verify improvement

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Allocation profile data for a specific site.
#[derive(Debug, Clone)]
pub struct AllocationSite {
    pub location: String,
    pub count: u64,
    pub total_bytes: u64,
    pub peak_bytes: u64,
    pub current_bytes: u64,
}

impl AllocationSite {
    pub fn new(location: &str) -> Self {
        Self {
            location: location.to_string(),
            count: 0,
            total_bytes: 0,
            peak_bytes: 0,
            current_bytes: 0,
        }
    }

    pub fn record_alloc(&mut self, size: u64) {
        self.count += 1;
        self.total_bytes += size;
        self.current_bytes += size;
        self.peak_bytes = self.peak_bytes.max(self.current_bytes);
    }

    pub fn record_dealloc(&mut self, size: u64) {
        self.current_bytes = self.current_bytes.saturating_sub(size);
    }

    pub fn average_size(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.total_bytes as f64 / self.count as f64
    }
}

/// Memory profiler that tracks allocations by call site.
pub struct MemoryProfiler {
    sites: Mutex<HashMap<String, AllocationSite>>,
    total_allocs: AtomicU64,
    total_deallocs: AtomicU64,
    start_time: Instant,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            sites: Mutex::new(HashMap::new()),
            total_allocs: AtomicU64::new(0),
            total_deallocs: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record an allocation.
    pub fn record_alloc(&self, location: &str, size: u64) {
        self.total_allocs.fetch_add(1, Ordering::Relaxed);
        let mut sites = self.sites.lock().unwrap();
        sites
            .entry(location.to_string())
            .or_insert_with(|| AllocationSite::new(location))
            .record_alloc(size);
    }

    /// Record a deallocation.
    pub fn record_dealloc(&self, location: &str, size: u64) {
        self.total_deallocs.fetch_add(1, Ordering::Relaxed);
        let mut sites = self.sites.lock().unwrap();
        if let Some(site) = sites.get_mut(location) {
            site.record_dealloc(size);
        }
    }

    /// Get the top N allocation sites by total bytes.
    pub fn top_sites(&self, n: usize) -> Vec<AllocationSite> {
        let sites = self.sites.lock().unwrap();
        let mut sorted: Vec<AllocationSite> = sites.values().cloned().collect();
        sorted.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        sorted.into_iter().take(n).collect()
    }

    /// Get total allocation count.
    pub fn total_allocations(&self) -> u64 {
        self.total_allocs.load(Ordering::Relaxed)
    }

    /// Get profiling duration.
    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Generate a profiling report.
    pub fn report(&self) -> ProfilingReport {
        let sites = self.sites.lock().unwrap();
        let total_bytes: u64 = sites.values().map(|s| s.total_bytes).sum();
        let peak_bytes: u64 = sites.values().map(|s| s.peak_bytes).sum();
        let current_bytes: u64 = sites.values().map(|s| s.current_bytes).sum();
        let num_sites = sites.len();

        let mut top: Vec<AllocationSite> = sites.values().cloned().collect();
        top.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        let top_sites = top.into_iter().take(10).collect();

        ProfilingReport {
            total_allocations: self.total_allocations(),
            total_deallocations: self.total_deallocs.load(Ordering::Relaxed),
            total_bytes,
            peak_bytes,
            current_bytes,
            num_sites,
            duration: self.duration(),
            top_sites,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub total_bytes: u64,
    pub peak_bytes: u64,
    pub current_bytes: u64,
    pub num_sites: usize,
    pub duration: std::time::Duration,
    pub top_sites: Vec<AllocationSite>,
}

impl ProfilingReport {
    pub fn to_string_pretty(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Memory Profiling Report ===\n\n");
        output.push_str(&format!("Duration: {:?}\n", self.duration));
        output.push_str(&format!("Total allocations: {}\n", self.total_allocations));
        output.push_str(&format!("Total deallocations: {}\n", self.total_deallocations));
        output.push_str(&format!("Total bytes allocated: {}\n", self.total_bytes));
        output.push_str(&format!("Peak memory usage: {}\n", self.peak_bytes));
        output.push_str(&format!("Current memory usage: {}\n", self.current_bytes));
        output.push_str(&format!("Allocation sites: {}\n\n", self.num_sites));

        output.push_str("Top allocation sites:\n");
        for (i, site) in self.top_sites.iter().enumerate() {
            output.push_str(&format!(
                "  {}. {} - {} allocs, {} bytes total, {:.0} avg\n",
                i + 1,
                site.location,
                site.count,
                site.total_bytes,
                site.average_size()
            ));
        }
        output
    }

    pub fn leak_detected(&self) -> bool {
        self.current_bytes > 0 && self.total_deallocations < self.total_allocations
    }
}

/// Allocation flamegraph data generator.
pub struct FlameGraphData {
    nodes: HashMap<String, FlameNode>,
}

#[derive(Debug, Clone)]
pub struct FlameNode {
    pub name: String,
    pub self_bytes: u64,
    pub total_bytes: u64,
    pub children: HashMap<String, FlameNode>,
}

impl FlameGraphData {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Record an allocation with a call stack.
    pub fn record(&mut self, stack: &[&str], bytes: u64) {
        let mut current = &mut self.nodes;
        for (i, frame) in stack.iter().enumerate() {
            let node = current
                .entry(frame.to_string())
                .or_insert_with(|| FlameNode {
                    name: frame.to_string(),
                    self_bytes: 0,
                    total_bytes: 0,
                    children: HashMap::new(),
                });
            node.total_bytes += bytes;
            if i == stack.len() - 1 {
                node.self_bytes += bytes;
            }
            current = &mut node.children;
        }
    }

    /// Get total bytes at the root level.
    pub fn total_bytes(&self) -> u64 {
        self.nodes.values().map(|n| n.total_bytes).sum()
    }
}

/// Memory snapshot for comparing memory state at different points.
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub heap_bytes: usize,
    pub stack_bytes: usize,
    pub allocations: usize,
    pub label: String,
}

impl MemorySnapshot {
    pub fn capture(label: &str) -> Self {
        Self {
            timestamp: Instant::now(),
            heap_bytes: 0, // Would use platform API in production
            stack_bytes: 0,
            allocations: 0,
            label: label.to_string(),
        }
    }

    /// Compare two snapshots to find memory growth.
    pub fn diff(&self, other: &MemorySnapshot) -> MemoryDiff {
        MemoryDiff {
            from_label: self.label.clone(),
            to_label: other.label.clone(),
            heap_growth: other.heap_bytes as i64 - self.heap_bytes as i64,
            allocation_growth: other.allocations as i64 - self.allocations as i64,
            duration: other.timestamp.duration_since(self.timestamp),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryDiff {
    pub from_label: String,
    pub to_label: String,
    pub heap_growth: i64,
    pub allocation_growth: i64,
    pub duration: std::time::Duration,
}

impl MemoryDiff {
    pub fn is_leaking(&self) -> bool {
        self.heap_growth > 0 && self.allocation_growth > 0
    }
}

/// Lightweight allocation counter for benchmarks.
pub struct AllocationCounter {
    count: AtomicU64,
    bytes: AtomicU64,
}

impl AllocationCounter {
    pub const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        }
    }

    pub fn record(&self, size: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(size, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.bytes.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_basic() {
        let profiler = MemoryProfiler::new();
        profiler.record_alloc("main::create_vec", 1024);
        profiler.record_alloc("main::create_vec", 2048);
        profiler.record_alloc("main::create_string", 512);

        assert_eq!(profiler.total_allocations(), 3);

        let report = profiler.report();
        assert_eq!(report.total_allocations, 3);
        assert_eq!(report.total_bytes, 3584);
    }

    #[test]
    fn test_memory_profiler_top_sites() {
        let profiler = MemoryProfiler::new();
        for _ in 0..100 {
            profiler.record_alloc("hot_path", 100);
        }
        for _ in 0..10 {
            profiler.record_alloc("cold_path", 1000);
        }

        let top = profiler.top_sites(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].location, "hot_path");
    }

    #[test]
    fn test_allocation_site() {
        let mut site = AllocationSite::new("test");
        site.record_alloc(100);
        site.record_alloc(200);
        site.record_dealloc(100);

        assert_eq!(site.count, 2);
        assert_eq!(site.total_bytes, 300);
        assert_eq!(site.current_bytes, 200);
        assert_eq!(site.peak_bytes, 300);
        assert!((site.average_size() - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_profiling_report() {
        let profiler = MemoryProfiler::new();
        profiler.record_alloc("alloc::vec", 1024);

        let report = profiler.report();
        let pretty = report.to_string_pretty();
        assert!(pretty.contains("Memory Profiling Report"));
        assert!(pretty.contains("1024"));
    }

    #[test]
    fn test_profiling_report_leak_detection() {
        let profiler = MemoryProfiler::new();
        profiler.record_alloc("test", 100);
        // No dealloc recorded
        let report = profiler.report();
        assert!(report.leak_detected());
    }

    #[test]
    fn test_flamegraph_data() {
        let mut fg = FlameGraphData::new();
        fg.record(&["main", "process", "alloc_vec"], 1024);
        fg.record(&["main", "process", "alloc_string"], 512);
        fg.record(&["main", "init"], 256);

        assert_eq!(fg.total_bytes(), 1792);
    }

    #[test]
    fn test_memory_snapshot() {
        let snap1 = MemorySnapshot::capture("before");
        let snap2 = MemorySnapshot::capture("after");

        let diff = snap1.diff(&snap2);
        assert_eq!(diff.from_label, "before");
        assert_eq!(diff.to_label, "after");
    }

    #[test]
    fn test_allocation_counter() {
        let counter = AllocationCounter::new();
        counter.record(100);
        counter.record(200);

        assert_eq!(counter.count(), 2);
        assert_eq!(counter.bytes(), 300);

        counter.reset();
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn test_memory_diff_leaking() {
        let diff = MemoryDiff {
            from_label: "before".into(),
            to_label: "after".into(),
            heap_growth: 1000,
            allocation_growth: 10,
            duration: std::time::Duration::from_secs(1),
        };
        assert!(diff.is_leaking());

        let no_leak = MemoryDiff {
            from_label: "before".into(),
            to_label: "after".into(),
            heap_growth: 0,
            allocation_growth: 0,
            duration: std::time::Duration::from_secs(1),
        };
        assert!(!no_leak.is_leaking());
    }

    #[test]
    fn test_profiler_dealloc() {
        let profiler = MemoryProfiler::new();
        profiler.record_alloc("test", 1000);
        profiler.record_dealloc("test", 1000);

        let report = profiler.report();
        assert!(!report.leak_detected());
    }

    #[test]
    fn test_flamegraph_multiple_paths() {
        let mut fg = FlameGraphData::new();
        fg.record(&["a", "b", "c"], 100);
        fg.record(&["a", "b", "d"], 200);
        fg.record(&["a", "e"], 300);

        assert_eq!(fg.total_bytes(), 600);
    }
}
