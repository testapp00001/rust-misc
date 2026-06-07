//! # Production Operations for Rust Services
//!
//! Running Rust services in production requires operational excellence:
//! runbooks, incident response, post-mortems, and SLO management. This
//! module covers building operational tooling and processes.
//!
//! ## Key Concepts:
//!
//! - **SLOs/SLIs**: Service Level Objectives and Indicators
//! - **Runbooks**: Step-by-step operational procedures
//! - **Incident Response**: Structured approach to handling incidents
//! - **Post-Mortems**: Blameless analysis of incidents
//! - **On-Call**: Rotation management and escalation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service Level Indicator (SLI) - a quantitative measure of service behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelIndicator {
    pub name: String,
    pub description: String,
    pub query: String,
    pub unit: SliUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SliUnit {
    Percentage,
    Milliseconds,
    Count,
    Bytes,
}

/// Service Level Objective (SLO) - target value for an SLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelObjective {
    pub name: String,
    pub sli: ServiceLevelIndicator,
    pub target: f64,
    pub window: String,
    pub error_budget_remaining: f64,
    pub current_value: f64,
}

impl ServiceLevelObjective {
    pub fn new(
        name: &str,
        sli: ServiceLevelIndicator,
        target: f64,
        window: &str,
    ) -> Self {
        Self {
            name: name.into(),
            sli,
            target,
            window: window.into(),
            error_budget_remaining: 1.0,
            current_value: 0.0,
        }
    }

    /// Check if the SLO is being met.
    pub fn is_met(&self) -> bool {
        self.current_value >= self.target
    }

    /// Calculate error budget consumption (0.0 = full budget, 1.0 = exhausted).
    pub fn error_budget_consumed(&self) -> f64 {
        1.0 - self.error_budget_remaining
    }

    /// Check if error budget is critically low (< 10% remaining).
    pub fn is_error_budget_critical(&self) -> bool {
        self.error_budget_remaining < 0.1
    }

    /// Generate an alert message if SLO is at risk.
    pub fn alert_message(&self) -> Option<String> {
        if self.is_error_budget_critical() {
            Some(format!(
                "CRITICAL: SLO '{}' error budget {:.1}% remaining. Current: {:.4}, Target: {:.4}",
                self.name,
                self.error_budget_remaining * 100.0,
                self.current_value,
                self.target
            ))
        } else if !self.is_met() {
            Some(format!(
                "WARNING: SLO '{}' not met. Current: {:.4}, Target: {:.4}",
                self.name, self.current_value, self.target
            ))
        } else {
            None
        }
    }
}

/// SLO dashboard configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct SloDashboard {
    pub service_name: String,
    pub slos: Vec<ServiceLevelObjective>,
    pub overall_status: SloStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SloStatus {
    Healthy,
    AtRisk,
    Breached,
}

impl SloDashboard {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.into(),
            slos: Vec::new(),
            overall_status: SloStatus::Healthy,
        }
    }

    pub fn add_slo(&mut self, slo: ServiceLevelObjective) {
        self.slos.push(slo);
        self.update_status();
    }

    fn update_status(&mut self) {
        if self.slos.iter().any(|s| !s.is_met()) {
            self.overall_status = SloStatus::Breached;
        } else if self.slos.iter().any(|s| s.is_error_budget_critical()) {
            self.overall_status = SloStatus::AtRisk;
        } else {
            self.overall_status = SloStatus::Healthy;
        }
    }

    /// Generate a text summary of all SLOs.
    pub fn summary(&self) -> String {
        let mut output = format!("SLO Dashboard: {}\n", self.service_name);
        output.push_str(&format!("Overall Status: {:?}\n\n", self.overall_status));
        for slo in &self.slos {
            let status = if slo.is_met() { "OK" } else { "BREACH" };
            output.push_str(&format!(
                "  [{}] {} - Current: {:.4}, Target: {:.4}, Budget: {:.1}%\n",
                status,
                slo.name,
                slo.current_value,
                slo.target,
                slo.error_budget_remaining * 100.0
            ));
        }
        output
    }
}

/// Runbook for operational procedures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub title: String,
    pub description: String,
    pub trigger: String,
    pub severity: Severity,
    pub steps: Vec<RunbookStep>,
    pub rollback_steps: Vec<RunbookStep>,
    pub references: Vec<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub order: u32,
    pub description: String,
    pub command: Option<String>,
    pub expected_result: String,
    pub timeout_secs: Option<u32>,
    pub requires_approval: bool,
}

impl Runbook {
    pub fn new(title: &str, trigger: &str, severity: Severity) -> Self {
        Self {
            title: title.into(),
            description: String::new(),
            trigger: trigger.into(),
            severity,
            steps: Vec::new(),
            rollback_steps: Vec::new(),
            references: Vec::new(),
            last_updated: "2024-01-01".into(),
        }
    }

    pub fn add_step(
        &mut self,
        order: u32,
        description: &str,
        command: Option<&str>,
        expected: &str,
    ) {
        self.steps.push(RunbookStep {
            order,
            description: description.into(),
            command: command.map(|c| c.into()),
            expected_result: expected.into(),
            timeout_secs: None,
            requires_approval: false,
        });
    }

    pub fn render_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Runbook: {}\n\n", self.title));
        md.push_str(&format!("**Severity:** {:?}\n", self.severity));
        md.push_str(&format!("**Trigger:** {}\n", self.trigger));
        md.push_str(&format!("**Last Updated:** {}\n\n", self.last_updated));

        if !self.description.is_empty() {
            md.push_str(&format!("## Description\n\n{}\n\n", self.description));
        }

        md.push_str("## Steps\n\n");
        for step in &self.steps {
            md.push_str(&format!("### Step {}: {}\n\n", step.order, step.description));
            if let Some(ref cmd) = step.command {
                md.push_str(&format!("```bash\n{}\n```\n\n", cmd));
            }
            md.push_str(&format!("**Expected:** {}\n\n", step.expected_result));
            if step.requires_approval {
                md.push_str("**Requires approval before proceeding.**\n\n");
            }
        }

        if !self.rollback_steps.is_empty() {
            md.push_str("## Rollback Steps\n\n");
            for step in &self.rollback_steps {
                md.push_str(&format!("### Step {}: {}\n\n", step.order, step.description));
                if let Some(ref cmd) = step.command {
                    md.push_str(&format!("```bash\n{}\n```\n\n", cmd));
                }
            }
        }

        if !self.references.is_empty() {
            md.push_str("## References\n\n");
            for reference in &self.references {
                md.push_str(&format!("- {}\n", reference));
            }
        }

        md
    }
}

/// Incident report and tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub status: IncidentStatus,
    pub commander: String,
    pub affected_services: Vec<String>,
    pub timeline: Vec<TimelineEntry>,
    pub impact: ImpactAssessment,
    pub root_cause: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Identified,
    Mitigating,
    Resolved,
    PostMortem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub users_affected: Option<u64>,
    pub revenue_impact: Option<String>,
    pub duration_minutes: u32,
    pub slo_impact: Vec<String>,
}

impl Incident {
    pub fn new(id: &str, title: &str, severity: Severity, commander: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            severity,
            status: IncidentStatus::Detected,
            commander: commander.into(),
            affected_services: Vec::new(),
            timeline: Vec::new(),
            impact: ImpactAssessment {
                users_affected: None,
                revenue_impact: None,
                duration_minutes: 0,
                slo_impact: Vec::new(),
            },
            root_cause: None,
            created_at: "2024-06-01T12:00:00Z".into(),
            resolved_at: None,
        }
    }

    pub fn add_timeline_entry(&mut self, actor: &str, action: &str, details: Option<&str>) {
        self.timeline.push(TimelineEntry {
            timestamp: "2024-06-01T12:00:00Z".into(),
            actor: actor.into(),
            action: action.into(),
            details: details.map(|d| d.into()),
        });
    }

    pub fn update_status(&mut self, new_status: IncidentStatus) {
        self.status = new_status.clone();
        self.add_timeline_entry(
            &self.commander.clone(),
            &format!("Status changed to {:?}", new_status),
            None,
        );
    }

    pub fn resolve(&mut self, root_cause: &str) {
        self.root_cause = Some(root_cause.into());
        self.resolved_at = Some("2024-06-01T13:00:00Z".into());
        self.update_status(IncidentStatus::Resolved);
    }

    /// Generate a post-mortem report.
    pub fn generate_post_mortem(&self) -> PostMortem {
        PostMortem {
            incident_id: self.id.clone(),
            title: format!("Post-Mortem: {}", self.title),
            severity: self.severity.clone(),
            date: self.created_at.clone(),
            duration_minutes: self.impact.duration_minutes,
            impact_summary: format!(
                "{} users affected, {} services impacted",
                self.impact
                    .users_affected
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "unknown".into()),
                self.affected_services.len()
            ),
            root_cause: self
                .root_cause
                .clone()
                .unwrap_or_else(|| "Under investigation".into()),
            timeline: self.timeline.clone(),
            action_items: Vec::new(),
            lessons_learned: Vec::new(),
        }
    }
}

/// Post-mortem report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMortem {
    pub incident_id: String,
    pub title: String,
    pub severity: Severity,
    pub date: String,
    pub duration_minutes: u32,
    pub impact_summary: String,
    pub root_cause: String,
    pub timeline: Vec<TimelineEntry>,
    pub action_items: Vec<ActionItem>,
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub owner: String,
    pub priority: Priority,
    pub due_date: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
}

impl PostMortem {
    pub fn render_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("- **Incident ID:** {}\n", self.incident_id));
        md.push_str(&format!("- **Severity:** {:?}\n", self.severity));
        md.push_str(&format!("- **Date:** {}\n", self.date));
        md.push_str(&format!("- **Duration:** {} minutes\n", self.duration_minutes));
        md.push_str(&format!("- **Impact:** {}\n\n", self.impact_summary));

        md.push_str("## Root Cause\n\n");
        md.push_str(&format!("{}\n\n", self.root_cause));

        md.push_str("## Timeline\n\n");
        for entry in &self.timeline {
            md.push_str(&format!(
                "- **{}** [{}]: {}\n",
                entry.timestamp, entry.actor, entry.action
            ));
            if let Some(ref details) = entry.details {
                md.push_str(&format!("  - {}\n", details));
            }
        }
        md.push('\n');

        if !self.action_items.is_empty() {
            md.push_str("## Action Items\n\n");
            md.push_str("| Description | Owner | Priority | Due Date | Status |\n");
            md.push_str("|-------------|-------|----------|----------|--------|\n");
            for item in &self.action_items {
                let status = if item.completed { "Done" } else { "Open" };
                md.push_str(&format!(
                    "| {} | {} | {:?} | {} | {} |\n",
                    item.description, item.owner, item.priority, item.due_date, status
                ));
            }
            md.push('\n');
        }

        if !self.lessons_learned.is_empty() {
            md.push_str("## Lessons Learned\n\n");
            for lesson in &self.lessons_learned {
                md.push_str(&format!("- {}\n", lesson));
            }
        }

        md
    }
}

/// On-call rotation management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCallRotation {
    pub name: String,
    pub schedule: Vec<OnCallSlot>,
    pub escalation_policy: Vec<EscalationLevel>,
    pub current_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCallSlot {
    pub person: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub target: String,
    pub delay_minutes: u32,
    pub method: EscalationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationMethod {
    Page,
    Phone,
    Email,
    Slack,
}

impl OnCallRotation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            schedule: Vec::new(),
            escalation_policy: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_slot(&mut self, person: &str, start: &str, end: &str) {
        self.schedule.push(OnCallSlot {
            person: person.into(),
            start: start.into(),
            end: end.into(),
        });
    }

    pub fn add_escalation(&mut self, level: u32, target: &str, delay: u32, method: EscalationMethod) {
        self.escalation_policy.push(EscalationLevel {
            level,
            target: target.into(),
            delay_minutes: delay,
            method,
        });
    }

    pub fn current_on_call(&self) -> Option<&str> {
        self.schedule
            .get(self.current_index)
            .map(|s| s.person.as_str())
    }

    pub fn escalate(&self, level: u32) -> Option<&EscalationLevel> {
        self.escalation_policy.iter().find(|e| e.level == level)
    }
}

/// Standard runbook templates for common Rust service operations.
pub struct RunbookTemplates;

impl RunbookTemplates {
    /// High CPU usage runbook.
    pub fn high_cpu(service_name: &str) -> Runbook {
        let mut rb = Runbook::new(
            &format!("{} - High CPU Usage", service_name),
            "CPU usage > 90% for 5 minutes",
            Severity::High,
        );
        rb.description = "Investigate and mitigate high CPU usage on the service.".into();
        rb.add_step(
            1,
            "Check current CPU usage and identify top processes",
            Some("top -bn1 | head -20"),
            "Identify which process is consuming CPU",
        );
        rb.add_step(
            2,
            "Check application metrics for request rate spikes",
            Some("curl http://localhost:9090/metrics | grep request_rate"),
            "Request rate within expected bounds",
        );
        rb.add_step(
            3,
            "Check for infinite loops or stuck computations in logs",
            Some("journalctl -u {} --since '10 min ago' | grep -i error", ),
            "No unusual error patterns",
        );
        rb.add_step(
            4,
            "If CPU remains high, scale up instances",
            Some("kubectl scale deployment/{} --replicas=5"),
            "Additional instances are running",
        );
        rb
    }

    /// Database connection exhaustion runbook.
    pub fn db_connections(service_name: &str) -> Runbook {
        let mut rb = Runbook::new(
            &format!("{} - Database Connection Pool Exhausted", service_name),
            "Database connection pool at maximum capacity",
            Severity::Critical,
        );
        rb.add_step(
            1,
            "Check active database connections",
            Some("SELECT count(*) FROM pg_stat_activity;"),
            "Connection count is at or near max",
        );
        rb.add_step(
            2,
            "Identify long-running queries",
            Some("SELECT pid, now() - pg_stat_activity.query_start AS duration, query FROM pg_stat_activity WHERE state = 'active' ORDER BY duration DESC;"),
            "Identify queries running longer than expected",
        );
        rb.add_step(
            3,
            "Kill long-running queries if safe",
            Some("SELECT pg_terminate_backend(pid);"),
            "Connections freed",
        );
        rb.add_step(
            4,
            "Restart application pods to reset connection pools",
            Some("kubectl rollout restart deployment/{}"),
            "Pods restarted with fresh connections",
        );
        rb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_sli() -> ServiceLevelIndicator {
        ServiceLevelIndicator {
            name: "availability".into(),
            description: "Successful requests / total requests".into(),
            query: "rate(http_requests_total{status!~\"5..\"}[5m]) / rate(http_requests_total[5m])".into(),
            unit: SliUnit::Percentage,
        }
    }

    #[test]
    fn test_slo_is_met() {
        let mut slo = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo.current_value = 0.9995;
        slo.error_budget_remaining = 0.5;
        assert!(slo.is_met());
    }

    #[test]
    fn test_slo_not_met() {
        let mut slo = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo.current_value = 0.995;
        assert!(!slo.is_met());
    }

    #[test]
    fn test_slo_error_budget_critical() {
        let mut slo = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo.error_budget_remaining = 0.05;
        assert!(slo.is_error_budget_critical());

        slo.error_budget_remaining = 0.5;
        assert!(!slo.is_error_budget_critical());
    }

    #[test]
    fn test_slo_alert_message() {
        let mut slo = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo.current_value = 0.9995;
        slo.error_budget_remaining = 0.8;
        assert!(slo.alert_message().is_none());

        slo.error_budget_remaining = 0.05;
        let msg = slo.alert_message().unwrap();
        assert!(msg.contains("CRITICAL"));
    }

    #[test]
    fn test_slo_dashboard() {
        let mut dashboard = SloDashboard::new("my-api");

        let mut slo1 = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo1.current_value = 0.9995;
        slo1.error_budget_remaining = 0.5;
        dashboard.add_slo(slo1);

        assert_eq!(dashboard.overall_status, SloStatus::Healthy);

        let mut slo2 = ServiceLevelObjective::new("latency", make_test_sli(), 0.95, "30d");
        slo2.current_value = 0.90;
        slo2.error_budget_remaining = 0.0;
        dashboard.add_slo(slo2);

        assert_eq!(dashboard.overall_status, SloStatus::Breached);
    }

    #[test]
    fn test_slo_dashboard_summary() {
        let mut dashboard = SloDashboard::new("my-api");
        let mut slo = ServiceLevelObjective::new("availability", make_test_sli(), 0.999, "30d");
        slo.current_value = 0.9995;
        slo.error_budget_remaining = 0.5;
        dashboard.add_slo(slo);

        let summary = dashboard.summary();
        assert!(summary.contains("my-api"));
        assert!(summary.contains("availability"));
    }

    #[test]
    fn test_runbook_creation() {
        let mut rb = Runbook::new("High CPU", "CPU > 90%", Severity::High);
        rb.add_step(1, "Check top processes", Some("top"), "Identify CPU hog");
        rb.add_step(2, "Check logs", Some("journalctl -f"), "No errors");

        assert_eq!(rb.steps.len(), 2);
        assert_eq!(rb.severity, Severity::High);
    }

    #[test]
    fn test_runbook_markdown() {
        let mut rb = Runbook::new("Test Runbook", "Alert fired", Severity::Medium);
        rb.add_step(1, "First step", Some("echo hello"), "Output displayed");
        rb.add_step(
            2,
            "Second step (approval needed)",
            None,
            "Manual verification",
        );

        let md = rb.render_markdown();
        assert!(md.contains("# Runbook: Test Runbook"));
        assert!(md.contains("Medium"));
        assert!(md.contains("echo hello"));
        assert!(md.contains("Step 1"));
    }

    #[test]
    fn test_incident_creation() {
        let incident = Incident::new(
            "INC-001",
            "API outage",
            Severity::Critical,
            "on-call-engineer",
        );
        assert_eq!(incident.status, IncidentStatus::Detected);
        assert!(incident.root_cause.is_none());
    }

    #[test]
    fn test_incident_timeline() {
        let mut incident = Incident::new("INC-001", "Outage", Severity::Critical, "alice");
        incident.add_timeline_entry("alice", "Started investigation", Some("Checking logs"));
        incident.add_timeline_entry("bob", "Found root cause", Some("Memory leak in parser"));

        assert_eq!(incident.timeline.len(), 2);
    }

    #[test]
    fn test_incident_resolution() {
        let mut incident = Incident::new("INC-001", "Outage", Severity::Critical, "alice");
        incident.resolve("Memory leak in request parser");
        assert_eq!(incident.status, IncidentStatus::Resolved);
        assert!(incident.root_cause.is_some());
        assert!(incident.resolved_at.is_some());
    }

    #[test]
    fn test_incident_post_mortem() {
        let mut incident = Incident::new("INC-001", "API outage", Severity::Critical, "alice");
        incident.affected_services = vec!["api".into(), "auth".into()];
        incident.impact.users_affected = Some(10000);
        incident.impact.duration_minutes = 45;
        incident.resolve("Null pointer in auth middleware");

        let pm = incident.generate_post_mortem();
        assert_eq!(pm.incident_id, "INC-001");
        assert!(pm.root_cause.contains("Null pointer"));
        assert_eq!(pm.duration_minutes, 45);
    }

    #[test]
    fn test_post_mortem_markdown() {
        let pm = PostMortem {
            incident_id: "INC-001".into(),
            title: "Post-Mortem: API Outage".into(),
            severity: Severity::Critical,
            date: "2024-06-01".into(),
            duration_minutes: 30,
            impact_summary: "1000 users affected".into(),
            root_cause: "Memory leak in parser".into(),
            timeline: vec![TimelineEntry {
                timestamp: "12:00".into(),
                actor: "alice".into(),
                action: "Detected".into(),
                details: None,
            }],
            action_items: vec![ActionItem {
                description: "Fix memory leak".into(),
                owner: "bob".into(),
                priority: Priority::P0,
                due_date: "2024-06-08".into(),
                completed: false,
            }],
            lessons_learned: vec!["Need better memory monitoring".into()],
        };

        let md = pm.render_markdown();
        assert!(md.contains("Post-Mortem: API Outage"));
        assert!(md.contains("Memory leak"));
        assert!(md.contains("P0"));
        assert!(md.contains("memory monitoring"));
    }

    #[test]
    fn test_on_call_rotation() {
        let mut rotation = OnCallRotation::new("backend-team");
        rotation.add_slot("alice", "2024-06-01", "2024-06-08");
        rotation.add_slot("bob", "2024-06-08", "2024-06-15");
        rotation.add_escalation(1, "alice", 0, EscalationMethod::Page);
        rotation.add_escalation(2, "team-lead", 15, EscalationMethod::Phone);

        assert_eq!(rotation.current_on_call(), Some("alice"));
        assert!(rotation.escalate(1).is_some());
        assert!(rotation.escalate(3).is_none());
    }

    #[test]
    fn test_runbook_templates_high_cpu() {
        let rb = RunbookTemplates::high_cpu("my-api");
        assert!(rb.title.contains("High CPU"));
        assert!(!rb.steps.is_empty());
        assert_eq!(rb.severity, Severity::High);
    }

    #[test]
    fn test_runbook_templates_db_connections() {
        let rb = RunbookTemplates::db_connections("my-api");
        assert!(rb.title.contains("Database"));
        assert!(!rb.steps.is_empty());
        assert_eq!(rb.severity, Severity::Critical);
    }

    #[test]
    fn test_slo_error_budget_consumed() {
        let mut slo = ServiceLevelObjective::new("avail", make_test_sli(), 0.999, "30d");
        slo.error_budget_remaining = 0.3;
        assert!((slo.error_budget_consumed() - 0.7).abs() < f64::EPSILON);
    }
}
