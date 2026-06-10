//! # Lesson 03: DREAD Scoring
//!
//! ## The Problem
//!
//! After identifying threats (e.g., via STRIDE), you need to prioritize them. Not all threats
//! are equally dangerous — some are easy to exploit with massive impact, while others are
//! theoretical and unlikely. Without a scoring system, teams either fix everything (too slow)
//! or fix the wrong things (risky).
//!
//! ## The Solution: DREAD
//!
//! DREAD scores threats on five dimensions, each from 1 to 10:
//!
//! ```text
//! D — Damage             How severe is the impact if exploited?       (1=nuisance, 10=total destruction)
//! R — Reproducibility    How reliably can the attack be repeated?      (1=once, 10=always)
//! E — Exploitability     How much skill/effort/resources are needed?   (1=expert+0day, 10=script kiddie)
//! A — Affected Users     How many users are impacted?                  (1=one user, 10=all users)
//! D — Discoverability    How easy is it to find the vulnerability?     (1=hidden, 10=obvious)
//!
//! Risk Score = (Damage + Reproducibility + Exploitability + Affected Users + Discoverability) / 5
//! ```
//!
//! Scores map to priorities:
//! - 8.0-10.0: Critical — fix immediately
//! - 6.0-7.9:  High — fix in current sprint
//! - 4.0-5.9:  Medium — fix in next release
//! - 1.0-3.9:  Low — accept risk or fix later
//!
//! ## Attack Example: Prioritization Failure
//!
//! A team finds 50 vulnerabilities and tries to fix all at once. They ship nothing for 3 months.
//! Meanwhile, a trivially exploitable XSS (DREAD: 8.2) sits unfixed while they work on a
//! theoretical timing attack (DREAD: 2.1).
//!
//! Defense: Score with DREAD, fix highest-risk items first.

use serde::{Deserialize, Serialize};

/// A single DREAD score for a threat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreadScore {
    /// Name of the threat
    pub threat_name: String,
    /// Damage rating (1-10)
    pub damage: u8,
    /// Reproducibility rating (1-10)
    pub reproducibility: u8,
    /// Exploitability rating (1-10)
    pub exploitability: u8,
    /// Affected users rating (1-10)
    pub affected_users: u8,
    /// Discoverability rating (1-10)
    pub discoverability: u8,
}

/// Priority level derived from the DREAD score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Score 8.0-10.0: fix immediately
    Critical,
    /// Score 6.0-7.9: fix in current sprint
    High,
    /// Score 4.0-5.9: fix in next release
    Medium,
    /// Score 1.0-3.9: accept risk or fix later
    Low,
}

/// A scored threat with its DREAD score and computed priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredThreat {
    pub threat: DreadScore,
    pub score: f64,
    pub priority: Priority,
}

impl DreadScore {
    /// Create a new DREAD score. All values must be 1-10 inclusive.
    /// Returns Err if any value is out of range.
    pub fn new(
        threat_name: &str,
        damage: u8,
        reproducibility: u8,
        exploitability: u8,
        affected_users: u8,
        discoverability: u8,
    ) -> Result<Self, String> {
        todo!("Validate all values are 1-10, then construct")
    }

    /// Compute the average DREAD score (sum / 5.0).
    pub fn compute_score(&self) -> f64 {
        todo!("Sum all five dimensions and divide by 5.0")
    }

    /// Derive the priority level from the computed score.
    pub fn priority(&self) -> Priority {
        todo!("Map score ranges to Priority enum")
    }

    /// Score and classify this threat in one step.
    pub fn score_threat(&self) -> ScoredThreat {
        todo!("Compute score and priority, return ScoredThreat")
    }
}

/// Score a list of threats and return them sorted by score descending (highest risk first).
pub fn rank_threats(threats: &[DreadScore]) -> Vec<ScoredThreat> {
    todo!("Score each threat, then sort by score descending")
}

/// Filter scored threats to only those with Critical or High priority.
pub fn high_priority_threats(threats: &[DreadScore]) -> Vec<ScoredThreat> {
    todo!("Score and filter to Critical + High only")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dread_score_valid() {
        let score = DreadScore::new("XSS", 7, 9, 8, 6, 9);
        assert!(score.is_ok());
    }

    #[test]
    fn test_dread_score_invalid_zero() {
        let score = DreadScore::new("XSS", 0, 9, 8, 6, 9);
        assert!(score.is_err());
    }

    #[test]
    fn test_dread_score_invalid_eleven() {
        let score = DreadScore::new("XSS", 7, 11, 8, 6, 9);
        assert!(score.is_err());
    }

    #[test]
    fn test_compute_score() {
        let score = DreadScore::new("test", 10, 10, 10, 10, 10).unwrap();
        assert!((score.compute_score() - 10.0).abs() < 0.01);

        let score2 = DreadScore::new("test", 1, 1, 1, 1, 1).unwrap();
        assert!((score2.compute_score() - 1.0).abs() < 0.01);

        let score3 = DreadScore::new("test", 5, 6, 7, 8, 9).unwrap();
        assert!((score3.compute_score() - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_priority_critical() {
        let score = DreadScore::new("test", 10, 9, 9, 9, 9).unwrap();
        assert_eq!(score.priority(), Priority::Critical);
    }

    #[test]
    fn test_priority_high() {
        let score = DreadScore::new("test", 7, 7, 7, 7, 7).unwrap();
        assert_eq!(score.priority(), Priority::High);
    }

    #[test]
    fn test_priority_medium() {
        let score = DreadScore::new("test", 5, 5, 5, 5, 5).unwrap();
        assert_eq!(score.priority(), Priority::Medium);
    }

    #[test]
    fn test_priority_low() {
        let score = DreadScore::new("test", 2, 2, 2, 2, 2).unwrap();
        assert_eq!(score.priority(), Priority::Low);
    }

    #[test]
    fn test_rank_threats_order() {
        let threats = vec![
            DreadScore::new("Low threat", 2, 2, 2, 2, 2).unwrap(),
            DreadScore::new("Critical threat", 10, 10, 10, 10, 10).unwrap(),
            DreadScore::new("Medium threat", 5, 5, 5, 5, 5).unwrap(),
        ];
        let ranked = rank_threats(&threats);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].threat.threat_name, "Critical threat");
        assert_eq!(ranked[1].threat.threat_name, "Medium threat");
        assert_eq!(ranked[2].threat.threat_name, "Low threat");
    }

    #[test]
    fn test_high_priority_threats() {
        let threats = vec![
            DreadScore::new("Low", 1, 1, 1, 1, 1).unwrap(),
            DreadScore::new("Medium", 5, 5, 5, 5, 5).unwrap(),
            DreadScore::new("High", 7, 7, 7, 7, 7).unwrap(),
            DreadScore::new("Critical", 9, 9, 9, 9, 9).unwrap(),
        ];
        let high = high_priority_threats(&threats);
        assert_eq!(high.len(), 2);
        assert!(high.iter().all(|t| t.priority == Priority::Critical || t.priority == Priority::High));
    }
}
