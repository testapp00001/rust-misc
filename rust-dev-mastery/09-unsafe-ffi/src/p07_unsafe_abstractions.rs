//! # Unsafe Abstractions: Creating Safe Wrappers
//!
//! The hallmark of good Rust library design is providing a safe API that
//! internally uses unsafe code. This lesson covers the patterns for
//! encapsulating unsafe code behind safe interfaces.
//!
//! Key principles:
//! 1. **Minimize unsafe surface**: Keep unsafe blocks as small as possible
//! 2. **Document invariants**: Every unsafe block needs a SAFETY comment
//! 3. **Validate at the boundary**: Check preconditions in safe code
//! 4. **Make invalid states unrepresentable**: Use the type system to enforce rules
//! 5. **Test thoroughly**: Unsafe code needs more testing than safe code

use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr;

/// A safe, dynamically-sized array that uses unsafe internally for
/// raw memory management. This demonstrates the complete pattern of
/// wrapping unsafe code in a safe API.
pub struct RawVec<T> {
    ptr: *mut T,
    cap: usize,
    len: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        RawVec {
            ptr: ptr::dangling_mut(),
            cap: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        let layout = Layout::array::<T>(capacity).expect("invalid layout");
        // SAFETY: layout is non-zero (capacity > 0).
        let ptr = unsafe { alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        RawVec {
            ptr,
            cap: capacity,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: len < cap after grow(). The slot at index len is within the
        // allocated buffer and is currently uninitialized.
        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: len was > 0, so self.len is now a valid index. The value
        // at this index was previously written by push. We read it out and
        // the slot becomes logically uninitialized.
        Some(unsafe { ptr::read(self.ptr.add(self.len)) })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index < len, so the slot was initialized by push.
            Some(unsafe { &*self.ptr.add(index) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: index < len, so the slot was initialized by push.
            // &mut self ensures exclusive access.
            Some(unsafe { &mut *self.ptr.add(index) })
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Elements 0..len are all initialized (written by push).
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: Same as as_slice, plus &mut self for exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).expect("invalid layout");

        let new_ptr = if self.cap == 0 {
            // SAFETY: new_layout is non-zero (new_cap >= 4).
            unsafe { alloc::alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).expect("invalid old layout");
            // SAFETY: self.ptr was allocated with old_layout. new_layout is larger.
            unsafe { alloc::realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T }
        };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        // Drop all initialized elements
        for i in 0..self.len {
            // SAFETY: Elements 0..len were initialized by push.
            unsafe {
                ptr::drop_in_place(self.ptr.add(i));
            }
        }
        let layout = Layout::array::<T>(self.cap).expect("invalid layout");
        // SAFETY: self.ptr was allocated with this layout.
        unsafe {
            alloc::dealloc(self.ptr as *mut u8, layout);
        }
    }
}

/// Demonstrates the "typestate" pattern: using types to enforce invariants.
/// A network connection transitions through states that are tracked at compile time.
pub struct TcpStream<State> {
    fd: i32,
    _state: PhantomData<State>,
}

pub struct Disconnected;
pub struct Connected;
pub struct Listening;

impl TcpStream<Disconnected> {
    pub fn new() -> Self {
        TcpStream {
            fd: -1,
            _state: PhantomData,
        }
    }

    /// Transition from Disconnected to Connected.
    /// In a real implementation, this would call the OS.
    pub fn connect(self, _addr: &str) -> Result<TcpStream<Connected>, &'static str> {
        // Simulate connection
        Ok(TcpStream {
            fd: 42, // Simulated file descriptor
            _state: PhantomData,
        })
    }
}

impl TcpStream<Connected> {
    pub fn send(&self, data: &[u8]) -> Result<usize, &'static str> {
        if self.fd < 0 {
            return Err("invalid fd");
        }
        // Simulate sending
        Ok(data.len())
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.fd < 0 {
            return Err("invalid fd");
        }
        // Simulate receiving
        let len = buf.len().min(10);
        for i in 0..len {
            buf[i] = i as u8;
        }
        Ok(len)
    }

    /// Transition from Connected to Disconnected.
    pub fn disconnect(self) -> TcpStream<Disconnected> {
        // Simulate closing connection
        TcpStream {
            fd: -1,
            _state: PhantomData,
        }
    }
}

/// Demonstrates a safe wrapper around a C-style API with resource management.
/// RAII (Resource Acquisition Is Initialization) ensures cleanup.
pub struct DatabaseConnection {
    handle: *mut std::ffi::c_void,
    is_open: bool,
}

impl DatabaseConnection {
    /// Open a simulated database connection.
    pub fn open(path: &str) -> Result<Self, String> {
        if path.is_empty() {
            return Err("path cannot be empty".to_string());
        }
        // Simulate C library allocation
        let handle = Box::into_raw(Box::new(42_i32)) as *mut std::ffi::c_void;
        Ok(DatabaseConnection {
            handle,
            is_open: true,
        })
    }

    /// Execute a simulated query.
    pub fn execute(&self, query: &str) -> Result<Vec<String>, String> {
        if !self.is_open {
            return Err("connection is closed".to_string());
        }
        if query.is_empty() {
            return Err("query cannot be empty".to_string());
        }
        // Simulate query execution
        Ok(vec![format!("result for: {query}")])
    }

    /// Close the connection explicitly.
    pub fn close(&mut self) {
        if self.is_open {
            // SAFETY: handle was created from Box::into_raw in `open`.
            unsafe {
                drop(Box::from_raw(self.handle as *mut i32));
            }
            self.handle = ptr::null_mut();
            self.is_open = false;
        }
    }
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        self.close();
    }
}

/// Demonstrates the builder pattern with unsafe internals.
/// The builder accumulates configuration and validates it before
/// constructing the final object.
pub struct BufferBuilder {
    capacity: usize,
    alignment: usize,
    zero_init: bool,
}

impl BufferBuilder {
    pub fn new() -> Self {
        BufferBuilder {
            capacity: 4096,
            alignment: 8,
            zero_init: false,
        }
    }

    pub fn capacity(mut self, cap: usize) -> Self {
        self.capacity = cap;
        self
    }

    pub fn alignment(mut self, align: usize) -> Self {
        self.alignment = align;
        self
    }

    pub fn zero_initialized(mut self) -> Self {
        self.zero_init = true;
        self
    }

    /// Build the buffer, validating all parameters.
    pub fn build(self) -> Result<ManagedBuffer, String> {
        if !self.alignment.is_power_of_two() {
            return Err("alignment must be a power of two".to_string());
        }
        if self.capacity == 0 {
            return Err("capacity must be non-zero".to_string());
        }

        let layout =
            Layout::from_size_align(self.capacity, self.alignment).map_err(|e| e.to_string())?;

        // SAFETY: layout is valid (we checked above). Capacity is non-zero.
        let ptr = if self.zero_init {
            unsafe { alloc::alloc_zeroed(layout) }
        } else {
            unsafe { alloc::alloc(layout) }
        };

        if ptr.is_null() {
            return Err("allocation failed".to_string());
        }

        Ok(ManagedBuffer { ptr, layout })
    }
}

pub struct ManagedBuffer {
    ptr: *mut u8,
    layout: Layout,
}

impl ManagedBuffer {
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: ptr is valid for layout.size() bytes.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.layout.size()) }
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is valid for layout.size() bytes.
        unsafe { std::slice::from_raw_parts(self.ptr, self.layout.size()) }
    }

    pub fn size(&self) -> usize {
        self.layout.size()
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated with self.layout in BufferBuilder::build.
        unsafe {
            alloc::dealloc(self.ptr, self.layout);
        }
    }
}

/// Demonstrates a safe sorted container using unsafe for performance.
/// Uses a sorted Vec with binary search, but manages raw pointers
/// for zero-copy insertion.
pub struct SortedSet<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> SortedSet<T> {
    pub fn new() -> Self {
        SortedSet { data: Vec::new() }
    }

    /// Insert a value, maintaining sorted order.
    pub fn insert(&mut self, value: T) -> bool {
        match self.data.binary_search(&value) {
            Ok(_) => false, // Already exists
            Err(pos) => {
                self.data.insert(pos, value);
                true
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.data.binary_search(value).is_ok()
    }

    pub fn remove(&mut self, value: &T) -> bool {
        match self.data.binary_search(value) {
            Ok(pos) => {
                self.data.remove(pos);
                true
            }
            Err(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_vec_push_pop() {
        let mut rv = RawVec::new();
        assert!(rv.is_empty());

        for i in 0..100 {
            rv.push(i);
        }
        assert_eq!(rv.len(), 100);

        for i in (0..100).rev() {
            assert_eq!(rv.pop(), Some(i));
        }
        assert!(rv.is_empty());
    }

    #[test]
    fn test_raw_vec_with_capacity() {
        let rv = RawVec::<i32>::with_capacity(10);
        assert_eq!(rv.capacity(), 10);
        assert_eq!(rv.len(), 0);
    }

    #[test]
    fn test_raw_vec_get() {
        let mut rv = RawVec::new();
        rv.push("hello");
        rv.push("world");

        assert_eq!(rv.get(0), Some(&"hello"));
        assert_eq!(rv.get(1), Some(&"world"));
        assert_eq!(rv.get(2), None);
    }

    #[test]
    fn test_raw_vec_as_slice() {
        let mut rv = RawVec::new();
        rv.push(1);
        rv.push(2);
        rv.push(3);
        assert_eq!(rv.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_raw_vec_mutable() {
        let mut rv = RawVec::new();
        rv.push(10);
        rv.push(20);

        if let Some(val) = rv.get_mut(0) {
            *val = 100;
        }
        assert_eq!(rv.get(0), Some(&100));
    }

    #[test]
    fn test_raw_vec_grow() {
        let mut rv = RawVec::new();
        assert_eq!(rv.capacity(), 0);
        rv.push(1);
        assert!(rv.capacity() >= 4);
        for i in 2..=100 {
            rv.push(i);
        }
        assert!(rv.capacity() >= 100);
    }

    #[test]
    fn test_tcp_stream_typestate() {
        let stream = TcpStream::<Disconnected>::new();
        let connected = stream.connect("127.0.0.1:8080").unwrap();

        let sent = connected.send(b"hello").unwrap();
        assert_eq!(sent, 5);

        let mut buf = [0u8; 20];
        let received = connected.recv(&mut buf).unwrap();
        assert_eq!(received, 10);

        let disconnected = connected.disconnect();
        // Can't call send on disconnected -- it won't compile:
        // disconnected.send(b"fail"); // ERROR: no method `send` on TcpStream<Disconnected>

        // Can reconnect
        let _reconnected = disconnected.connect("127.0.0.1:8080").unwrap();
    }

    #[test]
    fn test_database_connection() {
        let mut conn = DatabaseConnection::open("test.db").unwrap();
        let results = conn.execute("SELECT * FROM users").unwrap();
        assert_eq!(results.len(), 1);
        conn.close();
        assert!(conn.execute("SELECT 1").is_err());
    }

    #[test]
    fn test_database_connection_drop() {
        {
            let _conn = DatabaseConnection::open("test.db").unwrap();
            // Automatically closed on drop
        }
        // No leak, no crash
    }

    #[test]
    fn test_database_empty_path() {
        assert!(DatabaseConnection::open("").is_err());
    }

    #[test]
    fn test_buffer_builder() {
        let mut buf = BufferBuilder::new()
            .capacity(1024)
            .alignment(16)
            .zero_initialized()
            .build()
            .unwrap();

        assert_eq!(buf.size(), 1024);
        // Zero-initialized should be all zeros
        assert!(buf.as_slice().iter().all(|&b| b == 0));

        // Can write to it
        buf.as_mut_slice()[0] = 42;
        assert_eq!(buf.as_slice()[0], 42);
    }

    #[test]
    fn test_buffer_builder_invalid_alignment() {
        let result = BufferBuilder::new().alignment(3).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_builder_zero_capacity() {
        let result = BufferBuilder::new().capacity(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_sorted_set() {
        let mut set = SortedSet::new();
        assert!(set.insert(3));
        assert!(set.insert(1));
        assert!(set.insert(4));
        assert!(!set.insert(1)); // Duplicate returns false
        assert!(!set.insert(1)); // Returns false for duplicate

        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&3));
        assert!(set.contains(&4));
        assert!(!set.contains(&2));

        let values: Vec<&i32> = set.iter().collect();
        assert_eq!(values, vec![&1, &3, &4]);

        assert!(set.remove(&3));
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&3));
    }
}
