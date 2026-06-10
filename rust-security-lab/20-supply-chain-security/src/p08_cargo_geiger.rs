//! # Lesson 08: cargo-geiger — Unsafe Code Audit
//!
//! ## What is cargo-geiger?
//!
//! `cargo-geiger` scans your dependency tree and reports the amount of `unsafe` code
//! in each crate. It counts both `unsafe` blocks in application code and in
//! transitive dependencies.
//!
//! ## Why Audit Unsafe Code?
//!
//! `unsafe` in Rust bypasses the borrow checker's guarantees. While sometimes necessary,
//! it is the primary source of memory safety bugs:
//!
//! - **Use-after-free**: Accessing freed memory
//! - **Buffer overflows**: Writing past allocation boundaries
//! - **Data races**: Concurrent unsynchronized access
//! - **Type confusion**: Invalid transmutes or pointer casts
//!
//! A crate with lots of `unsafe` code has a higher attack surface than one that
//! is 100% safe Rust.
//!
//! ## Metrics
//!
//! | Metric | Meaning |
//! |--------|---------|
//! | `unsafe` expressions | Raw count of `unsafe` blocks |
//! | Safe expressions | Count of safe code blocks |
//! | Unsafe ratio | `unsafe / total` — higher = more risk |
//!
//! ## Attack: Unsafe Code Exploitation
//!
//! A vulnerability in an `unsafe` block of a dependency can be exploited remotely.
//! For example, a buffer overflow in a parsing library can lead to RCE.
//!
//! ## Defense: Unsafe Code Analysis
//!
//! In this lesson, you will implement unsafe code detection, risk scoring,
//! and audit report generation.

use serde::{Deserialize, Serialize};

/// Represents the unsafe code metrics for a single crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMetrics {
    pub name: String,
    pub version: String,
    pub unsafe_expressions: u32,
    pub safe_expressions: u32,
    pub has_build_script: bool,
    pub has_proc_macro: bool,
}

impl CrateMetrics {
    /// Total number of expressions (safe + unsafe).
    pub fn total(&self) -> u32 {
        self.safe_expressions + self.unsafe_expressions
    }

    /// Ratio of unsafe to total expressions (0.0 = all safe, 1.0 = all unsafe).
    pub fn unsafe_ratio(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.unsafe_expressions as f64 / self.total() as f64
        }
    }
}

/// Risk level for a crate based on unsafe code metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No unsafe code at all.
    Safe,
    /// Some unsafe code but low ratio.
    Low,
    /// Moderate amount of unsafe code.
    Medium,
    /// High ratio of unsafe code.
    High,
    /// Critical: high unsafe ratio + build script or proc macro.
    Critical,
}

/// Exercise 1: Classify a crate's risk level based on its metrics.
///
/// Rules:
/// - Safe: 0 unsafe expressions
/// - Low: unsafe_ratio < 0.1 (less than 10%)
/// - Medium: unsafe_ratio < 0.3 (less than 30%)
/// - High: unsafe_ratio >= 0.3
/// - Critical: High + (has_build_script OR has_proc_macro)
///
/// Hints:
/// - Check safe case first (0 unsafe)
/// - Calculate unsafe_ratio
/// - Apply thresholds in order
/// - Check for Critical upgrade condition
pub fn classify_risk(metrics: &CrateMetrics) -> RiskLevel {
    todo!("Classify crate risk level from metrics")
}

/// Exercise 2: Scan a list of crate metrics and return those above a risk threshold.
///
/// Only include crates whose risk level >= the given threshold.
/// Use numeric comparison of risk levels (Safe=0, Low=1, Medium=2, High=3, Critical=4).
///
/// Hints:
/// - Iterate, classify each, compare levels
pub fn filter_by_risk<'a>(
    metrics: &'a [CrateMetrics],
    min_risk: &RiskLevel,
) -> Vec<(&'a CrateMetrics, RiskLevel)> {
    todo!("Filter crates by risk threshold")
}

/// Exercise 3: Calculate the overall unsafe code score for the entire dependency tree.
///
/// Score = (total unsafe expressions across all crates) / (total expressions across all crates)
///
/// Return 0.0 if no expressions exist.
///
/// Hints:
/// - Sum all unsafe_expressions and all total() across crates
/// - Divide
pub fn overall_unsafe_score(metrics: &[CrateMetrics]) -> f64 {
    todo!("Calculate overall unsafe score")
}

/// Exercise 4: Find the "riskiest" crate (highest unsafe ratio, with Critical tiebreaker).
///
/// Return the crate name and its risk level.
///
/// Hints:
/// - Iterate, classify each, compare by ratio then by risk level
/// - Use `partial_cmp` for f64 comparison
pub fn find_riskiest(metrics: &[CrateMetrics]) -> Option<(String, RiskLevel)> {
    todo!("Find the riskiest crate")
}

/// Exercise 5: Generate an audit report in text format.
///
/// For each crate, output one line:
/// `[{RISK}] name v{version}: {unsafe}/{total} unsafe ({ratio}%)`
///
/// Followed by a summary line:
/// `Total: {n} crates, {n} with unsafe code, overall score: {score:.1}%`
///
/// Hints:
/// - Iterate, classify each, format each line
/// - Use `format!()` with percentage formatting
pub fn generate_audit_report(metrics: &[CrateMetrics]) -> String {
    todo!("Generate unsafe code audit report")
}

/// Exercise 6: Recommend crates for replacement based on risk.
///
/// Return names of crates that are High or Critical risk
/// AND have at least 50 unsafe expressions.
///
/// These are the most dangerous dependencies that should be
/// replaced or heavily scrutinized.
///
/// Hints:
/// - Filter by risk level and unsafe count
pub fn recommend_replacements(metrics: &[CrateMetrics]) -> Vec<String> {
    todo!("Recommend crates for replacement")
}

/// Exercise 7: Serialize metrics to JSON.
///
/// Hints:
/// - Use `serde_json::to_string_pretty`
pub fn serialize_metrics(metrics: &[CrateMetrics]) -> String {
    todo!("Serialize crate metrics to JSON")
}

/// Exercise 8: Deserialize metrics from JSON.
///
/// Hints:
/// - Use `serde_json::from_str`
pub fn deserialize_metrics(json: &str) -> Option<Vec<CrateMetrics>> {
    todo!("Deserialize crate metrics from JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_crate(name: &str) -> CrateMetrics {
        CrateMetrics {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            unsafe_expressions: 0,
            safe_expressions: 100,
            has_build_script: false,
            has_proc_macro: false,
        }
    }

    fn medium_crate(name: &str) -> CrateMetrics {
        CrateMetrics {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            unsafe_expressions: 20,
            safe_expressions: 80,
            has_build_script: false,
            has_proc_macro: false,
        }
    }

    fn critical_crate(name: &str) -> CrateMetrics {
        CrateMetrics {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            unsafe_expressions: 90,
            safe_expressions: 10,
            has_build_script: true,
            has_proc_macro: false,
        }
    }

    #[test]
    fn test_classify_risk_safe() {
        assert_eq!(classify_risk(&safe_crate("serde")), RiskLevel::Safe);
    }

    #[test]
    fn test_classify_risk_low() {
        let m = CrateMetrics {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            unsafe_expressions: 5,
            safe_expressions: 95,
            has_build_script: false,
            has_proc_macro: false,
        };
        assert_eq!(classify_risk(&m), RiskLevel::Low);
    }

    #[test]
    fn test_classify_risk_medium() {
        assert_eq!(classify_risk(&medium_crate("tokio")), RiskLevel::Medium);
    }

    #[test]
    fn test_classify_risk_high() {
        let m = CrateMetrics {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            unsafe_expressions: 40,
            safe_expressions: 60,
            has_build_script: false,
            has_proc_macro: false,
        };
        assert_eq!(classify_risk(&m), RiskLevel::High);
    }

    #[test]
    fn test_classify_risk_critical() {
        assert_eq!(classify_risk(&critical_crate("ring")), RiskLevel::Critical);
    }

    #[test]
    fn test_filter_by_risk() {
        let metrics = vec![
            safe_crate("serde"),
            medium_crate("tokio"),
            critical_crate("ring"),
        ];
        let results = filter_by_risk(&metrics, &RiskLevel::Medium);
        assert_eq!(results.len(), 2); // medium + critical
    }

    #[test]
    fn test_overall_unsafe_score_zero() {
        let metrics = vec![safe_crate("a"), safe_crate("b")];
        let score = overall_unsafe_score(&metrics);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_overall_unsafe_score_nonzero() {
        let metrics = vec![safe_crate("a"), medium_crate("b")];
        let score = overall_unsafe_score(&metrics);
        assert!(score > 0.0);
        // 20 unsafe / 200 total = 0.1 = 10%
        assert!((score - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_riskiest() {
        let metrics = vec![
            safe_crate("serde"),
            medium_crate("tokio"),
            critical_crate("ring"),
        ];
        let (name, risk) = find_riskiest(&metrics).unwrap();
        assert_eq!(name, "ring");
        assert_eq!(risk, RiskLevel::Critical);
    }

    #[test]
    fn test_find_riskiest_empty() {
        assert!(find_riskiest(&[]).is_none());
    }

    #[test]
    fn test_generate_audit_report() {
        let metrics = vec![safe_crate("serde"), medium_crate("tokio")];
        let report = generate_audit_report(&metrics);
        assert!(report.contains("serde"));
        assert!(report.contains("tokio"));
        assert!(report.contains("Total:"));
    }

    #[test]
    fn test_recommend_replacements() {
        let metrics = vec![
            safe_crate("serde"),
            critical_crate("ring"),
        ];
        let recs = recommend_replacements(&metrics);
        assert_eq!(recs, vec!["ring"]);
    }

    #[test]
    fn test_recommend_replacements_none() {
        let metrics = vec![safe_crate("serde")];
        let recs = recommend_replacements(&metrics);
        assert!(recs.is_empty());
    }

    #[test]
    fn test_metrics_roundtrip() {
        let metrics = vec![safe_crate("serde"), critical_crate("ring")];
        let json = serialize_metrics(&metrics);
        let restored = deserialize_metrics(&json).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].name, "serde");
        assert_eq!(restored[1].name, "ring");
    }
}
