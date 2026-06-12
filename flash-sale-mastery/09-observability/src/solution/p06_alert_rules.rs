//! # Solution 06: Alert Rules
//!
//! Complete implementation of alert rule definitions and evaluation.

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
    pub stock_level: i64,
    pub error_rate: f64,
    pub latency_p99_ms: f64,
    pub redis_healthy: bool,
    pub oversell_count: i64,
}

/// An alert rule definition.
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub severity: Severity,
    pub message: String,
}

/// A fired alert with context.
#[derive(Debug, Clone)]
pub struct Alert {
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.rule_name, self.message)
    }
}

/// Create the default set of flash sale alert rules.
pub fn create_default_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "stock_below_threshold".to_string(),
            severity: Severity::Warning,
            message: "Stock level is critically low ({stock_level} remaining)".to_string(),
        },
        AlertRule {
            name: "error_rate_exceeded".to_string(),
            severity: Severity::Critical,
            message: "Error rate ({error_rate}%) exceeds 5% threshold".to_string(),
        },
        AlertRule {
            name: "latency_exceeded".to_string(),
            severity: Severity::Warning,
            message: "P99 latency ({latency_ms}ms) exceeds 500ms SLA".to_string(),
        },
        AlertRule {
            name: "redis_down".to_string(),
            severity: Severity::Critical,
            message: "Redis is unreachable -- all operations failing".to_string(),
        },
        AlertRule {
            name: "oversell_detected".to_string(),
            severity: Severity::Critical,
            message: "Oversell detected: {oversell_count} items sold beyond stock".to_string(),
        },
    ]
}

/// Evaluate alert rules against a metric snapshot.
pub fn evaluate_rules(rules: &[AlertRule], snapshot: &MetricSnapshot) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for rule in rules {
        let triggered = match rule.name.as_str() {
            "stock_below_threshold" => snapshot.stock_level < 10,
            "error_rate_exceeded" => snapshot.error_rate > 0.05,
            "latency_exceeded" => snapshot.latency_p99_ms > 500.0,
            "redis_down" => !snapshot.redis_healthy,
            "oversell_detected" => snapshot.oversell_count > 0,
            _ => false,
        };

        if triggered {
            let message = rule
                .message
                .replace("{stock_level}", &snapshot.stock_level.to_string())
                .replace("{error_rate}", &format!("{:.1}", snapshot.error_rate * 100.0))
                .replace("{latency_ms}", &format!("{:.0}", snapshot.latency_p99_ms))
                .replace("{oversell_count}", &snapshot.oversell_count.to_string());

            alerts.push(Alert {
                rule_name: rule.name.clone(),
                severity: rule.severity,
                message,
            });
        }
    }

    alerts
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
        snapshot.error_rate = 0.10;
        let alerts = evaluate_rules(&rules, &snapshot);
        let error_alert = alerts.iter().find(|a| a.rule_name == "error_rate_exceeded");
        assert!(error_alert.is_some(), "Should trigger error rate alert");
        assert_eq!(error_alert.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_latency_exceeded() {
        let rules = create_default_rules();
        let mut snapshot = healthy_snapshot();
        snapshot.latency_p99_ms = 600.0;
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
        assert!(alerts.len() >= 3, "Should fire at least 3 alerts for degraded state");
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
