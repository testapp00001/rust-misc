//! # Exercise 02: Event Replay
//!
//! ## Learning Objective
//! Implement event-sourced state reconstruction to verify that the current
//! inventory state matches what the event history says it should be.
//!
//! ## Flash Sale Context
//! Every inventory change (stock added, item reserved, purchase confirmed,
//! reservation released, manual adjustment) is recorded as an immutable event.
//! By replaying these events from the beginning, we can reconstruct what the
//! inventory *should* look like and compare it against what it *actually* is.
//! Any difference means there was data corruption or a missed event.
//!
//! ## Instructions
//! 1. Implement `EventReplay::new()` to hold the event store, current state,
//!    and initial stock level.
//! 2. Implement `replay_product_events()` to rebuild state from events:
//!    - Start from `initial_stock`
//!    - Process events in chronological order
//!    - Handle each EventType according to its semantics
//! 3. After replaying, compare rebuilt state against current state.
//! 4. Implement `replay_all()` to process every product in the event store.
//!
//! ## Hints
//! - `Reserved` moves stock from available to reserved.
//! - `Confirmed` removes from reserved (purchase done).
//! - `Released` moves from reserved back to available (user cancelled).
//! - `Adjusted` applies a signed delta (can be negative).
//! - If any operation would drive stock negative, return an error.

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::shared::{
    EventType, InventoryEvent, ReconciliationError, RebuiltState,
};

/// Simulated event store containing all inventory events.
pub struct MockEventStore {
    events: Vec<InventoryEvent>,
}

impl MockEventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: InventoryEvent) {
        self.events.push(event);
    }

    /// Get all events for a specific product, ordered by timestamp.
    pub fn get_events_for_product(&self, product_id: &str) -> Vec<InventoryEvent> {
        let mut filtered: Vec<InventoryEvent> = self
            .events
            .iter()
            .filter(|e| e.product_id == product_id)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        filtered
    }

    /// Get all events in the store.
    pub fn get_all_events(&self) -> Vec<InventoryEvent> {
        self.events.clone()
    }
}

/// Simulated current state store (what the system currently believes).
pub struct MockCurrentState {
    pub states: HashMap<String, (i64, i64)>, // (available, reserved)
}

impl MockCurrentState {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn set(&mut self, product_id: &str, available: i64, reserved: i64) {
        self.states
            .insert(product_id.to_string(), (available, reserved));
    }

    pub fn get(&self, product_id: &str) -> Option<(i64, i64)> {
        self.states.get(product_id).cloned()
    }
}

/// Replays events from the event store to rebuild inventory state,
/// then compares the rebuilt state against the current state to detect drift.
pub struct EventReplay {
    // TODO: add fields for event_store, current_state, initial_stock
}

impl EventReplay {
    /// Create a new EventReplay with the given stores and initial stock level.
    pub fn new(
        event_store: MockEventStore,
        current_state: MockCurrentState,
        initial_stock: i64,
    ) -> Self {
        todo!("Store all three parameters")
    }

    /// Replay all events for a product and rebuild its state from scratch.
    ///
    /// Starting from `initial_stock`, process each event in chronological order:
    /// - StockAdded: add quantity to available
    /// - Reserved: move quantity from available to reserved
    /// - Confirmed: remove from reserved (sale finalized)
    /// - Released: move from reserved back to available (cancellation)
    /// - Adjusted: apply signed adjustment to available
    pub fn replay_product_events(
        &self,
        product_id: &str,
    ) -> Result<RebuiltState, ReconciliationError> {
        todo!("Replay events for the product, rebuild state, and compare with current state")
    }

    /// Replay events for all products found in the event store.
    pub fn replay_all(&self) -> Result<Vec<RebuiltState>, ReconciliationError> {
        todo!("Collect unique product IDs and replay each one")
    }
}

/// Helper to create an InventoryEvent with defaults.
pub fn make_event(
    product_id: &str,
    event_type: EventType,
    quantity: i64,
) -> InventoryEvent {
    InventoryEvent {
        event_id: Uuid::new_v4(),
        product_id: product_id.to_string(),
        event_type,
        quantity,
        timestamp: Utc::now(),
        metadata: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_rebuild_no_events() {
        let event_store = MockEventStore::new();
        let mut current = MockCurrentState::new();
        current.set("SKU-1", 100, 0);

        let replay = EventReplay::new(event_store, current, 100);
        let state = replay.replay_product_events("SKU-1").unwrap();

        assert_eq!(state.computed_available, 100);
        assert_eq!(state.computed_reserved, 0);
        assert_eq!(state.events_processed, 0);
        assert!(state.matches_current);
    }

    #[test]
    fn test_correct_rebuild_with_events() {
        let mut event_store = MockEventStore::new();
        event_store.push(make_event("SKU-1", EventType::StockAdded, 50));
        event_store.push(make_event("SKU-1", EventType::Reserved, 30));
        event_store.push(make_event("SKU-1", EventType::Confirmed, 20));
        event_store.push(make_event("SKU-1", EventType::Released, 10));

        let mut current = MockCurrentState::new();
        // Initial 100 + 50 added - 30 reserved + 10 released = 130 available
        // Reserved: 30 - 20 confirmed - 10 released = 0
        current.set("SKU-1", 130, 0);

        let replay = EventReplay::new(event_store, current, 100);
        let state = replay.replay_product_events("SKU-1").unwrap();

        assert_eq!(state.computed_available, 130);
        assert_eq!(state.computed_reserved, 0);
        assert_eq!(state.events_processed, 4);
        assert!(state.matches_current);
    }

    #[test]
    fn test_detects_drift() {
        let mut event_store = MockEventStore::new();
        event_store.push(make_event("SKU-2", EventType::Reserved, 5));

        let mut current = MockCurrentState::new();
        // Current says 90 available, 3 reserved -- but replay says 95 available, 5 reserved.
        current.set("SKU-2", 90, 3);

        let replay = EventReplay::new(event_store, current, 100);
        let state = replay.replay_product_events("SKU-2").unwrap();

        assert_eq!(state.computed_available, 95);
        assert_eq!(state.computed_reserved, 5);
        assert!(!state.matches_current); // Drift detected!
    }

    #[test]
    fn test_adjustment_event() {
        let mut event_store = MockEventStore::new();
        event_store.push(make_event("SKU-3", EventType::Adjusted, -10));

        let mut current = MockCurrentState::new();
        current.set("SKU-3", 90, 0);

        let replay = EventReplay::new(event_store, current, 100);
        let state = replay.replay_product_events("SKU-3").unwrap();

        assert_eq!(state.computed_available, 90);
        assert!(state.matches_current);
    }

    #[test]
    fn test_replay_all_multiple_products() {
        let mut event_store = MockEventStore::new();
        event_store.push(make_event("A", EventType::Reserved, 10));
        event_store.push(make_event("B", EventType::StockAdded, 20));
        event_store.push(make_event("C", EventType::Reserved, 5));
        event_store.push(make_event("C", EventType::Confirmed, 5));

        let mut current = MockCurrentState::new();
        // A: 100 initial - 10 reserved = 90 available, 10 reserved
        current.set("A", 90, 10);
        // B: 100 initial + 20 added = 120 available, 0 reserved
        current.set("B", 120, 0);
        // C: 100 initial - 5 reserved + 5 confirmed(not affecting available) = 95 available, 0 reserved
        current.set("C", 95, 0);

        let replay = EventReplay::new(event_store, current, 100);
        let results = replay.replay_all().unwrap();

        assert_eq!(results.len(), 3);

        let a = results.iter().find(|r| r.product_id == "A").unwrap();
        assert!(a.matches_current);
        assert_eq!(a.computed_available, 90);
        assert_eq!(a.computed_reserved, 10);

        let b = results.iter().find(|r| r.product_id == "B").unwrap();
        assert!(b.matches_current);
        assert_eq!(b.computed_available, 120);

        let c = results.iter().find(|r| r.product_id == "C").unwrap();
        assert!(c.matches_current);
        assert_eq!(c.computed_available, 95);
        assert_eq!(c.computed_reserved, 0);
    }
}
