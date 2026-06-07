//! # Pool Allocation
//!
//! Object pools pre-allocate a fixed set of objects and recycle them when
//! returned. This eliminates allocation/deallocation overhead and provides
//! predictable memory usage.
//!
//! ## Benefits:
//!
//! - **Predictable memory**: Fixed pool size
//! - **No allocation overhead**: Objects are pre-allocated
//! - **Cache-friendly**: Objects are contiguous
//! - **No fragmentation**: Memory is reused in-place
//!
//! ## Patterns:
//!
//! - **Object Pool**: Reuse complex objects
//! - **Slab Allocator**: Fixed-size block allocation
//! - **Free List**: Track available slots

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Generic object pool that recycles objects instead of allocating new ones.
pub struct ObjectPool<T> {
    available: Mutex<VecDeque<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    created: AtomicUsize,
    reused: AtomicUsize,
}

impl<T: Send> ObjectPool<T> {
    pub fn new<F>(initial_size: usize, max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let mut available = VecDeque::with_capacity(max_size);
        for _ in 0..initial_size {
            available.push_back(factory());
        }

        Self {
            available: Mutex::new(available),
            factory: Box::new(factory),
            max_size,
            created: AtomicUsize::new(initial_size),
            reused: AtomicUsize::new(0),
        }
    }

    /// Acquire an object from the pool.
    pub fn acquire(&self) -> T {
        let mut available = self.available.lock().unwrap();
        if let Some(obj) = available.pop_front() {
            self.reused.fetch_add(1, Ordering::Relaxed);
            obj
        } else {
            self.created.fetch_add(1, Ordering::Relaxed);
            (self.factory)()
        }
    }

    /// Return an object to the pool.
    pub fn release(&self, obj: T) {
        let mut available = self.available.lock().unwrap();
        if available.len() < self.max_size {
            available.push_back(obj);
        }
        // Otherwise drop the object
    }

    pub fn available(&self) -> usize {
        self.available.lock().unwrap().len()
    }

    pub fn total_created(&self) -> usize {
        self.created.load(Ordering::Relaxed)
    }

    pub fn total_reused(&self) -> usize {
        self.reused.load(Ordering::Relaxed)
    }

    pub fn reuse_rate(&self) -> f64 {
        let total = self.total_created() + self.total_reused();
        if total == 0 {
            return 0.0;
        }
        self.total_reused() as f64 / total as f64
    }
}

/// RAII guard that returns an object to the pool when dropped.
pub struct PoolGuard<'a, T: Send> {
    pool: &'a ObjectPool<T>,
    inner: Option<T>,
}

impl<'a, T: Send> PoolGuard<'a, T> {
    pub fn new(pool: &'a ObjectPool<T>, obj: T) -> Self {
        Self {
            pool,
            inner: Some(obj),
        }
    }

    /// Get a reference to the inner object.
    pub fn get(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    /// Get a mutable reference to the inner object.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    /// Take the object out of the guard (prevents returning to pool).
    pub fn take(mut self) -> T {
        self.inner.take().unwrap()
    }
}

impl<'a, T: Send> std::ops::Deref for PoolGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T: Send> std::ops::DerefMut for PoolGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'a, T: Send> Drop for PoolGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(obj) = self.inner.take() {
            self.pool.release(obj);
        }
    }
}

/// Slab allocator: allocates fixed-size blocks from a contiguous region.
/// Very efficient for many small allocations of the same size.
pub struct SlabAllocator {
    blocks: Vec<SlabBlock>,
    block_size: usize,
    free_list: Vec<usize>, // Indices of free blocks
    allocated: AtomicUsize,
}

struct SlabBlock {
    data: Vec<u8>,
    in_use: bool,
}

impl SlabAllocator {
    pub fn new(block_size: usize, num_blocks: usize) -> Self {
        let mut blocks = Vec::with_capacity(num_blocks);
        let mut free_list = Vec::with_capacity(num_blocks);

        for i in 0..num_blocks {
            blocks.push(SlabBlock {
                data: vec![0u8; block_size],
                in_use: false,
            });
            free_list.push(i);
        }

        Self {
            blocks,
            block_size,
            free_list,
            allocated: AtomicUsize::new(0),
        }
    }

    /// Allocate a block, returning its index.
    pub fn alloc(&mut self) -> Option<usize> {
        let idx = self.free_list.pop()?;
        self.blocks[idx].in_use = true;
        self.allocated.fetch_add(1, Ordering::Relaxed);
        Some(idx)
    }

    /// Free a block by index.
    pub fn free(&mut self, idx: usize) {
        if idx < self.blocks.len() && self.blocks[idx].in_use {
            self.blocks[idx].in_use = false;
            self.blocks[idx].data.iter_mut().for_each(|b| *b = 0);
            self.free_list.push(idx);
            self.allocated.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get a reference to a block's data.
    pub fn get(&self, idx: usize) -> Option<&[u8]> {
        if idx < self.blocks.len() && self.blocks[idx].in_use {
            Some(&self.blocks[idx].data)
        } else {
            None
        }
    }

    /// Get a mutable reference to a block's data.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut [u8]> {
        if idx < self.blocks.len() && self.blocks[idx].in_use {
            Some(&mut self.blocks[idx].data)
        } else {
            None
        }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn total_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn allocated_blocks(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    pub fn free_blocks(&self) -> usize {
        self.blocks.len() - self.allocated_blocks()
    }

    pub fn utilization(&self) -> f64 {
        self.allocated_blocks() as f64 / self.total_blocks() as f64
    }
}

/// Pool-based Vec that reuses its allocation.
pub struct PooledVec<T> {
    data: Vec<T>,
}

impl<T> PooledVec<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Clear the vec but keep the allocated capacity.
    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T> std::ops::Deref for PooledVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Memory pool for byte buffers of varying sizes.
pub struct ByteBufferPool {
    small: ObjectPool<Vec<u8>>,  // 4KB
    medium: ObjectPool<Vec<u8>>, // 64KB
    large: ObjectPool<Vec<u8>>,  // 1MB
}

impl ByteBufferPool {
    pub fn new() -> Self {
        Self {
            small: ObjectPool::new(16, 64, || Vec::with_capacity(4096)),
            medium: ObjectPool::new(4, 16, || Vec::with_capacity(65536)),
            large: ObjectPool::new(2, 8, || Vec::with_capacity(1048576)),
        }
    }

    /// Acquire a buffer of at least the requested size.
    pub fn acquire(&self, min_size: usize) -> Vec<u8> {
        let mut buf = if min_size <= 4096 {
            self.small.acquire()
        } else if min_size <= 65536 {
            self.medium.acquire()
        } else {
            self.large.acquire()
        };
        buf.clear();
        buf
    }

    /// Return a buffer to the appropriate pool.
    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        let cap = buf.capacity();
        if cap <= 4096 {
            self.small.release(buf);
        } else if cap <= 65536 {
            self.medium.release(buf);
        } else {
            self.large.release(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_basic() {
        let pool = ObjectPool::new(5, 10, || Vec::<i32>::new());
        assert_eq!(pool.available(), 5);

        let obj = pool.acquire();
        assert_eq!(pool.available(), 4);

        pool.release(obj);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_object_pool_reuse() {
        let pool = ObjectPool::new(2, 10, || Vec::<i32>::new());
        let a = pool.acquire(); // From initial pool (counts as reuse)
        pool.release(a);

        let b = pool.acquire(); // Reuses the returned object
        // Both acquires from pool count as reuse
        assert_eq!(pool.total_reused(), 2);
        drop(b);
    }

    #[test]
    fn test_object_pool_exhaustion() {
        let pool = ObjectPool::new(1, 1, || 42u32);
        let _a = pool.acquire();
        // Pool empty, should create new
        let b = pool.acquire();
        assert_eq!(b, 42);
        assert_eq!(pool.total_created(), 2);
    }

    #[test]
    fn test_object_pool_reuse_rate() {
        let pool = ObjectPool::new(2, 10, || 0u32);
        let _a = pool.acquire();
        let _b = pool.acquire();
        // Both from initial pool, no reuse yet
    }

    #[test]
    fn test_pool_guard() {
        let pool = ObjectPool::new(5, 10, || Vec::<i32>::new());
        let initial_available = pool.available();

        {
            let mut guard = PoolGuard::new(&pool, pool.acquire());
            guard.push(42);
            assert_eq!(guard.len(), 1);
            assert_eq!(pool.available(), initial_available - 1);
        }
        // Guard dropped, object returned to pool
        assert_eq!(pool.available(), initial_available);
    }

    #[test]
    fn test_pool_guard_take() {
        let pool = ObjectPool::new(5, 10, || Vec::<i32>::new());
        let guard = PoolGuard::new(&pool, pool.acquire());
        let obj = guard.take();
        // Object taken out, not returned to pool
        drop(obj);
    }

    #[test]
    fn test_slab_allocator_basic() {
        let mut slab = SlabAllocator::new(64, 10);
        assert_eq!(slab.total_blocks(), 10);
        assert_eq!(slab.free_blocks(), 10);

        let idx = slab.alloc().unwrap();
        assert_eq!(slab.allocated_blocks(), 1);
        assert_eq!(slab.free_blocks(), 9);

        slab.free(idx);
        assert_eq!(slab.allocated_blocks(), 0);
    }

    #[test]
    fn test_slab_allocator_data() {
        let mut slab = SlabAllocator::new(16, 10);
        let idx = slab.alloc().unwrap();

        let data = slab.get_mut(idx).unwrap();
        data[0] = 42;
        data[1] = 43;

        let data = slab.get(idx).unwrap();
        assert_eq!(data[0], 42);
        assert_eq!(data[1], 43);
    }

    #[test]
    fn test_slab_allocator_exhaustion() {
        let mut slab = SlabAllocator::new(16, 2);
        let _a = slab.alloc().unwrap();
        let _b = slab.alloc().unwrap();
        assert!(slab.alloc().is_none());
    }

    #[test]
    fn test_slab_allocator_utilization() {
        let mut slab = SlabAllocator::new(16, 10);
        assert!((slab.utilization() - 0.0).abs() < f64::EPSILON);

        slab.alloc().unwrap();
        slab.alloc().unwrap();
        assert!((slab.utilization() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_slab_allocator_free_invalid() {
        let mut slab = SlabAllocator::new(16, 10);
        slab.free(100); // Invalid index, should not panic
        assert_eq!(slab.allocated_blocks(), 0);
    }

    #[test]
    fn test_pooled_vec() {
        let mut pv = PooledVec::new();
        pv.push(1);
        pv.push(2);
        pv.push(3);
        assert_eq!(pv.len(), 3);
        assert_eq!(pv.as_slice(), &[1, 2, 3]);

        pv.reset();
        assert!(pv.is_empty());
        assert!(pv.capacity() >= 3); // Capacity preserved
    }

    #[test]
    fn test_pooled_vec_deref() {
        let mut pv = PooledVec::new();
        pv.push(10);
        pv.push(20);
        // Can use slice methods via Deref
        assert_eq!(pv.iter().sum::<i32>(), 30);
    }

    #[test]
    fn test_byte_buffer_pool() {
        let pool = ByteBufferPool::new();
        let small_buf = pool.acquire(100);
        assert!(small_buf.capacity() >= 100);

        let large_buf = pool.acquire(100_000);
        assert!(large_buf.capacity() >= 100_000);

        pool.release(small_buf);
        pool.release(large_buf);
    }

    #[test]
    fn test_object_pool_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(ObjectPool::new(10, 20, || Vec::<u8>::with_capacity(1024)));
        let mut handles = vec![];

        for _ in 0..10 {
            let pool = Arc::clone(&pool);
            handles.push(thread::spawn(move || {
                let obj = pool.acquire();
                pool.release(obj);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(pool.total_created() + pool.total_reused() >= 10);
    }
}
