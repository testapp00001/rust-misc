//! # Shared State Concurrency
//!
//! Shared state concurrency uses locks to protect data accessed by multiple threads.
//! Rust's ownership system prevents data races at compile time, but deadlocks are
//! still possible. This module covers Mutex, RwLock, parking_lot, and strategies
//! for preventing deadlocks.
//!
//! ## Lock Types:
//!
//! | Type | Read Access | Write Access | Use Case |
//! |------|-------------|-------------|----------|
//! | `Mutex<T>` | Exclusive | Exclusive | General purpose |
//! | `RwLock<T>` | Shared | Exclusive | Read-heavy workloads |
//! | `parking_lot::Mutex` | Exclusive | Exclusive | Faster, no poisoning |
//! | `parking_lot::RwLock` | Shared | Exclusive | Faster, no poisoning |
//!
//! ## Deadlock Prevention:
//!
//! 1. **Lock ordering**: Always acquire locks in the same order
//! 2. **Lock timeouts**: Use `try_lock_for` to detect deadlocks
//! 3. **Minimize lock scope**: Hold locks for the shortest time possible
//! 4. **Avoid nested locks**: Use message passing instead

use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Thread-safe counter using parking_lot Mutex.
/// parking_lot is faster than std::sync::Mutex and doesn't poison on panic.
pub struct ConcurrentCounter {
    value: Mutex<u64>,
}

impl ConcurrentCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: Mutex::new(initial),
        }
    }

    pub fn increment(&self) -> u64 {
        let mut val = self.value.lock();
        *val += 1;
        *val
    }

    pub fn add(&self, amount: u64) -> u64 {
        let mut val = self.value.lock();
        *val += amount;
        *val
    }

    pub fn get(&self) -> u64 {
        *self.value.lock()
    }

    pub fn reset(&self) -> u64 {
        let mut val = self.value.lock();
        let old = *val;
        *val = 0;
        old
    }
}

/// Thread-safe cache with RwLock for read-heavy workloads.
pub struct ConcurrentCache<K, V> {
    data: RwLock<HashMap<K, V>>,
    hits: Mutex<u64>,
    misses: Mutex<u64>,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> ConcurrentCache<K, V> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }

    /// Get a value from the cache (read lock).
    pub fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read();
        if let Some(value) = data.get(key) {
            *self.hits.lock() += 1;
            Some(value.clone())
        } else {
            *self.misses.lock() += 1;
            None
        }
    }

    /// Insert a value into the cache (write lock).
    pub fn insert(&self, key: K, value: V) {
        let mut data = self.data.write();
        data.insert(key, value);
    }

    /// Remove a value from the cache (write lock).
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write();
        data.remove(key)
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.data.read().len(),
            hits: *self.hits.lock(),
            misses: *self.misses.lock(),
        }
    }

    /// Get or insert using a factory function (prevents double-computation).
    pub fn get_or_insert_with<F>(&self, key: K, factory: F) -> V
    where
        F: FnOnce() -> V,
    {
        // Try read first
        {
            let data = self.data.read();
            if let Some(value) = data.get(&key) {
                *self.hits.lock() += 1;
                return value.clone();
            }
        }
        *self.misses.lock() += 1;

        // Upgrade to write
        let mut data = self.data.write();
        // Double-check (another thread may have inserted)
        if let Some(value) = data.get(&key) {
            return value.clone();
        }
        let value = factory();
        data.insert(key, value.clone());
        value
    }

    pub fn clear(&self) {
        self.data.write().clear();
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub hits: u64,
    pub misses: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }
}

/// Lock ordering utility to prevent deadlocks.
/// Assigns an order number to each lock and enforces acquisition order.
pub struct LockOrdering {
    order: parking_lot::Mutex<HashMap<String, usize>>,
}

impl LockOrdering {
    pub fn new() -> Self {
        Self {
            order: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: &str, order: usize) {
        self.order.lock().insert(name.to_string(), order);
    }

    /// Check if acquiring `new_lock` is safe given currently held locks.
    pub fn can_acquire(&self, held: &[&str], new_lock: &str) -> bool {
        let order = self.order.lock();
        let new_order = match order.get(new_lock) {
            Some(o) => *o,
            None => return true, // Unknown lock, allow
        };

        for held_name in held {
            if let Some(held_order) = order.get(*held_name) {
                if *held_order >= new_order {
                    return false; // Would violate ordering
                }
            }
        }
        true
    }
}

/// A guarded resource that enforces lock ordering.
pub struct OrderedResource<T> {
    name: String,
    data: Mutex<T>,
    order: usize,
}

impl<T> OrderedResource<T> {
    pub fn new(name: &str, data: T, order: usize) -> Self {
        Self {
            name: name.into(),
            data: Mutex::new(data),
            order,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn lock(&self) -> parking_lot::MutexGuard<'_, T> {
        self.data.lock()
    }

    pub fn try_lock_for(&self, timeout: Duration) -> Option<parking_lot::MutexGuard<'_, T>> {
        self.data.try_lock_for(timeout)
    }
}

/// Thread-safe rate limiter using a sliding window.
pub struct RateLimiter {
    window: Mutex<Vec<std::time::Instant>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            window: Mutex::new(Vec::new()),
            max_requests,
            window_duration,
        }
    }

    /// Check if a request is allowed under the rate limit.
    pub fn allow(&self) -> bool {
        let now = std::time::Instant::now();
        let mut window = self.window.lock();

        // Remove expired entries
        window.retain(|&t| now.duration_since(t) < self.window_duration);

        if window.len() < self.max_requests {
            window.push(now);
            true
        } else {
            false
        }
    }

    pub fn current_rate(&self) -> usize {
        let now = std::time::Instant::now();
        let window = self.window.lock();
        window
            .iter()
            .filter(|&&t| now.duration_since(t) < self.window_duration)
            .count()
    }
}

/// Thread-safe object pool using a Mutex-protected Vec.
pub struct ObjectPool<T> {
    objects: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T: Send> ObjectPool<T> {
    pub fn new<F>(initial_size: usize, max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let mut objects = Vec::with_capacity(max_size);
        for _ in 0..initial_size {
            objects.push(factory());
        }
        Self {
            objects: Mutex::new(objects),
            factory: Box::new(factory),
            max_size,
        }
    }

    /// Get an object from the pool, creating one if the pool is empty.
    pub fn acquire(&self) -> T {
        let mut objects = self.objects.lock();
        objects.pop().unwrap_or_else(|| (self.factory)())
    }

    /// Return an object to the pool.
    pub fn release(&self, obj: T) {
        let mut objects = self.objects.lock();
        if objects.len() < self.max_size {
            objects.push(obj);
        }
        // Otherwise drop the object
    }

    pub fn available(&self) -> usize {
        self.objects.lock().len()
    }
}

/// Scoped lock utility that acquires multiple locks in order.
pub fn multi_lock_ordered<T1, T2, R, F>(
    res1: &OrderedResource<T1>,
    res2: &OrderedResource<T2>,
    f: F,
) -> R
where
    F: FnOnce(&mut T1, &mut T2) -> R,
{
    assert_ne!(res1.order(), res2.order(), "Resources must have different orders");

    if res1.order() < res2.order() {
        let mut guard1 = res1.lock();
        let mut guard2 = res2.lock();
        f(&mut guard1, &mut guard2)
    } else {
        let mut guard2 = res2.lock();
        let mut guard1 = res1.lock();
        f(&mut guard1, &mut guard2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_concurrent_counter() {
        let counter = Arc::new(ConcurrentCounter::new(0));
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
    fn test_concurrent_counter_add() {
        let counter = ConcurrentCounter::new(0);
        counter.add(42);
        assert_eq!(counter.get(), 42);
        counter.add(8);
        assert_eq!(counter.get(), 50);
    }

    #[test]
    fn test_concurrent_counter_reset() {
        let counter = ConcurrentCounter::new(100);
        let old = counter.reset();
        assert_eq!(old, 100);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = ConcurrentCache::new();
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), None);

        cache.remove(&"key1");
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = ConcurrentCache::new();
        cache.insert("a", 1);
        cache.get(&"a"); // hit
        cache.get(&"a"); // hit
        cache.get(&"b"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_get_or_insert() {
        let cache = ConcurrentCache::new();
        let val = cache.get_or_insert_with("key", || 42);
        assert_eq!(val, 42);

        // Should return cached value, not re-compute
        let val = cache.get_or_insert_with("key", || 99);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_cache_concurrent_access() {
        let cache = Arc::new(ConcurrentCache::new());
        let mut handles = vec![];

        for i in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key-{}-{}", i, j);
                    cache.insert(key.clone(), i * 100 + j);
                    cache.get(&key);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cache.stats();
        assert!(stats.hits > 0);
    }

    #[test]
    fn test_lock_ordering_prevents_deadlock() {
        let ordering = LockOrdering::new();
        ordering.register("users", 1);
        ordering.register("orders", 2);

        assert!(ordering.can_acquire(&[], "users"));
        assert!(ordering.can_acquire(&["users"], "orders"));
        assert!(!ordering.can_acquire(&["orders"], "users")); // Would violate ordering
    }

    #[test]
    fn test_ordered_resource() {
        let r1 = OrderedResource::new("first", vec![1, 2, 3], 1);
        let r2 = OrderedResource::new("second", vec![4, 5, 6], 2);

        assert_eq!(r1.order(), 1);
        assert_eq!(r2.order(), 2);

        let val = r1.lock();
        assert_eq!(*val, vec![1, 2, 3]);
    }

    #[test]
    fn test_multi_lock_ordered() {
        let r1 = OrderedResource::new("first", 10, 1);
        let r2 = OrderedResource::new("second", 20, 2);

        let result = multi_lock_ordered(&r1, &r2, |a, b| *a + *b);
        assert_eq!(result, 30);

        // Works regardless of argument order
        let result = multi_lock_ordered(&r2, &r1, |a, b| *a + *b);
        assert_eq!(result, 30);
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));

        for _ in 0..5 {
            assert!(limiter.allow());
        }
        assert!(!limiter.allow()); // 6th request denied
        assert_eq!(limiter.current_rate(), 5);
    }

    #[test]
    fn test_rate_limiter_window_expiry() {
        let limiter = RateLimiter::new(2, Duration::from_millis(50));

        assert!(limiter.allow());
        assert!(limiter.allow());
        assert!(!limiter.allow());

        thread::sleep(Duration::from_millis(60));
        assert!(limiter.allow()); // Window expired
    }

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(3, 10, || Vec::<i32>::new());
        assert_eq!(pool.available(), 3);

        let mut obj = pool.acquire();
        obj.push(42);
        assert_eq!(pool.available(), 2);

        pool.release(obj);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_object_pool_exhaustion_creates_new() {
        let pool = ObjectPool::new(1, 1, || 0);
        let _a = pool.acquire();
        assert_eq!(pool.available(), 0);

        // Pool empty, should create new object
        let b = pool.acquire();
        assert_eq!(b, 0);
    }

    #[test]
    fn test_object_pool_max_size() {
        let pool = ObjectPool::new(2, 2, || 0);
        let a = pool.acquire();
        let b = pool.acquire();
        pool.release(a);
        pool.release(b);
        assert_eq!(pool.available(), 2);

        let extra = pool.acquire();
        pool.release(extra);
        // Should not exceed max
        assert!(pool.available() <= 2);
    }

    #[test]
    fn test_parking_lot_mutex_no_poisoning() {
        // Unlike std::Mutex, parking_lot doesn't poison on panic
        let m = Arc::new(Mutex::new(42));
        let m_clone = Arc::clone(&m);

        let handle = thread::spawn(move || {
            let _guard = m_clone.lock();
            panic!("Thread panicked!");
        });

        let _ = handle.join();
        // parking_lot mutex is still usable after panic
        let val = m.lock();
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_rwlock_concurrent_reads() {
        let lock = Arc::new(RwLock::new(vec![1, 2, 3]));
        let mut handles = vec![];

        // Multiple concurrent readers
        for _ in 0..5 {
            let lock = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let data = lock.read();
                data.iter().sum::<i32>()
            }));
        }

        for handle in handles {
            let sum = handle.join().unwrap();
            assert_eq!(sum, 6);
        }
    }
}
