//! # Lesson 04: Data Flow Analysis for Security
//!
//! ## The Problem
//!
//! You can't protect what you don't understand. Before identifying threats, you need to map
//! how data moves through your system: where it enters, where it's processed, where it's stored,
//! and — critically — where trust levels change.
//!
//! ## The Solution: Data Flow Diagrams (DFDs) for Security
//!
//! A security-focused DFD identifies:
//!
//! ```text
//! External Entity (untrusted) ──[Data Flow]──> Process (trusted)
//!                        │                          │
//!                   Trust Boundary              Data Store
//! ```
//!
//! Key elements:
//! - **External entities**: Users, APIs, third-party services (outside your control)
//! - **Processes**: Application components, microservices (you control these)
//! - **Data stores**: Databases, files, caches, queues
//! - **Data flows**: HTTP requests, database queries, message queues
//! - **Trust boundaries**: Lines where the privilege level changes
//!
//! ## Why Trust Boundaries Matter
//!
//! Threats arise at trust boundary crossings. When data crosses from an untrusted zone
//! to a trusted zone, it must be validated, sanitized, and authenticated. Without explicit
//! trust boundaries, developers assume all input is safe.
//!
//! ## Attack Example: Unvalidated Boundary Crossing
//!
//! An API accepts JSON from the internet (untrusted) and passes it directly to a database
//! query (trusted). The trust boundary crossing has no validation. Result: SQL injection.
//!
//! Defense: Mark every trust boundary and enforce validation at each crossing.

use serde::{Deserialize, Serialize};

/// The type of a node in the data flow diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// External user, API, or service outside your control
    ExternalEntity,
    /// Application component that processes data
    Process,
    /// Database, file system, cache, or queue
    DataStore,
}

/// A node in the data flow diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub trust_level: u8, // 1 = untrusted, 10 = fully trusted
}

/// A data flow between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub description: String,
    pub crosses_trust_boundary: bool,
}

/// A complete data flow diagram with security annotations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowDiagram {
    pub name: String,
    pub nodes: Vec<Node>,
    pub flows: Vec<DataFlow>,
}

impl DataFlowDiagram {
    /// Create a new empty data flow diagram.
    pub fn new(name: &str) -> Self {
        todo!("Create a new empty diagram")
    }

    /// Add a node to the diagram.
    pub fn add_node(&mut self, node: Node) {
        todo!("Add a node to the diagram")
    }

    /// Add a data flow to the diagram.
    /// Automatically set `crosses_trust_boundary` if source and target have different trust levels.
    pub fn add_flow(&mut self, flow: DataFlow) {
        todo!("Add flow; detect trust boundary crossing from node trust levels")
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        todo!("Find node by ID")
    }

    /// Return all flows that cross trust boundaries.
    pub fn trust_boundary_flows(&self) -> Vec<&DataFlow> {
        todo!("Filter flows where crosses_trust_boundary is true")
    }

    /// Return all external entities.
    pub fn external_entities(&self) -> Vec<&Node> {
        todo!("Filter nodes by ExternalEntity type")
    }

    /// Return all data stores.
    pub fn data_stores(&self) -> Vec<&Node> {
        todo!("Filter nodes by DataStore type")
    }

    /// Count total trust boundary crossings.
    pub fn trust_boundary_count(&self) -> usize {
        todo!("Count flows that cross trust boundaries")
    }

    /// Find nodes with no incoming flows (entry points).
    pub fn entry_points(&self) -> Vec<&Node> {
        todo!("Find nodes that have no incoming flows (sources only)")
    }
}

/// Build a data flow diagram for a typical web application.
pub fn build_webapp_dfd() -> DataFlowDiagram {
    todo!("Build a DFD with: Browser -> API Gateway -> App Server -> Database, plus a cache and external auth provider")
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
            crosses_trust_boundary: false, // should be auto-set
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
        // browser(trust=1) -> api(trust=7) should cross boundary
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
        // browser has no incoming flows
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
