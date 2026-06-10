//! # Lesson 10: Deployment Incident Response
//!
//! ## Attack: Production Compromise
//!
//! At 2 AM, your monitoring alerts: unusual outbound traffic from production.
//! The attacker exploited a zero-day in a dependency. Every minute counts.
//!
//! Without a practiced incident response plan, the team panics:
//! - Nobody knows who's responsible for what
//! - The rollback procedure has never been tested
//! - Communication to users is delayed by hours
//! - The attacker exfiltrates data while the team argues
//!
//! ## Defend: Incident Response Playbook
//!
//! 1. **Detect**: Automated monitoring triggers alerts
//! 2. **Triage**: Determine severity and blast radius
//! 3. **Contain**: Isolate the affected system (rollback, network quarantine)
//! 4. **Eradicate**: Remove the threat (patch, rotate credentials)
//! 5. **Recover**: Restore normal operations
//! 6. **Learn**: Post-mortem, update runbooks
//!
//! ## Audit: Incident Readiness
//!
//! - [ ] Rollback procedure tested monthly
//! - [ ] On-call rotation documented
//! - [ ] Communication templates prepared
//! - [ ] Credential rotation procedures documented
//! - [ ] Forensic data collection automated
//! - [ ] Post-mortem process defined

use serde::{Deserialize, Serialize};

/// Severity of an incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncidentSeverity {
    /// Minor issue, no user impact
    Low,
    /// Some users affected, workaround available
    Medium,
    /// Significant user impact, no workaround
    High,
    /// Complete outage or data breach
    Critical,
}

/// Status of an incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    /// Just detected
    Detected,
    /// Team is investigating
    Investigating,
    /// Containment in progress
    Containing,
    /// Threat removed, recovering
    Eradicating,
    /// Service restored
    Recovered,
    /// Post-mortem complete
    Closed,
}

/// An incident record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Incident {
    /// Unique incident ID
    pub id: String,
    /// Short title
    pub title: String,
    /// Severity
    pub severity: IncidentSeverity,
    /// Current status
    pub status: IncidentStatus,
    /// ISO 8601 timestamp when detected
    pub detected_at: String,
    /// ISO 8601 timestamp when contained (None if not yet)
    pub contained_at: Option<String>,
    /// ISO 8601 timestamp when closed (None if not yet)
    pub closed_at: Option<String>,
    /// List of actions taken
    pub actions: Vec<Action>,
    /// Whether data breach notification is required
    pub data_breach: bool,
    /// Systems affected
    pub affected_systems: Vec<String>,
}

/// An action taken during incident response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Who performed the action
    pub actor: String,
    /// What was done
    pub description: String,
    /// Action category
    pub category: String,
}

/// Exercise 1: Calculate incident response metrics.
///
/// Given an incident, compute:
/// - **MTTD** (Mean Time to Detect): Always 0 for a single incident (detection is the start)
/// - **MTTC** (Mean Time to Contain): Minutes from detection to containment
/// - **MTTR** (Mean Time to Recover): Minutes from detection to closure
///
/// Parse timestamps as "YYYY-MM-DDTHH:MM:SSZ" format.
/// If containment or closure hasn't happened yet, return None for that metric.
///
/// Return (MTTD, MTTC, MTTR) where each is Option<u64> (minutes).
pub fn calculate_metrics(incident: &Incident) -> (Option<u64>, Option<u64>, Option<u64>) {
    todo!("Calculate incident response time metrics")
}

/// Exercise 2: Generate an incident status update.
///
/// Return a formatted string:
///
/// ```text
/// Incident Update: {id}
/// Title: {title}
/// Severity: {severity}
/// Status: {status}
/// Detected: {detected_at}
/// Contained: {contained_at or "Not yet contained"}
/// Systems: {comma-separated list}
/// Actions taken: {count}
/// Data breach: {yes/no}
/// ```
pub fn status_update(incident: &Incident) -> String {
    todo!("Generate a formatted incident status update")
}

/// Exercise 3: Determine required next steps.
///
/// Based on the incident status, return a list of recommended actions:
///
/// - Detected: ["Assign incident commander", "Assess blast radius",
///   "Notify on-call team", "Begin investigation"]
/// - Investigating: ["Identify attack vector", "Determine data exposure",
///   "Prepare containment plan"]
/// - Containing: ["Execute rollback", "Rotate credentials",
///   "Block attacker IP ranges", "Preserve forensic evidence"]
/// - Eradicating: ["Apply security patch", "Verify threat removal",
///   "Update monitoring rules"]
/// - Recovered: ["Monitor for recurrence", "Verify normal operations",
///   "Schedule post-mortem"]
/// - Closed: ["Update runbooks", "Share lessons learned"]
pub fn recommended_actions(status: IncidentStatus) -> Vec<&'static str> {
    todo!("Determine recommended next steps based on incident status")
}

/// Exercise 4: Check if an incident requires regulatory notification.
///
/// Data breach notification is required if:
/// 1. `data_breach` is true
/// 2. AND severity is High or Critical
/// 3. AND status is not Closed (still active)
///
/// Return true if notification is required.
pub fn requires_notification(incident: &Incident) -> bool {
    todo!("Check if incident requires regulatory data breach notification")
}

/// Exercise 5: Generate a post-mortem template.
///
/// Given an incident, return a structured post-mortem template:
///
/// ```text
/// POST-MORTEM: {id} - {title}
/// ====================================
/// Severity: {severity}
/// Duration: {duration_minutes or "ongoing"} minutes
/// Data Breach: {yes/no}
///
/// Timeline:
///   {detected_at} - Incident detected
///   {contained_at or "..."} - Containment achieved
///   {closed_at or "..."} - Incident closed
///
/// Actions Taken:
///   [{timestamp}] {actor}: {description}
///   ...
///
/// Affected Systems:
///   - {system1}
///   - {system2}
///
/// Lessons Learned:
///   [To be filled during post-mortem meeting]
/// ```
pub fn post_mortem_template(incident: &Incident) -> String {
    todo!("Generate a post-mortem template")
}

/// Helper: parse "YYYY-MM-DDTHH:MM:SSZ" to minutes since epoch (approximate).
/// This is a simplified parser for the exercise — just parse the key fields.
pub fn parse_timestamp_minutes(ts: &str) -> Option<u64> {
    // Parse: "2024-01-15T10:30:00Z"
    // Extract year, month, day, hour, minute, second
    let parts: Vec<&str> = ts.split('T').collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].trim_end_matches('Z').split(':').collect();
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }
    let year: u64 = date_parts[0].parse().ok()?;
    let month: u64 = date_parts[1].parse().ok()?;
    let day: u64 = date_parts[2].parse().ok()?;
    let hour: u64 = time_parts[0].parse().ok()?;
    let minute: u64 = time_parts[1].parse().ok()?;
    let second: u64 = time_parts[2].parse().ok()?;

    // Convert to total minutes (approximate, good enough for delta calculation)
    let days = (year - 2000) * 365 + (month - 1) * 30 + (day - 1);
    Some(days * 24 * 60 + hour * 60 + minute + second / 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_incident() -> Incident {
        Incident {
            id: "INC-2024-001".to_string(),
            title: "Production database compromise".to_string(),
            severity: IncidentSeverity::Critical,
            status: IncidentStatus::Recovered,
            detected_at: "2024-01-15T02:00:00Z".to_string(),
            contained_at: Some("2024-01-15T02:30:00Z".to_string()),
            closed_at: Some("2024-01-15T06:00:00Z".to_string()),
            actions: vec![
                Action {
                    timestamp: "2024-01-15T02:05:00Z".to_string(),
                    actor: "alice".to_string(),
                    description: "Rolled back to previous deployment".to_string(),
                    category: "containment".to_string(),
                },
                Action {
                    timestamp: "2024-01-15T02:15:00Z".to_string(),
                    actor: "bob".to_string(),
                    description: "Rotated database credentials".to_string(),
                    category: "eradication".to_string(),
                },
            ],
            data_breach: true,
            affected_systems: vec!["database".to_string(), "api-server".to_string()],
        }
    }

    #[test]
    fn test_parse_timestamp() {
        let mins = parse_timestamp_minutes("2024-01-15T02:00:00Z");
        assert!(mins.is_some());
        let mins2 = parse_timestamp_minutes("2024-01-15T02:30:00Z");
        assert!(mins2.is_some());
        assert_eq!(mins2.unwrap() - mins.unwrap(), 30);
    }

    #[test]
    fn test_calculate_metrics_contained() {
        let incident = sample_incident();
        let (mttd, mttc, mttr) = calculate_metrics(&incident);
        assert_eq!(mttd, Some(0), "MTTD should be 0 (detection = start)");
        assert_eq!(mttc, Some(30), "MTTC should be 30 minutes");
        assert_eq!(mttr, Some(240), "MTTR should be 240 minutes (4 hours)");
    }

    #[test]
    fn test_calculate_metrics_not_contained() {
        let mut incident = sample_incident();
        incident.contained_at = None;
        incident.closed_at = None;
        let (mttd, mttc, mttr) = calculate_metrics(&incident);
        assert_eq!(mttd, Some(0));
        assert_eq!(mttc, None);
        assert_eq!(mttr, None);
    }

    #[test]
    fn test_status_update() {
        let incident = sample_incident();
        let update = status_update(&incident);
        assert!(update.contains("INC-2024-001"));
        assert!(update.contains("Critical"));
        assert!(update.contains("Recovered"));
        assert!(update.contains("database"));
    }

    #[test]
    fn test_recommended_actions_detected() {
        let actions = recommended_actions(IncidentStatus::Detected);
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.contains("commander") || a.contains("investigate")));
    }

    #[test]
    fn test_recommended_actions_containing() {
        let actions = recommended_actions(IncidentStatus::Containing);
        assert!(actions.iter().any(|a| a.contains("rollback") || a.contains("credential")));
    }

    #[test]
    fn test_requires_notification_breach_critical() {
        let incident = sample_incident();
        assert!(requires_notification(&incident));
    }

    #[test]
    fn test_requires_notification_no_breach() {
        let mut incident = sample_incident();
        incident.data_breach = false;
        assert!(!requires_notification(&incident));
    }

    #[test]
    fn test_requires_notification_low_severity() {
        let mut incident = sample_incident();
        incident.severity = IncidentSeverity::Low;
        assert!(!requires_notification(&incident));
    }

    #[test]
    fn test_requires_notification_closed() {
        let mut incident = sample_incident();
        incident.status = IncidentStatus::Closed;
        assert!(!requires_notification(&incident));
    }

    #[test]
    fn test_post_mortem_template() {
        let incident = sample_incident();
        let template = post_mortem_template(&incident);
        assert!(template.contains("INC-2024-001"));
        assert!(template.contains("POST-MORTEM"));
        assert!(template.contains("alice"));
        assert!(template.contains("Rolled back"));
        assert!(template.contains("database"));
        assert!(template.contains("Lessons Learned"));
    }
}
