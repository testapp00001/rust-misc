//! # Module 14: API Security
//!
//! Protecting web APIs from abuse, forgery, and exploitation.
//!
//! ## Learning Path
//! 1. Start with `p01_rate_limiting` — learn to throttle abusive clients
//! 2. Sign and verify requests with `p02_request_signing` and `p03_hmac_webhooks`
//! 3. Harden endpoints with CORS, CSRF, input limits, and secure parsing
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 14-api-security              # Test your implementation
//! cargo test -p 14-api-security --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_rate_limiting;
#[cfg(not(feature = "solution"))]
pub mod p02_request_signing;
#[cfg(not(feature = "solution"))]
pub mod p03_hmac_webhooks;
#[cfg(not(feature = "solution"))]
pub mod p04_cors_security;
#[cfg(not(feature = "solution"))]
pub mod p05_csrf_protection;
#[cfg(not(feature = "solution"))]
pub mod p06_api_key_management;
#[cfg(not(feature = "solution"))]
pub mod p07_input_size_limits;
#[cfg(not(feature = "solution"))]
pub mod p08_json_security;
#[cfg(not(feature = "solution"))]
pub mod p09_graphql_security;
#[cfg(not(feature = "solution"))]
pub mod p10_api_versioning;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_rate_limiting.rs"]
pub mod p01_rate_limiting;
#[cfg(feature = "solution")]
#[path = "solution/p02_request_signing.rs"]
pub mod p02_request_signing;
#[cfg(feature = "solution")]
#[path = "solution/p03_hmac_webhooks.rs"]
pub mod p03_hmac_webhooks;
#[cfg(feature = "solution")]
#[path = "solution/p04_cors_security.rs"]
pub mod p04_cors_security;
#[cfg(feature = "solution")]
#[path = "solution/p05_csrf_protection.rs"]
pub mod p05_csrf_protection;
#[cfg(feature = "solution")]
#[path = "solution/p06_api_key_management.rs"]
pub mod p06_api_key_management;
#[cfg(feature = "solution")]
#[path = "solution/p07_input_size_limits.rs"]
pub mod p07_input_size_limits;
#[cfg(feature = "solution")]
#[path = "solution/p08_json_security.rs"]
pub mod p08_json_security;
#[cfg(feature = "solution")]
#[path = "solution/p09_graphql_security.rs"]
pub mod p09_graphql_security;
#[cfg(feature = "solution")]
#[path = "solution/p10_api_versioning.rs"]
pub mod p10_api_versioning;
