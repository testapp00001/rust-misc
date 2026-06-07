//! # Clap Mastery
//!
//! `clap` is the most popular CLI argument parser in Rust. This lesson covers
//! the derive API for building complex CLIs with subcommands, typed arguments,
//! validation, and custom help text.
//!
//! ## Key Concepts
//! - Derive API with `#[derive(Parser)]`
//! - Subcommands with `#[derive(Subcommand)]`
//! - Positional arguments, flags, and options
//! - Value validation
//! - Custom help templates
//! - Environment variable fallbacks

use clap::{Parser, Subcommand, ValueEnum, Args};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// 1. Top-Level CLI Definition
// ---------------------------------------------------------------------------

/// A file management CLI tool.
/// Demonstrates a realistic multi-subcommand CLI application.
#[derive(Debug, Parser)]
#[command(
    name = "fileman",
    about = "A modern file management tool",
    version = "0.1.0",
    author = "Rust Learner",
    long_about = "fileman is a demonstration CLI tool that shows how to build\n\
                  production-quality command-line applications with clap."
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(short, long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List files in a directory
    List(ListArgs),

    /// Copy a file or directory
    Copy(CopyArgs),

    /// Search for files matching a pattern
    Search(SearchArgs),

    /// Show file information
    Info(InfoArgs),

    /// Manage configuration
    Config(ConfigArgs),
}

/// Arguments for the `list` subcommand.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Directory to list
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Show hidden files
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Sort order
    #[arg(short, long, default_value = "name")]
    pub sort: SortField,

    /// Maximum depth to recurse
    #[arg(short, long)]
    pub depth: Option<u32>,

    /// Filter by file extension
    #[arg(short, long)]
    pub extension: Option<String>,

    /// Show file sizes in human-readable format
    #[arg(short = 'H', long)]
    pub human_readable: bool,
}

/// Arguments for the `copy` subcommand.
#[derive(Debug, Args)]
pub struct CopyArgs {
    /// Source path
    pub source: PathBuf,

    /// Destination path
    pub destination: PathBuf,

    /// Overwrite existing files
    #[arg(short = 'F', long)]
    pub force: bool,

    /// Copy recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Preserve file permissions
    #[arg(short, long)]
    pub preserve: bool,

    /// Dry run (show what would be copied)
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `search` subcommand.
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Search pattern (glob or regex)
    pub pattern: String,

    /// Directory to search in
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Use regex instead of glob
    #[arg(short, long)]
    pub regex: bool,

    /// Case-insensitive search
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Maximum results
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Search file contents
    #[arg(long)]
    pub content: bool,
}

/// Arguments for the `info` subcommand.
#[derive(Debug, Args)]
pub struct InfoArgs {
    /// File path
    pub path: PathBuf,

    /// Show checksum
    #[arg(long)]
    pub checksum: bool,

    /// Checksum algorithm
    #[arg(long, value_enum, default_value = "sha256")]
    pub algorithm: ChecksumAlgorithm,
}

/// Arguments for the `config` subcommand.
#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
    /// List all config values
    List,
    /// Reset config to defaults
    Reset {
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// 2. Value Enums
// ---------------------------------------------------------------------------

/// Output format options.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Table,
}

/// Sort field options for file listing.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum SortField {
    Name,
    Size,
    Modified,
    Extension,
}

/// Checksum algorithm options.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ChecksumAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

// ---------------------------------------------------------------------------
// 3. Validation Helpers
// ---------------------------------------------------------------------------

/// Validate that a path exists.
pub fn validate_path_exists(path: &PathBuf) -> Result<PathBuf, String> {
    if path.exists() {
        Ok(path.clone())
    } else {
        Err(format!("path does not exist: {}", path.display()))
    }
}

/// Validate that a value is within a range.
pub fn validate_range(value: usize, min: usize, max: usize, name: &str) -> Result<usize, String> {
    if value < min || value > max {
        Err(format!("{name} must be between {min} and {max}, got {value}"))
    } else {
        Ok(value)
    }
}

/// Validate a glob pattern.
pub fn validate_glob(pattern: &str) -> Result<String, String> {
    if pattern.is_empty() {
        Err("pattern cannot be empty".into())
    } else if pattern.contains("..") {
        Err("pattern cannot contain '..'".into())
    } else {
        Ok(pattern.to_string())
    }
}

// ---------------------------------------------------------------------------
// 4. CLI Execution Context
// ---------------------------------------------------------------------------

/// Context available to all subcommand handlers.
#[derive(Debug)]
pub struct CommandContext {
    pub verbose: bool,
    pub format: OutputFormat,
}

impl From<&Cli> for CommandContext {
    fn from(cli: &Cli) -> Self {
        Self {
            verbose: cli.verbose,
            format: cli.format.clone(),
        }
    }
}

impl CommandContext {
    pub fn log(&self, message: &str) {
        if self.verbose {
            eprintln!("[verbose] {message}");
        }
    }

    pub fn format_output<T: std::fmt::Debug + serde::Serialize>(&self, data: &T) -> String {
        match self.format {
            OutputFormat::Text => format!("{data:?}"),
            OutputFormat::Json => serde_json::to_string_pretty(data).unwrap_or_default(),
            OutputFormat::Csv => "csv output not implemented".into(),
            OutputFormat::Table => format!("| {data:?} |"),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Help Generation Utilities
// ---------------------------------------------------------------------------

/// Generate usage examples for a subcommand.
pub fn usage_examples(subcommand: &str) -> &'static str {
    match subcommand {
        "list" => {
            "Examples:\n  \
               fileman list                    # List current directory\n  \
               fileman list /tmp -a            # List /tmp with hidden files\n  \
               fileman list -s size -H         # Sort by size, human-readable\n  \
               fileman list --depth 2 -e rs    # Rust files, max depth 2"
        }
        "search" => {
            "Examples:\n  \
               fileman search '*.rs'           # Find all .rs files\n  \
               fileman search -i 'readme'      # Case-insensitive search\n  \
               fileman search -r 'test_\\w+'    # Regex search"
        }
        "copy" => {
            "Examples:\n  \
               fileman copy src/ dest/ -r      # Copy directory recursively\n  \
               fileman copy file.txt /tmp/ -f  # Force overwrite\n  \
               fileman copy big.bin /dst/ --dry-run"
        }
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_list_default() {
        let cli = Cli::try_parse_from(["fileman", "list"]).unwrap();
        assert!(!cli.verbose);
        assert_eq!(cli.format, OutputFormat::Text);
        match cli.command {
            Commands::List(args) => {
                assert_eq!(args.path, PathBuf::from("."));
                assert!(!args.all);
                assert_eq!(args.sort, SortField::Name);
                assert!(args.depth.is_none());
            }
            _ => panic!("expected List command"),
        }
    }

    #[test]
    fn test_cli_parse_list_with_flags() {
        let cli = Cli::try_parse_from([
            "fileman", "-v", "-f", "json", "list", "-a", "-s", "size",
            "--depth", "3", "-e", "rs", "-H",
        ])
        .unwrap();

        assert!(cli.verbose);
        assert_eq!(cli.format, OutputFormat::Json);
        match cli.command {
            Commands::List(args) => {
                assert!(args.all);
                assert_eq!(args.sort, SortField::Size);
                assert_eq!(args.depth, Some(3));
                assert_eq!(args.extension.as_deref(), Some("rs"));
                assert!(args.human_readable);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_cli_parse_copy() {
        let cli = Cli::try_parse_from([
            "fileman", "copy", "src.txt", "dest.txt", "-F", "-p",
        ])
        .unwrap();

        match cli.command {
            Commands::Copy(args) => {
                assert_eq!(args.source, PathBuf::from("src.txt"));
                assert_eq!(args.destination, PathBuf::from("dest.txt"));
                assert!(args.force);
                assert!(args.preserve);
                assert!(!args.recursive);
            }
            _ => panic!("expected Copy"),
        }
    }

    #[test]
    fn test_cli_parse_copy_dry_run() {
        let cli = Cli::try_parse_from([
            "fileman", "copy", "a", "b", "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Commands::Copy(args) => assert!(args.dry_run),
            _ => panic!("expected Copy"),
        }
    }

    #[test]
    fn test_cli_parse_search() {
        let cli = Cli::try_parse_from([
            "fileman", "search", "*.rs", "/home", "-r", "-i", "--limit", "50",
        ])
        .unwrap();

        match cli.command {
            Commands::Search(args) => {
                assert_eq!(args.pattern, "*.rs");
                assert_eq!(args.path, PathBuf::from("/home"));
                assert!(args.regex);
                assert!(args.ignore_case);
                assert_eq!(args.limit, 50);
            }
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn test_cli_parse_info() {
        let cli = Cli::try_parse_from([
            "fileman", "info", "myfile.txt", "--checksum", "--algorithm", "sha512",
        ])
        .unwrap();

        match cli.command {
            Commands::Info(args) => {
                assert_eq!(args.path, PathBuf::from("myfile.txt"));
                assert!(args.checksum);
                assert_eq!(args.algorithm, ChecksumAlgorithm::Sha512);
            }
            _ => panic!("expected Info"),
        }
    }

    #[test]
    fn test_cli_parse_config_subcommands() {
        // Config get
        let cli = Cli::try_parse_from(["fileman", "config", "get", "editor"]).unwrap();
        match cli.command {
            Commands::Config(args) => match args.action {
                ConfigAction::Get { key } => assert_eq!(key, "editor"),
                _ => panic!("expected Get"),
            },
            _ => panic!("expected Config"),
        }

        // Config set
        let cli =
            Cli::try_parse_from(["fileman", "config", "set", "editor", "vim"]).unwrap();
        match cli.command {
            Commands::Config(args) => match args.action {
                ConfigAction::Set { key, value } => {
                    assert_eq!(key, "editor");
                    assert_eq!(value, "vim");
                }
                _ => panic!("expected Set"),
            },
            _ => panic!("expected Config"),
        }

        // Config list
        let cli = Cli::try_parse_from(["fileman", "config", "list"]).unwrap();
        match cli.command {
            Commands::Config(args) => match args.action {
                ConfigAction::List => {}
                _ => panic!("expected List"),
            },
            _ => panic!("expected Config"),
        }

        // Config reset
        let cli = Cli::try_parse_from(["fileman", "config", "reset", "-y"]).unwrap();
        match cli.command {
            Commands::Config(args) => match args.action {
                ConfigAction::Reset { yes } => assert!(yes),
                _ => panic!("expected Reset"),
            },
            _ => panic!("expected Config"),
        }
    }

    #[test]
    fn test_output_format_value_enum() {
        assert_eq!(
            OutputFormat::from_str("json", true).unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_str("JSON", true).unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_str("text", true).unwrap(),
            OutputFormat::Text
        );
    }

    #[test]
    fn test_sort_field_value_enum() {
        assert_eq!(
            SortField::from_str("name", true).unwrap(),
            SortField::Name
        );
        assert_eq!(
            SortField::from_str("size", true).unwrap(),
            SortField::Size
        );
        assert_eq!(
            SortField::from_str("modified", true).unwrap(),
            SortField::Modified
        );
    }

    #[test]
    fn test_checksum_algorithm_value_enum() {
        assert_eq!(
            ChecksumAlgorithm::from_str("md5", true).unwrap(),
            ChecksumAlgorithm::Md5
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("sha256", true).unwrap(),
            ChecksumAlgorithm::Sha256
        );
    }

    #[test]
    fn test_validate_path_exists() {
        // Current directory should exist
        let result = validate_path_exists(&PathBuf::from("."));
        assert!(result.is_ok());

        let result = validate_path_exists(&PathBuf::from("/nonexistent/path/12345"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 1, 10, "val").is_ok());
        assert!(validate_range(0, 1, 10, "val").is_err());
        assert!(validate_range(11, 1, 10, "val").is_err());
    }

    #[test]
    fn test_validate_glob() {
        assert!(validate_glob("*.rs").is_ok());
        assert!(validate_glob("").is_err());
        assert!(validate_glob("../etc/passwd").is_err());
    }

    #[test]
    fn test_command_context() {
        let cli = Cli::try_parse_from(["fileman", "-v", "list"]).unwrap();
        let ctx = CommandContext::from(&cli);
        assert!(ctx.verbose);
        assert_eq!(ctx.format, OutputFormat::Text);
    }

    #[test]
    fn test_usage_examples() {
        assert!(!usage_examples("list").is_empty());
        assert!(!usage_examples("search").is_empty());
        assert!(!usage_examples("copy").is_empty());
        assert!(usage_examples("nonexistent").is_empty());
    }

    #[test]
    fn test_cli_parse_verbose_global() {
        let cli = Cli::try_parse_from(["fileman", "-v", "list"]).unwrap();
        assert!(cli.verbose);

        let cli = Cli::try_parse_from(["fileman", "list"]).unwrap();
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_parse_format_global() {
        let cli = Cli::try_parse_from(["fileman", "-f", "json", "list"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Json);

        let cli = Cli::try_parse_from(["fileman", "-f", "table", "list"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Table);
    }

    #[test]
    fn test_cli_help_does_not_panic() {
        let result = Cli::try_parse_from(["fileman", "--help"]);
        // --help causes an error (exit 0), which is expected
        assert!(result.is_err());
    }
}
