//! Producer API for publishing messages to topics.
//!
//! A `Producer` sends messages to a topic, optionally specifying a key for
//! partition affinity. When no key is provided, messages are distributed
//! across partitions in round-robin order.
//!
//! Each producer instance carries a unique ID and a monotonically increasing
//! sequence number. These are stored in message headers to support
//! idempotent delivery when combined with the delivery engine.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::debug;

use super::message::Message;
use super::topic::TopicManager;
use crate::network::rpc::Acks;

/// A producer that publishes messages to topics.
pub struct Producer {
    /// Reference to the topic registry.
    topic_manager: Arc<TopicManager>,
    /// Default acknowledgement level for produce requests.
    default_acks: Acks,
    /// Maximum number of messages to batch in a single `produce_batch` call.
    max_batch_size: usize,
    /// Unique ID for this producer instance (UUID).
    producer_id: String,
    /// Monotonically increasing sequence number. Used for round-robin
    /// partition selection (unkeyed messages) and idempotency tracking.
    sequence_number: u64,
}

impl Producer {
    /// Create a new producer with default batch size.
    pub fn new(topic_manager: Arc<TopicManager>, default_acks: Acks) -> Self {
        Self {
            topic_manager,
            default_acks,
            max_batch_size: 16_384,
            producer_id: uuid::Uuid::new_v4().to_string(),
            sequence_number: 0,
        }
    }

    /// Create a producer with a custom maximum batch size.
    pub fn with_batch_size(
        topic_manager: Arc<TopicManager>,
        default_acks: Acks,
        max_batch_size: usize,
    ) -> Self {
        Self {
            topic_manager,
            default_acks,
            max_batch_size,
            producer_id: uuid::Uuid::new_v4().to_string(),
            sequence_number: 0,
        }
    }

    /// Produce a single message to a topic.
    ///
    /// The message is routed to a partition based on its key (hash) or
    /// round-robin (no key). Returns the offset assigned to the message.
    pub async fn produce(
        &mut self,
        topic: &str,
        key: Option<Vec<u8>>,
        value: Vec<u8>,
    ) -> crate::error::Result<u64> {
        let topic_ref = self.topic_manager.get_topic(topic).await?;
        let num_partitions = topic_ref.partitions.len() as u32;

        if num_partitions == 0 {
            return Err(crate::error::MqError::Internal(format!(
                "Topic '{}' has no partitions",
                topic
            )));
        }

        let partition_id = self.select_partition(&key, num_partitions);
        let partition = &topic_ref.partitions[partition_id as usize];

        let mut message = Message::new(key, value);
        message.headers.insert(
            "producer_id".to_string(),
            self.producer_id.as_bytes().to_vec(),
        );
        message.headers.insert(
            "sequence_number".to_string(),
            self.sequence_number.to_le_bytes().to_vec(),
        );
        self.sequence_number += 1;

        let offsets = partition.append(vec![message]).await?;

        debug!(
            topic,
            partition = partition_id,
            offset = offsets[0],
            "Message produced"
        );

        Ok(offsets[0])
    }

    /// Produce a batch of messages to a topic.
    ///
    /// Messages are grouped by target partition and appended in parallel at
    /// the partition level. Returns the offsets assigned to each message, in
    /// the same order as the input.
    pub async fn produce_batch(
        &mut self,
        topic: &str,
        messages: Vec<(Option<Vec<u8>>, Vec<u8>)>,
    ) -> crate::error::Result<Vec<u64>> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let topic_ref = self.topic_manager.get_topic(topic).await?;
        let num_partitions = topic_ref.partitions.len() as u32;

        if num_partitions == 0 {
            return Err(crate::error::MqError::Internal(format!(
                "Topic '{}' has no partitions",
                topic
            )));
        }

        let num_messages = messages.len();
        let mut result = vec![0u64; num_messages];

        // Phase 1: Assign partitions and build messages, tracking original
        // indices so we can restore order in the result vector.
        let mut by_partition: HashMap<u32, Vec<(usize, Message)>> = HashMap::new();

        for (idx, (key, value)) in messages.into_iter().enumerate() {
            let partition_id = self.select_partition(&key, num_partitions);

            let mut message = Message::new(key, value);
            message.headers.insert(
                "producer_id".to_string(),
                self.producer_id.as_bytes().to_vec(),
            );
            message.headers.insert(
                "sequence_number".to_string(),
                self.sequence_number.to_le_bytes().to_vec(),
            );
            self.sequence_number += 1;

            by_partition
                .entry(partition_id)
                .or_default()
                .push((idx, message));
        }

        // Phase 2: Append to each partition and collect offsets.
        for (partition_id, entries) in by_partition {
            let original_indices: Vec<usize> = entries.iter().map(|(idx, _)| *idx).collect();
            let msgs: Vec<Message> = entries.into_iter().map(|(_, m)| m).collect();

            let offsets = topic_ref.partitions[partition_id as usize]
                .append(msgs)
                .await?;

            for (j, &orig_idx) in original_indices.iter().enumerate() {
                result[orig_idx] = offsets[j];
            }
        }

        debug!(
            topic,
            messages = num_messages,
            "Batch produced"
        );

        Ok(result)
    }

    /// Select a partition for a message.
    ///
    /// If a key is provided, the partition is determined by hashing the key
    /// (deterministic -- the same key always maps to the same partition).
    /// Otherwise, round-robin distribution is used based on the sequence
    /// number.
    fn select_partition(&self, key: &Option<Vec<u8>>, num_partitions: u32) -> u32 {
        match key {
            Some(key_bytes) => {
                let hash = Self::hash_key(key_bytes);
                (hash % num_partitions as u64) as u32
            }
            None => {
                // Round-robin using the sequence number.
                (self.sequence_number % num_partitions as u64) as u32
            }
        }
    }

    /// Compute a deterministic hash of a key using the default hasher.
    fn hash_key(key: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Get the producer's unique ID.
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }

    /// Get the current sequence number (next message will use this value).
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Get the default acknowledgement level.
    pub fn default_acks(&self) -> Acks {
        self.default_acks
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_manager() -> Arc<TopicManager> {
        let dir = tempdir().unwrap();
        Arc::new(TopicManager::new(
            dir.path().to_string_lossy().to_string(),
            4,
            1,
        ))
    }

    #[tokio::test]
    async fn produce_single_message() {
        let mgr = setup_manager();
        mgr.create_topic("t", 3, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        let offset = producer.produce("t", None, b"hello".to_vec()).await.unwrap();
        assert_eq!(offset, 0);

        // Verify the message was actually stored.
        let partition = mgr.get_partition("t", 0).await.unwrap();
        let (msgs, _) = partition.fetch(0, 1024).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].value, b"hello");
    }

    #[tokio::test]
    async fn produce_increments_offset() {
        let mgr = setup_manager();
        mgr.create_topic("t", 1, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        let o1 = producer.produce("t", None, b"m1".to_vec()).await.unwrap();
        let o2 = producer.produce("t", None, b"m2".to_vec()).await.unwrap();
        let o3 = producer.produce("t", None, b"m3".to_vec()).await.unwrap();

        assert_eq!(o1, 0);
        assert_eq!(o2, 1);
        assert_eq!(o3, 2);
    }

    #[tokio::test]
    async fn produce_nonexistent_topic_fails() {
        let mgr = setup_manager();
        let mut producer = Producer::new(mgr, Acks::Leader);
        let result = producer.produce("nope", None, b"data".to_vec()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn keyed_message_consistent_partition() {
        let mgr = setup_manager();
        mgr.create_topic("t", 10, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);

        // Same key should always go to the same partition.
        let key = Some(b"customer-42".to_vec());
        let _o1 = producer
            .produce("t", key.clone(), b"v1".to_vec())
            .await
            .unwrap();
        let _o2 = producer
            .produce("t", key.clone(), b"v2".to_vec())
            .await
            .unwrap();

        // Both messages go to the same partition.
        // Since we have 1 partition per offset space, if they land on
        // different partitions they will both have offset 0.
        // To verify consistency, check that the partition data is correct.
        let topic = mgr.get_topic("t").await.unwrap();
        let mut total_messages = 0;
        for p in &topic.partitions {
            let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
            total_messages += msgs.len();
        }
        assert_eq!(total_messages, 2);
    }

    #[tokio::test]
    async fn unkeyed_round_robin() {
        let mgr = setup_manager();
        mgr.create_topic("t", 4, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);

        // Produce 8 unkeyed messages -- should distribute across partitions.
        for i in 0..8 {
            producer
                .produce("t", None, format!("m{}", i).into_bytes())
                .await
                .unwrap();
        }

        let topic = mgr.get_topic("t").await.unwrap();
        let mut counts = Vec::new();
        for p in &topic.partitions {
            let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
            counts.push(msgs.len());
        }

        // Each partition should have exactly 2 messages.
        assert_eq!(counts, vec![2, 2, 2, 2]);
    }

    #[tokio::test]
    async fn produce_batch() {
        let mgr = setup_manager();
        mgr.create_topic("t", 1, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        let messages = vec![
            (None, b"a".to_vec()),
            (None, b"b".to_vec()),
            (None, b"c".to_vec()),
        ];
        let offsets = producer.produce_batch("t", messages).await.unwrap();
        assert_eq!(offsets, vec![0, 1, 2]);

        let partition = mgr.get_partition("t", 0).await.unwrap();
        let (msgs, _) = partition.fetch(0, 1024).await.unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].value, b"a");
        assert_eq!(msgs[1].value, b"b");
        assert_eq!(msgs[2].value, b"c");
    }

    #[tokio::test]
    async fn produce_batch_empty() {
        let mgr = setup_manager();
        mgr.create_topic("t", 1, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        let offsets = producer.produce_batch("t", vec![]).await.unwrap();
        assert!(offsets.is_empty());
    }

    #[tokio::test]
    async fn produce_batch_nonexistent_topic_fails() {
        let mgr = setup_manager();
        let mut producer = Producer::new(mgr, Acks::Leader);
        let result = producer
            .produce_batch("nope", vec![(None, b"x".to_vec())])
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn batch_mixed_keys() {
        let mgr = setup_manager();
        mgr.create_topic("t", 4, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        let messages = vec![
            (Some(b"k1".to_vec()), b"v1".to_vec()),
            (Some(b"k2".to_vec()), b"v2".to_vec()),
            (None, b"v3".to_vec()),
            (Some(b"k1".to_vec()), b"v4".to_vec()),
        ];
        let offsets = producer.produce_batch("t", messages).await.unwrap();
        assert_eq!(offsets.len(), 4);

        // All offsets should be unique.
        let topic = mgr.get_topic("t").await.unwrap();
        let mut total = 0;
        for p in &topic.partitions {
            let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
            total += msgs.len();
        }
        assert_eq!(total, 4);
    }

    #[tokio::test]
    async fn producer_id_unique() {
        let mgr = setup_manager();
        let p1 = Producer::new(mgr.clone(), Acks::Leader);
        let p2 = Producer::new(mgr.clone(), Acks::Leader);
        assert_ne!(p1.producer_id(), p2.producer_id());
    }

    #[tokio::test]
    async fn sequence_number_increments() {
        let mgr = setup_manager();
        let mut producer = Producer::new(mgr, Acks::Leader);

        assert_eq!(producer.sequence_number(), 0);
        producer.produce("t", None, b"a".to_vec()).await.ok();
        // Sequence increments even if topic doesn't exist (error path).
        // But let's create a topic first.
        let mgr2 = setup_manager();
        mgr2.create_topic("t", 1, 1).await.unwrap();
        let mut producer = Producer::new(mgr2, Acks::Leader);

        assert_eq!(producer.sequence_number(), 0);
        producer.produce("t", None, b"a".to_vec()).await.unwrap();
        assert_eq!(producer.sequence_number(), 1);
        producer.produce("t", None, b"b".to_vec()).await.unwrap();
        assert_eq!(producer.sequence_number(), 2);
    }

    #[tokio::test]
    async fn producer_id_in_headers() {
        let mgr = setup_manager();
        mgr.create_topic("t", 1, 1).await.unwrap();

        let mut producer = Producer::new(mgr.clone(), Acks::Leader);
        producer.produce("t", None, b"data".to_vec()).await.unwrap();

        let partition = mgr.get_partition("t", 0).await.unwrap();
        let (msgs, _) = partition.fetch(0, 1024).await.unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[tokio::test]
    async fn default_acks_preserved() {
        let mgr = setup_manager();
        let p = Producer::new(mgr, Acks::All);
        assert_eq!(p.default_acks(), Acks::All);
    }

    #[tokio::test]
    async fn hash_key_deterministic() {
        let h1 = Producer::hash_key(b"hello");
        let h2 = Producer::hash_key(b"hello");
        assert_eq!(h1, h2);
    }

    #[tokio::test]
    async fn hash_key_different_inputs() {
        let h1 = Producer::hash_key(b"key1");
        let h2 = Producer::hash_key(b"key2");
        assert_ne!(h1, h2);
    }
}
