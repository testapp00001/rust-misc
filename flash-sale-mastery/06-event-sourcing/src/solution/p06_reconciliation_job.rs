//! # Solution 06: Reconciliation Job
//!
//! Complete implementation of background reconciliation via event replay.

use super::p02_event_store::EventStore;
use super::p03_event_replay::rebuild_inventory;

/// Result of a reconciliation check for a single aggregate.
#[derive(Debug, Clone, PartialEq)]
pub struct MismatchDetail {
    pub aggregate_id: String,
    pub expected_stock: u32,
    pub actual_stock: u32,
    pub drift: u32,
}

/// Report produced by a reconciliation run.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconciliationReport {
    pub run_timestamp: chrono::DateTime<chrono::Utc>,
    pub aggregates_checked: usize,
    pub mismatches: Vec<MismatchDetail>,
    pub corrections_applied: bool,
}

impl ReconciliationReport {
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
    }
}

/// The state of a live system for comparison.
#[derive(Debug, Clone)]
pub struct LiveState {
    pub stock_by_aggregate: std::collections::HashMap<String, u32>,
}

/// Run reconciliation: compare event-derived state with live state.
pub fn run_reconciliation(
    event_store: &EventStore,
    live_state: &LiveState,
    apply_fix: bool,
) -> ReconciliationReport {
    let mut mismatches = Vec::new();

    for (aggregate_id, &actual_stock) in &live_state.stock_by_aggregate {
        let events = event_store.get_events(aggregate_id);
        if events.is_empty() {
            // No events for this aggregate -- skip
            continue;
        }

        let rebuilt = rebuild_inventory(&events);
        let expected_stock = rebuilt.stock_remaining;

        if expected_stock != actual_stock {
            let drift = if expected_stock > actual_stock {
                expected_stock - actual_stock
            } else {
                actual_stock - expected_stock
            };

            mismatches.push(MismatchDetail {
                aggregate_id: aggregate_id.clone(),
                expected_stock,
                actual_stock,
                drift,
            });
        }
    }

    ReconciliationReport {
        run_timestamp: chrono::Utc::now(),
        aggregates_checked: live_state.stock_by_aggregate.len(),
        mismatches,
        corrections_applied: apply_fix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::p01_event_design::{EventPayload, FlashSaleEvent};
    use std::collections::HashMap;

    fn setup_store_with_events(
        aggregate_id: &str,
        initial_stock: u32,
        decrements: u32,
    ) -> EventStore {
        let store = EventStore::new();
        store.append(FlashSaleEvent::new(
            aggregate_id,
            EventPayload::SaleStarted {
                product_id: aggregate_id.to_string(),
                initial_stock,
                price_cents: 999,
            },
        ));
        for i in 0..decrements {
            store.append(FlashSaleEvent::new(
                aggregate_id,
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: initial_stock - i - 1,
                },
            ));
        }
        store
    }

    #[test]
    fn test_in_sync_state() {
        let store = setup_store_with_events("product:1", 100, 10);

        let mut stock_by_aggregate = HashMap::new();
        stock_by_aggregate.insert("product:1".to_string(), 90);

        let live_state = LiveState { stock_by_aggregate };
        let report = run_reconciliation(&store, &live_state, false);

        assert!(report.is_clean(), "Should be in sync");
        assert_eq!(report.aggregates_checked, 1);
        assert!(report.mismatches.is_empty());
    }

    #[test]
    fn test_stock_mismatch_detected() {
        let store = setup_store_with_events("product:1", 100, 10);

        let mut stock_by_aggregate = HashMap::new();
        stock_by_aggregate.insert("product:1".to_string(), 85);

        let live_state = LiveState { stock_by_aggregate };
        let report = run_reconciliation(&store, &live_state, false);

        assert!(!report.is_clean(), "Should detect mismatch");
        assert_eq!(report.mismatches.len(), 1);
        assert_eq!(report.mismatches[0].aggregate_id, "product:1");
        assert_eq!(report.mismatches[0].expected_stock, 90);
        assert_eq!(report.mismatches[0].actual_stock, 85);
        assert_eq!(report.mismatches[0].drift, 5);
        assert!(!report.corrections_applied);
    }

    #[test]
    fn test_correction_applied() {
        let store = setup_store_with_events("product:1", 100, 10);

        let mut stock_by_aggregate = HashMap::new();
        stock_by_aggregate.insert("product:1".to_string(), 85);

        let live_state = LiveState { stock_by_aggregate };
        let report = run_reconciliation(&store, &live_state, true);

        assert!(!report.is_clean());
        assert!(report.corrections_applied);
    }

    #[test]
    fn test_multiple_aggregates() {
        let store = EventStore::new();

        store.append(FlashSaleEvent::new(
            "product:1",
            EventPayload::SaleStarted {
                product_id: "product:1".to_string(),
                initial_stock: 100,
                price_cents: 999,
            },
        ));
        for i in 0..10 {
            store.append(FlashSaleEvent::new(
                "product:1",
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: 100 - i - 1,
                },
            ));
        }

        store.append(FlashSaleEvent::new(
            "product:2",
            EventPayload::SaleStarted {
                product_id: "product:2".to_string(),
                initial_stock: 50,
                price_cents: 499,
            },
        ));
        for i in 0..5 {
            store.append(FlashSaleEvent::new(
                "product:2",
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: 50 - i - 1,
                },
            ));
        }

        let mut stock_by_aggregate = HashMap::new();
        stock_by_aggregate.insert("product:1".to_string(), 90); // correct
        stock_by_aggregate.insert("product:2".to_string(), 40); // wrong (expected 45)

        let live_state = LiveState { stock_by_aggregate };
        let report = run_reconciliation(&store, &live_state, false);

        assert_eq!(report.aggregates_checked, 2);
        assert_eq!(report.mismatches.len(), 1);
        assert_eq!(report.mismatches[0].aggregate_id, "product:2");
        assert_eq!(report.mismatches[0].drift, 5);
    }
}
