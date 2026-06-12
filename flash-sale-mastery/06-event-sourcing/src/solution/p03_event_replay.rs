//! # Solution 03: Event Replay
//!
//! Complete implementation of inventory state reconstruction via event replay.

use std::collections::HashMap;

use super::p01_event_design::{EventPayload, FlashSaleEvent};

/// Represents the current inventory state, derived from event replay.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryState {
    pub stock_remaining: u32,
    pub claims_per_account: HashMap<String, u32>,
    pub voucher_claims: HashMap<String, u32>,
    pub completed_purchases: u32,
    pub failed_purchases: u32,
}

impl InventoryState {
    /// Create a new inventory state with the given initial stock.
    pub fn new(initial_stock: u32) -> Self {
        Self {
            stock_remaining: initial_stock,
            claims_per_account: HashMap::new(),
            voucher_claims: HashMap::new(),
            completed_purchases: 0,
            failed_purchases: 0,
        }
    }

    /// Apply a single event to update this state.
    pub fn apply_event(&mut self, event: &FlashSaleEvent) {
        match &event.payload {
            EventPayload::SaleStarted { initial_stock, .. } => {
                self.stock_remaining = *initial_stock;
            }
            EventPayload::StockDecremented {
                account_id,
                remaining,
                ..
            } => {
                self.stock_remaining = *remaining;
                *self.claims_per_account.entry(account_id.clone()).or_insert(0) += 1;
            }
            EventPayload::VoucherClaimed { account_id, .. } => {
                *self.voucher_claims.entry(account_id.clone()).or_insert(0) += 1;
            }
            EventPayload::PurchaseCompleted { .. } => {
                self.completed_purchases += 1;
            }
            EventPayload::PurchaseFailed { .. } => {
                self.failed_purchases += 1;
            }
            EventPayload::SaleEnded { final_stock, .. } => {
                self.stock_remaining = *final_stock;
            }
        }
    }
}

/// Rebuild inventory state from a slice of events.
///
/// Events are sorted by timestamp before replay. If no SaleStarted event is
/// found, a default initial stock of 0 is used.
pub fn rebuild_inventory(events: &[FlashSaleEvent]) -> InventoryState {
    let mut sorted: Vec<&FlashSaleEvent> = events.iter().collect();
    sorted.sort_by_key(|e| e.timestamp);

    // Determine initial stock from SaleStarted events (use the last one if multiple)
    let initial_stock = sorted
        .iter()
        .filter_map(|e| match &e.payload {
            EventPayload::SaleStarted { initial_stock, .. } => Some(*initial_stock),
            _ => None,
        })
        .next_back()
        .unwrap_or(0);

    let mut state = InventoryState::new(initial_stock);
    for event in &sorted {
        state.apply_event(event);
    }
    state
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
        assert_eq!(*state.claims_per_account.get("user:1").unwrap(), 1);
        assert_eq!(*state.claims_per_account.get("user:2").unwrap(), 1);
        assert_eq!(*state.claims_per_account.get("user:3").unwrap(), 1);
    }
}
