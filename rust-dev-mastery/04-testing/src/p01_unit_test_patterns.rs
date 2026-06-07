//! # Lesson 1: Unit Test Patterns
//!
//! Unit tests verify individual functions and methods in isolation.
//! This lesson covers test organization, assertion macros, custom assert
//! helpers, and testing patterns for production code.

// ---------------------------------------------------------------------------
// Code under test
// ---------------------------------------------------------------------------

/// A calculator with basic operations.
pub struct Calculator {
    history: Vec<Calculation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Calculation {
    pub operation: String,
    pub result: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.history.push(Calculation {
            operation: format!("{} + {}", a, b),
            result,
        });
        result
    }

    pub fn subtract(&mut self, a: f64, b: f64) -> f64 {
        let result = a - b;
        self.history.push(Calculation {
            operation: format!("{} - {}", a, b),
            result,
        });
        result
    }

    pub fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.history.push(Calculation {
            operation: format!("{} * {}", a, b),
            result,
        });
        result
    }

    pub fn divide(&mut self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            return Err("division by zero".to_string());
        }
        let result = a / b;
        self.history.push(Calculation {
            operation: format!("{} / {}", a, b),
            result,
        });
        Ok(result)
    }

    pub fn history(&self) -> &[Calculation] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Custom assertion macros
// ---------------------------------------------------------------------------

/// Assert that two f64 values are approximately equal.
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        assert_approx_eq!($left, $right, 1e-10);
    };
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left: f64 = $left;
        let right: f64 = $right;
        let eps = $epsilon;
        assert!(
            (left - right).abs() < eps,
            "assertion failed: {:?} ≈ {:?} (epsilon={})",
            left,
            right,
            eps
        );
    };
}

/// Assert that a collection is sorted.
macro_rules! assert_sorted {
    ($slice:expr) => {
        let slice = $slice;
        for i in 1..slice.len() {
            assert!(
                slice[i - 1] <= slice[i],
                "assertion failed: slice is not sorted at index {} ({} > {})",
                i,
                slice[i - 1],
                slice[i]
            );
        }
    };
}

/// Assert that a Result is an Err containing a specific message.
macro_rules! assert_err_msg {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => panic!("expected Err, got Ok({:?})", val),
            Err(e) => {
                let err_str = format!("{}", e);
                assert!(
                    err_str.contains($msg),
                    "error '{}' does not contain '{}'",
                    err_str,
                    $msg
                );
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Parameterized test helper
// ---------------------------------------------------------------------------

/// Helper for running the same test logic with multiple inputs.
pub struct TestCase<I, E> {
    pub name: String,
    pub input: I,
    pub expected: E,
}

pub fn run_test_cases<I: std::fmt::Debug, E: PartialEq + std::fmt::Debug>(
    cases: Vec<TestCase<I, E>>,
    mut f: impl FnMut(&I) -> E,
) {
    for case in &cases {
        let result = f(&case.input);
        assert_eq!(
            result, case.expected,
            "test case '{}': expected {:?}, got {:?}",
            case.name, case.expected, result
        );
    }
}

// ---------------------------------------------------------------------------
// Table-driven tests
// ---------------------------------------------------------------------------

/// Parse a boolean from various string representations.
pub fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => Err(format!("invalid boolean: '{}'", s)),
    }
}

/// Validate an email address (simplified).
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("email cannot be empty".to_string());
    }
    if !email.contains('@') {
        return Err("email must contain @".to_string());
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err("email must have exactly one @".to_string());
    }
    if parts[0].is_empty() {
        return Err("email local part cannot be empty".to_string());
    }
    if parts[1].is_empty() || !parts[1].contains('.') {
        return Err("email domain must be valid".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Basic unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_calculator_add() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(2.0, 3.0), 5.0);
    }

    #[test]
    fn test_calculator_subtract() {
        let mut calc = Calculator::new();
        assert_eq!(calc.subtract(10.0, 3.0), 7.0);
    }

    #[test]
    fn test_calculator_multiply() {
        let mut calc = Calculator::new();
        assert_eq!(calc.multiply(4.0, 5.0), 20.0);
    }

    #[test]
    fn test_calculator_divide() {
        let mut calc = Calculator::new();
        assert_eq!(calc.divide(10.0, 2.0).unwrap(), 5.0);
    }

    #[test]
    fn test_calculator_divide_by_zero() {
        let mut calc = Calculator::new();
        assert_err_msg!(calc.divide(10.0, 0.0), "division by zero");
    }

    // -----------------------------------------------------------------------
    // History tracking
    // -----------------------------------------------------------------------

    #[test]
    fn test_calculator_history() {
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0);
        calc.multiply(3.0, 4.0);

        let history = calc.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].result, 3.0);
        assert_eq!(history[1].result, 12.0);
    }

    #[test]
    fn test_calculator_clear_history() {
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0);
        calc.clear_history();
        assert!(calc.history().is_empty());
    }

    // -----------------------------------------------------------------------
    // Custom assertion macros
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq!(1.0 / 3.0 * 3.0, 1.0);
    }

    #[test]
    fn test_assert_sorted() {
        assert_sorted!(&[1, 2, 3, 4, 5]);
        assert_sorted!(&[1, 1, 2, 2, 3]);
    }

    #[test]
    fn test_assert_err_msg() {
        let mut calc = Calculator::new();
        assert_err_msg!(calc.divide(1.0, 0.0), "division by zero");
    }

    // -----------------------------------------------------------------------
    // Parameterized tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_bool_true_values() {
        run_test_cases(
            vec![
                TestCase { name: "true".into(), input: "true".into(), expected: Ok(true) },
                TestCase { name: "yes".into(), input: "yes".into(), expected: Ok(true) },
                TestCase { name: "1".into(), input: "1".into(), expected: Ok(true) },
                TestCase { name: "on".into(), input: "on".into(), expected: Ok(true) },
                TestCase { name: "TRUE".into(), input: "TRUE".into(), expected: Ok(true) },
            ],
            |input: &String| parse_bool(input),
        );
    }

    #[test]
    fn test_parse_bool_false_values() {
        run_test_cases(
            vec![
                TestCase { name: "false".into(), input: "false".into(), expected: Ok(false) },
                TestCase { name: "no".into(), input: "no".into(), expected: Ok(false) },
                TestCase { name: "0".into(), input: "0".into(), expected: Ok(false) },
                TestCase { name: "off".into(), input: "off".into(), expected: Ok(false) },
            ],
            |input: &String| parse_bool(input),
        );
    }

    #[test]
    fn test_parse_bool_invalid() {
        let result = parse_bool("maybe");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Table-driven email validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_email_valid() {
        let valid = vec![
            "user@example.com",
            "user.name@example.com",
            "user+tag@example.com",
            "a@b.co",
        ];
        for email in valid {
            assert!(
                validate_email(email).is_ok(),
                "expected '{}' to be valid",
                email
            );
        }
    }

    #[test]
    fn test_validate_email_invalid() {
        let cases = vec![
            ("", "cannot be empty"),
            ("no-at-sign", "must contain @"),
            ("@domain.com", "local part cannot be empty"),
            ("user@", "domain must be valid"),
            ("user@nodot", "domain must be valid"),
        ];
        for (email, expected_err) in cases {
            let result = validate_email(email);
            assert_err_msg!(result, expected_err);
        }
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_calculator_with_zero() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(0.0, 0.0), 0.0);
        assert_eq!(calc.multiply(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_calculator_negative_numbers() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(-5.0, 3.0), -2.0);
        assert_eq!(calc.subtract(-5.0, -3.0), -2.0);
    }

    #[test]
    fn test_calculator_floating_point() {
        let mut calc = Calculator::new();
        assert_approx_eq!(calc.add(0.1, 0.2), 0.3);
    }

    #[test]
    fn test_calculator_divide_negative() {
        let mut calc = Calculator::new();
        assert_eq!(calc.divide(-10.0, 2.0).unwrap(), -5.0);
    }

    #[test]
    fn test_calculator_default() {
        let calc = Calculator::default();
        assert!(calc.history().is_empty());
    }

    #[test]
    fn test_assert_sorted_empty() {
        let empty: &[i32] = &[];
        assert_sorted!(empty);
    }

    #[test]
    fn test_assert_sorted_single() {
        assert_sorted!(&[42]);
    }

    #[test]
    fn test_calculation_debug() {
        let calc = Calculation {
            operation: "1 + 2".into(),
            result: 3.0,
        };
        let debug = format!("{:?}", calc);
        assert!(debug.contains("1 + 2"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn test_calculation_clone() {
        let calc = Calculation {
            operation: "test".into(),
            result: 1.0,
        };
        let cloned = calc.clone();
        assert_eq!(calc, cloned);
    }
}
