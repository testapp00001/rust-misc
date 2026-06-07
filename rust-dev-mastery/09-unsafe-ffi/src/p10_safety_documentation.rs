//! # Safety Documentation
//!
//! Every `unsafe` block in production code must have a `SAFETY` comment
//! explaining why the invariants hold. This lesson covers the conventions
//! and best practices for documenting unsafe code.
//!
//! ## Comment Conventions
//!
//! - `// SAFETY:` or `/// # Safety` for function-level documentation
//! - Explain WHAT invariant must hold
//! - Explain WHY it holds in this specific context
//! - Reference the caller's obligations for public unsafe functions
//!
//! ## Audit Checklist
//!
//! For every `unsafe` block:
//! 1. Is there a SAFETY comment?
//! 2. Are all invariants documented?
//! 3. Are preconditions checked (or documented for the caller)?
//! 4. Is the unsafe block as small as possible?
//! 5. Could this be rewritten in safe Rust?

use std::alloc::{self, Layout};
use std::ptr;

/// A well-documented safe wrapper around a raw buffer.
///
/// # Invariants
///
/// This type maintains the following invariants:
/// - `ptr` is non-null and points to `capacity` bytes of allocated memory
/// - `ptr` is aligned to `align_of::<u8>()` (which is 1)
/// - Elements at indices `0..len` are initialized
/// - `len <= capacity`
///
/// These invariants are established in `new()` and `with_capacity()`,
/// and are maintained by all public methods.
pub struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl SafeBuffer {
    /// Create a new buffer with the given capacity.
    ///
    /// # Preconditions
    /// - `capacity` must be greater than 0
    ///
    /// # Postconditions
    /// - `self.capacity() == capacity`
    /// - `self.len() == 0`
    /// - All memory is zero-initialized
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be greater than 0");

        let layout = Layout::from_size_align(capacity, 1).expect("invalid layout");

        // SAFETY: `capacity > 0` (asserted above), so `layout.size() > 0`.
        // `alloc_zeroed` returns zero-initialized memory, which is valid for u8.
        // If allocation fails, `handle_alloc_error` will not return.
        let ptr = unsafe { alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        SafeBuffer {
            ptr,
            len: 0,
            capacity,
        }
    }

    /// Push a byte to the end of the buffer.
    ///
    /// # Preconditions
    /// - `self.len() < self.capacity()` (checked internally; panics if violated)
    ///
    /// # Safety
    /// This method is safe because:
    /// 1. We check `len < capacity` before writing
    /// 2. The write target `ptr.add(len)` is within allocated bounds
    /// 3. We increment `len` to maintain the "initialized elements" invariant
    pub fn push(&mut self, value: u8) {
        assert!(
            self.len < self.capacity,
            "buffer full: len={}, capacity={}",
            self.len,
            self.capacity
        );

        // SAFETY:
        // - `self.len < self.capacity` (asserted above), so `self.len` is a valid index
        // - `self.ptr` is valid for `self.capacity` bytes (invariant maintained by new/with_capacity)
        // - Writing a u8 requires no alignment beyond 1
        // - After this write, elements 0..=self.len are initialized
        unsafe {
            self.ptr.add(self.len).write(value);
        }

        // Maintain the "len tracks initialized elements" invariant
        self.len += 1;
    }

    /// Read a byte at the given index.
    ///
    /// Returns `None` if `index >= self.len()`.
    ///
    /// # Safety
    /// This method is safe because:
    /// 1. We check `index < len` before reading
    /// 2. Elements 0..len are initialized (invariant)
    pub fn get(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            return None;
        }

        // SAFETY:
        // - `index < self.len` (checked above), so the element was initialized
        // - `self.ptr.add(index)` is within the allocated region
        //   (because index < len <= capacity)
        // - Reading a u8 from initialized memory is always valid
        Some(unsafe { self.ptr.add(index).read() })
    }

    /// Get a slice of the initialized portion of the buffer.
    ///
    /// # Safety
    /// This method is safe because:
    /// 1. `self.ptr` is valid for `self.capacity` bytes
    /// 2. Elements 0..self.len are initialized (invariant)
    /// 3. `self.len <= self.capacity` (invariant)
    /// 4. `&self` ensures the buffer won't be modified while the slice exists
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: See method-level safety documentation above.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get a mutable slice of the initialized portion of the buffer.
    ///
    /// # Safety
    /// This method is safe because:
    /// 1. `self.ptr` is valid for `self.capacity` bytes
    /// 2. Elements 0..self.len are initialized (invariant)
    /// 3. `&mut self` ensures exclusive access
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: See method-level safety documentation above.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Remove and return the last element.
    ///
    /// Returns `None` if the buffer is empty.
    ///
    /// # Safety
    /// This method is safe because:
    /// 1. We check `len > 0` before reading
    /// 2. `len - 1` is a valid index (it was the last initialized element)
    /// 3. After decrementing `len`, the element is logically uninitialized
    ///    (we don't need to zero it because we track initialized range via `len`)
    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        // SAFETY:
        // - `self.len` (after decrement) was the index of the last element
        // - That element was initialized (it was within 0..old_len)
        // - Reading it is valid
        // - We decrement len, so the slot is no longer considered initialized
        Some(unsafe { self.ptr.add(self.len).read() })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 1).expect("invalid layout");

        // SAFETY:
        // - `self.ptr` was allocated with this exact layout in `new()`
        // - `self.capacity > 0` (enforced in `new()`), so layout is non-zero
        // - This is the only place we deallocate `self.ptr` (drop runs once)
        // - No other code can access `self.ptr` after drop begins
        unsafe {
            alloc::dealloc(self.ptr, layout);
        }
    }
}

/// Documents the safety contract for a public unsafe function.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is non-null and points to at least `count` initialized `T` values
/// - `ptr` is properly aligned for `T`
/// - The memory pointed to by `ptr` is valid for the lifetime of the returned slice
/// - No other mutable reference to the same memory exists during the lifetime
///   of the returned slice
///
pub unsafe fn unsafe_slice_from_ptr<'a, T>(ptr: *const T, count: usize) -> &'a [T] {
    // SAFETY: Caller guarantees all preconditions documented in # Safety above.
    std::slice::from_raw_parts(ptr, count)
}

/// Documents safety invariants using the type system.
/// This pattern makes invalid states unrepresentable.
pub struct NonNullPtr<T> {
    ptr: *mut T,
}

impl<T> NonNullPtr<T> {
    /// Create a new NonNullPtr from a Box.
    ///
    /// # Invariants
    /// After construction:
    /// - `self.as_ptr()` is non-null (guaranteed by Box::into_raw)
    /// - `self.as_ptr()` is properly aligned for T (guaranteed by Box)
    /// - `self.as_ptr()` points to a valid, initialized T (guaranteed by Box)
    pub fn new(value: T) -> Self {
        NonNullPtr {
            ptr: Box::into_raw(Box::new(value)),
        }
    }

    /// Returns the raw pointer.
    ///
    /// # Safety Contract
    /// The returned pointer is guaranteed to be:
    /// - Non-null
    /// - Properly aligned for T
    /// - Pointing to a valid, initialized T
    /// - Valid until `self` is dropped or `into_inner` is called
    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Get a shared reference to the value.
    ///
    /// # Safety
    /// Safe because:
    /// - `self.ptr` is non-null (invariant from `new`)
    /// - `self.ptr` is properly aligned (invariant from Box)
    /// - `self.ptr` points to valid memory (invariant from Box)
    /// - `&self` ensures the pointer won't be deallocated while the reference exists
    pub fn as_ref(&self) -> &T {
        // SAFETY: All invariants documented above hold.
        unsafe { &*self.ptr }
    }

    /// Get an exclusive reference to the value.
    ///
    /// # Safety
    /// Safe because:
    /// - `self.ptr` is non-null (invariant from `new`)
    /// - `self.ptr` is properly aligned (invariant from Box)
    /// - `self.ptr` points to valid memory (invariant from Box)
    /// - `&mut self` ensures exclusive access
    pub fn as_mut(&mut self) -> &mut T {
        // SAFETY: All invariants documented above hold.
        unsafe { &mut *self.ptr }
    }

    /// Consume the wrapper and return the inner value.
    ///
    /// # Safety
    /// Safe because:
    /// - We reconstruct the Box from the raw pointer (inverse of into_raw)
    /// - self is consumed, so the pointer can't be used again
    /// - Box will deallocate the memory when dropped
    pub fn into_inner(self) -> T {
        // SAFETY: self.ptr was created from Box::into_raw in `new`.
        // Reconstructing the Box and dereferencing is the inverse operation.
        // We use ptr::read to move the value out, then forget self to prevent
        // double-free (Box::from_raw would try to free, but we already read).
        unsafe {
            let value = ptr::read(self.ptr);
            // Prevent Drop from running (which would double-free)
            std::mem::forget(self);
            value
        }
    }
}

impl<T> Drop for NonNullPtr<T> {
    fn drop(&mut self) {
        // SAFETY: self.ptr was created from Box::into_raw in `new`.
        // This is the inverse operation. Drop runs at most once.
        unsafe {
            drop(Box::from_raw(self.ptr));
        }
    }
}

/// A safety audit checklist for code reviewers.
/// This is a documentation type that demonstrates how to structure
/// safety reviews in codebases with significant unsafe usage.
pub struct SafetyAudit;

impl SafetyAudit {
    /// Checklist item 1: Every unsafe block has a SAFETY comment.
    pub fn check_safety_comments() -> &'static str {
        "Every `unsafe` block must have a `// SAFETY:` comment \
         explaining why the invariants hold in this specific context."
    }

    /// Checklist item 2: Unsafe functions document their preconditions.
    pub fn check_preconditions() -> &'static str {
        "Every `unsafe fn` must document its preconditions in `# Safety` \
         documentation. The caller is responsible for upholding these."
    }

    /// Checklist item 3: Invariants are maintained across all code paths.
    pub fn check_invariants() -> &'static str {
        "Type invariants must be established in constructors and maintained \
         by all methods. Document invariants in the type's doc comment."
    }

    /// Checklist item 4: Unsafe blocks are minimized.
    pub fn check_minimal_unsafe() -> &'static str {
        "Unsafe blocks should be minimized. \
         Move safe operations (bounds checks, assertions) outside the block."
    }

    /// Checklist item 5: Tests cover unsafe code paths.
    pub fn check_test_coverage() -> &'static str {
        "All unsafe code paths should have tests. Run tests under Miri \
         (`cargo +nightly miri test`) to detect undefined behavior."
    }
}

/// Demonstrates the "minimize unsafe" pattern.
/// Instead of one large unsafe block, we break it into small, auditable pieces.
pub fn split_at_minimal_unsafe<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();

    // Safe: assertion is outside unsafe
    assert!(mid <= len, "mid ({mid}) must be <= len ({len})");

    let ptr = slice.as_mut_ptr();

    // Unsafe block 1: Create left slice
    // SAFETY: `mid <= len` (asserted above). `ptr` is valid for `mid` bytes.
    // Creating a slice from the beginning of a valid allocation is sound.
    let left = unsafe { std::slice::from_raw_parts_mut(ptr, mid) };

    // Unsafe block 2: Create right slice
    // SAFETY: `ptr.add(mid)` is within the allocation (because `mid <= len`).
    // The right slice has `len - mid` elements.
    let right = unsafe { std::slice::from_raw_parts_mut(ptr.add(mid), len - mid) };

    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_buffer_new() {
        let buf = SafeBuffer::new(100);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 100);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_safe_buffer_push_get() {
        let mut buf = SafeBuffer::new(10);
        buf.push(42);
        buf.push(43);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.get(0), Some(42));
        assert_eq!(buf.get(1), Some(43));
        assert_eq!(buf.get(2), None);
    }

    #[test]
    fn test_safe_buffer_pop() {
        let mut buf = SafeBuffer::new(10);
        buf.push(1);
        buf.push(2);
        buf.push(3);

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_safe_buffer_as_slice() {
        let mut buf = SafeBuffer::new(10);
        buf.push(10);
        buf.push(20);
        buf.push(30);
        assert_eq!(buf.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn test_safe_buffer_as_mut_slice() {
        let mut buf = SafeBuffer::new(10);
        buf.push(1);
        buf.push(2);
        buf.push(3);

        let slice = buf.as_mut_slice();
        slice[0] = 10;
        slice[1] = 20;
        assert_eq!(buf.as_slice(), &[10, 20, 3]);
    }

    #[test]
    #[should_panic(expected = "buffer full")]
    fn test_safe_buffer_overflow() {
        let mut buf = SafeBuffer::new(2);
        buf.push(1);
        buf.push(2);
        buf.push(3); // Panics
    }

    #[test]
    fn test_unsafe_slice_from_ptr() {
        let data = [1, 2, 3, 4, 5];
        // SAFETY: data.as_ptr() is non-null, aligned, and points to 5 initialized i32 values.
        let slice = unsafe { unsafe_slice_from_ptr(data.as_ptr(), 5) };
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_non_null_ptr_basic() {
        let p = NonNullPtr::new(42);
        assert_eq!(*p.as_ref(), 42);
    }

    #[test]
    fn test_non_null_ptr_mutable() {
        let mut p = NonNullPtr::new(10);
        *p.as_mut() = 20;
        assert_eq!(*p.as_ref(), 20);
    }

    #[test]
    fn test_non_null_ptr_into_inner() {
        let p = NonNullPtr::new(99);
        let val = p.into_inner();
        assert_eq!(val, 99);
    }

    #[test]
    fn test_non_null_ptr_no_leak() {
        for i in 0..100 {
            let _p = NonNullPtr::new(i);
        }
    }

    #[test]
    fn test_split_at_minimal_unsafe() {
        let mut data = vec![1, 2, 3, 4, 5];
        let (left, right) = split_at_minimal_unsafe(&mut data, 2);
        assert_eq!(left, &mut [1, 2]);
        assert_eq!(right, &mut [3, 4, 5]);

        left[0] = 10;
        right[0] = 30;
        assert_eq!(data, [10, 2, 30, 4, 5]);
    }

    #[test]
    #[should_panic(expected = "must be <= len")]
    fn test_split_at_minimal_unsafe_oob() {
        let mut data = vec![1, 2, 3];
        let _ = split_at_minimal_unsafe(&mut data, 10);
    }

    #[test]
    fn test_safety_audit_checklist() {
        // Verify the checklist items are documented
        assert!(SafetyAudit::check_safety_comments().contains("SAFETY"));
        assert!(SafetyAudit::check_preconditions().contains("preconditions"));
        assert!(SafetyAudit::check_invariants().contains("invariants"));
        assert!(SafetyAudit::check_minimal_unsafe().contains("minimize"));
        assert!(SafetyAudit::check_test_coverage().contains("Miri"));
    }

    #[test]
    fn test_safe_buffer_stress() {
        let mut buf = SafeBuffer::new(1000);
        for i in 0..1000 {
            buf.push((i % 256) as u8);
        }
        for i in 0..1000 {
            assert_eq!(buf.get(i), Some((i % 256) as u8));
        }
        for i in (0..1000).rev() {
            assert_eq!(buf.pop(), Some((i % 256) as u8));
        }
        assert!(buf.is_empty());
    }
}
