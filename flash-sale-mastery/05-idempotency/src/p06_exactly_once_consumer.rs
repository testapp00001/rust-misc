//! # Exercise 06: Exactly-Once Message Consumer
//!
//! ## Learning Objective
//! Implement effectively-once message processing by combining message
//! deduplication with idempotent processing. Even if a message broker
//! delivers the same message twice (at-least-once delivery), the consumer
//! processes it exactly once.
//!
//! ## Flash Sale Context
//! After a purchase is authorized, a message is sent to a queue for voucher
//! generation. The queue guarantees at-least-once delivery, meaning the same
//! "generate voucher" message might arrive twice. The consumer must:
//! 1. Check if this message ID was already processed (deduplication)
//! 2. If not, process it (generate the voucher)
//! 3. Store the result keyed by message ID
//! 4. If the same message arrives again, return the cached result
//!
//! ## Instructions
//! 1. Implement `IdempotentConsumer::new()` with a deduplication store
//! 2. Implement `process()` that combines dedup + idempotent handler
//! 3. Implement `ProcessResult` enum: Processed(voucher) | Skipped(cached_voucher)
//! 4. Handle the case where processing is in-flight (another consumer has it)
//!
//! ## Hints
//! - Reuse concepts from p04 (ConcurrentDeduplicator) for the dedup check
//! - The "voucher generation" can be simulated with a deterministic function
//! - Use a trait for the actual processing logic so it's testable

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Result of processing a message.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessResult<T> {
    /// The message was processed for the first time.
    Processed(T),
    /// The message was a duplicate; the cached result is returned.
    Skipped(T),
}

/// A generated voucher.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Voucher {
    pub code: String,
    pub account_id: String,
    pub product_id: String,
}

/// Trait for the actual processing logic.
///
/// Implement this trait to define what "processing a message" means.
/// The consumer handles deduplication; this trait handles the business logic.
pub trait VoucherGenerator: Send + Sync {
    /// Generate a voucher for the given account and product.
    fn generate(&self, account_id: &str, product_id: &str) -> Voucher;
}

/// A deterministic voucher generator for testing.
pub struct DeterministicGenerator;

impl VoucherGenerator for DeterministicGenerator {
    fn generate(&self, account_id: &str, product_id: &str) -> Voucher {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{account_id}:{product_id}"));
        let hash = hex::encode(hasher.finalize());
        Voucher {
            code: format!("VCH-{}", &hash[..8].to_uppercase()),
            account_id: account_id.to_string(),
            product_id: product_id.to_string(),
        }
    }
}

/// A message from the queue requesting voucher generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherMessage {
    /// Unique message ID from the broker (used for deduplication).
    pub message_id: String,
    /// The account to generate a voucher for.
    pub account_id: String,
    /// The product the voucher is for.
    pub product_id: String,
}

/// In-memory deduplication store for message processing.
///
/// Tracks which message IDs have been processed and their results.
pub struct DedupStore {
    processed: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl DedupStore {
    pub fn new() -> Self {
        Self {
            processed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a message has already been processed.
    ///
    /// Returns `Some(cached_result)` if already processed, `None` if new.
    pub fn check(&self, message_id: &str) -> Option<Voucher> {
        // TODO: Lock the mutex and check if message_id exists
        // TODO: If it exists, deserialize the cached result and return Some
        // TODO: If not, return None
        todo!("Implement dedup check")
    }

    /// Mark a message as processed with its result.
    pub fn mark_processed(&self, message_id: &str, voucher: &Voucher) {
        // TODO: Lock the mutex and insert the serialized voucher
        todo!("Implement mark_processed")
    }
}

/// An idempotent message consumer that ensures each message is processed
/// exactly once, even if delivered multiple times.
pub struct IdempotentConsumer<G: VoucherGenerator> {
    store: DedupStore,
    generator: G,
}

impl<G: VoucherGenerator> IdempotentConsumer<G> {
    /// Create a new idempotent consumer.
    pub fn new(generator: G) -> Self {
        Self {
            store: DedupStore::new(),
            generator,
        }
    }

    /// Process a voucher generation message.
    ///
    /// If the message ID has been seen before, returns the cached voucher.
    /// If not, generates a new voucher and caches it.
    ///
    /// # Arguments
    /// * `message` - The voucher generation message from the queue
    pub fn process(&self, message: &VoucherMessage) -> ProcessResult<Voucher> {
        // TODO: Check the dedup store for this message_id
        // TODO: If already processed -> return Skipped(cached_voucher)
        // TODO: If new -> generate voucher, store it, return Processed(voucher)
        todo!("Implement idempotent message processing")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_message(suffix: &str) -> VoucherMessage {
        VoucherMessage {
            message_id: format!("msg-{}-{}", std::process::id(), suffix),
            account_id: "user-001".to_string(),
            product_id: "prod-001".to_string(),
        }
    }

    /// Test that a first-time message is processed.
    #[tokio::test]
    async fn test_first_message_processed() {
        let consumer = IdempotentConsumer::new(DeterministicGenerator);
        let msg = test_message("first");

        let result = consumer.process(&msg);
        match result {
            ProcessResult::Processed(voucher) => {
                assert_eq!(voucher.account_id, "user-001");
                assert_eq!(voucher.product_id, "prod-001");
                assert!(voucher.code.starts_with("VCH-"));
            }
            other => panic!("Expected Processed, got: {other:?}"),
        }
    }

    /// Test that a duplicate message is skipped with cached result.
    #[tokio::test]
    async fn test_duplicate_message_skipped() {
        let consumer = IdempotentConsumer::new(DeterministicGenerator);
        let msg = test_message("duplicate");

        let r1 = consumer.process(&msg);
        let r2 = consumer.process(&msg);

        let voucher1 = match r1 {
            ProcessResult::Processed(v) => v,
            other => panic!("Expected Processed, got: {other:?}"),
        };

        match r2 {
            ProcessResult::Skipped(voucher2) => {
                assert_eq!(voucher1, voucher2, "Cached result should match original");
            }
            other => panic!("Expected Skipped, got: {other:?}"),
        }
    }

    /// Test that different messages produce different vouchers.
    #[tokio::test]
    async fn test_different_messages_independent() {
        let consumer = IdempotentConsumer::new(DeterministicGenerator);
        let msg1 = test_message("independent_1");
        let msg2 = test_message("independent_2");

        let r1 = consumer.process(&msg1);
        let r2 = consumer.process(&msg2);

        let v1 = match r1 {
            ProcessResult::Processed(v) => v,
            other => panic!("Expected Processed for msg1, got: {other:?}"),
        };
        let v2 = match r2 {
            ProcessResult::Processed(v) => v,
            other => panic!("Expected Processed for msg2, got: {other:?}"),
        };

        assert_ne!(v1.code, v2.code, "Different messages should produce different vouchers");
    }

    /// Test that the same message always produces the same voucher (deterministic).
    #[tokio::test]
    async fn test_deterministic_voucher_generation() {
        let consumer = IdempotentConsumer::new(DeterministicGenerator);
        let msg = test_message("deterministic");

        let voucher_a = match consumer.process(&msg) {
            ProcessResult::Processed(v) => v,
            other => panic!("Expected Processed, got: {other:?}"),
        };

        // Process again -- should get the same voucher (via dedup cache)
        let voucher_b = match consumer.process(&msg) {
            ProcessResult::Skipped(v) => v,
            other => panic!("Expected Skipped, got: {other:?}"),
        };

        assert_eq!(voucher_a.code, voucher_b.code);
    }

    /// Test concurrent processing of the same message.
    #[tokio::test]
    async fn test_concurrent_same_message() {
        let consumer = Arc::new(IdempotentConsumer::new(DeterministicGenerator));
        let msg = test_message("concurrent");

        let mut handles = Vec::new();
        for _ in 0..20 {
            let c = consumer.clone();
            let m = msg.clone();
            handles.push(tokio::spawn(async move { c.process(&m) }));
        }

        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.expect("task panicked"));
        }

        let processed_count = results
            .iter()
            .filter(|r| matches!(r, ProcessResult::Processed(_)))
            .count();
        let skipped_count = results
            .iter()
            .filter(|r| matches!(r, ProcessResult::Skipped(_)))
            .count();

        // The first gets Processed, the rest get Skipped
        assert_eq!(processed_count, 1, "Exactly one should be Processed");
        assert_eq!(skipped_count, 19, "The rest should be Skipped");

        // All vouchers should be identical
        let first_voucher = results
            .iter()
            .find_map(|r| match r {
                ProcessResult::Processed(v) | ProcessResult::Skipped(v) => Some(v),
            })
            .unwrap();
        for r in &results {
            let v = match r {
                ProcessResult::Processed(v) | ProcessResult::Skipped(v) => v,
            };
            assert_eq!(v.code, first_voucher.code);
        }
    }
}
