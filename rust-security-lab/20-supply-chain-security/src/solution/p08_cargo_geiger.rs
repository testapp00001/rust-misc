//! # Lesson 08: cargo-geiger — Unsafe Code Audit (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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
    pub fn total(&self) -> u32 {
        self.safe_expressions + self.unsafe_expressions
    }

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
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn numeric(&self) -> u8 {
        match self {
            RiskLevel::Safe => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
}

/// Classify a crate's risk level based on its metrics.
pub fn classify_risk(metrics: &CrateMetrics) -> RiskLevel {
    if metrics.unsafe_expressions == 0 {
        return RiskLevel::Safe;
    }

    let ratio = metrics.unsafe_ratio();

    let base = if ratio < 0.1 {
        RiskLevel::Low
    } else if ratio < 0.3 {
        RiskLevel::Medium
    } else {
        RiskLevel::High
    };

    // Upgrade to Critical if High + build script or proc macro
    if base == RiskLevel::High && (metrics.has_build_script || metrics.has_proc_macro) {
        RiskLevel::Critical
    } else {
        base
    }
}

/// Filter crates by risk threshold.
pub fn filter_by_risk<'a>(
    metrics: &'a [CrateMetrics],
    min_risk: &RiskLevel,
) -> Vec<(&'a CrateMetrics, RiskLevel)> {
    let threshold = min_risk.numeric();
    metrics
        .iter()
        .map(|m| (m, classify_risk(m)))
        .filter(|(_, risk)| risk.numeric() >= threshold)
        .collect()
}

/// Calculate overall unsafe code score for the entire dependency tree.
pub fn overall_unsafe_score(metrics: &[CrateMetrics]) -> f64 {
    let total_unsafe: u32 = metrics.iter().map(|m| m.unsafe_expressions).sum();
    let total_all: u32 = metrics.iter().map(|m| m.total()).sum();
    if total_all == 0 {
        0.0
    } else {
        total_unsafe as f64 / total_all as f64
    }
}

/// Find the riskiest crate (highest unsafe ratio, with Critical tiebreaker).
pub fn find_riskiest(metrics: &[CrateMetrics]) -> Option<(String, RiskLevel)> {
    metrics
        .iter()
        .map(|m| (m, classify_risk(m)))
        .max_by(|(a, risk_a), (b, risk_b)| {
            risk_a
                .numeric()
                .cmp(&risk_b.numeric())
                .then_with(|| {
                    a.unsafe_ratio()
                        .partial_cmp(&b.unsafe_ratio())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .map(|(m, risk)| (m.name.clone(), risk))
}

/// Generate an audit report in text format.
pub fn generate_audit_report(metrics: &[CrateMetrics]) -> String {
    let entries: Vec<String> = metrics
        .iter()
        .map(|m| {
            let risk = classify_risk(m);
            let ratio_pct = m.unsafe_ratio() * 100.0;
            format!(
                "[{:?}] {} v{}: {}/{} unsafe ({:.1}%)",
                risk, m.name, m.version, m.unsafe_expressions, m.total(), ratio_pct
            )
        })
        .collect();

    let total = metrics.len();
    let with_unsafe = metrics.iter().filter(|m| m.unsafe_expressions > 0).count();
    let score = overall_unsafe_score(metrics) * 100.0;

    let mut report = entries.join("\n");
    report.push_str(&format!(
        "\nTotal: {} crates, {} with unsafe code, overall score: {:.1}%",
        total, with_unsafe, score
    ));
    report
}

/// Recommend crates for replacement based on risk.
pub fn recommend_replacements(metrics: &[CrateMetrics]) -> Vec<String> {
    metrics
        .iter()
        .filter(|m| {
            let risk = classify_risk(m);
            (risk == RiskLevel::High || risk == RiskLevel::Critical) && m.unsafe_expressions >= 50
        })
        .map(|m| m.name.clone())
        .collect()
}

/// Serialize metrics to JSON.
pub fn serialize_metrics(metrics: &[CrateMetrics]) -> String {
    serde_json::to_string_pretty(metrics).unwrap_or_else(|_| "[]".to_string())
}

/// Deserialize metrics from JSON.
pub fn deserialize_metrics(json: &str) -> Option<Vec<CrateMetrics>> {
    serde_json::from_str(json).ok()
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
        assert_eq!(results.len(), 2);
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
