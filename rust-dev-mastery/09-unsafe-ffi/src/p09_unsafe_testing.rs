//! # Testing Unsafe Code
//!
//! Unsafe code requires more rigorous testing than safe code because the
//! compiler cannot catch safety violations. This lesson covers tools and
//! techniques for testing unsafe Rust:
//!
//! - **Miri**: Detects undefined behavior at runtime (aliasing violations,
//!   use-after-free, uninitialized reads, etc.)
//! - **AddressSanitizer (ASan)**: Detects memory errors (buffer overflows,
//!   use-after-free, etc.)
//! - **ThreadSanitizer (TSan)**: Detects data races
//! - **Loom**: Exhaustive concurrency testing
//! - **Property-based testing**: Generates random inputs to find edge cases

use std::alloc::{self, Layout};
use std::ptr;

/// A simple linked list for testing unsafe code patterns.
/// Each node owns the next node via a raw pointer.
pub struct List<T> {
    head: *mut Node<T>,
    len: usize,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn push_front(&mut self, value: T) {
        let node = Box::new(Node {
            data: value,
            next: self.head,
        });
        self.head = Box::into_raw(node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        // SAFETY: head is non-null (we checked). It was created from Box::into_raw
        // in push_front, so it's valid and properly aligned.
        unsafe {
            let node = Box::from_raw(self.head);
            self.head = node.next;
            self.len -= 1;
            Some(node.data)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over the list, yielding references to each element.
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter {
            current: self.head,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct ListIter<'a, T> {
    current: *mut Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        // SAFETY: current is non-null (we checked). It was created from
        // Box::into_raw, so it's valid. We return a shared reference,
        // which is safe because we don't provide mutable access.
        unsafe {
            let node = &*self.current;
            self.current = node.next;
            Some(&node.data)
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

/// A buffer with bounds-checked access that uses unsafe internally.
/// The tests verify that all bounds checks work correctly.
pub struct BoundedBuffer {
    ptr: *mut u8,
    capacity: usize,
}

impl BoundedBuffer {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 1).unwrap();
        // SAFETY: layout has non-zero size (capacity > 0 assumed).
        let ptr = unsafe { alloc::alloc(layout) };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        BoundedBuffer { ptr, capacity }
    }

    /// Write a byte at the given index.
    /// Returns false if out of bounds.
    pub fn write(&mut self, index: usize, value: u8) -> bool {
        if index >= self.capacity {
            return false;
        }
        // SAFETY: index < capacity, so the write is within bounds.
        unsafe {
            *self.ptr.add(index) = value;
        }
        true
    }

    /// Read a byte at the given index.
    /// Returns None if out of bounds.
    pub fn read(&self, index: usize) -> Option<u8> {
        if index >= self.capacity {
            return None;
        }
        // SAFETY: index < capacity, so the read is within bounds.
        Some(unsafe { *self.ptr.add(index) })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Fill the buffer with a value using ptr::write_bytes (memset).
    pub fn fill(&mut self, value: u8) {
        // SAFETY: self.ptr is valid for self.capacity bytes.
        unsafe {
            ptr::write_bytes(self.ptr, value, self.capacity);
        }
    }
}

impl Drop for BoundedBuffer {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 1).unwrap();
        // SAFETY: self.ptr was allocated with this layout.
        unsafe {
            alloc::dealloc(self.ptr, layout);
        }
    }
}

/// A concurrent counter that uses atomics.
/// Tests verify correctness under concurrent access.
pub struct AtomicCounter {
    value: std::sync::atomic::AtomicU64,
}

impl AtomicCounter {
    pub fn new(initial: u64) -> Self {
        AtomicCounter {
            value: std::sync::atomic::AtomicU64::new(initial),
        }
    }

    pub fn increment(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .wrapping_add(1)
    }

    pub fn decrement(&self) -> u64 {
        self.value
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst)
            .wrapping_sub(1)
    }

    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn compare_and_swap(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.value.compare_exchange(
            current,
            new,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
    }
}

/// Tests for the linked list using property-based-style testing.
/// These tests verify invariants that should hold for all inputs.
mod list_tests {
    use super::*;

    #[test]
    fn test_list_push_pop() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let values: Vec<&i32> = list.iter().collect();
        assert_eq!(values, vec![&3, &2, &1]);
    }

    #[test]
    fn test_list_empty_iter() {
        let list = List::<i32>::new();
        assert_eq!(list.iter().count(), 0);
    }

    #[test]
    fn test_list_push_pop_interleaved() {
        let mut list = List::new();
        list.push_front(1);
        assert_eq!(list.pop_front(), Some(1));
        assert!(list.is_empty());

        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.pop_front(), Some(3));

        list.push_front(4);
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(2));
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_many_elements() {
        let mut list = List::new();
        for i in 0..1000 {
            list.push_front(i);
        }
        assert_eq!(list.len(), 1000);

        for i in (0..1000).rev() {
            assert_eq!(list.pop_front(), Some(i));
        }
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_no_leak() {
        // Run many iterations to stress-test drop
        for _ in 0..100 {
            let mut list = List::new();
            for i in 0..100 {
                list.push_front(format!("item_{i}"));
            }
            // list drops here, should free all nodes
        }
    }
}

mod buffer_tests {
    use super::*;

    #[test]
    fn test_bounded_buffer_write_read() {
        let mut buf = BoundedBuffer::new(100);
        assert!(buf.write(0, 42));
        assert!(buf.write(99, 99));
        assert_eq!(buf.read(0), Some(42));
        assert_eq!(buf.read(99), Some(99));
    }

    #[test]
    fn test_bounded_buffer_out_of_bounds() {
        let mut buf = BoundedBuffer::new(10);
        assert!(!buf.write(10, 0));
        assert!(!buf.write(100, 0));
        assert_eq!(buf.read(10), None);
        assert_eq!(buf.read(100), None);
    }

    #[test]
    fn test_bounded_buffer_fill() {
        let mut buf = BoundedBuffer::new(256);
        buf.fill(0xAB);
        for i in 0..256 {
            assert_eq!(buf.read(i), Some(0xAB));
        }
    }

    #[test]
    fn test_bounded_buffer_overwrite() {
        let mut buf = BoundedBuffer::new(10);
        buf.write(5, 1);
        buf.write(5, 2);
        buf.write(5, 3);
        assert_eq!(buf.read(5), Some(3));
    }

    #[test]
    fn test_bounded_buffer_all_positions() {
        let size = 64;
        let mut buf = BoundedBuffer::new(size);
        for i in 0..size {
            assert!(buf.write(i, i as u8));
        }
        for i in 0..size {
            assert_eq!(buf.read(i), Some(i as u8));
        }
    }
}

mod counter_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_counter_single_thread() {
        let counter = AtomicCounter::new(0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_counter_cas() {
        let counter = AtomicCounter::new(10);
        assert_eq!(counter.compare_and_swap(10, 20), Ok(10));
        assert_eq!(counter.get(), 20);
        assert_eq!(counter.compare_and_swap(10, 30), Err(20));
        assert_eq!(counter.get(), 20);
    }

    #[test]
    fn test_counter_concurrent_increments() {
        let counter = Arc::new(AtomicCounter::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    c.increment();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(counter.get(), 10000);
    }

    #[test]
    fn test_counter_concurrent_increment_decrement() {
        let counter = Arc::new(AtomicCounter::new(0));
        let mut handles = vec![];

        // 5 threads incrementing
        for _ in 0..5 {
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    c.increment();
                }
            }));
        }

        // 5 threads decrementing
        for _ in 0..5 {
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    c.decrement();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
        // Net should be 0 (5 * 1000 increments - 5 * 1000 decrements)
        assert_eq!(counter.get(), 0);
    }
}

/// Demonstrates testing patterns for unsafe code.
/// These tests specifically target common sources of UB.
mod ub_prevention_tests {
    use super::*;

    #[test]
    fn test_no_uninitialized_reads() {
        // Using MaybeUninit correctly
        let mut uninit = std::mem::MaybeUninit::<i32>::uninit();
        uninit.write(42);
        // SAFETY: We wrote 42 into the MaybeUninit.
        let value = unsafe { uninit.assume_init() };
        assert_eq!(value, 42);
    }

    #[test]
    fn test_no_buffer_overflow() {
        let mut buf = BoundedBuffer::new(10);
        // Write exactly to capacity
        for i in 0..10 {
            assert!(buf.write(i, i as u8));
        }
        // Writing beyond capacity should fail (not overflow)
        assert!(!buf.write(10, 0));
        assert!(!buf.write(usize::MAX, 0));
    }

    #[test]
    fn test_no_double_free() {
        // List::drop should handle this correctly
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        drop(list);
        // If there's a double-free, Miri or ASan would catch it
    }

    #[test]
    fn test_no_use_after_free() {
        let mut list = List::new();
        list.push_front(42);
        let val = list.pop_front().unwrap();
        assert_eq!(val, 42);
        // The node is freed after pop_front, but we have the value
        assert!(list.is_empty());
    }

    #[test]
    fn test_alignment_preserved() {
        // Ensure allocations respect alignment requirements
        let buf = BoundedBuffer::new(100);
        assert_eq!(buf.ptr as usize % 1, 0); // 1-byte aligned for u8

        let layout = Layout::from_size_align(100, 16).unwrap();
        // SAFETY: layout is valid.
        let ptr = unsafe { alloc::alloc(layout) };
        assert!(!ptr.is_null());
        assert_eq!(ptr as usize % 16, 0); // 16-byte aligned
        // SAFETY: ptr was allocated with this layout.
        unsafe {
            alloc::dealloc(ptr, layout);
        }
    }
}

/// Demonstrates testing with Miri annotations.
/// To run with Miri: `cargo +nightly miri test`
///
/// Miri checks:
/// - Uninitialized memory reads
/// - Use-after-free
/// - Double-free
/// - Buffer overflows
/// - Invalid pointer arithmetic
/// - Violation of aliasing rules (Stacked Borrows / Tree Borrows)
///
/// Example commands:
/// ```sh
/// cargo +nightly miri test
/// MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo +nightly miri test
/// MIRIFLAGS="-Zmiri-tag-raw-pointers" cargo +nightly miri test
/// ```
mod miri_tests {
    use super::*;

    #[test]
    fn test_pointer_arithmetic_bounds() {
        let data = [1u8, 2, 3, 4, 5];
        let ptr = data.as_ptr();

        // Valid: access within bounds
        for i in 0..5 {
            // SAFETY: i is in 0..5, within the array bounds.
            unsafe {
                assert_eq!(*ptr.add(i), (i + 1) as u8);
            }
        }
    }

    #[test]
    fn test_raw_pointer_aliasing() {
        let mut value = 42i32;
        let ptr1 = &mut value as *mut i32;
        let ptr2 = &value as *const i32;

        // SAFETY: We only read through ptr2, never write through it
        // while ptr1 exists. This respects the aliasing rules.
        unsafe {
            assert_eq!(*ptr2, 42);
            *ptr1 = 100;
            // After writing through ptr1, we should not read through ptr2
            // (which was derived from a shared reference). We read through ptr1 instead.
            assert_eq!(*ptr1, 100);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_string_elements() {
        let mut list = List::new();
        list.push_front("hello".to_string());
        list.push_front("world".to_string());

        let values: Vec<&String> = list.iter().collect();
        assert_eq!(values, vec![&"world".to_string(), &"hello".to_string()]);
    }

    #[test]
    fn test_buffer_stress() {
        let mut buf = BoundedBuffer::new(4096);
        for round in 0..100 {
            buf.fill(round as u8);
            for i in 0..4096 {
                assert_eq!(buf.read(i), Some(round as u8));
            }
        }
    }

    #[test]
    fn test_counter_initial_value() {
        let c = AtomicCounter::new(100);
        assert_eq!(c.get(), 100);
    }

    #[test]
    fn test_layout_properties() {
        // Verify layout computation is correct
        let layout = Layout::from_size_align(100, 16).unwrap();
        assert_eq!(layout.size(), 100);
        assert_eq!(layout.align(), 16);

        // Invalid layouts should fail
        assert!(Layout::from_size_align(100, 3).is_err()); // 3 is not power of 2
    }
}
