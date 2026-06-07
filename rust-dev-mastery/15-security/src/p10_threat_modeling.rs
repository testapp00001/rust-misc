// ===========================================================================
// Threat Modeling
//
// Threat modeling is the practice of systematically identifying what can go
// wrong in a system before writing (or shipping) code. This lesson covers
// attack surface analysis, the STRIDE threat classification framework,
// a security review checklist, and incident response planning.
//
// Key Concepts:
//   - Attack surface: all points where an adversary can interact with the system
//   - STRIDE: Spoofing, Tampering, Repudiation, Info disclosure, DoS, Elevation
//   - Security review checklist: structured way to audit code/config before release
//   - Incident response: structured plan for handling security breaches
//   - Risk scoring: likelihood x impact
// ===========================================================================

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. Attack Surface Analysis
// ---------------------------------------------------------------------------

/// A single entry point or interface that an attacker could target.
#[derive(Debug, Clone)]
pub struct AttackSurfaceEntry {
    pub name: String,
    pub category: SurfaceCategory,
    pub exposure: Exposure,
    pub description: String,
    pub mitigations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceCategory {
    /// Network-facing endpoints (HTTP, gRPC, WebSocket).
    Network,
    /// File system access (uploads, logs, configs).
    FileSystem,
    /// External service integrations (APIs, databases, queues).
    Integration,
    /// User-supplied input (forms, query params, headers).
    UserInput,
    /// Authentication / session management.
    Auth,
    /// Build and deployment pipeline.
    Pipeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exposure {
    /// Only reachable from a private network.
    Internal,
    /// Reachable through a controlled proxy or VPN.
    Restricted,
    /// Publicly accessible on the internet.
    Public,
}

impl fmt::Display for Exposure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal"),
            Self::Restricted => write!(f, "restricted"),
            Self::Public => write!(f, "public"),
        }
    }
}

/// Catalogs and analyzes the attack surface of a system.
#[derive(Debug)]
pub struct AttackSurface {
    entries: Vec<AttackSurfaceEntry>,
}

impl AttackSurface {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: AttackSurfaceEntry) {
        self.entries.push(entry);
    }

    /// Return entries sorted by exposure (public first).
    pub fn public_entries(&self) -> Vec<&AttackSurfaceEntry> {
        let mut public: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.exposure == Exposure::Public)
            .collect();
        public.sort_by(|a, b| a.name.cmp(&b.name));
        public
    }

    /// Return all entries grouped by category.
    pub fn by_category(&self) -> HashMap<SurfaceCategory, Vec<&AttackSurfaceEntry>> {
        let mut map: HashMap<SurfaceCategory, Vec<&AttackSurfaceEntry>> = HashMap::new();
        for entry in &self.entries {
            map.entry(entry.category).or_default().push(entry);
        }
        map
    }

    /// Summary: total entries and unmitigated count.
    pub fn summary(&self) -> SurfaceSummary {
        let total = self.entries.len();
        let unmitigated = self
            .entries
            .iter()
            .filter(|e| e.mitigations.is_empty())
            .count();
        let public = self.entries.iter()
            .filter(|e| e.exposure == Exposure::Public)
            .count();
        SurfaceSummary {
            total,
            unmitigated,
            public,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SurfaceSummary {
    pub total: usize,
    pub unmitigated: usize,
    pub public: usize,
}

// ---------------------------------------------------------------------------
// 2. STRIDE Threat Classification
// ---------------------------------------------------------------------------

/// The six STRIDE threat categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatCategory {
    /// Pretending to be someone else.
    Spoofing,
    /// Modifying data in transit or at rest.
    Tampering,
    /// Denying an action was performed.
    Repudiation,
    /// Leaking confidential information.
    InformationDisclosure,
    /// Making a service unavailable.
    DenialOfService,
    /// Gaining unauthorized privileges.
    ElevationOfPrivilege,
}

impl ThreatCategory {
    /// All six STRIDE categories.
    pub fn all() -> &'static [ThreatCategory] {
        &[
            Self::Spoofing,
            Self::Tampering,
            Self::Repudiation,
            Self::InformationDisclosure,
            Self::DenialOfService,
            Self::ElevationOfPrivilege,
        ]
    }

    /// Recommended mitigations for each category.
    pub fn default_mitigations(&self) -> &'static [&'static str] {
        match self {
            Self::Spoofing => &[
                "Enforce strong authentication (MFA)",
                "Validate session tokens",
                "Use mutual TLS for service-to-service calls",
            ],
            Self::Tampering => &[
                "Use HMAC or digital signatures",
                "Enforce TLS in transit",
                "Use database integrity constraints",
            ],
            Self::Repudiation => &[
                "Write immutable audit logs",
                "Use signed timestamps",
                "Require authentication for all state changes",
            ],
            Self::InformationDisclosure => &[
                "Encrypt data at rest and in transit",
                "Apply least-privilege access control",
                "Sanitize error messages and logs",
            ],
            Self::DenialOfService => &[
                "Apply rate limiting and throttling",
                "Use resource quotas",
                "Deploy behind a CDN / DDoS protection",
            ],
            Self::ElevationOfPrivilege => &[
                "Enforce least-privilege principle",
                "Validate authorization on every request",
                "Sandbox untrusted code execution",
            ],
        }
    }
}

impl fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spoofing => write!(f, "Spoofing"),
            Self::Tampering => write!(f, "Tampering"),
            Self::Repudiation => write!(f, "Repudiation"),
            Self::InformationDisclosure => write!(f, "Information Disclosure"),
            Self::DenialOfService => write!(f, "Denial of Service"),
            Self::ElevationOfPrivilege => write!(f, "Elevation of Privilege"),
        }
    }
}

/// A single identified threat.
#[derive(Debug, Clone)]
pub struct Threat {
    pub id: String,
    pub description: String,
    pub category: ThreatCategory,
    pub affected_component: String,
    pub risk: RiskLevel,
    pub mitigations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A threat model is a collection of identified threats for a system.
#[derive(Debug)]
pub struct ThreatModel {
    pub system_name: String,
    pub threats: Vec<Threat>,
}

impl ThreatModel {
    pub fn new(system_name: impl Into<String>) -> Self {
        Self {
            system_name: system_name.into(),
            threats: Vec::new(),
        }
    }

    pub fn add_threat(&mut self, threat: Threat) {
        self.threats.push(threat);
    }

    /// Filter threats by category.
    pub fn threats_by_category(&self, category: ThreatCategory) -> Vec<&Threat> {
        self.threats
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Filter threats above a given risk level.
    pub fn high_risk_threats(&self, min_risk: RiskLevel) -> Vec<&Threat> {
        self.threats
            .iter()
            .filter(|t| t.risk >= min_risk)
            .collect()
    }

    /// Generate a summary report.
    pub fn report(&self) -> String {
        let mut out = format!("Threat Model: {}\n", self.system_name);
        out.push_str(&format!("Total threats identified: {}\n\n", self.threats.len()));

        for category in ThreatCategory::all() {
            let threats = self.threats_by_category(*category);
            if !threats.is_empty() {
                out.push_str(&format!("[{category}] ({} threats)\n", threats.len()));
                for t in &threats {
                    out.push_str(&format!(
                        "  - {} [{}]: {} (component: {})\n",
                        t.id, t.risk, t.description, t.affected_component,
                    ));
                }
                out.push('\n');
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// 3. Security Review Checklist
// ---------------------------------------------------------------------------

/// A checklist item for security reviews.
#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub id: String,
    pub category: &'static str,
    pub question: &'static str,
    pub critical: bool,
}

/// Result of reviewing a single checklist item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewStatus {
    Pass,
    Fail,
    NotApplicable,
    Deferred,
}

/// A completed security review.
#[derive(Debug)]
pub struct SecurityReview {
    pub reviewer: String,
    pub results: Vec<(ChecklistItem, ReviewStatus, String)>, // item, status, notes
}

impl SecurityReview {
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, s, _)| *s == ReviewStatus::Pass)
            .count()
    }

    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, s, _)| *s == ReviewStatus::Fail)
            .count()
    }

    pub fn critical_failures(&self) -> Vec<&ChecklistItem> {
        self.results
            .iter()
            .filter(|(item, status, _)| item.critical && *status == ReviewStatus::Fail)
            .map(|(item, _, _)| item)
            .collect()
    }

    pub fn can_release(&self) -> bool {
        self.critical_failures().is_empty()
    }
}

/// Returns the standard security review checklist.
pub fn default_checklist() -> Vec<ChecklistItem> {
    vec![
        ChecklistItem {
            id: "AUTH-01".into(),
            category: "Authentication",
            question: "Does the system enforce strong password requirements?",
            critical: true,
        },
        ChecklistItem {
            id: "AUTH-02".into(),
            category: "Authentication",
            question: "Is multi-factor authentication available for privileged accounts?",
            critical: true,
        },
        ChecklistItem {
            id: "INP-01".into(),
            category: "Input Validation",
            question: "Are all user inputs validated and sanitized on the server side?",
            critical: true,
        },
        ChecklistItem {
            id: "INP-02".into(),
            category: "Input Validation",
            question: "Is parameterized SQL used to prevent injection?",
            critical: true,
        },
        ChecklistItem {
            id: "CRYP-01".into(),
            category: "Cryptography",
            question: "Is TLS 1.2+ enforced for all external connections?",
            critical: true,
        },
        ChecklistItem {
            id: "CRYP-02".into(),
            category: "Cryptography",
            question: "Are secrets stored in a vault, not in source code?",
            critical: true,
        },
        ChecklistItem {
            id: "LOG-01".into(),
            category: "Logging",
            question: "Do logs omit sensitive data (passwords, tokens, PII)?",
            critical: false,
        },
        ChecklistItem {
            id: "LOG-02".into(),
            category: "Logging",
            question: "Are security events logged with sufficient detail for forensics?",
            critical: false,
        },
        ChecklistItem {
            id: "DEP-01".into(),
            category: "Dependencies",
            question: "Has `cargo audit` been run with zero critical findings?",
            critical: true,
        },
        ChecklistItem {
            id: "DEP-02".into(),
            category: "Dependencies",
            question: "Are all dependencies pinned with integrity hashes?",
            critical: false,
        },
        ChecklistItem {
            id: "ERR-01".into(),
            category: "Error Handling",
            question: "Do error responses avoid leaking internal details?",
            critical: true,
        },
        ChecklistItem {
            id: "ERR-02".into(),
            category: "Error Handling",
            question: "Does the server handle panics gracefully without crashing?",
            critical: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// 4. Incident Response Plan
// ---------------------------------------------------------------------------

/// Severity of a security incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IncidentSeverity {
    /// Minor, no data exposure.
    Low,
    /// Potential data exposure or service degradation.
    Medium,
    /// Confirmed data breach or service outage.
    High,
    /// Active exploitation, widespread impact.
    Critical,
}

impl fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A security incident record.
#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub severity: IncidentSeverity,
    pub detected_at: u64, // unix timestamp
    pub status: IncidentStatus,
    pub actions_taken: Vec<String>,
    pub affected_systems: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentStatus {
    Detected,
    Triaged,
    Contained,
    Eradicated,
    Recovered,
    Closed,
}

/// Manages the lifecycle of security incidents.
#[derive(Debug)]
pub struct IncidentResponse {
    pub incidents: Vec<Incident>,
}

impl IncidentResponse {
    pub fn new() -> Self {
        Self {
            incidents: Vec::new(),
        }
    }

    pub fn report_incident(&mut self, incident: Incident) {
        self.incidents.push(incident);
    }

    /// Update the status of an incident.
    pub fn update_status(&mut self, id: &str, new_status: IncidentStatus) -> bool {
        if let Some(inc) = self.incidents.iter_mut().find(|i| i.id == id) {
            inc.status = new_status;
            true
        } else {
            false
        }
    }

    /// Record an action taken during incident response.
    pub fn record_action(&mut self, id: &str, action: impl Into<String>) -> bool {
        if let Some(inc) = self.incidents.iter_mut().find(|i| i.id == id) {
            inc.actions_taken.push(action.into());
            true
        } else {
            false
        }
    }

    /// Return all open incidents (not yet closed).
    pub fn open_incidents(&self) -> Vec<&Incident> {
        self.incidents
            .iter()
            .filter(|i| i.status != IncidentStatus::Closed)
            .collect()
    }

    /// Return incidents at or above a given severity.
    pub fn incidents_by_severity(&self, min: IncidentSeverity) -> Vec<&Incident> {
        self.incidents
            .iter()
            .filter(|i| i.severity >= min)
            .collect()
    }

    /// Generate a post-incident summary.
    pub fn post_mortem(&self, id: &str) -> Option<String> {
        let inc = self.incidents.iter().find(|i| i.id == id)?;
        let mut out = format!(
            "Post-Incident Report: {}\nTitle: {}\nSeverity: {}\nStatus: {:?}\n\n",
            inc.id, inc.title, inc.severity, inc.status,
        );
        out.push_str("Affected systems:\n");
        for sys in &inc.affected_systems {
            out.push_str(&format!("  - {sys}\n"));
        }
        out.push_str("\nActions taken:\n");
        for (i, action) in inc.actions_taken.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, action));
        }
        Some(out)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_attack_surface() -> AttackSurface {
        let mut surface = AttackSurface::new();
        surface.add_entry(AttackSurfaceEntry {
            name: "REST API".into(),
            category: SurfaceCategory::Network,
            exposure: Exposure::Public,
            description: "Main HTTP API".into(),
            mitigations: vec!["rate limiting".into(), "TLS".into()],
        });
        surface.add_entry(AttackSurfaceEntry {
            name: "Admin SSH".into(),
            category: SurfaceCategory::Network,
            exposure: Exposure::Restricted,
            description: "Admin access via SSH".into(),
            mitigations: vec!["VPN required".into()],
        });
        surface.add_entry(AttackSurfaceEntry {
            name: "File Upload".into(),
            category: SurfaceCategory::UserInput,
            exposure: Exposure::Public,
            description: "User file upload endpoint".into(),
            mitigations: vec![],
        });
        surface
    }

    #[test]
    fn test_attack_surface_summary() {
        let surface = sample_attack_surface();
        let summary = surface.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.public, 2);
        assert_eq!(summary.unmitigated, 1); // file upload
    }

    #[test]
    fn test_attack_surface_by_category() {
        let surface = sample_attack_surface();
        let by_cat = surface.by_category();
        assert_eq!(by_cat.get(&SurfaceCategory::Network).unwrap().len(), 2);
        assert_eq!(
            by_cat.get(&SurfaceCategory::UserInput).unwrap().len(),
            1
        );
    }

    #[test]
    fn test_stride_categories() {
        assert_eq!(ThreatCategory::all().len(), 6);
        // Every category has at least one mitigation
        for cat in ThreatCategory::all() {
            assert!(
                !cat.default_mitigations().is_empty(),
                "{cat} should have mitigations"
            );
        }
    }

    #[test]
    fn test_threat_model_report() {
        let mut model = ThreatModel::new("MyService");
        model.add_threat(Threat {
            id: "T-001".into(),
            description: "Forged JWT tokens".into(),
            category: ThreatCategory::Spoofing,
            affected_component: "auth-service".into(),
            risk: RiskLevel::High,
            mitigations: vec!["Use RS256 with key rotation".into()],
        });
        model.add_threat(Threat {
            id: "T-002".into(),
            description: "SQL injection in search".into(),
            category: ThreatCategory::Tampering,
            affected_component: "search-api".into(),
            risk: RiskLevel::Critical,
            mitigations: vec!["Parameterized queries".into()],
        });

        let report = model.report();
        assert!(report.contains("MyService"));
        assert!(report.contains("T-001"));
        assert!(report.contains("Spoofing"));
        assert!(report.contains("Tampering"));

        let high = model.high_risk_threats(RiskLevel::High);
        assert_eq!(high.len(), 2); // both are >= High
    }

    #[test]
    fn test_security_review_pass() {
        let checklist = default_checklist();
        assert!(checklist.iter().any(|c| c.critical));

        // Simulate a review where everything passes
        let review = SecurityReview {
            reviewer: "alice".into(),
            results: checklist
                .into_iter()
                .map(|item| (item, ReviewStatus::Pass, "ok".into()))
                .collect(),
        };
        assert_eq!(review.failed(), 0);
        assert!(review.can_release());
    }

    #[test]
    fn test_security_review_blocks_on_critical_fail() {
        let checklist = default_checklist();
        let review = SecurityReview {
            reviewer: "bob".into(),
            results: checklist
                .into_iter()
                .map(|item| {
                    let status = if item.id == "INP-01" {
                        ReviewStatus::Fail
                    } else {
                        ReviewStatus::Pass
                    };
                    (item, status, "".into())
                })
                .collect(),
        };
        assert_eq!(review.failed(), 1);
        assert_eq!(review.critical_failures().len(), 1);
        assert!(!review.can_release()); // INP-01 is critical
    }

    #[test]
    fn test_incident_response_lifecycle() {
        let mut ir = IncidentResponse::new();
        ir.report_incident(Incident {
            id: "INC-001".into(),
            title: "Suspicious login activity".into(),
            severity: IncidentSeverity::Medium,
            detected_at: 1700000000,
            status: IncidentStatus::Detected,
            actions_taken: Vec::new(),
            affected_systems: vec!["auth-service".into()],
        });

        assert_eq!(ir.open_incidents().len(), 1);

        ir.update_status("INC-001", IncidentStatus::Triaged);
        ir.record_action("INC-001", "Reviewed access logs");
        ir.record_action("INC-001", "Blocked suspicious IP range");
        ir.update_status("INC-001", IncidentStatus::Contained);
        ir.update_status("INC-001", IncidentStatus::Closed);

        assert_eq!(ir.open_incidents().len(), 0);

        let report = ir.post_mortem("INC-001").unwrap();
        assert!(report.contains("INC-001"));
        assert!(report.contains("Blocked suspicious IP range"));
    }

    #[test]
    fn test_incident_severity_filtering() {
        let mut ir = IncidentResponse::new();
        ir.report_incident(Incident {
            id: "INC-001".into(),
            title: "Low severity".into(),
            severity: IncidentSeverity::Low,
            detected_at: 0,
            status: IncidentStatus::Detected,
            actions_taken: vec![],
            affected_systems: vec![],
        });
        ir.report_incident(Incident {
            id: "INC-002".into(),
            title: "Critical breach".into(),
            severity: IncidentSeverity::Critical,
            detected_at: 0,
            status: IncidentStatus::Detected,
            actions_taken: vec![],
            affected_systems: vec![],
        });

        assert_eq!(ir.incidents_by_severity(IncidentSeverity::High).len(), 1);
        assert_eq!(ir.incidents_by_severity(IncidentSeverity::Low).len(), 2);
    }
}
