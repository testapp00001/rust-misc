//! # Memory Safety: Preventing Undefined Behavior
//!
//! This lesson covers the rules that unsafe Rust code must follow to avoid
//! undefined behavior (UB). The compiler assumes UB never happens, so violating
//! these rules can lead to arbitrary miscompilation, not just crashes.
//!
//! Key rules:
//! 1. **Aliasing**: No mutable reference may alias with any other reference
//! 2. **Validity**: References must always be valid (non-null, aligned, dereferenceable)
//! 3. **Initialization**: Memory must be initialized before reading
//! 4. **Alignment**: Pointers must be aligned for their type
//! 5. **Provenance**: Pointers derived from one allocation should not access another
//! 6. **No data races**: Concurrent access requires synchronization

use std::alloc::{self, Layout};
use std::ptr;

/// Demonstrates correct alignment handling.
/// Many hardware architectures require aligned memory access, and Rust's
/// type system assumes alignment for references.
pub struct AlignedBuffer {
    ptr: *mut u8,
    layout: Layout,
}

impl AlignedBuffer {
    /// Allocate a buffer with the specified size and minimum alignment.
    pub fn allocate(size: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(size, align).expect("invalid layout");
        // SAFETY: layout has non-zero size.
        let ptr = unsafe { alloc::alloc(layout) };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        AlignedBuffer { ptr, layout }
    }

    /// Get the buffer as a mutable slice of T.
    ///
    /// # Safety
    /// - The buffer must be large enough for `count` elements of T
    /// - The buffer must be aligned to `align_of::<T>()`
    /// - The memory must be initialized for T values
    pub unsafe fn as_slice_mut<T>(&mut self, count: usize) -> &mut [T] {
        assert!(
            self.layout.size() >= count * std::mem::size_of::<T>(),
            "buffer too small"
        );
        assert!(
            self.layout.align() >= std::mem::align_of::<T>(),
            "buffer misaligned for T"
        );
        // SAFETY: We've verified size and alignment. Caller guarantees initialization.
        std::slice::from_raw_parts_mut(self.ptr as *mut T, count)
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn size(&self) -> usize {
        self.layout.size()
    }

    pub fn align(&self) -> usize {
        self.layout.align()
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated with self.layout in `allocate`.
        unsafe {
            alloc::dealloc(self.ptr, self.layout);
        }
    }
}

/// Demonstrates correct uninitialized memory handling with MaybeUninit.
/// This is the safe pattern for working with uninitialized memory.
pub fn read_from_c_into_buffer(buf: &mut [u8]) -> usize {
    use std::mem::MaybeUninit;

    // Create a buffer of MaybeUninit<u8> - no initialization needed
    let mut uninit_buf: [MaybeUninit<u8>; 256] = unsafe { MaybeUninit::uninit().assume_init() };

    // Simulate filling some elements (e.g., from a C read() call)
    let count = buf.len().min(256);
    for i in 0..count {
        uninit_buf[i] = MaybeUninit::new(buf[i]);
    }

    // Convert initialized portion to &[u8]
    // SAFETY: Elements 0..count have been initialized.
    let initialized: &[u8] =
        unsafe { std::slice::from_raw_parts(uninit_buf.as_ptr() as *const u8, count) };

    // Copy back to output
    let copy_len = initialized.len().min(buf.len());
    buf[..copy_len].copy_from_slice(&initialized[..copy_len]);
    copy_len
}

/// Demonstrates the aliasing rules with a split-at implementation
/// that correctly handles mutable references.
///
/// The key rule: you cannot have two `&mut T` pointing to the same memory.
/// This function is safe because it produces non-overlapping mutable slices.
pub fn split_at_mut_safe<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= slice.len(), "mid out of bounds");
    let ptr = slice.as_mut_ptr();
    let len = slice.len();

    // SAFETY: We've verified mid <= len. The two slices:
    // - [ptr, ptr+mid) and [ptr+mid, ptr+len)
    // - Do not overlap because mid splits the range
    // - Are both valid because they're within the original slice
    // - Are properly aligned because they start at aligned offsets from the base
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

/// Demonstrates a common source of UB: use-after-free via raw pointers.
/// This struct prevents that by enforcing ownership through the type system.
pub struct OwnedPtr<T> {
    ptr: *mut T,
    /// Track whether we've been dropped to detect use-after-free
    /// (only in debug builds; production code uses the type system instead)
    #[cfg(debug_assertions)]
    alive: bool,
}

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        OwnedPtr {
            ptr: Box::into_raw(boxed),
            #[cfg(debug_assertions)]
            alive: true,
        }
    }

    /// Get a reference to the contained value.
    pub fn as_ref(&self) -> &T {
        #[cfg(debug_assertions)]
        assert!(self.alive, "use after free detected!");
        // SAFETY: ptr was created from Box::into_raw and has not been freed.
        // &self ensures the pointer is not dropped while the reference exists.
        unsafe { &*self.ptr }
    }

    /// Get a mutable reference to the contained value.
    pub fn as_mut(&mut self) -> &mut T {
        #[cfg(debug_assertions)]
        assert!(self.alive, "use after free detected!");
        // SAFETY: &mut self ensures exclusive access. ptr is valid.
        unsafe { &mut *self.ptr }
    }

    /// Consume the OwnedPtr and return the inner value.
    pub fn into_inner(mut self) -> T {
        // Take the pointer and set it to null to prevent Drop from freeing it
        let ptr = std::mem::replace(&mut self.ptr, std::ptr::null_mut());
        // SAFETY: ptr was created from Box::into_raw in `new`.
        // We've set self.ptr to null so Drop won't double-free.
        unsafe { *Box::from_raw(ptr) }
    }
}

impl<T> Drop for OwnedPtr<T> {
    fn drop(&mut self) {
        // Only free if ptr is non-null (into_inner sets it to null)
        if !self.ptr.is_null() {
            // SAFETY: ptr was created from Box::into_raw in `new`.
            // After this drop, the pointer is invalid and won't be used.
            unsafe {
                #[cfg(debug_assertions)]
                {
                    self.alive = false;
                }
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}

/// Demonstrates preventing data races through the type system.
/// A type that is `!Sync` cannot be shared between threads.
use std::cell::Cell;

/// A counter that is safe for single-threaded use only.
/// Cell makes it !Sync, preventing data races at compile time.
pub struct LocalCounter {
    count: Cell<u64>,
}

impl LocalCounter {
    pub fn new() -> Self {
        LocalCounter {
            count: Cell::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.set(self.count.get() + 1);
    }

    pub fn get(&self) -> u64 {
        self.count.get()
    }
}

/// Demonstrates correct provenance handling.
/// When casting between pointer types, the provenance (which allocation
/// the pointer comes from) must be preserved.
pub fn read_field_from_struct<T>(base: *const u8, offset: usize) -> T
where
    T: Copy,
{
    // SAFETY: The caller must ensure that `base + offset` is a valid,
    // aligned, initialized T value. The pointer provenance from `base`
    // is preserved through the arithmetic.
    unsafe {
        let field_ptr = base.add(offset) as *const T;
        ptr::read(field_ptr)
    }
}

/// A safe wrapper that maintains provenance correctly.
pub struct StructView<T: Copy> {
    base: *const u8,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy> StructView<T> {
    /// Create a view of a struct at the given base pointer.
    ///
    /// # Safety
    /// `base` must point to a valid, aligned struct containing T at offset 0.
    pub unsafe fn new(base: *const u8) -> Self {
        StructView {
            base,
            _marker: std::marker::PhantomData,
        }
    }

    /// Read the value at the base offset.
    pub fn read(&self) -> T {
        // SAFETY: We maintain the invariant that base is valid and aligned for T.
        unsafe { ptr::read(self.base as *const T) }
    }
}

/// Demonstrates the "self-referential struct" problem and solution.
/// Self-referential structs are fundamentally unsafe in Rust because moving
/// the struct invalidates the internal pointer.
pub struct SelfReferential {
    data: Vec<u8>,
    /// We cannot store a pointer to `data` here because moving the struct
    /// would invalidate the pointer. Instead, we use an offset.
    offset: usize,
    len: usize,
}

impl SelfReferential {
    pub fn new(data: Vec<u8>, offset: usize, len: usize) -> Self {
        assert!(offset + len <= data.len(), "invalid slice bounds");
        SelfReferential { data, offset, len }
    }

    /// Get the slice by computing the pointer each time (safe because
    /// we always derive from the current location of `data`).
    pub fn get_slice(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.len]
    }

    /// This is safe because we recompute the pointer from `self.data`
    /// each time, never storing a dangling pointer.
    pub fn update_data(&mut self, new_data: Vec<u8>) {
        assert!(self.offset + self.len <= new_data.len());
        self.data = new_data;
        // No pointer to fix up -- we use offsets!
    }
}

/// Demonstrates correct memory ordering for atomics.
/// Memory ordering prevents reordering of operations across threads.
pub use std::sync::atomic::{AtomicBool, AtomicI64, Ordering as MemOrdering};
use std::cell::UnsafeCell;

pub struct SpinLock {
    locked: AtomicBool,
    data: UnsafeCell<i64>,
}

// SAFETY: SpinLock provides mutual exclusion through the locked flag.
// Only one thread can hold the lock at a time (guaranteed by try_lock).
unsafe impl Send for SpinLock {}
unsafe impl Sync for SpinLock {}

impl SpinLock {
    pub fn new(value: i64) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to acquire the lock. Returns None if already locked.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_>> {
        if self
            .locked
            .compare_exchange(false, true, MemOrdering::Acquire, MemOrdering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }
}

pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
}

impl SpinLockGuard<'_> {
    pub fn get(&self) -> i64 {
        // SAFETY: We hold the lock, so no other thread is accessing the data.
        unsafe { *self.lock.data.get() }
    }

    pub fn set(&self, value: i64) {
        // SAFETY: We hold the lock, so no other thread is accessing the data.
        unsafe { *self.lock.data.get() = value };
    }
}

impl Drop for SpinLockGuard<'_> {
    fn drop(&mut self) {
        // Release the lock with Release ordering to ensure all writes
        // are visible before the next thread acquires.
        self.lock.locked.store(false, MemOrdering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_aligned_buffer() {
        let mut buf = AlignedBuffer::allocate(1024, 8);
        assert!(buf.size() >= 1024);
        assert!(buf.align() >= 8);

        // Write and read through properly aligned access
        unsafe {
            let slice = buf.as_slice_mut::<u64>(128);
            for i in 0..128 {
                slice[i] = i as u64;
            }
            assert_eq!(slice[0], 0);
            assert_eq!(slice[127], 127);
        }
    }

    #[test]
    fn test_read_from_c_into_buffer() {
        let mut buf = [42u8; 10];
        let count = read_from_c_into_buffer(&mut buf);
        assert_eq!(count, 10);
        assert_eq!(buf, [42u8; 10]);
    }

    #[test]
    fn test_split_at_mut_safe() {
        let mut data = vec![1, 2, 3, 4, 5];
        let (left, right) = split_at_mut_safe(&mut data, 2);
        assert_eq!(left, &mut [1, 2]);
        assert_eq!(right, &mut [3, 4, 5]);

        left[0] = 10;
        right[0] = 30;
        assert_eq!(data, [10, 2, 30, 4, 5]);
    }

    #[test]
    #[should_panic(expected = "mid out of bounds")]
    fn test_split_at_mut_safe_oob() {
        let mut data = vec![1, 2, 3];
        let _ = split_at_mut_safe(&mut data, 10);
    }

    #[test]
    fn test_owned_ptr_basic() {
        let p = OwnedPtr::new(42);
        assert_eq!(*p.as_ref(), 42);
    }

    #[test]
    fn test_owned_ptr_mutable() {
        let mut p = OwnedPtr::new(10);
        *p.as_mut() = 20;
        assert_eq!(*p.as_ref(), 20);
    }

    #[test]
    fn test_owned_ptr_into_inner() {
        let p = OwnedPtr::new(99);
        let val = p.into_inner();
        assert_eq!(val, 99);
        // p is consumed, no double-free
    }

    #[test]
    fn test_owned_ptr_no_leak() {
        for i in 0..1000 {
            let _p = OwnedPtr::new(i);
        }
    }

    #[test]
    fn test_local_counter() {
        let counter = LocalCounter::new();
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn test_struct_view() {
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct MyStruct {
            a: u32,
            b: u32,
        }
        let s = MyStruct { a: 10, b: 20 };
        let base = &s as *const MyStruct as *const u8;
        // SAFETY: base points to a valid MyStruct.
        unsafe {
            let view = StructView::<u32>::new(base);
            assert_eq!(view.read(), 10);
        }
    }

    #[test]
    fn test_self_referential() {
        let sr = SelfReferential::new(vec![10, 20, 30, 40, 50], 1, 3);
        assert_eq!(sr.get_slice(), &[20, 30, 40]);

        let mut sr2 = SelfReferential::new(vec![0; 10], 2, 4);
        sr2.update_data(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(sr2.get_slice(), &[3, 4, 5, 6]);
    }

    #[test]
    fn test_spinlock_single_thread() {
        let lock = SpinLock::new(42);
        let guard = lock.try_lock().unwrap();
        assert_eq!(guard.get(), 42);
        guard.set(100);
        drop(guard);

        let guard2 = lock.try_lock().unwrap();
        assert_eq!(guard2.get(), 100);
    }

    #[test]
    fn test_spinlock_contention() {
        let lock = std::sync::Arc::new(SpinLock::new(0));
        let mut handles = vec![];

        for _ in 0..4 {
            let lock = lock.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    loop {
                        if let Some(guard) = lock.try_lock() {
                            guard.set(guard.get() + 1);
                            break;
                        }
                        thread::yield_now();
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let guard = lock.try_lock().unwrap();
        assert_eq!(guard.get(), 400);
    }
}
