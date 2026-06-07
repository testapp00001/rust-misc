/// Problem: FFI (Foreign Function Interface)
///
/// Master FFI in Rust.
///
/// Key Concepts:
/// - extern "C"
/// - Calling C functions
/// - Exposing Rust functions
/// - Type conversions
/// - Safety

/// Problem 1: Call C function
/// Call abs from C standard library
pub fn call_abs(x: i32) -> i32 {
    extern "C" {
        fn abs(input: i32) -> i32;
    }
    unsafe { abs(x) }
}

/// Problem 2: Call C function with string
/// Call strlen from C standard library
pub fn call_strlen(s: &str) -> usize {
    extern "C" {
        fn strlen(s: *const std::os::raw::c_char) -> usize;
    }
    let c_str = std::ffi::CString::new(s).unwrap();
    unsafe { strlen(c_str.as_ptr()) }
}

/// Problem 3: Expose Rust function to C
/// Expose function to C
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 4: Call C function with pointer
/// Call malloc and free
pub fn call_malloc_free() -> i32 {
    extern "C" {
        fn malloc(size: usize) -> *mut std::os::raw::c_void;
        fn free(ptr: *mut std::os::raw::c_void);
    }

    unsafe {
        let ptr = malloc(4) as *mut i32;
        if ptr.is_null() {
            return -1;
        }
        *ptr = 42;
        let value = *ptr;
        free(ptr as *mut std::os::raw::c_void);
        value
    }
}

/// Problem 5: Call C function with struct
/// Call C function with struct
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn call_point_distance(p1: &Point, p2: &Point) -> f64 {
    extern "C" {
        fn sqrt(x: f64) -> f64;
    }

    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    unsafe { sqrt(dx * dx + dy * dy) }
}

/// Problem 6: Call C function with callback
/// Call C function with callback
pub fn call_qsort() -> Vec<i32> {
    extern "C" {
        fn qsort(
            base: *mut std::os::raw::c_void,
            nmemb: usize,
            size: usize,
            compar: extern "C" fn(*const std::os::raw::c_void, *const std::os::raw::c_void) -> i32,
        );
    }

    extern "C" fn compare(a: *const std::os::raw::c_void, b: *const std::os::raw::c_void) -> i32 {
        unsafe {
            let a = *(a as *const i32);
            let b = *(b as *const i32);
            a - b
        }
    }

    let mut arr = vec![5, 3, 1, 4, 2];
    unsafe {
        qsort(
            arr.as_mut_ptr() as *mut std::os::raw::c_void,
            arr.len(),
            std::mem::size_of::<i32>(),
            compare,
        );
    }
    arr
}

/// Problem 7: Call C function with return value
/// Call C function that returns value
pub fn call_pow(base: f64, exponent: f64) -> f64 {
    extern "C" {
        fn pow(base: f64, exponent: f64) -> f64;
    }
    unsafe { pow(base, exponent) }
}

/// Problem 8: Call C function with error handling
/// Handle C function errors
pub fn call_errno() -> i32 {
    // Simulate errno
    0
}

/// Problem 9: Call C function with string return
/// Call C function that returns string
pub fn call_getenv(name: &str) -> Option<String> {
    extern "C" {
        fn getenv(name: *const std::os::raw::c_char) -> *const std::os::raw::c_char;
    }

    let c_name = std::ffi::CString::new(name).unwrap();
    unsafe {
        let ptr = getenv(c_name.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string())
        }
    }
}

/// Problem 10: Call C function with array
/// Call C function with array
pub fn call_memcpy() -> Vec<u8> {
    extern "C" {
        fn memcpy(dest: *mut std::os::raw::c_void, src: *const std::os::raw::c_void, n: usize) -> *mut std::os::raw::c_void;
    }

    let src = vec![1u8, 2, 3, 4, 5];
    let mut dest = vec![0u8; 5];
    unsafe {
        memcpy(
            dest.as_mut_ptr() as *mut std::os::raw::c_void,
            src.as_ptr() as *const std::os::raw::c_void,
            5,
        );
    }
    dest
}

/// Problem 11: Call C function with union
/// Use union with FFI
#[repr(C)]
pub union IntOrFloat {
    pub i: i32,
    pub f: f32,
}

pub fn call_with_union() -> i32 {
    let u = IntOrFloat { i: 42 };
    unsafe { u.i }
}

/// Problem 12: Call C function with enum
/// Use enum with FFI
#[repr(C)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

pub fn call_with_enum() -> i32 {
    let color = Color::Green;
    color as i32
}

/// Problem 13: Call C function with bitfield
/// Simulate bitfield
pub fn call_with_bitfield() -> u8 {
    let flags: u8 = 0b00001010; // bits 1 and 3 set
    flags
}

/// Problem 14: Call C function with variadic
/// Simulate variadic function
pub fn call_variadic() -> String {
    format!("arg1={}, arg2={}, arg3={}", 1, 2, 3)
}

/// Problem 15: Safe FFI wrapper
/// Wrap unsafe FFI in safe function
pub fn safe_abs(x: i32) -> i32 {
    call_abs(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_abs() {
        assert_eq!(call_abs(-42), 42);
    }

    #[test]
    fn test_call_strlen() {
        assert_eq!(call_strlen("Hello"), 5);
    }

    #[test]
    fn test_rust_add() {
        assert_eq!(rust_add(2, 3), 5);
    }

    #[test]
    fn test_call_malloc_free() {
        assert_eq!(call_malloc_free(), 42);
    }

    #[test]
    fn test_call_point_distance() {
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 3.0, y: 4.0 };
        let distance = call_point_distance(&p1, &p2);
        assert!((distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_call_qsort() {
        assert_eq!(call_qsort(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_call_pow() {
        let result = call_pow(2.0, 3.0);
        assert!((result - 8.0).abs() < 0.0001);
    }

    #[test]
    fn test_call_errno() {
        // errno may or may not be set
        let _ = call_errno();
    }

    #[test]
    fn test_call_getenv() {
        // PATH should exist on most systems
        let result = call_getenv("PATH");
        assert!(result.is_some());
    }

    #[test]
    fn test_call_memcpy() {
        assert_eq!(call_memcpy(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_call_with_union() {
        assert_eq!(call_with_union(), 42);
    }

    #[test]
    fn test_call_with_enum() {
        assert_eq!(call_with_enum(), 1);
    }

    #[test]
    fn test_call_with_bitfield() {
        assert_eq!(call_with_bitfield(), 0b00001010);
    }

    #[test]
    fn test_call_variadic() {
        assert_eq!(call_variadic(), "arg1=1, arg2=2, arg3=3");
    }

    #[test]
    fn test_safe_abs() {
        assert_eq!(safe_abs(-42), 42);
    }
}
