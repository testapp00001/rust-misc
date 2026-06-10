//! # Lesson 06: Path Traversal (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::path::{Path, PathBuf};

pub fn validate_filename(input: &str) -> Result<&str, String> {
    if input.is_empty() {
        return Err("Filename must not be empty".to_string());
    }

    if input.contains('\0') {
        return Err("Filename must not contain null bytes".to_string());
    }

    if input.contains('/') || input.contains('\\') {
        return Err("Filename must not contain path separators".to_string());
    }

    if input.contains("..") {
        return Err("Filename must not contain '..' sequences".to_string());
    }

    if input.starts_with('.') {
        return Err("Filename must not start with a dot (hidden file)".to_string());
    }

    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ')
    {
        return Err("Filename contains disallowed characters".to_string());
    }

    Ok(input)
}

pub fn safe_path_join(base_dir: &str, filename: &str) -> Result<PathBuf, String> {
    validate_filename(filename)?;

    let base = Path::new(base_dir);
    let joined = base.join(filename);

    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize base dir: {}", e))?;

    let canonical_path = if joined.exists() {
        joined
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?
    } else {
        // For files that don't exist yet, verify the parent would be within base
        let parent = joined
            .parent()
            .ok_or_else(|| "Cannot determine parent directory".to_string())?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize parent: {}", e))?;
        canonical_parent.join(joined.file_name().unwrap_or_default())
    };

    if !canonical_path.starts_with(&canonical_base) {
        return Err("Path escapes the base directory".to_string());
    }

    Ok(canonical_path)
}

pub fn detect_path_traversal(input: &str) -> bool {
    let lower = input.to_lowercase();

    if input.contains("..") {
        return true;
    }

    if input.contains('\0') {
        return true;
    }

    if input.starts_with('/') || input.starts_with('\\') {
        return true;
    }

    if input.contains('\\') {
        return true;
    }

    // URL-encoded traversal
    if lower.contains("%2e%2e") || lower.contains("%2f") || lower.contains("%5c") {
        return true;
    }

    if lower.contains("%252f") || lower.contains("%255c") {
        return true;
    }

    false
}

pub fn sanitize_filename(input: &str) -> String {
    let mut result: String = input
        .chars()
        .filter(|c| *c != '\0')
        .map(|c| if c == '/' || c == '\\' { '_' } else { c })
        .collect();

    // Remove .. sequences
    while result.contains("..") {
        result = result.replace("..", "");
    }

    // Remove leading dots
    while result.starts_with('.') {
        result.remove(0);
    }

    // Truncate to 255 characters
    if result.len() > 255 {
        result.truncate(255);
    }

    if result.is_empty() {
        return "unnamed".to_string();
    }

    result
}

pub struct SandboxedFileAccess {
    base_dir: PathBuf,
}

impl SandboxedFileAccess {
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    pub fn get_path(&self, filename: &str) -> Result<PathBuf, String> {
        safe_path_join(
            self.base_dir.to_str().unwrap_or(""),
            filename,
        )
    }

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
