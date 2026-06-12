//! Integration tests for the production message queue.
//!
//! These tests exercise the public API surface across module boundaries,
//! verifying that the broker, storage, delivery, clock, and partitioning
//! subsystems work together correctly.

use std::collections::HashMap;
use std::sync::Arc;

use production_message_queue::broker::consumer::{Consumer, ConsumerGroupCoordinator};
use production_message_queue::broker::producer::Producer;
use production_message_queue::broker::topic::TopicManager;
use production_message_queue::clock::hlc::HybridLogicalClock;
use production_message_queue::delivery::idempotency::IdempotencyStore;
use production_message_queue::health::{HealthMonitor, HealthStatus};
use production_message_queue::network::rpc::Acks;
use production_message_queue::partitioning::hash_ring::HashRing;
use production_message_queue::storage::wal::WriteAheadLog;

// ---------------------------------------------------------------------------
// Helper: create a temporary TopicManager
// ---------------------------------------------------------------------------

fn temp_topic_manager() -> TopicManager {
    let dir = tempfile::tempdir().unwrap();
    // Leak the TempDir so it is not cleaned up during the test. This is
    // acceptable for integration tests where the OS handles cleanup.
    let path = dir.into_path();
    TopicManager::new(path.to_string_lossy().to_string(), 3, 1)
}

// ---------------------------------------------------------------------------
// 1. Produce and consume messages
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_produce_and_consume() {
    let tm = temp_topic_manager();
    tm.create_topic("test-topic", 3, 1).await.unwrap();
    let tm = Arc::new(tm);

    let mut producer = Producer::new(Arc::clone(&tm), Acks::Leader);
    let offset = producer
        .produce("test-topic", None, b"hello".to_vec())
        .await
        .unwrap();
    assert_eq!(offset, 0);

    // Read the message back from partition 0.
    let partition = tm.get_partition("test-topic", 0).await.unwrap();
    let (msgs, _hw) = partition.fetch(0, 1024 * 1024).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].value, b"hello");
    assert_eq!(msgs[0].offset, 0);
}

// ---------------------------------------------------------------------------
// 2. Topic creation and listing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_topic_creation_and_listing() {
    let tm = temp_topic_manager();

    tm.create_topic("topic-a", 3, 1).await.unwrap();
    tm.create_topic("topic-b", 6, 2).await.unwrap();
    tm.create_topic("topic-c", 2, 1).await.unwrap();

    let topics = tm.list_topics().await;
    assert_eq!(topics, vec!["topic-a", "topic-b", "topic-c"]);

    // Verify each topic has the correct partition count.
    let t_a = tm.get_topic("topic-a").await.unwrap();
    assert_eq!(t_a.partitions.len(), 3);

    let t_b = tm.get_topic("topic-b").await.unwrap();
    assert_eq!(t_b.partitions.len(), 6);

    let t_c = tm.get_topic("topic-c").await.unwrap();
    assert_eq!(t_c.partitions.len(), 2);
}

// ---------------------------------------------------------------------------
// 3. Multiple partitions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multiple_partitions() {
    let tm = temp_topic_manager();
    tm.create_topic("multi", 3, 1).await.unwrap();
    let tm = Arc::new(tm);

    let mut producer = Producer::new(Arc::clone(&tm), Acks::Leader);

    // Produce to each partition explicitly via key hashing.
    // With keys "p0", "p1", "p2" the messages will land on different
    // partitions (deterministic hash). We also produce unkeyed messages
    // to exercise round-robin.
    let o0 = producer
        .produce("multi", Some(b"p0".to_vec()), b"val-0".to_vec())
        .await
        .unwrap();
    let o1 = producer
        .produce("multi", Some(b"p1".to_vec()), b"val-1".to_vec())
        .await
        .unwrap();
    let o2 = producer
        .produce("multi", Some(b"p2".to_vec()), b"val-2".to_vec())
        .await
        .unwrap();

    // Each message should have a valid offset.
    assert!(o0 < 3);
    assert!(o1 < 3);
    assert!(o2 < 3);

    // Verify we can read messages from all 3 partitions.
    let topic = tm.get_topic("multi").await.unwrap();
    let mut total_messages = 0usize;
    for p in &topic.partitions {
        let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
        total_messages += msgs.len();
    }
    assert_eq!(total_messages, 3);
}

// ---------------------------------------------------------------------------
// 4. Consumer group coordination
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_consumer_group_coordination() {
    let tm = temp_topic_manager();
    tm.create_topic("cg-topic", 4, 1).await.unwrap();
    let tm = Arc::new(tm);
    let coord = Arc::new(ConsumerGroupCoordinator::new());

    // Produce messages across partitions.
    let mut producer = Producer::new(Arc::clone(&tm), Acks::Leader);
    for i in 0..4u32 {
        let key = format!("key-{}", i);
        producer
            .produce("cg-topic", Some(key.into_bytes()), format!("msg-{}", i).into_bytes())
            .await
            .unwrap();
    }

    // Create two consumers in the same group.
    let mut c1 = Consumer::new(
        "group-1".to_string(),
        Arc::clone(&tm),
        Arc::clone(&coord),
    );
    c1.subscribe("cg-topic").await.unwrap();

    let mut c2 = Consumer::new(
        "group-1".to_string(),
        Arc::clone(&tm),
        Arc::clone(&coord),
    );
    c2.subscribe("cg-topic").await.unwrap();

    // Together they should cover all 4 partitions.
    // Note: c1 subscribed first and got all 4; after c2 joins and triggers
    // a rebalance, c2's assignment reflects the new split. c1's local
    // assigned_partitions may be stale (still 4) because it has not
    // re-subscribed. We verify the coordinator's view instead.
    let a1 = coord.get_assignment("group-1", c1.consumer_id()).await;
    let a2 = coord.get_assignment("group-1", c2.consumer_id()).await;
    let total = a1.len() + a2.len();
    assert_eq!(total, 4, "coordinator should assign all 4 partitions");

    // Each consumer should have a unique ID.
    assert_ne!(c1.consumer_id(), c2.consumer_id());

    // Both are in the same group.
    assert_eq!(c1.consumer_group(), "group-1");
    assert_eq!(c2.consumer_group(), "group-1");

    // Each consumer can poll its assigned partitions and get messages.
    let msgs1 = c1.poll().await.unwrap();
    let msgs2 = c2.poll().await.unwrap();

    // The second consumer (c2) was assigned partitions after the rebalance
    // triggered by its subscribe, so it should have some partitions.
    assert!(
        !c2.assigned_partitions().is_empty(),
        "c2 should have at least one partition"
    );

    // Verify each consumer can read messages from its assigned partitions.
    // (The exact distribution depends on the round-robin assignment.)
    let mut all_values: Vec<Vec<u8>> = msgs1
        .iter()
        .chain(msgs2.iter())
        .map(|m| m.value.clone())
        .collect();
    all_values.sort();
    let mut expected: Vec<Vec<u8>> = (0..4u32)
        .map(|i| format!("msg-{}", i).into_bytes())
        .collect();
    expected.sort();
    assert_eq!(all_values, expected, "all 4 messages should be consumed");
}

// ---------------------------------------------------------------------------
// 5. Message ordering within partition
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_message_ordering_within_partition() {
    let tm = temp_topic_manager();
    tm.create_topic("order-topic", 1, 1).await.unwrap();

    // Append messages directly to the partition to verify ordering.
    let partition = tm.get_partition("order-topic", 0).await.unwrap();
    use production_message_queue::broker::message::Message;

    for i in 0..100u64 {
        let msg = Message::new(None, i.to_le_bytes().to_vec());
        let offsets = partition.append(vec![msg]).await.unwrap();
        assert_eq!(offsets, vec![i], "offsets must be sequential");
    }

    // Read all messages back and verify ordering.
    let (msgs, _hw) = partition.fetch(0, 10 * 1024 * 1024).await.unwrap();
    assert_eq!(msgs.len(), 100);

    for (i, msg) in msgs.iter().enumerate() {
        assert_eq!(msg.offset, i as u64);
        let value = u64::from_le_bytes(msg.value[..8].try_into().unwrap());
        assert_eq!(value, i as u64);
    }
}

// ---------------------------------------------------------------------------
// 6. HLC timestamp ordering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hlc_timestamp_ordering() {
    let mut clock1 = HybridLogicalClock::new(1);
    let mut clock2 = HybridLogicalClock::new(2);

    // Local events on clock1.
    let t1 = clock1.now(1000);
    let t2 = clock1.now(1001);
    let t3 = clock1.now(1001); // Same physical time, logical increments.

    assert!(t1 < t2);
    assert!(t2 < t3);

    // Receive a remote timestamp on clock2.
    let t4 = clock2.receive(1002, t2);
    assert!(t4 > t2);

    // The node_id should be preserved.
    assert_eq!(t1.node_id, 1);
    assert_eq!(t4.node_id, 2);

    // Physical time should be non-decreasing.
    assert!(t4.physical >= t2.physical);
}

// ---------------------------------------------------------------------------
// 7. Idempotency deduplication
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_idempotency_deduplication() {
    let mut store = IdempotencyStore::new(1000);

    // First message from producer-1 is accepted.
    assert!(store.record("producer-1", 0));
    // Duplicate is rejected.
    assert!(!store.record("producer-1", 0));
    // Next sequence is accepted.
    assert!(store.record("producer-1", 1));

    // Different producer with the same sequence is independent.
    assert!(store.record("producer-2", 0));
    assert!(!store.record("producer-2", 0));

    // is_duplicate returns true for any sequence that has been recorded.
    assert!(store.is_duplicate("producer-1", 0));
    assert!(store.is_duplicate("producer-1", 1)); // seq 1 was recorded above
    assert!(store.is_duplicate("producer-2", 0));
    assert!(!store.is_duplicate("producer-2", 1)); // seq 1 was never recorded for producer-2

    // Two producers tracked.
    assert_eq!(store.producer_count(), 2);

    // Remove a producer.
    store.remove_producer("producer-1");
    assert_eq!(store.producer_count(), 1);
    assert!(!store.is_duplicate("producer-1", 0));
}

// ---------------------------------------------------------------------------
// 8. WAL durability
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_wal_durability() {
    let dir = tempfile::tempdir().unwrap();
    let wal_path = dir.path().join("test.wal");

    // Write entries.
    {
        let mut wal = WriteAheadLog::open(&wal_path).unwrap();
        let s0 = wal.append(b"message-1").unwrap();
        let s1 = wal.append(b"message-2").unwrap();
        let s2 = wal.append(b"message-3").unwrap();
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }

    // Recover by reopening the WAL.
    let wal = WriteAheadLog::open(&wal_path).unwrap();
    let entries = wal.read_all().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].data, b"message-1");
    assert_eq!(entries[0].sequence, 0);
    assert!(entries[0].verify());

    assert_eq!(entries[1].data, b"message-2");
    assert_eq!(entries[1].sequence, 1);
    assert!(entries[1].verify());

    assert_eq!(entries[2].data, b"message-3");
    assert_eq!(entries[2].sequence, 2);
    assert!(entries[2].verify());

    // Sequence counter should be recovered.
    assert_eq!(wal.entry_count(), 3);
}

// ---------------------------------------------------------------------------
// 9. Consistent hashing distribution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_consistent_hashing_distribution() {
    let ring = HashRing::new(vec![0, 1, 2], 150);

    // Distribute 10,000 keys across 3 partitions.
    let mut counts: HashMap<u32, u32> = HashMap::new();
    for i in 0..10_000 {
        let key = format!("key-{}", i);
        let partition = ring.get_partition(key.as_bytes());
        *counts.entry(partition).or_insert(0) += 1;
    }

    // All 3 partitions should receive some keys.
    assert_eq!(counts.len(), 3, "all partitions should be used");

    // No partition should be starved (each should have > 10% of keys).
    for (partition, count) in &counts {
        assert!(
            *count > 1000,
            "Partition {} only received {} keys (expected > 1000)",
            partition,
            count
        );
    }

    // Test that adding a new partition redistributes some but not all keys.
    let mut ring2 = HashRing::new(vec![0, 1], 150);
    let before: Vec<u32> = (0..200)
        .map(|i| ring2.get_partition(format!("key-{}", i).as_bytes()))
        .collect();

    ring2.add_partition(2);
    let after: Vec<u32> = (0..200)
        .map(|i| ring2.get_partition(format!("key-{}", i).as_bytes()))
        .collect();

    // Some keys should remain on their original partitions.
    let unchanged = before
        .iter()
        .zip(after.iter())
        .filter(|(a, b)| a == b)
        .count();
    assert!(
        unchanged > 0,
        "at least some keys should remain on the same partition"
    );

    // But not all keys should remain (some should be redistributed).
    assert!(
        unchanged < 200,
        "not all keys should stay on the same partition"
    );

    // Test partition removal.
    ring2.remove_partition(1);
    assert_eq!(ring2.partition_count(), 2);

    // After removal, all keys should map to one of the remaining partitions.
    for i in 0..100 {
        let key = format!("key-{}", i);
        let p = ring2.get_partition(key.as_bytes());
        assert!(p == 0 || p == 2, "unexpected partition {}", p);
    }
}

// ---------------------------------------------------------------------------
// 10. Health check system
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_check_system() {
    let monitor = HealthMonitor::new();

    // Initially healthy.
    let check = monitor.check();
    assert_eq!(check.status, HealthStatus::Healthy);
    assert!(monitor.is_alive());
    assert!(monitor.is_ready());

    // Mark as leader.
    monitor.set_leader(true);
    assert!(monitor.is_leader());
    let check = monitor.check();
    assert!(check.is_leader);

    // Storage failure makes it unhealthy.
    monitor.set_storage_healthy(false);
    assert_eq!(monitor.check().status, HealthStatus::Unhealthy);
    assert!(!monitor.is_ready());

    // Restore storage, but leave peers disconnected -> degraded.
    monitor.set_storage_healthy(true);
    monitor.set_total_peers(3);
    monitor.set_connected_peers(2);
    assert_eq!(monitor.check().status, HealthStatus::Degraded);

    // WAL failure also makes it unhealthy.
    monitor.set_wal_healthy(false);
    assert_eq!(monitor.check().status, HealthStatus::Unhealthy);
    monitor.set_wal_healthy(true);

    // Connect all peers -> healthy again.
    monitor.set_connected_peers(3);
    assert_eq!(monitor.check().status, HealthStatus::Healthy);

    // High replication lag makes it degraded.
    monitor.set_replication_lag(5000);
    assert_eq!(monitor.check().status, HealthStatus::Degraded);

    // Low replication lag restores healthy.
    monitor.set_replication_lag(0);
    assert_eq!(monitor.check().status, HealthStatus::Healthy);

    // Liveness probe always returns true.
    assert!(monitor.is_alive());

    // Version is set.
    assert!(!check.version.is_empty());
}
