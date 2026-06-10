//! # Lesson 03: DREAD Scoring (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreadScore {
    pub threat_name: String,
    pub damage: u8,
    pub reproducibility: u8,
    pub exploitability: u8,
    pub affected_users: u8,
    pub discoverability: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredThreat {
    pub threat: DreadScore,
    pub score: f64,
    pub priority: Priority,
}

impl DreadScore {
    pub fn new(
        threat_name: &str,
        damage: u8,
        reproducibility: u8,
        exploitability: u8,
        affected_users: u8,
        discoverability: u8,
    ) -> Result<Self, String> {
        let values = [damage, reproducibility, exploitability, affected_users, discoverability];
        for (i, &v) in values.iter().enumerate() {
            if v < 1 || v > 10 {
                let names = ["damage", "reproducibility", "exploitability", "affected_users", "discoverability"];
                return Err(format!("{} must be 1-10, got {}", names[i], v));
            }
        }
        Ok(Self {
            threat_name: threat_name.to_string(),
            damage,
            reproducibility,
            exploitability,
            affected_users,
            discoverability,
        })
    }

    pub fn compute_score(&self) -> f64 {
        (self.damage as f64
            + self.reproducibility as f64
            + self.exploitability as f64
            + self.affected_users as f64
            + self.discoverability as f64)
            / 5.0
    }

    pub fn priority(&self) -> Priority {
        let score = self.compute_score();
        if score >= 8.0 {
            Priority::Critical
        } else if score >= 6.0 {
            Priority::High
        } else if score >= 4.0 {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    pub fn score_threat(&self) -> ScoredThreat {
        ScoredThreat {
            threat: self.clone(),
            score: self.compute_score(),
            priority: self.priority(),
        }
    }
}

pub fn rank_threats(threats: &[DreadScore]) -> Vec<ScoredThreat> {
    let mut scored: Vec<ScoredThreat> = threats.iter().map(|t| t.score_threat()).collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    scored
}

pub fn high_priority_threats(threats: &[DreadScore]) -> Vec<ScoredThreat> {
    threats
        .iter()
        .map(|t| t.score_threat())
        .filter(|st| st.priority == Priority::Critical || st.priority == Priority::High)
        .collect()
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
