/// Problem: Innovation
///
/// Master innovation in Rust.
///
/// Key Concepts:
/// - Innovation culture
/// - Experimentation
/// - R&D management
/// - Technology scouting
/// - Innovation metrics

/// Problem 1: Innovation culture
/// Build innovation culture
pub struct InnovationCulture {
    pub values: Vec<String>,
    pub practices: Vec<String>,
    pub incentives: Vec<String>,
}

impl InnovationCulture {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            practices: Vec::new(),
            incentives: Vec::new(),
        }
    }

    pub fn add_value(&mut self, value: &str) {
        self.values.push(value.to_string());
    }

    pub fn add_practice(&mut self, practice: &str) {
        self.practices.push(practice.to_string());
    }
}

/// Problem 2: Experimentation
/// Run experiments
pub struct ExperimentRunner {
    pub experiments: Vec<Experiment>,
}

pub struct Experiment {
    pub name: String,
    pub hypothesis: String,
    pub method: String,
    pub result: Option<String>,
}

impl ExperimentRunner {
    pub fn new() -> Self {
        Self { experiments: Vec::new() }
    }

    pub fn start_experiment(&mut self, name: &str, hypothesis: &str, method: &str) {
        self.experiments.push(Experiment {
            name: name.to_string(),
            hypothesis: hypothesis.to_string(),
            method: method.to_string(),
            result: None,
        });
    }

    pub fn record_result(&mut self, name: &str, result: &str) {
        if let Some(experiment) = self.experiments.iter_mut().find(|e| e.name == name) {
            experiment.result = Some(result.to_string());
        }
    }
}

/// Problem 3: R&D management
/// Manage R&D
pub struct RnDManager {
    pub projects: Vec<RnDProject>,
    pub budget: f64,
}

pub struct RnDProject {
    pub name: String,
    pub status: ProjectStatus,
    pub budget: f64,
    pub timeline: String,
}

#[derive(Debug)]
pub enum ProjectStatus {
    Planning,
    Active,
    Completed,
    Cancelled,
}

impl RnDManager {
    pub fn new(budget: f64) -> Self {
        Self {
            projects: Vec::new(),
            budget,
        }
    }

    pub fn add_project(&mut self, name: &str, budget: f64, timeline: &str) {
        self.projects.push(RnDProject {
            name: name.to_string(),
            status: ProjectStatus::Planning,
            budget,
            timeline: timeline.to_string(),
        });
    }

    pub fn start_project(&mut self, name: &str) {
        if let Some(project) = self.projects.iter_mut().find(|p| p.name == name) {
            project.status = ProjectStatus::Active;
        }
    }
}

/// Problem 4: Technology scouting
/// Scout new technologies
pub struct TechnologyScout {
    pub technologies: Vec<Technology>,
}

pub struct Technology {
    pub name: String,
    pub maturity: TechMaturity,
    pub relevance: f64,
    pub recommendation: String,
}

#[derive(Debug)]
pub enum TechMaturity {
    Emerging,
    Growing,
    Mature,
    Declining,
}

impl TechnologyScout {
    pub fn new() -> Self {
        Self { technologies: Vec::new() }
    }

    pub fn evaluate(&mut self, name: &str, maturity: TechMaturity, relevance: f64, recommendation: &str) {
        self.technologies.push(Technology {
            name: name.to_string(),
            maturity,
            relevance,
            recommendation: recommendation.to_string(),
        });
    }

    pub fn high_relevance(&self) -> Vec<&Technology> {
        self.technologies.iter().filter(|t| t.relevance > 0.7).collect()
    }
}

/// Problem 5: Innovation metrics
/// Track innovation metrics
pub struct InnovationMetrics {
    pub metrics: std::collections::HashMap<String, f64>,
}

impl InnovationMetrics {
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

/// Problem 6: Idea management
/// Manage ideas
pub struct IdeaManager {
    pub ideas: Vec<Idea>,
}

pub struct Idea {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub status: IdeaStatus,
    pub score: f64,
}

#[derive(Debug)]
pub enum IdeaStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Implemented,
}

impl IdeaManager {
    pub fn new() -> Self {
        Self { ideas: Vec::new() }
    }

    pub fn submit_idea(&mut self, title: &str, description: &str) {
        let id = self.ideas.len() as u32 + 1;
        self.ideas.push(Idea {
            id,
            title: title.to_string(),
            description: description.to_string(),
            status: IdeaStatus::Submitted,
            score: 0.0,
        });
    }

    pub fn review_idea(&mut self, id: u32, score: f64) {
        if let Some(idea) = self.ideas.iter_mut().find(|i| i.id == id) {
            idea.score = score;
            idea.status = IdeaStatus::UnderReview;
        }
    }
}

/// Problem 7: Innovation portfolio
/// Manage innovation portfolio
pub struct InnovationPortfolio {
    pub projects: Vec<PortfolioProject>,
}

pub struct PortfolioProject {
    pub name: String,
    pub category: InnovationCategory,
    pub risk: f64,
    pub potential: f64,
}

#[derive(Debug)]
pub enum InnovationCategory {
    Incremental,
    Architectural,
    Disruptive,
}

impl InnovationPortfolio {
    pub fn new() -> Self {
        Self { projects: Vec::new() }
    }

    pub fn add_project(&mut self, name: &str, category: InnovationCategory, risk: f64, potential: f64) {
        self.projects.push(PortfolioProject {
            name: name.to_string(),
            category,
            risk,
            potential,
        });
    }

    pub fn high_potential(&self) -> Vec<&PortfolioProject> {
        self.projects.iter().filter(|p| p.potential > 0.7).collect()
    }
}

/// Problem 8: Innovation partnerships
/// Manage partnerships
pub struct InnovationPartnership {
    pub partners: Vec<Partner>,
}

pub struct Partner {
    pub name: String,
    pub type_: PartnerType,
    pub focus: String,
}

#[derive(Debug)]
pub enum PartnerType {
    Academic,
    Startup,
    Corporate,
    Government,
}

impl InnovationPartnership {
    pub fn new() -> Self {
        Self { partners: Vec::new() }
    }

    pub fn add_partner(&mut self, name: &str, type_: PartnerType, focus: &str) {
        self.partners.push(Partner {
            name: name.to_string(),
            type_,
            focus: focus.to_string(),
        });
    }
}

/// Problem 9: Innovation events
/// Manage innovation events
pub struct InnovationEvents {
    pub events: Vec<InnovationEvent>,
}

pub struct InnovationEvent {
    pub name: String,
    pub type_: EventType,
    pub participants: u32,
    pub outcomes: Vec<String>,
}

#[derive(Debug)]
pub enum EventType {
    Hackathon,
    Workshop,
    Conference,
    Demo,
}

impl InnovationEvents {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn organize(&mut self, name: &str, type_: EventType, participants: u32) {
        self.events.push(InnovationEvent {
            name: name.to_string(),
            type_,
            participants,
            outcomes: Vec::new(),
        });
    }

    pub fn add_outcome(&mut self, event_name: &str, outcome: &str) {
        if let Some(event) = self.events.iter_mut().find(|e| e.name == event_name) {
            event.outcomes.push(outcome.to_string());
        }
    }
}

/// Problem 10: Innovation funding
/// Manage innovation funding
pub struct InnovationFunding {
    pub sources: Vec<FundingSource>,
    pub total: f64,
}

pub struct FundingSource {
    pub name: String,
    pub amount: f64,
    pub type_: FundingType,
}

#[derive(Debug)]
pub enum FundingType {
    Internal,
    Grant,
    Investment,
    Partnership,
}

impl InnovationFunding {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            total: 0.0,
        }
    }

    pub fn add_source(&mut self, name: &str, amount: f64, type_: FundingType) {
        self.sources.push(FundingSource {
            name: name.to_string(),
            amount,
            type_,
        });
        self.total += amount;
    }
}

/// Problem 11: Innovation roadmap
/// Create innovation roadmap
pub struct InnovationRoadmap {
    pub milestones: Vec<InnovationMilestone>,
}

pub struct InnovationMilestone {
    pub name: String,
    pub date: String,
    pub deliverables: Vec<String>,
}

impl InnovationRoadmap {
    pub fn new() -> Self {
        Self { milestones: Vec::new() }
    }

    pub fn add_milestone(&mut self, name: &str, date: &str, deliverables: Vec<String>) {
        self.milestones.push(InnovationMilestone {
            name: name.to_string(),
            date: date.to_string(),
            deliverables,
        });
    }
}

/// Problem 12: Innovation assessment
/// Assess innovation capability
pub struct InnovationAssessment {
    pub dimensions: Vec<AssessmentDimension>,
}

pub struct AssessmentDimension {
    pub name: String,
    pub score: f64,
    pub recommendations: Vec<String>,
}

impl InnovationAssessment {
    pub fn new() -> Self {
        Self { dimensions: Vec::new() }
    }

    pub fn assess(&mut self, name: &str, score: f64, recommendations: Vec<String>) {
        self.dimensions.push(AssessmentDimension {
            name: name.to_string(),
            score,
            recommendations,
        });
    }

    pub fn overall_score(&self) -> f64 {
        if self.dimensions.is_empty() {
            return 0.0;
        }
        self.dimensions.iter().map(|d| d.score).sum::<f64>() / self.dimensions.len() as f64
    }
}

/// Problem 13: Innovation communication
/// Communicate innovation
pub struct InnovationCommunicator {
    pub communications: Vec<InnovationMessage>,
}

pub struct InnovationMessage {
    pub topic: String,
    pub audience: String,
    pub channel: String,
    pub message: String,
}

impl InnovationCommunicator {
    pub fn new() -> Self {
        Self { communications: Vec::new() }
    }

    pub fn communicate(&mut self, topic: &str, audience: &str, channel: &str, message: &str) {
        self.communications.push(InnovationMessage {
            topic: topic.to_string(),
            audience: audience.to_string(),
            channel: channel.to_string(),
            message: message.to_string(),
        });
    }
}

/// Problem 14: Innovation governance
/// Govern innovation
pub struct InnovationGovernance {
    pub policies: Vec<String>,
    pub committees: Vec<String>,
    pub processes: Vec<String>,
}

impl InnovationGovernance {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            committees: Vec::new(),
            processes: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: &str) {
        self.policies.push(policy.to_string());
    }
}

/// Problem 15: Innovation strategy
/// Define innovation strategy
pub struct InnovationStrategy {
    pub vision: String,
    pub goals: Vec<String>,
    pub focus_areas: Vec<String>,
    pub investment_areas: Vec<String>,
}

impl InnovationStrategy {
    pub fn new(vision: &str) -> Self {
        Self {
            vision: vision.to_string(),
            goals: Vec::new(),
            focus_areas: Vec::new(),
            investment_areas: Vec::new(),
        }
    }

    pub fn add_goal(&mut self, goal: &str) {
        self.goals.push(goal.to_string());
    }

    pub fn add_focus_area(&mut self, area: &str) {
        self.focus_areas.push(area.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_innovation_culture() {
        let mut culture = InnovationCulture::new();
        culture.add_value("Experimentation");
        assert_eq!(culture.values.len(), 1);
    }

    #[test]
    fn test_experimentation() {
        let mut runner = ExperimentRunner::new();
        runner.start_experiment("Test", "Hypothesis", "Method");
        runner.record_result("Test", "Success");
        assert!(runner.experiments[0].result.is_some());
    }

    #[test]
    fn test_rnd_management() {
        let mut manager = RnDManager::new(100000.0);
        manager.add_project("Project A", 50000.0, "6 months");
        manager.start_project("Project A");
    }

    #[test]
    fn test_technology_scouting() {
        let mut scout = TechnologyScout::new();
        scout.evaluate("Rust", TechMaturity::Growing, 0.9, "Adopt");
        assert_eq!(scout.high_relevance().len(), 1);
    }

    #[test]
    fn test_innovation_metrics() {
        let mut metrics = InnovationMetrics::new();
        metrics.set_metric("ideas_submitted", 100.0);
        assert_eq!(metrics.get_metric("ideas_submitted"), Some(100.0));
    }

    #[test]
    fn test_idea_management() {
        let mut manager = IdeaManager::new();
        manager.submit_idea("Idea 1", "Description");
        manager.review_idea(1, 8.5);
        assert_eq!(manager.ideas.len(), 1);
    }

    #[test]
    fn test_innovation_portfolio() {
        let mut portfolio = InnovationPortfolio::new();
        portfolio.add_project("Project A", InnovationCategory::Disruptive, 0.8, 0.9);
        assert_eq!(portfolio.high_potential().len(), 1);
    }

    #[test]
    fn test_innovation_partnerships() {
        let mut partnership = InnovationPartnership::new();
        partnership.add_partner("University", PartnerType::Academic, "AI");
        assert_eq!(partnership.partners.len(), 1);
    }

    #[test]
    fn test_innovation_events() {
        let mut events = InnovationEvents::new();
        events.organize("Hackathon", EventType::Hackathon, 50);
        events.add_outcome("Hackathon", "3 prototypes");
        assert_eq!(events.events.len(), 1);
    }

    #[test]
    fn test_innovation_funding() {
        let mut funding = InnovationFunding::new();
        funding.add_source("Internal", 100000.0, FundingType::Internal);
        assert_eq!(funding.total, 100000.0);
    }

    #[test]
    fn test_innovation_roadmap() {
        let mut roadmap = InnovationRoadmap::new();
        roadmap.add_milestone("Phase 1", "Q1", vec!["Prototype".to_string()]);
        assert_eq!(roadmap.milestones.len(), 1);
    }

    #[test]
    fn test_innovation_assessment() {
        let mut assessment = InnovationAssessment::new();
        assessment.assess("Culture", 0.8, vec!["More training".to_string()]);
        assert!(assessment.overall_score() > 0.0);
    }

    #[test]
    fn test_innovation_communication() {
        let mut communicator = InnovationCommunicator::new();
        communicator.communicate("New tech", "Team", "Email", "Check out Rust");
        assert_eq!(communicator.communications.len(), 1);
    }

    #[test]
    fn test_innovation_governance() {
        let mut governance = InnovationGovernance::new();
        governance.add_policy("Innovation policy");
        assert_eq!(governance.policies.len(), 1);
    }

    #[test]
    fn test_innovation_strategy() {
        let mut strategy = InnovationStrategy::new("Lead in Rust");
        strategy.add_goal("100% Rust adoption");
        assert_eq!(strategy.goals.len(), 1);
    }
}
