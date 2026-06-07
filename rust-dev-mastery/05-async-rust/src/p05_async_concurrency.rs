//! # Async Concurrency Primitives
//!
//! Tokio provides async-aware synchronization primitives that must be used instead
//! of `std::sync` types in async code. Using `std::sync::Mutex` across `.await`
//! points can cause deadlocks and block the runtime.
//!
//! ## Key Concepts
//! - **tokio::sync::Mutex**: Async-aware mutex; `.lock().await` yields to the runtime
//! - **tokio::sync::RwLock**: Async reader-writer lock for read-heavy workloads
//! - **Semaphore**: Limits concurrent access to a resource
//! - **Channels**: mpsc (multi-producer/single-consumer), broadcast, oneshot, watch
//! - **Notify**: Simple signal primitive for task coordination

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// A thread-safe async cache using tokio::sync::RwLock.
/// Multiple readers can access simultaneously; writes take exclusive access.
pub struct AsyncCache<K, V> {
    data: Arc<tokio::sync::RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: tokio::time::Instant,
}

impl<K, V> Clone for AsyncCache<K, V> {
    fn clone(&self) -> Self {
        AsyncCache {
            data: self.data.clone(),
            default_ttl: self.default_ttl,
        }
    }
}

impl<K, V> AsyncCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new(default_ttl: Duration) -> Self {
        AsyncCache {
            data: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Gets a value from the cache. Uses a read lock for concurrent access.
    pub async fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().await;
        data.get(key).and_then(|entry| {
            if tokio::time::Instant::now() < entry.expires_at {
                Some(entry.value.clone())
            } else {
                None // Expired
            }
        })
    }

    /// Inserts a value with the default TTL. Uses a write lock.
    pub async fn insert(&self, key: K, value: V) {
        let mut data = self.data.write().await;
        data.insert(
            key,
            CacheEntry {
                value,
                expires_at: tokio::time::Instant::now() + self.default_ttl,
            },
        );
    }

    /// Gets a value, or computes and inserts it if missing.
    /// Uses a write lock to prevent thundering herd on cache misses.
    pub async fn get_or_insert_with<F, Fut>(&self, key: K, compute: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = V>,
    {
        // Fast path: read lock check
        {
            let data = self.data.read().await;
            if let Some(entry) = data.get(&key) {
                if tokio::time::Instant::now() < entry.expires_at {
                    return entry.value.clone();
                }
            }
        }

        // Slow path: compute and insert
        let value = compute().await;
        let mut data = self.data.write().await;
        data.insert(
            key.clone(),
            CacheEntry {
                value: value.clone(),
                expires_at: tokio::time::Instant::now() + self.default_ttl,
            },
        );
        value
    }

    pub async fn len(&self) -> usize {
        self.data.read().await.len()
    }

    pub async fn clear(&self) {
        self.data.write().await.clear();
    }
}

/// A connection pool using Semaphore for concurrency limiting.
/// Demonstrates the semaphore-as-mutex pattern for bounded resource pools.
pub struct ConnectionPool<T> {
    connections: Arc<tokio::sync::Mutex<Vec<T>>>,
    semaphore: Arc<tokio::sync::Semaphore>,
    max_size: usize,
}

pub struct PooledConnection<T: Send + 'static> {
    connection: Option<T>,
    pool: Arc<tokio::sync::Mutex<Vec<T>>>,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl<T: Send + 'static> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            let sem = self.semaphore.clone();
            // Return connection to pool
            tokio::spawn(async move {
                pool.lock().await.push(conn);
                sem.add_permits(1);
            });
        }
    }
}

impl<T: Send> PooledConnection<T> {
    pub fn get(&self) -> &T {
        self.connection.as_ref().expect("connection already taken")
    }
}

impl<T: Clone + Send + 'static> ConnectionPool<T> {
    pub fn new(initial: Vec<T>, max_size: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(tokio::sync::Mutex::new(initial)),
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_size)),
            max_size,
        }
    }

    /// Acquires a connection from the pool. Blocks if the pool is at capacity.
    pub async fn acquire(&self) -> Result<PooledConnection<T>, PoolError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| PoolError::Closed)?;

        let conn = {
            let mut pool = self.connections.lock().await;
            pool.pop()
        };

        match conn {
            Some(connection) => Ok(PooledConnection {
                connection: Some(connection),
                pool: self.connections.clone(),
                semaphore: self.semaphore.clone(),
            }),
            None => Err(PoolError::Exhausted),
        }
    }

    pub async fn available(&self) -> usize {
        self.connections.lock().await.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PoolError {
    Exhausted,
    Closed,
}

/// Demonstrates the broadcast channel for fan-out messaging.
/// All subscribers receive every message.
pub struct EventBus<T: Clone> {
    sender: tokio::sync::broadcast::Sender<T>,
}

impl<T: Clone> EventBus<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        EventBus { sender }
    }

    pub fn publish(&self, event: T) -> Result<(), tokio::sync::broadcast::error::SendError<T>> {
        self.sender.send(event).map(|_| ())
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

/// Demonstrates the watch channel for latest-value broadcasting.
/// Useful for configuration updates, health status, etc.
pub struct ConfigStore {
    config: tokio::sync::watch::Receiver<AppConfig>,
    sender: tokio::sync::watch::Sender<AppConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub max_connections: u32,
    pub timeout_ms: u64,
    pub feature_flags: Vec<String>,
}

impl ConfigStore {
    pub fn new(initial: AppConfig) -> Self {
        let (sender, config) = tokio::sync::watch::channel(initial);
        ConfigStore { config, sender }
    }

    pub fn current(&self) -> AppConfig {
        self.config.borrow().clone()
    }

    pub fn update(&self, config: AppConfig) {
        let _ = self.sender.send(config);
    }

    /// Returns a future that resolves when the config changes.
    pub async fn wait_for_change(&mut self) -> AppConfig {
        let _ = self.config.changed().await;
        self.config.borrow().clone()
    }
}

/// Uses Notify for simple async task coordination.
/// One task signals, another waits.
pub struct TaskCoordinator {
    notify: Arc<tokio::sync::Notify>,
}

impl TaskCoordinator {
    pub fn new() -> Self {
        TaskCoordinator {
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Signal that work is ready.
    pub fn signal(&self) {
        self.notify.notify_one();
    }

    /// Signal all waiters.
    pub fn signal_all(&self) {
        self.notify.notify_waiters();
    }

    /// Wait for a signal.
    pub async fn wait(&self) {
        self.notify.notified().await;
    }
}

impl Clone for TaskCoordinator {
    fn clone(&self) -> Self {
        TaskCoordinator {
            notify: self.notify.clone(),
        }
    }
}

/// Rate limiter using Semaphore — limits operations per time window.
pub struct RateLimiter {
    semaphore: Arc<tokio::sync::Semaphore>,
    permits: usize,
}

impl RateLimiter {
    pub fn new(permits_per_second: usize) -> Self {
        RateLimiter {
            semaphore: Arc::new(tokio::sync::Semaphore::new(permits_per_second)),
            permits: permits_per_second,
        }
    }

    /// Acquire a permit before making a request. Blocks if rate limit is exceeded.
    pub async fn acquire(&self) {
        let _ = self.semaphore.acquire().await.unwrap();
    }

    /// Spawns a background task to refill permits periodically.
    pub fn start_refill(&self) -> tokio::task::JoinHandle<()> {
        let sem = self.semaphore.clone();
        let permits = self.permits;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let available = sem.available_permits();
                if available < permits {
                    sem.add_permits(permits - available);
                }
            }
        })
    }
}

/// A pub-sub system with topic-based routing using mpsc channels.
pub struct PubSub<T: Clone> {
    subscribers: Arc<tokio::sync::Mutex<HashMap<String, Vec<tokio::sync::mpsc::Sender<T>>>>>,
}

impl<T: Clone + Send + 'static> PubSub<T> {
    pub fn new() -> Self {
        PubSub {
            subscribers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to a topic. Returns a receiver for messages.
    pub async fn subscribe(&self, topic: impl Into<String>, buffer: usize) -> tokio::sync::mpsc::Receiver<T> {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        let mut subs = self.subscribers.lock().await;
        subs.entry(topic.into()).or_default().push(tx);
        rx
    }

    /// Publish a message to a topic.
    pub async fn publish(&self, topic: &str, message: T) {
        let mut subs = self.subscribers.lock().await;
        if let Some(senders) = subs.get_mut(topic) {
            // Remove closed senders
            senders.retain(|tx| !tx.is_closed());
            for tx in senders {
                let _ = tx.send(message.clone()).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_cache_basic() {
        let cache = AsyncCache::new(Duration::from_secs(60));
        cache.insert("key1", 42).await;
        assert_eq!(cache.get(&"key1").await, Some(42));
        assert_eq!(cache.get(&"key2").await, None);
    }

    #[tokio::test]
    async fn test_async_cache_ttl() {
        let cache = AsyncCache::new(Duration::from_millis(50));
        cache.insert("key", "value").await;
        assert_eq!(cache.get(&"key").await, Some("value"));

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(cache.get(&"key").await, None); // Expired
    }

    #[tokio::test]
    async fn test_async_cache_get_or_insert() {
        let cache = AsyncCache::new(Duration::from_secs(60));
        let value = cache
            .get_or_insert_with("key", || async { 42 })
            .await;
        assert_eq!(value, 42);

        // Second call should return cached value
        let value = cache
            .get_or_insert_with("key", || async { 99 })
            .await;
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(vec![1, 2, 3], 3);
        assert_eq!(pool.available().await, 3);

        let conn = pool.acquire().await.unwrap();
        assert_eq!(*conn.get(), 3); // LIFO
        assert_eq!(pool.available().await, 2);

        drop(conn);
        // Connection returns to pool asynchronously
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(pool.available().await, 3);
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::<String>::new(64);
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();

        bus.publish("hello".to_string()).unwrap();

        assert_eq!(sub1.recv().await.unwrap(), "hello");
        assert_eq!(sub2.recv().await.unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_config_store() {
        let store = ConfigStore::new(AppConfig {
            max_connections: 100,
            timeout_ms: 5000,
            feature_flags: vec!["v2".into()],
        });

        assert_eq!(store.current().max_connections, 100);

        let mut store2 = ConfigStore {
            config: store.config.clone(),
            sender: store.sender.clone(),
        };

        store.update(AppConfig {
            max_connections: 200,
            timeout_ms: 5000,
            feature_flags: vec!["v2".into()],
        });

        let changed = store2.wait_for_change().await;
        assert_eq!(changed.max_connections, 200);
    }

    #[tokio::test]
    async fn test_task_coordinator() {
        let coord = TaskCoordinator::new();
        let coord2 = coord.clone();

        let handle = tokio::spawn(async move {
            coord2.wait().await;
            42
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        coord.signal();

        assert_eq!(handle.await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_task_coordinator_signal_all() {
        let coord = TaskCoordinator::new();
        let mut handles = Vec::new();

        for _ in 0..3 {
            let c = coord.clone();
            handles.push(tokio::spawn(async move {
                c.wait().await;
                1
            }));
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
        coord.signal_all();

        let mut sum = 0;
        for h in handles {
            sum += h.await.unwrap();
        }
        assert_eq!(sum, 3);
    }

    #[tokio::test]
    async fn test_pub_sub() {
        let pubsub = PubSub::<String>::new();
        let mut rx1 = pubsub.subscribe("events", 10).await;
        let mut rx2 = pubsub.subscribe("events", 10).await;
        let mut rx3 = pubsub.subscribe("other", 10).await;

        pubsub.publish("events", "event1".to_string()).await;

        assert_eq!(rx1.recv().await.unwrap(), "event1");
        assert_eq!(rx2.recv().await.unwrap(), "event1");
        // Other topic should not receive
        assert!(rx3.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3);

        // Acquire permits and hold them
        let _p1 = limiter.semaphore.clone().acquire_owned().await.unwrap();
        let _p2 = limiter.semaphore.clone().acquire_owned().await.unwrap();
        let _p3 = limiter.semaphore.clone().acquire_owned().await.unwrap();

        // Semaphore is now exhausted
        assert!(limiter.semaphore.try_acquire().is_err());
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let cache = AsyncCache::new(Duration::from_secs(60));
        let mut handles = Vec::new();

        for i in 0..10 {
            let c = cache.clone();
            handles.push(tokio::spawn(async move {
                c.insert(format!("key-{i}"), i).await;
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(cache.len().await, 10);
    }
}
