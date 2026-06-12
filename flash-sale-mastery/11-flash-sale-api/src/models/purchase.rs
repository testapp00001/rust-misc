//! # Purchase Models
//!
//! Request and response types for the purchase flow.

use serde::{Deserialize, Serialize};

use crate::models::voucher::VoucherCode;

/// Incoming purchase request from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub product_id: String,
    pub account_id: String,
    pub idempotency_key: String,
}

/// Response returned after a purchase attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseResponse {
    pub status: PurchaseStatus,
    pub voucher_code: Option<String>,
    pub message: String,
}

/// Possible outcomes of a purchase attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseStatus {
    /// Stock was available, account eligible -- voucher issued.
    Success,
    /// No stock remaining for this product.
    SoldOut,
    /// This account already claimed a voucher for this product.
    AlreadyClaimed,
    /// The product-level voucher cap has been reached.
    VoucherLimitReached,
    /// The caller exceeded the per-account rate limit.
    RateLimited,
    /// The idempotency key was already processed; returning the original result.
    IdempotentReplay,
}

/// Internal result from the stock service after an atomic Lua check-and-decrement.
#[derive(Debug, Clone)]
pub enum PurchaseResult {
    Success {
        product_id: String,
        account_id: String,
    },
    SoldOut,
    AlreadyClaimed,
    VoucherLimitReached,
}

impl PurchaseResult {
    /// Convert an internal result into an API response.
    pub fn into_response(self, voucher: Option<VoucherCode>) -> PurchaseResponse {
        match self {
            PurchaseResult::Success { product_id, account_id } => {
                let code = voucher.map(|v| v.code).unwrap_or_default();
                PurchaseResponse {
                    status: PurchaseStatus::Success,
                    voucher_code: Some(code),
                    message: format!("Purchase successful for product {product_id} by account {account_id}"),
                }
            }
            PurchaseResult::SoldOut => PurchaseResponse {
                status: PurchaseStatus::SoldOut,
                voucher_code: None,
                message: "Product is sold out".to_string(),
            },
            PurchaseResult::AlreadyClaimed => PurchaseResponse {
                status: PurchaseStatus::AlreadyClaimed,
                voucher_code: None,
                message: "Account has already claimed a voucher for this product".to_string(),
            },
            PurchaseResult::VoucherLimitReached => PurchaseResponse {
                status: PurchaseStatus::VoucherLimitReached,
                voucher_code: None,
                message: "Product voucher limit reached".to_string(),
            },
        }
    }
}
