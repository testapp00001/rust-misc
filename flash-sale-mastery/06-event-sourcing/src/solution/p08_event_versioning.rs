//! # Solution 08: Event Versioning
//!
//! Complete implementation of event schema versioning with backward compatibility.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::p01_event_design::{EventError, FlashSaleEvent};

/// A versioned event envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedEvent {
    pub version: u32,
    pub payload: Value,
}

/// The latest schema version number.
pub const LATEST_VERSION: u32 = 2;

impl VersionedEvent {
    /// Wrap a FlashSaleEvent in a v1 envelope.
    pub fn wrap_v1(event: &FlashSaleEvent) -> Result<Self, EventError> {
        let payload = serde_json::to_value(event)
            .map_err(|e| EventError::SerializationFailed(e.to_string()))?;
        Ok(Self {
            version: 1,
            payload,
        })
    }

    /// Wrap a FlashSaleEvent in a v2 envelope.
    pub fn wrap_v2(event: &FlashSaleEvent) -> Result<Self, EventError> {
        let payload = serde_json::to_value(event)
            .map_err(|e| EventError::SerializationFailed(e.to_string()))?;
        Ok(Self {
            version: 2,
            payload,
        })
    }

    /// Unwrap this envelope, migrating to the latest schema version.
    pub fn unwrap_latest(&self) -> Result<FlashSaleEvent, EventError> {
        let migrated = migrate(&self.payload, self.version)?;
        serde_json::from_value(migrated)
            .map_err(|e| EventError::DeserializationFailed(e.to_string()))
    }
}

/// Migrate a v1 event payload to v2 schema.
///
/// V2 changes for PurchaseCompleted:
/// - `amount_cents` is renamed to `amount_minor_units`
/// - A `currency` field is added (defaults to "USD")
pub fn migrate_v1_to_v2(v1_payload: &Value) -> Result<Value, EventError> {
    let mut payload = v1_payload.clone();

    // The VersionedEvent wraps the full FlashSaleEvent, so the structure is:
    // { "id": ..., "payload": { "PurchaseCompleted": { ... } } }
    // We need to navigate to payload.payload.PurchaseCompleted
    if let Some(outer) = payload.as_object_mut() {
        if let Some(inner_payload) = outer.get_mut("payload") {
            if let Some(inner_obj) = inner_payload.as_object_mut() {
                if let Some(purchase) = inner_obj.get_mut("PurchaseCompleted") {
                    if let Some(purchase_obj) = purchase.as_object_mut() {
                        // Rename amount_cents -> amount_minor_units
                        if let Some(amount) = purchase_obj.remove("amount_cents") {
                            purchase_obj.insert("amount_minor_units".to_string(), amount);
                        }
                        // Add currency field if not present
                        if !purchase_obj.contains_key("currency") {
                            purchase_obj
                                .insert("currency".to_string(), Value::String("USD".to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(payload)
}

/// Apply all migrations from a given version to the latest.
pub fn migrate(payload: &Value, from_version: u32) -> Result<Value, EventError> {
    let mut current = payload.clone();
    if from_version < 2 {
        current = migrate_v1_to_v2(&current)?;
    }
    // Future migrations:
    // if from_version < 3 { current = migrate_v2_to_v3(&current)?; }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_event_design::EventPayload;

    #[test]
    fn test_wrap_v1_roundtrip() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::StockDecremented {
                account_id: "user:42".to_string(),
                quantity: 1,
                remaining: 99,
            },
        );

        let versioned = VersionedEvent::wrap_v1(&event).expect("wrap should succeed");
        assert_eq!(versioned.version, 1);

        let restored = versioned.unwrap_latest().expect("unwrap should succeed");
        assert_eq!(restored.id, event.id);
        assert_eq!(restored.payload, event.payload);
    }

    #[test]
    fn test_old_events_still_parse() {
        let old_event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::PurchaseCompleted {
                account_id: "user:42".to_string(),
                order_id: "order:1".to_string(),
                amount_cents: 1999,
            },
        );

        let v1 = VersionedEvent::wrap_v1(&old_event).expect("wrap v1");
        let json = serde_json::to_string(&v1).expect("serialize envelope");
        let loaded: VersionedEvent = serde_json::from_str(&json).expect("deserialize envelope");

        let restored = loaded.unwrap_latest().expect("unwrap v1 should work");
        assert_eq!(restored.id, old_event.id);
    }

    #[test]
    fn test_migration_v1_to_v2_purchase_completed() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::PurchaseCompleted {
                account_id: "user:42".to_string(),
                order_id: "order:1".to_string(),
                amount_cents: 1999,
            },
        );

        let v1 = VersionedEvent::wrap_v1(&event).expect("wrap v1");
        let v2_payload = migrate_v1_to_v2(&v1.payload).expect("migration should succeed");

        assert!(v2_payload.is_object());

        // Verify the migration applied correctly
        // The structure is: { "id": ..., "payload": { "PurchaseCompleted": { ... } } }
        let inner_payload = v2_payload
            .get("payload")
            .expect("Should have payload field");
        if let Some(purchase) = inner_payload.get("PurchaseCompleted") {
            // amount_cents should be renamed to amount_minor_units
            assert!(
                purchase.get("amount_minor_units").is_some(),
                "Should have amount_minor_units"
            );
            assert!(
                purchase.get("amount_cents").is_none(),
                "Should not have old amount_cents"
            );
            // currency should be added
            assert_eq!(
                purchase.get("currency").and_then(|v| v.as_str()),
                Some("USD"),
                "Should have currency=USD"
            );
        } else {
            panic!("Expected PurchaseCompleted variant");
        }
    }

    #[test]
    fn test_migration_preserves_non_purchase_events() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::StockDecremented {
                account_id: "user:42".to_string(),
                quantity: 1,
                remaining: 99,
            },
        );

        let v1 = VersionedEvent::wrap_v1(&event).expect("wrap v1");
        let v2_payload = migrate_v1_to_v2(&v1.payload).expect("migration should succeed");

        // Non-purchase events should pass through unchanged
        assert_eq!(v1.payload, v2_payload);
    }

    #[test]
    fn test_v2_wrap_and_unwrap() {
        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::PurchaseCompleted {
                account_id: "user:42".to_string(),
                order_id: "order:1".to_string(),
                amount_cents: 1999,
            },
        );

        let v2 = VersionedEvent::wrap_v2(&event).expect("wrap v2");
        assert_eq!(v2.version, 2);

        let restored = v2.unwrap_latest().expect("unwrap v2");
        assert_eq!(restored.id, event.id);
        assert_eq!(restored.payload, event.payload);
    }

    #[test]
    fn test_unknown_version_errors() {
        let bad = VersionedEvent {
            version: 99,
            payload: serde_json::json!({"unknown": true}),
        };
        let result = bad.unwrap_latest();
        assert!(result.is_err(), "Unknown version should produce an error");
    }
}
