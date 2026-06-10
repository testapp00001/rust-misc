//! # Lesson 02: Attack Trees
//!
//! ## The Problem
//!
//! When analyzing a system's security, you need to decompose complex attacks into
//! manageable steps. Without structured decomposition, you miss attack paths or
//! waste resources defending against expensive attacks while ignoring cheap ones.
//!
//! ## The Solution: Attack Trees
//!
//! An attack tree is a hierarchical representation of an attack:
//!
//! ```text
//!                     [Root Goal: Steal User Data]
//!                     /            |            \
//!            [SQL Injection]   [XSS Attack]   [Social Engineering]
//!            /          \          |
//!     [Union Query]  [Blind SQL]  [Stored XSS]
//! ```
//!
//! Each node can have child nodes connected by:
//! - **AND**: All children must succeed for the parent to succeed
//! - **OR**: Any one child succeeding is enough for the parent
//!
//! ## Weakest Path Analysis
//!
//! The "weakest path" is the attack path with the lowest total cost. By assigning
//! cost/effort to leaf nodes, you can compute the minimum cost to achieve the root goal.
//! This tells defenders where to invest first.
//!
//! ## Attack Example: Finding the Weakest Path
//!
//! A system has three attack paths:
//! - Path A: SQL injection (cost: 2) + privilege escalation (cost: 8) = 10
//! - Path B: Social engineering (cost: 3) = 3
//! - Path C: Zero-day exploit (cost: 10) = 10
//!
//! The weakest path is B (cost 3). Defenders should harden against social engineering first.

use serde::{Deserialize, Serialize};

/// The logical gate type for a node's children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    /// All children must succeed
    And,
    /// Any one child succeeding is sufficient
    Or,
}

/// A single node in an attack tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackNode {
    pub id: String,
    pub description: String,
    pub cost: u32,
    pub gate: GateType,
    pub children: Vec<AttackNode>,
}

impl AttackNode {
    /// Create a leaf node (no children) with a given cost.
    pub fn leaf(id: &str, description: &str, cost: u32) -> Self {
        todo!("Create a leaf node with the Or gate type (leaf default)")
    }

    /// Create an intermediate node with a gate type and children.
    pub fn node(id: &str, description: &str, gate: GateType, children: Vec<AttackNode>) -> Self {
        todo!("Create a node; cost should be 0 (computed from children)")
    }

    /// Check if this node is a leaf (no children).
    pub fn is_leaf(&self) -> bool {
        todo!("Return true if children is empty")
    }

    /// Compute the minimum cost to achieve this attack goal.
    ///
    /// For a leaf node: return its own cost.
    /// For an OR gate: return the minimum cost among children.
    /// For an AND gate: return the sum of all children's costs.
    pub fn min_cost(&self) -> u32 {
        todo!("Recursively compute minimum attack cost")
    }

    /// Find the weakest path (sequence of node IDs from root to the cheapest leaf).
    ///
    /// For a leaf node: return vec![self.id].
    /// For an OR gate: follow the child with the lowest min_cost.
    /// For an AND gate: follow ALL children (concatenate paths).
    pub fn weakest_path(&self) -> Vec<String> {
        todo!("Find the path(s) with minimum total cost")
    }

    /// Count total number of nodes in this tree (including self).
    pub fn node_count(&self) -> usize {
        todo!("Count all nodes recursively")
    }

    /// Find all leaf nodes in this tree.
    pub fn all_leaves(&self) -> Vec<&AttackNode> {
        todo!("Collect all leaf nodes recursively")
    }
}

/// Build an attack tree for "Gain unauthorized access to a web application".
pub fn build_webapp_attack_tree() -> AttackNode {
    todo!("Build a realistic attack tree with at least 3 levels and mixed AND/OR gates")
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
        // OR( AND(leaf(2), leaf(3)), leaf(7) ) => min(2+3, 7) = 5
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
