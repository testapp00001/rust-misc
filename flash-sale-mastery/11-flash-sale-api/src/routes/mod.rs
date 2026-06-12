//! # Routes Module
//!
//! Axum route handlers for the flash sale API.

pub mod health;
pub mod purchase;
pub mod stock_query;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

/// Build the API router with all endpoints.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/purchase", post(purchase::handle_purchase))
        .route("/stock/{product_id}", get(stock_query::handle_stock_query))
        .route("/health", get(health::handle_health))
        .with_state(state)
}
