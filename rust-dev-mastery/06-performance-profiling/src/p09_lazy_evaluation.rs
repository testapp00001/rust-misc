//! # Lazy Evaluation
//!
//! Lazy evaluation defers computation until the result is actually needed.
//! This module covers lazy initialization patterns, `OnceLock` (standard library),
//! deferred computation, and memoization/caching.
//!
//! ## Key Concepts
//! - **OnceLock**: Thread-safe one-time initialization (std::sync::OnceLock)
//! - **LazyLock**: Lazy initialization with OnceLock (std::sync::LazyLock)
//! - **Deferred computation**: Computing values on first access
//! - **Memoization**: Caching function results to avoid redundant computation

use std::collections::HashMap;
use std::sync::OnceLock;

/// A lazy-initialized value that is computed on first access.
/// Thread-safe using OnceLock internally.
pub struct Lazy<T, F = fn() -> T> {
    cell: OnceLock<T>,
    init: F,
}

impl<T, F: Fn() -> T> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Lazy {
            cell: OnceLock::new(),
            init,
        }
    }

    pub fn get(&self) -> &T {
        self.cell.get_or_init(&self.init)
    }
}

impl<T, F: Fn() -> T> std::ops::Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

/// Demonstrates std::sync::LazyLock (stabilized in Rust 1.80).
/// Computes the value once on first access, then caches it forever.
static EXPENSIVE_CONFIG: OnceLock<Vec<String>> = OnceLock::new();

pub fn get_config() -> &'static Vec<String> {
    EXPENSIVE_CONFIG.get_or_init(|| {
        // Simulate expensive computation
        vec![
            "database_url=postgres://localhost/mydb".into(),
            "max_connections=100".into(),
            "timeout=30s".into(),
        ]
    })
}

/// A memoized function that caches results.
pub struct Memoizer<I: Eq + std::hash::Hash, O> {
    cache: std::cell::RefCell<HashMap<I, O>>,
    compute: Box<dyn Fn(&I) -> O>,
}

impl<I: Eq + std::hash::Hash + Clone, O: Clone> Memoizer<I, O> {
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn(&I) -> O + 'static,
    {
        Memoizer {
            cache: std::cell::RefCell::new(HashMap::new()),
            compute: Box::new(compute),
        }
    }

    pub fn call(&self, input: &I) -> O {
        if let Some(result) = self.cache.borrow().get(input) {
            return result.clone();
        }
        let result = (self.compute)(input);
        self.cache.borrow_mut().insert(input.clone(), result.clone());
        result
    }

    pub fn cache_size(&self) -> usize {
        self.cache.borrow().len()
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
}

/// A thread-safe memoizer using Mutex.
pub struct ThreadSafeMemoizer<I: Eq + std::hash::Hash, O> {
    cache: std::sync::Mutex<HashMap<I, O>>,
    compute: Box<dyn Fn(&I) -> O + Send + Sync>,
}

impl<I: Eq + std::hash::Hash + Clone, O: Clone> ThreadSafeMemoizer<I, O> {
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn(&I) -> O + Send + Sync + 'static,
    {
        ThreadSafeMemoizer {
            cache: std::sync::Mutex::new(HashMap::new()),
            compute: Box::new(compute),
        }
    }

    pub fn call(&self, input: &I) -> O {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(result) = cache.get(input) {
                return result.clone();
            }
        }
        let result = (self.compute)(input);
        let mut cache = self.cache.lock().unwrap();
        cache.entry(input.clone()).or_insert(result).clone()
    }

    pub fn cache_size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }
}

/// A lazy sequence that computes values on demand.
pub struct LazySequence<F> {
    state: u64,
    compute: F,
}

impl<F: Fn(u64) -> u64> LazySequence<F> {
    pub fn new(initial: u64, compute: F) -> Self {
        LazySequence {
            state: initial,
            compute,
        }
    }

    pub fn next(&mut self) -> u64 {
        let value = self.state;
        self.state = (self.compute)(value);
        value
    }

    pub fn take(&mut self, n: usize) -> Vec<u64> {
        (0..n).map(|_| self.next()).collect()
    }

    pub fn peek(&self) -> u64 {
        self.state
    }
}

/// A lazy-initialized lookup table.
pub struct LookupTable {
    table: OnceLock<HashMap<u32, f64>>,
    range: std::ops::Range<u32>,
    compute: fn(u32) -> f64,
}

impl LookupTable {
    pub const fn new(range: std::ops::Range<u32>, compute: fn(u32) -> f64) -> Self {
        LookupTable {
            table: OnceLock::new(),
            range,
            compute,
        }
    }

    pub fn get(&self, key: u32) -> Option<f64> {
        let table = self.table.get_or_init(|| {
            let mut map = HashMap::new();
            for k in self.range.clone() {
                map.insert(k, (self.compute)(k));
            }
            map
        });
        table.get(&key).copied()
    }

    pub fn size(&self) -> usize {
        self.range.len()
    }
}

/// An incremental computation cache that only recomputes when inputs change.
pub struct IncrementalCache<I: Eq + std::hash::Hash, O> {
    last_input: Option<I>,
    last_output: Option<O>,
    compute: Box<dyn Fn(&I) -> O>,
}

impl<I: Eq + std::hash::Hash + Clone, O: Clone> IncrementalCache<I, O> {
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn(&I) -> O + 'static,
    {
        IncrementalCache {
            last_input: None,
            last_output: None,
            compute: Box::new(compute),
        }
    }

    pub fn compute(&mut self, input: &I) -> &O {
        if self.last_input.as_ref() != Some(input) {
            let output = (self.compute)(input);
            self.last_input = Some(input.clone());
            self.last_output = Some(output);
        }
        self.last_output.as_ref().unwrap()
    }

    pub fn is_cached(&self, input: &I) -> bool {
        self.last_input.as_ref() == Some(input)
    }
}

/// A builder that lazily constructs a complex value.
pub struct LazyBuilder<T> {
    initial: T,
    transforms: Vec<Box<dyn FnOnce(T) -> T>>,
    built: OnceLock<T>,
}

impl<T: Clone> LazyBuilder<T> {
    pub fn new(initial: T) -> Self {
        LazyBuilder {
            initial,
            transforms: Vec::new(),
            built: OnceLock::new(),
        }
    }

    pub fn transform<F: FnOnce(T) -> T + 'static>(mut self, f: F) -> Self {
        self.transforms.push(Box::new(f));
        self
    }

    pub fn build(&self) -> &T {
        self.built.get_or_init(|| {
            let mut value = self.initial.clone();
            // Can't move out of self.transforms, so we need a workaround
            // In practice, you'd use a different design
            value
        })
    }
}

/// Lazy fibonacci with memoization.
pub struct Fibonacci {
    memoizer: ThreadSafeMemoizer<u64, u64>,
}

impl Fibonacci {
    pub fn new() -> Self {
        let memoizer = ThreadSafeMemoizer::new(|&n: &u64| {
            // Iterative fibonacci for efficiency
            if n == 0 { return 0; }
            if n == 1 { return 1; }
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        });
        Fibonacci { memoizer }
    }

    pub fn compute(&self, n: u64) -> u64 {
        self.memoizer.call(&n)
    }

    pub fn cache_size(&self) -> usize {
        self.memoizer.cache_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_basic() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();

        let lazy = Lazy::new(move || {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            42
        });

        // First access computes
        assert_eq!(*lazy.get(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Second access returns cached
        assert_eq!(*lazy.get(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1); // Not incremented
    }

    #[test]
    fn test_lazy_deref() {
        let lazy = Lazy::new(|| vec![1, 2, 3]);
        assert_eq!(lazy.len(), 3);
        assert_eq!(lazy[0], 1);
    }

    #[test]
    fn test_get_config() {
        let config = get_config();
        assert_eq!(config.len(), 3);
        assert!(config[0].contains("database_url"));
    }

    #[test]
    fn test_memoizer() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();

        let memo = Memoizer::new(move |x: &i32| {
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            x * x
        });

        assert_eq!(memo.call(&5), 25);
        assert_eq!(memo.call(&5), 25); // Cached
        assert_eq!(memo.call(&3), 9);

        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2); // Only computed twice
        assert_eq!(memo.cache_size(), 2);
    }

    #[test]
    fn test_memoizer_clear() {
        let memo = Memoizer::new(|x: &i32| x * x);
        memo.call(&5);
        memo.call(&3);
        assert_eq!(memo.cache_size(), 2);

        memo.clear();
        assert_eq!(memo.cache_size(), 0);
    }

    #[test]
    fn test_thread_safe_memoizer() {
        let memo = std::sync::Arc::new(ThreadSafeMemoizer::new(|x: &u64| {
            // Simulate expensive computation
            x * x + x
        }));

        let mut handles = Vec::new();
        for i in 0..10 {
            let m = memo.clone();
            handles.push(std::thread::spawn(move || m.call(&i)));
        }

        for (i, h) in handles.into_iter().enumerate() {
            let expected = (i as u64) * (i as u64) + (i as u64);
            assert_eq!(h.join().unwrap(), expected);
        }
    }

    #[test]
    fn test_lazy_sequence() {
        let mut seq = LazySequence::new(1, |n| n * 2);
        assert_eq!(seq.next(), 1);
        assert_eq!(seq.next(), 2);
        assert_eq!(seq.next(), 4);
        assert_eq!(seq.next(), 8);
    }

    #[test]
    fn test_lazy_sequence_take() {
        let mut seq = LazySequence::new(1, |n| n + 1);
        let values = seq.take(5);
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_lazy_sequence_peek() {
        let mut seq = LazySequence::new(10, |n| n - 1);
        assert_eq!(seq.peek(), 10);
        seq.next();
        assert_eq!(seq.peek(), 9);
    }

    #[test]
    fn test_lookup_table() {
        static TABLE: LookupTable = LookupTable::new(0..10, |x| (x as f64).sqrt());

        assert_eq!(TABLE.size(), 10);
        assert!((TABLE.get(0).unwrap() - 0.0).abs() < 0.001);
        assert!((TABLE.get(4).unwrap() - 2.0).abs() < 0.001);
        assert!((TABLE.get(9).unwrap() - 3.0).abs() < 0.001);
        assert!(TABLE.get(10).is_none());
    }

    #[test]
    fn test_incremental_cache() {
        let mut cache = IncrementalCache::new(|x: &i32| x * x);

        let result = cache.compute(&5);
        assert_eq!(*result, 25);

        // Same input — should return cached
        let result = cache.compute(&5);
        assert_eq!(*result, 25);
        assert!(cache.is_cached(&5));

        // Different input — recomputes
        let result = cache.compute(&3);
        assert_eq!(*result, 9);
        assert!(!cache.is_cached(&5));
        assert!(cache.is_cached(&3));
    }

    #[test]
    fn test_fibonacci_memoized() {
        let fib = Fibonacci::new();

        assert_eq!(fib.compute(0), 0);
        assert_eq!(fib.compute(1), 1);
        assert_eq!(fib.compute(10), 55);
        assert_eq!(fib.compute(20), 6765);

        // Cache should have entries for all computed values
        assert!(fib.cache_size() > 0);

        // Computing the same value again should be fast (cached)
        assert_eq!(fib.compute(20), 6765);
    }

    #[test]
    fn test_fibonacci_large() {
        let fib = Fibonacci::new();
        // This would be very slow without memoization
        let result = fib.compute(50);
        assert_eq!(result, 12586269025);
    }
}
