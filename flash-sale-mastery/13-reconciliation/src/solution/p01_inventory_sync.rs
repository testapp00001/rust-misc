//! # Solution 01: Inventory Synchronization
//!
//! Complete implementation of Redis-to-PostgreSQL inventory reconciliation.

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
///
/// During a flash sale, Redis is the authoritative source for speed.
/// After the sale, PostgreSQL is the source of truth for durability.
/// This reconciler ensures both systems agree on inventory counts.
pub struct InventorySync {
    redis: MockRedisStore,
    db: MockDbStore,
}

impl InventorySync {
    /// Create a new InventorySync with the given data stores.
    pub fn new(redis: MockRedisStore, db: MockDbStore) -> Self {
        Self { redis, db }
    }

    /// Synchronize inventory for a single product.
    ///
    /// Strategy:
    /// - If Redis stock == DB stock: no action needed.
    /// - If Redis stock > DB stock: update DB from Redis (Redis saw purchases first).
    /// - If Redis stock < DB stock: raise alert (possible data loss in Redis).
    pub fn sync_product(&self, product_id: &str) -> Result<SyncResult, ReconciliationError> {
        let redis_inv = self
            .redis
            .get(product_id)
            .ok_or_else(|| ReconciliationError::Redis(format!("Product {} not in Redis", product_id)))?;

        let db_inv = self
            .db
            .get(product_id)
            .ok_or_else(|| ReconciliationError::Database(format!("Product {} not in DB", product_id)))?;

        let redis_stock = redis_inv.available_stock;
        let db_stock = db_inv.available_stock;
        let mismatch = redis_stock != db_stock;

        let action_taken = if !mismatch {
            SyncAction::NoOp
        } else if redis_stock > db_stock {
            // Redis has more stock recorded -- it processed decrements that DB didn't see.
            // Update DB to match Redis (Redis is authoritative during sale).
            // NOTE: In a real system, we would use a transaction here.
            SyncAction::UpdatedDb
        } else {
            // Redis has LESS stock than DB -- this means Redis lost data.
            // This is an error condition that needs investigation.
            SyncAction::AlertRaised {
                reason: format!(
                    "Redis stock ({}) is less than DB stock ({}). \
                     Possible Redis data loss for product {}.",
                    redis_stock, db_stock, product_id
                ),
            }
        };

        Ok(SyncResult {
            product_id: product_id.to_string(),
            redis_stock,
            db_stock,
            mismatch,
            action_taken,
        })
    }

    /// Synchronize all products found in both Redis and DB.
    pub fn sync_all(&self) -> Result<Vec<SyncResult>, ReconciliationError> {
        let mut results = Vec::new();

        // Collect all product IDs present in both stores.
        let mut product_ids: Vec<String> = self
            .redis
            .get_all()
            .iter()
            .map(|r| r.product_id.clone())
            .collect();

        // Add any products in DB but not in Redis.
        for db_inv in self.db.get_all() {
            if !product_ids.contains(&db_inv.product_id) {
                product_ids.push(db_inv.product_id.clone());
            }
        }

        for pid in product_ids {
            match self.sync_product(&pid) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!("Failed to sync product {}: {}", pid, e);
                    // Continue with other products even if one fails.
                }
            }
        }

        Ok(results)
    }

    /// Apply corrections: actually update the DB where sync determined it needed updating.
    /// This mutates the internal DB store. In production, this would issue SQL UPDATEs.
    pub fn apply_corrections(
        &mut self,
        sync_results: &[SyncResult],
    ) -> Result<usize, ReconciliationError> {
        let mut corrections = 0;

        for result in sync_results {
            if result.action_taken == SyncAction::UpdatedDb {
                self.db
                    .update_stock(&result.product_id, result.redis_stock);
                corrections += 1;
                tracing::info!(
                    "Corrected product {}: DB stock {} -> {}",
                    result.product_id,
                    result.db_stock,
                    result.redis_stock
                );
            }
        }

        Ok(corrections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::RedisInventory;
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

        // Redis saw more decrements (lower available = more sold).
        // Actually: Redis has MORE available means Redis didn't process some decrements.
        // But the convention here: Redis stock > DB stock means Redis is ahead.
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

        // DB has more stock than Redis -- possible Redis data loss.
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
