//! # Module 31: Capstone — Secure Web API
//!
//! Production-grade secure web API combining all security knowledge from the course.
//! Each lesson builds one security layer. Lesson 10 composes them all.
//!
//! ## Learning Path
//! 1. Start with `p01_tls_setup` — encrypt the transport
//! 2. Add `p02_authentication_layer` — identify users
//! 3. Add `p03_authorization_layer` — control access
//! 4. Add `p04_rate_limiting` — prevent abuse
//! 5. Add `p05_input_validation` — reject bad data
//! 6. Add `p06_request_signing` — verify integrity
//! 7. Add `p07_audit_logging` — log everything
//! 8. Add `p08_error_handling` — fail safely
//! 9. Add `p09_secret_management` — protect credentials
//! 10. Build `p10_full_api` — combine all layers
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 31-capstone-secure-web-api              # Test your implementation
//! cargo test -p 31-capstone-secure-web-api --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_tls_setup;
#[cfg(not(feature = "solution"))]
pub mod p02_authentication_layer;
#[cfg(not(feature = "solution"))]
pub mod p03_authorization_layer;
#[cfg(not(feature = "solution"))]
pub mod p04_rate_limiting;
#[cfg(not(feature = "solution"))]
pub mod p05_input_validation;
#[cfg(not(feature = "solution"))]
pub mod p06_request_signing;
#[cfg(not(feature = "solution"))]
pub mod p07_audit_logging;
#[cfg(not(feature = "solution"))]
pub mod p08_error_handling;
#[cfg(not(feature = "solution"))]
pub mod p09_secret_management;
#[cfg(not(feature = "solution"))]
pub mod p10_full_api;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_tls_setup.rs"]
pub mod p01_tls_setup;
#[cfg(feature = "solution")]
#[path = "solution/p02_authentication_layer.rs"]
pub mod p02_authentication_layer;
#[cfg(feature = "solution")]
#[path = "solution/p03_authorization_layer.rs"]
pub mod p03_authorization_layer;
#[cfg(feature = "solution")]
#[path = "solution/p04_rate_limiting.rs"]
pub mod p04_rate_limiting;
#[cfg(feature = "solution")]
#[path = "solution/p05_input_validation.rs"]
pub mod p05_input_validation;
#[cfg(feature = "solution")]
#[path = "solution/p06_request_signing.rs"]
pub mod p06_request_signing;
#[cfg(feature = "solution")]
#[path = "solution/p07_audit_logging.rs"]
pub mod p07_audit_logging;
#[cfg(feature = "solution")]
#[path = "solution/p08_error_handling.rs"]
pub mod p08_error_handling;
#[cfg(feature = "solution")]
#[path = "solution/p09_secret_management.rs"]
pub mod p09_secret_management;
#[cfg(feature = "solution")]
#[path = "solution/p10_full_api.rs"]
pub mod p10_full_api;
