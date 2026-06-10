//! # Module 22: Static Analysis
//!
//! Clippy is your first line of defense. Static analysis catches security bugs
//! before they reach production -- without running the code. This module covers
//! Clippy security lints, unsafe auditing, Miri, Kani formal verification,
//! custom lints, dependency auditing, and CI security gates.
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 22-static-analysis              # Test your implementation
//! cargo test -p 22-static-analysis --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_clippy_security;
#[cfg(not(feature = "solution"))]
pub mod p02_unsafe_audit;
#[cfg(not(feature = "solution"))]
pub mod p03_miri_basics;
#[cfg(not(feature = "solution"))]
pub mod p04_kani_verification;
#[cfg(not(feature = "solution"))]
pub mod p05_custom_lints;
#[cfg(not(feature = "solution"))]
pub mod p06_dependency_auditing;
#[cfg(not(feature = "solution"))]
pub mod p07_code_review_checklist;
#[cfg(not(feature = "solution"))]
pub mod p08_semgrep_rust;
#[cfg(not(feature = "solution"))]
pub mod p09_taint_analysis;
#[cfg(not(feature = "solution"))]
pub mod p10_ci_security_gate;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_clippy_security.rs"]
pub mod p01_clippy_security;
#[cfg(feature = "solution")]
#[path = "solution/p02_unsafe_audit.rs"]
pub mod p02_unsafe_audit;
#[cfg(feature = "solution")]
#[path = "solution/p03_miri_basics.rs"]
pub mod p03_miri_basics;
#[cfg(feature = "solution")]
#[path = "solution/p04_kani_verification.rs"]
pub mod p04_kani_verification;
#[cfg(feature = "solution")]
#[path = "solution/p05_custom_lints.rs"]
pub mod p05_custom_lints;
#[cfg(feature = "solution")]
#[path = "solution/p06_dependency_auditing.rs"]
pub mod p06_dependency_auditing;
#[cfg(feature = "solution")]
#[path = "solution/p07_code_review_checklist.rs"]
pub mod p07_code_review_checklist;
#[cfg(feature = "solution")]
#[path = "solution/p08_semgrep_rust.rs"]
pub mod p08_semgrep_rust;
#[cfg(feature = "solution")]
#[path = "solution/p09_taint_analysis.rs"]
pub mod p09_taint_analysis;
#[cfg(feature = "solution")]
#[path = "solution/p10_ci_security_gate.rs"]
pub mod p10_ci_security_gate;
