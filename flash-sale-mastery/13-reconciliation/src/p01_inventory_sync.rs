//! # Exercise 01: Inventory Synchronization
//!
//! ## Learning Objective
//! Implement the core reconciliation logic that compares Redis inventory counts
//! against PostgreSQL and determines what corrective action to take.
//!
//! ## Flash Sale Context
//! During a flash sale, Redis is the authoritative inventory source for speed.
//! Every purchase decrements the Redis counter. After the sale ends, we need to
//! sync those counts back to PostgreSQL for durable storage. Sometimes Redis and
//! DB disagree -- this exercise teaches you how to handle each case safely.
//!
//! ## Instructions
//! 1. Implement `InventorySync::new()` to hold references to both data stores.
//! 2. Implement `sync_product()` which compares a single product's stock:
//!    - If Redis == DB: return `SyncAction::NoOp`
//!    - If Redis > DB: return `SyncAction::UpdatedDb` (Redis is authoritative)
//!    - If Redis < DB: return `SyncAction::AlertRaised` (possible data loss)
//! 3. Implement `sync_all()` to synchronize every product present in either store.
//! 4. Implement `apply_corrections()` to actually update the DB where needed.
//!
//! ## Hints
//! - The rule is: during a flash sale, Redis wins. If Redis has more stock
//!   (fewer decrements recorded), something is wrong with Redis.
//! - Use `tracing::warn!` for products that fail to sync so others can continue.
//! - `apply_corrections` should only act on `SyncAction::UpdatedDb` results.

use chrono::Utc;
use std::collections::HashMap;

use crate::shared::{
    DbInventory, ReconciliationError, RedisInventory, SyncAction, SyncResult,
};

/// Simulated Redis data source for testing.
pub struct MockRedisStore {
    pub inventories: HashMap<String, RedisInventory>,
}

impl MockRedisStore {
    pub fn new() -> Self {
        Self {
            inventories: HashMap::new(),
        }
    }

    pub fn insert(&mut self, inv: RedisInventory) {
        self.inventories.insert(inv.product_id.clone(), inv);
    }

    pub fn get(&self, product_id: &str) -> Option<RedisInventory> {
        self.inventories.get(product_id).cloned()
    }

    pub fn get_all(&self) -> Vec<RedisInventory> {
        self.inventories.values().cloned().collect()
    }

    pub fn update_stock(&mut self, product_id: &str, new_stock: i64) {
        if let Some(inv) = self.inventories.get_mut(product_id) {
            inv.available_stock = new_stock;
            inv.last_updated = Utc::now();
            inv.version += 1;
        }
    }
}

/// Simulated PostgreSQL data source for testing.
pub struct MockDbStore {
    pub inventories: HashMap<String, DbInventory>,
}

impl MockDbStore {
    pub fn new() -> Self {
        Self {
            inventories: HashMap::new(),
        }
    }

    pub fn insert(&mut self, inv: DbInventory) {
        self.inventories.insert(inv.product_id.clone(), inv);
    }

    pub fn get(&self, product_id: &str) -> Option<DbInventory> {
        self.inventories.get(product_id).cloned()
    }

    pub fn get_all(&self) -> Vec<DbInventory> {
        self.inventories.values().cloned().collect()
    }

    pub fn update_stock(&mut self, product_id: &str, new_stock: i64) {
        if let Some(inv) = self.inventories.get_mut(product_id) {
            inv.available_stock = new_stock;
            inv.last_updated = Utc::now();
            inv.version += 1;
        }
    }
}

/// Handles synchronization of inventory between Redis and PostgreSQL.
pub struct InventorySync {
    // TODO: add fields for the mock stores
}

impl InventorySync {
    /// Create a new InventorySync with the given data stores.
    pub fn new(redis: MockRedisStore, db: MockDbStore) -> Self {
        todo!("Store references to both data stores")
    }

    /// Synchronize inventory for a single product.
    ///
    /// Compare the stock levels and determine the correct action:
    /// - Equal stocks: NoOp
    /// - Redis ahead (higher stock): UpdatedDb
    /// - DB ahead (higher stock): AlertRaised
    pub fn sync_product(&self, product_id: &str) -> Result<SyncResult, ReconciliationError> {
        todo!("Compare Redis and DB stock for the given product and return the appropriate SyncResult")
    }

    /// Synchronize all products found in both Redis and DB.
    pub fn sync_all(&self) -> Result<Vec<SyncResult>, ReconciliationError> {
        todo!("Collect all product IDs from both stores and sync each one")
    }

    /// Apply corrections to the DB for all sync results that require updating.
    /// Returns the number of corrections applied.
    pub fn apply_corrections(
        &mut self,
        sync_results: &[SyncResult],
    ) -> Result<usize, ReconciliationError> {
        todo!("Update DB stock for each result with action UpdatedDb")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{DbInventory, RedisInventory};
    use chrono::Utc;

    fn make_redis_inv(product_id: &str, stock: i64) -> RedisInventory {
        RedisInventory {
            product_id: product_id.to_string(),
            available_stock: stock,
            reserved: 0,
            version: 1,
            last_updated: Utc::now(),
        }
    }

    fn make_db_inv(product_id: &str, stock: i64) -> DbInventory {
        DbInventory {
            product_id: product_id.to_string(),
            available_stock: stock,
            reserved: 0,
            total_sold: 0,
            version: 1,
            last_updated: Utc::now(),
        }
    }

    #[test]
    fn test_in_sync_returns_noop() {
        let mut redis = MockRedisStore::new();
        let mut db = MockDbStore::new();

        redis.insert(make_redis_inv("SKU-1", 100));
        db.insert(make_db_inv("SKU-1", 100));

        let sync = InventorySync::new(redis, db);
        let result = sync.sync_product("SKU-1").unwrap();

        assert!(!result.mismatch);
        assert_eq!(result.action_taken, SyncAction::NoOp);
        assert_eq!(result.redis_stock, 100);
        assert_eq!(result.db_stock, 100);
    }

    #[test]
    fn test_redis_ahead_updates_db() {
        let mut redis = MockRedisStore::new();
        let mut db = MockDbStore::new();

        redis.insert(make_redis_inv("SKU-2", 100));
        db.insert(make_db_inv("SKU-2", 80));

        let sync = InventorySync::new(redis, db);
        let result = sync.sync_product("SKU-2").unwrap();

        assert!(result.mismatch);
        assert_eq!(result.action_taken, SyncAction::UpdatedDb);
    }

    #[test]
    fn test_db_ahead_raises_alert() {
        let mut redis = MockRedisStore::new();
        let mut db = MockDbStore::new();

        redis.insert(make_redis_inv("SKU-3", 50));
        db.insert(make_db_inv("SKU-3", 100));

        let sync = InventorySync::new(redis, db);
        let result = sync.sync_product("SKU-3").unwrap();

        assert!(result.mismatch);
        match &result.action_taken {
            SyncAction::AlertRaised { reason } => {
                assert!(reason.contains("data loss"));
            }
            _ => panic!("Expected AlertRaised action"),
        }
    }

    #[test]
    fn test_sync_corrects_after_apply() {
        let mut redis = MockRedisStore::new();
        let mut db = MockDbStore::new();

        redis.insert(make_redis_inv("SKU-4", 100));
        db.insert(make_db_inv("SKU-4", 80));

        let mut sync = InventorySync::new(redis, db);
        let results = sync.sync_product("SKU-4").unwrap();
        let corrections = sync.apply_corrections(&[results]).unwrap();

        assert_eq!(corrections, 1);

        // Verify DB now matches Redis.
        let db_inv = sync.db.get("SKU-4").unwrap();
        assert_eq!(db_inv.available_stock, 100);
    }

    #[test]
    fn test_missing_product_returns_error() {
        let redis = MockRedisStore::new();
        let db = MockDbStore::new();

        let sync = InventorySync::new(redis, db);
        let result = sync.sync_product("NONEXISTENT");

        assert!(result.is_err());
    }

    #[test]
    fn test_sync_all_multiple_products() {
        let mut redis = MockRedisStore::new();
        let mut db = MockDbStore::new();

        redis.insert(make_redis_inv("A", 100));
        redis.insert(make_redis_inv("B", 50));
        redis.insert(make_redis_inv("C", 200));
        db.insert(make_db_inv("A", 100));
        db.insert(make_db_inv("B", 30));
        db.insert(make_db_inv("C", 200));

        let sync = InventorySync::new(redis, db);
        let results = sync.sync_all().unwrap();

        assert_eq!(results.len(), 3);

        let a = results.iter().find(|r| r.product_id == "A").unwrap();
        assert!(!a.mismatch);

        let b = results.iter().find(|r| r.product_id == "B").unwrap();
        assert!(b.mismatch);
        assert_eq!(b.action_taken, SyncAction::UpdatedDb);

        let c = results.iter().find(|r| r.product_id == "C").unwrap();
        assert!(!c.mismatch);
    }
}
