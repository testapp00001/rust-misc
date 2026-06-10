//! # Lesson 08: Security Regression Tests (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;
use std::collections::BTreeMap;

/// A stored test case.
#[derive(Debug, Clone)]
pub struct RegressionCase {
    pub id: String,
    pub description: String,
    pub input: Vec<u8>,
    pub expected: ExpectedBehavior,
}

/// Expected behavior after fix.
#[derive(Debug, Clone)]
pub enum ExpectedBehavior {
    ReturnsOk(Vec<u8>),
    ReturnsError,
    OutputLength(usize),
    NoPanic,
}

/// A regression test suite.
pub struct RegressionSuite {
    cases: BTreeMap<String, RegressionCase>,
}

impl RegressionSuite {
    pub fn new() -> Self {
        Self {
            cases: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, case: RegressionCase) {
        self.cases.insert(case.id.clone(), case);
    }

    pub fn run_all<F>(&self, f: F) -> Vec<(String, bool, String)>
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        self.cases
            .values()
            .map(|case| {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    f(&case.input)
                }));
                let (passed, details) = match (&case.expected, result) {
                    (ExpectedBehavior::NoPanic, Err(_)) => {
                        (false, "PANICKED".to_string())
                    }
                    (ExpectedBehavior::NoPanic, Ok(_)) => {
                        (true, "No panic".to_string())
                    }
                    (ExpectedBehavior::ReturnsError, Ok(Ok(_))) => {
                        (false, "Expected error but got Ok".to_string())
                    }
                    (ExpectedBehavior::ReturnsError, Ok(Err(_))) => {
                        (true, "Returned error as expected".to_string())
                    }
                    (ExpectedBehavior::ReturnsError, Err(_)) => {
                        (false, "PANICKED instead of returning error".to_string())
                    }
                    (ExpectedBehavior::ReturnsOk(expected), Ok(Ok(output))) => {
                        if output == *expected {
                            (true, "Output matches expected".to_string())
                        } else {
                            (false, format!("Output mismatch: got {} bytes, expected {} bytes", output.len(), expected.len()))
                        }
                    }
                    (ExpectedBehavior::ReturnsOk(_), Ok(Err(e))) => {
                        (false, format!("Expected Ok but got Err: {}", e))
                    }
                    (ExpectedBehavior::ReturnsOk(_), Err(_)) => {
                        (false, "PANICKED".to_string())
                    }
                    (ExpectedBehavior::OutputLength(expected_len), Ok(Ok(output))) => {
                        if output.len() == *expected_len {
                            (true, format!("Output length correct: {}", output.len()))
                        } else {
                            (false, format!("Output length {} != expected {}", output.len(), expected_len))
                        }
                    }
                    (ExpectedBehavior::OutputLength(_), Ok(Err(e))) => {
                        (false, format!("Expected Ok but got Err: {}", e))
                    }
                    (ExpectedBehavior::OutputLength(_), Err(_)) => {
                        (false, "PANICKED".to_string())
                    }
                };
                (case.id.clone(), passed, details)
            })
            .collect()
    }

    pub fn assert_all_pass<F>(&self, f: F)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let results = self.run_all(f);
        let failures: Vec<String> = results
            .iter()
            .filter(|(_, passed, _)| !passed)
            .map(|(id, _, details)| format!("  {}: {}", id, details))
            .collect();

        if !failures.is_empty() {
            panic!(
                "Regression tests failed:\n{}",
                failures.join("\n")
            );
        }
    }

    pub fn len(&self) -> usize {
        self.cases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

/// Helper: "should not panic" regression case.
pub fn no_panic_case(id: &str, description: &str, input: Vec<u8>) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::NoPanic,
    }
}

/// Helper: "should return error" regression case.
pub fn error_case(id: &str, description: &str, input: Vec<u8>) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::ReturnsError,
    }
}

/// Helper: "output length" regression case.
pub fn output_length_case(id: &str, description: &str, input: Vec<u8>, len: usize) -> RegressionCase {
    RegressionCase {
        id: id.to_string(),
        description: description.to_string(),
        input,
        expected: ExpectedBehavior::OutputLength(len),
    }
}

/// Fixed version of reverse_or_empty.
pub fn reverse_or_empty(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        b"empty".to_vec()
    } else {
        data.iter().rev().cloned().collect()
    }
}

/// Serialize regression suite to tuples.
pub fn serialize_suite(suite: &RegressionSuite) -> Vec<(String, String, String, String)> {
    suite
        .cases
        .values()
        .map(|case| {
            let expected_type = match &case.expected {
                ExpectedBehavior::NoPanic => "NoPanic",
                ExpectedBehavior::ReturnsError => "ReturnsError",
                ExpectedBehavior::ReturnsOk(_) => "ReturnsOk",
                ExpectedBehavior::OutputLength(_) => "OutputLength",
            };
            (
                case.id.clone(),
                case.description.clone(),
                hex::encode(&case.input),
                expected_type.to_string(),
            )
        })
        .collect()
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

        let results = suite.run_all(|data| Ok(data.to_vec()));
        assert_eq!(results.len(), 1);
        assert!(results[0].1);
    }

    #[test]
    fn test_suite_run_failing() {
        let mut suite = RegressionSuite::new();
        suite.add(error_case("r001", "should error", b"bad".to_vec()));

        let results = suite.run_all(|data| Ok(data.to_vec()));
        assert_eq!(results.len(), 1);
        assert!(!results[0].1);
    }

    #[test]
    fn test_suite_output_length() {
        let mut suite = RegressionSuite::new();
        suite.add(output_length_case("r001", "SHA-256 output", b"test".to_vec(), 32));

        let results = suite.run_all(|_| Ok(vec![0u8; 32]));
        assert!(results[0].1);
    }

    #[test]
    fn test_suite_output_length_fail() {
        let mut suite = RegressionSuite::new();
        suite.add(output_length_case("r001", "wrong size", b"test".to_vec(), 32));

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
