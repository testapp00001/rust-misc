/// Problem: Memory Optimization
///
/// Master memory optimization in Rust.
///
/// Key Concepts:
/// - Memory layout
/// - Allocation optimization
/// - Data structure choice
/// - Reference vs owned
/// - Memory pooling

/// Problem 1: Memory layout
/// Check memory layout
#[repr(C)]
pub struct OptimizedStruct {
    pub a: u8,
    pub b: u32,
    pub c: u8,
}

pub fn check_layout() -> usize {
    std::mem::size_of::<OptimizedStruct>()
}

/// Problem 2: Allocation optimization
/// Minimize allocations
pub fn minimize_allocations() -> Vec<i32> {
    let mut v = Vec::with_capacity(100);
    for i in 0..100 {
        v.push(i);
    }
    v
}

/// Problem 3: Reference vs owned
/// Use references when possible
pub fn use_references(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 4: Avoid cloning
/// Minimize cloning
pub fn avoid_clone(data: &[String]) -> Vec<&str> {
    data.iter().map(|s| s.as_str()).collect()
}

/// Problem 5: Use appropriate data structures
/// Choose right data structure
pub fn appropriate_data_structure() -> std::collections::HashMap<String, i32> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), 42);
    map
}

/// Problem 6: Memory pooling
/// Reuse memory
pub struct Pool {
    objects: Vec<Vec<u8>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn get(&mut self) -> Vec<u8> {
        self.objects.pop().unwrap_or_else(|| Vec::with_capacity(1024))
    }

    pub fn put(&mut self, mut obj: Vec<u8>) {
        obj.clear();
        self.objects.push(obj);
    }
}

/// Problem 7: Avoid unnecessary copies
/// Use move semantics
pub fn avoid_copies(data: Vec<i32>) -> Vec<i32> {
    data // Move, don't copy
}

/// Problem 8: Use stack allocation
/// Prefer stack over heap
pub fn stack_allocation() -> [i32; 100] {
    [0; 100]
}

/// Problem 9: Use Box for large data
/// Box large structs
pub fn box_large_data() -> Box<[i32; 1000]> {
    Box::new([0; 1000])
}

/// Problem 10: Use Rc for shared ownership
/// Share data with Rc
pub fn shared_ownership() -> std::rc::Rc<Vec<i32>> {
    std::rc::Rc::new(vec![1, 2, 3, 4, 5])
}

/// Problem 11: Use Cow for clone-on-write
/// Use Cow for efficiency
pub fn cow_example(data: std::borrow::Cow<str>) -> std::borrow::Cow<str> {
    if data.contains("error") {
        std::borrow::Cow::Owned(data.replace("error", "warning"))
    } else {
        data
    }
}

/// Problem 12: Use SmallVec for small collections
/// Use SmallVec for small collections (simulated)
pub fn small_vec_example() -> Vec<i32> {
    let mut v = Vec::with_capacity(8);
    for i in 0..8 {
        v.push(i);
    }
    v
}

/// Problem 13: Use Box<str> instead of String
/// Use Box<str> for immutable strings
pub fn box_str_example() -> Box<str> {
    "hello".into()
}

/// Problem 14: Use arrays instead of Vec
/// Use arrays for fixed-size data
pub fn array_example() -> [i32; 5] {
    [1, 2, 3, 4, 5]
}

/// Problem 15: Use enum instead of trait object
/// Use enum for known types
#[derive(Debug)]
pub enum Value {
    Int(i32),
    Float(f64),
    Text(String),
}

pub fn enum_example() -> Value {
    Value::Int(42)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_layout() {
        let size = check_layout();
        assert!(size > 0);
    }

    #[test]
    fn test_minimize_allocations() {
        let v = minimize_allocations();
        assert_eq!(v.len(), 100);
    }

    #[test]
    fn test_use_references() {
        assert_eq!(use_references(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_avoid_clone() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let refs = avoid_clone(&data);
        assert_eq!(refs, vec!["hello", "world"]);
    }

    #[test]
    fn test_appropriate_data_structure() {
        let map = appropriate_data_structure();
        assert_eq!(map.get("key"), Some(&42));
    }

    #[test]
    fn test_pool() {
        let mut pool = Pool::new();
        let obj = pool.get();
        assert!(obj.capacity() >= 1024);
        pool.put(obj);
        let obj2 = pool.get();
        assert!(obj2.capacity() >= 1024);
    }

    #[test]
    fn test_avoid_copies() {
        let data = vec![1, 2, 3];
        let result = avoid_copies(data);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_stack_allocation() {
        let arr = stack_allocation();
        assert_eq!(arr.len(), 100);
    }

    #[test]
    fn test_box_large_data() {
        let data = box_large_data();
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_shared_ownership() {
        let data = shared_ownership();
        assert_eq!(data.len(), 5);
    }

    #[test]
    fn test_cow_example() {
        let data = std::borrow::Cow::Borrowed("hello");
        let result = cow_example(data);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_small_vec_example() {
        let v = small_vec_example();
        assert_eq!(v.len(), 8);
    }

    #[test]
    fn test_box_str_example() {
        let s = box_str_example();
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn test_array_example() {
        let arr = array_example();
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_enum_example() {
        let value = enum_example();
        assert!(matches!(value, Value::Int(42)));
    }
}
