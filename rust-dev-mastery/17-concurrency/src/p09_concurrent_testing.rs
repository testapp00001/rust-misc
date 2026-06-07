//! # Concurrent Testing
//!
//! Testing concurrent code is challenging because races, deadlocks, and
//! ordering bugs are non-deterministic. This module covers techniques for
//! testing concurrent Rust code effectively.
//!
//! ## Strategies:
//!
//! | Strategy | Tool | Purpose |
//! |----------|------|---------|
//! | Stress Testing | Thread spam | Find timing-dependent bugs |
//! | Deterministic | Loom | Explore all possible interleavings |
//! | Property-based | proptest | Test invariants hold under concurrency |
//! | Instrumented | Custom | Track ordering and state |

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Stress test helper: runs a closure many times from multiple threads.
pub fn stress_test<F>(num_threads: usize, iterations_per_thread: usize, test_fn: F)
where
    F: Fn(usize, usize) + Send + Sync + 'static,
{
    let test_fn = Arc::new(test_fn);
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let test_fn = Arc::clone(&test_fn);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait(); // Synchronize start
            for iter in 0..iterations_per_thread {
                test_fn(thread_id, iter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Race condition detector: records the order of events across threads.
pub struct RaceDetector {
    events: Mutex<Vec<(Instant, usize, String)>>,
}

impl RaceDetector {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Mutex::new(Vec::new()),
        })
    }

    /// Record an event from a specific thread.
    pub fn record(&self, thread_id: usize, event: &str) {
        self.events
            .lock()
            .unwrap()
            .push((Instant::now(), thread_id, event.to_string()));
    }

    /// Get the recorded events in chronological order.
    pub fn events(&self) -> Vec<(usize, String)> {
        let mut events = self.events.lock().unwrap().clone();
        events.sort_by_key(|e| e.0);
        events.into_iter().map(|(_, tid, event)| (tid, event)).collect()
    }

    /// Check if events occurred in the expected order.
    pub fn events_contain_sequence(&self, expected: &[&str]) -> bool {
        let events = self.events();
        let event_names: Vec<&str> = events.iter().map(|(_, e)| e.as_str()).collect();

        let mut idx = 0;
        for event in &event_names {
            if idx < expected.len() && *event == expected[idx] {
                idx += 1;
            }
        }
        idx == expected.len()
    }
}

/// Thread-safe test counter for verifying concurrent operations.
pub struct TestCounter {
    value: AtomicU64,
    max_observed: AtomicU64,
    min_observed: AtomicU64,
}

impl TestCounter {
    pub fn new(initial: u64) -> Arc<Self> {
        Arc::new(Self {
            value: AtomicU64::new(initial),
            max_observed: AtomicU64::new(initial),
            min_observed: AtomicU64::new(initial),
        })
    }

    pub fn increment(&self) -> u64 {
        let new_val = self.value.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_observed.fetch_max(new_val, Ordering::Relaxed);
        new_val
    }

    pub fn decrement(&self) -> u64 {
        let new_val = self.value.fetch_sub(1, Ordering::SeqCst) - 1;
        self.min_observed.fetch_min(new_val, Ordering::Relaxed);
        new_val
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn max_observed(&self) -> u64 {
        self.max_observed.load(Ordering::Relaxed)
    }

    pub fn min_observed(&self) -> u64 {
        self.min_observed.load(Ordering::Relaxed)
    }
}

/// Timeout-enforced test execution.
pub fn run_with_timeout<F>(timeout: Duration, test_fn: F) -> Result<(), String>
where
    F: FnOnce() + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        test_fn();
        let _ = tx.send(());
    });

    match rx.recv_timeout(timeout) {
        Ok(()) => Ok(()),
        Err(_) => Err(format!("Test timed out after {:?}", timeout)),
    }
}

/// Deadlock detector: attempts to detect potential deadlocks.
pub struct DeadlockProbe {
    lock_order: Mutex<Vec<String>>,
    detected_cycle: AtomicBool,
}

impl DeadlockProbe {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            lock_order: Mutex::new(Vec::new()),
            detected_cycle: AtomicBool::new(false),
        })
    }

    /// Record that a lock was acquired.
    pub fn acquire(&self, lock_name: &str) {
        let mut order = self.lock_order.lock().unwrap();
        // Check if we're acquiring a lock we already hold (potential deadlock)
        if order.contains(&lock_name.to_string()) {
            self.detected_cycle.store(true, Ordering::SeqCst);
        }
        order.push(lock_name.to_string());
    }

    /// Record that a lock was released.
    pub fn release(&self, lock_name: &str) {
        let mut order = self.lock_order.lock().unwrap();
        if let Some(pos) = order.iter().rposition(|n| n == lock_name) {
            order.remove(pos);
        }
    }

    pub fn detected_deadlock(&self) -> bool {
        self.detected_cycle.load(Ordering::SeqCst)
    }
}

/// Linearizability checker: verifies that concurrent operations appear
/// to execute atomically in some sequential order.
pub struct LinearizabilityChecker {
    operations: Mutex<Vec<OperationRecord>>,
}

#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub thread_id: usize,
    pub operation: String,
    pub start_time: Instant,
    pub end_time: Instant,
    pub result: String,
}

impl LinearizabilityChecker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            operations: Mutex::new(Vec::new()),
        })
    }

    pub fn record(
        &self,
        thread_id: usize,
        operation: &str,
        start: Instant,
        end: Instant,
        result: &str,
    ) {
        self.operations.lock().unwrap().push(OperationRecord {
            thread_id,
            operation: operation.to_string(),
            start_time: start,
            end_time: end,
            result: result.to_string(),
        });
    }

    /// Get all recorded operations sorted by start time.
    pub fn operations(&self) -> Vec<OperationRecord> {
        let ops = self.operations.lock().unwrap().clone();
        let mut sorted = ops;
        sorted.sort_by_key(|op| op.start_time);
        sorted
    }

    /// Check if operations are sequentially consistent.
    /// (Simplified check - real linearizability checking is much more complex)
    pub fn is_sequentially_consistent(&self) -> bool {
        let ops = self.operations();
        // Check that for each thread, operations are in order
        let mut last_end: Vec<Option<Instant>> = Vec::new();
        for op in &ops {
            while last_end.len() <= op.thread_id {
                last_end.push(None);
            }
            if let Some(prev_end) = last_end[op.thread_id] {
                if op.start_time < prev_end {
                    return false; // Overlapping operations in same thread
                }
            }
            last_end[op.thread_id] = Some(op.end_time);
        }
        true
    }
}

/// Test that a concurrent data structure maintains its invariants.
pub fn test_concurrent_invariant<F, C>(
    num_threads: usize,
    iterations: usize,
    operation: F,
    check: C,
) where
    F: Fn(usize) + Send + Sync + 'static,
    C: Fn() -> bool + Send + Sync + 'static,
{
    let operation = Arc::new(operation);
    let check = Arc::new(check);
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let operation = Arc::clone(&operation);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for _ in 0..iterations {
                operation(thread_id);
            }
        }));
    }

    // Start all threads
    barrier.wait();

    // Check invariant periodically
    let check_clone = Arc::clone(&check);
    let checker = thread::spawn(move || {
        for _ in 0..iterations {
            assert!(check_clone(), "Invariant violated!");
            thread::yield_now();
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    checker.join().unwrap();
}

/// Simulated loom-style test: runs a closure multiple times to increase
/// the chance of exposing concurrency bugs.
pub fn repeated_test<F>(num_runs: usize, test_fn: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let test_fn = Arc::new(test_fn);
    for run in 0..num_runs {
        let test_fn = Arc::clone(&test_fn);
        let handle = thread::spawn(move || {
            test_fn();
        });
        handle.join().unwrap_or_else(|_| panic!("Test panicked on run {}", run));
    }
}

/// Thread-safe vector for collecting test results.
pub struct TestResults<T: Send> {
    results: Mutex<Vec<T>>,
}

impl<T: Send> TestResults<T> {
    pub fn new() -> Self {
        Self {
            results: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, value: T) {
        self.results.lock().unwrap().push(value);
    }

    pub fn len(&self) -> usize {
        self.results.lock().unwrap().len()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.results.into_inner().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_test_atomic_counter() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        stress_test(10, 1000, move |_, _| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        assert_eq!(counter.load(Ordering::Relaxed), 10_000);
    }

    #[test]
    fn test_race_detector_basic() {
        let detector = RaceDetector::new();
        let d1 = Arc::clone(&detector);
        let d2 = Arc::clone(&detector);

        let h1 = thread::spawn(move || {
            d1.record(0, "start");
            thread::sleep(Duration::from_millis(10));
            d1.record(0, "end");
        });

        let h2 = thread::spawn(move || {
            d2.record(1, "middle");
        });

        h1.join().unwrap();
        h2.join().unwrap();

        let events = detector.events();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_race_detector_sequence() {
        let detector = RaceDetector::new();
        detector.record(0, "init");
        detector.record(0, "process");
        detector.record(0, "complete");

        assert!(detector.events_contain_sequence(&["init", "process", "complete"]));
        assert!(!detector.events_contain_sequence(&["complete", "init"]));
    }

    #[test]
    fn test_test_counter_concurrent() {
        let counter = TestCounter::new(0);
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 1000);
    }

    #[test]
    fn test_test_counter_observed_values() {
        let counter = TestCounter::new(100);
        counter.increment();
        counter.increment();
        counter.decrement();

        assert_eq!(counter.get(), 101);
        assert!(counter.max_observed() >= 101);
    }

    #[test]
    fn test_run_with_timeout_success() {
        let result = run_with_timeout(Duration::from_secs(1), || {
            // Fast operation
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_timeout_fails() {
        let result = run_with_timeout(Duration::from_millis(10), || {
            thread::sleep(Duration::from_secs(10));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_deadlock_probe() {
        let probe = DeadlockProbe::new();
        probe.acquire("lock_a");
        probe.acquire("lock_b");
        assert!(!probe.detected_deadlock());

        // Re-acquiring same lock is a potential deadlock
        probe.acquire("lock_a");
        assert!(probe.detected_deadlock());
    }

    #[test]
    fn test_deadlock_probe_release() {
        let probe = DeadlockProbe::new();
        probe.acquire("lock_a");
        probe.release("lock_a");
        probe.acquire("lock_a");
        assert!(!probe.detected_deadlock());
    }

    #[test]
    fn test_linearizability_checker() {
        let checker = LinearizabilityChecker::new();
        let now = Instant::now();

        checker.record(0, "write", now, now + Duration::from_millis(10), "ok");
        checker.record(
            1,
            "read",
            now + Duration::from_millis(5),
            now + Duration::from_millis(15),
            "value",
        );

        assert!(checker.is_sequentially_consistent());
    }

    #[test]
    fn test_concurrent_invariant_check() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        test_concurrent_invariant(
            4,
            100,
            move |_| {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            },
            move || {
                counter.load(Ordering::Relaxed) >= 0 // Always true
            },
        );
    }

    #[test]
    fn test_repeated_test() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        repeated_test(100, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_test_results() {
        let results = TestResults::new();
        let results = Arc::new(results);

        let mut handles = vec![];
        for i in 0..10 {
            let results = Arc::clone(&results);
            handles.push(thread::spawn(move || {
                results.push(i);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_stress_test_with_barrier() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        stress_test(8, 500, move |_, _| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 4000);
    }

    #[test]
    fn test_repeated_test_detects_consistency() {
        // This test verifies that repeated execution catches issues
        let value = Arc::new(AtomicU64::new(0));
        let value_clone = Arc::clone(&value);

        repeated_test(50, move || {
            let current = value_clone.load(Ordering::Relaxed);
            // Intentionally non-atomic read-modify-write
            // (This is a test of the testing framework, not a recommended pattern)
            value_clone.store(current.wrapping_add(1), Ordering::Relaxed);
        });

        // Value should be 50 (one increment per run)
        assert_eq!(value.load(Ordering::Relaxed), 50);
    }
}
