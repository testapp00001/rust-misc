//! # ABI Stability
//!
//! Application Binary Interface (ABI) stability ensures that compiled code
//! remains compatible across versions. This module covers `repr(C)`,
//! versioned APIs, and ABI evolution strategies.
//!
//! ## Key Concepts:
//!
//! - `repr(C)`: Use C-compatible memory layout
//! - `repr(packed)`: No padding between fields
//! - `repr(transparent)`: Same layout as inner type
//! - Versioned APIs: Change without breaking compatibility

/// C-compatible struct with repr(C).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CRect {
    pub origin: CPoint,
    pub width: f64,
    pub height: f64,
}

/// ABI-stable string wrapper.
#[repr(C)]
pub struct AbiString {
    data: *const u8,
    len: usize,
}

impl AbiString {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    pub unsafe fn as_str(&self) -> &str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.len))
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

/// Versioned API with backward compatibility.
pub mod api_v1 {
    use super::*;

    #[repr(C)]
    pub struct Config {
        pub version: u32,
        pub name: AbiString,
        pub value: f64,
    }

    impl Config {
        pub fn new(name: &str, value: f64) -> Self {
            Self {
                version: 1,
                name: AbiString::new(name),
                value,
            }
        }
    }

    pub fn process(config: &Config) -> f64 {
        config.value * 2.0
    }
}

pub mod api_v2 {
    use super::*;

    #[repr(C)]
    pub struct Config {
        pub version: u32,
        pub name: AbiString,
        pub value: f64,
        pub flags: u32, // New field in v2
    }

    impl Config {
        pub fn new(name: &str, value: f64, flags: u32) -> Self {
            Self {
                version: 2,
                name: AbiString::new(name),
                value,
                flags,
            }
        }

        /// Convert from v1 config (backward compatibility).
        pub fn from_v1(v1: &api_v1::Config) -> Self {
            Self {
                version: 2,
                name: AbiString::new(unsafe { v1.name.as_str() }),
                value: v1.value,
                flags: 0,
            }
        }
    }

    pub fn process(config: &Config) -> f64 {
        let multiplier = if config.flags & 0x01 != 0 { 3.0 } else { 2.0 };
        config.value * multiplier
    }
}

/// ABI version negotiation.
pub struct AbiVersionNegotiator {
    supported_versions: Vec<u32>,
}

impl AbiVersionNegotiator {
    pub fn new(supported_versions: Vec<u32>) -> Self {
        Self { supported_versions }
    }

    /// Negotiate the highest mutually supported version.
    pub fn negotiate(&self, client_versions: &[u32]) -> Option<u32> {
        self.supported_versions
            .iter()
            .filter(|v| client_versions.contains(v))
            .max()
            .copied()
    }

    pub fn is_supported(&self, version: u32) -> bool {
        self.supported_versions.contains(&version)
    }
}

/// Opaque type pattern: hide implementation details behind a pointer.
#[repr(C)]
pub struct OpaqueHandle {
    _private: [u8; 0],
}

pub struct OpaqueInner {
    pub data: Vec<u8>,
    pub metadata: String,
}

/// Handle manager for opaque types.
pub struct HandleManager<T> {
    handles: std::collections::HashMap<u64, T>,
    next_id: u64,
}

impl<T> HandleManager<T> {
    pub fn new() -> Self {
        Self {
            handles: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, value: T) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.handles.insert(id, value);
        id
    }

    pub fn get(&self, handle: u64) -> Option<&T> {
        self.handles.get(&handle)
    }

    pub fn get_mut(&mut self, handle: u64) -> Option<&mut T> {
        self.handles.get_mut(&handle)
    }

    pub fn destroy(&mut self, handle: u64) -> Option<T> {
        self.handles.remove(&handle)
    }

    pub fn count(&self) -> usize {
        self.handles.len()
    }
}

/// ABI-stable callback with context pointer.
pub type AbiCallback = extern "C" fn(context: *mut std::ffi::c_void, data: u64) -> i32;

/// Wrapper for ABI-stable callbacks with context.
pub struct AbiCallbackWrapper {
    callback: AbiCallback,
    context: *mut std::ffi::c_void,
}

impl AbiCallbackWrapper {
    pub fn new(callback: AbiCallback, context: *mut std::ffi::c_void) -> Self {
        Self { callback, context }
    }

    pub fn call(&self, data: u64) -> i32 {
        (self.callback)(self.context, data)
    }
}

/// Feature flag for ABI compatibility.
#[derive(Debug, Clone)]
pub struct AbiFeatureFlags {
    flags: u64,
}

impl AbiFeatureFlags {
    pub fn new() -> Self {
        Self { flags: 0 }
    }

    pub fn set(&mut self, flag: u64) {
        self.flags |= flag;
    }

    pub fn has(&self, flag: u64) -> bool {
        self.flags & flag != 0
    }

    pub fn clear(&mut self, flag: u64) {
        self.flags &= !flag;
    }

    pub fn bits(&self) -> u64 {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_point_layout() {
        assert_eq!(std::mem::size_of::<CPoint>(), 16); // 2 * f64
        assert_eq!(std::mem::align_of::<CPoint>(), 8);
    }

    #[test]
    fn test_c_rect_layout() {
        assert_eq!(std::mem::size_of::<CRect>(), 32); // 4 * f64
    }

    #[test]
    fn test_abi_string() {
        let s = AbiString::new("hello");
        assert_eq!(s.len(), 5);
        unsafe {
            assert_eq!(s.as_str(), "hello");
        }
    }

    #[test]
    fn test_api_v1() {
        let config = api_v1::Config::new("test", 10.0);
        let result = api_v1::process(&config);
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_api_v2_from_v1() {
        let v1 = api_v1::Config::new("test", 10.0);
        let v2 = api_v2::Config::from_v1(&v1);

        assert_eq!(v2.version, 2);
        assert_eq!(v2.flags, 0);
        let result = api_v2::process(&v2);
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_api_v2_with_flags() {
        let config = api_v2::Config::new("test", 10.0, 0x01);
        let result = api_v2::process(&config);
        assert!((result - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_abi_version_negotiation() {
        let negotiator = AbiVersionNegotiator::new(vec![1, 2, 3]);
        assert_eq!(negotiator.negotiate(&[1, 2, 3]), Some(3));
        assert_eq!(negotiator.negotiate(&[1, 2]), Some(2));
        assert_eq!(negotiator.negotiate(&[4, 5]), None);
    }

    #[test]
    fn test_abi_version_supported() {
        let negotiator = AbiVersionNegotiator::new(vec![1, 2]);
        assert!(negotiator.is_supported(1));
        assert!(negotiator.is_supported(2));
        assert!(!negotiator.is_supported(3));
    }

    #[test]
    fn test_handle_manager() {
        let mut manager = HandleManager::new();
        let h1 = manager.create("data1".to_string());
        let h2 = manager.create("data2".to_string());

        assert_eq!(manager.count(), 2);
        assert_eq!(manager.get(h1).unwrap(), "data1");
        assert_eq!(manager.get(h2).unwrap(), "data2");

        manager.destroy(h1);
        assert_eq!(manager.count(), 1);
        assert!(manager.get(h1).is_none());
    }

    #[test]
    fn test_handle_manager_mut() {
        let mut manager = HandleManager::new();
        let handle = manager.create(42u32);

        if let Some(val) = manager.get_mut(handle) {
            *val = 100;
        }

        assert_eq!(*manager.get(handle).unwrap(), 100);
    }

    #[test]
    fn test_abi_feature_flags() {
        let mut flags = AbiFeatureFlags::new();
        assert!(!flags.has(0x01));

        flags.set(0x01);
        assert!(flags.has(0x01));
        assert!(!flags.has(0x02));

        flags.set(0x02);
        assert!(flags.has(0x01));
        assert!(flags.has(0x02));

        flags.clear(0x01);
        assert!(!flags.has(0x01));
        assert!(flags.has(0x02));
    }

    #[test]
    fn test_abi_feature_flags_bits() {
        let mut flags = AbiFeatureFlags::new();
        flags.set(0x01);
        flags.set(0x04);
        assert_eq!(flags.bits(), 0x05);
    }
}
