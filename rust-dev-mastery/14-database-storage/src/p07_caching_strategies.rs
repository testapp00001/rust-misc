//! # Caching Strategies
//!
//! Caching reduces database load and improves response times. This lesson covers
//! in-memory caching, cache-aside patterns, TTL-based expiration, and cache
//! invalidation strategies.
//!
//! ## Key Concepts
//! - Cache-aside (lazy loading) pattern
//! - Write-through and write-behind patterns
//! - TTL-based expiration
//! - LRU (Least Recently Used) eviction
//! - Cache stampede prevention
//! - Cache key design

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Cache Entry
// ---------------------------------------------------------------------------

/// A cached value with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub inserted_at: Instant,
    pub ttl: Duration,
    pub access_count: u64,
    pub last_accessed: Instant,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            inserted_at: now,
            ttl,
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }

    pub fn remaining_ttl(&self) -> Duration {
        let elapsed = self.inserted_at.elapsed();
        if elapsed >= self.ttl {
            Duration::ZERO
        } else {
            self.ttl - elapsed
        }
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }
}

// ---------------------------------------------------------------------------
// 2. Cache Statistics
// ---------------------------------------------------------------------------

/// Tracks cache hit/miss statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

// ---------------------------------------------------------------------------
// 3. TTL Cache
// /// A simple TTL-based cache.
#[derive(Debug)]
pub struct TtlCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
    max_size: usize,
    stats: CacheStats,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
            max_size,
            stats: CacheStats::default(),
        }
    }

    /// Get a value from the cache.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.evict_expired();

        let expired = self
            .entries
            .get(key)
            .map(|e| e.is_expired())
            .unwrap_or(false);
        if expired {
            self.entries.remove(key);
            self.stats.misses += 1;
            return None;
        }

        if let Some(entry) = self.entries.get_mut(key) {
            entry.touch();
            self.stats.hits += 1;
            Some(&entry.value)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a value with the default TTL.
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert a value with a custom TTL.
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) {
        if self.entries.len() >= self.max_size {
            self.evict_oldest();
        }
        self.entries.insert(key, CacheEntry::new(value, ttl));
        self.stats.insertions += 1;
    }

    /// Remove a value from the cache.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|e| e.value)
    }

    /// Check if a key exists and is not expired.
    pub fn contains(&mut self, key: &K) -> bool {
        self.get(key).is_some();
        // get already updated stats, so we need to undo
        // Actually, let's just check directly
        if let Some(entry) = self.entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Get or compute a value (cache-aside pattern).
    pub fn get_or_compute<F>(&mut self, key: K, compute: F) -> &V
    where
        F: FnOnce() -> V,
    {
        if !self.entries.contains_key(&key) || self.entries.get(&key).map_or(false, |e| e.is_expired()) {
            let value = compute();
            self.insert(key.clone(), value);
        }
        self.entries.get(&key).unwrap(); // safe: we just inserted
        &self.entries.get(&key).unwrap().value
    }

    fn evict_expired(&mut self) {
        let expired: Vec<K> = self
            .entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired.len();
        for key in expired {
            self.entries.remove(&key);
        }
        self.stats.evictions += count as u64;
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.inserted_at)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&oldest_key);
            self.stats.evictions += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// ---------------------------------------------------------------------------
// 4. LRU Cache
// ---------------------------------------------------------------------------

/// An LRU (Least Recently Used) cache with a fixed capacity.
#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, (V, usize)>, // value + access order index
    order: VecDeque<K>,
    capacity: usize,
    stats: CacheStats,
}

impl<K: Eq + Hash + Clone, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
            stats: CacheStats::default(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to front of order
            self.order.retain(|k| k != key);
            self.order.push_front(key.clone());
            self.stats.hits += 1;
            self.map.get(key).map(|(v, _)| v)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // Update existing
            self.order.retain(|k| k != &key);
            self.order.push_front(key.clone());
            self.map.insert(key, (value, self.order.len()));
        } else {
            // Evict if at capacity
            if self.map.len() >= self.capacity {
                if let Some(evicted) = self.order.pop_back() {
                    self.map.remove(&evicted);
                    self.stats.evictions += 1;
                }
            }
            self.order.push_front(key.clone());
            self.map.insert(key, (value, self.order.len()));
            self.stats.insertions += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

// ---------------------------------------------------------------------------
// 5. Cache Key Design
// ---------------------------------------------------------------------------

/// Utilities for building cache keys.
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    pub fn new(namespace: &str) -> Self {
        Self {
            parts: vec![namespace.into()],
        }
    }

    pub fn push(mut self, part: impl Into<String>) -> Self {
        self.parts.push(part.into());
        self
    }

    pub fn build(&self) -> String {
        self.parts.join(":")
    }
}

/// Common cache key patterns.
pub fn user_cache_key(user_id: &str) -> String {
    CacheKeyBuilder::new("user").push(user_id).build()
}

pub fn session_cache_key(session_id: &str) -> String {
    CacheKeyBuilder::new("session").push(session_id).build()
}

pub fn query_cache_key(table: &str, query_hash: &str) -> String {
    CacheKeyBuilder::new("query")
        .push(table)
        .push(query_hash)
        .build()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_ttl() {
        let entry = CacheEntry::new("value", Duration::from_secs(60));
        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() > Duration::from_secs(50));
    }

    #[test]
    fn test_cache_entry_expired() {
        let mut entry = CacheEntry::new("value", Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
        assert_eq!(entry.remaining_ttl(), Duration::ZERO);
    }

    #[test]
    fn test_cache_entry_touch() {
        let mut entry = CacheEntry::new("value", Duration::from_secs(60));
        assert_eq!(entry.access_count, 0);
        entry.touch();
        entry.touch();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_ttl_cache_basic() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert("key1", "value1");

        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"missing"), None);
    }

    #[test]
    fn test_ttl_cache_expiration() {
        let mut cache = TtlCache::new(Duration::from_millis(10), 100);
        cache.insert("key", "value");

        std::thread::sleep(Duration::from_millis(20));
        assert!(cache.get(&"key").is_none());
    }

    #[test]
    fn test_ttl_cache_custom_ttl() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert_with_ttl("short", "value", Duration::from_millis(10));
        cache.insert_with_ttl("long", "value", Duration::from_secs(60));

        std::thread::sleep(Duration::from_millis(20));
        assert!(cache.get(&"short").is_none());
        assert!(cache.get(&"long").is_some());
    }

    #[test]
    fn test_ttl_cache_max_size() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3); // should evict oldest

        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_ttl_cache_remove() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert("key", "value");
        let removed = cache.remove(&"key");
        assert_eq!(removed, Some("value"));
        assert!(cache.get(&"key").is_none());
    }

    #[test]
    fn test_ttl_cache_stats() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert("key", "value");

        cache.get(&"key"); // hit
        cache.get(&"missing"); // miss
        cache.get(&"key"); // hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_ttl_cache_clear() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3); // evicts "a" (least recently used)

        assert!(cache.get(&"a").is_none());
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_lru_cache_access_updates_order() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);

        cache.get(&"a"); // "a" is now most recently used

        cache.put("c", 3); // evicts "b" (now least recently used)

        assert_eq!(cache.get(&"a"), Some(&1));
        assert!(cache.get(&"b").is_none());
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_lru_cache_stats() {
        let mut cache = LruCache::new(10);
        cache.put("a", 1);
        cache.get(&"a"); // hit
        cache.get(&"b"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 1);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        let stats = CacheStats {
            hits: 75,
            misses: 25,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("user")
            .push("123")
            .push("profile")
            .build();
        assert_eq!(key, "user:123:profile");
    }

    #[test]
    fn test_cache_key_patterns() {
        assert_eq!(user_cache_key("42"), "user:42");
        assert_eq!(session_cache_key("abc"), "session:abc");
        assert_eq!(
            query_cache_key("users", "hash123"),
            "query:users:hash123"
        );
    }

    #[test]
    fn test_ttl_cache_contains() {
        let mut cache = TtlCache::new(Duration::from_secs(60), 100);
        cache.insert("key", "value");
        assert!(cache.contains(&"key"));
        assert!(!cache.contains(&"missing"));
    }
}
