//! # WebAssembly Fundamentals
//!
//! Rust compiles to WebAssembly (WASM) for running in browsers, Node.js,
//! and other WASM runtimes. This module covers the fundamental patterns
//! for Rust-WASM interop.
//!
//! ## Setup:
//!
//! ```bash
//! cargo install wasm-pack
//! wasm-pack build --target web
//! ```
//!
//! ## Key Concepts:
//!
//! - `#[wasm_bindgen]`: Bridge between Rust and JavaScript
//! - `wasm-bindgen`: Crate for JS interop
//! - `web-sys`: Bindings to Web APIs
//! - `js-sys`: Bindings to JavaScript built-in objects
//!
//! ## Memory Model:
//!
//! WASM has a linear memory buffer. Data is shared between Rust and JS
//! through this buffer. Strings and complex types require serialization.

/// Simulated WASM-compatible type for demonstration.
/// In real WASM code, these would use #[wasm_bindgen].
pub struct WasmProcessor {
    data: Vec<f64>,
    result: Option<f64>,
}

impl WasmProcessor {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            result: None,
        }
    }

    /// Expose to JavaScript as a constructor-like function.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            result: None,
        }
    }

    /// Method exposed to JS.
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Compute and cache result.
    pub fn compute(&mut self) -> f64 {
        let result = self.data.iter().sum::<f64>() / self.data.len() as f64;
        self.result = Some(result);
        result
    }

    pub fn get_result(&self) -> Option<f64> {
        self.result
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get data as a pointer (for WASM memory sharing).
    pub fn data_ptr(&self) -> *const f64 {
        self.data.as_ptr()
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }
}

/// WASM-compatible string handling.
/// WASM doesn't have a native string type; strings are passed through linear memory.
pub struct WasmString {
    inner: String,
}

impl WasmString {
    pub fn new(s: &str) -> Self {
        Self { inner: s.into() }
    }

    /// Get the byte representation for WASM memory.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    /// Get the length in bytes.
    pub fn byte_len(&self) -> usize {
        self.inner.len()
    }

    /// Create from bytes (from WASM memory).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, std::str::Utf8Error> {
        let s = std::str::from_utf8(bytes)?;
        Ok(Self { inner: s.into() })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn to_uppercase(&self) -> Self {
        Self {
            inner: self.inner.to_uppercase(),
        }
    }

    pub fn to_lowercase(&self) -> Self {
        Self {
            inner: self.inner.to_lowercase(),
        }
    }
}

/// WASM-compatible callback pattern.
/// JavaScript functions passed to Rust via function pointers.
pub struct WasmCallback {
    callback_id: u32,
    // In real WASM, this would be a js_sys::Function
}

impl WasmCallback {
    pub fn new(callback_id: u32) -> Self {
        Self { callback_id }
    }

    pub fn id(&self) -> u32 {
        self.callback_id
    }

    /// Simulate calling a JS callback.
    pub fn call(&self, args: &[f64]) -> f64 {
        // In real WASM, this would call the JS function
        args.iter().sum()
    }
}

/// Event system for WASM-JS communication.
pub struct WasmEventSystem {
    listeners: std::collections::HashMap<String, Vec<u32>>,
    next_id: u32,
}

impl WasmEventSystem {
    pub fn new() -> Self {
        Self {
            listeners: std::collections::HashMap::new(),
            next_id: 0,
        }
    }

    /// Register a listener for an event type.
    pub fn add_listener(&mut self, event_type: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners
            .entry(event_type.into())
            .or_default()
            .push(id);
        id
    }

    /// Get listeners for an event type.
    pub fn get_listeners(&self, event_type: &str) -> Vec<u32> {
        self.listeners
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    pub fn remove_listener(&mut self, id: u32) {
        for listeners in self.listeners.values_mut() {
            listeners.retain(|&lid| lid != id);
        }
    }
}

/// WASM memory management utilities.
pub struct WasmMemory {
    buffer: Vec<u8>,
    allocated: std::collections::HashMap<u32, (usize, usize)>, // id -> (offset, size)
    next_id: u32,
}

impl WasmMemory {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size],
            allocated: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// Allocate memory in the WASM buffer.
    pub fn alloc(&mut self, size: usize) -> Option<u32> {
        let offset = self.find_free_offset(size)?;
        let id = self.next_id;
        self.next_id += 1;
        self.allocated.insert(id, (offset, size));
        Some(id)
    }

    fn find_free_offset(&self, size: usize) -> Option<usize> {
        let mut used: Vec<(usize, usize)> = self.allocated.values().copied().collect();
        used.sort_by_key(|&(offset, _)| offset);

        let mut candidate = 0;
        for &(offset, alloc_size) in &used {
            if candidate + size <= offset {
                return Some(candidate);
            }
            candidate = offset + alloc_size;
        }

        if candidate + size <= self.buffer.len() {
            Some(candidate)
        } else {
            None
        }
    }

    /// Free an allocation.
    pub fn free(&mut self, id: u32) -> bool {
        self.allocated.remove(&id).is_some()
    }

    /// Get a reference to allocated memory.
    pub fn get(&self, id: u32) -> Option<&[u8]> {
        let &(offset, size) = self.allocated.get(&id)?;
        Some(&self.buffer[offset..offset + size])
    }

    /// Get a mutable reference to allocated memory.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let &(offset, size) = self.allocated.get(&id)?;
        Some(&mut self.buffer[offset..offset + size])
    }

    pub fn total_allocated(&self) -> usize {
        self.allocated.values().map(|&(_, size)| size).sum()
    }

    pub fn allocation_count(&self) -> usize {
        self.allocated.len()
    }
}

/// JSON serialization for WASM-JS communication.
pub struct WasmJson;

impl WasmJson {
    /// Serialize a value to JSON bytes for WASM memory.
    /// This is a simplified demonstration — in real WASM interop you'd use
    /// serde + serde_json, but here we show the concept with a basic
    /// manual serializer for common types.
    pub fn to_json_bytes<T: std::fmt::Debug + std::fmt::Display>(
        value: &T,
    ) -> Result<Vec<u8>, String> {
        // For demonstration: convert to debug string and wrap as JSON.
        // A real implementation would use serde_json::to_vec.
        let json = format!("{:?}", value);
        Ok(json.into_bytes())
    }

    /// Deserialize from JSON bytes (from WASM memory).
    /// This is a placeholder showing the interop pattern.
    pub fn from_json_string(bytes: &[u8]) -> Result<String, String> {
        String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_processor() {
        let mut proc = WasmProcessor::new();
        proc.push(1.0);
        proc.push(2.0);
        proc.push(3.0);

        let result = proc.compute();
        assert!((result - 2.0).abs() < f64::EPSILON);
        assert_eq!(proc.get_result(), Some(2.0));
    }

    #[test]
    fn test_wasm_processor_with_capacity() {
        let proc = WasmProcessor::with_capacity(100);
        assert_eq!(proc.len(), 0);
    }

    #[test]
    fn test_wasm_string() {
        let ws = WasmString::new("hello");
        assert_eq!(ws.as_str(), "hello");
        assert_eq!(ws.byte_len(), 5);

        let upper = ws.to_uppercase();
        assert_eq!(upper.as_str(), "HELLO");
    }

    #[test]
    fn test_wasm_string_from_bytes() {
        let bytes = b"world";
        let ws = WasmString::from_bytes(bytes).unwrap();
        assert_eq!(ws.as_str(), "world");
    }

    #[test]
    fn test_wasm_string_invalid_utf8() {
        let bytes = [0xFF, 0xFE];
        assert!(WasmString::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_wasm_callback() {
        let cb = WasmCallback::new(42);
        assert_eq!(cb.id(), 42);
        let result = cb.call(&[1.0, 2.0, 3.0]);
        assert!((result - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wasm_event_system() {
        let mut events = WasmEventSystem::new();
        let id1 = events.add_listener("click");
        let id2 = events.add_listener("click");
        let id3 = events.add_listener("keydown");

        assert_eq!(events.get_listeners("click").len(), 2);
        assert_eq!(events.get_listeners("keydown").len(), 1);

        events.remove_listener(id1);
        assert_eq!(events.get_listeners("click").len(), 1);
    }

    #[test]
    fn test_wasm_memory_alloc() {
        let mut mem = WasmMemory::new(1024);
        let id1 = mem.alloc(100).unwrap();
        let id2 = mem.alloc(200).unwrap();

        assert_eq!(mem.allocation_count(), 2);
        assert_eq!(mem.total_allocated(), 300);

        mem.free(id1);
        assert_eq!(mem.allocation_count(), 1);
    }

    #[test]
    fn test_wasm_memory_read_write() {
        let mut mem = WasmMemory::new(1024);
        let id = mem.alloc(16).unwrap();

        let data = mem.get_mut(id).unwrap();
        data[0] = 42;
        data[1] = 43;

        let data = mem.get(id).unwrap();
        assert_eq!(data[0], 42);
        assert_eq!(data[1], 43);
    }

    #[test]
    fn test_wasm_memory_exhaustion() {
        let mut mem = WasmMemory::new(100);
        mem.alloc(60).unwrap();
        assert!(mem.alloc(60).is_none()); // Only 40 bytes left
    }

    #[test]
    fn test_wasm_memory_reuse() {
        let mut mem = WasmMemory::new(100);
        let id1 = mem.alloc(50).unwrap();
        mem.free(id1);
        let id2 = mem.alloc(50).unwrap(); // Should reuse the freed space
        assert!(mem.get(id2).is_some());
    }

    #[test]
    fn test_wasm_string_lowercase() {
        let ws = WasmString::new("HELLO WORLD");
        let lower = ws.to_lowercase();
        assert_eq!(lower.as_str(), "hello world");
    }

    #[test]
    fn test_wasm_processor_data_ptr() {
        let mut proc = WasmProcessor::new();
        proc.push(1.0);
        let ptr = proc.data_ptr();
        assert!(!ptr.is_null());
        assert_eq!(proc.data_len(), 1);
    }
}
