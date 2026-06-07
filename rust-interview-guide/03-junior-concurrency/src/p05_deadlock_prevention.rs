/// Problem: Deadlock Prevention
///
/// Master deadlock prevention in Rust.
///
/// Key Concepts:
/// - What is a deadlock
/// - Lock ordering
/// - Timeout-based locking
/// - Lock-free algorithms
/// - Avoiding nested locks

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

/// Problem 1: Lock ordering
/// Prevent deadlock by always locking in the same order
pub fn lock_ordering() -> (i32, i32) {
    let a = Arc::new(Mutex::new(1));
    let b = Arc::new(Mutex::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let handle = thread::spawn(move || {
        // Always lock a before b
        let mut a = a_clone.lock().unwrap();
        let mut b = b_clone.lock().unwrap();
        *a += 1;
        *b += 1;
    });

    handle.join().unwrap();

    let result = (*a.lock().unwrap(), *b.lock().unwrap()); result
}

/// Problem 2: Avoid nested locks
/// Use separate scopes for locks
pub fn avoid_nested_locks() -> (i32, i32) {
    let a = Arc::new(Mutex::new(1));
    let b = Arc::new(Mutex::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let handle = thread::spawn(move || {
        {
            let mut a = a_clone.lock().unwrap();
            *a += 1;
        } // Lock on a is released here

        {
            let mut b = b_clone.lock().unwrap();
            *b += 1;
        } // Lock on b is released here
    });

    handle.join().unwrap();

    let result = (*a.lock().unwrap(), *b.lock().unwrap()); result
}

/// Problem 3: Try lock
/// Use try_lock to avoid blocking
pub fn try_lock() -> Option<i32> {
    let m = Arc::new(Mutex::new(42));

    let m_clone = Arc::clone(&m);
    let handle = thread::spawn(move || {
        // Hold the lock for a while
        let mut num = m_clone.lock().unwrap();
        *num = 100;
        thread::sleep(Duration::from_millis(100));
    });

    thread::sleep(Duration::from_millis(10));

    // Try to lock (non-blocking)
    let lock_result = m.try_lock(); match lock_result {
        Ok(num) => Some(*num),
        Err(_) => None, // Lock is held by other thread
    }
}

/// Problem 4: Timeout-based locking
/// Use timeout to avoid infinite blocking
pub fn timeout_lock() -> Option<i32> {
    let m = Arc::new(Mutex::new(42));

    let m_clone = Arc::clone(&m);
    let handle = thread::spawn(move || {
        let mut num = m_clone.lock().unwrap();
        *num = 100;
        thread::sleep(Duration::from_millis(100));
    });

    thread::sleep(Duration::from_millis(10));

    // Try to lock with timeout
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_millis(50) {
            return None; // Timeout
        }
        let lock_result = m.try_lock(); match lock_result {
            Ok(num) => return Some(*num),
            Err(_) => thread::sleep(Duration::from_millis(1)),
        }
    }
}

/// Problem 5: Lock-free counter
/// Use atomic operations instead of locks
pub fn lock_free_counter() -> i32 {
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

/// Problem 6: Lock-free stack
/// Use atomic operations for a lock-free stack
pub fn lock_free_stack() -> Vec<i32> {
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::Arc;

    struct Node {
        data: i32,
        next: *mut Node,
    }

    let head = Arc::new(AtomicPtr::new(std::ptr::null_mut()));
    let mut handles = vec![];

    for i in 0..5 {
        let head = Arc::clone(&head);
        let handle = thread::spawn(move || {
            let node = Box::into_raw(Box::new(Node {
                data: i,
                next: std::ptr::null_mut(),
            }));

            loop {
                let current = head.load(Ordering::SeqCst);
                unsafe { (*node).next = current; }
                if head.compare_exchange(current, node, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    break;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Collect results
    let mut result = Vec::new();
    let mut current = head.load(Ordering::SeqCst);
    while !current.is_null() {
        unsafe {
            result.push((*current).data);
            current = (*current).next;
        }
    }
    result
}

/// Problem 7: Avoid deadlock with channels
/// Use channels instead of shared state
pub fn avoid_deadlock_channels() -> Vec<i32> {
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

/// Problem 8: Read-write lock deadlock prevention
/// Use RwLock correctly
pub fn rwlock_deadlock_prevention() -> i32 {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers (no deadlock)
    for _ in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let data = data.read().unwrap();
            data.iter().sum::<i32>()
        });
        handles.push(handle);
    }

    // One writer (no deadlock with readers)
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

/// Problem 9: Deadlock detection (simulated)
/// Simulate deadlock detection
pub fn deadlock_detection() -> bool {
    let a = Arc::new(Mutex::new(1));
    let b = Arc::new(Mutex::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let handle = thread::spawn(move || {
        // This could deadlock if we lock in wrong order
        let _a = a_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _b = b_clone.lock().unwrap();
    });

    // Check if thread is still running after timeout
    thread::sleep(Duration::from_millis(50));
    !handle.is_finished()
}

/// Problem 10: Deadlock-free pattern
/// Use a pattern that prevents deadlocks
pub fn deadlock_free_pattern() -> (i32, i32) {
    let a = Arc::new(Mutex::new(1));
    let b = Arc::new(Mutex::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let handle = thread::spawn(move || {
        // Always lock in the same order
        let mut a = a_clone.lock().unwrap();
        let mut b = b_clone.lock().unwrap();
        *a += 1;
        *b += 1;
    });

    handle.join().unwrap();

    let result = (*a.lock().unwrap(), *b.lock().unwrap()); result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_ordering() {
        let (a, b) = lock_ordering();
        assert_eq!(a, 2);
        assert_eq!(b, 3);
    }

    #[test]
    fn test_avoid_nested_locks() {
        let (a, b) = avoid_nested_locks();
        assert_eq!(a, 2);
        assert_eq!(b, 3);
    }

    #[test]
    fn test_try_lock() {
        // May or may not get the lock
        let _ = try_lock();
    }

    #[test]
    fn test_timeout_lock() {
        // May or may not get the lock
        let _ = timeout_lock();
    }

    #[test]
    fn test_lock_free_counter() {
        assert_eq!(lock_free_counter(), 10);
    }

    #[test]
    fn test_lock_free_stack() {
        let result = lock_free_stack();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_avoid_deadlock_channels() {
        let result = avoid_deadlock_channels();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_rwlock_deadlock_prevention() {
        assert_eq!(rwlock_deadlock_prevention(), 10); // 1+2+3+4 = 10
    }

    #[test]
    fn test_deadlock_detection() {
        // This test may or may not detect a deadlock
        let _ = deadlock_detection();
    }

    #[test]
    fn test_deadlock_free_pattern() {
        let (a, b) = deadlock_free_pattern();
        assert_eq!(a, 2);
        assert_eq!(b, 3);
    }
}
