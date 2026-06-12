//! # Exercise: Effectively-Once Delivery
//!
//! ## Theory
//!
//! **Effectively-once** delivery combines two mechanisms:
//!
//! 1. **At-least-once delivery**: The network may deliver a message multiple times
//!    (due to retries, redeliveries, etc.). We accept this and ensure every message
//!    is delivered at least once.
//!
//! 2. **Idempotent processing**: The application ensures that processing the same
//!    message multiple times produces the same result as processing it once.
//!
//! Together, these give "effectively-once" semantics: the message is delivered
//! multiple times, but its effect is as if it were delivered exactly once.
//!
//! This is the standard pattern in distributed systems:
//! - **Kafka**: At-least-once delivery + idempotent consumers
//! - **RabbitMQ**: At-least-once delivery with manual acknowledgements
//! - **AWS SQS**: At-least-once delivery + deduplication window
//! - **Database replication**: WAL shipping (at-least-once) + idempotent replay
//!
//! ## Proof / Intuition
//!
//! Let P(m) be the effect of processing message m once.
//! Let P^n(m) be the effect of processing message m n times.
//!
//! For an idempotent function: P(m) = P^2(m) = ... = P^n(m)
//!
//! Therefore, regardless of how many times m is delivered:
//! P^n(m) = P(m)
//!
//! The effect is the same as if m were delivered exactly once.
//!
//! ## Implementation Task
//!
//! Implement:
//! - `IdempotencyKey` to uniquely identify operations
//! - `EffectivelyOnceProcessor<T>` that combines deduplication with processing
//! - An `IdempotencyStore` that caches results
//!
//! ## Verification
//!
//! - Same message processed twice returns the same result
//! - Unique messages are processed independently
//! - The idempotency store correctly tracks processed keys

use std::collections::HashMap;

/// A unique key for identifying an idempotent operation.
/// This is typically derived from the message ID + operation type.
pub type IdempotencyKey = String;

/// The result of processing a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult<T> {
    /// The message was processed for the first time.
    Processed(T),
    /// The message was a duplicate; returns the cached result.
    Cached(T),
}

impl<T> ProcessResult<T> {
    /// Unwrap the result, whether processed or cached.
    pub fn into_inner(self) -> T {
        match self {
            ProcessResult::Processed(v) => v,
            ProcessResult::Cached(v) => v,
        }
    }

    /// Returns true if this was a fresh processing (not a cache hit).
    pub fn was_processed(&self) -> bool {
        matches!(self, ProcessResult::Processed(_))
    }

    /// Returns true if this was a cache hit (duplicate).
    pub fn was_cached(&self) -> bool {
        matches!(self, ProcessResult::Cached(_))
    }
}

/// A store that caches the results of processed idempotency keys.
pub struct IdempotencyStore<T> {
    /// Map from idempotency key to the cached result.
    cache: HashMap<IdempotencyKey, T>,
}

impl<T: Clone> IdempotencyStore<T> {
    /// Create a new empty idempotency store.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Check if a key has already been processed.
    pub fn contains(&self, key: &IdempotencyKey) -> bool {
        self.cache.contains_key(key)
    }

    /// Get the cached result for a key.
    pub fn get(&self, key: &IdempotencyKey) -> Option<&T> {
        self.cache.get(key)
    }

    /// Store a result for a key.
    pub fn store(&mut self, key: IdempotencyKey, result: T) {
        self.cache.insert(key, result);
    }

    /// Return the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Return true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl<T: Clone> Default for IdempotencyStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A message with an idempotency key and a payload.
#[derive(Debug, Clone)]
pub struct IdempotentMessage<T> {
    /// Unique key for deduplication.
    pub idempotency_key: IdempotencyKey,
    /// The payload to process.
    pub payload: T,
}

/// An effectively-once processor that combines idempotency checking with processing.
///
/// Messages are processed by a user-supplied function. Results are cached
/// by idempotency key. Duplicate messages return the cached result.
pub struct EffectivelyOnceProcessor<T> {
    /// The idempotency store for caching results.
    store: IdempotencyStore<T>,
}

impl<T: Clone> EffectivelyOnceProcessor<T> {
    /// Create a new effectively-once processor.
    pub fn new() -> Self {
        Self {
            store: IdempotencyStore::new(),
        }
    }

    /// Process a message with idempotency guarantees.
    ///
    /// If the idempotency key has been seen before, returns the cached result.
    /// Otherwise, runs the processing function and caches the result.
    pub fn process<F>(&mut self, msg: IdempotentMessage<T>, processor: F) -> ProcessResult<T>
    where
        F: FnOnce(&T) -> T,
    {
        if let Some(cached) = self.store.get(&msg.idempotency_key) {
            return ProcessResult::Cached(cached.clone());
        }

        let result = processor(&msg.payload);
        self.store.store(msg.idempotency_key, result.clone());
        ProcessResult::Processed(result)
    }

    /// Check if a key has already been processed.
    pub fn is_processed(&self, key: &IdempotencyKey) -> bool {
        self.store.contains(key)
    }

    /// Get the cached result for a key.
    pub fn get_cached(&self, key: &IdempotencyKey) -> Option<&T> {
        self.store.get(key)
    }

    /// Return the number of unique messages processed.
    pub fn unique_count(&self) -> usize {
        self.store.len()
    }
}

impl<T: Clone> Default for EffectivelyOnceProcessor<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn same_message_twice_returns_same_result() {
        let mut processor = EffectivelyOnceProcessor::new();

        let msg = IdempotentMessage {
            idempotency_key: "op-001".to_string(),
            payload: 10,
        };

        let result1 = processor.process(msg.clone(), |x| x * 2);
        let result2 = processor.process(msg, |x| x * 2);

        assert!(result1.was_processed());
        assert!(result2.was_cached());
        assert_eq!(result1.into_inner(), 20);
        assert_eq!(result2.into_inner(), 20);
    }

    #[test]
    fn unique_messages_processed_independently() {
        let mut processor = EffectivelyOnceProcessor::new();

        let results: Vec<_> = (0..5)
            .map(|i| {
                let msg = IdempotentMessage {
                    idempotency_key: format!("op-{i:03}"),
                    payload: i,
                };
                processor.process(msg, |x| x * 10)
            })
            .collect();

        // All should be freshly processed
        assert!(results.iter().all(|r| r.was_processed()));
        assert_eq!(results[0].clone().into_inner(), 0);
        assert_eq!(results[1].clone().into_inner(), 10);
        assert_eq!(results[2].clone().into_inner(), 20);
        assert_eq!(results[3].clone().into_inner(), 30);
        assert_eq!(results[4].clone().into_inner(), 40);
        assert_eq!(processor.unique_count(), 5);
    }

    #[test]
    fn processor_counting_simulation() {
        let mut processor = EffectivelyOnceProcessor::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let msg = IdempotentMessage {
            idempotency_key: "payment-123".to_string(),
            payload: 100_u64,
        };

        let processor_fn = |amount: &u64| -> u64 {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            amount * 2 // e.g., double the payment (processing logic)
        };

        // Simulate at-least-once delivery: message arrives 3 times
        let r1 = processor.process(msg.clone(), processor_fn);
        let r2 = processor.process(msg.clone(), processor_fn);
        let r3 = processor.process(msg, processor_fn);

        // Processor function should only be called once
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert!(r1.was_processed());
        assert!(r2.was_cached());
        assert!(r3.was_cached());
    }

    #[test]
    fn idempotency_store_basics() {
        let mut store = IdempotencyStore::new();

        assert!(store.is_empty());
        assert!(!store.contains(&"key-1".to_string()));

        store.store("key-1".to_string(), 42);
        assert_eq!(store.len(), 1);
        assert!(store.contains(&"key-1".to_string()));
        assert_eq!(store.get(&"key-1".to_string()), Some(&42));
        assert!(!store.contains(&"key-2".to_string()));
    }

    #[test]
    fn mixed_unique_and_duplicate_messages() {
        let mut processor = EffectivelyOnceProcessor::new();
        let mut processed_count = 0;
        let mut cached_count = 0;

        // Mix of unique and duplicate messages
        let messages = vec![
            ("a", 1),
            ("b", 2),
            ("a", 1), // duplicate
            ("c", 3),
            ("b", 2), // duplicate
            ("a", 1), // duplicate
            ("d", 4),
        ];

        for (key, value) in messages {
            let msg = IdempotentMessage {
                idempotency_key: key.to_string(),
                payload: value,
            };
            let result = processor.process(msg, |x| x * 10);
            if result.was_processed() {
                processed_count += 1;
            } else {
                cached_count += 1;
            }
        }

        assert_eq!(processed_count, 4, "4 unique messages should be processed");
        assert_eq!(cached_count, 3, "3 duplicates should be cached");
    }
}
