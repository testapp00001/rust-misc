//! # Memory Profiling
//!
//! Understanding memory usage is critical for performance. This module covers
//! tools and techniques for tracking allocations, identifying memory leaks,
//! and optimizing memory layout.
//!
//! ## Key Concepts
//! - **heaptrack/dhat**: Heap profiling tools that track every allocation
//! - **massif**: Valgrind tool for heap profiling over time
//! - **Allocation tracking**: Counting and sizing allocations in code
//! - **Memory usage reporting**: Understanding how much memory your data structures use

use std::collections::HashMap;

/// Tracks memory allocations at the application level.
/// In production, you'd use dhat or a custom global allocator.
pub struct MemoryTracker {
    allocations: Vec<AllocRecord>,
    total_allocated: usize,
    total_freed: usize,
    peak_usage: usize,
    current_usage: usize,
}

#[derive(Debug, Clone)]
pub struct AllocRecord {
    pub size: usize,
    pub timestamp: u64,
    pub label: String,
}

impl MemoryTracker {
    pub fn new() -> Self {
        MemoryTracker {
            allocations: Vec::new(),
            total_allocated: 0,
            total_freed: 0,
            peak_usage: 0,
            current_usage: 0,
        }
    }

    pub fn record_alloc(&mut self, size: usize, label: impl Into<String>) {
        self.allocations.push(AllocRecord {
            size,
            timestamp: self.allocations.len() as u64,
            label: label.into(),
        });
        self.total_allocated += size;
        self.current_usage += size;
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }
    }

    pub fn record_free(&mut self, size: usize) {
        self.total_freed += size;
        self.current_usage = self.current_usage.saturating_sub(size);
    }

    pub fn current_usage(&self) -> usize {
        self.current_usage
    }

    pub fn peak_usage(&self) -> usize {
        self.peak_usage
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn report(&self) -> MemoryReport {
        let mut by_label: HashMap<String, (usize, usize)> = HashMap::new();
        for alloc in &self.allocations {
            let entry = by_label.entry(alloc.label.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += alloc.size;
        }

        let mut breakdown: Vec<(String, usize, usize)> = by_label
            .into_iter()
            .map(|(label, (count, size))| (label, count, size))
            .collect();
        breakdown.sort_by(|a, b| b.2.cmp(&a.2));

        MemoryReport {
            total_allocated: self.total_allocated,
            total_freed: self.total_freed,
            current_usage: self.current_usage,
            peak_usage: self.peak_usage,
            allocation_count: self.allocations.len(),
            breakdown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryReport {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub breakdown: Vec<(String, usize, usize)>,
}

impl std::fmt::Display for MemoryReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Memory Report:")?;
        writeln!(f, "  Total allocated: {} bytes", self.total_allocated)?;
        writeln!(f, "  Total freed: {} bytes", self.total_freed)?;
        writeln!(f, "  Current usage: {} bytes", self.current_usage)?;
        writeln!(f, "  Peak usage: {} bytes", self.peak_usage)?;
        writeln!(f, "  Allocation count: {}", self.allocation_count)?;
        writeln!(f, "  Breakdown by label:")?;
        for (label, count, size) in &self.breakdown {
            writeln!(f, "    {label}: {count} allocs, {size} bytes")?;
        }
        Ok(())
    }
}

/// Estimates the memory size of various Rust types.
/// Uses `std::mem::size_of` for stack size and estimates for heap allocations.
pub struct SizeEstimator;

impl SizeEstimator {
    pub fn size_of<T>() -> usize {
        std::mem::size_of::<T>()
    }

    pub fn vec_size<T>(v: &Vec<T>) -> usize {
        std::mem::size_of::<Vec<T>>() + v.capacity() * std::mem::size_of::<T>()
    }

    pub fn string_size(s: &String) -> usize {
        std::mem::size_of::<String>() + s.capacity()
    }

    pub fn hashmap_size<K, V>(map: &HashMap<K, V>) -> usize {
        // Rough estimate: HashMap has overhead per bucket
        std::mem::size_of::<HashMap<K, V>>()
            + map.capacity() * (std::mem::size_of::<K>() + std::mem::size_of::<V>() + 16) // 16 bytes overhead per entry
    }

    pub fn boxed_size<T>(b: &Box<T>) -> usize {
        std::mem::size_of::<Box<T>>() + std::mem::size_of::<T>()
    }
}

/// A memory-efficient string interner.
/// Stores each unique string once and returns integer handles.
/// Useful when many duplicate strings are used (e.g., parsed identifiers).
pub struct StringInterner {
    strings: Vec<String>,
    lookup: HashMap<String, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(usize);

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            strings: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Interns a string, returning a handle. Reuses existing entry if already interned.
    pub fn intern(&mut self, s: &str) -> InternedString {
        if let Some(&idx) = self.lookup.get(s) {
            return InternedString(idx);
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        self.lookup.insert(s.to_string(), idx);
        InternedString(idx)
    }

    /// Resolves an interned string back to its value.
    pub fn resolve(&self, handle: InternedString) -> &str {
        &self.strings[handle.0]
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Returns the total bytes used by all interned strings.
    pub fn total_bytes(&self) -> usize {
        self.strings.iter().map(|s| s.len()).sum()
    }

    /// Returns the memory saved by interning (rough estimate).
    pub fn memory_saved(&self, total_references: usize) -> usize {
        let without_interning: usize = self.strings.iter().map(|s| s.len() * total_references).sum();
        let with_interning = self.total_bytes() + total_references * std::mem::size_of::<usize>();
        without_interning.saturating_sub(with_interning)
    }
}

/// An arena allocator for short-lived allocations.
/// All allocations are freed at once when the arena is dropped.
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current: Vec<u8>,
    chunk_size: usize,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self {
        Arena {
            chunks: Vec::new(),
            current: Vec::with_capacity(chunk_size),
            chunk_size,
        }
    }

    /// Allocates a slice of bytes from the arena.
    pub fn alloc_bytes(&mut self, size: usize) -> &mut [u8] {
        if self.current.len() + size > self.current.capacity() {
            // Current chunk is full, start a new one
            let new_chunk = Vec::with_capacity(self.chunk_size.max(size));
            let old = std::mem::replace(&mut self.current, new_chunk);
            self.chunks.push(old);
        }

        let start = self.current.len();
        self.current.resize(start + size, 0);
        &mut self.current[start..start + size]
    }

    /// Allocates space for a value and writes it there.
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let bytes = self.alloc_bytes(size + align - 1);

        // Align the pointer
        let ptr = bytes.as_mut_ptr() as usize;
        let aligned = (ptr + align - 1) & !(align - 1);
        let offset = aligned - ptr;

        let slot = &mut bytes[offset..offset + size];
        unsafe {
            let ptr = slot.as_mut_ptr() as *mut T;
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }

    pub fn total_allocated(&self) -> usize {
        self.chunks.iter().map(|c| c.capacity()).sum::<usize>() + self.current.capacity()
    }
}

/// Tracks memory fragmentation by analyzing allocation patterns.
pub struct FragmentationAnalyzer {
    allocations: Vec<(usize, usize)>, // (offset, size)
    total_size: usize,
}

impl FragmentationAnalyzer {
    pub fn new(total_size: usize) -> Self {
        FragmentationAnalyzer {
            allocations: Vec::new(),
            total_size,
        }
    }

    pub fn add_allocation(&mut self, offset: usize, size: usize) {
        self.allocations.push((offset, size));
        self.allocations.sort_by_key(|&(off, _)| off);
    }

    /// Calculates the fragmentation ratio (0.0 = no fragmentation, 1.0 = fully fragmented).
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.allocations.is_empty() {
            return 0.0;
        }

        let mut free_gaps = Vec::new();
        let mut last_end = 0;

        for &(offset, size) in &self.allocations {
            if offset > last_end {
                free_gaps.push(offset - last_end);
            }
            last_end = offset + size;
        }

        if last_end < self.total_size {
            free_gaps.push(self.total_size - last_end);
        }

        if free_gaps.is_empty() {
            return 0.0;
        }

        let total_free: usize = free_gaps.iter().sum();
        let largest_free = *free_gaps.iter().max().unwrap();

        if total_free == 0 {
            0.0
        } else {
            1.0 - (largest_free as f64 / total_free as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_basic() {
        let mut tracker = MemoryTracker::new();
        tracker.record_alloc(100, "vec");
        tracker.record_alloc(200, "string");

        assert_eq!(tracker.current_usage(), 300);
        assert_eq!(tracker.peak_usage(), 300);
    }

    #[test]
    fn test_memory_tracker_with_free() {
        let mut tracker = MemoryTracker::new();
        tracker.record_alloc(100, "a");
        tracker.record_alloc(200, "b");
        tracker.record_free(100);

        assert_eq!(tracker.current_usage(), 200);
        assert_eq!(tracker.peak_usage(), 300);
        assert_eq!(tracker.total_allocated(), 300);
    }

    #[test]
    fn test_memory_tracker_report() {
        let mut tracker = MemoryTracker::new();
        tracker.record_alloc(100, "data");
        tracker.record_alloc(50, "data");
        tracker.record_alloc(200, "config");

        let report = tracker.report();
        assert_eq!(report.allocation_count, 3);
        assert!(report.breakdown.len() >= 2);
    }

    #[test]
    fn test_size_estimator() {
        assert_eq!(SizeEstimator::size_of::<u8>(), 1);
        assert_eq!(SizeEstimator::size_of::<u32>(), 4);
        assert_eq!(SizeEstimator::size_of::<u64>(), 8);

        let v: Vec<u32> = Vec::with_capacity(100);
        let size = SizeEstimator::vec_size(&v);
        assert!(size >= 100 * 4);

        let s = String::with_capacity(50);
        let size = SizeEstimator::string_size(&s);
        assert!(size >= 50);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let h1 = interner.intern("hello");
        let h2 = interner.intern("world");
        let h3 = interner.intern("hello"); // Duplicate

        assert_eq!(h1, h3); // Same handle for same string
        assert_ne!(h1, h2);

        assert_eq!(interner.resolve(h1), "hello");
        assert_eq!(interner.resolve(h2), "world");
        assert_eq!(interner.len(), 2); // Only 2 unique strings
    }

    #[test]
    fn test_string_interner_memory() {
        let mut interner = StringInterner::new();

        // Intern the same string 1000 times
        let mut handles = Vec::new();
        for _ in 0..1000 {
            handles.push(interner.intern("duplicate_string"));
        }

        assert_eq!(interner.len(), 1); // Only stored once
        assert_eq!(interner.total_bytes(), "duplicate_string".len());
    }

    #[test]
    fn test_arena_basic() {
        let mut arena = Arena::new(1024);

        let bytes = arena.alloc_bytes(10);
        assert_eq!(bytes.len(), 10);

        let value = arena.alloc(42u64);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_arena_multiple_allocations() {
        let mut arena = Arena::new(64);

        for i in 0..100 {
            let val = arena.alloc(i);
            assert_eq!(*val, i);
        }
    }

    #[test]
    fn test_arena_total_allocated() {
        let mut arena = Arena::new(256);
        let before = arena.total_allocated();

        arena.alloc_bytes(100);
        let after = arena.total_allocated();

        assert!(after >= before); // Arena may have pre-allocated capacity
    }

    #[test]
    fn test_fragmentation_analyzer_no_fragmentation() {
        let mut analyzer = FragmentationAnalyzer::new(1000);
        analyzer.add_allocation(0, 500);
        analyzer.add_allocation(500, 500);

        // Fully packed, no fragmentation
        assert!((analyzer.fragmentation_ratio() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_fragmentation_analyzer_with_gaps() {
        let mut analyzer = FragmentationAnalyzer::new(1000);
        analyzer.add_allocation(0, 100);
        analyzer.add_allocation(200, 100);
        analyzer.add_allocation(400, 100);

        // Lots of free space between allocations
        let ratio = analyzer.fragmentation_ratio();
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_memory_report_display() {
        let mut tracker = MemoryTracker::new();
        tracker.record_alloc(100, "test");

        let report = tracker.report();
        let display = format!("{report}");
        assert!(display.contains("Memory Report"));
        assert!(display.contains("100 bytes"));
    }
}
