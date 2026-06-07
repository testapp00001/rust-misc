//! # Cross-Language Testing
//!
//! Testing FFI boundaries requires special care. This module covers strategies
//! for testing Rust code that interfaces with other languages.
//!
//! ## Strategies:
//!
//! - **Mock FFI**: Test without actual foreign runtime
//! - **Property-based testing**: Verify FFI contracts
//! - **Boundary fuzzing**: Test edge cases at FFI boundaries
//! - **Integration tests**: Full end-to-end tests with foreign runtime


/// FFI boundary validator that checks data integrity across language boundaries.
pub struct FfiBoundaryValidator {
    checks: Vec<Box<dyn Fn(&[u8]) -> bool + Send + Sync>>,
}

impl FfiBoundaryValidator {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check<F>(&mut self, check: F)
    where
        F: Fn(&[u8]) -> bool + Send + Sync + 'static,
    {
        self.checks.push(Box::new(check));
    }

    /// Validate data passed across an FFI boundary.
    pub fn validate(&self, data: &[u8]) -> Result<(), FfiValidationError> {
        for (i, check) in self.checks.iter().enumerate() {
            if !check(data) {
                return Err(FfiValidationError {
                    check_index: i,
                    message: format!("Check {} failed", i),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FfiValidationError {
    pub check_index: usize,
    pub message: String,
}

impl std::fmt::Display for FfiValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FFI validation error: {}", self.message)
    }
}

impl std::error::Error for FfiValidationError {}

/// Property-based testing for FFI functions.
pub struct FfiPropertyTest {
    name: String,
    properties: Vec<Box<dyn Fn() -> bool + Send + Sync>>,
}

impl FfiPropertyTest {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            properties: Vec::new(),
        }
    }

    pub fn add_property<F>(&mut self, name: &str, property: F)
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.properties.push(Box::new(property));
    }

    pub fn run(&self) -> Vec<FfiTestResult> {
        self.properties
            .iter()
            .enumerate()
            .map(|(i, prop)| FfiTestResult {
                property_index: i,
                passed: prop(),
            })
            .collect()
    }

    pub fn all_passed(&self) -> bool {
        self.properties.iter().all(|p| p())
    }
}

#[derive(Debug)]
pub struct FfiTestResult {
    pub property_index: usize,
    pub passed: bool,
}

/// FFI contract testing: verify that Rust and foreign implementations match.
pub trait FfiContract {
    type Input;
    type Output;

    fn rust_impl(&self, input: &Self::Input) -> Self::Output;
    fn foreign_impl(&self, input: &Self::Input) -> Self::Output;

    fn verify_contract(&self, input: &Self::Input) -> bool
    where
        Self::Output: PartialEq,
    {
        self.rust_impl(input) == self.foreign_impl(input)
    }
}

/// Mock FFI function for testing without the foreign runtime.
pub struct MockFfiFunction {
    name: String,
    implementation: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>,
    call_log: std::sync::Mutex<Vec<Vec<u8>>>,
}

impl MockFfiFunction {
    pub fn new<F>(name: &str, implementation: F) -> Self
    where
        F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            implementation: Box::new(implementation),
            call_log: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn call(&self, input: &[u8]) -> Vec<u8> {
        self.call_log.lock().unwrap().push(input.to_vec());
        (self.implementation)(input)
    }

    pub fn call_count(&self) -> usize {
        self.call_log.lock().unwrap().len()
    }

    pub fn was_called_with(&self, input: &[u8]) -> bool {
        self.call_log
            .lock()
            .unwrap()
            .iter()
            .any(|logged| logged == input)
    }

    pub fn clear_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
}

/// FFI test harness that runs a suite of tests against an FFI interface.
pub struct FfiTestHarness {
    tests: Vec<Box<dyn FfiTestCase>>,
}

pub trait FfiTestCase: Send {
    fn name(&self) -> &str;
    fn run(&self) -> FfiTestOutcome;
}

#[derive(Debug)]
pub struct FfiTestOutcome {
    pub test_name: String,
    pub passed: bool,
    pub message: String,
    pub duration_ms: f64,
}

impl FfiTestHarness {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    pub fn add_test<T: FfiTestCase + 'static>(&mut self, test: T) {
        self.tests.push(Box::new(test));
    }

    pub fn run_all(&self) -> Vec<FfiTestOutcome> {
        self.tests.iter().map(|t| t.run()).collect()
    }

    pub fn summary(&self) -> FfiTestSummary {
        let outcomes = self.run_all();
        let passed = outcomes.iter().filter(|o| o.passed).count();
        let failed = outcomes.len() - passed;

        FfiTestSummary {
            total: outcomes.len(),
            passed,
            failed,
            outcomes,
        }
    }
}

#[derive(Debug)]
pub struct FfiTestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub outcomes: Vec<FfiTestOutcome>,
}

impl FfiTestSummary {
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    pub fn report(&self) -> String {
        let mut output = format!(
            "FFI Test Results: {} passed, {} failed, {} total\n\n",
            self.passed, self.failed, self.total
        );
        for outcome in &self.outcomes {
            let status = if outcome.passed { "PASS" } else { "FAIL" };
            output.push_str(&format!(
                "  [{}] {} - {} ({:.1}ms)\n",
                status, outcome.test_name, outcome.message, outcome.duration_ms
            ));
        }
        output
    }
}

/// Trait for types that can be serialized to/from JSON for cross-language testing.
/// In production code you'd use serde; this trait demonstrates the pattern
/// using only the standard library.
pub trait JsonSerializable: PartialEq {
    fn to_json(&self) -> String;
    fn from_json(json: &str) -> Result<Self, String>
    where
        Self: Sized;
}

impl JsonSerializable for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
    fn from_json(json: &str) -> Result<Self, String> {
        json.trim().parse::<i32>().map_err(|e| e.to_string())
    }
}

impl JsonSerializable for String {
    fn to_json(&self) -> String {
        // Simple JSON string encoding
        let escaped = self.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    }
    fn from_json(json: &str) -> Result<Self, String> {
        let trimmed = json.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let inner = &trimmed[1..trimmed.len() - 1];
            Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
        } else {
            Err("Not a JSON string".into())
        }
    }
}

impl<T: JsonSerializable + Clone> JsonSerializable for Vec<T> {
    fn to_json(&self) -> String {
        let items: Vec<String> = self.iter().map(|v| v.to_json()).collect();
        format!("[{}]", items.join(","))
    }
    fn from_json(json: &str) -> Result<Self, String> {
        let trimmed = json.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err("Not a JSON array".into());
        }
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        inner
            .split(',')
            .map(|s| T::from_json(s.trim()))
            .collect()
    }
}

/// Cross-language data serializer for testing.
pub struct CrossLangSerializer;

impl CrossLangSerializer {
    /// Serialize to a format that can be tested across languages.
    pub fn serialize_json<T: JsonSerializable>(value: &T) -> Result<String, String> {
        Ok(value.to_json())
    }

    pub fn deserialize_json<T: JsonSerializable>(json: &str) -> Result<T, String> {
        T::from_json(json)
    }

    /// Verify round-trip serialization.
    pub fn verify_roundtrip<T: JsonSerializable>(value: &T) -> bool {
        match Self::serialize_json(value) {
            Ok(json) => match Self::deserialize_json::<T>(&json) {
                Ok(deserialized) => deserialized == *value,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_boundary_validator() {
        let mut validator = FfiBoundaryValidator::new();
        validator.add_check(|data| !data.is_empty());
        validator.add_check(|data| data.len() < 1024);

        assert!(validator.validate(b"hello").is_ok());
        assert!(validator.validate(b"").is_err());
    }

    #[test]
    fn test_ffi_boundary_validator_large_data() {
        let mut validator = FfiBoundaryValidator::new();
        validator.add_check(|data| data.len() < 10);

        assert!(validator.validate(b"short").is_ok());
        assert!(validator.validate(b"this is too long").is_err());
    }

    #[test]
    fn test_ffi_property_test() {
        let mut test = FfiPropertyTest::new("string_props");
        test.add_property("non-empty", || !"hello".is_empty());
        test.add_property("length", || "hello".len() == 5);

        assert!(test.all_passed());
    }

    #[test]
    fn test_ffi_property_test_failing() {
        let mut test = FfiPropertyTest::new("failing");
        test.add_property("always_pass", || true);
        test.add_property("always_fail", || false);

        assert!(!test.all_passed());
    }

    #[test]
    fn test_mock_ffi_function() {
        let mock = MockFfiFunction::new("double", |data| {
            data.iter().map(|b| b.wrapping_mul(2)).collect()
        });

        let result = mock.call(&[1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
        assert_eq!(mock.call_count(), 1);
        assert!(mock.was_called_with(&[1, 2, 3]));
    }

    #[test]
    fn test_mock_ffi_function_log() {
        let mock = MockFfiFunction::new("echo", |data| data.to_vec());

        mock.call(b"first");
        mock.call(b"second");

        assert_eq!(mock.call_count(), 2);
        mock.clear_log();
        assert_eq!(mock.call_count(), 0);
    }

    #[test]
    fn test_cross_lang_serializer_roundtrip() {
        assert!(CrossLangSerializer::verify_roundtrip(&42i32));
        assert!(CrossLangSerializer::verify_roundtrip(&"hello".to_string()));
        assert!(CrossLangSerializer::verify_roundtrip(&vec![1, 2, 3]));
    }

    #[test]
    fn test_cross_lang_serializer_json() {
        let json = CrossLangSerializer::serialize_json(&vec![1, 2, 3]).unwrap();
        assert_eq!(json, "[1,2,3]");

        let value: Vec<i32> = CrossLangSerializer::deserialize_json(&json).unwrap();
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_ffi_test_harness() {
        let mut harness = FfiTestHarness::new();
        // Add tests would go here in a real implementation

        let summary = harness.summary();
        assert!(summary.all_passed()); // No tests = all passed
        assert_eq!(summary.total, 0);
    }

    #[test]
    fn test_ffi_test_summary_report() {
        let summary = FfiTestSummary {
            total: 2,
            passed: 1,
            failed: 1,
            outcomes: vec![
                FfiTestOutcome {
                    test_name: "test1".into(),
                    passed: true,
                    message: "OK".into(),
                    duration_ms: 1.5,
                },
                FfiTestOutcome {
                    test_name: "test2".into(),
                    passed: false,
                    message: "Failed".into(),
                    duration_ms: 0.5,
                },
            ],
        };

        let report = summary.report();
        assert!(report.contains("PASS"));
        assert!(report.contains("FAIL"));
    }
}
