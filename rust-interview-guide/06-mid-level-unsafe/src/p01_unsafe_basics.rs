/// Problem: Unsafe Basics
///
/// Master Rust's unsafe blocks.
///
/// Key Concepts:
/// - Unsafe blocks
/// - Unsafe operations
/// - Safe abstractions
/// - Safety invariants
/// - Documentation

/// Problem 1: Basic unsafe block
/// Use unsafe block
pub fn basic_unsafe() -> i32 {
    let mut x = 42;
    unsafe {
        let raw = &mut x as *mut i32;
        *raw = 100;
    }
    x
}

/// Problem 2: Unsafe with raw pointer
/// Create and use raw pointer
pub fn unsafe_raw_pointer() -> i32 {
    let x = 42;
    let raw = &x as *const i32;
    unsafe { *raw }
}

/// Problem 3: Unsafe with mutable raw pointer
/// Use mutable raw pointer
pub fn unsafe_mutable_raw_pointer() -> i32 {
    let mut x = 42;
    let raw = &mut x as *mut i32;
    unsafe {
        *raw = 100;
    }
    x
}

/// Problem 4: Unsafe with null pointer
/// Check for null pointer
pub fn unsafe_null_pointer() -> bool {
    let raw: *const i32 = std::ptr::null();
    raw.is_null()
}

/// Problem 5: Unsafe with pointer arithmetic
/// Use pointer arithmetic
pub fn unsafe_pointer_arithmetic() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    unsafe { *ptr.add(2) } // arr[2]
}

/// Problem 6: Unsafe with transmute
/// Use transmute (dangerous)
pub fn unsafe_transmute() -> u32 {
    let x: i32 = -1;
    unsafe { std::mem::transmute(x) }
}

/// Problem 7: Unsafe with uninitialized memory
/// Use uninitialized memory (dangerous)
pub fn unsafe_uninitialized() -> i32 {
    unsafe {
        let x: i32 = std::mem::zeroed();
        x
    }
}

/// Problem 8: Unsafe with static mut
/// Use static mutable variable
pub fn unsafe_static_mut() -> i32 {
    static mut COUNTER: i32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

/// Problem 9: Unsafe with union
/// Use union
pub union MyUnion {
    pub i: i32,
    pub f: f32,
}

pub fn unsafe_union() -> i32 {
    let u = MyUnion { i: 42 };
    unsafe { u.i }
}

/// Problem 10: Unsafe with function pointer
/// Call function pointer
pub fn unsafe_function_pointer() -> i32 {
    let f: fn(i32) -> i32 = |x| x * 2;
    f(21)
}

/// Problem 11: Unsafe with slice
/// Create slice from raw pointer
pub fn unsafe_slice() -> Vec<i32> {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    unsafe { std::slice::from_raw_parts(ptr, 5).to_vec() }
}

/// Problem 12: Unsafe with string
/// Create string from raw pointer
pub fn unsafe_string() -> String {
    let s = String::from("Hello, World!");
    let ptr = s.as_ptr();
    let len = s.len();
    let capacity = s.capacity();
    std::mem::forget(s);
    unsafe { String::from_raw_parts(ptr as *mut u8, len, capacity) }
}

/// Problem 13: Unsafe with Box
/// Use Box with raw pointer
pub fn unsafe_box() -> i32 {
    let b = Box::new(42);
    let raw = Box::into_raw(b);
    unsafe {
        let b = Box::from_raw(raw);
        *b
    }
}

/// Problem 14: Unsafe with Vec
/// Use Vec with raw pointer
pub fn unsafe_vec() -> Vec<i32> {
    let v = vec![1, 2, 3, 4, 5];
    let mut v = std::mem::ManuallyDrop::new(v);
    let ptr = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

/// Problem 15: Safe abstraction over unsafe
/// Wrap unsafe code in safe function
pub fn safe_abstraction(data: &mut [i32], index: usize, value: i32) -> bool {
    if index < data.len() {
        unsafe {
            let ptr = data.as_mut_ptr().add(index);
            *ptr = value;
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_unsafe() {
        assert_eq!(basic_unsafe(), 100);
    }

    #[test]
    fn test_unsafe_raw_pointer() {
        assert_eq!(unsafe_raw_pointer(), 42);
    }

    #[test]
    fn test_unsafe_mutable_raw_pointer() {
        assert_eq!(unsafe_mutable_raw_pointer(), 100);
    }

    #[test]
    fn test_unsafe_null_pointer() {
        assert!(unsafe_null_pointer());
    }

    #[test]
    fn test_unsafe_pointer_arithmetic() {
        assert_eq!(unsafe_pointer_arithmetic(), 3);
    }

    #[test]
    fn test_unsafe_transmute() {
        let result = unsafe_transmute();
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn test_unsafe_uninitialized() {
        assert_eq!(unsafe_uninitialized(), 0);
    }

    #[test]
    fn test_unsafe_static_mut() {
        assert_eq!(unsafe_static_mut(), 1);
    }

    #[test]
    fn test_unsafe_union() {
        assert_eq!(unsafe_union(), 42);
    }

    #[test]
    fn test_unsafe_function_pointer() {
        assert_eq!(unsafe_function_pointer(), 42);
    }

    #[test]
    fn test_unsafe_slice() {
        assert_eq!(unsafe_slice(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_unsafe_string() {
        assert_eq!(unsafe_string(), "Hello, World!");
    }

    #[test]
    fn test_unsafe_box() {
        assert_eq!(unsafe_box(), 42);
    }

    #[test]
    fn test_unsafe_vec() {
        assert_eq!(unsafe_vec(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_safe_abstraction() {
        let mut data = vec![1, 2, 3, 4, 5];
        assert!(safe_abstraction(&mut data, 2, 100));
        assert_eq!(data[2], 100);
    }
}
