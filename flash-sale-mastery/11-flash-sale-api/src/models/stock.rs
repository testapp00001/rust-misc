//! # Stock Models
//!
//! Data types representing stock information and updates.

use serde::{Deserialize, Serialize};

/// Current stock state for a product (returned by the stock query endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub product_id: String,
    pub stock_remaining: u64,
    pub total_vouchers: u64,
    pub sale_active: bool,
}

/// A pending stock mutation (used for audit / event sourcing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockUpdate {
    pub product_id: String,
    pub account_id: String,
    pub decrement: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
