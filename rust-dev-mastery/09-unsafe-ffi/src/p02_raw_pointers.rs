//! # Raw Pointers: `*const T` and `*mut T`
//!
//! Raw pointers are the most fundamental unsafe building block. Unlike references,
//! raw pointers:
//! - Are allowed to be null
//! - Are not guaranteed to point to valid memory
//! - Do not have automatic cleanup (no Drop)
//! - Can alias (multiple pointers to the same location)
//! - Can be unaligned
//!
//! However, you can only *dereference* a raw pointer inside an `unsafe` block,
//! because the compiler cannot verify the pointer is valid.

use std::ptr;

/// A simple arena allocator demonstrating raw pointer arithmetic.
/// This is a common pattern in high-performance systems where you want
/// to allocate many small objects and free them all at once.
pub struct Arena {
    buf: *mut u8,
    cap: usize,
    offset: usize,
}

impl Arena {
    /// Create a new arena with the given capacity in bytes.
    pub fn new(capacity: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(capacity, 8).unwrap();
        // SAFETY: Layout has non-zero size (capacity > 0 assumed from caller).
        // We handle allocation failure by panicking (alloc returns null on failure).
        let buf = unsafe { std::alloc::alloc(layout) };
        if buf.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Arena {
            buf,
            cap: capacity,
            offset: 0,
        }
    }

    /// Allocate `size` bytes from the arena, aligned to `align`.
    /// Returns a raw pointer to the allocated memory, or null if out of space.
    ///
    /// # Safety
    /// The returned pointer is valid for `size` bytes. The caller must ensure
    /// the memory is properly initialized before reading from it.
    pub fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        // Align the current offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        if aligned_offset + size > self.cap {
            return ptr::null_mut();
        }
        // SAFETY: We've verified that `aligned_offset + size <= self.cap`,
        // so the pointer arithmetic stays within the allocated buffer.
        let result = unsafe { self.buf.add(aligned_offset) };
        self.offset = aligned_offset + size;
        result
    }

    /// Allocate space for a single value of type T and write it into the arena.
    pub fn alloc_value<T>(&mut self, value: T) -> Option<*mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = self.alloc(size, align);
        if ptr.is_null() {
            return None;
        }
        // SAFETY: ptr is valid for `size` bytes and properly aligned for T.
        // We write `value` into this memory, initializing it.
        unsafe {
            let typed_ptr = ptr.cast::<T>();
            ptr::write(typed_ptr, value);
            Some(typed_ptr)
        }
    }

    /// Reset the arena, allowing all memory to be reused.
    /// Does NOT call Drop on any allocated values.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn used(&self) -> usize {
        self.offset
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.cap, 8).unwrap();
        // SAFETY: self.buf was allocated with this layout in `new`.
        unsafe {
            std::alloc::dealloc(self.buf, layout);
        }
    }
}

/// Demonstrates pointer arithmetic for iterating over a raw buffer.
/// This is similar to how C code iterates over arrays.
///
/// # Safety
/// `ptr` must point to a valid array of at least `count` elements of type `T`.
pub unsafe fn sum_raw_array(ptr: *const i32, count: usize) -> i64 {
    let mut sum: i64 = 0;
    for i in 0..count {
        // SAFETY: Caller guarantees ptr points to at least `count` elements.
        // ptr.add(i) is within bounds for i in 0..count.
        sum += *ptr.add(i) as i64;
    }
    sum
}

/// Demonstrates null pointer checks and pointer validity patterns.
pub struct NullablePtr<T> {
    ptr: *mut T,
}

impl<T> NullablePtr<T> {
    pub fn null() -> Self {
        NullablePtr {
            ptr: ptr::null_mut(),
        }
    }

    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        NullablePtr {
            ptr: Box::into_raw(boxed),
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn as_ref(&self) -> Option<&T> {
        // SAFETY: If ptr is non-null, it was created from Box::into_raw in `new`,
        // so it points to a valid, heap-allocated T. The Box guarantees proper
        // alignment and initialization. We return a shared reference, which is
        // safe because we don't provide mutable access while the reference exists.
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        // SAFETY: Same as as_ref, but we return a mutable reference.
        // The &mut self ensures exclusive access to the pointer.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for NullablePtr<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: If ptr is non-null, it was created from Box::into_raw,
            // so we can reconstruct the Box to deallocate it. This is the
            // inverse operation of Box::into_raw.
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}

/// Demonstrates pointer casting between types.
/// This is common in low-level code that needs to reinterpret memory.
///
/// # Safety
/// `src` must point to a valid `Src` value. The caller must ensure
/// the bit representation is meaningful as `Dst`.
pub unsafe fn transmute_via_ptr<Src, Dst>(src: &Src) -> Dst {
    // SAFETY: We read from src as a Src and write the same bits as Dst.
    // This is equivalent to std::mem::transmute but uses pointer casting.
    // The caller must guarantee the bit pattern is valid for Dst.
    ptr::read(src as *const Src as *const Dst)
}

/// A raw intrusive linked list using raw pointers.
/// Intrusive lists embed the list node directly in the stored element,
/// avoiding heap allocation per node.
pub struct IntrusiveNode {
    next: *mut IntrusiveNode,
    prev: *mut IntrusiveNode,
    value: i64,
}

pub struct IntrusiveList {
    head: *mut IntrusiveNode,
    tail: *mut IntrusiveNode,
    len: usize,
}

impl IntrusiveNode {
    pub fn new(value: i64) -> Self {
        IntrusiveNode {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
            value,
        }
    }
}

impl IntrusiveList {
    pub fn new() -> Self {
        IntrusiveList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
        }
    }

    /// Push a node to the back of the list.
    ///
    /// # Safety
    /// `node` must point to a valid, heap-allocated IntrusiveNode that is not
    /// already part of any list. The caller must ensure the node outlives the list.
    pub unsafe fn push_back(&mut self, node: *mut IntrusiveNode) {
        // SAFETY: Caller guarantees node is valid and not in another list.
        (*node).next = ptr::null_mut();
        (*node).prev = self.tail;

        if self.tail.is_null() {
            self.head = node;
        } else {
            // SAFETY: tail is non-null, so it's valid (maintained by list invariants).
            (*self.tail).next = node;
        }
        self.tail = node;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Collect values from the list into a Vec (for testing).
    pub fn values(&self) -> Vec<i64> {
        let mut result = Vec::new();
        let mut current = self.head;
        while !current.is_null() {
            // SAFETY: head/tail/next pointers form a valid linked list.
            // We only read the value and follow next pointers.
            unsafe {
                result.push((*current).value);
                current = (*current).next;
            }
        }
        result
    }
}

/// Demonstrates pointer provenance: a concept where the compiler tracks
/// which allocation a pointer is derived from.
pub fn demonstrate_provenance() -> bool {
    let mut x: u32 = 1;
    let mut y: u32 = 2;

    let px = &mut x as *mut u32;
    let py = &mut y as *mut u32;

    // These are two different allocations, so px and py can never alias.
    // The compiler uses provenance information for optimization.
    unsafe {
        *px = 10;
        *py = 20;
        // This is safe because px and py point to different allocations.
        // If they could alias, the optimizer might reorder these writes.
        (*px + *py) == 30
    }
}

/// Demonstrates `copy_nonoverlapping` for efficient memory copies.
///
/// # Safety
/// - `src` must be valid for reads of `count * size_of::<T>()` bytes
/// - `dst` must be valid for writes of `count * size_of::<T>()` bytes
/// - Both must be properly aligned
/// - The regions must not overlap
pub unsafe fn copy_slice_nonoverlapping<T>(src: &[T], dst: &mut [T]) {
    assert_eq!(src.len(), dst.len());
    // SAFETY: We've asserted equal lengths. The slice references guarantee
    // proper alignment and validity. Non-overlapping is guaranteed by the
    // mutable reference to dst (it cannot alias with src).
    ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_allocation() {
        let mut arena = Arena::new(1024);
        let p1 = arena.alloc_value(42_i32).unwrap();
        let p2 = arena.alloc_value(100_i64).unwrap();

        unsafe {
            assert_eq!(*p1, 42);
            assert_eq!(*p2, 100);
        }
        assert!(arena.used() > 0);
    }

    #[test]
    fn test_arena_alignment() {
        let mut arena = Arena::new(1024);
        // Allocate a u8 (1-byte aligned) then a u64 (8-byte aligned)
        let _ = arena.alloc(1, 1);
        let p = arena.alloc_value(42_u64).unwrap();
        // Verify the pointer is 8-byte aligned
        assert_eq!(p as usize % 8, 0);
        unsafe {
            assert_eq!(*p, 42);
        }
    }

    #[test]
    fn test_arena_exhaustion() {
        let mut arena = Arena::new(16);
        // Should succeed for small allocations
        assert!(arena.alloc_value(1_u8).is_some());
        assert!(arena.alloc_value(2_u8).is_some());
        // Eventually the arena should run out
        let mut allocated = 2;
        while arena.alloc_value(1_u8).is_some() {
            allocated += 1;
        }
        assert!(allocated > 1);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new(1024);
        let _ = arena.alloc_value(1_i32);
        let _ = arena.alloc_value(2_i32);
        assert!(arena.used() > 0);
        arena.reset();
        assert_eq!(arena.used(), 0);
        // Can allocate again after reset
        assert!(arena.alloc_value(3_i32).is_some());
    }

    #[test]
    fn test_sum_raw_array() {
        let data = [1, 2, 3, 4, 5];
        // SAFETY: data is a valid array of 5 i32 values.
        let sum = unsafe { sum_raw_array(data.as_ptr(), data.len()) };
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_sum_raw_array_empty() {
        let data: [i32; 0] = [];
        let sum = unsafe { sum_raw_array(data.as_ptr(), 0) };
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_nullable_ptr_null() {
        let p = NullablePtr::<i32>::null();
        assert!(p.is_null());
        assert!(p.as_ref().is_none());
    }

    #[test]
    fn test_nullable_ptr_value() {
        let mut p = NullablePtr::new(42);
        assert!(!p.is_null());
        assert_eq!(p.as_ref(), Some(&42));
        *p.as_mut().unwrap() = 100;
        assert_eq!(p.as_ref(), Some(&100));
    }

    #[test]
    fn test_nullable_ptr_drop() {
        // Ensure no memory leak (would show up in Miri or valgrind)
        for i in 0..1000 {
            let _p = NullablePtr::new(i);
        }
    }

    #[test]
    fn test_intrusive_list_push_and_collect() {
        let mut list = IntrusiveList::new();
        assert!(list.is_empty());

        let mut n1 = Box::new(IntrusiveNode::new(10));
        let mut n2 = Box::new(IntrusiveNode::new(20));
        let mut n3 = Box::new(IntrusiveNode::new(30));

        unsafe {
            list.push_back(&mut *n1);
            list.push_back(&mut *n2);
            list.push_back(&mut *n3);
        }

        assert_eq!(list.len(), 3);
        assert_eq!(list.values(), vec![10, 20, 30]);
    }

    #[test]
    fn test_intrusive_list_empty() {
        let list = IntrusiveList::new();
        assert_eq!(list.values(), Vec::<i64>::new());
    }

    #[test]
    fn test_pointer_provenance() {
        assert!(demonstrate_provenance());
    }

    #[test]
    fn test_copy_nonoverlapping() {
        let src = [1, 2, 3, 4, 5];
        let mut dst = [0; 5];
        unsafe {
            copy_slice_nonoverlapping(&src, &mut dst);
        }
        assert_eq!(dst, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_transmute_via_ptr() {
        let val: u32 = 0x40490FDB; // Approximate bits of pi as f32
        // SAFETY: We're reading the bit pattern of a u32 as an f32.
        // This is valid because every u32 bit pattern is a valid f32.
        let f: f32 = unsafe { transmute_via_ptr(&val) };
        // Convert back and verify roundtrip
        let back: u32 = unsafe { transmute_via_ptr(&f) };
        assert_eq!(val, back);
    }
}
