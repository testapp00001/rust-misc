//! # Module 23: Threat Modeling
//!
//! Learn to systematically identify, categorize, score, and mitigate security threats.
//! STRIDE, attack trees, DREAD scoring, data flow analysis, and security design review.
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 23-threat-modeling              # Test your implementation
//! cargo test -p 23-threat-modeling --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_stride_model;
#[cfg(not(feature = "solution"))]
pub mod p02_attack_trees;
#[cfg(not(feature = "solution"))]
pub mod p03_dread_scoring;
#[cfg(not(feature = "solution"))]
pub mod p04_data_flow_analysis;
#[cfg(not(feature = "solution"))]
pub mod p05_asset_identification;
#[cfg(not(feature = "solution"))]
pub mod p06_threat_enumeration;
#[cfg(not(feature = "solution"))]
pub mod p07_mitigation_strategies;
#[cfg(not(feature = "solution"))]
pub mod p08_security_requirements;
#[cfg(not(feature = "solution"))]
pub mod p09_risk_assessment;
#[cfg(not(feature = "solution"))]
pub mod p10_security_design_review;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_stride_model.rs"]
pub mod p01_stride_model;
#[cfg(feature = "solution")]
#[path = "solution/p02_attack_trees.rs"]
pub mod p02_attack_trees;
#[cfg(feature = "solution")]
#[path = "solution/p03_dread_scoring.rs"]
pub mod p03_dread_scoring;
#[cfg(feature = "solution")]
#[path = "solution/p04_data_flow_analysis.rs"]
pub mod p04_data_flow_analysis;
#[cfg(feature = "solution")]
#[path = "solution/p05_asset_identification.rs"]
pub mod p05_asset_identification;
#[cfg(feature = "solution")]
#[path = "solution/p06_threat_enumeration.rs"]
pub mod p06_threat_enumeration;
#[cfg(feature = "solution")]
#[path = "solution/p07_mitigation_strategies.rs"]
pub mod p07_mitigation_strategies;
#[cfg(feature = "solution")]
#[path = "solution/p08_security_requirements.rs"]
pub mod p08_security_requirements;
#[cfg(feature = "solution")]
#[path = "solution/p09_risk_assessment.rs"]
pub mod p09_risk_assessment;
#[cfg(feature = "solution")]
#[path = "solution/p10_security_design_review.rs"]
pub mod p10_security_design_review;
