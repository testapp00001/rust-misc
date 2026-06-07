/// Problem: Trade-offs
///
/// Master trade-offs in Rust.
///
/// Key Concepts:
/// - Performance vs safety
/// - Simplicity vs flexibility
/// - Memory vs CPU
/// - Latency vs throughput
/// - Consistency vs availability

/// Problem 1: Performance vs Safety
/// Demonstrate performance vs safety trade-off
pub fn safe_version(data: &[i32]) -> i32 {
    data.iter().sum()
}

pub fn unsafe_version(data: &[i32]) -> i32 {
    unsafe {
        let mut sum = 0;
        for i in 0..data.len() {
            sum += *data.get_unchecked(i);
        }
        sum
    }
}

/// Problem 2: Simplicity vs Flexibility
/// Demonstrate simplicity vs flexibility
pub struct SimpleStore {
    data: std::collections::HashMap<String, String>,
}

impl SimpleStore {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

pub struct FlexibleStore<K, V> {
    data: std::collections::HashMap<K, V>,
}

impl<K: Eq + std::hash::Hash, V> FlexibleStore<K, V> {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }
}

/// Problem 3: Memory vs CPU
/// Demonstrate memory vs CPU trade-off
pub fn memory_efficient(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

pub fn cpu_efficient(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 4: Latency vs Throughput
/// Demonstrate latency vs throughput
pub fn low_latency(data: &[i32]) -> i32 {
    data[0]
}

pub fn high_throughput(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 5: Consistency vs Availability
/// Demonstrate consistency vs availability
pub struct ConsistentStore {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl ConsistentStore {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

pub struct AvailableStore {
    data: std::sync::RwLock<std::collections::HashMap<String, String>>,
}

impl AvailableStore {
    pub fn new() -> Self {
        Self {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        self.data.write().unwrap().insert(key.to_string(), value.to_string());
    }
}

/// Problem 6: Static vs Dynamic
/// Demonstrate static vs dynamic dispatch
pub trait Processor {
    fn process(&self, x: i32) -> i32;
}

pub struct Double;

impl Processor for Double {
    fn process(&self, x: i32) -> i32 {
        x * 2
    }
}

pub fn static_dispatch(p: &impl Processor, x: i32) -> i32 {
    p.process(x)
}

pub fn dynamic_dispatch(p: &dyn Processor, x: i32) -> i32 {
    p.process(x)
}

/// Problem 7: Owned vs Borrowed
/// Demonstrate owned vs borrowed
pub fn owned_version(s: String) -> String {
    s
}

pub fn borrowed_version(s: &str) -> &str {
    s
}

/// Problem 8: Enum vs Trait Object
/// Demonstrate enum vs trait object
#[derive(Debug)]
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
        }
    }
}

pub trait ShapeTrait {
    fn area(&self) -> f64;
}

pub struct CircleTrait(pub f64);

impl ShapeTrait for CircleTrait {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.0 * self.0
    }
}

/// Problem 9: Clone vs Reference
/// Demonstrate clone vs reference
pub fn clone_version(data: Vec<i32>) -> Vec<i32> {
    data.clone()
}

pub fn reference_version(data: &[i32]) -> &[i32] {
    data
}

/// Problem 10: Error Handling Trade-offs
/// Demonstrate error handling trade-offs
pub fn result_version(x: i32) -> Result<i32, String> {
    if x > 0 {
        Ok(x)
    } else {
        Err("Invalid".to_string())
    }
}

pub fn option_version(x: i32) -> Option<i32> {
    if x > 0 {
        Some(x)
    } else {
        None
    }
}

/// Problem 11: Synchronous vs Asynchronous
/// Demonstrate sync vs async
pub fn sync_version() -> i32 {
    42
}

pub async fn async_version() -> i32 {
    42
}

/// Problem 12: Single-threaded vs Multi-threaded
/// Demonstrate single vs multi-threaded
pub fn single_threaded(data: &[i32]) -> i32 {
    data.iter().sum()
}

pub fn multi_threaded(data: &[i32]) -> i32 {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let chunk_size = (data.len() + 3) / 4;
    let result = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let result = Arc::clone(&result);
        handles.push(thread::spawn(move || {
            let sum: i32 = chunk.iter().sum();
            *result.lock().unwrap() += sum;
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *result.lock().unwrap();
    result
}

/// Problem 13: Custom vs Standard
/// Demonstrate custom vs standard
pub struct CustomVec {
    data: Vec<i32>,
}

impl CustomVec {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&i32> {
        self.data.get(index)
    }
}

/// Problem 14: Monolithic vs Modular
/// Demonstrate monolithic vs modular
pub struct Monolithic {
    data: std::collections::HashMap<String, String>,
    cache: std::collections::HashMap<String, String>,
}

impl Monolithic {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            cache: std::collections::HashMap::new(),
        }
    }
}

pub struct Modular {
    storage: Storage,
    cache: Cache,
}

pub struct Storage {
    data: std::collections::HashMap<String, String>,
}

pub struct Cache {
    data: std::collections::HashMap<String, String>,
}

/// Problem 15: Trade-off Documentation
/// Document trade-offs
/// # Trade-offs
///
/// - Performance: Optimized for speed
/// - Safety: Uses unsafe for performance
/// - Memory: Uses more memory for caching
pub fn documented_trade_off() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_vs_unsafe() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(safe_version(&data), 15);
        assert_eq!(unsafe_version(&data), 15);
    }

    #[test]
    fn test_simple_vs_flexible() {
        let mut simple = SimpleStore::new();
        simple.set("key", "value");
        assert_eq!(simple.get("key"), Some("value"));

        let mut flexible = FlexibleStore::new();
        flexible.set("key", 42);
        assert_eq!(flexible.get(&"key"), Some(&42));
    }

    #[test]
    fn test_memory_vs_cpu() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(memory_efficient(&data), 15);
        assert_eq!(cpu_efficient(&data), 15);
    }

    #[test]
    fn test_latency_vs_throughput() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(low_latency(&data), 1);
        assert_eq!(high_throughput(&data), 15);
    }

    #[test]
    fn test_consistency_vs_availability() {
        let consistent = ConsistentStore::new();
        consistent.set("key", "value");
        assert_eq!(consistent.get("key"), Some("value".to_string()));

        let available = AvailableStore::new();
        available.set("key", "value");
        assert_eq!(available.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_static_vs_dynamic() {
        let processor = Double;
        assert_eq!(static_dispatch(&processor, 21), 42);
        assert_eq!(dynamic_dispatch(&processor, 21), 42);
    }

    #[test]
    fn test_owned_vs_borrowed() {
        let s = String::from("hello");
        assert_eq!(owned_version(s), "hello");
        assert_eq!(borrowed_version("hello"), "hello");
    }

    #[test]
    fn test_enum_vs_trait() {
        let circle = Shape::Circle(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);

        let circle_trait = CircleTrait(5.0);
        assert!((circle_trait.area() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clone_vs_reference() {
        let data = vec![1, 2, 3];
        assert_eq!(clone_version(data.clone()), vec![1, 2, 3]);
        assert_eq!(reference_version(&data), &[1, 2, 3]);
    }

    #[test]
    fn test_error_handling() {
        assert_eq!(result_version(42), Ok(42));
        assert_eq!(option_version(42), Some(42));
    }

    #[tokio::test]
    async fn test_sync_vs_async() {
        assert_eq!(sync_version(), 42);
        assert_eq!(async_version().await, 42);
    }

    #[test]
    fn test_single_vs_multi() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(single_threaded(&data), 15);
        assert_eq!(multi_threaded(&data), 15);
    }
}
