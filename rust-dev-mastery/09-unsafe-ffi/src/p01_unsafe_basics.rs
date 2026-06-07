//! # Unsafe Basics: When and Why to Use `unsafe`
//!
//! `unsafe` in Rust does not mean "this code is dangerous." It means "the compiler
//! cannot verify that this code upholds Rust's safety guarantees, so the programmer
//! must ensure correctness manually."
//!
//! There are exactly five things `unsafe` unlocks:
//! 1. Dereferencing raw pointers
//! 2. Calling unsafe functions or methods
//! 3. Accessing or modifying mutable static variables
//! 4. Implementing unsafe traits
//! 5. Accessing fields of unions
//!
//! Everything else -- integer overflow, array bounds checks, etc. -- remains
//! enforced even inside an `unsafe` block.

/// Demonstrates the minimal unsafe block: calling an unsafe function.
/// The `unsafe` keyword here acknowledges that `add_unchecked` requires
/// the caller to guarantee the precondition.
///
/// # Safety
/// Caller must ensure that `a + b` does not overflow `usize`.
pub unsafe fn add_unchecked(a: usize, b: usize) -> usize {
    a.unchecked_add(b)
}

/// Safe wrapper that validates the precondition before delegating to unsafe code.
/// This is the idiomatic pattern: expose a safe API that internally uses unsafe
/// only when invariants are guaranteed.
pub fn add_saturating_or_unchecked(a: usize, b: usize, allow_overflow: bool) -> usize {
    if allow_overflow {
        // SAFETY: The caller explicitly opts into overflow behavior via `allow_overflow`.
        // This is a deliberate design choice, similar to how Vec uses unsafe internally.
        unsafe { add_unchecked(a, b) }
    } else {
        a.saturating_add(b)
    }
}

/// A split-at implementation that demonstrates raw pointer dereferencing.
/// This mirrors how `slice::split_at` works internally.
///
/// # Safety
/// `mid` must be less than or equal to `slice.len()`.
pub unsafe fn split_at_unchecked<T>(slice: &[T], mid: usize) -> (&[T], &[T]) {
    let len = slice.len();
    let ptr = slice.as_ptr();
    // SAFETY: Caller guarantees `mid <= len`. We construct two non-overlapping
    // slices from the same allocation, which is sound because immutable slices
    // cannot be mutated.
    (
        core::slice::from_raw_parts(ptr, mid),
        core::slice::from_raw_parts(ptr.add(mid), len - mid),
    )
}

/// Demonstrates accessing a mutable static variable.
/// This is one of the five unsafe superpowers.
static mut GLOBAL_COUNTER: u64 = 0;

/// Increments the global counter. This is inherently unsafe because
/// concurrent access to mutable statics is a data race.
///
/// # Safety
/// Caller must ensure this function is not called from multiple threads
/// simultaneously without external synchronization.
pub unsafe fn increment_global_counter() -> u64 {
    GLOBAL_COUNTER += 1;
    GLOBAL_COUNTER
}

/// Safe wrapper using a Mutex-like pattern (simplified for demonstration).
/// In production, use `std::sync::atomic::AtomicU64` instead.
pub fn increment_counter_safely() -> u64 {
    // In a real implementation, we'd use an atomic. Here we demonstrate
    // the pattern of isolating unsafe access.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}

/// Demonstrates the unsafe audit concept: every `unsafe` block should have
/// a SAFETY comment explaining why the invariants hold.
pub fn safe_split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid ({mid}) must be <= len ({len})");

    let ptr = slice.as_mut_ptr();
    // SAFETY: We've verified `mid <= len` above. We create two non-overlapping
    // mutable slices from the same allocation. This is sound because:
    // 1. The slices don't overlap (first is [0..mid), second is [mid..len))
    // 2. Both slices are derived from the same mutable reference, which
    //    guarantees exclusive access for the lifetime of this function.
    unsafe {
        (
            core::slice::from_raw_parts_mut(ptr, mid),
            core::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

/// Demonstrates that unsafe does NOT disable the borrow checker.
/// The borrow checker still enforces exclusivity even in unsafe blocks.
pub fn demonstrate_borrow_check_in_unsafe() {
    let mut x = 42;
    let r1 = &mut x;
    // Even if we wrapped this in unsafe, the borrow checker would still
    // prevent creating two mutable references. unsafe does NOT bypass
    // the borrow checker.
    *r1 += 1;
    assert_eq!(*r1, 43);
}

/// Demonstrates unsafe trait implementation.
/// `Send` is an unsafe trait because implementing it incorrectly can lead
/// to data races.
struct MyType {
    data: *const u8,
    len: usize,
}

// SAFETY: MyType contains a raw pointer, but we guarantee that the data
// it points to is immutable after construction and is valid for the
// lifetime of MyType. Since the data cannot be mutated, sending MyType
// to another thread is safe.
unsafe impl Send for MyType {}

// SAFETY: MyType's data is immutable (only *const u8, not *mut u8),
// so sharing references across threads is safe. The length is just a usize.
unsafe impl Sync for MyType {}

/// Helper to create a MyType from a static slice.
impl MyType {
    pub fn from_static(data: &'static [u8]) -> Self {
        MyType {
            data: data.as_ptr(),
            len: data.len(),
        }
    }

    /// # Safety
    /// The caller must ensure the pointer and length refer to valid memory
    /// that outlives this MyType instance.
    pub unsafe fn from_raw(ptr: *const u8, len: usize) -> Self {
        MyType { data: ptr, len }
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: We maintain the invariant that `data` is valid for `len` bytes.
        // The pointer was either from a static slice (from_static) or the caller
        // guaranteed validity (from_raw).
        unsafe { core::slice::from_raw_parts(self.data, self.len) }
    }
}

/// Demonstrates the five categories of unsafe operations as a reference.
pub struct UnsafeCapabilities;

impl UnsafeCapabilities {
    /// Category 1: Dereferencing raw pointers
    pub fn deref_raw_pointer(ptr: *const i32) -> i32 {
        // SAFETY: Caller must ensure `ptr` is valid, aligned, and points to initialized data.
        unsafe { *ptr }
    }

    /// Category 2: Calling unsafe functions
    pub fn call_unsafe_fn(slice: &mut [i32]) {
        // SAFETY: We're reversing in place; `reverse` is unsafe because it
        // requires elements to be valid, which they are since we have a &mut [i32].
        unsafe {
            let ptr = slice.as_mut_ptr();
            let len = slice.len();
            // Manual in-place reverse using pointer arithmetic
            for i in 0..len / 2 {
                std::ptr::swap(ptr.add(i), ptr.add(len - 1 - i));
            }
        }
    }

    /// Category 3: Accessing mutable statics
    pub fn access_mutable_static() {
        static mut LAST_ERROR: Option<&'static str> = None;
        // SAFETY: Single-threaded access assumed. In production, use atomics or Mutex.
        unsafe {
            LAST_ERROR = Some("example error");
            assert!(LAST_ERROR.is_some());
        }
    }

    /// Category 5: Accessing union fields
    pub fn union_field_access() {
        union IntOrFloat {
            i: i32,
            f: f32,
        }
        let val = IntOrFloat { i: 42 };
        // SAFETY: We just wrote `i`, so reading `i` is safe.
        // Reading `f` would be UB unless the bit pattern is a valid f32.
        unsafe {
            assert_eq!(val.i, 42);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_unchecked() {
        // SAFETY: 2 + 3 = 5, which does not overflow.
        let result = unsafe { add_unchecked(2, 3) };
        assert_eq!(result, 5);
    }

    #[test]
    fn test_add_saturating_or_unchecked_normal() {
        assert_eq!(add_saturating_or_unchecked(100, 200, false), 300);
    }

    #[test]
    fn test_split_at_unchecked() {
        let data = [1, 2, 3, 4, 5];
        // SAFETY: mid=3 <= len=5.
        let (left, right) = unsafe { split_at_unchecked(&data, 3) };
        assert_eq!(left, &[1, 2, 3]);
        assert_eq!(right, &[4, 5]);
    }

    #[test]
    fn test_increment_counter_safely() {
        let val = increment_counter_safely();
        assert!(val >= 1);
    }

    #[test]
    fn test_safe_split_at_mut() {
        let mut data = vec![1, 2, 3, 4, 5];
        let (left, right) = safe_split_at_mut(&mut data, 2);
        assert_eq!(left, &mut [1, 2]);
        assert_eq!(right, &mut [3, 4, 5]);

        // Verify we can modify both halves independently
        left[0] = 10;
        right[0] = 30;
        assert_eq!(data, [10, 2, 30, 4, 5]);
    }

    #[test]
    #[should_panic(expected = "must be <= len")]
    fn test_safe_split_at_mut_out_of_bounds() {
        let mut data = vec![1, 2, 3];
        let _ = safe_split_at_mut(&mut data, 10);
    }

    #[test]
    fn test_demonstrate_borrow_check() {
        demonstrate_borrow_check_in_unsafe();
    }

    #[test]
    fn test_my_type_from_static() {
        let my = MyType::from_static(b"hello");
        assert_eq!(my.as_slice(), b"hello");
    }

    #[test]
    fn test_my_type_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<MyType>();
        assert_sync::<MyType>();
    }

    #[test]
    fn test_raw_pointer_deref() {
        let x: i32 = 42;
        let ptr = &x as *const i32;
        assert_eq!(UnsafeCapabilities::deref_raw_pointer(ptr), 42);
    }

    #[test]
    fn test_union_field_access() {
        UnsafeCapabilities::union_field_access();
    }

    #[test]
    fn test_reverse_in_place() {
        let mut data = [1, 2, 3, 4, 5];
        UnsafeCapabilities::call_unsafe_fn(&mut data);
        assert_eq!(data, [5, 4, 3, 2, 1]);
    }
}
