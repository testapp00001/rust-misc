//! # Lesson 10: Secret Leak Incident Response (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncidentStatus {
    Detected,
    Contained,
    Rotating,
    Auditing,
    Remediated,
    Closed,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub secret_path: String,
    pub severity: Severity,
    pub status: IncidentStatus,
    pub detected_at: Instant,
    pub leak_source: String,
    pub actions_taken: Vec<String>,
    pub unauthorized_accesses: Vec<String>,
}

#[derive(Debug)]
pub struct IncidentResponseSystem {
    incidents: Vec<Incident>,
    next_id: u32,
}

impl IncidentResponseSystem {
    pub fn new() -> Self {
        Self {
            incidents: Vec::new(),
            next_id: 1,
        }
    }
}

pub fn report_incident(
    system: &mut IncidentResponseSystem,
    secret_path: &str,
    severity: Severity,
    leak_source: &str,
) -> String {
    let id = format!("INC-{:04}", system.next_id);
    system.next_id += 1;

    let incident = Incident {
        id: id.clone(),
        secret_path: secret_path.to_string(),
        severity,
        status: IncidentStatus::Detected,
        detected_at: Instant::now(),
        leak_source: leak_source.to_string(),
        actions_taken: Vec::new(),
        unauthorized_accesses: Vec::new(),
    };

    system.incidents.push(incident);
    id
}

fn find_incident_mut<'a>(
    system: &'a mut IncidentResponseSystem,
    incident_id: &str,
) -> Result<&'a mut Incident, String> {
    system
        .incidents
        .iter_mut()
        .find(|i| i.id == incident_id)
        .ok_or_else(|| "Incident not found".to_string())
}

fn find_incident<'a>(
    system: &'a IncidentResponseSystem,
    incident_id: &str,
) -> Result<&'a Incident, String> {
    system
        .incidents
        .iter()
        .find(|i| i.id == incident_id)
        .ok_or_else(|| "Incident not found".to_string())
}

pub fn contain_incident(system: &mut IncidentResponseSystem, incident_id: &str) -> Result<(), String> {
    let incident = find_incident_mut(system, incident_id)?;
    incident.status = IncidentStatus::Contained;
    incident
        .actions_taken
        .push(format!("Revoked secret: {}", incident.secret_path.clone()));
    Ok(())
}

pub fn record_rotation(
    system: &mut IncidentResponseSystem,
    incident_id: &str,
    consumer_count: usize,
) -> Result<(), String> {
    let incident = find_incident_mut(system, incident_id)?;
    incident.status = IncidentStatus::Rotating;
    incident
        .actions_taken
        .push(format!("Generated new secret: {}", incident.secret_path.clone()));
    incident
        .actions_taken
        .push(format!("Distributed new secret to {} consumers", consumer_count));
    Ok(())
}

pub fn record_audit_results(
    system: &mut IncidentResponseSystem,
    incident_id: &str,
    unauthorized_accesses: Vec<String>,
) -> Result<(), String> {
    let incident = find_incident_mut(system, incident_id)?;
    incident.status = IncidentStatus::Auditing;
    let count = unauthorized_accesses.len();
    incident.unauthorized_accesses = unauthorized_accesses;
    incident
        .actions_taken
        .push(format!("Audit completed: {} unauthorized access(es) found", count));
    Ok(())
}

pub fn close_incident(
    system: &mut IncidentResponseSystem,
    incident_id: &str,
    root_cause: &str,
    remediation: &str,
) -> Result<(), String> {
    let incident = find_incident_mut(system, incident_id)?;
    incident.status = IncidentStatus::Closed;
    incident
        .actions_taken
        .push(format!("Root cause: {}", root_cause));
    incident
        .actions_taken
        .push(format!("Remediation: {}", remediation));
    Ok(())
}

pub fn generate_incident_report(incident: &Incident) -> String {
    let mut report = format!(
        "Incident Report: {}\n========================\nSecret: {}\nSeverity: {:?}\nStatus: {:?}\nLeak Source: {}\n\nTimeline:\n",
        incident.id, incident.secret_path, incident.severity, incident.status, incident.leak_source
    );

    for (i, action) in incident.actions_taken.iter().enumerate() {
        report.push_str(&format!("  [{}] {}\n", i + 1, action));
    }

    report.push_str("\nUnauthorized Accesses:\n");
    if incident.unauthorized_accesses.is_empty() {
        report.push_str("  None detected\n");
    } else {
        for access in &incident.unauthorized_accesses {
            report.push_str(&format!("  - {}\n", access));
        }
    }

    report
}

pub fn get_incidents_by_severity<'a>(
    system: &'a IncidentResponseSystem,
    min_severity: Severity,
) -> Vec<&'a Incident> {
    system
        .incidents
        .iter()
        .filter(|i| i.severity >= min_severity)
        .collect()
}

pub fn calculate_mttc(system: &IncidentResponseSystem) -> Duration {
    let closed: Vec<&Incident> = system
        .incidents
        .iter()
        .filter(|i| i.status == IncidentStatus::Closed)
        .collect();

    if closed.is_empty() {
        return Duration::ZERO;
    }

    let total_actions_before_containment: usize = closed
        .iter()
        .map(|incident| {
            // Count actions up to and including the "Revoked" action
            incident
                .actions_taken
                .iter()
                .position(|a| a.starts_with("Revoked"))
                .map(|pos| pos + 1)
                .unwrap_or(0)
        })
        .sum();

    // Each action takes approximately 5 minutes
    let avg_actions = total_actions_before_containment as f64 / closed.len() as f64;
    Duration::from_secs((avg_actions * 300.0) as u64) // 300 seconds = 5 minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_system() -> IncidentResponseSystem {
        IncidentResponseSystem::new()
    }

    #[test]
    fn test_report_incident() {
        let mut system = create_test_system();
        let id = report_incident(&mut system, "db/password", Severity::Critical, "git-commit");
        assert_eq!(id, "INC-0001");
        assert_eq!(system.incidents.len(), 1);
        assert_eq!(system.incidents[0].status, IncidentStatus::Detected);
    }

    #[test]
    fn test_report_multiple_incidents() {
        let mut system = create_test_system();
        let id1 = report_incident(&mut system, "secret1", Severity::Low, "slack");
        let id2 = report_incident(&mut system, "secret2", Severity::High, "log-file");
        assert_eq!(id1, "INC-0001");
        assert_eq!(id2, "INC-0002");
    }

    #[test]
    fn test_contain_incident() {
        let mut system = create_test_system();
        let id = report_incident(&mut system, "db/password", Severity::Critical, "git-commit");
        assert!(contain_incident(&mut system, &id).is_ok());
        assert_eq!(system.incidents[0].status, IncidentStatus::Contained);
        assert!(!system.incidents[0].actions_taken.is_empty());
    }

    #[test]
    fn test_contain_nonexistent() {
        let mut system = create_test_system();
        assert!(contain_incident(&mut system, "INC-9999").is_err());
    }

    #[test]
    fn test_full_incident_lifecycle() {
        let mut system = create_test_system();
        let id = report_incident(&mut system, "api/key", Severity::High, "github");

        contain_incident(&mut system, &id).unwrap();
        record_rotation(&mut system, &id, 5).unwrap();
        record_audit_results(
            &mut system,
            &id,
            vec!["10.0.0.5".to_string(), "192.168.1.1".to_string()],
        )
        .unwrap();
        close_incident(
            &mut system,
            &id,
            "Developer committed .env file",
            "Added pre-commit hook",
        )
        .unwrap();

        let incident = &system.incidents[0];
        assert_eq!(incident.status, IncidentStatus::Closed);
        assert!(incident.actions_taken.len() >= 4);
        assert_eq!(incident.unauthorized_accesses.len(), 2);
    }

    #[test]
    fn test_generate_incident_report() {
        let mut system = create_test_system();
        let id = report_incident(&mut system, "db/password", Severity::Critical, "git-commit");
        contain_incident(&mut system, &id).unwrap();

        let report = generate_incident_report(&system.incidents[0]);
        assert!(report.contains("INC-0001"));
        assert!(report.contains("db/password"));
        assert!(report.contains("Critical"));
    }

    #[test]
    fn test_get_incidents_by_severity() {
        let mut system = create_test_system();
        report_incident(&mut system, "low_secret", Severity::Low, "slack");
        report_incident(&mut system, "high_secret", Severity::High, "git");
        report_incident(&mut system, "critical_secret", Severity::Critical, "public-paste");

        let high_and_above = get_incidents_by_severity(&system, Severity::High);
        assert_eq!(high_and_above.len(), 2);

        let critical_only = get_incidents_by_severity(&system, Severity::Critical);
        assert_eq!(critical_only.len(), 1);
    }

    #[test]
    fn test_record_audit_results() {
        let mut system = create_test_system();
        let id = report_incident(&mut system, "api/key", Severity::High, "github");
        contain_incident(&mut system, &id).unwrap();

        let accesses = vec!["attacker-ip-1".to_string(), "attacker-ip-2".to_string()];
        assert!(record_audit_results(&mut system, &id, accesses).is_ok());
        assert_eq!(system.incidents[0].unauthorized_accesses.len(), 2);
        assert_eq!(system.incidents[0].status, IncidentStatus::Auditing);
    }
}
