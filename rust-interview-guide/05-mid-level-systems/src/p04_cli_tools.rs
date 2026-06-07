/// Problem: CLI Tools
///
/// Master building CLI tools in Rust.
///
/// Key Concepts:
/// - Argument parsing
/// - Subcommands
/// - Help messages
/// - Error handling
/// - User input

use std::io::{self, Write};

/// Problem 1: Basic argument parsing
/// Parse command line arguments
pub fn parse_args(args: &[String]) -> Vec<String> {
    args.to_vec()
}

/// Problem 2: Parse named arguments
/// Parse named arguments
pub fn parse_named_args(args: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            let key = args[i].trim_start_matches("--").to_string();
            if i + 1 < args.len() {
                map.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    map
}

/// Problem 3: Parse flags
/// Parse boolean flags
pub fn parse_flags(args: &[String]) -> std::collections::HashMap<String, bool> {
    let mut map = std::collections::HashMap::new();
    for arg in args {
        if arg.starts_with("--") {
            let flag = arg.trim_start_matches("--").to_string();
            map.insert(flag, true);
        }
    }
    map
}

/// Problem 4: Validate arguments
/// Validate arguments
pub fn validate_args(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }
    Ok(())
}

/// Problem 5: Parse subcommand
/// Parse subcommand
pub fn parse_subcommand(args: &[String]) -> Option<String> {
    args.first().cloned()
}

/// Problem 6: Generate help message
/// Generate help message
pub fn generate_help(program: &str, description: &str) -> String {
    format!(
        "{} - {}\n\nUsage: {} [OPTIONS] [ARGS]\n\nOptions:\n  --help  Show this help message\n  --version  Show version",
        program, description, program
    )
}

/// Problem 7: Read user input
/// Read user input
pub fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Problem 8: Confirm action
/// Ask for confirmation
pub fn confirm(prompt: &str) -> bool {
    let input = read_input(&format!("{} (y/n): ", prompt));
    input.to_lowercase() == "y" || input.to_lowercase() == "yes"
}

/// Problem 9: Parse environment variables
/// Read environment variables
pub fn get_env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Problem 10: Set environment variable
/// Set environment variable
pub fn set_env_var(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// Problem 11: Parse config file
/// Parse config from string
pub fn parse_config(content: &str) -> std::collections::HashMap<String, String> {
    let mut config = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            config.insert(
                key.trim().to_string(),
                value.trim().to_string(),
            );
        }
    }
    config
}

/// Problem 12: Format output
/// Format output for display
pub fn format_output(data: &[(&str, &str)]) -> String {
    let mut output = String::new();
    for (key, value) in data {
        output.push_str(&format!("{}: {}\n", key, value));
    }
    output
}

/// Problem 13: Color output (simulated)
/// Add color to output
pub fn colorize(text: &str, color: &str) -> String {
    match color {
        "red" => format!("\x1b[31m{}\x1b[0m", text),
        "green" => format!("\x1b[32m{}\x1b[0m", text),
        "yellow" => format!("\x1b[33m{}\x1b[0m", text),
        "blue" => format!("\x1b[34m{}\x1b[0m", text),
        _ => text.to_string(),
    }
}

/// Problem 14: Progress bar (simulated)
/// Create progress bar
pub fn progress_bar(current: usize, total: usize) -> String {
    let width = 50;
    let progress = (current as f64 / total as f64 * width as f64) as usize;
    let empty = width - progress;
    format!(
        "[{}{}] {}%",
        "=".repeat(progress),
        " ".repeat(empty),
        (current as f64 / total as f64 * 100.0) as usize
    )
}

/// Problem 15: Table output
/// Format as table
pub fn table_output(headers: &[&str], rows: &[Vec<&str>]) -> String {
    let mut output = String::new();

    // Headers
    for header in headers {
        output.push_str(&format!("{:20}", header));
    }
    output.push('\n');

    // Separator
    for _ in headers {
        output.push_str(&format!("{:-<20}", ""));
    }
    output.push('\n');

    // Rows
    for row in rows {
        for cell in row {
            output.push_str(&format!("{:20}", cell));
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        assert_eq!(parse_args(&args), vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_parse_named_args() {
        let args = vec!["--name".to_string(), "Alice".to_string(), "--age".to_string(), "30".to_string()];
        let result = parse_named_args(&args);
        assert_eq!(result.get("name"), Some(&"Alice".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_parse_flags() {
        let args = vec!["--verbose".to_string(), "--debug".to_string()];
        let result = parse_flags(&args);
        assert_eq!(result.get("verbose"), Some(&true));
        assert_eq!(result.get("debug"), Some(&true));
    }

    #[test]
    fn test_validate_args() {
        assert!(validate_args(&["arg".to_string()]).is_ok());
        assert!(validate_args(&[]).is_err());
    }

    #[test]
    fn test_parse_subcommand() {
        let args = vec!["subcommand".to_string(), "arg".to_string()];
        assert_eq!(parse_subcommand(&args), Some("subcommand".to_string()));
    }

    #[test]
    fn test_generate_help() {
        let help = generate_help("myapp", "My application");
        assert!(help.contains("myapp"));
        assert!(help.contains("My application"));
    }

    #[test]
    fn test_parse_config() {
        let config = "key1=value1\nkey2=value2\n# comment";
        let result = parse_config(config);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_format_output() {
        let data = vec![("name", "Alice"), ("age", "30")];
        let output = format_output(&data);
        assert!(output.contains("name: Alice"));
    }

    #[test]
    fn test_colorize() {
        let colored = colorize("hello", "red");
        assert!(colored.contains("\x1b[31m"));
    }

    #[test]
    fn test_progress_bar() {
        let bar = progress_bar(50, 100);
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_table_output() {
        let headers = vec!["Name", "Age"];
        let rows = vec![vec!["Alice", "30"], vec!["Bob", "25"]];
        let output = table_output(&headers, &rows);
        assert!(output.contains("Name"));
    }

    #[test]
    fn test_get_env_var() {
        // This test may not work in all environments
        let _ = get_env_var("PATH");
    }

    #[test]
    fn test_set_env_var() {
        set_env_var("TEST_VAR", "test_value");
        assert_eq!(get_env_var("TEST_VAR"), Some("test_value".to_string()));
    }
}
