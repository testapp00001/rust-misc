//! # Exercise 08: Event Versioning
//!
//! ## Learning Objective
//! Handle event schema evolution with versioned event envelopes. As the system
//! evolves, event schemas change. Old events stored in the log must still be
//! readable. Versioned envelopes and migration functions ensure backward
//! compatibility.
//!
//! ## Flash Sale Context
//! Six months after launch, the team decides to add a `currency` field to
//! `PurchaseCompleted` events and rename `amount_cents` to `amount_minor_units`.
//! The old events in the log don't have these fields. Without versioning, replay
//! would fail on old events. With versioned envelopes, old events are migrated
//! to the current schema before processing.
//!
//! ## Instructions
//! 1. Implement `VersionedEvent` envelope with version number and JSON payload
//! 2. Implement `wrap_v1` to create a v1 envelope from a `FlashSaleEvent`
//! 3. Implement `migrate_v1_to_v2` to transform a v1 event to v2 schema
//! 4. Implement `unwrap_latest` to deserialize any version into the latest schema
//!
//! ## Hints
//! - The envelope stores the version as a `u32` and the payload as `serde_json::Value`
//! - Migration functions take a `Value` and return a transformed `Value`
//! - `unwrap_latest` should dispatch to the appropriate migration based on version
//! - Missing fields during migration should get sensible defaults

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::p01_event_design::{EventError, FlashSaleEvent};

/// A versioned event envelope.
///
/// Wraps the event payload with a version number so that old events can
/// be migrated to the current schema on read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedEvent {
    /// Schema version of the payload.
    pub version: u32,
    /// The event data as a JSON value.
    pub payload: Value,
}

/// The latest schema version number.
pub const LATEST_VERSION: u32 = 2;

impl VersionedEvent {
    /// Wrap a FlashSaleEvent in a v1 envelope.
    pub fn wrap_v1(event: &FlashSaleEvent) -> Result<Self, EventError> {
        // TODO: Serialize event to JSON Value, create VersionedEvent with version=1
        todo!("Implement wrap_v1")
    }

    /// Wrap a FlashSaleEvent in a v2 envelope.
    pub fn wrap_v2(event: &FlashSaleEvent) -> Result<Self, EventError> {
        // TODO: Serialize event to JSON Value, create VersionedEvent with version=2
        todo!("Implement wrap_v2")
    }

    /// Unwrap this envelope, migrating to the latest schema version.
    ///
    /// Returns the deserialized FlashSaleEvent with all migrations applied.
    pub fn unwrap_latest(&self) -> Result<FlashSaleEvent, EventError> {
        // TODO: Based on self.version, apply appropriate migrations
        //       version 1 -> migrate_v1_to_v2 -> deserialize
        //       version 2 -> deserialize directly
        //       unknown version -> error
        todo!("Implement unwrap_latest")
    }
}

/// Migrate a v1 event payload to v2 schema.
///
/// V2 changes:
/// - `PurchaseCompleted.amount_cents` is renamed to `PurchaseCompleted.amount_minor_units`
/// - A `currency` field is added to `PurchaseCompleted` (defaults to "USD")
///
/// # Arguments
/// * `v1_payload` - The v1 event as a JSON Value
///
/// # Returns
/// The migrated v2 event as a JSON Value.
pub fn migrate_v1_to_v2(v1_payload: &Value) -> Result<Value, EventError> {
    // TODO: Clone the payload, check if it's a PurchaseCompleted variant
    //       If so, rename amount_cents -> amount_minor_units and add currency: "USD"
    //       For other variants, return as-is
    todo!("Implement migrate_v1_to_v2")
}

/// Helper: apply all migrations from a given version to the latest.
pub fn migrate(payload: &Value, from_version: u32) -> Result<Value, EventError> {
    let mut current = payload.clone();
    if from_version < 2 {
        current = migrate_v1_to_v2(&current)?;
    }
    // Future migrations go here:
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
        // Simulate an old v1 event stored as JSON
        let old_event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::PurchaseCompleted {
                account_id: "user:42".to_string(),
                order_id: "order:1".to_string(),
                amount_cents: 1999,
            },
        );

        let v1 = VersionedEvent::wrap_v1(&old_event).expect("wrap v1");
        // Simulate reading from storage: deserialize the raw JSON
        let json = serde_json::to_string(&v1).expect("serialize envelope");
        let loaded: VersionedEvent = serde_json::from_str(&json).expect("deserialize envelope");

        // Should be able to unwrap even though it's v1
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

        // Check that the migration transformed the payload
        // The exact structure depends on implementation, but the key checks are:
        // 1. It's valid JSON
        // 2. The type discriminant is preserved
        assert!(v2_payload.is_object());
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
