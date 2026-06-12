//! # Solution 07: Audit Query
//!
//! Complete implementation of event store queries for audit purposes.

use chrono::{DateTime, Utc};

use super::p01_event_design::{EventPayload, FlashSaleEvent};
use super::p02_event_store::EventStore;

/// Error type for audit query operations.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("No events found matching the query criteria")]
    NoResults,

    #[error("Invalid time range: start ({start}) is after end ({end})")]
    InvalidTimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// Get the account_id from an event's payload, if present.
fn extract_account_id(event: &FlashSaleEvent) -> Option<&str> {
    match &event.payload {
        EventPayload::StockDecremented { account_id, .. } => Some(account_id),
        EventPayload::VoucherClaimed { account_id, .. } => Some(account_id),
        EventPayload::PurchaseCompleted { account_id, .. } => Some(account_id),
        EventPayload::PurchaseFailed { account_id, .. } => Some(account_id),
        EventPayload::SaleStarted { .. } => None,
        EventPayload::SaleEnded { .. } => None,
    }
}

/// Retrieve all events where the given account_id appears in the payload.
pub fn events_for_account(store: &EventStore, account_id: &str) -> Vec<FlashSaleEvent> {
    let mut events: Vec<FlashSaleEvent> = store
        .get_all_events()
        .into_iter()
        .filter(|e| extract_account_id(e).map_or(false, |aid| aid == account_id))
        .collect();
    events.sort_by_key(|e| e.timestamp);
    events
}

/// Retrieve all events for a given product (aggregate_id).
pub fn events_for_product(store: &EventStore, product_id: &str) -> Vec<FlashSaleEvent> {
    let mut events = store.get_events(product_id);
    events.sort_by_key(|e| e.timestamp);
    events
}

/// Retrieve all events within a time range (inclusive).
pub fn events_in_time_range(
    store: &EventStore,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<FlashSaleEvent> {
    let mut events: Vec<FlashSaleEvent> = store
        .get_all_events()
        .into_iter()
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .collect();
    events.sort_by_key(|e| e.timestamp);
    events
}

/// Retrieve events for a specific account AND product combination.
pub fn events_for_account_and_product(
    store: &EventStore,
    account_id: &str,
    product_id: &str,
) -> Vec<FlashSaleEvent> {
    let mut events: Vec<FlashSaleEvent> = store
        .get_events(product_id)
        .into_iter()
        .filter(|e| extract_account_id(e).map_or(false, |aid| aid == account_id))
        .collect();
    events.sort_by_key(|e| e.timestamp);
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_event_design::EventPayload;
    use chrono::Duration;

    fn build_test_store() -> EventStore {
        let store = EventStore::new();

        store.append(FlashSaleEvent::new(
            "product:1",
            EventPayload::SaleStarted {
                product_id: "product:1".to_string(),
                initial_stock: 100,
                price_cents: 999,
            },
        ));
        store.append(FlashSaleEvent::new(
            "product:1",
            EventPayload::StockDecremented {
                account_id: "user:alice".to_string(),
                quantity: 1,
                remaining: 99,
            },
        ));
        store.append(FlashSaleEvent::new(
            "product:1",
            EventPayload::StockDecremented {
                account_id: "user:bob".to_string(),
                quantity: 1,
                remaining: 98,
            },
        ));
        store.append(FlashSaleEvent::new(
            "product:1",
            EventPayload::PurchaseCompleted {
                account_id: "user:alice".to_string(),
                order_id: "order:1".to_string(),
                amount_cents: 999,
            },
        ));

        store.append(FlashSaleEvent::new(
            "product:2",
            EventPayload::SaleStarted {
                product_id: "product:2".to_string(),
                initial_stock: 50,
                price_cents: 499,
            },
        ));
        store.append(FlashSaleEvent::new(
            "product:2",
            EventPayload::VoucherClaimed {
                account_id: "user:alice".to_string(),
                voucher_code: "FLASH50".to_string(),
            },
        ));
        store.append(FlashSaleEvent::new(
            "product:2",
            EventPayload::PurchaseFailed {
                account_id: "user:charlie".to_string(),
                reason: "out of stock".to_string(),
            },
        ));

        store
    }

    #[test]
    fn test_events_for_account() {
        let store = build_test_store();

        let alice_events = events_for_account(&store, "user:alice");
        assert_eq!(alice_events.len(), 3);

        let bob_events = events_for_account(&store, "user:bob");
        assert_eq!(bob_events.len(), 1);

        let charlie_events = events_for_account(&store, "user:charlie");
        assert_eq!(charlie_events.len(), 1);
    }

    #[test]
    fn test_events_for_product() {
        let store = build_test_store();

        let p1 = events_for_product(&store, "product:1");
        assert_eq!(p1.len(), 4);

        let p2 = events_for_product(&store, "product:2");
        assert_eq!(p2.len(), 3);

        let p3 = events_for_product(&store, "product:999");
        assert!(p3.is_empty());
    }

    #[test]
    fn test_events_in_time_range() {
        let store = EventStore::new();

        let now = Utc::now();
        let mut e1 = FlashSaleEvent::new(
            "product:1",
            EventPayload::SaleStarted {
                product_id: "product:1".to_string(),
                initial_stock: 100,
                price_cents: 999,
            },
        );
        e1.timestamp = now - Duration::hours(2);

        let mut e2 = FlashSaleEvent::new(
            "product:1",
            EventPayload::StockDecremented {
                account_id: "user:1".to_string(),
                quantity: 1,
                remaining: 99,
            },
        );
        e2.timestamp = now - Duration::hours(1);

        let mut e3 = FlashSaleEvent::new(
            "product:1",
            EventPayload::StockDecremented {
                account_id: "user:2".to_string(),
                quantity: 1,
                remaining: 98,
            },
        );
        e3.timestamp = now;

        store.append(e1);
        store.append(e2);
        store.append(e3);

        let recent = events_in_time_range(&store, now - Duration::minutes(90), now);
        assert_eq!(recent.len(), 2);

        let all = events_in_time_range(&store, now - Duration::hours(3), now);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_events_for_account_and_product() {
        let store = build_test_store();

        let events = events_for_account_and_product(&store, "user:alice", "product:1");
        assert_eq!(events.len(), 2);

        let events = events_for_account_and_product(&store, "user:alice", "product:2");
        assert_eq!(events.len(), 1);

        let events = events_for_account_and_product(&store, "user:bob", "product:2");
        assert!(events.is_empty());
    }

    #[test]
    fn test_results_sorted_by_timestamp() {
        let store = build_test_store();
        let events = events_for_product(&store, "product:1");
        for window in events.windows(2) {
            assert!(window[0].timestamp <= window[1].timestamp);
        }
    }
}
