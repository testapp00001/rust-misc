//! # Concurrent Data Structures
//!
//! Concurrent data structures allow safe access from multiple threads without
//! external synchronization. This module covers DashMap, concurrent queues,
//! sharded collections, and patterns for building concurrent data structures.
//!
//! ## Key Structures:
//!
//! | Structure | Crate | Use Case |
//! |-----------|-------|----------|
//! | `DashMap` | dashmap | Concurrent hash map |
//! | `ConcurrentQueue` | concurrent-queue | Lock-free queue |
//! | `ShardedMap` | custom | Sharded hash map |
//! | `ConcurrentBag` | custom | Lock-free bag |
//!
//! ## Sharding Strategy:
//!
//! Instead of a single lock, sharded collections use multiple locks (one per shard).
//! Access is routed to a shard based on the key's hash, reducing contention.

use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Wrapper around DashMap with additional concurrent operations.
pub struct ConcurrentMap<K, V> {
    inner: DashMap<K, V>,
    access_count: AtomicU64,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> ConcurrentMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
            access_count: AtomicU64::new(0),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.inner.get(key).map(|v| v.clone())
    }

    pub fn remove(&self, key: &K) -> Option<(K, V)> {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.inner.remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// Get or insert using a factory function.
    pub fn get_or_insert_with<F>(&self, key: K, factory: F) -> V
    where
        F: FnOnce() -> V,
        K: Clone,
        V: Clone,
    {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        if let Some(value) = self.inner.get(&key) {
            return value.clone();
        }
        let value = factory();
        self.inner.entry(key).or_insert(value).clone()
    }

    /// Update a value using a function.
    pub fn update<F>(&self, key: &K, updater: F) -> Option<V>
    where
        F: FnOnce(&V) -> V,
        V: Clone,
    {
        self.inner.get_mut(key).map(|mut v| {
            let new_val = updater(&*v);
            *v = new_val.clone();
            new_val
        })
    }

    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    /// Iterate over all entries (snapshot).
    pub fn iter_collect(&self) -> Vec<(K, V)> {
        self.inner
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

/// Sharded concurrent map for reduced contention.
/// Splits the keyspace across N shards, each with its own lock.
pub struct ShardedMap<K, V> {
    shards: Vec<parking_lot::Mutex<HashMap<K, V>>>,
    num_shards: usize,
}

impl<K: Eq + std::hash::Hash, V> ShardedMap<K, V> {
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(parking_lot::Mutex::new(HashMap::new()));
        }
        Self { shards, num_shards }
    }

    fn shard_index(&self, key: &K) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.num_shards
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let idx = self.shard_index(&key);
        self.shards[idx].lock().insert(key, value)
    }

    pub fn get<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&V) -> R,
    {
        let idx = self.shard_index(key);
        self.shards[idx].lock().get(key).map(|v| f(v))
    }

    pub fn remove(&self, key: &K) -> Option<(K, V)> {
        let idx = self.shard_index(key);
        self.shards[idx].lock().remove_entry(key)
    }

    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.lock().len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.lock().is_empty())
    }
}

/// Concurrent counter map - counts occurrences of keys.
pub struct ConcurrentCounter<K> {
    counts: DashMap<K, u64>,
}

impl<K: Eq + std::hash::Hash + Clone> ConcurrentCounter<K> {
    pub fn new() -> Self {
        Self {
            counts: DashMap::new(),
        }
    }

    /// Increment the counter for a key.
    pub fn increment(&self, key: &K) -> u64 {
        let mut entry = self.counts.entry(key.clone()).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Add a value to the counter.
    pub fn add(&self, key: &K, value: u64) -> u64 {
        let mut entry = self.counts.entry(key.clone()).or_insert(0);
        *entry += value;
        *entry
    }

    pub fn get(&self, key: &K) -> u64 {
        self.counts.get(key).map(|v| *v).unwrap_or(0)
    }

    /// Get the top N keys by count.
    pub fn top_n(&self, n: usize) -> Vec<(K, u64)> {
        let mut entries: Vec<(K, u64)> = self
            .counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(n).collect()
    }

    pub fn total(&self) -> u64 {
        self.counts.iter().map(|entry| *entry.value()).sum()
    }

    pub fn unique_keys(&self) -> usize {
        self.counts.len()
    }
}

/// Concurrent set backed by DashMap.
pub struct ConcurrentSet<T> {
    inner: DashMap<T, ()>,
}

impl<T: Eq + std::hash::Hash + Clone> ConcurrentSet<T> {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    pub fn insert(&self, value: T) -> bool {
        self.inner.insert(value, ()).is_none()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.inner.contains_key(value)
    }

    pub fn remove(&self, value: &T) -> bool {
        self.inner.remove(value).is_some()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Convert to a Vec.
    pub fn to_vec(&self) -> Vec<T> {
        self.inner.iter().map(|entry| entry.key().clone()).collect()
    }
}

/// Concurrent LRU cache using DashMap with access tracking.
pub struct ConcurrentLruCache<K, V> {
    data: DashMap<K, V>,
    access_order: DashMap<K, AtomicU64>,
    max_size: usize,
    clock: AtomicU64,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> ConcurrentLruCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: DashMap::new(),
            access_order: DashMap::new(),
            max_size,
            clock: AtomicU64::new(0),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let timestamp = self.clock.fetch_add(1, Ordering::Relaxed);
        if let Some(order) = self.access_order.get(key) {
            order.store(timestamp, Ordering::Relaxed);
        }
        self.data.get(key).map(|v| v.clone())
    }

    pub fn insert(&self, key: K, value: V) {
        if self.data.len() >= self.max_size {
            self.evict_lru();
        }
        let timestamp = self.clock.fetch_add(1, Ordering::Relaxed);
        self.data.insert(key.clone(), value);
        self.access_order.insert(key, AtomicU64::new(timestamp));
    }

    fn evict_lru(&self) {
        if let Some(oldest_key) = self
            .access_order
            .iter()
            .min_by_key(|entry| entry.value().load(Ordering::Relaxed))
            .map(|entry| entry.key().clone())
        {
            self.data.remove(&oldest_key);
            self.access_order.remove(&oldest_key);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
}

/// Concurrent histogram using sharded counters.
pub struct ConcurrentHistogram {
    buckets: Vec<AtomicU64>,
    boundaries: Vec<f64>,
}

impl ConcurrentHistogram {
    pub fn new(boundaries: Vec<f64>) -> Self {
        let num_buckets = boundaries.len() + 1;
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(AtomicU64::new(0));
        }
        Self { buckets, boundaries }
    }

    /// Record a value into the appropriate bucket.
    pub fn record(&self, value: f64) {
        let bucket = self.find_bucket(value);
        self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
    }

    fn find_bucket(&self, value: f64) -> usize {
        for (i, &boundary) in self.boundaries.iter().enumerate() {
            if value < boundary {
                return i;
            }
        }
        self.boundaries.len() // Overflow bucket
    }

    /// Get the count for each bucket.
    pub fn bucket_counts(&self) -> Vec<u64> {
        self.buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }

    pub fn total_count(&self) -> u64 {
        self.buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_concurrent_map_basic() {
        let map = ConcurrentMap::new();
        map.insert("key1", "value1");
        assert_eq!(map.get(&"key1"), Some("value1"));
        assert_eq!(map.get(&"key2"), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_concurrent_map_concurrent_inserts() {
        let map = Arc::new(ConcurrentMap::new());
        let mut handles = vec![];

        for i in 0..10 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key-{}-{}", i, j);
                    map.insert(key, i * 100 + j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 1000);
        assert!(map.access_count() > 0);
    }

    #[test]
    fn test_concurrent_map_get_or_insert() {
        let map = ConcurrentMap::new();
        let val = map.get_or_insert_with("key", || 42);
        assert_eq!(val, 42);
        let val = map.get_or_insert_with("key", || 99);
        assert_eq!(val, 42); // Should not re-insert
    }

    #[test]
    fn test_concurrent_map_update() {
        let map = ConcurrentMap::new();
        map.insert("counter", 10);
        let new_val = map.update(&"counter", |v| v + 5);
        assert_eq!(new_val, Some(15));
    }

    #[test]
    fn test_sharded_map() {
        let map = ShardedMap::new(16);
        map.insert("key1", "value1");
        assert_eq!(map.get(&"key1", |v| *v), Some("value1"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_sharded_map_concurrent() {
        let map = Arc::new(ShardedMap::new(16));
        let mut handles = vec![];

        for i in 0..10 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    map.insert(format!("{}-{}", i, j), i * 100 + j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 1000);
    }

    #[test]
    fn test_sharded_map_remove() {
        let map = ShardedMap::new(4);
        map.insert("key", "value");
        assert!(map.remove(&"key").is_some());
        assert!(map.remove(&"key").is_none());
    }

    #[test]
    fn test_concurrent_counter() {
        let counter = ConcurrentCounter::new();
        counter.increment(&"a");
        counter.increment(&"a");
        counter.increment(&"b");
        counter.add(&"a", 3);

        assert_eq!(counter.get(&"a"), 5);
        assert_eq!(counter.get(&"b"), 1);
        assert_eq!(counter.total(), 6);
        assert_eq!(counter.unique_keys(), 2);
    }

    #[test]
    fn test_concurrent_counter_top_n() {
        let counter = ConcurrentCounter::new();
        for _ in 0..10 { counter.increment(&"a"); }
        for _ in 0..5 { counter.increment(&"b"); }
        for _ in 0..3 { counter.increment(&"c"); }

        let top = counter.top_n(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("a", 10));
        assert_eq!(top[1], ("b", 5));
    }

    #[test]
    fn test_concurrent_counter_concurrent() {
        let counter = Arc::new(ConcurrentCounter::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    counter.increment(&"total");
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(&"total"), 10_000);
    }

    #[test]
    fn test_concurrent_set() {
        let set = ConcurrentSet::new();
        assert!(set.insert(1));
        assert!(!set.insert(1)); // Already exists
        assert!(set.contains(&1));
        assert!(!set.contains(&2));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_concurrent_set_remove() {
        let set = ConcurrentSet::new();
        set.insert(42);
        assert!(set.remove(&42));
        assert!(!set.contains(&42));
    }

    #[test]
    fn test_concurrent_lru_cache() {
        let cache = ConcurrentLruCache::new(3);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.len(), 3);

        // Insert one more - should evict LRU (b, since a was accessed)
        cache.insert("d", 4);
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_concurrent_histogram() {
        let hist = ConcurrentHistogram::new(vec![10.0, 20.0, 30.0, 40.0]);
        hist.record(5.0);   // bucket 0 (< 10)
        hist.record(15.0);  // bucket 1 (10-20)
        hist.record(25.0);  // bucket 2 (20-30)
        hist.record(35.0);  // bucket 3 (30-40)
        hist.record(50.0);  // bucket 4 (>= 40)

        let counts = hist.bucket_counts();
        assert_eq!(counts, vec![1, 1, 1, 1, 1]);
        assert_eq!(hist.total_count(), 5);
    }

    #[test]
    fn test_concurrent_histogram_concurrent() {
        let hist = Arc::new(ConcurrentHistogram::new(vec![100.0, 200.0]));
        let mut handles = vec![];

        for i in 0..10 {
            let hist = Arc::clone(&hist);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    hist.record((i * 100 + j) as f64);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(hist.total_count(), 1000);
    }

    #[test]
    fn test_concurrent_map_iter_collect() {
        let map = ConcurrentMap::new();
        map.insert("a", 1);
        map.insert("b", 2);

        let entries = map.iter_collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_concurrent_set_to_vec() {
        let set = ConcurrentSet::new();
        set.insert(3);
        set.insert(1);
        set.insert(2);

        let mut vec = set.to_vec();
        vec.sort();
        assert_eq!(vec, vec![1, 2, 3]);
    }
}
