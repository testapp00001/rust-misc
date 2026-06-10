//! # Lesson 08: Security Regression Tests
//!
//! ## What are Regression Tests?
//!
//! Regression tests capture previously-discovered bugs as permanent test cases.
//! When a fuzzer finds a crash, the crashing input becomes a regression test that
//! runs on every build to ensure the bug never reappears.
//!
//! ## The Regression Test Workflow
//!
//! ```
//! 1. Fuzzer finds crash → saves input to artifacts/
//! 2. Developer reproduces crash with saved input
//! 3. Developer fixes the bug
//! 4. Developer adds regression test with the crashing input
//! 5. CI runs regression test on every commit
//! 6. Bug can never reappear silently
//! ```
//!
//! ## Security Perspective
//!
//! ### Attack: Regression Exploitation
//! Attackers search for known CVEs that were "fixed" but regressed in a later release.
//! If a fix isn't protected by a regression test, a code refactor can reintroduce it.
//!
//! ### Defense: Every Bug Gets a Regression Test
//! - Capture the exact input that triggered the bug
//! - Add it to the test suite
//! - Run it in CI with `--test-threads=1` for determinism

use std::collections::BTreeMap;

/// A stored test case (input that triggered a bug).
#[derive(Debug, Clone)]
pub struct RegressionCase {
    /// Unique identifier for this regression.
    pub id: String,
    /// Human-readable description of the bug.
    pub description: String,
    /// The input that triggered the bug.
    pub input: Vec<u8>,
    /// The expected behavior after the fix.
    pub expected: ExpectedBehavior,
}

/// What the function should do with this input after the fix.
#[derive(Debug, Clone)]
pub enum ExpectedBehavior {
    /// Should return Ok with this output.
    ReturnsOk(Vec<u8>),
    /// Should return Err (not panic).
    ReturnsError,
    /// Should return a value of this exact length.
    OutputLength(usize),
    /// Should not panic (any output is acceptable).
    NoPanic,
}

/// A regression test suite that stores and replays known bug-triggering inputs.
pub struct RegressionSuite {
    cases: BTreeMap<String, RegressionCase>,
}

impl RegressionSuite {
    pub fn new() -> Self {
        todo!("Initialize empty regression suite")
    }

    /// Add a regression test case.
    pub fn add(&mut self, case: RegressionCase) {
        todo!("Add regression case")
    }

    /// Run a function against all stored regression cases.
    ///
    /// Returns a list of (case_id, passed, details) tuples.
    ///
    /// Hints:
    /// - For each case, call the function with case.input
    /// - Match on case.expected to determine pass/fail
    /// - Use catch_unwind to detect panics
    pub fn run_all<F>(&self, f: F) -> Vec<(String, bool, String)>
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        todo!("Run function against all regression cases")
    }

    /// Run and assert all pass. Panics with details if any fail.
    pub fn assert_all_pass<F>(&self, f: F)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        todo!("Assert all regression cases pass")
    }

    /// Return the number of stored cases.
    pub fn len(&self) -> usize {
        self.cases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

/// Helper to create a RegressionCase for a "should not panic" scenario.
pub fn no_panic_case(id: &str, description: &str, input: Vec<u8>) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::NoPanic,
    }
}

/// Helper to create a RegressionCase for a "should return error" scenario.
pub fn error_case(id: &str, description: &str, input: Vec<u8>) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::ReturnsError,
    }
}

/// Helper to create a RegressionCase for an "output length" scenario.
pub fn output_length_case(id: &str, description: &str, input: Vec<u8>, len: usize) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::OutputLength(len),
    }
}

/// Demonstrate: a buggy function and its regression tests.
///
/// This function has a known bug: it panics when input starts with 0xFF.
/// The fix: handle 0xFF gracefully.
///
/// Implement a fixed version that:
/// - Returns the input bytes reversed
/// - Never panics on any input
/// - Returns b"empty" for empty input
pub fn reverse_or_empty(data: &[u8]) -> Vec<u8> {
    todo!("Implement fixed reverse_or_empty function")
}

/// Serialize a regression suite to a JSON-compatible format.
///
/// Returns a Vec of (id, description, input_hex, expected_type) tuples.
///
/// Hints:
/// - Iterate over cases
/// - Convert input bytes to hex string
/// - Map expected behavior to a string tag
pub fn serialize_suite(suite: &RegressionSuite) -> Vec<(String, String, String, String)> {
    todo!("Serialize regression suite")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RegressionSuite ---

    #[test]
    fn test_suite_add_and_count() {
        let mut suite = RegressionSuite::new();
        assert_eq!(suite.len(), 0);

        suite.add(no_panic_case("r001", "empty input", b"".to_vec()));
        assert_eq!(suite.len(), 1);

        suite.add(no_panic_case("r002", "null byte", vec![0]));
        assert_eq!(suite.len(), 2);
    }

    #[test]
    fn test_suite_run_passing() {
        let mut suite = RegressionSuite::new();
        suite.add(no_panic_case("r001", "test", b"hello".to_vec()));

        // A function that never panics
        let results = suite.run_all(|data| Ok(data.to_vec()));
        assert_eq!(results.len(), 1);
        assert!(results[0].1, "Should pass");
    }

    #[test]
    fn test_suite_run_failing() {
        let mut suite = RegressionSuite::new();
        suite.add(error_case("r001", "should error", b"bad".to_vec()));

        // A function that returns Ok (should fail the test since we expect error)
        let results = suite.run_all(|data| Ok(data.to_vec()));
        assert_eq!(results.len(), 1);
        assert!(!results[0].1, "Should fail (expected error but got Ok)");
    }

    #[test]
    fn test_suite_output_length() {
        let mut suite = RegressionSuite::new();
        suite.add(output_length_case("r001", "SHA-256 output", b"test".to_vec(), 32));

        // A function that always returns 32 bytes
        let results = suite.run_all(|_| Ok(vec![0u8; 32]));
        assert!(results[0].1);
    }

    #[test]
    fn test_suite_output_length_fail() {
        let mut suite = RegressionSuite::new();
        suite.add(output_length_case("r001", "wrong size", b"test".to_vec(), 32));

        // Returns 16 bytes, expected 32
        let results = suite.run_all(|_| Ok(vec![0u8; 16]));
        assert!(!results[0].1);
    }

    // --- Helper functions ---

    #[test]
    fn test_no_panic_case_helper() {
        let case = no_panic_case("r001", "test", vec![1, 2, 3]);
        assert_eq!(case.id, "r001");
        assert!(matches!(case.expected, ExpectedBehavior::NoPanic));
    }

    #[test]
    fn test_error_case_helper() {
        let case = error_case("r002", "test", vec![0xFF]);
        assert!(matches!(case.expected, ExpectedBehavior::ReturnsError));
    }

    #[test]
    fn test_output_length_case_helper() {
        let case = output_length_case("r003", "test", vec![], 32);
        assert!(matches!(case.expected, ExpectedBehavior::OutputLength(32)));
    }

    // --- reverse_or_empty ---

    #[test]
    fn test_reverse_or_empty_empty() {
        assert_eq!(reverse_or_empty(b""), b"empty");
    }

    #[test]
    fn test_reverse_or_empty_basic() {
        assert_eq!(reverse_or_empty(b"abc"), b"cba");
    }

    #[test]
    fn test_reverse_or_empty_0xff() {
        // This was the known bug: input starting with 0xFF
        let result = reverse_or_empty(&[0xFF, 0x01, 0x02]);
        assert_eq!(result, vec![0x02, 0x01, 0xFF]);
    }

    proptest::proptest! {
        #[test]
        fn test_reverse_or_empty_never_panics(
            data in prop::collection::vec(prop::num::u8::ANY, 0..500)
        ) {
            let _ = reverse_or_empty(&data);
        }

        #[test]
        fn test_reverse_or_empty_correctness(
            data in prop::collection::vec(prop::num::u8::ANY, 1..500)
        ) {
            let result = reverse_or_empty(&data);
            let expected: Vec<u8> = data.iter().rev().cloned().collect();
            prop_assert_eq!(result, expected);
        }
    }

    // --- serialize_suite ---

    #[test]
    fn test_serialize_suite() {
        let mut suite = RegressionSuite::new();
        suite.add(no_panic_case("r001", "test", vec![0x41, 0x42]));
        let serialized = serialize_suite(&suite);
        assert_eq!(serialized.len(), 1);
        assert_eq!(serialized[0].0, "r001");
        assert!(serialized[0].2.contains("41"));
    }
}
