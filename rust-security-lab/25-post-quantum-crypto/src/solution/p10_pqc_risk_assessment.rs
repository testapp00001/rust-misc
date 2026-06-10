//! # Lesson 10: PQC Risk Assessment — "Harvest Now, Decrypt Later" (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Check if an algorithm is quantum-vulnerable.
pub fn is_quantum_vulnerable(algorithm: &str) -> bool {
    let vulnerable = ["RSA", "DH", "ECDH", "ECDSA", "DSA", "ElGamal"];
    vulnerable.iter().any(|&v| algorithm.contains(v))
}

/// Assess risk for a data asset.
pub fn assess_risk(
    asset: &DataAsset,
    years_until_quantum: u32,
) -> RiskLevel {
    // Already PQC-protected → Low
    if !is_quantum_vulnerable(&asset.current_algorithm) {
        return RiskLevel::Low;
    }

    // Data expires before quantum arrives → Low
    if asset.shelf_life_years <= years_until_quantum {
        return RiskLevel::Low;
    }

    // Data must outlive the quantum threat — classify by sensitivity
    match asset.classification {
        Classification::TopSecret | Classification::Secret => RiskLevel::Critical,
        Classification::Confidential => RiskLevel::High,
        Classification::Internal => RiskLevel::Medium,
        Classification::Public => RiskLevel::Low,
    }
}

/// Generate a risk assessment report.
pub fn generate_assessment(
    asset: &DataAsset,
    years_until_quantum: u32,
) -> RiskAssessment {
    let risk_level = assess_risk(asset, years_until_quantum);

    let migration_urgency = match risk_level {
        RiskLevel::Critical => "IMMEDIATE",
        RiskLevel::High => "HIGH",
        RiskLevel::Medium => "PLANNED",
        RiskLevel::Low => "LOW",
    };

    let recommended_action = match risk_level {
        RiskLevel::Critical => "Migrate to PQC immediately. Re-encrypt existing data.",
        RiskLevel::High => "Schedule PQC migration within 6 months.",
        RiskLevel::Medium => "Add to PQC migration roadmap. Target 12 months.",
        RiskLevel::Low => "No immediate action. Monitor quantum computing progress.",
    };

    RiskAssessment {
        asset_id: asset.id.clone(),
        risk_level,
        years_until_quantum,
        migration_urgency: migration_urgency.to_string(),
        recommended_action: recommended_action.to_string(),
    }
}

/// Batch-assess and sort by risk level (Critical first).
pub fn batch_assess(
    assets: &[DataAsset],
    years_until_quantum: u32,
) -> Vec<RiskAssessment> {
    let mut assessments: Vec<RiskAssessment> = assets
        .iter()
        .map(|a| generate_assessment(a, years_until_quantum))
        .collect();

    assessments.sort_by(|a, b| b.risk_level.cmp(&a.risk_level));
    assessments
}

/// Estimate quantum timeline risk.
pub fn quantum_timeline_risk(
    current_year: u32,
    quantum_arrival_year: u32,
) -> (u32, bool, u32) {
    let years_remaining = quantum_arrival_year.saturating_sub(current_year);
    let is_urgent = years_remaining < 5;

    let budget_hint = if years_remaining < 3 {
        40
    } else if years_remaining < 7 {
        25
    } else if years_remaining < 15 {
        15
    } else {
        5
    };

    (years_remaining, is_urgent, budget_hint)
}

/// Calculate total crypto debt: bytes of quantum-vulnerable data
/// that must outlive the quantum arrival.
pub fn calculate_crypto_debt(
    assets: &[DataAsset],
    years_until_quantum: u32,
) -> u64 {
    assets
        .iter()
        .filter(|a| {
            is_quantum_vulnerable(&a.current_algorithm)
                && a.shelf_life_years > years_until_quantum
        })
        .map(|a| a.data_volume_bytes)
        .sum()
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
        assert_eq!(assess_risk(&asset, 10), RiskLevel::Low);
    }

    #[test]
    fn test_assess_risk_low_pqc() {
        let asset = make_asset("a1", "ML-KEM-768", Classification::TopSecret, 50);
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
            make_asset("a1", "RSA", Classification::TopSecret, 50),
            make_asset("a2", "ML-KEM-768", Classification::Secret, 50),
            make_asset("a3", "ECDH", Classification::Internal, 5),
        ];
        let debt = calculate_crypto_debt(&assets, 10);
        assert_eq!(debt, 1000);
    }
}
