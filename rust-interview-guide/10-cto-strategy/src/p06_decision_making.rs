/// Problem: Decision Making
///
/// Master decision making in Rust.
///
/// Key Concepts:
/// - Frameworks
/// - Analysis
/// - Trade-offs
/// - Risk assessment
/// - Implementation

/// Problem 1: Decision framework
/// Use decision framework
pub struct DecisionFramework {
    pub criteria: Vec<Criterion>,
    pub options: Vec<DecisionOption>,
}

pub struct Criterion {
    pub name: String,
    pub weight: f64,
}

pub struct DecisionOption {
    pub name: String,
    pub scores: std::collections::HashMap<String, f64>,
}

impl DecisionFramework {
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
            options: Vec::new(),
        }
    }

    pub fn add_criterion(&mut self, name: &str, weight: f64) {
        self.criteria.push(Criterion {
            name: name.to_string(),
            weight,
        });
    }

    pub fn add_option(&mut self, name: &str, scores: std::collections::HashMap<String, f64>) {
        self.options.push(DecisionOption {
            name: name.to_string(),
            scores,
        });
    }

    pub fn decide(&self) -> Option<String> {
        let mut best_score = 0.0;
        let mut best_option = None;

        for option in &self.options {
            let mut total_score = 0.0;
            for criterion in &self.criteria {
                if let Some(score) = option.scores.get(&criterion.name) {
                    total_score += score * criterion.weight;
                }
            }
            if total_score > best_score {
                best_score = total_score;
                best_option = Some(option.name.clone());
            }
        }

        best_option
    }
}

/// Problem 2: Cost-benefit analysis
/// Perform cost-benefit analysis
pub struct CostBenefitAnalysis {
    pub costs: Vec<Cost>,
    pub benefits: Vec<Benefit>,
}

pub struct Cost {
    pub description: String,
    pub amount: f64,
}

pub struct Benefit {
    pub description: String,
    pub amount: f64,
}

impl CostBenefitAnalysis {
    pub fn new() -> Self {
        Self {
            costs: Vec::new(),
            benefits: Vec::new(),
        }
    }

    pub fn add_cost(&mut self, description: &str, amount: f64) {
        self.costs.push(Cost {
            description: description.to_string(),
            amount,
        });
    }

    pub fn add_benefit(&mut self, description: &str, amount: f64) {
        self.benefits.push(Benefit {
            description: description.to_string(),
            amount,
        });
    }

    pub fn net_value(&self) -> f64 {
        let total_costs: f64 = self.costs.iter().map(|c| c.amount).sum();
        let total_benefits: f64 = self.benefits.iter().map(|b| b.amount).sum();
        total_benefits - total_costs
    }
}

/// Problem 3: Risk assessment
/// Assess risks
pub struct RiskAssessment {
    pub risks: Vec<Risk>,
}

pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub mitigation: String,
}

impl RiskAssessment {
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

/// Problem 4: Trade-off analysis
/// Analyze trade-offs
pub struct TradeOffAnalysis {
    pub dimensions: Vec<Dimension>,
}

pub struct Dimension {
    pub name: String,
    pub options: Vec<DimensionOption>,
}

pub struct DimensionOption {
    pub name: String,
    pub value: f64,
}

impl TradeOffAnalysis {
    pub fn new() -> Self {
        Self { dimensions: Vec::new() }
    }

    pub fn add_dimension(&mut self, name: &str) {
        self.dimensions.push(Dimension {
            name: name.to_string(),
            options: Vec::new(),
        });
    }

    pub fn add_option(&mut self, dimension: &str, name: &str, value: f64) {
        if let Some(dim) = self.dimensions.iter_mut().find(|d| d.name == dimension) {
            dim.options.push(DimensionOption {
                name: name.to_string(),
                value,
            });
        }
    }
}

/// Problem 5: Stakeholder analysis
/// Analyze stakeholders
pub struct StakeholderAnalysis {
    pub stakeholders: Vec<Stakeholder>,
}

pub struct Stakeholder {
    pub name: String,
    pub interest: f64,
    pub influence: f64,
    pub attitude: String,
}

impl StakeholderAnalysis {
    pub fn new() -> Self {
        Self { stakeholders: Vec::new() }
    }

    pub fn add_stakeholder(&mut self, name: &str, interest: f64, influence: f64, attitude: &str) {
        self.stakeholders.push(Stakeholder {
            name: name.to_string(),
            interest,
            influence,
            attitude: attitude.to_string(),
        });
    }

    pub fn key_stakeholders(&self) -> Vec<&Stakeholder> {
        self.stakeholders.iter().filter(|s| s.influence > 0.7).collect()
    }
}

/// Problem 6: Impact assessment
/// Assess impact
pub struct ImpactAssessment {
    pub impacts: Vec<Impact>,
}

pub struct Impact {
    pub area: String,
    pub severity: f64,
    pub duration: String,
    pub reversibility: bool,
}

impl ImpactAssessment {
    pub fn new() -> Self {
        Self { impacts: Vec::new() }
    }

    pub fn add_impact(&mut self, area: &str, severity: f64, duration: &str, reversibility: bool) {
        self.impacts.push(Impact {
            area: area.to_string(),
            severity,
            duration: duration.to_string(),
            reversibility,
        });
    }
}

/// Problem 7: Feasibility analysis
/// Analyze feasibility
pub struct FeasibilityAnalysis {
    pub technical: f64,
    pub economic: f64,
    pub operational: f64,
    pub schedule: f64,
}

impl FeasibilityAnalysis {
    pub fn new(technical: f64, economic: f64, operational: f64, schedule: f64) -> Self {
        Self {
            technical,
            economic,
            operational,
            schedule,
        }
    }

    pub fn overall(&self) -> f64 {
        (self.technical + self.economic + self.operational + self.schedule) / 4.0
    }

    pub fn is_feasible(&self) -> bool {
        self.overall() > 0.6
    }
}

/// Problem 8: Prioritization
/// Prioritize options
pub struct Prioritizer {
    pub items: Vec<PrioritizedItem>,
}

pub struct PrioritizedItem {
    pub name: String,
    pub value: f64,
    pub effort: f64,
}

impl Prioritizer {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, name: &str, value: f64, effort: f64) {
        self.items.push(PrioritizedItem {
            name: name.to_string(),
            value,
            effort,
        });
    }

    pub fn prioritize(&self) -> Vec<&PrioritizedItem> {
        let mut items: Vec<&PrioritizedItem> = self.items.iter().collect();
        items.sort_by(|a, b| {
            let ratio_a = a.value / a.effort;
            let ratio_b = b.value / b.effort;
            ratio_b.partial_cmp(&ratio_a).unwrap()
        });
        items
    }
}

/// Problem 9: Scenario analysis
/// Analyze scenarios
pub struct ScenarioAnalysis {
    pub scenarios: Vec<Scenario>,
}

pub struct Scenario {
    pub name: String,
    pub probability: f64,
    pub outcome: String,
}

impl ScenarioAnalysis {
    pub fn new() -> Self {
        Self { scenarios: Vec::new() }
    }

    pub fn add_scenario(&mut self, name: &str, probability: f64, outcome: &str) {
        self.scenarios.push(Scenario {
            name: name.to_string(),
            probability,
            outcome: outcome.to_string(),
        });
    }

    pub fn expected_outcome(&self) -> String {
        self.scenarios
            .iter()
            .max_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap())
            .map(|s| s.outcome.clone())
            .unwrap_or_default()
    }
}

/// Problem 10: Consensus building
/// Build consensus
pub struct ConsensusBuilder {
    pub stakeholders: Vec<String>,
    pub positions: std::collections::HashMap<String, String>,
}

impl ConsensusBuilder {
    pub fn new() -> Self {
        Self {
            stakeholders: Vec::new(),
            positions: std::collections::HashMap::new(),
        }
    }

    pub fn add_stakeholder(&mut self, name: &str) {
        self.stakeholders.push(name.to_string());
    }

    pub fn record_position(&mut self, stakeholder: &str, position: &str) {
        self.positions.insert(stakeholder.to_string(), position.to_string());
    }

    pub fn has_consensus(&self) -> bool {
        let positions: Vec<&String> = self.positions.values().collect();
        positions.windows(2).all(|w| w[0] == w[1])
    }
}

/// Problem 11: Decision criteria
/// Define decision criteria
pub struct DecisionCriteria {
    pub criteria: Vec<String>,
    pub weights: std::collections::HashMap<String, f64>,
}

impl DecisionCriteria {
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
            weights: std::collections::HashMap::new(),
        }
    }

    pub fn add_criterion(&mut self, name: &str, weight: f64) {
        self.criteria.push(name.to_string());
        self.weights.insert(name.to_string(), weight);
    }
}

/// Problem 12: Decision matrix
/// Create decision matrix
pub struct DecisionMatrix {
    pub options: Vec<String>,
    pub criteria: Vec<String>,
    pub scores: std::collections::HashMap<(String, String), f64>,
}

impl DecisionMatrix {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            criteria: Vec::new(),
            scores: std::collections::HashMap::new(),
        }
    }

    pub fn add_option(&mut self, option: &str) {
        self.options.push(option.to_string());
    }

    pub fn add_criterion(&mut self, criterion: &str) {
        self.criteria.push(criterion.to_string());
    }

    pub fn set_score(&mut self, option: &str, criterion: &str, score: f64) {
        self.scores.insert((option.to_string(), criterion.to_string()), score);
    }
}

/// Problem 13: Decision review
/// Review decisions
pub struct DecisionReview {
    pub decisions: Vec<ReviewedDecision>,
}

pub struct ReviewedDecision {
    pub decision: String,
    pub outcome: String,
    pub lessons: Vec<String>,
}

impl DecisionReview {
    pub fn new() -> Self {
        Self { decisions: Vec::new() }
    }

    pub fn review(&mut self, decision: &str, outcome: &str, lessons: Vec<String>) {
        self.decisions.push(ReviewedDecision {
            decision: decision.to_string(),
            outcome: outcome.to_string(),
            lessons,
        });
    }
}

/// Problem 14: Decision communication
/// Communicate decisions
pub struct DecisionCommunicator {
    pub communications: Vec<DecisionCommunication>,
}

pub struct DecisionCommunication {
    pub decision: String,
    pub audience: String,
    pub message: String,
}

impl DecisionCommunicator {
    pub fn new() -> Self {
        Self { communications: Vec::new() }
    }

    pub fn communicate(&mut self, decision: &str, audience: &str, message: &str) {
        self.communications.push(DecisionCommunication {
            decision: decision.to_string(),
            audience: audience.to_string(),
            message: message.to_string(),
        });
    }
}

/// Problem 15: Decision implementation
/// Implement decisions
pub struct DecisionImplementer {
    pub implementations: Vec<DecisionImplementation>,
}

pub struct DecisionImplementation {
    pub decision: String,
    pub actions: Vec<String>,
    pub timeline: String,
    pub owner: String,
}

impl DecisionImplementer {
    pub fn new() -> Self {
        Self { implementations: Vec::new() }
    }

    pub fn implement(&mut self, decision: &str, actions: Vec<String>, timeline: &str, owner: &str) {
        self.implementations.push(DecisionImplementation {
            decision: decision.to_string(),
            actions,
            timeline: timeline.to_string(),
            owner: owner.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_framework() {
        let mut framework = DecisionFramework::new();
        framework.add_criterion("Performance", 0.4);
        framework.add_criterion("Safety", 0.6);
        let mut scores = std::collections::HashMap::new();
        scores.insert("Performance".to_string(), 0.9);
        scores.insert("Safety".to_string(), 0.8);
        framework.add_option("Rust", scores);
        assert!(framework.decide().is_some());
    }

    #[test]
    fn test_cost_benefit() {
        let mut analysis = CostBenefitAnalysis::new();
        analysis.add_cost("Training", 10000.0);
        analysis.add_benefit("Performance", 50000.0);
        assert!(analysis.net_value() > 0.0);
    }

    #[test]
    fn test_risk_assessment() {
        let mut assessment = RiskAssessment::new();
        assessment.add_risk("Learning curve", 0.8, 0.8, "Training");
        assert_eq!(assessment.high_risks().len(), 1);
    }

    #[test]
    fn test_trade_off() {
        let mut analysis = TradeOffAnalysis::new();
        analysis.add_dimension("Performance");
        analysis.add_option("Performance", "Rust", 0.9);
        assert_eq!(analysis.dimensions.len(), 1);
    }

    #[test]
    fn test_stakeholder_analysis() {
        let mut analysis = StakeholderAnalysis::new();
        analysis.add_stakeholder("CEO", 0.9, 0.9, "Supportive");
        assert_eq!(analysis.key_stakeholders().len(), 1);
    }

    #[test]
    fn test_impact_assessment() {
        let mut assessment = ImpactAssessment::new();
        assessment.add_impact("Performance", 0.8, "Long-term", true);
        assert_eq!(assessment.impacts.len(), 1);
    }

    #[test]
    fn test_feasibility() {
        let analysis = FeasibilityAnalysis::new(0.8, 0.7, 0.9, 0.6);
        assert!(analysis.is_feasible());
    }

    #[test]
    fn test_prioritization() {
        let mut prioritizer = Prioritizer::new();
        prioritizer.add_item("Feature A", 10.0, 5.0);
        prioritizer.add_item("Feature B", 8.0, 2.0);
        let prioritized = prioritizer.prioritize();
        assert_eq!(prioritized[0].name, "Feature B");
    }

    #[test]
    fn test_scenario_analysis() {
        let mut analysis = ScenarioAnalysis::new();
        analysis.add_scenario("Best", 0.3, "Success");
        analysis.add_scenario("Worst", 0.7, "Failure");
        assert_eq!(analysis.expected_outcome(), "Failure");
    }

    #[test]
    fn test_consensus() {
        let mut builder = ConsensusBuilder::new();
        builder.add_stakeholder("Alice");
        builder.add_stakeholder("Bob");
        builder.record_position("Alice", "Yes");
        builder.record_position("Bob", "Yes");
        assert!(builder.has_consensus());
    }

    #[test]
    fn test_decision_matrix() {
        let mut matrix = DecisionMatrix::new();
        matrix.add_option("Rust");
        matrix.add_criterion("Performance");
        matrix.set_score("Rust", "Performance", 0.9);
        assert_eq!(matrix.scores.len(), 1);
    }

    #[test]
    fn test_decision_review() {
        let mut review = DecisionReview::new();
        review.review("Use Rust", "Success", vec!["Good choice".to_string()]);
        assert_eq!(review.decisions.len(), 1);
    }

    #[test]
    fn test_decision_communication() {
        let mut communicator = DecisionCommunicator::new();
        communicator.communicate("Use Rust", "Team", "We're adopting Rust");
        assert_eq!(communicator.communications.len(), 1);
    }

    #[test]
    fn test_decision_implementation() {
        let mut implementer = DecisionImplementer::new();
        implementer.implement("Use Rust", vec!["Training".to_string()], "Q1", "CTO");
        assert_eq!(implementer.implementations.len(), 1);
    }
}
