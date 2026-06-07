//! # Lesson 4: Feature Flags
//!
//! Cargo features allow conditional compilation and optional dependencies.
//! This lesson covers defining features, optional deps, feature unification,
//! and using `#[cfg]` attributes for conditional code.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Feature-gated modules: in real code, you'd gate entire modules
// ---------------------------------------------------------------------------

/// Simulates a feature-gated database module.
/// In Cargo.toml: `db = { ..., optional = true }`
/// Feature: `database`
#[cfg(feature = "database")]
pub mod database {
    pub fn connect(url: &str) -> Result<Connection, String> {
        Ok(Connection {
            url: url.to_string(),
            connected: true,
        })
    }

    pub struct Connection {
        pub url: String,
        pub connected: bool,
    }
}

/// Feature definitions and metadata.
/// In a real project, these map to `[features]` in Cargo.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub name: String,
    pub description: String,
    pub enables: Vec<String>,
    pub requires: Vec<String>,
    pub default: bool,
}

impl FeatureDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            enables: Vec::new(),
            requires: Vec::new(),
            default: false,
        }
    }

    pub fn enables(mut self, dep: impl Into<String>) -> Self {
        self.enables.push(dep.into());
        self
    }

    pub fn requires(mut self, feature: impl Into<String>) -> Self {
        self.requires.push(feature.into());
        self
    }

    pub fn as_default(mut self) -> Self {
        self.default = true;
        self
    }
}

/// Models a crate's feature configuration.
/// This helps understand how features interact.
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureSet {
    pub crate_name: String,
    pub features: Vec<FeatureDefinition>,
    pub active: Vec<String>,
}

impl FeatureSet {
    pub fn new(crate_name: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            features: Vec::new(),
            active: Vec::new(),
        }
    }

    pub fn add_feature(&mut self, feature: FeatureDefinition) {
        if feature.default {
            self.active.push(feature.name.clone());
        }
        self.features.push(feature);
    }

    /// Enable a feature and all its transitive dependencies.
    pub fn enable(&mut self, name: &str) -> Result<(), FeatureError> {
        let feature = self
            .features
            .iter()
            .find(|f| f.name == name)
            .ok_or_else(|| FeatureError::UnknownFeature(name.to_string()))?;

        // Check requirements
        for req in &feature.requires {
            if !self.active.contains(req) {
                return Err(FeatureError::MissingRequirement {
                    feature: name.to_string(),
                    requirement: req.clone(),
                });
            }
        }

        // Enable this feature
        if !self.active.contains(&name.to_string()) {
            self.active.push(name.to_string());
        }

        // Enable dependencies
        for dep in &feature.enables {
            if !self.active.contains(dep) {
                self.active.push(dep.clone());
            }
        }

        Ok(())
    }

    /// Disable a feature. Returns error if other active features require it.
    pub fn disable(&mut self, name: &str) -> Result<(), FeatureError> {
        // Check if any active feature requires this one
        for active_name in &self.active {
            if let Some(f) = self.features.iter().find(|f| &f.name == active_name) {
                if f.requires.contains(&name.to_string()) {
                    return Err(FeatureError::RequiredBy {
                        feature: name.to_string(),
                        required_by: active_name.clone(),
                    });
                }
            }
        }

        self.active.retain(|f| f != name);
        Ok(())
    }

    pub fn is_active(&self, name: &str) -> bool {
        self.active.contains(&name.to_string())
    }

    /// Get the active features as a sorted list.
    pub fn active_features(&self) -> Vec<&str> {
        let mut features: Vec<&str> = self.active.iter().map(|s| s.as_str()).collect();
        features.sort();
        features
    }
}

#[derive(Debug, PartialEq)]
pub enum FeatureError {
    UnknownFeature(String),
    MissingRequirement {
        feature: String,
        requirement: String,
    },
    RequiredBy {
        feature: String,
        required_by: String,
    },
}

/// A builder for generating Cargo.toml [features] section.
pub struct FeaturesTomlBuilder {
    features: Vec<FeatureDefinition>,
}

impl FeaturesTomlBuilder {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    pub fn add(mut self, feature: FeatureDefinition) -> Self {
        self.features.push(feature);
        self
    }

    pub fn build(&self) -> String {
        let mut toml = String::from("[features]\n");

        // Default feature
        let defaults: Vec<&str> = self
            .features
            .iter()
            .filter(|f| f.default)
            .map(|f| f.name.as_str())
            .collect();
        if !defaults.is_empty() {
            let default_list: Vec<String> =
                defaults.iter().map(|d| format!("\"{}\"", d)).collect();
            toml.push_str(&format!("default = [{}]\n", default_list.join(", ")));
        }

        // Each feature
        for feature in &self.features {
            if feature.enables.is_empty() {
                toml.push_str(&format!("\"{}\" = []\n", feature.name));
            } else {
                let deps: Vec<String> = feature
                    .enables
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect();
                toml.push_str(&format!(
                    "\"{}\" = [{}]\n",
                    feature.name,
                    deps.join(", ")
                ));
            }
        }

        toml
    }
}

impl Default for FeaturesTomlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serde_feature_set() -> FeatureSet {
        let mut fs = FeatureSet::new("mycrate");

        fs.add_feature(
            FeatureDefinition::new("default", "Default features").as_default(),
        );
        fs.add_feature(
            FeatureDefinition::new("json", "JSON support")
                .enables("serde_json"),
        );
        fs.add_feature(
            FeatureDefinition::new("full", "All features")
                .enables("json")
                .enables("database"),
        );
        fs.add_feature(
            FeatureDefinition::new("database", "Database support")
                .enables("sqlx"),
        );

        fs
    }

    #[test]
    fn test_default_features() {
        let fs = serde_feature_set();
        assert!(fs.is_active("default"));
        assert!(!fs.is_active("json"));
        assert!(!fs.is_active("full"));
    }

    #[test]
    fn test_enable_feature() {
        let mut fs = serde_feature_set();
        fs.enable("json").unwrap();
        assert!(fs.is_active("json"));
        assert!(fs.is_active("serde_json")); // transitive
    }

    #[test]
    fn test_enable_unknown_feature() {
        let mut fs = serde_feature_set();
        let result = fs.enable("nonexistent");
        assert_eq!(
            result,
            Err(FeatureError::UnknownFeature("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_feature_requirements() {
        let mut fs = FeatureSet::new("mycrate");
        fs.add_feature(FeatureDefinition::new("base", "Base feature"));
        fs.add_feature(
            FeatureDefinition::new("extended", "Extended feature").requires("base"),
        );

        // Can't enable extended without base
        let result = fs.enable("extended");
        assert!(matches!(
            result,
            Err(FeatureError::MissingRequirement { .. })
        ));

        // Enable base first, then extended works
        fs.enable("base").unwrap();
        fs.enable("extended").unwrap();
        assert!(fs.is_active("extended"));
    }

    #[test]
    fn test_disable_feature() {
        let mut fs = serde_feature_set();
        fs.enable("json").unwrap();
        assert!(fs.is_active("json"));

        fs.disable("json").unwrap();
        assert!(!fs.is_active("json"));
    }

    #[test]
    fn test_disable_required_feature() {
        let mut fs = FeatureSet::new("mycrate");
        fs.add_feature(FeatureDefinition::new("base", "Base feature"));
        fs.add_feature(
            FeatureDefinition::new("extended", "Extended feature").requires("base"),
        );

        fs.enable("base").unwrap();
        fs.enable("extended").unwrap();

        // Can't disable base while extended is active
        let result = fs.disable("base");
        assert!(matches!(result, Err(FeatureError::RequiredBy { .. })));
    }

    #[test]
    fn test_disable_unknown_feature() {
        let mut fs = serde_feature_set();
        // Disabling a feature that doesn't exist should succeed (no-op)
        fs.disable("nonexistent").unwrap();
    }

    #[test]
    fn test_active_features_sorted() {
        let mut fs = serde_feature_set();
        fs.enable("json").unwrap();
        let active = fs.active_features();
        // Should be sorted
        let mut sorted = active.clone();
        sorted.sort();
        assert_eq!(active, sorted);
    }

    #[test]
    fn test_feature_definition_builder() {
        let feat = FeatureDefinition::new("tls", "TLS support")
            .enables("rustls")
            .enables("tokio-rustls")
            .requires("network");

        assert_eq!(feat.name, "tls");
        assert_eq!(feat.enables.len(), 2);
        assert_eq!(feat.requires.len(), 1);
        assert!(!feat.default);
    }

    #[test]
    fn test_features_toml_builder() {
        let builder = FeaturesTomlBuilder::new()
            .add(FeatureDefinition::new("default", "defaults").as_default())
            .add(
                FeatureDefinition::new("json", "JSON support").enables("serde_json"),
            )
            .add(FeatureDefinition::new("empty", "empty feature"));

        let toml = builder.build();
        assert!(toml.contains("[features]"));
        assert!(toml.contains("default = [\"default\"]"));
        assert!(toml.contains("\"json\" = [\"serde_json\"]"));
        assert!(toml.contains("\"empty\" = []"));
    }

    #[test]
    fn test_features_toml_no_defaults() {
        let builder = FeaturesTomlBuilder::new()
            .add(FeatureDefinition::new("extra", "Extra feature").enables("dep1"));

        let toml = builder.build();
        assert!(!toml.contains("default"));
        assert!(toml.contains("\"extra\" = [\"dep1\"]"));
    }
}
