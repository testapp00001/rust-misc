//! # Redis Lua Scripting for Flash Sales
//!
//! This module teaches Redis Lua scripting for atomic multi-step operations
//! in a flash sale system. Lua scripts execute atomically inside Redis's
//! single thread, making them the ideal tool for operations that require
//! multiple steps to remain consistent under extreme concurrency.
//!
//! ## Module Structure
//!
//! - `p01_eval_basics` - EVAL and EVALSHA fundamentals
//! - `p02_stock_decrement` - Atomic check-and-decrement (CRITICAL)
//! - `p03_account_claim_check` - Atomic per-account claim deduplication (CRITICAL)
//! - `p04_product_voucher_limit` - Atomic per-product voucher limiting (CRITICAL)
//! - `p05_combined_purchase` - Master atomic purchase script (CRITICAL)
//! - `p06_sliding_window_rate_limit` - Sliding window rate limiting
//! - `p07_idempotency_check` - Atomic idempotency check-and-set
//! - `p08_batch_operations` - Batch stock checks in a single script
//! - `p09_script_caching` - EVALSHA vs EVAL caching strategy
//! - `p10_lua_error_patterns` - Error handling within Lua scripts

// ---------------------------------------------------------------------------
// Exercise stubs (used when "solution" feature is NOT enabled)
// ---------------------------------------------------------------------------
#[cfg(not(feature = "solution"))]
pub mod p01_eval_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_stock_decrement;
#[cfg(not(feature = "solution"))]
pub mod p03_account_claim_check;
#[cfg(not(feature = "solution"))]
pub mod p04_product_voucher_limit;
#[cfg(not(feature = "solution"))]
pub mod p05_combined_purchase;
#[cfg(not(feature = "solution"))]
pub mod p06_sliding_window_rate_limit;
#[cfg(not(feature = "solution"))]
pub mod p07_idempotency_check;
#[cfg(not(feature = "solution"))]
pub mod p08_batch_operations;
#[cfg(not(feature = "solution"))]
pub mod p09_script_caching;
#[cfg(not(feature = "solution"))]
pub mod p10_lua_error_patterns;

// ---------------------------------------------------------------------------
// Solution implementations (used when "solution" feature IS enabled)
// ---------------------------------------------------------------------------
#[cfg(feature = "solution")]
#[path = "solution/p01_eval_basics.rs"]
pub mod p01_eval_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_stock_decrement.rs"]
pub mod p02_stock_decrement;
#[cfg(feature = "solution")]
#[path = "solution/p03_account_claim_check.rs"]
pub mod p03_account_claim_check;
#[cfg(feature = "solution")]
#[path = "solution/p04_product_voucher_limit.rs"]
pub mod p04_product_voucher_limit;
#[cfg(feature = "solution")]
#[path = "solution/p05_combined_purchase.rs"]
pub mod p05_combined_purchase;
#[cfg(feature = "solution")]
#[path = "solution/p06_sliding_window_rate_limit.rs"]
pub mod p06_sliding_window_rate_limit;
#[cfg(feature = "solution")]
#[path = "solution/p07_idempotency_check.rs"]
pub mod p07_idempotency_check;
#[cfg(feature = "solution")]
#[path = "solution/p08_batch_operations.rs"]
pub mod p08_batch_operations;
#[cfg(feature = "solution")]
#[path = "solution/p09_script_caching.rs"]
pub mod p09_script_caching;
#[cfg(feature = "solution")]
#[path = "solution/p10_lua_error_patterns.rs"]
pub mod p10_lua_error_patterns;
