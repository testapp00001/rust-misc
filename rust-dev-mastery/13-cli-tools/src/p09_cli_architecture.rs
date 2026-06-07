//! # CLI Architecture
//!
//! Well-structured CLI applications separate concerns: argument parsing, command
//! execution, output formatting, and error handling. This lesson covers the
//! command pattern, dependency injection, and building testable CLI architectures.
//!
//! ## Key Concepts
//! - Command pattern for CLI operations
//! - Separation of concerns (parse -> execute -> format)
//! - Dependency injection for testability
//! - Error handling strategies
//! - Output formatting
//! - Exit code conventions

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. Command Trait
// ---------------------------------------------------------------------------

/// The result of executing a command.
#[derive(Debug)]
pub enum CommandResult {
    /// Command succeeded with a message.
    Success(String),
    /// Command succeeded with structured data.
    Data(serde_json::Value),
    /// Command failed with an error.
    Error(CommandError),
}

#[derive(Debug)]
pub struct CommandError {
    pub code: ExitCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl CommandError {
    pub fn new(code: ExitCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ExitCode::NotFound, message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ExitCode::PermissionDenied, message)
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::new(ExitCode::AlreadyExists, message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ExitCode::InvalidInput, message)
    }
}

/// Standard exit codes for CLI applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    InvalidInput = 2,
    NotFound = 3,
    PermissionDenied = 4,
    AlreadyExists = 5,
    Timeout = 6,
    NetworkError = 7,
}

impl ExitCode {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// Trait that all CLI commands must implement.
pub trait Command {
    /// The name of the command.
    fn name(&self) -> &str;

    /// A brief description.
    fn description(&self) -> &str;

    /// Execute the command with the given context.
    fn execute(&self, ctx: &CommandContext) -> CommandResult;
}

// ---------------------------------------------------------------------------
// 2. Command Context (Dependency Injection)
// ---------------------------------------------------------------------------

/// Context passed to commands, providing access to shared resources.
/// Using a trait allows mocking in tests.
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &str) -> Result<String, std::io::Error>;
    fn write(&self, path: &str, contents: &str) -> Result<(), std::io::Error>;
    fn exists(&self, path: &str) -> bool;
    fn remove(&self, path: &str) -> Result<(), std::io::Error>;
    fn list_dir(&self, path: &str) -> Result<Vec<String>, std::io::Error>;
}

/// Real file system implementation.
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &str) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }

    fn write(&self, path: &str, contents: &str) -> Result<(), std::io::Error> {
        std::fs::write(path, contents)
    }

    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn remove(&self, path: &str) -> Result<(), std::io::Error> {
        std::fs::remove_file(path)
    }

    fn list_dir(&self, path: &str) -> Result<Vec<String>, std::io::Error> {
        let entries = std::fs::read_dir(path)?;
        Ok(entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }
}

/// In-memory file system for testing.
#[derive(Debug, Default)]
pub struct MockFileSystem {
    pub files: HashMap<String, String>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(path.into(), contents.into());
        self
    }
}

impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &str) -> Result<String, std::io::Error> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
    }

    fn write(&self, _path: &str, _contents: &str) -> Result<(), std::io::Error> {
        // Would need &mut self in real impl; simplified for learning
        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn remove(&self, _path: &str) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn list_dir(&self, path: &str) -> Result<Vec<String>, std::io::Error> {
        Ok(self
            .files
            .keys()
            .filter(|k| k.starts_with(path))
            .cloned()
            .collect())
    }
}

/// The command execution context.
pub struct CommandContext {
    pub fs: Box<dyn FileSystem>,
    pub verbose: bool,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl CommandContext {
    pub fn new(fs: impl FileSystem + 'static) -> Self {
        Self {
            fs: Box::new(fs),
            verbose: false,
            format: OutputFormat::Text,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }
}

// ---------------------------------------------------------------------------
// 3. Command Registry
// ---------------------------------------------------------------------------

/// Registry of available commands.
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(mut self, command: impl Command + 'static) -> Self {
        self.commands
            .insert(command.name().to_string(), Box::new(command));
        self
    }

    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|c| c.as_ref())
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.commands
            .values()
            .map(|c| (c.name(), c.description()))
            .collect()
    }

    pub fn has_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

// ---------------------------------------------------------------------------
// 4. Output Formatting
// ---------------------------------------------------------------------------

/// Format command results for display.
pub fn format_result(result: &CommandResult, format: &OutputFormat) -> (String, i32) {
    match result {
        CommandResult::Success(msg) => (msg.clone(), 0),
        CommandResult::Data(data) => {
            let output = match format {
                OutputFormat::Json => serde_json::to_string_pretty(data).unwrap_or_default(),
                OutputFormat::Text => format!("{data}"),
            };
            (output, 0)
        }
        CommandResult::Error(err) => {
            let output = format!("error: {}", err.message);
            (output, err.code.as_i32())
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Example Commands
// ---------------------------------------------------------------------------

/// A command that reads and displays a file.
pub struct CatCommand {
    pub path: String,
}

impl Command for CatCommand {
    fn name(&self) -> &str {
        "cat"
    }

    fn description(&self) -> &str {
        "Display file contents"
    }

    fn execute(&self, ctx: &CommandContext) -> CommandResult {
        match ctx.fs.read_to_string(&self.path) {
            Ok(contents) => CommandResult::Success(contents),
            Err(e) => CommandResult::Error(CommandError::new(
                ExitCode::NotFound,
                format!("cannot read '{}': {e}", self.path),
            )),
        }
    }
}

/// A command that counts lines in a file.
pub struct LineCountCommand {
    pub path: String,
}

impl Command for LineCountCommand {
    fn name(&self) -> &str {
        "linecount"
    }

    fn description(&self) -> &str {
        "Count lines in a file"
    }

    fn execute(&self, ctx: &CommandContext) -> CommandResult {
        match ctx.fs.read_to_string(&self.path) {
            Ok(contents) => {
                let count = contents.lines().count();
                CommandResult::Data(serde_json::json!({
                    "path": self.path,
                    "lines": count,
                }))
            }
            Err(e) => CommandResult::Error(CommandError::new(
                ExitCode::NotFound,
                format!("cannot read '{}': {e}", self.path),
            )),
        }
    }
}

/// A command that checks if a file exists.
pub struct ExistsCommand {
    pub path: String,
}

impl Command for ExistsCommand {
    fn name(&self) -> &str {
        "exists"
    }

    fn description(&self) -> &str {
        "Check if a file exists"
    }

    fn execute(&self, ctx: &CommandContext) -> CommandResult {
        let exists = ctx.fs.exists(&self.path);
        CommandResult::Data(serde_json::json!({
            "path": self.path,
            "exists": exists,
        }))
    }
}

// ---------------------------------------------------------------------------
// 6. Pipeline (chaining commands)
// ---------------------------------------------------------------------------

/// A pipeline that chains multiple commands together.
pub struct Pipeline {
    steps: Vec<Box<dyn Command>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn pipe(mut self, command: impl Command + 'static) -> Self {
        self.steps.push(Box::new(command));
        self
    }

    pub fn execute(&self, ctx: &CommandContext) -> Vec<CommandResult> {
        self.steps.iter().map(|cmd| cmd.execute(ctx)).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_ctx() -> CommandContext {
        CommandContext::new(
            MockFileSystem::new()
                .with_file("/test.txt", "hello\nworld\nfoo")
                .with_file("/empty.txt", ""),
        )
    }

    #[test]
    fn test_cat_command_success() {
        let ctx = mock_ctx();
        let cmd = CatCommand {
            path: "/test.txt".into(),
        };
        let result = cmd.execute(&ctx);
        match result {
            CommandResult::Success(s) => assert_eq!(s, "hello\nworld\nfoo"),
            _ => panic!("expected success"),
        }
    }

    #[test]
    fn test_cat_command_not_found() {
        let ctx = mock_ctx();
        let cmd = CatCommand {
            path: "/nonexistent".into(),
        };
        let result = cmd.execute(&ctx);
        match result {
            CommandResult::Error(err) => {
                assert_eq!(err.code, ExitCode::NotFound);
            }
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn test_linecount_command() {
        let ctx = mock_ctx();
        let cmd = LineCountCommand {
            path: "/test.txt".into(),
        };
        let result = cmd.execute(&ctx);
        match result {
            CommandResult::Data(data) => {
                assert_eq!(data["lines"], 3);
                assert_eq!(data["path"], "/test.txt");
            }
            _ => panic!("expected data"),
        }
    }

    #[test]
    fn test_linecount_empty_file() {
        let ctx = mock_ctx();
        let cmd = LineCountCommand {
            path: "/empty.txt".into(),
        };
        let result = cmd.execute(&ctx);
        match result {
            CommandResult::Data(data) => {
                assert_eq!(data["lines"], 0);
            }
            _ => panic!("expected data"),
        }
    }

    #[test]
    fn test_exists_command() {
        let ctx = mock_ctx();

        let cmd = ExistsCommand {
            path: "/test.txt".into(),
        };
        match cmd.execute(&ctx) {
            CommandResult::Data(data) => assert_eq!(data["exists"], true),
            _ => panic!("expected data"),
        }

        let cmd = ExistsCommand {
            path: "/nonexistent".into(),
        };
        match cmd.execute(&ctx) {
            CommandResult::Data(data) => assert_eq!(data["exists"], false),
            _ => panic!("expected data"),
        }
    }

    #[test]
    fn test_command_name_and_description() {
        let cmd = CatCommand {
            path: "test".into(),
        };
        assert_eq!(cmd.name(), "cat");
        assert!(!cmd.description().is_empty());
    }

    #[test]
    fn test_command_registry() {
        let registry = CommandRegistry::new()
            .register(CatCommand {
                path: String::new(),
            })
            .register(LineCountCommand {
                path: String::new(),
            });

        assert!(registry.has_command("cat"));
        assert!(registry.has_command("linecount"));
        assert!(!registry.has_command("nonexistent"));

        let commands = registry.list_commands();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_command_registry_get() {
        let registry = CommandRegistry::new().register(CatCommand {
            path: String::new(),
        });

        let cmd = registry.get("cat");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name(), "cat");

        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn test_format_result_success() {
        let result = CommandResult::Success("ok".into());
        let (output, code) = format_result(&result, &OutputFormat::Text);
        assert_eq!(output, "ok");
        assert_eq!(code, 0);
    }

    #[test]
    fn test_format_result_data_json() {
        let data = serde_json::json!({"key": "value"});
        let result = CommandResult::Data(data);
        let (output, code) = format_result(&result, &OutputFormat::Json);
        assert!(output.contains("key"));
        assert_eq!(code, 0);
    }

    #[test]
    fn test_format_result_error() {
        let result = CommandResult::Error(CommandError::new(ExitCode::NotFound, "not found"));
        let (output, code) = format_result(&result, &OutputFormat::Text);
        assert!(output.contains("not found"));
        assert_eq!(code, 3);
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::GeneralError.as_i32(), 1);
        assert_eq!(ExitCode::InvalidInput.as_i32(), 2);
        assert_eq!(ExitCode::NotFound.as_i32(), 3);
        assert_eq!(ExitCode::PermissionDenied.as_i32(), 4);
    }

    #[test]
    fn test_pipeline() {
        let ctx = mock_ctx();
        let pipeline = Pipeline::new()
            .pipe(CatCommand {
                path: "/test.txt".into(),
            })
            .pipe(ExistsCommand {
                path: "/test.txt".into(),
            });

        let results = pipeline.execute(&ctx);
        assert_eq!(results.len(), 2);

        match &results[0] {
            CommandResult::Success(_) => {}
            _ => panic!("expected success for cat"),
        }

        match &results[1] {
            CommandResult::Data(data) => assert_eq!(data["exists"], true),
            _ => panic!("expected data for exists"),
        }
    }

    #[test]
    fn test_mock_fs_with_file() {
        let fs = MockFileSystem::new().with_file("/a.txt", "content");
        assert!(fs.exists("/a.txt"));
        assert!(!fs.exists("/b.txt"));
        assert_eq!(fs.read_to_string("/a.txt").unwrap(), "content");
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::not_found("file missing");
        assert_eq!(err.to_string(), "file missing");
        assert_eq!(err.code, ExitCode::NotFound);
    }

    #[test]
    fn test_context_verbose() {
        let ctx = mock_ctx().with_verbose(true);
        assert!(ctx.verbose);
    }

    #[test]
    fn test_context_format() {
        let ctx = mock_ctx().with_format(OutputFormat::Json);
        assert_eq!(ctx.format, OutputFormat::Json);
    }
}
