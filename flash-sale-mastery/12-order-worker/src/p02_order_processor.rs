//! # Exercise 02: Order Processor
//!
//! ## Learning Objective
//! Learn to implement idempotent event processing -- converting raw stream
//! events into structured order records while handling duplicates gracefully.
//!
//! ## Flash Sale Context
//! Redis Streams guarantee at-least-once delivery. If a consumer crashes after
//! processing but before acknowledging, the event will be re-delivered. The
//! order processor must detect duplicates and return the existing order instead
//! of creating a new one. This is the same idempotency pattern from Module 05,
//! applied to the consumer side.
//!
//! ## Instructions
//! 1. Implement `OrderProcessor::new` to initialize the processor
//! 2. Implement `process` to convert an `OrderEvent` into an `Order`
//! 3. Track processed order IDs to detect duplicates (use a `HashSet`)
//! 4. Validate that required fields are present and non-empty
//!
//! ## Hints
//! - Use `std::collections::HashSet<String>` to track seen order IDs
//! - Use `uuid::Uuid` for generating order IDs if the event doesn't have one
//! - Return a specific error for invalid events (empty product_id, etc.)
//! - For duplicates, return the same order that was created the first time

use std::collections::HashSet;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::p01_consumer::OrderEvent;

/// Order status in its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Just created, awaiting payment.
    Pending,
    /// Payment completed successfully.
    Paid,
    /// Payment failed.
    PaymentFailed,
    /// Order was cancelled.
    Cancelled,
}

/// A processed order record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier.
    pub id: Uuid,
    /// The product being purchased.
    pub product_id: String,
    /// The account that made the purchase.
    pub account_id: String,
    /// Assigned voucher code.
    pub voucher_code: String,
    /// Current order status.
    pub status: OrderStatus,
    /// When the order was created.
    pub created_at: DateTime<Utc>,
}

/// Custom error type for order processing.
#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Duplicate order: {order_id} already processed")]
    DuplicateOrder { order_id: String },
}

/// Processes raw order events into structured Order records.
pub struct OrderProcessor {
    /// Track which order IDs have been processed (for duplicate detection).
    processed: Mutex<HashSet<String>>,
}

impl OrderProcessor {
    /// Create a new order processor.
    pub fn new() -> Self {
        todo!("Initialize the OrderProcessor with an empty HashSet")
    }

    /// Process an order event into an Order record.
    ///
    /// - If the event has already been processed (duplicate), return the
    ///   `ProcessorError::DuplicateOrder` error with the original order ID.
    /// - If the event is invalid (empty product_id or account_id), return
    ///   `ProcessorError::InvalidEvent`.
    /// - Otherwise, create a new `Order` with status `Pending`.
    ///
    /// # Arguments
    /// * `event` - The raw order event from the Redis Stream
    ///
    /// # Returns
    /// The processed `Order` record.
    pub fn process(&self, event: OrderEvent) -> Result<Order, ProcessorError> {
        // TODO: Validate that product_id and account_id are non-empty
        // TODO: Check if order_id is already in the processed set
        // TODO: Add order_id to the processed set
        // TODO: Create and return the Order
        todo!("Implement order processing with duplicate detection")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test successful order processing.
    #[test]
    fn test_process_valid_event() {
        let processor = OrderProcessor::new();
        let event = OrderEvent {
            stream_id: "1234567890-0".to_string(),
            order_id: "ord-valid-001".to_string(),
            product_id: "prod-A".to_string(),
            account_id: "acc-100".to_string(),
            voucher_code: "VC-001".to_string(),
        };

        let order = processor.process(event).expect("Should process valid event");
        assert_eq!(order.product_id, "prod-A");
        assert_eq!(order.account_id, "acc-100");
        assert_eq!(order.voucher_code, "VC-001");
        assert_eq!(order.status, OrderStatus::Pending);
    }

    /// Test that duplicate events are rejected.
    #[test]
    fn test_duplicate_order_rejected() {
        let processor = OrderProcessor::new();
        let event = OrderEvent {
            stream_id: "1234567890-1".to_string(),
            order_id: "ord-dup-001".to_string(),
            product_id: "prod-B".to_string(),
            account_id: "acc-200".to_string(),
            voucher_code: "VC-002".to_string(),
        };

        // First processing should succeed
        processor
            .process(event.clone())
            .expect("First processing should succeed");

        // Second processing should fail with DuplicateOrder
        let result = processor.process(event);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessorError::DuplicateOrder { order_id } => {
                assert_eq!(order_id, "ord-dup-001");
            }
            other => panic!("Expected DuplicateOrder, got: {other}"),
        }
    }

    /// Test that invalid events (empty fields) are rejected.
    #[test]
    fn test_invalid_event_rejected() {
        let processor = OrderProcessor::new();

        // Empty product_id
        let event = OrderEvent {
            stream_id: "1234567890-2".to_string(),
            order_id: "ord-invalid-001".to_string(),
            product_id: "".to_string(),
            account_id: "acc-300".to_string(),
            voucher_code: "VC-003".to_string(),
        };
        let result = processor.process(event);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessorError::InvalidEvent(_) => {} // expected
            other => panic!("Expected InvalidEvent, got: {other}"),
        }

        // Empty account_id
        let event = OrderEvent {
            stream_id: "1234567890-3".to_string(),
            order_id: "ord-invalid-002".to_string(),
            product_id: "prod-C".to_string(),
            account_id: "".to_string(),
            voucher_code: "VC-004".to_string(),
        };
        let result = processor.process(event);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProcessorError::InvalidEvent(_) => {} // expected
            other => panic!("Expected InvalidEvent, got: {other}"),
        }
    }

    /// Test that multiple different orders can be processed.
    #[test]
    fn test_multiple_orders() {
        let processor = OrderProcessor::new();

        for i in 0..5 {
            let event = OrderEvent {
                stream_id: format!("1234567890-{i}"),
                order_id: format!("ord-multi-{i:03}"),
                product_id: "prod-D".to_string(),
                account_id: format!("acc-{i}"),
                voucher_code: format!("VC-{i:03}"),
            };
            let order = processor.process(event).expect("Should process");
            assert_eq!(order.status, OrderStatus::Pending);
        }
    }
}
