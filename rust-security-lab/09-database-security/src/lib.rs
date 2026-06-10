//! # Module 09: Database Security
//!
//! Parameterized queries, SQL injection, field-level encryption, encryption at rest,
//! secure backups, audit logging, data masking, secure migrations, connection security,
//! and row-level security.
//!
//! ## Learning Path
//! 1. Start with `p01_parameterized_queries` — learn the single most important defense
//! 2. Then `p02_sql_injection_attack` — understand what happens without that defense
//! 3. Continue through encryption, auditing, and access control layers
//! 4. Build up to row-level security as the capstone concept
//!
//! ## Quick Test
//! ```bash
//! cargo test -p database_security              # Test your implementation
//! cargo test -p database_security --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_parameterized_queries;
#[cfg(not(feature = "solution"))]
pub mod p02_sql_injection_attack;
#[cfg(not(feature = "solution"))]
pub mod p03_field_level_encryption;
#[cfg(not(feature = "solution"))]
pub mod p04_encryption_at_rest;
#[cfg(not(feature = "solution"))]
pub mod p05_secure_backups;
#[cfg(not(feature = "solution"))]
pub mod p06_audit_logging;
#[cfg(not(feature = "solution"))]
pub mod p07_data_masking;
#[cfg(not(feature = "solution"))]
pub mod p08_secure_migrations;
#[cfg(not(feature = "solution"))]
pub mod p09_connection_security;
#[cfg(not(feature = "solution"))]
pub mod p10_row_level_security;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_parameterized_queries.rs"]
pub mod p01_parameterized_queries;
#[cfg(feature = "solution")]
#[path = "solution/p02_sql_injection_attack.rs"]
pub mod p02_sql_injection_attack;
#[cfg(feature = "solution")]
#[path = "solution/p03_field_level_encryption.rs"]
pub mod p03_field_level_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p04_encryption_at_rest.rs"]
pub mod p04_encryption_at_rest;
#[cfg(feature = "solution")]
#[path = "solution/p05_secure_backups.rs"]
pub mod p05_secure_backups;
#[cfg(feature = "solution")]
#[path = "solution/p06_audit_logging.rs"]
pub mod p06_audit_logging;
#[cfg(feature = "solution")]
#[path = "solution/p07_data_masking.rs"]
pub mod p07_data_masking;
#[cfg(feature = "solution")]
#[path = "solution/p08_secure_migrations.rs"]
pub mod p08_secure_migrations;
#[cfg(feature = "solution")]
#[path = "solution/p09_connection_security.rs"]
pub mod p09_connection_security;
#[cfg(feature = "solution")]
#[path = "solution/p10_row_level_security.rs"]
pub mod p10_row_level_security;
