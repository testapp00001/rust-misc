//! # Exercise 01: Local Atomic Counter
//!
//! ## Learning Objective
//! Understand how `AtomicI64` provides lock-free thread-safe counters
//! within a single process, and why this approach fails for distributed
//! flash sale systems.
//!
//! ## Flash Sale Context
//! The simplest stock counter is an in-memory atomic integer. When a
//! customer requests an item, we decrement the counter. If it reaches
//! zero, we reject further requests. `AtomicI64` uses CPU-level CAS
//! (Compare-And-Swap) instructions to ensure thread safety without
//! mutexes.
//!
//! However, this counter lives in one process's memory. In a real flash
//! sale with multiple API servers (pods), each pod would have its own
//! counter -- leading to overselling. This exercise demonstrates the
//! local pattern and its limitations.
//!
//! ## Instructions
//! 1. Implement `AtomicStockCounter::new(initial)` to create a counter
//! 2. Implement `decrement()` that atomically subtracts 1 and returns the new value
//! 3. Implement `get()` to read the current stock
//! 4. Ensure stock never goes negative (reject decrements at zero)
//!
//! ## Hints
//! - Use `AtomicI64::new(initial)` to initialize
//! - `fetch_sub` returns the previous value, so check it before subtracting
//! - Or use a compare-exchange loop for the guard
//! - Wrap the counter in `Arc` to share across threads
//!
//! ## Trade-offs
//! - **Pros:** Zero-latency (no network), lock-free, no serialization overhead
//! - **Cons:** Single-process only, lost on restart, no cross-instance consistency
//! - **When to use:** Single-instance apps, local caching layer, unit test scenarios

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

/// Error type for atomic counter operations.
#[derive(Debug, thiserror::Error)]
pub enum CounterError {
    #[error("Stock exhausted: no items remaining")]
    StockExhausted,

    #[error("Invalid initial stock: {0}")]
    InvalidInitialStock(i64),
}

/// An in-memory atomic stock counter.
///
/// Uses `AtomicI64` for lock-free thread-safe decrements within a single
/// process. Wrapping in `Arc` allows sharing across threads.
///
/// ## Why This Doesn't Work for Flash Sales
///
/// In production, you have multiple API servers (pods) each running
/// their own process. Each pod would have its own `AtomicStockCounter`.
/// If you have 100 items and 3 pods, each pod thinks it has 100 items,
/// potentially selling 300 items total. You need a shared counter in
/// Redis (see p02).
pub struct AtomicStockCounter {
    stock: AtomicI64,
}

impl AtomicStockCounter {
    /// Create a new counter with the given initial stock.
    ///
    /// # Arguments
    /// * `initial` - Starting stock quantity (must be >= 0)
    ///
    /// # Returns
    /// A new `AtomicStockCounter` wrapped in `Arc` for thread-safe sharing.
    ///
    /// # Errors
    /// Returns `CounterError::InvalidInitialStock` if initial < 0.
    pub fn new(initial: i64) -> Result<Arc<Self>, CounterError> {
        // TODO: Validate that initial >= 0
        // TODO: Create AtomicI64 with the initial value
        // TODO: Wrap in Arc and return
        todo!("Implement AtomicStockCounter::new")
    }

    /// Atomically decrement stock by 1.
    ///
    /// Uses a compare-exchange loop to ensure the stock never goes negative.
    /// This is the standard pattern for atomic decrement-with-guard:
    ///
    /// ```text
    /// loop {
    ///     current = load();
    ///     if current <= 0 { return StockExhausted; }
    ///     match compare_exchange(current, current - 1) {
    ///         Ok(_) => return current - 1,
    ///         Err(_) => continue, // retry
    ///     }
    /// }
    /// ```
    ///
    /// # Returns
    /// The new stock value after decrement.
    ///
    /// # Errors
    /// Returns `CounterError::StockExhausted` if stock is already 0.
    pub fn decrement(&self) -> Result<i64, CounterError> {
        // TODO: Use a compare-exchange loop:
        //   1. Load current value with SeqCst ordering
        //   2. If <= 0, return StockExhausted
        //   3. Try compare_exchange_weak(current, current - 1)
        //   4. If Ok, return the new value (current - 1)
        //   5. If Err, retry the loop
        todo!("Implement atomic decrement with zero-guard")
    }

    /// Read the current stock value.
    pub fn get(&self) -> i64 {
        // TODO: Load the atomic value with appropriate ordering
        todo!("Implement stock read")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// Test basic creation and reading.
    #[test]
    fn test_basic_creation() {
        let counter = AtomicStockCounter::new(100).expect("Should create counter");
        assert_eq!(counter.get(), 100);
    }

    /// Test that stock decrements correctly.
    #[test]
    fn test_single_decrement() {
        let counter = AtomicStockCounter::new(10).expect("Should create counter");
        let new_val = counter.decrement().expect("Should decrement");
        assert_eq!(new_val, 9);
        assert_eq!(counter.get(), 9);
    }

    /// Test that stock never goes negative -- the fundamental invariant.
    #[test]
    fn test_stock_never_negative() {
        let counter = AtomicStockCounter::new(3).expect("Should create counter");
        counter.decrement().expect("Decrement 1");
        counter.decrement().expect("Decrement 2");
        counter.decrement().expect("Decrement 3");
        let result = counter.decrement();
        assert!(result.is_err(), "Fourth decrement should fail");
        assert_eq!(counter.get(), 0, "Stock should be exactly 0, not negative");
    }

    /// Test concurrent decrements from multiple threads.
    ///
    /// Spawns 10 threads each attempting 200 decrements on a counter
    /// with 1000 items. Exactly 1000 should succeed, 1000 should fail.
    #[test]
    fn test_concurrent_decrements() {
        let initial: i64 = 1000;
        let counter = AtomicStockCounter::new(initial).expect("Should create counter");

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = Arc::clone(&counter);
                thread::spawn(move || {
                    let mut success = 0;
                    for _ in 0..200 {
                        if c.decrement().is_ok() {
                            success += 1;
                        }
                    }
                    success
                })
            })
            .collect();

        let total_success: i64 = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum();

        assert_eq!(
            total_success, initial,
            "Total successful decrements should equal initial stock"
        );
        assert_eq!(counter.get(), 0, "Stock should be exactly 0");
    }

    /// Test that invalid initial stock is rejected.
    #[test]
    fn test_invalid_initial_stock() {
        let result = AtomicStockCounter::new(-1);
        assert!(result.is_err(), "Negative initial stock should be rejected");
    }

    /// Test zero initial stock.
    #[test]
    fn test_zero_initial_stock() {
        let counter = AtomicStockCounter::new(0).expect("Zero should be valid");
        assert_eq!(counter.get(), 0);
        assert!(counter.decrement().is_err(), "Cannot decrement from zero");
    }
}
