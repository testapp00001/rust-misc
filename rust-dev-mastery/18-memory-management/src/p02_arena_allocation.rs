//! # Arena Allocation
//!
//! Arena (bump) allocation is a fast allocation strategy where memory is
//! allocated by simply advancing a pointer. All allocations in an arena are
//! freed at once when the arena is dropped, making it ideal for temporary
//! allocations in a processing phase.
//!
//! ## Benefits:
//!
//! - **Extremely fast**: Just a pointer increment (no free list management)
//! - **Cache-friendly**: Allocations are contiguous in memory
//! - **No fragmentation**: Memory is always used sequentially
//! - **Batch deallocation**: Everything freed at once
//!
//! ## Use Cases:
//!
//! - Parser temporary data
//! - Request-scoped allocations
//! - Compilation phases
//! - Game frame allocations

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A bump allocator arena.
/// Allocates memory by advancing a pointer into a pre-allocated buffer.
/// All memory is freed when the arena is dropped.
pub struct Arena {
    buffer: UnsafeCell<Vec<u8>>,
    offset: UnsafeCell<usize>,
    capacity: usize,
}

impl Arena {
    /// Create a new arena with the given capacity in bytes.
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        // Safety: We're setting the length to capacity since we'll manage it ourselves
        unsafe {
            buffer.set_len(capacity);
            // Zero-initialize
            std::ptr::write_bytes(buffer.as_mut_ptr(), 0, capacity);
        }

        Self {
            buffer: UnsafeCell::new(buffer),
            offset: UnsafeCell::new(0),
            capacity,
        }
    }

    /// Allocate memory with the given layout.
    /// Returns None if the arena is full.
    pub fn alloc_layout(&self, layout: std::alloc::Layout) -> Option<NonNull<u8>> {
        let offset = unsafe { *self.offset.get() };
        let aligned_offset = align_up(offset, layout.align());

        if aligned_offset + layout.size() > self.capacity {
            return None; // Out of memory
        }

        unsafe {
            let buffer = &mut *self.buffer.get();
            let ptr = buffer.as_mut_ptr().add(aligned_offset);
            *self.offset.get() = aligned_offset + layout.size();
            Some(NonNull::new_unchecked(ptr))
        }
    }

    /// Allocate space for a value and write it there.
    pub fn alloc<T>(&self, value: T) -> Option<&mut T> {
        let layout = std::alloc::Layout::new::<T>();
        let ptr = self.alloc_layout(layout)?;
        unsafe {
            (ptr.as_ptr() as *mut T).write(value);
            Some(&mut *(ptr.as_ptr() as *mut T))
        }
    }

    /// Allocate a slice of items.
    pub fn alloc_slice<T: Copy>(&self, items: &[T]) -> Option<&mut [T]> {
        let layout = std::alloc::Layout::array::<T>(items.len()).ok()?;
        let ptr = self.alloc_layout(layout)?;
        unsafe {
            let dest = ptr.as_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(items.as_ptr(), dest, items.len());
            Some(std::slice::from_raw_parts_mut(dest, items.len()))
        }
    }

    /// Allocate a string in the arena.
    pub fn alloc_str(&self, s: &str) -> Option<&str> {
        let bytes = self.alloc_slice(s.as_bytes())?;
        // Safety: we copied valid UTF-8 bytes
        unsafe { Some(std::str::from_utf8_unchecked(bytes)) }
    }

    /// Get the number of bytes used.
    pub fn used(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    /// Get the number of bytes remaining.
    pub fn remaining(&self) -> usize {
        self.capacity - self.used()
    }

    /// Reset the arena, allowing all memory to be reused.
    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Get usage as a percentage.
    pub fn usage_percentage(&self) -> f64 {
        (self.used() as f64 / self.capacity as f64) * 100.0
    }
}

fn align_up(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}

/// A typed arena that allocates objects of a specific type.
/// More type-safe than the generic Arena.
pub struct TypedArena<T> {
    chunks: UnsafeCell<Vec<Vec<T>>>,
    current_chunk: UnsafeCell<usize>,
    current_pos: UnsafeCell<usize>,
    chunk_size: usize,
}

impl<T> TypedArena<T> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: UnsafeCell::new(vec![Vec::with_capacity(chunk_size)]),
            current_chunk: UnsafeCell::new(0),
            current_pos: UnsafeCell::new(0),
            chunk_size,
        }
    }

    /// Allocate a value in the arena.
    pub fn alloc(&self, value: T) -> &mut T {
        unsafe {
            let chunks = &mut *self.chunks.get();
            let current_chunk = *self.current_chunk.get();
            let current_pos = *self.current_pos.get();

            if current_pos >= chunks[current_chunk].len() {
                // Need more space in current chunk
                if chunks[current_chunk].capacity() == chunks[current_chunk].len() {
                    // Chunk is full, allocate new chunk
                    chunks.push(Vec::with_capacity(self.chunk_size));
                    *self.current_chunk.get() = chunks.len() - 1;
                    *self.current_pos.get() = 0;
                }
            }

            let chunk_idx = *self.current_chunk.get();
            let pos = *self.current_pos.get();
            chunks[chunk_idx].push(value);
            *self.current_pos.get() = pos + 1;

            let chunk = &mut chunks[chunk_idx];
            &mut *(chunk.as_mut_ptr().add(chunk.len() - 1))
        }
    }

    /// Get the total number of allocated items.
    pub fn len(&self) -> usize {
        unsafe {
            let chunks = &*self.chunks.get();
            chunks.iter().map(|c| c.len()).sum()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Arena-allocated linked list node.
pub struct ArenaList<'a, T> {
    arena: &'a Arena,
    head: Option<&'a ArenaNode<'a, T>>,
}

struct ArenaNode<'a, T> {
    data: T,
    next: Option<&'a ArenaNode<'a, T>>,
}

impl<'a, T> ArenaList<'a, T> {
    pub fn new(arena: &'a Arena) -> Self {
        Self { arena, head: None }
    }

    pub fn push_front(&mut self, value: T) -> Option<()> {
        let node = self.arena.alloc(ArenaNode {
            data: value,
            next: self.head,
        })?;
        self.head = Some(node);
        Some(())
    }

    pub fn head(&self) -> Option<&T> {
        self.head.map(|node| &node.data)
    }

    pub fn iter(&self) -> ArenaListIter<'a, T> {
        ArenaListIter {
            current: self.head,
        }
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

pub struct ArenaListIter<'a, T> {
    current: Option<&'a ArenaNode<'a, T>>,
}

impl<'a, T> Iterator for ArenaListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next;
            &node.data
        })
    }
}

/// Scoped arena that ties allocations to a lifetime.
pub struct ScopedArena {
    inner: Arena,
}

impl ScopedArena {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arena::new(capacity),
        }
    }

    /// Allocate within a scope. The returned reference is valid as long as
    /// the ScopedArena exists.
    pub fn alloc_in_scope<T>(&self, value: T) -> Option<&T> {
        self.inner.alloc(value).map(|r| &*r)
    }

    pub fn used(&self) -> usize {
        self.inner.used()
    }

    pub fn remaining(&self) -> usize {
        self.inner.remaining()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_allocation() {
        let arena = Arena::new(1024);
        let val = arena.alloc(42u32).unwrap();
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_arena_multiple_allocations() {
        let arena = Arena::new(1024);
        let a = arena.alloc(1u64).unwrap();
        let b = arena.alloc(2u64).unwrap();
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert!(arena.used() > 0);
    }

    #[test]
    fn test_arena_out_of_memory() {
        let arena = Arena::new(4); // Very small arena
        let _ = arena.alloc(1u32).unwrap();
        // Second allocation should fail
        assert!(arena.alloc(2u32).is_none());
    }

    #[test]
    fn test_arena_slice() {
        let arena = Arena::new(1024);
        let data = vec![1u32, 2, 3, 4, 5];
        let slice = arena.alloc_slice(&data).unwrap();
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_str() {
        let arena = Arena::new(1024);
        let s = arena.alloc_str("hello world").unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new(1024);
        arena.alloc(1u32).unwrap();
        arena.alloc(2u32).unwrap();
        assert!(arena.used() > 0);

        arena.reset();
        assert_eq!(arena.used(), 0);

        // Can allocate again after reset
        let val = arena.alloc(3u32).unwrap();
        assert_eq!(*val, 3);
    }

    #[test]
    fn test_arena_usage_percentage() {
        let arena = Arena::new(1000);
        assert!((arena.usage_percentage() - 0.0).abs() < f64::EPSILON);

        arena.alloc([0u8; 500]).unwrap();
        let pct = arena.usage_percentage();
        assert!((pct - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_arena_remaining() {
        let arena = Arena::new(1024);
        let initial = arena.remaining();
        arena.alloc([0u8; 256]).unwrap();
        assert!(arena.remaining() < initial);
    }

    #[test]
    fn test_typed_arena() {
        let arena = TypedArena::new(10);
        let a = arena.alloc(String::from("hello"));
        let b = arena.alloc(String::from("world"));
        assert_eq!(a, "hello");
        assert_eq!(b, "world");
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_typed_arena_many() {
        let arena = TypedArena::new(5);
        for i in 0..100 {
            arena.alloc(i);
        }
        assert_eq!(arena.len(), 100);
    }

    #[test]
    fn test_arena_linked_list() {
        let arena = Arena::new(4096);
        let mut list = ArenaList::new(&arena);

        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        assert_eq!(list.head(), Some(&1));
        assert_eq!(list.len(), 3);

        let items: Vec<&i32> = list.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn test_arena_linked_list_iteration() {
        let arena = Arena::new(4096);
        let mut list = ArenaList::new(&arena);

        for i in 0..10 {
            list.push_front(i);
        }

        let sum: i32 = list.iter().sum();
        assert_eq!(sum, 45); // 0+1+2+...+9
    }

    #[test]
    fn test_scoped_arena() {
        let scoped = ScopedArena::new(1024);
        let val = scoped.alloc_in_scope(42u32).unwrap();
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_arena_alignment() {
        let arena = Arena::new(1024);
        // Allocate a u8 then a u64 (which needs 8-byte alignment)
        arena.alloc(1u8).unwrap();
        let val = arena.alloc(42u64).unwrap();
        assert_eq!(*val, 42);
        // Verify alignment
        let addr = val as *const u64 as usize;
        assert_eq!(addr % 8, 0);
    }

    #[test]
    fn test_arena_zero_initialized() {
        let arena = Arena::new(64);
        // Memory should be zero-initialized
        let layout = std::alloc::Layout::new::<[u8; 32]>();
        let ptr = arena.alloc_layout(layout).unwrap();
        unsafe {
            let slice = std::slice::from_raw_parts(ptr.as_ptr(), 32);
            assert!(slice.iter().all(|&b| b == 0));
        }
    }

    #[test]
    fn test_typed_arena_empty() {
        let arena = TypedArena::<i32>::new(10);
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }
}
