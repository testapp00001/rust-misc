//! # Solution 02: Event Store
//!
//! Complete implementation of an in-memory append-only event store.

use std::sync::Mutex;

use super::p01_event_design::FlashSaleEvent;

/// An in-memory, append-only event store with thread-safe access.
pub struct EventStore {
    events: Mutex<Vec<FlashSaleEvent>>,
}

impl EventStore {
    /// Create a new, empty event store.
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    /// Append an event to the store.
    pub fn append(&self, event: FlashSaleEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Retrieve all events for a specific aggregate, in insertion order.
    pub fn get_events(&self, aggregate_id: &str) -> Vec<FlashSaleEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }

    /// Retrieve every event in the store, in insertion order.
    pub fn get_all_events(&self) -> Vec<FlashSaleEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Return the total number of events in the store.
    pub fn event_count(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::p01_event_design::EventPayload;

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
