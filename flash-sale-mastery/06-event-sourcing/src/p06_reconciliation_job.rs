//! # Exercise 06: Reconciliation Job
//!
//! ## Learning Objective
//! Build a background reconciliation job that compares the state derived from
//! event replay against the actual current state. This detects drift caused by
//! bugs, race conditions, or data corruption.
//!
//! ## Flash Sale Context
//! In a high-throughput flash sale, the live stock counter (in Redis) and the
//! event-sourced state can drift apart due to network issues, partial failures,
//! or software bugs. A periodic reconciliation job replays events and compares
//! the result with the live state, reporting and correcting any discrepancies.
//!
//! ## Instructions
//! 1. Implement `ReconciliationReport` struct to capture findings
//! 2. Implement `run_reconciliation` that compares event-derived state vs actual
//! 3. Report should include: aggregate_id, expected vs actual stock, corrections applied
//! 4. Implement `apply_corrections` to fix detected mismatches
//!
//! ## Hints
//! - Rebuild state from events using the `rebuild_inventory` function from p03
//! - Compare `stock_remaining` from rebuilt state vs the `actual_stock` parameter
//! - The report should list all mismatches and whether they were corrected

use crate::p01_event_design::FlashSaleEvent;
use crate::p02_event_store::EventStore;
use crate::p03_event_replay::{rebuild_inventory, InventoryState};

/// Result of a reconciliation check for a single aggregate.
#[derive(Debug, Clone, PartialEq)]
pub struct MismatchDetail {
    /// The aggregate that has a mismatch.
    pub aggregate_id: String,
    /// Stock as computed from event replay.
    pub expected_stock: u32,
    /// Stock as reported by the live system.
    pub actual_stock: u32,
    /// The absolute difference.
    pub drift: u32,
}

/// Report produced by a reconciliation run.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconciliationReport {
    /// When the reconciliation ran.
    pub run_timestamp: chrono::DateTime<chrono::Utc>,
    /// Total aggregates checked.
    pub aggregates_checked: usize,
    /// Mismatches found.
    pub mismatches: Vec<MismatchDetail>,
    /// Whether corrections were applied.
    pub corrections_applied: bool,
}

impl ReconciliationReport {
    /// Returns true if the system is fully in sync.
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
    }
}

/// The state of a live system for comparison.
#[derive(Debug, Clone)]
pub struct LiveState {
    /// Aggregate ID -> actual stock count.
    pub stock_by_aggregate: std::collections::HashMap<String, u32>,
}

/// Run reconciliation: compare event-derived state with live state.
///
/// For each aggregate in the live state, rebuild the expected state from
/// events and compare stock values.
///
/// # Arguments
/// * `event_store` - The event store to replay from
/// * `live_state` - The current live state to compare against
/// * `apply_fix` - If true, report that corrections would be applied
///
/// # Returns
/// A `ReconciliationReport` with all findings.
pub fn run_reconciliation(
    event_store: &EventStore,
    live_state: &LiveState,
    apply_fix: bool,
) -> ReconciliationReport {
    // TODO: For each aggregate in live_state.stock_by_aggregate:
    //   1. Get events from event_store for that aggregate
    //   2. Rebuild inventory state using rebuild_inventory
    //   3. Compare expected_stock (from events) with actual_stock (from live_state)
    //   4. If they differ, add a MismatchDetail to the report
    // Return the report with corrections_applied = apply_fix
    todo!("Implement run_reconciliation")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_event_design::EventPayload;
    use std::collections::HashMap;

    fn setup_store_with_events(aggregate_id: &str, initial_stock: u32, decrements: u32) -> EventStore {
        let store = EventStore::new();
        // SaleStarted
        store.append(FlashSaleEvent::new(
            aggregate_id,
            EventPayload::SaleStarted {
                product_id: aggregate_id.to_string(),
                initial_stock,
                price_cents: 999,
            },
        ));
        // Decrement stock
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
        // Live says 85, but events say 90 -- drift of 5
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
        assert!(report.corrections_applied, "Corrections should be marked as applied");
    }

    #[test]
    fn test_multiple_aggregates() {
        let store = EventStore::new();

        // product:1 -- 100 initial, 10 decrements = 90
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

        // product:2 -- 50 initial, 5 decrements = 45
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
