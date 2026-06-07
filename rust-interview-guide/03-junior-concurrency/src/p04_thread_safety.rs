/// Problem: Thread Safety
///
/// Master Rust's thread safety concepts.
///
/// Key Concepts:
/// - Send trait
/// - Sync trait
/// - Thread-safe types
/// - Data race prevention
/// - Unsafe thread safety

use std::sync::{Arc, Mutex};
use std::thread;

/// Problem 1: Send trait
/// Types that can be sent between threads
pub fn send_example() -> i32 {
    let x = 42; // i32 is Send
    let handle = thread::spawn(move || x);
    handle.join().unwrap()
}

/// Problem 2: Sync trait
/// Types that can be shared between threads
pub fn sync_example() -> i32 {
    let x = Arc::new(42); // Arc<i32> is Sync
    let x_clone = Arc::clone(&x);

    let handle = thread::spawn(move || *x_clone);
    handle.join().unwrap()
}

/// Problem 3: Thread-safe counter
/// Use Arc<Mutex<T>> for thread-safe counter
pub fn thread_safe_counter() -> i32 {
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

/// Problem 4: Thread-safe vector
/// Use Arc<Mutex<Vec<T>>> for thread-safe vector
pub fn thread_safe_vector() -> Vec<i32> {
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

/// Problem 5: Thread-safe HashMap
/// Use Arc<Mutex<HashMap<K, V>>> for thread-safe HashMap
pub fn thread_safe_hashmap() -> std::collections::HashMap<String, i32> {
    use std::collections::HashMap;

    let map = Arc::new(Mutex::new(HashMap::new()));
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

/// Problem 6: Thread-safe string
/// Use Arc<Mutex<String>> for thread-safe string
pub fn thread_safe_string() -> String {
    let s = Arc::new(Mutex::new(String::new()));
    let mut handles = vec![];

    for i in 0..5 {
        let s = Arc::clone(&s);
        let handle = thread::spawn(move || {
            let mut s = s.lock().unwrap();
            s.push_str(&format!("{} ", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

let result =     s.lock().unwrap().clone(); result
}

/// Problem 7: Thread-safe with RwLock
/// Use RwLock for multiple readers
pub fn thread_safe_rwlock() -> i32 {
    use std::sync::RwLock;

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

    let result = data.read().unwrap().iter().sum(); result
}

/// Problem 8: Thread-safe with atomic
/// Use atomic types for simple operations
pub fn thread_safe_atomic() -> i32 {
    use std::sync::atomic::{AtomicI32, Ordering};

    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    counter.load(Ordering::SeqCst)
}

/// Problem 9: Thread-safe with channel
/// Use channels for thread-safe communication
pub fn thread_safe_channel() -> Vec<i32> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for i in 0..5 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            tx.send(i * 2).unwrap();
        });
        handles.push(handle);
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }

    rx.iter().collect()
}

/// Problem 10: Thread-safe with barrier
/// Use barrier for synchronization
pub fn thread_safe_barrier() -> Vec<i32> {
    use std::sync::Barrier;

    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            barrier.wait();
            i * 10
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Problem 11: Thread-safe with Once
/// Use Once for one-time initialization
pub fn thread_safe_once() -> i32 {
    use std::sync::Once;

    static mut VALUE: i32 = 0;
    static INIT: Once = Once::new();

    let mut handles = vec![];

    for _ in 0..5 {
        let handle = thread::spawn(|| {
            INIT.call_once(|| {
                unsafe {
                    VALUE = 42;
                }
            });
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    unsafe { VALUE }
}

/// Problem 12: Thread-safe data race prevention
/// Demonstrate data race prevention
pub fn thread_safe_data_race() -> i32 {
    let data = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let mut num = data.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *data.lock().unwrap(); result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_example() {
        assert_eq!(send_example(), 42);
    }

    #[test]
    fn test_sync_example() {
        assert_eq!(sync_example(), 42);
    }

    #[test]
    fn test_thread_safe_counter() {
        assert_eq!(thread_safe_counter(), 10);
    }

    #[test]
    fn test_thread_safe_vector() {
        let result = thread_safe_vector();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_thread_safe_hashmap() {
        let map = thread_safe_hashmap();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_thread_safe_string() {
        let s = thread_safe_string();
        assert!(s.len() > 0);
    }

    #[test]
    fn test_thread_safe_rwlock() {
        assert_eq!(thread_safe_rwlock(), 10); // 1+2+3+4 = 10
    }

    #[test]
    fn test_thread_safe_atomic() {
        assert_eq!(thread_safe_atomic(), 10);
    }

    #[test]
    fn test_thread_safe_channel() {
        let result = thread_safe_channel();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_thread_safe_barrier() {
        let result = thread_safe_barrier();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_thread_safe_once() {
        assert_eq!(thread_safe_once(), 42);
    }

    #[test]
    fn test_thread_safe_data_race() {
        assert_eq!(thread_safe_data_race(), 10);
    }
}
