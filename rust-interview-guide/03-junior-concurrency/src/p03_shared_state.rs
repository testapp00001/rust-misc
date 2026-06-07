/// Problem: Shared State
///
/// Master Rust's shared state concurrency.
///
/// Key Concepts:
/// - Mutex for mutual exclusion
/// - Arc for thread-safe reference counting
/// - RwLock for reader-writer locks
/// - Deadlock prevention
/// - Lock granularity

use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Problem 1: Basic Mutex
/// Use Mutex to protect shared data
pub fn basic_mutex() -> i32 {
    let m = Mutex::new(5);

    {
        let mut num = m.lock().unwrap();
        *num = 10;
    }

    let result = *m.lock().unwrap(); result
}

/// Problem 2: Arc<Mutex<T>>
/// Share Mutex across threads
pub fn arc_mutex() -> i32 {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *counter.lock().unwrap();
    result
}

/// Problem 3: Mutex with complex data
/// Protect a HashMap with Mutex
pub fn mutex_hashmap() -> std::collections::HashMap<String, i32> {
    let map = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let mut handles = vec![];

    for i in 0..5 {
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

/// Problem 4: RwLock
/// Use RwLock for multiple readers
pub fn rwlock() -> i32 {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Readers
    for _ in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let data = data.read().unwrap();
            data.iter().sum::<i32>()
        });
        handles.push(handle);
    }

    // Writer
    let data_clone = Arc::clone(&data);
    let writer_handle = thread::spawn(move || {
        let mut data = data_clone.write().unwrap();
        data.push(4);
    });

    // Wait for writer
    writer_handle.join().unwrap();

    // Wait for readers
    for handle in handles {
        handle.join().unwrap();
    }

    let result = data.read().unwrap().iter().sum();
    result
}

/// Problem 5: Mutex with Vec
/// Protect a Vec with Mutex
pub fn mutex_vec() -> Vec<i32> {
    let vec = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for i in 0..5 {
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

/// Problem 6: Mutex with custom type
/// Protect a custom struct with Mutex
#[derive(Debug, Clone)]
pub struct Counter {
    pub value: i32,
    pub name: String,
}

pub fn mutex_custom_type() -> Counter {
    let counter = Arc::new(Mutex::new(Counter {
        value: 0,
        name: "counter".to_string(),
    }));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut counter = counter.lock().unwrap();
            counter.value += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = counter.lock().unwrap().clone();
    result
}

/// Problem 7: Lock granularity
/// Demonstrate lock granularity
pub fn lock_granularity() -> i32 {
    let data = Arc::new(Mutex::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];

    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            // Lock for minimal time
            let mut data = data.lock().unwrap();
            data[i] *= 2;
            // Lock is dropped here
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = data.lock().unwrap().iter().sum(); result
}

/// Problem 8: Mutex with error handling
/// Handle mutex poisoning
pub fn mutex_error_handling() -> Result<i32, String> {
    let m = Arc::new(Mutex::new(0));

    let m_clone = Arc::clone(&m);
    let handle = thread::spawn(move || {
        let mut num = m_clone.lock().unwrap();
        *num = 42;
    });

    handle.join().unwrap();

    m.lock().map(|num| *num).map_err(|_| "Mutex poisoned".to_string())
}

/// Problem 9: RwLock with multiple operations
/// Use RwLock for complex operations
pub fn rwlock_complex() -> Vec<i32> {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers
    for _ in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let data = data.read().unwrap();
            data.clone()
        });
        handles.push(handle);
    }

    // One writer
    let data_clone = Arc::clone(&data);
    let writer_handle = thread::spawn(move || {
        let mut data = data_clone.write().unwrap();
        data.push(4);
        data.push(5);
    });

    // Wait for writer
    writer_handle.join().unwrap();

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.join() {
            results.push(result);
        }
    }

    results.into_iter().flatten().collect()
}

/// Problem 10: Mutex with condition variable
/// Use Condvar for signaling
pub fn mutex_condvar() -> i32 {
    use std::sync::Condvar;

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }

    handle.join().unwrap();
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mutex() {
        assert_eq!(basic_mutex(), 10);
    }

    #[test]
    fn test_arc_mutex() {
        assert_eq!(arc_mutex(), 10);
    }

    #[test]
    fn test_mutex_hashmap() {
        let map = mutex_hashmap();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_rwlock() {
        assert_eq!(rwlock(), 10); // 1+2+3+4 = 10
    }

    #[test]
    fn test_mutex_vec() {
        let result = mutex_vec();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_mutex_custom_type() {
        let counter = mutex_custom_type();
        assert_eq!(counter.value, 10);
        assert_eq!(counter.name, "counter");
    }

    #[test]
    fn test_lock_granularity() {
        assert_eq!(lock_granularity(), 30); // (1+2+3+4+5) * 2
    }

    #[test]
    fn test_mutex_error_handling() {
        assert_eq!(mutex_error_handling(), Ok(42));
    }

    #[test]
    fn test_rwlock_complex() {
        let result = rwlock_complex();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_mutex_condvar() {
        assert_eq!(mutex_condvar(), 42);
    }
}
