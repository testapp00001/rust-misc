//! # Solution 05: Primary vs Cache Mode Switching
//!
//! Complete implementation of stock store with mode-based source switching.

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
    pub fn new(
        redis_data: DashMap<String, i64>,
        db_data: DashMap<String, i64>,
    ) -> Self {
        Self {
            redis_store: redis_data,
            db_store: db_data,
            sale_mode: AtomicBool::new(false),
        }
    }

    /// Get the current stock for a product from the active store.
    pub fn get_stock(&self, product_id: &str) -> Result<i64, StockError> {
        let store = if self.sale_mode.load(Ordering::SeqCst) {
            &self.redis_store
        } else {
            &self.db_store
        };

        store
            .get(product_id)
            .map(|v| *v)
            .ok_or_else(|| StockError::ProductNotFound(product_id.to_string()))
    }

    /// Update stock for a product in the active store.
    pub fn update_stock(
        &self,
        product_id: &str,
        delta: i64,
    ) -> Result<i64, StockError> {
        let store = if self.sale_mode.load(Ordering::SeqCst) {
            &self.redis_store
        } else {
            &self.db_store
        };

        store
            .get_mut(product_id)
            .map(|mut v| {
                let new_val = *v + delta;
                if new_val < 0 {
                    return Err(StockError::InsufficientStock {
                        product_id: product_id.to_string(),
                        requested: -delta,
                        available: *v,
                    });
                }
                *v = new_val;
                Ok(new_val)
            })
            .unwrap_or_else(|| Err(StockError::ProductNotFound(product_id.to_string())))
    }

    /// Switch to sale mode (Redis is primary).
    pub fn switch_to_sale_mode(&self) {
        self.sale_mode.store(true, Ordering::SeqCst);
    }

    /// Switch to normal mode (DB is primary).
    pub fn switch_to_normal_mode(&self) {
        self.sale_mode.store(false, Ordering::SeqCst);
    }

    /// Get the current operating mode.
    pub fn current_mode(&self) -> StoreMode {
        if self.sale_mode.load(Ordering::SeqCst) {
            StoreMode::Sale
        } else {
            StoreMode::Normal
        }
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
