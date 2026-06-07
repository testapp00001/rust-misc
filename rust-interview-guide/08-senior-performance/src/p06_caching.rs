/// Problem: Caching
///
/// Master caching in Rust.
///
/// Key Concepts:
/// - In-memory caching
/// - LRU cache
/// - TTL cache
/// - Cache invalidation
/// - Cache warming

use std::collections::HashMap;

/// Problem 1: Basic cache
/// Create basic cache
pub struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 2: LRU cache
/// Implement LRU cache
pub struct LruCache {
    data: HashMap<String, (String, u64)>,
    access_order: Vec<String>,
    max_size: usize,
    counter: u64,
}

impl LruCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            counter: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some((value, _)) = self.data.get(key) {
            let cloned_value = value.clone();
            self.counter += 1;
            self.data.insert(key.to_string(), (cloned_value.clone(), self.counter));
            Some(cloned_value)
        } else {
            None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if self.data.len() >= self.max_size {
            self.evict();
        }
        self.counter += 1;
        self.data.insert(key.to_string(), (value.to_string(), self.counter));
        self.access_order.push(key.to_string());
    }

    fn evict(&mut self) {
        if let Some(oldest_key) = self.access_order.first().cloned() {
            self.data.remove(&oldest_key);
            self.access_order.remove(0);
        }
    }
}

/// Problem 3: TTL cache
/// Cache with time-to-live
pub struct TtlCache {
    data: HashMap<String, (String, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl TtlCache {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Some((value, time)) = self.data.get(key) {
            if time.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), (value.to_string(), std::time::Instant::now()));
    }
}

/// Problem 4: Memoization cache
/// Memoize function results
pub struct MemoCache {
    cache: HashMap<i32, i32>,
}

impl MemoCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn fibonacci(&mut self, n: i32) -> i32 {
        if let Some(&result) = self.cache.get(&n) {
            return result;
        }
        let result = match n {
            0 => 0,
            1 => 1,
            _ => self.fibonacci(n - 1) + self.fibonacci(n - 2),
        };
        self.cache.insert(n, result);
        result
    }
}

/// Problem 5: Write-through cache
/// Write to cache and backing store
pub struct WriteThroughCache {
    cache: HashMap<String, String>,
    backing: HashMap<String, String>,
}

impl WriteThroughCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            backing: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
        self.backing.insert(key.to_string(), value.to_string());
    }
}

/// Problem 6: Write-back cache
/// Write to cache, flush to backing store
pub struct WriteBackCache {
    cache: HashMap<String, String>,
    backing: HashMap<String, String>,
    dirty: Vec<String>,
}

impl WriteBackCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            backing: HashMap::new(),
            dirty: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
        self.dirty.push(key.to_string());
    }

    pub fn flush(&mut self) {
        for key in &self.dirty {
            if let Some(value) = self.cache.get(key) {
                self.backing.insert(key.clone(), value.clone());
            }
        }
        self.dirty.clear();
    }
}

/// Problem 7: Multi-level cache
/// Multiple cache levels
pub struct MultiLevelCache {
    l1: HashMap<String, String>,
    l2: HashMap<String, String>,
}

impl MultiLevelCache {
    pub fn new() -> Self {
        Self {
            l1: HashMap::new(),
            l2: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.l1.get(key).or_else(|| self.l2.get(key))
    }

    pub fn set(&mut self, key: &str, value: &str, level: u8) {
        match level {
            1 => self.l1.insert(key.to_string(), value.to_string()),
            _ => self.l2.insert(key.to_string(), value.to_string()),
        };
    }
}

/// Problem 8: Cache with statistics
/// Track cache statistics
pub struct StatsCache {
    data: HashMap<String, String>,
    hits: u64,
    misses: u64,
}

impl StatsCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if let Some(value) = self.data.get(key) {
            self.hits += 1;
            Some(value)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Problem 9: Cache warming
/// Pre-populate cache
pub fn warm_cache(cache: &mut Cache, data: &[(&str, &str)]) {
    for (key, value) in data {
        cache.set(key, value);
    }
}

/// Problem 10: Cache invalidation
/// Invalidate cache entries
pub struct InvalidationCache {
    data: HashMap<String, String>,
}

impl InvalidationCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn invalidate(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn invalidate_all(&mut self) {
        self.data.clear();
    }
}

/// Problem 11: Generic cache
/// Generic cache implementation
pub struct GenericCache<K, V> {
    data: HashMap<K, V>,
}

impl<K: Eq + std::hash::Hash, V> GenericCache<K, V> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }
}

/// Problem 12: Thread-safe cache
/// Thread-safe cache
pub struct ThreadSafeCache {
    data: std::sync::Mutex<HashMap<String, String>>,
}

impl ThreadSafeCache {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

/// Problem 13: Async cache
/// Async cache (simulated)
pub struct AsyncCache {
    data: HashMap<String, String>,
}

impl AsyncCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub async fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 14: Cache with size limit
/// Cache with maximum size
pub struct SizeLimitedCache {
    data: HashMap<String, String>,
    max_size: usize,
}

impl SizeLimitedCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if self.data.len() >= self.max_size {
            if let Some(oldest_key) = self.data.keys().next().cloned() {
                self.data.remove(&oldest_key);
            }
        }
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 15: Cache with eviction policy
/// Cache with custom eviction
pub struct EvictionCache {
    data: HashMap<String, String>,
    access_count: HashMap<String, u64>,
    max_size: usize,
}

impl EvictionCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            access_count: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if self.data.contains_key(key) {
            *self.access_count.entry(key.to_string()).or_insert(0) += 1;
            self.data.get(key)
        } else {
            None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if self.data.len() >= self.max_size {
            self.evict_lfu();
        }
        self.data.insert(key.to_string(), value.to_string());
        self.access_count.insert(key.to_string(), 1);
    }

    fn evict_lfu(&mut self) {
        if let Some(key) = self.access_count
            .iter()
            .min_by_key(|(_, &count)| count)
            .map(|(key, _)| key.clone())
        {
            self.data.remove(&key);
            self.access_count.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cache() {
        let mut cache = Cache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        cache.set("a", "1");
        cache.set("b", "2");
        assert_eq!(cache.get("a"), Some("1".to_string()));
    }

    #[test]
    fn test_ttl_cache() {
        let mut cache = TtlCache::new(std::time::Duration::from_secs(1));
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_memo_cache() {
        let mut cache = MemoCache::new();
        assert_eq!(cache.fibonacci(10), 55);
    }

    #[test]
    fn test_write_through_cache() {
        let mut cache = WriteThroughCache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_write_back_cache() {
        let mut cache = WriteBackCache::new();
        cache.set("key", "value");
        cache.flush();
        assert_eq!(cache.backing.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_multi_level_cache() {
        let mut cache = MultiLevelCache::new();
        cache.set("key", "value", 1);
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_stats_cache() {
        let mut cache = StatsCache::new();
        cache.set("key", "value");
        cache.get("key");
        cache.get("missing");
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_cache_warming() {
        let mut cache = Cache::new();
        warm_cache(&mut cache, &[("key", "value")]);
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_invalidation_cache() {
        let mut cache = InvalidationCache::new();
        cache.set("key", "value");
        cache.invalidate("key");
        assert!(cache.get("key").is_none());
    }

    #[test]
    fn test_generic_cache() {
        let mut cache = GenericCache::new();
        cache.set("key", 42);
        assert_eq!(cache.get(&"key"), Some(&42));
    }

    #[test]
    fn test_thread_safe_cache() {
        let cache = ThreadSafeCache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_async_cache() {
        let mut cache = AsyncCache::new();
        cache.set("key", "value").await;
        assert_eq!(cache.get("key").await, Some(&"value".to_string()));
    }

    #[test]
    fn test_size_limited_cache() {
        let mut cache = SizeLimitedCache::new(2);
        cache.set("a", "1");
        cache.set("b", "2");
        cache.set("c", "3");
        assert_eq!(cache.data.len(), 2);
    }

    #[test]
    fn test_eviction_cache() {
        let mut cache = EvictionCache::new(2);
        cache.set("a", "1");
        cache.set("b", "2");
        cache.get("a");
        cache.set("c", "3");
        assert!(cache.get("a").is_some());
    }
}
