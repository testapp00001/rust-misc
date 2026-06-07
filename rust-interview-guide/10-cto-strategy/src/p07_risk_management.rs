/// Problem: Risk Management
///
/// Master risk management in Rust.
///
/// Key Concepts:
/// - Risk identification
/// - Risk assessment
/// - Risk mitigation
/// - Risk monitoring
/// - Risk reporting

/// Problem 1: Risk identification
/// Identify risks
pub struct RiskIdentifier {
    pub risks: Vec<IdentifiedRisk>,
}

pub struct IdentifiedRisk {
    pub id: u32,
    pub description: String,
    pub category: RiskCategory,
    pub source: String,
}

#[derive(Debug)]
pub enum RiskCategory {
    Technical,
    Operational,
    Financial,
    Strategic,
    Compliance,
}

impl RiskIdentifier {
    pub fn new() -> Self {
        Self { risks: Vec::new() }
    }

    pub fn identify(&mut self, description: &str, category: RiskCategory, source: &str) {
        let id = self.risks.len() as u32 + 1;
        self.risks.push(IdentifiedRisk {
            id,
            description: description.to_string(),
            category,
            source: source.to_string(),
        });
    }
}

/// Problem 2: Risk assessment
/// Assess risks
pub struct RiskAssessor {
    pub assessments: Vec<RiskAssessment>,
}

pub struct RiskAssessment {
    pub risk_id: u32,
    pub probability: f64,
    pub impact: f64,
    pub severity: RiskSeverity,
}

#[derive(Debug)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskAssessor {
    pub fn new() -> Self {
        Self { assessments: Vec::new() }
    }

    pub fn assess(&mut self, risk_id: u32, probability: f64, impact: f64) {
        let severity = if probability * impact > 0.7 {
            RiskSeverity::Critical
        } else if probability * impact > 0.5 {
            RiskSeverity::High
        } else if probability * impact > 0.3 {
            RiskSeverity::Medium
        } else {
            RiskSeverity::Low
        };

        self.assessments.push(RiskAssessment {
            risk_id,
            probability,
            impact,
            severity,
        });
    }

    pub fn critical_risks(&self) -> Vec<&RiskAssessment> {
        self.assessments.iter().filter(|a| matches!(a.severity, RiskSeverity::Critical)).collect()
    }
}

/// Problem 3: Risk mitigation
/// Mitigate risks
pub struct RiskMitigator {
    pub mitigations: Vec<Mitigation>,
}

pub struct Mitigation {
    pub risk_id: u32,
    pub strategy: MitigationStrategy,
    pub actions: Vec<String>,
    pub owner: String,
}

#[derive(Debug)]
pub enum MitigationStrategy {
    Avoid,
    Transfer,
    Mitigate,
    Accept,
}

impl RiskMitigator {
    pub fn new() -> Self {
        Self { mitigations: Vec::new() }
    }

    pub fn mitigate(&mut self, risk_id: u32, strategy: MitigationStrategy, actions: Vec<String>, owner: &str) {
        self.mitigations.push(Mitigation {
            risk_id,
            strategy,
            actions,
            owner: owner.to_string(),
        });
    }
}

/// Problem 4: Risk monitoring
/// Monitor risks
pub struct RiskMonitor {
    pub monitored_risks: Vec<MonitoredRisk>,
}

pub struct MonitoredRisk {
    pub risk_id: u32,
    pub status: RiskStatus,
    pub last_review: String,
    pub next_review: String,
}

#[derive(Debug)]
pub enum RiskStatus {
    Open,
    Mitigated,
    Closed,
    Escalated,
}

impl RiskMonitor {
    pub fn new() -> Self {
        Self { monitored_risks: Vec::new() }
    }

    pub fn monitor(&mut self, risk_id: u32, status: RiskStatus, last_review: &str, next_review: &str) {
        self.monitored_risks.push(MonitoredRisk {
            risk_id,
            status,
            last_review: last_review.to_string(),
            next_review: next_review.to_string(),
        });
    }

    pub fn escalate(&mut self, risk_id: u32) {
        if let Some(risk) = self.monitored_risks.iter_mut().find(|r| r.risk_id == risk_id) {
            risk.status = RiskStatus::Escalated;
        }
    }
}

/// Problem 5: Risk reporting
/// Report risks
pub struct RiskReporter {
    pub reports: Vec<RiskReport>,
}

pub struct RiskReport {
    pub date: String,
    pub risks: Vec<RiskSummary>,
    pub recommendations: Vec<String>,
}

pub struct RiskSummary {
    pub risk_id: u32,
    pub description: String,
    pub severity: String,
    pub status: String,
}

impl RiskReporter {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn report(&mut self, date: &str, risks: Vec<RiskSummary>, recommendations: Vec<String>) {
        self.reports.push(RiskReport {
            date: date.to_string(),
            risks,
            recommendations,
        });
    }
}

/// Problem 6: Risk register
/// Maintain risk register
pub struct RiskRegister {
    pub entries: Vec<RiskEntry>,
}

pub struct RiskEntry {
    pub id: u32,
    pub description: String,
    pub category: String,
    pub probability: f64,
    pub impact: f64,
    pub mitigation: String,
    pub owner: String,
    pub status: String,
}

impl RiskRegister {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, description: &str, category: &str, probability: f64, impact: f64, mitigation: &str, owner: &str) {
        let id = self.entries.len() as u32 + 1;
        self.entries.push(RiskEntry {
            id,
            description: description.to_string(),
            category: category.to_string(),
            probability,
            impact,
            mitigation: mitigation.to_string(),
            owner: owner.to_string(),
            status: "Open".to_string(),
        });
    }

    pub fn update_status(&mut self, id: u32, status: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.status = status.to_string();
        }
    }
}

/// Problem 7: Risk appetite
/// Define risk appetite
pub struct RiskAppetite {
    pub categories: std::collections::HashMap<String, RiskTolerance>,
}

pub struct RiskTolerance {
    pub min: f64,
    pub max: f64,
}

impl RiskAppetite {
    pub fn new() -> Self {
        Self {
            categories: std::collections::HashMap::new(),
        }
    }

    pub fn set_tolerance(&mut self, category: &str, min: f64, max: f64) {
        self.categories.insert(category.to_string(), RiskTolerance { min, max });
    }

    pub fn is_acceptable(&self, category: &str, risk_level: f64) -> bool {
        if let Some(tolerance) = self.categories.get(category) {
            risk_level >= tolerance.min && risk_level <= tolerance.max
        } else {
            false
        }
    }
}

/// Problem 8: Risk response planning
/// Plan risk responses
pub struct RiskResponsePlanner {
    pub plans: Vec<RiskResponsePlan>,
}

pub struct RiskResponsePlan {
    pub risk_id: u32,
    pub response_type: ResponseType,
    pub actions: Vec<String>,
    pub timeline: String,
    pub budget: f64,
}

#[derive(Debug)]
pub enum ResponseType {
    Avoid,
    Mitigate,
    Transfer,
    Accept,
}

impl RiskResponsePlanner {
    pub fn new() -> Self {
        Self { plans: Vec::new() }
    }

    pub fn plan(&mut self, risk_id: u32, response_type: ResponseType, actions: Vec<String>, timeline: &str, budget: f64) {
        self.plans.push(RiskResponsePlan {
            risk_id,
            response_type,
            actions,
            timeline: timeline.to_string(),
            budget,
        });
    }
}

/// Problem 9: Risk quantification
/// Quantify risks
pub struct RiskQuantifier {
    pub quantifications: Vec<RiskQuantification>,
}

pub struct RiskQuantification {
    pub risk_id: u32,
    pub expected_loss: f64,
    pub worst_case: f64,
    pub best_case: f64,
}

impl RiskQuantifier {
    pub fn new() -> Self {
        Self { quantifications: Vec::new() }
    }

    pub fn quantify(&mut self, risk_id: u32, expected_loss: f64, worst_case: f64, best_case: f64) {
        self.quantifications.push(RiskQuantification {
            risk_id,
            expected_loss,
            worst_case,
            best_case,
        });
    }

    pub fn total_expected_loss(&self) -> f64 {
        self.quantifications.iter().map(|q| q.expected_loss).sum()
    }
}

/// Problem 10: Risk trend analysis
/// Analyze risk trends
pub struct RiskTrendAnalyzer {
    pub trends: Vec<RiskTrend>,
}

pub struct RiskTrend {
    pub period: String,
    pub risk_count: u32,
    pub avg_severity: f64,
}

impl RiskTrendAnalyzer {
    pub fn new() -> Self {
        Self { trends: Vec::new() }
    }

    pub fn add_trend(&mut self, period: &str, risk_count: u32, avg_severity: f64) {
        self.trends.push(RiskTrend {
            period: period.to_string(),
            risk_count,
            avg_severity,
        });
    }

    pub fn is_improving(&self) -> bool {
        if self.trends.len() < 2 {
            return true;
        }
        let last = self.trends.last().unwrap();
        let prev = &self.trends[self.trends.len() - 2];
        last.avg_severity < prev.avg_severity
    }
}

/// Problem 11: Risk communication
/// Communicate risks
pub struct RiskCommunicator {
    pub communications: Vec<RiskCommunication>,
}

pub struct RiskCommunication {
    pub risk_id: u32,
    pub audience: String,
    pub message: String,
    pub channel: String,
}

impl RiskCommunicator {
    pub fn new() -> Self {
        Self { communications: Vec::new() }
    }

    pub fn communicate(&mut self, risk_id: u32, audience: &str, message: &str, channel: &str) {
        self.communications.push(RiskCommunication {
            risk_id,
            audience: audience.to_string(),
            message: message.to_string(),
            channel: channel.to_string(),
        });
    }
}

/// Problem 12: Risk governance
/// Govern risks
pub struct RiskGovernance {
    pub policies: Vec<String>,
    pub roles: Vec<String>,
    pub processes: Vec<String>,
}

impl RiskGovernance {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            roles: Vec::new(),
            processes: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: &str) {
        self.policies.push(policy.to_string());
    }

    pub fn add_role(&mut self, role: &str) {
        self.roles.push(role.to_string());
    }
}

/// Problem 13: Risk culture
/// Build risk culture
pub struct RiskCulture {
    pub awareness_programs: Vec<String>,
    pub training: Vec<String>,
    pub incentives: Vec<String>,
}

impl RiskCulture {
    pub fn new() -> Self {
        Self {
            awareness_programs: Vec::new(),
            training: Vec::new(),
            incentives: Vec::new(),
        }
    }

    pub fn add_awareness_program(&mut self, program: &str) {
        self.awareness_programs.push(program.to_string());
    }
}

/// Problem 14: Risk technology
/// Use risk technology
pub struct RiskTechnology {
    pub tools: Vec<String>,
    pub systems: Vec<String>,
}

impl RiskTechnology {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_tool(&mut self, tool: &str) {
        self.tools.push(tool.to_string());
    }
}

/// Problem 15: Risk integration
/// Integrate risk management
pub struct RiskIntegrator {
    pub integrations: Vec<String>,
}

impl RiskIntegrator {
    pub fn new() -> Self {
        Self { integrations: Vec::new() }
    }

    pub fn integrate(&mut self, system: &str) {
        self.integrations.push(system.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_identification() {
        let mut identifier = RiskIdentifier::new();
        identifier.identify("Learning curve", RiskCategory::Technical, "Team");
        assert_eq!(identifier.risks.len(), 1);
    }

    #[test]
    fn test_risk_assessment() {
        let mut assessor = RiskAssessor::new();
        assessor.assess(1, 0.8, 0.9);
        assert_eq!(assessor.critical_risks().len(), 1);
    }

    #[test]
    fn test_risk_mitigation() {
        let mut mitigator = RiskMitigator::new();
        mitigator.mitigate(1, MitigationStrategy::Mitigate, vec!["Training".to_string()], "CTO");
        assert_eq!(mitigator.mitigations.len(), 1);
    }

    #[test]
    fn test_risk_monitoring() {
        let mut monitor = RiskMonitor::new();
        monitor.monitor(1, RiskStatus::Open, "2024-01-01", "2024-02-01");
        monitor.escalate(1);
    }

    #[test]
    fn test_risk_register() {
        let mut register = RiskRegister::new();
        register.add_entry("Learning curve", "Technical", 0.8, 0.6, "Training", "CTO");
        register.update_status(1, "Mitigated");
        assert_eq!(register.entries.len(), 1);
    }

    #[test]
    fn test_risk_appetite() {
        let mut appetite = RiskAppetite::new();
        appetite.set_tolerance("Technical", 0.0, 0.5);
        assert!(appetite.is_acceptable("Technical", 0.3));
        assert!(!appetite.is_acceptable("Technical", 0.8));
    }

    #[test]
    fn test_risk_response() {
        let mut planner = RiskResponsePlanner::new();
        planner.plan(1, ResponseType::Mitigate, vec!["Training".to_string()], "Q1", 10000.0);
        assert_eq!(planner.plans.len(), 1);
    }

    #[test]
    fn test_risk_quantification() {
        let mut quantifier = RiskQuantifier::new();
        quantifier.quantify(1, 50000.0, 100000.0, 10000.0);
        assert_eq!(quantifier.total_expected_loss(), 50000.0);
    }

    #[test]
    fn test_risk_trends() {
        let mut analyzer = RiskTrendAnalyzer::new();
        analyzer.add_trend("Q1", 10, 0.6);
        analyzer.add_trend("Q2", 8, 0.4);
        assert!(analyzer.is_improving());
    }

    #[test]
    fn test_risk_communication() {
        let mut communicator = RiskCommunicator::new();
        communicator.communicate(1, "Team", "Risk update", "Email");
        assert_eq!(communicator.communications.len(), 1);
    }

    #[test]
    fn test_risk_governance() {
        let mut governance = RiskGovernance::new();
        governance.add_policy("Risk appetite statement");
        assert_eq!(governance.policies.len(), 1);
    }

    #[test]
    fn test_risk_culture() {
        let mut culture = RiskCulture::new();
        culture.add_awareness_program("Risk workshop");
        assert_eq!(culture.awareness_programs.len(), 1);
    }

    #[test]
    fn test_risk_technology() {
        let mut tech = RiskTechnology::new();
        tech.add_tool("Risk register");
        assert_eq!(tech.tools.len(), 1);
    }

    #[test]
    fn test_risk_integration() {
        let mut integrator = RiskIntegrator::new();
        integrator.integrate("Project management");
        assert_eq!(integrator.integrations.len(), 1);
    }
}
