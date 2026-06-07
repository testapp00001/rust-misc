//! # Allocator Basics
//!
//! Rust uses allocators to manage heap memory. Understanding allocators is
//! essential for optimizing memory-intensive applications. This module covers
//! the GlobalAlloc trait, custom allocators, and memory allocation patterns.
//!
//! ## Allocator Hierarchy:
//!
//! ```text
//! Global Allocator (set once per program)
//!   ├── System allocator (default - libc malloc/free)
//!   ├── jemalloc (alternative - better for many small allocations)
//!   ├── Custom allocator (tracking, arena, pool)
//!   └── Per-thread allocators (thread-local caches)
//! ```
//!
//! ## Key Traits:
//!
//! - `GlobalAlloc`: Unsafe trait for the global allocator
//! - `Allocator`: (nightly) Safe trait for custom allocators
//! - `AllocRef`: Previous name for Allocator trait

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracking allocator that wraps the system allocator and counts allocations.
/// This is useful for debugging memory usage and finding allocation hotspots.
pub struct TrackingAllocator {
    inner: System,
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    alloc_count: AtomicUsize,
    dealloc_count: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocated: AtomicUsize::new(0),
            deallocated: AtomicUsize::new(0),
            alloc_count: AtomicUsize::new(0),
            dealloc_count: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    /// Get total bytes allocated.
    pub fn total_allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get total bytes deallocated.
    pub fn total_deallocated(&self) -> usize {
        self.deallocated.load(Ordering::Relaxed)
    }

    /// Get current memory usage (allocated - deallocated).
    pub fn current_usage(&self) -> usize {
        let alloc = self.allocated.load(Ordering::Relaxed);
        let dealloc = self.deallocated.load(Ordering::Relaxed);
        alloc.saturating_sub(dealloc)
    }

    /// Get peak memory usage.
    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Get number of allocation calls.
    pub fn allocation_count(&self) -> usize {
        self.alloc_count.load(Ordering::Relaxed)
    }

    /// Get number of deallocation calls.
    pub fn deallocation_count(&self) -> usize {
        self.dealloc_count.load(Ordering::Relaxed)
    }

    /// Get average allocation size.
    pub fn average_alloc_size(&self) -> f64 {
        let count = self.allocation_count();
        if count == 0 {
            return 0.0;
        }
        self.total_allocated() as f64 / count as f64
    }

    /// Reset all counters.
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.deallocated.store(0, Ordering::Relaxed);
        self.alloc_count.store(0, Ordering::Relaxed);
        self.dealloc_count.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
    }

    /// Get a summary of allocation statistics.
    pub fn summary(&self) -> AllocationSummary {
        AllocationSummary {
            total_allocated: self.total_allocated(),
            total_deallocated: self.total_deallocated(),
            current_usage: self.current_usage(),
            peak_usage: self.peak_usage(),
            alloc_count: self.allocation_count(),
            dealloc_count: self.deallocation_count(),
            avg_alloc_size: self.average_alloc_size(),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.allocated.fetch_add(layout.size(), Ordering::Relaxed);
            self.alloc_count.fetch_add(1, Ordering::Relaxed);
            let current = self.current_usage() + layout.size();
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        self.deallocated.fetch_add(layout.size(), Ordering::Relaxed);
        self.dealloc_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct AllocationSummary {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub alloc_count: usize,
    pub dealloc_count: usize,
    pub avg_alloc_size: f64,
}

impl AllocationSummary {
    pub fn human_readable(&self) -> String {
        format!(
            "Allocations: {} (avg {:.0} B)\n\
             Total allocated: {} B\n\
             Total deallocated: {} B\n\
             Current usage: {} B\n\
             Peak usage: {} B",
            self.alloc_count,
            self.avg_alloc_size,
            self.total_allocated,
            self.total_deallocated,
            self.current_usage,
            self.peak_usage,
        )
    }
}

/// Memory budget tracker that warns when approaching limits.
pub struct MemoryBudget {
    limit_bytes: usize,
    warning_threshold: f64, // 0.0 to 1.0
    allocated: AtomicUsize,
}

impl MemoryBudget {
    pub fn new(limit_bytes: usize, warning_threshold: f64) -> Self {
        Self {
            limit_bytes,
            warning_threshold: warning_threshold.clamp(0.0, 1.0),
            allocated: AtomicUsize::new(0),
        }
    }

    /// Allocate memory within the budget.
    pub fn allocate(&self, size: usize) -> Result<usize, BudgetExceeded> {
        let current = self.allocated.fetch_add(size, Ordering::SeqCst);
        let new_total = current + size;

        if new_total > self.limit_bytes {
            self.allocated.fetch_sub(size, Ordering::SeqCst);
            return Err(BudgetExceeded {
                requested: size,
                current: current,
                limit: self.limit_bytes,
            });
        }

        Ok(new_total)
    }

    /// Deallocate memory.
    pub fn deallocate(&self, size: usize) {
        self.allocated.fetch_sub(size, Ordering::SeqCst);
    }

    /// Check if we're within the budget.
    pub fn current_usage(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Check if we're approaching the limit.
    pub fn is_warning(&self) -> bool {
        let usage = self.current_usage() as f64;
        let limit = self.limit_bytes as f64;
        usage / limit >= self.warning_threshold
    }

    pub fn remaining(&self) -> usize {
        self.limit_bytes.saturating_sub(self.current_usage())
    }

    pub fn usage_percentage(&self) -> f64 {
        (self.current_usage() as f64 / self.limit_bytes as f64) * 100.0
    }
}

#[derive(Debug)]
pub struct BudgetExceeded {
    pub requested: usize,
    pub current: usize,
    pub limit: usize,
}

impl std::fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory budget exceeded: requested {} bytes, current {} bytes, limit {} bytes",
            self.requested, self.current, self.limit
        )
    }
}

impl std::error::Error for BudgetExceeded {}

/// Helper to measure the memory cost of a closure.
pub fn measure_allocation<F, R>(f: F) -> (R, usize)
where
    F: FnOnce() -> R,
{
    let before = allocated_bytes();
    let result = f();
    let after = allocated_bytes();
    (result, after.saturating_sub(before))
}

/// Get the current process RSS (Resident Set Size) approximation.
/// This is a simplified version; real implementation would use /proc/self/status.
pub fn allocated_bytes() -> usize {
    // In a real implementation, this would read from /proc/self/statm
    // or use platform-specific APIs. For testing, we return 0.
    0
}

/// Memory alignment utilities.
pub struct AlignmentHelper;

impl AlignmentHelper {
    /// Align a size up to the given alignment.
    pub fn align_up(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    /// Check if a value is aligned.
    pub fn is_aligned(addr: usize, align: usize) -> bool {
        addr % align == 0
    }

    /// Get the alignment of a type.
    pub fn type_align<T>() -> usize {
        std::mem::align_of::<T>()
    }

    /// Get the size of a type.
    pub fn type_size<T>() -> usize {
        std::mem::size_of::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_budget_allocate() {
        let budget = MemoryBudget::new(1024, 0.8);
        assert!(budget.allocate(512).is_ok());
        assert_eq!(budget.current_usage(), 512);
    }

    #[test]
    fn test_memory_budget_exceeded() {
        let budget = MemoryBudget::new(100, 0.8);
        let result = budget.allocate(200);
        assert!(result.is_err());
        assert_eq!(budget.current_usage(), 0); // Should be rolled back
    }

    #[test]
    fn test_memory_budget_deallocate() {
        let budget = MemoryBudget::new(1024, 0.8);
        budget.allocate(512).unwrap();
        budget.deallocate(256);
        assert_eq!(budget.current_usage(), 256);
    }

    #[test]
    fn test_memory_budget_warning() {
        let budget = MemoryBudget::new(100, 0.8);
        assert!(!budget.is_warning());
        budget.allocate(85).unwrap();
        assert!(budget.is_warning());
    }

    #[test]
    fn test_memory_budget_remaining() {
        let budget = MemoryBudget::new(1000, 0.8);
        budget.allocate(300).unwrap();
        assert_eq!(budget.remaining(), 700);
    }

    #[test]
    fn test_memory_budget_usage_percentage() {
        let budget = MemoryBudget::new(200, 0.8);
        budget.allocate(100).unwrap();
        let pct = budget.usage_percentage();
        assert!((pct - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alignment_helper() {
        assert_eq!(AlignmentHelper::align_up(5, 4), 8);
        assert_eq!(AlignmentHelper::align_up(8, 4), 8);
        assert_eq!(AlignmentHelper::align_up(1, 8), 8);
        assert_eq!(AlignmentHelper::align_up(17, 16), 32);
    }

    #[test]
    fn test_alignment_check() {
        assert!(AlignmentHelper::is_aligned(16, 8));
        assert!(AlignmentHelper::is_aligned(0, 4));
        assert!(!AlignmentHelper::is_aligned(5, 4));
    }

    #[test]
    fn test_type_info() {
        assert_eq!(AlignmentHelper::type_size::<u64>(), 8);
        assert_eq!(AlignmentHelper::type_align::<u64>(), 8);
        assert_eq!(AlignmentHelper::type_size::<u32>(), 4);
        assert_eq!(AlignmentHelper::type_size::<u8>(), 1);
    }

    #[test]
    fn test_allocation_summary() {
        // Note: TrackingAllocator can't be tested directly since it's a global
        // allocator. We test the summary struct instead.
        let summary = AllocationSummary {
            total_allocated: 1024,
            total_deallocated: 512,
            current_usage: 512,
            peak_usage: 1024,
            alloc_count: 10,
            dealloc_count: 5,
            avg_alloc_size: 102.4,
        };

        let readable = summary.human_readable();
        assert!(readable.contains("1024"));
        assert!(readable.contains("512"));
    }

    #[test]
    fn test_budget_exceeded_display() {
        let err = BudgetExceeded {
            requested: 200,
            current: 900,
            limit: 1024,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("200"));
        assert!(msg.contains("1024"));
    }

    #[test]
    fn test_budget_multiple_allocations() {
        let budget = MemoryBudget::new(1000, 0.9);
        for _ in 0..10 {
            budget.allocate(50).unwrap();
        }
        assert_eq!(budget.current_usage(), 500);
        assert!(!budget.is_warning());
    }

    #[test]
    fn test_budget_exact_limit() {
        let budget = MemoryBudget::new(100, 0.8);
        assert!(budget.allocate(100).is_ok());
        assert!(budget.allocate(1).is_err());
    }

    #[test]
    fn test_alignment_edge_cases() {
        assert_eq!(AlignmentHelper::align_up(0, 4), 0);
        assert_eq!(AlignmentHelper::align_up(1, 1), 1);
        assert_eq!(AlignmentHelper::align_up(7, 8), 8);
        assert_eq!(AlignmentHelper::align_up(8, 8), 8);
    }
}
