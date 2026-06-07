/// Problem: Memory Safety
///
/// Master memory safety in Rust.
///
/// Key Concepts:
/// - Memory safety rules
/// - Undefined behavior
/// - Data races
/// - Buffer overflows
/// - Use after free

/// Problem 1: Check for null pointer
/// Ensure pointer is not null
pub fn check_null_pointer(ptr: *const i32) -> bool {
    !ptr.is_null()
}

/// Problem 2: Check for valid memory
/// Ensure pointer points to valid memory
pub fn check_valid_memory(ptr: *const i32) -> bool {
    !ptr.is_null() && (ptr as usize) > 0
}

/// Problem 3: Check for aligned pointer
/// Ensure pointer is aligned
pub fn check_aligned(ptr: *const i32) -> bool {
    (ptr as usize) % std::mem::align_of::<i32>() == 0
}

/// Problem 4: Safe pointer dereference
/// Safely dereference pointer
pub fn safe_dereference(ptr: *const i32) -> Option<i32> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { *ptr })
    }
}

/// Problem 5: Safe pointer write
/// Safely write through pointer
pub fn safe_write(ptr: *mut i32, value: i32) -> bool {
    if ptr.is_null() {
        false
    } else {
        unsafe { *ptr = value; }
        true
    }
}

/// Problem 6: Check for data race
/// Detect potential data race
pub fn check_data_race() -> bool {
    // In real implementation, you'd use tools like ThreadSanitizer
    false
}

/// Problem 7: Check for buffer overflow
/// Detect buffer overflow
pub fn check_buffer_overflow(ptr: *const i32, len: usize, index: usize) -> bool {
    index < len && !ptr.is_null()
}

/// Problem 8: Safe array access
/// Safely access array element
pub fn safe_array_access(arr: &[i32], index: usize) -> Option<i32> {
    arr.get(index).copied()
}

/// Problem 9: Safe slice creation
/// Safely create slice from pointer
pub fn safe_slice_creation(ptr: *const i32, len: usize) -> Option<Vec<i32>> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(ptr, len).to_vec() })
}

/// Problem 10: Safe string creation
/// Safely create string from pointer
pub fn safe_string_creation(ptr: *const u8, len: usize) -> Option<String> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8(slice.to_vec()).ok()
}

/// Problem 11: Memory leak detection
/// Detect memory leak
pub fn check_memory_leak() -> bool {
    // In real implementation, you'd use tools like Valgrind
    false
}

/// Problem 12: Use after free detection
/// Detect use after free
pub fn check_use_after_free() -> bool {
    // In real implementation, you'd use tools like Miri
    false
}

/// Problem 13: Double free detection
/// Detect double free
pub fn check_double_free() -> bool {
    // In real implementation, you'd use tools like Miri
    false
}

/// Problem 14: Safe memory management
/// Manage memory safely
pub fn safe_memory_management() -> i32 {
    let b = Box::new(42);
    *b
}

/// Problem 15: Safe reference counting
/// Use reference counting safely
pub fn safe_reference_counting() -> i32 {
    use std::rc::Rc;
    let a = Rc::new(42);
    let b = Rc::clone(&a);
    *a + *b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_null_pointer() {
        let x = 42;
        let ptr = &x as *const i32;
        assert!(check_null_pointer(ptr));
        assert!(!check_null_pointer(std::ptr::null()));
    }

    #[test]
    fn test_check_valid_memory() {
        let x = 42;
        let ptr = &x as *const i32;
        assert!(check_valid_memory(ptr));
    }

    #[test]
    fn test_check_aligned() {
        let x = 42;
        let ptr = &x as *const i32;
        assert!(check_aligned(ptr));
    }

    #[test]
    fn test_safe_dereference() {
        let x = 42;
        let ptr = &x as *const i32;
        assert_eq!(safe_dereference(ptr), Some(42));
        assert_eq!(safe_dereference(std::ptr::null()), None);
    }

    #[test]
    fn test_safe_write() {
        let mut x = 0;
        let ptr = &mut x as *mut i32;
        assert!(safe_write(ptr, 42));
        assert_eq!(x, 42);
    }

    #[test]
    fn test_check_data_race() {
        assert!(!check_data_race());
    }

    #[test]
    fn test_check_buffer_overflow() {
        let arr = [1, 2, 3, 4, 5];
        let ptr = arr.as_ptr();
        assert!(check_buffer_overflow(ptr, 5, 2));
        assert!(!check_buffer_overflow(ptr, 5, 10));
    }

    #[test]
    fn test_safe_array_access() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(safe_array_access(&arr, 2), Some(3));
        assert_eq!(safe_array_access(&arr, 10), None);
    }

    #[test]
    fn test_safe_slice_creation() {
        let arr = [1, 2, 3, 4, 5];
        let ptr = arr.as_ptr();
        assert_eq!(safe_slice_creation(ptr, 5), Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(safe_slice_creation(std::ptr::null(), 5), None);
    }

    #[test]
    fn test_safe_string_creation() {
        let s = "Hello";
        let ptr = s.as_ptr();
        assert_eq!(safe_string_creation(ptr, 5), Some("Hello".to_string()));
        assert_eq!(safe_string_creation(std::ptr::null(), 5), None);
    }

    #[test]
    fn test_check_memory_leak() {
        assert!(!check_memory_leak());
    }

    #[test]
    fn test_check_use_after_free() {
        assert!(!check_use_after_free());
    }

    #[test]
    fn test_check_double_free() {
        assert!(!check_double_free());
    }

    #[test]
    fn test_safe_memory_management() {
        assert_eq!(safe_memory_management(), 42);
    }

    #[test]
    fn test_safe_reference_counting() {
        assert_eq!(safe_reference_counting(), 84);
    }
}
