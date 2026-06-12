//! # Solution 07: Post-Sale Transition Strategy
//!
//! Complete implementation of post-sale transition: drain, sync, switch.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use dashmap::DashMap;

/// Error type for transition operations.
#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: TransitionState,
        to: TransitionState,
    },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Transition error: {0}")]
    TransitionError(String),
}

/// Report from the Redis drain operation.
#[derive(Debug, Clone, PartialEq)]
pub struct DrainReport {
    pub keys_drained: usize,
    pub errors: usize,
}

/// Report from the DB sync operation.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncReport {
    pub keys_synced: usize,
    pub conflicts: usize,
}

/// States of the transition process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionState {
    /// Normal operation, Redis is primary during sale.
    SaleMode,
    /// Draining data from Redis.
    Draining,
    /// Syncing drained data to the database.
    Syncing,
    /// Transition complete, DB is primary.
    Complete,
}

/// Manages the post-sale transition from Redis-primary to DB-primary.
pub struct TransitionManager {
    /// Simulated Redis store.
    pub redis_store: DashMap<String, i64>,
    /// Simulated DB store.
    pub db_store: DashMap<String, i64>,
    /// Current transition state.
    state: Mutex<TransitionState>,
    /// Number of in-flight requests.
    in_flight: AtomicUsize,
    /// Drained data buffer.
    drained: DashMap<String, i64>,
}

impl TransitionManager {
    /// Create a new transition manager.
    pub fn new(
        redis_data: DashMap<String, i64>,
        db_data: DashMap<String, i64>,
    ) -> Self {
        Self {
            redis_store: redis_data,
            db_store: db_data,
            state: Mutex::new(TransitionState::SaleMode),
            in_flight: AtomicUsize::new(0),
            drained: DashMap::new(),
        }
    }

    /// Get the current transition state.
    pub fn current_state(&self) -> TransitionState {
        *self.state.lock().unwrap()
    }

    /// Increment the in-flight request counter.
    pub fn in_flight_increment(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement the in-flight request counter.
    pub fn in_flight_decrement(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get the number of in-flight requests.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }

    /// Drain all data from Redis into an internal buffer.
    pub fn drain_redis(&self) -> Result<DrainReport, TransitionError> {
        let mut state = self.state.lock().unwrap();
        if *state != TransitionState::SaleMode {
            return Err(TransitionError::InvalidTransition {
                from: *state,
                to: TransitionState::Draining,
            });
        }
        *state = TransitionState::Draining;

        // Copy all Redis entries to the drained buffer
        let mut count = 0;
        for entry in self.redis_store.iter() {
            self.drained.insert(entry.key().clone(), *entry.value());
            count += 1;
        }

        // Transition to Syncing state
        *state = TransitionState::Syncing;

        Ok(DrainReport {
            keys_drained: count,
            errors: 0,
        })
    }

    /// Sync drained data to the database.
    pub fn sync_to_db(&self) -> Result<SyncReport, TransitionError> {
        let state = self.state.lock().unwrap();
        if *state != TransitionState::Syncing {
            return Err(TransitionError::InvalidTransition {
                from: *state,
                to: TransitionState::Syncing,
            });
        }
        drop(state);

        let mut keys_synced = 0;
        let mut conflicts = 0;

        for entry in self.drained.iter() {
            let key = entry.key();
            let redis_val = *entry.value();

            // Check if key exists in DB (conflict)
            if self.db_store.contains_key(key) {
                conflicts += 1;
            }

            // Redis value wins (it's more recent)
            self.db_store.insert(key.clone(), redis_val);
            keys_synced += 1;
        }

        Ok(SyncReport {
            keys_synced,
            conflicts,
        })
    }

    /// Complete the transition: switch to DB-primary mode.
    pub fn complete_transition(&self) -> Result<(), TransitionError> {
        let mut state = self.state.lock().unwrap();
        if *state != TransitionState::Syncing {
            return Err(TransitionError::InvalidTransition {
                from: *state,
                to: TransitionState::Complete,
            });
        }
        *state = TransitionState::Complete;
        Ok(())
    }

    /// Get stock for a product, respecting the current transition state.
    pub fn get_stock(&self, key: &str) -> Result<i64, TransitionError> {
        let state = *self.state.lock().unwrap();

        let store = match state {
            TransitionState::SaleMode
            | TransitionState::Draining
            | TransitionState::Syncing => &self.redis_store,
            TransitionState::Complete => &self.db_store,
        };

        store
            .get(key)
            .map(|v| *v)
            .ok_or_else(|| TransitionError::KeyNotFound(key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_manager() -> TransitionManager {
        let redis_data = DashMap::new();
        redis_data.insert("product:1001".to_string(), 100);
        redis_data.insert("product:1002".to_string(), 50);

        let db_data = DashMap::new();
        db_data.insert("product:1001".to_string(), 100);
        db_data.insert("product:1002".to_string(), 50);

        TransitionManager::new(redis_data, db_data)
    }

    #[test]
    fn test_initial_state_is_sale_mode() {
        let mgr = make_test_manager();
        assert_eq!(mgr.current_state(), TransitionState::SaleMode);
    }

    #[test]
    fn test_clean_transition() {
        let mgr = make_test_manager();

        // Step 1: Drain Redis
        let drain = mgr.drain_redis().expect("Drain should succeed");
        assert_eq!(drain.keys_drained, 2);
        assert_eq!(drain.errors, 0);
        assert_eq!(mgr.current_state(), TransitionState::Syncing);

        // Step 2: Sync to DB
        let sync = mgr.sync_to_db().expect("Sync should succeed");
        assert_eq!(sync.keys_synced, 2);
        // Both keys exist in DB with same values, so both are conflicts
        assert_eq!(sync.conflicts, 2);

        // Step 3: Complete transition
        mgr.complete_transition()
            .expect("Complete should succeed");
        assert_eq!(mgr.current_state(), TransitionState::Complete);

        // After transition, should read from DB
        let stock = mgr.get_stock("product:1001").expect("Should find product");
        assert_eq!(stock, 100);
    }

    #[test]
    fn test_transition_with_conflicts() {
        let redis_data = DashMap::new();
        redis_data.insert("product:1001".to_string(), 80); // Lower stock after sale

        let db_data = DashMap::new();
        db_data.insert("product:1001".to_string(), 100); // Original stock

        let mgr = TransitionManager::new(redis_data, db_data);

        mgr.drain_redis().expect("Drain should succeed");
        let sync = mgr.sync_to_db().expect("Sync should succeed");

        // Redis value (80) should win over DB value (100)
        assert_eq!(sync.conflicts, 1, "Should detect 1 conflict");
        let db_val = mgr.db_store.get("product:1001").unwrap();
        assert_eq!(*db_val, 80, "Redis value should overwrite DB value");
    }

    #[test]
    fn test_get_stock_respects_state() {
        let redis_data = DashMap::new();
        redis_data.insert("product:1001".to_string(), 80);

        let db_data = DashMap::new();
        db_data.insert("product:1001".to_string(), 100);

        let mgr = TransitionManager::new(redis_data, db_data);

        // SaleMode: reads from Redis
        let stock = mgr.get_stock("product:1001").expect("Should find");
        assert_eq!(stock, 80, "SaleMode should read from Redis");

        // Complete transition
        mgr.drain_redis().expect("Drain");
        mgr.sync_to_db().expect("Sync");
        mgr.complete_transition().expect("Complete");

        // Complete: reads from DB (which now has the synced value)
        let stock = mgr.get_stock("product:1001").expect("Should find");
        assert_eq!(stock, 80, "Complete should read from DB");
    }

    #[test]
    fn test_in_flight_tracking() {
        let mgr = make_test_manager();
        assert_eq!(mgr.in_flight_count(), 0);

        mgr.in_flight_increment();
        mgr.in_flight_increment();
        assert_eq!(mgr.in_flight_count(), 2);

        mgr.in_flight_decrement();
        assert_eq!(mgr.in_flight_count(), 1);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mgr = make_test_manager();
        // Cannot sync without draining first
        let result = mgr.sync_to_db();
        assert!(result.is_err(), "Should fail when not in Syncing state");
    }
}
