//! # Lesson 10: Production Observability
//!
//! Production systems need structured logging, alerting, and dashboards.
//! This lesson covers log aggregation patterns, structured JSON output,
//! log levels strategy, and operational dashboards.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Structured JSON log output
// ---------------------------------------------------------------------------

/// A production log entry in JSON format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub request_id: Option<String>,
    pub caller: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl JsonLogEntry {
    pub fn new(
        level: &str,
        message: &str,
        service: &str,
        version: &str,
        environment: &str,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            service: service.to_string(),
            version: version.to_string(),
            environment: environment.to_string(),
            trace_id: None,
            span_id: None,
            request_id: None,
            caller: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_trace(mut self, trace_id: &str, span_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self.span_id = Some(span_id.to_string());
        self
    }

    pub fn with_request_id(mut self, id: &str) -> Self {
        self.request_id = Some(id.to_string());
        self
    }

    pub fn with_caller(mut self, caller: &str) -> Self {
        self.caller = Some(caller.to_string());
        self
    }

    pub fn with_field(mut self, key: &str, value: serde_json::Value) -> Self {
        self.extra.insert(key.to_string(), value);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Log level strategy
// ---------------------------------------------------------------------------

/// Guidelines for when to use each log level in production.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevelStrategy {
    pub entries: Vec<LogLevelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevelEntry {
    pub level: String,
    pub when_to_use: String,
    pub examples: Vec<String>,
    pub alert: bool,
    pub retention_days: u32,
}

impl LogLevelStrategy {
    pub fn standard() -> Self {
        Self {
            entries: vec![
                LogLevelEntry {
                    level: "ERROR".into(),
                    when_to_use: "Something failed that requires attention".into(),
                    examples: vec![
                        "Database connection failed".into(),
                        "Unhandled exception".into(),
                        "External service unreachable".into(),
                    ],
                    alert: true,
                    retention_days: 90,
                },
                LogLevelEntry {
                    level: "WARN".into(),
                    when_to_use: "Something unexpected but recoverable".into(),
                    examples: vec![
                        "Retry attempt".into(),
                        "Deprecated API usage".into(),
                        "Configuration fallback".into(),
                    ],
                    alert: false,
                    retention_days: 30,
                },
                LogLevelEntry {
                    level: "INFO".into(),
                    when_to_use: "Significant business events".into(),
                    examples: vec![
                        "User logged in".into(),
                        "Order placed".into(),
                        "Service started".into(),
                    ],
                    alert: false,
                    retention_days: 14,
                },
                LogLevelEntry {
                    level: "DEBUG".into(),
                    when_to_use: "Detailed diagnostic info for troubleshooting".into(),
                    examples: vec![
                        "Query parameters".into(),
                        "Cache hit/miss".into(),
                        "Configuration loaded".into(),
                    ],
                    alert: false,
                    retention_days: 7,
                },
                LogLevelEntry {
                    level: "TRACE".into(),
                    when_to_use: "Very detailed, high-volume data".into(),
                    examples: vec![
                        "Individual function calls".into(),
                        "Loop iterations".into(),
                        "Raw request/response bodies".into(),
                    ],
                    alert: false,
                    retention_days: 1,
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Alert rules
// ---------------------------------------------------------------------------

/// An alert rule for production monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ErrorRateAbove { threshold_percent: f64, window_minutes: u32 },
    LatencyAbove { threshold_ms: u64, percentile: u8 },
    LogPatternMatch { level: String, pattern: String, count: u32 },
    ServiceDown { check_interval_secs: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl AlertRule {
    pub fn high_error_rate() -> Self {
        Self {
            name: "high_error_rate".into(),
            description: "Error rate exceeds threshold".into(),
            condition: AlertCondition::ErrorRateAbove {
                threshold_percent: 5.0,
                window_minutes: 5,
            },
            severity: AlertSeverity::Critical,
            channels: vec!["pagerduty".into(), "slack".into()],
        }
    }

    pub fn high_latency() -> Self {
        Self {
            name: "high_latency".into(),
            description: "P99 latency exceeds threshold".into(),
            condition: AlertCondition::LatencyAbove {
                threshold_ms: 1000,
                percentile: 99,
            },
            severity: AlertSeverity::Warning,
            channels: vec!["slack".into()],
        }
    }

    pub fn service_down() -> Self {
        Self {
            name: "service_down".into(),
            description: "Service health check failing".into(),
            condition: AlertCondition::ServiceDown {
                check_interval_secs: 30,
            },
            severity: AlertSeverity::Critical,
            channels: vec!["pagerduty".into()],
        }
    }
}

// ---------------------------------------------------------------------------
// Dashboard model
// ---------------------------------------------------------------------------

/// A monitoring dashboard configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub name: String,
    pub description: String,
    pub panels: Vec<Panel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub title: String,
    pub panel_type: PanelType,
    pub query: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    TimeSeries,
    Gauge,
    Counter,
    Table,
    Heatmap,
}

impl Dashboard {
    pub fn request_overview() -> Self {
        Self {
            name: "Request Overview".into(),
            description: "High-level request metrics".into(),
            panels: vec![
                Panel {
                    title: "Request Rate".into(),
                    panel_type: PanelType::TimeSeries,
                    query: "rate(http_requests_total[5m])".into(),
                    unit: "req/s".into(),
                },
                Panel {
                    title: "Error Rate".into(),
                    panel_type: PanelType::TimeSeries,
                    query: "rate(http_errors_total[5m])".into(),
                    unit: "%".into(),
                },
                Panel {
                    title: "Active Requests".into(),
                    panel_type: PanelType::Gauge,
                    query: "http_active_requests".into(),
                    unit: "count".into(),
                },
                Panel {
                    title: "P99 Latency".into(),
                    panel_type: PanelType::TimeSeries,
                    query: "histogram_quantile(0.99, http_request_duration_ms)".into(),
                    unit: "ms".into(),
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// A health check endpoint response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: Vec<ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

impl HealthCheck {
    pub fn healthy(version: &str, uptime: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            version: version.to_string(),
            uptime_secs: uptime,
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, check: ComponentHealth) {
        if check.status == HealthStatus::Unhealthy {
            self.status = HealthStatus::Unhealthy;
        } else if check.status == HealthStatus::Degraded && self.status == HealthStatus::Healthy {
            self.status = HealthStatus::Degraded;
        }
        self.checks.push(check);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_log_entry() {
        let entry = JsonLogEntry::new("INFO", "user logged in", "api", "1.0.0", "prod")
            .with_request_id("req-123")
            .with_trace("trace-abc", "span-def")
            .with_caller("auth::login")
            .with_field("user_id", serde_json::json!(42));

        let json = entry.to_json();
        assert!(json.contains("INFO"));
        assert!(json.contains("user logged in"));
        assert!(json.contains("req-123"));
        assert!(json.contains("trace-abc"));
    }

    #[test]
    fn test_json_log_entry_serialization() {
        let entry = JsonLogEntry::new("ERROR", "failed", "svc", "1.0", "dev");
        let json = entry.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["level"], "ERROR");
        assert_eq!(parsed["service"], "svc");
    }

    #[test]
    fn test_log_level_strategy() {
        let strategy = LogLevelStrategy::standard();
        assert_eq!(strategy.entries.len(), 5);

        let error = strategy.entries.iter().find(|e| e.level == "ERROR").unwrap();
        assert!(error.alert);
        assert_eq!(error.retention_days, 90);

        let debug = strategy.entries.iter().find(|e| e.level == "DEBUG").unwrap();
        assert!(!debug.alert);
        assert_eq!(debug.retention_days, 7);
    }

    #[test]
    fn test_alert_rules() {
        let high_err = AlertRule::high_error_rate();
        assert_eq!(high_err.severity, AlertSeverity::Critical);
        assert!(high_err.channels.contains(&"pagerduty".to_string()));

        let latency = AlertRule::high_latency();
        assert_eq!(latency.severity, AlertSeverity::Warning);

        let down = AlertRule::service_down();
        assert_eq!(down.severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_dashboard() {
        let dashboard = Dashboard::request_overview();
        assert_eq!(dashboard.name, "Request Overview");
        assert_eq!(dashboard.panels.len(), 4);

        let rate_panel = dashboard.panels.iter().find(|p| p.title == "Request Rate").unwrap();
        assert!(rate_panel.query.contains("rate"));
    }

    #[test]
    fn test_health_check_healthy() {
        let health = HealthCheck::healthy("1.0.0", 3600);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.version, "1.0.0");
        assert_eq!(health.uptime_secs, 3600);
    }

    #[test]
    fn test_health_check_degraded() {
        let mut health = HealthCheck::healthy("1.0.0", 100);
        health.add_check(ComponentHealth {
            name: "database".into(),
            status: HealthStatus::Degraded,
            latency_ms: Some(500),
            message: Some("high latency".into()),
        });
        assert_eq!(health.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_check_unhealthy() {
        let mut health = HealthCheck::healthy("1.0.0", 100);
        health.add_check(ComponentHealth {
            name: "database".into(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            message: Some("connection refused".into()),
        });
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_multiple() {
        let mut health = HealthCheck::healthy("1.0.0", 100);
        health.add_check(ComponentHealth {
            name: "db".into(),
            status: HealthStatus::Healthy,
            latency_ms: Some(10),
            message: None,
        });
        health.add_check(ComponentHealth {
            name: "cache".into(),
            status: HealthStatus::Degraded,
            latency_ms: Some(200),
            message: Some("slow".into()),
        });
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.checks.len(), 2);
    }

    #[test]
    fn test_health_check_json() {
        let health = HealthCheck::healthy("1.0.0", 100);
        let json = health.to_json();
        assert!(json.contains("Healthy"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_alert_condition_variants() {
        let rule = AlertRule {
            name: "test".into(),
            description: "test".into(),
            condition: AlertCondition::LogPatternMatch {
                level: "ERROR".into(),
                pattern: "connection refused".into(),
                count: 10,
            },
            severity: AlertSeverity::Warning,
            channels: vec!["slack".into()],
        };
        assert_eq!(rule.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_panel_type_variants() {
        let panel = Panel {
            title: "test".into(),
            panel_type: PanelType::Heatmap,
            query: "test".into(),
            unit: "count".into(),
        };
        assert!(matches!(panel.panel_type, PanelType::Heatmap));
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }
}
