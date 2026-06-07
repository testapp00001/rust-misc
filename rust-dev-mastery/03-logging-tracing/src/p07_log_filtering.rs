//! # Lesson 7: Log Filtering
//!
//! Filtering controls which log messages are emitted. This lesson covers
//! env-filter directives, per-module filtering, dynamic filtering, and
//! filter reload patterns.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Filter directive model
// ---------------------------------------------------------------------------

/// A filter directive like "mycrate::module=debug".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterDirective {
    pub target: Option<String>,
    pub level: FilterLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum FilterLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl FilterLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" => Some(FilterLevel::Off),
            "error" => Some(FilterLevel::Error),
            "warn" | "warning" => Some(FilterLevel::Warn),
            "info" => Some(FilterLevel::Info),
            "debug" => Some(FilterLevel::Debug),
            "trace" => Some(FilterLevel::Trace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FilterLevel::Off => "off",
            FilterLevel::Error => "error",
            FilterLevel::Warn => "warn",
            FilterLevel::Info => "info",
            FilterLevel::Debug => "debug",
            FilterLevel::Trace => "trace",
        }
    }
}

impl FilterDirective {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() == 2 {
            let level = FilterLevel::from_str(parts[1])?;
            Some(Self {
                target: Some(parts[0].to_string()),
                level,
            })
        } else {
            let level = FilterLevel::from_str(s)?;
            Some(Self {
                target: None,
                level,
            })
        }
    }

    pub fn matches(&self, target: &str) -> bool {
        match &self.target {
            Some(t) => target.starts_with(t),
            None => true, // global directive
        }
    }
}

// ---------------------------------------------------------------------------
// Filter configuration
// ---------------------------------------------------------------------------

/// A complete filter configuration with directives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub directives: Vec<FilterDirective>,
    pub global_level: FilterLevel,
}

impl FilterConfig {
    pub fn new(global_level: FilterLevel) -> Self {
        Self {
            directives: Vec::new(),
            global_level,
        }
    }

    pub fn add_directive(&mut self, directive: FilterDirective) {
        self.directives.push(directive);
    }

    /// Parse a filter string like "info,mycrate=debug,hyper=warn".
    pub fn parse(filter_str: &str) -> Self {
        let mut config = FilterConfig::new(FilterLevel::Info);

        for part in filter_str.split(',') {
            let part = part.trim();
            if let Some(directive) = FilterDirective::parse(part) {
                if directive.target.is_none() {
                    config.global_level = directive.level;
                } else {
                    config.directives.push(directive);
                }
            }
        }

        config
    }

    /// Check if a log event at the given level and target should be emitted.
    pub fn should_emit(&self, level: FilterLevel, target: &str) -> bool {
        // Check specific directives first (most specific wins)
        for directive in &self.directives {
            if directive.matches(target) {
                return level <= directive.level;
            }
        }
        // Fall back to global level
        level <= self.global_level
    }

    /// Serialize to an env-filter string.
    pub fn to_env_string(&self) -> String {
        let mut parts = vec![self.global_level.as_str().to_string()];
        for directive in &self.directives {
            if let Some(ref target) = directive.target {
                parts.push(format!("{}={}", target, directive.level.as_str()));
            }
        }
        parts.join(",")
    }
}

// ---------------------------------------------------------------------------
// Dynamic filter with reload
// ---------------------------------------------------------------------------

/// A filter that can be reloaded at runtime.
#[derive(Debug)]
pub struct DynamicFilter {
    config: Arc<Mutex<FilterConfig>>,
}

impl DynamicFilter {
    pub fn new(initial: FilterConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(initial)),
        }
    }

    /// Check if an event should be emitted.
    pub fn should_emit(&self, level: FilterLevel, target: &str) -> bool {
        self.config.lock().unwrap().should_emit(level, target)
    }

    /// Reload the filter configuration.
    pub fn reload(&self, new_config: FilterConfig) {
        *self.config.lock().unwrap() = new_config;
    }

    /// Get a snapshot of the current configuration.
    pub fn current_config(&self) -> FilterConfig {
        self.config.lock().unwrap().clone()
    }

    /// Create a handle for reloading from another thread.
    pub fn reload_handle(&self) -> FilterReloadHandle {
        FilterReloadHandle {
            config: Arc::clone(&self.config),
        }
    }
}

/// A handle for reloading filters from another thread.
pub struct FilterReloadHandle {
    config: Arc<Mutex<FilterConfig>>,
}

impl FilterReloadHandle {
    pub fn reload(&self, new_config: FilterConfig) {
        *self.config.lock().unwrap() = new_config;
    }
}

// ---------------------------------------------------------------------------
// Filter presets
// ---------------------------------------------------------------------------

/// Common filter presets for different environments.
pub fn dev_filter() -> FilterConfig {
    FilterConfig::parse("debug,hyper=warn,tower=warn,sqlx=warn")
}

pub fn prod_filter() -> FilterConfig {
    FilterConfig::parse("info,mycrate=info")
}

pub fn test_filter() -> FilterConfig {
    FilterConfig::parse("off")
}

pub fn verbose_filter() -> FilterConfig {
    FilterConfig::parse("trace")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_level_ordering() {
        assert!(FilterLevel::Error < FilterLevel::Warn);
        assert!(FilterLevel::Warn < FilterLevel::Info);
        assert!(FilterLevel::Info < FilterLevel::Debug);
        assert!(FilterLevel::Debug < FilterLevel::Trace);
        assert!(FilterLevel::Off < FilterLevel::Error);
    }

    #[test]
    fn test_filter_level_from_str() {
        assert_eq!(FilterLevel::from_str("error"), Some(FilterLevel::Error));
        assert_eq!(FilterLevel::from_str("WARN"), Some(FilterLevel::Warn));
        assert_eq!(FilterLevel::from_str("warning"), Some(FilterLevel::Warn));
        assert_eq!(FilterLevel::from_str("info"), Some(FilterLevel::Info));
        assert_eq!(FilterLevel::from_str("debug"), Some(FilterLevel::Debug));
        assert_eq!(FilterLevel::from_str("trace"), Some(FilterLevel::Trace));
        assert_eq!(FilterLevel::from_str("off"), Some(FilterLevel::Off));
        assert_eq!(FilterLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_filter_level_as_str() {
        assert_eq!(FilterLevel::Error.as_str(), "error");
        assert_eq!(FilterLevel::Trace.as_str(), "trace");
    }

    #[test]
    fn test_filter_directive_parse() {
        let d = FilterDirective::parse("mycrate=debug").unwrap();
        assert_eq!(d.target, Some("mycrate".to_string()));
        assert_eq!(d.level, FilterLevel::Debug);
    }

    #[test]
    fn test_filter_directive_parse_global() {
        let d = FilterDirective::parse("info").unwrap();
        assert!(d.target.is_none());
        assert_eq!(d.level, FilterLevel::Info);
    }

    #[test]
    fn test_filter_directive_parse_invalid() {
        assert!(FilterDirective::parse("invalid").is_none());
    }

    #[test]
    fn test_filter_directive_matches() {
        let d = FilterDirective {
            target: Some("mycrate".to_string()),
            level: FilterLevel::Debug,
        };
        assert!(d.matches("mycrate"));
        assert!(d.matches("mycrate::module"));
        assert!(!d.matches("other"));
    }

    #[test]
    fn test_filter_directive_matches_global() {
        let d = FilterDirective {
            target: None,
            level: FilterLevel::Info,
        };
        assert!(d.matches("anything"));
    }

    #[test]
    fn test_filter_config_should_emit() {
        let config = FilterConfig::parse("info,mycrate=debug");
        assert!(config.should_emit(FilterLevel::Info, "other"));
        assert!(config.should_emit(FilterLevel::Debug, "mycrate"));
        assert!(!config.should_emit(FilterLevel::Trace, "mycrate"));
        assert!(!config.should_emit(FilterLevel::Debug, "other"));
    }

    #[test]
    fn test_filter_config_parse() {
        let config = FilterConfig::parse("warn,mycrate=debug,sqlx=error");
        assert_eq!(config.global_level, FilterLevel::Warn);
        assert_eq!(config.directives.len(), 2);
    }

    #[test]
    fn test_filter_config_to_env_string() {
        let config = FilterConfig::parse("info,mycrate=debug");
        let env = config.to_env_string();
        assert!(env.contains("info"));
        assert!(env.contains("mycrate=debug"));
    }

    #[test]
    fn test_dynamic_filter() {
        let config = FilterConfig::parse("info");
        let filter = DynamicFilter::new(config);

        assert!(filter.should_emit(FilterLevel::Info, "test"));
        assert!(!filter.should_emit(FilterLevel::Debug, "test"));

        // Reload with more permissive filter
        let new_config = FilterConfig::parse("debug");
        filter.reload(new_config);

        assert!(filter.should_emit(FilterLevel::Debug, "test"));
    }

    #[test]
    fn test_dynamic_filter_reload_handle() {
        let config = FilterConfig::parse("info");
        let filter = DynamicFilter::new(config);
        let handle = filter.reload_handle();

        handle.reload(FilterConfig::parse("trace"));
        assert!(filter.should_emit(FilterLevel::Trace, "test"));
    }

    #[test]
    fn test_dynamic_filter_current_config() {
        let config = FilterConfig::parse("warn");
        let filter = DynamicFilter::new(config);
        let current = filter.current_config();
        assert_eq!(current.global_level, FilterLevel::Warn);
    }

    #[test]
    fn test_dev_filter() {
        let filter = dev_filter();
        assert!(filter.should_emit(FilterLevel::Debug, "mycrate"));
        assert!(!filter.should_emit(FilterLevel::Debug, "hyper"));
    }

    #[test]
    fn test_prod_filter() {
        let filter = prod_filter();
        assert!(filter.should_emit(FilterLevel::Info, "mycrate"));
        assert!(!filter.should_emit(FilterLevel::Debug, "mycrate"));
    }

    #[test]
    fn test_test_filter() {
        let filter = test_filter();
        assert!(!filter.should_emit(FilterLevel::Error, "anything"));
    }

    #[test]
    fn test_verbose_filter() {
        let filter = verbose_filter();
        assert!(filter.should_emit(FilterLevel::Trace, "anything"));
    }
}
