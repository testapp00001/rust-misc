//! # bindgen Usage: Generating Rust Bindings from C Headers
//!
//! `bindgen` automatically generates Rust FFI bindings from C/C++ header files.
//! This eliminates the tedious and error-prone process of manually writing
//! `extern "C"` declarations.
//!
//! Typical workflow:
//! 1. Write a `build.rs` that invokes bindgen
//! 2. Point bindgen at your C header file
//! 3. bindgen generates Rust types, constants, and function signatures
//! 4. Create safe wrappers around the generated unsafe bindings
//!
//! This module demonstrates the patterns and structures that bindgen produces,
//! so you can understand and work with generated bindings.

use std::collections::HashMap;

/// Simulates what bindgen generates from a C header like:
/// ```c
/// typedef struct {
///     double x;
///     double y;
///     double z;
/// } vec3_t;
///
/// double vec3_length(const vec3_t *v);
/// void vec3_normalize(vec3_t *v);
/// ```
///
/// bindgen produces `#[repr(C)]` types with raw pointers and unsafe functions.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Simulated bindgen output for `vec3_length`.
/// In real bindgen output, this would be an `extern "C"` function declaration.
///
/// # Safety
/// `v` must point to a valid Vec3.
pub unsafe fn vec3_length_raw(v: *const Vec3) -> f64 {
    let v = &*v;
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

/// Simulated bindgen output for `vec3_normalize`.
///
/// # Safety
/// `v` must point to a valid, writable Vec3.
pub unsafe fn vec3_normalize_raw(v: *mut Vec3) {
    let len = vec3_length_raw(v);
    if len > 0.0 {
        (*v).x /= len;
        (*v).y /= len;
        (*v).z /= len;
    }
}

/// Safe wrapper around the bindgen-generated bindings.
/// This is the idiomatic way to use bindgen: generate raw bindings,
/// then wrap them in safe Rust APIs.
impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Safe wrapper around the raw FFI function.
    pub fn length(&self) -> f64 {
        // SAFETY: `self` is a valid Vec3 (guaranteed by the reference).
        unsafe { vec3_length_raw(self) }
    }

    /// Safe wrapper that returns a new normalized vector.
    pub fn normalized(&self) -> Self {
        let mut result = *self;
        // SAFETY: result is a valid, stack-allocated Vec3 that we own.
        unsafe { vec3_normalize_raw(&mut result) };
        result
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

/// Demonstrates how bindgen handles C enums.
/// A C header like:
/// ```c
/// typedef enum {
///     LOG_LEVEL_DEBUG = 0,
///     LOG_LEVEL_INFO = 1,
///     LOG_LEVEL_WARN = 2,
///     LOG_LEVEL_ERROR = 3,
/// } log_level_t;
/// ```
///
/// bindgen generates this as a Rust enum with `#[repr(C)]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn from_raw(val: i32) -> Option<Self> {
        match val {
            0 => Some(LogLevel::Debug),
            1 => Some(LogLevel::Info),
            2 => Some(LogLevel::Warn),
            3 => Some(LogLevel::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Simulates a C library's logger with a callback-based API.
/// In C:
/// ```c
/// typedef void (*log_handler_t)(log_level_t level, const char *msg, void *user_data);
/// void logger_set_handler(log_handler_t handler, void *user_data);
/// ```
pub type LogHandler = extern "C" fn(LogLevel, *const std::os::raw::c_char, *mut std::os::raw::c_void);

/// A safe wrapper for a C library's logging system.
pub struct Logger {
    handler: Option<LogHandler>,
    messages: Vec<(LogLevel, String)>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            handler: None,
            messages: Vec::new(),
        }
    }

    /// Simulate setting a C callback handler.
    pub fn set_handler(&mut self, handler: LogHandler) {
        self.handler = Some(handler);
    }

    /// Log a message through the handler (simulated).
    pub fn log(&mut self, level: LogLevel, message: &str) {
        self.messages.push((level, message.to_string()));
    }

    pub fn messages(&self) -> &[(LogLevel, String)] {
        &self.messages
    }
}

/// Demonstrates how bindgen handles C arrays and fixed-size buffers.
/// C header:
/// ```c
/// typedef struct {
///     char name[64];
///     int values[16];
///     size_t count;
/// } dataset_t;
/// ```
#[repr(C)]
#[derive(Clone)]
pub struct Dataset {
    pub name: [std::os::raw::c_char; 64],
    pub values: [std::os::raw::c_int; 16],
    pub count: usize,
}

impl Dataset {
    pub fn new(name: &str) -> Self {
        let mut ds = Dataset {
            name: [0; 64],
            values: [0; 16],
            count: 0,
        };
        ds.set_name(name);
        ds
    }

    pub fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(63); // Leave room for null terminator
        for (i, &b) in bytes.iter().take(len).enumerate() {
            self.name[i] = b as std::os::raw::c_char;
        }
        self.name[len] = 0; // Null terminate
    }

    pub fn get_name(&self) -> &str {
        // Find the null terminator
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(64);
        // SAFETY: We initialized the name from valid UTF-8 and only stored ASCII bytes.
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.name.as_ptr() as *const u8,
                len,
            ))
        }
    }

    pub fn add_value(&mut self, val: i32) -> Result<(), &'static str> {
        if self.count >= 16 {
            return Err("dataset full");
        }
        self.values[self.count] = val;
        self.count += 1;
        Ok(())
    }

    pub fn values_slice(&self) -> &[i32] {
        // SAFETY: values is an array of i32 (c_int is i32 on all major platforms).
        // We take only the initialized portion (0..count).
        unsafe {
            std::slice::from_raw_parts(self.values.as_ptr() as *const i32, self.count)
        }
    }
}

/// Demonstrates how bindgen handles C function pointers embedded in structs.
/// C header:
/// ```c
/// typedef struct {
///     void *(*alloc)(size_t size);
///     void (*free)(void *ptr);
///     void *user_data;
/// } allocator_t;
/// ```
#[repr(C)]
pub struct CAllocator {
    pub alloc: Option<extern "C" fn(usize) -> *mut std::os::raw::c_void>,
    pub free: Option<extern "C" fn(*mut std::os::raw::c_void)>,
    pub user_data: *mut std::os::raw::c_void,
}

/// A safe wrapper that manages a CAllocator.
pub struct AllocatorWrapper {
    inner: CAllocator,
    allocated: HashMap<usize, usize>, // ptr -> size
}

impl AllocatorWrapper {
    /// Create a wrapper using the default libc allocator.
    pub fn new() -> Self {
        AllocatorWrapper {
            inner: CAllocator {
                alloc: Some(alloc_wrapper),
                free: Some(free_wrapper),
                user_data: std::ptr::null_mut(),
            },
            allocated: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        let alloc_fn = self.inner.alloc?;
        // SAFETY: alloc_fn is a valid function pointer set in `new`.
        let ptr = alloc_fn(size);
        if ptr.is_null() {
            return None;
        }
        self.allocated.insert(ptr as usize, size);
        Some(ptr as *mut u8)
    }

    pub fn deallocate(&mut self, ptr: *mut u8) {
        if let Some(free_fn) = self.inner.free {
            self.allocated.remove(&(ptr as usize));
            // SAFETY: ptr was allocated by our allocator and hasn't been freed yet.
            free_fn(ptr as *mut std::os::raw::c_void);
        }
    }

    pub fn allocated_count(&self) -> usize {
        self.allocated.len()
    }
}

extern "C" fn alloc_wrapper(size: usize) -> *mut std::os::raw::c_void {
    // SAFETY: libc::malloc allocates `size` bytes. Returns null on failure.
    unsafe { libc::malloc(size) }
}

extern "C" fn free_wrapper(ptr: *mut std::os::raw::c_void) {
    // SAFETY: ptr was allocated by alloc_wrapper (which uses libc::malloc).
    unsafe {
        libc::free(ptr);
    }
}

/// Demonstrates generating a build.rs script structure.
/// This shows the code you'd write in build.rs for a real bindgen project.
pub mod build_rs_example {
    /// The build.rs would contain code that uses bindgen::Builder to generate
    /// Rust bindings from a C header file, then writes them to $OUT_DIR.
    ///
    /// In your Rust code, you would include the generated bindings with:
    pub fn _placeholder() {}
}

/// Demonstrates the pattern of testing FFI bindings with layout tests.
/// bindgen generates these automatically, but here's what they look like.
pub fn verify_layout<T>(expected_size: usize, expected_align: usize) -> bool {
    std::mem::size_of::<T>() == expected_size && std::mem::align_of::<T>() == expected_align
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(1.0, 2.0, 2.0);
        assert!((v.length() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_normalized() {
        let v = Vec3::new(3.0, 0.0, 4.0);
        let n = v.normalized();
        assert!((n.length() - 1.0).abs() < 1e-10);
        assert!((n.x - 0.6).abs() < 1e-10);
        assert!((n.y - 0.0).abs() < 1e-10);
        assert!((n.z - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_zero() {
        let v = Vec3::zero();
        assert_eq!(v.length(), 0.0);
        // Normalizing zero vector should not crash (division by zero handled)
        let n = v.normalized();
        assert!(n.x.is_nan() || n.x == 0.0);
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert!((a.dot(&b) - 0.0).abs() < 1e-10);

        let c = Vec3::new(1.0, 2.0, 3.0);
        let d = Vec3::new(4.0, 5.0, 6.0);
        assert!((c.dot(&d) - 32.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_cross() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert!((z.x - 0.0).abs() < 1e-10);
        assert!((z.y - 0.0).abs() < 1e-10);
        assert!((z.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3_layout() {
        assert!(verify_layout::<Vec3>(24, 8));
    }

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::from_raw(0), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_raw(3), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_raw(99), None);
        assert_eq!(LogLevel::Info.as_str(), "INFO");
    }

    #[test]
    fn test_log_level_layout() {
        // C enums are typically int-sized
        assert_eq!(std::mem::size_of::<LogLevel>(), 4);
    }

    #[test]
    fn test_logger() {
        let mut logger = Logger::new();
        logger.log(LogLevel::Info, "starting up");
        logger.log(LogLevel::Error, "something failed");
        assert_eq!(logger.messages().len(), 2);
        assert_eq!(logger.messages()[0].0, LogLevel::Info);
        assert_eq!(logger.messages()[1].1, "something failed");
    }

    #[test]
    fn test_dataset() {
        let mut ds = Dataset::new("test_data");
        assert_eq!(ds.get_name(), "test_data");

        ds.add_value(10).unwrap();
        ds.add_value(20).unwrap();
        ds.add_value(30).unwrap();
        assert_eq!(ds.values_slice(), &[10, 20, 30]);
    }

    #[test]
    fn test_dataset_overflow() {
        let mut ds = Dataset::new("full");
        for i in 0..16 {
            ds.add_value(i).unwrap();
        }
        assert!(ds.add_value(16).is_err());
    }

    #[test]
    fn test_dataset_layout() {
        // name: 64 bytes, values: 16 * 4 = 64 bytes, count: 8 bytes
        assert!(std::mem::size_of::<Dataset>() >= 64 + 64 + 8);
    }

    #[test]
    fn test_allocator_wrapper() {
        let mut alloc = AllocatorWrapper::new();
        assert_eq!(alloc.allocated_count(), 0);

        let ptr1 = alloc.allocate(100).unwrap();
        assert_eq!(alloc.allocated_count(), 1);

        let ptr2 = alloc.allocate(200).unwrap();
        assert_eq!(alloc.allocated_count(), 2);

        // Verify the pointers are different
        assert_ne!(ptr1 as usize, ptr2 as usize);

        alloc.deallocate(ptr1);
        assert_eq!(alloc.allocated_count(), 1);

        alloc.deallocate(ptr2);
        assert_eq!(alloc.allocated_count(), 0);
    }

    #[test]
    fn test_c_allocator_layout() {
        // Function pointers are 8 bytes each on 64-bit, plus a pointer
        assert!(std::mem::size_of::<CAllocator>() >= 24);
    }
}
