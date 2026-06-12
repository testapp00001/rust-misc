//! # Exercise 01: Event Design
//!
//! ## Learning Objective
//! Design domain event types for a flash sale system. Events are the foundation
//! of event sourcing -- they represent facts that happened in the system. Every
//! event must be self-describing, immutable, and serializable for storage.
//!
//! ## Flash Sale Context
//! During a flash sale, many things happen: stock decreases, vouchers are claimed,
//! purchases succeed or fail, and sales start and end. Each of these is an event
//! that must be recorded for audit trails and state reconstruction.
//!
//! ## Instructions
//! 1. Implement the `EventPayload` enum with all six variants
//! 2. Implement the `FlashSaleEvent` struct with id, timestamp, aggregate_id,
//!    correlation_id, and payload fields
//! 3. Add `Serialize`/`Deserialize` derives so events can be stored as JSON
//! 4. Implement `FlashSaleEvent::new` as a constructor that auto-generates id and timestamp
//! 5. Implement `FlashSaleEvent::with_correlation` to set a specific correlation_id
//!
//! ## Hints
//! - Use `Uuid::new_v4()` for generating unique IDs
//! - Use `Utc::now()` for timestamps
//! - The aggregate_id is the product being acted upon (e.g., "product:1001")
//! - Correlation IDs link related events (e.g., a purchase flow that triggers
//!   stock decrement + purchase completion)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The specific payload for each type of flash sale event.
///
/// Each variant carries the data specific to that event type.
/// Common fields (id, timestamp, aggregate_id, correlation_id) live
/// on the parent `FlashSaleEvent` struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventPayload {
    /// Stock was decremented for a product.
    StockDecremented {
        account_id: String,
        quantity: u32,
        remaining: u32,
    },
    /// A discount voucher was claimed by an account.
    VoucherClaimed {
        account_id: String,
        voucher_code: String,
    },
    /// A purchase was completed successfully.
    PurchaseCompleted {
        account_id: String,
        order_id: String,
        #[serde(alias = "amount_minor_units")]
        amount_cents: u64,
    },
    /// A purchase attempt failed.
    PurchaseFailed {
        account_id: String,
        reason: String,
    },
    /// A flash sale has started for a product.
    SaleStarted {
        product_id: String,
        initial_stock: u32,
        price_cents: u64,
    },
    /// A flash sale has ended for a product.
    SaleEnded {
        product_id: String,
        final_stock: u32,
    },
}

/// A domain event in the flash sale system.
///
/// Events are immutable facts. Once recorded, they are never modified or deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashSaleEvent {
    /// Unique identifier for this event.
    pub id: Uuid,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// The aggregate this event belongs to (typically a product_id).
    pub aggregate_id: String,
    /// Groups related events from the same operation/flow.
    pub correlation_id: Uuid,
    /// The specific event data.
    pub payload: EventPayload,
}

/// Error type for event operations.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    SerializationFailed(String),

    #[error("Deserialization error: {0}")]
    DeserializationFailed(String),
}

impl FlashSaleEvent {
    /// Create a new event with an auto-generated id, timestamp, and correlation_id.
    ///
    /// # Arguments
    /// * `aggregate_id` - The product/entity this event relates to
    /// * `payload` - The specific event data
    pub fn new(aggregate_id: impl Into<String>, payload: EventPayload) -> Self {
        // TODO: Create a new FlashSaleEvent with Uuid::new_v4(), Utc::now(),
        //       and a fresh correlation_id
        todo!("Implement FlashSaleEvent::new")
    }

    /// Create a new event with a specific correlation_id (for linking related events).
    ///
    /// # Arguments
    /// * `aggregate_id` - The product/entity this event relates to
    /// * `correlation_id` - The correlation ID to use
    /// * `payload` - The specific event data
    pub fn with_correlation(
        aggregate_id: impl Into<String>,
        correlation_id: Uuid,
        payload: EventPayload,
    ) -> Self {
        // TODO: Same as new() but use the provided correlation_id
        todo!("Implement FlashSaleEvent::with_correlation")
    }

    /// Serialize this event to a JSON string.
    pub fn to_json(&self) -> Result<String, EventError> {
        // TODO: Use serde_json::to_string and map errors
        todo!("Implement to_json")
    }

    /// Deserialize an event from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, EventError> {
        // TODO: Use serde_json::from_str and map errors
        todo!("Implement from_json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::StockDecremented {
                account_id: "user:42".to_string(),
                quantity: 1,
                remaining: 99,
            },
        );
        assert_eq!(event.aggregate_id, "product:1001");
        assert!(!event.id.is_nil());
        assert!(!event.correlation_id.is_nil());
        assert!(event.timestamp <= Utc::now());
    }

    #[test]
    fn test_event_with_correlation() {
        let cid = Uuid::new_v4();
        let event = FlashSaleEvent::with_correlation(
            "product:1001",
            cid,
            EventPayload::PurchaseCompleted {
                account_id: "user:42".to_string(),
                order_id: "order:999".to_string(),
                amount_cents: 1999,
            },
        );
        assert_eq!(event.correlation_id, cid);
        assert_eq!(event.aggregate_id, "product:1001");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::VoucherClaimed {
                account_id: "user:42".to_string(),
                voucher_code: "FLASH50".to_string(),
            },
        );
        let json = event.to_json().expect("serialization should succeed");
        let restored = FlashSaleEvent::from_json(&json).expect("deserialization should succeed");
        assert_eq!(restored.id, event.id);
        assert_eq!(restored.aggregate_id, event.aggregate_id);
        assert_eq!(restored.correlation_id, event.correlation_id);
        assert_eq!(restored.payload, event.payload);
    }

    #[test]
    fn test_event_ordering_by_timestamp() {
        let mut events = Vec::new();
        for _ in 0..5 {
            events.push(FlashSaleEvent::new(
                "product:1001",
                EventPayload::StockDecremented {
                    account_id: "user:1".to_string(),
                    quantity: 1,
                    remaining: 100,
                },
            ));
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        events.sort_by_key(|e| e.timestamp);
        for window in events.windows(2) {
            assert!(window[0].timestamp <= window[1].timestamp);
        }
    }

    #[test]
    fn test_all_payload_variants_serialize() {
        let variants = vec![
            EventPayload::StockDecremented {
                account_id: "u1".into(),
                quantity: 1,
                remaining: 9,
            },
            EventPayload::VoucherClaimed {
                account_id: "u1".into(),
                voucher_code: "V1".into(),
            },
            EventPayload::PurchaseCompleted {
                account_id: "u1".into(),
                order_id: "o1".into(),
                amount_cents: 100,
            },
            EventPayload::PurchaseFailed {
                account_id: "u1".into(),
                reason: "out of stock".into(),
            },
            EventPayload::SaleStarted {
                product_id: "p1".into(),
                initial_stock: 100,
                price_cents: 999,
            },
            EventPayload::SaleEnded {
                product_id: "p1".into(),
                final_stock: 0,
            },
        ];
        for payload in variants {
            let event = FlashSaleEvent::new("p1", payload.clone());
            let json = event.to_json().expect("should serialize");
            let restored = FlashSaleEvent::from_json(&json).expect("should deserialize");
            assert_eq!(restored.payload, payload);
        }
    }
}
