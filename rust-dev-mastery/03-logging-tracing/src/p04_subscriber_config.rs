//! # Lesson 4: Subscriber Configuration
//!
//! Configuring tracing-subscriber is key to getting useful output.
//! This lesson covers the layer architecture, fmt layer, filter layer,
//! env-filter, and how to compose them.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Subscriber configuration model
// ---------------------------------------------------------------------------

/// Models a tracing-subscriber configuration.
/// In real code, you'd use the actual tracing-subscriber API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberConfig {
    pub fmt_layer: FmtLayerConfig,
    pub filter_layer: FilterLayerConfig,
    pub output_format: OutputFormat,
    pub log_file: Option<String>,
    pub include_source_location: bool,
    pub include_span_fields: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmtLayerConfig {
    pub target: bool,
    pub thread_ids: bool,
    pub thread_names: bool,
    pub file: bool,
    pub line_number: bool,
    pub timestamps: bool,
    pub timestamp_format: String,
    pub ansi_colors: bool,
}

impl Default for FmtLayerConfig {
    fn default() -> Self {
        Self {
            target: true,
            thread_ids: false,
            thread_names: false,
            file: false,
            line_number: false,
            timestamps: true,
            timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
            ansi_colors: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterLayerConfig {
    pub global_level: String,
    pub module_overrides: Vec<ModuleFilter>,
    pub env_var: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFilter {
    pub module: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Pretty,
    Compact,
    Json,
    Full,
}

impl Default for FilterLayerConfig {
    fn default() -> Self {
        Self {
            global_level: "info".to_string(),
            module_overrides: Vec::new(),
            env_var: Some("RUST_LOG".to_string()),
        }
    }
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            fmt_layer: FmtLayerConfig::default(),
            filter_layer: FilterLayerConfig::default(),
            output_format: OutputFormat::Full,
            log_file: None,
            include_source_location: false,
            include_span_fields: true,
        }
    }
}

impl SubscriberConfig {
    /// Configuration for development (verbose, colorful).
    pub fn development() -> Self {
        Self {
            fmt_layer: FmtLayerConfig {
                file: true,
                line_number: true,
                thread_names: true,
                ansi_colors: true,
                ..Default::default()
            },
            filter_layer: FilterLayerConfig {
                global_level: "debug".to_string(),
                module_overrides: vec![
                    ModuleFilter {
                        module: "hyper".to_string(),
                        level: "warn".to_string(),
                    },
                    ModuleFilter {
                        module: "tower".to_string(),
                        level: "warn".to_string(),
                    },
                ],
                ..Default::default()
            },
            output_format: OutputFormat::Pretty,
            include_source_location: true,
            include_span_fields: true,
            ..Default::default()
        }
    }

    /// Configuration for production (structured JSON, minimal).
    pub fn production() -> Self {
        Self {
            fmt_layer: FmtLayerConfig {
                target: true,
                thread_ids: true,
                timestamps: true,
                ansi_colors: false,
                ..Default::default()
            },
            filter_layer: FilterLayerConfig {
                global_level: "info".to_string(),
                module_overrides: vec![],
                ..Default::default()
            },
            output_format: OutputFormat::Json,
            include_source_location: false,
            include_span_fields: true,
            ..Default::default()
        }
    }

    /// Configuration for CI (structured but human-readable).
    pub fn ci() -> Self {
        Self {
            fmt_layer: FmtLayerConfig {
                target: true,
                ansi_colors: false,
                timestamps: true,
                ..Default::default()
            },
            filter_layer: FilterLayerConfig {
                global_level: "debug".to_string(),
                ..Default::default()
            },
            output_format: OutputFormat::Compact,
            include_source_location: true,
            include_span_fields: false,
            ..Default::default()
        }
    }

    /// Generate the env-filter directive string.
    pub fn env_filter_directive(&self) -> String {
        let mut directive = self.filter_layer.global_level.clone();
        for mo in &self.filter_layer.module_overrides {
            directive.push_str(&format!(",{}={}", mo.module, mo.level));
        }
        directive
    }
}

// ---------------------------------------------------------------------------
// Layer composition patterns
// ---------------------------------------------------------------------------

/// Describes a composed subscriber with multiple layers.
pub struct LayeredSubscriber {
    pub layers: Vec<LayerConfig>,
}

#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub name: String,
    pub purpose: String,
    pub order: usize,
}

impl LayeredSubscriber {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add_layer(&mut self, name: &str, purpose: &str, order: usize) {
        self.layers.push(LayerConfig {
            name: name.to_string(),
            purpose: purpose.to_string(),
            order,
        });
    }

    /// Create a standard production subscriber stack.
    pub fn production_stack() -> Self {
        let mut sub = Self::new();
        sub.add_layer("EnvFilter", "Filter by log level and module", 0);
        sub.add_layer("fmt::Layer", "Format and output log messages", 1);
        sub.add_layer("FlameLayer", "Record span timings for flamegraphs", 2);
        sub.add_layer("OpenTelemetry", "Export spans to tracing backend", 3);
        sub
    }

    pub fn layer_names(&self) -> Vec<String> {
        let mut sorted = self.layers.clone();
        sorted.sort_by_key(|l| l.order);
        sorted.iter().map(|l| l.name.clone()).collect()
    }
}

impl Default for LayeredSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SubscriberConfig::default();
        assert_eq!(config.output_format, OutputFormat::Full);
        assert!(config.fmt_layer.target);
        assert!(config.fmt_layer.timestamps);
        assert_eq!(config.filter_layer.global_level, "info");
    }

    #[test]
    fn test_development_config() {
        let config = SubscriberConfig::development();
        assert_eq!(config.output_format, OutputFormat::Pretty);
        assert!(config.fmt_layer.file);
        assert!(config.fmt_layer.line_number);
        assert!(config.include_source_location);
        assert_eq!(config.filter_layer.global_level, "debug");
    }

    #[test]
    fn test_production_config() {
        let config = SubscriberConfig::production();
        assert_eq!(config.output_format, OutputFormat::Json);
        assert!(!config.fmt_layer.ansi_colors);
        assert!(config.fmt_layer.thread_ids);
        assert!(!config.include_source_location);
    }

    #[test]
    fn test_ci_config() {
        let config = SubscriberConfig::ci();
        assert_eq!(config.output_format, OutputFormat::Compact);
        assert!(!config.fmt_layer.ansi_colors);
        assert!(config.include_source_location);
    }

    #[test]
    fn test_env_filter_directive() {
        let config = SubscriberConfig::development();
        let directive = config.env_filter_directive();
        assert!(directive.starts_with("debug"));
        assert!(directive.contains("hyper=warn"));
        assert!(directive.contains("tower=warn"));
    }

    #[test]
    fn test_env_filter_directive_default() {
        let config = SubscriberConfig::default();
        let directive = config.env_filter_directive();
        assert_eq!(directive, "info");
    }

    #[test]
    fn test_fmt_layer_config_defaults() {
        let fmt = FmtLayerConfig::default();
        assert!(fmt.target);
        assert!(!fmt.thread_ids);
        assert!(!fmt.thread_names);
        assert!(!fmt.file);
        assert!(!fmt.line_number);
        assert!(fmt.timestamps);
        assert!(fmt.ansi_colors);
    }

    #[test]
    fn test_filter_layer_config_defaults() {
        let filter = FilterLayerConfig::default();
        assert_eq!(filter.global_level, "info");
        assert!(filter.module_overrides.is_empty());
        assert_eq!(filter.env_var, Some("RUST_LOG".to_string()));
    }

    #[test]
    fn test_output_format_variants() {
        assert_ne!(OutputFormat::Pretty, OutputFormat::Json);
        assert_ne!(OutputFormat::Compact, OutputFormat::Full);
    }

    #[test]
    fn test_layered_subscriber() {
        let mut sub = LayeredSubscriber::new();
        sub.add_layer("filter", "filter events", 0);
        sub.add_layer("fmt", "format output", 1);

        assert_eq!(sub.layers.len(), 2);
        let names = sub.layer_names();
        assert_eq!(names, vec!["filter", "fmt"]);
    }

    #[test]
    fn test_production_stack() {
        let sub = LayeredSubscriber::production_stack();
        assert_eq!(sub.layers.len(), 4);
        let names = sub.layer_names();
        assert_eq!(names[0], "EnvFilter");
        assert_eq!(names[1], "fmt::Layer");
        assert_eq!(names[2], "FlameLayer");
        assert_eq!(names[3], "OpenTelemetry");
    }

    #[test]
    fn test_layer_ordering() {
        let mut sub = LayeredSubscriber::new();
        sub.add_layer("third", "c", 2);
        sub.add_layer("first", "a", 0);
        sub.add_layer("second", "b", 1);

        let names = sub.layer_names();
        assert_eq!(names, vec!["first", "second", "third"]);
    }

    #[test]
    fn test_subscriber_config_serialization() {
        let config = SubscriberConfig::production();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SubscriberConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

}
