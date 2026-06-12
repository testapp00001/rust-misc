//! # Exercise 05: Primary vs Cache Mode Switching
//!
//! ## Learning Objective
//! Implement a stock store that switches its data source based on operating
//! mode: during a flash sale, reads come from Redis (fast, in-memory); after
//! the sale, reads come from the database (durable, authoritative).
//!
//! ## Flash Sale Context
//! During a flash sale, the database cannot handle the read throughput required
//! for stock checks. The solution is to promote Redis to primary store during
//! the sale -- all stock reads and writes go to Redis. After the sale ends,
//! the system switches back to database-primary mode. This exercise simulates
//! the mode-switching logic without requiring an actual Redis connection.
//!
//! ## Instructions
//! 1. Implement `StockStore::new` with initial data for both Redis and DB
//! 2. Implement `StockStore::get_stock` to read from the correct source based on mode
//! 3. Implement `StockStore::update_stock` to write to the correct source
//! 4. Implement `StockStore::switch_to_sale_mode` and `switch_to_normal_mode`
//! 5. Implement `StockStore::current_mode` to report the active mode
//!
//! ## Hints
//! - Use an `AtomicBool` or `Mutex<StoreMode>` for the mode flag
//! - The "Redis" store is simulated with a `DashMap` (no actual Redis needed)
//! - The "DB" store is also a `DashMap` with different data

use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;

/// Error type for stock store operations.
#[derive(Debug, thiserror::Error)]
pub enum StockError {
    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Insufficient stock for product {product_id}: requested {requested}, available {available}")]
    InsufficientStock {
        product_id: String,
        requested: i64,
        available: i64,
    },

    #[error("Store error: {0}")]
    StoreError(String),
}

/// Operating mode for the stock store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    /// Normal mode: database is the source of truth.
    Normal,
    /// Sale mode: Redis (in-memory) is the source of truth.
    Sale,
}

/// Stock store that switches between Redis-primary and DB-primary modes.
pub struct StockStore {
    /// Simulated Redis store (in-memory).
    pub redis_store: DashMap<String, i64>,
    /// Simulated database store (in-memory).
    pub db_store: DashMap<String, i64>,
    /// Current operating mode. true = Sale mode, false = Normal mode.
    sale_mode: AtomicBool,
}

impl StockStore {
    /// Create a new stock store with initial data for both Redis and DB.
    ///
    /// # Arguments
    /// * `redis_data` - Initial stock data for the Redis store
    /// * `db_data` - Initial stock data for the DB store
    pub fn new(
        redis_data: DashMap<String, i64>,
        db_data: DashMap<String, i64>,
    ) -> Self {
        // TODO: Store redis_data and db_data
        // TODO: Initialize sale_mode to false (Normal mode)
        todo!("Implement StockStore::new")
    }

    /// Get the current stock for a product from the active store.
    ///
    /// # Arguments
    /// * `product_id` - The product to look up
    pub fn get_stock(&self, product_id: &str) -> Result<i64, StockError> {
        // TODO: Check current mode
        // TODO: If Sale mode, read from redis_store
        // TODO: If Normal mode, read from db_store
        // TODO: Return StockError::ProductNotFound if not found
        todo!("Implement StockStore::get_stock")
    }

    /// Update stock for a product in the active store.
    ///
    /// # Arguments
    /// * `product_id` - The product to update
    /// * `delta` - The change in stock (negative for decrement)
    pub fn update_stock(
        &self,
        product_id: &str,
        delta: i64,
    ) -> Result<i64, StockError> {
        // TODO: Check current mode
        // TODO: Update the appropriate store
        // TODO: Return the new stock value
        // TODO: Return StockError::InsufficientStock if stock would go negative
        todo!("Implement StockStore::update_stock")
    }

    /// Switch to sale mode (Redis is primary).
    pub fn switch_to_sale_mode(&self) {
        // TODO: Set sale_mode to true
        todo!("Implement StockStore::switch_to_sale_mode")
    }

    /// Switch to normal mode (DB is primary).
    pub fn switch_to_normal_mode(&self) {
        // TODO: Set sale_mode to false
        todo!("Implement StockStore::switch_to_normal_mode")
    }

    /// Get the current operating mode.
    pub fn current_mode(&self) -> StoreMode {
        // TODO: Check sale_mode and return the appropriate StoreMode
        todo!("Implement StockStore::current_mode")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_store() -> StockStore {
        let redis_data = DashMap::new();
        redis_data.insert("product:1001".to_string(), 100);
        redis_data.insert("product:1002".to_string(), 50);

        let db_data = DashMap::new();
        db_data.insert("product:1001".to_string(), 100);
        db_data.insert("product:1002".to_string(), 50);

        StockStore::new(redis_data, db_data)
    }

    #[test]
    fn test_default_mode_is_normal() {
        let store = make_test_store();
        assert_eq!(store.current_mode(), StoreMode::Normal);
    }

    #[test]
    fn test_normal_mode_reads_from_db() {
        let store = make_test_store();
        // Modify Redis to have different values
        store.redis_store.insert("product:1001".to_string(), 999);

        // In Normal mode, should read DB value (100), not Redis value (999)
        let stock = store.get_stock("product:1001").expect("Should find product");
        assert_eq!(stock, 100, "Normal mode should read from DB");
    }

    #[test]
    fn test_sale_mode_reads_from_redis() {
        let store = make_test_store();
        // Modify DB to have different values
        store.db_store.insert("product:1001".to_string(), 999);

        store.switch_to_sale_mode();
        assert_eq!(store.current_mode(), StoreMode::Sale);

        // In Sale mode, should read Redis value (100), not DB value (999)
        let stock = store.get_stock("product:1001").expect("Should find product");
        assert_eq!(stock, 100, "Sale mode should read from Redis");
    }

    #[test]
    fn test_mode_transition() {
        let store = make_test_store();
        assert_eq!(store.current_mode(), StoreMode::Normal);

        store.switch_to_sale_mode();
        assert_eq!(store.current_mode(), StoreMode::Sale);

        store.switch_to_normal_mode();
        assert_eq!(store.current_mode(), StoreMode::Normal);
    }

    #[test]
    fn test_update_stock_in_sale_mode() {
        let store = make_test_store();
        store.switch_to_sale_mode();

        let new_stock = store
            .update_stock("product:1001", -10)
            .expect("Should update");
        assert_eq!(new_stock, 90);

        // Verify it updated Redis, not DB
        let redis_val = store.redis_store.get("product:1001").unwrap();
        assert_eq!(*redis_val, 90);
        let db_val = store.db_store.get("product:1001").unwrap();
        assert_eq!(*db_val, 100, "DB should be unchanged in sale mode");
    }

    #[test]
    fn test_product_not_found() {
        let store = make_test_store();
        let result = store.get_stock("product:nonexistent");
        assert!(result.is_err(), "Should return error for nonexistent product");
    }
}
