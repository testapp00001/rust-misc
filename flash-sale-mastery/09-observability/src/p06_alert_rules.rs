//! # Exercise 06: Alert Rules
//!
//! ## Learning Objective
//! Learn how to define and evaluate alert rules based on metric thresholds.
//! Alerts turn raw metrics into actionable notifications. A well-designed alert
//! system reduces noise and ensures operators respond to real problems.
//!
//! ## Flash Sale Context
//! During a flash sale, you need immediate alerts when: stock runs critically
//! low, error rates spike, latency exceeds SLA, Redis goes down, or an oversell
//! is detected. Each alert needs a severity level to determine notification
//! routing.
//!
//! ## Instructions
//! 1. Implement the `AlertRule` and `Alert` structs
//! 2. Implement `create_default_rules` to define the 5 standard alert rules
//! 3. Implement `evaluate_rules` to check rules against current metric values
//!
//! ## Hints
//! - Use an enum for severity: Info, Warning, Critical
//! - Each rule has a condition function that takes metric values and returns bool
//! - `evaluate_rules` returns only the alerts whose conditions are met

use std::fmt;

/// Severity level for alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Snapshot of current metric values for rule evaluation.
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    /// Current stock level for the product being evaluated.
    pub stock_level: i64,
    /// Current error rate as a fraction (0.0 to 1.0).
    pub error_rate: f64,
    /// Current p99 latency in milliseconds.
    pub latency_p99_ms: f64,
    /// Whether Redis is currently reachable.
    pub redis_healthy: bool,
    /// Total items sold minus total stock decrements (positive = oversell).
    pub oversell_count: i64,
}

/// An alert rule definition.
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Human-readable name for the rule.
    pub name: String,
    /// Severity level when triggered.
    pub severity: Severity,
    /// Template message for the alert (may contain placeholders).
    pub message: String,
}

/// A fired alert with context.
#[derive(Debug, Clone)]
pub struct Alert {
    /// The rule that triggered this alert.
    pub rule_name: String,
    /// Severity level.
    pub severity: Severity,
    /// The rendered alert message.
    pub message: String,
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.rule_name, self.message)
    }
}

/// Create the default set of flash sale alert rules.
///
/// Returns 5 rules:
/// 1. `stock_below_threshold` - Warning when stock < 10
/// 2. `error_rate_exceeded` - Critical when error rate > 5%
/// 3. `latency_exceeded` - Warning when p99 latency > 500ms
/// 4. `redis_down` - Critical when Redis is unhealthy
/// 5. `oversell_detected` - Critical when oversell count > 0
///
/// # Returns
/// A vector of `AlertRule` definitions.
pub fn create_default_rules() -> Vec<AlertRule> {
    // TODO: Create 5 AlertRule instances with appropriate names, severities, and messages
    todo!("Implement default alert rules")
}

/// Evaluate alert rules against a metric snapshot.
///
/// Checks each rule's condition against the snapshot and returns alerts
/// for rules whose conditions are met.
///
/// # Arguments
/// * `rules` - The alert rules to evaluate
/// * `snapshot` - Current metric values
///
/// # Returns
/// A vector of triggered `Alert` instances.
pub fn evaluate_rules(rules: &[AlertRule], snapshot: &MetricSnapshot) -> Vec<Alert> {
    // TODO: For each rule, check if its condition is met
    // TODO: If met, create an Alert with the rule's name, severity, and message
    // TODO: Return only triggered alerts
    todo!("Implement rule evaluation")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_snapshot() -> MetricSnapshot {
        MetricSnapshot {
            stock_level: 100,
            error_rate: 0.01,
            latency_p99_ms: 50.0,
            redis_healthy: true,
            oversell_count: 0,
        }
    }

    #[test]
    fn test_create_default_rules_count() {
        let rules = create_default_rules();
        assert_eq!(rules.len(), 5, "Should have 5 default rules");
    }

    #[test]
    fn test_healthy_snapshot_no_alerts() {
        let rules = create_default_rules();
        let snapshot = healthy_snapshot();
        let alerts = evaluate_rules(&rules, &snapshot);
        assert!(alerts.is_empty(), "Healthy system should not trigger alerts");
    }

    #[test]
    fn test_stock_below_threshold() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.stock_level = 5;
        let alerts = evaluate_rules(&rules, &snapshot);
        let stock_alert = alerts.iter().find(|a| a.rule_name == "stock_below_threshold");
        assert!(stock_alert.is_some(), "Should trigger stock alert");
        assert_eq!(stock_alert.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn test_error_rate_exceeded() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.error_rate = 0.10; // 10% > 5% threshold
        let alerts = evaluate_rules(&rules, &snapshot);
        let error_alert = alerts.iter().find(|a| a.rule_name == "error_rate_exceeded");
        assert!(error_alert.is_some(), "Should trigger error rate alert");
        assert_eq!(error_alert.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_latency_exceeded() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.latency_p99_ms = 600.0; // > 500ms threshold
        let alerts = evaluate_rules(&rules, &snapshot);
        let latency_alert = alerts.iter().find(|a| a.rule_name == "latency_exceeded");
        assert!(latency_alert.is_some(), "Should trigger latency alert");
    }

    #[test]
    fn test_redis_down() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.redis_healthy = false;
        let alerts = evaluate_rules(&rules, &snapshot);
        let redis_alert = alerts.iter().find(|a| a.rule_name == "redis_down");
        assert!(redis_alert.is_some(), "Should trigger Redis down alert");
        assert_eq!(redis_alert.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_oversell_detected() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.oversell_count = 3;
        let alerts = evaluate_rules(&rules, &snapshot);
        let oversell_alert = alerts.iter().find(|a| a.rule_name == "oversell_detected");
        assert!(oversell_alert.is_some(), "Should trigger oversell alert");
        assert_eq!(oversell_alert.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_multiple_alerts_fire_simultaneously() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.stock_level = 3;
        snapshot.error_rate = 0.15;
        snapshot.redis_healthy = false;
        let alerts = evaluate_rules(&rules, &snapshot);
        assert!(
            alerts.len() >= 3,
            "Should fire at least 3 alerts for degraded state"
        );
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Info), "INFO");
        assert_eq!(format!("{}", Severity::Warning), "WARNING");
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
    }

    #[test]
    fn test_alert_display() {
        let alert = Alert {
            rule_name: "test_rule".to_string(),
            severity: Severity::Critical,
            message: "Something broke".to_string(),
        };
        let display = format!("{alert}");
        assert!(display.contains("CRITICAL"));
        assert!(display.contains("test_rule"));
        assert!(display.contains("Something broke"));
    }
}
