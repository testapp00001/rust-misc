//! # Lesson 10: Secret Leak Incident Response
//!
//! ## The Scenario
//!
//! A secret has been leaked. Maybe it was:
//! - Committed to a public git repository
//! - Posted in a Slack channel
//! - Found in a log file
//! - Discovered by an external security researcher
//! - Detected by a secret scanning service
//!
//! ## Incident Response Steps
//!
//! ```text
//! 1. DETECT    — How was the leak discovered?
//! 2. CONTAIN   — Revoke the compromised secret immediately
//! 3. ROTATE    — Generate and distribute a new secret
//! 4. AUDIT     — Check logs for unauthorized usage of the leaked secret
//! 5. REMEDIATE — Fix the root cause (why was it leaked?)
//! 6. DOCUMENT  — Write an incident report
//! 7. PREVENT   — Implement controls to prevent recurrence
//! ```
//!
//! ## Time is Critical
//!
//! Every minute a leaked secret is active is a minute an attacker can use it.
//! The goal is to minimize "time to revoke" — the gap between leak and revocation.
//!
//! ## Attack: Exploiting Leaked Secrets
//!
//! Automated bots scan GitHub, Pastebin, and other sites for leaked credentials:
//! - Average time to exploit a leaked AWS key: < 5 minutes
//! - Automated scanners check thousands of repos per minute
//! - Even "force-pushed" secrets remain in git history
//!
//! ## Defense
//!
//! 1. Automate secret scanning in CI/CD
//! 2. Pre-commit hooks to catch secrets before they're committed
//! 3. Incident response playbooks for secret leaks
//! 4. Automated revocation and rotation
//! 5. Tabletop exercises to practice response

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Severity level of a secret leak incident.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Status of an incident response.
#[derive(Debug, Clone, PartialEq)]
pub enum IncidentStatus {
    Detected,
    Contained,
    Rotating,
    Auditing,
    Remediated,
    Closed,
}

/// A secret leak incident.
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

/// The incident response system.
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

/// Exercise 1: Report a new secret leak incident.
///
/// Create a new Incident with:
/// - Auto-generated ID (format: "INC-{next_id:04}")
/// - Status: Detected
/// - Empty actions_taken and unauthorized_accesses
/// - detected_at: Instant::now()
///
/// Add it to the system's incidents list and return the incident ID.
///
/// Hints:
/// - Increment `next_id` before creating the incident
/// - Format the ID as `INC-{:04}`
/// - Push the incident to the Vec
/// - Return the ID string
pub fn report_incident(
    system: &mut IncidentResponseSystem,
    secret_path: &str,
    severity: Severity,
    leak_source: &str,
) -> String {
    todo!("Report a new secret leak incident")
}

/// Exercise 2: Contain an incident by adding a containment action.
///
/// Find the incident by ID and:
/// 1. Change its status to Contained
/// 2. Add "Revoked secret: {secret_path}" to actions_taken
///
/// Return `Ok(())` if found, `Err("Incident not found")` if not.
///
/// Hints:
/// - Find the incident by `id`
/// - Update `status` to `IncidentStatus::Contained`
/// - Push the action string
pub fn contain_incident(system: &mut IncidentResponseSystem, incident_id: &str) -> Result<(), String> {
    todo!("Contain an incident by revoking the secret")
}

/// Exercise 3: Record that a secret was rotated after containment.
///
/// Find the incident by ID and:
/// 1. Change its status to Rotating
/// 2. Add "Generated new secret: {secret_path}" to actions_taken
/// 3. Add "Distributed new secret to N consumers" to actions_taken
///
/// Return `Ok(())` if found, `Err(msg)` if not.
pub fn record_rotation(system: &mut IncidentResponseSystem, incident_id: &str, consumer_count: usize) -> Result<(), String> {
    todo!("Record that a secret was rotated")
}

/// Exercise 4: Record unauthorized access attempts found during audit.
///
/// Find the incident by ID and:
/// 1. Change its status to Auditing
/// 2. Add each access string to unauthorized_accesses
/// 3. Add "Audit completed: {count} unauthorized access(es) found" to actions_taken
///
/// Return `Ok(())` if found, `Err(msg)` if not.
pub fn record_audit_results(
    system: &mut IncidentResponseSystem,
    incident_id: &str,
    unauthorized_accesses: Vec<String>,
) -> Result<(), String> {
    todo!("Record audit results for an incident")
}

/// Exercise 5: Close an incident with a root cause and remediation.
///
/// Find the incident by ID and:
/// 1. Change its status to Closed
/// 2. Add "Root cause: {root_cause}" to actions_taken
/// 3. Add "Remediation: {remediation}" to actions_taken
///
/// Return `Ok(())` if found, `Err(msg)` if not.
pub fn close_incident(
    system: &mut IncidentResponseSystem,
    incident_id: &str,
    root_cause: &str,
    remediation: &str,
) -> Result<(), String> {
    todo!("Close an incident with root cause and remediation")
}

/// Exercise 6: Generate an incident report.
///
/// Return a human-readable string for the given incident.
///
/// Format:
/// ```
/// Incident Report: INC-0001
/// ========================
/// Secret: db/password
/// Severity: Critical
/// Status: Closed
/// Leak Source: git-commit
///
/// Timeline:
///   [1] Revoked secret: db/password
///   [2] Generated new secret: db/password
///   ...
///
/// Unauthorized Accesses:
///   - 10.0.0.5 at 2024-01-15
///   - 192.168.1.100 at 2024-01-15
/// ```
///
/// Hints:
/// - Use the incident fields
/// - Number the actions
/// - List unauthorized accesses (or "None detected" if empty)
pub fn generate_incident_report(incident: &Incident) -> String {
    todo!("Generate a human-readable incident report")
}

/// Exercise 7: Get all incidents with a specific severity or higher.
///
/// Return a Vec of references to incidents with severity >= the given threshold.
///
/// Severity ordering: Low < Medium < High < Critical
///
/// Hints:
/// - Use the `PartialOrd` derive on Severity
/// - Filter by `incident.severity >= min_severity`
pub fn get_incidents_by_severity<'a>(
    system: &'a IncidentResponseSystem,
    min_severity: Severity,
) -> Vec<&'a Incident> {
    todo!("Get incidents filtered by severity")
}

/// Exercise 8: Calculate the mean time to contain (MTTC) for closed incidents.
///
/// For all closed incidents, calculate the average time between detection
/// and containment (approximated as the number of actions taken * a fixed
/// duration per action, since we can't easily diff Instants).
///
/// Use a simplified model: each action takes approximately 5 minutes.
/// MTTC = (number of actions before containment) * 5 minutes.
///
/// Return the average MTTC across all closed incidents, or Duration::ZERO
/// if there are no closed incidents.
///
/// Hints:
/// - Filter for closed incidents
/// - For each, count actions that happened before containment
///   (up to and including the "Revoked" action)
/// - Compute average across all closed incidents
pub fn calculate_mttc(system: &IncidentResponseSystem) -> Duration {
    todo!("Calculate mean time to contain for closed incidents")
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
        close_incident(&mut system, &id, "Developer committed .env file", "Added pre-commit hook").unwrap();

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
