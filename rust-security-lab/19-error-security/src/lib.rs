//! # Module 19: Error Security
//!
//! Information leakage in error messages, timing oracles, fail-closed vs fail-open,
//! internal vs external errors, secure error design patterns.
//!
//! ## Learning Path
//! 1. Start with `p01_information_leakage` -- understand how error messages leak secrets
//! 2. Design secure error types (p02) that separate internal from user-facing details
//! 3. Handle panics safely (p03) -- don't leak secrets in panic messages
//! 4. Fix error oracles (p04) -- "user not found" vs "wrong password"
//! 5. Close timing oracles (p05) -- constant-time auth for valid/invalid users
//! 6. Defend against error enumeration (p06) -- generic auth error messages
//! 7. Suppress stack traces in production (p07) -- disable RUST_BACKTRACE
//! 8. Aggregate validation errors (p08) -- don't reveal which field failed first
//! 9. Implement secure defaults (p09) -- fail closed, deny by default
//! 10. Log errors securely (p10) -- log details internally, return generic to user
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 19-error-security              # Test your implementation
//! cargo test -p 19-error-security --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_information_leakage;
#[cfg(not(feature = "solution"))]
pub mod p02_secure_error_types;
#[cfg(not(feature = "solution"))]
pub mod p03_panic_safety;
#[cfg(not(feature = "solution"))]
pub mod p04_error_oracle;
#[cfg(not(feature = "solution"))]
pub mod p05_timing_oracle;
#[cfg(not(feature = "solution"))]
pub mod p06_error_enumeration;
#[cfg(not(feature = "solution"))]
pub mod p07_stack_trace_exposure;
#[cfg(not(feature = "solution"))]
pub mod p08_error_aggregation;
#[cfg(not(feature = "solution"))]
pub mod p09_secure_defaults;
#[cfg(not(feature = "solution"))]
pub mod p10_error_logging;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_information_leakage.rs"]
pub mod p01_information_leakage;
#[cfg(feature = "solution")]
#[path = "solution/p02_secure_error_types.rs"]
pub mod p02_secure_error_types;
#[cfg(feature = "solution")]
#[path = "solution/p03_panic_safety.rs"]
pub mod p03_panic_safety;
#[cfg(feature = "solution")]
#[path = "solution/p04_error_oracle.rs"]
pub mod p04_error_oracle;
#[cfg(feature = "solution")]
#[path = "solution/p05_timing_oracle.rs"]
pub mod p05_timing_oracle;
#[cfg(feature = "solution")]
#[path = "solution/p06_error_enumeration.rs"]
pub mod p06_error_enumeration;
#[cfg(feature = "solution")]
#[path = "solution/p07_stack_trace_exposure.rs"]
pub mod p07_stack_trace_exposure;
#[cfg(feature = "solution")]
#[path = "solution/p08_error_aggregation.rs"]
pub mod p08_error_aggregation;
#[cfg(feature = "solution")]
#[path = "solution/p09_secure_defaults.rs"]
pub mod p09_secure_defaults;
#[cfg(feature = "solution")]
#[path = "solution/p10_error_logging.rs"]
pub mod p10_error_logging;
