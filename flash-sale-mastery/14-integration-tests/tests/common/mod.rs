//! # Common Test Infrastructure
//!
//! Provides mock implementations of Redis and database layers for integration
//! testing. All mocks use DashMap for thread-safe in-memory storage, simulating
//! the atomic operations that real Redis Lua scripts would perform.

#![allow(dead_code)]

use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Product available in the flash sale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockProduct {
    pub id: String,
    pub name: String,
    pub price_cents: u64,
    pub initial_stock: u32,
    pub max_vouchers: Option<u32>,
}

/// A purchase request from a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPurchaseRequest {
    pub product_id: String,
    pub account_id: String,
    pub idempotency_key: String,
}

/// The outcome of a purchase attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MockPurchaseResult {
    Success {
        voucher_id: String,
        remaining_stock: u32,
    },
    SoldOut,
    AlreadyClaimed,
    VoucherLimitReached,
    Duplicate {
        cached: Box<MockPurchaseResult>,
    },
    RedisUnavailable,
    DbUnavailable,
}

impl fmt::Display for MockPurchaseResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success { voucher_id, remaining_stock } => {
                write!(f, "Success(voucher={voucher_id}, stock={remaining_stock})")
            }
            Self::SoldOut => write!(f, "SoldOut"),
            Self::AlreadyClaimed => write!(f, "AlreadyClaimed"),
            Self::VoucherLimitReached => write!(f, "VoucherLimitReached"),
            Self::Duplicate { cached } => write!(f, "Duplicate({cached})"),
            Self::RedisUnavailable => write!(f, "RedisUnavailable"),
            Self::DbUnavailable => write!(f, "DbUnavailable"),
        }
    }
}

/// An issued voucher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockVoucher {
    pub id: String,
    pub product_id: String,
    pub account_id: String,
    pub issued_at: String,
}

/// A pending order awaiting database persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockOrder {
    pub id: String,
    pub product_id: String,
    pub account_id: String,
    pub voucher_id: String,
    pub created_at: String,
}

/// Cached result stored for idempotency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub status_code: u16,
    pub body: String,
}

impl CachedResult {
    pub fn new(result: &MockPurchaseResult) -> Self {
        let status_code = match result {
            MockPurchaseResult::Success { .. } => 200,
            MockPurchaseResult::SoldOut => 409,
            MockPurchaseResult::AlreadyClaimed => 409,
            MockPurchaseResult::VoucherLimitReached => 409,
            MockPurchaseResult::Duplicate { .. } => 200,
            MockPurchaseResult::RedisUnavailable => 503,
            MockPurchaseResult::DbUnavailable => 503,
        };
        Self {
            status_code,
            body: serde_json::to_string(result).unwrap_or_default(),
        }
    }
}

/// Type of event recorded during processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    StockDecremented,
    VoucherIssued,
    PurchaseCompleted,
    PurchaseFailed,
}

/// An audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub product_id: String,
    pub account_id: Option<String>,
    pub message: String,
    pub timestamp: String,
}

/// A mismatch found during reconciliation.
#[derive(Debug, Clone)]
pub struct Mismatch {
    pub product_id: String,
    pub redis_stock: u32,
    pub db_orders: u32,
    pub initial_stock: u32,
}

/// The result of running reconciliation.
#[derive(Debug)]
pub struct ReconciliationResult {
    pub mismatches: Vec<Mismatch>,
    pub corrections: u32,
}

// ---------------------------------------------------------------------------
// MockRedis — simulates Redis with atomic operations via DashMap
// ---------------------------------------------------------------------------

/// In-memory mock of Redis, using DashMap for thread-safe concurrent access.
///
/// Stock decrements are atomic (compare-and-set via DashMap::entry), closely
/// simulating the behaviour of a real Redis Lua script.
pub struct MockRedis {
    /// product_id -> remaining stock (AtomicU32 for true atomicity)
    stock: DashMap<String, Arc<AtomicU32>>,
    /// idempotency_key -> CachedResult
    idempotency: DashMap<String, CachedResult>,
    /// "account_id:product_id" -> true if already claimed
    account_claims: DashMap<String, bool>,
    /// product_id -> number of vouchers issued
    voucher_counts: DashMap<String, Arc<AtomicU32>>,
    /// Append-only event log
    events: DashMap<usize, Event>,
    event_counter: AtomicU32,
    /// When true, all operations return errors
    pub failing: AtomicBool,
}

impl MockRedis {
    pub fn new() -> Self {
        Self {
            stock: DashMap::new(),
            idempotency: DashMap::new(),
            account_claims: DashMap::new(),
            voucher_counts: DashMap::new(),
            events: DashMap::new(),
            event_counter: AtomicU32::new(0),
            failing: AtomicBool::new(false),
        }
    }

    /// Initialise stock for a product.
    pub fn set_stock(&self, product_id: &str, stock: u32) {
        self.stock
            .insert(product_id.to_string(), Arc::new(AtomicU32::new(stock)));
    }

    /// Read the current stock for a product.
    pub fn get_stock(&self, product_id: &str) -> u32 {
        self.stock
            .get(product_id)
            .map(|entry| entry.value().load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Atomically decrement stock by 1. Returns `true` if successful,
    /// `false` if stock was already 0.
    pub fn decrement_stock(&self, product_id: &str) -> bool {
        let Some(stock_ref) = self.stock.get(product_id) else {
            return false;
        };
        let stock = stock_ref.value();
        // Spin-CAS: load, check, compare_exchange
        loop {
            let current = stock.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }
            match stock.compare_exchange_weak(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    self.record_event(
                        EventType::StockDecremented,
                        product_id,
                        None,
                        &format!("stock {current} -> {}", current - 1),
                    );
                    return true;
                }
                Err(_) => continue, // lost the race, retry
            }
        }
    }

    /// Check whether an idempotency key has been seen before.
    /// Returns `Some(cached)` if duplicate, `None` if first time.
    pub fn check_idempotency(&self, key: &str) -> Option<CachedResult> {
        self.idempotency.get(key).map(|r| r.value().clone())
    }

    /// Cache the result for an idempotency key.
    pub fn cache_idempotency(&self, key: &str, result: &MockPurchaseResult) {
        self.idempotency
            .insert(key.to_string(), CachedResult::new(result));
    }

    /// Record that an account has claimed a product. Returns `true` if this is
    /// the first claim, `false` if already claimed.
    pub fn record_claim(&self, account_id: &str, product_id: &str) -> bool {
        let key = format!("{account_id}:{product_id}");
        self.account_claims.insert(key, true).is_none()
    }

    /// Atomically increment voucher count. Returns `Some(new_count)` on
    /// success, or `None` if the limit would be exceeded.
    pub fn increment_voucher_count(
        &self,
        product_id: &str,
        max_vouchers: u32,
    ) -> Option<u32> {
        let Some(count_ref) = self.voucher_counts.get(product_id) else {
            return None;
        };
        let counter = count_ref.value();
        loop {
            let current = counter.load(Ordering::SeqCst);
            if current >= max_vouchers {
                return None;
            }
            match counter.compare_exchange_weak(
                current,
                current + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Some(current + 1),
                Err(_) => continue,
            }
        }
    }

    /// Set the initial voucher count (for test setup).
    pub fn set_voucher_count(&self, product_id: &str, count: u32) {
        self.voucher_counts
            .insert(product_id.to_string(), Arc::new(AtomicU32::new(count)));
    }

    /// Append an event to the audit log.
    pub fn record_event(
        &self,
        event_type: EventType,
        product_id: &str,
        account_id: Option<&str>,
        message: &str,
    ) {
        let idx = self.event_counter.fetch_add(1, Ordering::SeqCst) as usize;
        self.events.insert(
            idx,
            Event {
                event_type,
                product_id: product_id.to_string(),
                account_id: account_id.map(|s| s.to_string()),
                message: message.to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
        );
    }

    /// Return all recorded events in order.
    pub fn get_events(&self) -> Vec<Event> {
        let mut events: Vec<(usize, Event)> = self
            .events
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();
        events.sort_by_key(|(idx, _)| *idx);
        events.into_iter().map(|(_, e)| e).collect()
    }
}

// ---------------------------------------------------------------------------
// MockDb — simulates the database layer
// ---------------------------------------------------------------------------

/// In-memory mock of the database.
pub struct MockDb {
    orders: DashMap<String, MockOrder>,
    vouchers: DashMap<String, MockVoucher>,
    pub failing: AtomicBool,
}

impl MockDb {
    pub fn new() -> Self {
        Self {
            orders: DashMap::new(),
            vouchers: DashMap::new(),
            failing: AtomicBool::new(false),
        }
    }

    /// Persist an order. Returns `Err` if the DB is marked as failing.
    pub fn store_order(&self, order: &MockOrder) -> Result<(), String> {
        if self.failing.load(Ordering::SeqCst) {
            return Err("database unavailable".into());
        }
        self.orders.insert(order.id.clone(), order.clone());
        Ok(())
    }

    /// Persist a voucher. Returns `Err` if the DB is marked as failing.
    pub fn store_voucher(&self, voucher: &MockVoucher) -> Result<(), String> {
        if self.failing.load(Ordering::SeqCst) {
            return Err("database unavailable".into());
        }
        self.vouchers.insert(voucher.id.clone(), voucher.clone());
        Ok(())
    }

    /// Count orders for a product.
    pub fn count_orders(&self, product_id: &str) -> u32 {
        self.orders
            .iter()
            .filter(|entry| entry.value().product_id == product_id)
            .count() as u32
    }

    /// Count vouchers for a product.
    pub fn count_vouchers(&self, product_id: &str) -> u32 {
        self.vouchers
            .iter()
            .filter(|entry| entry.value().product_id == product_id)
            .count() as u32
    }

    /// Return all orders (for reconciliation).
    pub fn get_all_orders(&self) -> Vec<MockOrder> {
        self.orders
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// MockFlashSaleService — the service under test
// ---------------------------------------------------------------------------

/// The flash sale service, wired to mock Redis and DB backends.
pub struct MockFlashSaleService {
    pub redis: Arc<MockRedis>,
    pub db: Arc<MockDb>,
    pub products: DashMap<String, MockProduct>,
}

impl MockFlashSaleService {
    pub fn new(redis: Arc<MockRedis>, db: Arc<MockDb>) -> Self {
        Self {
            redis,
            db,
            products: DashMap::new(),
        }
    }

    /// Register a product for the flash sale.
    pub fn add_product(&self, product: MockProduct) {
        self.redis.set_stock(&product.id, product.initial_stock);
        if product.max_vouchers.is_some() {
            self.redis.set_voucher_count(&product.id, 0);
        }
        self.products.insert(product.id.clone(), product);
    }

    /// Process a purchase request. This is the hot path under test.
    pub async fn purchase(
        &self,
        request: &MockPurchaseRequest,
    ) -> MockPurchaseResult {
        // 1. Check Redis availability
        if self.redis.failing.load(Ordering::SeqCst) {
            self.redis.record_event(
                EventType::PurchaseFailed,
                &request.product_id,
                Some(&request.account_id),
                "redis unavailable",
            );
            return MockPurchaseResult::RedisUnavailable;
        }

        // 2. Idempotency check
        if let Some(cached) = self.redis.check_idempotency(&request.idempotency_key) {
            return MockPurchaseResult::Duplicate {
                cached: Box::new(deserialize_result(&cached.body)),
            };
        }

        // 3. Look up product
        let Some(product_ref) = self.products.get(&request.product_id) else {
            self.redis.record_event(
                EventType::PurchaseFailed,
                &request.product_id,
                Some(&request.account_id),
                "product not found",
            );
            let result = MockPurchaseResult::SoldOut;
            self.redis
                .cache_idempotency(&request.idempotency_key, &result);
            return result;
        };
        let product = product_ref.value().clone();
        drop(product_ref);

        // 4. Per-account claim check
        if !self
            .redis
            .record_claim(&request.account_id, &request.product_id)
        {
            self.redis.record_event(
                EventType::PurchaseFailed,
                &request.product_id,
                Some(&request.account_id),
                "already claimed",
            );
            let result = MockPurchaseResult::AlreadyClaimed;
            self.redis
                .cache_idempotency(&request.idempotency_key, &result);
            return result;
        }

        // 5. Per-product voucher limit check
        if let Some(max_vouchers) = product.max_vouchers {
            if self
                .redis
                .increment_voucher_count(&request.product_id, max_vouchers)
                .is_none()
            {
                self.redis.record_event(
                    EventType::PurchaseFailed,
                    &request.product_id,
                    Some(&request.account_id),
                    "voucher limit reached",
                );
                let result = MockPurchaseResult::VoucherLimitReached;
                self.redis
                    .cache_idempotency(&request.idempotency_key, &result);
                return result;
            }
        }

        // 6. Atomic stock decrement (the critical CAS operation)
        if !self.redis.decrement_stock(&request.product_id) {
            self.redis.record_event(
                EventType::PurchaseFailed,
                &request.product_id,
                Some(&request.account_id),
                "sold out",
            );
            let result = MockPurchaseResult::SoldOut;
            self.redis
                .cache_idempotency(&request.idempotency_key, &result);
            return result;
        }

        // 7. Issue voucher
        let voucher_id = Uuid::new_v4().to_string();
        let voucher = MockVoucher {
            id: voucher_id.clone(),
            product_id: request.product_id.clone(),
            account_id: request.account_id.clone(),
            issued_at: Utc::now().to_rfc3339(),
        };
        self.redis.record_event(
            EventType::VoucherIssued,
            &request.product_id,
            Some(&request.account_id),
            &format!("voucher {voucher_id} issued"),
        );

        // 8. Persist to database (may fail independently)
        let order = MockOrder {
            id: Uuid::new_v4().to_string(),
            product_id: request.product_id.clone(),
            account_id: request.account_id.clone(),
            voucher_id: voucher_id.clone(),
            created_at: Utc::now().to_rfc3339(),
        };
        if let Err(e) = self.db.store_order(&order) {
            tracing::warn!(
                product_id = %request.product_id,
                error = %e,
                "DB write failed — order queued for retry"
            );
        }
        let _ = self.db.store_voucher(&voucher);

        // 9. Record completion and cache result
        self.redis.record_event(
            EventType::PurchaseCompleted,
            &request.product_id,
            Some(&request.account_id),
            "purchase completed",
        );
        let remaining = self.redis.get_stock(&request.product_id);
        let result = MockPurchaseResult::Success {
            voucher_id,
            remaining_stock: remaining,
        };
        self.redis
            .cache_idempotency(&request.idempotency_key, &result);
        result
    }

    /// Run reconciliation between Redis stock and DB orders.
    pub fn reconcile(&self, initial_stocks: &std::collections::HashMap<String, u32>) -> ReconciliationResult {
        let mut mismatches = Vec::new();
        let mut corrections = 0u32;

        for entry in self.products.iter() {
            let product_id = entry.key();
            let redis_stock = self.redis.get_stock(product_id);
            let db_orders = self.db.count_orders(product_id);
            let initial = initial_stocks.get(product_id).copied().unwrap_or(0);

            let expected_remaining = initial.saturating_sub(db_orders);
            if redis_stock != expected_remaining {
                mismatches.push(Mismatch {
                    product_id: product_id.clone(),
                    redis_stock,
                    db_orders,
                    initial_stock: initial,
                });
                corrections += 1;
            }
        }

        ReconciliationResult {
            mismatches,
            corrections,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deserialise a `MockPurchaseResult` from its JSON body.
fn deserialize_result(body: &str) -> MockPurchaseResult {
    serde_json::from_str(body).unwrap_or(MockPurchaseResult::SoldOut)
}

/// Generate a unique idempotency key for test isolation.
pub fn make_idempotency_key(test_name: &str, account_id: &str) -> String {
    format!("{test_name}:{account_id}:{}", Uuid::new_v4())
}

/// Create a standard test product.
pub fn make_product(id: &str, stock: u32) -> MockProduct {
    MockProduct {
        id: id.to_string(),
        name: format!("Product {id}"),
        price_cents: 999,
        initial_stock: stock,
        max_vouchers: None,
    }
}

/// Create a test product with a voucher limit.
pub fn make_product_with_limit(id: &str, stock: u32, max_vouchers: u32) -> MockProduct {
    MockProduct {
        id: id.to_string(),
        name: format!("Product {id}"),
        price_cents: 999,
        initial_stock: stock,
        max_vouchers: Some(max_vouchers),
    }
}

/// Create a purchase request.
pub fn make_request(test_name: &str, product_id: &str, account_id: &str) -> MockPurchaseRequest {
    MockPurchaseRequest {
        product_id: product_id.to_string(),
        account_id: account_id.to_string(),
        idempotency_key: make_idempotency_key(test_name, account_id),
    }
}
