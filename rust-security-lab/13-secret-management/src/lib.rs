//! # Module 13: Secret Management
//!
//! Storing, distributing, rotating, and auditing sensitive credentials.
//!
//! ## Learning Path
//! 1. Start with `p01_env_var_secrets` — understand why hardcoding secrets is dangerous
//! 2. Learn to detect secrets with `p02_secret_scanning`
//! 3. Build up to vault integration, rotation, and incident response
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 13-secret-management              # Test your implementation
//! cargo test -p 13-secret-management --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_env_var_secrets;
#[cfg(not(feature = "solution"))]
pub mod p02_secret_scanning;
#[cfg(not(feature = "solution"))]
pub mod p03_vault_integration;
#[cfg(not(feature = "solution"))]
pub mod p04_secret_rotation;
#[cfg(not(feature = "solution"))]
pub mod p05_sealed_secrets;
#[cfg(not(feature = "solution"))]
pub mod p06_secret_injection;
#[cfg(not(feature = "solution"))]
pub mod p07_memory_protection;
#[cfg(not(feature = "solution"))]
pub mod p08_secret_sharing;
#[cfg(not(feature = "solution"))]
pub mod p09_audit_secret_access;
#[cfg(not(feature = "solution"))]
pub mod p10_secret_leak_response;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_env_var_secrets.rs"]
pub mod p01_env_var_secrets;
#[cfg(feature = "solution")]
#[path = "solution/p02_secret_scanning.rs"]
pub mod p02_secret_scanning;
#[cfg(feature = "solution")]
#[path = "solution/p03_vault_integration.rs"]
pub mod p03_vault_integration;
#[cfg(feature = "solution")]
#[path = "solution/p04_secret_rotation.rs"]
pub mod p04_secret_rotation;
#[cfg(feature = "solution")]
#[path = "solution/p05_sealed_secrets.rs"]
pub mod p05_sealed_secrets;
#[cfg(feature = "solution")]
#[path = "solution/p06_secret_injection.rs"]
pub mod p06_secret_injection;
#[cfg(feature = "solution")]
#[path = "solution/p07_memory_protection.rs"]
pub mod p07_memory_protection;
#[cfg(feature = "solution")]
#[path = "solution/p08_secret_sharing.rs"]
pub mod p08_secret_sharing;
#[cfg(feature = "solution")]
#[path = "solution/p09_audit_secret_access.rs"]
pub mod p09_audit_secret_access;
#[cfg(feature = "solution")]
#[path = "solution/p10_secret_leak_response.rs"]
pub mod p10_secret_leak_response;
