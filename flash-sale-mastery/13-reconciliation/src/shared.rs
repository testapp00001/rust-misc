//! # Shared Types for Reconciliation
//!
//! Common data structures used across all reconciliation exercises.
//! These represent the canonical types for inventory, events, and discrepancies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during reconciliation operations.
#[derive(Debug, Error)]
pub enum ReconciliationError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Data corruption detected: {0}")]
    Corruption(String),

    #[error("Reconciliation conflict: {0}")]
    Conflict(String),
}

/// Inventory record as stored in Redis (the fast-path, authoritative during sale).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedisInventory {
    pub product_id: String,
    pub available_stock: i64,
    pub reserved: i64,
    pub version: u64,
    pub last_updated: DateTime<Utc>,
}

/// Inventory record as stored in PostgreSQL (the durable, authoritative after sale).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbInventory {
    pub product_id: String,
    pub available_stock: i64,
    pub reserved: i64,
    pub total_sold: i64,
    pub version: u64,
    pub last_updated: DateTime<Utc>,
}

/// Result of synchronizing a single product's inventory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncResult {
    pub product_id: String,
    pub redis_stock: i64,
    pub db_stock: i64,
    pub mismatch: bool,
    pub action_taken: SyncAction,
}

/// The action taken during synchronization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncAction {
    /// No action needed -- Redis and DB already agree.
    NoOp,
    /// DB was updated to match Redis (Redis is authoritative during sale).
    UpdatedDb,
    /// Alert raised: Redis stock is less than DB stock, which indicates data loss.
    AlertRaised {
        reason: String,
    },
}

/// An event recorded in the event store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryEvent {
    pub event_id: Uuid,
    pub product_id: String,
    pub event_type: EventType,
    pub quantity: i64,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<String>,
}

/// Types of inventory events that can occur.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Stock was added (restock).
    StockAdded,
    /// An item was reserved for purchase.
    Reserved,
    /// A reservation was confirmed (purchase completed).
    Confirmed,
    /// A reservation was released (purchase cancelled/timed out).
    Released,
    /// An adjustment was made (correction, audit).
    Adjusted,
}

/// State rebuilt from replaying events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RebuiltState {
    pub product_id: String,
    pub computed_available: i64,
    pub computed_reserved: i64,
    pub events_processed: usize,
    pub matches_current: bool,
}

/// A detected discrepancy between data sources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Discrepancy {
    pub discrepancy_id: Uuid,
    pub discrepancy_type: DiscrepancyType,
    pub product_id: Option<String>,
    pub description: String,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
}

/// Types of discrepancies that can be detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscrepancyType {
    /// Redis stock does not match DB stock.
    StockMismatch,
    /// An order exists in one system but not the other.
    MissingOrder,
    /// A claim exists with no corresponding order or reservation.
    OrphanedClaim,
    /// A voucher was used more times than allowed.
    DuplicateVoucher,
}

/// Severity level of a discrepancy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// A complete reconciliation report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReconciliationReport {
    pub report_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub total_products_checked: usize,
    pub discrepancies_found: Vec<Discrepancy>,
    pub corrections_applied: usize,
    pub sync_results: Vec<SyncResult>,
    pub summary: ReportSummary,
}

/// High-level summary of reconciliation results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportSummary {
    pub healthy: bool,
    pub stock_mismatches: usize,
    pub missing_orders: usize,
    pub orphaned_claims: usize,
    pub duplicate_vouchers: usize,
    pub total_corrections: usize,
}

impl ReconciliationReport {
    /// Render the report as a human-readable string.
    pub fn to_human_readable(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "=== Reconciliation Report {} ===\n",
            self.report_id
        ));
        out.push_str(&format!("Generated: {}\n", self.generated_at));
        out.push_str(&format!(
            "Products checked: {}\n",
            self.total_products_checked
        ));
        out.push_str(&format!(
            "Discrepancies found: {}\n",
            self.discrepancies_found.len()
        ));
        out.push_str(&format!(
            "Corrections applied: {}\n",
            self.corrections_applied
        ));
        out.push_str(&format!(
            "Status: {}\n",
            if self.summary.healthy {
                "HEALTHY"
            } else {
                "ATTENTION REQUIRED"
            }
        ));

        if !self.discrepancies_found.is_empty() {
            out.push_str("\n--- Discrepancies ---\n");
            for d in &self.discrepancies_found {
                out.push_str(&format!(
                    "  [{:?}] {:?}: {}\n",
                    d.severity, d.discrepancy_type, d.description
                ));
            }
        }

        out
    }
}
