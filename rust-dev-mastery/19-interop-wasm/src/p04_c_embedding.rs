//! # C Embedding — Exposing Rust to C/C++
//!
//! Rust can be compiled as a C-compatible dynamic library (cdylib) or
//! static library (staticlib) for embedding in C/C++ applications.
//!
//! ## Cargo.toml:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib", "staticlib"]
//! ```
//!
//! ## Key Concepts:
//!
//! - `extern "C"`: Use C calling convention
//! - `#[no_mangle]`: Preserve function names
//! - `repr(C)`: Make structs C-compatible
//! - Safety: All FFI functions are unsafe at the boundary

/// C-compatible string type.
/// Wraps a Rust String and provides C-compatible access.
#[repr(C)]
pub struct CString {
    data: *const u8,
    len: usize,
    capacity: usize,
}

impl CString {
    /// Create from a Rust string (transfers ownership).
    pub fn from_rust(s: String) -> Self {
        let len = s.len();
        let capacity = s.capacity();
        let data = s.as_ptr();
        std::mem::forget(s);
        Self { data, len, capacity }
    }

    /// Get the string as a Rust &str.
    /// Safety: The CString must still be valid.
    pub unsafe fn as_str(&self) -> &str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.len))
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Free the string.
    /// Safety: Must only be called once.
    pub unsafe fn free(self) {
        let _ = String::from_raw_parts(
            self.data as *mut u8,
            self.len,
            self.capacity,
        );
        // String is dropped here
    }
}

/// C-compatible result type.
#[repr(C)]
pub struct CResult<T> {
    pub value: T,
    pub is_error: bool,
    pub error_message: *const u8,
    pub error_len: usize,
}

impl<T> CResult<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            is_error: false,
            error_message: std::ptr::null(),
            error_len: 0,
        }
    }

    pub fn error(msg: &str) -> Self
    where
        T: Default,
    {
        let bytes = msg.as_bytes();
        Self {
            value: T::default(),
            is_error: true,
            error_message: bytes.as_ptr(),
            error_len: bytes.len(),
        }
    }

    pub fn is_ok(&self) -> bool {
        !self.is_error
    }

    pub unsafe fn error_str(&self) -> Option<&str> {
        if self.is_error && !self.error_message.is_null() {
            Some(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.error_message,
                self.error_len,
            )))
        } else {
            None
        }
    }
}

/// C-compatible API for a calculator.
pub struct CCalculator {
    history: Vec<f64>,
}

impl CCalculator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.history.push(result);
        result
    }

    pub fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.history.push(result);
        result
    }

    pub fn divide(&mut self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            return Err("Division by zero".into());
        }
        let result = a / b;
        self.history.push(result);
        Ok(result)
    }

    pub fn history(&self) -> &[f64] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// C API functions (in real code, these would be extern "C" #[no_mangle]).
pub mod c_api {
    use super::*;

    /// Create a new calculator.
    pub fn calculator_new() -> *mut CCalculator {
        Box::into_raw(Box::new(CCalculator::new()))
    }

    /// Destroy a calculator.
    /// Safety: `calc` must be a valid pointer from `calculator_new`.
    pub unsafe fn calculator_free(calc: *mut CCalculator) {
        if !calc.is_null() {
            drop(Box::from_raw(calc));
        }
    }

    /// Add two numbers.
    pub unsafe fn calculator_add(calc: *mut CCalculator, a: f64, b: f64) -> f64 {
        (*calc).add(a, b)
    }

    /// Multiply two numbers.
    pub unsafe fn calculator_multiply(calc: *mut CCalculator, a: f64, b: f64) -> f64 {
        (*calc).multiply(a, b)
    }

    /// Divide two numbers.
    pub unsafe fn calculator_divide(
        calc: *mut CCalculator,
        a: f64,
        b: f64,
        result: *mut f64,
    ) -> i32 {
        match (*calc).divide(a, b) {
            Ok(value) => {
                *result = value;
                0 // Success
            }
            Err(_) => -1, // Error
        }
    }

    /// Get history length.
    pub unsafe fn calculator_history_len(calc: *const CCalculator) -> usize {
        (*calc).history().len()
    }

    /// Get a history entry.
    pub unsafe fn calculator_history_get(calc: *const CCalculator, index: usize) -> f64 {
        (*calc).history()[index]
    }
}

/// C-compatible callback function type.
pub type CCallback = extern "C" fn(data: *const u8, len: usize) -> i32;

/// Event handler that accepts C callbacks.
pub struct CEventHandler {
    callbacks: Vec<CCallback>,
}

impl CEventHandler {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn register_callback(&mut self, callback: CCallback) {
        self.callbacks.push(callback);
    }

    pub fn fire_event(&self, data: &[u8]) -> Vec<i32> {
        self.callbacks
            .iter()
            .map(|cb| cb(data.as_ptr(), data.len()))
            .collect()
    }
}

/// C-compatible byte buffer for passing binary data.
#[repr(C)]
pub struct CBuffer {
    data: *mut u8,
    len: usize,
    capacity: usize,
}

impl CBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);
        let data = vec.as_mut_ptr();
        let cap = vec.capacity();
        std::mem::forget(vec);
        Self {
            data,
            len: 0,
            capacity: cap,
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let mut vec = data.to_vec();
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        std::mem::forget(vec);
        Self {
            data: ptr,
            len,
            capacity: cap,
        }
    }

    /// Get as a slice.
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.data, self.len)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Free the buffer.
    pub unsafe fn free(self) {
        let _ = Vec::from_raw_parts(self.data, self.len, self.capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_string() {
        let s = String::from("hello");
        let c_str = CString::from_rust(s);
        assert_eq!(c_str.len(), 5);
        unsafe {
            assert_eq!(c_str.as_str(), "hello");
            c_str.free();
        }
    }

    #[test]
    fn test_c_result_ok() {
        let result = CResult::ok(42);
        assert!(result.is_ok());
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_c_result_error() {
        let result: CResult<i32> = CResult::error("something went wrong");
        assert!(!result.is_ok());
        unsafe {
            assert_eq!(result.error_str(), Some("something went wrong"));
        }
    }

    #[test]
    fn test_c_api_calculator() {
        let calc = c_api::calculator_new();

        unsafe {
            let result = c_api::calculator_add(calc, 2.0, 3.0);
            assert!((result - 5.0).abs() < f64::EPSILON);

            let result = c_api::calculator_multiply(calc, 4.0, 5.0);
            assert!((result - 20.0).abs() < f64::EPSILON);

            let mut result = 0.0f64;
            let status = c_api::calculator_divide(calc, 10.0, 2.0, &mut result);
            assert_eq!(status, 0);
            assert!((result - 5.0).abs() < f64::EPSILON);

            assert_eq!(c_api::calculator_history_len(calc), 3);

            c_api::calculator_free(calc);
        }
    }

    #[test]
    fn test_c_api_divide_by_zero() {
        let calc = c_api::calculator_new();

        unsafe {
            let mut result = 0.0f64;
            let status = c_api::calculator_divide(calc, 1.0, 0.0, &mut result);
            assert_eq!(status, -1); // Error

            c_api::calculator_free(calc);
        }
    }

    #[test]
    fn test_c_event_handler() {
        extern "C" fn callback(data: *const u8, len: usize) -> i32 {
            unsafe {
                let slice = std::slice::from_raw_parts(data, len);
                slice.len() as i32
            }
        }

        let mut handler = CEventHandler::new();
        handler.register_callback(callback);

        let results = handler.fire_event(b"hello");
        assert_eq!(results, vec![5]);
    }

    #[test]
    fn test_c_buffer() {
        let buf = CBuffer::from_slice(b"hello");
        assert_eq!(buf.len(), 5);
        unsafe {
            assert_eq!(buf.as_slice(), b"hello");
            buf.free();
        }
    }

    #[test]
    fn test_c_buffer_new() {
        let buf = CBuffer::new(100);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity, 100);
        unsafe { buf.free(); }
    }

    #[test]
    fn test_calculator_direct() {
        let mut calc = CCalculator::new();
        assert!((calc.add(1.0, 2.0) - 3.0).abs() < f64::EPSILON);
        assert!((calc.multiply(3.0, 4.0) - 12.0).abs() < f64::EPSILON);
        assert!((calc.divide(10.0, 2.0).unwrap() - 5.0).abs() < f64::EPSILON);
        assert!(calc.divide(1.0, 0.0).is_err());

        assert_eq!(calc.history().len(), 3);
        calc.clear_history();
        assert!(calc.history().is_empty());
    }
}
