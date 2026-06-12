//! Consumer with consumer-group support.
//!
//! A `Consumer` belongs to a consumer group. Partitions of subscribed topics
//! are assigned to group members using a round-robin strategy. The consumer
//! tracks its local read offsets and can commit them to the
//! `ConsumerGroupCoordinator` for durability.
//!
//! The coordinator manages group membership, partition assignment, and
//! committed offsets. It triggers a rebalance whenever a member joins or
//! leaves.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;

use super::message::FetchMessage;
use super::topic::TopicManager;

/// Default maximum bytes to fetch per partition per poll.
const DEFAULT_FETCH_BYTES: u64 = 1024 * 1024; // 1 MiB

// ---------------------------------------------------------------------------
// Consumer
// ---------------------------------------------------------------------------

/// A consumer that reads messages from assigned partitions within a
/// consumer group.
pub struct Consumer {
    /// Consumer group name.
    consumer_group: String,
    /// Auto-generated consumer instance ID.
    consumer_id: String,
    /// Reference to the topic registry.
    topic_manager: Arc<TopicManager>,
    /// Consumer group coordinator for membership and offset tracking.
    coordinator: Arc<ConsumerGroupCoordinator>,
    /// Local offset cache: `(topic, partition) -> next offset to read`.
    offsets: Arc<RwLock<HashMap<(String, u32), u64>>>,
    /// Currently assigned `(topic, partition)` pairs.
    assigned_partitions: Vec<(String, u32)>,
}

impl Consumer {
    /// Create a new consumer.
    ///
    /// The consumer ID is auto-generated. Call `subscribe` to join a
    /// consumer group and receive a partition assignment.
    pub fn new(
        consumer_group: String,
        topic_manager: Arc<TopicManager>,
        coordinator: Arc<ConsumerGroupCoordinator>,
    ) -> Self {
        let consumer_id = uuid::Uuid::new_v4().to_string();
        Self {
            consumer_group,
            consumer_id,
            topic_manager,
            coordinator,
            offsets: Arc::new(RwLock::new(HashMap::new())),
            assigned_partitions: Vec::new(),
        }
    }

    /// Subscribe to a topic.
    ///
    /// Joins the consumer group (if not already a member), registers the
    /// topic, and receives a partition assignment via the coordinator.
    pub async fn subscribe(&mut self, topic: &str) -> crate::error::Result<()> {
        // Verify the topic exists.
        let topic_ref = self.topic_manager.get_topic(topic).await?;
        let num_partitions = topic_ref.partitions.len() as u32;

        // Join the group.
        self.coordinator
            .join_group(&self.consumer_group, &self.consumer_id)
            .await?;

        // Register the topic subscription (triggers rebalance).
        self.coordinator
            .subscribe_topic(&self.consumer_group, topic, num_partitions)
            .await?;

        // Fetch this consumer's assignment.
        let assignment = self
            .coordinator
            .get_assignment(&self.consumer_group, &self.consumer_id)
            .await;

        self.assigned_partitions = assignment;

        // Initialise local offsets from any previously committed offsets.
        let mut offsets = self.offsets.write().await;
        for (ref t, p) in &self.assigned_partitions {
            let committed = self
                .coordinator
                .get_offset(&self.consumer_group, t, *p)
                .await;
            offsets.entry((t.clone(), *p)).or_insert(committed);
        }

        debug!(
            group = %self.consumer_group,
            consumer = %self.consumer_id,
            topic,
            partitions = self.assigned_partitions.len(),
            "Subscribed"
        );

        Ok(())
    }

    /// Poll for new messages from all assigned partitions.
    ///
    /// Before fetching, the consumer re-queries the coordinator for its
    /// current assignment so that it picks up any rebalance that happened
    /// since the last poll.
    ///
    /// Fetches up to `DEFAULT_FETCH_BYTES` per partition and advances the
    /// local offset. Returns an empty vector if no new messages are
    /// available.
    pub async fn poll(&mut self) -> crate::error::Result<Vec<FetchMessage>> {
        // Re-fetch assignment from the coordinator to handle rebalances.
        let assignment = self
            .coordinator
            .get_assignment(&self.consumer_group, &self.consumer_id)
            .await;
        self.assigned_partitions = assignment;

        let mut all_messages = Vec::new();
        let mut offsets = self.offsets.write().await;

        for &(ref topic_name, partition_id) in &self.assigned_partitions {
            let current_offset = offsets
                .get(&(topic_name.clone(), partition_id))
                .copied()
                .unwrap_or(0);

            let partition = match self
                .topic_manager
                .get_partition(topic_name, partition_id)
                .await
            {
                Ok(p) => p,
                Err(_) => continue,
            };

            // If fetch returns OffsetOutOfRange (caught-up consumer),
            // treat it as "no new messages" rather than an error.
            let (messages, _hw) = match partition
                .fetch(current_offset, DEFAULT_FETCH_BYTES)
                .await
            {
                Ok(result) => result,
                Err(crate::error::MqError::OffsetOutOfRange { .. }) => continue,
                Err(e) => return Err(e),
            };

            if !messages.is_empty() {
                // Advance offset past the last message.
                let last_offset = messages.last().unwrap().offset + 1;
                offsets.insert((topic_name.clone(), partition_id), last_offset);
                all_messages.extend(messages);
            }
        }

        Ok(all_messages)
    }

    /// Commit an offset for a specific topic-partition.
    ///
    /// Updates both the local offset cache and the coordinator's durable
    /// store.
    pub async fn commit_offset(
        &self,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> crate::error::Result<()> {
        // Update local cache.
        {
            let mut offsets = self.offsets.write().await;
            offsets.insert((topic.to_string(), partition), offset);
        }

        // Commit to coordinator.
        self.coordinator
            .commit_offset(&self.consumer_group, topic, partition, offset)
            .await
    }

    /// Seek to a specific offset for a topic-partition.
    ///
    /// This only changes the local offset. It does NOT commit the offset
    /// to the coordinator. Call `commit_offset` explicitly if you want the
    /// seek to be durable.
    pub async fn seek(
        &mut self,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> crate::error::Result<()> {
        let mut offsets = self.offsets.write().await;
        offsets.insert((topic.to_string(), partition), offset);

        debug!(
            topic,
            partition,
            offset,
            "Seeked"
        );

        Ok(())
    }

    /// Leave the consumer group and release partition assignments.
    pub async fn leave(&self) -> crate::error::Result<()> {
        self.coordinator
            .leave_group(&self.consumer_group, &self.consumer_id)
            .await
    }

    /// Get the current assigned partitions.
    pub fn assigned_partitions(&self) -> &[(String, u32)] {
        &self.assigned_partitions
    }

    /// Get the consumer group name.
    pub fn consumer_group(&self) -> &str {
        &self.consumer_group
    }

    /// Get this consumer's unique instance ID.
    pub fn consumer_id(&self) -> &str {
        &self.consumer_id
    }
}

// ---------------------------------------------------------------------------
// ConsumerGroupCoordinator
// ---------------------------------------------------------------------------

/// Coordinates consumer group membership, partition assignment, and
/// committed offsets.
pub struct ConsumerGroupCoordinator {
    /// Group name -> group state.
    groups: Arc<RwLock<HashMap<String, ConsumerGroupState>>>,
}

/// Internal state for a single consumer group.
struct ConsumerGroupState {
    /// Consumer instance IDs that have joined the group.
    members: Vec<String>,
    /// Topic name -> number of partitions. Tracked so the coordinator can
    /// compute assignments without consulting the TopicManager.
    topic_partitions: HashMap<String, u32>,
    /// Consumer ID -> assigned `(topic, partition)` pairs.
    partition_assignment: HashMap<String, Vec<(String, u32)>>,
    /// `(topic, partition) -> committed offset`. Stored at the group level
    /// so it persists across consumer rebalances.
    offsets: HashMap<(String, u32), u64>,
}

impl ConsumerGroupCoordinator {
    /// Create a new coordinator with no groups.
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Join a consumer to a group.
    ///
    /// If the group does not exist, it is created. Joining always triggers
    /// a rebalance.
    pub async fn join_group(
        &self,
        group: &str,
        consumer_id: &str,
    ) -> crate::error::Result<()> {
        let mut groups = self.groups.write().await;
        let state = groups
            .entry(group.to_string())
            .or_insert_with(ConsumerGroupState::new);

        if !state.members.contains(&consumer_id.to_string()) {
            state.members.push(consumer_id.to_string());
        }

        // Rebalance with current membership.
        let members = state.members.clone();
        let topic_partitions = state.topic_partitions.clone();
        state.partition_assignment = Self::compute_assignment(&members, &topic_partitions);

        debug!(
            group,
            consumer = consumer_id,
            members = state.members.len(),
            "Consumer joined group"
        );

        Ok(())
    }

    /// Remove a consumer from a group and rebalance.
    ///
    /// If the group becomes empty it is removed entirely.
    pub async fn leave_group(
        &self,
        group: &str,
        consumer_id: &str,
    ) -> crate::error::Result<()> {
        let mut groups = self.groups.write().await;

        let state = groups.get_mut(group).ok_or_else(|| {
            crate::error::MqError::Consumer(format!("Consumer group '{}' not found", group))
        })?;

        state.members.retain(|m| m != consumer_id);
        state.partition_assignment.remove(consumer_id);

        // Rebalance.
        let members = state.members.clone();
        let topic_partitions = state.topic_partitions.clone();
        state.partition_assignment = Self::compute_assignment(&members, &topic_partitions);

        debug!(
            group,
            consumer = consumer_id,
            remaining = state.members.len(),
            "Consumer left group"
        );

        if state.members.is_empty() {
            groups.remove(group);
        }

        Ok(())
    }

    /// Subscribe a consumer group to a topic and trigger a rebalance.
    ///
    /// Records the number of partitions for the topic so that assignment
    /// can be computed.
    pub async fn subscribe_topic(
        &self,
        group: &str,
        topic: &str,
        num_partitions: u32,
    ) -> crate::error::Result<()> {
        let mut groups = self.groups.write().await;
        let state = groups
            .entry(group.to_string())
            .or_insert_with(ConsumerGroupState::new);

        state
            .topic_partitions
            .insert(topic.to_string(), num_partitions);

        // Rebalance.
        let members = state.members.clone();
        let topic_partitions = state.topic_partitions.clone();
        state.partition_assignment = Self::compute_assignment(&members, &topic_partitions);

        Ok(())
    }

    /// Get the partition assignment for a specific consumer.
    ///
    /// Returns an empty vector if the consumer is not a member of the
    /// group or has no assigned partitions.
    pub async fn get_assignment(&self, group: &str, consumer_id: &str) -> Vec<(String, u32)> {
        let groups = self.groups.read().await;
        groups
            .get(group)
            .and_then(|s| s.partition_assignment.get(consumer_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Commit a consumer offset for a specific topic-partition.
    pub async fn commit_offset(
        &self,
        group: &str,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> crate::error::Result<()> {
        let mut groups = self.groups.write().await;
        let state = groups.get_mut(group).ok_or_else(|| {
            crate::error::MqError::Consumer(format!("Consumer group '{}' not found", group))
        })?;

        state
            .offsets
            .insert((topic.to_string(), partition), offset);

        debug!(
            group,
            topic,
            partition,
            offset,
            "Offset committed"
        );

        Ok(())
    }

    /// Get the committed offset for a topic-partition within a group.
    ///
    /// Returns `0` if no offset has been committed (the default starting
    /// position).
    pub async fn get_offset(&self, group: &str, topic: &str, partition: u32) -> u64 {
        let groups = self.groups.read().await;
        groups
            .get(group)
            .and_then(|s| s.offsets.get(&(topic.to_string(), partition)))
            .copied()
            .unwrap_or(0)
    }

    /// List all consumer group names.
    pub async fn list_groups(&self) -> Vec<String> {
        let groups = self.groups.read().await;
        let mut names: Vec<String> = groups.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get the number of members in a group.
    pub async fn group_size(&self, group: &str) -> usize {
        let groups = self.groups.read().await;
        groups.get(group).map_or(0, |s| s.members.len())
    }

    // -----------------------------------------------------------------------
    // Assignment algorithm
    // -----------------------------------------------------------------------

    /// Compute a round-robin partition assignment across consumers.
    ///
    /// All `(topic, partition)` pairs are collected, sorted by topic name
    /// then partition index (for determinism), and distributed round-robin
    /// among the consumers.
    fn compute_assignment(
        members: &[String],
        topic_partitions: &HashMap<String, u32>,
    ) -> HashMap<String, Vec<(String, u32)>> {
        let mut assignment: HashMap<String, Vec<(String, u32)>> = HashMap::new();

        if members.is_empty() {
            return assignment;
        }

        for member in members {
            assignment.insert(member.clone(), Vec::new());
        }

        // Collect and sort all partitions for deterministic assignment.
        let mut all_partitions: Vec<(String, u32)> = Vec::new();
        let mut topics: Vec<&String> = topic_partitions.keys().collect();
        topics.sort();

        for topic in topics {
            let num = topic_partitions[topic];
            for p in 0..num {
                all_partitions.push((topic.clone(), p));
            }
        }

        // Round-robin assignment.
        for (i, tp) in all_partitions.into_iter().enumerate() {
            let member = &members[i % members.len()];
            assignment
                .get_mut(member)
                .unwrap()
                .push(tp);
        }

        assignment
    }
}

impl Default for ConsumerGroupCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConsumerGroupState
// ---------------------------------------------------------------------------

impl ConsumerGroupState {
    fn new() -> Self {
        Self {
            members: Vec::new(),
            topic_partitions: HashMap::new(),
            partition_assignment: HashMap::new(),
            offsets: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup() -> (Arc<TopicManager>, Arc<ConsumerGroupCoordinator>) {
        let dir = tempdir().unwrap();
        let tm = Arc::new(TopicManager::new(
            dir.path().to_string_lossy().to_string(),
            4,
            1,
        ));
        let coord = Arc::new(ConsumerGroupCoordinator::new());
        (tm, coord)
    }

    // -- Coordinator unit tests --

    #[tokio::test]
    async fn coordinator_join_and_leave() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        assert_eq!(coord.group_size("g1").await, 1);

        coord.join_group("g1", "c2").await.unwrap();
        assert_eq!(coord.group_size("g1").await, 2);

        coord.leave_group("g1", "c1").await.unwrap();
        assert_eq!(coord.group_size("g1").await, 1);

        coord.leave_group("g1", "c2").await.unwrap();
        // Group should be removed when empty.
        assert_eq!(coord.group_size("g1").await, 0);
    }

    #[tokio::test]
    async fn coordinator_leave_nonexistent_group() {
        let coord = ConsumerGroupCoordinator::new();
        let result = coord.leave_group("nope", "c1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn coordinator_subscribe_topic() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord
            .subscribe_topic("g1", "topic-a", 4)
            .await
            .unwrap();

        let assignment = coord.get_assignment("g1", "c1").await;
        assert_eq!(assignment.len(), 4);
    }

    #[tokio::test]
    async fn coordinator_round_robin_assignment() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord.join_group("g1", "c2").await.unwrap();
        coord
            .subscribe_topic("g1", "topic", 4)
            .await
            .unwrap();

        let a1 = coord.get_assignment("g1", "c1").await;
        let a2 = coord.get_assignment("g1", "c2").await;

        assert_eq!(a1.len(), 2);
        assert_eq!(a2.len(), 2);

        // Collect all assigned partitions and verify they are unique.
        let mut all = a1.clone();
        all.extend(a2.clone());
        all.sort();
        let mut expected: Vec<(String, u32)> = (0..4).map(|p| ("topic".to_string(), p)).collect();
        expected.sort();
        assert_eq!(all, expected);
    }

    #[tokio::test]
    async fn coordinator_rebalance_on_leave() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord.join_group("g1", "c2").await.unwrap();
        coord
            .subscribe_topic("g1", "topic", 4)
            .await
            .unwrap();

        coord.leave_group("g1", "c2").await.unwrap();

        // c1 should now have all 4 partitions.
        let a1 = coord.get_assignment("g1", "c1").await;
        assert_eq!(a1.len(), 4);
    }

    #[tokio::test]
    async fn coordinator_commit_and_get_offset() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();

        // Default offset is 0.
        let off = coord.get_offset("g1", "topic", 0).await;
        assert_eq!(off, 0);

        coord
            .commit_offset("g1", "topic", 0, 42)
            .await
            .unwrap();
        let off = coord.get_offset("g1", "topic", 0).await;
        assert_eq!(off, 42);
    }

    #[tokio::test]
    async fn coordinator_commit_nonexistent_group() {
        let coord = ConsumerGroupCoordinator::new();
        let result = coord.commit_offset("nope", "t", 0, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn coordinator_get_assignment_nonexistent() {
        let coord = ConsumerGroupCoordinator::new();
        let a = coord.get_assignment("nope", "c1").await;
        assert!(a.is_empty());
    }

    #[tokio::test]
    async fn coordinator_list_groups() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord.join_group("g2", "c2").await.unwrap();
        coord.join_group("g3", "c3").await.unwrap();

        let mut groups = coord.list_groups().await;
        groups.sort();
        assert_eq!(groups, vec!["g1", "g2", "g3"]);
    }

    #[tokio::test]
    async fn coordinator_multiple_topics() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord.join_group("g1", "c2").await.unwrap();
        coord
            .subscribe_topic("g1", "topic-a", 2)
            .await
            .unwrap();
        coord
            .subscribe_topic("g1", "topic-b", 3)
            .await
            .unwrap();

        let a1 = coord.get_assignment("g1", "c1").await;
        let a2 = coord.get_assignment("g1", "c2").await;
        let total: usize = a1.len() + a2.len();
        assert_eq!(total, 5); // 2 + 3
    }

    #[tokio::test]
    async fn coordinator_duplicate_join_idempotent() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord.join_group("g1", "c1").await.unwrap();
        assert_eq!(coord.group_size("g1").await, 1);
    }

    #[tokio::test]
    async fn coordinator_offsets_persist_across_rebalance() {
        let coord = ConsumerGroupCoordinator::new();
        coord.join_group("g1", "c1").await.unwrap();
        coord
            .subscribe_topic("g1", "topic", 2)
            .await
            .unwrap();

        coord
            .commit_offset("g1", "topic", 0, 100)
            .await
            .unwrap();
        coord
            .commit_offset("g1", "topic", 1, 200)
            .await
            .unwrap();

        // Add another consumer -- rebalance happens.
        coord.join_group("g1", "c2").await.unwrap();

        // Offsets should survive the rebalance.
        assert_eq!(coord.get_offset("g1", "topic", 0).await, 100);
        assert_eq!(coord.get_offset("g1", "topic", 1).await, 200);
    }

    // -- Consumer integration tests --

    #[tokio::test]
    async fn consumer_subscribe() {
        let (tm, coord) = setup();
        tm.create_topic("t", 4, 1).await.unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm.clone(), coord);
        consumer.subscribe("t").await.unwrap();

        assert_eq!(consumer.assigned_partitions().len(), 4);
    }

    #[tokio::test]
    async fn consumer_subscribe_nonexistent_topic() {
        let (tm, coord) = setup();
        let mut consumer = Consumer::new("g1".to_string(), tm, coord);
        let result = consumer.subscribe("nope").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn consumer_poll_returns_messages() {
        let (tm, coord) = setup();
        tm.create_topic("t", 1, 1).await.unwrap();

        // Produce some messages directly via the partition.
        let partition = tm.get_partition("t", 0).await.unwrap();
        use super::super::message::Message;
        partition
            .append(vec![
                Message::new(None, b"m1".to_vec()),
                Message::new(None, b"m2".to_vec()),
            ])
            .await
            .unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm, coord);
        consumer.subscribe("t").await.unwrap();

        let messages = consumer.poll().await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].value, b"m1");
        assert_eq!(messages[1].value, b"m2");
    }

    #[tokio::test]
    async fn consumer_poll_advances_offset() {
        let (tm, coord) = setup();
        tm.create_topic("t", 1, 1).await.unwrap();

        let partition = tm.get_partition("t", 0).await.unwrap();
        use super::super::message::Message;
        partition
            .append(vec![Message::new(None, b"m1".to_vec())])
            .await
            .unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm, coord);
        consumer.subscribe("t").await.unwrap();

        // First poll returns the message.
        let msgs = consumer.poll().await.unwrap();
        assert_eq!(msgs.len(), 1);

        // Second poll returns nothing (offset advanced).
        let msgs = consumer.poll().await.unwrap();
        assert!(msgs.is_empty());
    }

    #[tokio::test]
    async fn consumer_poll_picks_up_new_messages() {
        let (tm, coord) = setup();
        tm.create_topic("t", 1, 1).await.unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm.clone(), coord);
        consumer.subscribe("t").await.unwrap();

        // First poll: empty.
        let msgs = consumer.poll().await.unwrap();
        assert!(msgs.is_empty());

        // Produce a message after subscribing.
        let partition = tm.get_partition("t", 0).await.unwrap();
        use super::super::message::Message;
        partition
            .append(vec![Message::new(None, b"new".to_vec())])
            .await
            .unwrap();

        // Second poll: should pick up the new message.
        let msgs = consumer.poll().await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].value, b"new");
    }

    #[tokio::test]
    async fn consumer_commit_offset() {
        let (tm, coord) = setup();
        tm.create_topic("t", 1, 1).await.unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm, coord.clone());
        consumer.subscribe("t").await.unwrap();

        consumer.commit_offset("t", 0, 42).await.unwrap();

        let offset = coord.get_offset("g1", "t", 0).await;
        assert_eq!(offset, 42);
    }

    #[tokio::test]
    async fn consumer_seek() {
        let (tm, coord) = setup();
        tm.create_topic("t", 1, 1).await.unwrap();

        // Write 5 messages.
        let partition = tm.get_partition("t", 0).await.unwrap();
        use super::super::message::Message;
        for i in 0..5 {
            partition
                .append(vec![Message::new(None, format!("m{}", i).into_bytes())])
                .await
                .unwrap();
        }

        let mut consumer = Consumer::new("g1".to_string(), tm, coord);
        consumer.subscribe("t").await.unwrap();

        // Seek to offset 3.
        consumer.seek("t", 0, 3).await.unwrap();

        let msgs = consumer.poll().await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].value, b"m3");
        assert_eq!(msgs[1].value, b"m4");
    }

    #[tokio::test]
    async fn consumer_leave_group() {
        let (tm, coord) = setup();
        tm.create_topic("t", 4, 1).await.unwrap();

        let mut consumer = Consumer::new("g1".to_string(), tm, coord.clone());
        consumer.subscribe("t").await.unwrap();

        assert_eq!(coord.group_size("g1").await, 1);
        consumer.leave().await.unwrap();
        assert_eq!(coord.group_size("g1").await, 0);
    }

    #[tokio::test]
    async fn two_consumers_share_partitions() {
        let (tm, coord) = setup();
        tm.create_topic("t", 4, 1).await.unwrap();

        let mut c1 = Consumer::new("g1".to_string(), tm.clone(), coord.clone());
        c1.subscribe("t").await.unwrap();

        let mut c2 = Consumer::new("g1".to_string(), tm.clone(), coord.clone());
        c2.subscribe("t").await.unwrap();

        // Poll c1 so it picks up the rebalance triggered by c2 joining.
        c1.poll().await.unwrap();

        // Each consumer should have some partitions.
        let total = c1.assigned_partitions().len() + c2.assigned_partitions().len();
        assert_eq!(total, 4);
    }

    #[tokio::test]
    async fn consumer_id_unique() {
        let (tm, coord) = setup();
        let c1 = Consumer::new("g1".to_string(), tm.clone(), coord.clone());
        let c2 = Consumer::new("g1".to_string(), tm, coord);
        assert_ne!(c1.consumer_id(), c2.consumer_id());
    }

    #[tokio::test]
    async fn consumer_group_name() {
        let (tm, coord) = setup();
        let c = Consumer::new("my-group".to_string(), tm, coord);
        assert_eq!(c.consumer_group(), "my-group");
    }
}
