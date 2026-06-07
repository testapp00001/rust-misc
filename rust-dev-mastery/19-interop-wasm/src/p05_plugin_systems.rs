//! # Plugin Systems
//!
//! Plugin systems allow extending applications at runtime by loading
//! dynamically-linked libraries. This module covers safe plugin interfaces,
//! dynamic loading patterns, and versioned plugin APIs.
//!
//! ## Key Concepts:
//!
//! - **Plugin Trait**: Common interface all plugins implement
//! - **Dynamic Loading**: Load `.so`/`.dylib`/`.dll` at runtime
//! - **Version Negotiation**: Ensure plugin compatibility
//! - **Safety**: Isolate plugin failures from host application

use std::collections::HashMap;

/// Plugin metadata for discovery and versioning.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub author: String,
    pub description: String,
}

impl PluginMetadata {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            api_version: 1,
            author: String::new(),
            description: String::new(),
        }
    }

    pub fn with_api_version(mut self, version: u32) -> Self {
        self.api_version = version;
        self
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.into();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.into();
        self
    }

    /// Check if this plugin is compatible with the host API version.
    pub fn is_compatible(&self, host_api_version: u32) -> bool {
        self.api_version == host_api_version
    }
}

/// Trait that all plugins must implement.
pub trait Plugin: Send {
    /// Get plugin metadata.
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin.
    fn initialize(&mut self) -> Result<(), PluginError>;

    /// Shutdown the plugin.
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Handle a command/request.
    fn handle_command(&self, command: &str, args: &[&str]) -> Result<String, PluginError>;

    /// Get the plugin's capabilities.
    fn capabilities(&self) -> Vec<String>;
}

/// Plugin error type.
#[derive(Debug, Clone)]
pub enum PluginError {
    InitializationFailed(String),
    CommandFailed(String),
    VersionMismatch { expected: u32, actual: u32 },
    InternalError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Init failed: {}", msg),
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::VersionMismatch { expected, actual } => {
                write!(f, "Version mismatch: expected {}, got {}", expected, actual)
            }
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

/// Plugin registry that manages loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    api_version: u32,
}

impl PluginRegistry {
    pub fn new(api_version: u32) -> Self {
        Self {
            plugins: HashMap::new(),
            api_version,
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let metadata = plugin.metadata().clone();

        if !metadata.is_compatible(self.api_version) {
            return Err(PluginError::VersionMismatch {
                expected: self.api_version,
                actual: metadata.api_version,
            });
        }

        let name = metadata.name.clone();
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Initialize all registered plugins.
    pub fn initialize_all(&mut self) -> Result<(), PluginError> {
        for (name, plugin) in &mut self.plugins {
            plugin
                .initialize()
                .map_err(|e| PluginError::InitializationFailed(format!("{}: {}", name, e)))?;
        }
        Ok(())
    }

    /// Shutdown all plugins.
    pub fn shutdown_all(&mut self) -> Result<(), PluginError> {
        for (name, plugin) in &mut self.plugins {
            plugin
                .shutdown()
                .map_err(|e| PluginError::InternalError(format!("{}: {}", name, e)))?;
        }
        Ok(())
    }

    /// Dispatch a command to a specific plugin.
    pub fn dispatch(
        &self,
        plugin_name: &str,
        command: &str,
        args: &[&str],
    ) -> Result<String, PluginError> {
        let plugin = self
            .plugins
            .get(plugin_name)
            .ok_or_else(|| PluginError::CommandFailed(format!("Plugin '{}' not found", plugin_name)))?;
        plugin.handle_command(command, args)
    }

    /// List all registered plugins.
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

/// Plugin factory for creating plugin instances.
pub trait PluginFactory: Send {
    fn create(&self) -> Box<dyn Plugin>;
    fn metadata(&self) -> &PluginMetadata;
}

/// Plugin loader that simulates dynamic library loading.
pub struct PluginLoader {
    factories: HashMap<String, Box<dyn PluginFactory>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register_factory(&mut self, name: &str, factory: Box<dyn PluginFactory>) {
        self.factories.insert(name.into(), factory);
    }

    pub fn load_plugin(&self, name: &str) -> Option<Box<dyn Plugin>> {
        self.factories.get(name).map(|f| f.create())
    }

    pub fn available_plugins(&self) -> Vec<&PluginMetadata> {
        self.factories.values().map(|f| f.metadata()).collect()
    }
}

/// Example plugin: Echo plugin.
pub struct EchoPlugin {
    metadata: PluginMetadata,
    initialized: bool,
}

impl EchoPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("echo", "1.0.0")
                .with_author("Rust DSA")
                .with_description("Echoes back the input"),
            initialized: false,
        }
    }
}

impl Plugin for EchoPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.initialized = false;
        Ok(())
    }

    fn handle_command(&self, command: &str, args: &[&str]) -> Result<String, PluginError> {
        match command {
            "echo" => Ok(args.join(" ")),
            "repeat" => {
                let count: usize = args
                    .first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                let text = args.get(1).unwrap_or(&"");
                Ok(text.repeat(count))
            }
            _ => Err(PluginError::CommandFailed(format!(
                "Unknown command: {}",
                command
            ))),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["echo".into(), "repeat".into()]
    }
}

/// Plugin sandbox for isolating plugin execution.
pub struct PluginSandbox {
    plugin: Box<dyn Plugin>,
    max_command_length: usize,
    allowed_commands: Vec<String>,
}

impl PluginSandbox {
    pub fn new(plugin: Box<dyn Plugin>, max_command_length: usize) -> Self {
        Self {
            plugin,
            max_command_length,
            allowed_commands: Vec::new(),
        }
    }

    pub fn allow_command(&mut self, command: &str) {
        self.allowed_commands.push(command.into());
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> Result<String, PluginError> {
        if command.len() > self.max_command_length {
            return Err(PluginError::CommandFailed("Command too long".into()));
        }

        if !self.allowed_commands.is_empty() && !self.allowed_commands.contains(&command.to_string()) {
            return Err(PluginError::CommandFailed(format!(
                "Command '{}' not allowed",
                command
            )));
        }

        self.plugin.handle_command(command, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let meta = PluginMetadata::new("test", "1.0.0")
            .with_api_version(2)
            .with_author("author")
            .with_description("A test plugin");

        assert_eq!(meta.name, "test");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.api_version, 2);
        assert!(meta.is_compatible(2));
        assert!(!meta.is_compatible(1));
    }

    #[test]
    fn test_echo_plugin() {
        let mut plugin = EchoPlugin::new();
        assert_eq!(plugin.metadata().name, "echo");

        plugin.initialize().unwrap();
        assert!(plugin.initialized);

        let result = plugin.handle_command("echo", &["hello", "world"]).unwrap();
        assert_eq!(result, "hello world");

        let result = plugin.handle_command("repeat", &["3", "ab"]).unwrap();
        assert_eq!(result, "ababab");
    }

    #[test]
    fn test_echo_plugin_unknown_command() {
        let plugin = EchoPlugin::new();
        let result = plugin.handle_command("unknown", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new(1);
        registry.register(Box::new(EchoPlugin::new())).unwrap();
        assert_eq!(registry.plugin_count(), 1);
        assert!(registry.has_plugin("echo"));
    }

    #[test]
    fn test_plugin_registry_version_mismatch() {
        let mut registry = PluginRegistry::new(2);
        let plugin = EchoPlugin::new(); // API version 1
        let result = registry.register(Box::new(plugin));
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_registry_dispatch() {
        let mut registry = PluginRegistry::new(1);
        registry.register(Box::new(EchoPlugin::new())).unwrap();
        registry.initialize_all().unwrap();

        let result = registry.dispatch("echo", "echo", &["test"]).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_plugin_registry_dispatch_not_found() {
        let registry = PluginRegistry::new(1);
        let result = registry.dispatch("nonexistent", "cmd", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_registry_list() {
        let mut registry = PluginRegistry::new(1);
        registry.register(Box::new(EchoPlugin::new())).unwrap();

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "echo");
    }

    #[test]
    fn test_plugin_sandbox() {
        let plugin = Box::new(EchoPlugin::new());
        let mut sandbox = PluginSandbox::new(plugin, 100);
        sandbox.allow_command("echo");

        let result = sandbox.execute("echo", &["hello"]).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_plugin_sandbox_command_not_allowed() {
        let plugin = Box::new(EchoPlugin::new());
        let mut sandbox = PluginSandbox::new(plugin, 100);
        sandbox.allow_command("echo");

        let result = sandbox.execute("repeat", &["1", "a"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_sandbox_command_too_long() {
        let plugin = Box::new(EchoPlugin::new());
        let sandbox = PluginSandbox::new(plugin, 5);

        let result = sandbox.execute("toolongcommand", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_capabilities() {
        let plugin = EchoPlugin::new();
        let caps = plugin.capabilities();
        assert!(caps.contains(&"echo".to_string()));
        assert!(caps.contains(&"repeat".to_string()));
    }

    #[test]
    fn test_plugin_loader() {
        // PluginLoader requires PluginFactory which is more complex
        let loader = PluginLoader::new();
        assert!(loader.available_plugins().is_empty());
    }

    #[test]
    fn test_plugin_shutdown() {
        let mut plugin = EchoPlugin::new();
        plugin.initialize().unwrap();
        plugin.shutdown().unwrap();
        assert!(!plugin.initialized);
    }
}
