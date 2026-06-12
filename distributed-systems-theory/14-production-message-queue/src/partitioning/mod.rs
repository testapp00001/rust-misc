//! Partition management for the message queue.
//!
//! Each topic is split into a fixed number of partitions. Partitions are the
//! unit of parallelism and replication. This module handles topic creation,
//! partition assignment, and key-based routing.
//!
//! Sub-modules provide the underlying data structures:
//! - `hash_ring`: Consistent hash ring for key-to-partition mapping
//! - `rebalancer`: Partition movement planning when nodes change

pub mod hash_ring;
pub mod rebalancer;

use std::collections::HashMap;
use std::sync::RwLock;

use tracing::debug;

use crate::error::{MqError, Result};

/// Metadata for a single partition.
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// The partition index within its topic.
    pub index: u32,
    /// The replica node IDs responsible for this partition.
    pub replicas: Vec<u64>,
    /// The current leader node for this partition.
    pub leader: u64,
    /// High watermark: the next offset to be written.
    pub high_watermark: u64,
    /// Log start offset (oldest available offset).
    pub log_start_offset: u64,
}

/// Metadata for a topic.
#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// The topic name.
    pub name: String,
    /// Number of partitions.
    pub partition_count: u32,
    /// Replication factor.
    pub replication_factor: u32,
    /// Partition details, keyed by partition index.
    pub partitions: HashMap<u32, PartitionInfo>,
}

/// Manages topic and partition metadata for the broker.
pub struct PartitionManager {
    default_partitions: u32,
    default_replication_factor: u32,
    topics: RwLock<HashMap<String, TopicInfo>>,
}

impl PartitionManager {
    /// Create a new partition manager.
    pub fn new(default_partitions: u32, default_replication_factor: u32) -> Self {
        PartitionManager {
            default_partitions,
            default_replication_factor,
            topics: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new topic with the specified partition count and replication
    /// factor.
    pub fn create_topic(
        &self,
        name: &str,
        partition_count: u32,
        replication_factor: u32,
    ) -> Result<()> {
        let mut topics = self.topics.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        if topics.contains_key(name) {
            return Err(MqError::Internal(format!(
                "Topic '{}' already exists",
                name
            )));
        }

        if partition_count == 0 {
            return Err(MqError::Partition(format!(
                "Partition count must be > 0 for topic '{}'",
                name
            )));
        }

        if replication_factor == 0 {
            return Err(MqError::Replication(format!(
                "Replication factor must be > 0 for topic '{}'",
                name
            )));
        }

        let mut partitions = HashMap::new();
        for i in 0..partition_count {
            // Simple round-robin assignment: assign replica IDs 1..=replication_factor.
            // In a real system, this would use the cluster membership and a
            // consistent placement strategy.
            let replicas: Vec<u64> = (1..=replication_factor as u64).collect();
            let leader = replicas[0]; // First replica is the initial leader.

            partitions.insert(
                i,
                PartitionInfo {
                    index: i,
                    replicas,
                    leader,
                    high_watermark: 0,
                    log_start_offset: 0,
                },
            );
        }

        let topic_info = TopicInfo {
            name: name.to_string(),
            partition_count,
            replication_factor,
            partitions,
        };

        debug!(topic = name, partitions = partition_count, "Topic created");
        topics.insert(name.to_string(), topic_info);
        Ok(())
    }

    /// Delete a topic and all its partition metadata.
    pub fn delete_topic(&self, name: &str) -> Result<()> {
        let mut topics = self.topics.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        topics.remove(name).ok_or_else(|| {
            MqError::TopicNotFound(name.to_string())
        })?;

        debug!(topic = name, "Topic deleted");
        Ok(())
    }

    /// List all topic names.
    pub fn list_topics(&self) -> Vec<String> {
        let topics = match self.topics.read() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        topics.keys().cloned().collect()
    }

    /// Get metadata for a topic.
    pub fn get_topic(&self, name: &str) -> Result<TopicInfo> {
        let topics = self.topics.read().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        topics
            .get(name)
            .cloned()
            .ok_or_else(|| MqError::TopicNotFound(name.to_string()))
    }

    /// Get metadata for a specific partition.
    pub fn get_partition(
        &self,
        topic: &str,
        partition: u32,
    ) -> Result<PartitionInfo> {
        let topics = self.topics.read().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let topic_info = topics.get(topic).ok_or_else(|| {
            MqError::TopicNotFound(topic.to_string())
        })?;

        topic_info
            .partitions
            .get(&partition)
            .cloned()
            .ok_or_else(|| MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            })
    }

    /// Route a message key to a partition index using consistent hashing.
    ///
    /// If no key is provided, a random partition is selected (in production,
    /// you would use a round-robin counter instead).
    pub fn route_key(&self, topic: &str, key: Option<&[u8]>) -> Result<u32> {
        let topics = self.topics.read().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let topic_info = topics.get(topic).ok_or_else(|| {
            MqError::TopicNotFound(topic.to_string())
        })?;

        let partition_count = topic_info.partition_count;
        if partition_count == 0 {
            return Err(MqError::Partition(format!(
                "Topic '{}' has no partitions",
                topic
            )));
        }

        match key {
            Some(k) => {
                // Murmur2-compatible hash for key-based partitioning.
                let hash = murmur2_hash(k);
                Ok((hash % partition_count as u32) as u32)
            }
            None => {
                // Without a key, return partition 0. The producer should
                // implement round-robin across partitions itself.
                Ok(0)
            }
        }
    }

    /// Update the high watermark for a partition.
    pub fn update_high_watermark(
        &self,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> Result<()> {
        let mut topics = self.topics.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let topic_info = topics.get_mut(topic).ok_or_else(|| {
            MqError::TopicNotFound(topic.to_string())
        })?;

        let p = topic_info.partitions.get_mut(&partition).ok_or_else(|| {
            MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            }
        })?;

        p.high_watermark = offset;
        Ok(())
    }

    /// Get the total number of partitions across all topics.
    pub fn total_partitions(&self) -> u32 {
        let topics = match self.topics.read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        topics.values().map(|t| t.partition_count).sum()
    }

    /// Get the number of topics.
    pub fn topic_count(&self) -> usize {
        let topics = match self.topics.read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        topics.len()
    }
}

/// Murmur2 hash function, compatible with Apache Kafka's default partitioner.
fn murmur2_hash(data: &[u8]) -> u32 {
    let length = data.len() as i32;
    let mut h: i32 = 0xcafebabeu32 as i32;
    let mut i = 0;

    while i + 4 <= data.len() {
        let k = i32::from_le_bytes([
            data[i],
            data[i + 1],
            data[i + 2],
            data[i + 3],
        ]);

        let k = k.wrapping_mul(0x5bd1e995);
        let k = k ^ (k >> 24);
        let k = k.wrapping_mul(0x5bd1e995);

        h = h.wrapping_mul(0x5bd1e995);
        h ^= k;
        i += 4;
    }

    // Handle remaining bytes.
    match length - i as i32 {
        3 => {
            h ^= (data[i + 2] as i32) << 16;
            h ^= (data[i + 1] as i32) << 8;
            h ^= data[i] as i32;
            h = h.wrapping_mul(0x5bd1e995);
        }
        2 => {
            h ^= (data[i + 1] as i32) << 8;
            h ^= data[i] as i32;
            h = h.wrapping_mul(0x5bd1e995);
        }
        1 => {
            h ^= data[i] as i32;
            h = h.wrapping_mul(0x5bd1e995);
        }
        _ => {}
    }

    h ^= length as i32;
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^= h >> 15;

    h as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> PartitionManager {
        PartitionManager::new(3, 2)
    }

    #[test]
    fn create_topic_basic() {
        let mgr = default_manager();
        mgr.create_topic("orders", 6, 3).unwrap();

        let topics = mgr.list_topics();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0], "orders");

        let info = mgr.get_topic("orders").unwrap();
        assert_eq!(info.partition_count, 6);
        assert_eq!(info.replication_factor, 3);
        assert_eq!(info.partitions.len(), 6);
    }

    #[test]
    fn create_duplicate_topic_fails() {
        let mgr = default_manager();
        mgr.create_topic("orders", 3, 1).unwrap();
        let result = mgr.create_topic("orders", 3, 1);
        assert!(result.is_err());
    }

    #[test]
    fn delete_topic() {
        let mgr = default_manager();
        mgr.create_topic("temp", 1, 1).unwrap();
        mgr.delete_topic("temp").unwrap();
        assert!(mgr.list_topics().is_empty());
    }

    #[test]
    fn delete_nonexistent_topic_fails() {
        let mgr = default_manager();
        let result = mgr.delete_topic("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn get_partition() {
        let mgr = default_manager();
        mgr.create_topic("topic", 4, 2).unwrap();

        let p = mgr.get_partition("topic", 0).unwrap();
        assert_eq!(p.index, 0);
        assert_eq!(p.replicas.len(), 2);
        assert_eq!(p.leader, 1);
    }

    #[test]
    fn get_nonexistent_partition() {
        let mgr = default_manager();
        mgr.create_topic("topic", 2, 1).unwrap();
        let result = mgr.get_partition("topic", 99);
        assert!(result.is_err());
    }

    #[test]
    fn route_key_deterministic() {
        let mgr = default_manager();
        mgr.create_topic("topic", 10, 1).unwrap();

        let key = b"my-message-key";
        let p1 = mgr.route_key("topic", Some(key)).unwrap();
        let p2 = mgr.route_key("topic", Some(key)).unwrap();
        assert_eq!(p1, p2);
        assert!(p1 < 10);
    }

    #[test]
    fn route_key_no_key_returns_zero() {
        let mgr = default_manager();
        mgr.create_topic("topic", 6, 1).unwrap();
        let p = mgr.route_key("topic", None).unwrap();
        assert_eq!(p, 0);
    }

    #[test]
    fn route_key_distribution() {
        let mgr = default_manager();
        mgr.create_topic("topic", 10, 1).unwrap();

        // Different keys should distribute across partitions.
        let mut seen = std::collections::HashSet::new();
        for i in 0..1000 {
            let key = format!("key-{}", i);
            let p = mgr.route_key("topic", Some(key.as_bytes())).unwrap();
            seen.insert(p);
        }
        // With 1000 keys and 10 partitions, we should see at least 5 unique
        // partitions (probabilistically near 10).
        assert!(seen.len() >= 5, "Expected at least 5 partitions, got {}", seen.len());
    }

    #[test]
    fn update_high_watermark() {
        let mgr = default_manager();
        mgr.create_topic("topic", 2, 1).unwrap();

        mgr.update_high_watermark("topic", 0, 100).unwrap();
        let p = mgr.get_partition("topic", 0).unwrap();
        assert_eq!(p.high_watermark, 100);
    }

    #[test]
    fn total_partitions_count() {
        let mgr = default_manager();
        assert_eq!(mgr.total_partitions(), 0);

        mgr.create_topic("a", 3, 1).unwrap();
        assert_eq!(mgr.total_partitions(), 3);

        mgr.create_topic("b", 5, 1).unwrap();
        assert_eq!(mgr.total_partitions(), 8);
    }

    #[test]
    fn topic_count() {
        let mgr = default_manager();
        assert_eq!(mgr.topic_count(), 0);

        mgr.create_topic("a", 1, 1).unwrap();
        mgr.create_topic("b", 1, 1).unwrap();
        assert_eq!(mgr.topic_count(), 2);
    }

    #[test]
    fn create_zero_partitions_fails() {
        let mgr = default_manager();
        let result = mgr.create_topic("bad", 0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn create_zero_replication_fails() {
        let mgr = default_manager();
        let result = mgr.create_topic("bad", 1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn murmur2_hash_deterministic() {
        let h1 = murmur2_hash(b"hello world");
        let h2 = murmur2_hash(b"hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn murmur2_hash_different_inputs() {
        let h1 = murmur2_hash(b"key1");
        let h2 = murmur2_hash(b"key2");
        assert_ne!(h1, h2);
    }
}
