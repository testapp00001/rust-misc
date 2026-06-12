//! # Voucher Models
//!
//! Data types for voucher codes and per-product voucher limits.

use serde::{Deserialize, Serialize};

/// A generated voucher code tied to a product and account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherCode {
    pub code: String,
    pub product_id: String,
    pub account_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Tracks how many vouchers have been issued for a product vs. the cap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherLimit {
    pub product_id: String,
    pub max_vouchers: u64,
    pub issued_count: u64,
}

impl VoucherLimit {
    /// Returns true when no more vouchers may be issued.
    pub fn is_reached(&self) -> bool {
        self.issued_count >= self.max_vouchers
    }
}
