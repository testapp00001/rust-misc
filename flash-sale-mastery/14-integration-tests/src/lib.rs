//! # Integration Tests for Flash Sale System
//!
//! This crate contains end-to-end integration tests that verify the flash sale
//! system works correctly under realistic conditions. All tests use in-memory
//! mock implementations (DashMap-based) to avoid external dependencies.
//!
//! ## Test Files
//!
//! - `p01_single_purchase` - Single successful purchase flow
//! - `p02_concurrent_purchases` - Concurrent purchase requests
//! - `p03_oversell_prevention` - Overselling is impossible (CRITICAL)
//! - `p04_duplicate_request` - Idempotency for duplicate requests
//! - `p05_sold_out_handling` - Sold out behavior
//! - `p06_per_account_limit` - Per-account claim limit
//! - `p07_per_product_limit` - Per-product voucher limit
//! - `p08_redis_failure` - Behavior when Redis is unavailable
//! - `p09_db_failure` - Behavior when database is unavailable
//! - `p10_full_load_simulation` - Full flash sale simulation
//! - `p11_reconciliation_test` - Post-sale reconciliation
