//! # Exercise 02: Event Store
//!
//! ## Learning Objective
//! Build an in-memory append-only event store. The event store is the central
//! component of event sourcing -- it records every event and provides retrieval
//! by aggregate. Events are never updated or deleted, only appended.
//!
//! ## Flash Sale Context
//! When 10,000 users try to buy the same product simultaneously, each successful
//! stock decrement must be recorded atomically. The event store guarantees that
//! every change is captured in order, providing a complete history that can be
//! replayed to reconstruct state at any point in time.
//!
//! ## Instructions
//! 1. Implement `EventStore::new()` to create an empty store
//! 2. Implement `append` to add an event (stores in memory, preserves insertion order)
//! 3. Implement `get_events` to retrieve all events for a given aggregate_id
//! 4. Implement `get_all_events` to retrieve every event in the store
//! 5. Implement `event_count` to return the total number of stored events
//!
//! ## Hints
//! - Use a `Vec<FlashSaleEvent>` internally, wrapped in a `Mutex` for thread safety
//! - `get_events` should filter by aggregate_id and return events in insertion order
//! - The store is append-only: no update or delete operations

use std::sync::Mutex;

use crate::p01_event_design::{EventPayload, FlashSaleEvent};

/// An in-memory, append-only event store.
///
/// Events are stored in insertion order and can be retrieved by aggregate_id
/// or as a complete log. The store is thread-safe via internal Mutex.
pub struct EventStore {
    // TODO: Add an internal field to store events
    //       Hint: Mutex<Vec<FlashSaleEvent>>
}

impl EventStore {
    /// Create a new, empty event store.
    pub fn new() -> Self {
        todo!("Implement EventStore::new")
    }

    /// Append an event to the store.
    ///
    /// The event is added to the end of the log. Events are immutable once stored.
    pub fn append(&self, event: FlashSaleEvent) {
        todo!("Implement append")
    }

    /// Retrieve all events for a specific aggregate, in insertion order.
    ///
    /// # Arguments
    /// * `aggregate_id` - The aggregate to retrieve events for
    pub fn get_events(&self, aggregate_id: &str) -> Vec<FlashSaleEvent> {
        todo!("Implement get_events")
    }

    /// Retrieve every event in the store, in insertion order.
    pub fn get_all_events(&self) -> Vec<FlashSaleEvent> {
        todo!("Implement get_all_events")
    }

    /// Return the total number of events in the store.
    pub fn event_count(&self) -> usize {
        todo!("Implement event_count")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(aggregate_id: &str, account_id: &str, remaining: u32) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::StockDecremented {
                account_id: account_id.to_string(),
                quantity: 1,
                remaining,
            },
        )
    }

    #[test]
    fn test_empty_store() {
        let store = EventStore::new();
        assert_eq!(store.event_count(), 0);
        assert!(store.get_all_events().is_empty());
        assert!(store.get_events("product:1").is_empty());
    }

    #[test]
    fn test_append_and_retrieve() {
        let store = EventStore::new();
        let e1 = make_event("product:1", "user:1", 99);
        let e2 = make_event("product:1", "user:2", 98);
        store.append(e1.clone());
        store.append(e2.clone());

        assert_eq!(store.event_count(), 2);
        let events = store.get_events("product:1");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, e1.id);
        assert_eq!(events[1].id, e2.id);
    }

    #[test]
    fn test_events_filtered_by_aggregate() {
        let store = EventStore::new();
        store.append(make_event("product:1", "user:1", 99));
        store.append(make_event("product:2", "user:1", 49));
        store.append(make_event("product:1", "user:2", 98));
        store.append(make_event("product:2", "user:2", 48));

        let p1 = store.get_events("product:1");
        let p2 = store.get_events("product:2");
        assert_eq!(p1.len(), 2);
        assert_eq!(p2.len(), 2);
        assert!(p1.iter().all(|e| e.aggregate_id == "product:1"));
        assert!(p2.iter().all(|e| e.aggregate_id == "product:2"));
    }

    #[test]
    fn test_get_all_events_ordering() {
        let store = EventStore::new();
        let events: Vec<_> = (0..10)
            .map(|i| make_event("product:1", &format!("user:{i}"), 100 - i))
            .collect();
        for e in &events {
            store.append(e.clone());
        }
        let all = store.get_all_events();
        assert_eq!(all.len(), 10);
        for (stored, original) in all.iter().zip(events.iter()) {
            assert_eq!(stored.id, original.id);
        }
    }

    #[test]
    fn test_event_count() {
        let store = EventStore::new();
        assert_eq!(store.event_count(), 0);
        store.append(make_event("product:1", "user:1", 99));
        assert_eq!(store.event_count(), 1);
        store.append(make_event("product:2", "user:1", 49));
        assert_eq!(store.event_count(), 2);
    }
}
