//! # Module 11: Integration Exercises
//!
//! This module contains seven integration exercises that combine concepts
//! from all previous modules into practical distributed systems implementations.
//!
//! Each exercise builds on previous modules and demonstrates how the
//! theoretical foundations translate into working code.
//!
//! ## Exercise List
//!
//! - `p01_distributed_kv_with_raft` - Distributed KV store with Raft consensus
//! - `p02_eventually_consistent_store` - Eventually consistent store with vector clocks
//! - `p03_crdt_replicated_shopping_cart` - Shopping cart using CRDT (OR-Set)
//! - `p04_saga_based_order_system` - Order system with Saga pattern
//! - `p05_sharded_cache_with_consistent_hashing` - Sharded cache with consistent hashing
//! - `p06_pbft_key_value_store` - KV store with PBFT consensus
//! - `p07_full_distributed_system` - Full system combining all concepts

pub mod p01_distributed_kv_with_raft;
pub mod p02_eventually_consistent_store;
pub mod p03_crdt_replicated_shopping_cart;
pub mod p04_saga_based_order_system;
pub mod p05_sharded_cache_with_consistent_hashing;
pub mod p06_pbft_key_value_store;
pub mod p07_full_distributed_system;
