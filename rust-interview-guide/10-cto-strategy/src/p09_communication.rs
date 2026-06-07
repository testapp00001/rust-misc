/// Problem: Communication
///
/// Master communication in Rust.
///
/// Key Concepts:
/// - Stakeholder communication
/// - Technical communication
/// - Executive communication
/// - Crisis communication
/// - Change communication

/// Problem 1: Stakeholder communication
/// Communicate with stakeholders
pub struct StakeholderCommunicator {
    pub communications: Vec<StakeholderMessage>,
}

pub struct StakeholderMessage {
    pub stakeholder: String,
    pub topic: String,
    pub message: String,
    pub channel: String,
    pub date: String,
}

impl StakeholderCommunicator {
    pub fn new() -> Self {
        Self { communications: Vec::new() }
    }

    pub fn communicate(&mut self, stakeholder: &str, topic: &str, message: &str, channel: &str, date: &str) {
        self.communications.push(StakeholderMessage {
            stakeholder: stakeholder.to_string(),
            topic: topic.to_string(),
            message: message.to_string(),
            channel: channel.to_string(),
            date: date.to_string(),
        });
    }
}

/// Problem 2: Technical communication
/// Communicate technical information
pub struct TechnicalCommunicator {
    pub documents: Vec<TechnicalDocument>,
}

pub struct TechnicalDocument {
    pub title: String,
    pub audience: String,
    pub content: String,
    pub format: String,
}

impl TechnicalCommunicator {
    pub fn new() -> Self {
        Self { documents: Vec::new() }
    }

    pub fn create_document(&mut self, title: &str, audience: &str, content: &str, format: &str) {
        self.documents.push(TechnicalDocument {
            title: title.to_string(),
            audience: audience.to_string(),
            content: content.to_string(),
            format: format.to_string(),
        });
    }
}

/// Problem 3: Executive communication
/// Communicate with executives
pub struct ExecutiveCommunicator {
    pub reports: Vec<ExecutiveReport>,
}

pub struct ExecutiveReport {
    pub title: String,
    pub summary: String,
    pub key_metrics: std::collections::HashMap<String, String>,
    pub recommendations: Vec<String>,
}

impl ExecutiveCommunicator {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn create_report(&mut self, title: &str, summary: &str, key_metrics: std::collections::HashMap<String, String>, recommendations: Vec<String>) {
        self.reports.push(ExecutiveReport {
            title: title.to_string(),
            summary: summary.to_string(),
            key_metrics,
            recommendations,
        });
    }
}

/// Problem 4: Crisis communication
/// Handle crisis communication
pub struct CrisisCommunicator {
    pub messages: Vec<CrisisMessage>,
}

pub struct CrisisMessage {
    pub crisis: String,
    pub audience: String,
    pub message: String,
    pub channel: String,
    pub timestamp: String,
}

impl CrisisCommunicator {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn communicate(&mut self, crisis: &str, audience: &str, message: &str, channel: &str, timestamp: &str) {
        self.messages.push(CrisisMessage {
            crisis: crisis.to_string(),
            audience: audience.to_string(),
            message: message.to_string(),
            channel: channel.to_string(),
            timestamp: timestamp.to_string(),
        });
    }
}

/// Problem 5: Change communication
/// Communicate changes
pub struct ChangeCommunicator {
    pub messages: Vec<ChangeMessage>,
}

pub struct ChangeMessage {
    pub change: String,
    pub impact: String,
    pub audience: String,
    pub timeline: String,
}

impl ChangeCommunicator {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn communicate(&mut self, change: &str, impact: &str, audience: &str, timeline: &str) {
        self.messages.push(ChangeMessage {
            change: change.to_string(),
            impact: impact.to_string(),
            audience: audience.to_string(),
            timeline: timeline.to_string(),
        });
    }
}

/// Problem 6: Team communication
/// Communicate with team
pub struct TeamCommunicator {
    pub updates: Vec<TeamUpdate>,
}

pub struct TeamUpdate {
    pub topic: String,
    pub message: String,
    pub priority: Priority,
    pub date: String,
}

#[derive(Debug)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl TeamCommunicator {
    pub fn new() -> Self {
        Self { updates: Vec::new() }
    }

    pub fn update(&mut self, topic: &str, message: &str, priority: Priority, date: &str) {
        self.updates.push(TeamUpdate {
            topic: topic.to_string(),
            message: message.to_string(),
            priority,
            date: date.to_string(),
        });
    }
}

/// Problem 7: Presentation
/// Create presentations
pub struct PresentationCreator {
    pub presentations: Vec<Presentation>,
}

pub struct Presentation {
    pub title: String,
    pub audience: String,
    pub slides: Vec<Slide>,
}

pub struct Slide {
    pub title: String,
    pub content: String,
}

impl PresentationCreator {
    pub fn new() -> Self {
        Self { presentations: Vec::new() }
    }

    pub fn create(&mut self, title: &str, audience: &str) {
        self.presentations.push(Presentation {
            title: title.to_string(),
            audience: audience.to_string(),
            slides: Vec::new(),
        });
    }

    pub fn add_slide(&mut self, presentation_title: &str, slide_title: &str, content: &str) {
        if let Some(presentation) = self.presentations.iter_mut().find(|p| p.title == presentation_title) {
            presentation.slides.push(Slide {
                title: slide_title.to_string(),
                content: content.to_string(),
            });
        }
    }
}

/// Problem 8: Documentation
/// Create documentation
pub struct DocumentationCreator {
    pub documents: Vec<Documentation>,
}

pub struct Documentation {
    pub title: String,
    pub type_: DocType,
    pub content: String,
    pub audience: String,
}

#[derive(Debug)]
pub enum DocType {
    Technical,
    User,
    API,
    Architecture,
}

impl DocumentationCreator {
    pub fn new() -> Self {
        Self { documents: Vec::new() }
    }

    pub fn create(&mut self, title: &str, type_: DocType, content: &str, audience: &str) {
        self.documents.push(Documentation {
            title: title.to_string(),
            type_,
            content: content.to_string(),
            audience: audience.to_string(),
        });
    }
}

/// Problem 9: Meeting management
/// Manage meetings
pub struct MeetingManager {
    pub meetings: Vec<Meeting>,
}

pub struct Meeting {
    pub title: String,
    pub participants: Vec<String>,
    pub agenda: Vec<String>,
    pub outcomes: Vec<String>,
}

impl MeetingManager {
    pub fn new() -> Self {
        Self { meetings: Vec::new() }
    }

    pub fn schedule(&mut self, title: &str, participants: Vec<String>, agenda: Vec<String>) {
        self.meetings.push(Meeting {
            title: title.to_string(),
            participants,
            agenda,
            outcomes: Vec::new(),
        });
    }

    pub fn add_outcome(&mut self, meeting_title: &str, outcome: &str) {
        if let Some(meeting) = self.meetings.iter_mut().find(|m| m.title == meeting_title) {
            meeting.outcomes.push(outcome.to_string());
        }
    }
}

/// Problem 10: Feedback management
/// Manage feedback
pub struct FeedbackManager {
    pub feedback: Vec<Feedback>,
}

pub struct Feedback {
    pub from: String,
    pub to: String,
    pub message: String,
    pub category: FeedbackCategory,
    pub date: String,
}

#[derive(Debug)]
pub enum FeedbackCategory {
    Positive,
    Constructive,
    Critical,
}

impl FeedbackManager {
    pub fn new() -> Self {
        Self { feedback: Vec::new() }
    }

    pub fn give_feedback(&mut self, from: &str, to: &str, message: &str, category: FeedbackCategory, date: &str) {
        self.feedback.push(Feedback {
            from: from.to_string(),
            to: to.to_string(),
            message: message.to_string(),
            category,
            date: date.to_string(),
        });
    }
}

/// Problem 11: Status reporting
/// Report status
pub struct StatusReporter {
    pub reports: Vec<StatusReport>,
}

pub struct StatusReport {
    pub project: String,
    pub status: ProjectStatus,
    pub progress: f64,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug)]
pub enum ProjectStatus {
    OnTrack,
    AtRisk,
    Delayed,
    Completed,
}

impl StatusReporter {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn report(&mut self, project: &str, status: ProjectStatus, progress: f64, blockers: Vec<String>, next_steps: Vec<String>) {
        self.reports.push(StatusReport {
            project: project.to_string(),
            status,
            progress,
            blockers,
            next_steps,
        });
    }
}

/// Problem 12: Announcement
/// Make announcements
pub struct AnnouncementManager {
    pub announcements: Vec<Announcement>,
}

pub struct Announcement {
    pub title: String,
    pub message: String,
    pub audience: String,
    pub date: String,
}

impl AnnouncementManager {
    pub fn new() -> Self {
        Self { announcements: Vec::new() }
    }

    pub fn announce(&mut self, title: &str, message: &str, audience: &str, date: &str) {
        self.announcements.push(Announcement {
            title: title.to_string(),
            message: message.to_string(),
            audience: audience.to_string(),
            date: date.to_string(),
        });
    }
}

/// Problem 13: Escalation
/// Handle escalations
pub struct EscalationManager {
    pub escalations: Vec<Escalation>,
}

pub struct Escalation {
    pub issue: String,
    pub from: String,
    pub to: String,
    pub urgency: Urgency,
    pub date: String,
}

#[derive(Debug)]
pub enum Urgency {
    Low,
    Medium,
    High,
    Critical,
}

impl EscalationManager {
    pub fn new() -> Self {
        Self { escalations: Vec::new() }
    }

    pub fn escalate(&mut self, issue: &str, from: &str, to: &str, urgency: Urgency, date: &str) {
        self.escalations.push(Escalation {
            issue: issue.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            urgency,
            date: date.to_string(),
        });
    }
}

/// Problem 14: Communication plan
/// Create communication plan
pub struct CommunicationPlan {
    pub stakeholders: Vec<StakeholderComm>,
    pub channels: Vec<String>,
    pub frequency: std::collections::HashMap<String, String>,
}

pub struct StakeholderComm {
    pub name: String,
    pub needs: Vec<String>,
    pub preferred_channel: String,
}

impl CommunicationPlan {
    pub fn new() -> Self {
        Self {
            stakeholders: Vec::new(),
            channels: Vec::new(),
            frequency: std::collections::HashMap::new(),
        }
    }

    pub fn add_stakeholder(&mut self, name: &str, needs: Vec<String>, preferred_channel: &str) {
        self.stakeholders.push(StakeholderComm {
            name: name.to_string(),
            needs,
            preferred_channel: preferred_channel.to_string(),
        });
    }
}

/// Problem 15: Communication metrics
/// Track communication metrics
pub struct CommunicationMetrics {
    pub metrics: std::collections::HashMap<String, f64>,
}

impl CommunicationMetrics {
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
        }
    }

    pub fn set_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stakeholder_communication() {
        let mut communicator = StakeholderCommunicator::new();
        communicator.communicate("CEO", "Project status", "On track", "Email", "2024-01-01");
        assert_eq!(communicator.communications.len(), 1);
    }

    #[test]
    fn test_technical_communication() {
        let mut communicator = TechnicalCommunicator::new();
        communicator.create_document("API Guide", "Developers", "Content", "Markdown");
        assert_eq!(communicator.documents.len(), 1);
    }

    #[test]
    fn test_executive_communication() {
        let mut communicator = ExecutiveCommunicator::new();
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("Revenue".to_string(), "$1M".to_string());
        communicator.create_report("Q1 Report", "Good progress", metrics, vec!["Continue".to_string()]);
        assert_eq!(communicator.reports.len(), 1);
    }

    #[test]
    fn test_crisis_communication() {
        let mut communicator = CrisisCommunicator::new();
        communicator.communicate("Outage", "All", "Service restored", "Slack", "2024-01-01 10:00");
        assert_eq!(communicator.messages.len(), 1);
    }

    #[test]
    fn test_change_communication() {
        let mut communicator = ChangeCommunicator::new();
        communicator.communicate("New tool", "All team", "Team", "Q1");
        assert_eq!(communicator.messages.len(), 1);
    }

    #[test]
    fn test_team_communication() {
        let mut communicator = TeamCommunicator::new();
        communicator.update("Sprint review", "Completed 80%", Priority::Medium, "2024-01-01");
        assert_eq!(communicator.updates.len(), 1);
    }

    #[test]
    fn test_presentation() {
        let mut creator = PresentationCreator::new();
        creator.create("Q1 Review", "Team");
        creator.add_slide("Q1 Review", "Summary", "Good progress");
        assert_eq!(creator.presentations[0].slides.len(), 1);
    }

    #[test]
    fn test_documentation() {
        let mut creator = DocumentationCreator::new();
        creator.create("API Guide", DocType::API, "Content", "Developers");
        assert_eq!(creator.documents.len(), 1);
    }

    #[test]
    fn test_meeting_management() {
        let mut manager = MeetingManager::new();
        manager.schedule("Sprint planning", vec!["Alice".to_string()], vec!["Plan sprint".to_string()]);
        manager.add_outcome("Sprint planning", "Sprint planned");
        assert_eq!(manager.meetings[0].outcomes.len(), 1);
    }

    #[test]
    fn test_feedback() {
        let mut manager = FeedbackManager::new();
        manager.give_feedback("Alice", "Bob", "Great work", FeedbackCategory::Positive, "2024-01-01");
        assert_eq!(manager.feedback.len(), 1);
    }

    #[test]
    fn test_status_reporting() {
        let mut reporter = StatusReporter::new();
        reporter.report("Project A", ProjectStatus::OnTrack, 0.8, vec![], vec!["Continue".to_string()]);
        assert_eq!(reporter.reports.len(), 1);
    }

    #[test]
    fn test_announcement() {
        let mut manager = AnnouncementManager::new();
        manager.announce("New hire", "Welcome Alice", "All", "2024-01-01");
        assert_eq!(manager.announcements.len(), 1);
    }

    #[test]
    fn test_escalation() {
        let mut manager = EscalationManager::new();
        manager.escalate("Critical bug", "Dev", "CTO", Urgency::Critical, "2024-01-01");
        assert_eq!(manager.escalations.len(), 1);
    }

    #[test]
    fn test_communication_plan() {
        let mut plan = CommunicationPlan::new();
        plan.add_stakeholder("CEO", vec!["Status".to_string()], "Email");
        assert_eq!(plan.stakeholders.len(), 1);
    }

    #[test]
    fn test_communication_metrics() {
        let mut metrics = CommunicationMetrics::new();
        metrics.set_metric("emails_sent", 100.0);
        assert_eq!(metrics.get_metric("emails_sent"), Some(100.0));
    }
}
