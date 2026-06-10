//! # Module 26: Privacy Engineering
//!
//! Differential privacy, k-anonymity, GDPR compliance, and privacy by design.
//!
//! ## Learning Path
//! 1. Start with `p01_differential_privacy` -- understand noise calibration and epsilon
//! 2. Progress through anonymization, minimization, and GDPR rights
//! 3. Build up to privacy-preserving computation and impact assessment
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 26-privacy-engineering              # Test your implementation
//! cargo test -p 26-privacy-engineering --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_differential_privacy;
#[cfg(not(feature = "solution"))]
pub mod p02_k_anonymity;
#[cfg(not(feature = "solution"))]
pub mod p03_data_minimization;
#[cfg(not(feature = "solution"))]
pub mod p04_anonymization;
#[cfg(not(feature = "solution"))]
pub mod p05_gdpr_technical;
#[cfg(not(feature = "solution"))]
pub mod p06_privacy_by_design;
#[cfg(not(feature = "solution"))]
pub mod p07_data_retention;
#[cfg(not(feature = "solution"))]
pub mod p08_consent_management;
#[cfg(not(feature = "solution"))]
pub mod p09_privacy_preserving_computation;
#[cfg(not(feature = "solution"))]
pub mod p10_privacy_impact;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_differential_privacy.rs"]
pub mod p01_differential_privacy;
#[cfg(feature = "solution")]
#[path = "solution/p02_k_anonymity.rs"]
pub mod p02_k_anonymity;
#[cfg(feature = "solution")]
#[path = "solution/p03_data_minimization.rs"]
pub mod p03_data_minimization;
#[cfg(feature = "solution")]
#[path = "solution/p04_anonymization.rs"]
pub mod p04_anonymization;
#[cfg(feature = "solution")]
#[path = "solution/p05_gdpr_technical.rs"]
pub mod p05_gdpr_technical;
#[cfg(feature = "solution")]
#[path = "solution/p06_privacy_by_design.rs"]
pub mod p06_privacy_by_design;
#[cfg(feature = "solution")]
#[path = "solution/p07_data_retention.rs"]
pub mod p07_data_retention;
#[cfg(feature = "solution")]
#[path = "solution/p08_consent_management.rs"]
pub mod p08_consent_management;
#[cfg(feature = "solution")]
#[path = "solution/p09_privacy_preserving_computation.rs"]
pub mod p09_privacy_preserving_computation;
#[cfg(feature = "solution")]
#[path = "solution/p10_privacy_impact.rs"]
pub mod p10_privacy_impact;
