//! # Module 13: Reconciliation
//!
//! Reconcile the fast-path (Redis) with the durable store (PostgreSQL)
//! after a flash sale. Detect discrepancies, replay events to rebuild
//! state, correct mismatches, and generate structured reports.
//!
//! ## Module Structure
//! - `shared` - Common types used across all exercises
//! - `p01_inventory_sync` - Sync inventory counts between Redis and DB
//! - `p02_event_replay` - Replay events to rebuild and verify state
//! - `p03_discrepancy_detector` - Detect inconsistencies across systems
//! - `p04_report_generator` - Generate reconciliation reports

pub mod shared;

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_inventory_sync;
#[cfg(not(feature = "solution"))]
pub mod p02_event_replay;
#[cfg(not(feature = "solution"))]
pub mod p03_discrepancy_detector;
#[cfg(not(feature = "solution"))]
pub mod p04_report_generator;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_inventory_sync.rs"]
pub mod p01_inventory_sync;
#[cfg(feature = "solution")]
#[path = "solution/p02_event_replay.rs"]
pub mod p02_event_replay;
#[cfg(feature = "solution")]
#[path = "solution/p03_discrepancy_detector.rs"]
pub mod p03_discrepancy_detector;
#[cfg(feature = "solution")]
#[path = "solution/p04_report_generator.rs"]
pub mod p04_report_generator;
