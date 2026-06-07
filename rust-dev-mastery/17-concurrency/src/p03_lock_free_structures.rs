//! # Lock-Free Data Structures
//!
//! Lock-free data structures use atomic operations instead of locks to achieve
//! thread safety. They offer better performance under contention but are more
//! complex to implement correctly.
//!
//! ## Key Concepts:
//!
//! - **Compare-and-Swap (CAS)**: Atomic operation that updates a value only if it
//!   matches an expected value
//! - **ABA Problem**: A value changes A -> B -> A, CAS succeeds but state is wrong
//! - **Memory Ordering**: How operations are ordered across threads
//! - **crossbeam-epoch**: Epoch-based garbage collection for lock-free structures
//!
//! ## Atomic Types:
//!
//! | Type | Use Case |
//! |------|----------|
//! | `AtomicBool` | Flags, state machines |
//! | `AtomicU64` | Counters, IDs |
//! | `AtomicPtr` | Lock-free pointers |
//! | `AtomicUsize` | Sizes, indices |

use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free counter using atomic operations.
/// This is the simplest example of lock-free programming.
pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }

    /// Atomically increment and return the new value.
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Atomically add and return the previous value.
    pub fn add(&self, amount: u64) -> u64 {
        self.value.fetch_add(amount, Ordering::Relaxed)
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Compare-and-swap: only update if current value matches `current`.
    pub fn compare_and_swap(&self, current: u64, new: u64) -> Result<u64, u64> {
        match self
            .value
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(prev) => Ok(prev),
            Err(actual) => Err(actual),
        }
    }
}

/// Lock-free flag using AtomicBool.
pub struct AtomicFlag {
    flag: AtomicBool,
}

impl AtomicFlag {
    pub fn new(initial: bool) -> Self {
        Self {
            flag: AtomicBool::new(initial),
        }
    }

    /// Set the flag and return the previous value.
    pub fn set(&self) -> bool {
        self.flag.swap(true, Ordering::SeqCst)
    }

    /// Clear the flag and return the previous value.
    pub fn clear(&self) -> bool {
        self.flag.swap(false, Ordering::SeqCst)
    }

    /// Check if the flag is set.
    pub fn is_set(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Test-and-set: atomically set and return whether it was already set.
    pub fn test_and_set(&self) -> bool {
        self.flag.swap(true, Ordering::SeqCst)
    }
}

/// Lock-free ID generator using atomic operations.
pub struct IdGenerator {
    current: AtomicU64,
}

impl IdGenerator {
    pub fn new(start: u64) -> Self {
        Self {
            current: AtomicU64::new(start),
        }
    }

    /// Generate the next unique ID.
    pub fn next(&self) -> u64 {
        self.current.fetch_add(1, Ordering::Relaxed)
    }

    /// Generate multiple IDs at once, returning the start of the range.
    pub fn next_batch(&self, count: u64) -> u64 {
        self.current.fetch_add(count, Ordering::Relaxed)
    }
}

/// Lock-free Treiber stack (LIFO).
/// A classic lock-free data structure using a singly-linked list with CAS.
pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
    len: AtomicUsize,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    /// Push an element onto the stack.
    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: std::ptr::null_mut(),
        }));

        loop {
            let current_head = self.head.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = current_head;
            }

            match self.head.compare_exchange_weak(
                current_head,
                new_node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.len.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(_) => continue, // Retry
            }
        }
    }

    /// Pop an element from the stack.
    pub fn pop(&self) -> Option<T> {
        loop {
            let current_head = self.head.load(Ordering::Acquire);
            if current_head.is_null() {
                return None;
            }

            let next = unsafe { (*current_head).next };

            match self.head.compare_exchange_weak(
                current_head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.len.fetch_sub(1, Ordering::Relaxed);
                    let node = unsafe { Box::from_raw(current_head) };
                    return Some(node.data);
                }
                Err(_) => continue, // Retry
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Lock-free queue implementation using crossbeam channels.
/// A proper Michael-Scott queue requires careful unsafe code;
/// this demonstrates the concept using crossbeam as the backend.
pub struct LockFreeQueue<T> {
    sender: crossbeam::channel::Sender<T>,
    receiver: crossbeam::channel::Receiver<T>,
    len: AtomicUsize,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded();
        Self {
            sender,
            receiver,
            len: AtomicUsize::new(0),
        }
    }

    /// Enqueue an element.
    pub fn enqueue(&self, data: T) {
        let _ = self.sender.send(data);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    /// Dequeue an element.
    pub fn dequeue(&self) -> Option<T> {
        let result = self.receiver.try_recv().ok();
        if result.is_some() {
            self.len.fetch_sub(1, Ordering::Relaxed);
        }
        result
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Lock-free spin lock using AtomicBool.
/// Simple but only suitable for very short critical sections.
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    /// Acquire the spin lock by spinning until it becomes available.
    pub fn lock(&self) -> SpinLockGuard<'_> {
        loop {
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return SpinLockGuard { lock: self };
            }
            // Spin hint (x86 PAUSE instruction)
            std::hint::spin_loop();
        }
    }

    /// Try to acquire the lock without spinning.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }
}

pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
}

impl<'a> Drop for SpinLockGuard<'a> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

/// Lock-free epoch-based reclamation hint.
/// In production, use crossbeam-epoch for safe memory reclamation.
pub struct EpochGuard {
    _private: (),
}

impl EpochGuard {
    pub fn pin() -> Self {
        // In a real implementation, this would pin the current thread
        // to an epoch using crossbeam-epoch
        Self { _private: () }
    }
}

/// Lock-free statistics collector.
pub struct AtomicStats {
    count: AtomicU64,
    sum: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicStats {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    /// Record a value (thread-safe, lock-free).
    pub fn record(&self, value: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);

        // Update min with CAS loop
        loop {
            let current_min = self.min.load(Ordering::Relaxed);
            if value >= current_min {
                break;
            }
            if self
                .min
                .compare_exchange_weak(current_min, value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Update max with CAS loop
        loop {
            let current_max = self.max.load(Ordering::Relaxed);
            if value <= current_max {
                break;
            }
            if self
                .max
                .compare_exchange_weak(current_max, value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            return 0.0;
        }
        self.sum() as f64 / count as f64
    }

    pub fn min(&self) -> u64 {
        let m = self.min.load(Ordering::Relaxed);
        if m == u64::MAX { 0 } else { m }
    }

    pub fn max(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_atomic_counter_concurrent() {
        let counter = Arc::new(AtomicCounter::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    counter.increment();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 10_000);
    }

    #[test]
    fn test_atomic_counter_cas() {
        let counter = AtomicCounter::new(10);
        let result = counter.compare_and_swap(10, 20);
        assert!(result.is_ok());
        assert_eq!(counter.get(), 20);

        let result = counter.compare_and_swap(10, 30);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 20);
    }

    #[test]
    fn test_atomic_flag() {
        let flag = AtomicFlag::new(false);
        assert!(!flag.is_set());

        let was_set = flag.test_and_set();
        assert!(!was_set); // Was not set before
        assert!(flag.is_set());

        let was_set = flag.test_and_set();
        assert!(was_set); // Was already set

        flag.clear();
        assert!(!flag.is_set());
    }

    #[test]
    fn test_id_generator_unique() {
        let gen = Arc::new(IdGenerator::new(0));
        let mut handles = vec![];
        let ids = Arc::new(std::sync::Mutex::new(Vec::new()));

        for _ in 0..10 {
            let gen = Arc::clone(&gen);
            let ids = Arc::clone(&ids);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let id = gen.next();
                    ids.lock().unwrap().push(id);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let ids = ids.lock().unwrap();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 1000); // All unique
    }

    #[test]
    fn test_id_generator_batch() {
        let gen = IdGenerator::new(0);
        let start = gen.next_batch(10);
        assert_eq!(start, 0);
        assert_eq!(gen.next(), 10);
    }

    #[test]
    fn test_lock_free_stack_push_pop() {
        let stack = LockFreeStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_lock_free_stack_concurrent() {
        let stack = Arc::new(LockFreeStack::new());
        let mut handles = vec![];

        // Push from multiple threads
        for i in 0..5 {
            let stack = Arc::clone(&stack);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    stack.push(i * 100 + j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(stack.len(), 500);

        // Pop all
        let mut count = 0;
        while stack.pop().is_some() {
            count += 1;
        }
        assert_eq!(count, 500);
    }

    #[test]
    fn test_atomic_stats_basic() {
        let stats = AtomicStats::new();
        stats.record(10);
        stats.record(20);
        stats.record(30);

        assert_eq!(stats.count(), 3);
        assert_eq!(stats.sum(), 60);
        assert!((stats.mean() - 20.0).abs() < f64::EPSILON);
        assert_eq!(stats.min(), 10);
        assert_eq!(stats.max(), 30);
    }

    #[test]
    fn test_atomic_stats_concurrent() {
        let stats = Arc::new(AtomicStats::new());
        let mut handles = vec![];

        for i in 0..10 {
            let stats = Arc::clone(&stats);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    stats.record(i * 100 + j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(stats.count(), 1000);
        assert!(stats.mean() > 0.0);
        assert!(stats.min() <= stats.max());
    }

    #[test]
    fn test_atomic_stats_empty() {
        let stats = AtomicStats::new();
        assert_eq!(stats.count(), 0);
        assert_eq!(stats.mean(), 0.0);
        assert_eq!(stats.min(), 0);
        assert_eq!(stats.max(), 0);
    }

    #[test]
    fn test_spin_lock_basic() {
        let lock = SpinLock::new();
        {
            let guard = lock.lock();
            // Lock is held
            assert!(lock.try_lock().is_none());
        }
        // Lock is released
        assert!(lock.try_lock().is_some());
    }

    #[test]
    fn test_spin_lock_try_lock() {
        let lock = SpinLock::new();
        let guard = lock.lock();
        assert!(lock.try_lock().is_none());
        drop(guard);
        assert!(lock.try_lock().is_some());
    }

    #[test]
    fn test_memory_ordering_relaxed() {
        let val = AtomicU64::new(0);
        val.store(42, Ordering::Relaxed);
        assert_eq!(val.load(Ordering::Relaxed), 42);
    }

    #[test]
    fn test_memory_ordering_seq_cst() {
        let val = AtomicU64::new(0);
        val.store(42, Ordering::SeqCst);
        assert_eq!(val.load(Ordering::SeqCst), 42);
    }
}
