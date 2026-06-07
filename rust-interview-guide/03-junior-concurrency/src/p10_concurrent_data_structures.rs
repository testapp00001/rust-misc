/// Problem: Concurrent Data Structures
///
/// Master concurrent data structures.
///
/// Key Concepts:
/// - Thread-safe collections
/// - Lock-free structures
/// - Concurrent queues
/// - Concurrent maps
/// - Concurrent stacks

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::thread;

/// Problem 1: Thread-safe vector
/// Use Mutex<Vec<T>> for thread-safe vector
pub fn thread_safe_vector() -> Vec<i32> {
    let vec = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for i in 0..10 {
        let vec = Arc::clone(&vec);
        let handle = thread::spawn(move || {
            let mut vec = vec.lock().unwrap();
            vec.push(i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

let result =     vec.lock().unwrap().clone(); result
}

/// Problem 2: Thread-safe HashMap
/// Use Mutex<HashMap<K, V>> for thread-safe HashMap
pub fn thread_safe_hashmap() -> HashMap<String, i32> {
    let map = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];

    for i in 0..10 {
        let map = Arc::clone(&map);
        let handle = thread::spawn(move || {
            let mut map = map.lock().unwrap();
            map.insert(format!("key{}", i), i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

let result =     map.lock().unwrap().clone(); result
}

/// Problem 3: Concurrent queue
/// Implement a concurrent queue
pub struct ConcurrentQueue<T> {
    queue: Mutex<Vec<T>>,
}

impl<T> ConcurrentQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(item);
    }

    pub fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }
}

/// Problem 4: Concurrent stack
/// Implement a concurrent stack
pub struct ConcurrentStack<T> {
    stack: Mutex<Vec<T>>,
}

impl<T> ConcurrentStack<T> {
    pub fn new() -> Self {
        Self {
            stack: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, item: T) {
        let mut stack = self.stack.lock().unwrap();
        stack.push(item);
    }

    pub fn pop(&self) -> Option<T> {
        let mut stack = self.stack.lock().unwrap();
        stack.pop()
    }

    pub fn len(&self) -> usize {
        let stack = self.stack.lock().unwrap();
        stack.len()
    }

    pub fn is_empty(&self) -> bool {
        let stack = self.stack.lock().unwrap();
        stack.is_empty()
    }
}

/// Problem 5: Concurrent counter
/// Implement a concurrent counter
pub struct ConcurrentCounter {
    count: Mutex<i32>,
}

impl ConcurrentCounter {
    pub fn new() -> Self {
        Self {
            count: Mutex::new(0),
        }
    }

    pub fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    pub fn decrement(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
    }

    pub fn get(&self) -> i32 {
        let count = self.count.lock().unwrap();
        *count
    }
}

/// Problem 6: Concurrent map with RwLock
/// Use RwLock for concurrent map with multiple readers
pub fn concurrent_map_rwlock() -> HashMap<String, i32> {
    let map = Arc::new(RwLock::new(HashMap::new()));
    let mut handles = vec![];

    // Writers
    for i in 0..5 {
        let map = Arc::clone(&map);
        let handle = thread::spawn(move || {
            let mut map = map.write().unwrap();
            map.insert(format!("key{}", i), i);
        });
        handles.push(handle);
    }

    // Wait for writers
    for handle in handles {
        handle.join().unwrap();
    }

    let result = map.read().unwrap().clone(); result
}

/// Problem 7: Concurrent queue with channels
/// Use channels as a concurrent queue
pub fn concurrent_queue_channels() -> Vec<i32> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Producers
    for i in 0..5 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            tx.send(i * 2).unwrap();
        });
        handles.push(handle);
    }

    drop(tx);

    // Consumer
    let mut results = Vec::new();
    for val in rx {
        results.push(val);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    results
}

/// Problem 8: Concurrent set
/// Implement a concurrent set
pub struct ConcurrentSet {
    set: Mutex<std::collections::HashSet<i32>>,
}

impl ConcurrentSet {
    pub fn new() -> Self {
        Self {
            set: Mutex::new(std::collections::HashSet::new()),
        }
    }

    pub fn insert(&self, item: i32) -> bool {
        let mut set = self.set.lock().unwrap();
        set.insert(item)
    }

    pub fn contains(&self, item: &i32) -> bool {
        let set = self.set.lock().unwrap();
        set.contains(item)
    }

    pub fn remove(&self, item: &i32) -> bool {
        let mut set = self.set.lock().unwrap();
        set.remove(item)
    }

    pub fn len(&self) -> usize {
        let set = self.set.lock().unwrap();
        set.len()
    }
}

/// Problem 9: Concurrent buffer
/// Implement a concurrent buffer
pub struct ConcurrentBuffer {
    buffer: Mutex<Vec<i32>>,
    capacity: usize,
}

impl ConcurrentBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, item: i32) -> bool {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() < self.capacity {
            buffer.push(item);
            true
        } else {
            false
        }
    }

    pub fn pop(&self) -> Option<i32> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            None
        } else {
            Some(buffer.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        let buffer = self.buffer.lock().unwrap();
        buffer.len()
    }
}

/// Problem 10: Concurrent statistics
/// Track statistics concurrently
pub struct ConcurrentStats {
    sum: Mutex<f64>,
    count: Mutex<usize>,
    min: Mutex<f64>,
    max: Mutex<f64>,
}

impl ConcurrentStats {
    pub fn new() -> Self {
        Self {
            sum: Mutex::new(0.0),
            count: Mutex::new(0),
            min: Mutex::new(f64::INFINITY),
            max: Mutex::new(f64::NEG_INFINITY),
        }
    }

    pub fn add(&self, value: f64) {
        let mut sum = self.sum.lock().unwrap();
        *sum += value;

        let mut count = self.count.lock().unwrap();
        *count += 1;

        let mut min = self.min.lock().unwrap();
        if value < *min {
            *min = value;
        }

        let mut max = self.max.lock().unwrap();
        if value > *max {
            *max = value;
        }
    }

    pub fn mean(&self) -> f64 {
        let sum = self.sum.lock().unwrap();
        let count = self.count.lock().unwrap();
        if *count == 0 {
            0.0
        } else {
            *sum / *count as f64
        }
    }

    pub fn min(&self) -> f64 {
        let result = *self.min.lock().unwrap(); result
    }

    pub fn max(&self) -> f64 {
        let result = *self.max.lock().unwrap(); result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_safe_vector() {
        let result = thread_safe_vector();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_thread_safe_hashmap() {
        let map = thread_safe_hashmap();
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn test_concurrent_queue() {
        let queue = ConcurrentQueue::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_concurrent_stack() {
        let stack = ConcurrentStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_concurrent_counter() {
        let counter = ConcurrentCounter::new();
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 3);
        counter.decrement();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_concurrent_map_rwlock() {
        let map = concurrent_map_rwlock();
        assert!(map.len() > 0);
    }

    #[test]
    fn test_concurrent_queue_channels() {
        let result = concurrent_queue_channels();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_concurrent_set() {
        let set = ConcurrentSet::new();
        assert!(set.insert(1));
        assert!(set.insert(2));
        assert!(!set.insert(1)); // Already exists
        assert!(set.contains(&1));
        assert!(set.remove(&1));
        assert!(!set.contains(&1));
    }

    #[test]
    fn test_concurrent_buffer() {
        let buffer = ConcurrentBuffer::new(3);
        assert!(buffer.push(1));
        assert!(buffer.push(2));
        assert!(buffer.push(3));
        assert!(!buffer.push(4)); // Full
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_concurrent_stats() {
        let stats = ConcurrentStats::new();
        stats.add(10.0);
        stats.add(20.0);
        stats.add(30.0);
        assert_eq!(stats.mean(), 20.0);
        assert_eq!(stats.min(), 10.0);
        assert_eq!(stats.max(), 30.0);
    }
}
