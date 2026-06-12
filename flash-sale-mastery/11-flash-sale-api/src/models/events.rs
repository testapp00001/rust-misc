//! # Flash Sale Events
//!
//! Audit trail events emitted during the purchase flow.

use serde::{Deserialize, Serialize};

/// Events written to the audit trail (Redis Stream or DB).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum FlashSaleEvent {
    /// A purchase attempt was initiated.
    PurchaseAttempt {
        product_id: String,
        account_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// A purchase completed successfully.
    PurchaseSuccess {
        product_id: String,
        account_id: String,
        voucher_code: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// A purchase was rejected.
    PurchaseFailed {
        product_id: String,
        account_id: String,
        reason: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Stock for a product reached zero.
    StockDepleted {
        product_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// The voucher cap for a product was reached.
    VoucherLimitReached {
        product_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}
