/// Problem: Thread Basics
///
/// Master Rust's thread system.
///
/// Key Concepts:
/// - Thread spawning
/// - Thread joining
/// - Thread closures
/// - Thread panics
/// - Scoped threads

use std::sync::Arc;

use std::thread;
use std::time::Duration;

/// Problem 1: Basic thread spawn
/// Spawn a thread and wait for it
pub fn basic_thread() -> i32 {
    let handle = thread::spawn(|| {
        42
    });

    handle.join().unwrap()
}

/// Problem 2: Thread with move
/// Move data into a thread
pub fn thread_with_move() -> String {
    let s = String::from("hello");
    let handle = thread::spawn(move || {
        s // s is moved into the thread
    });

    handle.join().unwrap()
}

/// Problem 3: Multiple threads
/// Spawn multiple threads
pub fn multiple_threads() -> Vec<i32> {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = thread::spawn(move || {
            i * 2
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Problem 4: Thread with sleep
/// Thread that sleeps
pub fn thread_with_sleep() -> i32 {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
        42
    });

    handle.join().unwrap()
}

/// Problem 5: Thread panic handling
/// Handle thread panics
pub fn thread_panic() -> Result<i32, String> {
    let handle = thread::spawn(|| {
        panic!("Something went wrong");
    });

    match handle.join() {
        Ok(val) => Ok(val),
        Err(_) => Err("Thread panicked".to_string()),
    }
}

/// Problem 6: Thread with shared data
/// Use Arc for shared data
pub fn thread_with_shared_data() -> i32 {
    use std::sync::Arc;

    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            data[i] * 2
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// Problem 7: Scoped threads
/// Use scoped threads for borrowing
pub fn scoped_threads() -> Vec<i32> {
    let data = vec![1, 2, 3, 4, 5];
    let data = Arc::new(data);
    let mut handles = vec![];

    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = std::thread::spawn(move || data[i] * 2);
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Problem 8: Thread builder
/// Use thread builder for configuration
pub fn thread_builder() -> i32 {
    let builder = thread::Builder::new()
        .name("my-thread".to_string())
        .stack_size(1024 * 1024); // 1MB stack

    let handle = builder.spawn(|| {
        42
    }).unwrap();

    handle.join().unwrap()
}

/// Problem 9: Thread with channels
/// Use channels for communication
pub fn thread_with_channels() -> i32 {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(42).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 10: Thread with barrier
/// Use barrier to synchronize threads
pub fn thread_with_barrier() -> Vec<i32> {
    use std::sync::{Arc, Barrier};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_thread() {
        assert_eq!(basic_thread(), 42);
    }

    #[test]
    fn test_thread_with_move() {
        assert_eq!(thread_with_move(), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let result = multiple_threads();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_thread_with_sleep() {
        assert_eq!(thread_with_sleep(), 42);
    }

    #[test]
    fn test_thread_panic() {
        assert!(thread_panic().is_err());
    }

    #[test]
    fn test_thread_with_shared_data() {
        assert_eq!(thread_with_shared_data(), 12); // 2 + 4 + 6
    }

    #[test]
    fn test_scoped_threads() {
        let result = scoped_threads();
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_thread_builder() {
        assert_eq!(thread_builder(), 42);
    }

    #[test]
    fn test_thread_with_channels() {
        assert_eq!(thread_with_channels(), 42);
    }

    #[test]
    fn test_thread_with_barrier() {
        let result = thread_with_barrier();
        assert_eq!(result.len(), 3);
    }
}
