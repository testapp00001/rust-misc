//! # Lesson 10: Deployment Incident Response (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Containing,
    Eradicating,
    Recovered,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub detected_at: String,
    pub contained_at: Option<String>,
    pub closed_at: Option<String>,
    pub actions: Vec<Action>,
    pub data_breach: bool,
    pub affected_systems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub timestamp: String,
    pub actor: String,
    pub description: String,
    pub category: String,
}

/// Parse "YYYY-MM-DDTHH:MM:SSZ" to minutes since epoch (approximate).
pub fn parse_timestamp_minutes(ts: &str) -> Option<u64> {
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

    let days = (year - 2000) * 365 + (month - 1) * 30 + (day - 1);
    Some(days * 24 * 60 + hour * 60 + minute + second / 60)
}

/// Calculate incident response time metrics.
///
/// Returns (MTTD, MTTC, MTTR) where each is Option<u64> (minutes).
pub fn calculate_metrics(incident: &Incident) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mttd = Some(0); // Detection is always the start

    let mttc = if let Some(ref contained) = incident.contained_at {
        match (parse_timestamp_minutes(&incident.detected_at), parse_timestamp_minutes(contained)) {
            (Some(detect), Some(contain)) => Some(contain - detect),
            _ => None,
        }
    } else {
        None
    };

    let mttr = if let Some(ref closed) = incident.closed_at {
        match (parse_timestamp_minutes(&incident.detected_at), parse_timestamp_minutes(closed)) {
            (Some(detect), Some(close)) => Some(close - detect),
            _ => None,
        }
    } else {
        None
    };

    (mttd, mttc, mttr)
}

/// Generate a formatted incident status update.
pub fn status_update(incident: &Incident) -> String {
    let contained = incident.contained_at.as_deref().unwrap_or("Not yet contained");
    let systems = incident.affected_systems.join(", ");
    let breach = if incident.data_breach { "yes" } else { "no" };
    let severity = format!("{:?}", incident.severity);
    let status = format!("{:?}", incident.status);

    format!(
        "Incident Update: {}\n\
         Title: {}\n\
         Severity: {}\n\
         Status: {}\n\
         Detected: {}\n\
         Contained: {}\n\
         Systems: {}\n\
         Actions taken: {}\n\
         Data breach: {}",
        incident.id, incident.title, severity, status,
        incident.detected_at, contained, systems,
        incident.actions.len(), breach
    )
}

/// Determine recommended next steps based on incident status.
pub fn recommended_actions(status: IncidentStatus) -> Vec<&'static str> {
    match status {
        IncidentStatus::Detected => vec![
            "Assign incident commander",
            "Assess blast radius",
            "Notify on-call team",
            "Begin investigation",
        ],
        IncidentStatus::Investigating => vec![
            "Identify attack vector",
            "Determine data exposure",
            "Prepare containment plan",
        ],
        IncidentStatus::Containing => vec![
            "Execute rollback",
            "Rotate credentials",
            "Block attacker IP ranges",
            "Preserve forensic evidence",
        ],
        IncidentStatus::Eradicating => vec![
            "Apply security patch",
            "Verify threat removal",
            "Update monitoring rules",
        ],
        IncidentStatus::Recovered => vec![
            "Monitor for recurrence",
            "Verify normal operations",
            "Schedule post-mortem",
        ],
        IncidentStatus::Closed => vec![
            "Update runbooks",
            "Share lessons learned",
        ],
    }
}

/// Check if incident requires regulatory data breach notification.
pub fn requires_notification(incident: &Incident) -> bool {
    incident.data_breach
        && incident.severity >= IncidentSeverity::High
        && incident.status != IncidentStatus::Closed
}

/// Generate a post-mortem template.
pub fn post_mortem_template(incident: &Incident) -> String {
    let severity = format!("{:?}", incident.severity);
    let contained = incident.contained_at.as_deref().unwrap_or("...");
    let closed = incident.closed_at.as_deref().unwrap_or("...");

    let duration = match (&incident.contained_at, &incident.closed_at) {
        (Some(_), Some(closed_ts)) => {
            if let (Some(d), Some(c)) = (parse_timestamp_minutes(&incident.detected_at), parse_timestamp_minutes(closed_ts)) {
                format!("{} minutes", c - d)
            } else {
                "unknown".to_string()
            }
        }
        _ => "ongoing".to_string(),
    };

    let breach = if incident.data_breach { "yes" } else { "no" };

    let mut actions_str = String::new();
    for action in &incident.actions {
        actions_str.push_str(&format!("  [{}] {}: {}\n", action.timestamp, action.actor, action.description));
    }

    let mut systems_str = String::new();
    for system in &incident.affected_systems {
        systems_str.push_str(&format!("  - {}\n", system));
    }

    format!(
        "POST-MORTEM: {} - {}\n\
         ====================================\n\
         Severity: {}\n\
         Duration: {}\n\
         Data Breach: {}\n\n\
         Timeline:\n\
           {} - Incident detected\n\
           {} - Containment achieved\n\
           {} - Incident closed\n\n\
         Actions Taken:\n\
         {}\n\
         Affected Systems:\n\
         {}\n\
         Lessons Learned:\n\
           [To be filled during post-mortem meeting]",
        incident.id, incident.title, severity, duration, breach,
        incident.detected_at, contained, closed,
        actions_str, systems_str
    )
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
