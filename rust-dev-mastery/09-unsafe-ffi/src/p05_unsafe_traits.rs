//! # Unsafe Traits: Send, Sync, and Marker Traits
//!
//! An `unsafe trait` is a trait where the implementor must uphold certain
//! invariants that the compiler cannot automatically verify. Implementing
//! an unsafe trait requires an `unsafe impl` block.
//!
//! The most common unsafe traits are:
//! - `Send`: Type can be transferred across thread boundaries
//! - `Sync`: Type can be shared between threads via `&T`
//! - `GlobalAlloc`: Custom global allocator
//!
//! Marker traits like `Send` and `Sync` are automatically derived for types
//! where all fields implement the trait. You only need `unsafe impl` when
//! your type contains something that doesn't automatically implement it
//! (like a raw pointer) but you can guarantee the safety invariant.

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// A thread-safe, lock-free, single-producer single-consumer (SPSC) queue.
/// This demonstrates `unsafe impl Send` and `unsafe impl Sync`.
///
/// The queue uses a ring buffer with atomic head/tail pointers.
/// - The producer advances `write_pos`
/// - The consumer advances `read_pos`
/// - Both use atomic operations for synchronization
pub struct SpscQueue<T> {
    buffer: *mut T,
    capacity: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    /// PhantomData to indicate we semantically own T values
    _marker: PhantomData<T>,
}

// SAFETY: SpscQueue<T> is Send if T is Send. The raw pointer to the buffer
// is owned by SpscQueue, and all access is through atomic operations.
// The buffer is allocated on one thread and deallocated on the same thread (Drop).
unsafe impl<T: Send> Send for SpscQueue<T> {}

// SAFETY: SpscQueue<T> is Sync if T is Send. The producer (push) and consumer (pop)
// use different atomic indices, so there's no data race. The T values are moved
// (not shared) between threads, so T doesn't need to be Sync, only Send.
unsafe impl<T: Send> Sync for SpscQueue<T> {}

impl<T> SpscQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        let layout = std::alloc::Layout::array::<T>(capacity).unwrap();
        // SAFETY: layout is non-zero. We handle allocation failure.
        let buffer = unsafe { std::alloc::alloc(layout) as *mut T };
        if buffer.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        SpscQueue {
            buffer,
            capacity,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Push a value into the queue. Returns Err(value) if the queue is full.
    /// This is safe to call from one thread while another thread calls `pop`.
    pub fn push(&self, value: T) -> Result<(), T> {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Acquire);
        let next_write = (write + 1) % self.capacity;

        if next_write == read {
            return Err(value); // Queue full
        }

        // SAFETY: write position is within bounds and not the same as read_pos
        // (we checked above). We have exclusive write access to this slot
        // because only the producer advances write_pos.
        unsafe {
            ptr::write(self.buffer.add(write), value);
        }

        self.write_pos.store(next_write, Ordering::Release);
        Ok(())
    }

    /// Pop a value from the queue. Returns None if the queue is empty.
    /// This is safe to call from one thread while another thread calls `push`.
    pub fn pop(&self) -> Option<T> {
        let read = self.read_pos.load(Ordering::Relaxed);
        let write = self.write_pos.load(Ordering::Acquire);

        if read == write {
            return None; // Queue empty
        }

        // SAFETY: read position is within bounds and not the same as write_pos
        // (we checked above). We have exclusive read access to this slot
        // because only the consumer advances read_pos.
        let value = unsafe { ptr::read(self.buffer.add(read)) };

        let next_read = (read + 1) % self.capacity;
        self.read_pos.store(next_read, Ordering::Release);
        Some(value)
    }

    pub fn len(&self) -> usize {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Relaxed);
        if write >= read {
            write - read
        } else {
            self.capacity - read + write
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for SpscQueue<T> {
    fn drop(&mut self) {
        // Drain any remaining elements
        while self.pop().is_some() {}

        let layout = std::alloc::Layout::array::<T>(self.capacity).unwrap();
        // SAFETY: buffer was allocated with this layout in `new`.
        unsafe {
            std::alloc::dealloc(self.buffer as *mut u8, layout);
        }
    }
}

/// Demonstrates a custom unsafe trait for a type with specific invariants.
///
/// # Safety Contract
/// Implementors must guarantee that `as_raw_ptr()` returns a valid, aligned
/// pointer to initialized memory of the correct type.
pub unsafe trait RawPointer {
    type Target;

    /// Returns a raw pointer to the contained value.
    ///
    /// # Safety Contract
    /// The returned pointer must:
    /// - Be non-null
    /// - Be properly aligned for `Self::Target`
    /// - Point to a valid, initialized `Self::Target`
    /// - Remain valid for at least the lifetime of `&self`
    fn as_raw_ptr(&self) -> *const Self::Target;
}

/// A fixed-capacity vector that stores elements inline (no heap).
/// Uses unsafe internally to manage uninitialized memory.
pub struct FixedVec<T, const N: usize> {
    data: [std::mem::MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub fn new() -> Self {
        FixedVec {
            // SAFETY: MaybeUninit does not require initialization.
            data: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        // SAFETY: len < N, so we're within bounds. MaybeUninit allows writing
        // without reading the previous (uninitialized) value.
        self.data[self.len] = std::mem::MaybeUninit::new(value);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: len was > 0, so self.len is now a valid initialized index.
        // We read the value and mark it as uninitialized.
        Some(unsafe { self.data[self.len].assume_init_read() })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index < len, so this slot was initialized by push.
            Some(unsafe { self.data[index].assume_init_ref() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Elements 0..len are all initialized (guaranteed by push).
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    fn drop(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            // SAFETY: All elements 0..len are initialized.
            unsafe {
                self.data[i].assume_init_drop();
            }
        }
    }
}

// SAFETY: FixedVec is Send if T is Send, because all data is inline (no sharing).
unsafe impl<T: Send, const N: usize> Send for FixedVec<T, N> {}
// SAFETY: FixedVec is Sync if T is Sync, because &FixedVec only provides
// shared access to its elements through &T.
unsafe impl<T: Sync, const N: usize> Sync for FixedVec<T, N> {}

/// Demonstrates PhantomData for controlling auto-traits.
/// PhantomData<T> tells the compiler "this type logically owns a T"
/// without actually storing one.
pub struct MyIterator<'a, T> {
    ptr: *const T,
    end: *const T,
    /// PhantomData<&'a T> makes this type:
    /// - NOT Send (because &'a T might not be Send)
    /// - NOT Sync (because &'a T might not be Sync)
    /// - NOT 'static (because it borrows T for 'a)
    _marker: PhantomData<&'a T>,
}

impl<'a, T> MyIterator<'a, T> {
    /// Create an iterator over a slice.
    pub fn new(slice: &'a [T]) -> Self {
        let ptr = slice.as_ptr();
        // SAFETY: ptr.add(len) is a valid pointer for one-past-the-end.
        let end = unsafe { ptr.add(slice.len()) };
        MyIterator {
            ptr,
            end,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for MyIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            return None;
        }
        // SAFETY: ptr is within the slice bounds (between start and end).
        // We advance ptr so each element is yielded exactly once.
        let value = unsafe { &*self.ptr };
        self.ptr = unsafe { self.ptr.add(1) };
        Some(value)
    }
}

/// A wrapper type that opts out of Send and Sync.
/// This is useful for types that are inherently thread-unsafe,
/// like types containing Cell or RefCell.
pub struct ThreadUnsafe {
    value: UnsafeCell<i32>,
    /// PhantomData<*mut ()> opts out of both Send and Sync.
    /// *mut () is neither Send nor Sync, so neither is ThreadUnsafe.
    _not_send_sync: PhantomData<*mut ()>,
}

impl ThreadUnsafe {
    pub fn new(value: i32) -> Self {
        ThreadUnsafe {
            value: UnsafeCell::new(value),
            _not_send_sync: PhantomData,
        }
    }

    pub fn get(&self) -> i32 {
        // SAFETY: We only create one ThreadUnsafe per value and don't share
        // across threads (not Send or Sync).
        unsafe { *self.value.get() }
    }

    pub fn set(&self, value: i32) {
        // SAFETY: Same as get - single-threaded access guaranteed.
        unsafe { *self.value.get() = value };
    }
}

/// Demonstrates a custom allocator using the GlobalAlloc trait.
/// This is a simple bump allocator for demonstration purposes.
pub struct BumpAllocator {
    arena: AtomicPtr<u8>,
    capacity: usize,
    offset: AtomicUsize,
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(capacity, 8).unwrap();
        // SAFETY: layout is non-zero.
        let arena = unsafe { std::alloc::alloc(layout) };
        if arena.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        BumpAllocator {
            arena: AtomicPtr::new(arena),
            capacity,
            offset: AtomicUsize::new(0),
        }
    }

    /// Allocate `size` bytes with `align` alignment.
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        loop {
            let current = self.offset.load(Ordering::Relaxed);
            let aligned = (current + align - 1) & !(align - 1);
            if aligned + size > self.capacity {
                return ptr::null_mut();
            }
            match self.offset.compare_exchange_weak(
                current,
                aligned + size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // SAFETY: aligned + size <= self.capacity, so this is within bounds.
                    return unsafe { self.arena.load(Ordering::Relaxed).add(aligned) };
                }
                Err(_) => continue, // Retry
            }
        }
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let arena = *self.arena.get_mut();
        if !arena.is_null() {
            let layout = std::alloc::Layout::from_size_align(self.capacity, 8).unwrap();
            // SAFETY: arena was allocated with this layout in `new`.
            unsafe {
                std::alloc::dealloc(arena, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_spsc_queue_basic() {
        let queue = SpscQueue::<i32>::new(16);
        assert!(queue.is_empty());

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_spsc_queue_full() {
        let queue = SpscQueue::<i32>::new(4); // capacity 4, usable 3
        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();
        assert!(queue.push(4).is_err()); // Full
    }

    #[test]
    fn test_spsc_queue_threaded() {
        let queue = std::sync::Arc::new(SpscQueue::<i32>::new(1024));
        let q2 = queue.clone();

        let producer = thread::spawn(move || {
            for i in 0..100 {
                while queue.push(i).is_err() {
                    thread::yield_now();
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut received = Vec::new();
            while received.len() < 100 {
                if let Some(val) = q2.pop() {
                    received.push(val);
                } else {
                    thread::yield_now();
                }
            }
            received
        });

        producer.join().unwrap();
        let received = consumer.join().unwrap();
        assert_eq!(received, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_fixed_vec_push_pop() {
        let mut fv = FixedVec::<i32, 8>::new();
        assert!(fv.is_empty());

        for i in 0..8 {
            fv.push(i).unwrap();
        }
        assert_eq!(fv.len(), 8);
        assert!(fv.push(8).is_err()); // Full

        for i in (0..8).rev() {
            assert_eq!(fv.pop(), Some(i));
        }
        assert_eq!(fv.pop(), None);
    }

    #[test]
    fn test_fixed_vec_get() {
        let mut fv = FixedVec::<&str, 4>::new();
        fv.push("hello").unwrap();
        fv.push("world").unwrap();

        assert_eq!(fv.get(0), Some(&"hello"));
        assert_eq!(fv.get(1), Some(&"world"));
        assert_eq!(fv.get(2), None);
    }

    #[test]
    fn test_fixed_vec_as_slice() {
        let mut fv = FixedVec::<i32, 4>::new();
        fv.push(10).unwrap();
        fv.push(20).unwrap();
        fv.push(30).unwrap();

        assert_eq!(fv.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn test_my_iterator() {
        let data = vec![1, 2, 3, 4, 5];
        let iter = MyIterator::new(&data);
        let collected: Vec<&i32> = iter.collect();
        assert_eq!(collected, vec![&1, &2, &3, &4, &5]);
    }

    #[test]
    fn test_my_iterator_empty() {
        let data: Vec<i32> = vec![];
        let mut iter = MyIterator::new(&data);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_thread_unsafe_single_threaded() {
        let tu = ThreadUnsafe::new(42);
        assert_eq!(tu.get(), 42);
        tu.set(100);
        assert_eq!(tu.get(), 100);
    }

    #[test]
    fn test_bump_allocator() {
        let alloc = BumpAllocator::new(1024);
        let p1 = alloc.alloc(64, 8);
        assert!(!p1.is_null());
        assert_eq!(p1 as usize % 8, 0); // Aligned to 8

        let p2 = alloc.alloc(32, 16);
        assert!(!p2.is_null());
        assert_eq!(p2 as usize % 16, 0); // Aligned to 16

        // Different pointers
        assert_ne!(p1 as usize, p2 as usize);
    }

    #[test]
    fn test_bump_allocator_exhaustion() {
        let alloc = BumpAllocator::new(64);
        let _p1 = alloc.alloc(32, 8);
        let _p2 = alloc.alloc(32, 8);
        let p3 = alloc.alloc(1, 8);
        assert!(p3.is_null()); // Should fail - out of space
    }

    #[test]
    fn test_send_sync_bounds() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SpscQueue<i32>>();
        assert_sync::<SpscQueue<i32>>();

        assert_send::<FixedVec<i32, 8>>();
        assert_sync::<FixedVec<i32, 8>>();
    }
}
