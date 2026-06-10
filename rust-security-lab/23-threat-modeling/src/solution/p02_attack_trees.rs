//! # Lesson 02: Attack Trees (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackNode {
    pub id: String,
    pub description: String,
    pub cost: u32,
    pub gate: GateType,
    pub children: Vec<AttackNode>,
}

impl AttackNode {
    pub fn leaf(id: &str, description: &str, cost: u32) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            cost,
            gate: GateType::Or,
            children: Vec::new(),
        }
    }

    pub fn node(id: &str, description: &str, gate: GateType, children: Vec<AttackNode>) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            cost: 0,
            gate,
            children,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn min_cost(&self) -> u32 {
        if self.is_leaf() {
            return self.cost;
        }
        match self.gate {
            GateType::Or => self
                .children
                .iter()
                .map(|c| c.min_cost())
                .min()
                .unwrap_or(u32::MAX),
            GateType::And => self.children.iter().map(|c| c.min_cost()).sum(),
        }
    }

    pub fn weakest_path(&self) -> Vec<String> {
        if self.is_leaf() {
            return vec![self.id.clone()];
        }
        match self.gate {
            GateType::Or => {
                let best_child = self
                    .children
                    .iter()
                    .min_by_key(|c| c.min_cost())
                    .unwrap();
                let mut path = vec![self.id.clone()];
                path.extend(best_child.weakest_path());
                path
            }
            GateType::And => {
                let mut path = vec![self.id.clone()];
                for child in &self.children {
                    path.extend(child.weakest_path());
                }
                path
            }
        }
    }

    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    pub fn all_leaves(&self) -> Vec<&AttackNode> {
        if self.is_leaf() {
            return vec![self];
        }
        self.children
            .iter()
            .flat_map(|c| c.all_leaves())
            .collect()
    }
}

pub fn build_webapp_attack_tree() -> AttackNode {
    AttackNode::node(
        "ROOT",
        "Gain unauthorized access to web application",
        GateType::Or,
        vec![
            AttackNode::node(
                "TECH",
                "Technical attack",
                GateType::Or,
                vec![
                    AttackNode::node(
                        "WEB",
                        "Web application attack",
                        GateType::And,
                        vec![
                            AttackNode::leaf("RECON", "Reconnaissance and fingerprinting", 1),
                            AttackNode::leaf("VULN", "Find exploitable vulnerability", 3),
                        ],
                    ),
                    AttackNode::node(
                        "NET",
                        "Network attack",
                        GateType::And,
                        vec![
                            AttackNode::leaf("SCAN", "Network scanning", 2),
                            AttackNode::leaf("EXPLOIT", "Exploit network service", 5),
                        ],
                    ),
                ],
            ),
            AttackNode::node(
                "HUMAN",
                "Human attack",
                GateType::Or,
                vec![
                    AttackNode::leaf("PHISH", "Phishing campaign", 3),
                    AttackNode::leaf("INSIDER", "Insider threat / bribery", 8),
                ],
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_node() {
        let leaf = AttackNode::leaf("L1", "SQL Injection", 3);
        assert!(leaf.is_leaf());
        assert_eq!(leaf.min_cost(), 3);
        assert_eq!(leaf.node_count(), 1);
    }

    #[test]
    fn test_or_gate_min_cost() {
        let node = AttackNode::node(
            "OR1",
            "Exploit vulnerability",
            GateType::Or,
            vec![
                AttackNode::leaf("L1", "SQL Injection", 2),
                AttackNode::leaf("L2", "XSS Attack", 5),
                AttackNode::leaf("L3", "CSRF Attack", 8),
            ],
        );
        assert_eq!(node.min_cost(), 2);
    }

    #[test]
    fn test_and_gate_min_cost() {
        let node = AttackNode::node(
            "AND1",
            "Full compromise",
            GateType::And,
            vec![
                AttackNode::leaf("L1", "Gain initial access", 3),
                AttackNode::leaf("L2", "Escalate privileges", 5),
            ],
        );
        assert_eq!(node.min_cost(), 8);
    }

    #[test]
    fn test_nested_tree_min_cost() {
        let tree = AttackNode::node(
            "ROOT",
            "Steal data",
            GateType::Or,
            vec![
                AttackNode::node(
                    "AND1",
                    "Technical attack",
                    GateType::And,
                    vec![
                        AttackNode::leaf("L1", "Find vulnerability", 2),
                        AttackNode::leaf("L2", "Exploit it", 3),
                    ],
                ),
                AttackNode::leaf("L3", "Social engineering", 7),
            ],
        );
        assert_eq!(tree.min_cost(), 5);
    }

    #[test]
    fn test_weakest_path_leaf() {
        let leaf = AttackNode::leaf("L1", "SQL Injection", 3);
        assert_eq!(leaf.weakest_path(), vec!["L1"]);
    }

    #[test]
    fn test_weakest_path_or() {
        let tree = AttackNode::node(
            "ROOT",
            "Attack",
            GateType::Or,
            vec![
                AttackNode::leaf("L1", "Expensive", 10),
                AttackNode::leaf("L2", "Cheap", 2),
            ],
        );
        assert_eq!(tree.weakest_path(), vec!["ROOT", "L2"]);
    }

    #[test]
    fn test_node_count() {
        let tree = AttackNode::node(
            "ROOT",
            "Root",
            GateType::Or,
            vec![
                AttackNode::node(
                    "A1",
                    "Branch A",
                    GateType::And,
                    vec![
                        AttackNode::leaf("L1", "Leaf 1", 1),
                        AttackNode::leaf("L2", "Leaf 2", 2),
                    ],
                ),
                AttackNode::leaf("L3", "Leaf 3", 3),
            ],
        );
        assert_eq!(tree.node_count(), 5);
    }

    #[test]
    fn test_all_leaves() {
        let tree = AttackNode::node(
            "ROOT",
            "Root",
            GateType::Or,
            vec![
                AttackNode::node(
                    "A1",
                    "Branch",
                    GateType::And,
                    vec![
                        AttackNode::leaf("L1", "Leaf 1", 1),
                        AttackNode::leaf("L2", "Leaf 2", 2),
                    ],
                ),
                AttackNode::leaf("L3", "Leaf 3", 3),
            ],
        );
        let leaves = tree.all_leaves();
        assert_eq!(leaves.len(), 3);
        let ids: Vec<&str> = leaves.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"L1"));
        assert!(ids.contains(&"L2"));
        assert!(ids.contains(&"L3"));
    }

    #[test]
    fn test_build_webapp_attack_tree() {
        let tree = build_webapp_attack_tree();
        assert!(tree.node_count() >= 7, "Tree should have at least 7 nodes");
        assert!(!tree.is_leaf(), "Root should not be a leaf");
        let cost = tree.min_cost();
        assert!(cost > 0, "Min cost should be positive");
        let leaves = tree.all_leaves();
        assert!(leaves.len() >= 3, "Should have at least 3 leaf attack vectors");
    }
}
