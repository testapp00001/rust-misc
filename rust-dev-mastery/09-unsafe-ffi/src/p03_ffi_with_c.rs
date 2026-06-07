//! # FFI with C: Calling C Functions from Rust
//!
//! Foreign Function Interface (FFI) allows Rust to interoperate with C code.
//! All FFI calls are inherently `unsafe` because Rust cannot verify the
//! invariants of code written in another language.
//!
//! Key concepts:
//! - `extern "C"` declares the ABI convention
//! - `#[repr(C)]` ensures Rust types have C-compatible layout
//! - Strings must be converted to null-terminated C strings (`CString`/`CStr`)
//! - Callbacks use function pointers with `extern "C"` ABI
//!
//! This module demonstrates FFI patterns using libc functions as examples.

use libc::{self, c_char, c_int, c_void, size_t};
use std::ffi::{CStr, CString};
use std::ptr;

/// Wraps the C `strlen` function safely.
/// This demonstrates the FFI pattern: unsafe C call wrapped in a safe Rust function.
pub fn safe_strlen(s: &str) -> usize {
    let c_string = CString::new(s).expect("string must not contain null bytes");
    // SAFETY: CString guarantees a null-terminated, valid C string.
    // `strlen` only reads the string and does not modify it.
    unsafe { libc::strlen(c_string.as_ptr()) }
}

/// Wraps the C `strncmp` function for comparing C strings.
///
/// # Safety
/// Both pointers must point to valid, null-terminated C strings.
pub unsafe fn c_strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    libc::strncmp(s1, s2, n as size_t)
}

/// Demonstrates passing Rust strings to C and reading the result.
pub fn c_toupper(s: &str) -> String {
    let c_string = CString::new(s).expect("string must not contain null bytes");
    let mut result = Vec::with_capacity(s.len());

    for &byte in c_string.to_bytes() {
        // SAFETY: libc::toupper expects an int; bytes 0-127 are safe.
        // We cast the result back to u8 (toupper returns int for EOF handling).
        let upper = unsafe { libc::toupper(byte as c_int) } as u8;
        result.push(upper);
    }

    String::from_utf8(result).expect("toupper should produce valid UTF-8 for ASCII input")
}

/// A C-compatible structure for passing data to/from C code.
/// `#[repr(C)]` ensures the fields are laid out in memory exactly as C would.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A C-compatible structure with nested types.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

/// C-compatible enum using `#[repr(C)]`.
/// The discriminant values are explicit to match the C side.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = 0,
    ErrorInvalidInput = 1,
    ErrorOverflow = 2,
    ErrorUnknown = 99,
}

impl Status {
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Status::Ok),
            1 => Some(Status::ErrorInvalidInput),
            2 => Some(Status::ErrorOverflow),
            99 => Some(Status::ErrorUnknown),
            _ => None,
        }
    }
}

/// Compute the Euclidean distance between two points.
/// This is a pure Rust function that could be exported to C via
/// `#[no_mangle] pub extern "C" fn point_distance(...)`.
pub fn point_distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

/// Demonstrates calling a C function that takes a callback.
/// In C, this would be something like:
/// ```c
/// void qsort(void *base, size_t nmemb, size_t size,
///             int (*compar)(const void *, const void *));
/// ```
pub fn sort_with_c_qsort(data: &mut [i32]) {
    // The callback must be an `extern "C"` function.
    extern "C" fn compare(a: *const c_void, b: *const c_void) -> c_int {
        // SAFETY: qsort guarantees that a and b point to elements of the array.
        // We cast them to i32 pointers and dereference.
        unsafe {
            let a_val = *(a as *const i32);
            let b_val = *(b as *const i32);
            a_val.cmp(&b_val) as c_int
        }
    }

    // SAFETY: data.as_mut_ptr() is valid for data.len() elements.
    // Each element is size_of::<i32>() bytes.
    // The compare function correctly compares two i32 values.
    unsafe {
        libc::qsort(
            data.as_mut_ptr() as *mut c_void,
            data.len(),
            std::mem::size_of::<i32>(),
            Some(compare),
        );
    }
}

/// Demonstrates the C string conversion pattern for function arguments.
pub struct CArgv {
    c_strings: Vec<CString>,
    ptrs: Vec<*const c_char>,
}

impl CArgv {
    /// Create a C-compatible argv array from Rust strings.
    pub fn new(args: &[&str]) -> Self {
        let c_strings: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s).expect("no null bytes in args"))
            .collect();

        let ptrs: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();

        CArgv { c_strings, ptrs }
    }

    /// Get the argv pointer suitable for C functions.
    pub fn as_ptr(&self) -> *const *const c_char {
        self.ptrs.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.ptrs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ptrs.is_empty()
    }
}

/// Demonstrates wrapping a C function that allocates memory.
/// Many C APIs follow this pattern: caller allocates, callee fills.
pub struct CBuffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl CBuffer {
    /// Allocate a buffer using libc's malloc.
    pub fn new(capacity: usize) -> Self {
        // SAFETY: libc::malloc returns a pointer to `capacity` bytes, or null on failure.
        let ptr = unsafe { libc::malloc(capacity) as *mut u8 };
        if ptr.is_null() {
            panic!("malloc failed for {capacity} bytes");
        }
        CBuffer {
            ptr,
            len: 0,
            cap: capacity,
        }
    }

    /// Write data into the buffer.
    ///
    /// # Safety
    /// This is safe because we track the buffer capacity and only write within bounds.
    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if self.len + data.len() > self.cap {
            return Err("buffer overflow");
        }
        // SAFETY: We've verified that the write fits within the buffer capacity.
        // The pointer was allocated by malloc and is valid for `cap` bytes.
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(self.len), data.len());
        }
        self.len += data.len();
        Ok(())
    }

    /// Read the written portion of the buffer.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: self.ptr is valid for self.len bytes (we wrote them).
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for CBuffer {
    fn drop(&mut self) {
        // SAFETY: self.ptr was allocated by libc::malloc in `new`.
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

/// Demonstrates creating a Rust callback that C code can call.
/// This pattern is used for event handlers, logging callbacks, etc.
pub type CCallback = extern "C" fn(c_int, *const c_char) -> c_int;

/// Simulates a C function that takes a callback and calls it.
/// In real FFI, this would be an `extern "C"` function in a C library.
pub fn simulate_c_callback(callback: CCallback, events: &[(i32, &str)]) -> Vec<i32> {
    let mut results = Vec::new();
    for &(code, msg) in events {
        let c_msg = CString::new(msg).unwrap();
        // SAFETY: callback is a valid function pointer. c_msg is a valid C string.
        let result = callback(code, c_msg.as_ptr());
        results.push(result);
    }
    results
}

/// A safe wrapper around a C-style opaque pointer.
/// The C side would define a struct and provide create/destroy/process functions.
pub struct OpaqueResource {
    handle: *mut c_void,
}

impl OpaqueResource {
    /// Create a new resource (simulated C allocation).
    pub fn new(initial_value: i32) -> Self {
        let boxed = Box::new(initial_value);
        OpaqueResource {
            handle: Box::into_raw(boxed) as *mut c_void,
        }
    }

    /// Process the resource value.
    pub fn process(&self) -> i32 {
        // SAFETY: handle was created from Box::into_raw in `new` and is valid
        // until we drop it in Drop. We only read through a shared reference.
        unsafe { *(self.handle as *const i32) * 2 }
    }

    /// Update the resource value.
    pub fn update(&mut self, value: i32) {
        // SAFETY: handle is valid (created in `new`). &mut self ensures
        // exclusive access.
        unsafe {
            *(self.handle as *mut i32) = value;
        }
    }
}

impl Drop for OpaqueResource {
    fn drop(&mut self) {
        // SAFETY: handle was created from Box::into_raw. Reconstructing
        // the Box and dropping it frees the memory.
        unsafe {
            drop(Box::from_raw(self.handle as *mut i32));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_strlen() {
        assert_eq!(safe_strlen("hello"), 5);
        assert_eq!(safe_strlen(""), 0);
        assert_eq!(safe_strlen("hello world"), 11);
    }

    #[test]
    fn test_c_toupper() {
        assert_eq!(c_toupper("hello"), "HELLO");
        assert_eq!(c_toupper("Hello World!"), "HELLO WORLD!");
        assert_eq!(c_toupper("123abc"), "123ABC");
    }

    #[test]
    fn test_point_distance() {
        let a = Point { x: 0.0, y: 0.0 };
        let b = Point { x: 3.0, y: 4.0 };
        let dist = point_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_segment_repr() {
        let seg = LineSegment {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 1.0, y: 1.0 },
        };
        // Verify the struct is laid out as expected
        let ptr = &seg as *const LineSegment as *const f64;
        unsafe {
            assert_eq!(*ptr.add(0), 0.0); // start.x
            assert_eq!(*ptr.add(1), 0.0); // start.y
            assert_eq!(*ptr.add(2), 1.0); // end.x
            assert_eq!(*ptr.add(3), 1.0); // end.y
        }
    }

    #[test]
    fn test_status_from_c_int() {
        assert_eq!(Status::from_c_int(0), Some(Status::Ok));
        assert_eq!(Status::from_c_int(1), Some(Status::ErrorInvalidInput));
        assert_eq!(Status::from_c_int(2), Some(Status::ErrorOverflow));
        assert_eq!(Status::from_c_int(99), Some(Status::ErrorUnknown));
        assert_eq!(Status::from_c_int(42), None);
    }

    #[test]
    fn test_sort_with_c_qsort() {
        let mut data = [5, 3, 1, 4, 2];
        sort_with_c_qsort(&mut data);
        assert_eq!(data, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_sort_with_c_qsort_empty() {
        let mut data: [i32; 0] = [];
        sort_with_c_qsort(&mut data);
        assert_eq!(data, []);
    }

    #[test]
    fn test_sort_with_c_qsort_presorted() {
        let mut data = [1, 2, 3, 4, 5];
        sort_with_c_qsort(&mut data);
        assert_eq!(data, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_c_argv() {
        let argv = CArgv::new(&["program", "--flag", "value"]);
        assert_eq!(argv.len(), 3);
        assert!(!argv.is_empty());
        // The pointer should be non-null
        assert!(!argv.as_ptr().is_null());
    }

    #[test]
    fn test_c_buffer() {
        let mut buf = CBuffer::new(1024);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 1024);

        buf.write(b"hello").unwrap();
        buf.write(b" world").unwrap();
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.as_slice(), b"hello world");
    }

    #[test]
    fn test_c_buffer_overflow() {
        let mut buf = CBuffer::new(4);
        assert!(buf.write(b"hello").is_err());
        assert!(buf.write(b"hi").is_ok());
        assert!(buf.write(b"ab").is_ok()); // exactly fills buffer
        assert!(buf.write(b"!").is_err()); // would exceed capacity
    }

    #[test]
    fn test_simulate_c_callback() {
        extern "C" fn handler(code: c_int, msg: *const c_char) -> c_int {
            // SAFETY: msg is a valid C string passed from simulate_c_callback.
            let _cstr = unsafe { CStr::from_ptr(msg) };
            code * 10
        }

        let events = [(1, "event_a"), (2, "event_b"), (3, "event_c")];
        let results = simulate_c_callback(handler, &events);
        assert_eq!(results, vec![10, 20, 30]);
    }

    #[test]
    fn test_opaque_resource() {
        let mut resource = OpaqueResource::new(21);
        assert_eq!(resource.process(), 42); // 21 * 2

        resource.update(50);
        assert_eq!(resource.process(), 100); // 50 * 2
    }

    #[test]
    fn test_opaque_resource_drop() {
        // Ensure no memory leaks
        for i in 0..100 {
            let _r = OpaqueResource::new(i);
        }
    }
}
