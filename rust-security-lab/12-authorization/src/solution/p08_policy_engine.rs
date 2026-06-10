//! # Lesson 08: Policy Engine (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Always,
    Never,
    SubjectEquals(String),
    ResourceStartsWith(String),
    ActionEquals(String),
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub priority: u32,
    pub condition: Condition,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub subject: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombiningAlgorithm {
    FirstMatch,
    DenyOverrides,
    AllowOverrides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub decision: Effect,
    pub matched_rule: Option<String>,
    pub reason: String,
}

pub struct PolicyEngine {
    rules: Vec<Rule>,
    combining_algorithm: CombiningAlgorithm,
}

impl PolicyEngine {
    pub fn new(algorithm: CombiningAlgorithm) -> Self {
        Self {
            rules: Vec::new(),
            combining_algorithm: algorithm,
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
    }

    pub fn evaluate(&self, request: &Request) -> EvaluationResult {
        match self.combining_algorithm {
            CombiningAlgorithm::FirstMatch => self.evaluate_first_match(request),
            CombiningAlgorithm::DenyOverrides => self.evaluate_deny_overrides(request),
            CombiningAlgorithm::AllowOverrides => self.evaluate_allow_overrides(request),
        }
    }

    fn evaluate_first_match(&self, request: &Request) -> EvaluationResult {
        for rule in &self.rules {
            if Self::evaluate_condition(&rule.condition, request) {
                return EvaluationResult {
                    decision: rule.effect.clone(),
                    matched_rule: Some(rule.name.clone()),
                    reason: format!("Matched rule '{}'", rule.name),
                };
            }
        }

        EvaluationResult {
            decision: Effect::Deny,
            matched_rule: None,
            reason: "No rules matched (default deny)".to_string(),
        }
    }

    fn evaluate_deny_overrides(&self, request: &Request) -> EvaluationResult {
        let mut allow_result: Option<EvaluationResult> = None;

        for rule in &self.rules {
            if Self::evaluate_condition(&rule.condition, request) {
                match rule.effect {
                    Effect::Deny => {
                        return EvaluationResult {
                            decision: Effect::Deny,
                            matched_rule: Some(rule.name.clone()),
                            reason: format!("Deny override from rule '{}'", rule.name),
                        };
                    }
                    Effect::Allow => {
                        if allow_result.is_none() {
                            allow_result = Some(EvaluationResult {
                                decision: Effect::Allow,
                                matched_rule: Some(rule.name.clone()),
                                reason: format!("Allowed by rule '{}'", rule.name),
                            });
                        }
                    }
                }
            }
        }

        allow_result.unwrap_or(EvaluationResult {
            decision: Effect::Deny,
            matched_rule: None,
            reason: "No rules matched (default deny)".to_string(),
        })
    }

    fn evaluate_allow_overrides(&self, request: &Request) -> EvaluationResult {
        let mut deny_result: Option<EvaluationResult> = None;

        for rule in &self.rules {
            if Self::evaluate_condition(&rule.condition, request) {
                match rule.effect {
                    Effect::Allow => {
                        return EvaluationResult {
                            decision: Effect::Allow,
                            matched_rule: Some(rule.name.clone()),
                            reason: format!("Allow override from rule '{}'", rule.name),
                        };
                    }
                    Effect::Deny => {
                        if deny_result.is_none() {
                            deny_result = Some(EvaluationResult {
                                decision: Effect::Deny,
                                matched_rule: Some(rule.name.clone()),
                                reason: format!("Denied by rule '{}'", rule.name),
                            });
                        }
                    }
                }
            }
        }

        deny_result.unwrap_or(EvaluationResult {
            decision: Effect::Deny,
            matched_rule: None,
            reason: "No rules matched (default deny)".to_string(),
        })
    }

    pub fn evaluate_condition(condition: &Condition, request: &Request) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Never => false,
            Condition::SubjectEquals(s) => request.subject == *s,
            Condition::ResourceStartsWith(prefix) => request.resource.starts_with(prefix),
            Condition::ActionEquals(a) => request.action == *a,
            Condition::All(conditions) => {
                conditions.iter().all(|c| Self::evaluate_condition(c, request))
            }
            Condition::Any(conditions) => {
                conditions.iter().any(|c| Self::evaluate_condition(c, request))
            }
            Condition::Not(inner) => !Self::evaluate_condition(inner, request),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_first_match() -> PolicyEngine {
        let mut engine = PolicyEngine::new(CombiningAlgorithm::FirstMatch);

        engine.add_rule(Rule {
            name: "admin-full-access".to_string(),
            priority: 10,
            condition: Condition::SubjectEquals("admin".to_string()),
            effect: Effect::Allow,
        });

        engine.add_rule(Rule {
            name: "public-read".to_string(),
            priority: 20,
            condition: Condition::All(vec![
                Condition::ResourceStartsWith("/public/".to_string()),
                Condition::ActionEquals("read".to_string()),
            ]),
            effect: Effect::Allow,
        });

        engine.add_rule(Rule {
            name: "deny-delete".to_string(),
            priority: 5,
            condition: Condition::ActionEquals("delete".to_string()),
            effect: Effect::Deny,
        });

        engine
    }

    #[test]
    fn test_admin_allowed() {
        let engine = engine_first_match();
        let result = engine.evaluate(&Request {
            subject: "admin".to_string(),
            resource: "/secret/data".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Allow);
    }

    #[test]
    fn test_public_read_allowed() {
        let engine = engine_first_match();
        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/public/announcement".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Allow);
    }

    #[test]
    fn test_non_public_read_denied() {
        let engine = engine_first_match();
        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/private/data".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Deny);
    }

    #[test]
    fn test_deny_overrides_combining() {
        let mut engine = PolicyEngine::new(CombiningAlgorithm::DenyOverrides);

        engine.add_rule(Rule {
            name: "allow-read".to_string(),
            priority: 10,
            condition: Condition::ActionEquals("read".to_string()),
            effect: Effect::Allow,
        });

        engine.add_rule(Rule {
            name: "deny-secret".to_string(),
            priority: 20,
            condition: Condition::ResourceStartsWith("/secret/".to_string()),
            effect: Effect::Deny,
        });

        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/secret/data".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Deny);
    }

    #[test]
    fn test_allow_overrides_combining() {
        let mut engine = PolicyEngine::new(CombiningAlgorithm::AllowOverrides);

        engine.add_rule(Rule {
            name: "deny-all".to_string(),
            priority: 100,
            condition: Condition::Always,
            effect: Effect::Deny,
        });

        engine.add_rule(Rule {
            name: "allow-read".to_string(),
            priority: 10,
            condition: Condition::ActionEquals("read".to_string()),
            effect: Effect::Allow,
        });

        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/anything".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Allow);
    }

    #[test]
    fn test_default_deny() {
        let engine = PolicyEngine::new(CombiningAlgorithm::FirstMatch);
        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/anything".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Deny);
    }

    #[test]
    fn test_condition_not() {
        let engine = engine_first_match();
        let result = engine.evaluate(&Request {
            subject: "bob".to_string(),
            resource: "/private/data".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.decision, Effect::Deny);
    }

    #[test]
    fn test_matched_rule_name() {
        let engine = engine_first_match();
        let result = engine.evaluate(&Request {
            subject: "admin".to_string(),
            resource: "/anything".to_string(),
            action: "read".to_string(),
        });
        assert_eq!(result.matched_rule, Some("admin-full-access".to_string()));
    }
}
