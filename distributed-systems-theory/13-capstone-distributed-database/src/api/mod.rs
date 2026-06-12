//! Client-facing API module with GET, PUT, and DELETE handlers.
//!
//! Each handler operates on a shared `DatabaseState` that wraps the storage engine
//! and Hybrid Logical Clock behind `Arc<Mutex<...>>`, enabling safe concurrent access.

pub mod get;
pub mod put;
pub mod delete;

use std::sync::{Arc, Mutex};

use crate::clock::hlc::HLC;
use crate::storage::engine::StorageEngine;

/// Shared database state used by all API handlers.
///
/// Wraps the storage engine and HLC in `Arc<Mutex<...>>` so that handlers can
/// operate concurrently while maintaining mutual exclusion on the underlying state.
pub struct DatabaseState {
    /// The key-value storage engine.
    pub engine: Arc<Mutex<StorageEngine>>,
    /// The hybrid logical clock for timestamping operations.
    pub hlc: Arc<Mutex<HLC>>,
}

impl DatabaseState {
    /// Create a new `DatabaseState` for the given node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node, used to initialize the HLC.
    pub fn new(node_id: u64) -> Self {
        Self {
            engine: Arc::new(Mutex::new(StorageEngine::new())),
            hlc: Arc::new(Mutex::new(HLC::new(node_id))),
        }
    }
}
