/// Problem: Raw Pointers
///
/// Master raw pointers in Rust.
///
/// Key Concepts:
/// - Raw pointer creation
/// - Raw pointer dereferencing
/// - Pointer arithmetic
/// - Null pointers
/// - Pointer casting

/// Problem 1: Create raw pointer
/// Create raw pointer from reference
pub fn create_raw_pointer() -> *const i32 {
    let x = 42;
    &x as *const i32
}

/// Problem 2: Create mutable raw pointer
/// Create mutable raw pointer
pub fn create_mutable_raw_pointer() -> *mut i32 {
    let mut x = 42;
    &mut x as *mut i32
}

/// Problem 3: Dereference raw pointer
/// Dereference raw pointer
pub fn dereference_raw_pointer() -> i32 {
    let x = 42;
    let raw = &x as *const i32;
    unsafe { *raw }
}

/// Problem 4: Dereference mutable raw pointer
/// Dereference mutable raw pointer
pub fn dereference_mutable_raw_pointer() -> i32 {
    let mut x = 42;
    let raw = &mut x as *mut i32;
    unsafe {
        *raw = 100;
        *raw
    }
}

/// Problem 5: Null pointer
/// Create and check null pointer
pub fn null_pointer() -> bool {
    let raw: *const i32 = std::ptr::null();
    raw.is_null()
}

/// Problem 6: Pointer arithmetic
/// Use pointer arithmetic
pub fn pointer_arithmetic() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    unsafe { *ptr.add(2) }
}

/// Problem 7: Pointer offset
/// Use pointer offset
pub fn pointer_offset() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    unsafe { *ptr.offset(3) }
}

/// Problem 8: Pointer casting
/// Cast pointer types
pub fn pointer_casting() -> f32 {
    let x: i32 = 42;
    let raw = &x as *const i32 as *const f32;
    unsafe { *raw }
}

/// Problem 9: Pointer to usize
/// Convert pointer to usize
pub fn pointer_to_usize() -> usize {
    let x = 42;
    let raw = &x as *const i32;
    raw as usize
}

/// Problem 10: usize to pointer
/// Convert usize to pointer
pub fn usize_to_pointer() -> i32 {
    let x = 42;
    let addr = &x as *const i32 as usize;
    let raw = addr as *const i32;
    unsafe { *raw }
}

/// Problem 11: Pointer comparison
/// Compare pointers
pub fn pointer_comparison() -> bool {
    let x = 42;
    let raw1 = &x as *const i32;
    let raw2 = &x as *const i32;
    raw1 == raw2
}

/// Problem 12: Pointer copy
/// Copy data through pointers
pub fn pointer_copy() -> i32 {
    let x = 42;
    let mut y = 0;
    let src = &x as *const i32;
    let dst = &mut y as *mut i32;
    unsafe {
        std::ptr::copy(src, dst, 1);
    }
    y
}

/// Problem 13: Pointer write
/// Write through pointer
pub fn pointer_write() -> i32 {
    let mut x = 0;
    let raw = &mut x as *mut i32;
    unsafe {
        std::ptr::write(raw, 42);
    }
    x
}

/// Problem 14: Pointer read
/// Read through pointer
pub fn pointer_read() -> i32 {
    let x = 42;
    let raw = &x as *const i32;
    unsafe { std::ptr::read(raw) }
}

/// Problem 15: Pointer swap
/// Swap through pointers
pub fn pointer_swap() -> (i32, i32) {
    let mut x = 1;
    let mut y = 2;
    let raw_x = &mut x as *mut i32;
    let raw_y = &mut y as *mut i32;
    unsafe {
        std::ptr::swap(raw_x, raw_y);
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_raw_pointer() {
        let raw = create_raw_pointer();
        assert!(!raw.is_null());
    }

    #[test]
    fn test_create_mutable_raw_pointer() {
        let raw = create_mutable_raw_pointer();
        assert!(!raw.is_null());
    }

    #[test]
    fn test_dereference_raw_pointer() {
        assert_eq!(dereference_raw_pointer(), 42);
    }

    #[test]
    fn test_dereference_mutable_raw_pointer() {
        assert_eq!(dereference_mutable_raw_pointer(), 100);
    }

    #[test]
    fn test_null_pointer() {
        assert!(null_pointer());
    }

    #[test]
    fn test_pointer_arithmetic() {
        assert_eq!(pointer_arithmetic(), 3);
    }

    #[test]
    fn test_pointer_offset() {
        assert_eq!(pointer_offset(), 4);
    }

    #[test]
    fn test_pointer_casting() {
        let result = pointer_casting();
        assert!(result.is_finite());
    }

    #[test]
    fn test_pointer_to_usize() {
        let addr = pointer_to_usize();
        assert!(addr > 0);
    }

    #[test]
    fn test_usize_to_pointer() {
        assert_eq!(usize_to_pointer(), 42);
    }

    #[test]
    fn test_pointer_comparison() {
        assert!(pointer_comparison());
    }

    #[test]
    fn test_pointer_copy() {
        assert_eq!(pointer_copy(), 42);
    }

    #[test]
    fn test_pointer_write() {
        assert_eq!(pointer_write(), 42);
    }

    #[test]
    fn test_pointer_read() {
        assert_eq!(pointer_read(), 42);
    }

    #[test]
    fn test_pointer_swap() {
        assert_eq!(pointer_swap(), (2, 1));
    }
}
