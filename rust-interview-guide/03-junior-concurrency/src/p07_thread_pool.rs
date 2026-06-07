/// Problem: Thread Pool
///
/// Build a simple thread pool.
///
/// Key Concepts:
/// - Thread pool design
/// - Task queue
/// - Worker threads
/// - Graceful shutdown

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// Problem 1: Basic thread pool
/// Create a simple thread pool
pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            let worker = thread::spawn(move || loop {
                let task = receiver.lock().unwrap().recv();
                match task {
                    Ok(task) => task(),
                    Err(_) => break,
                }
            });
            workers.push(worker);
        }

        Self { workers, sender }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close the channel by dropping the sender
        // We need to explicitly drop the sender to signal workers to stop
        let sender = std::mem::replace(&mut self.sender, mpsc::channel().0);
        drop(sender);

        // Wait for all workers to finish
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

/// Problem 2: Thread pool with results
/// Execute tasks that return results
pub struct ThreadPoolWithResults {
    pool: ThreadPool,
}

impl ThreadPoolWithResults {
    pub fn new(size: usize) -> Self {
        Self {
            pool: ThreadPool::new(size),
        }
    }

    pub fn execute<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        self.pool.execute(move || {
            let result = f();
            tx.send(result).unwrap();
        });
        rx.recv().unwrap()
    }
}

/// Problem 3: Thread pool with task queue
/// Demonstrate task queuing
pub fn thread_pool_task_queue() -> Vec<i32> {
    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    for i in 0..10 {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(i * 2).unwrap();
        });
    }

    drop(tx);
    rx.iter().collect()
}

/// Problem 4: Thread pool with shared state
/// Use thread pool with shared state
pub fn thread_pool_shared_state() -> i32 {
    let pool = ThreadPool::new(4);
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let (tx, rx) = mpsc::channel();
        pool.execute(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
            tx.send(()).unwrap();
        });
        handles.push(rx);
    }

    for rx in handles {
        rx.recv().unwrap();
    }

    let result = *counter.lock().unwrap();
    result
}

/// Problem 5: Thread pool with error handling
/// Handle errors in thread pool tasks
pub fn thread_pool_error_handling() -> Result<Vec<i32>, String> {
    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    for i in 0..5 {
        let tx = tx.clone();
        pool.execute(move || {
            if i == 3 {
                tx.send(Err(format!("Error at {}", i))).unwrap();
            } else {
                tx.send(Ok(i * 2)).unwrap();
            }
        });
    }

    drop(tx);

    let mut results = Vec::new();
    for result in rx {
        match result {
            Ok(v) => results.push(v),
            Err(e) => return Err(e),
        }
    }
    Ok(results)
}

/// Problem 6: Thread pool with timeout
/// Execute tasks with timeout
pub fn thread_pool_timeout() -> Option<i32> {
    use std::time::Duration;

    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    pool.execute(move || {
        thread::sleep(Duration::from_millis(100));
        let _ = tx.send(42);
    });

    let result = rx.recv_timeout(Duration::from_millis(50)).ok();
    drop(pool); // Ensure pool is dropped after we're done
    result
}

/// Problem 7: Thread pool with multiple tasks
/// Execute multiple tasks concurrently
pub fn thread_pool_multiple_tasks() -> Vec<i32> {
    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    for i in 0..10 {
        let tx = tx.clone();
        pool.execute(move || {
            let result = i * i;
            tx.send(result).unwrap();
        });
    }

    drop(tx);
    let mut results: Vec<i32> = rx.iter().collect();
    results.sort();
    results
}

/// Problem 8: Thread pool with graceful shutdown
/// Demonstrate graceful shutdown
pub fn thread_pool_graceful_shutdown() -> Vec<i32> {
    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    for i in 0..5 {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(i * 2).unwrap();
        });
    }

    drop(tx);
    drop(pool); // Graceful shutdown

    rx.iter().collect()
}

/// Problem 9: Thread pool with task dependencies
/// Execute tasks with dependencies
pub fn thread_pool_dependencies() -> i32 {
    let pool = ThreadPool::new(4);
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Task 1: Compute base value
    pool.execute(move || {
        let base = 10;
        tx1.send(base).unwrap();
    });

    // Task 2: Use result from task 1
    let base = rx1.recv().unwrap();
    pool.execute(move || {
        let result = base * 2;
        tx2.send(result).unwrap();
    });

    rx2.recv().unwrap()
}

/// Problem 10: Thread pool with work stealing
/// Simulate work stealing pattern
pub fn thread_pool_work_stealing() -> Vec<i32> {
    let pool = ThreadPool::new(4);
    let (tx, rx) = mpsc::channel();

    // Create tasks with varying workloads
    for i in 0..10 {
        let tx = tx.clone();
        pool.execute(move || {
            // Simulate varying work
            let work = i * 10;
            tx.send(work).unwrap();
        });
    }

    drop(tx);
    rx.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);
        let (tx, rx) = mpsc::channel();

        for i in 0..10 {
            let tx = tx.clone();
            pool.execute(move || {
                tx.send(i * 2).unwrap();
            });
        }

        drop(tx);
        let results: Vec<i32> = rx.iter().collect();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_thread_pool_with_results() {
        let pool = ThreadPoolWithResults::new(4);
        let result = pool.execute(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_pool_task_queue() {
        let result = thread_pool_task_queue();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_thread_pool_shared_state() {
        assert_eq!(thread_pool_shared_state(), 10);
    }

    #[test]
    fn test_thread_pool_error_handling() {
        assert!(thread_pool_error_handling().is_err());
    }

    #[test]
    fn test_thread_pool_timeout() {
        // May or may not timeout
        let _ = thread_pool_timeout();
    }

    #[test]
    fn test_thread_pool_multiple_tasks() {
        let result = thread_pool_multiple_tasks();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_thread_pool_graceful_shutdown() {
        let result = thread_pool_graceful_shutdown();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_thread_pool_dependencies() {
        assert_eq!(thread_pool_dependencies(), 20);
    }

    #[test]
    fn test_thread_pool_work_stealing() {
        let result = thread_pool_work_stealing();
        assert_eq!(result.len(), 10);
    }
}
