//! # Lesson 06: Path Traversal
//!
//! ## The Problem
//!
//! Path traversal (also called directory traversal) occurs when user input
//! is used to construct file paths without proper validation. Attackers use
//! `../` sequences to escape the intended directory and access arbitrary
//! files on the system.
//!
//! ```ignore
//! // VULNERABLE: Direct path construction
//! let path = format!("/uploads/{}", user_filename);
//! let data = std::fs::read_to_string(&path);
//! // If user_filename is "../../etc/passwd", reads /etc/passwd!
//! ```
//!
//! ## Attack Techniques
//!
//! 1. **Basic traversal**: `../../../etc/passwd`
//! 2. **Encoded traversal**: `..%2F..%2F..%2Fetc%2Fpasswd` (URL-encoded)
//! 3. **Double encoding**: `..%252F..%252F` (double URL-encoded)
//! 4. **Null byte injection**: `../../../etc/passwd\0.png` (older systems)
//! 5. **Windows paths**: `..\..\..\windows\system32\config\sam`
//! 6. **Symlink following**: Create a symlink inside the allowed directory
//!    that points outside it
//!
//! ## Defense Strategy
//!
//! 1. **Canonicalize** the path (resolve `..`, `.`, symlinks)
//! 2. **Verify** the canonical path starts with the intended base directory
//! 3. **Reject** null bytes, backslashes (on non-Windows), and other
//!    suspicious characters
//! 4. **Use allowlists** for file extensions when possible

use std::path::{Path, PathBuf};

/// Validate that a filename is safe for use within a base directory.
///
/// A safe filename:
/// - Is non-empty
/// - Does not contain path separators (/ or \\)
/// - Does not contain `..` sequences
/// - Does not contain null bytes
/// - Does not start with a dot (hidden files)
/// - Contains only alphanumeric chars, dots, hyphens, underscores
///
/// Returns Ok(filename) if safe, Err(message) if dangerous.
pub fn validate_filename(input: &str) -> Result<&str, String> {
    todo!("Validate filename for path safety")
}

/// Build a safe path by joining a base directory with a user-provided filename.
///
/// This function:
/// 1. Validates the filename (using `validate_filename`)
/// 2. Joins it with the base directory
/// 3. Canonicalizes the result (to resolve any `..` that might sneak through)
/// 4. Verifies the canonical path starts with the base directory
///
/// Returns Ok(path) if safe, Err(message) if the path escapes the base.
pub fn safe_path_join(base_dir: &str, filename: &str) -> Result<PathBuf, String> {
    todo!("Build and verify a safe file path")
}

/// Detect path traversal attempts in a filename.
///
/// Checks for:
/// - `..` sequences (in any form: `..`, `../`, `..\\`)
/// - Null bytes
/// - URL-encoded traversal: `%2e%2e`, `%2f`, `%5c`, `%252f`
/// - Absolute paths (starting with `/` or `\\`)
/// - Backslash separators (on any platform, for defense in depth)
///
/// Returns true if traversal patterns are detected.
pub fn detect_path_traversal(input: &str) -> bool {
    todo!("Detect path traversal patterns")
}

/// Sanitize a filename by removing or replacing dangerous characters.
///
/// Rules:
/// - Replace path separators with underscores
/// - Remove `..` sequences
/// - Remove null bytes
/// - Remove leading dots (prevent hidden files)
/// - Truncate to 255 characters
/// - If the result is empty, return "unnamed"
///
/// Returns the sanitized filename.
pub fn sanitize_filename(input: &str) -> String {
    todo!("Sanitize a filename for safe storage")
}

/// A safe file accessor that enforces a base directory.
///
/// All file operations are restricted to files within the base directory.
/// Path traversal attempts are rejected.
pub struct SandboxedFileAccess {
    base_dir: PathBuf,
}

impl SandboxedFileAccess {
    /// Create a new sandboxed file accessor.
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    /// Get the full path for a filename, ensuring it stays within the sandbox.
    ///
    /// Returns Ok(path) if the file is within the sandbox,
    /// Err(message) if the path would escape.
    pub fn get_path(&self, filename: &str) -> Result<PathBuf, String> {
        todo!("Get a validated path within the sandbox")
    }

    /// Check if a file exists within the sandbox.
    pub fn file_exists(&self, filename: &str) -> Result<bool, String> {
        let path = self.get_path(filename)?;
        Ok(path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_filename() {
        assert!(validate_filename("report.pdf").is_ok());
    }

    #[test]
    fn test_reject_traversal() {
        assert!(validate_filename("../../etc/passwd").is_err());
    }

    #[test]
    fn test_reject_null_byte() {
        assert!(validate_filename("file.txt\0.png").is_err());
    }

    #[test]
    fn test_reject_hidden_file() {
        assert!(validate_filename(".bashrc").is_err());
    }

    #[test]
    fn test_detect_traversal_basic() {
        assert!(detect_path_traversal("../../../etc/passwd"));
    }

    #[test]
    fn test_detect_traversal_encoded() {
        assert!(detect_path_traversal("..%2F..%2Fetc%2Fpasswd"));
    }

    #[test]
    fn test_detect_traversal_null_byte() {
        assert!(detect_path_traversal("file.txt\0.png"));
    }

    #[test]
    fn test_detect_clean_filename() {
        assert!(!detect_path_traversal("report_2024.pdf"));
    }

    #[test]
    fn test_sanitize_removes_traversal() {
        let result = sanitize_filename("../../etc/passwd");
        assert!(!result.contains(".."));
    }

    #[test]
    fn test_sanitize_empty_becomes_unnamed() {
        let result = sanitize_filename("...");
        assert_eq!(result, "unnamed");
    }

    #[test]
    fn test_sanitize_truncates_long_name() {
        let long_name = "a".repeat(300);
        let result = sanitize_filename(&long_name);
        assert!(result.len() <= 255);
    }
}
