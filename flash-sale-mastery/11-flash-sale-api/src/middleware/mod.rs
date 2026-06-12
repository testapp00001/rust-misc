//! # Middleware Module
//!
//! Tower middleware layers for the flash sale API pipeline:
//!
//! ```text
//! Request
//!   -> Error Handler (catches panics, formats JSON errors)
//!   -> Tracing (spans, request_id, latency)
//!   -> Rate Limit (per-account token bucket)
//!   -> Idempotency (caches responses by Idempotency-Key)
//!   -> Handler
//! ```

pub mod error_handler;
pub mod idempotency;
pub mod rate_limit;
pub mod tracing;
