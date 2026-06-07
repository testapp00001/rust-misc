//! # Stack vs Heap Allocation
//!
//! Understanding when Rust allocates on the stack versus the heap is crucial
//! for writing performant code. Stack allocation is faster but limited in size;
//! heap allocation is slower but flexible.
//!
//! ## Stack Allocation:
//!
//! - Fixed-size types (i32, f64, arrays, tuples)
//! - Extremely fast (just moving the stack pointer)
//! - Limited size (typically 8MB on Linux)
//! - Automatically freed when scope ends
//!
//! ## Heap Allocation:
//!
//! - Dynamic-size types (Vec, String, Box)
//! - Slower (requires allocator call)
//! - Large size available
//! - Freed when owner drops

use std::mem;

/// Demonstrates the size of common types on stack vs heap.
pub struct TypeSizes;

impl TypeSizes {
    pub fn report<T>(name: &str) -> String {
        format!(
            "{}: size={}, align={}",
            name,
            mem::size_of::<T>(),
            mem::align_of::<T>()
        )
    }
}

/// Stack-allocated fixed-size buffer.
/// Avoids heap allocation for small, known-size data.
pub struct StackBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> StackBuffer<N> {
    pub fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), BufferFull> {
        if self.len >= N {
            return Err(BufferFull);
        }
        self.data[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn remaining(&self) -> usize {
        N - self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

#[derive(Debug)]
pub struct BufferFull;

impl std::fmt::Display for BufferFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buffer is full")
    }
}

/// Small string that lives on the stack for short strings.
/// Similar to the smallvec/string pattern.
pub struct SmallString<const N: usize> {
    inline: [u8; N],
    len: usize,
    heap: Option<String>,
}

impl<const N: usize> SmallString<N> {
    pub fn new(s: &str) -> Self {
        if s.len() <= N {
            let mut inline = [0u8; N];
            inline[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                inline,
                len: s.len(),
                heap: None,
            }
        } else {
            Self {
                inline: [0u8; N],
                len: s.len(),
                heap: Some(s.to_string()),
            }
        }
    }

    pub fn as_str(&self) -> &str {
        if let Some(ref heap) = self.heap {
            heap.as_str()
        } else {
            // Safety: we only store valid UTF-8
            unsafe { std::str::from_utf8_unchecked(&self.inline[..self.len]) }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_on_heap(&self) -> bool {
        self.heap.is_some()
    }

    pub fn is_inline(&self) -> bool {
        self.heap.is_none()
    }
}

/// Measure approximate stack usage by recursion depth.
pub fn stack_depth() -> usize {
    let mut depth = 0usize;
    measure_stack_recursion(&mut depth);
    depth
}

fn measure_stack_recursion(depth: &mut usize) {
    let marker = 0u8;
    let addr = &marker as *const u8 as usize;
    if *depth == 0 {
        *depth = addr;
    }
    // Approximate: each frame uses some stack
    // In practice, use tools like `-Z emit-stack-sizes`
}

/// Box allocation pattern: when to use Box.
pub enum RecursiveType {
    Leaf(i32),
    Node {
        value: i32,
        left: Box<RecursiveType>,
        right: Box<RecursiveType>,
    },
}

impl RecursiveType {
    pub fn leaf(value: i32) -> Self {
        RecursiveType::Leaf(value)
    }

    pub fn node(value: i32, left: RecursiveType, right: RecursiveType) -> Self {
        RecursiveType::Node {
            value,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn sum(&self) -> i32 {
        match self {
            RecursiveType::Leaf(v) => *v,
            RecursiveType::Node { value, left, right } => value + left.sum() + right.sum(),
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            RecursiveType::Leaf(_) => 1,
            RecursiveType::Node { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }
}

/// Pattern: Avoid heap allocation in hot paths.
pub struct HotPathOptimized {
    // Stack-allocated buffer for common case
    inline_buf: [u8; 256],
    inline_len: usize,
    // Heap-allocated for overflow
    overflow: Option<Vec<u8>>,
}

impl HotPathOptimized {
    pub fn new() -> Self {
        Self {
            inline_buf: [0u8; 256],
            inline_len: 0,
            overflow: None,
        }
    }

    pub fn push(&mut self, byte: u8) {
        if self.inline_len < 256 && self.overflow.is_none() {
            self.inline_buf[self.inline_len] = byte;
            self.inline_len += 1;
        } else {
            let overflow = self.overflow.get_or_insert_with(|| {
                let mut v = Vec::with_capacity(512);
                v.extend_from_slice(&self.inline_buf[..self.inline_len]);
                v
            });
            overflow.push(byte);
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        if let Some(ref overflow) = self.overflow {
            overflow.as_slice()
        } else {
            &self.inline_buf[..self.inline_len]
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref overflow) = self.overflow {
            overflow.len()
        } else {
            self.inline_len
        }
    }
}

/// Stack overflow prevention utilities.
pub struct StackGuard {
    max_depth: usize,
    current: std::cell::Cell<usize>,
}

impl StackGuard {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            current: std::cell::Cell::new(0),
        }
    }

    /// Execute a function with stack depth checking.
    pub fn execute<F, R>(&self, f: F) -> Result<R, StackOverflow>
    where
        F: FnOnce() -> R,
    {
        let current = self.current.get();
        if current >= self.max_depth {
            return Err(StackOverflow {
                current_depth: current,
                max_depth: self.max_depth,
            });
        }
        self.current.set(current + 1);
        let result = f();
        self.current.set(current);
        Ok(result)
    }

    pub fn current_depth(&self) -> usize {
        self.current.get()
    }
}

#[derive(Debug)]
pub struct StackOverflow {
    pub current_depth: usize,
    pub max_depth: usize,
}

impl std::fmt::Display for StackOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stack overflow: depth {} exceeds max {}",
            self.current_depth, self.max_depth
        )
    }
}

impl std::error::Error for StackOverflow {}

/// Heap fragmentation awareness utilities.
pub struct FragmentationTracker {
    allocations: Vec<(usize, usize)>, // (size, address)
}

impl FragmentationTracker {
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
        }
    }

    pub fn record_alloc(&mut self, size: usize, addr: usize) {
        self.allocations.push((size, addr));
    }

    /// Estimate fragmentation based on allocation pattern.
    pub fn estimate_fragmentation(&self) -> f64 {
        if self.allocations.len() < 2 {
            return 0.0;
        }

        let mut sorted = self.allocations.clone();
        sorted.sort_by_key(|a| a.1); // Sort by address

        let mut gaps = Vec::new();
        for window in sorted.windows(2) {
            let (size_a, addr_a) = window[0];
            let (size_b, addr_b) = window[1];
            let gap = addr_b.saturating_sub(addr_a + size_a);
            gaps.push(gap);
        }

        if gaps.is_empty() {
            return 0.0;
        }

        let total_gap: usize = gaps.iter().sum();
        let total_allocated: usize = self.allocations.iter().map(|a| a.0).sum();

        if total_allocated == 0 {
            return 0.0;
        }

        total_gap as f64 / (total_allocated + total_gap) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_sizes() {
        assert_eq!(mem::size_of::<u8>(), 1);
        assert_eq!(mem::size_of::<u32>(), 4);
        assert_eq!(mem::size_of::<u64>(), 8);
        assert_eq!(mem::size_of::<f64>(), 8);
        assert_eq!(mem::size_of::<bool>(), 1);
    }

    #[test]
    fn test_box_size() {
        // Box is a pointer (usize)
        assert_eq!(mem::size_of::<Box<u8>>(), mem::size_of::<usize>());
        // Vec is 3 pointers (ptr, len, cap)
        assert_eq!(mem::size_of::<Vec<u8>>(), mem::size_of::<usize>() * 3);
    }

    #[test]
    fn test_stack_buffer() {
        let mut buf = StackBuffer::<16>::new();
        assert_eq!(buf.capacity(), 16);
        assert!(buf.is_empty());

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        assert_eq!(buf.as_slice(), &[1, 2, 3]);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.remaining(), 13);
    }

    #[test]
    fn test_stack_buffer_full() {
        let mut buf = StackBuffer::<2>::new();
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        assert!(buf.push(3).is_err());
    }

    #[test]
    fn test_stack_buffer_clear() {
        let mut buf = StackBuffer::<8>::new();
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_small_string_inline() {
        let s = SmallString::<16>::new("hello");
        assert_eq!(s.as_str(), "hello");
        assert!(s.is_inline());
        assert!(!s.is_on_heap());
    }

    #[test]
    fn test_small_string_heap() {
        let long = "this is a string that is definitely longer than 16 bytes";
        let s = SmallString::<16>::new(long);
        assert_eq!(s.as_str(), long);
        assert!(!s.is_inline());
        assert!(s.is_on_heap());
    }

    #[test]
    fn test_small_string_empty() {
        let s = SmallString::<8>::new("");
        assert_eq!(s.as_str(), "");
        assert_eq!(s.len(), 0);
        assert!(s.is_inline());
    }

    #[test]
    fn test_recursive_type() {
        let tree = RecursiveType::node(
            1,
            RecursiveType::leaf(2),
            RecursiveType::node(3, RecursiveType::leaf(4), RecursiveType::leaf(5)),
        );
        assert_eq!(tree.sum(), 15);
        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_recursive_type_leaf() {
        let leaf = RecursiveType::leaf(42);
        assert_eq!(leaf.sum(), 42);
        assert_eq!(leaf.depth(), 1);
    }

    #[test]
    fn test_hot_path_optimized() {
        let mut opt = HotPathOptimized::new();
        for i in 0..100 {
            opt.push(i);
        }
        assert_eq!(opt.len(), 100);
        assert_eq!(opt.as_slice()[0], 0);
        assert_eq!(opt.as_slice()[99], 99);
    }

    #[test]
    fn test_hot_path_overflow() {
        let mut opt = HotPathOptimized::new();
        // Fill inline buffer
        for i in 0..256 {
            opt.push(i as u8);
        }
        assert!(opt.overflow.is_none());

        // Trigger overflow
        opt.push(255);
        assert!(opt.overflow.is_some());
        assert_eq!(opt.len(), 257);
    }

    #[test]
    fn test_stack_guard() {
        let guard = StackGuard::new(5);
        let result = guard.execute(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_stack_guard_overflow() {
        let guard = StackGuard::new(2);
        let result = guard.execute(|| {
            guard.execute(|| {
                guard.execute(|| 42) // This should fail
            })
        });
        assert!(result.unwrap().unwrap().is_err());
    }

    #[test]
    fn test_fragmentation_tracker() {
        let mut tracker = FragmentationTracker::new();
        tracker.record_alloc(100, 0);
        tracker.record_alloc(200, 200); // Gap of 100
        tracker.record_alloc(150, 500); // Gap of 100

        let frag = tracker.estimate_fragmentation();
        assert!(frag > 0.0);
    }

    #[test]
    fn test_fragmentation_tracker_no_gaps() {
        let mut tracker = FragmentationTracker::new();
        tracker.record_alloc(100, 0);
        tracker.record_alloc(100, 100); // No gap

        let frag = tracker.estimate_fragmentation();
        assert!((frag - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fragmentation_tracker_empty() {
        let tracker = FragmentationTracker::new();
        assert!((tracker.estimate_fragmentation() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_option_size() {
        // Option<Box<T>> is the same size as Box<T> (niche optimization)
        assert_eq!(
            mem::size_of::<Option<Box<u8>>>(),
            mem::size_of::<Box<u8>>()
        );
    }

    #[test]
    fn test_enum_size() {
        // Enum size is the largest variant + discriminant
        enum Small {
            A(u8),
            B(u8),
        }
        assert_eq!(mem::size_of::<Small>(), 2); // 1 byte data + 1 byte discriminant
    }
}
