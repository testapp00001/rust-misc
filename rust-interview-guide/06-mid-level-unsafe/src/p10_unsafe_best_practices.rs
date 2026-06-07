/// Problem: Unsafe Best Practices
///
/// Master best practices for unsafe code.
///
/// Key Concepts:
/// - Minimize unsafe code
/// - Document safety invariants
/// - Use safe abstractions
/// - Test thoroughly
/// - Review carefully

/// Problem 1: Minimize unsafe surface
/// Keep unsafe code minimal
pub fn safe_function() -> i32 {
    // Safe code
    42
}

/// Problem 2: Document safety invariants
/// Document why unsafe code is safe
/// SAFETY: The pointer is valid because it was created from a Box
pub fn documented_unsafe() -> i32 {
    let b = Box::new(42);
    let ptr = Box::into_raw(b);
    // SAFETY: ptr is valid because it was created from Box
    unsafe {
        let value = *ptr;
        let _ = Box::from_raw(ptr);
        value
    }
}

/// Problem 3: Use safe wrappers
/// Wrap unsafe code in safe functions
pub fn safe_wrapper() -> i32 {
    let mut x = 0;
    // SAFETY: We're writing to a valid mutable reference
    unsafe {
        let ptr = &mut x as *mut i32;
        *ptr = 42;
    }
    x
}

/// Problem 4: Validate inputs
/// Validate before unsafe operations
pub fn validated_unsafe(ptr: *const i32) -> Option<i32> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: We checked that ptr is not null
    Some(unsafe { *ptr })
}

/// Problem 5: Use RAII
/// Use RAII for resource management
pub struct Resource {
    data: *mut i32,
}

impl Resource {
    pub fn new(value: i32) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Self { data: ptr }
    }

    pub fn get(&self) -> i32 {
        // SAFETY: data is valid because we own it
        unsafe { *self.data }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        // SAFETY: data is valid and we're the only owner
        unsafe {
            let _ = Box::from_raw(self.data);
        }
    }
}

/// Problem 6: Use type system
/// Use type system for safety
pub struct SafeType {
    value: i32,
}

impl SafeType {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn get(&self) -> i32 {
        self.value
    }
}

/// Problem 7: Use bounds checking
/// Check bounds before unsafe access
pub fn bounded_unsafe(arr: &[i32], index: usize) -> Option<i32> {
    if index >= arr.len() {
        return None;
    }
    // SAFETY: We checked that index is within bounds
    Some(unsafe { *arr.get_unchecked(index) })
}

/// Problem 8: Use error handling
/// Handle errors in unsafe code
pub fn error_handling_unsafe(ptr: *const i32) -> Result<i32, String> {
    if ptr.is_null() {
        return Err("Null pointer".to_string());
    }
    // SAFETY: We checked that ptr is not null
    Ok(unsafe { *ptr })
}

/// Problem 9: Use testing
/// Test unsafe code thoroughly
pub fn tested_unsafe(x: i32) -> i32 {
    // SAFETY: This is a simple arithmetic operation
    unsafe {
        let ptr = &x as *const i32;
        *ptr
    }
}

/// Problem 10: Use code review
/// Review unsafe code carefully
pub fn reviewed_unsafe() -> i32 {
    // Code review checklist:
    // 1. Is unsafe necessary?
    // 2. Are invariants documented?
    // 3. Are inputs validated?
    // 4. Is resource management correct?
    // 5. Are there tests?
    42
}

/// Problem 11: Use safe abstractions
/// Create safe abstractions over unsafe code
pub fn safe_abstraction() -> i32 {
    let mut v = Vec::new();
    v.push(42);
    v[0]
}

/// Problem 12: Use standard library
/// Prefer standard library over unsafe
pub fn use_std() -> i32 {
    let v = vec![1, 2, 3, 4, 5];
    v.iter().sum()
}

/// Problem 13: Use third-party crates
/// Prefer third-party crates over unsafe
pub fn use_third_party() -> i32 {
    // In real implementation, you'd use crates like rayon, serde, etc.
    42
}

/// Problem 14: Use unsafe only when necessary
/// Use unsafe only for performance-critical code
pub fn performance_critical(data: &[i32]) -> i32 {
    // Safe version
    data.iter().sum()
}

/// Problem 15: Use unsafe with justification
/// Justify every unsafe block
pub fn justified_unsafe() -> i32 {
    // Justification: We need to use raw pointers for FFI
    let x = 42;
    let ptr = &x as *const i32;
    // SAFETY: ptr is valid because it was created from a reference
    unsafe { *ptr }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_function() {
        assert_eq!(safe_function(), 42);
    }

    #[test]
    fn test_documented_unsafe() {
        assert_eq!(documented_unsafe(), 42);
    }

    #[test]
    fn test_safe_wrapper() {
        assert_eq!(safe_wrapper(), 42);
    }

    #[test]
    fn test_validated_unsafe() {
        let x = 42;
        let ptr = &x as *const i32;
        assert_eq!(validated_unsafe(ptr), Some(42));
        assert_eq!(validated_unsafe(std::ptr::null()), None);
    }

    #[test]
    fn test_resource() {
        let r = Resource::new(42);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_safe_type() {
        let s = SafeType::new(42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_bounded_unsafe() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(bounded_unsafe(&arr, 2), Some(3));
        assert_eq!(bounded_unsafe(&arr, 10), None);
    }

    #[test]
    fn test_error_handling_unsafe() {
        let x = 42;
        let ptr = &x as *const i32;
        assert_eq!(error_handling_unsafe(ptr), Ok(42));
        assert!(error_handling_unsafe(std::ptr::null()).is_err());
    }

    #[test]
    fn test_tested_unsafe() {
        assert_eq!(tested_unsafe(42), 42);
    }

    #[test]
    fn test_reviewed_unsafe() {
        assert_eq!(reviewed_unsafe(), 42);
    }

    #[test]
    fn test_safe_abstraction() {
        assert_eq!(safe_abstraction(), 42);
    }

    #[test]
    fn test_use_std() {
        assert_eq!(use_std(), 15);
    }

    #[test]
    fn test_use_third_party() {
        assert_eq!(use_third_party(), 42);
    }

    #[test]
    fn test_performance_critical() {
        assert_eq!(performance_critical(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_justified_unsafe() {
        assert_eq!(justified_unsafe(), 42);
    }
}
