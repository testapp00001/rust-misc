//! # Solution 01: Event Design
//!
//! Complete implementation of domain event types for the flash sale system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The specific payload for each type of flash sale event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventPayload {
    StockDecremented {
        account_id: String,
        quantity: u32,
        remaining: u32,
    },
    VoucherClaimed {
        account_id: String,
        voucher_code: String,
    },
    PurchaseCompleted {
        account_id: String,
        order_id: String,
        #[serde(alias = "amount_minor_units")]
        amount_cents: u64,
    },
    PurchaseFailed {
        account_id: String,
        reason: String,
    },
    SaleStarted {
        product_id: String,
        initial_stock: u32,
        price_cents: u64,
    },
    SaleEnded {
        product_id: String,
        final_stock: u32,
    },
}

/// A domain event in the flash sale system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashSaleEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub aggregate_id: String,
    pub correlation_id: Uuid,
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
    /// Create a new event with auto-generated id, timestamp, and correlation_id.
    pub fn new(aggregate_id: impl Into<String>, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            aggregate_id: aggregate_id.into(),
            correlation_id: Uuid::new_v4(),
            payload,
        }
    }

    /// Create a new event with a specific correlation_id.
    pub fn with_correlation(
        aggregate_id: impl Into<String>,
        correlation_id: Uuid,
        payload: EventPayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            aggregate_id: aggregate_id.into(),
            correlation_id,
            payload,
        }
    }

    /// Serialize this event to a JSON string.
    pub fn to_json(&self) -> Result<String, EventError> {
        serde_json::to_string(self).map_err(|e| EventError::SerializationFailed(e.to_string()))
    }

    /// Deserialize an event from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, EventError> {
        serde_json::from_str(json).map_err(|e| EventError::DeserializationFailed(e.to_string()))
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
