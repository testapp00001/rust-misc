/// Problem: Scalability
///
/// Master scalability in Rust.
///
/// Key Concepts:
/// - Horizontal scaling
/// - Vertical scaling
/// - Load balancing
/// - Caching
/// - Database scaling

/// Problem 1: Connection pooling
/// Scale with connection pooling
pub struct ConnectionPool {
    connections: Vec<String>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self) -> String {
        self.connections.pop().unwrap_or_else(|| "new_connection".to_string())
    }

    pub fn release(&mut self, conn: String) {
        if self.connections.len() < self.max_size {
            self.connections.push(conn);
        }
    }
}

/// Problem 2: Caching layer
/// Scale with caching
pub struct CacheLayer {
    cache: std::collections::HashMap<String, String>,
    hit_count: u64,
    miss_count: u64,
}

impl CacheLayer {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if let Some(value) = self.cache.get(key) {
            self.hit_count += 1;
            Some(value)
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

/// Problem 3: Load balancing
/// Scale with load balancing
pub struct LoadBalancer {
    servers: Vec<String>,
    current: usize,
}

impl LoadBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        Self { servers, current: 0 }
    }

    pub fn next_server(&mut self) -> &str {
        let server = &self.servers[self.current % self.servers.len()];
        self.current += 1;
        server
    }
}

/// Problem 4: Rate limiting
/// Scale with rate limiting
pub struct RateLimiter {
    requests: std::collections::HashMap<String, Vec<std::time::Instant>>,
    max_requests: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: std::time::Duration) -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            max_requests,
            window,
        }
    }

    pub fn allow(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        requests.retain(|t| now.duration_since(*t) < self.window);
        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Problem 5: Queue-based scaling
/// Scale with queues
pub struct TaskQueue {
    queue: std::collections::VecDeque<String>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, task: String) {
        self.queue.push_back(task);
    }

    pub fn dequeue(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Problem 6: Batch processing
/// Scale with batch processing
pub struct BatchProcessor {
    batch: Vec<i32>,
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::new(),
            batch_size,
        }
    }

    pub fn add(&mut self, item: i32) -> Option<Vec<i32>> {
        self.batch.push(item);
        if self.batch.len() >= self.batch_size {
            Some(std::mem::take(&mut self.batch))
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.batch)
    }
}

/// Problem 7: Async processing
/// Scale with async
pub async fn async_process(data: Vec<i32>) -> Vec<i32> {
    data.into_iter().map(|x| x * 2).collect()
}

/// Problem 8: Parallel processing
/// Scale with parallelism
pub fn parallel_process(data: &[i32]) -> i32 {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let chunk_size = (data.len() + 3) / 4;
    let result = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let result = Arc::clone(&result);
        handles.push(thread::spawn(move || {
            let sum: i32 = chunk.iter().sum();
            *result.lock().unwrap() += sum;
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *result.lock().unwrap();
    result
}

/// Problem 9: Sharding
/// Scale with sharding
pub struct Shard {
    data: std::collections::HashMap<String, String>,
}

impl Shard {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

pub struct ShardedStore {
    shards: Vec<Shard>,
}

impl ShardedStore {
    pub fn new(num_shards: usize) -> Self {
        let shards = (0..num_shards).map(|_| Shard::new()).collect();
        Self { shards }
    }

    fn shard_index(&self, key: &str) -> usize {
        let hash: usize = key.chars().map(|c| c as usize).sum();
        hash % self.shards.len()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        let index = self.shard_index(key);
        self.shards[index].get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let index = self.shard_index(key);
        self.shards[index].set(key, value);
    }
}

/// Problem 10: Replication
/// Scale with replication
pub struct Replica {
    data: std::collections::HashMap<String, String>,
}

impl Replica {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

pub struct ReplicatedStore {
    primary: Replica,
    replicas: Vec<Replica>,
}

impl ReplicatedStore {
    pub fn new(num_replicas: usize) -> Self {
        let replicas = (0..num_replicas).map(|_| Replica::new()).collect();
        Self {
            primary: Replica::new(),
            replicas,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.primary.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.primary.set(key, value);
        for replica in &mut self.replicas {
            replica.set(key, value);
        }
    }
}

/// Problem 11: Horizontal scaling
/// Simulate horizontal scaling
pub struct HorizontalScaler {
    instances: Vec<String>,
}

impl HorizontalScaler {
    pub fn new() -> Self {
        Self {
            instances: vec!["instance1".to_string()],
        }
    }

    pub fn scale_up(&mut self) {
        let new_instance = format!("instance{}", self.instances.len() + 1);
        self.instances.push(new_instance);
    }

    pub fn scale_down(&mut self) {
        self.instances.pop();
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

/// Problem 12: Vertical scaling
/// Simulate vertical scaling
pub struct VerticalScaler {
    cpu_cores: u32,
    memory_gb: u32,
}

impl VerticalScaler {
    pub fn new() -> Self {
        Self {
            cpu_cores: 1,
            memory_gb: 1,
        }
    }

    pub fn add_cpu(&mut self, cores: u32) {
        self.cpu_cores += cores;
    }

    pub fn add_memory(&mut self, gb: u32) {
        self.memory_gb += gb;
    }

    pub fn specs(&self) -> (u32, u32) {
        (self.cpu_cores, self.memory_gb)
    }
}

/// Problem 13: Auto-scaling
/// Simulate auto-scaling
pub struct AutoScaler {
    min_instances: usize,
    max_instances: usize,
    current_instances: usize,
    target_cpu: f64,
}

impl AutoScaler {
    pub fn new(min: usize, max: usize, target_cpu: f64) -> Self {
        Self {
            min_instances: min,
            max_instances: max,
            current_instances: min,
            target_cpu,
        }
    }

    pub fn scale(&mut self, current_cpu: f64) {
        if current_cpu > self.target_cpu && self.current_instances < self.max_instances {
            self.current_instances += 1;
        } else if current_cpu < self.target_cpu * 0.5 && self.current_instances > self.min_instances {
            self.current_instances -= 1;
        }
    }

    pub fn instances(&self) -> usize {
        self.current_instances
    }
}

/// Problem 14: Database scaling
/// Simulate database scaling
pub struct DatabaseScaler {
    read_replicas: usize,
    write_primary: usize,
}

impl DatabaseScaler {
    pub fn new() -> Self {
        Self {
            read_replicas: 1,
            write_primary: 1,
        }
    }

    pub fn add_read_replica(&mut self) {
        self.read_replicas += 1;
    }

    pub fn specs(&self) -> (usize, usize) {
        (self.write_primary, self.read_replicas)
    }
}

/// Problem 15: CDN simulation
/// Simulate CDN
pub struct Cdn {
    nodes: Vec<String>,
}

impl Cdn {
    pub fn new() -> Self {
        Self {
            nodes: vec!["node1".to_string()],
        }
    }

    pub fn add_node(&mut self, node: &str) {
        self.nodes.push(node.to_string());
    }

    pub fn nearest_node(&self) -> &str {
        &self.nodes[0]
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        let mut pool = ConnectionPool::new(2);
        let conn = pool.get();
        pool.release(conn);
        assert_eq!(pool.connections.len(), 1);
    }

    #[test]
    fn test_cache_layer() {
        let mut cache = CacheLayer::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_load_balancer() {
        let mut lb = LoadBalancer::new(vec!["s1".to_string(), "s2".to_string()]);
        assert_eq!(lb.next_server(), "s1");
        assert_eq!(lb.next_server(), "s2");
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, std::time::Duration::from_secs(1));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(!limiter.allow("user1"));
    }

    #[test]
    fn test_task_queue() {
        let mut queue = TaskQueue::new();
        queue.enqueue("task1".to_string());
        assert_eq!(queue.dequeue(), Some("task1".to_string()));
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new(3);
        assert!(processor.add(1).is_none());
        assert!(processor.add(2).is_none());
        let batch = processor.add(3);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_async_process() {
        let result = async_process(vec![1, 2, 3]).await;
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_parallel_process() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(parallel_process(&data), 15);
    }

    #[test]
    fn test_sharded_store() {
        let mut store = ShardedStore::new(4);
        store.set("key", "value");
        assert_eq!(store.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_replicated_store() {
        let mut store = ReplicatedStore::new(2);
        store.set("key", "value");
        assert_eq!(store.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_horizontal_scaler() {
        let mut scaler = HorizontalScaler::new();
        assert_eq!(scaler.instance_count(), 1);
        scaler.scale_up();
        assert_eq!(scaler.instance_count(), 2);
    }

    #[test]
    fn test_vertical_scaler() {
        let mut scaler = VerticalScaler::new();
        scaler.add_cpu(2);
        scaler.add_memory(4);
        assert_eq!(scaler.specs(), (3, 5));
    }

    #[test]
    fn test_auto_scaler() {
        let mut scaler = AutoScaler::new(1, 10, 0.7);
        scaler.scale(0.9);
        assert_eq!(scaler.instances(), 2);
    }

    #[test]
    fn test_database_scaler() {
        let mut scaler = DatabaseScaler::new();
        scaler.add_read_replica();
        assert_eq!(scaler.specs(), (1, 2));
    }

    #[test]
    fn test_cdn() {
        let mut cdn = Cdn::new();
        cdn.add_node("node2");
        assert_eq!(cdn.node_count(), 2);
    }
}
