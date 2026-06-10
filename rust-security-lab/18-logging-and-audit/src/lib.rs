//! # Module 18: Logging and Audit
//!
//! PII redaction, structured logging, log injection defense, tamper-evident logs,
//! audit trails, sensitive data masking, security-aware log levels, correlation IDs,
//! compliance logging, and log retention policies.
//!
//! ## Learning Path
//! 1. Start with `p01_pii_redaction` -- never log raw PII
//! 2. Learn structured logging (p02) for machine-parseable security events
//! 3. Understand log injection attacks (p03) and how to defend against them
//! 4. Build tamper-evident logs (p04) with hash chains
//! 5. Implement audit trails (p05) for compliance
//! 6. Mask sensitive data (p06) with partial reveal
//! 7. Use security-aware log levels (p07)
//! 8. Add correlation IDs (p08) for distributed tracing
//! 9. Meet compliance requirements (p09) for GDPR, SOC2, HIPAA
//! 10. Implement log retention (p10) with secure deletion
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 18-logging-and-audit              # Test your implementation
//! cargo test -p 18-logging-and-audit --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_pii_redaction;
#[cfg(not(feature = "solution"))]
pub mod p02_structured_logging;
#[cfg(not(feature = "solution"))]
pub mod p03_log_injection;
#[cfg(not(feature = "solution"))]
pub mod p04_tamper_evident_logs;
#[cfg(not(feature = "solution"))]
pub mod p05_audit_trail;
#[cfg(not(feature = "solution"))]
pub mod p06_sensitive_data_masking;
#[cfg(not(feature = "solution"))]
pub mod p07_log_levels_security;
#[cfg(not(feature = "solution"))]
pub mod p08_correlation_ids;
#[cfg(not(feature = "solution"))]
pub mod p09_compliance_logging;
#[cfg(not(feature = "solution"))]
pub mod p10_log_retention;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_pii_redaction.rs"]
pub mod p01_pii_redaction;
#[cfg(feature = "solution")]
#[path = "solution/p02_structured_logging.rs"]
pub mod p02_structured_logging;
#[cfg(feature = "solution")]
#[path = "solution/p03_log_injection.rs"]
pub mod p03_log_injection;
#[cfg(feature = "solution")]
#[path = "solution/p04_tamper_evident_logs.rs"]
pub mod p04_tamper_evident_logs;
#[cfg(feature = "solution")]
#[path = "solution/p05_audit_trail.rs"]
pub mod p05_audit_trail;
#[cfg(feature = "solution")]
#[path = "solution/p06_sensitive_data_masking.rs"]
pub mod p06_sensitive_data_masking;
#[cfg(feature = "solution")]
#[path = "solution/p07_log_levels_security.rs"]
pub mod p07_log_levels_security;
#[cfg(feature = "solution")]
#[path = "solution/p08_correlation_ids.rs"]
pub mod p08_correlation_ids;
#[cfg(feature = "solution")]
#[path = "solution/p09_compliance_logging.rs"]
pub mod p09_compliance_logging;
#[cfg(feature = "solution")]
#[path = "solution/p10_log_retention.rs"]
pub mod p10_log_retention;
