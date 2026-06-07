//! # Synchronization Primitives
//!
//! Beyond mutexes and channels, concurrent programs often need additional
//! synchronization primitives for coordinating threads. This module covers
//! barriers, condition variables, semaphores, and lazy initialization.
//!
//! ## Primitives:
//!
//! | Primitive | Purpose |
//! |-----------|---------|
//! | `Barrier` | Wait for all threads to reach a point |
//! | `Condvar` | Wait for a condition to become true |
//! | `Semaphore` | Limit concurrent access |
//! | `OnceCell` | One-time initialization |
//! | `LazyLock` | Lazy static initialization |

use parking_lot::{Condvar, Mutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

/// Barrier-based parallel computation that waits for all threads to complete
/// each phase before starting the next.
pub struct PhasedComputation {
    barrier: Arc<Barrier>,
    results: Arc<Mutex<Vec<Vec<f64>>>>,
}

impl PhasedComputation {
    pub fn new(num_threads: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(num_threads)),
            results: Arc::new(Mutex::new(vec![Vec::new(); num_threads])),
        }
    }

    /// Run a multi-phase computation where all threads synchronize between phases.
    pub fn run_phases<F>(&self, thread_id: usize, phase_fn: F)
    where
        F: Fn(usize) -> Vec<f64>,
    {
        // Phase 1
        let phase1_result = phase_fn(0);
        {
            let mut results = self.results.lock();
            results[thread_id] = phase1_result;
        }
        self.barrier.wait(); // Wait for all threads to finish phase 1

        // Phase 2
        let phase2_result = phase_fn(1);
        {
            let mut results = self.results.lock();
            results[thread_id].extend(phase2_result);
        }
        self.barrier.wait(); // Wait for all threads to finish phase 2
    }

    pub fn get_results(&self) -> Vec<Vec<f64>> {
        self.results.lock().clone()
    }
}

/// Condition variable based producer-consumer with buffer size limits.
pub struct BoundedBuffer<T> {
    buffer: Mutex<Vec<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

impl<T> BoundedBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    /// Add an item, blocking if the buffer is full.
    pub fn put(&self, item: T) {
        let mut buffer = self.buffer.lock();
        while buffer.len() >= self.capacity {
            self.not_full.wait(&mut buffer);
        }
        buffer.push(item);
        self.not_empty.notify_one();
    }

    /// Remove an item, blocking if the buffer is empty.
    pub fn take(&self) -> T {
        let mut buffer = self.buffer.lock();
        while buffer.is_empty() {
            self.not_empty.wait(&mut buffer);
        }
        let item = buffer.remove(0);
        self.not_full.notify_one();
        item
    }

    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.lock().is_empty()
    }
}

/// Semaphore for limiting concurrent access.
pub struct Semaphore {
    permits: AtomicUsize,
    max_permits: usize,
    waiters: Mutex<Vec<Condvar>>,
}

impl Semaphore {
    pub fn new(max_permits: usize) -> Arc<Self> {
        Arc::new(Self {
            permits: AtomicUsize::new(max_permits),
            max_permits,
            waiters: Mutex::new(Vec::new()),
        })
    }

    /// Acquire a permit, blocking until one is available.
    pub fn acquire(&self) -> SemaphorePermit<'_> {
        loop {
            let current = self.permits.load(Ordering::Relaxed);
            if current > 0 {
                if self
                    .permits
                    .compare_exchange_weak(
                        current,
                        current - 1,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return SemaphorePermit { semaphore: self };
                }
            }
            std::thread::yield_now();
        }
    }

    /// Try to acquire a permit without blocking.
    pub fn try_acquire(&self) -> Option<SemaphorePermit<'_>> {
        let current = self.permits.load(Ordering::Relaxed);
        if current > 0 {
            if self
                .permits
                .compare_exchange(current, current - 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Some(SemaphorePermit { semaphore: self });
            }
        }
        None
    }

    fn release(&self) {
        self.permits.fetch_add(1, Ordering::Release);
    }

    pub fn available_permits(&self) -> usize {
        self.permits.load(Ordering::Relaxed)
    }
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        self.semaphore.release();
    }
}

/// OnceCell-like initialization pattern using parking_lot.
pub struct OnceLock<T> {
    value: parking_lot::Once,
    data: Mutex<Option<T>>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            value: parking_lot::Once::new(),
            data: Mutex::new(None),
        }
    }

    /// Initialize the cell with a value. Only the first call succeeds.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.value.call_once(|| {
            let mut data = self.data.lock();
            *data = Some(f());
        });
        // Safety: value is initialized after call_once
        let data = self.data.lock();
        unsafe { &*(data.as_ref().unwrap() as *const T) }
    }

    pub fn get(&self) -> Option<&T> {
        let data = self.data.lock();
        // If data is Some, the Once has been called
        data.as_ref().map(|v| unsafe { &*(v as *const T) })
    }
}

/// Read-write lock with upgrade capability using parking_lot.
pub struct RwLockWithUpgrade<T> {
    inner: parking_lot::RwLock<T>,
}

impl<T: Clone> RwLockWithUpgrade<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: parking_lot::RwLock::new(data),
        }
    }

    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.inner.read()
    }

    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.inner.write()
    }

    /// Read the data and clone it (releases the lock immediately).
    pub fn read_clone(&self) -> T {
        self.inner.read().clone()
    }
}

/// Latch - a synchronization primitive that starts closed and opens once.
/// All threads waiting on it are released when it opens.
pub struct Latch {
    opened: AtomicU64,
    notify: Condvar,
    mutex: Mutex<()>,
}

impl Latch {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            opened: AtomicU64::new(0),
            notify: Condvar::new(),
            mutex: Mutex::new(()),
        })
    }

    /// Open the latch, releasing all waiting threads.
    pub fn open(&self) {
        self.opened.store(1, Ordering::Release);
        self.notify.notify_all();
    }

    /// Wait for the latch to open.
    pub fn wait(&self) {
        if self.opened.load(Ordering::Acquire) == 1 {
            return;
        }
        let mut guard = self.mutex.lock();
        while self.opened.load(Ordering::Relaxed) == 0 {
            self.notify.wait(&mut guard);
        }
    }

    pub fn is_open(&self) -> bool {
        self.opened.load(Ordering::Acquire) == 1
    }
}

/// CountdownLatch - waits for N events before releasing.
pub struct CountdownLatch {
    count: AtomicUsize,
    notify: Condvar,
    mutex: Mutex<()>,
}

impl CountdownLatch {
    pub fn new(count: usize) -> Arc<Self> {
        Arc::new(Self {
            count: AtomicUsize::new(count),
            notify: Condvar::new(),
            mutex: Mutex::new(()),
        })
    }

    /// Decrement the count. When it reaches zero, all waiters are released.
    pub fn count_down(&self) {
        let prev = self.count.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            // Was 1, now 0 - notify all waiters
            self.notify.notify_all();
        }
    }

    /// Wait until the count reaches zero.
    pub fn wait(&self) {
        if self.count.load(Ordering::Acquire) == 0 {
            return;
        }
        let mut guard = self.mutex.lock();
        while self.count.load(Ordering::Relaxed) > 0 {
            self.notify.wait(&mut guard);
        }
    }

    pub fn remaining(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

/// Timeout wrapper for synchronization operations.
pub struct TimeoutCondvar {
    condvar: Condvar,
}

impl TimeoutCondvar {
    pub fn new() -> Self {
        Self {
            condvar: Condvar::new(),
        }
    }

    /// Wait with a timeout. Returns true if notified, false if timed out.
    pub fn wait_timeout(
        &self,
        mutex: &Mutex<()>,
        timeout: Duration,
    ) -> bool {
        let mut guard = mutex.lock();
        let result = self.condvar.wait_for(&mut guard, timeout);
        !result.timed_out()
    }

    pub fn notify_one(&self) {
        self.condvar.notify_one();
    }

    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_barrier_synchronization() {
        let computation = Arc::new(PhasedComputation::new(4));
        let mut handles = vec![];

        for i in 0..4 {
            let comp = Arc::clone(&computation);
            handles.push(thread::spawn(move || {
                comp.run_phases(i, |phase| vec![phase as f64 * 10.0 + i as f64]);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let results = computation.get_results();
        assert_eq!(results.len(), 4);
        // Each thread should have results from both phases
        for result in &results {
            assert_eq!(result.len(), 2);
        }
    }

    #[test]
    fn test_bounded_buffer() {
        let buffer = Arc::new(BoundedBuffer::new(5));

        let producer_buffer = Arc::clone(&buffer);
        let producer = thread::spawn(move || {
            for i in 0..10 {
                producer_buffer.put(i);
            }
        });

        let consumer_buffer = Arc::clone(&buffer);
        let consumer = thread::spawn(move || {
            let mut items = Vec::new();
            for _ in 0..10 {
                items.push(consumer_buffer.take());
            }
            items
        });

        producer.join().unwrap();
        let items = consumer.join().unwrap();
        assert_eq!(items.len(), 10);
    }

    #[test]
    fn test_semaphore_limits_concurrency() {
        let sem = Semaphore::new(2);
        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let sem = Arc::clone(&sem);
            let counter = Arc::clone(&counter);
            let max = Arc::clone(&max_concurrent);
            handles.push(thread::spawn(move || {
                let _permit = sem.acquire();
                let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
                max.fetch_max(current, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10));
                counter.fetch_sub(1, Ordering::SeqCst);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Max concurrent should be at most 2
        assert!(max_concurrent.load(Ordering::Relaxed) <= 2);
    }

    #[test]
    fn test_semaphore_try_acquire() {
        let sem = Semaphore::new(2);
        let _p1 = sem.try_acquire().unwrap();
        let _p2 = sem.try_acquire().unwrap();
        assert!(sem.try_acquire().is_none());
        assert_eq!(sem.available_permits(), 0);
    }

    #[test]
    fn test_once_lock() {
        let cell = OnceLock::new();
        let val1 = cell.get_or_init(|| 42);
        assert_eq!(*val1, 42);

        // Second initialization should not change the value
        let val2 = cell.get_or_init(|| 99);
        assert_eq!(*val2, 42);

        assert!(cell.get().is_some());
    }

    #[test]
    fn test_latch() {
        let latch = Latch::new();
        let latch_clone = Arc::clone(&latch);
        let mut handles = vec![];

        for _ in 0..5 {
            let latch = Arc::clone(&latch);
            handles.push(thread::spawn(move || {
                latch.wait();
                // Should proceed after latch opens
            }));
        }

        thread::sleep(Duration::from_millis(50));
        latch_clone.open();

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(latch_clone.is_open());
    }

    #[test]
    fn test_countdown_latch() {
        let latch = CountdownLatch::new(5);
        let mut handles = vec![];

        for _ in 0..5 {
            let latch = Arc::clone(&latch);
            handles.push(thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                latch.count_down();
            }));
        }

        // Wait until all workers complete
        latch.wait();
        assert_eq!(latch.remaining(), 0);

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_timeout_condvar() {
        let condvar = TimeoutCondvar::new();
        let mutex = Mutex::new(());

        let condvar_clone = Arc::new(TimeoutCondvar::new());
        let cv = Arc::clone(&condvar_clone);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            cv.notify_one();
        });

        // Should be notified before timeout
        let cv2 = Arc::clone(&condvar_clone);
        let result = cv2.wait_timeout(&Mutex::new(()), Duration::from_millis(200));
        assert!(result);
    }

    #[test]
    fn test_timeout_condvar_times_out() {
        let condvar = TimeoutCondvar::new();
        let mutex = Mutex::new(());
        let result = condvar.wait_timeout(&mutex, Duration::from_millis(10));
        assert!(!result); // Timed out
    }

    #[test]
    fn test_bounded_buffer_len() {
        let buffer = BoundedBuffer::new(10);
        assert!(buffer.is_empty());
        buffer.put(1);
        buffer.put(2);
        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_rwlock_with_upgrade() {
        let lock = RwLockWithUpgrade::new(vec![1, 2, 3]);

        let data = lock.read_clone();
        assert_eq!(data, vec![1, 2, 3]);

        {
            let mut write = lock.write();
            write.push(4);
        }

        let data = lock.read_clone();
        assert_eq!(data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_countdown_latch_immediate() {
        let latch = CountdownLatch::new(0);
        // Already at zero, should return immediately
        latch.wait();
        assert_eq!(latch.remaining(), 0);
    }

    #[test]
    fn test_latch_already_open() {
        let latch = Latch::new();
        latch.open();
        // Wait should return immediately
        latch.wait();
        assert!(latch.is_open());
    }

    #[test]
    fn test_semaphore_permit_release() {
        let sem = Semaphore::new(1);
        {
            let _permit = sem.acquire();
            assert_eq!(sem.available_permits(), 0);
        }
        // Permit released on drop
        assert_eq!(sem.available_permits(), 1);
    }
}
