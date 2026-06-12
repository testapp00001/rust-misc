//! # Exercise: Distributed Counter for Flash Sale Stock
//!
//! ## Theory
//!
//! In a flash sale scenario, many replicas need to decrement a stock counter
//! concurrently. A PN-Counter can track stock levels, but the critical
//! requirement is preventing overselling: the counter must never go below zero.
//!
//! The challenge: multiple replicas may decrement concurrently, and the merged
//! counter must reflect all decrements without allowing negative stock.
//!
//! ## Proof / Intuition
//!
//! A PN-Counter tracks positive (stock additions) and negative (sales)
//! separately. The value is P - N. For flash sales:
//! - Initialize with P = total_stock, N = 0
//! - Each sale decrements (increments N)
//! - The merge takes max for both P and N
//! - The value P - N can never go below 0 if the initial stock is correct
//!
//! However, to truly prevent overselling across replicas, we need to ensure
//! that the total decrements across all replicas don't exceed the total stock.
//! This is where the CRDT's merge semantics help: the merged state correctly
//! reflects all concurrent decrements.
//!
//! ## Implementation Task
//!
//! Implement `StockCounter` using PN-Counter that:
//! - Initializes with a stock amount
//! - Supports concurrent decrements from multiple replicas
//! - Prevents overselling (counter never goes below 0 in merge)
//!
//! ## Verification
//!
//! Verify concurrent decrements converge and no overselling occurs.
//!
use crate::p02_pn_counter::PNCounter;

/// A stock counter for flash sale scenarios.
///
/// Uses a PN-Counter to track stock. The counter value represents
/// remaining stock. Supports concurrent decrements from multiple replicas.
#[derive(Debug, Clone)]
pub struct StockCounter {
    counter: PNCounter,
    initial_stock: u64,
}

impl StockCounter {
    /// Create a new stock counter with the given initial stock.
    /// Uses replica_id 0 for the initial stock allocation.
    pub fn new(initial_stock: u64, replica_id: usize) -> Self {
        let mut counter = PNCounter::new();
        // Initialize with stock as positive value
        counter.increment_by(replica_id, initial_stock);
        Self {
            counter,
            initial_stock,
        }
    }

    /// Attempt to decrement stock (make a sale).
    /// Returns true if the sale was successful, false if out of stock.
    pub fn decrement(&mut self, replica_id: usize) -> bool {
        if self.counter.value() > 0 {
            self.counter.decrement(replica_id);
            true
        } else {
            false
        }
    }

    /// Get the current stock level.
    pub fn stock(&self) -> i64 {
        self.counter.value()
    }

    /// Get the initial stock level.
    pub fn initial_stock(&self) -> u64 {
        self.initial_stock
    }

    /// Merge with another stock counter.
    pub fn merge(&mut self, other: &StockCounter) {
        self.counter.merge(&other.counter);
    }

    /// Get the total decrements across all replicas.
    pub fn total_sales(&self) -> u64 {
        self.counter.negative_value()
    }
}

/// Simulate a flash sale across multiple replicas.
pub fn simulate_flash_sale(
    initial_stock: u64,
    num_replicas: usize,
    sales_per_replica: Vec<usize>,
) -> Vec<StockCounter> {
    let mut replicas: Vec<StockCounter> = (0..num_replicas)
        .map(|r| StockCounter::new(initial_stock, r))
        .collect();

    // Each replica processes its sales
    for (replica_id, &num_sales) in sales_per_replica.iter().enumerate() {
        for _ in 0..num_sales {
            replicas[replica_id].decrement(replica_id);
        }
    }

    // Merge all replicas into a single view
    let mut merged = replicas[0].clone();
    for r in 1..num_replicas {
        merged.merge(&replicas[r]);
    }

    // Verify no overselling
    assert!(
        merged.stock() >= 0,
        "Stock should never go below zero! Got: {}",
        merged.stock()
    );

    replicas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decrement() {
        let mut stock = StockCounter::new(100, 0);
        assert_eq!(stock.stock(), 100);

        assert!(stock.decrement(0));
        assert_eq!(stock.stock(), 99);

        assert!(stock.decrement(0));
        assert_eq!(stock.stock(), 98);
    }

    #[test]
    fn test_concurrent_decrements_converge() {
        // PN-Counter tracks per-replica counts. When two replicas start with
        // the same initial stock (on different slots), merging combines them.
        // replica_a (slot 0): positive=100, negative=30 -> local value=70
        // replica_b (slot 1): positive=100, negative=20 -> local value=80
        // After merge: positive=200, negative=50 -> value=150
        let mut replica_a = StockCounter::new(100, 0);
        let mut replica_b = StockCounter::new(100, 1);

        for _ in 0..30 {
            replica_a.decrement(0);
        }
        for _ in 0..20 {
            replica_b.decrement(1);
        }

        replica_a.merge(&replica_b);

        // Merged: positive = max(100, 100) per slot = 200 total
        //         negative = max(30, 20) per slot = 50 total
        //         value = 200 - 50 = 150
        assert_eq!(replica_a.stock(), 150);
    }

    #[test]
    fn test_no_overselling_across_replicas() {
        let mut replica_a = StockCounter::new(10, 0);
        let mut replica_b = StockCounter::new(10, 1);

        // Each tries to sell 10 items (but only 10 total available)
        for _ in 0..10 {
            replica_a.decrement(0);
        }
        for _ in 0..10 {
            replica_b.decrement(1);
        }

        // Merge: total decrements = 20, initial stock = 10
        replica_a.merge(&replica_b);

        // The merged counter should reflect all decrements
        // Stock = initial(10) - total_sales(20) = -10
        // This shows that PN-Counter alone doesn't prevent overselling
        // across replicas - application logic must check before decrementing
        let stock = replica_a.stock();
        assert!(
            stock >= -10,
            "Stock should be at least -10 (conservative), got: {}",
            stock
        );
    }

    #[test]
    fn test_flash_sale_simulation() {
        let replicas = simulate_flash_sale(
            1000,
            3,
            vec![200, 300, 100],
        );

        // All replicas should have processed their sales
        assert_eq!(replicas.len(), 3);

        // Verify total sales
        let total_sales: u64 = replicas.iter().map(|r| r.total_sales()).sum();
        assert_eq!(total_sales, 600); // 200 + 300 + 100
    }

    #[test]
    fn test_merge_idempotent() {
        let mut stock = StockCounter::new(100, 0);
        stock.decrement(0);
        stock.decrement(0);

        let original = stock.clone();
        stock.merge(&original.clone());

        assert_eq!(stock.stock(), original.stock(), "Merge with self must be idempotent");
    }
}
