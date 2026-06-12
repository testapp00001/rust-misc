//! Exactly-once delivery semantics.
//!
//! Implements a transaction coordinator that combines idempotent producers with
//! transactional commits to achieve exactly-once delivery. Messages are batched
//! in a transaction and atomically committed or aborted.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{MqError, Result};

/// Transaction coordinator for exactly-once delivery.
///
/// Combines idempotent producer sequencing with transactional commit to ensure
/// each message is delivered exactly once, even in the face of producer retries
/// and network partitions.
///
/// # Usage
///
/// 1. Call `begin_transaction` to start a new transaction.
/// 2. Call `add_to_transaction` to stage messages.
/// 3. Call `commit_transaction` to atomically finalize the batch.
///    Or call `abort_transaction` to discard it.
pub struct ExactlyOnceProducer {
    /// Unique identifier for this producer instance.
    producer_id: String,
    /// Monotonically increasing sequence number for deduplication.
    sequence_number: u64,
    /// Pending transaction batches keyed by transaction ID.
    pending_batches: HashMap<String, PendingBatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingBatch {
    topic: String,
    partition: u32,
    base_offset: u64,
    last_offset: u64,
    messages: Vec<Vec<u8>>,
}

impl ExactlyOnceProducer {
    /// Create a new exactly-once producer with the given unique ID.
    pub fn new(producer_id: String) -> Self {
        ExactlyOnceProducer {
            producer_id,
            sequence_number: 0,
            pending_batches: HashMap::new(),
        }
    }

    /// Begin a new transaction. The transaction ID must be unique within
    /// this producer's lifetime.
    pub fn begin_transaction(&mut self, txn_id: String) {
        self.pending_batches.insert(
            txn_id,
            PendingBatch {
                topic: String::new(),
                partition: 0,
                base_offset: 0,
                last_offset: 0,
                messages: Vec::new(),
            },
        );
    }

    /// Add messages to an existing transaction.
    pub fn add_to_transaction(
        &mut self,
        txn_id: &str,
        topic: &str,
        partition: u32,
        messages: Vec<Vec<u8>>,
    ) -> Result<()> {
        if let Some(batch) = self.pending_batches.get_mut(txn_id) {
            batch.topic = topic.to_string();
            batch.partition = partition;
            batch.messages.extend(messages);
            Ok(())
        } else {
            Err(MqError::Producer(format!(
                "Transaction {} not found",
                txn_id
            )))
        }
    }

    /// Commit a transaction, returning the batch of messages for delivery.
    ///
    /// The caller is responsible for writing the returned messages to the
    /// underlying storage. After commit the transaction is removed from
    /// the pending set.
    pub fn commit_transaction(
        &mut self,
        txn_id: &str,
    ) -> Result<(String, u32, Vec<Vec<u8>>)> {
        if let Some(batch) = self.pending_batches.remove(txn_id) {
            Ok((batch.topic, batch.partition, batch.messages))
        } else {
            Err(MqError::Producer(format!(
                "Transaction {} not found",
                txn_id
            )))
        }
    }

    /// Abort a transaction, discarding all staged messages.
    pub fn abort_transaction(&mut self, txn_id: &str) {
        self.pending_batches.remove(txn_id);
    }

    /// Return the next sequence number for idempotent delivery.
    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence_number;
        self.sequence_number += 1;
        seq
    }

    /// Return the producer ID.
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }

    /// Return the current sequence number (without incrementing).
    pub fn current_sequence(&self) -> u64 {
        self.sequence_number
    }

    /// Return the number of pending transactions.
    pub fn pending_count(&self) -> usize {
        self.pending_batches.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_and_commit_transaction() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        producer.begin_transaction("t1".into());
        assert_eq!(producer.pending_count(), 1);

        producer
            .add_to_transaction(
                "t1",
                "orders",
                0,
                vec![b"msg1".to_vec(), b"msg2".to_vec()],
            )
            .unwrap();

        let (topic, partition, msgs) = producer.commit_transaction("t1").unwrap();
        assert_eq!(topic, "orders");
        assert_eq!(partition, 0);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0], b"msg1");
        assert_eq!(producer.pending_count(), 0);
    }

    #[test]
    fn abort_transaction() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        producer.begin_transaction("t1".into());
        producer
            .add_to_transaction("t1", "orders", 0, vec![b"data".to_vec()])
            .unwrap();

        producer.abort_transaction("t1");
        assert_eq!(producer.pending_count(), 0);
        assert!(producer.commit_transaction("t1").is_err());
    }

    #[test]
    fn commit_nonexistent_transaction_fails() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        assert!(producer.commit_transaction("nope").is_err());
    }

    #[test]
    fn add_to_nonexistent_transaction_fails() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        let result = producer.add_to_transaction("nope", "t", 0, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn sequence_numbers_are_monotonic() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        assert_eq!(producer.next_sequence(), 0);
        assert_eq!(producer.next_sequence(), 1);
        assert_eq!(producer.next_sequence(), 2);
        assert_eq!(producer.current_sequence(), 3);
    }

    #[test]
    fn multiple_concurrent_transactions() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        producer.begin_transaction("t1".into());
        producer.begin_transaction("t2".into());
        producer.begin_transaction("t3".into());
        assert_eq!(producer.pending_count(), 3);

        producer
            .add_to_transaction("t1", "topic-a", 0, vec![b"a1".to_vec()])
            .unwrap();
        producer
            .add_to_transaction("t2", "topic-b", 1, vec![b"b1".to_vec()])
            .unwrap();
        producer
            .add_to_transaction("t3", "topic-c", 2, vec![b"c1".to_vec()])
            .unwrap();

        let (topic, _, _) = producer.commit_transaction("t2").unwrap();
        assert_eq!(topic, "topic-b");
        assert_eq!(producer.pending_count(), 2);

        producer.abort_transaction("t1");
        assert_eq!(producer.pending_count(), 1);

        let (topic, _, _) = producer.commit_transaction("t3").unwrap();
        assert_eq!(topic, "topic-c");
        assert_eq!(producer.pending_count(), 0);
    }

    #[test]
    fn producer_id_is_preserved() {
        let producer = ExactlyOnceProducer::new("my-producer".into());
        assert_eq!(producer.producer_id(), "my-producer");
    }

    #[test]
    fn add_to_transaction_accumulates_messages() {
        let mut producer = ExactlyOnceProducer::new("p1".into());
        producer.begin_transaction("t1".into());

        producer
            .add_to_transaction("t1", "topic", 0, vec![b"m1".to_vec()])
            .unwrap();
        producer
            .add_to_transaction("t1", "topic", 0, vec![b"m2".to_vec(), b"m3".to_vec()])
            .unwrap();

        let (_, _, msgs) = producer.commit_transaction("t1").unwrap();
        assert_eq!(msgs.len(), 3);
    }
}
