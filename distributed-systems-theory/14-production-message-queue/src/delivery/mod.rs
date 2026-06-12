//! Delivery engine handling idempotent produce, consumer group management,
//! and at-least-once / exactly-once delivery semantics.
//!
//! The delivery engine sits between the network layer and the storage layer,
//! ensuring messages are delivered correctly even in the face of retries,
//! network partitions, and broker failovers.

pub mod exactly_once;
pub mod idempotency;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use crate::error::{MqError, Result};

/// Status of a pending acknowledgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckStatus {
    /// Waiting for consumer acknowledgment.
    Pending,
    /// Consumer acknowledged within the timeout.
    Acked,
    /// Acknowledgment timed out; message should be redelivered.
    TimedOut,
    /// Acknowledgment was explicitly rejected by the consumer.
    Rejected,
}

/// A pending message awaiting acknowledgment.
#[derive(Debug, Clone)]
struct PendingAck {
    /// Unique producer-assigned ID for deduplication.
    producer_id: u64,
    /// Sequence number from the producer (monotonic within producer).
    sequence: u64,
    /// When the message was sent.
    sent_at: Instant,
    /// Current ack status.
    status: AckStatus,
}

/// Consumer group membership information.
#[derive(Debug, Clone)]
pub struct ConsumerGroup {
    /// The group name.
    pub name: String,
    /// Members of this group (consumer_id -> assignment).
    pub members: HashMap<String, ConsumerAssignment>,
    /// Generation ID (incremented on rebalance).
    pub generation: u64,
}

/// Partition assignment for a single consumer within a group.
#[derive(Debug, Clone)]
pub struct ConsumerAssignment {
    /// Consumer instance ID.
    pub consumer_id: String,
    /// Assigned topic-partition pairs.
    pub partitions: Vec<(String, u32)>,
    /// Current offset for each assigned partition.
    pub offsets: HashMap<(String, u32), u64>,
}

/// The delivery engine manages message delivery semantics.
pub struct DeliveryEngine {
    /// Whether idempotency checks are enabled.
    idempotency_enabled: bool,
    /// Maximum in-flight requests per producer.
    max_in_flight: u32,
    /// Acknowledgment timeout.
    ack_timeout: Duration,
    /// Pending acks, keyed by (producer_id, sequence).
    pending_acks: Mutex<HashMap<(u64, u64), PendingAck>>,
    /// Deduplication window: tracks the last sequence seen per producer.
    last_sequence: Mutex<HashMap<u64, u64>>,
    /// Consumer groups.
    consumer_groups: RwLock<HashMap<String, ConsumerGroup>>,
    /// Total messages delivered.
    delivered_count: AtomicU64,
    /// Total messages redelivered (due to timeout).
    redelivered_count: AtomicU64,
    /// Total deduplicated messages.
    deduplicated_count: AtomicU64,
}

impl DeliveryEngine {
    /// Create a new delivery engine.
    pub fn new(
        idempotency_enabled: bool,
        max_in_flight: u32,
        ack_timeout_ms: u64,
    ) -> Self {
        DeliveryEngine {
            idempotency_enabled,
            max_in_flight,
            ack_timeout: Duration::from_millis(ack_timeout_ms),
            pending_acks: Mutex::new(HashMap::new()),
            last_sequence: Mutex::new(HashMap::new()),
            consumer_groups: RwLock::new(HashMap::new()),
            delivered_count: AtomicU64::new(0),
            redelivered_count: AtomicU64::new(0),
            deduplicated_count: AtomicU64::new(0),
        }
    }

    /// Validate and record a new produce request from an idempotent producer.
    ///
    /// Returns `Ok(true)` if the message should be accepted, `Ok(false)` if it
    /// is a duplicate and should be silently acknowledged, or `Err` if the
    /// producer's sequence is too far out of order.
    pub fn idempotent_check(
        &self,
        producer_id: u64,
        sequence: u64,
    ) -> Result<bool> {
        if !self.idempotency_enabled {
            return Ok(true);
        }

        let mut last_seq = self.last_sequence.lock().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        match last_seq.get(&producer_id) {
            Some(&last) => {
                if sequence == last {
                    // Exact duplicate.
                    self.deduplicated_count
                        .fetch_add(1, Ordering::Relaxed);
                    debug!(
                        producer_id,
                        sequence,
                        "Duplicate message detected, deduplicating"
                    );
                    return Ok(false);
                }

                if sequence < last {
                    // Old message from before a retry window.
                    self.deduplicated_count
                        .fetch_add(1, Ordering::Relaxed);
                    debug!(
                        producer_id,
                        sequence,
                        last,
                        "Out-of-order duplicate, deduplicating"
                    );
                    return Ok(false);
                }

                if sequence > last + self.max_in_flight as u64 + 1 {
                    // Sequence is too far ahead; the producer may have lost
                    // state.
                    return Err(MqError::Producer(format!(
                        "Sequence {} is too far ahead of last seen {} \
                         (max_in_flight={})",
                        sequence, last, self.max_in_flight,
                    )));
                }

                last_seq.insert(producer_id, sequence);
                Ok(true)
            }
            None => {
                // First message from this producer.
                last_seq.insert(producer_id, sequence);
                Ok(true)
            }
        }
    }

    /// Record a pending acknowledgment for a produced message.
    pub fn record_pending_ack(
        &self,
        producer_id: u64,
        sequence: u64,
    ) -> Result<()> {
        let mut pending = self.pending_acks.lock().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        pending.insert(
            (producer_id, sequence),
            PendingAck {
                producer_id,
                sequence,
                sent_at: Instant::now(),
                status: AckStatus::Pending,
            },
        );

        Ok(())
    }

    /// Acknowledge a message.
    pub fn ack(
        &self,
        producer_id: u64,
        sequence: u64,
    ) -> Result<bool> {
        let mut pending = self.pending_acks.lock().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        if let Some(entry) = pending.remove(&(producer_id, sequence)) {
            self.delivered_count.fetch_add(1, Ordering::Relaxed);
            debug!(producer_id, sequence, "Message acknowledged");
            Ok(true)
        } else {
            debug!(producer_id, sequence, "Ack for unknown message");
            Ok(false)
        }
    }

    /// Reject a message (consumer explicitly refused it).
    pub fn reject(
        &self,
        producer_id: u64,
        sequence: u64,
    ) -> Result<()> {
        let mut pending = self.pending_acks.lock().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        if let Some(entry) = pending.get_mut(&(producer_id, sequence)) {
            entry.status = AckStatus::Rejected;
        }

        Ok(())
    }

    /// Check for timed-out pending acks and return their IDs for
    /// redelivery.
    pub fn check_timeouts(&self) -> Result<Vec<(u64, u64)>> {
        let mut pending = self.pending_acks.lock().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let mut timed_out = Vec::new();
        let now = Instant::now();

        for (&(pid, seq), entry) in pending.iter_mut() {
            if entry.status == AckStatus::Pending && now.duration_since(entry.sent_at) > self.ack_timeout
            {
                entry.status = AckStatus::TimedOut;
                timed_out.push((pid, seq));
            }
        }

        // Remove timed-out entries.
        for &(pid, seq) in &timed_out {
            pending.remove(&(pid, seq));
        }

        if !timed_out.is_empty() {
            self.redelivered_count
                .fetch_add(timed_out.len() as u64, Ordering::Relaxed);
        }

        Ok(timed_out)
    }

    /// Get the number of pending (unacknowledged) messages.
    pub fn pending_count(&self) -> usize {
        self.pending_acks
            .lock()
            .map(|p| p.len())
            .unwrap_or(0)
    }

    /// Get the maximum allowed in-flight requests.
    pub fn max_in_flight(&self) -> u32 {
        self.max_in_flight
    }

    /// Check if idempotency is enabled.
    pub fn idempotency_enabled(&self) -> bool {
        self.idempotency_enabled
    }

    /// Get the ack timeout duration.
    pub fn ack_timeout(&self) -> Duration {
        self.ack_timeout
    }

    // -- Consumer Group Management --

    /// Create a new consumer group.
    pub fn create_consumer_group(
        &self,
        group_name: &str,
    ) -> Result<()> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        if groups.contains_key(group_name) {
            return Err(MqError::Consumer(format!(
                "Consumer group '{}' already exists",
                group_name
            )));
        }

        groups.insert(
            group_name.to_string(),
            ConsumerGroup {
                name: group_name.to_string(),
                members: HashMap::new(),
                generation: 0,
            },
        );

        info!(group = group_name, "Consumer group created");
        Ok(())
    }

    /// Delete a consumer group.
    pub fn delete_consumer_group(
        &self,
        group_name: &str,
    ) -> Result<()> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        groups
            .remove(group_name)
            .ok_or_else(|| MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            )))?;

        info!(group = group_name, "Consumer group deleted");
        Ok(())
    }

    /// Join a consumer group.
    pub fn join_consumer_group(
        &self,
        group_name: &str,
        consumer_id: &str,
    ) -> Result<u64> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            ))
        })?;

        group.generation += 1;
        let generation = group.generation;

        group.members.insert(
            consumer_id.to_string(),
            ConsumerAssignment {
                consumer_id: consumer_id.to_string(),
                partitions: Vec::new(),
                offsets: HashMap::new(),
            },
        );

        debug!(
            group = group_name,
            consumer = consumer_id,
            generation,
            "Consumer joined group"
        );

        Ok(generation)
    }

    /// Leave a consumer group.
    pub fn leave_consumer_group(
        &self,
        group_name: &str,
        consumer_id: &str,
    ) -> Result<()> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            ))
        })?;

        group.members.remove(consumer_id);
        group.generation += 1;

        debug!(
            group = group_name,
            consumer = consumer_id,
            "Consumer left group"
        );

        Ok(())
    }

    /// Assign partitions to consumers in a group using round-robin.
    pub fn assign_partitions(
        &self,
        group_name: &str,
        topic: &str,
        partitions: &[u32],
    ) -> Result<HashMap<String, Vec<u32>>> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            ))
        })?;

        if group.members.is_empty() {
            return Ok(HashMap::new());
        }

        // Round-robin assignment.
        let mut assignments: HashMap<String, Vec<u32>> = HashMap::new();
        let consumer_ids: Vec<String> =
            group.members.keys().cloned().collect();

        for (i, &partition) in partitions.iter().enumerate() {
            let consumer = &consumer_ids[i % consumer_ids.len()];
            assignments
                .entry(consumer.clone())
                .or_insert_with(Vec::new)
                .push(partition);

            // Update the member's assignment.
            if let Some(member) = group.members.get_mut(consumer) {
                member
                    .partitions
                    .push((topic.to_string(), partition));
            }
        }

        group.generation += 1;

        Ok(assignments)
    }

    /// Commit a consumer offset for a specific partition.
    pub fn commit_offset(
        &self,
        group_name: &str,
        consumer_id: &str,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> Result<()> {
        let mut groups = self.consumer_groups.write().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            ))
        })?;

        let member = group.members.get_mut(consumer_id).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer '{}' not in group '{}'",
                consumer_id, group_name
            ))
        })?;

        member.offsets.insert((topic.to_string(), partition), offset);

        debug!(
            group = group_name,
            consumer = consumer_id,
            topic,
            partition,
            offset,
            "Offset committed"
        );

        Ok(())
    }

    /// Get the committed offset for a consumer on a specific partition.
    pub fn get_committed_offset(
        &self,
        group_name: &str,
        consumer_id: &str,
        topic: &str,
        partition: u32,
    ) -> Result<Option<u64>> {
        let groups = self.consumer_groups.read().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        let group = groups.get(group_name).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            ))
        })?;

        let member = group.members.get(consumer_id).ok_or_else(|| {
            MqError::Consumer(format!(
                "Consumer '{}' not in group '{}'",
                consumer_id, group_name
            ))
        })?;

        Ok(member.offsets.get(&(topic.to_string(), partition)).copied())
    }

    /// List all consumer groups.
    pub fn list_consumer_groups(&self) -> Vec<String> {
        let groups = match self.consumer_groups.read() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        groups.keys().cloned().collect()
    }

    /// Get consumer group info.
    pub fn get_consumer_group(
        &self,
        group_name: &str,
    ) -> Result<ConsumerGroup> {
        let groups = self.consumer_groups.read().map_err(|e| {
            MqError::Internal(format!("Lock poisoned: {}", e))
        })?;

        groups
            .get(group_name)
            .cloned()
            .ok_or_else(|| MqError::Consumer(format!(
                "Consumer group '{}' not found",
                group_name
            )))
    }

    /// Get delivery statistics.
    pub fn stats(&self) -> DeliveryStats {
        DeliveryStats {
            delivered: self.delivered_count.load(Ordering::Relaxed),
            redelivered: self.redelivered_count.load(Ordering::Relaxed),
            deduplicated: self.deduplicated_count.load(Ordering::Relaxed),
            pending: self.pending_count() as u64,
        }
    }
}

/// Delivery statistics snapshot.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeliveryStats {
    pub delivered: u64,
    pub redelivered: u64,
    pub deduplicated: u64,
    pub pending: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> DeliveryEngine {
        DeliveryEngine::new(true, 5, 5000)
    }

    #[test]
    fn idempotent_first_message_accepted() {
        let engine = default_engine();
        assert!(engine.idempotent_check(1, 0).unwrap());
    }

    #[test]
    fn idempotent_exact_duplicate_rejected() {
        let engine = default_engine();
        assert!(engine.idempotent_check(1, 0).unwrap());
        assert!(!engine.idempotent_check(1, 0).unwrap());
    }

    #[test]
    fn idempotent_increasing_sequence_accepted() {
        let engine = default_engine();
        assert!(engine.idempotent_check(1, 0).unwrap());
        assert!(engine.idempotent_check(1, 1).unwrap());
        assert!(engine.idempotent_check(1, 2).unwrap());
    }

    #[test]
    fn idempotent_out_of_order_duplicate_rejected() {
        let engine = default_engine();
        assert!(engine.idempotent_check(1, 5).unwrap());
        assert!(!engine.idempotent_check(1, 3).unwrap());
    }

    #[test]
    fn idempotent_sequence_too_far_ahead() {
        let engine = DeliveryEngine::new(true, 5, 5000);
        assert!(engine.idempotent_check(1, 0).unwrap());
        // Sequence is 100 ahead, max_in_flight is 5 -- should error.
        let result = engine.idempotent_check(1, 100);
        assert!(result.is_err());
    }

    #[test]
    fn idempotent_disabled_always_accepts() {
        let engine = DeliveryEngine::new(false, 5, 5000);
        assert!(engine.idempotent_check(1, 0).unwrap());
        assert!(engine.idempotent_check(1, 0).unwrap());
        assert!(engine.idempotent_check(1, 999).unwrap());
    }

    #[test]
    fn ack_removes_pending() {
        let engine = default_engine();
        engine.record_pending_ack(1, 0).unwrap();
        assert_eq!(engine.pending_count(), 1);

        let acked = engine.ack(1, 0).unwrap();
        assert!(acked);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn ack_unknown_returns_false() {
        let engine = default_engine();
        let acked = engine.ack(999, 0).unwrap();
        assert!(!acked);
    }

    #[test]
    fn reject_marks_status() {
        let engine = default_engine();
        engine.record_pending_ack(1, 0).unwrap();
        engine.reject(1, 0).unwrap();

        let pending = engine.pending_acks.lock().unwrap();
        let entry = pending.get(&(1, 0)).unwrap();
        assert_eq!(entry.status, AckStatus::Rejected);
    }

    #[test]
    fn timeout_detection() {
        // Use a very short ack timeout.
        let engine = DeliveryEngine::new(true, 5, 1); // 1ms timeout
        engine.record_pending_ack(1, 0).unwrap();

        // Wait for timeout.
        std::thread::sleep(Duration::from_millis(10));

        let timed_out = engine.check_timeouts().unwrap();
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0], (1, 0));
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn create_and_delete_consumer_group() {
        let engine = default_engine();
        engine.create_consumer_group("g1").unwrap();

        let groups = engine.list_consumer_groups();
        assert_eq!(groups, vec!["g1".to_string()]);

        engine.delete_consumer_group("g1").unwrap();
        assert!(engine.list_consumer_groups().is_empty());
    }

    #[test]
    fn duplicate_consumer_group_fails() {
        let engine = default_engine();
        engine.create_consumer_group("g1").unwrap();
        let result = engine.create_consumer_group("g1");
        assert!(result.is_err());
    }

    #[test]
    fn join_and_leave_consumer_group() {
        let engine = default_engine();
        engine.create_consumer_group("g1").unwrap();

        let gen = engine.join_consumer_group("g1", "c1").unwrap();
        assert_eq!(gen, 1);

        let group = engine.get_consumer_group("g1").unwrap();
        assert_eq!(group.members.len(), 1);
        assert!(group.members.contains_key("c1"));

        engine.leave_consumer_group("g1", "c1").unwrap();
        let group = engine.get_consumer_group("g1").unwrap();
        assert!(group.members.is_empty());
    }

    #[test]
    fn partition_assignment_round_robin() {
        let engine = default_engine();
        engine.create_consumer_group("g1").unwrap();
        engine.join_consumer_group("g1", "c1").unwrap();
        engine.join_consumer_group("g1", "c2").unwrap();

        let assignments = engine
            .assign_partitions("g1", "topic", &[0, 1, 2, 3])
            .unwrap();

        // Each consumer should get exactly 2 partitions.
        let c1 = assignments.get("c1").unwrap();
        let c2 = assignments.get("c2").unwrap();
        assert_eq!(c1.len(), 2);
        assert_eq!(c2.len(), 2);

        // All partitions should be covered exactly once.
        let mut all: Vec<u32> = c1.iter().chain(c2.iter()).copied().collect();
        all.sort();
        assert_eq!(all, vec![0, 1, 2, 3]);
    }

    #[test]
    fn commit_and_get_offset() {
        let engine = default_engine();
        engine.create_consumer_group("g1").unwrap();
        engine.join_consumer_group("g1", "c1").unwrap();

        engine
            .commit_offset("g1", "c1", "topic", 0, 42)
            .unwrap();

        let offset = engine
            .get_committed_offset("g1", "c1", "topic", 0)
            .unwrap();
        assert_eq!(offset, Some(42));

        // Different partition returns None.
        let offset = engine
            .get_committed_offset("g1", "c1", "topic", 1)
            .unwrap();
        assert_eq!(offset, None);
    }

    #[test]
    fn delivery_stats() {
        let engine = default_engine();
        engine.record_pending_ack(1, 0).unwrap();
        engine.record_pending_ack(1, 1).unwrap();
        engine.ack(1, 0).unwrap();

        let stats = engine.stats();
        assert_eq!(stats.delivered, 1);
        assert_eq!(stats.pending, 1);
    }

    #[test]
    fn multiple_producers_independent() {
        let engine = default_engine();

        // Producer 1
        assert!(engine.idempotent_check(1, 0).unwrap());
        assert!(engine.idempotent_check(1, 1).unwrap());

        // Producer 2 with the same sequence numbers -- should be independent.
        assert!(engine.idempotent_check(2, 0).unwrap());
        assert!(engine.idempotent_check(2, 1).unwrap());

        // Producer 1 duplicate
        assert!(!engine.idempotent_check(1, 0).unwrap());

        // Producer 2 duplicate
        assert!(!engine.idempotent_check(2, 0).unwrap());
    }
}
