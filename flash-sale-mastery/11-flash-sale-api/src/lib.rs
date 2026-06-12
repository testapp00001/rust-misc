//! # Flash Sale API (Module 11)
//!
//! This is the **integration module** that wires together all previous modules
//! into a working Axum-based flash sale API.
//!
//! ## Module Structure
//!
//! | Directory         | Purpose                                        |
//! |-------------------|------------------------------------------------|
//! | `config`          | Environment-based configuration                |
//! | `models/`         | Shared request/response/event types            |
//! | `routes/`         | Axum route handlers (purchase, stock, health)  |
//! | `middleware/`     | Tower middleware (rate limit, idempotency, etc.)|
//! | `services/`       | Business logic (stock, voucher, order, notify)  |
//! | `resilience/`     | Circuit breaker and fallback strategies        |
//!
//! ## Request Flow
//!
//! ```text
//! Client
//!   -> Error Handler middleware (catches panics)
//!   -> Tracing middleware (spans, request_id, latency)
//!   -> Rate Limit middleware (per-account sliding window)
//!   -> Idempotency middleware (caches by Idempotency-Key)
//!   -> Route Handler
//!       -> Stock Service (Redis Lua script)
//!       -> Voucher Service (code generation)
//!       -> Order Service (async queue)
//!       -> Notification Service (fire-and-forget)
//! ```
//!
//! ## Quick Start
//!
//! ```bash
//! REDIS_URL=redis://127.0.0.1:6379 cargo run
//! curl -X POST http://localhost:3000/purchase \
//!   -H 'Content-Type: application/json' \
//!   -d '{"product_id":"prod-1","account_id":"acct-1","idempotency_key":"key-1"}'
//! ```

pub mod config;
pub mod middleware;
pub mod models;
pub mod resilience;
pub mod routes;
pub mod services;

use axum::Router;
use deadpool_redis::Config as RedisConfig;
use std::sync::Arc;
use std::time::Duration;

use config::Config;
use middleware::idempotency::{IdempotencyLayer, IdempotencyStore};
use middleware::rate_limit::{RateLimitLayer, RateLimiter};
use middleware::tracing::TracingLayer;
use middleware::error_handler::ErrorHandlerLayer;
use resilience::circuit_breaker::CircuitBreaker;
use services::notification_service::NotificationService;
use services::order_service::{create_order_queue, OrderQueue};
use services::stock_service::StockService;
use services::voucher_service::VoucherService;

/// Shared application state passed to all route handlers via Axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub redis_pool: deadpool_redis::Pool,
    pub config: Arc<Config>,
    pub stock_service: StockService,
    pub voucher_service: VoucherService,
    pub order_queue: OrderQueue,
    pub notification_service: NotificationService,
    pub circuit_breaker: CircuitBreaker,
}

/// Create the fully-wired Axum application.
#[cfg(feature = "solution")]
pub async fn create_app(config: Config) -> anyhow::Result<Router> {
    // Create Redis connection pool.
    let redis_cfg = RedisConfig::from_url(&config.redis_url);
    let redis_pool = redis_cfg
        .builder()
        .map_err(|e| anyhow::anyhow!("Failed to create Redis pool builder: {e}"))?
        .max_size(32)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create Redis pool: {e}"))?;

    // Create services.
    let stock_service = StockService::new(redis_pool.clone(), config.max_vouchers_per_product);
    let voucher_service = VoucherService::new(redis_pool.clone(), config.max_vouchers_per_product);
    let notification_service = NotificationService::new(true);

    // Create async order queue (buffer 10000 orders).
    let (order_queue, _order_rx) = create_order_queue(10_000);
    // In production, order_rx would be handed to a background worker task.

    // Create resilience components.
    let circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(30));

    // Build shared state.
    let state = AppState {
        redis_pool,
        config: Arc::new(config),
        stock_service,
        voucher_service,
        order_queue,
        notification_service,
        circuit_breaker,
    };

    // Build middleware stack (applied bottom-up: first layer listed = outermost).
    //
    //   Error Handler  (catches panics)
    //     -> Tracing   (spans, request_id)
    //     -> Rate Limit (per-account sliding window)
    //     -> Idempotency (cache by Idempotency-Key)
    //     -> Handler
    //
    let rate_limiter = RateLimiter::new(
        state.config.rate_limit_config.max_requests,
        Duration::from_secs(state.config.rate_limit_config.window_seconds),
    );

    let idempotency_store = IdempotencyStore::new(Duration::from_secs(3600));

    // Initialize health check start time.
    routes::health::init_start_time();

    let app = routes::create_router(state)
        .layer(IdempotencyLayer::new(idempotency_store))
        .layer(RateLimitLayer::new(rate_limiter))
        .layer(TracingLayer::new())
        .layer(ErrorHandlerLayer::new());

    Ok(app)
}

#[cfg(not(feature = "solution"))]
pub async fn create_app(config: Config) -> anyhow::Result<Router> {
    todo!("Wire up the Redis pool, services, middleware stack, and return the Router")
}

/// Helper for integration tests: create a test app with default config.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Create a test app, returning None if Redis is not available.
    pub async fn test_app() -> Option<Router> {
        let config = Config::default_test();
        create_app(config).await.ok()
    }
}
