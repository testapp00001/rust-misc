//! # Strategy Pattern
//!
//! The strategy pattern defines a family of algorithms, encapsulates each one,
//! and makes them interchangeable. In Rust, strategies can be implemented via
//! generics (static dispatch) or trait objects (dynamic dispatch).
//!
//! ## Key Concepts
//! - **Static dispatch**: Generics with trait bounds — zero overhead
//! - **Dynamic dispatch**: Trait objects (`dyn Trait`) — runtime flexibility
//! - **When to use each**: Static for performance, dynamic for runtime selection

/// A sorting strategy that can be swapped at runtime.
pub trait SortStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn sort(&self, data: &mut [i32]);
}

pub struct BubbleSort;
pub struct InsertionSort;
pub struct QuickSort;

impl SortStrategy for BubbleSort {
    fn name(&self) -> &str {
        "bubble"
    }

    fn sort(&self, data: &mut [i32]) {
        let n = data.len();
        for i in 0..n {
            for j in 0..n - 1 - i {
                if data[j] > data[j + 1] {
                    data.swap(j, j + 1);
                }
            }
        }
    }
}

impl SortStrategy for InsertionSort {
    fn name(&self) -> &str {
        "insertion"
    }

    fn sort(&self, data: &mut [i32]) {
        for i in 1..data.len() {
            let key = data[i];
            let mut j = i;
            while j > 0 && data[j - 1] > key {
                data[j] = data[j - 1];
                j -= 1;
            }
            data[j] = key;
        }
    }
}

impl SortStrategy for QuickSort {
    fn name(&self) -> &str {
        "quick"
    }

    fn sort(&self, data: &mut [i32]) {
        if data.len() <= 1 {
            return;
        }
        quicksort(data, 0, data.len() as isize - 1);
    }
}

fn quicksort(data: &mut [i32], low: isize, high: isize) {
    if low < high {
        let pivot = partition(data, low, high);
        quicksort(data, low, pivot - 1);
        quicksort(data, pivot + 1, high);
    }
}

fn partition(data: &mut [i32], low: isize, high: isize) -> isize {
    let pivot = data[high as usize];
    let mut i = low - 1;
    for j in low..high {
        if data[j as usize] <= pivot {
            i += 1;
            data.swap(i as usize, j as usize);
        }
    }
    data.swap((i + 1) as usize, high as usize);
    i + 1
}

/// Uses a sort strategy dynamically (trait object).
pub fn sort_with_strategy(strategy: &dyn SortStrategy, data: &mut [i32]) {
    strategy.sort(data);
}

/// Uses a sort strategy statically (generic).
pub fn sort_generic<S: SortStrategy>(strategy: &S, data: &mut [i32]) {
    strategy.sort(data);
}

/// A logging strategy that can be changed at runtime.
pub trait LogStrategy: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct ConsoleLogger;
pub struct BufferedLogger {
    buffer: std::sync::Mutex<Vec<String>>,
}

impl LogStrategy for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // In real code, print to stderr
        let _ = (level, message);
    }
}

impl BufferedLogger {
    pub fn new() -> Self {
        BufferedLogger {
            buffer: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn drain(&self) -> Vec<String> {
        self.buffer.lock().unwrap().drain(..).collect()
    }
}

impl LogStrategy for BufferedLogger {
    fn log(&self, level: LogLevel, message: &str) {
        self.buffer
            .lock()
            .unwrap()
            .push(format!("[{level:?}] {message}"));
    }
}

/// A configurable processor that uses a strategy for transformation.
pub trait TransformStrategy<T>: Send + Sync {
    fn transform(&self, input: T) -> T;
}

pub struct DoubleStrategy;
pub struct NegateStrategy;
pub struct AbsStrategy;

impl TransformStrategy<i32> for DoubleStrategy {
    fn transform(&self, input: i32) -> i32 {
        input * 2
    }
}

impl TransformStrategy<i32> for NegateStrategy {
    fn transform(&self, input: i32) -> i32 {
        -input
    }
}

impl TransformStrategy<i32> for AbsStrategy {
    fn transform(&self, input: i32) -> i32 {
        input.abs()
    }
}

/// A pipeline that chains multiple strategies.
pub struct Pipeline<T> {
    strategies: Vec<Box<dyn TransformStrategy<T>>>,
}

impl<T: Clone> Pipeline<T> {
    pub fn new() -> Self {
        Pipeline {
            strategies: Vec::new(),
        }
    }

    pub fn add<S: TransformStrategy<T> + 'static>(mut self, strategy: S) -> Self {
        self.strategies.push(Box::new(strategy));
        self
    }

    pub fn execute(&self, input: T) -> T {
        let mut current = input;
        for strategy in &self.strategies {
            current = strategy.transform(current);
        }
        current
    }
}

/// A hash strategy for choosing different hashing algorithms.
pub trait HashStrategy: Send + Sync {
    fn hash(&self, data: &[u8]) -> u64;
}

pub struct FnvHash;
pub struct Djb2Hash;

impl HashStrategy for FnvHash {
    fn hash(&self, data: &[u8]) -> u64 {
        let mut hash: u64 = 14695981039346656037;
        for &byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }
}

impl HashStrategy for Djb2Hash {
    fn hash(&self, data: &[u8]) -> u64 {
        let mut hash: u64 = 5381;
        for &byte in data {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

/// A hash map that uses a pluggable hash strategy.
pub struct CustomHashMap<V> {
    buckets: Vec<Vec<(String, V)>>,
    strategy: Box<dyn HashStrategy>,
    size: usize,
}

impl<V> CustomHashMap<V> {
    pub fn new(strategy: impl HashStrategy + 'static, bucket_count: usize) -> Self {
        CustomHashMap {
            buckets: (0..bucket_count).map(|_| Vec::new()).collect(),
            strategy: Box::new(strategy),
            size: 0,
        }
    }

    fn bucket_index(&self, key: &str) -> usize {
        let hash = self.strategy.hash(key.as_bytes());
        (hash as usize) % self.buckets.len()
    }

    pub fn insert(&mut self, key: String, value: V) {
        let idx = self.bucket_index(&key);
        if let Some(entry) = self.buckets[idx].iter_mut().find(|(k, _)| k == &key) {
            entry.1 = value;
        } else {
            self.buckets[idx].push((key, value));
            self.size += 1;
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        let idx = self.bucket_index(key);
        self.buckets[idx]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_sort() {
        let mut data = vec![5, 3, 1, 4, 2];
        BubbleSort.sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_insertion_sort() {
        let mut data = vec![5, 3, 1, 4, 2];
        InsertionSort.sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_quick_sort() {
        let mut data = vec![5, 3, 1, 4, 2];
        QuickSort.sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_sort_with_strategy() {
        let strategies: Vec<Box<dyn SortStrategy>> = vec![
            Box::new(BubbleSort),
            Box::new(InsertionSort),
            Box::new(QuickSort),
        ];

        for strategy in &strategies {
            let mut data = vec![5, 3, 1, 4, 2];
            sort_with_strategy(strategy.as_ref(), &mut data);
            assert_eq!(data, vec![1, 2, 3, 4, 5], "Failed for {}", strategy.name());
        }
    }

    #[test]
    fn test_buffered_logger() {
        let logger = BufferedLogger::new();
        logger.log(LogLevel::Info, "test message");
        logger.log(LogLevel::Error, "error message");

        let entries = logger.drain();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("Info"));
        assert!(entries[1].contains("Error"));
    }

    #[test]
    fn test_pipeline() {
        let pipeline = Pipeline::new()
            .add(DoubleStrategy)
            .add(NegateStrategy);

        let result = pipeline.execute(5);
        assert_eq!(result, -10); // 5 * 2 = 10, negate = -10
    }

    #[test]
    fn test_pipeline_abs() {
        let pipeline = Pipeline::new()
            .add(NegateStrategy)
            .add(AbsStrategy);

        let result = pipeline.execute(-5);
        assert_eq!(result, 5); // negate(-5) = 5, abs(5) = 5
    }

    #[test]
    fn test_custom_hash_map_fnv() {
        let mut map = CustomHashMap::new(FnvHash, 16);
        map.insert("key1".into(), 100);
        map.insert("key2".into(), 200);

        assert_eq!(map.get("key1"), Some(&100));
        assert_eq!(map.get("key2"), Some(&200));
        assert_eq!(map.get("key3"), None);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_custom_hash_map_djb2() {
        let mut map = CustomHashMap::new(Djb2Hash, 16);
        map.insert("key1".into(), 100);
        assert_eq!(map.get("key1"), Some(&100));
    }

    #[test]
    fn test_custom_hash_map_update() {
        let mut map = CustomHashMap::new(FnvHash, 16);
        map.insert("key".into(), 1);
        map.insert("key".into(), 2);
        assert_eq!(map.get("key"), Some(&2));
        assert_eq!(map.len(), 1);
    }
}
