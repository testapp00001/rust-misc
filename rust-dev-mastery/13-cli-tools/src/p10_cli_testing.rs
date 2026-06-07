//! # CLI Testing
//!
//! Testing CLI applications requires verifying both the parsing layer and the
//! execution layer. This lesson covers testing strategies including unit testing
//! individual commands, integration testing with mock I/O, and snapshot testing
//! of CLI output.
//!
//! ## Key Concepts
//! - Unit testing commands with mock dependencies
//! - Integration testing the full CLI pipeline
//! - Testing stdout/stderr output
//! - Testing exit codes
//! - Snapshot testing patterns
//! - Testing error cases

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. Test Harness
// ---------------------------------------------------------------------------

/// Captures output from CLI commands for testing.
#[derive(Clone, Debug, Default)]
pub struct TestOutput {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub exit_code: i32,
}

impl TestOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stdout_text(&self) -> String {
        self.stdout.join("\n")
    }

    pub fn stderr_text(&self) -> String {
        self.stderr.join("\n")
    }

    pub fn all_output(&self) -> String {
        let mut all = self.stdout_text();
        if !self.stderr.is_empty() {
            all.push('\n');
            all.push_str(&self.stderr_text());
        }
        all
    }

    pub fn assert_success(&self) {
        assert_eq!(self.exit_code, 0, "expected success, got exit code {}. stderr: {}", self.exit_code, self.stderr_text());
    }

    pub fn assert_exit_code(&self, code: i32) {
        assert_eq!(self.exit_code, code, "expected exit code {code}, got {}", self.exit_code);
    }

    pub fn assert_stdout_contains(&self, text: &str) {
        let stdout = self.stdout_text();
        assert!(
            stdout.contains(text),
            "stdout does not contain '{text}'. actual: '{stdout}'"
        );
    }

    pub fn assert_stderr_contains(&self, text: &str) {
        let stderr = self.stderr_text();
        assert!(
            stderr.contains(text),
            "stderr does not contain '{text}'. actual: '{stderr}'"
        );
    }

    pub fn assert_stdout_empty(&self) {
        assert!(self.stdout.is_empty(), "expected empty stdout, got: '{}'", self.stdout_text());
    }

    pub fn assert_stderr_empty(&self) {
        assert!(self.stderr.is_empty(), "expected empty stderr, got: '{}'", self.stderr_text());
    }
}

// ---------------------------------------------------------------------------
// 2. Mock Command Runner
// ---------------------------------------------------------------------------

/// A trait for running CLI commands, enabling test mocking.
pub trait CommandRunner {
    fn run(&self, args: &[&str]) -> TestOutput;
}

/// A real command runner that would invoke the actual CLI.
pub struct RealCommandRunner {
    program: String,
}

impl RealCommandRunner {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl CommandRunner for RealCommandRunner {
    fn run(&self, args: &[&str]) -> TestOutput {
        let output = std::process::Command::new(&self.program)
            .args(args)
            .output()
            .expect("failed to execute command");

        TestOutput {
            stdout: vec![String::from_utf8_lossy(&output.stdout).to_string()],
            stderr: vec![String::from_utf8_lossy(&output.stderr).to_string()],
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

/// A mock command runner for testing.
pub struct MockCommandRunner {
    responses: HashMap<String, TestOutput>,
    default_response: TestOutput,
}

impl MockCommandRunner {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            default_response: TestOutput {
                exit_code: 1,
                stderr: vec!["unknown command".into()],
                ..Default::default()
            },
        }
    }

    pub fn with_response(mut self, args: &str, output: TestOutput) -> Self {
        self.responses.insert(args.into(), output);
        self
    }

    pub fn with_success(mut self, args: &str, stdout: &str) -> Self {
        self.responses.insert(
            args.into(),
            TestOutput {
                stdout: vec![stdout.into()],
                exit_code: 0,
                ..Default::default()
            },
        );
        self
    }

    pub fn with_error(mut self, args: &str, exit_code: i32, stderr: &str) -> Self {
        self.responses.insert(
            args.into(),
            TestOutput {
                exit_code,
                stderr: vec![stderr.into()],
                ..Default::default()
            },
        );
        self
    }
}

impl CommandRunner for MockCommandRunner {
    fn run(&self, args: &[&str]) -> TestOutput {
        let key = args.join(" ");
        self.responses
            .get(&key)
            .cloned()
            .unwrap_or_else(|| TestOutput {
                stdout: self.default_response.stdout.clone(),
                stderr: self.default_response.stderr.clone(),
                exit_code: self.default_response.exit_code,
            })
    }
}

// ---------------------------------------------------------------------------
// 3. Snapshot Testing
// ---------------------------------------------------------------------------

/// A simple snapshot testing framework for CLI output.
pub struct SnapshotStore {
    snapshots: HashMap<String, String>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
        }
    }

    /// Check if output matches a stored snapshot.
    /// Returns Ok(()) if matching, Err(diff) if not.
    pub fn check(&self, name: &str, actual: &str) -> Result<(), SnapshotDiff> {
        match self.snapshots.get(name) {
            Some(expected) => {
                if expected == actual {
                    Ok(())
                } else {
                    Err(SnapshotDiff {
                        name: name.into(),
                        expected: expected.clone(),
                        actual: actual.to_string(),
                    })
                }
            }
            None => Err(SnapshotDiff {
                name: name.into(),
                expected: String::new(),
                actual: actual.to_string(),
            }),
        }
    }

    /// Store or update a snapshot.
    pub fn update(&mut self, name: &str, value: &str) {
        self.snapshots.insert(name.into(), value.into());
    }

    /// Assert that output matches the snapshot.
    pub fn assert_snapshot(&self, name: &str, actual: &str) {
        if let Err(diff) = self.check(name, actual) {
            panic!(
                "snapshot '{}' mismatch:\n--- expected\n+++ actual\n{}",
                diff.name,
                diff.unified_diff()
            );
        }
    }
}

#[derive(Debug)]
pub struct SnapshotDiff {
    pub name: String,
    pub expected: String,
    pub actual: String,
}

impl SnapshotDiff {
    pub fn unified_diff(&self) -> String {
        let expected_lines: Vec<&str> = self.expected.lines().collect();
        let actual_lines: Vec<&str> = self.actual.lines().collect();

        let mut diff = String::new();
        let max_lines = expected_lines.len().max(actual_lines.len());

        for i in 0..max_lines {
            let exp = expected_lines.get(i).unwrap_or(&"<missing>");
            let act = actual_lines.get(i).unwrap_or(&"<missing>");

            if exp != act {
                diff.push_str(&format!("-{exp}\n+{act}\n"));
            }
        }

        diff
    }
}

impl fmt::Display for SnapshotDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "snapshot '{}' mismatch:\n{}",
            self.name,
            self.unified_diff()
        )
    }
}

// ---------------------------------------------------------------------------
// 4. CLI Test Builder
// ---------------------------------------------------------------------------

/// A fluent builder for CLI integration tests.
pub struct CliTest {
    runner: Box<dyn CommandRunner>,
    args: Vec<String>,
}

impl CliTest {
    pub fn new(runner: impl CommandRunner + 'static) -> Self {
        Self {
            runner: Box::new(runner),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    pub fn run(&self) -> TestOutput {
        let arg_refs: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        self.runner.run(&arg_refs)
    }

    /// Run and assert success.
    pub fn run_success(&self) -> TestOutput {
        let output = self.run();
        output.assert_success();
        output
    }

    /// Run and assert failure.
    pub fn run_failure(&self) -> TestOutput {
        let output = self.run();
        assert_ne!(output.exit_code, 0, "expected failure but got success");
        output
    }
}

// ---------------------------------------------------------------------------
// 5. Predicate Testing
// ---------------------------------------------------------------------------

/// Composable predicates for testing CLI output.
pub struct OutputPredicate {
    description: String,
    check: Box<dyn Fn(&TestOutput) -> bool>,
}

impl OutputPredicate {
    pub fn new(
        description: impl Into<String>,
        check: impl Fn(&TestOutput) -> bool + 'static,
    ) -> Self {
        Self {
            description: description.into(),
            check: Box::new(check),
        }
    }

    pub fn check(&self, output: &TestOutput) -> bool {
        (self.check)(output)
    }

    /// Assert the predicate holds.
    pub fn assert(&self, output: &TestOutput) {
        assert!(
            self.check(output),
            "predicate '{}' failed",
            self.description
        );
    }
}

/// Common predicates.
pub mod predicates {
    use super::*;

    pub fn success() -> OutputPredicate {
        OutputPredicate::new("exit code is 0", |o| o.exit_code == 0)
    }

    pub fn failure() -> OutputPredicate {
        OutputPredicate::new("exit code is not 0", |o| o.exit_code != 0)
    }

    pub fn exit_code(code: i32) -> OutputPredicate {
        OutputPredicate::new(format!("exit code is {code}"), move |o| o.exit_code == code)
    }

    pub fn stdout_contains(text: &str) -> OutputPredicate {
        let text = text.to_string();
        OutputPredicate::new(
            format!("stdout contains '{text}'"),
            move |o| o.stdout_text().contains(&text),
        )
    }

    pub fn stderr_contains(text: &str) -> OutputPredicate {
        let text = text.to_string();
        OutputPredicate::new(
            format!("stderr contains '{text}'"),
            move |o| o.stderr_text().contains(&text),
        )
    }

    pub fn stdout_empty() -> OutputPredicate {
        OutputPredicate::new("stdout is empty", |o| o.stdout.is_empty())
    }

    pub fn stderr_empty() -> OutputPredicate {
        OutputPredicate::new("stderr is empty", |o| o.stderr.is_empty())
    }
}

// ---------------------------------------------------------------------------
// 6. Test Fixtures
// ---------------------------------------------------------------------------

/// Creates a standard set of mock responses for testing.
pub fn create_test_runner() -> MockCommandRunner {
    MockCommandRunner::new()
        .with_success("version", "mycli 1.0.0")
        .with_success("list", "file1.txt\nfile2.txt\nfile3.txt")
        .with_success("list --all", ".hidden\nfile1.txt\nfile2.txt\nfile3.txt")
        .with_error("list /nonexistent", 3, "error: path not found: /nonexistent")
        .with_success("cat test.txt", "hello world")
        .with_error("cat missing.txt", 3, "error: file not found: missing.txt")
        .with_success("config get editor", "vim")
        .with_error("config get missing_key", 1, "error: key not found")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_stdout_contains() {
        let output = TestOutput {
            stdout: vec!["hello world".into()],
            ..Default::default()
        };
        output.assert_stdout_contains("hello");
        output.assert_stdout_contains("world");
    }

    #[test]
    fn test_output_stdout_empty() {
        let output = TestOutput::new();
        output.assert_stdout_empty();
    }

    #[test]
    fn test_output_stderr_contains() {
        let output = TestOutput {
            stderr: vec!["error: something failed".into()],
            ..Default::default()
        };
        output.assert_stderr_contains("something failed");
    }

    #[test]
    fn test_output_assert_success() {
        let output = TestOutput {
            exit_code: 0,
            ..Default::default()
        };
        output.assert_success();
    }

    #[test]
    fn test_output_assert_exit_code() {
        let output = TestOutput {
            exit_code: 3,
            ..Default::default()
        };
        output.assert_exit_code(3);
    }

    #[test]
    fn test_output_all_output() {
        let output = TestOutput {
            stdout: vec!["line1".into(), "line2".into()],
            stderr: vec!["err1".into()],
            exit_code: 0,
        };
        let all = output.all_output();
        assert!(all.contains("line1"));
        assert!(all.contains("line2"));
        assert!(all.contains("err1"));
    }

    #[test]
    fn test_mock_runner_success() {
        let runner = MockCommandRunner::new().with_success("test", "output");
        let output = runner.run(&["test"]);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout_text(), "output");
    }

    #[test]
    fn test_mock_runner_error() {
        let runner = MockCommandRunner::new().with_error("fail", 1, "oops");
        let output = runner.run(&["fail"]);
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stderr_text(), "oops");
    }

    #[test]
    fn test_mock_runner_unknown_command() {
        let runner = MockCommandRunner::new();
        let output = runner.run(&["unknown"]);
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_snapshot_match() {
        let mut store = SnapshotStore::new();
        store.update("test-snap", "expected output");
        store.assert_snapshot("test-snap", "expected output");
    }

    #[test]
    #[should_panic(expected = "snapshot")]
    fn test_snapshot_mismatch() {
        let mut store = SnapshotStore::new();
        store.update("test-snap", "expected");
        store.assert_snapshot("test-snap", "actual");
    }

    #[test]
    fn test_snapshot_diff_format() {
        let diff = SnapshotDiff {
            name: "test".into(),
            expected: "line1\nline2\n".into(),
            actual: "line1\nline3\n".into(),
        };
        let formatted = diff.unified_diff();
        assert!(formatted.contains("-line2"));
        assert!(formatted.contains("+line3"));
    }

    #[test]
    fn test_cli_test_builder() {
        let runner = create_test_runner();
        let output = CliTest::new(runner).arg("version").run_success();
        output.assert_stdout_contains("1.0.0");
    }

    #[test]
    fn test_cli_test_builder_args() {
        let runner = create_test_runner();
        let output = CliTest::new(runner).args(&["list"]).run_success();
        output.assert_stdout_contains("file1.txt");
    }

    #[test]
    fn test_cli_test_builder_failure() {
        let runner = create_test_runner();
        let output = CliTest::new(runner)
            .arg("list")
            .arg("/nonexistent")
            .run_failure();
        output.assert_exit_code(3);
    }

    #[test]
    fn test_predicates_success() {
        let output = TestOutput {
            exit_code: 0,
            ..Default::default()
        };
        predicates::success().assert(&output);
        assert!(!predicates::failure().check(&output));
    }

    #[test]
    fn test_predicates_failure() {
        let output = TestOutput {
            exit_code: 1,
            ..Default::default()
        };
        predicates::failure().assert(&output);
        assert!(!predicates::success().check(&output));
    }

    #[test]
    fn test_predicates_exit_code() {
        let output = TestOutput {
            exit_code: 42,
            ..Default::default()
        };
        predicates::exit_code(42).assert(&output);
        assert!(!predicates::exit_code(0).check(&output));
    }

    #[test]
    fn test_predicates_stdout_contains() {
        let output = TestOutput {
            stdout: vec!["hello world".into()],
            ..Default::default()
        };
        predicates::stdout_contains("hello").assert(&output);
        assert!(!predicates::stdout_contains("goodbye").check(&output));
    }

    #[test]
    fn test_predicates_stderr_contains() {
        let output = TestOutput {
            stderr: vec!["error: bad input".into()],
            ..Default::default()
        };
        predicates::stderr_contains("bad input").assert(&output);
    }

    #[test]
    fn test_predicates_stdout_empty() {
        let empty = TestOutput::new();
        predicates::stdout_empty().assert(&empty);

        let non_empty = TestOutput {
            stdout: vec!["something".into()],
            ..Default::default()
        };
        assert!(!predicates::stdout_empty().check(&non_empty));
    }

    #[test]
    fn test_create_test_runner() {
        let runner = create_test_runner();

        // Version
        let output = runner.run(&["version"]);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout_text(), "mycli 1.0.0");

        // List
        let output = runner.run(&["list"]);
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout_text().contains("file1.txt"));

        // Cat
        let output = runner.run(&["cat", "test.txt"]);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout_text(), "hello world");

        // Error case
        let output = runner.run(&["cat", "missing.txt"]);
        assert_eq!(output.exit_code, 3);
    }

    #[test]
    fn test_snapshot_store_update_and_check() {
        let mut store = SnapshotStore::new();
        store.update("greeting", "hello world");

        assert!(store.check("greeting", "hello world").is_ok());
        assert!(store.check("greeting", "goodbye").is_err());
        assert!(store.check("missing", "anything").is_err());
    }

    #[test]
    fn test_snapshot_store_update_overwrites() {
        let mut store = SnapshotStore::new();
        store.update("key", "old");
        store.update("key", "new");

        assert!(store.check("key", "new").is_ok());
        assert!(store.check("key", "old").is_err());
    }

    #[test]
    fn test_output_predicate_display() {
        let pred = predicates::success();
        assert!(!pred.description.is_empty());
    }

    #[test]
    fn test_real_command_runner_echo() {
        let runner = RealCommandRunner::new("echo");
        let output = runner.run(&["hello"]);
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout_text().contains("hello"));
    }
}
