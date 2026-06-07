/// Problem: Unsafe Testing
///
/// Master testing unsafe code in Rust.
///
/// Key Concepts:
/// - Testing unsafe code
/// - Miri for UB detection
/// - Fuzzing
/// - Property-based testing
/// - Stress testing

/// Problem 1: Test unsafe function
/// Test unsafe function safely
pub unsafe fn unsafe_add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 2: Test unsafe struct
/// Test unsafe struct
pub struct UnsafeStruct {
    data: *mut i32,
}

impl UnsafeStruct {
    pub fn new(value: i32) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Self { data: ptr }
    }

    pub unsafe fn get(&self) -> i32 {
        *self.data
    }

    pub unsafe fn set(&mut self, value: i32) {
        *self.data = value;
    }
}

impl Drop for UnsafeStruct {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.data);
        }
    }
}

/// Problem 3: Test unsafe trait
/// Test unsafe trait
unsafe trait UnsafeTrait {
    fn unsafe_method(&self) -> i32;
}

struct TraitImpl {
    value: i32,
}

unsafe impl UnsafeTrait for TraitImpl {
    fn unsafe_method(&self) -> i32 {
        self.value
    }
}

/// Problem 4: Test unsafe with property
/// Property-based testing
pub fn property_test_add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 5: Test unsafe with fuzzing
/// Fuzz testing
pub fn fuzz_target(input: &[u8]) -> usize {
    input.len()
}

/// Problem 6: Test unsafe with stress
/// Stress testing
pub fn stress_test() -> i32 {
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }
    sum
}

/// Problem 7: Test unsafe with memory leak
/// Test for memory leaks
pub fn test_memory_leak() -> i32 {
    let b = Box::new(42);
    *b
}

/// Problem 8: Test unsafe with data race
/// Test for data races
pub fn test_data_race() -> i32 {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *counter.lock().unwrap();
    result
}

/// Problem 9: Test unsafe with undefined behavior
/// Test for undefined behavior
pub fn test_ub_detection() -> i32 {
    let x = 42;
    let raw = &x as *const i32;
    unsafe { *raw }
}

/// Problem 10: Test unsafe with invalid pointer
/// Test with invalid pointer
pub fn test_invalid_pointer() -> bool {
    let raw: *const i32 = std::ptr::null();
    raw.is_null()
}

/// Problem 11: Test unsafe with out of bounds
/// Test with out of bounds access
pub fn test_out_of_bounds() -> Option<i32> {
    let arr = [1, 2, 3, 4, 5];
    arr.get(10).copied()
}

/// Problem 12: Test unsafe with transmute
/// Test transmute
pub fn test_transmute() -> u32 {
    let x: i32 = -1;
    unsafe { std::mem::transmute(x) }
}

/// Problem 13: Test unsafe with uninitialized
/// Test uninitialized memory
pub fn test_uninitialized() -> i32 {
    unsafe {
        let x: i32 = std::mem::zeroed();
        x
    }
}

/// Problem 14: Test unsafe with double free
/// Test double free detection
pub fn test_double_free() -> i32 {
    let b = Box::new(42);
    *b
}

/// Problem 15: Test unsafe with use after free
/// Test use after free detection
pub fn test_use_after_free() -> i32 {
    let b = Box::new(42);
    let value = *b;
    drop(b);
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe_add() {
        assert_eq!(unsafe { unsafe_add(20, 22) }, 42);
    }

    #[test]
    fn test_unsafe_struct() {
        let mut s = UnsafeStruct::new(42);
        assert_eq!(unsafe { s.get() }, 42);
        unsafe { s.set(100); }
        assert_eq!(unsafe { s.get() }, 100);
    }

    #[test]
    fn test_unsafe_trait() {
        let s = TraitImpl { value: 42 };
        assert_eq!(s.unsafe_method(), 42);
    }

    #[test]
    fn test_property_test_add() {
        assert_eq!(property_test_add(20, 22), 42);
        assert_eq!(property_test_add(0, 0), 0);
        assert_eq!(property_test_add(-1, 1), 0);
    }

    #[test]
    fn test_fuzz_target() {
        assert_eq!(fuzz_target(b"hello"), 5);
        assert_eq!(fuzz_target(b""), 0);
    }

    #[test]
    fn test_stress_test() {
        assert_eq!(stress_test(), 499500);
    }

    #[test]
    fn test_memory_leak_fn() {
        assert_eq!(test_memory_leak(), 42);
    }

    #[test]
    fn test_data_race_fn() {
        assert_eq!(test_data_race(), 10);
    }

    #[test]
    fn test_ub_detection_fn() {
        assert_eq!(test_ub_detection(), 42);
    }

    #[test]
    fn test_invalid_pointer_fn() {
        assert!(test_invalid_pointer());
    }

    #[test]
    fn test_out_of_bounds_fn() {
        assert_eq!(test_out_of_bounds(), None);
    }

    #[test]
    fn test_transmute_fn() {
        assert_eq!(test_transmute(), u32::MAX);
    }

    #[test]
    fn test_uninitialized_fn() {
        assert_eq!(test_uninitialized(), 0);
    }

    #[test]
    fn test_double_free_fn() {
        assert_eq!(test_double_free(), 42);
    }

    #[test]
    fn test_use_after_free_fn() {
        assert_eq!(test_use_after_free(), 42);
    }
}
