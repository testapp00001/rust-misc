//! # Lesson 08: Policy Engine
//!
//! ## What is a Policy Engine?
//!
//! A policy engine evaluates access requests against a set of rules and produces
//! allow/deny decisions. It centralizes authorization logic, making it:
//! - **Auditable**: All rules in one place
//! - **Testable**: Rules can be unit tested
//! - **Dynamic**: Rules can be updated without code changes
//!
//! ## Rule Structure
//!
//! Each rule has:
//! - A **name** for debugging and auditing
//! - A **priority** (lower = evaluated first)
//! - A **condition** that determines if the rule applies
//! - An **effect** (Allow or Deny)
//!
//! ## Combining Algorithm
//!
//! When multiple rules match:
//! 1. **First-match**: Use the first matching rule (by priority)
//! 2. **Deny-overrides**: Any Deny wins over all Allows
//! 3. **Allow-overrides**: Any Allow wins over all Denies
//!
//! We implement first-match with explicit priority ordering.
//!
//! ## 🔴 Attack: Policy Injection
//!
//! If policy rules are loaded from user-controllable sources (e.g., database,
//! config file writable by the app), an attacker who compromises that source
//! can inject "ALLOW everything" rules.

/// The effect of a policy rule.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Allow,
    Deny,
}

/// Conditions that can be evaluated.
#[derive(Debug, Clone)]
pub enum Condition {
    /// Always matches.
    Always,
    /// Never matches.
    Never,
    /// Matches if the subject equals the given string.
    SubjectEquals(String),
    /// Matches if the resource starts with the given prefix.
    ResourceStartsWith(String),
    /// Matches if the action equals the given string.
    ActionEquals(String),
    /// Matches if all sub-conditions match.
    All(Vec<Condition>),
    /// Matches if any sub-condition matches.
    Any(Vec<Condition>),
    /// Negates a condition.
    Not(Box<Condition>),
}

/// A policy rule with name, priority, condition, and effect.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    /// Lower priority numbers are evaluated first.
    pub priority: u32,
    pub condition: Condition,
    pub effect: Effect,
}

/// An access request to be evaluated by the policy engine.
#[derive(Debug, Clone)]
pub struct Request {
    pub subject: String,
    pub resource: String,
    pub action: String,
}

/// The policy engine that evaluates requests against rules.
pub struct PolicyEngine {
    rules: Vec<Rule>,
    /// The combining algorithm to use.
    combining_algorithm: CombiningAlgorithm,
}

/// How to combine results when multiple rules match.
#[derive(Debug, Clone, PartialEq)]
pub enum CombiningAlgorithm {
    /// Use the first matching rule (by priority order).
    FirstMatch,
    /// Any Deny result causes a final Deny.
    DenyOverrides,
    /// Any Allow result causes a final Allow.
    AllowOverrides,
}

/// Result of a policy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub decision: Effect,
    pub matched_rule: Option<String>,
    pub reason: String,
}

impl PolicyEngine {
    /// Create a new policy engine with the given combining algorithm.
    pub fn new(algorithm: CombiningAlgorithm) -> Self {
        todo!("Create a PolicyEngine with empty rules")
    }

    /// Add a rule to the engine.
    ///
    /// After adding, rules should be sorted by priority (lower first).
    pub fn add_rule(&mut self, rule: Rule) {
        todo!("Add the rule and sort by priority")
    }

    /// Evaluate a request against all rules.
    ///
    /// Returns an EvaluationResult with the decision, which rule matched, and a reason.
    ///
    /// If no rules match, the default decision is Deny (default-deny).
    pub fn evaluate(&self, request: &Request) -> EvaluationResult {
        todo!("Evaluate rules according to the combining algorithm")
    }

    /// Evaluate a single condition against a request.
    pub fn evaluate_condition(condition: &Condition, request: &Request) -> bool {
        todo!("Recursively evaluate the condition tree")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_first_match() -> PolicyEngine {
        let mut engine = PolicyEngine::new(CombiningAlgorithm::FirstMatch);

        // Admins can do anything
        engine.add_rule(Rule {
            name: "admin-full-access".to_string(),
            priority: 10,
            condition: Condition::SubjectEquals("admin".to_string()),
            effect: Effect::Allow,
        });

        // Users can read public resources
        engine.add_rule(Rule {
            name: "public-read".to_string(),
            priority: 20,
            condition: Condition::All(vec![
                Condition::ResourceStartsWith("/public/".to_string()),
                Condition::ActionEquals("read".to_string()),
            ]),
            effect: Effect::Allow,
        });

        // Deny delete for everyone except admin (already handled above)
        engine.add_rule(Rule {
            name: "deny-delete".to_string(),
            priority: 5,
            condition: Condition::ActionEquals("delete".to_string()),
            effect: Effect::Deny,
        });

        // Default deny is implicit
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

        // Even though read is allowed, /secret/ deny overrides
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
        // Non-admin trying to read non-public → deny (default)
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
