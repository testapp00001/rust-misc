//! # Lesson 10: PQC Risk Assessment — "Harvest Now, Decrypt Later"
//!
//! ## The "Harvest Now, Decrypt Later" (HNDL) Threat
//!
//! Nation-state adversaries are collecting encrypted traffic TODAY with the
//! expectation that future quantum computers will allow them to decrypt it.
//!
//! ```
//! 2024: Adversary intercepts and stores TLS traffic
//! 2030: Quantum computer breaks RSA/ECDH
//! 2030: Adversary retroactively decrypts all stored traffic
//! ```
//!
//! ## What Data Is At Risk?
//!
//! | Data Type | Shelf Life | Risk Level |
//! |-----------|-----------|------------|
//! | Session cookies | Hours | LOW |
//! | Financial transactions | 7 years (regulatory) | MEDIUM |
//! | Medical records | Lifetime (50+ years) | HIGH |
//! | State secrets | 75+ years | CRITICAL |
//! | Infrastructure keys | 20+ years | HIGH |
//! | Personal identity data | Lifetime | HIGH |
//!
//! ## Risk Assessment Framework
//!
//! ```
//! Risk = Likelihood x Impact x Exposure_Time
//!
//! Likelihood: How likely is a quantum computer within the data's lifetime?
//! Impact: How damaging is disclosure?
//! Exposure_Time: How long must the data remain secret?
//! ```
//!
//! ## Timeline Estimates
//!
//! - NIST PQC standards finalized: 2024 (DONE)
//! - Early quantum advantage (specific problems): 2025-2030
//! - Cryptographically relevant quantum computer: 2030-2040 (estimates vary)
//! - Large-scale quantum computer: 2035-2050
//!
//! The uncertainty is why we must start NOW.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A data asset that needs quantum-safe protection.
#[derive(Debug, Clone)]
pub struct DataAsset {
    pub id: String,
    pub name: String,
    pub classification: Classification,
    pub shelf_life_years: u32,
    pub current_algorithm: String,
    pub data_volume_bytes: u64,
}

/// Data classification levels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Risk level for a PQC assessment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// A PQC risk assessment result.
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub asset_id: String,
    pub risk_level: RiskLevel,
    pub years_until_quantum: u32,
    pub migration_urgency: String,
    pub recommended_action: String,
}

/// Exercise 1: Calculate the risk level for a data asset.
///
/// Risk assessment rules:
/// - If shelf_life > years_until_quantum AND algorithm is quantum-vulnerable:
///   - TopSecret/Secret → Critical
///   - Confidential → High
///   - Internal → Medium
///   - Public → Low
/// - If shelf_life <= years_until_quantum → Low (data expires before quantum threat)
/// - If algorithm is already PQC → Low (already protected)
///
/// Quantum-vulnerable algorithms: RSA, DH, ECDH, ECDSA, DSA, ElGamal
pub fn assess_risk(
    asset: &DataAsset,
    years_until_quantum: u32,
) -> RiskLevel {
    todo!("Calculate risk level for a data asset")
}

/// Exercise 2: Generate a risk assessment report for a single asset.
///
/// Create a RiskAssessment struct with:
/// - risk_level: from assess_risk()
/// - migration_urgency: "IMMEDIATE" (Critical), "HIGH" (High), "PLANNED" (Medium), "LOW" (Low)
/// - recommended_action: specific recommendation based on risk level
///   - Critical: "Migrate to PQC immediately. Re-encrypt existing data."
///   - High: "Schedule PQC migration within 6 months."
///   - Medium: "Add to PQC migration roadmap. Target 12 months."
///   - Low: "No immediate action. Monitor quantum computing progress."
pub fn generate_assessment(
    asset: &DataAsset,
    years_until_quantum: u32,
) -> RiskAssessment {
    todo!("Generate risk assessment for a data asset")
}

/// Exercise 3: Batch-assess multiple assets and return sorted by risk.
///
/// Assess all assets and return them sorted by risk level (Critical first).
pub fn batch_assess(
    assets: &[DataAsset],
    years_until_quantum: u32,
) -> Vec<RiskAssessment> {
    todo!("Batch-assess and sort assets by risk level")
}

/// Exercise 4: Estimate the quantum timeline risk.
///
/// Given a current year and estimated quantum arrival year, compute:
/// - years_remaining: how many years until quantum computers arrive
/// - is_urgent: true if years_remaining < 5
/// - transition_budget_hint: recommended percentage of security budget for PQC
///   - < 3 years: 40%
///   - 3-7 years: 25%
///   - 7-15 years: 15%
///   - > 15 years: 5%
pub fn quantum_timeline_risk(
    current_year: u32,
    quantum_arrival_year: u32,
) -> (u32, bool, u32) {
    todo!("Estimate quantum timeline risk and budget allocation")
}

/// Exercise 5: Check if an algorithm is quantum-vulnerable.
///
/// Return true for: RSA, DH, ECDH, ECDSA, DSA, ElGamal
/// Return false for: ML-KEM, ML-DSA, SPHINCS+, AES, ChaCha20, SHA-256, SHA-3
pub fn is_quantum_vulnerable(algorithm: &str) -> bool {
    todo!("Check if algorithm is vulnerable to quantum attacks")
}

/// Exercise 6: Calculate the "crypto debt" of an organization.
///
/// Crypto debt = total bytes of data encrypted with quantum-vulnerable algorithms
/// that must remain secret beyond the quantum arrival date.
///
/// Given a list of assets and years_until_quantum, sum the data_volume_bytes
/// of all assets that are:
/// 1. Using quantum-vulnerable algorithms
/// 2. Have shelf_life > years_until_quantum
///
/// Return total vulnerable bytes.
pub fn calculate_crypto_debt(
    assets: &[DataAsset],
    years_until_quantum: u32,
) -> u64 {
    todo!("Calculate total crypto debt in bytes")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_asset(id: &str, alg: &str, class: Classification, shelf: u32) -> DataAsset {
        DataAsset {
            id: id.to_string(),
            name: format!("Asset {}", id),
            classification: class,
            shelf_life_years: shelf,
            current_algorithm: alg.to_string(),
            data_volume_bytes: 1000,
        }
    }

    #[test]
    fn test_assess_risk_critical() {
        let asset = make_asset("a1", "RSA", Classification::TopSecret, 50);
        assert_eq!(assess_risk(&asset, 10), RiskLevel::Critical);
    }

    #[test]
    fn test_assess_risk_high() {
        let asset = make_asset("a1", "ECDH-P256", Classification::Confidential, 30);
        assert_eq!(assess_risk(&asset, 10), RiskLevel::High);
    }

    #[test]
    fn test_assess_risk_low_expired() {
        let asset = make_asset("a1", "RSA", Classification::TopSecret, 5);
        // Shelf life (5) <= years_until_quantum (10) → data expires before quantum
        assert_eq!(assess_risk(&asset, 10), RiskLevel::Low);
    }

    #[test]
    fn test_assess_risk_low_pqc() {
        let asset = make_asset("a1", "ML-KEM-768", Classification::TopSecret, 50);
        // Already PQC → Low risk regardless
        assert_eq!(assess_risk(&asset, 10), RiskLevel::Low);
    }

    #[test]
    fn test_generate_assessment_urgency() {
        let asset = make_asset("a1", "RSA", Classification::TopSecret, 50);
        let assessment = generate_assessment(&asset, 10);
        assert_eq!(assessment.risk_level, RiskLevel::Critical);
        assert_eq!(assessment.migration_urgency, "IMMEDIATE");
    }

    #[test]
    fn test_batch_assess_sorted() {
        let assets = vec![
            make_asset("low", "ML-KEM-768", Classification::Public, 5),
            make_asset("crit", "RSA", Classification::TopSecret, 50),
            make_asset("high", "ECDH", Classification::Confidential, 30),
        ];
        let assessments = batch_assess(&assets, 10);
        assert_eq!(assessments[0].risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_quantum_timeline_urgent() {
        let (years, urgent, budget) = quantum_timeline_risk(2024, 2027);
        assert_eq!(years, 3);
        assert!(urgent);
        assert_eq!(budget, 40);
    }

    #[test]
    fn test_quantum_timeline_far() {
        let (years, urgent, budget) = quantum_timeline_risk(2024, 2045);
        assert_eq!(years, 21);
        assert!(!urgent);
        assert_eq!(budget, 5);
    }

    #[test]
    fn test_is_quantum_vulnerable() {
        assert!(is_quantum_vulnerable("RSA-2048"));
        assert!(is_quantum_vulnerable("ECDH-P256"));
        assert!(!is_quantum_vulnerable("ML-KEM-768"));
        assert!(!is_quantum_vulnerable("AES-256-GCM"));
    }

    #[test]
    fn test_calculate_crypto_debt() {
        let assets = vec![
            make_asset("a1", "RSA", Classification::TopSecret, 50),     // vulnerable, 50 > 10
            make_asset("a2", "ML-KEM-768", Classification::Secret, 50), // PQC, no debt
            make_asset("a3", "ECDH", Classification::Internal, 5),      // vulnerable, 5 <= 10
        ];
        let debt = calculate_crypto_debt(&assets, 10);
        assert_eq!(debt, 1000); // only a1 contributes
    }
}
