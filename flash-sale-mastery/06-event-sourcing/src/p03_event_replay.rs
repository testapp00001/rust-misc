//! # Exercise 03: Event Replay
//!
//! ## Learning Objective
//! Rebuild application state by replaying a sequence of events. This is the
//! core mechanism of event sourcing: the current state is derived entirely
//! from the event log, not from a separate database.
//!
//! ## Flash Sale Context
//! After a flash sale, you need to know exactly how many items remain, how
//! many each account claimed, and how many vouchers were issued. Instead of
//! trusting a mutable counter (which could be wrong due to bugs or race
//! conditions), replay every event to reconstruct the true state.
//!
//! ## Instructions
//! 1. Implement the `InventoryState` struct to track stock, per-account claims,
//!    and voucher counts
//! 2. Implement `InventoryState::new` with an initial stock value
//! 3. Implement `apply_event` to update state based on a single event
//! 4. Implement `rebuild_inventory` to reconstruct state from a slice of events
//! 5. Sort events by timestamp before replaying to handle out-of-order delivery
//!
//! ## Hints
//! - `rebuild_inventory` should sort events by timestamp before applying
//! - Use a `HashMap<String, u32>` for per-account claim counts
//! - Use a `HashMap<String, u32>` for voucher counts per account
//! - Unknown event types can be safely ignored during replay

use std::collections::HashMap;

use crate::p01_event_design::{EventPayload, FlashSaleEvent};

/// Represents the current inventory state, derived from event replay.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryState {
    /// Remaining stock for the product.
    pub stock_remaining: u32,
    /// How many items each account has claimed.
    pub claims_per_account: HashMap<String, u32>,
    /// How many vouchers each account has claimed.
    pub voucher_claims: HashMap<String, u32>,
    /// Total number of completed purchases.
    pub completed_purchases: u32,
    /// Total number of failed purchases.
    pub failed_purchases: u32,
}

impl InventoryState {
    /// Create a new inventory state with the given initial stock.
    pub fn new(initial_stock: u32) -> Self {
        todo!("Implement InventoryState::new")
    }

    /// Apply a single event to update this state.
    ///
    /// This is the "fold" operation: given a state and an event, produce
    /// the new state.
    pub fn apply_event(&mut self, event: &FlashSaleEvent) {
        todo!("Implement apply_event -- match on event.payload variants")
    }
}

/// Rebuild inventory state from a slice of events.
///
/// Events are sorted by timestamp before replay to handle out-of-order delivery.
///
/// # Arguments
/// * `events` - The events to replay
///
/// # Returns
/// The reconstructed `InventoryState`.
pub fn rebuild_inventory(events: &[FlashSaleEvent]) -> InventoryState {
    // TODO: Sort events by timestamp, create a default InventoryState,
    //       and apply each event in order.
    //       You'll need to figure out the initial stock from SaleStarted events,
    //       or use a reasonable default.
    todo!("Implement rebuild_inventory")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sale_started(aggregate_id: &str, initial_stock: u32) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::SaleStarted {
                product_id: aggregate_id.to_string(),
                initial_stock,
                price_cents: 999,
            },
        )
    }

    fn stock_decremented(
        aggregate_id: &str,
        account_id: &str,
        remaining: u32,
    ) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::StockDecremented {
                account_id: account_id.to_string(),
                quantity: 1,
                remaining,
            },
        )
    }

    fn voucher_claimed(aggregate_id: &str, account_id: &str) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::VoucherClaimed {
                account_id: account_id.to_string(),
                voucher_code: format!("VOUCHER-{}", Uuid::new_v4()),
            },
        )
    }

    fn purchase_completed(aggregate_id: &str, account_id: &str) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::PurchaseCompleted {
                account_id: account_id.to_string(),
                order_id: format!("order:{}", Uuid::new_v4()),
                amount_cents: 999,
            },
        )
    }

    fn purchase_failed(aggregate_id: &str, account_id: &str) -> FlashSaleEvent {
        FlashSaleEvent::new(
            aggregate_id,
            EventPayload::PurchaseFailed {
                account_id: account_id.to_string(),
                reason: "out of stock".to_string(),
            },
        )
    }

    #[test]
    fn test_initial_state() {
        let state = InventoryState::new(100);
        assert_eq!(state.stock_remaining, 100);
        assert!(state.claims_per_account.is_empty());
        assert!(state.voucher_claims.is_empty());
        assert_eq!(state.completed_purchases, 0);
    }

    #[test]
    fn test_apply_stock_decremented() {
        let mut state = InventoryState::new(100);
        let event = stock_decremented("product:1", "user:1", 99);
        state.apply_event(&event);
        assert_eq!(state.stock_remaining, 99);
        assert_eq!(*state.claims_per_account.get("user:1").unwrap(), 1);
    }

    #[test]
    fn test_apply_voucher_claimed() {
        let mut state = InventoryState::new(100);
        let event = voucher_claimed("product:1", "user:1");
        state.apply_event(&event);
        assert_eq!(*state.voucher_claims.get("user:1").unwrap(), 1);
    }

    #[test]
    fn test_rebuild_from_events() {
        let events = vec![
            sale_started("product:1", 100),
            stock_decremented("product:1", "user:1", 99),
            stock_decremented("product:1", "user:2", 98),
            stock_decremented("product:1", "user:1", 97),
            voucher_claimed("product:1", "user:1"),
            purchase_completed("product:1", "user:1"),
            purchase_failed("product:1", "user:3"),
        ];
        let state = rebuild_inventory(&events);
        assert_eq!(state.stock_remaining, 97);
        assert_eq!(*state.claims_per_account.get("user:1").unwrap(), 2);
        assert_eq!(*state.claims_per_account.get("user:2").unwrap(), 1);
        assert_eq!(*state.voucher_claims.get("user:1").unwrap(), 1);
        assert_eq!(state.completed_purchases, 1);
        assert_eq!(state.failed_purchases, 1);
    }

    #[test]
    fn test_out_of_order_events_handled() {
        // Events arrive out of order -- rebuild should sort by timestamp
        let mut e1 = stock_decremented("product:1", "user:1", 99);
        let mut e2 = stock_decremented("product:1", "user:2", 98);
        let mut e3 = stock_decremented("product:1", "user:3", 97);

        // Set timestamps so e1 < e3 < e2 (chronologically)
        e1.timestamp = Utc::now() - chrono::Duration::seconds(10);
        e2.timestamp = Utc::now();
        e3.timestamp = Utc::now() - chrono::Duration::seconds(5);

        // Events provided in wrong order: e2, e3, e1
        // But timestamps say: e1 (oldest), e3, e2 (newest)
        let events = vec![e2, e3, e1];
        let state = rebuild_inventory(&events);
        // After sorting by timestamp: e1 (remaining=99), e3 (remaining=98), e2 (remaining=98)
        // The last event's remaining value wins
        assert_eq!(state.stock_remaining, 98);
    }
}
