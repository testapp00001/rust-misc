//! # Module Organization
//!
//! Rust's module system controls visibility and organization. This module covers
//! best practices for structuring large Rust projects.
//!
//! ## File Structure:
//!
//! ```text
//! src/
//!   lib.rs          # Crate root, re-exports
//!   main.rs         # Binary entry point
//!   config.rs       # Simple module
//!   models/         # Module directory
//!     mod.rs        # Module root
//!     user.rs       # Submodule
//!     order.rs      # Submodule
//!   services/       # Module directory
//!     mod.rs
//!     auth.rs
//!     payment.rs
//! ```
//!
//! ## Visibility:
//!
//! | Keyword | Scope |
//! |---------|-------|
//! | `pub` | Public |
//! | `pub(crate)` | Within crate |
//! | `pub(super)` | Parent module |
//! | `pub(in path)` | Specific path |
//! | (none) | Private |

use std::collections::HashMap;

/// Module structure analyzer.
pub struct ModuleAnalyzer;

impl ModuleAnalyzer {
    /// Check if a module name follows conventions.
    pub fn is_valid_module_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
            && !name.starts_with('_')
            && !name.ends_with('_')
    }

    /// Suggest module name from a type name.
    pub fn suggest_module_name(type_name: &str) -> String {
        let mut result = String::new();
        for (i, c) in type_name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }
}

/// Re-export pattern for clean public API.
/// In real code, this would be in lib.rs.
pub mod prelude {
    pub use super::config::AppConfig;
    pub use super::models::{User, Order};
}

/// Example module structure.
pub mod config {
    #[derive(Debug, Clone)]
    pub struct AppConfig {
        pub name: String,
        pub version: String,
    }

    impl AppConfig {
        pub fn new(name: &str, version: &str) -> Self {
            Self {
                name: name.into(),
                version: version.into(),
            }
        }
    }
}

pub mod models {
    #[derive(Debug, Clone)]
    pub struct User {
        pub id: u64,
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: u64,
        pub user_id: u64,
        pub total: f64,
    }
}

pub mod services {
    use super::models;

    pub trait UserService {
        fn find_user(&self, id: u64) -> Option<models::User>;
    }

    pub struct MockUserService {
        users: std::collections::HashMap<u64, models::User>,
    }

    impl MockUserService {
        pub fn new() -> Self {
            Self {
                users: std::collections::HashMap::new(),
            }
        }

        pub fn add_user(&mut self, user: models::User) {
            self.users.insert(user.id, user);
        }
    }

    impl UserService for MockUserService {
        fn find_user(&self, id: u64) -> Option<models::User> {
            self.users.get(&id).cloned()
        }
    }
}

/// Visibility pattern analysis.
pub struct VisibilityAnalysis {
    pub items: Vec<VisibilityItem>,
}

#[derive(Debug, Clone)]
pub struct VisibilityItem {
    pub name: String,
    pub visibility: Visibility,
    pub is_used_externally: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    PubCrate,
    PubSuper,
    Private,
}

impl VisibilityAnalysis {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, name: &str, visibility: Visibility, used_externally: bool) {
        self.items.push(VisibilityItem {
            name: name.into(),
            visibility,
            is_used_externally: used_externally,
        });
    }

    /// Find items that are public but not used externally.
    pub fn over_exposed(&self) -> Vec<&VisibilityItem> {
        self.items
            .iter()
            .filter(|i| i.visibility == Visibility::Public && !i.is_used_externally)
            .collect()
    }

    /// Find items that are private but used externally (should be public).
    pub fn under_exposed(&self) -> Vec<&VisibilityItem> {
        self.items
            .iter()
            .filter(|i| i.visibility == Visibility::Private && i.is_used_externally)
            .collect()
    }

    pub fn suggest_improvements(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        for item in self.over_exposed() {
            suggestions.push(format!(
                "'{}' is pub but not used externally - consider pub(crate)",
                item.name
            ));
        }
        for item in self.under_exposed() {
            suggestions.push(format!(
                "'{}' is private but used externally - consider making it pub",
                item.name
            ));
        }
        suggestions
    }
}

/// Module dependency graph.
pub struct ModuleGraph {
    dependencies: HashMap<String, Vec<String>>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, module: &str, depends_on: &str) {
        self.dependencies
            .entry(module.into())
            .or_default()
            .push(depends_on.into());
    }

    pub fn dependencies_of(&self, module: &str) -> Vec<&str> {
        self.dependencies
            .get(module)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn has_circular_dependency(&self) -> bool {
        for module in self.dependencies.keys() {
            if self.detect_cycle(module, &mut std::collections::HashSet::new()) {
                return true;
            }
        }
        false
    }

    fn detect_cycle(&self, module: &str, visited: &mut std::collections::HashSet<String>) -> bool {
        if !visited.insert(module.into()) {
            return true;
        }
        if let Some(deps) = self.dependencies.get(module) {
            for dep in deps {
                if self.detect_cycle(dep, visited) {
                    return true;
                }
            }
        }
        visited.remove(module);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_module_name() {
        assert!(ModuleAnalyzer::is_valid_module_name("my_module"));
        assert!(ModuleAnalyzer::is_valid_module_name("models"));
        assert!(!ModuleAnalyzer::is_valid_module_name("MyModule"));
        assert!(!ModuleAnalyzer::is_valid_module_name("_private"));
        assert!(!ModuleAnalyzer::is_valid_module_name(""));
    }

    #[test]
    fn test_suggest_module_name() {
        assert_eq!(ModuleAnalyzer::suggest_module_name("UserService"), "user_service");
        assert_eq!(ModuleAnalyzer::suggest_module_name("HTTPClient"), "h_t_t_p_client");
        assert_eq!(ModuleAnalyzer::suggest_module_name("simple"), "simple");
    }

    #[test]
    fn test_module_structure() {
        let config = config::AppConfig::new("test", "1.0");
        assert_eq!(config.name, "test");

        let user = models::User {
            id: 1,
            name: "Alice".into(),
        };
        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_service_layer() {
        use services::UserService;

        let mut service = services::MockUserService::new();
        service.add_user(models::User {
            id: 1,
            name: "Bob".into(),
        });

        let user = service.find_user(1);
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Bob");
    }

    #[test]
    fn test_visibility_analysis() {
        let mut analysis = VisibilityAnalysis::new();
        analysis.add_item("public_func", Visibility::Public, false);
        analysis.add_item("private_func", Visibility::Private, true);

        assert_eq!(analysis.over_exposed().len(), 1);
        assert_eq!(analysis.under_exposed().len(), 1);
    }

    #[test]
    fn test_visibility_suggestions() {
        let mut analysis = VisibilityAnalysis::new();
        analysis.add_item("helper", Visibility::Public, false);

        let suggestions = analysis.suggest_improvements();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("pub(crate)"));
    }

    #[test]
    fn test_module_graph() {
        let mut graph = ModuleGraph::new();
        graph.add_dependency("main", "config");
        graph.add_dependency("main", "models");
        graph.add_dependency("services", "models");

        assert_eq!(graph.dependencies_of("main").len(), 2);
        assert!(!graph.has_circular_dependency());
    }

    #[test]
    fn test_module_graph_circular() {
        let mut graph = ModuleGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "a");

        assert!(graph.has_circular_dependency());
    }

    #[test]
    fn test_prelude_re_exports() {
        let config = prelude::AppConfig::new("test", "1.0");
        assert_eq!(config.name, "test");

        let user = prelude::User {
            id: 1,
            name: "Alice".into(),
        };
        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_module_graph_no_dependencies() {
        let graph = ModuleGraph::new();
        assert!(graph.dependencies_of("any").is_empty());
        assert!(!graph.has_circular_dependency());
    }
}
