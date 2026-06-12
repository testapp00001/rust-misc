//! Topic management: creation, deletion, and partition lookup.
//!
//! A `Topic` groups one or more `Partition`s under a single name. The
//! `TopicManager` is the central registry that owns all topics and provides
//! an async-safe interface for lifecycle operations.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{info, warn};

use super::partition::Partition;

/// A topic comprising one or more partitions.
pub struct Topic {
    /// The topic name.
    pub name: String,
    /// Ordered partitions. `partitions[i]` has `partition_id == i`.
    pub partitions: Vec<Arc<Partition>>,
    /// Replication factor for this topic.
    pub replication_factor: u32,
}

/// Manages the lifecycle of topics: creation, deletion, and lookup.
///
/// Thread-safe via `Arc<RwLock<...>>` internally. All public methods are
/// async to allow cooperative locking in an async runtime.
#[derive(Clone)]
pub struct TopicManager {
    /// Topic name -> Topic mapping.
    topics: Arc<RwLock<HashMap<String, Arc<Topic>>>>,
    /// Default partition count for topics created without an explicit count.
    default_partitions: u32,
    /// Default replication factor for topics created without an explicit RF.
    default_replication_factor: u32,
    /// Root data directory on disk.
    data_dir: String,
}

impl TopicManager {
    /// Create a new topic manager.
    ///
    /// The data directory is created if it does not already exist.
    pub fn new(data_dir: String, default_partitions: u32, default_replication_factor: u32) -> Self {
        // Best-effort directory creation.
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            warn!(
                dir = %data_dir,
                error = %e,
                "Failed to create data directory"
            );
        }

        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            default_partitions,
            default_replication_factor,
            data_dir,
        }
    }

    /// Create a new topic with explicit partition count and replication factor.
    ///
    /// Each partition gets its own subdirectory under the data directory.
    /// Returns an error if a topic with the same name already exists.
    pub async fn create_topic(
        &self,
        name: &str,
        partitions: u32,
        replication_factor: u32,
    ) -> crate::error::Result<Arc<Topic>> {
        let mut topics = self.topics.write().await;

        if topics.contains_key(name) {
            return Err(crate::error::MqError::Internal(format!(
                "Topic '{}' already exists",
                name
            )));
        }

        if partitions == 0 {
            return Err(crate::error::MqError::Partition(format!(
                "Partition count must be > 0 for topic '{}'",
                name
            )));
        }

        if replication_factor == 0 {
            return Err(crate::error::MqError::Replication(format!(
                "Replication factor must be > 0 for topic '{}'",
                name
            )));
        }

        // Build partition replicas.
        let replica_ids: Vec<u64> = (1..=replication_factor as u64).collect();
        let leader = replica_ids[0];

        let mut partition_vec = Vec::with_capacity(partitions as usize);
        for i in 0..partitions {
            let partition_dir = format!("{}/{}/p{}", self.data_dir, name, i);

            // Best-effort directory creation.
            if let Err(e) = std::fs::create_dir_all(&partition_dir) {
                warn!(
                    dir = %partition_dir,
                    error = %e,
                    "Failed to create partition directory"
                );
            }

            let mut partition =
                Partition::new(name.to_string(), i, &partition_dir, 64 * 1024 * 1024);
            partition.leader = leader;
            partition.replicas = replica_ids.clone();

            partition_vec.push(Arc::new(partition));
        }

        let topic = Arc::new(Topic {
            name: name.to_string(),
            partitions: partition_vec,
            replication_factor,
        });

        info!(
            topic = name,
            partitions,
            replication_factor,
            "Topic created"
        );

        topics.insert(name.to_string(), topic.clone());
        Ok(topic)
    }

    /// Create a topic using the manager's default partition count and
    /// replication factor.
    pub async fn create_topic_default(&self, name: &str) -> crate::error::Result<Arc<Topic>> {
        self.create_topic(
            name,
            self.default_partitions,
            self.default_replication_factor,
        )
        .await
    }

    /// Get a topic by name.
    ///
    /// Returns `TopicNotFound` if no topic with the given name exists.
    pub async fn get_topic(&self, name: &str) -> crate::error::Result<Arc<Topic>> {
        let topics = self.topics.read().await;
        topics
            .get(name)
            .cloned()
            .ok_or_else(|| crate::error::MqError::TopicNotFound(name.to_string()))
    }

    /// List all topic names.
    pub async fn list_topics(&self) -> Vec<String> {
        let topics = self.topics.read().await;
        let mut names: Vec<String> = topics.keys().cloned().collect();
        names.sort();
        names
    }

    /// Delete a topic and its partitions.
    ///
    /// The topic's data directory is removed from disk (best-effort).
    /// Returns `TopicNotFound` if the topic does not exist.
    pub async fn delete_topic(&self, name: &str) -> crate::error::Result<()> {
        let mut topics = self.topics.write().await;

        if topics.remove(name).is_none() {
            return Err(crate::error::MqError::TopicNotFound(name.to_string()));
        }

        // Clean up data directory.
        let topic_dir = format!("{}/{}", self.data_dir, name);
        if let Err(e) = std::fs::remove_dir_all(&topic_dir) {
            warn!(
                dir = %topic_dir,
                error = %e,
                "Failed to remove topic data directory"
            );
        }

        info!(topic = name, "Topic deleted");
        Ok(())
    }

    /// Get a specific partition from a topic.
    ///
    /// Returns `TopicNotFound` if the topic does not exist, or
    /// `PartitionNotFound` if the partition index is out of range.
    pub async fn get_partition(
        &self,
        topic: &str,
        partition: u32,
    ) -> crate::error::Result<Arc<Partition>> {
        let t = self.get_topic(topic).await?;
        t.partitions
            .get(partition as usize)
            .cloned()
            .ok_or_else(|| crate::error::MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_manager() -> TopicManager {
        let dir = tempfile::tempdir().unwrap();
        TopicManager::new(dir.path().to_string_lossy().to_string(), 3, 1)
    }

    #[tokio::test]
    async fn create_topic() {
        let mgr = temp_manager();
        let topic = mgr.create_topic("orders", 6, 2).await.unwrap();

        assert_eq!(topic.name, "orders");
        assert_eq!(topic.partitions.len(), 6);
        assert_eq!(topic.replication_factor, 2);

        // Each partition should have the correct ID.
        for (i, p) in topic.partitions.iter().enumerate() {
            assert_eq!(p.partition_id, i as u32);
            assert_eq!(p.topic, "orders");
            assert_eq!(p.leader, 1);
            assert_eq!(p.replicas, vec![1, 2]);
        }
    }

    #[tokio::test]
    async fn create_topic_default() {
        let mgr = temp_manager();
        let topic = mgr.create_topic_default("events").await.unwrap();

        assert_eq!(topic.partitions.len(), 3);
        assert_eq!(topic.replication_factor, 1);
    }

    #[tokio::test]
    async fn create_duplicate_topic_fails() {
        let mgr = temp_manager();
        mgr.create_topic("t", 1, 1).await.unwrap();
        let result = mgr.create_topic("t", 1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_zero_partitions_fails() {
        let mgr = temp_manager();
        let result = mgr.create_topic("t", 0, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_zero_replication_fails() {
        let mgr = temp_manager();
        let result = mgr.create_topic("t", 1, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_topic() {
        let mgr = temp_manager();
        mgr.create_topic("orders", 3, 1).await.unwrap();

        let topic = mgr.get_topic("orders").await.unwrap();
        assert_eq!(topic.name, "orders");
        assert_eq!(topic.partitions.len(), 3);
    }

    #[tokio::test]
    async fn get_nonexistent_topic_fails() {
        let mgr = temp_manager();
        let result = mgr.get_topic("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_topics_sorted() {
        let mgr = temp_manager();
        mgr.create_topic("gamma", 1, 1).await.unwrap();
        mgr.create_topic("alpha", 1, 1).await.unwrap();
        mgr.create_topic("beta", 1, 1).await.unwrap();

        let topics = mgr.list_topics().await;
        assert_eq!(topics, vec!["alpha", "beta", "gamma"]);
    }

    #[tokio::test]
    async fn list_topics_empty() {
        let mgr = temp_manager();
        let topics = mgr.list_topics().await;
        assert!(topics.is_empty());
    }

    #[tokio::test]
    async fn delete_topic() {
        let mgr = temp_manager();
        mgr.create_topic("temp", 1, 1).await.unwrap();
        assert_eq!(mgr.list_topics().await.len(), 1);

        mgr.delete_topic("temp").await.unwrap();
        assert!(mgr.list_topics().await.is_empty());
    }

    #[tokio::test]
    async fn delete_nonexistent_topic_fails() {
        let mgr = temp_manager();
        let result = mgr.delete_topic("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_then_recreate() {
        let mgr = temp_manager();
        mgr.create_topic("t", 2, 1).await.unwrap();
        mgr.delete_topic("t").await.unwrap();

        let topic = mgr.create_topic("t", 4, 1).await.unwrap();
        assert_eq!(topic.partitions.len(), 4);
    }

    #[tokio::test]
    async fn get_partition() {
        let mgr = temp_manager();
        mgr.create_topic("t", 4, 2).await.unwrap();

        let p = mgr.get_partition("t", 2).await.unwrap();
        assert_eq!(p.partition_id, 2);
        assert_eq!(p.topic, "t");
    }

    #[tokio::test]
    async fn get_partition_out_of_range() {
        let mgr = temp_manager();
        mgr.create_topic("t", 2, 1).await.unwrap();

        let result = mgr.get_partition("t", 99).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_partition_nonexistent_topic() {
        let mgr = temp_manager();
        let result = mgr.get_partition("nope", 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn topic_partitions_are_independent() {
        let mgr = temp_manager();
        let topic = mgr.create_topic("t", 3, 1).await.unwrap();

        // Append to different partitions.
        use super::super::message::Message;
        topic.partitions[0]
            .append(vec![Message::new(None, b"p0".to_vec())])
            .await
            .unwrap();
        topic.partitions[1]
            .append(vec![Message::new(None, b"p1".to_vec())])
            .await
            .unwrap();

        // Each partition has its own log.
        let (m0, _) = topic.partitions[0].fetch(0, 1024).await.unwrap();
        let (m1, _) = topic.partitions[1].fetch(0, 1024).await.unwrap();
        let (m2, _) = topic.partitions[2].fetch(0, 1024).await.unwrap();

        assert_eq!(m0[0].value, b"p0");
        assert_eq!(m1[0].value, b"p1");
        assert!(m2.is_empty());
    }

    #[tokio::test]
    async fn multiple_topics_coexist() {
        let mgr = temp_manager();
        mgr.create_topic("a", 1, 1).await.unwrap();
        mgr.create_topic("b", 2, 1).await.unwrap();
        mgr.create_topic("c", 3, 1).await.unwrap();

        let topics = mgr.list_topics().await;
        assert_eq!(topics, vec!["a", "b", "c"]);

        let t_a = mgr.get_topic("a").await.unwrap();
        let t_b = mgr.get_topic("b").await.unwrap();
        let t_c = mgr.get_topic("c").await.unwrap();

        assert_eq!(t_a.partitions.len(), 1);
        assert_eq!(t_b.partitions.len(), 2);
        assert_eq!(t_c.partitions.len(), 3);
    }
}
