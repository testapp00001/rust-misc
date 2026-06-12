//! # Solution 01: Local Atomic Counter
//!
//! Complete implementation of an in-memory atomic stock counter using `AtomicI64`.

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
/// ## Trade-offs
/// - **Pros:** Zero-latency (no network), lock-free, no serialization overhead
/// - **Cons:** Single-process only, lost on restart, no cross-instance consistency
/// - **When to use:** Single-instance apps, local caching layer, unit test scenarios
pub struct AtomicStockCounter {
    stock: AtomicI64,
}

impl AtomicStockCounter {
    /// Create a new counter with the given initial stock.
    pub fn new(initial: i64) -> Result<Arc<Self>, CounterError> {
        if initial < 0 {
            return Err(CounterError::InvalidInitialStock(initial));
        }
        Ok(Arc::new(Self {
            stock: AtomicI64::new(initial),
        }))
    }

    /// Atomically decrement stock by 1. Returns the new stock value.
    ///
    /// Uses a compare-exchange loop to ensure the stock never goes negative.
    /// `compare_exchange_weak` is used because we're already in a loop --
    /// it can have spurious failures on ARM but is faster in a retry loop.
    pub fn decrement(&self) -> Result<i64, CounterError> {
        loop {
            let current = self.stock.load(Ordering::SeqCst);
            if current <= 0 {
                return Err(CounterError::StockExhausted);
            }
            match self.stock.compare_exchange_weak(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Ok(current - 1),
                Err(_) => continue, // Another thread changed it, retry
            }
        }
    }

    /// Read the current stock value.
    pub fn get(&self) -> i64 {
        self.stock.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_creation() {
        let counter = AtomicStockCounter::new(100).expect("Should create counter");
        assert_eq!(counter.get(), 100);
    }

    #[test]
    fn test_single_decrement() {
        let counter = AtomicStockCounter::new(10).expect("Should create counter");
        let new_val = counter.decrement().expect("Should decrement");
        assert_eq!(new_val, 9);
        assert_eq!(counter.get(), 9);
    }

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

    #[test]
    fn test_invalid_initial_stock() {
        let result = AtomicStockCounter::new(-1);
        assert!(result.is_err(), "Negative initial stock should be rejected");
    }

    #[test]
    fn test_zero_initial_stock() {
        let counter = AtomicStockCounter::new(0).expect("Zero should be valid");
        assert_eq!(counter.get(), 0);
        assert!(counter.decrement().is_err(), "Cannot decrement from zero");
    }
}
