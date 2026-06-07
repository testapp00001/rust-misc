/// Problem: Project Management
///
/// Master project management in Rust.
///
/// Key Concepts:
/// - Planning
/// - Execution
/// - Monitoring
/// - Risk management
/// - Delivery

/// Problem 1: Project planning
/// Plan a project
pub struct ProjectPlan {
    pub name: String,
    pub phases: Vec<Phase>,
    pub timeline: u32,
}

pub struct Phase {
    pub name: String,
    pub duration: u32,
    pub tasks: Vec<String>,
}

impl ProjectPlan {
    pub fn new(name: &str, timeline: u32) -> Self {
        Self {
            name: name.to_string(),
            phases: Vec::new(),
            timeline,
        }
    }

    pub fn add_phase(&mut self, name: &str, duration: u32) {
        self.phases.push(Phase {
            name: name.to_string(),
            duration,
            tasks: Vec::new(),
        });
    }
}

/// Problem 2: Agile methodology
/// Implement agile practices
pub struct AgileTeam {
    pub sprint_length: u32,
    pub backlog: Vec<String>,
    pub current_sprint: Vec<String>,
}

impl AgileTeam {
    pub fn new(sprint_length: u32) -> Self {
        Self {
            sprint_length,
            backlog: Vec::new(),
            current_sprint: Vec::new(),
        }
    }

    pub fn add_to_backlog(&mut self, item: &str) {
        self.backlog.push(item.to_string());
    }

    pub fn start_sprint(&mut self) {
        self.current_sprint = self.backlog.drain(..).collect();
    }
}

/// Problem 3: Risk management
/// Manage project risks
pub struct RiskManager {
    pub risks: Vec<Risk>,
}

pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub mitigation: String,
}

impl RiskManager {
    pub fn new() -> Self {
        Self { risks: Vec::new() }
    }

    pub fn add_risk(&mut self, description: &str, probability: f64, impact: f64, mitigation: &str) {
        self.risks.push(Risk {
            description: description.to_string(),
            probability,
            impact,
            mitigation: mitigation.to_string(),
        });
    }

    pub fn high_risks(&self) -> Vec<&Risk> {
        self.risks.iter().filter(|r| r.probability * r.impact > 0.5).collect()
    }
}

/// Problem 4: Resource management
/// Manage resources
pub struct ResourceManager {
    pub resources: Vec<Resource>,
}

pub struct Resource {
    pub name: String,
    pub availability: f64,
    pub skills: Vec<String>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self { resources: Vec::new() }
    }

    pub fn add_resource(&mut self, name: &str, availability: f64, skills: Vec<String>) {
        self.resources.push(Resource {
            name: name.to_string(),
            availability,
            skills,
        });
    }
}

/// Problem 5: Timeline management
/// Manage project timeline
pub struct Timeline {
    pub milestones: Vec<Milestone>,
}

pub struct Milestone {
    pub name: String,
    pub date: String,
    pub completed: bool,
}

impl Timeline {
    pub fn new() -> Self {
        Self { milestones: Vec::new() }
    }

    pub fn add_milestone(&mut self, name: &str, date: &str) {
        self.milestones.push(Milestone {
            name: name.to_string(),
            date: date.to_string(),
            completed: false,
        });
    }

    pub fn complete_milestone(&mut self, name: &str) {
        if let Some(milestone) = self.milestones.iter_mut().find(|m| m.name == name) {
            milestone.completed = true;
        }
    }
}

/// Problem 6: Budget management
/// Manage project budget
pub struct Budget {
    pub total: f64,
    pub spent: f64,
    pub categories: std::collections::HashMap<String, f64>,
}

impl Budget {
    pub fn new(total: f64) -> Self {
        Self {
            total,
            spent: 0.0,
            categories: std::collections::HashMap::new(),
        }
    }

    pub fn spend(&mut self, amount: f64, category: &str) {
        self.spent += amount;
        *self.categories.entry(category.to_string()).or_insert(0.0) += amount;
    }

    pub fn remaining(&self) -> f64 {
        self.total - self.spent
    }
}

/// Problem 7: Quality management
/// Manage quality
pub struct QualityManager {
    pub standards: Vec<String>,
    pub metrics: std::collections::HashMap<String, f64>,
}

impl QualityManager {
    pub fn new() -> Self {
        Self {
            standards: Vec::new(),
            metrics: std::collections::HashMap::new(),
        }
    }

    pub fn add_standard(&mut self, standard: &str) {
        self.standards.push(standard.to_string());
    }

    pub fn set_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }
}

/// Problem 8: Communication management
/// Manage communication
pub struct CommunicationManager {
    pub stakeholders: Vec<String>,
    pub channels: Vec<String>,
    pub frequency: std::collections::HashMap<String, String>,
}

impl CommunicationManager {
    pub fn new() -> Self {
        Self {
            stakeholders: Vec::new(),
            channels: Vec::new(),
            frequency: std::collections::HashMap::new(),
        }
    }

    pub fn add_stakeholder(&mut self, name: &str) {
        self.stakeholders.push(name.to_string());
    }
}

/// Problem 9: Change management
/// Manage changes
pub struct ChangeManager {
    pub changes: Vec<Change>,
}

pub struct Change {
    pub description: String,
    pub impact: String,
    pub approved: bool,
}

impl ChangeManager {
    pub fn new() -> Self {
        Self { changes: Vec::new() }
    }

    pub fn request_change(&mut self, description: &str, impact: &str) {
        self.changes.push(Change {
            description: description.to_string(),
            impact: impact.to_string(),
            approved: false,
        });
    }

    pub fn approve_change(&mut self, description: &str) {
        if let Some(change) = self.changes.iter_mut().find(|c| c.description == description) {
            change.approved = true;
        }
    }
}

/// Problem 10: Delivery management
/// Manage delivery
pub struct DeliveryManager {
    pub deliverables: Vec<Deliverable>,
}

pub struct Deliverable {
    pub name: String,
    pub status: DeliveryStatus,
}

#[derive(Debug)]
pub enum DeliveryStatus {
    NotStarted,
    InProgress,
    Completed,
    Delayed,
}

impl DeliveryManager {
    pub fn new() -> Self {
        Self { deliverables: Vec::new() }
    }

    pub fn add_deliverable(&mut self, name: &str) {
        self.deliverables.push(Deliverable {
            name: name.to_string(),
            status: DeliveryStatus::NotStarted,
        });
    }

    pub fn update_status(&mut self, name: &str, status: DeliveryStatus) {
        if let Some(deliverable) = self.deliverables.iter_mut().find(|d| d.name == name) {
            deliverable.status = status;
        }
    }
}

/// Problem 11: Stakeholder management
/// Manage stakeholders
pub struct StakeholderManager {
    pub stakeholders: Vec<Stakeholder>,
}

pub struct Stakeholder {
    pub name: String,
    pub role: String,
    pub influence: f64,
}

impl StakeholderManager {
    pub fn new() -> Self {
        Self { stakeholders: Vec::new() }
    }

    pub fn add_stakeholder(&mut self, name: &str, role: &str, influence: f64) {
        self.stakeholders.push(Stakeholder {
            name: name.to_string(),
            role: role.to_string(),
            influence,
        });
    }
}

/// Problem 12: Scope management
/// Manage scope
pub struct ScopeManager {
    pub in_scope: Vec<String>,
    pub out_scope: Vec<String>,
}

impl ScopeManager {
    pub fn new() -> Self {
        Self {
            in_scope: Vec::new(),
            out_scope: Vec::new(),
        }
    }

    pub fn add_in_scope(&mut self, item: &str) {
        self.in_scope.push(item.to_string());
    }

    pub fn add_out_scope(&mut self, item: &str) {
        self.out_scope.push(item.to_string());
    }
}

/// Problem 13: Issue tracking
/// Track issues
pub struct IssueTracker {
    pub issues: Vec<Issue>,
}

pub struct Issue {
    pub id: u32,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
}

#[derive(Debug)]
pub enum IssueStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug)]
pub enum IssuePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl IssueTracker {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_issue(&mut self, title: &str, priority: IssuePriority) {
        let id = self.issues.len() as u32 + 1;
        self.issues.push(Issue {
            id,
            title: title.to_string(),
            status: IssueStatus::Open,
            priority,
        });
    }

    pub fn resolve_issue(&mut self, id: u32) {
        if let Some(issue) = self.issues.iter_mut().find(|i| i.id == id) {
            issue.status = IssueStatus::Resolved;
        }
    }
}

/// Problem 14: Reporting
/// Generate reports
pub struct ReportGenerator {
    pub reports: Vec<Report>,
}

pub struct Report {
    pub name: String,
    pub data: std::collections::HashMap<String, String>,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn generate(&mut self, name: &str, data: std::collections::HashMap<String, String>) {
        self.reports.push(Report {
            name: name.to_string(),
            data,
        });
    }
}

/// Problem 15: Lessons learned
/// Capture lessons learned
pub struct LessonsLearned {
    pub lessons: Vec<Lesson>,
}

pub struct Lesson {
    pub context: String,
    pub lesson: String,
    pub action: String,
}

impl LessonsLearned {
    pub fn new() -> Self {
        Self { lessons: Vec::new() }
    }

    pub fn add_lesson(&mut self, context: &str, lesson: &str, action: &str) {
        self.lessons.push(Lesson {
            context: context.to_string(),
            lesson: lesson.to_string(),
            action: action.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_plan() {
        let mut plan = ProjectPlan::new("Rust Migration", 12);
        plan.add_phase("Planning", 2);
        assert_eq!(plan.phases.len(), 1);
    }

    #[test]
    fn test_agile_team() {
        let mut team = AgileTeam::new(14);
        team.add_to_backlog("Feature 1");
        team.start_sprint();
        assert_eq!(team.current_sprint.len(), 1);
    }

    #[test]
    fn test_risk_manager() {
        let mut manager = RiskManager::new();
        manager.add_risk("Learning curve", 0.8, 0.8, "Training");
        assert_eq!(manager.high_risks().len(), 1);
    }

    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        manager.add_resource("Alice", 1.0, vec!["Rust".to_string()]);
        assert_eq!(manager.resources.len(), 1);
    }

    #[test]
    fn test_timeline() {
        let mut timeline = Timeline::new();
        timeline.add_milestone("M1", "2024-01-01");
        timeline.complete_milestone("M1");
        assert!(timeline.milestones[0].completed);
    }

    #[test]
    fn test_budget() {
        let mut budget = Budget::new(100000.0);
        budget.spend(50000.0, "Development");
        assert_eq!(budget.remaining(), 50000.0);
    }

    #[test]
    fn test_quality_manager() {
        let mut manager = QualityManager::new();
        manager.add_standard("Code coverage > 80%");
        assert_eq!(manager.standards.len(), 1);
    }

    #[test]
    fn test_communication_manager() {
        let mut manager = CommunicationManager::new();
        manager.add_stakeholder("CEO");
        assert_eq!(manager.stakeholders.len(), 1);
    }

    #[test]
    fn test_change_manager() {
        let mut manager = ChangeManager::new();
        manager.request_change("Add feature X", "High");
        manager.approve_change("Add feature X");
        assert!(manager.changes[0].approved);
    }

    #[test]
    fn test_delivery_manager() {
        let mut manager = DeliveryManager::new();
        manager.add_deliverable("API");
        manager.update_status("API", DeliveryStatus::Completed);
    }

    #[test]
    fn test_stakeholder_manager() {
        let mut manager = StakeholderManager::new();
        manager.add_stakeholder("Alice", "PM", 0.9);
        assert_eq!(manager.stakeholders.len(), 1);
    }

    #[test]
    fn test_scope_manager() {
        let mut manager = ScopeManager::new();
        manager.add_in_scope("Feature A");
        manager.add_out_scope("Feature B");
        assert_eq!(manager.in_scope.len(), 1);
    }

    #[test]
    fn test_issue_tracker() {
        let mut tracker = IssueTracker::new();
        tracker.add_issue("Bug 1", IssuePriority::High);
        tracker.resolve_issue(1);
    }

    #[test]
    fn test_report_generator() {
        let mut generator = ReportGenerator::new();
        let mut data = std::collections::HashMap::new();
        data.insert("progress".to_string(), "50%".to_string());
        generator.generate("Weekly", data);
        assert_eq!(generator.reports.len(), 1);
    }

    #[test]
    fn test_lessons_learned() {
        let mut lessons = LessonsLearned::new();
        lessons.add_lesson("Migration", "Start small", "Pilot project");
        assert_eq!(lessons.lessons.len(), 1);
    }
}
