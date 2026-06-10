//! # Lesson 07: Security Test Harness (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;

/// Security properties that can be tested.
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityProperty {
    Determinism,
    EmptyInput,
    MaxInput,
    NoPanic,
    FixedOutputSize,
    NonDegeneracy,
}

/// Result of a single security test.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub property: SecurityProperty,
    pub passed: bool,
    pub details: String,
    pub input: Vec<u8>,
}

/// A security test harness for hash functions.
pub struct HashTestHarness {
    hash_fn: fn(&[u8]) -> Vec<u8>,
    expected_size: usize,
    results: Vec<TestResult>,
}

impl HashTestHarness {
    pub fn new(hash_fn: fn(&[u8]) -> Vec<u8>, expected_size: usize) -> Self {
        Self {
            hash_fn,
            expected_size,
            results: Vec::new(),
        }
    }

    pub fn check_determinism(&mut self, input: &[u8]) {
        let h1 = (self.hash_fn)(input);
        let h2 = (self.hash_fn)(input);
        self.results.push(TestResult {
            property: SecurityProperty::Determinism,
            passed: h1 == h2,
            details: if h1 == h2 {
                "Output is deterministic".to_string()
            } else {
                format!("Outputs differ: {:?} vs {:?}", &h1[..4], &h2[..4])
            },
            input: input.to_vec(),
        });
    }

    pub fn check_empty_input(&mut self) {
        let hash_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.hash_fn)(b"")
        }));
        match hash_result {
            Ok(output) => {
                let size_ok = output.len() == self.expected_size;
                self.results.push(TestResult {
                    property: SecurityProperty::EmptyInput,
                    passed: size_ok,
                    details: if size_ok {
                        format!("Empty input handled correctly, output {} bytes", output.len())
                    } else {
                        format!(
                            "Output size {} != expected {}",
                            output.len(),
                            self.expected_size
                        )
                    },
                    input: b"".to_vec(),
                });
            }
            Err(_) => {
                self.results.push(TestResult {
                    property: SecurityProperty::EmptyInput,
                    passed: false,
                    details: "PANICKED on empty input".to_string(),
                    input: b"".to_vec(),
                });
            }
        }
    }

    pub fn check_output_size(&mut self, input: &[u8]) {
        let output = (self.hash_fn)(input);
        let size_ok = output.len() == self.expected_size;
        self.results.push(TestResult {
            property: SecurityProperty::FixedOutputSize,
            passed: size_ok,
            details: if size_ok {
                format!("Output size correct: {} bytes", output.len())
            } else {
                format!(
                    "Output size {} != expected {}",
                    output.len(),
                    self.expected_size
                )
            },
            input: input.to_vec(),
        });
    }

    pub fn check_no_panic(&mut self, input: &[u8]) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.hash_fn)(input)
        }));
        self.results.push(TestResult {
            property: SecurityProperty::NoPanic,
            passed: result.is_ok(),
            details: if result.is_ok() {
                "No panic".to_string()
            } else {
                "PANICKED".to_string()
            },
            input: input.to_vec(),
        });
    }

    pub fn run_all_checks(&mut self, input: &[u8]) {
        self.check_determinism(input);
        self.check_output_size(input);
        self.check_no_panic(input);
    }

    pub fn run_corpus(&mut self) {
        self.check_empty_input();
        self.run_all_checks(&[0u8]);
        self.run_all_checks(&[0xFFu8]);
        self.run_all_checks(&[0u8; 1000]);
        self.run_all_checks(&[0xFFu8; 1000]);
        self.run_all_checks(b"hello world");
        self.run_all_checks(&[0x41; 64]);
    }

    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    pub fn summary(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let failed_details: Vec<String> = self
            .results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| format!("  {:?}: {}", r.property, r.details))
            .collect();

        if failed == 0 {
            format!("{}/{} tests passed", passed, total)
        } else {
            format!(
                "{}/{} tests passed, {} FAILED:\n{}",
                passed,
                total,
                failed,
                failed_details.join("\n")
            )
        }
    }
}

/// A security test harness for encoding functions.
pub struct EncodingTestHarness {
    encode_fn: fn(&[u8]) -> String,
    decode_fn: fn(&str) -> Result<Vec<u8>, String>,
    results: Vec<TestResult>,
}

impl EncodingTestHarness {
    pub fn new(
        encode_fn: fn(&[u8]) -> String,
        decode_fn: fn(&str) -> Result<Vec<u8>, String>,
    ) -> Self {
        Self {
            encode_fn,
            decode_fn,
            results: Vec::new(),
        }
    }

    pub fn check_roundtrip(&mut self, input: &[u8]) {
        let encoded = (self.encode_fn)(input);
        match (self.decode_fn)(&encoded) {
            Ok(decoded) => {
                self.results.push(TestResult {
                    property: SecurityProperty::Determinism,
                    passed: decoded == input,
                    details: if decoded == input {
                        "Round-trip successful".to_string()
                    } else {
                        format!("Round-trip failed: input len={}, output len={}", input.len(), decoded.len())
                    },
                    input: input.to_vec(),
                });
            }
            Err(e) => {
                self.results.push(TestResult {
                    property: SecurityProperty::Determinism,
                    passed: false,
                    details: format!("Decode failed: {}", e),
                    input: input.to_vec(),
                });
            }
        }
    }

    pub fn check_encode_no_panic(&mut self, input: &[u8]) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.encode_fn)(input)
        }));
        self.results.push(TestResult {
            property: SecurityProperty::NoPanic,
            passed: result.is_ok(),
            details: if result.is_ok() {
                "Encode no panic".to_string()
            } else {
                "PANICKED during encode".to_string()
            },
            input: input.to_vec(),
        });
    }

    pub fn check_decode_robustness(&mut self, input: &str) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.decode_fn)(input)
        }));
        self.results.push(TestResult {
            property: SecurityProperty::NoPanic,
            passed: result.is_ok(),
            details: if result.is_ok() {
                "Decode returned Result (no panic)".to_string()
            } else {
                "PANICKED during decode".to_string()
            },
            input: input.as_bytes().to_vec(),
        });
    }

    pub fn run_all_checks(&mut self, input: &[u8]) {
        self.check_encode_no_panic(input);
        self.check_roundtrip(input);
    }

    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }
}

/// Generic property checker with panic catching.
pub fn check_property<F: FnOnce() -> Result<(), String> + std::panic::UnwindSafe>(
    property: SecurityProperty,
    input: Vec<u8>,
    test_fn: F,
) -> TestResult {
    let result = std::panic::catch_unwind(test_fn);
    match result {
        Ok(Ok(())) => TestResult {
            property,
            passed: true,
            details: "Property holds".to_string(),
            input,
        },
        Ok(Err(msg)) => TestResult {
            property,
            passed: false,
            details: msg,
            input,
        },
        Err(_) => TestResult {
            property,
            passed: false,
            details: "PANICKED".to_string(),
            input,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_sha256(data: &[u8]) -> Vec<u8> {
        ring::digest::digest(&ring::digest::SHA256, data).as_ref().to_vec()
    }

    fn dummy_base64_encode(data: &[u8]) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
    }

    fn dummy_base64_decode(s: &str) -> Result<Vec<u8>, String> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
            .map_err(|e| e.to_string())
    }

    // --- HashTestHarness ---

    #[test]
    fn test_hash_harness_determinism() {
        let mut harness = HashTestHarness::new(dummy_sha256, 32);
        harness.check_determinism(b"test");
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_hash_harness_empty_input() {
        let mut harness = HashTestHarness::new(dummy_sha256, 32);
        harness.check_empty_input();
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_hash_harness_output_size() {
        let mut harness = HashTestHarness::new(dummy_sha256, 32);
        harness.check_output_size(b"hello");
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_hash_harness_no_panic() {
        let mut harness = HashTestHarness::new(dummy_sha256, 32);
        harness.check_no_panic(b"test data");
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_hash_harness_corpus() {
        let mut harness = HashTestHarness::new(dummy_sha256, 32);
        harness.run_corpus();
        assert!(harness.all_passed(), "All corpus tests should pass: {}", harness.summary());
    }

    #[test]
    fn test_hash_harness_wrong_size_detected() {
        fn bad_hash(_data: &[u8]) -> Vec<u8> {
            vec![0u8; 16]
        }
        let mut harness = HashTestHarness::new(bad_hash, 32);
        harness.check_output_size(b"hello");
        assert!(harness.results.iter().any(|r| !r.passed));
    }

    // --- EncodingTestHarness ---

    #[test]
    fn test_encoding_harness_roundtrip() {
        let mut harness = EncodingTestHarness::new(dummy_base64_encode, dummy_base64_decode);
        harness.check_roundtrip(b"hello world");
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_encoding_harness_encode_no_panic() {
        let mut harness = EncodingTestHarness::new(dummy_base64_encode, dummy_base64_decode);
        harness.check_encode_no_panic(b"test");
        assert!(harness.results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_encoding_harness_decode_robustness() {
        let mut harness = EncodingTestHarness::new(dummy_base64_encode, dummy_base64_decode);
        harness.check_decode_robustness("not-valid-base64!!!");
        assert!(!harness.results.is_empty());
    }

    // --- check_property ---

    #[test]
    fn test_check_property_passing() {
        let result = check_property(
            SecurityProperty::Determinism,
            b"test".to_vec(),
            || Ok(()),
        );
        assert!(result.passed);
    }

    #[test]
    fn test_check_property_failing() {
        let result = check_property(
            SecurityProperty::Determinism,
            b"test".to_vec(),
            || Err("outputs differ".to_string()),
        );
        assert!(!result.passed);
        assert!(result.details.contains("outputs differ"));
    }

    proptest::proptest! {
        #[test]
        fn test_hash_harness_never_panics_on_arbitrary_input(
            data in prop::collection::vec(prop::num::u8::ANY, 0..500)
        ) {
            let mut harness = HashTestHarness::new(dummy_sha256, 32);
            harness.check_no_panic(&data);
            assert!(harness.results.iter().all(|r| r.passed));
        }
    }
}
