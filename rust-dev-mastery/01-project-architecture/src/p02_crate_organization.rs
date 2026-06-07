//! # Lesson 2: Crate Organization
//!
//! Rust projects can contain library crates, binary crates, or both.
//! This lesson covers the conventions for organizing a crate with multiple
//! binaries, tests, examples, and benchmarks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the layout of a well-organized Rust project.
/// Real projects follow these conventions:
/// - `src/lib.rs` for the library
/// - `src/main.rs` for the default binary
/// - `src/bin/` for additional binaries
/// - `tests/` for integration tests
/// - `examples/` for runnable examples
/// - `benches/` for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLayout {
    pub name: String,
    pub has_lib: bool,
    pub main_binary: Option<String>,
    pub extra_binaries: Vec<String>,
    pub integration_tests: Vec<String>,
    pub examples: Vec<String>,
    pub benches: Vec<String>,
}

impl ProjectLayout {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            has_lib: false,
            main_binary: None,
            extra_binaries: Vec::new(),
            integration_tests: Vec::new(),
            examples: Vec::new(),
            benches: Vec::new(),
        }
    }

    /// Mark this project as having a library crate.
    pub fn with_lib(mut self) -> Self {
        self.has_lib = true;
        self
    }

    /// Set the main binary name (src/main.rs).
    pub fn with_main_binary(mut self, name: impl Into<String>) -> Self {
        self.main_binary = Some(name.into());
        self
    }

    /// Add an extra binary (src/bin/{name}.rs).
    pub fn with_extra_binary(mut self, name: impl Into<String>) -> Self {
        self.extra_binaries.push(name.into());
        self
    }

    /// Add an integration test file (tests/{name}.rs).
    pub fn with_integration_test(mut self, name: impl Into<String>) -> Self {
        self.integration_tests.push(name.into());
        self
    }

    /// Add an example (examples/{name}.rs).
    pub fn with_example(mut self, name: impl Into<String>) -> Self {
        self.examples.push(name.into());
        self
    }

    /// Add a benchmark (benches/{name}.rs).
    pub fn with_bench(mut self, name: impl Into<String>) -> Self {
        self.benches.push(name.into());
        self
    }

    /// Generate the directory tree for this project.
    pub fn directory_tree(&self) -> Vec<String> {
        let mut tree = vec![format!("{}/", self.name)];

        if self.has_lib {
            tree.push("  src/lib.rs".to_string());
        }

        if self.main_binary.is_some() {
            tree.push("  src/main.rs".to_string());
        }

        if !self.extra_binaries.is_empty() {
            tree.push("  src/bin/".to_string());
            for bin in &self.extra_binaries {
                tree.push(format!("    {}.rs", bin));
            }
        }

        if !self.integration_tests.is_empty() {
            tree.push("  tests/".to_string());
            for test in &self.integration_tests {
                tree.push(format!("    {}.rs", test));
            }
        }

        if !self.examples.is_empty() {
            tree.push("  examples/".to_string());
            for ex in &self.examples {
                tree.push(format!("    {}.rs", ex));
            }
        }

        if !self.benches.is_empty() {
            tree.push("  benches/".to_string());
            for bench in &self.benches {
                tree.push(format!("    {}.rs", bench));
            }
        }

        tree.push("  Cargo.toml".to_string());
        tree
    }
}

/// Represents a module in a library crate's module tree.
/// Modules can contain items (functions, types) and sub-modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    pub name: String,
    pub is_public: bool,
    pub items: Vec<Item>,
    pub children: Vec<ModuleNode>,
}

/// A public item within a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Const,
    Static,
}

impl ModuleNode {
    pub fn new(name: impl Into<String>, is_public: bool) -> Self {
        Self {
            name: name.into(),
            is_public,
            items: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn add_item(&mut self, name: impl Into<String>, kind: ItemKind, is_public: bool) {
        self.items.push(Item {
            name: name.into(),
            kind,
            is_public,
        });
    }

    pub fn add_child(&mut self, child: ModuleNode) {
        self.children.push(child);
    }

    /// Flatten the module tree into a list of fully qualified paths.
    pub fn flatten(&self, prefix: &str) -> Vec<String> {
        let path = if prefix.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", prefix, self.name)
        };

        let mut result = Vec::new();
        for item in &self.items {
            if item.is_public {
                result.push(format!("{}::{}", path, item.name));
            }
        }
        for child in &self.children {
            result.extend(child.flatten(&path));
        }
        result
    }
}

/// A registry of crates and their roles in a larger system.
/// This models how a team might think about crate responsibilities.
#[derive(Debug, Serialize, Deserialize)]
pub struct CrateRegistry {
    pub crates: HashMap<String, CrateRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateRole {
    pub purpose: String,
    pub public_api_size: usize,
    pub internal_only: bool,
}

impl CrateRegistry {
    pub fn new() -> Self {
        Self {
            crates: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: impl Into<String>,
        purpose: impl Into<String>,
        api_size: usize,
        internal: bool,
    ) {
        self.crates.insert(
            name.into(),
            CrateRole {
                purpose: purpose.into(),
                public_api_size: api_size,
                internal_only: internal,
            },
        );
    }

    /// List all public-facing crates (not internal-only).
    pub fn public_crates(&self) -> Vec<(&str, &CrateRole)> {
        self.crates
            .iter()
            .filter(|(_, role)| !role.internal_only)
            .map(|(name, role)| (name.as_str(), role))
            .collect()
    }

    /// Find the crate with the largest public API surface.
    pub fn largest_api(&self) -> Option<(&str, &CrateRole)> {
        self.crates
            .iter()
            .max_by_key(|(_, role)| role.public_api_size)
            .map(|(name, role)| (name.as_str(), role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lib_only_project() {
        let layout = ProjectLayout::new("mylib").with_lib();
        assert!(layout.has_lib);
        assert!(layout.main_binary.is_none());
        assert!(layout.extra_binaries.is_empty());
    }

    #[test]
    fn test_full_project() {
        let layout = ProjectLayout::new("myapp")
            .with_lib()
            .with_main_binary("myapp")
            .with_extra_binary("worker")
            .with_extra_binary("migrator")
            .with_integration_test("api_tests")
            .with_example("basic_usage")
            .with_bench("throughput");

        let tree = layout.directory_tree();
        assert!(tree.contains(&"  src/lib.rs".to_string()));
        assert!(tree.contains(&"  src/main.rs".to_string()));
        assert!(tree.contains(&"  src/bin/".to_string()));
        assert!(tree.contains(&"    worker.rs".to_string()));
        assert!(tree.contains(&"    migrator.rs".to_string()));
        assert!(tree.contains(&"  tests/".to_string()));
        assert!(tree.contains(&"    api_tests.rs".to_string()));
        assert!(tree.contains(&"  examples/".to_string()));
        assert!(tree.contains(&"    basic_usage.rs".to_string()));
        assert!(tree.contains(&"  benches/".to_string()));
        assert!(tree.contains(&"    throughput.rs".to_string()));
        assert!(tree.contains(&"  Cargo.toml".to_string()));
    }

    #[test]
    fn test_directory_tree_root() {
        let layout = ProjectLayout::new("test-proj").with_lib();
        let tree = layout.directory_tree();
        assert_eq!(tree[0], "test-proj/");
    }

    #[test]
    fn test_module_tree() {
        let mut root = ModuleNode::new("lib", true);
        root.add_item("Config", ItemKind::Struct, true);
        root.add_item("Error", ItemKind::Enum, true);
        root.add_item("internal_helper", ItemKind::Function, false);

        let mut db = ModuleNode::new("db", true);
        db.add_item("Connection", ItemKind::Struct, true);
        db.add_item("Pool", ItemKind::Struct, true);

        let mut cache = ModuleNode::new("cache", true);
        cache.add_item("CacheKey", ItemKind::TypeAlias, true);

        root.add_child(db);
        root.add_child(cache);

        let flat = root.flatten("");
        assert!(flat.contains(&"lib::Config".to_string()));
        assert!(flat.contains(&"lib::Error".to_string()));
        assert!(flat.contains(&"lib::db::Connection".to_string()));
        assert!(flat.contains(&"lib::db::Pool".to_string()));
        assert!(flat.contains(&"lib::cache::CacheKey".to_string()));
        // private items should not appear
        assert!(!flat.iter().any(|s| s.contains("internal_helper")));
    }

    #[test]
    fn test_module_tree_nested() {
        let mut root = ModuleNode::new("lib", true);
        let mut inner = ModuleNode::new("inner", true);
        inner.add_item("DeepType", ItemKind::Struct, true);
        root.add_child(inner);

        let flat = root.flatten("");
        assert!(flat.contains(&"lib::inner::DeepType".to_string()));
    }

    #[test]
    fn test_crate_registry() {
        let mut reg = CrateRegistry::new();
        reg.register("core", "Core business logic", 50, false);
        reg.register("db", "Database access layer", 20, false);
        reg.register("test-utils", "Test helpers", 10, true);

        let public = reg.public_crates();
        assert_eq!(public.len(), 2);
        let names: Vec<&str> = public.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"db"));
        assert!(!names.contains(&"test-utils"));
    }

    #[test]
    fn test_largest_api() {
        let mut reg = CrateRegistry::new();
        reg.register("small", "Small crate", 5, false);
        reg.register("large", "Large crate", 100, false);
        reg.register("medium", "Medium crate", 30, false);

        let (name, role) = reg.largest_api().unwrap();
        assert_eq!(name, "large");
        assert_eq!(role.public_api_size, 100);
    }

    #[test]
    fn test_empty_registry() {
        let reg = CrateRegistry::new();
        assert!(reg.public_crates().is_empty());
        assert!(reg.largest_api().is_none());
    }
}
