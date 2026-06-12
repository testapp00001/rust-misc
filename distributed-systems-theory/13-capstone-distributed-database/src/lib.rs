//! # Capstone Distributed Database
//!
//! A simplified distributed database combining all concepts from the distributed systems
//! theory curriculum: consensus, storage, replication, sharding, transactions, networking,
//! CRDTs, logical clocks, and client APIs.

/// Consensus protocol implementation (simplified Raft).
pub mod consensus;
/// Storage engine and write-ahead log.
pub mod storage;
/// Log replication between leader and followers.
pub mod replication;
/// Consistent hashing and shard rebalancing.
pub mod sharding;
/// Saga transactions and isolation levels.
pub mod transactions;
/// RPC framework and failure detection.
pub mod networking;
/// Conflict-free replicated data types.
pub mod crdt;
/// Hybrid Logical Clock.
pub mod clock;
/// Client-facing GET, PUT, DELETE API handlers.
pub mod api;
