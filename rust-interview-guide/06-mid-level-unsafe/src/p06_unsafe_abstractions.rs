/// Problem: Unsafe Abstractions
///
/// Master creating safe abstractions over unsafe code.
///
/// Key Concepts:
/// - Safe wrappers
/// - Invariant enforcement
/// - API design
/// - Documentation
/// - Testing

/// Problem 1: Safe wrapper around raw pointer
/// Create safe wrapper
pub struct SafePtr {
    ptr: *mut i32,
}

impl SafePtr {
    pub fn new(value: i32) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Self { ptr }
    }

    pub fn get(&self) -> i32 {
        unsafe { *self.ptr }
    }

    pub fn set(&mut self, value: i32) {
        unsafe { *self.ptr = value; }
    }
}

impl Drop for SafePtr {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.ptr);
        }
    }
}

/// Problem 2: Safe wrapper around Vec with raw pointer
/// Create safe Vec wrapper
pub struct SafeVec {
    ptr: *mut i32,
    len: usize,
    capacity: usize,
}

impl SafeVec {
    pub fn new() -> Self {
        let mut v = Vec::new();
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let capacity = v.capacity();
        std::mem::forget(v);
        Self { ptr, len, capacity }
    }

    pub fn push(&mut self, value: i32) {
        if self.len == self.capacity {
            self.grow();
        }
        unsafe {
            self.ptr.add(self.len).write(value);
        }
        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        if index < self.len {
            Some(unsafe { self.ptr.add(index).read() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };
        let mut new_vec = Vec::with_capacity(new_capacity);
        let new_ptr = new_vec.as_mut_ptr();
        std::mem::forget(new_vec);

        if !self.ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
            }
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl Drop for SafeVec {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = Vec::from_raw_parts(self.ptr, self.len, self.capacity);
            }
        }
    }
}

/// Problem 3: Safe wrapper around string
/// Create safe string wrapper
pub struct SafeString {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl SafeString {
    pub fn new(s: &str) -> Self {
        let mut string = s.to_string();
        let ptr = string.as_mut_ptr();
        let len = string.len();
        let capacity = string.capacity();
        std::mem::forget(string);
        Self { ptr, len, capacity }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for SafeString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = String::from_raw_parts(self.ptr, self.len, self.capacity);
            }
        }
    }
}

/// Problem 4: Safe wrapper around HashMap
/// Create safe HashMap wrapper
pub struct SafeHashMap {
    data: std::collections::HashMap<String, String>,
}

impl SafeHashMap {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
}

/// Problem 5: Safe wrapper around unsafe function
/// Wrap unsafe function safely
pub fn safe_transmute<T, U>(value: T) -> U
where
    T: Copy,
    U: Copy,
{
    unsafe { std::mem::transmute_copy(&value) }
}

/// Problem 6: Safe wrapper around unsafe trait
/// Create safe trait wrapper
pub trait SafeTrait {
    fn safe_method(&self) -> i32;
}

pub struct SafeWrapper<T> {
    inner: T,
}

impl<T> SafeWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl SafeTrait for SafeWrapper<i32> {
    fn safe_method(&self) -> i32 {
        self.inner
    }
}

/// Problem 7: Safe wrapper around unsafe struct
/// Create safe struct wrapper
pub struct SafeStruct {
    data: Vec<i32>,
}

impl SafeStruct {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }
}

/// Problem 8: Safe wrapper around unsafe enum
/// Create safe enum wrapper
pub enum SafeEnum {
    Int(i32),
    Float(f64),
    Text(String),
}

impl SafeEnum {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            SafeEnum::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            SafeEnum::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            SafeEnum::Text(v) => Some(v),
            _ => None,
        }
    }
}

/// Problem 9: Safe wrapper around unsafe closure
/// Create safe closure wrapper
pub struct SafeClosure {
    closure: Box<dyn Fn(i32) -> i32>,
}

impl SafeClosure {
    pub fn new<F: Fn(i32) -> i32 + 'static>(closure: F) -> Self {
        Self {
            closure: Box::new(closure),
        }
    }

    pub fn call(&self, x: i32) -> i32 {
        (self.closure)(x)
    }
}

/// Problem 10: Safe wrapper around unsafe iterator
/// Create safe iterator wrapper
pub struct SafeIterator {
    data: Vec<i32>,
    index: usize,
}

impl SafeIterator {
    pub fn new(data: Vec<i32>) -> Self {
        Self { data, index: 0 }
    }

    pub fn next(&mut self) -> Option<i32> {
        if self.index < self.data.len() {
            let value = self.data[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// Problem 11: Safe wrapper around unsafe channel
/// Create safe channel wrapper
pub struct SafeChannel {
    sender: std::sync::mpsc::Sender<i32>,
    receiver: std::sync::mpsc::Receiver<i32>,
}

impl SafeChannel {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self { sender, receiver }
    }

    pub fn send(&self, value: i32) -> Result<(), String> {
        self.sender.send(value).map_err(|e| e.to_string())
    }

    pub fn recv(&self) -> Result<i32, String> {
        self.receiver.recv().map_err(|e| e.to_string())
    }
}

/// Problem 12: Safe wrapper around unsafe mutex
/// Create safe mutex wrapper
pub struct SafeMutex {
    data: std::sync::Mutex<i32>,
}

impl SafeMutex {
    pub fn new(value: i32) -> Self {
        Self {
            data: std::sync::Mutex::new(value),
        }
    }

    pub fn lock(&self) -> i32 {
        let result = *self.data.lock().unwrap(); result
    }

    pub fn set(&self, value: i32) {
        *self.data.lock().unwrap() = value;
    }
}

/// Problem 13: Safe wrapper around unsafe Arc
/// Create safe Arc wrapper
pub struct SafeArc {
    data: std::sync::Arc<i32>,
}

impl SafeArc {
    pub fn new(value: i32) -> Self {
        Self {
            data: std::sync::Arc::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        *self.data
    }

    pub fn clone(&self) -> Self {
        Self {
            data: std::sync::Arc::clone(&self.data),
        }
    }
}

/// Problem 14: Safe wrapper around unsafe Box
/// Create safe Box wrapper
pub struct SafeBox {
    data: Box<i32>,
}

impl SafeBox {
    pub fn new(value: i32) -> Self {
        Self {
            data: Box::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        *self.data
    }
}

/// Problem 15: Safe wrapper around unsafe Rc
/// Create safe Rc wrapper
pub struct SafeRc {
    data: std::rc::Rc<i32>,
}

impl SafeRc {
    pub fn new(value: i32) -> Self {
        Self {
            data: std::rc::Rc::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        *self.data
    }

    pub fn clone(&self) -> Self {
        Self {
            data: std::rc::Rc::clone(&self.data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_ptr() {
        let mut ptr = SafePtr::new(42);
        assert_eq!(ptr.get(), 42);
        ptr.set(100);
        assert_eq!(ptr.get(), 100);
    }

    #[test]
    fn test_safe_vec() {
        let mut vec = SafeVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(1), Some(2));
    }

    #[test]
    fn test_safe_string() {
        let s = SafeString::new("Hello");
        assert_eq!(s.as_str(), "Hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_safe_hashmap() {
        let mut map = SafeHashMap::new();
        map.insert("key", "value");
        assert_eq!(map.get("key"), Some("value"));
    }

    #[test]
    fn test_safe_transmute() {
        let x: i32 = 42;
        let y: u32 = safe_transmute(x);
        assert_eq!(y, 42);
    }

    #[test]
    fn test_safe_wrapper() {
        let wrapper = SafeWrapper::new(42);
        assert_eq!(wrapper.safe_method(), 42);
    }

    #[test]
    fn test_safe_struct() {
        let mut s = SafeStruct::new();
        s.push(42);
        assert_eq!(s.get(0), Some(42));
    }

    #[test]
    fn test_safe_enum() {
        let e = SafeEnum::Int(42);
        assert_eq!(e.as_int(), Some(42));
    }

    #[test]
    fn test_safe_closure() {
        let c = SafeClosure::new(|x| x * 2);
        assert_eq!(c.call(21), 42);
    }

    #[test]
    fn test_safe_iterator() {
        let mut iter = SafeIterator::new(vec![1, 2, 3]);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_safe_channel() {
        let ch = SafeChannel::new();
        ch.send(42).unwrap();
        assert_eq!(ch.recv().unwrap(), 42);
    }

    #[test]
    fn test_safe_mutex() {
        let m = SafeMutex::new(42);
        assert_eq!(m.lock(), 42);
        m.set(100);
        assert_eq!(m.lock(), 100);
    }

    #[test]
    fn test_safe_arc() {
        let a = SafeArc::new(42);
        let b = a.clone();
        assert_eq!(a.get(), 42);
        assert_eq!(b.get(), 42);
    }

    #[test]
    fn test_safe_box() {
        let b = SafeBox::new(42);
        assert_eq!(b.get(), 42);
    }

    #[test]
    fn test_safe_rc() {
        let a = SafeRc::new(42);
        let b = a.clone();
        assert_eq!(a.get(), 42);
        assert_eq!(b.get(), 42);
    }
}
