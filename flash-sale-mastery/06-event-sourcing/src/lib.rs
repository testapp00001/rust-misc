//! # Event Sourcing for Flash Sale Inventory
//!
//! This module teaches event sourcing and audit trails through the lens of
//! flash sale inventory tracking. Instead of storing only the current state,
//! every change is recorded as an immutable event. The current state can
//! always be rebuilt by replaying the event log.
//!
//! ## Module Structure
//!
//! - `p01_event_design` - Domain event types and serialization
//! - `p02_event_store` - In-memory append-only event store
//! - `p03_event_replay` - Rebuilding state from events
//! - `p04_redis_streams` - Publishing events to Redis Streams
//! - `p05_consumer_groups` - Consumer groups and acknowledgment
//! - `p06_reconciliation_job` - Detecting state drift via event replay
//! - `p07_audit_query` - Querying events for audit purposes
//! - `p08_event_versioning` - Schema evolution and backward compatibility

// Exercise stubs (used when "solution" feature is NOT enabled)
#[cfg(not(feature = "solution"))]
pub mod p01_event_design;
#[cfg(not(feature = "solution"))]
pub mod p02_event_store;
#[cfg(not(feature = "solution"))]
pub mod p03_event_replay;
#[cfg(not(feature = "solution"))]
pub mod p04_redis_streams;
#[cfg(not(feature = "solution"))]
pub mod p05_consumer_groups;
#[cfg(not(feature = "solution"))]
pub mod p06_reconciliation_job;
#[cfg(not(feature = "solution"))]
pub mod p07_audit_query;
#[cfg(not(feature = "solution"))]
pub mod p08_event_versioning;

// Solution implementations (used when "solution" feature IS enabled)
#[cfg(feature = "solution")]
#[path = "solution/p01_event_design.rs"]
pub mod p01_event_design;
#[cfg(feature = "solution")]
#[path = "solution/p02_event_store.rs"]
pub mod p02_event_store;
#[cfg(feature = "solution")]
#[path = "solution/p03_event_replay.rs"]
pub mod p03_event_replay;
#[cfg(feature = "solution")]
#[path = "solution/p04_redis_streams.rs"]
pub mod p04_redis_streams;
#[cfg(feature = "solution")]
#[path = "solution/p05_consumer_groups.rs"]
pub mod p05_consumer_groups;
#[cfg(feature = "solution")]
#[path = "solution/p06_reconciliation_job.rs"]
pub mod p06_reconciliation_job;
#[cfg(feature = "solution")]
#[path = "solution/p07_audit_query.rs"]
pub mod p07_audit_query;
#[cfg(feature = "solution")]
#[path = "solution/p08_event_versioning.rs"]
pub mod p08_event_versioning;
