//! Idempotent message deduplication.
//!
//! Tracks producer sequences to detect and reject duplicate messages.
//! Each producer is assigned a monotonically increasing sequence number;
//! the broker records seen sequences and rejects any message whose sequence
//! has already been processed.

use std::collections::{HashMap, HashSet};

/// Tracks producer sequences to detect and reject duplicate messages.
///
/// The store maintains a bounded window of seen sequence numbers per producer.
/// When the window fills up, older entries are discarded (trading memory for
/// tolerance of large gaps in sequence numbers).
pub struct IdempotencyStore {
    /// Maps `producer_id` -> set of seen sequence numbers.
    seen: HashMap<String, HashSet<u64>>,
    /// Maximum sequences to retain per producer before pruning.
    max_sequences_per_producer: usize,
}

impl IdempotencyStore {
    /// Create a new store. `max_sequences_per_producer` controls the
    /// per-producer deduplication window size.
    pub fn new(max_sequences_per_producer: usize) -> Self {
        IdempotencyStore {
            seen: HashMap::new(),
            max_sequences_per_producer,
        }
    }

    /// Check if a message is a duplicate. Returns `true` if the sequence
    /// number has already been recorded for this producer.
    pub fn is_duplicate(&self, producer_id: &str, sequence: u64) -> bool {
        self.seen
            .get(producer_id)
            .map_or(false, |seqs| seqs.contains(&sequence))
    }

    /// Record a message as seen. Returns `true` if this is the first time
    /// the sequence has been seen; `false` if it is a duplicate.
    pub fn record(&mut self, producer_id: &str, sequence: u64) -> bool {
        let seqs = self
            .seen
            .entry(producer_id.to_string())
            .or_insert_with(HashSet::new);
        if seqs.contains(&sequence) {
            return false; // Already seen
        }
        seqs.insert(sequence);

        // Cleanup old sequences if the window is too large.
        if seqs.len() > self.max_sequences_per_producer {
            let min_seq = sequence.saturating_sub((self.max_sequences_per_producer - 1) as u64);
            seqs.retain(|&s| s >= min_seq);
        }
        true
    }

    /// Get the count of tracked producers.
    pub fn producer_count(&self) -> usize {
        self.seen.len()
    }

    /// Remove all tracking for a producer.
    pub fn remove_producer(&mut self, producer_id: &str) {
        self.seen.remove(producer_id);
    }

    /// Get the number of tracked sequences for a producer.
    pub fn sequence_count(&self, producer_id: &str) -> usize {
        self.seen.get(producer_id).map_or(0, |s| s.len())
    }
}

/// Idempotency key generator for request-level deduplication.
///
/// Produces unique string keys of the form `{producer_id}-{sequence}` that
/// can be used as idempotency keys in external systems (e.g. HTTP APIs).
pub struct IdempotencyKeyGenerator {
    producer_id: String,
    next_key: u64,
}

impl IdempotencyKeyGenerator {
    /// Create a new generator for the given producer.
    pub fn new(producer_id: String) -> Self {
        IdempotencyKeyGenerator {
            producer_id,
            next_key: 0,
        }
    }

    /// Generate the next unique idempotency key.
    pub fn next_key(&mut self) -> String {
        let key = format!("{}-{}", self.producer_id, self.next_key);
        self.next_key += 1;
        key
    }

    /// Return the number of keys generated so far.
    pub fn keys_generated(&self) -> u64 {
        self.next_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_message_is_not_duplicate() {
        let mut store = IdempotencyStore::new(1000);
        assert!(!store.is_duplicate("p1", 0));
        assert!(store.record("p1", 0));
        assert!(store.is_duplicate("p1", 0));
    }

    #[test]
    fn duplicate_detection() {
        let mut store = IdempotencyStore::new(1000);
        assert!(store.record("p1", 0));
        assert!(store.record("p1", 1));
        assert!(!store.record("p1", 0)); // duplicate
        assert!(!store.record("p1", 1)); // duplicate
        assert!(store.record("p1", 2)); // new
    }

    #[test]
    fn different_producers_are_independent() {
        let mut store = IdempotencyStore::new(1000);
        assert!(store.record("p1", 0));
        assert!(store.record("p2", 0)); // Different producer, same sequence
        assert!(store.is_duplicate("p1", 0));
        assert!(store.is_duplicate("p2", 0));
        assert_eq!(store.producer_count(), 2);
    }

    #[test]
    fn remove_producer() {
        let mut store = IdempotencyStore::new(1000);
        store.record("p1", 0);
        assert_eq!(store.producer_count(), 1);

        store.remove_producer("p1");
        assert_eq!(store.producer_count(), 0);
        assert!(!store.is_duplicate("p1", 0));
    }

    #[test]
    fn sequence_window_cleanup() {
        let mut store = IdempotencyStore::new(5);
        for i in 0..10 {
            store.record("p1", i);
        }
        // Only the last 5 sequences should be retained.
        assert_eq!(store.sequence_count("p1"), 5);
        // Old sequences should have been cleaned up.
        assert!(!store.is_duplicate("p1", 0));
        assert!(!store.is_duplicate("p1", 4));
        // Recent sequences should still be present.
        assert!(store.is_duplicate("p1", 9));
        assert!(store.is_duplicate("p1", 5));
    }

    #[test]
    fn key_generator_produces_unique_keys() {
        let mut gen = IdempotencyKeyGenerator::new("p1".into());
        assert_eq!(gen.next_key(), "p1-0");
        assert_eq!(gen.next_key(), "p1-1");
        assert_eq!(gen.next_key(), "p1-2");
        assert_eq!(gen.keys_generated(), 3);
    }

    #[test]
    fn key_generator_different_producers() {
        let mut gen1 = IdempotencyKeyGenerator::new("a".into());
        let mut gen2 = IdempotencyKeyGenerator::new("b".into());
        assert_eq!(gen1.next_key(), "a-0");
        assert_eq!(gen2.next_key(), "b-0");
        assert_eq!(gen1.next_key(), "a-1");
    }

    #[test]
    fn large_sequence_numbers() {
        let mut store = IdempotencyStore::new(100);
        assert!(store.record("p1", u64::MAX - 1));
        assert!(!store.record("p1", u64::MAX - 1));
        assert!(store.record("p1", u64::MAX));
    }
}
