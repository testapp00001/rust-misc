//! # Lesson 04: Data Flow Analysis for Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    ExternalEntity,
    Process,
    DataStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub trust_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub description: String,
    pub crosses_trust_boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowDiagram {
    pub name: String,
    pub nodes: Vec<Node>,
    pub flows: Vec<DataFlow>,
}

impl DataFlowDiagram {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: Vec::new(),
            flows: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_flow(&mut self, mut flow: DataFlow) {
        // Auto-detect trust boundary crossing
        let source_trust = self.nodes.iter().find(|n| n.id == flow.source_id).map(|n| n.trust_level);
        let target_trust = self.nodes.iter().find(|n| n.id == flow.target_id).map(|n| n.trust_level);

        if let (Some(s), Some(t)) = (source_trust, target_trust) {
            flow.crosses_trust_boundary = s != t;
        }

        self.flows.push(flow);
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn trust_boundary_flows(&self) -> Vec<&DataFlow> {
        self.flows.iter().filter(|f| f.crosses_trust_boundary).collect()
    }

    pub fn external_entities(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.node_type == NodeType::ExternalEntity).collect()
    }

    pub fn data_stores(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.node_type == NodeType::DataStore).collect()
    }

    pub fn trust_boundary_count(&self) -> usize {
        self.flows.iter().filter(|f| f.crosses_trust_boundary).count()
    }

    pub fn entry_points(&self) -> Vec<&Node> {
        let target_ids: Vec<&str> = self.flows.iter().map(|f| f.target_id.as_str()).collect();
        self.nodes
            .iter()
            .filter(|n| !target_ids.contains(&n.id.as_str()))
            .collect()
    }
}

pub fn build_webapp_dfd() -> DataFlowDiagram {
    let mut dfd = DataFlowDiagram::new("web-application");

    dfd.add_node(Node {
        id: "browser".into(),
        name: "User Browser".into(),
        node_type: NodeType::ExternalEntity,
        trust_level: 1,
    });

    dfd.add_node(Node {
        id: "auth-provider".into(),
        name: "External OAuth Provider".into(),
        node_type: NodeType::ExternalEntity,
        trust_level: 3,
    });

    dfd.add_node(Node {
        id: "api-gateway".into(),
        name: "API Gateway".into(),
        node_type: NodeType::Process,
        trust_level: 6,
    });

    dfd.add_node(Node {
        id: "app-server".into(),
        name: "Application Server".into(),
        node_type: NodeType::Process,
        trust_level: 8,
    });

    dfd.add_node(Node {
        id: "cache".into(),
        name: "Redis Cache".into(),
        node_type: NodeType::DataStore,
        trust_level: 7,
    });

    dfd.add_node(Node {
        id: "database".into(),
        name: "PostgreSQL Database".into(),
        node_type: NodeType::DataStore,
        trust_level: 9,
    });

    dfd.add_flow(DataFlow {
        id: "f1".into(),
        source_id: "browser".into(),
        target_id: "api-gateway".into(),
        description: "HTTPS requests from user".into(),
        crosses_trust_boundary: false,
    });

    dfd.add_flow(DataFlow {
        id: "f2".into(),
        source_id: "api-gateway".into(),
        target_id: "auth-provider".into(),
        description: "OAuth token validation".into(),
        crosses_trust_boundary: false,
    });

    dfd.add_flow(DataFlow {
        id: "f3".into(),
        source_id: "api-gateway".into(),
        target_id: "app-server".into(),
        description: "Authenticated API requests".into(),
        crosses_trust_boundary: false,
    });

    dfd.add_flow(DataFlow {
        id: "f4".into(),
        source_id: "app-server".into(),
        target_id: "cache".into(),
        description: "Cache read/write".into(),
        crosses_trust_boundary: false,
    });

    dfd.add_flow(DataFlow {
        id: "f5".into(),
        source_id: "app-server".into(),
        target_id: "database".into(),
        description: "SQL queries via parameterized statements".into(),
        crosses_trust_boundary: false,
    });

    dfd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_dfd() -> DataFlowDiagram {
        let mut dfd = DataFlowDiagram::new("test-system");
        dfd.add_node(Node {
            id: "browser".into(),
            name: "User Browser".into(),
            node_type: NodeType::ExternalEntity,
            trust_level: 1,
        });
        dfd.add_node(Node {
            id: "api".into(),
            name: "API Server".into(),
            node_type: NodeType::Process,
            trust_level: 7,
        });
        dfd.add_node(Node {
            id: "db".into(),
            name: "Database".into(),
            node_type: NodeType::DataStore,
            trust_level: 9,
        });
        dfd.add_flow(DataFlow {
            id: "f1".into(),
            source_id: "browser".into(),
            target_id: "api".into(),
            description: "HTTP request".into(),
            crosses_trust_boundary: false,
        });
        dfd.add_flow(DataFlow {
            id: "f2".into(),
            source_id: "api".into(),
            target_id: "db".into(),
            description: "SQL query".into(),
            crosses_trust_boundary: false,
        });
        dfd
    }

    #[test]
    fn test_dfd_creation() {
        let dfd = DataFlowDiagram::new("my-system");
        assert_eq!(dfd.name, "my-system");
        assert_eq!(dfd.nodes.len(), 0);
        assert_eq!(dfd.flows.len(), 0);
    }

    #[test]
    fn test_add_nodes_and_flows() {
        let dfd = make_simple_dfd();
        assert_eq!(dfd.nodes.len(), 3);
        assert_eq!(dfd.flows.len(), 2);
    }

    #[test]
    fn test_get_node() {
        let dfd = make_simple_dfd();
        let node = dfd.get_node("api");
        assert!(node.is_some());
        assert_eq!(node.unwrap().name, "API Server");
        assert!(dfd.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_trust_boundary_detection() {
        let dfd = make_simple_dfd();
        let boundary_flows = dfd.trust_boundary_flows();
        assert!(boundary_flows.len() >= 1, "Should detect trust boundary crossing");
    }

    #[test]
    fn test_external_entities() {
        let dfd = make_simple_dfd();
        let externals = dfd.external_entities();
        assert_eq!(externals.len(), 1);
        assert_eq!(externals[0].id, "browser");
    }

    #[test]
    fn test_data_stores() {
        let dfd = make_simple_dfd();
        let stores = dfd.data_stores();
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].id, "db");
    }

    #[test]
    fn test_entry_points() {
        let dfd = make_simple_dfd();
        let entries = dfd.entry_points();
        assert!(entries.iter().any(|n| n.id == "browser"));
    }

    #[test]
    fn test_build_webapp_dfd() {
        let dfd = build_webapp_dfd();
        assert!(dfd.nodes.len() >= 4, "Should have at least 4 nodes");
        assert!(dfd.flows.len() >= 3, "Should have at least 3 flows");
        assert!(
            dfd.trust_boundary_count() >= 1,
            "Should have at least 1 trust boundary crossing"
        );
        assert!(
            dfd.external_entities().len() >= 1,
            "Should have at least 1 external entity"
        );
    }
}
