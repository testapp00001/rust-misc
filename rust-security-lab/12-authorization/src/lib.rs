//! # Module 12: Authorization
//!
//! Role-Based Access Control, Attribute-Based Access Control, capability tokens,
//! permission hierarchies, and authorization bypass attacks.
//!
//! ## Learning Path
//! 1. Start with `p01_rbac_basics` -- understand roles and permissions
//! 2. Move to ABAC and capability tokens for advanced patterns
//! 3. Study bypass attacks and resource scoping
//! 4. Build a policy engine and audit system
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 12-authorization              # Test your implementation
//! cargo test -p 12-authorization --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_rbac_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_abac_basics;
#[cfg(not(feature = "solution"))]
pub mod p03_capability_tokens;
#[cfg(not(feature = "solution"))]
pub mod p04_permission_hierarchy;
#[cfg(not(feature = "solution"))]
pub mod p05_least_privilege;
#[cfg(not(feature = "solution"))]
pub mod p06_authorization_bypass;
#[cfg(not(feature = "solution"))]
pub mod p07_resource_scoping;
#[cfg(not(feature = "solution"))]
pub mod p08_policy_engine;
#[cfg(not(feature = "solution"))]
pub mod p09_audit_decisions;
#[cfg(not(feature = "solution"))]
pub mod p10_graceful_denial;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_rbac_basics.rs"]
pub mod p01_rbac_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_abac_basics.rs"]
pub mod p02_abac_basics;
#[cfg(feature = "solution")]
#[path = "solution/p03_capability_tokens.rs"]
pub mod p03_capability_tokens;
#[cfg(feature = "solution")]
#[path = "solution/p04_permission_hierarchy.rs"]
pub mod p04_permission_hierarchy;
#[cfg(feature = "solution")]
#[path = "solution/p05_least_privilege.rs"]
pub mod p05_least_privilege;
#[cfg(feature = "solution")]
#[path = "solution/p06_authorization_bypass.rs"]
pub mod p06_authorization_bypass;
#[cfg(feature = "solution")]
#[path = "solution/p07_resource_scoping.rs"]
pub mod p07_resource_scoping;
#[cfg(feature = "solution")]
#[path = "solution/p08_policy_engine.rs"]
pub mod p08_policy_engine;
#[cfg(feature = "solution")]
#[path = "solution/p09_audit_decisions.rs"]
pub mod p09_audit_decisions;
#[cfg(feature = "solution")]
#[path = "solution/p10_graceful_denial.rs"]
pub mod p10_graceful_denial;
