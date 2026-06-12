//! # Exercise 07: Post-Sale Transition Strategy
//!
//! ## Learning Objective
//! Implement the post-sale transition: drain Redis data, sync it to the
//! database, and switch the system back to normal-mode operation. Handle
//! in-flight requests during the transition gracefully.
//!
//! ## Flash Sale Context
//! After a flash sale ends, the system must transition from "Redis as primary
//! store" back to "database as primary store." This is a multi-step process:
//! (1) drain remaining data from Redis, (2) sync it to the database, resolving
//! conflicts, (3) switch the read path to the database. During this transition,
//! in-flight requests must still be served correctly.
//!
//! ## Instructions
//! 1. Implement `TransitionManager::new` with Redis and DB data stores
//! 2. Implement `TransitionManager::drain_redis` to read all data from Redis
//! 3. Implement `TransitionManager::sync_to_db` to write drained data to DB
//! 4. Implement `TransitionManager::complete_transition` to switch mode
//! 5. Implement `TransitionManager::get_stock` that respects the current state
//! 6. Implement `in_flight_increment` and `in_flight_decrement` for tracking
//!
//! ## Hints
//! - Use `Mutex<TransitionState>` to track the current phase
//! - Use `AtomicUsize` for lock-free in-flight request counting
//! - The drain step should collect all Redis entries into a Vec

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
    ///
    /// # Arguments
    /// * `redis_data` - Data currently in Redis
    /// * `db_data` - Data currently in the database
    pub fn new(
        redis_data: DashMap<String, i64>,
        db_data: DashMap<String, i64>,
    ) -> Self {
        // TODO: Initialize all fields
        // TODO: State should start as SaleMode
        todo!("Implement TransitionManager::new")
    }

    /// Get the current transition state.
    pub fn current_state(&self) -> TransitionState {
        // TODO: Lock the state mutex and return the current state
        todo!("Implement TransitionManager::current_state")
    }

    /// Increment the in-flight request counter.
    pub fn in_flight_increment(&self) {
        // TODO: Increment the AtomicUsize
        todo!("Implement TransitionManager::in_flight_increment")
    }

    /// Decrement the in-flight request counter.
    pub fn in_flight_decrement(&self) {
        // TODO: Decrement the AtomicUsize
        todo!("Implement TransitionManager::in_flight_decrement")
    }

    /// Get the number of in-flight requests.
    pub fn in_flight_count(&self) -> usize {
        // TODO: Load the AtomicUsize value
        todo!("Implement TransitionManager::in_flight_count")
    }

    /// Drain all data from Redis into an internal buffer.
    ///
    /// This reads all entries from the Redis store and prepares them for
    /// syncing to the database. Must be called from SaleMode state.
    pub fn drain_redis(&self) -> Result<DrainReport, TransitionError> {
        // TODO: Check that current state is SaleMode
        // TODO: Transition state to Draining
        // TODO: Copy all Redis entries to the drained buffer
        // TODO: Return DrainReport with count of drained keys
        // TODO: Transition state to Syncing (or handle errors)
        todo!("Implement TransitionManager::drain_redis")
    }

    /// Sync drained data to the database.
    ///
    /// This writes all drained entries to the DB store. If a key already
    /// exists in the DB, the Redis value wins (it's more recent).
    /// Must be called from Syncing state.
    pub fn sync_to_db(&self) -> Result<SyncReport, TransitionError> {
        // TODO: Check that current state is Syncing
        // TODO: For each entry in drained buffer:
        // TODO:   - Check if key exists in DB (conflict)
        // TODO:   - Overwrite DB value with Redis value
        // TODO: Return SyncReport with counts
        todo!("Implement TransitionManager::sync_to_db")
    }

    /// Complete the transition: switch to DB-primary mode.
    ///
    /// Must be called from Syncing state after sync_to_db.
    pub fn complete_transition(&self) -> Result<(), TransitionError> {
        // TODO: Check that current state is Syncing
        // TODO: Transition state to Complete
        todo!("Implement TransitionManager::complete_transition")
    }

    /// Get stock for a product, respecting the current transition state.
    ///
    /// During SaleMode, reads from Redis. During Complete, reads from DB.
    /// During Draining/Syncing, reads from Redis (still authoritative).
    pub fn get_stock(&self, key: &str) -> Result<i64, TransitionError> {
        // TODO: Check current state
        // TODO: If SaleMode or Draining or Syncing, read from redis_store
        // TODO: If Complete, read from db_store
        // TODO: Return KeyNotFound if not found
        todo!("Implement TransitionManager::get_stock")
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
